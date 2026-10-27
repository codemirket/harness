#!/usr/bin/env node
// Repository authoring utility. No server, browser, renderer or dependency.
// The generated page reviews a same-origin project entry using its native animation handles.
import { readFile, realpath, writeFile, lstat } from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';

export function validateMotion(value) {
  const int = (n, min, max) => Number.isSafeInteger(n) && n >= min && n <= max;
  if (!value || !int(value.fps?.numerator, 1, 60000) || !int(value.fps?.denominator, 1, 1001)
    || value.fps.numerator < value.fps.denominator || value.fps.numerator > 60 * value.fps.denominator
    || !int(value.frame_count, 1, 18000) || value.frame_count * value.fps.denominator > 600 * value.fps.numerator
    || !Array.isArray(value.beats) || !value.beats.length || value.beats.length > 128) throw Error('Invalid finite motion profile');
  const ids = new Set();
  const beats = value.beats.map(b => {
    if (!b || typeof b.id !== 'string' || !b.id.trim() || b.id.length > 120 || ids.has(b.id)
      || !int(b.frame, 0, value.frame_count - 1) || typeof b.expected !== 'string' || !b.expected.trim() || b.expected.length > 2000) throw Error('Invalid motion beat');
    ids.add(b.id);
    return { id: b.id, frame: b.frame, expected: b.expected };
  });
  return { fps: { ...value.fps }, frame_count: value.frame_count, beats };
}

export function assertSeekStable(animations, milliseconds) {
  for (const animation of animations) {
    if (animation.playState === 'running' || typeof animation.currentTime !== 'number'
      || Math.abs(animation.currentTime - milliseconds) > 0.001) {
      throw Error('A source clock changed the selected animation time; stop its playback before reviewing frames');
    }
  }
}

