// Mirket owns job isolation and encoding; Motion Canvas owns scene geometry and frame playback.
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { once } from 'node:events';
import * as fs from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const ORIGIN = 'https://mirket.invalid';
const MAX_FILE = 8 * 1024 * 1024;
const MAX_OUTPUT = 32 * 1024 * 1024;
const MAX_LOG = 128 * 1024;
const FRAME_TIMEOUT = 3000;
const children = new Set();
const failures = [];
let browser;
let stopping = false;
let diagnosticBytes = 0;

function diagnostic(message) {
  if (diagnosticBytes >= 8192) return;
  const text = String(message).replace(/[\x00-\x08\x0b-\x1f\x7f]/g, '?').slice(0, 1200);
  diagnosticBytes += Buffer.byteLength(text);
  process.stderr.write(`${text}\n`);
}

function problem(kind, message) {
  if (failures.length < 12) failures.push(`${kind}: ${String(message).slice(0, 600)}`);
}

function assertClean() {
  if (stopping) throw new Error('Job cancelled.');
  if (failures.length) throw new Error(failures.join('\n'));
}

function bounded(promise, ms, label) {
  let timer;
  return Promise.race([
    promise,
    new Promise((_, reject) => {
      timer = setTimeout(() => reject(new Error(`${label} exceeded ${ms} ms.`)), ms);
    }),
  ]).finally(() => clearTimeout(timer));
}

function childProcess(executable, args, { input = false, limit = MAX_LOG, onStderr } = {}) {
  const child = spawn(executable, args, {
    cwd: process.cwd(),
    detached: false,
    stdio: [input ? 'pipe' : 'ignore', 'pipe', 'pipe'],
    windowsHide: true,
  });
  children.add(child);
  let stdout = Buffer.alloc(0);
  let stderr = Buffer.alloc(0);
  let overflow = false;
  const receive = (which, chunk) => {
    if (which === 'stderr') onStderr?.(chunk.toString('utf8'));
    const prior = which === 'stdout' ? stdout : stderr;
    if (prior.length + chunk.length > limit) {
      overflow = true;
      child.kill('SIGKILL');
      return;
    }
    if (which === 'stdout') stdout = Buffer.concat([stdout, chunk]);
    else stderr = Buffer.concat([stderr, chunk]);
  };
  child.stdout.on('data', chunk => receive('stdout', chunk));
  child.stderr.on('data', chunk => receive('stderr', chunk));
  // A closed encoder pipe must reject the job, never become an unhandled event.
  child.stdin?.on('error', error => problem('encoder input', error.message));
  const done = new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('close', (code, signal) => {
      children.delete(child);
      if (overflow) reject(new Error(`${path.basename(executable)} exceeded its diagnostic output limit.`));
      else if (code !== 0) reject(new Error(`${path.basename(executable)} failed (${signal || code}): ${stderr.toString('utf8').slice(-2400)}`));
      else resolve({ stdout: stdout.toString('utf8'), stderr: stderr.toString('utf8') });
    });
  });
  done.catch(() => {});
  return { child, done };
}

async function cleanup() {
  const active = [...children];
  // Browser.close asks Chromium to reap its renderer children before its root exits.
  if (browser) await bounded(browser.close(), 1000, 'Browser shutdown').catch(() => {});
  for (const child of active) if (child.exitCode === null) child.kill('SIGTERM');
  await Promise.all(active.map(child => child.exitCode !== null
    ? undefined : bounded(once(child, 'close'), 1000, 'Child shutdown').catch(() => {})));
  for (const child of active) if (child.exitCode === null) child.kill('SIGKILL');
}

for (const signal of ['SIGTERM', 'SIGINT']) {
  process.once(signal, () => {
    stopping = true;
    diagnostic('Motion Canvas job cancelled; closing browser and encoder.');
    void cleanup().finally(() => process.exit(signal === 'SIGINT' ? 130 : 143));
  });
}
process.once('exit', () => {
  for (const child of children) if (child.exitCode === null) child.kill('SIGKILL');
});


