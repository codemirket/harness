// Metadata checks do not establish decoded playback, visual quality or audible sound.
export function keyframeVerdict(keyframes) {
  const statuses = keyframes.map(item => item.inspection.status);
  if (statuses.includes('failed')) return 'failed';
  if (statuses.includes('changed_pixels')) return 'changed_pixels';
  return statuses.length > 0 && statuses.every(status => status === 'match') ? 'match' : 'captured';
}

function positiveNumber(value) {
  const number = Number(value);
  return Number.isFinite(number) && number > 0 ? number : null;
}

function streamDuration(stream) {
  const direct = positiveNumber(stream.duration);
  if (direct !== null) return direct;
  // Matroska/WebM commonly exposes duration as a tag, not stream.duration.
  const match = /^(\d+):([0-5]\d):([0-5]\d(?:\.\d+)?)$/.exec(stream.tags?.DURATION || '');
  return match ? positiveNumber(Number(match[1]) * 3600 + Number(match[2]) * 60 + Number(match[3])) : null;
}

function frameDuration(stream) {
  const match = /^(\d+)\/(\d+)$/.exec(stream.avg_frame_rate || '');
  const fps = match && Number(match[2]) > 0 ? positiveNumber(Number(match[1]) / Number(match[2])) : null;
  return fps === null ? 0 : 1 / fps;
}

export function encodedMediaEvidence(media, snapshotTimes = []) {
  const duration = positiveNumber(media?.format?.duration);
  const streams = Array.isArray(media?.streams) ? media.streams : [];
  const video = streams.filter(stream => stream.codec_type === 'video' && stream.disposition?.attached_pic !== 1);
  if (duration === null || !video.length || video.some(stream =>
    !Number.isSafeInteger(stream.width) || stream.width < 1 ||
    !Number.isSafeInteger(stream.height) || stream.height < 1)) {
    throw Error('HyperFrames CLI produced unusable video metadata');
  }
  if (snapshotTimes?.some(time => time > duration)) throw Error('Snapshot time exceeds the encoded duration');
  const videoEvidence = video.map(stream => {
    const streamSeconds = streamDuration(stream);
    const start = Number(stream.start_time ?? 0);
    const end = streamSeconds === null || !Number.isFinite(start) ? null : start + streamSeconds;
    // Allow ordinary mux rounding, but reject an audio/container tail that hides
    // a truncated visual stream. Shorter audio is valid and is not rejected.
    const tolerance = Math.max(0.1, 2 * frameDuration(stream));
    if (end !== null && end + tolerance < duration) throw Error('Encoded video stream ends before the container duration');
    return {codec: stream.codec_name, width: stream.width, height: stream.height,
      duration: streamSeconds, end, duration_check: end === null ? 'unavailable' : 'passed'};
  });
  return {duration, video: videoEvidence,
    audio: streams.filter(stream => stream.codec_type === 'audio').map(stream =>
      ({codec: stream.codec_name, duration: streamDuration(stream), channels: stream.channels})),
    requires_playback_inspection: true};
}
