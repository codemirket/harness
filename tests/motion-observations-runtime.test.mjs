// Runtime-backed observation trial. It is skipped unless the caller supplies
// the already-qualified private Chromium/FFmpeg paths; no browser is bundled.
import assert from "node:assert/strict";
import { test } from "node:test";
import { createHash } from "node:crypto";
import { spawn, spawnSync } from "node:child_process";
import { copyFile, mkdir, mkdtemp, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { validateObservations } from "../scripts/check-motion-observations.mjs";

const chromium = process.env.MIRKET_PRIVATE_CHROMIUM;
const ffmpeg = process.env.MIRKET_PRIVATE_FFMPEG;
const ffprobe = process.env.MIRKET_PRIVATE_FFPROBE;
const producer = path.resolve("adapters/node_modules/@hyperframes/producer/dist/index.js");
const runtimeReady = [chromium, ffmpeg, ffprobe].every(value => typeof value === "string" && value.startsWith("/"));
const sha = bytes => `sha256:${createHash("sha256").update(bytes).digest("hex")}`;
const sourceHtml = `<!doctype html><html><head><meta charset="utf-8"><style>
@font-face{font-family:Local;src:url("./NotoSans.ttf") format("truetype");font-display:block}
html,body{margin:0;padding:0;background:#fff}#stage{position:relative;width:640px;height:360px;overflow:hidden;background:#fff}
#ball{position:absolute;left:0;top:0;width:20px;height:20px;background:#e22;border-radius:4px}
#label{position:absolute;left:120px;top:10px;width:40px;height:22px;line-height:20px;overflow:hidden;font:16px/20px Local,monospace;color:#111}
</style></head><body><main id="stage" data-composition-id="runtime" data-no-timeline data-width="640" data-height="360" data-duration="1.0833333333333334">
<div id="ball"></div><div id="label">y</div></main><pre id="observation"></pre><script>
const query=new URLSearchParams(location.search), defect=query.get("defect"), requested=Number(query.get("ms")||0);
if(defect==="placement") label.style.left="128px";
if(defect==="clip") label.style.height="12px";
const keyframes=defect==="midpoint"
  ? [{offset:0,transform:"translate(10px,10px)"},{offset:.5,transform:"translate(50px,10px)"},{offset:.75,transform:"translate(70px,10px)"},{offset:1,transform:"translate(50px,10px)"}]
  : [{offset:0,transform:"translate(10px,10px)"},{offset:.5,transform:"translate(50px,10px)"},{offset:1,transform:"translate(50px,10px)"}];
const motion=ball.animate(keyframes,{duration:1000,fill:"both",easing:"linear"}); motion.pause();
window.__hf={duration:1.0833333333333334,seek(seconds){motion.currentTime=Number(seconds)*1000;}};
async function observe(){await document.fonts.load("16px Local","y");await document.fonts.ready;motion.currentTime=requested;
  await new Promise(resolve=>setTimeout(resolve,0));
  const box=element=>{const r=element.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height,visible:r.width>0&&r.height>0,clip:{client_width:element.clientWidth,client_height:element.clientHeight,scroll_width:element.scrollWidth,scroll_height:element.scrollHeight,clipped:element.scrollWidth>element.clientWidth||element.scrollHeight>element.clientHeight}}};
  const loaded=[...document.fonts].some(face=>face.family==="Local"&&face.status==="loaded");
  const payload={frame:Number(query.get("frame")||0),requested_ms:requested,viewport:{width:innerWidth,height:innerHeight,dpr:devicePixelRatio},theme:matchMedia("(prefers-color-scheme: dark)").matches?"dark":"light",locale:navigator.language,font_loaded:loaded,entities:{stage:box(stage),ball:box(ball),label:box(label)}};
  observation.textContent=encodeURIComponent(JSON.stringify(payload));
} observe();
</script></body></html>`;

function decodeDump(stdout) {
  const match = stdout.match(/<pre id="observation">([^<]*)<\/pre>/);
  assert.ok(match, "Chromium dump did not contain the observation payload");
  try { return JSON.parse(decodeURIComponent(match[1])); }
  catch (error) { throw new Error(`invalid browser payload (${match[1].length} chars): ${match[1].slice(-240)}`, { cause: error }); }
}

async function capture(project, frame, ms, defect) {
  const profile = path.join(project, `browser-${defect ?? "clean"}-${frame}`);
  const url = `file://${path.join(project, "index.html")}?frame=${frame}&ms=${ms}${defect ? `&defect=${defect}` : ""}`;
  const child = spawn(chromium, ["--headless=new", "--no-sandbox", "--disable-gpu", "--allow-file-access-from-files",
    "--run-all-compositor-stages-before-draw", "--virtual-time-budget=3000", "--window-size=640,360",
    `--user-data-dir=${profile}`, "--dump-dom", url], { stdio: ["ignore", "pipe", "pipe"] });
  const stdout = await new Promise((resolve, reject) => {
    let output = "", error = "", settled = false;
    const finish = (fn, value) => { if (settled) return; settled = true; clearTimeout(timer); child.kill("SIGKILL"); setTimeout(() => fn(value), 150); };
    const timer = setTimeout(() => finish(reject, new Error(`Chromium observation timeout: ${error.slice(-500)}`)), 8_000);
    child.stdout.setEncoding("utf8"); child.stderr.setEncoding("utf8");
    child.stdout.on("data", chunk => { output += chunk; if (/<pre id="observation">[^<]+<\/pre>/.test(output)) finish(resolve, output); });
    child.stderr.on("data", chunk => { error += chunk; });
    child.once("error", value => finish(reject, value));
  });
  const value = decodeDump(stdout);
  assert.equal(value.frame, frame);
  assert.equal(value.font_loaded, true, `font did not load at frame ${frame}`);
  return value;
}

function packet({ id, sourceBytes, exportBytes = null, fontBytes, observations, checks }) {
  const first = observations[0];
  const captureEnvironment = { viewport: { width: first.viewport.width, height: first.viewport.height }, dpr: first.viewport.dpr, theme: first.theme, locale: first.locale };
  return { schema_version: 1, capture: { id, source_revision: sha(sourceBytes), ...(exportBytes ? { export_binding: "bound", export_revision: sha(exportBytes) } : { export_binding: "unbound_browser_capture" }),
    environment: captureEnvironment,
    fps: { numerator: 12, denominator: 1 }, frame_count: 13, measurement_basis: "browser_frame_geometry",
    font: { path: "NotoSans.ttf", sha256: sha(fontBytes), loaded: true },
    frames: observations.map(value => ({ frame: value.frame, timestamp_ms: value.requested_ms, environment: { viewport: { width: value.viewport.width, height: value.viewport.height }, dpr: value.viewport.dpr, theme: value.theme, locale: value.locale }, entities: value.entities })) }, checks };
}

test("real browser observations catch clean, midpoint, descender clipping and placement defects", { skip: !runtimeReady }, async () => {
  const evidenceParent = process.env.MIRKET_MOTION_EVIDENCE_DIR ?? "/Users/nazmiyilmaz/.codex/task-state/motion-observations-runtime-20261028";
  await mkdir(evidenceParent, { recursive: true, mode: 0o700 });
  const root = await mkdtemp(path.join(evidenceParent, "trial-"));
  try {
    const project = path.join(root, "project");
    await mkdir(project, { mode: 0o700 });
    const fontSource = path.resolve("specimens/daybreak/assets/NotoSans.ttf");
    const fontBytes = await readFile(fontSource);
    await writeFile(path.join(project, "index.html"), sourceHtml, { mode: 0o600 });
    await copyFile(fontSource, path.join(project, "NotoSans.ttf"));
    const sourceBytes = Buffer.from(sourceHtml);
    const times = [[0, 0], [6, 500], [9, 750], [12, 1000]];
    const clean = await Promise.all(times.map(([frame, ms]) => capture(project, frame, ms, null)));
    const cleanChecks = [
      { id: "move", kind: "moves", entity: "ball", from: 0, to: 6, axis: "x", min_delta: 39 },
      { id: "hold", kind: "holds", entity: "ball", frames: [6, 9, 12], tolerance: 0 },
      { id: "inside", kind: "inside", entity: "ball", container: "stage", frames: [0, 6, 9, 12] },
      { id: "position", kind: "position", entity: "label", frames: [0, 6, 9, 12], expected: { x: 120, y: 10 }, tolerance: 0 },
      { id: "no_clip", kind: "no_clip", entity: "label", frames: [0, 6, 9, 12] },
      { id: "separation", kind: "no_overlap", a: "ball", b: "label", frames: [0, 6, 9, 12] }
    ];
    const cleanPacket = packet({ id: "browser-clean", sourceBytes, fontBytes, observations: clean, checks: cleanChecks });
    await writeFile(path.join(root, "browser-clean-observations.json"), JSON.stringify(cleanPacket, null, 2), { mode: 0o600 });
    const midpoint = await Promise.all(times.map(([frame, ms]) => capture(project, frame, ms, "midpoint")));
    const midpointPacket = packet({ id: "browser-midpoint", sourceBytes, fontBytes, observations: midpoint,
      checks: [{ id: "hold", kind: "holds", entity: "ball", frames: [6, 9, 12], tolerance: 0 }] });
    await writeFile(path.join(root, "browser-midpoint-observations.json"), JSON.stringify(midpointPacket, null, 2), { mode: 0o600 });
    const midpointResult = await validateObservations(midpointPacket, { source: path.join(project, "index.html"), font: path.join(project, "NotoSans.ttf") });
    assert.deepEqual(midpointResult.failures, ["hold"]);
    const clipped = await Promise.all(times.map(([frame, ms]) => capture(project, frame, ms, "clip")));
    const clippedPacket = packet({ id: "browser-clip", sourceBytes, fontBytes, observations: clipped,
      checks: [{ id: "no_clip", kind: "no_clip", entity: "label", frames: [0, 6, 9, 12] }] });
    await writeFile(path.join(root, "browser-clip-observations.json"), JSON.stringify(clippedPacket, null, 2), { mode: 0o600 });
    const clippedResult = await validateObservations(clippedPacket, { source: path.join(project, "index.html"), font: path.join(project, "NotoSans.ttf") });
    assert.deepEqual(clippedResult.failures, ["no_clip"]);
    const placement = await Promise.all(times.map(([frame, ms]) => capture(project, frame, ms, "placement")));
    const placementPacket = packet({ id: "browser-placement", sourceBytes, fontBytes, observations: placement,
      checks: [{ id: "position", kind: "position", entity: "label", frames: [0, 6, 9, 12], expected: { x: 120, y: 10 }, tolerance: 0 }] });
    await writeFile(path.join(root, "browser-placement-observations.json"), JSON.stringify(placementPacket, null, 2), { mode: 0o600 });
    const placementResult = await validateObservations(placementPacket, { source: path.join(project, "index.html"), font: path.join(project, "NotoSans.ttf") });
    assert.deepEqual(placementResult.failures, ["position"]);

    process.env.HYPERFRAMES_FFMPEG_PATH = ffmpeg; process.env.HYPERFRAMES_FFPROBE_PATH = ffprobe;
    process.env.MIRKET_VIDEO_POLICY_VERSION = "1"; process.env.MIRKET_VIDEO_CHROMIUM = chromium;
    const hf = await import(pathToFileURL(producer).href);
    const output = path.join(root, "decoded-check.mp4");
    const job = hf.createRenderJob({ fps: { num: 12, den: 1 }, quality: "standard", format: "mp4", strictness: "strict", entryFile: "index.html", workers: 1, useGpu: false, hdrMode: "force-sdr", producerConfig: { chromePath: chromium, forceScreenshot: true } });
    const previousCwd = process.cwd();
    try { process.chdir(root); await hf.executeRenderJob(job, project, output); } finally { process.chdir(previousCwd); }
    const renderedBytes = await readFile(output);
    const boundCleanPacket = packet({ id: "browser-clean-bound-export", sourceBytes, exportBytes: renderedBytes, fontBytes, observations: clean, checks: cleanChecks });
    await writeFile(path.join(root, "browser-clean-bound-export-observations.json"), JSON.stringify(boundCleanPacket, null, 2), { mode: 0o600 });
    const cleanResult = await validateObservations(boundCleanPacket, { source: path.join(project, "index.html"), export: output, font: path.join(project, "NotoSans.ttf") });
    assert.equal(cleanResult.status, "passed", JSON.stringify({ clean, result: cleanResult }));
    const probe = spawnSync(ffprobe, ["-v", "error", "-count_frames", "-show_streams", "-of", "json", output], { encoding: "utf8", timeout: 30_000 });
    assert.equal(probe.status, 0, probe.stderr);
    await writeFile(path.join(root, "decoded-check.ffprobe.json"), probe.stdout, { mode: 0o600 });
    const stream = JSON.parse(probe.stdout).streams.find(value => value.codec_type === "video");
    assert.equal(stream.width, 640); assert.equal(stream.height, 360); assert.ok(Number(stream.nb_read_frames) > 0);
    const decode = spawnSync(ffmpeg, ["-v", "error", "-nostdin", "-i", output, "-f", "null", "-"], { encoding: "utf8", timeout: 30_000 });
    assert.equal(decode.status, 0, decode.stderr);
    const png = path.join(root, "decoded-frame-06.png");
    const frame = spawnSync(ffmpeg, ["-v", "error", "-nostdin", "-ss", "0.5", "-i", output, "-frames:v", "1", "-y", png], { encoding: "utf8", timeout: 30_000 });
    assert.equal(frame.status, 0, frame.stderr);
  } finally { /* retain the task-private evidence root for pixel review */ }
});
