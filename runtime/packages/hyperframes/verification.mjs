// Validate the saved encoding independently of the producer's success receipt.
const MAX_REPORT = 256 * 1024;

export function verifyDecodedFrames(text, job, sourceUnique) {
  if (typeof text !== 'string' || Buffer.byteLength(text) > MAX_REPORT
      || !Number.isInteger(job.frames) || job.frames < 1 || job.frames > 900
      || !Number.isInteger(sourceUnique) || sourceUnique < 1 || sourceUnique > job.frames) {
    throw Error('Invalid bounded saved-video verification input');
  }
  const dimensions = text.match(/^#dimensions 0:\s*(\d+)x(\d+)\s*$/m);
  const timebase = text.match(/^#tb 0:\s*(\d+)\/(\d+)\s*$/m);
  if (!/^#hash:\s*SHA256\s*$/m.test(text)
      || !dimensions || +dimensions[1] !== job.width || +dimensions[2] !== job.height
      || !timebase || +timebase[1] < 1 || +timebase[2] < 1) {
    throw Error('Decoded saved-video hash format, dimensions or timebase differ from the composition');
  }
  const tick = +timebase[1] / +timebase[2];
  if (!Number.isFinite(tick) || tick > 1 / job.fps) throw Error('Saved-video timebase cannot establish the requested frame timing');
  const frames = text.split('\n').filter(line => line.trim() && !line.startsWith('#'));
  if (frames.length !== job.frames) throw Error(`Saved video decoded ${frames.length} frames; expected ${job.frames}`);
  const hashes = new Set();
  for (const [index, line] of frames.entries()) {
    const values = line.split(',').map(value => value.trim());
    if (values.length !== 6 || values[0] !== '0'
        || values.slice(1, 5).some(value => !/^-?\d+$/.test(value) || !Number.isSafeInteger(+value))
        || +values[3] <= 0 || +values[4] <= 0 || +values[4] > job.width * job.height * 8
        || !/^[0-9a-f]{64}$/.test(values[5])) {
      throw Error(`Saved video has an invalid decoded frame at index ${index}`);
    }
    const tolerance = tick / 2 + 1e-6;
    if (Math.abs(+values[2] * tick - index / job.fps) > tolerance
        || Math.abs(+values[3] * tick - 1 / job.fps) > tolerance) {
      throw Error(`Saved video has inconsistent decoded timing at frame ${index}`);
    }
    hashes.add(values[5]);
  }
  if (sourceUnique > 1 && hashes.size === 1) {
    throw Error('Saved video is frozen although the editable composition samples change');
  }
  return {decoded_frames: frames.length, decoded_unique_count: hashes.size};
}
