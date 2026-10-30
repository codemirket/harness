// Explicit fresh setup; edit only this package's native pi resource filter.
import { readFileSync, lstatSync, realpathSync, existsSync, readdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { execFileSync, execFile } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { isAbsolute, join, resolve } from 'node:path';
import { supportsMacOSVersion } from './macos-version.mjs';
if (process.platform !== 'darwin' || process.arch !== 'arm64') throw new Error('Mirket requires macOS ARM64 26.0 or newer');
const macOSVersion = execFileSync('/usr/bin/sw_vers', ['-productVersion'], {encoding:'utf8'}).trim();
if (!supportsMacOSVersion(macOSVersion)) throw new Error(`Mirket requires macOS ARM64 26.0 or newer; found ${macOSVersion}`);
const root = fileURLToPath(new URL('../', import.meta.url));
const args = process.argv.slice(2);
const resume=args.length===3 && args[2]==='--resume';
if ((args.length !== 2 && !resume) || args[0] !== '--agent-dir' || !isAbsolute(args[1])) throw new Error('Usage: setup.mjs --agent-dir ABSOLUTE_PRIVATE_PI_AGENT_DIR [--resume]');
const agentDir = resolve(args[1]);
if (realpathSync(agentDir) !== agentDir || !lstatSync(agentDir).isDirectory() || (lstatSync(agentDir).mode & 0o077) !== 0) throw new Error('Select a private real agent directory');
const home = join(agentDir, 'mirket');
if (existsSync(home) && !resume) throw new Error('Fresh setup only: existing home is preserved; use --resume only for an interrupted setup of this package');
if (!existsSync(home) && resume) throw new Error('No interrupted setup to resume');
const hash = file => createHash('sha256').update(readFileSync(file)).digest('hex');
const inventory = JSON.parse(readFileSync(join(root, 'package-inventory.json'), 'utf8'));
if (inventory.schema !== 1 || !Array.isArray(inventory.files)) throw new Error('Invalid package inventory');
const expected = new Set();
for (const entry of inventory.files) {
  if (typeof entry.path !== 'string' || entry.path.startsWith('/') || entry.path.split('/').some(part => !part || part === '.' || part === '..') || expected.has(entry.path)) throw new Error('Invalid inventory path');
  expected.add(entry.path);
  const file = join(root, entry.path), stat = lstatSync(file);
  if (!stat.isFile() || stat.nlink !== 1 || realpathSync(file) !== resolve(file) || stat.size !== entry.bytes || hash(file) !== entry.sha256) throw new Error(`Package file differs from inventory: ${entry.path}`);
}
function checkExtra(directory, relative = '') {
  for (const item of readdirSync(directory, {withFileTypes:true})) {
    const name = relative + item.name;
    if (item.isDirectory()) checkExtra(join(directory,item.name), name+'/');
    else if (item.isSymbolicLink() && name.startsWith('native/')) continue; // Native manifest validates its links.
    else if (!item.isFile() || (name !== 'package-inventory.json' && !expected.has(name))) throw new Error(`Uninventoried package file: ${name}`);
  }
}
checkExtra(root);
const binding = JSON.parse(readFileSync(join(root, 'native-binding.json'), 'utf8'));
const binary = join(root, 'native/bin/mirket');
if (lstatSync(binary).isSymbolicLink() || hash(binary) !== binding.coreSha256 || hash(join(root, 'package.json')) !== binding.packageSha256) throw new Error('Package identity mismatch');
if (hash(process.execPath) !== binding.nodeSha256) throw new Error('Run setup with the bound private Node executable');
const {prepareProfile, applyProfile} = await import('../dist/extension/profiles.js');
const profile = prepareProfile(root, join(agentDir,'settings.json'), [], []);
const result = JSON.parse(await new Promise((accept,reject)=>{
  let escalation;
  const stop=()=>{child.kill('SIGTERM');escalation??=setTimeout(()=>child.kill('SIGKILL'),2000);};
  // Full payload staging journals every file durably. Its finite install window
  // is separate from the status-only first-tool latency budget.
  const child=execFile(binary,['setup','--home',home,'--source',join(root,'native')],{encoding:'utf8',timeout:600000,killSignal:'SIGKILL',maxBuffer:1024*1024},(error,stdout)=>{
    clearTimeout(escalation);process.removeListener('SIGTERM',stop);process.removeListener('SIGINT',stop);
    if(error)reject(error);else accept(stdout);
  });
  process.once('SIGTERM',stop);process.once('SIGINT',stop);
}));
if (result.activation.release_id !== binding.releaseId || result.activation.status !== 'active') throw new Error('Setup did not activate the bound release');
applyProfile(profile);
console.log(JSON.stringify({home, releaseId: binding.releaseId, packageSource: root, selectedSkills:profile.skills, next: 'Reload pi to load the local package and its core skill filter.'}));
