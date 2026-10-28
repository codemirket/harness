// Real stdio integration fixture. No packages, host configuration or global tools.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdtemp, mkdir, readdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

const usage = "Usage: node tests/mcp-smoke.mjs --mirket /absolute/mirket --node /absolute/node";
const flags = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  const flag = process.argv[i], value = process.argv[i + 1];
  assert.ok(["--mirket", "--node"].includes(flag) && value && !flags.has(flag), usage);
  assert.ok(isAbsolute(value), usage);
  flags.set(flag, value);
}
assert.equal(flags.size, 2, usage);
for (const path of flags.values()) assert.ok((await stat(path)).isFile(), `Expected an existing file: ${path}`);

const MAX_FRAME = 1024 * 1024;
const MAX_STREAM = 4 * 1024 * 1024;
const RPC_TIMEOUT_MS = 5000;
const RELEASE_A = "m0-test-a", RELEASE_B = "m0-test-b";
const REQUIRED_TOOLS = ["runtime_status", "code_inspect", "code_navigate",
  "diagram_render_prepare", "diagram_render_start", "design_prepare", "design_start",
  "video_prepare", "video_start", "video_status", "video_cancel", "video_result",
  "job_status", "job_cancel", "memory_prepare", "memory_commit", "memory_status",
  "memory_search", "memory_read", "memory_history", "memory_list",
  "memory_forget_prepare", "memory_forget_commit", "memory_forget_reconcile",
  "memory_index_refresh"];