async function launchBrowser(chromium, executable) {
  const profile = await fs.mkdtemp(path.join(process.cwd(), 'browser-profile-'));
  let devtools = '';
  let announce;
  const announced = new Promise(resolve => { announce = resolve; });
  const launched = childProcess(executable, [
    '--no-sandbox', '--headless=new', '--remote-debugging-address=127.0.0.1', '--remote-debugging-port=0',
    `--user-data-dir=${profile}`, '--no-first-run', '--no-default-browser-check',
    '--disable-background-networking', '--disable-component-update', '--disable-default-apps',
    '--disable-extensions', '--disable-sync', '--disable-breakpad', '--disable-domain-reliability',
    '--disable-client-side-phishing-detection', '--metrics-recording-only', '--mute-audio',
    '--no-pings', '--force-color-profile=srgb',
    // Avoid background compositor scheduling during explicit Motion Canvas seeks.
    '--disable-threaded-animation',
    // Redraw whole tiles when the Stage presents a newly sought frame.
    '--disable-partial-raster',
    '--disable-features=Translate,MediaRouter,OptimizationHints,AutofillServerCommunication',
    '--proxy-server=http://127.0.0.1:9', '--proxy-bypass-list=<-loopback>',
    '--force-webrtc-ip-handling-policy=disable_non_proxied_udp', 'about:blank',
  ], {
    onStderr(chunk) {
      devtools = (devtools + chunk).slice(-16384);
      const match = devtools.match(/DevTools listening on (ws:\/\/127\.0\.0\.1:\d+\/devtools\/browser\/[\w-]+)/);
      if (match) announce(match[1]);
    },
  });
  const endpoint = await bounded(Promise.race([
    announced,
    launched.done.then(() => { throw new Error('Chromium exited before opening its local debugging endpoint.'); }),
  ]), 10000, 'Chromium startup').catch(error => { throw new Error(`${error.message} ${devtools.slice(-2400)}`); });
  browser = await chromium.connectOverCDP(endpoint, { timeout: 5000 });
  browser.on('disconnected', () => { if (!stopping) problem('browser', 'Chromium disconnected before job completion.'); });
  return profile;
}


