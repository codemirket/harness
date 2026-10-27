// Canonical memory through bounded MCP stdio; optional explicitly configured runtime journey.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtemp, mkdir, readFile, readdir, realpath, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

const usage = "Usage: node tests/memory-smoke.mjs --mirket /absolute/mirket --node /absolute/node [--embedding-package /absolute/verified/package]";
const flags = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  const flag = process.argv[i], value = process.argv[i + 1];
  assert.ok(["--mirket", "--node", "--embedding-package"].includes(flag) && value && !flags.has(flag), usage);
  assert.ok(isAbsolute(value), usage);
  flags.set(flag, value);
}
assert.ok(flags.size === 2 || flags.size === 3, usage);
for (const required of ["--mirket", "--node"]) {
  assert.ok(flags.has(required), usage);
  assert.ok((await stat(flags.get(required))).isFile(), `Expected file: ${flags.get(required)}`);
}
if (flags.has("--embedding-package")) assert.ok((await stat(flags.get("--embedding-package"))).isDirectory(), "Expected package directory");

const MAX_FRAME = 1024 * 1024;
const MAX_STREAM = 8 * 1024 * 1024;
const RPC_TIMEOUT_MS = 5000;
const STATUS_TIMEOUT_MS = 5000;
const RELEASE_A = "m2-test-a", RELEASE_B = "m2-test-b";
const REQUIRED_TOOLS = ["runtime_status", "memory_prepare", "memory_commit", "memory_status", "memory_search", "memory_list", "memory_read", "memory_history", "memory_forget_prepare", "memory_forget_commit", "memory_forget_reconcile", "memory_index_refresh"];

class Client {
  constructor(stateDir, releaseId, options = {}) {
    this.rpcTimeoutMs = options.rpcTimeoutMs ?? RPC_TIMEOUT_MS;
    this.releaseId = releaseId;
    this.pending = new Map();
    this.ignored = new Set();
    this.lostResponseDiagnostics = [];
    this.nextId = 1;
    this.buffer = Buffer.alloc(0);
    this.totalBytes = 0;
    this.stderr = "";
    this.failure = null;
    this.closed = false;
    const command = ["dev", "serve", "--state-dir", stateDir,
      "--release-id", releaseId, "--node", flags.get("--node")];
    if (options.backupDir) command.push("--backup-dir", options.backupDir);
    if (options.embeddingPackage) command.push("--embedding-package", options.embeddingPackage);
    this.child = spawn(flags.get("--mirket"), command, { stdio: ["pipe", "pipe", "pipe"] });
    this.exited = new Promise((resolve) => this.child.on("close", (code, signal) => {
      this.closed = true;
      this.fail(new Error(`Server exited (${code ?? signal}); stderr: ${this.stderr}`));
      resolve({ code, signal });
    }));
    this.child.on("error", (error) => this.fail(error));
    this.child.stdin.on("error", (error) => this.fail(error));
    this.child.stderr.setEncoding("utf8").on("data", (chunk) => {
      this.stderr += chunk;
      if (Buffer.byteLength(this.stderr) > 512 * 1024) this.fail(new Error("Server stderr exceeded fixture budget"));
    });
    this.child.stdout.on("data", (chunk) => {
      try {
        this.totalBytes += chunk.length;
        if (this.totalBytes > MAX_STREAM) throw new Error("Server stdout exceeded fixture budget");
        this.buffer = Buffer.concat([this.buffer, chunk]);
        for (;;) {
          const newline = this.buffer.indexOf(10);
          if (newline < 0) break;
          if (newline + 1 > MAX_FRAME) throw new Error("Server frame exceeded fixture budget");
          const frame = this.buffer.subarray(0, newline);
          this.buffer = this.buffer.subarray(newline + 1);
          const message = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(frame));
          assert.equal(message.jsonrpc, "2.0");
          if (message.id === undefined) continue; // Server notifications do not complete an RPC.
          if (this.ignored.delete(message.id)) {
            // Retained only to explain fixture failure, never used as commit evidence.
            this.lostResponseDiagnostics.push(message);
            continue;
          }
          const waiter = this.pending.get(message.id);
          if (!waiter) throw new Error(`Unexpected server response ID: ${message.id}`);
          this.pending.delete(message.id);
          clearTimeout(waiter.timer);
          if (message.error) waiter.reject(new Error(JSON.stringify(message.error)));
          else waiter.resolve(message.result);
        }
        if (this.buffer.length > MAX_FRAME) throw new Error("Unterminated server frame exceeded fixture budget");
      } catch (error) { this.fail(error); }
    });
  }

  fail(error) {
    if (!this.failure) this.failure = error;
    for (const { reject, timer } of this.pending.values()) { clearTimeout(timer); reject(error); }
    this.pending.clear();
    if (!this.closed) this.child.kill("SIGKILL");
  }

  send(message) {
    if (this.failure) throw this.failure;
    const frame = JSON.stringify({ jsonrpc: "2.0", ...message }) + "\n";
    assert.ok(Buffer.byteLength(frame) <= MAX_FRAME);
    this.child.stdin.write(frame);
  }

  rpc(method, params) {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        const error = new Error(`${method} exceeded ${this.rpcTimeoutMs}ms`);
        reject(error);
        this.fail(error);
      }, this.rpcTimeoutMs);
      this.pending.set(id, { resolve, reject, timer });
      try { this.send({ id, method, params }); }
      catch (error) { clearTimeout(timer); this.pending.delete(id); reject(error); }
    });
  }

  async initialize() {
    const result = await this.rpc("initialize", { protocolVersion: "2025-11-25", capabilities: {},
      clientInfo: { name: "mirket-disposable-memory-smoke", version: "0.0.1" } });
    assert.equal(result.protocolVersion, "2025-11-25", "Protocol negotiation changed");
    assert.ok(result.serverInfo?.name, "Missing server identity");
    this.send({ method: "notifications/initialized", params: {} });
    return this;
  }

  call(name, args = {}) { return this.rpc("tools/call", { name, arguments: args }); }

  loseCallResponse(name, args) {
    const id = this.nextId++;
    this.ignored.add(id);
    this.send({ id, method: "tools/call", params: { name, arguments: args } });
  }

  async stop(abrupt = false) {
    if (this.closed) return;
    if (abrupt) this.child.kill("SIGKILL");
    else this.child.stdin.end();
    // The runtime owns cleanup for up to35s. A forced fixture kill must never
    // masquerade as a successful graceful shutdown of a resident model.
    let forced = false;
    const timer = setTimeout(() => { forced = true; this.child.kill("SIGKILL"); }, 40000);
    const exit = await this.exited;
    clearTimeout(timer);
    if (!abrupt) {
      assert.equal(forced, false, "Server exceeded its cleanup deadline");
      assert.equal(exit.signal, null, `Graceful shutdown was signalled: ${exit.signal}`);
      assert.equal(exit.code, 0, `Graceful shutdown failed: ${this.stderr}`);
    }
  }
}

function body(result) {
  if (result?.structuredContent !== undefined) return result.structuredContent;
  const text = result?.content?.find((part) => part.type === "text")?.text;
  assert.equal(typeof text, "string", "Expected structuredContent or JSON text content");
  return JSON.parse(text);
}

async function success(client, name, args) {
  const result = await client.call(name, args);
  assert.notEqual(result.isError, true, `${name}: ${JSON.stringify(result)}`);
  return body(result);
}

async function rejected(client, name, args, reason) {
  let result;
  try { result = await client.call(name, args); }
  catch (error) { assert.match(String(error), reason); return; }
  assert.equal(result.isError, true, `${name} unexpectedly succeeded: ${JSON.stringify(result)}`);
  assert.match(JSON.stringify(result), reason);
}

async function treeSnapshot(root, prefix = "") {
  const entries = [];
  for (const entry of (await readdir(join(root, prefix), { withFileTypes: true })).sort((a, b) => a.name.localeCompare(b.name))) {
    const relative = join(prefix, entry.name);
    if (entry.isDirectory()) { entries.push([relative, "directory"]); entries.push(...await treeSnapshot(root, relative)); }
    else {
      assert.ok(entry.isFile(), `Unexpected nonregular fixture entry: ${relative}`);
      const hash = createHash("sha256").update(await readFile(join(root, relative))).digest("hex");
      entries.push([relative, hash]);
    }
  }
  return entries;
}

async function absent(path) {
  await assert.rejects(stat(path), { code: "ENOENT" });
}

const temporary = await mkdtemp(join(tmpdir(), "mirket-memory-smoke-"));
const state = join(temporary, "state"), project = join(temporary, "project"), independent = join(temporary, "independent");
for (const path of [state, project, independent]) await mkdir(path, { mode: 0o700 });
if (process.platform !== "win32") assert.equal((await stat(state)).mode & 0o777, 0o700);
const clients = [];
const connect = async (release = RELEASE_A) => {
  const client = new Client(state, release); clients.push(client); return client.initialize();
};
const projectScope = (root = project) => ({ kind: "project", project_root: root });
const sharedScope = { kind: "shared_preferences" };
const args = (scope = projectScope(), release = RELEASE_A) => ({ expected_release_id: release, scope });
const record = (content, overrides = {}) => ({ kind: "fact", basis: "observed", content,
  sources: [{ source_id: "fixture:source", revision: "fixture-revision-1", locator: "unknown+fixture://source/line/1", captured_at: 123 }],
  status: "current", critical: false, applicability: { task_key: null, checkout_id: null, operation: null },
  valid_from: null, valid_until: null, ...overrides });