class Client {
  constructor(stateDir, releaseId, extraArgs = []) {
    this.releaseId = releaseId;
    this.pending = new Map();
    this.ignored = new Set();
    this.nextId = 1;
    this.buffer = Buffer.alloc(0);
    this.totalBytes = 0;
    this.stderr = "";
    this.failure = null;
    this.closed = false;
    this.child = spawn(flags.get("--mirket"), ["dev", "serve", "--state-dir", stateDir,
      "--release-id", releaseId, "--node", flags.get("--node"), ...extraArgs],
      { stdio: ["pipe", "pipe", "pipe"] });
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
          if (this.ignored.delete(message.id)) continue; // Deliberately lost response.
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
        const error = new Error(`${method} exceeded ${RPC_TIMEOUT_MS}ms`);
        reject(error);
        this.fail(error);
      }, RPC_TIMEOUT_MS);
      this.pending.set(id, { resolve, reject, timer });
      try { this.send({ id, method, params }); }
      catch (error) { clearTimeout(timer); this.pending.delete(id); reject(error); }
    });
  }

  async initialize() {
    const result = await this.rpc("initialize", { protocolVersion: "2025-11-25", capabilities: {},
      clientInfo: { name: "mirket-disposable-m0-smoke", version: "0.0.1" } });
    assert.equal(result.protocolVersion, "2025-11-25", "M0 protocol negotiation changed");
    assert.ok(result.serverInfo?.name, "Missing server identity");
    this.send({ method: "notifications/initialized", params: {} });
    return this;
  }

  call(name, args = {}) { return this.rpc("tools/call", { name, arguments: args }); }

  async stop(abrupt = false) {
    if (this.closed) return;
    if (abrupt) this.child.kill("SIGKILL");
    else this.child.stdin.end();
    const timer = setTimeout(() => this.child.kill("SIGKILL"), 1000);
    await this.exited;
    clearTimeout(timer);
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

async function treeSnapshot(root) {
  const entries = [];
  async function visit(directory, prefix = "") {
    for (const entry of (await readdir(directory, { withFileTypes: true })).sort((a, b) => a.name.localeCompare(b.name))) {
      const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
      const path = join(directory, entry.name);
      if (entry.isDirectory()) {
        entries.push(`${relative}/`);
        await visit(path, relative);
      } else {
        entries.push(`${relative}:${createHash("sha256").update(await readFile(path)).digest("hex")}`);
      }
    }
  }
  await visit(root);
  return entries;
}


const temporary = await mkdtemp(join(tmpdir(), "mirket-mcp-smoke-"));
const project = join(temporary, "project"), state = join(temporary, "state"), configuredState = join(temporary, "configured-state");
await mkdir(project, { mode: 0o700 });
await mkdir(state, { mode: 0o700 });
await mkdir(configuredState, { mode: 0o700 });
if (process.platform !== "win32") assert.equal((await stat(state)).mode & 0o777, 0o700);
const source = join(project, "diagram.d2");
await writeFile(source, "a -> b\n");
const clients = [];
const connect = async (release, extraArgs = []) => {
  const client = new Client(extraArgs.length ? configuredState : state, release, extraArgs);
  clients.push(client);
  return client.initialize();
};
const scope = (release) => ({ expected_release_id: release, project_root: project });
const diagram = (release) => ({ ...scope(release), source_path: source,
  expected_revision: "sha256:unused", destination_path: join(project, "diagram.svg"), layout: "dagre" });

try {
  const client = await connect(RELEASE_A);
  const inventory = await client.rpc("tools/list", {});
  assert.deepEqual(inventory.tools.map((tool) => tool.name).sort(), [...REQUIRED_TOOLS].sort(),
    "Unexpected public capability surface");
  for (const removed of ["file_read", "file_read_ranges", "artifact_read", "text_replace_prepare",
    "text_replace_start", "job_result", "sql_navigate"]) {
    assert.ok(!inventory.tools.some((tool) => tool.name === removed), `${removed} remains public`);
  }
  assert.ok(!inventory.tools.some((tool) => tool.name.startsWith("browser_")));
  const runtime = await success(client, "runtime_status");
  assert.equal(runtime.release_id, RELEASE_A);
  assert.equal(runtime.video.configured, false);
  assert.equal(runtime.design.configured, false);
  assert.equal(runtime.diagrams.configured, false);
  assert.equal(runtime.code_source_inspection.configured, false);
  assert.deepEqual(runtime.code_navigation.languages, ["js", "ts", "jsx", "tsx", "ts_declaration"]);
  assert.deepEqual(runtime.skill_capabilities, [],
    "An unqualified development server must not advertise project skill capabilities");
  assert.deepEqual(runtime.capabilities, [
    "javascript_typescript_source_inspection", "selected_source_code_navigation",
    "operation_receipts", "scoped_canonical_memory", "bounded_lexical_recall",
    "canonical_memory_pagination", "explicit_memory_forgetting",
    "optional_managed_memory_backups", "maintenance_memory_restore",
    "explicit_derived_index_refresh", "design_prepare_start", "finite_video_jobs",
  ], "MCP capability surface must remain stable while readiness is reported separately");
  assert.ok(runtime.limits.includes("Codex Desktop GUI activation untested by user waiver"));
  assert.ok(!runtime.capabilities.includes("bounded_utf8_read"));
  assert.ok(!runtime.capabilities.includes("text_replace_new_copy"));
  console.log("PASS Codex-only MCP inventory and runtime status");

  await rejected(client, "diagram_render_prepare", diagram("stale-release"), /release/i);
  await rejected(client, "diagram_render_prepare", diagram(RELEASE_A), /private D2 runtime is not configured/i);
  await rejected(client, "design_prepare", { ...scope(RELEASE_A), source_path: source,
    expected_revision: "sha256:unused", destination_path: join(project, "tokens.css"),
    operation: { kind: "tokens_compile", format: "css" } }, /private design runtime is not configured/i);
  await rejected(client, "video_prepare", { ...scope(RELEASE_A), source_path: source,
    expected_revision: "sha256:unused", assets: [], destination_path: join(project, "absent.mp4"),
    profile: { width: 640, height: 360, fps: { numerator: 24, denominator: 1 }, frame_count: 24,
      format: "mp4", captions_path: null } }, /private video runtime is not configured/i);
  await rejected(client, "job_status", { ...scope(RELEASE_A), operation_id: "unknown" }, /operation|receipt|unknown|id/i);
  const unsupportedProjectBefore = await treeSnapshot(project);
  const unsupportedStateBefore = await treeSnapshot(state);
  await rejected(client, "file_read", { project_root: project, path: "diagram.d2" }, /tool|method|unknown|unsupported|not found/i);
  assert.deepEqual(await treeSnapshot(project), unsupportedProjectBefore,
    "unsupported tool must not write the selected project");
  assert.deepEqual(await treeSnapshot(state), unsupportedStateBefore,
    "unsupported tool must not create a receipt or state effect");
  console.log("PASS stale release, absent private payloads and unknown job refusal");

  // A configured private payload still binds the source revision. A stale
  // revision is refused before a job receipt or destination can be created.
  const designAdapter = join(temporary, "design-main.js");
  const designFont = join(temporary, "font.ttf");
  await writeFile(designAdapter, "// disposable adapter sentinel\n");
  await writeFile(designFont, "font sentinel\n");
  const configured = await connect(RELEASE_A, ["--design-adapter", designAdapter, "--design-font", designFont]);
  const configuredRuntime = await success(configured, "runtime_status");
  assert.equal(configuredRuntime.design.configured, true);
  assert.deepEqual(configuredRuntime.skill_capabilities, ["navigation.js-ts", "design.tokens", "design.vector"]);
  const staleDestination = join(project, "stale.css");
  await writeFile(join(project, "tokens.json"), JSON.stringify({ color: { brand: { value: "#123456" } } }));
  await rejected(configured, "design_prepare", {
    ...scope(RELEASE_A), source_path: join(project, "tokens.json"),
    expected_revision: "sha256:" + "0".repeat(64), destination_path: staleDestination,
    operation: { kind: "tokens_compile", format: "css" },
  }, /revision changed/i);
  await assert.rejects(stat(staleDestination), { code: "ENOENT" });
  const otherProject = join(temporary, "other-project");
  await mkdir(otherProject, { mode: 0o700 });
  const otherSource = join(otherProject, "tokens.json");
  const otherDestination = join(otherProject, "foreign.css");
  await writeFile(otherSource, JSON.stringify({ color: { foreign: { value: "#abcdef" } } }));
  const otherBefore = await treeSnapshot(otherProject);
  const projectBeforeWrongSource = await treeSnapshot(project);
  const otherSourceRevision = "sha256:" + createHash("sha256").update(await readFile(otherSource)).digest("hex");
  await rejected(configured, "design_prepare", {
    ...scope(RELEASE_A), source_path: otherSource,
    expected_revision: otherSourceRevision, destination_path: join(project, "wrong-source.css"),
    operation: { kind: "tokens_compile", format: "css" },
  }, /outside|project|scope/i);
  assert.deepEqual(await treeSnapshot(project), projectBeforeWrongSource,
    "wrong-project source must not write the selected project");
  assert.deepEqual(await treeSnapshot(otherProject), otherBefore,
    "wrong-project source must not write the source project");

  const projectSourceRevision = "sha256:" + createHash("sha256").update(await readFile(join(project, "tokens.json"))).digest("hex");
  const projectBeforeWrongDestination = await treeSnapshot(project);
  await rejected(configured, "design_prepare", {
    ...scope(RELEASE_A), source_path: join(project, "tokens.json"),
    expected_revision: projectSourceRevision, destination_path: otherDestination,
    operation: { kind: "tokens_compile", format: "css" },
  }, /outside|project|scope/i);
  assert.deepEqual(await treeSnapshot(project), projectBeforeWrongDestination,
    "wrong-project destination must not write the selected project");
  assert.deepEqual(await treeSnapshot(otherProject), otherBefore,
    "wrong-project destination must not write the destination project");
  await assert.rejects(stat(otherDestination), { code: "ENOENT" });
  console.log("PASS configured payload reports exact skill pairing and refuses stale, unsupported and wrong-project operations");
  await configured.stop();

  const next = await connect(RELEASE_B);
  assert.equal((await success(next, "runtime_status")).release_id, RELEASE_B);
  await rejected(next, "diagram_render_prepare", diagram(RELEASE_A), /release/i);
  console.log(`PASS stdio smoke on ${process.platform}/${process.arch}`);
} finally {
  await Promise.allSettled(clients.map((client) => client.stop()));
  await rm(temporary, { recursive: true, force: true });
}
