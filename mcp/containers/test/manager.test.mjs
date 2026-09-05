import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { Store } from '../src/store.mjs';
import { Manager } from '../src/manager.mjs';
import { templateSchema, nameSchema } from '../src/catalog.mjs';

async function fixture(t) {
  const dir = await mkdtemp(join(tmpdir(), 'development-manager-'));
  const store = new Store(join(dir, 'state'));
  const docker = {
    async localIdentity() { return 'unix:///test.sock'; },
    async list() { return []; },
    async inspect() { return undefined; },
  };
  const manager = new Manager(store, docker, join(dir, 'volumes'));
  t.after(async () => { store.close(); await rm(dir, { recursive: true, force: true }); });
  return { store, manager, docker };
}

test('aliases cannot escape the volume root or become command options', () => {
  for (const name of ['../data', '/tmp/x', '-v', 'project; rm', 'a/b', 'UPPER']) assert.equal(nameSchema.safeParse(name).success, false);
});

test('custom catalog persists additions, rejects duplicate ports and protects in-use entries', async t => {
  const { manager, store } = await fixture(t);
  const template = { id: 'mail', description: 'Mail testing', image: 'axllent/mailpit:v1.27', dataPath: '/data', ports: [{ name: 'smtp', container: 1025, host: 0 }] };
  await manager.addTemplate(template);
  assert.equal(store.get('templates', 'mail').kind, 'generic');
  assert.throws(() => templateSchema.parse({ ...template, ports: [template.ports[0], template.ports[0]] }));
  await assert.rejects(manager.addTemplate(template), /already exists/);
  store.put('services', 'mail', { alias: 'mail', template: store.get('templates', 'mail') });
  await assert.rejects(manager.removeTemplate('mail'), /in use/);
});

test('discovery excludes credentials and exposes unmanaged containers without adopting them', async t => {
  const { manager, store, docker } = await fixture(t);
  store.put('services', 'postgres', { alias: 'postgres', template: store.get('templates', 'postgres'), secrets: { password: 'do-not-leak' }, projects: {} });
  docker.list = async () => [{ id: 'unmanaged', name: 'external-db', image: 'postgres:18', state: 'running', managed: false }];
  const output = await manager.listServices();
  assert.equal(output.containers[0].managed, false);
  assert.equal(JSON.stringify(output).includes('do-not-leak'), false);
});

test('removing an unrelated container is rejected even when its name matches', async t => {
  const { manager, store, docker } = await fixture(t);
  store.put('services', 'postgres', { alias: 'postgres', template: store.get('templates', 'postgres'), secrets: {}, projects: {} });
  docker.inspect = async () => ({ Id: 'foreign', Config: { Labels: {} } });
  await assert.rejects(manager.removeService('postgres', 'postgres'), /not owned/);
});

test('container removal retains credentials, project records, and volume directory', async t => {
  const { manager, store } = await fixture(t);
  store.put('services', 'postgres', { alias: 'postgres', template: store.get('templates', 'postgres'), secrets: { password: 'keep' }, projects: { app: { database: 'app' } } });
  await manager.removeService('postgres', 'postgres');
  const saved = store.get('services', 'postgres');
  assert.equal(saved.secrets.password, 'keep');
  assert.equal(saved.projects.app.database, 'app');
  assert.equal(saved.removed, true);
});

test('recreation refuses symlinked credential directories and files without touching their targets', async t => {
  const { manager, store } = await fixture(t);
  const { writeFile, readFile, symlink, mkdir, rm } = await import('node:fs/promises');
  const volume = await manager.volume('postgres');
  const target = join(manager.volumesRoot, 'unrelated');
  await mkdir(target);
  await writeFile(join(target, 'password'), 'preserve');
  const service = { template: store.get('templates', 'postgres'), secrets: { password: 'new', environment: {} } };
  await symlink(target, join(volume, '.development'));
  await assert.rejects(manager.configuration(service, volume), /symbolic link/);
  assert.equal(await readFile(join(target, 'password'), 'utf8'), 'preserve');
  await rm(join(volume, '.development'));
  await mkdir(join(volume, '.development'));
  await symlink(join(target, 'password'), join(volume, '.development', 'password'));
  await assert.rejects(manager.configuration(service, volume));
  assert.equal(await readFile(join(target, 'password'), 'utf8'), 'preserve');
});

test('retained service definitions reserve their catalog IDs', async t => {
  const { manager, store } = await fixture(t);
  store.put('services', 'redis', { alias: 'redis', removed: true, template: store.get('templates', 'redis'), secrets: {}, projects: {} });
  await manager.removeTemplate('redis');
  await assert.rejects(manager.addTemplate({ id: 'redis', description: 'Another service', image: 'redis:8-alpine', dataPath: '/data', ports: [{ name: 'redis', container: 6379 }] }), /retained/);
});

test('generic connection exposes only configured credentials', async t => {
  const { manager, store, docker } = await fixture(t);
  store.put('services', 'generic', { alias: 'generic', template: { id: 'generic', kind: 'generic', ports: [{ name: 'api', container: 8080, host: 8080 }], environment: { USER: 'actual' } }, secrets: { password: 'unused', environment: { PASSWORD: 'actual-secret' } }, projects: {} });
  docker.inspect = async () => ({ Id: 'owned', State: { Running: true }, Config: { Labels: { 'local.development-mcp.owner': manager.owner, 'local.development-mcp.alias': 'generic' } }, NetworkSettings: { Ports: {} } });
  const result = await manager.connection('generic');
  assert.equal(result.username, undefined);
  assert.equal(result.password, undefined);
  assert.deepEqual(result.environment, { USER: 'actual', PASSWORD: 'actual-secret' });
});
