// Independent runtime oracle for the three frozen video trial specimens.
// It copies sources into disposable projects, calls the real Mirket video MCP
// route, and retains only receipts/evidence outside the trial artifacts.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawn } from "node:child_process";
import { copyFile, cp, mkdtemp, mkdir, readFile, readdir, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, relative, resolve } from "node:path";

const [,, mirket, nodePath, adapter, chromium, ffmpeg, ffprobe, evidenceArg, trialRoot] = process.argv;
for (const value of [mirket, nodePath, adapter, chromium, ffmpeg, ffprobe, evidenceArg, trialRoot]) assert.ok(value?.startsWith("/"));
const evidenceRoot = resolve(evidenceArg);
await mkdir(evidenceRoot, { recursive: true, mode: 0o700 });
assert.deepEqual(await readdir(evidenceRoot), [], "evidence directory must be empty");
const hash = (bytes) => `sha256:${createHash("sha256").update(bytes).digest("hex")}`;
const files = async (root) => {
  const out = [];
  for (const item of await readdir(root, { withFileTypes: true })) {
    const path = join(root, item.name);
    if (item.isDirectory()) out.push(...await files(path));
    else if (item.isFile()) out.push(path);
  }
  return out.sort();
};

async function runVariant(name, sourceRoot, tempRoot, evidenceRoot) {
  const project = join(tempRoot, name, "project");
  await mkdir(project, { recursive: true, mode: 0o700 });
  const sourceFiles = await files(sourceRoot);
  for (const source of sourceFiles) {
    const destination = join(project, relative(sourceRoot, source));
    await mkdir(dirname(destination), { recursive: true, mode: 0o700 });
    await cp(source, destination);
  }
  const entry = join(project, "index.html");
  assert.equal((await stat(entry)).isFile(), true);
  const relativeFiles = (await files(project)).map((file) => relative(project, file)).sort();
  const allSources = [];
  for (const path of relativeFiles) {
    const bytes = await readFile(join(project, path));
    allSources.push({ path, expected_revision: hash(bytes), bytes: bytes.length });
  }
  // README/BRIEF and copied skill material are trial metadata, not renderer
  // assets. Select only adapter-supported source types for the exact film.
  const sources = allSources.filter(({ path }) => !path.startsWith(".agents/")
    && /\.(html|css|js|mjs|json|svg|png|jpg|jpeg|webp|gif|ttf|otf|woff|woff2|wav|mp3|m4a|aac|opus|mp4|webm|ogg)$/i.test(path));
  const profile = { width: 640, height: 360, fps: { numerator: 12, denominator: 1 }, frame_count: 36, format: "mp4", captions_path: null };
  const intent = {
    expected_release_id: "video-trial-20261027", project_root: project,
    source_path: entry, expected_revision: sources.find((source) => source.path === "index.html").expected_revision,
    assets: sources.filter((source) => source.path !== "index.html").map(({ path, expected_revision }) => ({ path, expected_revision })),
    destination_path: join(project, "result.mp4"), profile,
  };
  const state = join(tempRoot, name, "state");
  const home = join(tempRoot, name, "home");
  const scratchTmp = join(tempRoot, name, "tmp");
  await mkdir(state, { recursive: true, mode: 0o700 });
  await mkdir(home, { recursive: true, mode: 0o700 });
  await mkdir(scratchTmp, { recursive: true, mode: 0o700 });
  const child = spawn(mirket, ["dev", "serve", "--state-dir", state, "--release-id", "video-trial-20261027",
    "--node", nodePath, "--video-adapter", adapter, "--video-chromium", chromium,
    "--video-ffmpeg", ffmpeg, "--video-ffprobe", ffprobe], {
    stdio: ["pipe", "pipe", "pipe"],
    env: { ...process.env, HOME: home, TMPDIR: scratchTmp, TMP: scratchTmp, TEMP: scratchTmp },
  });
  let buffer = "", stderr = "", nextId = 0, closed = false;
  const pending = new Map();
  let resolveClosed;
  const closedPromise = new Promise((resolvePromise) => { resolveClosed = resolvePromise; });
  child.stderr.setEncoding("utf8").on("data", (part) => { stderr += part; });
  child.stdout.setEncoding("utf8").on("data", (part) => {
    buffer += part;
    for (;;) {
      const end = buffer.indexOf("\n"); if (end < 0) break;
      const line = buffer.slice(0, end); buffer = buffer.slice(end + 1);
      if (!line.trim()) continue;
      const frame = JSON.parse(line);
      if (frame.id === undefined) continue;
      const waiter = pending.get(frame.id); assert.ok(waiter);
      pending.delete(frame.id); clearTimeout(waiter.timer);
      if (frame.error) waiter.resolve({ isError: true, error: frame.error }); else waiter.resolve(frame.result);
    }
  });
  child.once("error", (error) => {
    closed = true;
    for (const waiter of pending.values()) { clearTimeout(waiter.timer); waiter.reject(error); }
    pending.clear(); resolveClosed({ error: String(error) });
  });
  child.once("close", (code, signal) => {
    closed = true;
    const error = new Error(`server exited (${code ?? signal}); stderr=${stderr}`);
    for (const waiter of pending.values()) { clearTimeout(waiter.timer); waiter.reject(error); }
    pending.clear(); resolveClosed({ code, signal });
  });
  const rpc = (method, params) => new Promise((resolvePromise, rejectPromise) => {
    if (closed) { rejectPromise(new Error(`server already closed before ${method}: ${stderr}`)); return; }
    const id = ++nextId;
    const timer = setTimeout(() => { pending.delete(id); rejectPromise(new Error(`${method} timeout: ${stderr}`)); }, 20_000);
    pending.set(id, { resolve: resolvePromise, reject: rejectPromise, timer });
    try { child.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n"); }
    catch (error) { clearTimeout(timer); pending.delete(id); rejectPromise(error); }
  });
  const call = async (name, params) => {
    const result = await rpc("tools/call", { name, arguments: params });
    if (result?.isError) return result;
    return result?.structuredContent ?? JSON.parse(result.content.find((item) => item.type === "text").text);
  };
  try {
    await rpc("initialize", { protocolVersion: "2025-11-25", capabilities: {}, clientInfo: { name: "video-trial-oracle", version: "0.0.1" } });
    child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized", params: {} }) + "\n");
    const runtime = await call("runtime_status", {});
    const prepared = await call("video_prepare", intent);
    const record = { name, source_root: sourceRoot, all_source_files: allSources, source_files: sources, runtime, intent, prepare: prepared };
    if (!prepared?.operation_id) return record;
    const job = { expected_release_id: "video-trial-20261027", project_root: project, operation_id: prepared.operation_id };
    record.start = await call("video_start", job);
    const deadline = Date.now() + 120_000;
    do {
      record.status = await call("video_status", job);
      if (["failed", "cancelled", "succeeded", "interrupted"].includes(record.status?.status)) break;
      await new Promise((resolvePromise) => setTimeout(resolvePromise, 250));
    } while (Date.now() < deadline);
    record.result = await call("video_result", job);
    assert.ok(["failed", "cancelled", "succeeded", "interrupted"].includes(record.status?.status),
      `video job did not reach terminal status: ${JSON.stringify(record.status)}`);
    if (record.status.status === "succeeded") {
      const output = join(project, "result.mp4");
      await copyFile(output, join(evidenceRoot, `${name}.mp4`));
      record.retained_output = `${name}.mp4`;
    }
    return record;
  } finally {
    if (!closed) child.kill("SIGTERM");
    if (!closed) await Promise.race([closedPromise, new Promise((resolvePromise) => setTimeout(resolvePromise, 3_000))]);
    if (!closed) child.kill("SIGKILL");
    if (!closed) await Promise.race([closedPromise, new Promise((resolvePromise) => setTimeout(resolvePromise, 3_000))]);
  }
}

const tempRoot = await mkdtemp(join(tmpdir(), "mirket-video-trial-oracle-"));
try {
  const variants = [
    ["video-no-skill", join(trialRoot, "video-no-skill", "project")],
    ["video-baseline", join(trialRoot, "video-baseline", "project", "driftline")],
    ["video-current", join(trialRoot, "video-current", "project")],
  ];
  const records = [];
  for (const [name, root] of variants) records.push(await runVariant(name, root, tempRoot, evidenceRoot));
  const expectedErrors = new Map([
    ["video-no-skill", "authored_root_profile_mismatch"],
    ["video-baseline", "authored_root_profile_mismatch"],
    ["video-current", "explicit_local_font_required"],
  ]);
  for (const record of records) {
    assert.equal(record.prepare?.status, "prepared", `${record.name} did not prepare: ${JSON.stringify(record.prepare)}`);
    assert.equal(record.runtime?.video?.configured, true, `${record.name} video was not configured`);
    assert.equal(record.status?.status, "failed", `${record.name} unexpectedly changed outcome`);
    assert.match(record.status?.error ?? "", new RegExp(expectedErrors.get(record.name)));
    assert.equal(record.status.effect, "not_attempted");
    assert.equal(record.status.scratch_cleanup, "removed_verified_owned_files");
  }
  await writeFile(join(evidenceRoot, "runtime-oracle.json"), JSON.stringify({
    platform: `${process.platform}-${process.arch}`, node: nodePath,
    mirket, adapter, chromium, ffmpeg, ffprobe, records,
    limits: ["No trial source was edited", "Frozen expected refusals are asserted; future accepted sources retain MP4 before cleanup", "No visual claims from receipts"],
  }, null, 2));
  console.log(JSON.stringify({ status: "pass", evidence: join(evidenceRoot, "runtime-oracle.json"), records: records.map((record) => ({ name: record.name, prepare: record.prepare, status: record.status, result: record.result })) }));
} finally {
  await rm(tempRoot, { recursive: true, force: true });
}
