// Disposable development or installed qualification for the selected private
// Sharp candidate. --home uses only that verified installation.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { createHash, randomBytes } from "node:crypto";
import { mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";
import { pathToFileURL } from "node:url";

const developmentRequired = ["--mirket", "--node", "--design-adapter", "--font", "--raster-module"];
const flags = new Map();
for (let index = 2; index < process.argv.length; index += 2) {
  const flag = process.argv[index], value = process.argv[index + 1];
  assert.ok((developmentRequired.includes(flag) || flag === "--home") && value && !flags.has(flag) && isAbsolute(value));
  flags.set(flag, value);
}
const installed = flags.has("--home");
const required = installed ? ["--home"] : developmentRequired;
assert.deepEqual([...flags.keys()].sort(), [...required].sort());
let installedRelease;
if (installed) {
  const home = flags.get("--home");
  assert.ok((await stat(home)).isDirectory());
  const binary = join(home, "bin/mirket");
  const checked = spawnSync(binary, ["dev", "doctor-core", "--home", home, "--json"],
    { encoding: "utf8", timeout: 30_000 });
  assert.equal(checked.status, 0, checked.stderr);
  const doctor = JSON.parse(checked.stdout);
  assert.equal(doctor.status, "healthy");
  assert.ok(doctor.active_components.includes("design") && doctor.active_components.includes("raster"));
  installedRelease = doctor.active_release_id;
  assert.match(installedRelease, /^sha256:[0-9a-f]{64}$/);
  const root = join(home, "releases", `r-${installedRelease.slice(7)}`);
  flags.set("--mirket", binary);
  flags.set("--raster-module", join(root, "runtime/design/raster/node_modules/sharp/dist/index.mjs"));
  for (const flag of ["--mirket", "--raster-module"]) {
    assert.ok((await stat(flags.get(flag))).isFile(), `Expected installed file ${flag}`);
  }
} else {
  for (const path of flags.values()) assert.ok((await stat(path)).isFile(), path);
}

const root = await mkdtemp(join(tmpdir(), "mirket-raster-mcp-"));
const project = join(root, "project");
const state = join(root, "state");
await Promise.all([mkdir(project, { mode: 0o700 }), mkdir(state, { mode: 0o700 })]);
const release = installedRelease ?? "raster-development-test";
const digest = bytes => `sha256:${createHash("sha256").update(bytes).digest("hex")}`;
const { default: sharp } = await import(pathToFileURL(flags.get("--raster-module")).href);
const source = await sharp({ create: { width: 2, height: 2, channels: 4,
  background: { r: 12, g: 120, b: 230, alpha: 0.5 } } }).png().toBuffer();
const input = join(project, "source.png"), output = join(project, "result.webp");
await writeFile(input, source);

const args = installed ? ["serve", "--home", flags.get("--home")] : ["dev", "serve", "--state-dir", state, "--release-id", release,
  "--node", flags.get("--node"),
  "--design-adapter", flags.get("--design-adapter"), "--design-font", flags.get("--font"),
  "--raster-module", flags.get("--raster-module")];
const child = spawn(flags.get("--mirket"), args, { stdio: ["pipe", "pipe", "pipe"] });
let serial = 0, buffer = "", stderr = "";
const pending = new Map();
const closed = new Promise((resolve, reject) => {
  child.on("error", reject);
  child.on("close", (code, signal) => {
    for (const waiter of pending.values()) {
      clearTimeout(waiter.timer);
      waiter.reject(new Error(`MCP server exited (${code ?? signal}): ${stderr.slice(-2000)}`));
    }
    pending.clear();
    resolve({ code, signal });
  });
});
child.stdout.setEncoding("utf8").on("data", chunk => {
  buffer += chunk;
  assert.ok(Buffer.byteLength(buffer) <= 1024 * 1024);
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
child.stderr.setEncoding("utf8").on("data", chunk => { stderr += chunk; });
function rpc(method, params) {
  const id = ++serial;
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(new Error(`${method} timed out: ${stderr.slice(-2000)}`));
    }, 60_000);
    pending.set(id, { resolve, reject, timer });
    child.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n");
  });
}
async function tool(name, arguments_, errorExpected = false) {
  const response = await rpc("tools/call", { name, arguments: arguments_ });
  assert.equal(response.isError === true, errorExpected, JSON.stringify(response));
  return errorExpected ? response : response.structuredContent ?? JSON.parse(response.content[0].text);
}
const prepare = (revision, destination, operation) => ({
  expected_release_id: release, project_root: project, source_path: input,
  expected_revision: revision, destination_path: destination, operation,
});
const operation = { kind: "raster_transform", source_format: "png", output_format: "webp",
  width: 8, height: 8, fit: "fill" };