const query = (text, overrides = {}) => ({ query: text, status: "both", context: { task_key: null, checkout_id: null, operation: null, at_time: null }, limit: 20, max_bytes: 32768, ...overrides });
const listing = (overrides = {}) => ({ context: { task_key: null, checkout_id: null, operation: null, at_time: null },
  kind: null, status: "both", critical_only: false, applicability: "matching", limit: 20, max_bytes: 32768, cursor: null, ...overrides });
const operation = (prepared, scope = projectScope(), release = RELEASE_A) => ({ ...args(scope, release), operation_id: prepared.operation_id });
async function prepare(client, data, scope = projectScope()) {
  return success(client, "memory_prepare", { ...args(scope), mutation: { kind: "create", record: data, supersedes: null } });
}
async function save(client, data, scope = projectScope()) {
  const prepared = await prepare(client, data, scope);
  return success(client, "memory_commit", operation(prepared, scope));
}
const recall = (client, text, scope = projectScope(), overrides = {}) => success(client, "memory_search", { ...args(scope), request: query(text, overrides) });
async function waitCommitted(client, selected) {
  const until = Date.now() + STATUS_TIMEOUT_MS;
  let lastObserved;
  while (Date.now() < until) {
    const response = await client.call("memory_status", selected);
    if (response.isError) {
      assert.match(JSON.stringify(response), /busy/i);
      await new Promise((resolve) => setTimeout(resolve, 25));
      continue;
    }
    const status = body(response);
    lastObserved = status;
    if (status.status === "committed") return status;
    assert.ok(["prepared", "unknown"].includes(status.status), JSON.stringify(status));
    await new Promise((resolve) => setTimeout(resolve, 25));
  }
  throw new Error(`Memory commit did not become inspectable within fixture deadline; last status: ${JSON.stringify(lastObserved)}`);
}

