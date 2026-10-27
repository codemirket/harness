import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtemp, writeFile, readFile, rm, symlink, realpath } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import vm from 'node:vm';
import { createPreview, validateMotion, previewHtml, assertSeekStable, assertExpectedProperties } from '../scripts/create-motion-preview.mjs';

const motion = () => ({ fps: { numerator: 12, denominator: 1 }, frame_count: 36,
  beats: [{ id: 'start', frame: 0, expected: 'rest' }, { id: 'hold', frame: 24, expected: 'readable' }] });
const source = '<div data-composition-id="scene" data-no-timeline data-width="640" data-height="360" data-duration="3"></div>';
async function fixture(fn) {
  const root = await realpath(await mkdtemp(path.join(tmpdir(), 'motion-preview-')));
  const options = { project: root, entry: path.join(root, 'index.html'), motion: path.join(root, 'beats.json'), output: path.join(root, 'preview.html') };
  try {
    await writeFile(options.entry, source); await writeFile(options.motion, JSON.stringify(motion()));
    await fn(options);
  } finally { await rm(root, { recursive: true, force: true }); }
}

test('creates a separate review page without modifying source', async () => fixture(async opts => {
  const before = await readFile(opts.entry);
  const result = await createPreview(opts);
  assert.equal(result.entry, 'index.html');
  assert.deepEqual(await readFile(opts.entry), before);
  const html = await readFile(opts.output, 'utf8');
  assert.match(html, /max="35"/); assert.match(html, /Only document-timeline CSS\/WAAPI/);
  const script = html.match(/<script>([\s\S]*)<\/script>/)[1];
  new vm.Script(script);
}));
test('does not replace an existing file', async () => fixture(async opts => {
  await writeFile(opts.output, 'retain');
  await assert.rejects(createPreview(opts), /EEXIST/);
  assert.equal(await readFile(opts.output, 'utf8'), 'retain');
}));
test('requires same-project inputs and output', async () => fixture(async opts => {
  await assert.rejects(createPreview({ ...opts, output: path.join(path.dirname(opts.project), 'outside.html') }), /within the project/);
  await assert.rejects(createPreview({ ...opts, entry: 'index.html' }), /absolute/);
}));
test('refuses symlink source aliases', async () => fixture(async opts => {
  const alias = path.join(opts.project, 'alias.html'); await symlink(opts.entry, alias);
  await assert.rejects(createPreview({ ...opts, entry: alias }), /symlink ancestry/);
}));
test('profile cannot quietly drift from authored root', async () => fixture(async opts => {
  for (const html of [source.replace('data-duration="3"', 'data-duration="2"'), source.replace(' data-no-timeline', ''), source.replace('data-duration="3"', '')]) {
    await writeFile(opts.entry, html);
    await assert.rejects(createPreview(opts), /finite no-timeline profile/);
  }
}));
test('rational frame units, bounds and duplicate beats are checked', () => {
  for (const edit of [m => m.fps.numerator = 0, m => m.fps.denominator = 0, m => m.frame_count = 1.2,
    m => m.beats[0].frame = .5, m => m.beats[0].frame = 36, m => m.beats[1].id = 'start', m => m.beats = []]) {
    const value = motion(); edit(value); assert.throws(() => validateMotion(value), /Invalid/);
  }
  const value = motion(); value.fps = { numerator: 30000, denominator: 1001 };
  assert.deepEqual(validateMotion(value).fps, value.fps);
});
test('beat text cannot terminate the embedded script', () => {
  const value = motion(); value.beats[0].expected = '</script><script>throw 1</script>';
  const html = previewHtml({ motion: validateMotion(value), entry: 'index.html', url: 'index.html', width: 640, height: 360 });
  assert.equal((html.match(/<script>/g) ?? []).length, 1);
  new vm.Script(html.match(/<script>([\s\S]*)<\/script>/)[1]);
});
test('a competing source clock cannot pass the frame stability check', () => {
  assertSeekStable([{ playState: 'paused', currentTime: 1000 }, { playState: 'finished', currentTime: 1000 }], 1000);
  for (const animation of [{ playState: 'running', currentTime: 1000 }, { playState: 'paused', currentTime: 0 }, { playState: 'idle', currentTime: null }]) {
    assert.throws(() => assertSeekStable([animation], 1000), /source clock changed/);
  }
});

test('moving neighbours and surviving opacity cannot hide a dropped intended transform', () => {
  const intended = [{ target: 'heart', property: 'transform' }];
  const parsed = [{ target: 'flap', property: 'transform' }, { target: 'heart', property: 'opacity' }];
  assert.throws(() => assertExpectedProperties(parsed, intended), /heart\/transform/);
  assertExpectedProperties([...parsed, { target: 'heart', property: 'transform' }], intended);
  assert.throws(() => assertExpectedProperties([], intended), /Missing accepted/);
  // Presence is deliberately narrower than variation or visual success.
  assertExpectedProperties(parsed, []);
});

test('expected properties are optional, bounded, unique and retained through page generation', async () => fixture(async opts => {
  assert.deepEqual(validateMotion(motion()).expected_properties, []);
  const value = { ...motion(), expected_properties: [{ target: 'heart', property: 'transform' }] };
  await writeFile(opts.motion, JSON.stringify(value));
  await createPreview(opts);
  const script = (await readFile(opts.output, 'utf8')).match(/<script>([\s\S]*)<\/script>/)[1];
  new vm.Script(script);
  assert.deepEqual(validateMotion(value).expected_properties, value.expected_properties);
  for (const expected_properties of [false, {}, [null], [{ target: '', property: 'transform' }],
    [{ target: 'heart', property: 'transform()' }], [...value.expected_properties, ...value.expected_properties],
    Array.from({ length: 129 }, (_, i) => ({ target: `heart-${i}`, property: 'opacity' }))]) {
    assert.throws(() => validateMotion({ ...motion(), expected_properties }), /[Ii]nvalid|Duplicate/);
  }
}));
