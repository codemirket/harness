// Opt-in real Rust/MCP/worker/Node/OpenPencil qualification in disposable state.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtemp, mkdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

const args = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  assert.ok(process.argv[i]?.startsWith("--") && process.argv[i + 1] && !args.has(process.argv[i]));
  args.set(process.argv[i], process.argv[i + 1]);
}
const installed = args.has("--home");
const flags = installed ? ["--home", "--release-id"]
  : ["--mirket", "--node", "--openpencil-adapter", "--openpencil-runtime", "--openpencil-sha256"];
assert.deepEqual([...args.keys()].sort(), [...flags].sort());
if (installed) {
  assert.ok(isAbsolute(args.get("--home")));
  assert.ok((await stat(args.get("--home"))).isDirectory());
  assert.match(args.get("--release-id"), /^sha256:[a-f0-9]{64}$/);
} else {
  for (const flag of flags.slice(0, 4)) assert.ok(isAbsolute(args.get(flag)));
  for (const flag of flags.slice(0, 3)) assert.ok((await stat(args.get(flag))).isFile());
  assert.ok((await stat(args.get("--openpencil-runtime"))).isDirectory());
  assert.match(args.get("--openpencil-sha256"), /^[a-f0-9]{64}$/);
}

const sha = (bytes) => `sha256:${createHash("sha256").update(bytes).digest("hex")}`;
const temp = await mkdtemp(join(tmpdir(), "mirket-openpencil-mcp-"));
const state = join(temp, "state"), project = join(temp, "project");
const release = installed ? args.get("--release-id") : "openpencil-mcp-test";
await mkdir(state, { mode: 0o700 }); await mkdir(project, { mode: 0o700 });
const binary = installed ? join(args.get("--home"), "bin/mirket") : args.get("--mirket");
const serverArgs = installed ? ["serve", "--home", args.get("--home")]
  : ["dev", "serve", "--state-dir", state, "--release-id", release,
    "--node", args.get("--node"),
    "--openpencil-adapter", args.get("--openpencil-adapter"),
    "--openpencil-runtime", args.get("--openpencil-runtime"),
    "--openpencil-sha256", args.get("--openpencil-sha256")];
const child = spawn(binary, serverArgs,
  { stdio: ["pipe", "pipe", "pipe"] });