try {
  let client = await connect();
  const inventory = await client.rpc("tools/list", {});
  const toolNames = new Set(inventory.tools.map((tool) => tool.name));
  for (const name of REQUIRED_TOOLS) assert.ok(toolNames.has(name), `${name} missing from MCP`);
  const beforeLookup = await treeSnapshot(state);
  await rejected(client, "memory_search", { ...args(), request: query("absent") }, /registered|scope|found/i);
  await rejected(client, "memory_read", { ...args(), record_id: "mem-absent", revision: null }, /registered|scope|found/i);
  await rejected(client, "memory_list", { ...args(), request: listing() }, /registered|scope|found/i);
  await rejected(client, "memory_list", { ...args(projectScope(), "stale-release"), request: listing() }, /release/i);
  await rejected(client, "memory_prepare", { ...args(projectScope(), "stale-release"), mutation: { kind: "create", record: record("must not save"), supersedes: null } }, /release/i);
  assert.deepEqual(await treeSnapshot(state), beforeLookup);
  await absent(join(state, "identity")); await absent(join(state, "memory"));
  console.log("PASS absent read-only lookup and stale release create no identity or memory storage");

  const prepared = await prepare(client, record("original orchard decision"));
  await absent(join(state, "memory"));
  const registeredBeforeList = await treeSnapshot(state);
  await rejected(client, "memory_list", { ...args(), request: listing() }, /memory|found|available|such file/i);
  assert.deepEqual(await treeSnapshot(state), registeredBeforeList, "Listing a registered scope with no canonical store must not create it");
  assert.equal(prepared.status, "prepared");
  const other = await connect();
  const beforeForeign = await treeSnapshot(state);
  await rejected(other, "memory_commit", operation(prepared), /owner|connection/i);
  assert.deepEqual(await treeSnapshot(state), beforeForeign);
  const results = await Promise.all([
    client.call("memory_commit", operation(prepared)),
    client.call("memory_commit", operation(prepared)),
  ]);
  for (const result of results.filter((result) => result.isError)) assert.match(JSON.stringify(result), /busy/i);
  const committed = results.filter((result) => !result.isError).map(body).find((result) => result.record?.data);
  assert.ok(committed, "One concurrent commit must return the newly saved revision");
  const id = committed.record.record_id;
  assert.equal(committed.record.revision, 1);
  const repeated = await success(client, "memory_commit", operation(prepared));
  assert.equal(repeated.status, "committed"); assert.equal(repeated.mutation.record_id, id);
  assert.equal((await recall(client, "orchard")).hits.length, 1);
  const exact = await success(client, "memory_read", { ...args(), record_id: id, revision: 1 });
  assert.equal(exact.record.data.sources[0].locator, "unknown+fixture://source/line/1");
  console.log("PASS prepare is non-executing, foreign commit refused, concurrent/repeated commit saves once");

  const correction = { kind: "revise", record_id: id, expected_revision: 1, record: record("corrected orchard decision", { basis: "user_stated" }) };
  const correcting = await success(client, "memory_prepare", { ...args(), mutation: correction });
  const competing = await success(other, "memory_prepare", { ...args(), mutation: { ...correction, record: record("stale competing orchard") } });
  const corrected = await success(client, "memory_commit", operation(correcting));
  assert.equal(corrected.record.revision, 2);
  await rejected(other, "memory_commit", operation(competing), /current|stale|revision/i);
  const history = await success(client, "memory_history", { ...args(), record_id: id, before_revision: null, limit: 20, max_bytes: 32768 });
  assert.deepEqual(history.records.map((entry) => entry.revision), [2, 1]);
  assert.equal(history.records[1].data.content, "original orchard decision");
  assert.equal(history.records[1].is_current_revision, false);
  assert.equal((await recall(client, "original")).hits.length, 0);
  console.log("PASS stale concurrent correction refused; provenance/history retained and old head excluded");

  const lost = await prepare(client, record("lost-response durable evidence"));
  client.loseCallResponse("memory_commit", operation(lost));
  const observed = await waitCommitted(other, operation(lost)).catch((error) => {
    throw new Error(`${error}; discarded response diagnostics: ${JSON.stringify(client.lostResponseDiagnostics)}`);
  });
  assert.equal(observed.mutation.available, true);
  await client.stop(true);
  client = await connect();
  const recovered = await success(client, "memory_status", operation(lost));
  assert.equal(recovered.mutation.record_id, observed.mutation.record_id);
  assert.equal(recovered.replay_allowed, false);
  // An explicit repeat may repair confirmation from the available canonical
  // mutation, but cannot replay the save or create another revision.
  const canonicalBeforeRecovery = await treeSnapshot(join(state, "memory"));
  const confirmation = await success(client, "memory_commit", operation(lost));
  assert.equal(confirmation.status, "committed");
  assert.equal(confirmation.reconciled, true);
  assert.equal(confirmation.replay_allowed, false);
  assert.equal(confirmation.intent_available, false);
  assert.deepEqual(confirmation.record, {
    record_id: observed.mutation.record_id, revision: observed.mutation.revision,
  });
  assert.deepEqual(await treeSnapshot(join(state, "memory")), canonicalBeforeRecovery);
  assert.equal((await recall(client, "lost-response")).hits.length, 1);
  console.log("PASS deliberately lost commit response reconciles after abrupt owner exit without replay");

  const onlyCurrent = record("current safety constraint", { critical: true });
  await save(client, onlyCurrent);
  await save(client, record("publish safety constraint", { critical: true, applicability: { task_key: null, checkout_id: null, operation: "publish" } }));
  await save(client, record("expired safety constraint", { critical: true, valid_until: 100 }));
  await save(client, record("orchard unresolved alternative", { status: "unresolved" }));
  const focused = await recall(client, "orchard");
  assert.equal(focused.coverage.lexical_only, true); assert.equal(focused.coverage.semantic_available, false);
  assert.equal(focused.coverage.applicable_current_critical, 1);
  assert.equal(focused.coverage.unclassified_current_critical, 1);
  assert.ok(focused.hits.some((hit) => hit.record.data.content === "current safety constraint"));
  assert.ok(!focused.hits.some((hit) => /publish|expired/.test(hit.record.data.content)));
  const publishing = await recall(client, "orchard", projectScope(), { context: { task_key: null, checkout_id: null, operation: "publish", at_time: null }, limit: 1 });
  assert.equal(publishing.coverage.applicable_current_critical, 2); assert.equal(publishing.coverage.critical_overflow, true);
  const bounded = await recall(client, "orchard", projectScope(), { max_bytes: 1024 });
  assert.ok(Buffer.byteLength(JSON.stringify(bounded)) <= 1024); assert.equal(bounded.coverage.truncated, true);
  const unresolved = await recall(client, "orchard", projectScope(), { status: "unresolved" });
  assert.equal(unresolved.hits.filter((hit) => hit.reason === "lexical").length, 1);
  assert.equal(unresolved.hits.find((hit) => hit.reason === "lexical").record.data.status, "unresolved");
  console.log("PASS lexical/current/unresolved filters, critical applicability and count/byte coverage");

  await rejected(client, "memory_prepare", { ...args(sharedScope), mutation: { kind: "create", record: record("project fact cannot become shared"), supersedes: null } }, /preferences/i);
  const shared = await save(client, record("prefer concise replies", { kind: "preference", basis: "user_stated" }), sharedScope);
  assert.equal((await recall(client, "concise", sharedScope)).hits.length, 1);
  assert.ok(!(await recall(client, "concise")).hits.some((hit) => hit.record.record_id === shared.record.record_id));
  await save(client, record("independent volcano fact"), projectScope(independent));
  assert.equal((await recall(client, "orchard", projectScope(independent))).hits.length, 0);
  await rejected(client, "memory_read", { ...args(projectScope(independent)), record_id: id, revision: null }, /found|scope/i);
  console.log("PASS explicit shared preferences and separate project isolation");

  // A minimal inspected Git metadata fixture, without running Git or its hooks.
  const repository = join(temporary, "repository"), linked = join(temporary, "linked");
  await mkdir(repository, { mode: 0o700 }); await mkdir(linked, { mode: 0o700 });
  const common = join(repository, ".git"), admin = join(common, "worktrees", "linked");
  for (const path of [common, join(common, "objects"), join(common, "refs"), join(common, "worktrees"), admin]) await mkdir(path, { mode: 0o700 });
  await writeFile(join(common, "HEAD"), "ref: refs/heads/main\n");
  await writeFile(join(admin, "HEAD"), "ref: refs/heads/topic\n");
  await writeFile(join(admin, "commondir"), "../..\n");
  await writeFile(join(admin, "gitdir"), `${join(await realpath(linked), ".git")}\n`);
  await writeFile(join(linked, ".git"), `gitdir: ${await realpath(admin)}\n`);
  const repoFact = await save(client, record("repository-wide durable fact"), projectScope(repository));
  const task = await save(client, record("checkout-local task progress", { kind: "task_state" }), projectScope(repository));
  const linkedIntent = await prepare(client, record("linked checkout registration"), projectScope(linked));
  assert.equal(linkedIntent.scope.project_id, repoFact.scope.project_id);
  assert.notEqual(linkedIntent.checkout_id, task.checkout_id);
  assert.ok((await recall(client, "durable", projectScope(linked))).hits.some((hit) => hit.record.record_id === repoFact.record.record_id));
  assert.equal((await recall(client, "progress", projectScope(linked))).hits.length, 0);
  await rejected(client, "memory_read", { ...args(projectScope(linked)), record_id: task.record.record_id, revision: 1 }, /checkout/i);
  await rejected(client, "memory_history", { ...args(projectScope(linked)), record_id: task.record.record_id, before_revision: null, limit: 20, max_bytes: 32768 }, /checkout/i);
  await rejected(client, "memory_prepare", { ...args(projectScope(linked)), mutation: { kind: "revise", record_id: task.record.record_id, expected_revision: 1, record: record("cannot steal task", { kind: "task_state" }) } }, /checkout/i);
  const linkedFacts = await success(client, "memory_list", { ...args(projectScope(linked)), request: listing({ kind: "fact" }) });
  assert.ok(linkedFacts.entries.some((entry) => entry.reference.record_id === repoFact.record.record_id));
  const ownTasks = await success(client, "memory_list", { ...args(projectScope(repository)), request: listing({ kind: "task_state" }) });
  assert.ok(ownTasks.entries.some((entry) => entry.reference.record_id === task.record.record_id));
  for (const applicability of ["matching", "unclassified"]) {
    const siblingTasks = await success(client, "memory_list", { ...args(projectScope(linked)), request: listing({ kind: "task_state", applicability }) });
    assert.ok(!siblingTasks.entries.some((entry) => entry.reference.record_id === task.record.record_id));
  }
  console.log("PASS verified worktree knowledge sharing preserves checkout task/read/history/enumeration boundaries");

  await continuationJourney();
  await enumerationJourney();
  await concurrentShutdownJourney();

  const staleForget = await success(client, "memory_forget_prepare", { ...args(), record_id: id, expected_revision: 2 });
  const beforePreparedForget = await success(client, "memory_read", { ...args(), record_id: id, revision: null });
  assert.equal(beforePreparedForget.record.revision, 2);
  await rejected(client, "memory_commit", operation(staleForget), /matching|forgetting|commit tool/i);
  await rejected(other, "memory_forget_commit", operation(staleForget), /owner|connection/i);
  await rejected(client, "memory_forget_reconcile", operation(staleForget), /no committed|cannot start/i);
  const concurrentCorrection = await success(other, "memory_prepare", { ...args(), mutation: {
    kind: "revise", record_id: id, expected_revision: 2, record: record("newest orchard correction"),
  } });
  await success(other, "memory_commit", operation(concurrentCorrection));
  await rejected(client, "memory_forget_commit", operation(staleForget), /revision|changed|current/i);
  assert.equal((await success(client, "memory_read", { ...args(), record_id: id, revision: null })).record.revision, 3);
  console.log("PASS forgetting requires its distinct commit tool/owner and stale preparation cannot remove a correction");

  const pendingRevision = await success(other, "memory_prepare", { ...args(), mutation: {
    kind: "revise", record_id: id, expected_revision: 3, record: record("uncommitted private target correction"),
  } });
  const pendingSupersession = await success(client, "memory_prepare", { ...args(), mutation: {
    kind: "create", supersedes: { record_id: id, revision: 3 }, record: record("uncommitted private target successor"),
  } });
  const unrelatedPending = await success(client, "memory_prepare", { ...args(), mutation: {
    kind: "revise", record_id: observed.mutation.record_id, expected_revision: 1, record: record("unrelated pending correction survives"),
  } });
  const selectedForget = await success(client, "memory_forget_prepare", { ...args(), record_id: id, expected_revision: 3 });
  const uncommittedForget = await success(client, "memory_forget_prepare", { ...args(), record_id: id, expected_revision: 3 });
  const forgotten = await success(client, "memory_forget_commit", operation(selectedForget));
  assert.equal(forgotten.status, "forgotten");
  assert.equal(forgotten.forget.logically_forgotten, true);
  assert.equal(forgotten.forget.cleanup_complete, true);
  for (const revision of [null, 1, 2, 3]) {
    await rejected(client, "memory_read", { ...args(), record_id: id, revision }, /found|forgotten|excluded/i);
  }
  await rejected(client, "memory_history", { ...args(), record_id: id, before_revision: null, limit: 20, max_bytes: 32768 }, /found|forgotten|excluded/i);
  assert.ok(!(await recall(client, "orchard")).hits.some((hit) => hit.record.record_id === id));
  for (const consumed of [prepared, correcting, concurrentCorrection, competing, pendingRevision, pendingSupersession]) {
    await absent(join(state, "operations", `${consumed.operation_id}.json`));
    await rejected(client, "memory_status", operation(consumed), /available|found|such file/i);
  }
  await rejected(other, "memory_commit", operation(pendingRevision), /available|found|such file/i);
  await rejected(client, "memory_commit", operation(pendingSupersession), /available|found|such file/i);
  assert.equal((await success(client, "memory_status", operation(unrelatedPending))).status, "prepared");
  assert.ok((await stat(join(state, "operations", `${unrelatedPending.operation_id}.json`))).isFile());
  // An exclusion for this target must never prove a different prepared ID ran.
  const neverCommitted = await success(client, "memory_status", operation(uncommittedForget));
  assert.equal(neverCommitted.status, "prepared"); assert.equal(neverCommitted.forget, null);
  await rejected(client, "memory_forget_reconcile", operation(uncommittedForget), /no committed|cannot start/i);
  assert.equal((await success(client, "memory_status", operation(lost))).status, "committed");
  assert.equal((await recall(client, "concise", sharedScope)).hits.length, 1);
  assert.equal((await recall(client, "volcano", projectScope(independent))).hits.length, 1);
  console.log("PASS forgetting removes all target revisions/search/history and consumed/prepared target receipts, preserving other records/scopes");

  await client.stop();
  client = await connect();
  const forgetReceipt = await success(client, "memory_status", operation(selectedForget));
  assert.equal(forgetReceipt.status, "forgotten");
  assert.equal(forgetReceipt.forget.operation_id, selectedForget.operation_id);
  assert.equal(forgetReceipt.forget.expected_revision, 3);
  assert.equal(forgetReceipt.forget.status.cleanup_complete, true);
  assert.equal((await success(client, "memory_forget_reconcile", operation(selectedForget))).status, "forgotten");
  await rejected(client, "memory_forget_commit", operation(uncommittedForget), /owner|connection/i);
  console.log("PASS exact forgetting receipt survives reconnect and reconciliation cannot start an uncommitted decision");

  await client.stop();
  client = await connect(RELEASE_B);
  const beforeStale = await treeSnapshot(state);
  await rejected(client, "memory_search", { ...args(), request: query("orchard") }, /release/i);
  await rejected(client, "memory_list", { ...args(), request: listing() }, /release/i);
  await rejected(client, "memory_prepare", { ...args(), mutation: { kind: "create", record: record("stale new save"), supersedes: null } }, /release/i);
  await rejected(client, "memory_forget_prepare", { ...args(), record_id: observed.mutation.record_id, expected_revision: 1 }, /release/i);
  await rejected(client, "memory_forget_reconcile", operation(selectedForget), /release/i);
  assert.deepEqual(await treeSnapshot(state), beforeStale);
  assert.equal((await success(client, "memory_status", operation(lost))).status, "committed");
  assert.equal((await success(client, "memory_status", operation(selectedForget))).status, "forgotten");
  console.log("PASS release mismatch blocks new reads/writes while scoped save/forget receipts remain inspectable");
  console.log(`PASS canonical/lexical memory MCP fixture (${process.platform}/${process.arch}, Node ${process.version})`);
  await client.stop();
  await manualTransferJourney();
  if (flags.has("--embedding-package")) await configuredRuntimeJourney();
} catch (error) {
  // Preserve the initiating failure even if concurrent teardown also fails.
  console.error("PRIMARY memory fixture failure:", error);
  throw error;
} finally {
  await Promise.all(clients.map((client) => client.stop()));
  await rm(temporary, { recursive: true, force: true });
}

