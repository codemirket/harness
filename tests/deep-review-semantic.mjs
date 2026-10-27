// Held-out real-model memory journey. Run with explicit disposable paths:
// node tests/deep-review-semantic.mjs --mirket /abs/mirket --node /abs/node --embedding-package /abs/semantic
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdtemp, mkdir, rm, stat } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const flags = new Map();
for (let i = 2; i < process.argv.length; i += 2) flags.set(process.argv[i], process.argv[i + 1]);
for (const key of ["--mirket", "--node", "--embedding-package"]) {
  assert.ok(flags.has(key) && flags.get(key)?.startsWith("/"), `missing absolute ${key}`);
  assert.ok((await stat(flags.get(key))).isFile() || key === "--embedding-package");
}
assert.ok((await stat(flags.get("--embedding-package"))).isDirectory());

const root = await mkdtemp(join(tmpdir(), "mirket-heldout-semantic-"));
const state = join(root, "state"), projectA = join(root, "project-a"), projectB = join(root, "project-b");
await Promise.all([state, projectA, projectB].map((path) => mkdir(path, { mode: 0o700 })));
const release = "m2-heldout-semantic";
const child = spawn(flags.get("--mirket"), ["dev", "serve", "--state-dir", state,
  "--release-id", release, "--node", flags.get("--node"), "--embedding-package", flags.get("--embedding-package")],
  { stdio: ["pipe", "pipe", "pipe"] });
const exited = new Promise((resolve) => child.once("close", (code, signal) => resolve({ code, signal })));
let buffer = "", next = 0, stderr = ""; const pending = new Map();
child.stderr.setEncoding("utf8").on("data", (part) => { stderr += part; });
child.stdout.setEncoding("utf8").on("data", (part) => {
  buffer += part;
  for (;;) {
    const end = buffer.indexOf("\n"); if (end < 0) break;
    const message = JSON.parse(buffer.slice(0, end)); buffer = buffer.slice(end + 1);
    if (message.id === undefined) continue;
    const waiter = pending.get(message.id); if (!waiter) continue;
    pending.delete(message.id); clearTimeout(waiter.timer);
    if (message.error) waiter.reject(new Error(JSON.stringify(message.error))); else waiter.resolve(message.result);
  }
});
function rpc(method, params) {
  const id = ++next;
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => { pending.delete(id); reject(new Error(`${method} timeout: ${stderr}`)); }, 120000);
    pending.set(id, { resolve, reject, timer });
    child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", id, method, params })}\n`);
  });
}
function body(result) {
  if (result.structuredContent !== undefined) return result.structuredContent;
  return JSON.parse(result.content.find((part) => part.type === "text").text);
}
async function tool(name, args) {
  const result = await rpc("tools/call", { name, arguments: args });
  assert.notEqual(result.isError, true, `${name}: ${JSON.stringify(result)}`);
  return body(result);
}
const scope = (project_root) => ({ kind: "project", project_root });
const record = (content) => ({ kind: "fact", basis: "observed", content,
  sources: [{ source_id: "fixture:heldout", revision: "1", locator: "heldout:1", captured_at: 123 }],
  status: "current", critical: false, applicability: { task_key: null, checkout_id: null, operation: null },
  valid_from: null, valid_until: null });
