// Disposable authoring-contract fixtures. This is an adapter preflight test,
// not a renderer qualification and not a substitute for the native MCP smoke.
// Run after the locked build in pi/ with the private, compiled runtime.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFile, mkdtemp, mkdir, readFile, readdir, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const { digest, preflight } = await import("../pi/dist/runtime/video-export.js");
const repoRoot = process.cwd();
const nativeFlag = process.argv.indexOf("--native");
const evidenceFlag = process.argv.indexOf("--evidence-dir");
if (evidenceFlag >= 0) assert.ok(nativeFlag >= 0, "--evidence-dir requires --native");
const nativeArgs = (name) => {
  const index = process.argv.indexOf(name);
  assert.ok(index >= 0 && process.argv[index + 1], `Missing ${name}`);
  return process.argv[index + 1];
};
const root = await realpath(await mkdtemp(join(tmpdir(), "mirket-video-authoring-")));
const previousCwd = process.cwd();
let evidenceDir = null;
if (evidenceFlag >= 0) {
  evidenceDir = nativeArgs("--evidence-dir");
  assert.ok(evidenceDir.startsWith("/"), "--evidence-dir must be absolute");
  await mkdir(evidenceDir, { recursive: true, mode: 0o700 });
  assert.deepEqual(await readdir(evidenceDir), [], "--evidence-dir must be empty");
}
const source = join(root, "source");
await mkdir(source, { mode: 0o700 });

const validHtml = `<!doctype html><html><head><style>@font-face{font-family:Local;src:url("./NotoSans.ttf")}</style></head><body>
<div data-composition-id="demo" data-no-timeline data-width="640" data-height="360" data-duration="2"><div id="subject"></div></div>
<script src="./anime.umd.min.js"></script><script>
window.__hfAnime = [anime.animate("#subject",{x:[0,480],duration:2000,ease:"linear",autoplay:false})];
</script></body></html>`;

const assets = new Map([
  ["index.html", Buffer.from(validHtml)],
  ["NotoSans.ttf", Buffer.from("font fixture")],
  ["anime.umd.min.js", Buffer.from("const anime={animate(){return {seek(){},pause(){},play(){}}}}")],
]);
const writeFixture = async (html = validHtml, extra = new Map()) => {
  for (const [name, bytes] of [...assets, ...extra]) await writeFile(join(source, name), bytes);
  if (html !== validHtml) await writeFile(join(source, "index.html"), Buffer.from(html));
};
const request = async (html = validHtml, extra = new Map()) => {
  await writeFixture(html, extra);
  const selected = new Map([...assets, ...extra]);
  if (html !== validHtml) selected.set("index.html", Buffer.from(html));
  const sources = [...selected].map(([path, bytes]) => ({ path, revision: digest(bytes), bytes: bytes.length }));
  return {
    protocol_version: 1, release_id: "test:video", job_id: "authoring-contract",
    job_root: root, input_root: source, entry_file: "index.html", sources,
    output_path: join(root, "result.mp4"),
    profile: { width: 640, height: 360, fps: { numerator: 12, denominator: 1 }, frame_count: 24, format: "mp4", captions_path: null },
    timeout_ms: 30_000, max_output_bytes: 20 * 1024 * 1024,
  };
};
const rejected = async (html, expected, extra = new Map()) => {
  const value = await request(html, extra);
  await assert.rejects(preflight(value), new RegExp(expected));
};

