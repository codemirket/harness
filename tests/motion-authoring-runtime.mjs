// Bounded runtime oracle for one frozen motion-authoring source project.
// This invokes the real Mirket MCP video route and records effects/evidence;
// receipts and hashes never substitute for visual inspection.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawn, spawnSync } from "node:child_process";
import { copyFile, mkdtemp, mkdir, readFile, readdir, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, relative, resolve } from "node:path";

const [,, mirket, nodePath, adapter, chromium, ffmpeg, ffprobe, evidenceArg, sourceArg] = process.argv;
for (const value of [mirket, nodePath, adapter, chromium, ffmpeg, ffprobe, evidenceArg, sourceArg]) {
  assert.ok(typeof value === "string" && value.startsWith("/"), "all paths must be absolute");
}
const evidenceRoot = resolve(evidenceArg);
const sourceProject = resolve(sourceArg);
await mkdir(evidenceRoot, { recursive: true, mode: 0o700 });
assert.deepEqual(await readdir(evidenceRoot), [], "evidence directory must be empty");
assert.equal((await stat(sourceProject)).isDirectory(), true);
const digest = (bytes) => `sha256:${createHash("sha256").update(bytes).digest("hex")}`;
const supported = /\.(?:html|css|js|mjs|json|svg|vtt|srt|png|jpg|jpeg|webp|gif|ttf|otf|woff|woff2|wav|mp3|m4a|aac|opus|mp4|webm|ogg)$/i;
const walk = async (root) => {
  const output = [];
  for (const item of await readdir(root, { withFileTypes: true })) {
    const file = join(root, item.name);
    if (item.isSymbolicLink()) throw new Error(`source symlink refused: ${file}`);
    if (item.name === ".agents") continue;
    if (item.isDirectory()) output.push(...await walk(file));
    else if (item.isFile() && item.name !== "preview.html") output.push(file);
  }
  return output.sort();
};
const sourcePaths = await walk(sourceProject);
const selected = [];
for (const source of sourcePaths) {
  const path = relative(sourceProject, source);
  if (!supported.test(path)) continue;
  const bytes = await readFile(source);
  selected.push({ path, revision: digest(bytes), bytes: bytes.length, source, data: bytes });
}
assert.ok(selected.some((entry) => entry.path === "index.html"), "source project needs root index.html");
const sourceManifest = selected.map(({ path, revision, bytes }) => ({ path, revision, bytes }));
await writeFile(join(evidenceRoot, "source-manifest.json"), JSON.stringify({ source_project: sourceProject, files: sourceManifest }, null, 2));
const evidenceSource = join(evidenceRoot, "source");
for (const entry of selected) {
  const destination = join(evidenceSource, entry.path);
  await mkdir(dirname(destination), { recursive: true, mode: 0o700 });
  await writeFile(destination, entry.data, { mode: 0o600 });
}

const tempRoot = await mkdtemp(join(tmpdir(), "mirket-motion-authoring-"));
const project = join(tempRoot, "project");
const state = join(tempRoot, "state");
const home = join(tempRoot, "home");
const scratchTmp = join(tempRoot, "tmp");
await mkdir(project, { recursive: true, mode: 0o700 });
await mkdir(state, { recursive: true, mode: 0o700 });
await mkdir(home, { recursive: true, mode: 0o700 });
await mkdir(scratchTmp, { recursive: true, mode: 0o700 });
for (const entry of selected) {
  const destination = join(project, entry.path);
  await mkdir(dirname(destination), { recursive: true, mode: 0o700 });
  await writeFile(destination, entry.data, { mode: 0o600 });
}

const release = "motion-authoring-20261027";
const profile = {
  width: 640, height: 360, fps: { numerator: 12, denominator: 1 }, frame_count: 36,
  format: "mp4", captions_path: selected.some((entry) => entry.path === "captions.json") ? "captions.json" : null,
};
const intent = {
  expected_release_id: release, project_root: project, source_path: join(project, "index.html"),
  expected_revision: selected.find((entry) => entry.path === "index.html").revision,
  assets: selected.filter((entry) => entry.path !== "index.html").map(({ path, revision }) => ({ path, expected_revision: revision })),
  destination_path: join(project, "result.mp4"), profile,
};

