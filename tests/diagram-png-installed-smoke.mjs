// Disposable installed macOS ARM64 D2/MCP journey. No host config or downloads.
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, copyFile, cp, mkdir, mkdtemp, readFile, readdir, realpath, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

const args = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  const key = process.argv[i], value = process.argv[i + 1];
  assert.ok(["--mirket", "--node", "--node-license", "--d2-payload", "--ffmpeg"].includes(key));
  assert.ok(value && isAbsolute(value) && !args.has(key));
  args.set(key, value);
}
assert.equal(args.size, 5, "Five explicit private payload paths are required");
assert.equal(process.platform, "darwin");
assert.equal(process.arch, "arm64");
const sha = bytes => createHash("sha256").update(bytes).digest("hex");
const rev = bytes => `sha256:${sha(bytes)}`;
const root = await realpath(await mkdtemp(join(tmpdir(), "mirket-d2-png-installed-")));
const source = join(root, "source"), home = join(root, "home"), project = join(root, "project ç");
let client;
const evidence = { root, platform: process.platform, arch: process.arch, node: process.version,
  os: execFileSync("/usr/bin/sw_vers", [], { encoding: "utf8" }).trim(),
  limits: ["Installed MCP operations; Pi CLI/TUI and model image delivery are separate qualification gates"],
  commands: [], checks: [], outputs: [] };
const env = { HOME: join(root, "host-home"), PI_CODING_AGENT_DIR: join(root, "pi-agent"),
  PI_SESSION_DIR: join(root, "pi-sessions"), TMPDIR: root, PATH: "", LANG: "C" };
for (const dir of [env.HOME, env.PI_CODING_AGENT_DIR, env.PI_SESSION_DIR]) await mkdir(dir, { mode: 0o700 });
evidence.executables = await Promise.all(["--mirket", "--node", "--ffmpeg"].map(async key => ({
  path: args.get(key), sha256: sha(await readFile(args.get(key))) })));
assert.equal(evidence.executables[2].sha256, "bc48284aaf2b8ab0c9f10ff7f69615c466b7dd2936e759d3b80bfffbdc7793fb");
assert.equal(sha(await readFile(join(args.get("--d2-payload"), "d2"))),
  "e8ba501a413bac33a4d93014c475b804de10fb2863a391583e23a8d88160e311");
function decoded(file) {
  return execFileSync(args.get("--ffmpeg"), ["-v", "error", "-nostdin", "-i", file,
    "-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgba", "-"],
    { env, timeout: 30000, maxBuffer: 20 * 1024 * 1024 });
}
function chunks(png) {
  const names = []; let offset = 8;
  while (offset < png.length) {
    names.push(png.subarray(offset + 4, offset + 8).toString("ascii"));
    offset += png.readUInt32BE(offset) + 12;
  }
  return names;
}
function cli(...command) {
  evidence.commands.push(command);
  return JSON.parse(execFileSync(args.get("--mirket"), command,
    { env, encoding: "utf8", timeout: 30000, stdio: ["ignore", "pipe", "pipe"] }));
}

class Client {
  constructor() {
    this.next = 0; this.pending = new Map(); this.buffer = ""; this.stderr = "";
    this.child = spawn(join(home, "bin/mirket"), ["serve", "--home", home], { env, stdio: ["pipe", "pipe", "pipe"] });
    this.child.stdout.setEncoding("utf8").on("data", chunk => {
      this.buffer += chunk;
      for (;;) {
        const end = this.buffer.indexOf("\n"); if (end < 0) break;
        const message = JSON.parse(this.buffer.slice(0, end)); this.buffer = this.buffer.slice(end + 1);
        const pending = this.pending.get(message.id); if (!pending) continue;
        clearTimeout(pending.timer); this.pending.delete(message.id);
        if (message.error) pending.reject(new Error(JSON.stringify(message.error)));
        else pending.resolve(message.result);
      }
      assert.ok(this.buffer.length < 128 * 1024, "bounded MCP frame");
    });
    this.child.stderr.setEncoding("utf8").on("data", chunk => { this.stderr += chunk; });
    this.exit = new Promise(resolve => this.child.on("close", (code, signal) => {
      for (const pending of this.pending.values()) pending.reject(new Error(`MCP closed: ${this.stderr}`));
      this.pending.clear(); resolve({ code, signal });
    }));
  }
  rpc(method, params) {
    const id = ++this.next;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error(`${method} deadline: ${this.stderr}`)), 40000);
      this.pending.set(id, { resolve, reject, timer });
      this.child.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n");
    });
  }
  async init() {
    const result = await this.rpc("initialize", { protocolVersion: "2025-11-25", capabilities: {},
      clientInfo: { name: "diagram-installed-smoke", version: "0.0.1" } });
    assert.equal(result.protocolVersion, "2025-11-25");
    this.child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized", params: {} }) + "\n");
  }
  async tool(name, parameters = {}, expectError = false) {
    evidence.commands.push({ tool: name, arguments: parameters });
    const result = await this.rpc("tools/call", { name, arguments: parameters });
    await writeFile(join(root, "evidence.json"), JSON.stringify(evidence, null, 2));
    assert.equal(result.isError === true, expectError, `${name}: ${JSON.stringify(result)}`);
    if (expectError) return JSON.stringify(result);
    return result.structuredContent ?? JSON.parse(result.content.find(item => item.type === "text").text);
  }
  async close() {
    this.child.stdin.end();
    const result = await this.exit;
    assert.equal(result.code, 0, this.stderr);
  }
}

