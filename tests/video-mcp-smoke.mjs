// Opt-in real Rust/MCP/Node/Chromium/FFmpeg qualification in disposable
// development or installed state. --home uses only that verified installation.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtemp, mkdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";
import { deflateSync } from "node:zlib";

const flags = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  assert.ok(process.argv[i]?.startsWith("--") && process.argv[i + 1] && !flags.has(process.argv[i]));
  flags.set(process.argv[i], process.argv[i + 1]);
}
const developmentRequired = ["--mirket", "--node", "--video-adapter", "--chromium", "--ffmpeg", "--ffprobe", "--fixture"];
const installed = flags.has("--home");
const required = installed ? ["--home", "--fixture"] : developmentRequired;
assert.ok([...flags.keys()].every((flag) => required.includes(flag) || ["--format", "--scenario", "--retain-fixture"].includes(flag)));
assert.equal(flags.size, required.length + Number(flags.has("--format")) + Number(flags.has("--scenario")) + Number(flags.has("--retain-fixture")));
if (flags.has("--retain-fixture")) assert.equal(flags.get("--retain-fixture"), "yes");
for (const flag of required) assert.ok(isAbsolute(flags.get(flag)), `Expected absolute ${flag}`);
assert.ok((await stat(flags.get("--fixture"))).isDirectory());
let installedRelease;
if (installed) {
  const home = flags.get("--home");
  assert.ok((await stat(home)).isDirectory());
  const binary = join(home, "bin/mirket");
  const checked = spawnSync(binary, ["dev", "doctor-core", "--home", home, "--json"],
    { encoding: "utf8", timeout: 30_000 });
  assert.equal(checked.status, 0, checked.stderr);
  const doctor = JSON.parse(checked.stdout);
  assert.equal(doctor.status, "healthy");
  assert.ok(doctor.active_components.includes("video") && !doctor.active_components.includes("browser"));
  installedRelease = doctor.active_release_id;
  assert.match(installedRelease, /^sha256:[0-9a-f]{64}$/);
  const root = join(home, "releases", `r-${installedRelease.slice(7)}`);
  flags.set("--mirket", binary);
  flags.set("--ffmpeg", join(root, "runtime/video/bin/ffmpeg"));
  flags.set("--ffprobe", join(root, "runtime/video/bin/ffprobe"));
  for (const flag of ["--mirket", "--ffmpeg", "--ffprobe"]) {
    assert.ok((await stat(flags.get(flag))).isFile(), `Expected installed file ${flag}`);
  }
} else {
  for (const flag of developmentRequired.slice(0, 6)) {
    assert.ok((await stat(flags.get(flag))).isFile(), `Expected file ${flag}`);
  }
}
const format = flags.get("--format") ?? "mp4";
assert.ok(["mp4", "webm", "gif", "png_sequence_tar"].includes(format));
const scenario = flags.get("--scenario") ?? "publish";
assert.ok(["publish", "cancel", "large_png", "captions", "reedit", "audio", "remote_refusal"].includes(scenario));
if (["large_png", "captions"].includes(scenario)) assert.equal(format, "png_sequence_tar");
if (["reedit", "audio"].includes(scenario)) assert.equal(format, "mp4");