const child = spawn(mirket, ["dev", "serve", "--state-dir", state, "--release-id", release,
  "--node", nodePath, "--video-adapter", adapter, "--video-chromium", chromium,
  "--video-ffmpeg", ffmpeg, "--video-ffprobe", ffprobe], {
  stdio: ["pipe", "pipe", "pipe"],
  env: { ...process.env, HOME: home, TMPDIR: scratchTmp, TMP: scratchTmp, TEMP: scratchTmp },
});
let buffer = "", stderr = "", nextId = 0, closed = false;
const pending = new Map();
let resolveClosed;
const closedPromise = new Promise((resolvePromise) => { resolveClosed = resolvePromise; });
child.stdout.setEncoding("utf8").on("data", (part) => {
  buffer += part;
  for (;;) {
    const end = buffer.indexOf("\n"); if (end < 0) break;
    const line = buffer.slice(0, end); buffer = buffer.slice(end + 1);
    if (!line.trim()) continue;
    let frame;
    try { frame = JSON.parse(line); } catch (error) {
      for (const waiter of pending.values()) { clearTimeout(waiter.timer); waiter.reject(error); }
      pending.clear(); continue;
    }
    if (frame.id === undefined) continue;
    const waiter = pending.get(frame.id); if (!waiter) continue;
    pending.delete(frame.id); clearTimeout(waiter.timer);
    if (frame.error) waiter.resolve({ isError: true, error: frame.error }); else waiter.resolve(frame.result);
  }
});
child.stderr.setEncoding("utf8").on("data", (part) => { stderr += part; if (stderr.length > 131_072) stderr = stderr.slice(-131_072); });
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
  if (closed) { rejectPromise(new Error(`server closed before ${method}: ${stderr}`)); return; }
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
const toolError = (value) => value?.error ?? value?.content?.find?.((item) => item.type === "text")?.text ?? null;
const receipt = {
  platform: `${process.platform}-${process.arch}`, release, source_project: sourceProject,
  paths: { mirket, node: nodePath, adapter, chromium, ffmpeg, ffprobe },
  source_manifest: sourceManifest, intent, runtime: null,
};
const stopChild = async () => {
  if (!closed) child.kill("SIGTERM");
  if (!closed) await Promise.race([closedPromise, new Promise((resolvePromise) => setTimeout(resolvePromise, 3_000))]);
  if (!closed) child.kill("SIGKILL");
  if (!closed) await Promise.race([closedPromise, new Promise((resolvePromise) => setTimeout(resolvePromise, 3_000))]);
};