async function concurrentShutdownJourney() {
  const root = join(temporary, "concurrent-shutdown-project");
  await mkdir(root, { mode: 0o700 });
  const selection = projectScope(root);
  const owners = [await connect(), await connect(), await connect()];
  const intents = [];
  for (let i = 0; i < owners.length; i++) {
    intents.push(await prepare(owners[i], record(`Uncommitted shutdown intent ${i}`), selection));
  }
  const canonicalBefore = await treeSnapshot(join(state, "memory"));
  // Each owner has real unused work to retire under the shared receipt lock.
  await Promise.all(owners.map((owner) => owner.stop()));
  const observer = await connect();
  for (const intent of intents) {
    const status = await success(observer, "memory_status", operation(intent, selection));
    assert.equal(status.status, "cancelled");
    await rejected(observer, "memory_commit", operation(intent, selection), /owner|connection|prepared|intent/i);
  }
  assert.deepEqual(await treeSnapshot(join(state, "memory")), canonicalBefore,
    "Concurrent disconnect retirement must not commit or replay memory");
  await observer.stop();
  console.log("PASS concurrent client shutdown retires unused intents without replay or failed graceful exit");
}

// Exhaustive enumeration is a canonical read, independent of query wording or models.
async function enumerationJourney() {
  const root = join(temporary, "enumeration-project");
  await mkdir(root, { mode: 0o700 });
  const selection = projectScope(root);
  const client = await connect();
  const atTime = Math.floor(Date.now() / 1000);
  const context = { task_key: null, checkout_id: null, operation: null, at_time: atTime };
  const request = listing({ context, kind: "decision", status: "current", critical_only: true });
  const list = (overrides = {}) => success(client, "memory_list", { ...args(selection), request: { ...request, ...overrides } });
  const original = new Map();
  const decisiveTail = "Decisive exception: never submit twice when the first submission has an unknown effect.";
  for (let i = 0; i < 23; i++) {
    const data = record(`Constraint ${String(i).padStart(2, "0")}: ${"İnceleme önce gelir. ".repeat(35)}${decisiveTail}`, {
      kind: "decision", critical: true,
    });
    const saved = await save(client, data, selection);
    original.set(saved.record.record_id, { reference: saved.record, data });
  }
  const unresolved = await save(client, record("An unresolved critical decision needs review.", {
    kind: "decision", critical: true, status: "unresolved",
  }), selection);
  const notCritical = await save(client, record("An ordinary current decision.", { kind: "decision" }), selection);
  const otherKind = await save(client, record("A critical fact has another record kind.", { critical: true }), selection);
  const unclassified = await save(client, record("Publishing requires this caller task's approval.", {
    kind: "decision", critical: true,
    applicability: { task_key: "enumeration-publish", checkout_id: null, operation: "publish" },
  }), selection);
  const future = await save(client, record("This decision is not valid yet.", {
    kind: "decision", critical: true, valid_from: atTime + 1000,
  }), selection);
  const expired = await save(client, record("This decision has expired.", {
    kind: "decision", critical: true, valid_until: atTime - 1000,
  }), selection);

  const beforeList = await treeSnapshot(join(state, "memory"));
  const entries = [];
  let cursor = null, pages = 0;
  do {
    const page = await list({ cursor });
    assert.equal(page.stable_snapshot, false);
    assert.equal(page.valid_at, atTime);
    assert.equal(typeof page.observed_at, "number");
    assert.ok(page.entries.length <= 20);
    assert.ok(Buffer.byteLength(JSON.stringify(page)) <= request.max_bytes);
    for (const entry of page.entries) {
      assert.equal(entry.detail, "summary");
      assert.equal(entry.kind, "decision"); assert.equal(entry.status, "current"); assert.equal(entry.critical, true);
      assert.equal(entry.source_count, 1); assert.equal(entry.content_truncated, true);
      assert.ok(Buffer.byteLength(entry.excerpt) <= 256);
      assert.ok(!entry.excerpt.includes("Decisive exception"));
      assert.ok(!entry.excerpt.includes("\uFFFD"), "Excerpt split a UTF-8 character");
      entries.push(entry);
    }
    cursor = page.next_cursor;
    assert.equal(page.has_more, cursor !== null);
    if (cursor !== null) {
      assert.equal(cursor.schema, 1); assert.equal(cursor.valid_at, atTime);
      assert.equal(cursor.after_record_id, page.entries.at(-1).reference.record_id);
      assert.match(cursor.query_sha256, /^[a-f0-9]{64}$/);
    }
    pages++;
    assert.ok(pages <= 5, "Enumeration must progress within this bounded fixture");
  } while (cursor !== null);
  assert.equal(pages, 2, "Twenty-record pages must expose a continuation for 23 eligible heads");
  assert.deepEqual(entries.map((entry) => entry.reference.record_id), [...original.keys()].sort());
  assert.equal(new Set(entries.map((entry) => entry.reference.record_id)).size, 23);
  const selected = entries[0].reference;
  const exact = await success(client, "memory_read", { ...args(selection), record_id: selected.record_id, revision: selected.revision });
  assert.equal(exact.record.data.content, original.get(selected.record_id).data.content);
  assert.ok(exact.record.data.content.endsWith(decisiveTail));
  assert.equal(exact.record.is_current_revision, true);

  const pending = await list({ status: "unresolved" });
  assert.deepEqual(pending.entries.map((entry) => entry.reference.record_id), [unresolved.record.record_id]);
  const collect = async (overrides) => {
    const collected = []; let cursor = null, pages = 0;
    do {
      const page = await list({ ...overrides, cursor });
      collected.push(...page.entries); cursor = page.next_cursor;
      assert.equal(page.has_more, cursor !== null);
      assert.ok(++pages <= 5, "Filtered enumeration did not progress");
    } while (cursor !== null);
    return collected;
  };
  const ordinary = await collect({ critical_only: false });
  assert.deepEqual(ordinary.map((entry) => entry.reference.record_id), [...original.keys(), notCritical.record.record_id].sort());
  const facts = await list({ kind: "fact" });
  assert.deepEqual(facts.entries.map((entry) => entry.reference.record_id), [otherKind.record.record_id]);
  assert.ok(!entries.some((entry) => [notCritical, otherKind, future, expired, unclassified].some((saved) => saved.record.record_id === entry.reference.record_id)));
  const unknown = await list({ applicability: "unclassified" });
  assert.deepEqual(unknown.entries.map((entry) => entry.reference.record_id), [unclassified.record.record_id]);
  const mismatched = await list({ applicability: "unclassified", context: { ...context, task_key: "different-task" } });
  assert.equal(mismatched.entries.length, 0, "Unclassified must not include a known context mismatch");
  const applicable = await collect({ context: { ...context, task_key: "enumeration-publish", operation: "publish" } });
  assert.deepEqual(applicable.map((entry) => entry.reference.record_id), [...original.keys(), unclassified.record.record_id].sort());
  assert.deepEqual(await treeSnapshot(join(state, "memory")), beforeList, "Enumeration and exact reads must not mutate canonical state");

  // A long but valid applicability summary cannot fit 1024 bytes; the reference
  // still lets the caller inspect the exact full record and make progress.
  const longContext = { ...context, task_key: "T".repeat(256), operation: "O".repeat(256) };
  const large = await save(client, record("😀".repeat(100), { kind: "preference", critical: true,
    applicability: { task_key: longContext.task_key, checkout_id: null, operation: longContext.operation },
  }), selection);
  const tiny = await list({ context: longContext, kind: "preference", max_bytes: 1024 });
  assert.ok(Buffer.byteLength(JSON.stringify(tiny)) <= 1024);
  assert.equal(tiny.entries.length, 1); assert.equal(tiny.entries[0].detail, "reference_only");
  assert.equal(tiny.entries[0].reason, "byte_budget");
  assert.deepEqual(tiny.entries[0].reference, { record_id: large.record.record_id, revision: large.record.revision });
  assert.equal(tiny.has_more, false); assert.equal(tiny.next_cursor, null);
  const expanded = await success(client, "memory_read", { ...args(selection), record_id: large.record.record_id, revision: large.record.revision });
  assert.equal(expanded.record.data.content, "😀".repeat(100));

  const first = await list({ limit: 1 });
  assert.equal(first.has_more, true);
  const unchanged = await treeSnapshot(join(state, "memory"));
  await rejected(client, "memory_list", { ...args(selection), request: { ...request, critical_only: false, cursor: first.next_cursor } }, /cursor|binding|query|filter/i);
  assert.deepEqual(await treeSnapshot(join(state, "memory")), unchanged);
  const remaining = [...original.keys()].sort().filter((id) => id > first.next_cursor.after_record_id);
  const correctedId = remaining[0], forgottenId = remaining[1];
  assert.ok(correctedId && forgottenId);
  const revisedData = { ...original.get(correctedId).data, content: "Corrected current constraint; the previous revision is no longer the head." };
  const correction = await success(client, "memory_prepare", { ...args(selection), mutation: {
    kind: "revise", record_id: correctedId, expected_revision: 1, record: revisedData,
  } });
  await success(client, "memory_commit", operation(correction, selection));
  const forgetting = await success(client, "memory_forget_prepare", { ...args(selection), record_id: forgottenId, expected_revision: 1 });
  assert.equal((await success(client, "memory_forget_commit", operation(forgetting, selection))).status, "forgotten");
  const afterMutation = [];
  cursor = first.next_cursor; pages = 0;
  do {
    // Cursor permits a different page size; it does not pin canonical revisions.
    const page = await list({ limit: 7, cursor });
    assert.equal(page.stable_snapshot, false); assert.equal(page.valid_at, first.valid_at);
    afterMutation.push(...page.entries);
    cursor = page.next_cursor; pages++;
    assert.equal(page.has_more, cursor !== null); assert.ok(pages <= 5);
  } while (cursor !== null);
  assert.ok(!afterMutation.some((entry) => entry.reference.record_id === forgottenId));
  const revised = afterMutation.find((entry) => entry.reference.record_id === correctedId);
  assert.ok(revised); assert.equal(revised.reference.revision, 2); assert.match(revised.excerpt, /Corrected current constraint/);
  assert.deepEqual(afterMutation.map((entry) => entry.reference.record_id), remaining.filter((id) => id !== forgottenId));
  await client.stop();
  console.log("PASS memory_list enumerates 23 critical heads without query guessing, expands exact evidence, enforces filters/worktree context, and respects correction/forgetting between pages");
}

