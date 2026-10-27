// Exercise the installed private design payload through actual MCP prepare/start jobs.
import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

assert.equal(process.argv.length, 4, "Usage: node tests/design-installed-smoke.mjs --home /absolute/disposable-home");
assert.equal(process.argv[2], "--home");
const home = process.argv[3];
assert.ok(isAbsolute(home) && (await stat(home)).isDirectory());
const binary = join(home, "bin/mirket");
const doctor = JSON.parse(execFileSync(binary, ["dev", "doctor-core", "--home", home, "--json"],
  { encoding: "utf8", timeout: 30000, stdio: ["ignore", "pipe", "pipe"] }));
assert.equal(doctor.status, "healthy");
assert.ok(doctor.active_components.includes("design"));
const release = doctor.active_release_id;
const project = await mkdtemp(join(tmpdir(), "mirket-design-installed-"));
const hash = value => createHash("sha256").update(value).digest("hex");
const rev = value => `sha256:${hash(value)}`;

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
    const timer = setTimeout(() => reject(new Error(`${method} timed out: ${stderr}`)), 45000);
    pending.set(id, { resolve, reject, timer });
    child.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n");
  });
}
async function tool(name, args = {}, expectError = false) {
  const result = await rpc("tools/call", { name, arguments: args });
  assert.equal(result.isError === true, expectError, `${name}: ${JSON.stringify(result)}`);
  if (expectError) return JSON.stringify(result);
  return result.structuredContent ?? JSON.parse(result.content.find(item => item.type === "text").text);
}
async function terminal(operation) {
  const end = Date.now() + 45000;
  while (Date.now() < end) {
    const result = await tool("job_status", { expected_release_id: release, project_root: project,
      operation_id: operation.operation_id });
    if (["succeeded", "failed", "cancelled", "interrupted"].includes(result.status)
      && !result.scratch_cleanup?.startsWith("retained: terminal confirmation")) return result;
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  throw new Error("Design operation did not reach a terminal receipt");
}
async function execute(sourceName, sourceBytes, destinationName, operation) {
  const source = join(project, sourceName), destination = join(project, destinationName);
  await writeFile(source, sourceBytes, { mode: 0o600 });
  const prepared = await tool("design_prepare", {
    expected_release_id: release, project_root: project, source_path: source,
    expected_revision: rev(sourceBytes), destination_path: destination, operation,
  });
  await tool("design_start", { expected_release_id: release, project_root: project,
    operation_id: prepared.operation_id });
  const result = await terminal(prepared);
  return { result, destination };
}

let passed = false;
try {
  const initialized = await rpc("initialize", { protocolVersion: "2025-11-25", capabilities: {},
    clientInfo: { name: "design-installed-smoke", version: "0.0.1" } });
  assert.equal(initialized.protocolVersion, "2025-11-25");
  child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized", params: {} }) + "\n");
  const listing = await rpc("tools/list", {});
  assert.ok(listing.tools.some(entry => entry.name === "design_prepare"));
  assert.ok(listing.tools.some(entry => entry.name === "design_start"));
  const status = await tool("runtime_status");
  assert.equal(status.release_id, release);
  assert.equal(status.design.configured, true);

  const tokens = Buffer.from(JSON.stringify({ color: { brand: { value: "#1463ff" } } }));
  const css = await execute("tokens.json", tokens, "tokens.css", { kind: "tokens_compile", format: "css" });
  assert.equal(css.result.status, "succeeded", JSON.stringify(css.result));
  assert.equal(css.result.scratch_cleanup, "complete");
  const cssBytes = await readFile(css.destination);
  assert.match(cssBytes.toString(), /--color-brand: #1463ff;/);
  assert.equal(css.result.artifact_revision, rev(cssBytes));

  const svg = Buffer.from('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 40"><rect width="100" height="40" fill="#1463ff"/><text x="8" y="27" fill="#fff" font-family="Noto Sans" font-size="20">Mirket</text></svg>');
  const optimized = await execute("vector.svg", svg, "vector.min.svg", { kind: "svg_optimize" });
  assert.equal(optimized.result.status, "succeeded", JSON.stringify(optimized.result));
  assert.match((await readFile(optimized.destination)).toString(), /<svg/);

  const png = await execute("art.svg", svg, "art.png", { kind: "svg_render_png", width: 400, height: 160 });
  assert.equal(png.result.status, "succeeded", JSON.stringify(png.result));
  assert.equal(png.result.scratch_cleanup, "complete");
  const pixels = await readFile(png.destination);
  assert.deepEqual([...pixels.subarray(0, 8)], [137, 80, 78, 71, 13, 10, 26, 10]);
  assert.equal(pixels.readUInt32BE(16), 400);
  assert.equal(pixels.readUInt32BE(20), 160);
  assert.ok(pixels.length > 2000, "private font glyphs should contribute visible pixels");
  assert.equal(png.result.artifact_revision, rev(pixels));

  const unsafe = Buffer.from('<svg xmlns="http://www.w3.org/2000/svg"><image href="https://example.test/a.png"/></svg>');
  const refused = await execute("unsafe.svg", unsafe, "unsafe.min.svg", { kind: "svg_optimize" });
  assert.equal(refused.result.status, "failed", JSON.stringify(refused.result));
  await assert.rejects(stat(refused.destination), { code: "ENOENT" });
  const refusedRaster = await execute("unsafe-render.svg", unsafe, "unsafe-render.png",
    { kind: "svg_render_png", width: 100, height: 40 });
  assert.equal(refusedRaster.result.status, "failed", JSON.stringify(refusedRaster.result));
  assert.equal(refusedRaster.result.scratch_cleanup, "complete");
  await assert.rejects(stat(refusedRaster.destination), { code: "ENOENT" });
  assert.match(await tool("design_prepare", { expected_release_id: "stale", project_root: project,
    source_path: join(project, "vector.svg"), expected_revision: rev(svg),
    destination_path: join(project, "stale.min.svg"), operation: { kind: "svg_optimize" } }, true), /release/i);
  child.stdin.end();
  assert.equal((await closed).code, 0, stderr);
  passed = true;
  console.log(JSON.stringify({ status: "passed", release_id: release, components: doctor.active_components,
    css_bytes: cssBytes.length, png_bytes: pixels.length }));
} finally {
  if (!passed) {
    child.stdin.end();
    try { await Promise.race([closed, new Promise(resolve => setTimeout(resolve, 3000))]); } catch {}
    if (child.exitCode === null) child.kill("SIGKILL");
    console.error(`Retained failed design fixture: ${project}`);
  } else await rm(project, { recursive: true, force: true });
}
