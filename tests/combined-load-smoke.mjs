// Disposable macOS ARM64 qualification: semantic/render overlap, then bounded
// contention with navigation and independent semantic recovery. Paths are local.
import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, copyFile, mkdir, mkdtemp, readFile, readdir, realpath, rm, stat, statfs, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, isAbsolute, join } from "node:path";
import { fileURLToPath } from "node:url";

const required = ["--mirket", "--node", "--design-adapter", "--font", "--embedding-package", "--evidence-root"];
const flags = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  const key = process.argv[i], value = process.argv[i + 1];
  assert.ok(required.includes(key) && value && isAbsolute(value) && !flags.has(key));
  assert.equal(await realpath(value), value, `${key} must be canonical`);
  const info = await stat(value);
  assert.ok(["--embedding-package", "--evidence-root"].includes(key) ? info.isDirectory() : info.isFile());
  flags.set(key, value);
}
assert.equal(flags.size, required.length, `Supply ${required.join(", ")}`);
assert.equal(process.platform, "darwin"); assert.equal(process.arch, "arm64");
const minimumFreeBytes = 25 * 1024 ** 3;
async function disk() {
  const info = await statfs(flags.get("--evidence-root"));
  const available_bytes = Number(info.bavail) * Number(info.bsize);
  assert.ok(available_bytes >= minimumFreeBytes, "Keep at least 25 GiB available");
  return { available_bytes, minimum_free_bytes: minimumFreeBytes };
}
const hash = value => createHash("sha256").update(value).digest("hex");
const revision = value => `sha256:${hash(value)}`;
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
function bounded(promise, ms, message) {
  let timer;
  return Promise.race([promise, new Promise((_, reject) => { timer = setTimeout(() => reject(new Error(message)), ms); })]).finally(() => clearTimeout(timer));
}
const evidence = await mkdtemp(join(flags.get("--evidence-root"), "combined-load-"));
await chmod(evidence, 0o700);
const root = await realpath(await mkdtemp(join(tmpdir(), "mirket-combined-load-")));
await chmod(root, 0o700);
const state = join(root, "state"), project = join(root, "project"), binary = join(root, "mirket");
await mkdir(state, { mode: 0o700 }); await mkdir(project, { mode: 0o700 });
await copyFile(flags.get("--mirket"), binary); await chmod(binary, 0o700);
const release = revision(await readFile(binary));
const started = performance.now();
const report = { status: "running", evidence, fixture: root, release_id: release,
  platform: { platform: process.platform, arch: process.arch,
    version: execFileSync("/usr/bin/sw_vers", ["-productVersion"], { encoding: "utf8" }).trim(),
    physical_ram_bytes: Number(execFileSync("/usr/sbin/sysctl", ["-n", "hw.memsize"], { encoding: "utf8" }).trim()) },
  disk_before: await disk(), inputs: {}, requests: [], samples: [], rounds: [], limitations: [
    "Development fixture; no installed-package, rights, release or desktop activation claim.",
    "Sampled process RSS sums can double-count shared pages; these are not physical-memory peaks or hard limits.",
    "No synthetic memory pressure, 8 GiB threshold crossing, timeout injection, video, browser or large memory corpus.",
    "Admission files are independently read observations, not an atomic lock/accounting snapshot.",
    "Three heavy consumers share two slots; explicit idle-retirement fallback is not uninterrupted three-way semantic availability.",
  ] };
