import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';

// Use Codex's native versioned config API rather than rewriting the user's TOML.
export async function configureCodexTimeout({ command, args }) {
  const usePowerShell = process.platform === 'win32' && /\.(cmd|bat)$/i.test(command);
  const child = usePowerShell
    ? spawn('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', '$p = $env:DEVELOPMENT_SETUP_COMMAND | ConvertFrom-Json; $cliArgs = @($p.args); & ($p.command) @cliArgs; exit $LASTEXITCODE'], { stdio: ['pipe', 'pipe', 'pipe'], windowsHide: true, env: { ...process.env, DEVELOPMENT_SETUP_COMMAND: JSON.stringify({ command, args }) } })
    : spawn(command, args, { stdio: ['pipe', 'pipe', 'pipe'], windowsHide: true });
  const pending = new Map();
  let nextId = 1;
  const lines = createInterface({ input: child.stdout });
  const fail = () => { for (const request of pending.values()) request.reject(new Error('Codex configuration API disconnected. Rerun setup.')); pending.clear(); };
  child.on('error', fail);
  child.on('exit', fail);
  child.stdin.on('error', fail);
  child.stderr.resume(); // Do not echo app-server diagnostics or configuration content.
  lines.on('line', line => {
    let response;
    try { response = JSON.parse(line); } catch { return; }
    const request = pending.get(response.id);
    if (!request) return;
    pending.delete(response.id);
    if (response.error) request.reject(new Error('Codex rejected the versioned configuration update. Rerun setup after resolving concurrent or managed configuration changes.'));
    else request.resolve(response.result);
  });
  const request = (method, params) => new Promise((resolve, reject) => {
    const id = nextId++;
    pending.set(id, { resolve, reject });
    child.stdin.write(JSON.stringify({ id, method, params }) + '\n');
  });
  const timer = setTimeout(() => { fail(); child.kill(); }, 30_000);
  try {
    await request('initialize', { clientInfo: { name: 'containers-mcp-setup', version: '1.0.0' } });
    child.stdin.write(JSON.stringify({ method: 'initialized', params: {} }) + '\n');
    const config = await request('config/read', { includeLayers: true });
    const layer = config.layers?.find(item => item.name.type === 'user' && !item.name.profile);
    if (!layer) throw new Error('Codex did not return its writable user configuration layer.');
    const result = await request('config/value/write', { keyPath: 'mcp_servers.containers.tool_timeout_sec', value: 1800, mergeStrategy: 'replace', filePath: layer.name.file, expectedVersion: layer.version });
    if (result.status !== 'ok') throw new Error('A managed Codex setting overrides the containers tool timeout.');
  } finally {
    clearTimeout(timer);
    child.stdin.end();
    child.kill();
    lines.close();
  }
}
