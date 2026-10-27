#!/usr/bin/env node
// Create a fresh, dependency-free editable HyperFrames source specimen.
// The caller supplies the local font and its notice; no upstream asset is bundled.
import { constants } from 'node:fs';
import { lstat, mkdir, open, realpath, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';

const PROFILE = { width: 640, height: 360, fps: { numerator: 12, denominator: 1 }, frame_count: 36, format: 'mp4' };
const digest = bytes => `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
const absolute = value => typeof value === 'string' && path.isAbsolute(value);

class SeedError extends Error {
  constructor(publicCode) { super(publicCode); this.publicCode = publicCode; }
}

async function sourceFile(value, publicCode) {
  if (!absolute(value)) throw new SeedError(publicCode);
  try {
    const canonical = await realpath(value);
    const metadata = await lstat(value);
    if (!metadata.isFile() || metadata.isSymbolicLink() || canonical !== path.resolve(value)) throw new Error();
    return canonical;
  } catch { throw new SeedError(publicCode); }
}

async function readBounded(file, limit, publicCode) {
  let handle;
  try {
    handle = await open(file, constants.O_RDONLY | constants.O_NOFOLLOW);
    const before = await handle.stat();
    if (!before.isFile() || before.size > limit) throw new Error();
    const bytes = Buffer.alloc(before.size);
    let offset = 0;
    while (offset < bytes.length) {
      const { bytesRead } = await handle.read(bytes, offset, bytes.length - offset, offset);
      if (!bytesRead) throw new Error();
      offset += bytesRead;
    }
    const after = await handle.stat();
    if (after.size !== before.size || after.mtimeMs !== before.mtimeMs) throw new Error();
    return bytes;
  } catch { throw new SeedError(publicCode); }
  finally { if (handle) await handle.close().catch(() => {}); }
}

const seedHtml = fontFile => `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>Motion seed</title>
<style>
@font-face{font-family:SeedSans;src:url("./${fontFile}");font-display:swap}
html,body{margin:0;width:100%;height:100%;overflow:hidden;background:#f4efe4;color:#183b35;font-family:SeedSans,sans-serif}
[data-composition-id="seed"]{position:relative;width:640px;height:360px;background:#f4efe4}
#subject{position:absolute;left:32px;top:150px;width:120px;height:64px;border-radius:32px;background:#c79b45}
</style></head><body>
<div data-composition-id="seed" data-no-timeline data-width="640" data-height="360" data-duration="3">
  <div id="subject" aria-label="Moving seed shape"></div>
</div>
<script>
const subject = document.getElementById('subject');
subject.animate([{transform:'translateX(0px)'},{transform:'translateX(456px)'}], {duration:3000,easing:'linear',fill:'both'});
</script></body></html>
`;
const captions = '[{"start_frame":0,"end_frame":12,"text":"Motion seed"}]\n';

export async function createVideoSeed({ destination, font, fontNotice, withCaptions = false }) {
  if (![destination, font, fontNotice].every(absolute) || !path.isAbsolute(destination)) throw new SeedError('invalid_seed_paths');
  const fontPath = await sourceFile(font, 'invalid_font');
  const noticePath = await sourceFile(fontNotice, 'invalid_font_notice');
  const extension = path.extname(fontPath).toLowerCase();
  if (!['.ttf', '.otf', '.woff', '.woff2'].includes(extension)) throw new SeedError('unsupported_font_type');
  const fontFile = `SelectedFont${extension}`;
  let parent;
  try { parent = await realpath(path.dirname(destination)); } catch { throw new SeedError('invalid_destination'); }
  if (path.dirname(destination) !== parent || destination === parent) throw new SeedError('invalid_destination');
  try { await lstat(destination); throw new SeedError('destination_exists'); } catch (error) {
    if (error instanceof SeedError) throw error;
    if (error?.code !== 'ENOENT') throw new SeedError('invalid_destination');
  }
  const files = new Map([
    ['index.html', Buffer.from(seedHtml(fontFile))],
    [fontFile, await readBounded(fontPath, 16 * 1024 * 1024, 'invalid_font')],
    ['FONT-NOTICE.txt', await readBounded(noticePath, 65536, 'invalid_font_notice')],
  ]);
  const profile = { ...PROFILE, fps: { ...PROFILE.fps }, captions_path: withCaptions ? 'captions.json' : null };
  if (withCaptions) files.set('captions.json', Buffer.from(captions));
  const sources = [...files]
    .filter(([name]) => name !== 'FONT-NOTICE.txt')
    .map(([name, bytes]) => ({ path: name, revision: digest(bytes), bytes: bytes.length }));
  if (sources.reduce((total, source) => total + source.bytes, 0) > 16 * 1024 * 1024) {
    throw new SeedError('source_budget');
  }
  const manifest = { schema_version: 1, entry_file: 'index.html', profile, sources };
  try { await mkdir(destination, { mode: 0o700 }); }
  catch { throw new SeedError('destination_raced'); }
  try {
    for (const [name, bytes] of files) await writeFile(path.join(destination, name), bytes, { flag: 'wx', mode: 0o600 });
    await writeFile(path.join(destination, 'video-manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`, { flag: 'wx', mode: 0o600 });
  } catch (error) {
    throw new SeedError(error?.code === 'EEXIST' ? 'destination_raced' : 'seed_write_failed');
  }
  return { status: 'created', project: destination, manifest: 'video-manifest.json', with_captions: withCaptions };
}

function options(args) {
  if (args.length < 6 || args.length > 7 || args[0] !== '--destination' || args[2] !== '--font' || args[4] !== '--font-notice'
    || !absolute(args[1]) || !absolute(args[3]) || !absolute(args[5])) throw new SeedError('usage');
  if (args.length === 7 && args[6] !== '--with-captions') throw new SeedError('usage');
  return { destination: args[1], font: args[3], fontNotice: args[5], withCaptions: args.length === 7 };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  try { process.stdout.write(`${JSON.stringify(await createVideoSeed(options(process.argv.slice(2))))}\n`); }
  catch (error) { process.stdout.write(`${JSON.stringify({ status: 'refused', code: error?.publicCode ?? 'seed_failed' })}\n`); process.exitCode = 1; }
}