// Two caller-correlated tasks share a checkout, without inferred host thread IDs.
// This uses only the unconfigured lexical service, even in the optional model run.
async function continuationJourney() {
  const root = join(temporary, "continuation-project");
  await mkdir(root, { mode: 0o700 });
  const selection = projectScope(root);
  const evidence = [
    "Observed: İzin requires inspection.\nException: never replay a submission whose effect is unknown.\n",
    "Observed: the archive has three verified entries.\nNext: inspect their source revisions.\n",
  ];
  const paths = ["submission.txt", "archive.txt"];
  const keys = ["caller-task-submission", "caller-task-archive"];
  const revisions = evidence.map((text) => `sha256:${createHash("sha256").update(text).digest("hex")}`);
  await mkdir(join(root, "plans"), { mode: 0o700 });
  for (let i = 0; i < paths.length; i++) {
    await writeFile(join(root, paths[i]), evidence[i]);
    await writeFile(join(root, "plans", `${keys[i]}.md`), `# ${keys[i]}\nInspect sources before the next action.\n`);
  }
  const checkpoints = keys.map((key, i) => record([
    `Objective: continue ${key}. Scope: ${root}; task_key: ${key}.`,
    `Constraints: inspect evidence before action. Plan pointer: plans/${key}.md (not copied).`,
    `Completed observation: inspected ${paths[i]} at ${revisions[i]}.`,
    "Inference: the next action may be safe only after source revalidation.",
    "Proposal/next action: inspect the remaining source range; no action authorized by this note.",
    "Unresolved question: has the external submission completed?",
    "Unknown effect: external fixture submission external-case-17; inspect, never replay.",
  ].join("\n"), {
    kind: "task_state",
    applicability: { task_key: key, checkout_id: null, operation: null },
    sources: [{ source_id: paths[i], revision: revisions[i], locator: `bytes:0-${Buffer.byteLength(evidence[i])}`, captured_at: 123 }],
  }));
  let writer = await connect();
  const saved = [];
  for (const checkpoint of checkpoints) saved.push(await save(writer, checkpoint, selection));
  assert.equal(saved[0].checkout_id, saved[1].checkout_id);
  assert.equal(typeof saved[0].checkout_id, "string");
  assert.notEqual(saved[0].record.record_id, saved[1].record.record_id);
  await writer.stop();
  writer = await connect();
  const beforeResume = await treeSnapshot(join(state, "memory"));
  const exactRecords = [];
  for (let i = 0; i < keys.length; i++) {
    const recalled = await recall(writer, "", selection, {
      context: { task_key: keys[i], checkout_id: null, operation: null, at_time: null },
    });
    assert.ok(recalled.hits.some((hit) => hit.record.record_id === saved[i].record.record_id));
    assert.ok(!recalled.hits.some((hit) => hit.record.record_id === saved[1 - i].record.record_id));
    const exact = await success(writer, "memory_read", {
      ...args(selection), record_id: saved[i].record.record_id, revision: saved[i].record.revision,
    });
    assert.equal(exact.record.is_current_revision, true);
    assert.equal(exact.record.data.content, checkpoints[i].content);
    assert.deepEqual(exact.record.data.sources, checkpoints[i].sources);
    assert.equal(exact.record.data.applicability.task_key, keys[i]);
    exactRecords.push(exact.record);
  }
  // Codex reads project files directly; memory retains the original source
  // revision and does not mutate canonical records when a source later changes.
  for (let i = 0; i < paths.length; i++) {
    assert.equal(await readFile(join(root, paths[i]), "utf8"), evidence[i]);
    assert.equal(exactRecords[i].data.sources[0].revision, revisions[i]);
  }
  const newerSource = "Changed source: the old checkpoint must be rechecked before work.\n";
  await writeFile(join(root, paths[0]), newerSource);
  assert.equal(await readFile(join(root, paths[0]), "utf8"), newerSource);
  assert.deepEqual(await treeSnapshot(join(state, "memory")), beforeResume,
    "Direct source inspection must not save progress or replay checkpoint effects");

  const mutation = { kind: "revise", record_id: exactRecords[0].record_id,
    expected_revision: exactRecords[0].revision, record: { ...exactRecords[0].data,
      content: `${exactRecords[0].data.content}\nObserved on resume: source revision changed; continuation remains blocked.` } };
  const current = await success(writer, "memory_prepare", { ...args(selection), mutation });
  const stale = await success(writer, "memory_prepare", { ...args(selection), mutation });
  const revised = await success(writer, "memory_commit", operation(current, selection));
  assert.equal(revised.record.revision, exactRecords[0].revision + 1);
  await rejected(writer, "memory_commit", operation(stale, selection), /stale|revision|current/i);
  const untouched = await success(writer, "memory_read", { ...args(selection), record_id: exactRecords[1].record_id, revision: null });
  assert.deepEqual(untouched.record, exactRecords[1]);
  const old = await success(writer, "memory_read", { ...args(selection), record_id: exactRecords[0].record_id, revision: exactRecords[0].revision });
  assert.equal(old.record.is_current_revision, false);
  assert.equal(old.record.data.content, checkpoints[0].content);
  await writer.stop();
  console.log("PASS two caller-keyed tasks resume independently in one checkout; exact checkpoints retain evidence/unknown effects, direct source inspection shows drift, stale progress cannot overwrite");
}

