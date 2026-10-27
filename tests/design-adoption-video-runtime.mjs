// Independent runtime oracle for the three frozen video trial specimens.
// It copies sources into disposable projects, calls the real Mirket video MCP
// route, and retains only receipts/evidence outside the trial artifacts.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFile, mkdtemp, mkdir, readdir, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { buildVideoRequest, collectRevisionBoundSources, parseCaptionSelection, retainVideoTrialFailure, selectRevisionBoundSources, sourceManifest, writeVideoTrialEvidence } from "./video-fixture.mjs";

const [,, mirket, nodePath, adapter, chromium, ffmpeg, ffprobe, evidenceArg, trialRoot, captionsFlag, captionsArg] = process.argv;
assert.equal(captionsFlag, "--captions", "explicit --captions PATH|none|metadata-only is required");
const captions = parseCaptionSelection(captionsArg);
for (const value of [mirket, nodePath, adapter, chromium, ffmpeg, ffprobe, evidenceArg, trialRoot]) assert.ok(value?.startsWith("/"));
const evidenceRoot = resolve(evidenceArg);
await mkdir(evidenceRoot, { recursive: true, mode: 0o700 });
assert.deepEqual(await readdir(evidenceRoot), [], "evidence directory must be empty");
async function runVariant(name, sourceRoot, tempRoot, evidenceRoot) {
  const project = join(tempRoot, name, "project");
  await mkdir(project, { recursive: true, mode: 0o700 });
  const allSources = await collectRevisionBoundSources(sourceRoot);
  const { selectedSources: sources } = selectRevisionBoundSources(allSources, captions);
  for (const source of allSources) {
    const destination = join(project, source.path);
    await mkdir(dirname(destination), { recursive: true, mode: 0o700 });
    await writeFile(destination, source.data, { mode: 0o600 });
  }
  const entry = join(project, "index.html");
  assert.equal((await stat(entry)).isFile(), true);
  const intent = buildVideoRequest({ projectRoot: project, release: "video-trial-20261027", selectedSources: sources, captions });
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
  let record = { name, source_root: sourceRoot, captions_selection: captions };
  try {
    await rpc("initialize", { protocolVersion: "2025-11-25", capabilities: {}, clientInfo: { name: "video-trial-oracle", version: "0.0.1" } });
    child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized", params: {} }) + "\n");
    const runtime = await call("runtime_status", {});
    const prepared = await call("video_prepare", intent);
    record = { ...record, all_source_files: sourceManifest(allSources), source_files: sourceManifest(sources), runtime, intent, prepare: prepared };
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
  } catch (error) {
    return retainVideoTrialFailure(record, error);
  } finally {
    if (!closed) child.kill("SIGTERM");
    if (!closed) await Promise.race([closedPromise, new Promise((resolvePromise) => setTimeout(resolvePromise, 3_000))]);
    if (!closed) child.kill("SIGKILL");
    if (!closed) await Promise.race([closedPromise, new Promise((resolvePromise) => setTimeout(resolvePromise, 3_000))]);
  }
}

const tempRoot = await mkdtemp(join(tmpdir(), "mirket-video-trial-oracle-"));
const records = [];
const trialMetadata = {
  platform: `${process.platform}-${process.arch}`, node: nodePath,
  mirket, adapter, chromium, ffmpeg, ffprobe,
  limits: ["No trial source was edited", "Frozen expected refusals are asserted; future accepted sources retain MP4 before cleanup", "No visual claims from receipts"],
};
const writeRecords = async () => writeVideoTrialEvidence(join(evidenceRoot, "runtime-oracle.json"), trialMetadata, records);
try {
  const variants = [
    ["video-no-skill", join(trialRoot, "video-no-skill", "project")],
    ["video-baseline", join(trialRoot, "video-baseline", "project", "driftline")],
    ["video-current", join(trialRoot, "video-current", "project")],
  ];
  for (const [name, root] of variants) {
    let record;
    try {
      record = await runVariant(name, root, tempRoot, evidenceRoot);
    } catch (error) {
      record = retainVideoTrialFailure({ name, source_root: root, captions_selection: captions }, error);
    }
    records.push(record);
    await writeRecords();
  }
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
  await writeRecords();
  console.log(JSON.stringify({ status: "pass", evidence: join(evidenceRoot, "runtime-oracle.json"), records: records.map((record) => ({ name: record.name, prepare: record.prepare, status: record.status, result: record.result })) }));
} finally {
  await rm(tempRoot, { recursive: true, force: true });
}