let nextId = 0, buffer = "", stderr = "", outputBytes = 0;
const pending = new Map();
const exited = new Promise((resolve) => child.on("close", (code, signal) => resolve({ code, signal })));
child.stderr.setEncoding("utf8").on("data", (part) => { stderr += part; assert.ok(stderr.length < 16384); });
child.stdout.setEncoding("utf8").on("data", (part) => {
  outputBytes += Buffer.byteLength(part); assert.ok(outputBytes < 1024 * 1024);
  buffer += part;
  for (;;) {
    const end = buffer.indexOf("\n"); if (end < 0) break;
    const frame = JSON.parse(buffer.slice(0, end)); buffer = buffer.slice(end + 1);
    if (frame.id === undefined) continue;
    const entry = pending.get(frame.id); assert.ok(entry, `unexpected response: ${frame.id}`);
    pending.delete(frame.id); clearTimeout(entry.timer);
    frame.error ? entry.reject(new Error(JSON.stringify(frame.error))) : entry.resolve(frame.result);
  }
});
function rpc(method, params) {
  const id = ++nextId;
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`${method} timed out; stderr=${stderr}`)), 15000);
    pending.set(id, { resolve, reject, timer });
    child.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n");
  });
}
async function call(name, value) {
  const answer = await rpc("tools/call", { name, arguments: value });
  assert.notEqual(answer.isError, true, `${name}: ${JSON.stringify(answer)}`);
  return answer.structuredContent ?? JSON.parse(answer.content.find((v) => v.type === "text").text);
}
async function terminal(operation_id) {
  const until = Date.now() + 15000;
  while (Date.now() < until) {
    const status = await call("job_status", { expected_release_id: release, project_root: project, operation_id });
    if (["succeeded", "failed", "cancelled", "interrupted"].includes(status.status)) return status;
    await new Promise((resolve) => setTimeout(resolve, 30));
  }
  throw new Error("design operation did not reach a terminal receipt");
}
try {
  await rpc("initialize", { protocolVersion: "2025-11-25", capabilities: {}, clientInfo: { name: "openpencil-mcp-smoke", version: "0.0.1" } });
  child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized", params: {} }) + "\n");
  const runtime = await call("runtime_status", {});
  assert.equal(runtime.design.openpencil_editable_file_configured, true);
  const source = join(project, "shapes.json"), output = join(project, "shapes.fig");
  const scene = Buffer.from(JSON.stringify({ page: "Components", nodes: [{ kind: "component", name: "Button", width: 200, height: 40, fill: "#1463ff", layout: "horizontal", gap: 8, padding_top: 8, padding_bottom: 8, padding_left: 12, padding_right: 12, counter_align: "center", children: [{ kind: "ellipse", name: "Icon", width: 20, height: 20, fill: "#ffffff" }, { kind: "text", name: "Label", width: 120, height: 24, text: "Send İ", font_size: 16, line_height: 24, fill: "#ffffff" }] }] }));
  await writeFile(source, scene);
  const prepare = (destination_path, expected_revision = sha(scene)) => ({
    expected_release_id: release, project_root: project, source_path: source, expected_revision,
    destination_path, operation: { kind: "open_pencil", operation: { kind: "create" } },
  });
  const issued = await call("design_prepare", prepare(output));
  await call("design_start", { expected_release_id: release, project_root: project, operation_id: issued.operation_id });
  const completed = await terminal(issued.operation_id);
  assert.equal(completed.status, "succeeded", JSON.stringify(completed));
  const saved = await readFile(output);
  assert.equal(saved.subarray(0, 2).toString(), "PK");
  assert.equal(completed.artifact_revision, sha(saved));
  assert.deepEqual(await readFile(source), scene);

  const patchedPath = join(project, "patched.fig");
  const patch = await call("design_prepare", {
    expected_release_id: release, project_root: project, source_path: output,
    expected_revision: sha(saved), destination_path: patchedPath,
    operation: { kind: "open_pencil", operation: { kind: "patch",
      edits: [{ path: "Button", properties: { gap: 16, width: 240 } }, { path: "Button/Label", properties: { text: "Sent İ", font_size: 18 } }] } },
  });
  await call("design_start", { expected_release_id: release, project_root: project, operation_id: patch.operation_id });
  const patchedReceipt = await terminal(patch.operation_id);
  assert.equal(patchedReceipt.status, "succeeded", JSON.stringify(patchedReceipt));
  const patched = await readFile(patchedPath);
  assert.notEqual(sha(patched), sha(saved));
  assert.deepEqual(await readFile(output), saved);

  const svgPath = join(project, "shape.svg");
  const exportIntent = await call("design_prepare", {
    expected_release_id: release, project_root: project, source_path: patchedPath,
    expected_revision: sha(patched), destination_path: svgPath,
    operation: { kind: "open_pencil", operation: { kind: "export_svg", target_path: "Button" } },
  });
  await call("design_start", { expected_release_id: release, project_root: project, operation_id: exportIntent.operation_id });
  const exportedReceipt = await terminal(exportIntent.operation_id);
  assert.equal(exportedReceipt.status, "succeeded", JSON.stringify(exportedReceipt));
  const svg = await readFile(svgPath, "utf8");
  assert.match(svg, /<svg/u);
  assert.match(svg, /width="240"/u);
  assert.match(svg, /translate\(48, 8\)/u);
  assert.match(svg, /font-family="Noto Sans"/u);
  assert.match(svg, />Sent İ<\/text>/u);
  assert.deepEqual(await readFile(patchedPath), patched);

  const stale = await rpc("tools/call", { name: "design_prepare", arguments: prepare(join(project, "stale.fig"), `sha256:${"0".repeat(64)}`) });
  assert.equal(stale.isError, true);
  await assert.rejects(stat(join(project, "stale.fig")), { code: "ENOENT" });
  console.log(JSON.stringify({ status: "pass", platform: process.platform, arch: process.arch,
    installation: installed ? "disposable_active_release" : "explicit_development_payload",
    release_id: release, saved_revision: sha(saved), saved_bytes: saved.length,
    patched_revision: sha(patched), svg_revision: sha(Buffer.from(svg)),
    checks: ["Rust MCP create/patch/export preparation, start, receipts and new-copy publication", "real native component/text/layout .fig and changed SVG semantics", "stale source refusal", "source preservation"] }));
} finally {
  child.stdin.end();
  const timer = setTimeout(() => child.kill("SIGKILL"), 2000);
  await exited; clearTimeout(timer);
  await rm(temp, { recursive: true, force: true });
}