// This function is serialized into the generated project page, not imported
// into the composition. Controls never become part of the exported film.
function installPreview(config, verifySeek) {
  const $ = id => document.getElementById(id);
  const stage = $('stage'), slider = $('frame'), status = $('status'), details = $('details');
  const fps = config.motion.fps.numerator / config.motion.fps.denominator;
  const last = config.motion.frame_count - 1;
  let frame = 0, playing = false, tick = 0, monitor = 0, ready = false, expectedMs = 0;
  let handles = [], stageDocument;
  function stop() { playing = false; cancelAnimationFrame(tick); $('play').textContent = 'Play'; }
  function fail(error) {
    stop(); cancelAnimationFrame(monitor); ready = false;
    status.textContent = `Preview unavailable: ${error.message}. Export parity remains untested.`;
    for (const control of document.querySelectorAll('button,input,select')) control.disabled = true;
  }
  function inspect() {
    const animations = stageDocument.getAnimations();
    const owners = new Map();
    for (const a of animations) {
      if (a instanceof stage.contentWindow.CSSTransition || a.timeline !== stageDocument.timeline) throw Error('Only document-timeline CSS/WAAPI tracks are supported; transitions and scroll timelines need a separate test');
      const target = a.effect?.target;
      if (!target?.id || stageDocument.querySelectorAll(`[id="${CSS.escape(target.id)}"]`).length !== 1) throw Error('Animated targets need unique stable IDs');
      if (a.effect.pseudoElement) throw Error('Pseudo-element animation is outside this preview profile');
      if (a.effect.composite && a.effect.composite !== 'replace') throw Error('Additive animation is outside this preview profile');
      const props = new Set(a.effect.getKeyframes().flatMap(k => Object.keys(k)).filter(k => !['offset','computedOffset','easing','composite'].includes(k)));
      for (const prop of props) {
        const key = `${target.id}/${prop}`;
        if (owners.has(key)) throw Error(`Overlapping property owners: ${key}`);
        owners.set(key, a.animationName || a.id || 'WAAPI');
      }
    }
    if (!animations.length) throw Error('No native animation handles found');
    if (handles.length && (handles.length !== animations.length || handles.some(a => !animations.includes(a)))) throw Error('Animation handles changed; reload after source edits');
    handles = animations;
    return [...owners].map(([property, animation]) => ({ property, animation }));
  }
  function seek(value) {
    if (!ready) throw Error('Preview is not ready');
    if (!Number.isInteger(value) || value < 0 || value > last) throw Error('Frame is outside the finite profile');
    const owners = inspect();
    const ms = value * config.motion.fps.denominator * 1000 / config.motion.fps.numerator;
    for (const a of handles) { a.pause(); a.currentTime = ms; }
    expectedMs = ms;
    frame = value; slider.value = String(frame);
    const beat = config.motion.beats.find(b => b.frame === frame);
    status.textContent = `Frame ${frame} / ${last} · ${(ms / 1000).toFixed(3)} s${beat ? ` · ${beat.id}: ${beat.expected}` : ''}`;
    details.textContent = JSON.stringify({ entry: config.entry, entry_sha256_at_generation: config.entry_sha256, frame, milliseconds: ms, owners,
      tracks: handles.map(a => ({ name: a.animationName || a.id || 'WAAPI', effectTiming: a.effect.getTiming(), keyframes: a.effect.getKeyframes(), playbackRate: a.playbackRate })),
      coverage: 'Native CSS/WAAPI handles only; timers, canvas, media and arbitrary JS are not synchronized. Compare decoded exports.' }, null, 2);
    return { frame, milliseconds: ms, owners };
  }
  const safely = fn => { try { fn(); } catch (error) { fail(error); } };
  $('play').onclick = () => safely(() => {
    if (playing) { stop(); return; }
    if (frame === last) seek(0);
    const start = performance.now(), initial = frame;
    playing = true; $('play').textContent = 'Pause';
    // UI transport only: every tick seeks the existing native animation handles. No new
    // animation engine or independently interpolated source values.
    const advance = now => safely(() => {
      const next = Math.min(last, initial + Math.floor((now - start) * fps / 1000));
      seek(next);
      if (next === last) stop(); else tick = requestAnimationFrame(advance);
    });
    tick = requestAnimationFrame(advance);
  });
  $('back').onclick = () => safely(() => { stop(); seek(Math.max(0, frame - 1)); });
  $('next').onclick = () => safely(() => { stop(); seek(Math.min(last, frame + 1)); });
  slider.oninput = () => safely(() => { stop(); seek(Number(slider.value)); });
  $('beats').onchange = () => safely(() => { stop(); seek(Number($('beats').value)); });
  for (const beat of config.motion.beats) {
    const option = document.createElement('option'); option.value = String(beat.frame);
    option.textContent = `${beat.id} · frame ${beat.frame}`; $('beats').append(option);
  }
  stage.onload = async () => {
    try {
      stop(); cancelAnimationFrame(monitor); ready = false; handles = [];
      stageDocument = stage.contentDocument;
      if (!stageDocument || stage.contentWindow.location.origin !== location.origin) throw Error('Serve the preview and entry from the same project origin');
      await stageDocument.fonts.ready;
      await Promise.all([...stageDocument.images].map(image => image.decode()));
      const roots = stageDocument.querySelectorAll('[data-composition-id]');
      if (roots.length !== 1 || !roots[0].hasAttribute('data-no-timeline')) throw Error('Expected one no-timeline composition root');
      const root = roots[0];
      if (Number(root.dataset.width) !== config.width || Number(root.dataset.height) !== config.height
        || !Number.isFinite(Number(root.dataset.duration)) || Math.abs(Number(root.dataset.duration) - config.motion.frame_count / fps) > 1e-6) throw Error('Composition profile changed; regenerate the preview');
      if (stage.contentWindow.anime || stage.contentWindow.__hfAnime?.length || stage.contentWindow.__timelines) throw Error('External animation runtimes are outside this native CSS/WAAPI preview profile');
      // Force style calculation before discovering the current CSS handles.
      root.getBoundingClientRect(); inspect(); ready = true;
      for (const control of document.querySelectorAll('button,input,select')) control.disabled = false;
      seek(0);
      const watch = () => {
        if (!ready) return;
        try {
          const current = stageDocument.getAnimations();
          if (current.length !== handles.length || handles.some(a => !current.includes(a))) throw Error('Animation handles changed; reload after source edits');
          verifySeek(handles, expectedMs);
          monitor = requestAnimationFrame(watch);
        } catch (error) { fail(error); }
      };
      monitor = requestAnimationFrame(watch);
    } catch (error) { fail(error); }
  };
  window.motionPreview = { seek(value) { stop(); return seek(value); }, state() { return { ready, frame, playing }; } };
  stage.src = config.url;
}

