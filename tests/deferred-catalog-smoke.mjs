// Exercise an installed launcher's early catalog and verification-before-call gate.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { performance } from "node:perf_hooks";
import { readdir, stat } from "node:fs/promises";
import { join, isAbsolute } from "node:path";

const args = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  assert.ok(process.argv[i]?.startsWith("--") && process.argv[i + 1] && !args.has(process.argv[i]));
  args.set(process.argv[i], process.argv[i + 1]);
}
const required = ["--expect", "--home", "--release-id"];
assert.ok(required.every(key => args.has(key)));
assert.ok([...args.keys()].every(key => [...required, "--binary"].includes(key)));
assert.ok(isAbsolute(args.get("--home")));
if (args.has("--binary")) assert.ok(isAbsolute(args.get("--binary")));
assert.ok(["ready", "concurrent_ready", "verification_failure", "catalog_only"].includes(args.get("--expect")));
assert.match(args.get("--release-id"), /^sha256:[a-f0-9]{64}$/);
const binary = args.get("--binary") ?? join(args.get("--home"), "bin", "mirket");
assert.ok((await stat(binary)).isFile());
const ownerDirectory = join(args.get("--home"), "state", "owners");
const owners = async () => (await readdir(ownerDirectory).catch((error) => {
  if (error.code === "ENOENT") return [];
  throw error;
})).sort();
const ownersBefore = ["catalog_only", "concurrent_ready"].includes(args.get("--expect")) ? await owners() : null;

const started = performance.now();
const child = spawn(binary, ["serve", "--home", args.get("--home")], { stdio: ["pipe", "pipe", "pipe"] });
let nextId = 0, buffer = "", stderr = "", outputBytes = 0;
const pending = new Map();
const exited = new Promise((resolve) => child.on("close", (code, signal) => resolve({ code, signal })));
child.on("error", (error) => {
  for (const request of pending.values()) request.reject(error);
  pending.clear();
});
child.stderr.setEncoding("utf8").on("data", (part) => { stderr += part; assert.ok(stderr.length < 16384); });
child.stdout.setEncoding("utf8").on("data", (part) => {
  outputBytes += Buffer.byteLength(part); assert.ok(outputBytes < 1024 * 1024);
  buffer += part;
  for (;;) {
    const end = buffer.indexOf("\n");
    if (end < 0) break;
    const frame = JSON.parse(buffer.slice(0, end)); buffer = buffer.slice(end + 1);
    if (frame.id === undefined) continue;
    const request = pending.get(frame.id);
    assert.ok(request, `Unexpected MCP response ${frame.id}`);
    pending.delete(frame.id); clearTimeout(request.timer);
    frame.error ? request.reject(new Error(JSON.stringify(frame.error))) : request.resolve(frame.result);
  }
});
function rpc(method, params) {
  const id = ++nextId;
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`${method} timed out; stderr=${stderr}`)), 30000);
    pending.set(id, { resolve, reject, timer });
    child.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n");
  });
}
try {
  await rpc("initialize", { protocolVersion: "2025-11-25", capabilities: {}, clientInfo: { name: "deferred-catalog-smoke", version: "0.0.1" } });
  const initializeMs = Math.round(performance.now() - started);
  child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized", params: {} }) + "\n");
  const tools = await rpc("tools/list", {});
  const catalogMs = Math.round(performance.now() - started);
  assert.ok(tools.tools.some((tool) => tool.name === "runtime_status"));
  let callMs = null;
  if (args.get("--expect") !== "catalog_only") {
    const replies = args.get("--expect") === "concurrent_ready"
      ? await Promise.all([rpc("tools/call", { name: "runtime_status", arguments: {} }),
        rpc("tools/call", { name: "runtime_status", arguments: {} })])
      : [await rpc("tools/call", { name: "runtime_status", arguments: {} })];
    callMs = Math.round(performance.now() - started);
    for (const answer of replies) {
      if (args.get("--expect") === "verification_failure") {
        assert.equal(answer.isError, true);
        assert.match(answer.content.find((item) => item.type === "text").text, /installed release verification failed/u);
      } else {
        assert.notEqual(answer.isError, true, JSON.stringify(answer));
        const runtime = answer.structuredContent ?? JSON.parse(answer.content.find((item) => item.type === "text").text);
        assert.equal(runtime.release_id, args.get("--release-id"));
      }
    }
  }
  console.log(JSON.stringify({ status: "pass", expect: args.get("--expect"), initialize_ms: initializeMs,
    catalog_ms: catalogMs, first_call_ms: callMs, tools: tools.tools.length }));
} finally {
  child.stdin.end();
  const timer = setTimeout(() => child.kill("SIGKILL"), 3000);
  await exited; clearTimeout(timer);
  if (args.get("--expect") === "catalog_only")
    assert.deepEqual(await owners(), ownersBefore, "catalog-only connection opened writable engine state");
  if (args.get("--expect") === "concurrent_ready")
    assert.equal((await owners()).length, ownersBefore.length + 1, "concurrent first calls opened multiple engines");
}
