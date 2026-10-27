// Real-clock, abrupt-owner retirement qualification. No receipt/clock edits.
// Invoke with an explicitly built binary and private Node/design-adapter paths.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, mkdtemp, readFile, readdir, realpath, stat, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { isAbsolute, join } from 'node:path';
import { performance } from 'node:perf_hooks';

const usage = 'node tests/intent-retirement-smoke.mjs --mirket ABS --node ABS --design-adapter ABS';
const flags = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  const key = process.argv[i], value = process.argv[i + 1];
  assert.ok(['--mirket', '--node', '--design-adapter'].includes(key) && value && isAbsolute(value) && !flags.has(key), usage);
  flags.set(key, value);
}
assert.equal(flags.size, 3, usage);
assert.notEqual(process.platform, 'win32', 'This fixture requires native Unix SIGKILL/reap semantics');
for (const path of flags.values()) assert.ok((await stat(path)).isFile(), path);
const release = 'intent-retirement-test-a';
const MAX_FRAME = 1024 * 1024, MAX_STREAM = 4 * MAX_FRAME;
const began = performance.now(), DEADLINE_MS = 360_000;
const owned = new Set();
let timedOut = false;
const watchdog = setTimeout(() => {
  timedOut = true;
  for (const child of owned) child.kill('SIGKILL');
}, DEADLINE_MS);
function deadline() { assert.ok(!timedOut && performance.now() - began < DEADLINE_MS, 'Whole fixture exceeded 360 seconds'); }
const digest = (bytes) => createHash('sha256').update(bytes).digest('hex');
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const temporary = await realpath(await mkdtemp(join(tmpdir(), 'mirket-intent-retirement-')));
const state = join(temporary, 'state'), project = join(temporary, 'project ç');
for (const path of [state, project]) await mkdir(path, { mode: 0o700 });
assert.equal((await stat(state)).mode & 0o777, 0o700);
const source = join(project, 'tokens.json'), destination = join(project, 'never-published.css');
const original = '{"color":{"brand":{"value":"#1463ff"}}}\n';
await writeFile(source, original, { mode: 0o600, flag: 'wx' });
const evidence = { platform: process.platform, arch: process.arch, release, inputs: {}, phases: [], operations: [] };
for (const [name, path] of flags) evidence.inputs[name] = { path, sha256: digest(await readFile(path)) };
function phase(name, extra = {}) {
  const entry = { name, elapsed_ms: Math.round(performance.now() - began), ...extra };
  evidence.phases.push(entry); console.log(JSON.stringify(entry));
}
function launch(args) {
  deadline();
  const child = spawn(flags.get('--mirket'), args, { stdio: ['pipe', 'pipe', 'pipe'] });
  owned.add(child);
  child.closed = new Promise((resolve) => child.once('close', (code, signal) => {
    owned.delete(child); resolve({ code, signal });
  }));
  return child;
}
async function boundedExit(child, milliseconds = 5000) {
  let expired = false;
  const timer = setTimeout(() => { expired = true; child.kill('SIGKILL'); }, milliseconds);
  // A killed direct child must still be reaped. Bound that exceptional wait too.
  let reapTimer;
  try {
    const exit = await Promise.race([child.closed, new Promise((_, reject) => {
      reapTimer = setTimeout(() => reject(new Error('Direct-child reap unconfirmed; retain fixture')), milliseconds + 5000);
    })]);
    assert.equal(expired, false, 'Child exceeded its exit deadline');
    return exit;
  } finally { clearTimeout(timer); clearTimeout(reapTimer); }
}
class Client {
  constructor() {
    this.pending = new Map(); this.nextId = 1; this.buffer = Buffer.alloc(0); this.total = 0;
    this.stderr = ''; this.failure = null; this.stopping = false;
    this.child = launch(['dev', 'serve', '--state-dir', state, '--release-id', release,
      '--node', flags.get('--node'),
      '--design-adapter', flags.get('--design-adapter'), '--design-font', flags.get('--node')]);
    this.child.on('error', (error) => this.fail(error));
    this.child.stdin.on('error', (error) => { if (!this.stopping) this.fail(error); });
    this.child.stderr.on('data', (bytes) => {
      if (Buffer.byteLength(this.stderr) + bytes.length > 65536) this.fail(new Error('MCP stderr budget exceeded'));
      else this.stderr += bytes.toString('utf8');
    });
    this.child.closed.then((exit) => {
      if (!this.stopping) this.fail(new Error(`Unexpected MCP exit ${JSON.stringify(exit)}: ${this.stderr}`));
    });
    this.child.stdout.on('data', (chunk) => {
      try {
        this.total += chunk.length; assert.ok(this.total <= MAX_STREAM, 'MCP stream budget exceeded');
        this.buffer = Buffer.concat([this.buffer, chunk]);
        for (;;) {
          const end = this.buffer.indexOf(10); if (end < 0) break;
          assert.ok(end + 1 <= MAX_FRAME, 'MCP frame budget exceeded');
          const message = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(this.buffer.subarray(0, end)));
          this.buffer = this.buffer.subarray(end + 1);
          assert.equal(message.jsonrpc, '2.0');
          if (message.id === undefined) continue;
          const waiter = this.pending.get(message.id); assert.ok(waiter, 'Unexpected RPC response');
          this.pending.delete(message.id); clearTimeout(waiter.timer);
          if (message.error) waiter.reject(new Error(JSON.stringify(message.error))); else waiter.resolve(message.result);
        }
        assert.ok(this.buffer.length < MAX_FRAME, 'Unterminated MCP frame budget exceeded');
      } catch (error) { this.fail(error); }
    });
  }
  fail(error) {
    this.failure ??= error;
    for (const item of this.pending.values()) { clearTimeout(item.timer); item.reject(error); }
    this.pending.clear(); this.child.kill('SIGKILL');
  }
  send(message) {
    deadline(); if (this.failure) throw this.failure;
    const frame = JSON.stringify({ jsonrpc: '2.0', ...message }) + '\n';
    assert.ok(Buffer.byteLength(frame) <= MAX_FRAME); this.child.stdin.write(frame);
  }
  rpc(method, params) {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => this.fail(new Error(`${method} exceeded 5 seconds`)), 5000);
      this.pending.set(id, { resolve, reject, timer });
      try { this.send({ id, method, params }); } catch (error) { this.fail(error); }
    });
  }
  async initialize() {
    const result = await this.rpc('initialize', { protocolVersion: '2025-11-25', capabilities: {},
      clientInfo: { name: 'intent-retirement-smoke', version: '1' } });
    assert.equal(result.protocolVersion, '2025-11-25');
    this.send({ method: 'notifications/initialized', params: {} }); return this;
  }
  call(name, args) { return this.rpc('tools/call', { name, arguments: args }); }
  async stop(abrupt = false) {
    this.stopping = true;
    if (owned.has(this.child)) {
      if (abrupt) this.child.kill('SIGKILL'); else this.child.stdin.end();
    }
    const exit = await boundedExit(this.child);
    assert.deepEqual(exit, abrupt ? { code: null, signal: 'SIGKILL' } : { code: 0, signal: null }, this.stderr);
    assert.equal(this.buffer.length, 0, 'Trailing incomplete MCP frame');
    if (this.failure) throw this.failure;
  }
}
function body(result) {
  if (result.structuredContent !== undefined) return result.structuredContent;
  return JSON.parse(result.content.find((item) => item.type === 'text').text);
}
async function success(client, name, args) {
  const result = await client.call(name, args);
  assert.notEqual(result.isError, true, `${name}: ${JSON.stringify(result)}`); return body(result);
}
async function connect() { return new Client().initialize(); }
async function absent(path) { await assert.rejects(stat(path), { code: 'ENOENT' }, path); }
async function noEffects() {
  assert.equal(await readFile(source, 'utf8'), original);
  assert.deepEqual(await readdir(project), ['tokens.json']);
  await absent(destination); await absent(join(state, 'memory')); await absent(join(state, 'memory-lifecycle')); await absent(join(state, 'jobs'));
  for (const slot of [0, 1]) await absent(join(state, `heavy-slot-${slot}.unresolved`));
}
async function receipt(item) {
  const bytes = await readFile(join(state, 'operations', `${item.operation_id}.json`));
  const envelope = JSON.parse(bytes);
  assert.equal(envelope.schema, 2); assert.match(envelope.issued_payload_sha256, /^[0-9a-f]{64}$/);
  return { bytes, envelope, payload: JSON.parse(envelope.payload) };
}
async function unchanged(items) {
  for (const item of items) assert.deepEqual((await receipt(item)).bytes, item.original, `${item.label}: receipt changed during refusal`);
  await noEffects();
}
async function retire(item) {
  const args = ['dev', 'intent-retire', '--state-dir', state, '--release-id', release,
    '--kind', item.kind, '--operation-id', item.operation_id,
    ...(item.shared ? ['--shared-preferences'] : ['--project-root', project])];
  const child = launch(args); let stdout = '', stderr = '', failure;
  child.on('error', (error) => { failure = error; });
  child.stdin.on('error', (error) => { failure = error; }); child.stdin.end();
  for (const [stream, name] of [[child.stdout, 'stdout'], [child.stderr, 'stderr']]) stream.on('data', (bytes) => {
    if (Buffer.byteLength(name === 'stdout' ? stdout : stderr) + bytes.length > 65536) {
      failure = new Error(`CLI ${name} exceeded 64 KiB`); child.kill('SIGKILL');
    } else if (name === 'stdout') stdout += bytes.toString('utf8'); else stderr += bytes.toString('utf8');
  });
  const exit = await boundedExit(child); if (failure) throw failure;
  return { ...exit, stdout, stderr };
}
function refusal(result, pattern) {
  assert.equal(result.signal, null, JSON.stringify(result)); assert.notEqual(result.code, 0, JSON.stringify(result));
  assert.match(result.stderr, pattern); assert.equal(result.stdout, '', 'Refusal must not emit success JSON');
}
const scope = { kind: 'project', project_root: project }, shared = { kind: 'shared_preferences' };
const memoryArgs = (selection) => ({ expected_release_id: release, scope: selection });
const record = (kind) => ({ kind, basis: 'user_stated', content: 'Synthetic intent that must never become canonical memory.',
  sources: [{ source_id: 'fixture:intent-retirement', revision: 'fixture-1', locator: 'fixture://never-committed', captured_at: 123 }],
  status: 'current', critical: false, applicability: { task_key: null, checkout_id: null, operation: null },
  valid_from: null, valid_until: null });
