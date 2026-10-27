// Disposable installed macOS ARM64 D2/MCP journey. No host config or downloads.
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, copyFile, cp, mkdir, mkdtemp, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

const args = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  const key = process.argv[i], value = process.argv[i + 1];
  assert.ok(["--mirket", "--node", "--node-license", "--d2-payload"].includes(key));
  assert.ok(value && isAbsolute(value) && !args.has(key));
  args.set(key, value);
}
assert.equal(args.size, 4, "Four explicit private payload paths are required");
assert.equal(process.platform, "darwin");
assert.equal(process.arch, "arm64");
const sha = bytes => createHash("sha256").update(bytes).digest("hex");
const rev = bytes => `sha256:${sha(bytes)}`;
const root = await mkdtemp(join(tmpdir(), "mirket-d2-installed-"));
const source = join(root, "source"), home = join(root, "home"), project = join(root, "project ç");
let client, passed = false;
function cli(...command) {
  return JSON.parse(execFileSync(args.get("--mirket"), command,
    { encoding: "utf8", timeout: 30000, stdio: ["ignore", "pipe", "pipe"] }));
}

class Client {
  constructor() {
    this.next = 0; this.pending = new Map(); this.buffer = ""; this.stderr = "";
    this.child = spawn(join(home, "bin/mirket"), ["serve", "--home", home], { stdio: ["pipe", "pipe", "pipe"] });
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
    const result = await this.rpc("tools/call", { name, arguments: parameters });
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

  const body = Buffer.from("client -> api: request\napi -> store: write\nstore -> api: ack\n", "utf8");
  const input = join(project, "architecture.d2");
  await writeFile(input, body, { mode: 0o600 });
  const results = [];
  for (const layout of ["dagre", "elk", "tala"]) {
    const output = join(project, `${layout}.svg`);
    const prepared = await client.tool("diagram_render_prepare", {
      expected_release_id: release, project_root: project, source_path: input,
      expected_revision: rev(body), destination_path: output, layout,
    });
    await client.tool("diagram_render_start", { expected_release_id: release, project_root: project,
      operation_id: prepared.operation_id });
    const finished = await terminal(prepared, release);
    assert.equal(finished.status, "succeeded", JSON.stringify(finished));
    assert.equal(finished.scratch_cleanup, "complete");
    const svg = await readFile(output);
    assert.ok(svg.length > 1000 && svg.toString().includes("<svg"));
    assert.equal(finished.artifact_revision, rev(svg));
    results.push({ layout, bytes: svg.length, sha256: sha(svg) });
  }
  assert.equal(new Set(results.map(item => item.sha256)).size, 3);
  const common = { expected_release_id: release, project_root: project, source_path: input,
    expected_revision: rev(body), destination_path: join(project, "rejected.svg"), layout: "dagre" };
  assert.match(await client.tool("diagram_render_prepare", { ...common, expected_revision: rev("old") }, true), /revision/);
  await writeFile(input, "import \"outside.d2\"\n", { mode: 0o600 });
  assert.match(await client.tool("diagram_render_prepare", { ...common, expected_revision: rev(Buffer.from("import \"outside.d2\"\n")) }, true), /import|external/);
  await writeFile(input, body);
  assert.match(await client.tool("diagram_render_prepare", { ...common, expected_release_id: "stale" }, true), /release/);
  assert.deepEqual(await readdir(join(home, "state/jobs")), []);
  await client.close(); client = undefined;
  assert.equal(cli("dev", "doctor-core", "--home", home, "--json").status, "healthy");
  passed = true;
  console.log(`PASS installed D2 Dagre/ELK/TALA, revisions, refusal and cleanup: ${JSON.stringify(results)}`);
} finally {
  if (client) { client.child.kill("SIGKILL"); await client.exit; }
  if (passed) await rm(root, { recursive: true, force: true });
  else console.error(`Retained failed D2 fixture: ${root}`);
}
