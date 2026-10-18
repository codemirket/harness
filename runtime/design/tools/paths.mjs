import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawn} from 'node:child_process';
import {constants} from 'node:fs';
import {fileURLToPath} from 'node:url';
const guard = fileURLToPath(new URL('./runtime-guard', import.meta.url));
import {checkRequest, requestSignal, cancelAllRequests, redactSecrets} from './request.mjs';

export const workspace = path.resolve(process.env.MIRKET_DESIGN_WORKSPACE || '/workspace');
export const MAX_INPUT = 32 * 1024 * 1024;
export const MAX_OUTPUT = 64 * 1024 * 1024;
const activeChildren = new Set();
const CHILD_GRACE_MS = 3000;
function signalGroup(child, signal) {
  try { if (process.platform !== 'win32') process.kill(-child.pid, signal); else child.kill(signal); } catch {}
}

let shuttingDown = false;
let childrenShutdown;
export function shutdownChildren() {
  if (childrenShutdown) return childrenShutdown;
  childrenShutdown = (async () => {
    cancelAllRequests();
    const children = [...activeChildren];
    for (const record of children) record.stop('Design runtime terminating');
    let deadline;
    try {
      await Promise.race([
        Promise.allSettled(children.map(record => record.done)),
        new Promise(resolve => { deadline = setTimeout(resolve, CHILD_GRACE_MS + 1000); }),
      ]);
    } finally { clearTimeout(deadline); }
    for (const record of activeChildren) record.hardKill();
  })();
  return childrenShutdown;
}
for (const signal of ['SIGTERM', 'SIGINT']) process.once(signal, async () => {
  if (shuttingDown) return;
  shuttingDown = true;
  await shutdownChildren();
  process.exit(128 + (signal === 'SIGTERM' ? 15 : 2));
});
process.stdin?.on('end', () => { void shutdownChildren(); });