async function save(selected, content) {
  const common = { expected_release_id: release, scope: selected };
  const prepared = await tool("memory_prepare", { ...common,
    mutation: { kind: "create", record: record(content), supersedes: null } });
  return tool("memory_commit", { ...common, operation_id: prepared.operation_id });
}
async function search(selected, query) {
  return tool("memory_search", { expected_release_id: release, scope: selected,
    request: { query, status: "both", context: { task_key: null, checkout_id: null, operation: null, at_time: null }, limit: 5, max_bytes: 32768 } });
}
let passed = false;
try {
  const initialized = await rpc("initialize", { protocolVersion: "2025-11-25", capabilities: {}, clientInfo: { name: "heldout-semantic", version: "0.0.1" } });
  assert.equal(initialized.protocolVersion, "2025-11-25");
  child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized", params: {} }) + "\n");
  const a = scope(projectA), b = scope(projectB);
  const firstSaveStarted = performance.now();
  const aNine = await save(a, "The observatory telescope battery lasts nine hours before recharge.");
  const firstSaveMs = performance.now() - firstSaveStarted;
  const aHarbor = await save(a, "The harbor beacon battery is inspected every Tuesday.");
  const aDistractors = [];
  for (const content of [
    "The observatory telescope lens is cleaned before each launch.",
    "The observatory radio transmits weather data at noon.",
    "The telescope mount is locked during high winds.",
    "The observatory archive retains calibration logs for seven years.",
    "The night crew records the battery voltage after every observation.",
    "The backup generator is tested on the first Monday of the month.",
  ]) aDistractors.push(await save(a, content));
  const bTwo = await save(b, "The observatory telescope battery lasts two hours before recharge.");
  for (const saved of [aNine, aHarbor, ...aDistractors, bTwo]) assert.equal(saved.semantic_index.status, "indexed", JSON.stringify(saved));

  const recallStarted = performance.now();
  const aRecall = await search(a, "What is the operating duration of the observatory telescope's power source?");
  const warmRecallMs = performance.now() - recallStarted;
  assert.equal(aRecall.coverage.semantic_available, true, JSON.stringify(aRecall));
  assert.equal(aRecall.hits[0].record.record_id, aNine.record.record_id, JSON.stringify(aRecall));
  assert.ok(!aRecall.hits.some((hit) => hit.record.record_id === bTwo.record.record_id), "cross-project record leaked into A");
  const bRecall = await search(b, "What is the operating duration of the observatory telescope's power source?");
  assert.equal(bRecall.hits[0].record.record_id, bTwo.record.record_id, JSON.stringify(bRecall));
  assert.ok(!bRecall.hits.some((hit) => hit.record.record_id === aNine.record.record_id), "cross-project record leaked into B");

  const forgetPrepared = await tool("memory_forget_prepare", { expected_release_id: release, scope: a,
    record_id: aNine.record.record_id, expected_revision: 1 });
  const forgotten = await tool("memory_forget_commit", { expected_release_id: release, scope: a, operation_id: forgetPrepared.operation_id });
  assert.equal(forgotten.status, "forgotten", JSON.stringify(forgotten));
  const afterForget = await search(a, "What is the operating duration of the observatory telescope's power source?");
  assert.ok(!afterForget.hits.some((hit) => hit.record.record_id === aNine.record.record_id), JSON.stringify(afterForget));
  assert.equal(afterForget.coverage.semantic_missing_eligible, 0, JSON.stringify(afterForget));
  const bStill = await search(b, "What is the operating duration of the observatory telescope's power source?");
  assert.equal(bStill.hits[0].record.record_id, bTwo.record.record_id, JSON.stringify(bStill));
  assert.ok(!bStill.hits.some((hit) => hit.record.record_id === aNine.record.record_id));
  console.log(JSON.stringify({ status: "passed", release, platform: process.platform, arch: process.arch,
    timings: { first_save_including_cold_index_ms: firstSaveMs, warm_recall_ms: warmRecallMs,
      samples_per_measure: 1, qualification: "end-to-end disposable debug journey; not isolated inference or a latency benchmark" },
    project_a: { expected: aNine.record.record_id, distractors: aDistractors.length, harbor: aHarbor.record.record_id, after_forget_hits: afterForget.hits.length },
    project_b: { expected: bTwo.record.record_id, hits: bStill.hits.length },
    oracle: "held-out English paraphrase; same-content distractor isolated by project scope; forgotten vector absent" }));
  passed = true;
} finally {
  child.stdin.end();
  const closed = await Promise.race([
    exited,
    new Promise((resolve) => setTimeout(() => resolve(null), 10000)),
  ]);
  if (!closed) {
    child.kill("SIGKILL");
    await Promise.race([exited, new Promise((resolve) => setTimeout(resolve, 2000))]);
  }
  if (passed) await rm(root, { recursive: true, force: true });
  else console.error(`Retained failed semantic fixture: ${root}`);
}