async function terminal(operation, release) {
  const until = Date.now() + 40000;
  while (Date.now() < until) {
    const status = await client.tool("job_status", { expected_release_id: release, project_root: project,
      operation_id: operation.operation_id });
    if (["succeeded", "failed", "cancelled", "interrupted"].includes(status.status)
      && !status.scratch_cleanup?.startsWith("retained: terminal confirmation")) return status;
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  throw new Error("D2 operation did not reach a terminal receipt");
}

try {
  for (const directory of [source, join(source, "bin"),
    join(source, "runtime"), join(source, "runtime/node"), join(source, "runtime/node/bin"), project]) {
    await mkdir(directory, { mode: 0o700 });
  }
  for (const [key, relative, mode] of [
    ["--mirket", "bin/mirket", 0o700], ["--node", "runtime/node/bin/node", 0o700],
    ["--node-license", "runtime/node/LICENSE", 0o600],
  ]) {
    await copyFile(args.get(key), join(source, relative)); await chmod(join(source, relative), mode);
  }
  await cp(args.get("--d2-payload"), join(source, "runtime/d2"), { recursive: true });
  for (const name of await readdir(join(source, "runtime/d2"))) {
    await chmod(join(source, "runtime/d2", name), name === "d2" ? 0o700 : 0o600);
  }
  const manifested = cli("dev", "manifest", "--directory", source);
  const release = manifested.release_id;
  assert.match(release, /^sha256:[0-9a-f]{64}$/);
  const manifest = JSON.parse(await readFile(join(source, "manifest.json"), "utf8"));
  assert.deepEqual(manifest.components, ["diagram"]);
  const setup = cli("dev", "setup-core", "--home", home, "--source", source);
  assert.equal(setup.activation.status, "active");
  assert.equal(cli("dev", "doctor-core", "--home", home, "--json").status, "healthy");
  client = new Client(); await client.init();
  const status = await client.tool("runtime_status");
  assert.equal(status.release_id, release);
  assert.equal(status.diagrams.configured, true);

  const body = Buffer.from('client: "İstemci" {style.fill: "#aaddff"}\napi: "Hizmet"\nclient -> api: "İstek"\n', "utf8");
  const input = join(project, "architecture.d2");
  await writeFile(input, body, { mode: 0o600 });
  const results = [];
  for (const layout of ["dagre", "elk", "tala"]) {
    const output = join(project, `${layout}.png`);
    const prepared = await client.tool("diagram_render_prepare", {
      expected_release_id: release, project_root: project, source_path: input,
      expected_revision: rev(body), destination_path: output, layout, output_format: "png",
    });
    await client.tool("diagram_render_start", { expected_release_id: release, project_root: project,
      operation_id: prepared.operation_id });
    const finished = await terminal(prepared, release);
    assert.equal(finished.status, "succeeded", JSON.stringify(finished));
    assert.equal(finished.scratch_cleanup, "complete");
    assert.equal(prepared.output_format, "png");
    assert.equal(finished.output_format, "png");
    assert.equal(finished.source_revision, rev(body));
    const png = await readFile(output), rgba = decoded(output);
    assert.deepEqual([...png.subarray(0, 8)], [137, 80, 78, 71, 13, 10, 26, 10]);
    assert.equal(rgba.length, png.readUInt32BE(16) * png.readUInt32BE(20) * 4);
    assert.equal(finished.artifact_revision, rev(png));
    const result = { layout, bytes: png.length, sha256: sha(png), decoded_sha256: sha(rgba),
      width: png.readUInt32BE(16), height: png.readUInt32BE(20), chunks: chunks(png), receipt: finished };
    results.push(result); evidence.outputs.push(result);
  }
  const changed = Buffer.from(body.toString().replace("#aaddff", "#ffbb99"));
  await writeFile(input, changed);
  const changedOutput = join(project, "changed.png");
  const changedIntent = await client.tool("diagram_render_prepare", { expected_release_id: release,
    project_root: project, source_path: input, expected_revision: rev(changed),
    destination_path: changedOutput, layout: "dagre", output_format: "png" });
  await client.tool("diagram_render_start", { expected_release_id: release, project_root: project,
    operation_id: changedIntent.operation_id });
  const changedReceipt = await terminal(changedIntent, release);
  assert.equal(changedReceipt.status, "succeeded", JSON.stringify(changedReceipt));
  const originalPixels = decoded(join(project, "dagre.png")), changedPixels = decoded(changedOutput);
  assert.equal(changedPixels.length, originalPixels.length);
  let changedPixelsCount = 0;
  for (let i = 0; i < originalPixels.length; i += 4) {
    if (!originalPixels.subarray(i, i + 4).equals(changedPixels.subarray(i, i + 4))) changedPixelsCount++;
  }
  assert.ok(changedPixelsCount > 100);
  evidence.changed = { changedPixelsCount, receipt: changedReceipt, decoded_sha256: sha(changedPixels) };
  await writeFile(input, body);
  const svgOutput = join(project, "default.svg");
  const svgIntent = await client.tool("diagram_render_prepare", { expected_release_id: release,
    project_root: project, source_path: input, expected_revision: rev(body),
    destination_path: svgOutput, layout: "dagre" });
  assert.equal(svgIntent.output_format, "svg");
  await client.tool("diagram_render_start", { expected_release_id: release, project_root: project,
    operation_id: svgIntent.operation_id });
  assert.equal((await terminal(svgIntent, release)).status, "succeeded");
  assert.match((await readFile(svgOutput)).toString(), /<svg/);
  evidence.checks.push("default SVG preserved", "native PNG decoded across three layouts with Turkish text",
    "changed revision changes decoded pixels");
  const common = { expected_release_id: release, project_root: project, source_path: input,
    expected_revision: rev(body), destination_path: join(project, "rejected.svg"), layout: "dagre" };
  assert.match(await client.tool("diagram_render_prepare", { ...common, expected_revision: rev("old") }, true), /revision/);
  assert.match(await client.tool("diagram_render_prepare", { ...common, output_format: "png" }, true), /extension|format/);
  assert.match(await client.tool("diagram_render_prepare", { ...common, output_format: "pdf" }, true), /variant|format|invalid/i);
  const staleIntent = await client.tool("diagram_render_prepare", { ...common, output_format: "png",
    destination_path: join(project, "stale.png") });
  await writeFile(input, changed);
  await client.tool("diagram_render_start", { expected_release_id: release, project_root: project,
    operation_id: staleIntent.operation_id });
  assert.equal((await terminal(staleIntent, release)).status, "failed");
  assert.equal(await readFile(join(project, "stale.png")).then(() => true, () => false), false);
  await writeFile(input, body);
  await writeFile(input, "import \"outside.d2\"\n", { mode: 0o600 });
  assert.match(await client.tool("diagram_render_prepare", { ...common, expected_revision: rev(Buffer.from("import \"outside.d2\"\n")) }, true), /import|external/);
  await writeFile(input, body);
  assert.match(await client.tool("diagram_render_prepare", { ...common, expected_release_id: "stale" }, true), /release/);
  const cancelSource = Buffer.from(Array.from({ length: 700 }, (_, i) =>
    `n${i}: "Node ${i}"\nn${i} -> n${(i + 1) % 700}\nn${i} -> n${(i + 11) % 700}`).join("\n"));
  await writeFile(input, cancelSource);
  const cancelOutput = join(project, "cancel.png");
  const cancelIntent = await client.tool("diagram_render_prepare", { ...common, layout: "elk",
    expected_revision: rev(cancelSource), output_format: "png", destination_path: cancelOutput });
  const cancelArgs = { expected_release_id: release, project_root: project, operation_id: cancelIntent.operation_id };
  await client.tool("diagram_render_start", cancelArgs);
  const cancelDeadline = Date.now() + 10000;
  let beforeCancel;
  do {
    beforeCancel = await client.tool("job_status", cancelArgs);
    if (beforeCancel.status === "running") break;
    assert.equal(beforeCancel.status, "queued", JSON.stringify(beforeCancel));
    await new Promise(resolve => setTimeout(resolve, 20));
  } while (Date.now() < cancelDeadline);
  assert.equal(beforeCancel.status, "running");
  await new Promise(resolve => setTimeout(resolve, 500));
  beforeCancel = await client.tool("job_status", cancelArgs);
  assert.equal(beforeCancel.status, "running", JSON.stringify(beforeCancel));
  await client.tool("job_cancel", cancelArgs);
  const cancelled = await terminal(cancelIntent, release);
  assert.equal(cancelled.status, "cancelled", JSON.stringify(cancelled));
  assert.equal(cancelled.scratch_cleanup, "complete");
  assert.equal(await readFile(cancelOutput).then(() => true, () => false), false);
  evidence.cancelled = cancelled;
  evidence.checks.push("running PNG operation cancellation publishes no destination and confirms cleanup");
  assert.deepEqual(await readdir(join(home, "state/jobs")), []);
  await client.close(); client = undefined;
  assert.equal(cli("dev", "doctor-core", "--home", home, "--json").status, "healthy");
  evidence.checks.push("stale source fails without publication", "unknown/mismatched formats refused",
    "source external import refused", "private job cleanup complete");
  evidence.status = "passed";
  console.log(`PASS installed D2 PNG; ${changedPixelsCount} changed decoded pixels; evidence: ${root}`);
} catch (error) {
  evidence.error = { stack: error.stack, stderr: error.stderr?.toString(), stdout: error.stdout?.toString() };
  throw error;
} finally {
  if (client) { client.child.kill("SIGKILL"); await client.exit; }
  await writeFile(join(root, "evidence.json"), JSON.stringify(evidence, null, 2));
  console.error(`Retained D2 PNG fixture: ${root}`);
}