export function sha256(bytes) { return crypto.createHash('sha256').update(bytes).digest('hex'); }
export function assertRelative(name) {
  if (typeof name !== 'string' || !name || name.length > 512 || name.includes('\\') || name.includes('\0') || path.posix.isAbsolute(name)) throw Error('Expected a bounded workspace-relative path');
  const parts = name.split('/');
  if (parts.some(p => !p || p === '.' || p === '..')) throw Error('Workspace path contains an empty or traversal component');
  return name;
}
export async function workspacePath(name, {kind = 'file', missing = false, createParents = false} = {}) {
  assertRelative(name);
  const parts = name.split('/');
  let current = workspace;
  const root = await fs.lstat(current);
  if (!root.isDirectory() || root.isSymbolicLink()) throw Error('Workspace root is not a real directory');
  for (let i = 0; i < parts.length; i++) {
    current = path.join(current, parts[i]);
    let stat;
    try { stat = await fs.lstat(current); }
    catch (error) {
      if (error.code !== 'ENOENT') throw error;
      if (i < parts.length - 1 && createParents) { await fs.mkdir(current).catch(error => { if (error.code !== 'EEXIST') throw error; }); stat = await fs.lstat(current); }
      else if (i === parts.length - 1 && missing) return current;
      else throw Error(`Missing workspace path: ${name}`);
    }
    if (stat.isSymbolicLink()) throw Error(`Workspace path crosses a symlink: ${name}`);
    if (i < parts.length - 1 && !stat.isDirectory()) throw Error(`Workspace parent is not a directory: ${name}`);
    if (i === parts.length - 1 && kind === 'file' && !stat.isFile()) throw Error(`Expected regular file: ${name}`);
    if (i === parts.length - 1 && kind === 'directory' && !stat.isDirectory()) throw Error(`Expected directory: ${name}`);
  }
  return current;
}
export async function readBoundedFile(file, max, label = 'Input') {
  if (!Number.isSafeInteger(max) || max < 0) throw Error('Invalid file byte budget');
  const handle = await fs.open(file, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    const stat = await handle.stat();
    if (!stat.isFile()) throw Error(`${label} is not a regular file`);
    if (stat.size > max) throw Error(`${label} exceeds ${max} bytes`);
    const chunks = []; let size = 0;
    for (;;) {
      checkRequest();
      const buffer = Buffer.allocUnsafe(Math.min(65536, max - size + 1));
      const {bytesRead} = await handle.read(buffer, 0, buffer.length, null);
      if (!bytesRead) break;
      size += bytesRead;
      if (size > max) throw Error(`${label} exceeds ${max} bytes`);
      chunks.push(buffer.subarray(0, bytesRead));
    }
    return Buffer.concat(chunks, size);
  } finally { await handle.close(); }
}
export async function readFile(name, max = MAX_INPUT) {
  return readBoundedFile(await workspacePath(name), max, `Input ${name}`);
}
export async function readJson(name, max = MAX_INPUT) {
  return JSON.parse((await readFile(name, max)).toString('utf8'));
}
async function publicationLock(name, fn) {
  const root = await workspacePath('.mirket-design/locks', {kind:'directory', missing:true, createParents:true});
  await fs.mkdir(root, {recursive:true, mode:0o700});
  const lock = path.join(root, sha256(Buffer.from(name)) + '.lock');
  const child = spawn(guard, ['lock', lock], {stdio:['pipe','pipe','pipe'], env:{LANG:'C.UTF-8'}});
  const done = new Promise(resolve => { child.once('close', resolve); child.once('error', () => resolve(-1)); });
  try {
    await new Promise((resolve, reject) => {
      const timer = setTimeout(() => { child.kill('SIGKILL'); reject(Error('Publication lock timed out')); }, 3000);
      child.once('error', error => { clearTimeout(timer); reject(error); });
      child.once('close', () => { clearTimeout(timer); reject(Error(`Output publication is busy: ${name}`)); });
      child.stdout.once('data', data => { clearTimeout(timer); if (data.toString() === 'locked\n') resolve(); else reject(Error('Invalid publication lock response')); });
    });
    checkRequest();
    return await fn();
  } finally { child.stdin.destroy(); await done; }
}
export async function withRenderSlot(fn) {
  // Cross-process admission: desktop clients share one native renderer budget.
  return publicationLock('@mirket-native-render', fn);
}
export async function publish(name, bytes, options = {}) {
  assertRelative(name);
  await workspacePath(name, {missing:true});
  return publicationLock(name, () => publishLocked(name, bytes, options));
}
async function publishLocked(name, bytes, {overwrite = false, expected_sha256, max = MAX_OUTPUT} = {}) {
  checkRequest();
  if (!Buffer.isBuffer(bytes)) bytes = Buffer.from(bytes);
  if (!bytes.length || bytes.length > max) throw Error('Output is empty or exceeds the size limit');
  const dest = await workspacePath(name, {missing: true});
  let before;
  try { before = await readFile(name, max); }
  catch (error) { if (!String(error.message).startsWith('Missing workspace path:')) throw error; }
  if (before && !overwrite) throw Error(`Output exists; pass overwrite=true: ${name}`);
  if (before && expected_sha256 !== sha256(before)) throw Error(`Output revision mismatch: ${name}`);
  if (!before && expected_sha256) throw Error(`Output does not exist for expected_sha256: ${name}`);
  const stage = path.join(path.dirname(dest), `.${path.basename(dest)}.${crypto.randomUUID()}.stage`);
  try {
    await fs.writeFile(stage, bytes, {flag: 'wx', mode: 0o600});
    await workspacePath(name, {missing: true});
    if (before) {
      const current = await readFile(name, max);
      if (sha256(current) !== sha256(before)) throw Error(`Output changed during publication: ${name}`);
    }
    checkRequest();
    if (before) await fs.rename(stage, dest);
    else await fs.link(stage, dest);
    return {path: name, sha256: sha256(bytes), bytes: bytes.length};
  } finally { await fs.rm(stage, {force: true}).catch(() => {}); }
}
function appendWithinLimit(current, chunk, limit) {
  const remaining = Math.max(0, limit - current.length);
  if (!remaining) return current;
  return Buffer.concat([current, chunk.subarray(0, remaining)]);
}
export async function run(executable, args, {cwd = workspace, env = {}, timeoutMs = 90000, maxStdout = 262144, maxStderr = 262144, allowedCodes = [0]} = {}) {
  checkRequest();
  const allowed = new Set(['HYPERFRAMES_NO_UPDATE_CHECK', 'HYPERFRAMES_NO_TELEMETRY', 'HYPERFRAMES_BROWSER_PATH', 'HYPERFRAMES_FFMPEG_PATH', 'HYPERFRAMES_FFPROBE_PATH']);
  for (const key of Object.keys(env)) if (!allowed.has(key)) throw Error(`Unapproved child environment variable: ${key}`);
  const scratch = await temporaryDirectory('child');
  const childEnv = {PATH: process.env.PATH || '/usr/local/bin:/usr/bin:/bin', HOME: scratch, TMPDIR: scratch, LANG: 'C.UTF-8', XDG_CACHE_HOME: path.join(scratch, 'cache'), HYPERFRAMES_NO_TELEMETRY: '1', ...env};
  let record, finish;
  const done = new Promise(resolveDone => { finish = resolveDone; });
  try {
    return await new Promise((resolve, reject) => {
    const child = spawn(guard, ['run', executable, ...args], {cwd, env: childEnv, stdio: ['ignore', 'pipe', 'pipe'], detached: process.platform !== 'win32'});
    let stdout = Buffer.alloc(0), stderr = Buffer.alloc(0), settled = false, reason = '', stopping = false, closed = false, killTimer;
    const hardKill = () => { if (closed) return; signalGroup(child, 'SIGKILL'); };
    const stop = why => {
      if (closed) return;
      reason ||= why;
      if (stopping) return;
      stopping = true;
      signalGroup(child, 'SIGTERM');
      killTimer = setTimeout(hardKill, CHILD_GRACE_MS);
    };
    record = {stop, hardKill, done};
    activeChildren.add(record);
    const signal = requestSignal();
    const onAbort = () => stop('Process cancelled by client');
    signal?.addEventListener('abort', onAbort, {once: true});
    const cleanup = () => { closed = true; clearTimeout(timer); clearTimeout(killTimer); signal?.removeEventListener('abort', onAbort); };
    const timer = setTimeout(() => stop(`Process timed out after ${timeoutMs} ms`), timeoutMs);
    if (signal?.aborted) stop('Process cancelled by client');
    child.stdout.on('data', chunk => { const over = chunk.length > maxStdout - stdout.length; stdout = appendWithinLimit(stdout, chunk, maxStdout); if (over) stop('Process stdout exceeded limit'); });
    child.stderr.on('data', chunk => { const over = chunk.length > maxStderr - stderr.length; stderr = appendWithinLimit(stderr, chunk, maxStderr); if (over) stop('Process stderr exceeded limit'); });
    child.on('error', error => { cleanup(); if (!settled) { settled = true; reject(error); } });
    child.on('close', code => { cleanup(); if (settled) return; settled = true; const out = stdout.toString('utf8'), err = stderr.toString('utf8'); if (reason || !allowedCodes.includes(code)) reject(Error(redactSecrets(`${reason || `Process failed (${code})`}: ${err.slice(-4000) || out.slice(-4000)}`))); else resolve({code, stdout: redactSecrets(out), stderr: redactSecrets(err)}); });
    });
  } finally {
    try { await fs.rm(scratch, {recursive: true, force: true, maxRetries: 3, retryDelay: 100}); }
    finally { if (record) activeChildren.delete(record); finish(); }
  }
}
export async function temporaryDirectory(prefix = '.mirket-stage') {
  const base = await workspacePath('.mirket-design', {kind: 'directory', missing: true});
  await fs.mkdir(base, {recursive: true, mode: 0o700});
  return fs.mkdtemp(path.join(base, `${prefix}-`));
}
export async function regularOutput(file, max = MAX_OUTPUT) {
  const bytes = await readBoundedFile(file, max, 'Renderer output');
  if (!bytes.length) throw Error('Renderer did not produce a bounded regular output');
  return bytes;
}
