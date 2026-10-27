// Disposable two-release qualification; no downloads or host/client settings.
import assert from "node:assert/strict";
import { spawn, execFile } from "node:child_process";
import { createHash } from "node:crypto";
import { appendFile, chmod, copyFile, mkdir, mkdtemp, readFile, readdir, realpath, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";
import { promisify } from "node:util";

const run = promisify(execFile);
const usage = "Usage: node tests/release-activation.mjs --mirket ABS --next-mirket ABS --node ABS --node-license ABS";
const paths = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  const key = process.argv[i], value = process.argv[i + 1];
  assert.ok(["--mirket", "--next-mirket", "--node", "--node-license"].includes(key) && value && !paths.has(key), usage);
  assert.ok(isAbsolute(value), usage);
  assert.ok((await stat(value)).isFile(), `Expected file: ${value}`);
  paths.set(key, value);
}
assert.equal(paths.size, 4, usage);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const initialCoreSha = hash(await readFile(paths.get("--mirket")));
const nextCoreSha = hash(await readFile(paths.get("--next-mirket")));
assert.notEqual(initialCoreSha, nextCoreSha, "--next-mirket must be a second independently compiled binary with different saved bytes");
assert.equal(process.platform, "darwin", "This four-file fixture qualifies the macOS layout only");
assert.equal(process.arch, "arm64", "This fixture currently qualifies macOS ARM64 only");

async function cli(args) {
  const result = await run(paths.get("--mirket"), args, { timeout: 30000, maxBuffer: 1024 * 1024 });
  assert.equal(result.stderr, "", `Unexpected CLI diagnostic: ${result.stderr}`);
  return JSON.parse(result.stdout);
}
async function cliRejects(args, reason) {
  let caught;
  try { await cli(args); } catch (error) { caught = error; }
  assert.ok(caught && typeof caught.code === "number" && caught.code !== 0, "Expected CLI process refusal, not success/timeout/assertion");
  assert.match(caught.stderr ?? String(caught), reason);
}

