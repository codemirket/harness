import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm, stat, mkdir } from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { randomBytes } from 'node:crypto';
import { createConnection } from 'node:net';
import { Docker, OWNER_LABEL } from '../src/docker.mjs';
import { connect, call } from './support.mjs';

await mkdir(join(homedir(), '.dev.mcp'), { recursive: true, mode: 0o700 });
const docker = new Docker();
const resp = args => `*${args.length}\r\n` + args.map(v => `$${Buffer.byteLength(v)}\r\n${v}\r\n`).join('');
async function redis(connection, commands) {
  return new Promise((resolve, reject) => {
    let output = '';
    const socket = createConnection({ host: '127.0.0.1', port: connection.endpoints[0].port }, () => socket.write([['AUTH', connection.username, connection.password], ...commands, ['ECHO', 'done']].map(resp).join('')));
    socket.setTimeout(5000, () => socket.destroy(new Error('Redis test timed out')));
    socket.on('error', reject);
    socket.on('data', data => { output += data; if (output.endsWith('$4\r\ndone\r\n')) { socket.destroy(); resolve(output); } });
  });
}
async function sql(id, connection, query, database = connection.database) {
  return docker.run(['exec', '-i', id, 'sh', '-c', 'IFS= read -r PGPASSWORD; export PGPASSWORD; exec psql -h 127.0.0.1 -U "$1" -d "$2" -X -v ON_ERROR_STOP=1 -At', 'sh', connection.username, database], { input: `${connection.password}\n${query}\n` });
}
async function mc(id, connection, args, input) {
  const config = { version: '10', aliases: { project: { url: 'http://127.0.0.1:9000', accessKey: connection.accessKeyId, secretKey: connection.secretAccessKey, api: 'S3v4', path: 'auto' } } };
  await docker.run(['exec', '-i', id, 'sh', '-c', 'umask 077; mkdir -p /tmp/project-mc; cat > /tmp/project-mc/config.json'], { input: JSON.stringify(config) });
  return docker.run(['exec', '-i', id, 'mc', '--config-dir', '/tmp/project-mc', ...args], { input });
}