for (const [key, path] of flags) {
  if (key === "--evidence-root") continue;
  const identityPath = key === "--embedding-package" ? join(path, "package-receipt.json") : path;
  report.inputs[key] = { path, sha256: hash(await readFile(identityPath)) };
}
const script = fileURLToPath(import.meta.url), repository = dirname(dirname(script));
report.source_sha256 = {};
for (const relative of ["Cargo.lock", "src/resource_admission.rs", "src/resource_admission_macos.rs", "src/embedding_session.rs", "src/embedding_session_pressure.rs", "src/memory_runtime.rs", "src/engine.rs", "src/design.rs", "src/code_navigation.rs", "tests/combined-load-smoke.mjs"]) {
  report.source_sha256[relative] = hash(await readFile(join(repository, relative)));
}
const designInventoryBytes = await readFile(join(repository, "packaging/admission/m4-design-chromium-macos.runtime.json"));
const designInventory = JSON.parse(designInventoryBytes);
assert.equal(designInventory.target, "aarch64-apple-darwin");
let designBytes = 0;
for (const member of designInventory.files) {
  const bytes = await readFile(join(dirname(flags.get("--design-adapter")), "node_modules", member.path));
  assert.equal(bytes.length, member.bytes); assert.equal(hash(bytes), member.sha256, `Design payload drift: ${member.path}`); designBytes += bytes.length;
}
assert.equal(designBytes, designInventory.total_bytes);
assert.equal(report.inputs["--font"].sha256, designInventory.font.sha256);
const nodeSelection = JSON.parse(await readFile(join(repository, "packaging/admission/m0-node.selection.json")));
assert.equal(report.inputs["--node"].sha256, nodeSelection.targets.find(target => target.target === "aarch64-apple-darwin").native_binary.sha256);
report.payload_verification = { design_inventory_sha256: hash(designInventoryBytes), design_files: designInventory.files.length, design_bytes: designBytes, node_matches_selected_bytes: true };
report.adapter_modules = {};
for (const name of ["design-main.js", "design-operations.js", "code-navigation-main.js", "code-navigation.js"]) {
  const selected = await readFile(join(dirname(flags.get("--design-adapter")), name));
  assert.equal(hash(selected), hash(await readFile(join(repository, "adapters/dist", name))), `Selected adapter is stale: ${name}`);
  report.adapter_modules[name] = hash(selected);
}