class Connection {
  constructor(home) {
    this.nextId = 1; this.pending = new Map(); this.buffer = Buffer.alloc(0);
    this.bytes = 0; this.stderr = ""; this.closed = false; this.failure = null;
    this.stopping = false; this.forcedKill = false; this.cleanupConfirmed = false;
    this.child = spawn(paths.get("--mirket"), ["serve", "--home", home], { stdio: ["pipe", "pipe", "pipe"] });
    this.launchPid = this.child.pid;
    this.exited = new Promise((resolve) => this.child.on("close", (code, signal) => {
      this.closed = true;
      if (!this.stopping || code !== 0 || signal || this.pending.size) {
        this.fail(new Error(`Server exited (${code ?? signal}): ${this.stderr}`));
      }
      this.cleanupConfirmed = this.stopping && code === 0 && !signal && !this.forcedKill && !this.failure;
      resolve({ code, signal });
    }));
    this.child.on("error", (error) => this.fail(error));
    this.child.stdin.on("error", (error) => this.fail(error));
    this.child.stderr.setEncoding("utf8").on("data", (chunk) => {
      this.stderr += chunk;
      if (Buffer.byteLength(this.stderr) > 65536) this.fail(new Error("Server stderr exceeded limit"));
    });
    this.child.stdout.on("data", (chunk) => {
      try {
        this.bytes += chunk.length;
        assert.ok(this.bytes <= 4 * 1024 * 1024, "Server stdout exceeded limit");
        this.buffer = Buffer.concat([this.buffer, chunk]);
        for (;;) {
          const end = this.buffer.indexOf(10);
          if (end < 0) break;
          assert.ok(end < 1024 * 1024, "MCP frame exceeded limit");
          const response = JSON.parse(new TextDecoder("utf8", { fatal: true }).decode(this.buffer.subarray(0, end)));
          this.buffer = this.buffer.subarray(end + 1);
          assert.equal(response.jsonrpc, "2.0");
          if (response.id === undefined) continue;
          const waiter = this.pending.get(response.id);
          assert.ok(waiter, "Unexpected MCP response ID");
          this.pending.delete(response.id); clearTimeout(waiter.timer);
          if (response.error) waiter.reject(new Error(JSON.stringify(response.error)));
          else waiter.resolve(response.result);
        }
        assert.ok(this.buffer.length <= 1024 * 1024, "Unterminated MCP frame exceeded limit");
      } catch (error) { this.fail(error); }
    });
  }
  fail(error) {
    this.failure ??= error;
    for (const waiter of this.pending.values()) { clearTimeout(waiter.timer); waiter.reject(error); }
    this.pending.clear();
    if (!this.closed) { this.forcedKill = true; this.child.kill("SIGKILL"); }
  }
  send(message) {
    if (this.failure) throw this.failure;
    this.child.stdin.write(JSON.stringify({ jsonrpc: "2.0", ...message }) + "\n");
  }
  rpc(method, params) {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => this.fail(new Error(`${method} exceeded five seconds`)), 5000);
      this.pending.set(id, { resolve, reject, timer });
      try { this.send({ id, method, params }); }
      catch (error) { clearTimeout(timer); this.pending.delete(id); reject(error); }
    });
  }
  async initialize() {
    const hello = await this.rpc("initialize", { protocolVersion: "2025-11-25", capabilities: {},
      clientInfo: { name: "mirket-release-qualification", version: "0.0.1" } });
    assert.equal(hello.protocolVersion, "2025-11-25");
    this.send({ method: "notifications/initialized", params: {} });
    return this;
  }
  call(name, args = {}) { return this.rpc("tools/call", { name, arguments: args }); }
  async stop() {
    this.stopping = true;
    if (!this.closed) this.child.stdin.end();
    const timer = setTimeout(() => {
      if (!this.closed) { this.forcedKill = true; this.child.kill("SIGKILL"); }
    }, 3000);
    const outcome = await this.exited; clearTimeout(timer);
    assert.ok(this.cleanupConfirmed && outcome.code === 0,
      `Server cleanup unconfirmed: ${JSON.stringify(outcome)} ${this.stderr} ${this.failure ?? ""}`);
  }
}
function content(result) {
  if (result.structuredContent !== undefined) return result.structuredContent;
  return JSON.parse(result.content.find((item) => item.type === "text").text);
}
async function tool(client, name, args) {
  const result = await client.call(name, args);
  assert.notEqual(result.isError, true, `${name}: ${JSON.stringify(result)}`);
  return content(result);
}
async function toolRejects(client, name, args, reason) {
  const result = await client.call(name, args);
  assert.equal(result.isError, true, `${name} unexpectedly succeeded`);
  assert.match(JSON.stringify(result), reason);
}
async function snapshot(root, prefix = "") {
  const result = [];
  for (const entry of (await readdir(join(root, prefix), { withFileTypes: true })).sort((a, b) => a.name.localeCompare(b.name))) {
    const relative = join(prefix, entry.name);
    if (entry.isDirectory()) { result.push([relative, "directory"]); result.push(...await snapshot(root, relative)); }
    else {
      assert.ok(entry.isFile());
      result.push([relative, createHash("sha256").update(await readFile(join(root, relative))).digest("hex")]);
    }
  }
  return result;
}
const jsonFile = async (path) => JSON.parse(await readFile(path, "utf8"));
const absent = async (path) => assert.rejects(stat(path), { code: "ENOENT" });
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const temporary = await mkdtemp(join(tmpdir(), "mirket-release-qualification-"));
const home = join(temporary, "home"), project = join(temporary, "project");
const clients = [];
let succeeded = false;
let qualificationFailure = null;
const connect = async () => { const client = new Connection(home); clients.push(client); return client.initialize(); };
const activePath = join(home, "active.json"), stagedPath = join(home, "staged.json");
const activate = () => cli(["dev", "activate", "--home", home]);
const stage = (source) => cli(["dev", "stage", "--home", home, "--source", source]);
async function packageAt(name, coreKey) {
  const root = join(temporary, name);
  await mkdir(root, { mode: 0o700 });
  for (const directory of ["bin", "runtime", "runtime/node", "runtime/node/bin"]) {
    await mkdir(join(root, directory), { mode: 0o700 });
  }
  const inventory = [[coreKey, "bin/mirket"], ["--node", "runtime/node/bin/node"],
    ["--node-license", "runtime/node/LICENSE"]];
  for (const [key, relative] of inventory) {
    await copyFile(paths.get(key), join(root, relative));
    await chmod(join(root, relative), key === coreKey || key === "--node" ? 0o700 : 0o600);
  }
  assert.equal(hash(await readFile(join(root, "bin/mirket"))), coreKey === "--mirket" ? initialCoreSha : nextCoreSha);
  const manifest = await cli(["dev", "manifest", "--directory", root]);
  assert.equal(manifest.status, "manifest_created");
  assert.match(manifest.release_id, /^sha256:[a-f0-9]{64}$/);
  return { root, id: manifest.release_id };
}