// This is a transport/currentness journey, not a multilingual quality benchmark.
// It reuses the bounded client above; native model execution requires the explicit flag.
async function configuredRuntimeJourney() {
  const configuredState = join(temporary, "configured-state");
  const configuredProject = join(temporary, "configured-project");
  const backupDir = join(temporary, "configured-backups");
  const unavailablePackage = join(temporary, "unavailable-package");
  for (const path of [configuredState, configuredProject, backupDir, unavailablePackage]) await mkdir(path, { mode: 0o700 });
  const selection = projectScope(configuredProject);
  const selectedArgs = () => args(selection);
  const runtimeOptions = { embeddingPackage: flags.get("--embedding-package"), backupDir, rpcTimeoutMs: 90000 };
  const configuredConnect = async (options = {}) => {
    const client = new Client(configuredState, RELEASE_A, options);
    clients.push(client);
    return client.initialize();
  };
  const refresh = (client, overrides = {}) => success(client, "memory_index_refresh", {
    ...selectedArgs(), after_row_id: null, limit: 1, max_bytes: 16384, ...overrides,
  });
  const admin = async (command, extra = []) => {
    // The maintenance CLI explicitly requires canonical existing roots.
    const commandArgs = ["dev", "memory", command, "--state-dir", await realpath(configuredState),
      "--backup-dir", await realpath(backupDir), "--release-id", RELEASE_A, "--project-root", configuredProject, ...extra];
    return new Promise((resolve, reject) => {
      const child = spawn(flags.get("--mirket"), commandArgs, { stdio: ["ignore", "pipe", "pipe"] });
      let stdout = "", stderr = "", exceeded = false;
      const timer = setTimeout(() => { exceeded = true; child.kill("SIGKILL"); }, 90000);
      child.stdout.setEncoding("utf8").on("data", (part) => { stdout += part; if (Buffer.byteLength(stdout) > MAX_FRAME) { exceeded = true; child.kill("SIGKILL"); } });
      child.stderr.setEncoding("utf8").on("data", (part) => { stderr += part; if (Buffer.byteLength(stderr) > MAX_FRAME) { exceeded = true; child.kill("SIGKILL"); } });
      child.on("error", (error) => { clearTimeout(timer); reject(error); });
      child.on("close", (code, signal) => {
        clearTimeout(timer);
        if (exceeded || code !== 0) { reject(new Error(`memory ${command} failed (${code ?? signal}): ${stderr}`)); return; }
        try { resolve(JSON.parse(stdout)); } catch (error) { reject(error); }
      });
    });
  };
  const { DatabaseSync } = await import("node:sqlite");
  const inspectSnapshot = async (snapshot, recordId, expectedContent) => {
    assert.equal(snapshot.eligible, true, JSON.stringify(snapshot)); assert.equal(snapshot.phase, "eligible", JSON.stringify(snapshot));
    assert.match(snapshot.snapshot_id, /^[a-zA-Z0-9-]+$/);
    const path = join(backupDir, `${snapshot.snapshot_id}.sqlite3`);
    assert.equal((await stat(path)).size, snapshot.byte_len);
    const database = new DatabaseSync(path, { readOnly: true });
    try {
      const rows = database.prepare("SELECT content FROM revisions WHERE record_id=? ORDER BY revision").all(recordId);
      if (expectedContent === null) assert.equal(rows.length, 0, JSON.stringify(rows));
      else assert.ok(rows.some((row) => row.content === expectedContent), `Actual SQLite recovery point lacks committed canonical content: ${JSON.stringify({ snapshot, rows })}`);
      assert.equal(database.prepare("PRAGMA quick_check").get().quick_check, "ok");
    } finally { database.close(); }
    return path;
  };

  let configured = await configuredConnect();
  const meteorite = await save(configured, record("Meteorite calibration uses three independent measurements."), selection);
  const harbor = await save(configured, record("Harbor archive backups are inspected each Tuesday."), selection);
  await rejected(configured, "memory_index_refresh", { ...selectedArgs(), after_row_id: null, limit: 1, max_bytes: 16384 }, /configured|embedding/i);
  await configured.stop();

  configured = await configuredConnect(runtimeOptions);
  const resources = (await success(configured, "runtime_status")).memory_resources;
  assert.equal(resources.embedding_configured, true, JSON.stringify(resources)); assert.equal(resources.managed_backups_configured, true, JSON.stringify(resources));
  const beforeStaleRefresh = await treeSnapshot(configuredState);
  await rejected(configured, "memory_index_refresh", { ...args(selection, "stale-release"), after_row_id: null, limit: 1, max_bytes: 16384 }, /release/i);
  await rejected(configured, "memory_index_refresh", { ...selectedArgs(), after_row_id: null, limit: 0, max_bytes: 16384 }, /bound|limit|invalid/i);
  assert.deepEqual(await treeSnapshot(configuredState), beforeStaleRefresh);
  const rebuilt = [];
  let cursor = null, pageCount = 0;
  for (;;) {
    const page = await refresh(configured, { after_row_id: cursor });
    assert.equal(page.status, "page_complete", JSON.stringify(page)); assert.equal(page.stable_snapshot, false, JSON.stringify(page));
    assert.equal(page.restart_required, false, JSON.stringify(page)); assert.ok(page.scanned_heads <= 1); assert.ok(page.scanned_bytes <= 16384);
    assert.equal(page.failed.length, 0, JSON.stringify(page)); assert.equal(page.skipped.length, 0, JSON.stringify(page));
    rebuilt.push(...page.indexed.map((ref) => ref.record_id));
    assert.ok(!JSON.stringify(page).includes("Meteorite calibration"), "Index receipt must not echo record content");
    pageCount++;
    assert.ok(pageCount <= 4, "Index scan did not finish within fixture pages");
    if (!page.has_more) { assert.equal(page.coverage.missing_heads, 0, JSON.stringify(page)); break; }
    assert.ok(Number.isSafeInteger(page.next_after_row_id) && page.next_after_row_id > (cursor ?? 0));
    cursor = page.next_after_row_id;
  }
  assert.deepEqual(rebuilt.sort(), [meteorite.record.record_id, harbor.record.record_id].sort());
  const noWork = await refresh(configured, { limit: 32, max_bytes: 262144 });
  assert.equal(noWork.indexed.length, 0, JSON.stringify(noWork)); assert.equal(noWork.already_indexed_at_selection, 2, JSON.stringify(noWork));
  console.log("PASS configured MCP bounded index refresh publishes exact current revisions and exposes continuation/coverage");

  const original = "Gözlemevi teleskobunun bataryası dokuz saat çalışır; ölçüm soğuk gecede yapılmıştır.";
  const prepared = await prepare(configured, record(original), selection);
  const committed = await success(configured, "memory_commit", operation(prepared, selection));
  assert.equal(committed.status, "committed", JSON.stringify(committed)); assert.equal(committed.semantic_index.status, "indexed", JSON.stringify(committed));
  assert.equal(committed.backup.status, "eligible", JSON.stringify(committed));
  const snapshot = committed.backup.snapshot;
  const snapshotPath = await inspectSnapshot(snapshot, committed.record.record_id, original);
  const snapshotHash = createHash("sha256").update(await readFile(snapshotPath)).digest("hex");
  const repeated = await success(configured, "memory_commit", operation(prepared, selection));
  assert.equal(repeated.status, "committed", JSON.stringify(repeated));
  assert.equal(createHash("sha256").update(await readFile(snapshotPath)).digest("hex"), snapshotHash);
  assert.equal((await readdir(backupDir)).filter((name) => name === `${snapshot.snapshot_id}.sqlite3`).length, 1);
  const recalled = await recall(configured, "How many hours can the observatory telescope battery operate?", selection);
  assert.equal(recalled.coverage.semantic_available, true, JSON.stringify(recalled)); assert.equal(recalled.coverage.lexical_only, false, JSON.stringify(recalled));
  assert.equal(recalled.coverage.semantic_missing_eligible, 0, JSON.stringify(recalled));
  assert.ok(recalled.hits.some((hit) => hit.record.record_id === committed.record.record_id && hit.record.revision === 1), JSON.stringify(recalled));
  assert.ok(Buffer.byteLength(JSON.stringify(recalled)) <= 32768);
  console.log("PASS actual save returns eligible SQLite backup and indexed revision; cross-language recall uses configured native runtime");

  const correctedText = "Gözlemevi teleskobunun yeni bataryası on iki saat çalışır; dokuz saatlik eski karar değiştirilmiştir.";
  const correcting = await success(configured, "memory_prepare", { ...selectedArgs(), mutation: {
    kind: "revise", record_id: committed.record.record_id, expected_revision: 1, record: record(correctedText),
  } });
  const corrected = await success(configured, "memory_commit", operation(correcting, selection));
  assert.equal(corrected.record.revision, 2, JSON.stringify(corrected)); assert.equal(corrected.semantic_index.status, "indexed", JSON.stringify(corrected));
  const current = await recall(configured, "How long does the replacement telescope battery run?", selection);
  assert.ok(current.hits.some((hit) => hit.record.record_id === committed.record.record_id && hit.record.revision === 2), JSON.stringify(current));
  assert.ok(!current.hits.some((hit) => hit.record.record_id === committed.record.record_id && hit.record.revision === 1), JSON.stringify(current));
  const history = await success(configured, "memory_history", { ...selectedArgs(), record_id: committed.record.record_id, before_revision: null, limit: 20, max_bytes: 32768 });
  assert.deepEqual(history.records.map((entry) => entry.revision), [2, 1], JSON.stringify(history));
  const forget = await success(configured, "memory_forget_prepare", { ...selectedArgs(), record_id: committed.record.record_id, expected_revision: 2 });
  const forgotten = await success(configured, "memory_forget_commit", operation(forget, selection));
  assert.equal(forgotten.status, "forgotten", JSON.stringify(forgotten)); assert.equal(forgotten.forget.cleanup_complete, true, JSON.stringify(forgotten));
  await absent(snapshotPath);
  await rejected(configured, "memory_read", { ...selectedArgs(), record_id: committed.record.record_id, revision: 1 }, /found|unavailable|forgot/i);
  await rejected(configured, "memory_history", { ...selectedArgs(), record_id: committed.record.record_id, before_revision: null, limit: 20, max_bytes: 32768 }, /found|unavailable|forgot/i);
  const afterForget = await recall(configured, "telescope battery hours", selection);
  assert.ok(!afterForget.hits.some((hit) => hit.record.record_id === committed.record.record_id), JSON.stringify(afterForget));
  assert.equal(afterForget.coverage.semantic_missing_eligible, 0, JSON.stringify(afterForget));
  const afterForgetRefresh = await refresh(configured, { limit: 32, max_bytes: 262144 });
  assert.equal(afterForgetRefresh.coverage.current_heads, 2, JSON.stringify(afterForgetRefresh));
  assert.equal(afterForgetRefresh.indexed.length, 0, JSON.stringify(afterForgetRefresh));
  console.log("PASS configured semantic recall excludes obsolete and forgotten revisions; affected managed backup is removed");
  await configured.stop();

  configured = await configuredConnect({ ...runtimeOptions, embeddingPackage: await realpath(unavailablePackage) });
  const degraded = await recall(configured, "meteorite", selection);
  assert.equal(degraded.coverage.lexical_only, true, JSON.stringify(degraded)); assert.equal(degraded.coverage.semantic_available, false, JSON.stringify(degraded));
  assert.equal(typeof degraded.coverage.semantic_warning, "string", JSON.stringify(degraded));
  assert.ok(degraded.coverage.semantic_warning.length > 0 && Buffer.byteLength(degraded.coverage.semantic_warning) <= 512, JSON.stringify(degraded));
  assert.ok(degraded.hits.some((hit) => hit.record.record_id === meteorite.record.record_id), JSON.stringify(degraded));
  const pending = await save(configured, record("Quartz timing is preserved when semantic indexing is unavailable."), selection);
  assert.equal(pending.status, "committed", JSON.stringify(pending)); assert.equal(pending.semantic_index.status, "pending", JSON.stringify(pending));
  assert.equal(pending.backup.status, "eligible", JSON.stringify(pending));
  const preserved = await success(configured, "memory_read", { ...selectedArgs(), record_id: pending.record.record_id, revision: null });
  assert.equal(preserved.record.data.content, pending.record.data.content, JSON.stringify(preserved));
  console.log("INFO unavailable package warnings", JSON.stringify({ search: degraded.coverage.semantic_warning, indexing: pending.semantic_index }));
  await inspectSnapshot(pending.backup.snapshot, committed.record.record_id, null);
  const partial = await refresh(configured, { limit: 32, max_bytes: 262144 });
  assert.equal(partial.status, "pending", JSON.stringify(partial)); assert.equal(partial.restart_required, true, JSON.stringify(partial));
  assert.ok(partial.failed.some((entry) => entry.reference.record_id === pending.record.record_id && entry.reason === "embedding_unavailable"), JSON.stringify(partial));
  assert.ok(partial.coverage.missing_heads >= 1, JSON.stringify(partial));
  console.log("INFO unavailable package index receipt", JSON.stringify(partial));
  console.log("PASS unavailable configured payload produces explicit lexical fallback and pending index without losing canonical save/backup");
  await configured.stop();

  configured = await configuredConnect(runtimeOptions);
  const repaired = await refresh(configured, { limit: 32, max_bytes: 262144 });
  assert.equal(repaired.status, "page_complete", JSON.stringify(repaired)); assert.equal(repaired.restart_required, false, JSON.stringify(repaired));
  assert.ok(repaired.indexed.some((ref) => ref.record_id === pending.record.record_id && ref.revision === 1), JSON.stringify(repaired));
  assert.equal(repaired.coverage.missing_heads, 0, JSON.stringify(repaired));
  const final = await recall(configured, "Quartz timing", selection);
  assert.equal(final.coverage.semantic_available, true, JSON.stringify(final));
  assert.ok(final.hits.some((hit) => hit.record.record_id === pending.record.record_id), JSON.stringify(final));
  const receipt = await success(configured, "memory_status", operation(forget, selection));
  assert.equal(receipt.status, "forgotten", JSON.stringify(receipt));
  const laterIntent = await success(configured, "memory_prepare", { ...selectedArgs(), mutation: {
    kind: "revise", record_id: pending.record.record_id, expected_revision: 1,
    record: record("Quartz timing changed to an alternate calibration after the recovery point."),
  } });
  const later = await success(configured, "memory_commit", operation(laterIntent, selection));
  assert.equal(later.record.revision, 2, JSON.stringify(later)); assert.equal(later.semantic_index.status, "indexed", JSON.stringify(later));
  await configured.stop();
  console.log("PASS explicit rebuild recovers pending index from canonical source without replaying saves or forgotten records");

  const backups = await admin("backups");
  assert.ok(backups.some((point) => point.snapshot_id === pending.backup.snapshot.snapshot_id && point.eligible), JSON.stringify(backups));
  assert.ok(!backups.some((point) => point.snapshot_id === snapshot.snapshot_id && point.eligible), JSON.stringify(backups));
  const restore = await admin("restore-prepare", ["--snapshot-id", pending.backup.snapshot.snapshot_id]);
  assert.equal(restore.complete, false, JSON.stringify(restore)); assert.equal(restore.receipt_phase, "prepared", JSON.stringify(restore));
  const beforeRestore = await admin("restore-status", ["--operation-id", restore.operation_id]);
  assert.equal(beforeRestore.complete, false, JSON.stringify(beforeRestore));
  const restored = await admin("restore-commit", ["--operation-id", restore.operation_id]);
  assert.equal(restored.operation_id, restore.operation_id, JSON.stringify(restored)); assert.equal(restored.complete, true, JSON.stringify(restored));
  const restoredStatus = await admin("restore-status", ["--operation-id", restore.operation_id]);
  assert.equal(restoredStatus.complete, true, JSON.stringify(restoredStatus));
  const reconciledRestore = await admin("restore-reconcile", ["--operation-id", restore.operation_id]);
  assert.equal(reconciledRestore.complete, true, JSON.stringify(reconciledRestore));
  configured = await configuredConnect(runtimeOptions);
  const restoredRecord = await success(configured, "memory_read", { ...selectedArgs(), record_id: pending.record.record_id, revision: null });
  assert.equal(restoredRecord.record.revision, 1, JSON.stringify(restoredRecord)); assert.equal(restoredRecord.record.data.content, pending.record.data.content, JSON.stringify(restoredRecord));
  await rejected(configured, "memory_read", { ...selectedArgs(), record_id: committed.record.record_id, revision: null }, /found|unavailable|forgot/i);
  const withoutDerived = await recall(configured, "Quartz timing", selection);
  assert.ok(withoutDerived.coverage.semantic_missing_eligible > 0, JSON.stringify(withoutDerived));
  assert.ok(!withoutDerived.hits.some((hit) => hit.record.record_id === pending.record.record_id && hit.record.revision === 2), JSON.stringify(withoutDerived));
  const restoreIndex = await refresh(configured, { limit: 32, max_bytes: 262144 });
  assert.equal(restoreIndex.status, "page_complete", JSON.stringify(restoreIndex)); assert.equal(restoreIndex.coverage.missing_heads, 0, JSON.stringify(restoreIndex));
  assert.ok(restoreIndex.indexed.some((ref) => ref.record_id === pending.record.record_id && ref.revision === 1), JSON.stringify(restoreIndex));
  assert.ok(!restoreIndex.indexed.some((ref) => ref.record_id === committed.record.record_id), JSON.stringify(restoreIndex));
  const recalledRestore = await recall(configured, "Quartz timing", selection);
  assert.equal(recalledRestore.coverage.semantic_missing_eligible, 0, JSON.stringify(recalledRestore));
  assert.ok(recalledRestore.hits.some((hit) => hit.record.record_id === pending.record.record_id && hit.record.revision === 1), JSON.stringify(recalledRestore));
  await configured.stop();
  console.log("PASS actual maintenance restore confirms exact operation, restores canonical revision and requires explicit derived-index rebuild");
  const jobNames = await readdir(join(configuredState, "jobs"));
  assert.deepEqual(jobNames, [], "Completed configured leaves must leave no job scratch");
  console.log("PASS configured runtime leaves no completed job scratch");
}


