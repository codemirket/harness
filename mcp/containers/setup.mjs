import { configureCodexTimeout } from './setup-codex.mjs';
import { spawnSync } from 'node:child_process';
import { mkdir, readFile, writeFile, readdir, cp, rename, rm, lstat } from 'node:fs/promises';
import { createHash, randomUUID } from 'node:crypto';
import { homedir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const source = dirname(fileURLToPath(import.meta.url));
const windows = process.platform === 'win32';

function run(command, args, options = {}) {
  // PowerShell invokes .cmd/.exe using an argument array; user paths are data,
  // never interpolated into shell code (including paths with spaces or &).
  const payload = JSON.stringify({ command, args });
  const result = windows && /\.(cmd|bat)$/i.test(command)
    ? spawnSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', '$p = $env:DEVELOPMENT_SETUP_COMMAND | ConvertFrom-Json; $cliArgs = @($p.args); & ($p.command) @cliArgs; exit $LASTEXITCODE'], { ...options, encoding: 'utf8', windowsHide: true, env: { ...process.env, DEVELOPMENT_SETUP_COMMAND: payload } })
    : spawnSync(command, args, { ...options, encoding: 'utf8', windowsHide: true });
  return result;
}
function executable(name) {
  const result = run(windows ? 'where.exe' : '/usr/bin/which', [name]);
  if (result.status !== 0) throw new Error(`Install ${name} and make it available on PATH before setup.`);
  const paths = result.stdout.trim().split(/\r?\n/);
  const selected = windows ? paths.find(path => /\.(exe|cmd|bat)$/i.test(path)) : paths[0];
  if (!selected) throw new Error(`No executable Windows shim found for ${name}.`);
  return selected;
}
export function registrationArgs(agent, node, entry, docker, environment = {}) {
  const env = { DEVELOPMENT_MCP_DOCKER: docker, ...environment };
  const args = agent === 'codex' ? ['mcp', 'add', 'containers'] : ['mcp', 'add', '--scope', 'user', '--transport', 'stdio', 'containers'];
  for (const [key, value] of Object.entries(env)) args.push('--env', `${key}=${value}`);
  return [...args, '--', node, entry];
}
export function isOwnedRegistration(existing, prior) {
  const value = existing?.transport ?? existing;
  return !!prior && [prior, ...(prior.previous ?? [])].some(candidate => value?.command === candidate.node && JSON.stringify(value?.args) === JSON.stringify([candidate.entry]));
}
async function contentFiles(directory, prefix = '') {
  const result = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const relative = join(prefix, entry.name);
    if (entry.isDirectory()) result.push(...await contentFiles(join(directory, entry.name), relative));
    else if (entry.isFile()) result.push(relative);
  }
  return result.sort();
}
async function installRuntime(runtimeRoot) {
  const files = ['package.json', 'package-lock.json'];
  for (const dir of ['src', 'images']) for (const relative of await contentFiles(join(source, dir))) files.push(join(dir, relative));
  const hash = createHash('sha256');
  for (const file of files.sort()) hash.update(file).update(await readFile(join(source, file)));
  const destination = join(runtimeRoot, 'releases', hash.digest('hex').slice(0, 20));
  try { await readFile(join(destination, '.installed')); return destination; } catch (error) { if (error.code !== 'ENOENT') throw error; }
  const npmCli = process.env.npm_execpath;
  if (!npmCli) throw new Error('Run setup through npm: npm run setup -- --agent all');
  const staging = `${destination}.new-${randomUUID()}`;
  await mkdir(staging, { recursive: true, mode: 0o700 });
  try {
    for (const file of ['package.json', 'package-lock.json', 'src', 'images']) await cp(join(source, file), join(staging, file), { recursive: true });
    const install = run(process.execPath, [npmCli, 'ci', '--omit=dev', '--ignore-scripts', '--no-audit', '--no-fund'], { cwd: staging, stdio: 'inherit' });
    if (install.status !== 0) throw new Error('Runtime dependency installation failed.');
    await writeFile(join(staging, '.installed'), '1\n', { mode: 0o600 });
    try { await rename(staging, destination); }
    catch (error) { if (!['EEXIST', 'ENOTEMPTY'].includes(error.code)) throw error; await readFile(join(destination, '.installed')); }
    return destination;
  } finally { await rm(staging, { recursive: true, force: true }); }
}
async function claudeEntry(serverName = 'containers') {
  const directory = process.env.CLAUDE_CONFIG_DIR ?? homedir();
  const file = join(directory, '.claude.json');
  try { return JSON.parse(await readFile(file, 'utf8')).mcpServers?.[serverName]; }
  catch (error) { if (error.code === 'ENOENT') return undefined; throw new Error('Claude user configuration could not be read; fix it before registration.'); }
}

