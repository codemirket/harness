import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createHash } from 'node:crypto';
import { access, mkdtemp, readFile, realpath, rm, symlink, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { checkVideoSource } from '../scripts/check-video-source.mjs';
import { createVideoSeed } from '../scripts/create-video-seed.mjs';

const hash = bytes => `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
const root = await realpath(await mkdtemp(path.join('/tmp', 'mirket-video-authoring-tools-')));
const font = path.join(root, 'caller-font.ttf');
const notice = path.join(root, 'caller-notice.txt');
await writeFile(font, Buffer.from('selected local font bytes'));
await writeFile(notice, Buffer.from('Caller supplied notice\n'));

async function fixture(withCaptions = true) {
  const project = path.join(root, `project-${Math.random().toString(16).slice(2)}`);
  await createVideoSeed({ destination: project, font, fontNotice: notice, withCaptions });
  return { project, manifest: path.join(project, 'video-manifest.json') };
}

async function manifestAt(file) { return JSON.parse(await readFile(file, 'utf8')); }
async function saveManifest(file, value) { await writeFile(file, `${JSON.stringify(value, null, 2)}\n`); }

test.after(async () => { await rm(root, { recursive: true, force: true }); });

test('fresh seed is editable, explicit, and statically valid with exact captions', async () => {
  const { project, manifest } = await fixture(true);
  const before = await readFile(manifest);
  const sourceBefore = await readFile(path.join(project, 'index.html'));
  const result = await checkVideoSource({ project, manifest });
  assert.equal(result.status, 'valid');
  assert.deepEqual(await readFile(manifest), before);
  assert.deepEqual(await readFile(path.join(project, 'index.html')), sourceBefore);
  const value = await manifestAt(manifest);
  assert.equal(value.profile.captions_path, 'captions.json');
  assert.deepEqual(value.sources.map(source => source.path), ['index.html', 'SelectedFont.ttf', 'captions.json']);
  assert.equal((await readFile(path.join(project, 'FONT-NOTICE.txt'), 'utf8')), 'Caller supplied notice\n');
});

test('malformed root and two caption errors refuse without source mutation', async () => {
  const { project, manifest } = await fixture(true);
  const htmlPath = path.join(project, 'index.html');
  const captionsPath = path.join(project, 'captions.json');
  const before = await readFile(htmlPath);
  await writeFile(htmlPath, Buffer.from(before.toString().replace('data-no-timeline', '')));
  await writeFile(captionsPath, Buffer.from(JSON.stringify([{ start_frame: 0, endFrame: 2, text: 'wrong key' }, { start_frame: 35, end_frame: 40, text: 'too late' }])));
  const changedManifest = await manifestAt(manifest);
  const changedHtml = await readFile(htmlPath);
  const changedCaptions = await readFile(captionsPath);
  Object.assign(changedManifest.sources.find(source => source.path === 'index.html'), { revision: hash(changedHtml), bytes: changedHtml.length });
  Object.assign(changedManifest.sources.find(source => source.path === 'captions.json'), { revision: hash(changedCaptions), bytes: changedCaptions.length });
  await saveManifest(manifest, changedManifest);
  const result = await checkVideoSource({ project, manifest });
  assert.equal(result.status, 'refused');
  assert.ok(['authored_root_profile_mismatch', 'invalid_caption_cues'].includes(result.code));
  assert.ok(result.diagnostics.some(item => item.phase === 'root'));
  assert.deepEqual(result.diagnostics.filter(item => item.phase === 'caption').map(item => item.cue_index), [0, 1]);
  assert.equal(result.coverage, 'static_only; no_render_no_pixels_no_audio_playback');
  assert.deepEqual(await readFile(htmlPath), changedHtml);
  assert.deepEqual(await readFile(captionsPath), changedCaptions);
  assert.deepEqual(await manifestAt(manifest), changedManifest);
  assert.deepEqual(await readFile(path.join(project, 'FONT-NOTICE.txt')), Buffer.from('Caller supplied notice\n'));
  await writeFile(htmlPath, before);
  const restoredManifest = await manifestAt(manifest);
  const restoredHtml = await readFile(htmlPath);
  Object.assign(restoredManifest.sources.find(source => source.path === 'index.html'), { revision: hash(restoredHtml), bytes: restoredHtml.length });
  await saveManifest(manifest, restoredManifest);
  const captionResult = await checkVideoSource({ project, manifest });
  assert.equal(captionResult.status, 'refused');
  assert.equal(captionResult.code, 'invalid_caption_cues');
  assert.deepEqual(captionResult.diagnostics.filter(item => item.phase === 'caption').map(item => item.cue_index), [0, 1]);
});

test('stale manifest revision refuses and does not write an output', async () => {
  const { project, manifest } = await fixture(false);
  const value = await manifestAt(manifest);
  value.sources.find(source => source.path === 'index.html').revision = hash(Buffer.from('stale'));
  await saveManifest(manifest, value);
  const result = await checkVideoSource({ project, manifest });
  assert.equal(result.status, 'refused');
  assert.equal(result.code, 'source_revision_changed');
  await assert.rejects(access(path.join(project, 'result.mp4')));
});

test('manifest and project symlink paths refuse', async () => {
  const { project, manifest } = await fixture(false);
  const alias = path.join(root, 'project-alias');
  await symlink(project, alias);
  const linked = await checkVideoSource({ project: alias, manifest });
  assert.equal(linked.status, 'refused');
  assert.equal(linked.code, 'invalid_project');
  const manifestAlias = path.join(root, 'manifest-alias.json');
  await symlink(manifest, manifestAlias);
  const linkedManifest = await checkVideoSource({ project, manifest: manifestAlias });
  assert.equal(linkedManifest.status, 'refused');
  assert.equal(linkedManifest.code, 'invalid_manifest');
});

test('manifest path escapes and unknown fields refuse before preflight', async () => {
  const { project, manifest } = await fixture(false);
  const value = await manifestAt(manifest);
  value.sources[0].path = '../outside.html';
  await saveManifest(manifest, value);
  const escaped = await checkVideoSource({ project, manifest });
  assert.equal(escaped.status, 'refused');
  assert.equal(escaped.code, 'invalid_manifest_sources');
  const clean = await manifestAt(manifest);
  clean.sources[0].path = 'index.html';
  clean.unlisted = true;
  await saveManifest(manifest, clean);
  const unknown = await checkVideoSource({ project, manifest });
  assert.equal(unknown.status, 'refused');
  assert.equal(unknown.code, 'invalid_manifest');
});

test('selected source symlink refuses without following the alias', async () => {
  const { project, manifest } = await fixture(false);
  const selectedFont = path.join(project, 'SelectedFont.ttf');
  await rm(selectedFont);
  await symlink(font, selectedFont);
  const result = await checkVideoSource({ project, manifest });
  assert.equal(result.status, 'refused');
  assert.equal(result.code, 'source_symlink_or_ancestry_changed');
});

test('oversized and non-UTF8 manifests refuse with bounded public output', async () => {
  const { project, manifest } = await fixture(false);
  await writeFile(manifest, Buffer.alloc(65537, 0x20));
  const oversized = await checkVideoSource({ project, manifest });
  assert.equal(oversized.status, 'refused');
  assert.equal(oversized.code, 'invalid_manifest');
  await writeFile(manifest, Buffer.from([0xc3, 0x28]));
  const nonUtf8 = await checkVideoSource({ project, manifest });
  assert.equal(nonUtf8.status, 'refused');
  assert.equal(nonUtf8.code, 'invalid_manifest');
  assert.ok(JSON.stringify(nonUtf8).length < 1024);
});

test('seed destination is never overwritten', async () => {
  const destination = path.join(root, 'already-there');
  await writeFile(destination, Buffer.from('keep'));
  await assert.rejects(createVideoSeed({ destination, font, fontNotice: notice }), error => error.publicCode === 'destination_exists');
  assert.equal(await readFile(destination, 'utf8'), 'keep');
});

test('seed refuses a destination whose parent is a symlink alias', async () => {
  const alias = path.join(root, 'parent-alias');
  await symlink(root, alias);
  await assert.rejects(
    createVideoSeed({ destination: path.join(alias, 'seed'), font, fontNotice: notice }),
    error => error.publicCode === 'invalid_destination',
  );
});

test('seed refuses an aggregate source overflow before creating the destination', async () => {
  const largeFont = path.join(root, 'large-font.ttf');
  await writeFile(largeFont, Buffer.alloc(16 * 1024 * 1024));
  const destination = path.join(root, 'too-large');
  await assert.rejects(createVideoSeed({ destination, font: largeFont, fontNotice: notice }),
    error => error.publicCode === 'source_budget');
  await assert.rejects(access(destination), { code: 'ENOENT' });
});