let passed = false;
try {
  const init = await rpc("initialize", { protocolVersion: "2025-11-25", capabilities: {},
    clientInfo: { name: "raster-development-smoke", version: "0.0.1" } });
  assert.equal(init.protocolVersion, "2025-11-25");
  child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized", params: {} }) + "\n");
  const status = await tool("runtime_status", {});
  assert.equal(status.design.raster_configured, true);
  const stale = await tool("design_prepare", prepare(`sha256:${"0".repeat(64)}`, output, operation), true);
  assert.match(JSON.stringify(stale), /revision/i);
  const issued = await tool("design_prepare", prepare(digest(source), output, operation));
  await tool("design_start", { expected_release_id: release, project_root: project, operation_id: issued.operation_id });
  let terminal;
  for (let attempt = 0; attempt < 600; attempt++) {
    terminal = await tool("job_status", { expected_release_id: release, project_root: project,
      operation_id: issued.operation_id });
    if (["succeeded", "failed", "cancelled", "interrupted"].includes(terminal.status)
      && !terminal.scratch_cleanup?.startsWith("retained: terminal confirmation")) break;
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  assert.equal(terminal.status, "succeeded", JSON.stringify(terminal));
  assert.equal(terminal.scratch_cleanup, "complete");
  const bytes = await readFile(output);
  assert.equal(terminal.artifact_revision, digest(bytes));
  const decoded = await sharp(bytes).raw().toBuffer({ resolveWithObject: true });
  assert.equal(decoded.info.width, 8);
  assert.equal(decoded.info.height, 8);
  assert.equal(decoded.info.channels, 4);
  assert.ok(decoded.data[0] >= 5 && decoded.data[0] <= 25);
  assert.ok(decoded.data[3] >= 110 && decoded.data[3] <= 150);

  const jpegOutput = join(project, "result.jpg");
  const jpegIssued = await tool("design_prepare", prepare(digest(source), jpegOutput,
    { ...operation, output_format: "jpeg", width: 4, height: 4 }));
  await tool("design_start", { expected_release_id: release, project_root: project,
    operation_id: jpegIssued.operation_id });
  let jpegReceipt;
  for (let attempt = 0; attempt < 600; attempt++) {
    jpegReceipt = await tool("job_status", { expected_release_id: release, project_root: project,
      operation_id: jpegIssued.operation_id });
    if (["succeeded", "failed", "cancelled", "interrupted"].includes(jpegReceipt.status)
      && !jpegReceipt.scratch_cleanup?.startsWith("retained: terminal confirmation")) break;
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  assert.equal(jpegReceipt.status, "succeeded", JSON.stringify(jpegReceipt));
  const jpegSaved = await readFile(jpegOutput);
  assert.equal(jpegReceipt.artifact_revision, digest(jpegSaved));
  const jpegInfo = await sharp(jpegSaved).metadata();
  assert.equal(jpegInfo.format, "jpeg");
  assert.equal(jpegInfo.channels, 3);
  assert.equal(jpegInfo.width, 4);
  assert.equal(jpegInfo.height, 4);

  const extraOutputBytes = {};
  for (const [format, extension, decodedFormat] of [
    ["png", "png", "png"], ["avif", "avif", "heif"],
  ]) {
    const destination = join(project, `result.${extension}`);
    const issued = await tool("design_prepare", prepare(digest(source), destination,
      { ...operation, output_format: format, width: 4, height: 4 }));
    await tool("design_start", { expected_release_id: release, project_root: project,
      operation_id: issued.operation_id });
    let receipt;
    for (let attempt = 0; attempt < 600; attempt++) {
      receipt = await tool("job_status", { expected_release_id: release, project_root: project,
        operation_id: issued.operation_id });
      if (["succeeded", "failed", "cancelled", "interrupted"].includes(receipt.status)
        && !receipt.scratch_cleanup?.startsWith("retained: terminal confirmation")) break;
      await new Promise(resolve => setTimeout(resolve, 50));
    }
    assert.equal(receipt.status, "succeeded", JSON.stringify(receipt));
    assert.equal(receipt.scratch_cleanup, "complete");
    const saved = await readFile(destination);
    assert.equal(receipt.artifact_revision, digest(saved));
    const info = await sharp(saved).metadata();
    assert.equal(info.format, decodedFormat);
    assert.equal(info.width, 4);
    assert.equal(info.height, 4);
    if (format === "avif") assert.equal(info.compression, "av1");
    extraOutputBytes[format] = saved.length;
  }

  const large = await sharp(randomBytes(1024 * 1024 * 3), {
    raw: { width: 1024, height: 1024, channels: 3 },
  }).png({ compressionLevel: 0 }).toBuffer();
  assert.ok(large.length > 1024 * 1024 && large.length < 16 * 1024 * 1024);
  const largeInput = join(project, "large.png"), largeOutput = join(project, "large.webp");
  await writeFile(largeInput, large);
  const largeIssued = await tool("design_prepare", {
    ...prepare(digest(large), largeOutput, { ...operation, width: 256, height: 256, fit: "inside" }),
    source_path: largeInput,
  });
  await tool("design_start", { expected_release_id: release, project_root: project,
    operation_id: largeIssued.operation_id });
  let largeReceipt;
  for (let attempt = 0; attempt < 600; attempt++) {
    largeReceipt = await tool("job_status", { expected_release_id: release, project_root: project,
      operation_id: largeIssued.operation_id });
    if (["succeeded", "failed", "cancelled", "interrupted"].includes(largeReceipt.status)
      && !largeReceipt.scratch_cleanup?.startsWith("retained: terminal confirmation")) break;
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  assert.equal(largeReceipt.status, "succeeded", JSON.stringify(largeReceipt));
  const largeSaved = await readFile(largeOutput);
  assert.equal(largeReceipt.artifact_revision, digest(largeSaved));
  const largeInfo = await sharp(largeSaved).metadata();
  assert.equal(largeInfo.width, 256);
  assert.equal(largeInfo.height, 256);
  child.stdin.end();
  assert.equal((await closed).code, 0, stderr);
  passed = true;
  console.log(JSON.stringify({ status: "passed", release, source_revision: digest(source),
    output_revision: digest(bytes), output_bytes: bytes.length,
    jpeg_output_bytes: jpegSaved.length, extra_output_bytes: extraOutputBytes,
    large_input_bytes: large.length,
    large_output_bytes: largeSaved.length }));
} finally {
  if (!passed) {
    child.stdin.end();
    try { await Promise.race([closed, new Promise(resolve => setTimeout(resolve, 3000))]); } catch {}
    if (child.exitCode === null) child.kill("SIGKILL");
    console.error(`Retained failed raster fixture: ${root}`);
  } else await rm(root, { recursive: true, force: true });
}