try {
  const good = await request();
  const result = await preflight(good);
  assert.equal(result.expected_audio, false);
  assert.match((await readFile(join(source, "index.html"), "utf8")), /data-no-timeline/);
  assert.match((await readFile(join(source, "index.html"), "utf8")), /window\.__hfAnime/);

  await rejected(validHtml.replace(" data-no-timeline", ""), "authored_root_profile_mismatch");
  await rejected(validHtml.replace("./NotoSans.ttf", "https://example.invalid/font.ttf"), "remote_or_absolute_asset_refused");
  await rejected(validHtml.replace("./anime.umd.min.js", "./missing.js"), "missing_or_unselected_asset");

  const noRegistration = validHtml.replace("window.__hfAnime = [anime.animate", "window.__not_hfAnime = [anime.animate");
  const noRegistrationRequest = await request(noRegistration);
  await preflight(noRegistrationRequest);
  // Preflight intentionally cannot prove runtime animation registration; this
  // is a model/authoring and decoded-pixel gate, not a false server promise.
  assert.match(noRegistration, /anime\.animate\(/);
  assert.doesNotMatch(noRegistration, /window\.__hfAnime\s*=/);

  let nativeEvidence = null;
  if (nativeFlag >= 0) {
    const chromium = nativeArgs("--chromium");
    const ffmpeg = nativeArgs("--ffmpeg");
    const ffprobe = nativeArgs("--ffprobe");
    const fixture = nativeArgs("--fixture");
    const nativeProject = join(root, "native-project");
    const negativeProject = join(root, "negative-project");
    await mkdir(nativeProject, { mode: 0o700 });
    await mkdir(negativeProject, { mode: 0o700 });
    // The Mirket video policy binds browser profile/crash state below cwd.
    // Keep the direct producer probe inside this disposable root, never the
    // repository checkout or the user's project.
    process.chdir(root);
    const fixtureNames = ["index.html", "NotoSans.ttf", "anime.umd.min.js", "voice.wav", "music.wav"];
    for (const name of fixtureNames) {
      const bytes = await readFile(join(fixture, name));
      await writeFile(join(nativeProject, name), bytes);
      await writeFile(join(negativeProject, name), bytes);
    }
    const registeredHtml = await readFile(join(nativeProject, "index.html"), "utf8");
    const unregisteredHtml = registeredHtml.replace(/window\.__hfAnime\s*=/, "window.__not_hfAnime=");
    assert.notEqual(unregisteredHtml, registeredHtml, "fixture must contain the registered Anime instance");
    await writeFile(join(negativeProject, "index.html"), unregisteredHtml);
    process.env.HYPERFRAMES_FFMPEG_PATH = ffmpeg;
    process.env.HYPERFRAMES_FFPROBE_PATH = ffprobe;
    process.env.MIRKET_VIDEO_POLICY_VERSION = "1";
    process.env.MIRKET_VIDEO_CHROMIUM = chromium;
    const producer = await import(pathToFileURL(join(repoRoot, "pi/node_modules/@hyperframes/producer/dist/index.js")).href);
    const render = async (project, output) => {
      const job = producer.createRenderJob({
        fps: { num: 12, den: 1 }, quality: "standard", format: "mp4", strictness: "strict",
        entryFile: "index.html", workers: 1, useGpu: false, hdrMode: "force-sdr",
        producerConfig: { chromePath: chromium, forceScreenshot: true },
      });
      await producer.executeRenderJob(job, project, output);
    };
    const output = join(root, "native.mp4");
    const negativeOutput = join(root, "native-unregistered.mp4");
    await render(nativeProject, output);
    await render(negativeProject, negativeOutput);
    const frame = (file, seconds) => {
      const decoded = spawnSync(ffmpeg, ["-v", "error", "-nostdin", "-ss", String(seconds), "-i", file,
        "-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "pipe:1"], { maxBuffer: 2 * 1024 * 1024, timeout: 30_000 });
      assert.equal(decoded.status, 0, decoded.stderr?.toString());
      assert.equal(decoded.stdout.length, 640 * 360 * 3);
      return decoded.stdout;
    };
    const saveFrame = (file, seconds, destination) => {
      const saved = spawnSync(ffmpeg, ["-v", "error", "-nostdin", "-ss", String(seconds), "-i", file,
        "-frames:v", "1", "-y", destination], { maxBuffer: 64 * 1024, timeout: 30_000 });
      assert.equal(saved.status, 0, saved.stderr?.toString());
    };
    const pixels = (bytes, range, predicate) => {
      const positions = [];
      for (let y = range[0]; y < range[1]; y++) for (let x = 0; x < 640; x++) {
        const offset = (y * 640 + x) * 3;
        if (predicate(bytes[offset], bytes[offset + 1], bytes[offset + 2])) positions.push(x);
      }
      return positions;
    };
    const red = (bytes) => pixels(bytes, [100, 136], (r, g, b) => r > 180 && g < 130 && b < 120);
    const blue = (bytes) => pixels(bytes, [172, 208], (r, g, b) => r < 100 && g > 60 && g < 190 && b > 120);
    const first = red(frame(output, 0.1));
    const middle = red(frame(output, 1.0));
    const last = red(frame(output, 1.9));
    assert.ok(first.length > 500 && Math.min(...first) < 80, `Anime first frame did not start near x=32: ${first.length}`);
    assert.ok(last.length > 500 && Math.max(...last) > 450, `Anime final frame did not move near x=512: ${last.length}`);
    const negativeFirst = red(frame(negativeOutput, 0.1));
    const negativeMiddle = red(frame(negativeOutput, 1.0));
    const negativeLast = red(frame(negativeOutput, 1.9));
    const negativeBlueFirst = blue(frame(negativeOutput, 0.1));
    const negativeBlueLast = blue(frame(negativeOutput, 1.9));
    assert.ok(negativeFirst.length > 500 && negativeLast.length > 500);
    assert.ok(Math.abs(Math.min(...negativeFirst) - Math.min(...negativeLast)) <= 4,
      "Unregistered Anime object must remain frozen");
    assert.ok(negativeBlueFirst.length > 500 && negativeBlueLast.length > 500
      && Math.min(...negativeBlueLast) - Math.min(...negativeBlueFirst) > 300,
      "Negative control must retain independent CSS motion");
    if (evidenceDir) {
      const savedSource = join(evidenceDir, "source");
      await mkdir(savedSource, { mode: 0o700 });
      for (const name of fixtureNames) await copyFile(join(nativeProject, name), join(savedSource, name));
      await copyFile(output, join(evidenceDir, "registered.mp4"));
      await copyFile(negativeOutput, join(evidenceDir, "unregistered.mp4"));
      for (const [label, file] of [["registered", output], ["unregistered", negativeOutput]]) {
        for (const [point, seconds] of [["first", 0.1], ["middle", 1.0], ["last", 1.9]]) {
          saveFrame(file, seconds, join(evidenceDir, `${label}-${point}.png`));
        }
      }
      await writeFile(join(evidenceDir, "oracle.json"), JSON.stringify({
        platform: `${process.platform}-${process.arch}`, profile: { width: 640, height: 360, fps: 12, frames: 24 },
        registered: { first_red_min_x: Math.min(...first), middle_red_min_x: Math.min(...middle), last_red_min_x: Math.min(...last), last_red_max_x: Math.max(...last) },
        unregistered: { first_red_min_x: Math.min(...negativeFirst), middle_red_min_x: Math.min(...negativeMiddle), last_red_min_x: Math.min(...negativeLast), first_blue_min_x: Math.min(...negativeBlueFirst), last_blue_min_x: Math.min(...negativeBlueLast), assertion: "red frozen while blue CSS motion advances" },
        limits: ["one macOS fixture", "direct HyperFrames execution, not a second MCP run", "no player/listening/Windows/release qualification"],
      }, null, 2));
    }
    nativeEvidence = { output_bytes: (await readFile(output)).length, negative_output_bytes: (await readFile(negativeOutput)).length, first_red_min_x: Math.min(...first), middle_red_min_x: Math.min(...middle), last_red_min_x: Math.min(...last), unregistered_first_red_min_x: Math.min(...negativeFirst), unregistered_last_red_min_x: Math.min(...negativeLast), unregistered_first_blue_min_x: Math.min(...negativeBlueFirst), unregistered_last_blue_min_x: Math.min(...negativeBlueLast), evidence_dir: evidenceDir };
  }

  console.log(JSON.stringify({
    status: "pass", platform: process.platform, arch: process.arch,
    checks: [
      "valid pinned composition root and local selected assets pass adapter preflight",
      "missing data-no-timeline refuses before renderer launch",
      "remote and missing local assets refuse before renderer launch",
      "Anime registration remains an authoring/pixel assertion, not a receipt claim",
    ],
    ...(nativeEvidence ? { native_evidence: nativeEvidence } : {}),
  }));
} finally {
  process.chdir(previousCwd);
  await rm(root, { recursive: true, force: true });
}