export function previewHtml(config) {
  const encoded = JSON.stringify(config).replaceAll('<', '\\u003c');
  return `<!doctype html><html lang="en"><meta charset="utf-8"><title>Motion review</title>
<style>body{margin:24px;background:#f4f3ef;color:#202421;font:16px system-ui}main{max-width:1000px;margin:auto}iframe{display:block;border:0;background:white;width:${config.width}px;height:${config.height}px;max-width:none}nav{display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin:20px 0}button,select{font:inherit;padding:8px}input{width:260px}pre{white-space:pre-wrap;font-size:12px}.viewport{overflow:auto}p{line-height:1.5}</style>
<main><h1>Motion review</h1><p>Live project CSS/WAAPI preview. Reload after source edits. Source snapshots and export comparisons are recorded separately.</p>
<div class="viewport"><iframe id="stage" title="Authored composition"></iframe></div>
<nav><button id="back" disabled>Previous frame</button><button id="play" disabled>Play</button><button id="next" disabled>Next frame</button>
<label>Frame <input id="frame" type="range" min="0" max="${config.motion.frame_count - 1}" value="0" disabled></label>
<label>Beat <select id="beats" disabled></select></label></nav>
<p id="status" role="status">Loading local composition…</p><details><summary>Source and property owners</summary><pre id="details"></pre></details></main>
<script>(${installPreview.toString()})(${encoded}, ${assertSeekStable.toString()});</script></html>`;
}

export async function createPreview({ project, entry, motion: motionPath, output }) {
  if (![project, entry, motionPath, output].every(p => typeof p === 'string' && path.isAbsolute(p))) throw Error('Use explicit absolute paths');
  const root = await realpath(project);
  const within = p => p !== root && p.startsWith(root + path.sep);
  const source = await realpath(entry), manifest = await realpath(motionPath);
  const destination = path.resolve(output), parent = await realpath(path.dirname(destination));
  if (!within(source) || !within(manifest) || !within(destination) || parent !== path.dirname(destination)
    || root !== path.resolve(project) || source !== path.resolve(entry) || manifest !== path.resolve(motionPath)) throw Error('Paths must be within the project without symlink ancestry');
  for (const p of [source, manifest]) if (!(await lstat(p)).isFile() || (await lstat(p)).size > 1024 * 1024) throw Error('Input must be a bounded regular file');
  if (!source.endsWith('.html') || !destination.endsWith('.html')) throw Error('Entry and output must be HTML');
  const motion = validateMotion(JSON.parse(await readFile(manifest, 'utf8')));
  const html = await readFile(source, 'utf8');
  const tag = html.match(/<[^>]*\bdata-composition-id\s*=\s*["'][^"']+["'][^>]*>/i)?.[0] ?? '';
  const attr = n => Number(tag.match(new RegExp(`\\b${n}\\s*=\\s*["']([^"']+)["']`, 'i'))?.[1]);
  const width = attr('data-width'), height = attr('data-height');
  if (!/\bdata-no-timeline(?:\s|=|>)/i.test(tag) || !Number.isInteger(width) || !Number.isInteger(height)
    || width < 2 || height < 2 || width > 1920 || height > 1920 || width % 2 || height % 2 || width * height > 1920 * 1080
    || Math.abs(attr('data-duration') - motion.frame_count * motion.fps.denominator / motion.fps.numerator) > 1e-6
    || !Number.isFinite(attr('data-duration'))) throw Error('Entry does not match the finite no-timeline profile');
  const url = path.relative(parent, source).split(path.sep).map(encodeURIComponent).join('/');
  const config = { entry: path.relative(root, source), entry_sha256: createHash('sha256').update(html).digest('hex'), url, width, height, motion };
  await writeFile(destination, previewHtml(config), { flag: 'wx', mode: 0o600 });
  return { output: destination, entry: config.entry, coverage: 'project CSS/WAAPI preview; export parity untested' };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const args = process.argv.slice(2), options = {};
  try {
    for (let i = 0; i < args.length; i += 2) {
      const key = args[i]?.replace(/^--/, '');
      if (!['project','entry','motion','output'].includes(key) || !args[i + 1] || key in options) throw Error('Usage: --project ABS --entry ABS --motion ABS --output ABS');
      options[key] = args[i + 1];
    }
    console.log(JSON.stringify(await createPreview(options)));
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
