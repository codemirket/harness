import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

export const OWNER_LABEL = 'local.development-mcp.owner';
export const ALIAS_LABEL = 'local.development-mcp.alias';

export class Docker {
  constructor(executable = process.env.DEVELOPMENT_MCP_DOCKER ?? 'docker') { this.executable = executable; }
  run(args, { input, timeout = 60_000, allowFailure = false } = {}) {
    return new Promise((resolve, reject) => {
      const child = spawn(this.executable, args, { stdio: ['pipe', 'pipe', 'pipe'], windowsHide: true, shell: false });
      let stdout = '', stderr = '', killed = false;
      const timer = setTimeout(() => { killed = true; child.kill(); }, timeout);
      child.stdout.on('data', data => { stdout = (stdout + data).slice(-8_000_000); });
      child.stderr.on('data', data => { stderr = (stderr + data).slice(-500_000); });
      child.stdin.on('error', () => {});
      child.on('error', () => { clearTimeout(timer); reject(new Error('Docker CLI could not start. Install Docker Desktop and check DEVELOPMENT_MCP_DOCKER.')); });
      child.on('close', code => {
        clearTimeout(timer);
        if (code === 0 || allowFailure) return resolve({ stdout: stdout.trim(), stderr, code });
        const hint = /port is already allocated|address already in use/.test(stderr) ? 'Requested host port is occupied; choose host port 0 for automatic allocation.'
          : /permission denied/i.test(stderr) ? 'Docker or bind-directory permission was denied.'
          : /Cannot connect|Is the docker daemon running|failed to connect/i.test(stderr) ? 'Start Docker Desktop.'
          : /no space left/i.test(stderr) ? 'Docker or host disk is full.'
          : /manifest unknown|not found|pull access denied/i.test(stderr) ? 'Check the image reference and registry access.'
          : 'Check Docker Desktop, image availability, and service configuration.';
        reject(new Error(`Docker ${args[0]} ${killed ? 'timed out' : 'failed'}. ${hint}`));
      });
      child.stdin.end(input);
    });
  }
  async localIdentity() {
    let endpoint = process.env.DOCKER_HOST;
    if (process.env.DOCKER_CONTEXT || !endpoint) {
      const { stdout } = await this.run(['context', 'inspect', ...(process.env.DOCKER_CONTEXT ? [process.env.DOCKER_CONTEXT] : [])]);
      endpoint = JSON.parse(stdout)[0].Endpoints.docker.Host;
    }
    if (!/^(unix:\/\/|npipe:\/\/)/.test(endpoint ?? '')) throw new Error('Development MCP requires a local Docker socket or named pipe; remote Docker contexts are not supported.');
    const { stdout } = await this.run(['info', '--format', '{{.OSType}}']);
    if (stdout !== 'linux') throw new Error('Switch Docker Desktop to Linux containers.');
    return endpoint;
  }
  async inspect(name) {
    const result = await this.run(['container', 'inspect', name], { allowFailure: true });
    if (result.code === 0) return JSON.parse(result.stdout)[0];
    if (/No such (container|object)/i.test(result.stderr)) return undefined;
    throw new Error('Unable to inspect container; check Docker Desktop.');
  }
  async list(owner) {
    const { stdout } = await this.run(['ps', '-a', '--format', '{{json .}}']);
    if (!stdout) return [];
    const rows = stdout.split('\n').map(line => JSON.parse(line));
    return Promise.all(rows.map(async row => {
      const c = await this.inspect(row.ID);
      if (!c) return { id: row.ID, name: row.Names, state: 'removed' };
      return {
        id: c.Id, name: c.Name.replace(/^\//, ''), image: c.Config.Image,
        state: c.State.Status, health: c.State.Health?.Status,
        managed: c.Config.Labels?.[OWNER_LABEL] === owner,
        alias: c.Config.Labels?.[ALIAS_LABEL],
        ports: c.NetworkSettings.Ports,
        mounts: c.Mounts.map(m => ({ type: m.Type, source: m.Source, destination: m.Destination })),
      };
    }));
  }
  async image(reference, kind) {
    let result = await this.run(['image', 'inspect', reference], { allowFailure: true });
    if (result.code !== 0) {
      if (kind === 'minio') {
        await this.run(['build', '--tag', reference, fileURLToPath(new URL('../images/minio', import.meta.url))], { timeout: 900_000 });
      } else await this.run(['pull', reference], { timeout: 600_000 });
      result = await this.run(['image', 'inspect', reference]);
    }
    const image = JSON.parse(result.stdout)[0];
    return { id: image.Id, reference: image.RepoDigests?.[0] ?? reference };
  }
}