for (const kind of ['postgres', 'redis', 'minio']) {
  test(`${kind}: two MCP clients share one container; project isolation and data survive recreation`, { timeout: 900_000 }, async t => {
    const suffix = randomBytes(4).toString('hex');
    const alias = `mcp-test-${kind}-${suffix}`;
    const state = await mkdtemp(join(homedir(), '.dev.mcp', 'test-'));
    const volumes = join(homedir(), '.dev.volumes');
    let first, second;
    t.after(async () => {
      try {
        const c = await docker.inspect(`dev-${alias}`);
        if (c && c.Config.Labels?.[OWNER_LABEL]) await docker.run(['rm', '-f', c.Id]);
        await rm(join(volumes, alias), { recursive: true, force: true });
        // Remove only the disposable test catalog's network.
        const { Store } = await import('../src/store.mjs');
        const store = new Store(state);
        const owner = store.get('meta', 'initialized').owner;
        store.close();
        await docker.run(['network', 'rm', `development-${owner.slice(0, 8)}`], { allowFailure: true });
      } finally {
        await first?.client.close(); await second?.client.close();
        await rm(state, { recursive: true, force: true });
      }
    });
    first = await connect(state, volumes); second = await connect(state, volumes);
    const hostPorts = kind === 'postgres' ? { postgres: 0 } : kind === 'redis' ? { redis: 0 } : { s3: 0, console: 0 };
    await Promise.all([call(first.client, 'ensure_service', { template: kind, alias, hostPorts }), call(second.client, 'ensure_service', { template: kind, alias, hostPorts })]);
    const original = await docker.inspect(`dev-${alias}`);
    assert.equal(original.Mounts[0].Source, join(volumes, alias));
    for (const ports of Object.values(original.NetworkSettings.Ports)) for (const p of ports ?? []) assert.equal(p.HostIp, '127.0.0.1');
    await call(first.client, 'provision_project', { alias, project: 'project-one' });
    await call(second.client, 'provision_project', { alias, project: 'project-two-' });
    const one = await call(first.client, 'get_connection', { alias, project: 'project-one' });
    const two = await call(second.client, 'get_connection', { alias, project: 'project-two-' });
    assert.ok(one.password !== two.password, 'Projects must have distinct credentials');
    const listing = JSON.stringify(await call(second.client, 'list_services'));
    assert.ok(!listing.includes(one.password) && !listing.includes(two.password), 'Discovery must not reveal credentials');
    if (kind === 'postgres') {
      await sql(original.Id, one, 'CREATE TABLE marker (value text); INSERT INTO marker VALUES (\'persisted\');');
      await assert.rejects(sql(original.Id, one, 'SELECT 1', two.database));
    } else if (kind === 'redis') {
      assert.ok((await redis(one, [['SET', `${one.keyPrefix}marker`, 'persisted']])).includes('+OK'));
      assert.ok((await redis(two, [['GET', `${one.keyPrefix}marker`]])).includes('NOPERM'));
    } else {
      await mc(original.Id, one, ['pipe', `project/${one.bucket}/marker.txt`], 'persisted');
      await assert.rejects(mc(original.Id, two, ['cat', `project/${one.bucket}/marker.txt`]));
    }
    await call(first.client, 'remove_service', { alias, confirmAlias: alias });
    assert.ok((await stat(join(volumes, alias))).isDirectory());
    await call(second.client, 'ensure_service', { template: kind, alias });
    const recreated = await docker.inspect(`dev-${alias}`);
    assert.notEqual(recreated.Id, original.Id);
    const again = await call(second.client, 'get_connection', { alias, project: 'project-one' });
    assert.ok(again.password === one.password, 'Credentials must survive recreation');
    if (kind === 'postgres') assert.equal((await sql(recreated.Id, again, 'SELECT value FROM marker')).stdout, 'persisted');
    else if (kind === 'redis') assert.ok((await redis(again, [['GET', `${again.keyPrefix}marker`]])).includes('persisted'));
    else assert.equal((await mc(recreated.Id, again, ['cat', `project/${again.bucket}/marker.txt`])).stdout, 'persisted');
  });
}

test('a failed first start on an occupied host port can recover with automatic allocation', { timeout: 120_000 }, async t => {
  const { createServer } = await import('node:net');
  const blocker = createServer();
  await new Promise(resolve => blocker.listen(0, '127.0.0.1', resolve));
  const alias = `mcp-test-port-${randomBytes(4).toString('hex')}`;
  const state = await mkdtemp(join(homedir(), '.dev.mcp', 'test-'));
  const volumes = join(homedir(), '.dev.volumes', `${alias},volumes`);
  const connection = await connect(state, volumes);
  t.after(async () => {
    await new Promise(resolve => blocker.close(resolve));
    const c = await docker.inspect(`dev-${alias}`);
    if (c && c.Config.Labels?.[OWNER_LABEL]) await docker.run(['rm', '-f', c.Id]);
    await connection.client.close();
    const { Store } = await import('../src/store.mjs');
    const store = new Store(state); const owner = store.get('meta', 'initialized').owner; store.close();
    await docker.run(['network', 'rm', `development-${owner.slice(0, 8)}`], { allowFailure: true });
    await rm(volumes, { recursive: true, force: true });
    await rm(state, { recursive: true, force: true });
  });
  await assert.rejects(call(connection.client, 'ensure_service', { template: 'redis', alias, hostPorts: { redis: blocker.address().port } }));
  const result = await call(connection.client, 'ensure_service', { template: 'redis', alias, hostPorts: { redis: 0 } });
  assert.equal(result.state, 'running');
  assert.ok(result.endpoints[0].port > 0);
});