async function render(page, job, ffmpeg) {
  const encoder = childProcess(ffmpeg, [
    '-hide_banner', '-loglevel', 'error', '-nostdin', '-n',
    '-f', 'image2pipe', '-vcodec', 'png', '-framerate', String(job.fps), '-i', 'pipe:0',
    '-map', '0:v:0', '-an', '-c:v', 'libx264', '-threads', '2', '-preset', 'veryfast', '-crf', '18',
    '-pix_fmt', 'yuv420p', '-r', String(job.fps), '-frames:v', String(job.frames),
    '-movflags', '+faststart', '-f', 'mp4', job.output,
  ], { input: true });
  for (let frame = 0; frame < job.frames; frame++) {
    const bytes = await capture(page, job, frame);
    if (!encoder.child.stdin.write(bytes)) await bounded(once(encoder.child.stdin, 'drain'), FRAME_TIMEOUT, 'Encoder backpressure');
    if (frame % 30 === 0) {
      const size = await fs.stat(job.output).then(stat => stat.size).catch(() => 0);
      if (size > MAX_OUTPUT) throw new Error('Encoded output exceeds 32 MiB.');
    }
  }
  encoder.child.stdin.end();
  await bounded(encoder.done, 10000, 'MP4 finalization');
  const output = await fs.stat(job.output);
  if (!output.isFile() || !output.size || output.size > MAX_OUTPUT) throw new Error('Encoder produced no usable bounded MP4.');
  // Decode every saved frame. Framehash reports actual decoded geometry and timebase.
  const decoded = childProcess(ffmpeg, [
    '-hide_banner', '-loglevel', 'info', '-nostdin', '-xerror', '-err_detect', 'explode',
    '-i', job.output, '-map', '0:v:0', '-an', '-fps_mode', 'passthrough',
    '-f', 'framehash', '-hash', 'sha256', 'pipe:1',
  ], { limit: 256 * 1024 });
  const result = await bounded(decoded.done, 10000, 'Saved MP4 decoding');
  const dimension = result.stdout.match(/^#dimensions 0:\s*(\d+)x(\d+)\s*$/m);
  const timebase = result.stdout.match(/^#tb 0:\s*(\d+)\/(\d+)\s*$/m);
  const source = result.stderr.split('Stream mapping:')[0];
  if (!/Stream #0:0[^\n]*Video: h264\b/.test(source)) throw new Error('Saved output is not H.264 video.');
  if (!dimension || +dimension[1] !== job.width || +dimension[2] !== job.height
      || !timebase || +timebase[1] / +timebase[2] !== 1 / job.fps) {
    throw new Error('Decoded output geometry or frame timing does not match the manifest.');
  }
  const frames = result.stdout.split('\n').filter(line => /^0\s*,/.test(line)).map(line => line.split(',').map(value => value.trim()));
  if (frames.length !== job.frames || frames.some((frame, index) => frame.length !== 6
      || +frame[2] !== index || +frame[3] !== 1 || !/^[0-9a-f]{64}$/.test(frame[5]))) {
    throw new Error(`Saved video has ${frames.length} decoded frames or invalid timestamps; expected ${job.frames}.`);
  }
  return { decoded_frames: frames.length, decoded_unique_count: new Set(frames.map(frame => frame[5])).size,
    codec: 'h264', duration_seconds: frames.length / job.fps, output_bytes: output.size };
}


function object(value, keys, label) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error(`${label} must be an object.`);
  for (const key of Object.keys(value)) if (!keys.includes(key)) throw new Error(`${label}: unsupported field ${key}.`);
}
function number(value, min, max, label) {
  if (!Number.isFinite(value) || value < min || value > max) throw new Error(`${label} must be finite in ${min}..${max}.`);
}
function vector(value, min, max, label) {
  if (!Array.isArray(value) || value.length !== 2) throw new Error(`${label} must be a two-number vector.`);
  value.forEach(v => number(v, min, max, label));
}
function color(value, label) {
  if (typeof value !== 'string' || !/^#(?:[0-9a-f]{3,4}|[0-9a-f]{6}|[0-9a-f]{8})$/i.test(value)) throw new Error(`${label} requires a hex color.`);
}
function id(value) {
  if (typeof value !== 'string' || !/^[a-zA-Z][a-zA-Z0-9_-]{0,63}$/.test(value)) throw new Error('IDs must be 1–64 visible letters, digits, underscore or hyphen.');
}
const COMMON = ['x', 'y', 'position', 'rotation', 'scale', 'opacity'];
const SHAPE = ['fill', 'stroke', 'lineWidth', 'width', 'height', 'clip'];
const PROPS = {group: COMMON, rect: [...COMMON, ...SHAPE, 'radius'], ellipse: [...COMMON, ...SHAPE],
  path: [...COMMON, 'fill', 'stroke', 'lineWidth', 'clip', 'data', 'start', 'end'],
  text: [...COMMON, ...SHAPE, 'text', 'fontSize', 'fontWeight', 'letterSpacing', 'textAlign']};
function property(name, value, gradients, label) {
  if (['fill', 'stroke'].includes(name)) {
    if (value === null) return;
    if (typeof value === 'object') {
      object(value, ['gradient'], label);
      if (!gradients.has(value.gradient)) throw new Error(`${label}: unknown gradient.`);
    } else color(value, label);
  } else if (['position', 'from', 'to'].includes(name)) vector(value, -100000, 100000, label);
  else if (name === 'scale') {
    if (typeof value === 'number') number(value, -100, 100, label); else vector(value, -100, 100, label);
  } else if (name === 'opacity' || name === 'start' || name === 'end') number(value, 0, 1, label);
  else if (name === 'clip') { if (typeof value !== 'boolean') throw new Error(`${label} must be boolean.`); }
  else if (name === 'textAlign') { if (!['left', 'center', 'right'].includes(value)) throw new Error('textAlign must be left, center or right.'); }
  else if (name === 'text') {
    if (typeof value !== 'string' || !value || value.length > 4096 || /[^\x20-\x7e\n\t\u00a0-\u024f\u2000-\u206f\u20ac]/u.test(value)) {
      throw new Error(`${label}: use bounded text covered by bundled Inter Latin glyphs; emoji and unsupported scripts are refused.`);
    }
  } else if (name === 'data') {
    if (typeof value !== 'string' || !value.trim() || value.length > 16384 || !/^[Mm]/.test(value.trim())
        || /[^MmZzLlHhVvCcSsQqTtAaEe0-9+.,\s-]/.test(value) || (value.match(/[MmZzLlHhVvCcSsQqTtAa]/g) || []).length > 2048) {
      throw new Error(`${label}: expected bounded SVG path commands, not SVG markup or an asset URL.`);
    }
    // Canvas silently ignores infinite path coordinates; refuse blank-success fakes.
    const numbers = value.match(/[+-]?(?:\d+\.\d*|\.\d+|\d+)(?:[eE][+-]?\d+)?/g) || [];
    for (const token of numbers) number(Number(token), -100000, 100000, `${label} path coordinate`);
  } else {
    const range = ['width', 'height'].includes(name) ? [1, 4096]
      : name === 'fontSize' ? [6, 512] : name === 'fontWeight' ? [100, 900]
        : ['lineWidth', 'radius', 'fromRadius', 'toRadius'].includes(name) ? [0, 4096] : [-100000, 100000];
    number(value, ...range, label);
  }
}
export function validateScene(scene) {
  object(scene, ['version', 'width', 'height', 'fps', 'frames', 'background', 'gradients', 'nodes', 'tracks'], 'Scene');
  if (scene.version !== 1) throw new Error('Expected version-1 Motion Canvas scene.');
  for (const key of ['width', 'height', 'fps', 'frames']) if (!Number.isInteger(scene[key])) throw new Error(`${key} must be an integer.`);
  if (scene.width < 64 || scene.width > 1920 || scene.height < 64 || scene.height > 1920
      || scene.width % 2 || scene.height % 2 || scene.width * scene.height > 2073600
      || ![24, 30, 60].includes(scene.fps) || scene.frames < 1 || scene.frames > 900 || scene.frames / scene.fps > 30) {
    throw new Error('Scene exceeds the 1080p, 30-second/900-frame or 24/30/60-fps profile.');
  }
  if (scene.background !== undefined && scene.background !== null) color(scene.background, 'background');
  const gradients = new Map();
  if (scene.gradients !== undefined && (!Array.isArray(scene.gradients) || scene.gradients.length > 32)) throw new Error('Use at most 32 gradients.');
  for (const g of scene.gradients || []) {
    object(g, ['id', 'type', 'from', 'to', 'angle', 'fromRadius', 'toRadius', 'stops'], 'Gradient');
    id(g.id);
    if (gradients.has(g.id)) throw new Error('Duplicate gradient ID.');
    if (!['linear', 'radial', 'conic'].includes(g.type)) throw new Error('Gradient type must be linear, radial or conic.');
    for (const field of ['from', 'to', 'angle', 'fromRadius', 'toRadius']) if (g[field] !== undefined) property(field, g[field], gradients, `gradient.${field}`);
    if (!Array.isArray(g.stops) || g.stops.length < 2 || g.stops.length > 16) throw new Error('A gradient requires 2–16 stops.');
    let previous = -1;
    for (const stop of g.stops) {
      object(stop, ['offset', 'color'], 'Gradient stop');
      number(stop.offset, 0, 1, 'stop.offset'); color(stop.color, 'stop.color');
      if (stop.offset < previous) throw new Error('Initial gradient stops must be sorted.');
      previous = stop.offset;
    }
    gradients.set(g.id, g);
  }
  if (!Array.isArray(scene.nodes) || scene.nodes.length < 1 || scene.nodes.length > 256) throw new Error('Use 1–256 vector scene nodes.');
  const nodes = new Map();
  for (const node of scene.nodes) {
    object(node, ['id', 'type', 'parent', 'props'], 'Node'); id(node.id);
    if (nodes.has(node.id) || !PROPS[node.type]) throw new Error('Duplicate node ID or unsupported node type.');
    if (node.parent && !nodes.has(node.parent)) throw new Error('Parents must precede their children.');
    object(node.props, PROPS[node.type], `${node.id}.props`);
    for (const [key, value] of Object.entries(node.props)) property(key, value, gradients, `${node.id}.${key}`);
    if (node.type === 'path' && !node.props.data) throw new Error('Path nodes require SVG path data.');
    if (node.type === 'text' && !node.props.text) throw new Error('Text nodes require text.');
    if (node.props.x !== undefined && node.props.position || node.props.y !== undefined && node.props.position) throw new Error('Use position or x/y without ambiguous overrides.');
    nodes.set(node.id, node);
  }
  if (scene.tracks !== undefined && (!Array.isArray(scene.tracks) || scene.tracks.length > 512)) throw new Error('Use at most 512 choreography tracks.');
  const occupied = new Map();
  for (const track of scene.tracks || []) {
    object(track, ['target', 'property', 'start', 'end', 'to', 'easing'], 'Track');
    if (!Number.isInteger(track.start) || !Number.isInteger(track.end) || track.start < 0 || track.end <= track.start || track.end > scene.frames) throw new Error('Track bounds are frame integers, 0 <= start < end <= frames.');
    if (track.easing !== undefined && !['linear', 'easeInOutCubic', 'easeOutCubic', 'easeOutBack'].includes(track.easing)) throw new Error('Unsupported easing.');
    if (typeof track.target !== 'string' || typeof track.property !== 'string') throw new Error('Track target and property must be strings.');
    if (track.target.startsWith('gradient:')) {
      const g = gradients.get(track.target.slice(9));
      if (!g) throw new Error('Unknown gradient track target.');
      const stop = track.property.match(/^stop\.(\d+)\.(color|offset)$/);
      if (stop) {
        if (+stop[1] >= g.stops.length) throw new Error('Unknown gradient stop.');
        if (stop[2] === 'color') color(track.to, 'Track color'); else number(track.to, 0, 1, 'Track offset');
      } else {
        if (!['from', 'to', 'angle', 'fromRadius', 'toRadius'].includes(track.property)) throw new Error('Unsupported gradient track.');
        property(track.property, track.to, gradients, 'Gradient track');
      }
    } else {
      const node = nodes.get(track.target);
      if (!node || !PROPS[node.type].includes(track.property) || ['clip', 'fontWeight', 'textAlign'].includes(track.property)) throw new Error('Unknown node or unsupported animated property.');
      property(track.property, track.to, gradients, 'Track value');
      if (['fill', 'stroke'].includes(track.property) && (typeof track.to !== 'string' || typeof node.props[track.property] !== 'string')) throw new Error('Color tracks need explicit initial hex colors; animate gradient stops separately.');
    }
    const channels = track.property === 'position' ? ['x', 'y'] : [track.property];
    for (const property of channels) {
      const key = `${track.target}:${property}`; const ranges = occupied.get(key) || [];
      if (ranges.some(([start, end]) => track.start < end && track.end > start)) throw new Error(`Overlapping tracks for ${key}; use explicit sequential or disjoint choreography.`);
      ranges.push([track.start, track.end]); occupied.set(key, ranges);
    }
  }
  return scene;
}

async function regular(pathname, limit) {
  const stat = await fs.lstat(pathname);
  if (!stat.isFile() || stat.isSymbolicLink() || stat.size > limit) throw new Error('Expected bounded regular private job/runtime file.');
  const bytes = await fs.readFile(pathname);
  if (bytes.length > limit) throw new Error('Private input grew beyond its limit.');
  return bytes;
}
async function loadJob(filename) {
  if (filename !== 'job.json') throw new Error('Expected fixed private job.json.');
  const job = JSON.parse((await regular(filename, 2 * 1024 * 1024 + 65536)).toString('utf8'));
  object(job, ['operation', 'scene', 'frame', 'output'], 'Job');
  if (!['inspect', 'frame', 'verify', 'render'].includes(job.operation)) throw new Error('Unsupported operation.');
  validateScene(job.scene); Object.assign(job, {width: job.scene.width, height: job.scene.height, fps: job.scene.fps, frames: job.scene.frames});
  if (job.operation === 'frame' && (!Number.isInteger(job.frame) || job.frame < 0 || job.frame >= job.frames || job.output !== 'output.png')) throw new Error('Frame or private PNG output is invalid.');
  if (job.operation === 'render' && job.output !== 'output.mp4') throw new Error('Private MP4 output is invalid.');
  if (!['frame', 'render'].includes(job.operation) && job.output !== undefined) throw new Error('Read operations do not write an output.');
  return job;
}

async function bundle(root, scene) {
  for (const name of ['core', '2d']) {
    const metadata = JSON.parse((await regular(path.join(root, `node_modules/@motion-canvas/${name}/package.json`), 65536)).toString('utf8'));
    if (metadata.version !== '3.17.2') throw new Error('Motion Canvas source version differs from the reviewed pin.');
  }
  let browserSource = (await regular(path.join(root, 'browser.mjs'), 65536)).toString('utf8');
  for (const name of ['core', '2d']) browserSource = browserSource.replaceAll(`'@motion-canvas/${name}'`, JSON.stringify(path.join(root, `node_modules/@motion-canvas/${name}/lib/index.js`)));
  browserSource = browserSource.replaceAll("'@motion-canvas/core/lib/scenes/timeEvents'", JSON.stringify(path.join(root, 'node_modules/@motion-canvas/core/lib/scenes/timeEvents/index.js')));
  const compiler = childProcess(path.join(root, `node_modules/@esbuild/${process.platform === 'linux' ? 'linux' : 'darwin'}-${process.arch}/bin/esbuild`), [
    '--bundle', '--platform=browser', '--format=iife', '--target=chrome122', '--legal-comments=inline',
    '--log-level=error', '--define:import.meta.hot=undefined', '--tsconfig-raw={}', '--loader=js',
    '--sourcefile=mirket-fixed-scene.mjs',
  ], {input: true, limit: 8 * 1024 * 1024});
  compiler.child.stdin.end(browserSource + `\ninitialise(${JSON.stringify(scene)}).catch(error=>{window.__mirketError=error.message;});`);
  const compiled = await bounded(compiler.done, 15000, 'Fixed Motion Canvas compilation');
  const code = Buffer.from(compiled.stdout);
  if (!code.length || code.length > 8 * 1024 * 1024) throw new Error('Compiled runtime exceeds its limits.');
  return code;
}

const CSP = "default-src 'none'; script-src 'self'; style-src 'unsafe-inline'; font-src 'self'; img-src 'self' data:; connect-src 'none'; worker-src 'none'; frame-src 'none'; media-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'";
async function compositionPage(job, root) {
  const code = await bundle(root, job.scene);
  const files = new Map([
    ['/', {media_type: 'text/html', data: Buffer.from('<!doctype html><meta charset="utf-8"><style>html,body{margin:0;overflow:hidden}canvas{display:block}</style><script src="/scene.js" defer></script>')}],
    ['/scene.js', {media_type: 'text/javascript', data: code}],
    ['/font.woff2', {media_type: 'font/woff2', data: await regular(path.join(root, 'font.woff2'), 1024 * 1024)}],
  ]);
  const context = await browser.newContext({viewport: {width: job.width, height: job.height}, deviceScaleFactor: 1, acceptDownloads: false, serviceWorkers: 'block', locale: 'en-US', timezoneId: 'UTC', colorScheme: 'light'});
  let navigated = false;
  await context.route('**/*', async route => {
    const request = route.request(); const url = new URL(request.url()); const file = files.get(url.pathname);
    if (request.method() !== 'GET' || url.origin !== ORIGIN || url.search || !file || (request.isNavigationRequest() && (navigated || url.pathname !== '/'))) {
      problem('blocked resource', request.url()); await route.abort('blockedbyclient'); return;
    }
    if (request.isNavigationRequest()) navigated = true;
    await route.fulfill({status: 200, body: file.data, contentType: file.media_type, headers: {'Content-Security-Policy': CSP, 'Cache-Control': 'no-store'}});
  });
  await context.routeWebSocket('**/*', socket => {problem('blocked websocket', socket.url()); socket.close();});
  const page = await context.newPage();
  page.on('pageerror', error => problem('page error', error.message));
  page.on('console', message => {if (['warning', 'error'].includes(message.type())) problem(`page ${message.type()}`, message.text());});
  page.on('download', download => {problem('download', 'Unexpected download.'); void download.cancel();});
  page.on('dialog', dialog => {problem('dialog', 'Unexpected dialog.'); void dialog.dismiss();});
  page.on('crash', () => problem('page', 'Chromium renderer crashed.'));
  await page.goto(ORIGIN, {waitUntil: 'load', timeout: 10000});
  await page.waitForFunction(() => window.__mirketMotionCanvas || window.__mirketError, {timeout: 15000});
  const error = await page.evaluate(() => window.__mirketError);
  if (error) throw new Error(error);
  assertClean(); return page;
}
function validatePng(buffer, job) {
  if (buffer.length < 24 || buffer.length > MAX_OUTPUT || !buffer.subarray(0, 8).equals(Buffer.from([137,80,78,71,13,10,26,10])) || buffer.readUInt32BE(16) !== job.width || buffer.readUInt32BE(20) !== job.height) throw new Error('Motion Canvas did not produce a bounded PNG with the requested dimensions.');
}
async function capture(page, job, frame) {
  const url = await bounded(page.evaluate(frame => window.__mirketMotionCanvas.capture(frame), frame), FRAME_TIMEOUT, 'Motion Canvas frame');
  if (typeof url !== 'string' || !url.startsWith('data:image/png;base64,')) throw new Error('Motion Canvas stage returned no PNG.');
  const buffer = Buffer.from(url.slice(22), 'base64'); validatePng(buffer, job); assertClean(); return buffer;
}
async function verify(page, job) {
  const frames = [...new Set([0, Math.floor(job.frames / 4), Math.floor(job.frames / 2), Math.floor(job.frames * 3 / 4), job.frames - 1])].sort((a,b) => a-b);
  const hashes = new Map();
  for (const frame of frames) hashes.set(frame, createHash('sha256').update(await capture(page, job, frame)).digest('hex'));
  for (const frame of [...frames].reverse()) if (createHash('sha256').update(await capture(page, job, frame)).digest('hex') !== hashes.get(frame)) throw new Error(`Motion Canvas forward/reverse seek differs at frame ${frame}.`);
  return {deterministic: true, samples: frames.map(frame => ({frame, sha256: hashes.get(frame)})), unique_count: new Set(hashes.values()).size};
}
async function main() {
  const root = process.env.MIRKET_MOTION_CANVAS_ROOT; const chromiumPath = process.env.MIRKET_CHROMIUM; const ffmpeg = process.env.MIRKET_FFMPEG;
  if (![root, chromiumPath, ffmpeg].every(value => typeof value === 'string' && path.isAbsolute(value))) throw new Error('Mirket must provide absolute verified runtime, Chromium and FFmpeg paths.');
  if (process.argv.length !== 3) throw new Error('Expected --probe or fixed private job.json.');
  const probe = process.argv[2] === '--probe';
  const specimen = {version: 1, width: 128, height: 128, fps: 30, frames: 3, nodes: [{id: 'probe', type: 'path', props: {data: 'M-40 -30L40 -30L0 40Z', fill: '#36e8c1'}}], tracks: [{target: 'probe', property: 'rotation', start: 0, end: 3, to: 45}]};
  const job = probe ? {operation: 'verify', scene: specimen, width: 128, height: 128, fps: 30, frames: 3} : await loadJob(process.argv[2]);
  const {chromium} = await import(pathToFileURL(path.join(root, 'node_modules/playwright-core/index.mjs')).href);
  const version = await bounded(childProcess(ffmpeg, ['-hide_banner', '-version']).done, 5000, 'FFmpeg version');
  if (process.env.MIRKET_CONTAINER !== '1') throw new Error('Motion Canvas requires Mirket’s verified Podman runtime.');
  const expectedVersion = '9.0.2';
  const versionPattern = /^ffmpeg version 9\.0\.2(?:\s|-)/;
  if (!versionPattern.test(version.stdout)) throw new Error(`Expected pinned shared FFmpeg ${expectedVersion}.`);
  const profile = await launchBrowser(chromium, chromiumPath);
  try {
    if (job.output) {
      try {await fs.lstat(job.output); throw new Error('Private output already exists.');} catch (error) {if (error.code !== 'ENOENT') throw error;}
    }
    const page = await compositionPage(job, root); await capture(page, job, 0);
    const inspection = await page.evaluate(() => window.__mirketMotionCanvas.inspect());
    const result = {engine: 'motion-canvas', version: '3.17.2', source_commit: 'e735a995aa738ca682feb4204a39ee7db7b5d279', operation: job.operation,
      width: job.width, height: job.height, fps: job.fps, frames: job.frames, inspection};
    if (job.operation === 'frame') {
      const png = await capture(page, job, job.frame); await fs.writeFile(job.output, png, {flag: 'wx'});
      Object.assign(result, {frame: job.frame, time_seconds: job.frame/job.fps, output_bytes: png.length, sha256: createHash('sha256').update(png).digest('hex')});
    } else if (['verify', 'render'].includes(job.operation)) {
      result.verification = await verify(page, job);
      if (job.operation === 'render') {
        Object.assign(result, await render(page, job, ffmpeg));
        if (result.verification.unique_count > 1 && result.decoded_unique_count === 1) throw new Error('Saved video is frozen although source samples change.');
      }
    }
    if (probe) Object.assign(result, {browser_version: browser.version(), ffmpeg_version: expectedVersion});
    assertClean(); return result;
  } finally {stopping = true; await cleanup(); await fs.rm(profile, {recursive: true, force: true});}
}
const watchdog = setTimeout(() => {stopping = true; diagnostic('Motion Canvas exceeded its 85-second execution budget.'); void cleanup().finally(() => process.exit(1));}, 85000);
try {process.stdout.write(`${JSON.stringify(await main())}\n`);} catch (error) {diagnostic(error?.message || error); stopping = true; await cleanup(); process.exitCode = 1;} finally {clearTimeout(watchdog);}