async function usefulMemory(client, releaseId, content) {
  const scope = { kind: "project", project_root: project };
  const record = { kind: "fact", basis: "observed", content,
    sources: [{ source_id: "fixture:release-activation", revision: "fixture-1",
      locator: "fixture://release-activation", captured_at: 123 }],
    status: "current", critical: false,
    applicability: { task_key: null, checkout_id: null, operation: null },
    valid_from: null, valid_until: null };
  const prepared = await tool(client, "memory_prepare", { expected_release_id: releaseId,
    scope, mutation: { kind: "create", record, supersedes: null } });
  const operation = { expected_release_id: releaseId, scope, operation_id: prepared.operation_id };
  const completed = await tool(client, "memory_commit", operation);
  assert.equal(completed.status, "committed");
  const saved = await tool(client, "memory_read", { expected_release_id: releaseId, scope,
    record_id: completed.record.record_id, revision: completed.record.revision });
  assert.equal(saved.record.data.content, content);
}

try {
  const unknownHome = join(temporary, "unknown-home");
  await mkdir(unknownHome, { mode: 0o700 });
  await writeFile(join(unknownHome, "existing-settings.txt"), "Unrelated user settings stay intact.\n");
  const unknownBefore = await snapshot(unknownHome);
  await cliRejects(["dev", "home-init", "--home", unknownHome], /empty|existing|unrecognized/i);
  assert.deepEqual(await snapshot(unknownHome), unknownBefore);
  console.log("PASS existing unrecognized home is preserved");

  const a = await packageAt("package-a", "--mirket");
  const b = await packageAt("package-b", "--next-mirket");
  assert.notEqual(a.id, b.id, "Changed core bytes must produce a distinct release identity");
  await cli(["dev", "home-init", "--home", home]);
  assert.equal((await stat(home)).mode & 0o777, 0o700);
  assert.equal((await stage(a.root)).release_id, a.id);
  assert.equal((await activate()).status, "active");
  assert.equal((await jsonFile(activePath)).release_id, a.id);
  await absent(stagedPath);
  console.log(`PASS release A manifested, staged and activated (${a.id})`);

  await mkdir(project, { mode: 0o700 });
  let client = await connect();
  assert.equal((await tool(client, "runtime_status")).release_id, a.id);
  await usefulMemory(client, a.id, "Release fixture: active A fact.");
  console.log("PASS stable entry serves active A and commits scoped memory");

  const activeA = await readFile(activePath);
  assert.equal((await stage(b.root)).status, "staged");
  const busy = await activate();
  assert.equal(busy.status, "staged"); assert.equal(busy.release_id, b.id);
  assert.match(busy.reason, /busy/i);
  assert.deepEqual(await readFile(activePath), activeA);
  assert.equal((await jsonFile(stagedPath)).release_id, b.id);
  await client.stop();
  assert.deepEqual(await readFile(activePath), activeA);
  client = await connect();
  assert.equal((await tool(client, "runtime_status")).release_id, a.id, "Ordinary reconnect must not activate staged B");
  await client.stop();
  assert.deepEqual(await readFile(activePath), activeA);
  console.log("PASS live session blocks activation; disconnect/reconnect does not auto-activate B");

  const switched = await activate();
  assert.equal(switched.status, "active"); assert.equal(switched.release_id, b.id);
  assert.deepEqual(await jsonFile(activePath), { schema_version: 1, release_id: b.id, previous_release_id: a.id });
  await absent(stagedPath);
  client = await connect();
  assert.equal((await tool(client, "runtime_status")).release_id, b.id);
  const installedBCore = join(home, "releases", `r-${b.id.slice("sha256:".length)}`, "bin", "mirket");
  assert.equal(client.child.pid, client.launchPid, "Handoff must retain the spawned process identity");
  assert.ok(Number.isSafeInteger(client.launchPid) && client.launchPid > 0);
  const executable = await run("/bin/ps", ["-ww", "-p", String(client.launchPid), "-o", "comm="],
    { timeout: 5000, maxBuffer: 16384 });
  assert.equal(executable.stderr, "");
  assert.equal(await realpath(executable.stdout.trim()), await realpath(installedBCore),
    "Stable A entry must actually exec the installed B core, not only report its release ID");
  assert.equal(hash(await readFile(installedBCore)), nextCoreSha);
  console.log(`PASS live PID ${client.launchPid} executes installed B core ${executable.stdout.trim()}`);
  await usefulMemory(client, b.id, "Release fixture: active B fact.");
  console.log("PASS unchanged stable entry hands off to independently compiled B for a confirmed memory operation");
  const beforeStale = await snapshot(join(home, "state"));
  const beforeProject = await snapshot(project);
  await toolRejects(client, "memory_prepare", { expected_release_id: a.id,
    scope: { kind: "project", project_root: project },
    mutation: { kind: "create", record: { kind: "fact", basis: "observed", content: "stale",
      sources: [{ source_id: "fixture:release-activation", revision: "fixture-1",
        locator: "fixture://release-activation", captured_at: 123 }],
      status: "current", critical: false,
      applicability: { task_key: null, checkout_id: null, operation: null },
      valid_from: null, valid_until: null }, supersedes: null } }, /release/i);
  assert.deepEqual(await snapshot(join(home, "state")), beforeStale);
  assert.deepEqual(await snapshot(project), beforeProject);
  await client.stop();
  console.log("PASS explicit B activation preserves A rollback identity and rejects cached A intent before effects");

  // A handoff child must bind both active release and its own core bytes. It may
  // not redirect recursively when launched directly with a claimed identity.
  const beforeBadHandoff = await snapshot(home);
  const beforeBadProject = await snapshot(project);
  await cliRejects(["serve", "--home", home, "--expected-release-id", b.id], /core|executable|binary|handoff/i);
  await cliRejects(["serve", "--home", home, "--expected-release-id", a.id], /release|handoff/i);
  assert.deepEqual(await snapshot(home), beforeBadHandoff);
  assert.deepEqual(await snapshot(project), beforeBadProject);
  console.log("PASS direct wrong-core and stale-release handoff claims refuse without recursive handoff or effects");

  // Simulate saved activation crash states; these writes affect only this fixture.
  await stage(a.root);
  const activeB = await readFile(activePath);
  const proposedA = { schema_version: 1, release_id: a.id, previous_release_id: b.id };
  const pendingCommit = join(home, "active.json.next");
  const unrelatedCommit = { schema_version: 1, release_id: b.id, previous_release_id: a.id };
  await writeFile(pendingCommit, JSON.stringify(unrelatedCommit), { flag: "wx", mode: 0o600 });
  await cliRejects(["dev", "activate", "--home", home], /interrupted|recovery/i);
  assert.deepEqual(await readFile(activePath), activeB);
  assert.deepEqual(await jsonFile(pendingCommit), unrelatedCommit);
  client = await connect();
  assert.equal((await tool(client, "runtime_status")).release_id, b.id);
  await client.stop();
  await rm(pendingCommit); // Remove exactly the test-created incomplete record.
  await writeFile(pendingCommit, JSON.stringify(proposedA), { flag: "wx", mode: 0o600 });
  const resumedCommit = await activate();
  assert.equal(resumedCommit.status, "active"); assert.equal(resumedCommit.release_id, a.id);
  assert.deepEqual(await jsonFile(activePath), proposedA);
  await absent(pendingCommit); await absent(stagedPath);
  await stage(a.root); // Recreate the saved state after pointer commit but before staged cleanup.
  await writeFile(activePath, JSON.stringify(proposedA)); // Commit happened; staged cleanup did not.
  const retried = await activate();
  assert.equal(retried.status, "active"); assert.equal(retried.release_id, a.id);
  assert.deepEqual(await jsonFile(activePath), proposedA);
  await absent(stagedPath);
  console.log("PASS differing pre-commit residue is preserved/refused; exact-match recovery and post-commit retry preserve predecessor");

  await stage(b.root);
  const activeBeforeCorruption = await readFile(activePath);
  const marker = join(home, "state", "heavy-slot-0.unresolved");
  await writeFile(marker, "mirket-unresolved-v1\nfixture-unknown\n", { flag: "wx", mode: 0o600 });
  const unresolved = await activate();
  assert.equal(unresolved.status, "staged"); assert.match(unresolved.reason, /unresolved/i);
  assert.deepEqual(await readFile(activePath), activeBeforeCorruption);
  await rm(marker); // Test-created marker; not a product recovery procedure.
  const stagedLicense = join(home, "releases", `r-${b.id.slice("sha256:".length)}`, "runtime", "node", "LICENSE");
  await appendFile(stagedLicense, "\nDeliberate corruption after manifest verification.\n");
  await cliRejects(["dev", "activate", "--home", home], /manifest|bytes|identity|changed/i);
  assert.deepEqual(await readFile(activePath), activeBeforeCorruption);
  assert.equal((await jsonFile(stagedPath)).release_id, b.id);
  console.log("PASS unresolved ownership and corrupted staged payload both preserve the active release");
  console.log(`PASS changed-core qualification: A=${initialCoreSha} B=${nextCoreSha}; desktop activation remains a separate gate`);
  succeeded = true;
} catch (error) {
  qualificationFailure = String(error.stack ?? error).slice(0, 8192);
  throw error;
} finally {
  const cleanup = await Promise.allSettled(clients.map((client) => client.stop()));
  const allClean = cleanup.every((result) => result.status === "fulfilled") && clients.every((client) => client.cleanupConfirmed);
  if (succeeded && allClean) {
    await rm(temporary, { recursive: true });
  } else {
    console.error(`RETAINED release qualification evidence: ${temporary}`);
    await writeFile(join(temporary, "qualification-cleanup.json"), JSON.stringify({
      succeeded, failure: qualificationFailure, cleanup_confirmed: allClean, initial_core_sha256: initialCoreSha, next_core_sha256: nextCoreSha,
      clients: clients.map((client) => ({ pid: client.child.pid ?? null, closed: client.closed,
        forced_kill: client.forcedKill, cleanup_confirmed: client.cleanupConfirmed,
        failure: client.failure ? String(client.failure).slice(0, 2048) : null, stderr: client.stderr.slice(0, 65536) })),
    }, null, 2) + "\n", { flag: "wx", mode: 0o600 });
    if (succeeded) throw new Error("Qualification checks completed but child cleanup is unconfirmed; evidence retained");
  }
}
