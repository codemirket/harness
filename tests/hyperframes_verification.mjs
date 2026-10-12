import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {verifyDecodedFrames} from '../runtime/packages/hyperframes/verification.mjs';

const job = {width: 320, height: 180, fps: 30, frames: 3};
const first = 'a'.repeat(64);
const second = 'b'.repeat(64);
function decoded(hashes, {width = 320, tb = '1/30', pts = [0, 1, 2], durations = [1, 1, 1]} = {}) {
  return `#format: frame checksums\n#version: 2\n#hash: SHA256\n#tb 0: ${tb}\n#dimensions 0: ${width}x180\n`
    + hashes.map((hash, index) => `0, ${pts[index]}, ${pts[index]}, ${durations[index]}, 86400, ${hash}\n`).join('');
}

// A convincing fake: valid dimensions, timestamps and frame count, but frozen
// saved pixels for source samples that change. Static commissions remain valid.
assert.throws(() => verifyDecodedFrames(decoded([first, first, first]), job, 2), /frozen/);
assert.deepEqual(verifyDecodedFrames(decoded([first, first, first]), job, 1), {decoded_frames: 3, decoded_unique_count: 1});
assert.deepEqual(verifyDecodedFrames(decoded([first, second, first]), job, 2), {decoded_frames: 3, decoded_unique_count: 2});
assert.throws(() => verifyDecodedFrames(decoded([first, second]), job, 2), /decoded 2 frames/);
assert.throws(() => verifyDecodedFrames(decoded([first, second, first], {width: 640}), job, 2), /dimensions/);
assert.throws(() => verifyDecodedFrames(decoded([first, second, first], {pts: [0, 2, 2]}), job, 2), /timing/);
assert.throws(() => verifyDecodedFrames(decoded([first, second, first], {durations: [1, 0, 1]}), job, 2), /invalid decoded frame/);
assert.throws(() => verifyDecodedFrames(decoded([first, 'wrong hash', first]), job, 2), /invalid decoded frame/);
assert.throws(() => verifyDecodedFrames(decoded([first, second, first]).replace('#hash: SHA256', '#hash: MD5'), job, 2), /hash format/);
assert.throws(() => verifyDecodedFrames(decoded([first, second, first]) + '0, 3, 3, 1, 86400, '+first+'\n', job, 2), /decoded 4 frames/);
assert.throws(() => verifyDecodedFrames('x'.repeat(256 * 1024 + 1), job, 2), /bounded/);
assert.throws(() => verifyDecodedFrames(decoded([first, second, first]), job, 0), /bounded/);
// WebM timestamps may be quantized to milliseconds; permit one-half tick rather
// than forcing MP4's exact timebase onto a separately supported container.
assert.deepEqual(verifyDecodedFrames(decoded([first, second, first], {tb: '1/1000', pts: [0, 33, 67], durations: [33, 33, 33]}), job, 2), {decoded_frames: 3, decoded_unique_count: 2});
const evidence = {unit_probes: 'passed', real_decoder: 'not_run'};
if (process.argv[2] === '--ffmpeg') {
  const executable = process.argv[3];
  assert.ok(executable && path.isAbsolute(executable) && fs.statSync(executable).isFile(), 'supply an existing absolute FFmpeg executable');
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'mirket-hyperframes-decoder-'));
  const invoke = args => {
    const result = spawnSync(executable, args, {encoding: 'utf8', timeout: 30000, maxBuffer: 256 * 1024, stdio: ['ignore', 'pipe', 'pipe']});
    assert.ifError(result.error);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stderr, '', 'decoder/encoder diagnostics must remain clean');
    return result.stdout;
  };
  try {
    const version = invoke(['-version']).split('\n')[0];
    const cases = [];
    for (const format of ['mp4', 'webm']) {
      for (const fps of [24, 30, 60]) for (const mode of ['changing', 'static']) {
        const actualJob = {...job, fps};
        const output = path.join(temporary, `${mode}-${fps}.${format}`);
        const input = mode === 'changing' ? `testsrc2=size=320x180:rate=${fps}` : `color=c=red:size=320x180:rate=${fps}`;
        const encoding = format === 'mp4' ? ['-c:v', 'libx264', '-preset', 'veryfast'] : ['-c:v', 'libvpx-vp9', '-deadline', 'realtime', '-cpu-used', '8'];
        invoke(['-hide_banner', '-v', 'error', '-nostdin', '-n', '-f', 'lavfi', '-i', input, '-frames:v', '3', '-an', ...encoding, '-threads', '1', '-pix_fmt', 'yuv420p', output]);
        const packetInput = format === 'webm' ? ['-fflags', '+noparse+nofillin'] : [];
        const text = invoke(['-nostdin', '-v', 'error', '-xerror', '-err_detect', 'explode', '-protocol_whitelist', 'file,pipe', '-format_whitelist', 'mov,matroska,webm,wav,mp3', ...packetInput, ...(format === 'webm' ? ['-c:v', 'libvpx-vp9'] : []), '-i', output, '-map', '0:v:0', '-an', ...(format === 'webm' ? ['-pix_fmt', 'rgba'] : []), '-fps_mode', 'passthrough', '-f', 'framehash', '-hash', 'sha256', 'pipe:1']);
        const result = verifyDecodedFrames(text, actualJob, mode === 'changing' ? 2 : 1);
        assert.equal(result.decoded_frames, 3);
        if (mode === 'changing') assert.ok(result.decoded_unique_count > 1);
        else {
          assert.equal(result.decoded_unique_count, 1);
          assert.throws(() => verifyDecodedFrames(text, actualJob, 2), /frozen/);
        }
        const bytes = fs.readFileSync(output);
        cases.push({format, mode, fps, ...result, bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex'), frozen_failure_probe: mode === 'static' ? 'rejected_for_changing_source' : 'not_applicable'});
      }
    }
    const alphaOutput = path.join(temporary, 'alpha-only.webm');
    invoke(['-hide_banner', '-v', 'error', '-nostdin', '-n', '-f', 'lavfi', '-i', 'color=c=red:size=320x180:rate=30,format=rgba,fade=t=in:st=0:d=0.1:alpha=1', '-frames:v', '3', '-an', '-c:v', 'libvpx-vp9', '-deadline', 'realtime', '-cpu-used', '8', '-threads', '1', '-pix_fmt', 'yuva420p', '-auto-alt-ref', '0', alphaOutput]);
    const decodeAlpha = preserveAlpha => invoke(['-nostdin', '-v', 'error', '-xerror', '-err_detect', 'explode', '-protocol_whitelist', 'file,pipe', '-format_whitelist', 'mov,matroska,webm,wav,mp3', '-fflags', '+noparse+nofillin', ...(preserveAlpha ? ['-c:v', 'libvpx-vp9'] : []), '-i', alphaOutput, '-map', '0:v:0', '-an', ...(preserveAlpha ? ['-pix_fmt', 'rgba'] : []), '-fps_mode', 'passthrough', '-f', 'framehash', '-hash', 'sha256', 'pipe:1']);
    const opaqueHashes = decodeAlpha(false);
    assert.equal(verifyDecodedFrames(opaqueHashes, job, 1).decoded_unique_count, 1, 'fixture must change only alpha');
    assert.throws(() => verifyDecodedFrames(opaqueHashes, job, 2), /frozen/, 'built-in VP9 decoder loses alpha changes');
    const alphaResult = verifyDecodedFrames(decodeAlpha(true), job, 2);
    assert.ok(alphaResult.decoded_unique_count > 1, 'alpha-aware hashes must accept alpha-only choreography');
    const alphaBytes = fs.readFileSync(alphaOutput);
    cases.push({format: 'webm', mode: 'alpha_only', fps: 30, ...alphaResult, bytes: alphaBytes.length, sha256: createHash('sha256').update(alphaBytes).digest('hex'), built_in_decoder_drops_alpha: true, alpha_aware_decoder: 'libvpx-vp9+rgba'});
    Object.assign(evidence, {real_decoder: 'passed', version, ffmpeg_sha256: createHash('sha256').update(fs.readFileSync(executable)).digest('hex'), cases});
  } finally {
    fs.rmSync(temporary, {recursive: true, force: true});
  }
}
console.log(JSON.stringify(evidence, null, 2));
