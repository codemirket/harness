// Verify the design, diagram and memory components coexist in a disposable release.
import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import { stat } from "node:fs/promises";
import { isAbsolute, join } from "node:path";

assert.equal(process.argv.length, 4, "Usage: node tests/combined-installed-smoke.mjs --home /absolute/disposable-home");
assert.equal(process.argv[2], "--home");
const home = process.argv[3];
assert.ok(isAbsolute(home) && (await stat(home)).isDirectory());
const binary = join(home, "bin/mirket");
assert.ok((await stat(binary)).isFile());

const doctor = JSON.parse(execFileSync(binary, ["dev", "doctor-core", "--home", home, "--json"], {
  encoding: "utf8", timeout: 30000, stdio: ["ignore", "pipe", "pipe"],
}));
assert.equal(doctor.status, "healthy");
assert.deepEqual(doctor.active_components, ["diagram", "design", "semantic"]);
assert.equal(doctor.active_target, "aarch64-apple-darwin");

const child = spawn(binary, ["serve", "--home", home], { stdio: ["pipe", "pipe", "pipe"] });
let nextId = 0, buffer = "", stderr = "";
const pending = new Map();
const closed = new Promise((resolve, reject) => {
  child.on("error", reject);
  child.on("close", (code, signal) => resolve({ code, signal }));
});
child.stdout.setEncoding("utf8").on("data", chunk => {
  buffer += chunk;
  assert.ok(Buffer.byteLength(buffer) <= 1024 * 1024, "MCP frame exceeded bound");
  for (;;) {
    const end = buffer.indexOf("\n");
    if (end < 0) break;
    const frame = JSON.parse(buffer.slice(0, end));
    buffer = buffer.slice(end + 1);
    const waiter = pending.get(frame.id);
    if (!waiter) continue;
    pending.delete(frame.id);
    clearTimeout(waiter.timer);
    if (frame.error) waiter.reject(new Error(JSON.stringify(frame.error)));
    else waiter.resolve(frame.result);
  }
});
child.stderr.setEncoding("utf8").on("data", chunk => {
  stderr += chunk;
  assert.ok(Buffer.byteLength(stderr) <= 64 * 1024, "MCP diagnostics exceeded bound");
});
function rpc(method, params) {
  const id = ++nextId;
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`${method} timed out: ${stderr}`)), 30000);
    pending.set(id, { resolve, reject, timer });
    child.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n");
  });
}
async function tool(name, args = {}) {
  const result = await rpc("tools/call", { name, arguments: args });
  assert.notEqual(result.isError, true, JSON.stringify(result));
  return result.structuredContent ?? JSON.parse(result.content.find(item => item.type === "text").text);
}

try {
  const initialized = await rpc("initialize", {
    protocolVersion: "2025-11-25", capabilities: {},
    clientInfo: { name: "combined-installed-smoke", version: "0.0.1" },
  });
  assert.equal(initialized.protocolVersion, "2025-11-25");
  child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized", params: {} }) + "\n");
  const tools = await rpc("tools/list", {});
  const names = new Set(tools.tools.map(entry => entry.name));
  for (const name of ["code_inspect", "code_navigate",
    "diagram_render_prepare", "diagram_render_start", "design_prepare", "design_start",
    "memory_commit", "memory_search", "memory_index_refresh"]) {
    assert.ok(names.has(name), `${name} missing from installed MCP`);
  }
  assert.ok(![...names].some(name => name === "sql_navigate" || name.startsWith("browser_")));
  const status = await tool("runtime_status");
  assert.equal(status.release_id, doctor.active_release_id);
  assert.equal(status.sql_navigation, undefined);
  assert.equal(status.browser, undefined);
  assert.equal(status.diagrams.configured, true);
  assert.equal(status.design.configured, true);
  assert.equal(status.memory_resources.embedding_configured, true);
  child.stdin.end();
  const exit = await closed;
  assert.equal(exit.code, 0, stderr);
  console.log(JSON.stringify({ status: "passed", target: doctor.active_target,
    release_id: doctor.active_release_id, components: doctor.active_components }));
} catch (error) {
  child.stdin.end();
  try { await Promise.race([closed, new Promise(resolve => setTimeout(resolve, 3000))]); } catch {}
  if (child.exitCode === null) child.kill("SIGKILL");
  throw error;
}
