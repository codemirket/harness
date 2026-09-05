import { mkdir, lstat } from 'node:fs/promises';
import { join } from 'node:path';
import { randomBytes, createHash } from 'node:crypto';
import { setTimeout as delay } from 'node:timers/promises';
import { nameSchema, templateSchema } from './catalog.mjs';
import { OWNER_LABEL, ALIAS_LABEL } from './docker.mjs';
import { privateDirectory, privateFile } from './private-files.mjs';
import { provisionProject } from './projects.mjs';

export const password = () => randomBytes(24).toString('hex');
export const projectIdentity = project => `p_${project.replaceAll('-', '_')}_${createHash('sha256').update(project).digest('hex').slice(0, 8)}`;

export class Manager {
  constructor(store, docker, volumesRoot) {
    this.store = store; this.docker = docker; this.volumesRoot = volumesRoot;
    this.owner = store.get('meta', 'initialized').owner;
  }
  name(alias) { return `dev-${nameSchema.parse(alias)}`; }
  service(alias) {
    nameSchema.parse(alias);
    const value = this.store.get('services', alias);
    if (!value) throw new Error(`Unknown service ${alias}; call ensure_service first.`);
    return value;
  }
  async local() {
    const endpoint = await this.docker.localIdentity();
    this.store.insertOnce('meta', 'docker', { endpoint });
    const saved = this.store.get('meta', 'docker');
    if (saved && saved.endpoint !== endpoint) throw new Error('Docker context differs from this catalog. Restore the original local Docker context before managing its services.');
  }
  publicService(s) {
    return { alias: s.alias, template: s.template.id, kind: s.template.kind, image: s.imageReference ?? s.template.image,
      volume: join(this.volumesRoot, s.alias), containerName: this.name(s.alias), removed: !!s.removed,
      projects: Object.keys(s.projects ?? {}), requestedPorts: s.ports ?? s.template.ports };
  }
  listCatalog() { return this.store.all('templates').map(t => ({ ...t, environment: Object.keys(t.environment ?? {}) })); }
  async listServices() {
    await this.local();
    return { services: this.store.all('services').map(s => this.publicService(s)), containers: await this.docker.list(this.owner) };
  }
  addTemplate(input) {
    const template = { ...templateSchema.parse(input), kind: 'generic' };
    return this.store.exclusive(async () => {
      if (this.store.get('templates', template.id)) throw new Error('Catalog entry already exists; use a new ID to define a different service version.');
      if (this.store.all('services').some(s => s.template.id === template.id)) throw new Error('Catalog ID is reserved by retained service data; choose a new ID.');
      this.store.put('templates', template.id, template);
      return { id: template.id, added: true };
    });
  }
  removeTemplate(id) {
    nameSchema.parse(id);
    return this.store.exclusive(async () => {
      if (this.store.all('services').some(s => s.template.id === id && !s.removed)) throw new Error('Catalog entry is in use; remove its containers first.');
      if (!this.store.get('templates', id)) throw new Error('Unknown catalog entry.');
      this.store.delete('templates', id);
      return { id, removed: true };
    });
  }
  async owned(s) {
    const container = await this.docker.inspect(this.name(s.alias));
    if (container && (container.Config.Labels?.[OWNER_LABEL] !== this.owner || container.Config.Labels?.[ALIAS_LABEL] !== s.alias)) throw new Error('Container name is not owned by this development catalog; refusing to modify it.');
    return container;
  }
  async volume(alias) {
    await mkdir(this.volumesRoot, { recursive: true, mode: 0o700 });
    const directory = join(this.volumesRoot, nameSchema.parse(alias));
    try { if ((await lstat(directory)).isSymbolicLink()) throw new Error('Service volume must not be a symbolic link.'); }
    catch (error) { if (error.code !== 'ENOENT') throw error; }
    await mkdir(directory, { recursive: true, mode: 0o700 });
    return directory;
  }
  async configuration(s, directory) {
    const env = { ...s.template.environment, ...s.secrets.environment };
    let command = s.template.command;
    const config = join(directory, '.development');
    await privateDirectory(config);
    const write = (file, text) => privateFile(join(config, file), text);
    if (s.template.kind === 'postgres') {
      // The image entrypoint reads the password as root before switching user.
      await write('password', s.secrets.password);
      Object.assign(env, { POSTGRES_USER: 'devadmin', POSTGRES_PASSWORD_FILE: '/var/lib/postgresql/.development/password', PGDATA: '/var/lib/postgresql/18/docker' });
    } else if (s.template.kind === 'redis') {
      await write('redis.conf', `bind 0.0.0.0\nprotected-mode yes\nport 6379\ndir /data\nappendonly yes\naclfile /data/.development/users.acl\n`);
      await privateFile(join(config, 'users.acl'), `user default on >${s.secrets.password} ~* &* +@all\n`, { exclusive: true });
      // Root reads the mode-0600 config and starts Redis. Bound data remains private to the host user.
      command = ['redis-server', '/data/.development/redis.conf'];
    } else if (s.template.kind === 'minio') {
      await write('password', s.secrets.password);
      Object.assign(env, { MINIO_ROOT_USER: 'devadmin', MINIO_ROOT_PASSWORD: s.secrets.password });
      command = ['server', '/data/objects', '--console-address', ':9001'];
    }
    return { env, command };
  }
  ensureService(templateId, alias = templateId, hostPorts = {}) {
    nameSchema.parse(templateId); nameSchema.parse(alias);
    return this.store.exclusive(async () => {
      await this.local();
      let s = this.store.get('services', alias);
      if (s && s.template.id !== templateId) throw new Error('Service alias already belongs to another catalog entry.');
      if (!s) {
        const template = this.store.get('templates', templateId);
        if (!template) throw new Error('Unknown catalog entry.');
        s = { alias, template, ports: structuredClone(template.ports), secrets: { password: password(), environment: Object.fromEntries(template.secretEnvironment.map(key => [key, password()])) }, projects: {}, createdAt: new Date().toISOString() };
      }
      let c = await this.owned(s);
      let changedPorts = false;
      for (const [name, value] of Object.entries(hostPorts)) {
        if (!Number.isInteger(value) || value < 0 || value > 65535) throw new Error('Host ports must be integers from 0 through 65535.');
        const port = s.ports.find(p => p.name === name);
        if (!port) throw new Error(`Unknown port ${name}.`);
        if (port.host !== value) changedPorts = true;
        port.host = value;
      }
      if (changedPorts && c) {
        if (c.State.Running || (c.State.StartedAt && !c.State.StartedAt.startsWith('0001-'))) throw new Error('Remove the existing service container before changing its published ports. Data will be retained.');
        await this.docker.run(['rm', c.Id]);
        c = undefined; // A never-started container can be retried with corrected ports.
      }
      this.store.put('services', alias, s); // durable intent, including secrets, BEFORE Docker side effects
      if (!c) {
        const image = await this.docker.image(s.imageReference ?? s.template.image, s.template.kind);
        s.imageId = image.id; s.imageReference = image.reference;
        this.store.put('services', alias, s);
        const directory = await this.volume(alias);
        const { env, command } = await this.configuration(s, directory);
        const network = `development-${this.owner.slice(0, 8)}`;
        const net = await this.docker.run(['network', 'inspect', network], { allowFailure: true });
        if (net.code !== 0) await this.docker.run(['network', 'create', '--label', `${OWNER_LABEL}=${this.owner}`, network]);
        const args = ['create', '--name', this.name(alias), '--restart', 'unless-stopped', '--label', `${OWNER_LABEL}=${this.owner}`, '--label', `${ALIAS_LABEL}=${alias}`, '--network', network, '--network-alias', alias,
          '--mount', `type=bind,"source=${directory.replaceAll('"', '""')}",target=${s.template.dataPath}`];
        for (const p of s.ports) args.push('--publish', `127.0.0.1:${p.host || ''}:${p.container}`);
        // redis's entrypoint drops privileges before reading its configuration.
        // Starting as root avoids granting world-read access to credential files.
        if (s.template.kind === 'redis') args.push('--entrypoint', 'redis-server');
        // Pass secrets through a protected environment file, not process arguments.
        const envFile = join(directory, '.development', 'container.env');
        await privateFile(envFile, Object.entries(env).map(([k, v]) => {
          if (/[\r\n\0]/.test(v)) throw new Error('Container environment values must be single-line strings.');
          return `${k}=${v}`;
        }).join('\n'));
        args.push('--env-file', envFile, image.id, ...(s.template.kind === 'redis' ? command.slice(1) : command));
        const result = await this.docker.run(args);
        await this.docker.run(['start', result.stdout]);
        c = await this.owned(s);
      } else if (!c.State.Running) {
        await this.docker.run(['start', c.Id]);
        c = await this.owned(s);
      }
      s.removed = false;
      this.store.put('services', alias, s);
      await this.waitReady(s, c);
      // ACL users live in memory; restore them after any Redis restart.
      if (s.template.kind === 'redis') for (const project of Object.keys(s.projects)) await provisionProject(this, s, project);
      return this.details(s);
    });
  }
  async waitReady(s, c) {
    const deadline = Date.now() + 60_000;
    while (Date.now() < deadline) {
      const current = await this.owned(s);
      if (!current) throw new Error('Service disappeared during startup.');
      if (current.State.Running) {
        try {
          if (s.template.kind === 'postgres') await this.docker.run(['exec', current.Id, 'pg_isready', '-U', 'devadmin', '-d', 'postgres'], { timeout: 3000 });
          else if (s.template.kind === 'redis') await this.redis(s, ['PING']);
          else if (s.template.kind === 'minio') await this.docker.run(['exec', current.Id, 'curl', '--fail', '--silent', 'http://127.0.0.1:9000/minio/health/ready'], { timeout: 3000 });
          return;
        } catch { /* init scripts and databases may need a few seconds */ }
      }
      await delay(500);
    }
    throw new Error(`Service ${s.alias} did not become ready within 60 seconds. Its container and data were retained for diagnosis.`);
  }
  async details(s) {
    const c = await this.owned(s);
    return { ...this.publicService(s), state: c?.State.Status ?? 'missing', endpoints: this.endpoints(s, c), network: c ? Object.keys(c.NetworkSettings.Networks) : [] };
  }
  endpoints(s, c) {
    return (s.ports ?? s.template.ports).map(p => ({ name: p.name, host: '127.0.0.1', port: Number(c?.NetworkSettings?.Ports?.[`${p.container}/tcp`]?.[0]?.HostPort) || null, containerHost: s.alias, containerPort: p.container }));
  }
  async inspectService(alias) { await this.local(); return this.details(this.service(alias)); }
  stopService(alias, confirmAlias) {
    if (alias !== confirmAlias) throw new Error('Confirm the service alias; stopping interrupts every project using it.');
    return this.store.exclusive(async () => {
      await this.local(); const s = this.service(alias); const c = await this.owned(s);
      if (c?.State.Running) await this.docker.run(['stop', '--time', '20', c.Id]);
      return this.details(s);
    });
  }
  removeService(alias, confirmAlias) {
    if (alias !== confirmAlias) throw new Error('Confirm the service alias; removal interrupts every project using it.');
    return this.store.exclusive(async () => {
      await this.local(); const s = this.service(alias); const c = await this.owned(s);
      if (c) {
        if (c.State.Running) await this.docker.run(['stop', '--time', '20', c.Id]);
        await this.docker.run(['rm', c.Id]);
      }
      s.removed = true;
      this.store.put('services', alias, s);
      return { alias, removed: true, dataPreserved: true, volume: join(this.volumesRoot, alias), projects: Object.keys(s.projects) };
    });
  }
  async redis(s, args) {
    const c = await this.owned(s);
    // RESP on stdin keeps passwords out of process arguments and supports arbitrary strings.
    const command = values => `*${values.length}\r\n` + values.map(v => `$${Buffer.byteLength(String(v))}\r\n${v}\r\n`).join('');
    const result = await this.docker.run(['exec', '-i', c.Id, 'redis-cli', '--pipe'], { input: command(['AUTH', s.secrets.password]) + command(args) });
    if (!/errors: 0/.test(result.stdout)) throw new Error('Redis command failed; credentials or ACL configuration may differ from the catalog.');
    return result;
  }
  project(alias, project) {
    nameSchema.parse(project);
    return this.store.exclusive(async () => {
      await this.local(); const s = this.service(alias); const c = await this.owned(s);
      if (!c?.State.Running) throw new Error('Start the service with ensure_service before provisioning a project.');
      if (s.template.kind === 'generic') throw new Error('Custom services expose shared credentials; automatic project provisioning is available for PostgreSQL, Redis and MinIO.');
      if (!s.projects[project]) {
        s.projects[project] = { name: project, username: projectIdentity(project), password: password(), database: projectIdentity(project), prefix: `${project}:`, bucket: `dev-${project}-${createHash('sha256').update(project).digest('hex').slice(0, 8)}` };
        this.store.put('services', alias, s);
      }
      await provisionProject(this, s, project);
      s.projects[project].ready = true;
      this.store.put('services', alias, s);
      return { alias, project, ready: true, message: 'Use get_connection with this project to retrieve credentials.' };
    });
  }
  async connection(alias, project) {
    await this.local(); const s = this.service(alias); const c = await this.owned(s);
    if (!c?.State.Running) throw new Error('Service is not running; call ensure_service first.');
    const p = project ? s.projects[nameSchema.parse(project)] : undefined;
    if (project && !p?.ready) throw new Error('Project is not provisioned; call provision_project first.');
    const endpoints = this.endpoints(s, c);
    if (s.template.kind === 'generic') return { alias, scope: 'shared', endpoints, environment: { ...s.template.environment, ...s.secrets.environment } };
    const port = endpoints[0].port;
    const username = p?.username ?? (s.template.kind === 'redis' ? 'default' : 'devadmin');
    const secret = p?.password ?? s.secrets.password;
    const result = { alias, project: project ?? null, scope: project ? 'project' : 'administrator', endpoints, username, password: secret };
    if (s.template.kind === 'postgres') Object.assign(result, { database: p?.database ?? 'postgres', url: `postgresql://${username}:${secret}@127.0.0.1:${port}/${p?.database ?? 'postgres'}` });
    else if (s.template.kind === 'redis') Object.assign(result, { database: 0, keyPrefix: p?.prefix ?? '', url: `redis://${username}:${secret}@127.0.0.1:${port}/0`, note: 'Project ACLs require the key prefix. Shared infrastructure is for trusted local projects.' });
    else if (s.template.kind === 'minio') Object.assign(result, { endpoint: `http://127.0.0.1:${port}`, accessKeyId: username, secretAccessKey: secret, bucket: p?.bucket, region: 'us-east-1', forcePathStyle: true });
    return result;
  }
}