const decode = async (output, frame) => {
  // The admitted FFmpeg build intentionally has no select filter; seek by
  // exact rational frame time after input decode instead.
  const timestamp = (frame / 12).toFixed(6);
  const pngPath = join(evidenceRoot, "decoded", `frame-${String(frame).padStart(2, "0")}.png`);
  const rgbPath = join(evidenceRoot, "decoded", `frame-${String(frame).padStart(2, "0")}.rgb`);
  const png = spawnSync(ffmpeg, ["-v", "error", "-nostdin", "-i", output, "-ss", timestamp, "-frames:v", "1", "-y", pngPath], { encoding: "utf8", timeout: 30_000 });
  assert.equal(png.status, 0, png.stderr);
  const rgb = spawnSync(ffmpeg, ["-v", "error", "-nostdin", "-i", output, "-ss", timestamp, "-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "pipe:1"], { maxBuffer: 2 * 1024 * 1024, timeout: 30_000 });
  assert.equal(rgb.status, 0, rgb.stderr?.toString());
  assert.equal(rgb.stdout.length, 640 * 360 * 3);
  await writeFile(rgbPath, rgb.stdout, { mode: 0o600 });
  return { png: pngPath, rgb: rgbPath, png_bytes: (await stat(pngPath)).size, rgb_bytes: rgb.stdout.length, rgb_revision: digest(rgb.stdout) };
};
const redOracle = (rgbByFrame) => {
  const observed = {};
  for (const [frame, rgb] of Object.entries(rgbByFrame)) {
    const xs = [];
    for (let y = 250; y <= 280; y++) for (let x = 0; x < 640; x++) {
      const offset = (y * 640 + x) * 3;
      if (rgb[offset] > 180 && rgb[offset + 1] < 130 && rgb[offset + 2] < 120) xs.push(x);
    }
    observed[frame] = xs.length ? { min_x: Math.min(...xs), max_x: Math.max(...xs), pixels: xs.length, expected_min_x: 32 + Number(frame) * 8 } : null;
  }
  const present = Object.values(observed).filter(Boolean);
  if (!present.length) return { kind: "generic-output", region: "y=250..280", observed };
  return { kind: "optional-red-rectangle", region: "y=250..280", observed, matches_expected: present.length === Object.keys(observed).length && present.every((v) => Math.abs(v.min_x - v.expected_min_x) <= 4) };
};

try {
  try {
    await rpc("initialize", { protocolVersion: "2025-11-25", capabilities: {}, clientInfo: { name: "motion-authoring-runtime", version: "0.0.1" } });
    child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized", params: {} }) + "\n");
    receipt.runtime = await call("runtime_status", {});
    receipt.prepare = await call("video_prepare", intent);
    if (receipt.prepare?.operation_id) {
      const job = { expected_release_id: release, project_root: project, operation_id: receipt.prepare.operation_id };
      receipt.start = await call("video_start", job);
      const deadline = Date.now() + 150_000;
      do {
        receipt.status = await call("video_status", job);
        if (["failed", "cancelled", "succeeded", "interrupted"].includes(receipt.status?.status)) break;
        await new Promise((resolvePromise) => setTimeout(resolvePromise, 250));
      } while (Date.now() < deadline);
      receipt.result = await call("video_result", job);
      if (receipt.status?.status === "succeeded") {
        const output = join(project, "result.mp4");
        await copyFile(output, join(evidenceRoot, "result.mp4"));
        const probe = spawnSync(ffprobe, ["-v", "error", "-count_frames", "-show_streams", "-show_format", "-of", "json", output], { encoding: "utf8", timeout: 30_000 });
        assert.equal(probe.status, 0, probe.stderr);
        receipt.ffprobe = JSON.parse(probe.stdout);
        const video = receipt.ffprobe.streams?.find((stream) => stream.codec_type === "video");
        assert.equal(video?.width, 640);
        assert.equal(video?.height, 360);
        assert.equal(Number(video?.nb_read_frames), 36);
        const rate = String(video?.avg_frame_rate ?? video?.r_frame_rate ?? "").split("/");
        assert.equal(Number(rate[0]) / Number(rate[1]), 12);
        const decoded = {};
        await mkdir(join(evidenceRoot, "decoded"), { recursive: true, mode: 0o700 });
        for (const frame of [0, 6, 12, 24, 35]) {
          decoded[frame] = await decode(output, frame);
        }
        receipt.decoded = decoded;
        const rgbByFrame = {};
        for (const frame of Object.keys(decoded)) rgbByFrame[frame] = await readFile(decoded[frame].rgb);
        receipt.pixel_oracle = redOracle(rgbByFrame);
        receipt.visual_inspection = "not performed by this runner; inspect saved PNG pixels independently";
      }
    }
  } catch (error) {
    receipt.runner_error = String(error);
    receipt.stderr = stderr;
  }
} finally {
  await writeFile(join(evidenceRoot, "runtime-receipt.json"), JSON.stringify(receipt, null, 2));
  await stopChild();
  await rm(tempRoot, { recursive: true, force: true });
}
const routeError = receipt.status?.error ?? toolError(receipt.status) ?? receipt.prepare?.error ?? toolError(receipt.prepare) ?? receipt.runner_error ?? null;
const routeSucceeded = receipt.status?.status === "succeeded" && !receipt.runner_error;
console.log(JSON.stringify({ status: "observed", evidence: evidenceRoot, route_status: receipt.status?.status ?? null, route_error: routeError, visual_inspection: "not performed" }));
if (!routeSucceeded) process.exitCode = 1;