// Manual interchange is independent of embedding configuration and executes no
// model. Every maintenance command runs after this journey's client has stopped.
async function manualTransferJourney() {
  const paths = Object.fromEntries(["state", "project", "other-project", "backups", "external"]
    .map((name) => [name, join(temporary, `transfer-${name}`)]));
  for (const path of Object.values(paths)) await mkdir(path, { mode: 0o700 });
  for (const name of Object.keys(paths)) paths[name] = await realpath(paths[name]);
  const selection = projectScope(paths.project);
  const otherSelection = projectScope(paths["other-project"]);
  const open = async () => {
    const client = new Client(paths.state, RELEASE_A, { backupDir: paths.backups });
    clients.push(client);
    return client.initialize();
  };
  const admin = async (command, extra = [], rejection = null, selectedProject = paths.project) => {
    const argv = ["dev", "memory", command, "--state-dir", paths.state,
      "--backup-dir", paths.backups, "--release-id", RELEASE_A,
      "--project-root", selectedProject, ...extra];
    return new Promise((resolve, reject) => {
      const child = spawn(flags.get("--mirket"), argv, { stdio: ["ignore", "pipe", "pipe"] });
      let stdout = "", stderr = "", exceeded = false;
      const timer = setTimeout(() => { exceeded = true; child.kill("SIGKILL"); }, 150000);
      const collect = (name, part) => {
        if (exceeded) return;
        if (name === "stdout") stdout += part; else stderr += part;
        if (Buffer.byteLength(stdout) > MAX_FRAME || Buffer.byteLength(stderr) > MAX_FRAME) {
          exceeded = true; child.kill("SIGKILL");
        }
      };
      child.stdout.setEncoding("utf8").on("data", (part) => collect("stdout", part));
      child.stderr.setEncoding("utf8").on("data", (part) => collect("stderr", part));
      child.on("error", (error) => { clearTimeout(timer); reject(error); });
      child.on("close", (code, signal) => {
        clearTimeout(timer);
        try {
          assert.equal(exceeded, false, `memory ${command} exceeded fixture bounds`);
          assert.equal(signal, null, `memory ${command} was signalled`);
          if (rejection) {
            assert.notEqual(code, 0, `memory ${command} unexpectedly succeeded`);
            assert.match(stderr, rejection);
            resolve(null);
          } else {
            assert.equal(code, 0, `memory ${command}: ${stderr}`);
            resolve(JSON.parse(stdout));
          }
        } catch (error) { reject(error); }
      });
    });
  };
  const byId = (receipt) => ["--operation-id", receipt.operation_id];
  const history = (client, id) => success(client, "memory_history", { ...args(selection),
    record_id: id, before_revision: null, limit: 20, max_bytes: 32768 });
  const revise = async (client, id, revision, data) => {
    const intent = await success(client, "memory_prepare", { ...args(selection),
      mutation: { kind: "revise", record_id: id, expected_revision: revision, record: data } });
    return success(client, "memory_commit", operation(intent, selection));
  };
  let client = await open();
  const initial = record("Checkpoint: inspect the saved calibration source before resuming; submission outcome remains unknown.", {
    sources: [
      { source_id: "fixture:calibration", revision: "revision-1", locator: "unknown+fixture://calibration/line/2", captured_at: 123 },
      { source_id: "fixture:operator", revision: "revision-3", locator: "unknown+fixture://operator/line/7", captured_at: 124 },
    ],
  });
  const saved = await save(client, initial, selection);
  const checkpoint = await revise(client, saved.record.record_id, 1, { ...initial,
    content: "Checkpoint: calibration source inspected; confirm the unknown submission before any retry.", basis: "user_stated" });
  const other = await save(client, record("Independent scope must survive transfer maintenance."), otherSelection);
  const expectedHistory = (await history(client, saved.record.record_id)).records;
  assert.deepEqual(expectedHistory.map((r) => r.revision), [2, 1]);
  await client.stop();

  const archive = join(paths.external, "checkpoint.jsonl");
  const exportIntent = await admin("export-prepare", ["--path", archive]);
  assert.equal(exportIntent.receipt_phase, "prepared");
  await admin("transfer-retire", byId(exportIntent), /expired/i);
  await admin("transfer-reconcile", byId(exportIntent), /cannot start|prepared/i);
  const exported = await admin("transfer-commit", byId(exportIntent));
  assert.equal(exported.complete, true); assert.equal(exported.confirmed, true);
  assert.equal((await admin("transfer-status", byId(exportIntent))).complete, true);
  assert.equal((await admin("transfer-reconcile", byId(exportIntent))).complete, true);
  const archiveBytes = await readFile(archive);
  const archiveHash = createHash("sha256").update(archiveBytes).digest("hex");
  assert.equal(exported.authority.sha256, archiveHash);
  assert.equal(exported.authority.stats.revisions, 2);
  assert.equal(exported.authority.stats.records, 1);
  if (process.platform !== "win32") assert.equal((await stat(archive)).mode & 0o777, 0o600);
  const header = JSON.parse(archiveBytes.toString("utf8").split("\n")[0]);
  assert.equal(header.type, "header");

  // A destination created after prepare must survive the attempted publication.
  const occupied = join(paths.external, "occupied.jsonl");
  const occupiedIntent = await admin("export-prepare", ["--path", occupied]);
  const sentinel = "fixture-owned destination must remain intact\n";
  await writeFile(occupied, sentinel, { flag: "wx", mode: 0o600 });
  await admin("transfer-commit", byId(occupiedIntent), /exist|absent|destination/i);
  assert.equal(await readFile(occupied, "utf8"), sentinel);
  assert.equal((await admin("transfer-status", byId(occupiedIntent))).receipt_phase, "prepared");

  const drift = join(paths.external, "drift.jsonl");
  await writeFile(drift, archiveBytes, { flag: "wx", mode: 0o600 });
  const driftIntent = await admin("import-prepare", ["--path", drift]);
  await writeFile(drift, Buffer.concat([archiveBytes, Buffer.from("changed\n")]));
  await admin("transfer-commit", byId(driftIntent), /changed|checksum|identity/i);
  assert.equal((await admin("transfer-status", byId(driftIntent))).receipt_phase, "prepared");
  await admin("transfer-reconcile", byId(driftIntent), /cannot start|prepared/i);

  const wrongScope = await admin("import-prepare", ["--path", archive], null, paths["other-project"]);
  await admin("transfer-commit", byId(wrongScope), /scope/i, paths["other-project"]);
  const wrongReconcile = await admin("transfer-reconcile", byId(wrongScope), null, paths["other-project"]);
  assert.equal(wrongReconcile.receipt_phase, "invalidated");
  assert.equal(wrongReconcile.complete, false);
  console.log("PASS manual transfer binds private external bytes and scope; drift and occupied destinations refuse without overwriting");

  client = await open();
  const later = await revise(client, saved.record.record_id, 2, { ...initial, content: "Later canonical checkpoint after the exported recovery point." });
  assert.equal(later.record.revision, 3);
  await client.stop();
  const importIntent = await admin("import-prepare", ["--path", archive]);
  const imported = await admin("transfer-commit", byId(importIntent));
  assert.equal(imported.complete, true); assert.equal(imported.confirmed, true);
  assert.equal(imported.authority.eligible, true);
  assert.equal((await admin("transfer-status", byId(importIntent))).complete, true);
  assert.equal((await admin("transfer-reconcile", byId(importIntent))).complete, true);
  const snapshotId = imported.authority.snapshot_id;
  assert.match(snapshotId, /^[a-zA-Z0-9-]+$/);
  assert.ok((await admin("backups")).some((entry) => entry.snapshot_id === snapshotId && entry.eligible));
  const { DatabaseSync } = await import("node:sqlite");
  const snapshotDb = new DatabaseSync(join(paths.backups, `${snapshotId}.sqlite3`), { readOnly: true });
  try {
    assert.equal(snapshotDb.prepare("SELECT count(*) AS n FROM mutations").get().n, 0);
    assert.equal(snapshotDb.prepare("SELECT count(*) AS n FROM metadata WHERE key='last_restore_operation'").get().n, 0);
    assert.equal(snapshotDb.prepare("SELECT count(*) AS n FROM semantic_chunks").get().n, 0);
    assert.deepEqual(snapshotDb.prepare("SELECT revision FROM revisions ORDER BY revision").all().map((r) => r.revision), [1, 2]);
  } finally { snapshotDb.close(); }
  client = await open();
  const unchanged = await success(client, "memory_read", { ...args(selection), record_id: saved.record.record_id, revision: null });
  assert.deepEqual(unchanged.record, later.record, "Import must not activate or replace live canonical memory");
  await client.stop();

  const restore = await admin("restore-prepare", ["--snapshot-id", snapshotId]);
  await admin("restore-retire", byId(restore), /not expired|expired/i);
  assert.equal((await admin("restore-commit", byId(restore))).complete, true);
  assert.equal((await admin("restore-status", byId(restore))).complete, true);
  assert.equal((await admin("restore-reconcile", byId(restore))).complete, true);
  // Native SQLite inspection uses the already admitted Node runtime, read-only.
  let inspected = 0;
  for (const name of await readdir(join(paths.state, "memory"))) {
    if (!/^scope-.*\.sqlite3$/.test(name)) continue;
    const database = new DatabaseSync(join(paths.state, "memory", name), { readOnly: true });
    try {
      const scope = JSON.parse(database.prepare("SELECT value FROM metadata WHERE key='scope'").get().value);
      if (JSON.stringify(scope) !== JSON.stringify(header.scope)) continue;
      inspected++;
      assert.equal(database.prepare("SELECT count(*) AS n FROM mutations").get().n, 0,
        "External canonical records must not import historical operation confirmations");
      assert.equal(database.prepare("SELECT count(*) AS n FROM semantic_chunks").get().n, 0);
    } finally { database.close(); }
  }
  assert.equal(inspected, 1);
  client = await open();
  assert.deepEqual((await history(client, saved.record.record_id)).records, expectedHistory);
  const recovered = await success(client, "memory_read", { ...args(selection), record_id: saved.record.record_id, revision: null });
  assert.deepEqual(recovered.record, checkpoint.record);
  const preservedOther = await success(client, "memory_read", { ...args(otherSelection), record_id: other.record.record_id, revision: null });
  assert.deepEqual(preservedOther.record, other.record);
  console.log("PASS import admits an eligible recovery point without activation; explicit restore preserves full history/provenance and imports no mutation authority");

  const forget = await success(client, "memory_forget_prepare", { ...args(selection), record_id: saved.record.record_id, expected_revision: 2 });
  const forgotten = await success(client, "memory_forget_commit", operation(forget, selection));
  assert.equal(forgotten.status, "forgotten"); assert.equal(forgotten.forget.cleanup_complete, true);
  await client.stop();
  assert.equal(createHash("sha256").update(await readFile(archive)).digest("hex"), archiveHash,
    "External archive remains user-owned after forgetting");
  const forbidden = await admin("import-prepare", ["--path", archive]);
  await admin("transfer-commit", byId(forbidden), /forgot|exclu|ineligible|recover/i);
  const invalidated = await admin("transfer-reconcile", byId(forbidden));
  assert.equal(invalidated.receipt_phase, "invalidated");
  assert.equal(invalidated.complete, false); assert.equal(invalidated.confirmed, false);
  await admin("transfer-commit", byId(forbidden), /started|unknown|inspect|reconcile/i);
  assert.equal((await admin("transfer-status", byId(forbidden))).receipt_phase, "invalidated");
  client = await open();
  await rejected(client, "memory_read", { ...args(selection), record_id: saved.record.record_id, revision: null }, /found|unavailable|forgot/i);
  await client.stop();
  console.log("PASS old export cannot resurrect forgotten memory; consumed import reconciles invalidated, and fresh transfer/restore preparations cannot retire");
}
