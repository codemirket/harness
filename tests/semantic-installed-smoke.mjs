// Installed macOS ARM64 semantic-memory journey with an exact private model.
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { chmod, copyFile, cp, mkdir, mkdtemp, rm, stat } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

const args = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  const key = process.argv[i], value = process.argv[i + 1];
  assert.ok(["--mirket", "--node", "--node-license", "--semantic-payload"].includes(key));
  assert.ok(value && isAbsolute(value) && !args.has(key));
  args.set(key, value);
}
assert.equal(args.size, 4, "Four explicit private payload paths are required");
assert.equal(process.platform, "darwin"); assert.equal(process.arch, "arm64");
for (const key of args.keys()) assert.ok((await stat(args.get(key)))[key === "--semantic-payload" ? "isDirectory" : "isFile"]());
const root = await mkdtemp(join(tmpdir(), "mirket-semantic-installed-"));
const source = join(root, "source"), home = join(root, "home"), project = join(root, "project");
let client, peer, passed = false;
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
      assert.ok(this.buffer.length < 1024 * 1024, "bounded MCP frame");
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
      const timer = setTimeout(() => reject(new Error(`${method} deadline: ${this.stderr}`)), 90000);
      this.pending.set(id, { resolve, reject, timer });
      this.child.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n");
    });
  }
  async init() {
    const result = await this.rpc("initialize", { protocolVersion: "2025-11-25", capabilities: {},
      clientInfo: { name: "semantic-installed-smoke", version: "0.0.1" } });
    assert.equal(result.protocolVersion, "2025-11-25");
    this.child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized", params: {} }) + "\n");
  }
  async tool(name, parameters = {}) {
    const result = await this.rpc("tools/call", { name, arguments: parameters });
    assert.notEqual(result.isError, true, `${name}: ${JSON.stringify(result)}`);
    return result.structuredContent ?? JSON.parse(result.content.find(item => item.type === "text").text);
  }
  async close() {
    this.child.stdin.end(); const exit = await this.exit;
    assert.equal(exit.code, 0, this.stderr);
  }
}
function ownedRssSnapshot(pids) {
  const lines = execFileSync("ps", ["-axo", "pid=,ppid=,rss=,comm="],
    { encoding: "utf8", timeout: 10000 }).trim().split("\n");
  const entries = lines.map(line => {
    const match = line.trim().match(/^(\d+)\s+(\d+)\s+(\d+)\s+(.+)$/);
    assert.ok(match, `Unexpected ps output: ${line}`);
    return { pid: Number(match[1]), ppid: Number(match[2]), rss_kib: Number(match[3]), command: match[4] };
  });
  const descendants = new Set(pids);
  for (;;) {
    const size = descendants.size;
    for (const entry of entries) if (descendants.has(entry.ppid)) descendants.add(entry.pid);
    if (size === descendants.size) break;
  }
  const owned = entries.filter(entry => descendants.has(entry.pid));
  for (const pid of pids) assert.ok(owned.some(entry => entry.pid === pid), `Missing client process ${pid}`);
  return { rss_kib: owned.reduce((total, entry) => total + entry.rss_kib, 0),
    per_client: pids.map(pid => {
      const family = new Set([pid]);
      for (;;) {
        const size = family.size;
        for (const entry of owned) if (family.has(entry.ppid)) family.add(entry.pid);
        if (size === family.size) break;
      }
      const processes = owned.filter(entry => family.has(entry.pid));
      return { pid, rss_kib: processes.reduce((total, entry) => total + entry.rss_kib, 0),
        processes: processes.map(entry => ({ pid: entry.pid, ppid: entry.ppid,
          rss_kib: entry.rss_kib, executable: entry.command.split("/").at(-1) })) };
    }) };
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
  await cp(args.get("--semantic-payload"), join(source, "runtime/semantic"), { recursive: true });
  const release = cli("dev", "manifest", "--directory", source).release_id;
  assert.match(release, /^sha256:[0-9a-f]{64}$/);
  assert.equal(cli("dev", "setup-core", "--home", home, "--source", source).activation.status, "active");
  const doctor = cli("dev", "doctor-core", "--home", home, "--json");
  assert.equal(doctor.status, "healthy");
  assert.deepEqual(doctor.active_components, ["semantic"]);
  client = new Client(); peer = new Client();
  await Promise.all([client.init(), peer.init()]);
  const beforeUse = ownedRssSnapshot([client.child.pid, peer.child.pid]);
  const runtime = await client.tool("runtime_status");
  assert.equal(runtime.release_id, release);
  assert.equal(runtime.memory_resources.embedding_configured, true);
  const scope = { kind: "project", project_root: project };
  const common = { expected_release_id: release, scope };
  const content = "Gözlemevi teleskobunun bataryası dokuz saat çalışır.";
  const record = { kind: "fact", basis: "observed", content,
    sources: [{ source_id: "fixture:observatory", revision: "fixture-1", locator: "unknown+fixture://observatory/1", captured_at: 123 }],
    status: "current", critical: false, applicability: { task_key: null, checkout_id: null, operation: null },
    valid_from: null, valid_until: null };
  const prepared = await client.tool("memory_prepare", { ...common,
    mutation: { kind: "create", record, supersedes: null } });
  const committed = await client.tool("memory_commit", { ...common, operation_id: prepared.operation_id });
  assert.equal(committed.status, "committed");
  assert.equal(committed.semantic_index.status, "indexed", JSON.stringify(committed));
  const recalled = await client.tool("memory_search", { ...common, request: {
    query: "How long does the observatory telescope battery last?", status: "both",
    context: { task_key: null, checkout_id: null, operation: null, at_time: null }, limit: 5, max_bytes: 32768,
  } });
  assert.equal(recalled.coverage.semantic_available, true, JSON.stringify(recalled));
  assert.ok(recalled.hits.some(hit => hit.record.record_id === committed.record.record_id));
  const peerRuntime = await peer.tool("runtime_status");
  assert.equal(peerRuntime.release_id, release);
  const peerRecalled = await peer.tool("memory_search", { ...common, request: {
    query: "Gözlemevi teleskobunun bataryası ne kadar süre çalışır?", status: "both",
    context: { task_key: null, checkout_id: null, operation: null, at_time: null }, limit: 5, max_bytes: 32768,
  } });
  assert.equal(peerRecalled.coverage.semantic_available, true, JSON.stringify(peerRecalled));
  assert.ok(peerRecalled.hits.some(hit => hit.record.record_id === committed.record.record_id));
  const afterUse = ownedRssSnapshot([client.child.pid, peer.child.pid]);
  await new Promise(resolve => setTimeout(resolve, 30000));
  const idle30s = ownedRssSnapshot([client.child.pid, peer.child.pid]);
  let idle75s;
  if (process.env.MIRKET_SEMANTIC_IDLE_EXPIRY === "1") {
    await new Promise(resolve => setTimeout(resolve, 45000));
    idle75s = ownedRssSnapshot([client.child.pid, peer.child.pid]);
    assert.ok(idle75s.per_client.every(item => item.processes.length === 1),
      "Both warm model process trees must retire after the 60-second idle deadline");
  }
  await Promise.all([client.close(), peer.close()]); client = peer = undefined;
  assert.equal(cli("dev", "doctor-core", "--home", home, "--json").status, "healthy");
  passed = true;
  console.log(JSON.stringify({ status: "passed", release_id: release, platform: process.platform,
    arch: process.arch, before_use: beforeUse, after_use: afterUse, idle_30s: idle30s,
    ...(idle75s ? { idle_75s: idle75s } : {}) }));
} finally {
  for (const active of [client, peer]) if (active) { active.child.kill("SIGKILL"); await active.exit; }
  if (passed) await rm(root, { recursive: true, force: true });
  else console.error(`Retained failed semantic fixture: ${root}`);
}