const sha = (bytes) => `sha256:${createHash("sha256").update(bytes).digest("hex")}`;
function tarEntry(archive, wanted) {
  for (let offset = 0; offset + 512 <= archive.length;) {
    const header = archive.subarray(offset, offset + 512);
    if (header.every((byte) => byte === 0)) break;
    const name = header.subarray(0, 100).toString("utf8").replace(/\0.*$/s, "");
    const size = Number.parseInt(header.subarray(124, 136).toString("ascii").replace(/\0.*$/s, "").trim(), 8);
    assert.ok(Number.isSafeInteger(size) && size >= 0 && offset + 512 + size <= archive.length);
    if (name === wanted) return archive.subarray(offset + 512, offset + 512 + size);
    offset += 512 + Math.ceil(size / 512) * 512;
  }
  throw new Error(`Missing tar entry ${wanted}`);
}
async function decodedCaptionStrip(png, root, name) {
  const input = join(root, name);
  await writeFile(input, png, { flag: "wx" });
  const decoded = spawnSync(flags.get("--ffmpeg"), ["-v", "error", "-nostdin", "-f", "image2",
    "-i", input, "-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "pipe:1"],
  { maxBuffer: 2 * 1024 * 1024, timeout: 10_000 });
  assert.equal(decoded.status, 0, decoded.stderr?.toString());
  assert.equal(decoded.stdout.length, 640 * 360 * 3);
  return decoded.stdout.subarray(300 * 640 * 3, 350 * 640 * 3);
}
function changedPixels(left, right) {
  assert.equal(left.length, right.length);
  let count = 0;
  for (let offset = 0; offset < left.length; offset += 3) {
    if (left[offset] !== right[offset] || left[offset + 1] !== right[offset + 1] || left[offset + 2] !== right[offset + 2]) count++;
  }
  return count;
}
function decodedVideoHeading(file) {
  const decoded = spawnSync(flags.get("--ffmpeg"), ["-v", "error", "-nostdin", "-i", file,
    "-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "pipe:1"],
  { maxBuffer: 2 * 1024 * 1024, timeout: 10_000 });
  assert.equal(decoded.status, 0, decoded.stderr?.toString());
  assert.equal(decoded.stdout.length, 640 * 360 * 3);
  return decoded.stdout.subarray(20 * 640 * 3, 65 * 640 * 3);
}
async function decodedVideoAudio(file, root) {
  const output = join(root, "decoded-audio.wav");
  const decoded = spawnSync(flags.get("--ffmpeg"), ["-v", "error", "-nostdin", "-i", file,
    "-map", "0:a:0", "-ac", "1", "-ar", "48000", "-c:a", "pcm_s16le", "-f", "wav", output],
  { maxBuffer: 1024 * 1024, timeout: 10_000 });
  assert.equal(decoded.status, 0, decoded.stderr?.toString());
  const wav = await readFile(output);
  assert.equal(wav.toString("ascii", 0, 4), "RIFF");
  assert.equal(wav.toString("ascii", 8, 12), "WAVE");
  let pcm = false, data = null;
  for (let offset = 12; offset + 8 <= wav.length;) {
    const kind = wav.toString("ascii", offset, offset + 4);
    const length = wav.readUInt32LE(offset + 4);
    assert.ok(offset + 8 + length <= wav.length, "Invalid decoded WAV chunk");
    if (kind === "fmt ") {
      assert.ok(length >= 16);
      assert.equal(wav.readUInt16LE(offset + 8), 1); // PCM
      assert.equal(wav.readUInt16LE(offset + 10), 1); // mono
      assert.equal(wav.readUInt32LE(offset + 12), 48_000);
      assert.equal(wav.readUInt16LE(offset + 22), 16);
      pcm = true;
    }
    if (kind === "data") data = wav.subarray(offset + 8, offset + 8 + length);
    offset += 8 + length + (length % 2);
  }
  assert.equal(pcm, true);
  assert.ok(data && data.length === 2 * 96_000, "Expected exactly two seconds of decoded mono PCM");
  const amplitude = (start, hz) => {
    const first = Math.round(start * 48_000), count = 4_800;
    let sine = 0, cosine = 0;
    for (let sample = 0; sample < count; sample++) {
      const value = data.readInt16LE((first + sample) * 2) / 32_768;
      const phase = 2 * Math.PI * hz * sample / 48_000;
      sine += value * Math.sin(phase);
      cosine += value * Math.cos(phase);
    }
    return 2 * Math.hypot(sine, cosine) / count;
  };
  const voiceOnly = amplitude(0.30, 440);
  const bothVoice = amplitude(0.80, 440);
  const bothMusic = amplitude(0.80, 660);
  const musicOnly = amplitude(1.30, 660);
  assert.ok(amplitude(0.05, 440) < 0.001 && amplitude(0.05, 660) < 0.001);
  assert.ok(voiceOnly > 0.12 && amplitude(0.30, 660) < 0.001);
  assert.ok(bothVoice > 0.12 && bothMusic > 0.06);
  assert.ok(amplitude(1.30, 440) < 0.001 && musicOnly > 0.06);
  assert.ok(amplitude(1.80, 440) < 0.001 && amplitude(1.80, 660) < 0.001);
  const gainRatio = bothVoice / bothMusic;
  assert.ok(gainRatio > 1.8 && gainRatio < 2.2, `Unexpected voice/music gain ratio ${gainRatio}`);
  return { voice_only_440hz: voiceOnly, both_440hz: bothVoice,
    both_660hz: bothMusic, music_only_660hz: musicOnly, gain_ratio: gainRatio };
}
const temp = await mkdtemp(join(tmpdir(), "mirket-video-mcp-"));
const state = join(temp, "state"), project = join(temp, "project");
const release = installedRelease ?? "video-mcp-test";
await mkdir(state, { mode: 0o700 }); await mkdir(project, { mode: 0o700 });
const serverArgs = installed ? ["serve", "--home", flags.get("--home")] : ["dev", "serve", "--state-dir", state,
  "--release-id", release, "--node", flags.get("--node"),
  "--video-adapter", flags.get("--video-adapter"), "--video-chromium", flags.get("--chromium"),
  "--video-ffmpeg", flags.get("--ffmpeg"), "--video-ffprobe", flags.get("--ffprobe")];
const child = spawn(flags.get("--mirket"), serverArgs,
  { stdio: ["pipe", "pipe", "pipe"] });
let passed = false;
let nextId = 0, buffer = "", stderr = "", outputBytes = 0;
const pending = new Map();
const exited = new Promise((resolve) => child.on("close", (code, signal) => {
  const failure = new Error(`Server exited (${code ?? signal}); stderr=${stderr}`);
  for (const entry of pending.values()) { clearTimeout(entry.timer); entry.reject(failure); }
  pending.clear(); resolve({ code, signal });
}));
child.stderr.setEncoding("utf8").on("data", (part) => { stderr += part; assert.ok(stderr.length < 65536); });
child.stdout.setEncoding("utf8").on("data", (part) => {
  outputBytes += Buffer.byteLength(part); assert.ok(outputBytes < 2 * 1024 * 1024);
  buffer += part;
  for (;;) {
    const end = buffer.indexOf("\n"); if (end < 0) break;
    const frame = JSON.parse(buffer.slice(0, end)); buffer = buffer.slice(end + 1);
    if (frame.id === undefined) continue;
    const entry = pending.get(frame.id); assert.ok(entry, `Unexpected response ${frame.id}`);
    pending.delete(frame.id); clearTimeout(entry.timer);
    frame.error ? entry.reject(new Error(JSON.stringify(frame.error))) : entry.resolve(frame.result);
  }
});
function rpc(method, params) {
  const id = ++nextId;
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => { pending.delete(id); reject(new Error(`${method} timed out; stderr=${stderr}`)); }, 15000);
    pending.set(id, { resolve, reject, timer });
    child.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n");
  });
}
async function call(name, value) {
  const answer = await rpc("tools/call", { name, arguments: value });
  assert.notEqual(answer.isError, true, `${name}: ${JSON.stringify(answer)}`);
  return answer.structuredContent ?? JSON.parse(answer.content.find((v) => v.type === "text").text);
}
async function terminal(operation_id) {
  const until = Date.now() + 90_000;
  while (Date.now() < until) {
    const status = await call("video_status", { expected_release_id: release, project_root: project, operation_id });
    if (["failed", "cancelled", "interrupted"].includes(status.status)
      || (status.status === "succeeded" && status.scratch_cleanup === "removed_verified_owned_files")) return status;
    await new Promise((resolve) => setTimeout(resolve, 200));
  }
  throw new Error("Video operation did not reach a terminal receipt");
}
try {
  const names = ["index.html", "anime.umd.min.js", "NotoSans.ttf", "voice.wav", "music.wav"];
  try {
    if ((await stat(join(flags.get("--fixture"), "checker.png"))).isFile()) names.push("checker.png");
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }
  const revisions = new Map();
  for (const name of names) {
    const bytes = await readFile(join(flags.get("--fixture"), name));
    await writeFile(join(project, name), bytes);
    revisions.set(name, sha(bytes));
  }
  if (scenario === "captions") {
    const original = await readFile(join(project, "index.html"), "utf8");
    const edited = Buffer.from(original.replace(/<div id="caption">[^<]*<\/div>/, ""));
    assert.doesNotMatch(edited.toString("utf8"), /<div id="caption">/);
    await writeFile(join(project, "index.html"), edited);
    revisions.set("index.html", sha(edited));
    const caption = Buffer.from(JSON.stringify([{ start_frame: 3, end_frame: 21, text: "Hazır · yerel kaynaklar" }]));
    await writeFile(join(project, "captions.json"), caption);
    names.push("captions.json");
    revisions.set("captions.json", sha(caption));
  }
  if (scenario === "remote_refusal") {
    const original = await readFile(join(project, "index.html"), "utf8");
    assert.match(original, /<\/head>/i);
    const changed = Buffer.from(original.replace(/<\/head>/i,
      '<script src="https://example.invalid/mirket-video-probe.js"></script></head>'));
    await writeFile(join(project, "index.html"), changed);
    revisions.set("index.html", sha(changed));
  }
  if (scenario === "large_png") {
    // Fixed pseudorandom RGB source forces 48 lossless frames above 20 MiB.
    // Built-in zlib/CRC32 create a local asset; no image service or dependency.
    const crc = (bytes) => { let value = 0xffffffff; for (const byte of bytes) { value ^= byte; for (let bit = 0; bit < 8; bit++) value = (value >>> 1) ^ (value & 1 ? 0xedb88320 : 0); } return (value ^ 0xffffffff) >>> 0; };
    const chunk = (type, bytes) => { const label=Buffer.from(type),length=Buffer.alloc(4),checksum=Buffer.alloc(4);length.writeUInt32BE(bytes.length);checksum.writeUInt32BE(crc(Buffer.concat([label,bytes])));return Buffer.concat([length,label,bytes,checksum]); };
    const header=Buffer.alloc(13);header.writeUInt32BE(640,0);header.writeUInt32BE(360,4);header[8]=8;header[9]=2;
    const pixels=Buffer.alloc(360*(1+640*3));let seed=123456789;
    for(let y=0;y<360;y++)for(let x=1;x<=640*3;x++){seed^=seed<<13;seed^=seed>>>17;seed^=seed<<5;pixels[y*(1+640*3)+x]=seed&255;}
    const noise=Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),chunk("IHDR",header),chunk("IDAT",deflateSync(pixels)),chunk("IEND",Buffer.alloc(0))]);
    await writeFile(join(project,"noise.png"),noise);names.push("noise.png");revisions.set("noise.png",sha(noise));
    const html=Buffer.from((await readFile(join(project,"index.html"),"utf8")).replace(/<\/body\s*>/i,'<img src="noise.png" style="position:fixed;inset:0;width:640px;height:360px;z-index:2147483647">$&'));
    await writeFile(join(project,"index.html"),html);revisions.set("index.html",sha(html));
  }
  const selectedHtml=await readFile(join(project,"index.html"));
  await rpc("initialize", { protocolVersion: "2025-11-25", capabilities: {}, clientInfo: { name: "video-mcp-smoke", version: "0.0.1" } });
  child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized", params: {} }) + "\n");
  const runtime = await call("runtime_status", {});
  assert.equal(runtime.video.configured, true);
  assert.equal(runtime.video.max_output_bytes, 2 * 1024 * 1024 * 1024);
  assert.equal(runtime.video.max_work_bytes, 5 * 1024 * 1024 * 1024);
  const extension = format === "png_sequence_tar" ? "tar" : format;
  const destination = join(project, `result.${extension}`);
  const intent = {
    expected_release_id: release, project_root: project, source_path: join(project, "index.html"),
    expected_revision: revisions.get("index.html"), destination_path: destination,
    assets: names.slice(1).map((path) => ({ path, expected_revision: revisions.get(path) })),
    profile: { width: 640, height: 360, fps: { numerator: scenario === "large_png" ? 24 : 12, denominator: 1 }, frame_count: scenario === "large_png" ? 48 : 24,
      format, captions_path: scenario === "captions" ? "captions.json" : null },
  };
  const stale = await rpc("tools/call", { name: "video_prepare", arguments: { ...intent,
    destination_path: join(project, "stale.mp4"), expected_revision: `sha256:${"0".repeat(64)}` } });
  assert.equal(stale.isError, true);
  const prepared = await call("video_prepare", intent);
  assert.equal(prepared.status, "prepared");
  const job = { expected_release_id: release, project_root: project, operation_id: prepared.operation_id };
  await call("video_start", job);
  if (scenario === "remote_refusal") {
    const refused = await terminal(prepared.operation_id);
    assert.equal(refused.status, "failed", JSON.stringify(refused));
    assert.match(refused.error, /remote_or_absolute_asset_refused/);
    assert.equal(refused.effect, "not_attempted");
    assert.equal(refused.scratch_cleanup, "removed_verified_owned_files");
    await assert.rejects(readFile(destination), { code: "ENOENT" });
    assert.deepEqual(await readFile(join(project, "index.html")), selectedHtml);
    passed = true;
    console.log(JSON.stringify({ status: "pass", platform: process.platform, arch: process.arch,
      release_id: release, scenario, format,
      checks: ["remote source refused by installed preflight", "no artifact publication", "source preservation", "verified owned cleanup"] }));
  } else if (scenario === "cancel") {
    const until = Date.now() + 15000;
    let running = false;
    while (Date.now() < until) {
      const status = await call("video_status", job);
      if (status.status === "running" && status.scratch_path) { running = true; break; }
      assert.ok(["queued", "running"].includes(status.status), JSON.stringify(status));
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    assert.equal(running, true, "Video did not reach active scratch staging");
    await new Promise((resolve) => setTimeout(resolve, 400));
    await call("video_cancel", job);
    const untilCleanup = Date.now() + 90000;
    let cancelled;
    while (Date.now() < untilCleanup) {
      cancelled = await call("video_status", job);
      if (cancelled.status === "cancelled"
        && ["not_started", "removed_verified_owned_files"].includes(cancelled.scratch_cleanup)) break;
      await new Promise((resolve) => setTimeout(resolve, 200));
    }
    assert.equal(cancelled.status, "cancelled", JSON.stringify(cancelled));
    assert.equal(cancelled.effect, "not_attempted", JSON.stringify(cancelled));
    assert.ok(["not_started", "removed_verified_owned_files"].includes(cancelled.scratch_cleanup), JSON.stringify(cancelled));
    await assert.rejects(readFile(destination), { code: "ENOENT" });
    assert.deepEqual(await readFile(join(project, "index.html")), selectedHtml);
    passed = true;
    console.log(JSON.stringify({ status: "pass", platform: process.platform, arch: process.arch,
      release_id: release, scenario, format,
      checks: ["Rust MCP in-flight cancellation", "no artifact publication", "source preservation", "verified owned cleanup"] }));
  } else {
    const completed = await terminal(prepared.operation_id);
    assert.equal(completed.status, "succeeded", JSON.stringify(completed));
    assert.equal(completed.scratch_cleanup, "removed_verified_owned_files");
    const saved = await readFile(destination);
    if (scenario === "large_png") assert.ok(saved.length > 20 * 1024 * 1024, `Expected a real PNG archive above the old limit, got ${saved.length}`);
    if (format === "mp4") assert.equal(saved.subarray(4, 8).toString(), "ftyp");
    if (format === "webm") assert.deepEqual([...saved.subarray(0, 4)], [0x1a, 0x45, 0xdf, 0xa3]);
    if (format === "gif") assert.match(saved.subarray(0, 6).toString(), /^GIF8[79]a$/);
    if (format === "png_sequence_tar") assert.equal(saved.subarray(257, 263).toString(), "ustar\0");
    let captionPixels = null;
    if (scenario === "captions") {
      const srt = tarEntry(saved, "captions.srt").toString("utf8");
      const vtt = tarEntry(saved, "captions.vtt").toString("utf8");
      assert.match(srt, /00:00:00,250 --> 00:00:01,750/);
      assert.match(vtt, /Hazır · yerel kaynaklar/);
      const first = await decodedCaptionStrip(tarEntry(saved, "frame_000001.png"), project, "caption-frame-1.png");
      const during = await decodedCaptionStrip(tarEntry(saved, "frame_000004.png"), project, "caption-frame-4.png");
      const after = await decodedCaptionStrip(tarEntry(saved, "frame_000022.png"), project, "caption-frame-22.png");
      captionPixels = { within_cue: changedPixels(first, during), after_cue: changedPixels(first, after) };
      assert.ok(captionPixels.within_cue > 1000, "Burned cue must visibly change caption pixels");
      assert.ok(captionPixels.after_cue < 100, "Caption pixels must clear after the supplied cue");
    }
    let audioEvidence = null;
    if (scenario === "audio") {
      assert.match(selectedHtml.toString("utf8"), /data-start="0\.25" data-duration="1" data-volume="0\.5"/);
      assert.match(selectedHtml.toString("utf8"), /data-start="0\.75" data-duration="1" data-volume="0\.25"/);
      audioEvidence = await decodedVideoAudio(destination, project);
    }
    assert.equal(completed.artifact_revision, sha(saved));
    assert.deepEqual(await readFile(join(project, "index.html")), selectedHtml);
    const result = await call("video_result", job);
    assert.equal(result.artifact_revision, sha(saved));
    assert.equal(result.saved_artifact_observation.matches_receipt, true);
    assert.equal(result.saved_artifact_observation.bytes, saved.length);
    assert.equal(result.scratch_cleanup, "removed_verified_owned_files");
    let editEvidence = null;
    if (scenario === "reedit") {
      const beforeText = selectedHtml.toString("utf8");
      assert.match(beforeText, /Mirket motion · Türkçe:/);
      const edited = Buffer.from(beforeText.replace("Mirket motion · Türkçe:", "Mirket revised · Türkçe:"));
      await writeFile(join(project, "index.html"), edited);
      const revisedDestination = join(project, "revised.mp4");
      const revisedIntent = { ...intent, expected_revision: sha(edited), destination_path: revisedDestination };
      const revisedPrepared = await call("video_prepare", revisedIntent);
      const revisedJob = { expected_release_id: release, project_root: project, operation_id: revisedPrepared.operation_id };
      await call("video_start", revisedJob);
      const revisedStatus = await terminal(revisedPrepared.operation_id);
      assert.equal(revisedStatus.status, "succeeded", JSON.stringify(revisedStatus));
      assert.equal(revisedStatus.scratch_cleanup, "removed_verified_owned_files");
      const revised = await readFile(revisedDestination);
      assert.equal(revisedStatus.artifact_revision, sha(revised));
      assert.notEqual(sha(revised), sha(saved));
      assert.deepEqual(await readFile(join(project, "index.html")), edited);
      const revisedResult = await call("video_result", revisedJob);
      assert.equal(revisedResult.saved_artifact_observation.matches_receipt, true);
      editEvidence = { revised_revision: sha(revised), heading_changed_pixels: changedPixels(decodedVideoHeading(destination), decodedVideoHeading(revisedDestination)) };
      assert.ok(editEvidence.heading_changed_pixels > 100, "Edited heading must change decoded frame pixels");
    }
    passed = true;
    console.log(JSON.stringify({ status: "pass", platform: process.platform, arch: process.arch,
      release_id: release, scenario, format, output_revision: sha(saved), output_bytes: saved.length,
      ...(captionPixels ? { caption_pixels: captionPixels } : {}),
      ...(audioEvidence ? { audio_evidence: audioEvidence } : {}),
      ...(editEvidence ? { edit_evidence: editEvidence } : {}),
      checks: ["Rust MCP preparation/start/receipt/publication", "selected Chromium and FFmpeg render", "stale source refusal", "source preservation", ...(scenario === "captions" ? ["supplied sidecar timing and Unicode", "burned cue visible only within frame range"] : []), ...(scenario === "audio" ? ["independent decoded PCM onset and end windows", "two-tone voice/music gain ratio"] : []), ...(scenario === "reedit" ? ["source edit changes decoded frame content", "independent revised receipt and source preservation"] : [])] }));
  }
} finally {
  child.stdin.end();
  const timer = setTimeout(() => child.kill("SIGKILL"), 2000);
  await exited; clearTimeout(timer);
  if (passed && !flags.has("--retain-fixture")) await rm(temp, { recursive: true, force: true });
  else console.error(`Retained ${passed ? "passed" : "failed"} video MCP fixture: ${temp}`);
}
