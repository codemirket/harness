import test from 'node:test';
import assert from 'node:assert/strict';
import {keyframeVerdict, encodedMediaEvidence} from '../media-evidence.mjs';

const frames = (...statuses) => statuses.map(status => ({inspection: {status}}));
const video = {codec_type: 'video', codec_name: 'h264', width: 1280, height: 720, duration: '10', avg_frame_rate: '30/1'};
const media = (...streams) => ({format: {duration: '10'}, streams});

test('one matching baseline cannot certify other captured keyframes', () => {
  assert.equal(keyframeVerdict(frames('match', 'captured', 'ready_for_inspection')), 'captured');
  assert.equal(keyframeVerdict(frames('match')), 'match');
  assert.equal(keyframeVerdict(frames('match', 'changed_pixels')), 'changed_pixels');
  assert.equal(keyframeVerdict(frames('changed_pixels', 'failed')), 'failed');
  assert.equal(keyframeVerdict([]), 'captured');
});

test('a positive container duration cannot hide truncated video behind audio', () => {
  assert.throws(() => encodedMediaEvidence(media({...video, duration: '2'},
    {codec_type: 'audio', duration: '10'})), /video stream ends/);
});

test('short audio and mux rounding remain valid', () => {
  const evidence = encodedMediaEvidence(media({...video, duration: '9.97'},
    {codec_type: 'audio', codec_name: 'aac', duration: '2', channels: 2}));
  assert.equal(evidence.video[0].duration_check, 'passed');
  assert.equal(evidence.audio[0].duration, 2);
  assert.equal(evidence.requires_playback_inspection, true);
  assert.equal(encodedMediaEvidence(media(video), null).duration, 10);
});

test('WebM duration tags are checked; missing duration is explicitly unknown', () => {
  const webm = {...video, duration: undefined, tags: {DURATION: '00:00:10.000000000'}};
  assert.equal(encodedMediaEvidence(media(webm)).video[0].duration_check, 'passed');
  assert.throws(() => encodedMediaEvidence(media({...webm, tags: {DURATION: '00:00:02.000000000'}})), /video stream ends/);
  assert.equal(encodedMediaEvidence(media({...video, duration: undefined})).video[0].duration_check, 'unavailable');
});

test('bad container metadata and cover art do not establish usable video', () => {
  for (const duration of ['Infinity', 'NaN', '0', '-1']) {
    assert.throws(() => encodedMediaEvidence({...media(video), format: {duration}}), /unusable/);
  }
  assert.throws(() => encodedMediaEvidence(media({...video, disposition: {attached_pic: 1}})), /unusable/);
  assert.throws(() => encodedMediaEvidence(media(video), [11]), /Snapshot time/);
});