const clients = [], knownOwnedPids = new Set();
class Client {
  constructor(name) {
    this.name = name; this.next = 0; this.pending = new Map(); this.buffer = ""; this.stderr = "";
    this.child = spawn(binary, ["dev", "serve", "--state-dir", state, "--release-id", release,
      "--node", flags.get("--node"),
      "--design-adapter", flags.get("--design-adapter"), "--design-font", flags.get("--font"),
      "--embedding-package", flags.get("--embedding-package")], { stdio: ["pipe", "pipe", "pipe"] });
    clients.push(this); knownOwnedPids.add(this.child.pid);
    this.closed = new Promise((resolve, reject) => {
      this.child.on("error", reject);
      this.child.on("close", (code, signal) => {
        for (const waiter of this.pending.values()) { clearTimeout(waiter.timer); waiter.reject(new Error(`MCP closed: ${this.stderr}`)); }
        this.pending.clear(); resolve({ code, signal });
      });
    });
    this.child.stdout.setEncoding("utf8").on("data", chunk => {
      try {
        this.buffer += chunk; assert.ok(Buffer.byteLength(this.buffer) <= 2 * 1024 * 1024);
        for (;;) {
          const end = this.buffer.indexOf("\n"); if (end < 0) break;
          const frame = JSON.parse(this.buffer.slice(0, end)); this.buffer = this.buffer.slice(end + 1);
          const waiter = this.pending.get(frame.id); if (!waiter) continue;
          clearTimeout(waiter.timer); this.pending.delete(frame.id);
          frame.error ? waiter.reject(new Error(JSON.stringify(frame.error))) : waiter.resolve(frame.result);
        }
      } catch (error) {
        this.failure = error;
        for (const waiter of this.pending.values()) { clearTimeout(waiter.timer); waiter.reject(error); }
        this.pending.clear(); this.child.stdin.end();
      }
    });
    this.child.stderr.setEncoding("utf8").on("data", chunk => {
      this.stderr += chunk;
      if (Buffer.byteLength(this.stderr) > 65536) { this.failure = new Error("MCP diagnostics exceeded bound"); this.child.stdin.end(); }
    });
  }
  rpc(method, params, timeout = 90000) {
    if (this.failure) throw this.failure;
    const id = ++this.next;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => { this.pending.delete(id); reject(new Error(`${method} deadline: ${this.stderr}`)); }, timeout);
      this.pending.set(id, { resolve, reject, timer });
      this.child.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n");
    });
  }
  async init() {
    const value = await this.rpc("initialize", { protocolVersion: "2025-11-25", capabilities: {}, clientInfo: { name: `combined-load-${this.name}`, version: "0.0.1" } });
    assert.equal(value.protocolVersion, "2025-11-25");
    this.child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized", params: {} }) + "\n");
  }
  async tool(name, parameters = {}, expectError = false) {
    const begin = performance.now();
    const result = await this.rpc("tools/call", { name, arguments: { expected_release_id: release, ...parameters } });
    report.requests.push({ client: this.name, tool: name, begin_ms: begin - started, elapsed_ms: performance.now() - begin, is_error: result.isError === true });
    assert.equal(result.isError === true, expectError, `${name}: ${JSON.stringify(result)}`);
    if (expectError) return result;
    const text = JSON.parse(result.content.find(part => part.type === "text").text);
    if (result.structuredContent) assert.deepEqual(result.structuredContent, text);
    return result.structuredContent ?? text;
  }
}
function processTable() {
  const text = execFileSync("/bin/ps", ["-ww", "-axo", "pid=,ppid=,rss=,lstart=,command="], { encoding: "utf8", timeout: 3000, maxBuffer: 4 * 1024 * 1024 });
  return text.trim().split("\n").map(line => {
    const match = line.trim().match(/^(\d+)\s+(\d+)\s+(\d+)\s+(\w{3}\s+\w{3}\s+\d+\s+\d+:\d+:\d+\s+\d+)\s+(.+)$/);
    assert.ok(match, `Unexpected ps row: ${line}`);
    return { pid: Number(match[1]), ppid: Number(match[2]), rss_kib: Number(match[3]), birth: match[4], command: match[5] };
  });
}
async function admissionFiles() {
  const result = { registrations: [], bindings: [], markers: [], jobs: [] };
  for (const name of await readdir(join(state, "resources")).catch(error => error.code === "ENOENT" ? [] : Promise.reject(error))) {
    if (!name.endsWith(".json")) continue;
    try {
      const value = JSON.parse(await readFile(join(state, "resources", name), "utf8"));
      (name.startsWith("root-") ? result.registrations : result.bindings).push({ name, value });
    } catch (error) { if (error.code !== "ENOENT") result.changed_during_read = true; }
  }
  for (const name of await readdir(state)) {
    if (!/^heavy-slot-\d\.unresolved$/.test(name)) continue;
    try { result.markers.push({ name, text: await readFile(join(state, name), "utf8") }); }
    catch (error) { if (error.code !== "ENOENT") throw error; }
  }
  result.jobs = await readdir(join(state, "jobs")).catch(error => error.code === "ENOENT" ? [] : Promise.reject(error));
  return result;
}
let phase = "initializing", sampler, sampling = false;
async function sample() {
  const table = processTable(), family = new Set(clients.map(client => client.child.pid));
  for (;;) { const size = family.size; for (const row of table) if (family.has(row.ppid)) family.add(row.pid); if (family.size === size) break; }
  const processes = table.filter(row => family.has(row.pid));
  for (const row of processes) knownOwnedPids.add(row.pid);
  const pressure_level = Number(execFileSync("/usr/sbin/sysctl", ["-n", "kern.memorystatus_vm_pressure_level"], { encoding: "utf8", timeout: 3000 }).trim());
  assert.ok([1, 2, 4].includes(pressure_level));
  const value = { at_ms: performance.now() - started, phase, pressure_level, rss_kib: processes.reduce((sum, row) => sum + row.rss_kib, 0), processes, admission: await admissionFiles() };
  report.samples.push(value); return value;
}
async function terminal(client, operation_id) {
  const until = performance.now() + 45000;
  while (performance.now() < until) {
    const result = await client.tool("job_status", { project_root: project, operation_id });
    if (["succeeded", "failed", "cancelled", "interrupted"].includes(result.status)
      && !result.scratch_cleanup?.startsWith("retained: terminal confirmation")) return result;
    await delay(25);
  }
  throw new Error("Design terminal receipt deadline exceeded");
}
let recordId;
const scope = { kind: "project", project_root: project };
const retirementWarnings = new Set([
  "embedding inference not started: embedding session already stopped",
  "embedding inference not started: embedding session is retiring",
  "embedding inference may have started; request was not replayed: embedding session stopped during inference",
]);
async function recall(client, observations, allowRetirement = false) {
  const begin = performance.now();
  const value = await client.tool("memory_search", { scope, request: {
    query: "How long does the observatory telescope battery last?", status: "both",
    context: { task_key: null, checkout_id: null, operation: null, at_time: null }, limit: 5, max_bytes: 32768,
  } });
  const observation = { begin_ms: begin - started, end_ms: performance.now() - started,
    outcome: value.coverage.semantic_available ? "semantic" : "unavailable",
    coverage: value.coverage, record_ids: value.hits.map(hit => hit.record.record_id) };
  // Retain the actual response even when the following assertion fails.
  observations?.push(observation);
  if (value.coverage.semantic_available) {
    assert.equal(value.coverage.lexical_only, false, JSON.stringify(value));
    assert.equal(value.coverage.semantic_warning, undefined, JSON.stringify(value));
    assert.ok(observation.record_ids.includes(recordId), JSON.stringify(value));
  } else {
    assert.ok(allowRetirement, JSON.stringify(value));
    assert.equal(value.coverage.lexical_only, true, JSON.stringify(value));
    // Match only narrow stop messages, not arbitrary inference failures. These
    // messages do not expose the supervisor's cause; sampled pressure is checked
    // separately and does not constitute an atomic dispatch observation.
    assert.ok(retirementWarnings.has(value.coverage.semantic_warning), JSON.stringify(value));
    assert.deepEqual(observation.record_ids, [], "This bilingual fixture has no lexical match");
    observation.outcome = "retirement-fallback";
  }
  return observation;
}
const circles = Array.from({ length: 7500 }, (_, i) => `<circle cx="${(i * 47) % 2048}" cy="${160 + (i * 79) % 1850}" r="${8 + i % 23}" fill="${["#17594e", "#b58139", "#c45437", "#48789a"][i % 4]}" opacity="0.45"/>`).join("");
const svg = Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 2048 2048"><rect width="2048" height="2048" fill="#f7f2e7"/>${circles}<rect x="40" y="24" width="1968" height="110" rx="20" fill="#17594e"/><text x="80" y="100" fill="#fff" font-family="Noto Sans" font-size="58">Mirket combined load · Gözlemevi</text></svg>`);
assert.ok(svg.length < 1024 * 1024);
const source = join(project, "load.svg"); await writeFile(source, svg, { mode: 0o600 }); await writeFile(join(evidence, "load.svg"), svg, { mode: 0o600 });
const navText = "const telescopeBatteryHours = 9;\nexport const report = telescopeBatteryHours;\n";
const navPath = join(project, "navigation.ts"); await writeFile(navPath, navText, { mode: 0o600 }); await writeFile(join(evidence, "navigation.ts"), navText, { mode: 0o600 });
let succeeded = false;
try {
  const semantic = new Client("semantic"), design = new Client("design"), navigation = new Client("navigation");
  await Promise.all(clients.map(client => client.init()));
  report.runtime = await semantic.tool("runtime_status");
  assert.equal(report.runtime.release_id, release);
  assert.equal(report.runtime.design.configured, true); assert.equal(report.runtime.memory_resources.embedding_configured, true);
  assert.equal(report.runtime.memory_resources.heavy_job_slots, 2); assert.equal(report.runtime.memory_resources.soft_resident_limit_bytes, 8 * 1024 ** 3);
  await sample();
  sampler = setInterval(() => {
    if (sampling) return; sampling = true;
    sample().catch(error => { report.sampling_failure = String(error); }).finally(() => { sampling = false; });
  }, 50);
  phase = "cold-index";
  const prepared = await semantic.tool("memory_prepare", { scope, mutation: { kind: "create", supersedes: null, record: {
    kind: "fact", basis: "observed", content: "Gözlemevi teleskobunun bataryası dokuz saat çalışır.",
    sources: [{ source_id: "fixture:observatory", revision: "fixture-1", locator: "unknown+fixture://observatory/1", captured_at: 123 }],
    status: "current", critical: false, applicability: { task_key: null, checkout_id: null, operation: null }, valid_from: null, valid_until: null,
  } } });
  const committed = await semantic.tool("memory_commit", { scope, operation_id: prepared.operation_id });
  report.memory_commit = committed;
  assert.equal(committed.status, "committed"); assert.equal(committed.semantic_index.status, "indexed", JSON.stringify(committed));
  recordId = committed.record.record_id; report.memory = { record_id: recordId, semantic_index: committed.semantic_index };
  phase = "warm-baseline"; report.baseline_recall = await recall(semantic); await sample();
  for (const number of [1, 2]) {
    await disk(); phase = number === 1 ? "semantic-render" : "semantic-render-navigation";
    const destination = join(project, `render-${number}.png`);
    const preparedDesign = await design.tool("design_prepare", { project_root: project, source_path: source, expected_revision: revision(svg), destination_path: destination, operation: { kind: "svg_render_png", width: 2048, height: 2048 } });
    const round = { number, phase, begin_ms: performance.now() - started, operation_id: preparedDesign.operation_id, semantic: [] };
    report.rounds.push(round);
    await design.tool("design_start", { project_root: project, operation_id: preparedDesign.operation_id });
    let rendering = true;
    const searching = (async () => {
      // The first pair requires 24 successful recalls. With three consumers,
      // retain an explicit retirement response and await the other owners;
      // do not replay the interrupted inference or churn cold model loads.
      for (let i = 0; i < 24 && (number === 1 || rendering || i === 0); i++) {
        const value = await recall(semantic, round.semantic, number === 2);
        if (value.outcome === "retirement-fallback") break;
      }
    })();
    const renderingWork = terminal(design, preparedDesign.operation_id).then(value => { round.receipt = value; }).finally(() => { rendering = false; });
    let navigating = Promise.resolve();
    if (number === 2) {
      navigating = navigation.tool("code_navigate", { project_root: project, request: {
        sources: [{ path: navPath, language: "ts", expected_revision: revision(navText) }], source: 0,
        position_byte: Buffer.byteLength(navText.slice(0, navText.lastIndexOf("telescopeBatteryHours"))), operation: "definitions", limit: 10, typescript_model: { imports: [] },
      } }).then(value => {
        round.navigation = value;
        assert.equal(value.navigation.outcome.status, "locations", JSON.stringify(value));
        assert.deepEqual(value.navigation.outcome.locations[0].location, { start_byte: 6, end_byte: 27 });
      });
    }
    const settled = await Promise.allSettled([searching, renderingWork, navigating]);
    round.settled = settled.map((value, index) => ({ work: ["semantic", "render", "navigation"][index],
      status: value.status, ...(value.status === "rejected" ? { error: String(value.reason) } : {}) }));
    const failures = settled.filter(value => value.status === "rejected");
    assert.equal(failures.length, 0, JSON.stringify(round.settled));
    round.end_ms = performance.now() - started;
    round.semantic_successes = round.semantic.filter(value => value.outcome === "semantic").length;
    round.retirement_fallbacks = round.semantic.filter(value => value.outcome === "retirement-fallback").length;
    if (number === 1) assert.equal(round.semantic_successes, 24);
    const pressureSamples = report.samples.filter(row => row.at_ms >= round.begin_ms && row.at_ms <= round.end_ms);
    round.pressure_levels = [...new Set(pressureSamples.map(row => row.pressure_level))].sort();
    assert.deepEqual(round.pressure_levels, [1], "Qualify this workload only with independently sampled Normal pressure");
    assert.equal(round.receipt.status, "succeeded", JSON.stringify(round.receipt)); assert.equal(round.receipt.scratch_cleanup, "complete");
    const png = await readFile(destination);
    assert.deepEqual([...png.subarray(0, 8)], [137, 80, 78, 71, 13, 10, 26, 10]);
    assert.equal(png.readUInt32BE(16), 2048); assert.equal(png.readUInt32BE(20), 2048);
    assert.equal(round.receipt.artifact_revision, revision(png));
    round.artifact = { bytes: png.length, revision: revision(png), width: 2048, height: 2048 };
    await writeFile(join(evidence, `render-${number}.png`), png, { mode: 0o600 });
    await sample();
    if (number === 2) {
      phase = "post-contention-recovery";
      // A new independent call after both jobs finish must recover semantic
      // retrieval; this is distinct from replaying the retired request.
      report.post_contention_recall = await recall(semantic);
      await sample();
    }
  }
  phase = "external-resource-refusal";
  const unsafe = Buffer.from('<svg xmlns="http://www.w3.org/2000/svg"><image href="https://example.test/unavailable.png"/></svg>');
  const unsafePath = join(project, "unsafe.svg"), unsafeDestination = join(project, "unsafe.png"); await writeFile(unsafePath, unsafe, { mode: 0o600 });
  const refused = await design.tool("design_prepare", { project_root: project, source_path: unsafePath, expected_revision: revision(unsafe), destination_path: unsafeDestination, operation: { kind: "svg_render_png", width: 100, height: 40 } });
  await design.tool("design_start", { project_root: project, operation_id: refused.operation_id });
  [report.refusal] = await Promise.all([terminal(design, refused.operation_id), recall(semantic)]);
  assert.equal(report.refusal.status, "failed"); assert.equal(report.refusal.scratch_cleanup, "complete");
  await assert.rejects(stat(unsafeDestination), { code: "ENOENT" });
  assert.equal(await readFile(source, "utf8"), svg.toString()); assert.equal(await readFile(navPath, "utf8"), navText);
  phase = "before-eof"; await sample();
  const eof = performance.now(); clients.forEach(client => client.child.stdin.end());
  const exits = await bounded(Promise.all(clients.map(client => client.closed)), 45000, "EOF cleanup deadline exceeded");
  report.eof_cleanup_ms = performance.now() - eof; report.server_exits = exits;
  for (const exit of exits) assert.equal(exit.code, 0);
  clearInterval(sampler); while (sampling) await delay(5);
  phase = "after-eof"; report.cleanup = await sample();
  const remaining = processTable().filter(row => knownOwnedPids.has(row.pid) && row.command.includes(root));
  assert.deepEqual(remaining, [], "Previously observed fixture processes must be gone");
  assert.deepEqual(report.cleanup.admission.markers, []); assert.deepEqual(report.cleanup.admission.jobs, []);
  // Probe native flock availability only after EOF, never during admission.
  report.cleanup.locks = cleanupLocks();
  assert.equal(report.cleanup.locks.admission.length, 10);
  assert.equal(report.cleanup.locks.unlocked_registrations.length, report.cleanup.admission.registrations.length);
  assert.equal(report.sampling_failure, undefined);
  const overlap = report.samples.filter(row => row.phase === "semantic-render" && row.processes.some(process => process.command.includes("__embedding_leaf")) && row.processes.some(process => process.command.includes("__design_raster")));
  assert.ok(overlap.length > 0, "Observe real model and raster children simultaneously");
  report.observed_overlap_samples = overlap.length;
  report.semantic_request_overlap_samples = overlap.filter(row => report.requests.some(request => request.client === "semantic" && request.tool === "memory_search"
    && request.begin_ms <= row.at_ms && request.begin_ms + request.elapsed_ms >= row.at_ms)).length;
  assert.ok(report.semantic_request_overlap_samples > 0, "Observe an in-flight semantic request while the raster child lives");
  report.max_sampled_rss_kib = Math.max(...report.samples.map(row => row.rss_kib));
  report.max_sampled_processes = Math.max(...report.samples.map(row => row.processes.length));
  report.max_observed_markers = Math.max(...report.samples.map(row => row.admission.markers.length));
  assert.ok(report.max_observed_markers <= 2);
  report.disk_after = await disk(); report.status = "passed"; succeeded = true;
} catch (error) {
  report.status = "failed"; report.failure = { message: error.message, stack: error.stack };
  throw error;
} finally {
  clearInterval(sampler); while (sampling) await delay(5);
  if (!succeeded) {
    clients.forEach(client => client.child.stdin.end());
    try {
      report.failure_cleanup_exits = await bounded(Promise.allSettled(clients.map(client => client.closed)), 45000, "Failure cleanup deadline exceeded");
      phase = "failed-after-eof"; report.failure_cleanup = await sample();
      report.failure_cleanup.locks = cleanupLocks();
      report.failure_cleanup.remaining_owned = processTable().filter(row => knownOwnedPids.has(row.pid) && row.command.includes(root));
    } catch (error) { report.failure_cleanup_error = error.message; }
    report.retained_fixture = root;
  }
  report.elapsed_ms = performance.now() - started;
  report.stderr = clients.map(client => ({ client: client.name, stderr: client.stderr }));
  await writeFile(join(evidence, "result.json"), JSON.stringify(report, null, 2) + "\n", { mode: 0o600 });
  if (succeeded) await rm(root, { recursive: true, force: true });
  console.log(JSON.stringify({ status: report.status, evidence, release_id: release, elapsed_ms: report.elapsed_ms,
    observed_overlap_samples: report.observed_overlap_samples, max_sampled_rss_kib: report.max_sampled_rss_kib,
    max_observed_markers: report.max_observed_markers, eof_cleanup_ms: report.eof_cleanup_ms }));
}

function cleanupLocks() {
  return JSON.parse(execFileSync("/usr/bin/python3", ["-c", "import fcntl,json,pathlib,sys\np=pathlib.Path(sys.argv[1]);out={'admission':[],'unlocked_registrations':[]}\nfor name in ['heavy-slot-0.lock','heavy-slot-1.lock']+[f'waiting-slot-{i}.lock' for i in range(8)]:\n with (p/name).open('rb') as f:\n  fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB);out['admission'].append(name);fcntl.flock(f,fcntl.LOCK_UN)\nfor path in (p/'resources').glob('root-*.json'):\n with path.open('rb') as f:\n  fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB);out['unlocked_registrations'].append(path.name);fcntl.flock(f,fcntl.LOCK_UN)\nprint(json.dumps(out))", state], { encoding: "utf8", timeout: 10000 }));
}