let passed = false;
try {
  phase('prepare');
  const client = await connect();
  const design = await success(client, 'design_prepare', { expected_release_id: release, project_root: project,
    source_path: source, expected_revision: `sha256:${digest(Buffer.from(original))}`,
    destination_path: destination, operation: { kind: 'tokens_compile', format: 'css' } });
  const items = [{ ...design, label: 'design', kind: 'artifact', shared: false }];
  for (const [label, selection, kind] of [['project-memory', scope, 'fact'], ['shared-memory', shared, 'preference']]) {
    const prepared = await success(client, 'memory_prepare', { ...memoryArgs(selection), mutation: { kind: 'create', record: record(kind), supersedes: null } });
    items.push({ ...prepared, label, kind: 'memory', shared: selection === shared });
  }
  for (const item of items) {
    assert.equal(item.status, 'prepared'); assert.match(item.operation_id, /^op-[a-zA-Z0-9-]+$/);
    assert.ok(Number.isSafeInteger(item.expires_at));
    const saved = await receipt(item); item.original = saved.bytes;
    assert.equal(saved.envelope.retention.state, 'protected');
    assert.equal(saved.envelope.issued_payload_sha256, digest(Buffer.from(saved.envelope.payload)));
    assert.equal(saved.payload.expires_at, item.expires_at);
    evidence.operations.push({ label: item.label, operation_id: item.operation_id, expires_at: item.expires_at,
      issued_payload_sha256: saved.envelope.issued_payload_sha256, original_receipt_sha256: digest(saved.bytes) });
    refusal(await retire(item), /runtime maintenance lock is busy/);
  }
  await unchanged(items); phase('live maintenance refused');
  await client.stop(true); phase('SIGKILL owner reaped');
  for (const item of items) refusal(await retire(item), /only an expired preparation may be retired/);
  await unchanged(items); phase('unexpired retirement refused');
  const expiry = Math.max(...items.map((item) => item.expires_at));
  assert.ok(expiry * 1000 - Date.now() <= 301_000, 'Issued expiry exceeds the five-minute contract');
  let nextProgress = performance.now() + 60_000;
  for (let polls = 0; Math.floor(Date.now() / 1000) <= expiry; polls++) {
    deadline(); assert.ok(polls < 330, 'Expiry wait poll limit exceeded');
    await sleep(Math.min(1000, Math.max(1, (expiry + 1) * 1000 - Date.now())));
    if (performance.now() >= nextProgress) { phase('waiting for actual expiry', { remaining_seconds: Math.max(0, expiry + 1 - Math.floor(Date.now() / 1000)) }); nextProgress += 60_000; }
  }
  phase('actual expiry reached');
  await unchanged(items);
  for (const item of items) {
    const result = await retire(item); assert.deepEqual({ code: result.code, signal: result.signal }, { code: 0, signal: null }, JSON.stringify(result));
    const report = JSON.parse(result.stdout); assert.equal(report.operation_id, item.operation_id);
    assert.equal(report.release_id, release);
    assert.equal(report.status, item.kind === 'artifact' ? 'cancelled' : 'expired_not_started');
    if (item.kind === 'artifact') { assert.equal(report.effect, 'not_attempted'); assert.equal(report.scratch_cleanup, 'not_started'); }
    const saved = await receipt(item); assert.equal(saved.envelope.retention.state, 'retirable');
    assert.equal(saved.envelope.issued_payload_sha256, JSON.parse(item.original).issued_payload_sha256);
    // Cross a whole wall-clock second so an accidental completed_at reset is observable.
    await sleep(1100); deadline();
    const again = await retire(item); assert.equal(again.code, 0, JSON.stringify(again)); assert.equal(again.signal, null);
    assert.deepEqual(JSON.parse(again.stdout), report);
    assert.deepEqual((await receipt(item)).bytes, saved.bytes, 'Idempotent retirement must not reset completion age or receipt');
    item.retired = saved.bytes;
    evidence.operations.find((entry) => entry.operation_id === item.operation_id).retired_receipt_sha256 = digest(saved.bytes);
  }
  await noEffects(); phase('expired preparations retired idempotently');
  const reconnected = await connect();
  for (const item of items) {
    const args = item.kind === 'artifact' ? { expected_release_id: release, project_root: project, operation_id: item.operation_id }
      : { ...memoryArgs(item.shared ? shared : scope), operation_id: item.operation_id };
    const status = await success(reconnected, item.kind === 'artifact' ? 'job_status' : 'memory_status', args);
    assert.equal(status.status, item.kind === 'artifact' ? 'cancelled' : 'expired_not_started', JSON.stringify(status));
    if (item.kind === 'memory') { assert.equal(status.intent_available, false); assert.equal(status.mutation, null); assert.equal(status.replay_allowed, false); }
    const attempted = await reconnected.call(item.kind === 'artifact' ? 'design_start' : 'memory_commit', args);
    if (item.kind === 'artifact') {
      assert.notEqual(attempted.isError, true, JSON.stringify(attempted));
      assert.equal(body(attempted).status, 'cancelled'); assert.equal(body(attempted).effect, 'not_attempted');
    } else { assert.equal(attempted.isError, true, JSON.stringify(attempted)); assert.match(JSON.stringify(attempted), /another connection/); }
    assert.deepEqual((await receipt(item)).bytes, item.retired, 'Reconnect must not mutate or replay retired intent');
  }
  await reconnected.stop(); await noEffects(); phase('reconnect cannot execute retired IDs; no source/output/canonical effects');
  passed = true;
} catch (error) { evidence.failure = String(error.stack ?? error); throw error; }
finally {
  clearTimeout(watchdog);
  // No recursive cleanup until every owned direct process has definitely exited.
  for (const child of owned) child.kill('SIGKILL');
  const cleanup = await Promise.allSettled([...owned].map((child) => boundedExit(child)));
  evidence.passed = passed; evidence.elapsed_ms = Math.round(performance.now() - began);
  evidence.cleanup_confirmed = owned.size === 0 && cleanup.every((entry) => entry.status === 'fulfilled');
  await writeFile(join(temporary, 'evidence.json'), JSON.stringify(evidence, null, 2) + '\n', { mode: 0o600 });
  console.log(JSON.stringify({ evidence: join(temporary, 'evidence.json'), passed, cleanup_confirmed: evidence.cleanup_confirmed }));
  // Retain small receipts and diagnostics on success too: actual-clock evidence is
  // costly to recreate. The fixture owns this one disposable root; no host config.
  assert.equal(evidence.cleanup_confirmed, true, 'Child cleanup uncertain; preserve disposable root');
}