async function readDesktop(file) {
  try {
    if ((await lstat(file)).isSymbolicLink()) throw new Error('Claude Desktop config is a symbolic link; configure its target explicitly.');
    const value = JSON.parse(await readFile(file, 'utf8'));
    if (!value || typeof value !== 'object' || Array.isArray(value) || (value.mcpServers && (typeof value.mcpServers !== 'object' || Array.isArray(value.mcpServers)))) throw new Error('Claude Desktop configuration has an invalid shape.');
    return value;
  } catch (error) { if (error.code === 'ENOENT') return {}; throw error; }
}
export async function registerDesktop(file, registration, prior) {
  const config = await readDesktop(file);
  const existing = config.mcpServers?.containers;
  const desired = { node: registration.command, entry: registration.args[0] };
  if (existing && !isOwnedRegistration(existing, prior) && !isOwnedRegistration(existing, desired)) throw new Error('An unrelated Claude Desktop MCP server is already named containers.');
  config.mcpServers = { ...config.mcpServers, containers: registration };
  if (isOwnedRegistration(config.mcpServers.development, prior)) delete config.mcpServers.development;
  const temporary = `${file}.new-${randomUUID()}`;
  try {
    await writeFile(temporary, JSON.stringify(config, null, 2) + '\n', { mode: 0o600, flag: 'wx' });
    await rename(temporary, file);
  } finally { await rm(temporary, { force: true }); }
}
async function desktopPath() {
  const directory = windows ? join(process.env.APPDATA ?? join(homedir(), 'AppData', 'Roaming'), 'Claude') : join(homedir(), 'Library', 'Application Support', 'Claude');
  try { if ((await lstat(directory)).isDirectory()) return join(directory, 'claude_desktop_config.json'); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
}

export async function setup(agent = 'all') {
  if (Number(process.versions.node.split('.')[0]) < 24) throw new Error('Node.js 24 or newer is required.');
  if (!['all', 'codex', 'claude'].includes(agent)) throw new Error('Agent must be all, codex, or claude.');
  const agents = agent === 'all' ? ['codex', 'claude'] : [agent];
  const commands = Object.fromEntries(agents.map(name => [name, executable(name)]));
  const docker = executable('docker');
  const runtimeRoot = windows ? join(process.env.LOCALAPPDATA ?? join(homedir(), 'AppData', 'Local'), 'containers-mcp') : join(homedir(), '.local', 'share', 'containers-mcp');
  const runtime = await installRuntime(runtimeRoot);
  const entry = join(runtime, 'src', 'index.mjs');
  const state = resolve(process.env.DEVELOPMENT_MCP_HOME ?? join(homedir(), '.dev.mcp'));
  const volumes = resolve(process.env.DEVELOPMENT_MCP_VOLUMES ?? join(homedir(), '.dev.volumes'));
  await mkdir(state, { recursive: true, mode: 0o700 });
  await mkdir(volumes, { recursive: true, mode: 0o700 });
  if (windows) {
    const permissions = run('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', '$ErrorActionPreference = "Stop"; $sid = [System.Security.Principal.WindowsIdentity]::GetCurrent().User; foreach ($dir in @($env:DEVELOPMENT_SETUP_STATE, $env:DEVELOPMENT_SETUP_VOLUMES)) { $acl = New-Object System.Security.AccessControl.DirectorySecurity; $acl.SetOwner($sid); $acl.SetAccessRuleProtection($true, $false); foreach ($principal in @($sid, (New-Object System.Security.Principal.SecurityIdentifier "S-1-5-18"))) { $rule = New-Object System.Security.AccessControl.FileSystemAccessRule($principal, "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow"); $acl.AddAccessRule($rule) }; Set-Acl -LiteralPath $dir -AclObject $acl }'], { env: { ...process.env, DEVELOPMENT_SETUP_STATE: state, DEVELOPMENT_SETUP_VOLUMES: volumes } });
    if (permissions.status !== 0) throw new Error('Could not restrict local state and volume permissions.');
  }
  const { Store } = await import(pathToFileURL(join(runtime, 'src', 'store.mjs')).href);
  const store = new Store(state);
  try {
    await store.exclusive(async () => {
      // Validate every destination before modifying any registration.
      const registrations = [];
      for (const name of agents) {
        const prior = store.get('meta', `registration-${name}`);
        let existing;
        let legacy;
        if (name === 'codex') {
          const result = run(commands[name], ['mcp', 'get', 'containers', '--json']);
          if (result.status === 0) existing = JSON.parse(result.stdout);
          else if (!/No MCP server named|not found/i.test(result.stderr ?? '')) throw new Error('Could not inspect Codex MCP configuration.');
          const old = run(commands[name], ['mcp', 'get', 'development', '--json']);
          if (old.status === 0) legacy = JSON.parse(old.stdout);
          else if (!/No MCP server named|not found/i.test(old.stderr ?? '')) throw new Error('Could not inspect legacy Codex MCP configuration.');
        } else {
          existing = await claudeEntry();
          legacy = await claudeEntry('development');
        }
        if (existing && !isOwnedRegistration(existing, prior) && !isOwnedRegistration(existing, { node: process.execPath, entry })) throw new Error(`An unrelated ${name} MCP server is already named containers. Rename that registration before running setup.`);
        registrations.push({ name, existing, prior, legacy });
      }
      const desktop = agents.includes('claude') ? await desktopPath() : undefined;
      const desktopPrior = store.get('meta', 'registration-claude-desktop');
      if (desktop) {
        const existing = (await readDesktop(desktop)).mcpServers?.containers;
        if (existing && !isOwnedRegistration(existing, desktopPrior) && !isOwnedRegistration(existing, { node: process.execPath, entry })) throw new Error('An unrelated Claude Desktop MCP server is already named containers.');
      }
      for (const { name, existing, prior, legacy } of registrations) {
        // Save the intended registration before the external CLI operation so a retry
        // can recognize it if the process exits after the CLI succeeds.
        store.put('meta', `registration-${name}`, { node: process.execPath, entry, previous: prior ? [{ node: prior.node, entry: prior.entry }, ...(prior.previous ?? [])].slice(0, 8) : [] });
        if (name === 'claude' && existing) {
          if (run(commands[name], ['mcp', 'remove', '--scope', 'user', 'containers']).status !== 0) throw new Error('Could not update Claude containers registration.');
        }
        const args = registrationArgs(name, process.execPath, entry, docker, { DEVELOPMENT_MCP_HOME: state, DEVELOPMENT_MCP_VOLUMES: volumes });
        const result = run(commands[name], args);
        if (result.status !== 0) throw new Error(`${name} MCP registration failed. Rerun setup after checking its CLI configuration.`);
        if (name === 'codex') await configureCodexTimeout({ command: commands[name], args: ['app-server', '--stdio'] });
        if (isOwnedRegistration(legacy, prior)) {
          const remove = name === 'codex' ? ['mcp', 'remove', 'development'] : ['mcp', 'remove', '--scope', 'user', 'development'];
          if (run(commands[name], remove).status !== 0) throw new Error(`Registered containers, but could not remove the old ${name} development registration. Rerun setup.`);
        }
        console.log(`Registered containers MCP globally for ${name}.`);
      }
      if (desktop) {
        const next = { node: process.execPath, entry, previous: desktopPrior ? [desktopPrior, ...(desktopPrior.previous ?? [])].slice(0, 8) : [] };
        store.put('meta', 'registration-claude-desktop', next);
        await registerDesktop(desktop, { command: process.execPath, args: [entry], env: { DEVELOPMENT_MCP_DOCKER: docker, DEVELOPMENT_MCP_HOME: state, DEVELOPMENT_MCP_VOLUMES: volumes } }, next);
        console.log('Registered containers MCP for Claude Desktop chat.');
      }
    });
  } finally { store.close(); }
  console.log(`Runtime: ${runtime}\nCatalog: ${state}\nVolumes: ${volumes}\nRestart agent sessions to load containers MCP. Containers start on demand.`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  process.umask(0o077);
  const index = process.argv.indexOf('--agent');
  setup(index < 0 ? 'all' : process.argv[index + 1]).catch(error => { console.error(error.message); process.exitCode = 1; });
}
