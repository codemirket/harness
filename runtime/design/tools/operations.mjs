import fs from 'node:fs/promises';
import {constants as fsConstants} from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {optimize} from 'svgo';
import {SaxesParser} from 'saxes';
import {compileTokens} from './tokens.mjs';
import {workspace, workspacePath, readFile, readJson, publish, run, temporaryDirectory, regularOutput, sha256, assertRelative, MAX_OUTPUT} from './paths.mjs';
import {checkRequest, requestSignal, combinedSignal, redactSecrets} from './request.mjs';

const toolRoot = path.dirname(fileURLToPath(import.meta.url));
const hf = path.join(toolRoot, 'node_modules/hyperframes/bin/hyperframes.mjs');
const odiff = path.join(toolRoot, 'node_modules/.bin/odiff');
const sha = bytes => sha256(bytes);
const MAX_PROJECT_BYTES = 64 * 1024 * 1024;
const MAX_PROJECT_FILES = 128;
const MAX_PROJECT_ENTRIES = 128;
const MAX_PROJECT_DEPTH = 32;
const SCREENSHOT_MAX_PIXELS = 16 * 1024 * 1024;
const SCREENSHOT_SCALE = 2;
export const browserInspectCaptureOptions = Object.freeze({width: 800, height: 600, full_page: true});
const tokenSource = args => args.tokens || process.env.MIRKET_DESIGN_TOKENS || 'design/tokens.json';
const preflight = async args => compileTokens(tokenSource(args));
const outputOptions = args => ({overwrite: args.overwrite === true, expected_sha256: args.expected_sha256});
// Reject known output conflicts before starting a remote or CLI job. publish()
// still performs the authoritative check at the actual publication boundary.
async function assertOutputReady(args) {
  await workspacePath(args.output, {missing: true});
  let before;
  try { before = await readFile(args.output, MAX_OUTPUT); }
  catch (error) { if (!String(error.message).startsWith('Missing workspace path:')) throw error; }
  if (before !== undefined && args.overwrite !== true) throw Error(`Output exists; pass overwrite=true: ${args.output}`);
  if (before !== undefined && args.expected_sha256 !== sha(before)) throw Error(`Output revision mismatch: ${args.output}`);
  if (before === undefined && args.expected_sha256) throw Error(`Output does not exist for expected_sha256: ${args.output}`);
}
function flatTokenValues(tree, prefix = '', result = {}) {
  for (const [key, value] of Object.entries(tree)) {
    const name = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === 'object' && '$value' in value) result[name] = value.$value;
    else if (value && typeof value === 'object') flatTokenValues(value, name, result);
  }
  return result;
}

function pngDimensions(bytes) {
  if (bytes.length < 24 || !bytes.subarray(0, 8).equals(Buffer.from([137,80,78,71,13,10,26,10]))) throw Error('Expected a PNG response');
  const width = bytes.readUInt32BE(16), height = bytes.readUInt32BE(20);
  if (!width || !height || width * height > SCREENSHOT_MAX_PIXELS) throw Error('PNG exceeds 16 Mi decoded pixel budget');
  return [width, height];
}
export function screenshotPixelPlan({width, height, fullPage, cssContentSize}) {
  let clipWidth = width, clipHeight = height;
  if (fullPage) {
    if (!cssContentSize || !Number.isFinite(cssContentSize.width) || !Number.isFinite(cssContentSize.height) || cssContentSize.width <= 0 || cssContentSize.height <= 0) throw Error('Browserless content metrics are invalid');
    clipWidth = Math.max(width, Math.ceil(cssContentSize.width));
    clipHeight = Math.max(height, Math.ceil(cssContentSize.height));
  }
  const pixelWidth = clipWidth * SCREENSHOT_SCALE, pixelHeight = clipHeight * SCREENSHOT_SCALE;
  if (!Number.isSafeInteger(pixelWidth) || !Number.isSafeInteger(pixelHeight) || pixelWidth < 1 || pixelHeight < 1 || pixelWidth * pixelHeight > SCREENSHOT_MAX_PIXELS) throw Error('Browserless screenshot exceeds 16 Mi pixel budget');
  return {pixelWidth, pixelHeight, clipWidth, clipHeight};
}
export function verifyScreenshotPixels(width, height, plan) {
  if (!Number.isSafeInteger(width) || !Number.isSafeInteger(height) || width < 1 || height < 1) throw Error('Browserless screenshot dimensions are invalid');
  if (width * height > SCREENSHOT_MAX_PIXELS) throw Error('Browserless screenshot exceeds 16 Mi pixel budget');
  if (width !== plan.pixelWidth || height !== plan.pixelHeight) throw Error('Browserless screenshot dimensions differ from the bounded capture');
}
export function fullPageCaptureParams(plan) {
  return {format: 'png', fromSurface: true, captureBeyondViewport: true,
    clip: {x: 0, y: 0, width: plan.clipWidth, height: plan.clipHeight, scale: SCREENSHOT_SCALE}};
}
function svgBytes(source) {
  // Graphviz emits the standard SVG 1.1 external DTD declaration. Remove
  // exactly that header without resolving it; reject every other DTD or entity.
  source = source.replace(/^(\uFEFF?\s*(?:<\?xml\b[^>]*\?>\s*)?)<!DOCTYPE\s+svg\s+PUBLIC\s+"-\/\/W3C\/\/DTD SVG 1\.1\/\/EN"\s+"http:\/\/www\.w3\.org\/Graphics\/SVG\/1\.1\/DTD\/svg11\.dtd"\s*>/, '$1');
  if (/<!\s*(?:DOCTYPE|ENTITY)/i.test(source)) throw Error('SVG DTD and entity declarations are refused');
  const parser = new SaxesParser({xmlns: true});
  let styleDepth = 0;
  const safeResource = value => /^#[^\s]+$/.test(value) || /^data:(?:font\/(?:woff2?|ttf|otf)|image\/(?:png|jpeg|gif|webp|avif));base64,[a-zA-Z0-9+/=]+$/.test(value);
  const inspectCss = value => {
    if (/@import|\\/.test(value)) throw Error('SVG external or escaped CSS resources are refused');
    for (const match of value.matchAll(/url\(\s*(['"]?)(.*?)\1\s*\)/gi)) {
      if (!safeResource(match[2].trim())) throw Error('SVG external style resource is refused');
    }
  };
  parser.on('opentag', tag => {
    if (['script', 'foreignObject', 'animate', 'animateMotion', 'animateTransform', 'set'].includes(tag.local)) throw Error(`Unsafe SVG element: ${tag.local}`);
    if (tag.local === 'style') styleDepth++;
    for (const attr of Object.values(tag.attributes)) {
      if (attr.local.toLowerCase().startsWith('on')) throw Error('SVG event handlers are refused');
      if (['href', 'src'].includes(attr.local.toLowerCase()) && !safeResource(attr.value)) throw Error('SVG external resources are refused');
      if (/(?:url\s*\(|@import)/i.test(attr.value)) inspectCss(attr.value);
    }
  });
  parser.on('closetag', tag => { if (tag.local === 'style') styleDepth--; });
  parser.on('text', value => { if (styleDepth) inspectCss(value); });
  parser.write(source).close();
  const result = optimize(source, {multipass: true, plugins: ['preset-default', 'removeDimensions']});
  if (result.error || !result.data?.includes('<svg')) throw Error(`SVGO rejected SVG: ${result.error || 'missing root'}`);
  return Buffer.from(result.data);
}
export {svgBytes as optimizeSvgBytes};
async function fetchBytes(url, {method = 'GET', body, headers = {}, max = MAX_OUTPUT, timeoutMs = 30000} = {}) {
  checkRequest();
  const response = await fetch(url, {method, body, headers, signal: combinedSignal(timeoutMs)});
  const length = Number(response.headers.get('content-length'));
  if (length > max) { await response.body?.cancel(); throw Error('Service response exceeds output bound'); }
  const chunks = []; let size = 0;
  for await (const chunk of response.body) {
    size += chunk.length;
    if (size > max) throw Error('Service response exceeds output bound');
    chunks.push(chunk);
  }
  const data = Buffer.concat(chunks, size);
  if (!response.ok) throw Error(`Service ${new URL(url).host} returned HTTP ${response.status}: ${data.toString('utf8').slice(0, 1000)}`);
  if (!data.length || data.length > max) throw Error('Service response is empty or exceeds output bound');
  return {bytes: data, content_type: response.headers.get('content-type') || ''};
}
function signImgproxy(pathAfterSignature) {
  const key = process.env.MIRKET_IMGPROXY_KEY, salt = process.env.MIRKET_IMGPROXY_SALT;
  if (!key || !salt || !/^(?:[0-9a-f]{2})+$/i.test(key) || !/^(?:[0-9a-f]{2})+$/i.test(salt)) throw Error('imgproxy signing key and salt are required');
  return crypto.createHmac('sha256', Buffer.from(key, 'hex')).update(Buffer.from(salt, 'hex')).update(pathAfterSignature).digest('base64url');
}
export async function imgproxyBytes(args, max = MAX_OUTPUT) {
  assertRelative(args.source);
  await workspacePath(args.source);
  const width = Number(args.width), height = Number(args.height);
  if (!Number.isInteger(width) || !Number.isInteger(height) || width < 1 || height < 1 || width > 4096 || height > 4096) throw Error('imgproxy dimensions must be integers in 1..4096');
  const mode = args.mode || 'fit';
  if (!['fit', 'fill', 'force'].includes(mode)) throw Error('Unsupported imgproxy resize mode');
  const format = args.format || 'png';
  if (!['png', 'jpeg', 'webp', 'avif'].includes(format)) throw Error('Unsupported imgproxy output format');
  const encoded = args.source.split('/').map(encodeURIComponent).join('/');
  const suffix = format === 'jpeg' ? 'jpg' : format;
  const gravity = args.gravity || 'sm';
  if (!['sm', 'ce', 'no', 'ea', 'so', 'we'].includes(gravity)) throw Error('Unsupported imgproxy gravity');
  const quality = args.quality == null ? 82 : Number(args.quality);
  if (!Number.isInteger(quality) || quality < 1 || quality > 100) throw Error('imgproxy quality must be in 1..100');
  const remainder = `/rs:${mode}:${width}:${height}/g:${gravity}/q:${quality}/plain/local:///${encoded}@${suffix}`;
  const signature = signImgproxy(remainder);
  const result = await fetchBytes(`http://imgproxy:8080/${signature}${remainder}`, {max});
  if (format === 'png') {
    const [w, h] = pngDimensions(result.bytes);
    if (w > width || h > height || w < 1 || h < 1) throw Error('imgproxy output dimensions are outside requested bounds');
  }
  return result.bytes;
}
export async function imgproxyTransform(args) {
  await assertOutputReady(args);
  const tokens = await preflight(args);
  const bytes = await imgproxyBytes(args);
  const saved = await publish(args.output, bytes, outputOptions(args));
  return {tokens, artifact: saved, service: 'imgproxy'};
}

const allowedTags = new Set(['div', 'span', 'p', 'h1', 'h2', 'h3', 'img', 'strong', 'em']);
export async function prepareSatoriTree(tree, tokenValues, imageLoader = imgproxyBytes) {
  let nodes = 0, images = 0, pixels = 0, retained = 0;
  const cache = new Map();
  function validate(node, depth = 0) {
    if (++nodes > 512 || depth > 32) throw Error('Satori tree exceeds node or depth budget');
    if (node?.type === 'img') {
      const width = node.props?.width || 1024, height = node.props?.height || 1024;
      if (!Number.isInteger(width) || !Number.isInteger(height) || width < 1 || height < 1 || width > 4096 || height > 4096) throw Error('Satori image dimensions are invalid');
      pixels += width * height;
      if (++images > 16 || pixels > 16 * 1024 * 1024) throw Error('Satori images exceed aggregate count or pixel budget');
    }
    if (node && typeof node === 'object') for (const child of node.children == null ? [] : Array.isArray(node.children) ? node.children : [node.children]) validate(child, depth + 1);
  }
  validate(tree);
  async function image(args) {
    checkRequest();
    const key = JSON.stringify(args);
    if (!cache.has(key)) {
      const remaining = 12 * 1024 * 1024 - retained;
      if (remaining < 1) throw Error('Satori images exceed aggregate byte budget');
      const bytes = await imageLoader(args, remaining);
      retained += bytes.length;
      if (retained > 12 * 1024 * 1024) throw Error('Satori images exceed aggregate byte budget');
      cache.set(key, `data:image/png;base64,${bytes.toString('base64')}`);
    }
    return cache.get(key);
  }
  return satoriNode(tree, tokenValues, image);
}
async function satoriNode(node, tokenValues, image, depth = 0) {
  if (depth > 32) throw Error('Satori tree is too deep');
  if (typeof node === 'string' || typeof node === 'number') return node;
  if (!node || typeof node !== 'object' || Array.isArray(node) || !allowedTags.has(node.type)) throw Error('Satori node type is unsupported');
  const props = {...(node.props || {})};
  for (const key of Object.keys(props)) {
    if (key.startsWith('on') || ['dangerouslySetInnerHTML', 'script', 'srcDoc'].includes(key)) throw Error('Executable Satori props are refused');
  }
  if (props.style && (typeof props.style !== 'object' || (props.style.display && props.style.display !== 'flex') || Object.keys(props.style).some(key => /animation|transition|grid/i.test(key)))) throw Error('Static Satori Flexbox styles only');
  props.style = {display: 'flex', ...(props.style || {})};
  for (const [key, value] of Object.entries(props.style)) {
    if (value && typeof value === 'object' && !Array.isArray(value) && typeof value.$token === 'string') {
      if (!(value.$token in tokenValues)) throw Error(`Unknown design token: ${value.$token}`);
      props.style[key] = tokenValues[value.$token];
    }
  }
  if (node.type === 'img') {
    if (typeof props.src !== 'string') throw Error('Satori image source must be a workspace path');
    props.src = await image({source: props.src, width: props.width || 1024, height: props.height || 1024, mode: 'fit', format: 'png'});
  }
  const children = node.children == null ? [] : Array.isArray(node.children) ? node.children : [node.children];
  const prepared = [];
  for (const child of children) prepared.push(await satoriNode(child, tokenValues, image, depth + 1));
  return {type: node.type, props: {...props, children: prepared}};
}
export async function staticRender(args) {
  await assertOutputReady(args);
  const tokens = await preflight(args);
  const tokenValues = flatTokenValues(await readJson(tokens.outputs['penpot.tokens.json'], 512 * 1024));
  const width = Number(args.width), height = Number(args.height);
  if (!Number.isInteger(width) || !Number.isInteger(height) || width < 16 || height < 16 || width > 4096 || height > 4096) throw Error('Satori dimensions must be in 16..4096');
  if (!Array.isArray(args.fonts) || args.fonts.length < 1 || args.fonts.length > 8) throw Error('Satori requires 1..8 font files');
  const fonts = [];
  for (const font of args.fonts) {
    if (!/\.(ttf|otf|woff)$/i.test(font.path)) throw Error('Satori fonts must be TTF, OTF, or WOFF');
    const data = await readFile(font.path, 8 * 1024 * 1024);
    const signature = data.subarray(0, 4).toString('latin1');
    const expected = path.extname(font.path).toLowerCase() === '.ttf' ? '\0\x01\0\0' : path.extname(font.path).toLowerCase() === '.otf' ? 'OTTO' : 'wOFF';
    if (signature !== expected) throw Error('Satori font bytes do not match TTF, OTF or WOFF extension');
    fonts.push({name: font.name || 'Design', data, weight: font.weight || 400, style: font.style || 'normal'});
  }
  const tree = await prepareSatoriTree(args.tree, tokenValues);
  const [{default: satori}, {Resvg}] = await Promise.all([import('satori'), import('@resvg/resvg-js')]);
  const svg = await satori(tree, {width, height, fonts});
  const optimized = svgBytes(svg);
  const format = args.format || 'png';
  const bytes = format === 'svg' ? optimized : format === 'png' ? new Resvg(optimized.toString('utf8')).render().asPng() : null;
  if (!bytes) throw Error('Satori output must be SVG or PNG');
  if (format === 'png' && pngDimensions(bytes).join('x') !== `${width}x${height}`) throw Error('Satori PNG dimensions differ');
  const artifact = await publish(args.output, bytes, outputOptions(args));
  const qa = await inspectSaved(args.output, args.baseline);
  return {tokens, artifact, qa, engine: 'satori+resvg-js', svg_optimized: true};
}
export async function svgOptimize(args) {
  await assertOutputReady(args);
  const tokens = await preflight(args);
  const source = await readFile(args.source, 8 * 1024 * 1024);
  const bytes = svgBytes(source.toString('utf8'));
  const artifact = await publish(args.output, bytes, outputOptions(args));
  return {tokens, artifact, engine: 'svgo'};
}
export async function krokiRender(args) {
  await assertOutputReady(args);
  const tokens = await preflight(args);
  const type = args.diagram_type;
  if (!['plantuml', 'c4plantuml', 'vegalite', 'excalidraw', 'd2', 'graphviz'].includes(type)) throw Error('Unsupported Kroki diagram type');
  const format = args.format || 'svg';
  if (!['svg', 'png'].includes(format)) throw Error('Unsupported Kroki output format');
  const source = await readFile(args.source, 256 * 1024);
  const response = await fetchBytes(`http://kroki:8000/${type}/${format}`, {method: 'POST', body: source, headers: {'Content-Type': 'text/plain'}, max: 16 * 1024 * 1024, timeoutMs: 60000});
  const bytes = format === 'svg' ? svgBytes(response.bytes.toString('utf8')) : response.bytes;
  if (format === 'png') pngDimensions(bytes);
  const artifact = await publish(args.output, bytes, outputOptions(args));
  const qa = await inspectSaved(args.output, args.baseline);
  return {tokens, artifact, qa, service: 'kroki', svg_optimized: format === 'svg'};
}

export async function odiffCompare(args) {
  // Decode only stable, bounded PNG screenshots; input paths can change while a CLI runs.
  const leftBytes = await readFile(args.baseline), rightBytes = await readFile(args.candidate);
  pngDimensions(leftBytes); pngDimensions(rightBytes);
  const temp = await temporaryDirectory('odiff');
  try {
    const left = path.join(temp, 'left.png'), right = path.join(temp, 'right.png');
    await fs.writeFile(left, leftBytes); await fs.writeFile(right, rightBytes);
    const diff = path.join(temp, 'diff.png');
    const result = await run(odiff, [left, right, diff, '--parsable-stdout', '--fail-on-layout'], {allowedCodes: [0, 21, 22], timeoutMs: 60000});
    let artifact, bytes;
    try { bytes = await regularOutput(diff); }
    catch (error) { if (error.code !== 'ENOENT') throw error; }
    // Equal pixels and dimension mismatches may have no diagnostic image.
    if (bytes) {
      if (!args.output) await fs.mkdir(path.join(workspace, '.mirket-design/qa'), {recursive: true});
      artifact = await publish(args.output || `.mirket-design/qa/${crypto.randomUUID()}-diff.png`, bytes, outputOptions(args));
    }
    return {match: result.code === 0, exit_code: result.code, stdout: result.stdout.slice(0, 4096), diff: artifact || null, baseline: args.baseline, candidate: args.candidate};
  } finally { await fs.rm(temp, {recursive: true, force: true}); }
}

export function injectTokenStylesheet(html, stylesheetUrl) {
  const link = `<link rel="stylesheet" data-mirket-tokens href="${stylesheetUrl}">`;
  const head = /<head\b[^>]*>/i.exec(html);
  const root = /<html\b[^>]*>/i.exec(html);
  const script = /<script\b/i.exec(html);
  const insertion = head ? head.index : root ? root.index : 0;
  if (script && script.index < insertion) throw Error('HTML executes a script before the canonical token stylesheet can load');
  if (head) return html.replace(/<head\b[^>]*>/i, match => `${match}${link}`);
  if (root) return html.replace(/<html\b[^>]*>/i, match => `${match}<head>${link}</head>`);
  if (/^\s*<!doctype\b[^>]*>/i.test(html)) return html.replace(/^(\s*<!doctype\b[^>]*>)/i, match => `${match}<head>${link}</head>`);
  return `<head>${link}</head>${html}`;
}
async function tokenDocument(url, stylesheetUrl) {
  const target = new URL(url);
  if (target.host !== 'design-tools:9002' || !target.pathname.startsWith('/files/') || target.search || target.hash) return null;
  const relative = target.pathname.slice('/files/'.length).split('/').map(decodeURIComponent).join('/');
  if (path.extname(relative).toLowerCase() !== '.html') return null;
  const html = (await readFile(relative, 4 * 1024 * 1024)).toString('utf8');
  return injectTokenStylesheet(html, stylesheetUrl);
}
export async function withBrowserCaptureSession(connect, endpoint, connectOptions, operation, {signal = requestSignal(), timeoutMs = 120000} = {}) {
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs < 1) throw Error('Browserless capture deadline is invalid');
  const deadline = new AbortController();
  const timer = setTimeout(() => deadline.abort(Error('Browserless capture operation timed out')), timeoutMs);
  const operationSignal = signal ? AbortSignal.any([signal, deadline.signal]) : deadline.signal;
  let browser, closing;
  const closeBrowser = () => {
    if (!browser) return Promise.resolve();
    closing ||= Promise.resolve().then(() => browser.close()).catch(() => {});
    return closing;
  };
  try {
    if (operationSignal.aborted) throw operationSignal.reason || Error('Browserless capture operation cancelled');
    browser = await connect(endpoint, connectOptions);
    let rejectAbort;
    const aborted = new Promise((_, reject) => { rejectAbort = reject; });
    const abortReason = () => operationSignal.reason instanceof Error ? operationSignal.reason : Error('Browserless capture operation cancelled');
    const onAbort = () => {
      void closeBrowser();
      rejectAbort(abortReason());
    };
    operationSignal.addEventListener('abort', onAbort, {once: true});
    try {
      // connect() has its own timeout but no AbortSignal. Cancellation may have
      // arrived while it was pending, so refuse to create a context immediately.
      if (operationSignal.aborted) { onAbort(); return await aborted; }
      return await Promise.race([Promise.resolve().then(() => {
        if (operationSignal.aborted) throw abortReason();
        return operation(browser);
      }), aborted]);
    } finally {
      operationSignal.removeEventListener('abort', onAbort);
    }
  } finally {
    clearTimeout(timer);
    let closeTimer;
    await Promise.race([
      closeBrowser(),
      new Promise(resolve => { closeTimer = setTimeout(resolve, 1000); }),
    ]).finally(() => clearTimeout(closeTimer));
  }
}

async function browserCapture(url, {width = 1280, height = 720, wait_for, full_page = false, inspect_selectors = [], tokensCss} = {}) {
  if (!Number.isInteger(width) || !Number.isInteger(height) || width < 64 || height < 64 || width > 1920 || height > 1920) throw Error('Browser viewport out of range');
  const token = process.env.MIRKET_BROWSERLESS_TOKEN;
  if (!token) throw Error('Browserless token is required');
  // The native Playwright endpoint keeps context viewport and DPR authoritative.
  // Browserless's separate Puppeteer initializer on its CDP route can otherwise
  // replace them with 800x600/DPR1 before a document's first script runs.
  const endpoint = new URL('ws://browserless:3000/chromium/playwright');
  endpoint.searchParams.set('token', token);
  const {chromium} = await import('playwright-core');
  return withBrowserCaptureSession(chromium.connect.bind(chromium), endpoint.toString(), {timeout: 15000}, async browser => {
    const context = await browser.newContext({viewport: {width, height}, deviceScaleFactor: 2, javaScriptEnabled: true});
    try {
      const page = await context.newPage();
      const consoleErrors = [];
      page.on('pageerror', error => { if (consoleErrors.length < 20) consoleErrors.push(String(error).slice(0, 500)); });
      page.on('console', message => { if (message.type() === 'error' && consoleErrors.length < 20) consoleErrors.push(message.text().slice(0, 500)); });
      if (tokensCss) {
        const stylesheetUrl = workspaceUrl(tokensCss);
        await page.route('http://design-tools:9002/files/**', async route => {
          if (route.request().resourceType() !== 'document') { await route.continue(); return; }
          const html = await tokenDocument(route.request().url(), stylesheetUrl);
          if (html == null) { await route.continue(); return; }
          await route.fulfill({status: 200, contentType: 'text/html; charset=utf-8', body: html,
            headers: {'Cache-Control': 'no-store', 'X-Content-Type-Options': 'nosniff', 'Content-Security-Policy': "default-src 'self' data: blob: 'unsafe-inline'"}});
        });
      }
      await page.goto(url, {waitUntil: 'networkidle', timeout: 30000});
      if (wait_for) await page.locator(wait_for).waitFor({state: 'visible', timeout: 10000});
      if (!Array.isArray(inspect_selectors) || inspect_selectors.length > 16 || inspect_selectors.some(item => typeof item !== 'string' || item.length > 256)) throw Error('Inspect selectors are invalid');
      const evidence = await page.evaluate(selectors => ({
        title: document.title.slice(0, 256),
        text: (document.body?.innerText || '').slice(0, 8192),
        viewport: {width: innerWidth, height: innerHeight, device_scale_factor: devicePixelRatio},
        scroll_width: document.documentElement.scrollWidth,
        client_width: document.documentElement.clientWidth,
        scroll_height: document.documentElement.scrollHeight,
        client_height: document.documentElement.clientHeight,
        selectors: Object.fromEntries(selectors.map(selector => { const element = document.querySelector(selector); if (!element) return [selector, null]; const box = element.getBoundingClientRect(); return [selector, {text: (element.innerText || '').slice(0, 1024), visible: box.width > 0 && box.height > 0, x: box.x, y: box.y, width: box.width, height: box.height}]; })),
      }), inspect_selectors);
      if (evidence.viewport.width !== width || evidence.viewport.height !== height || evidence.viewport.device_scale_factor !== 2) throw Error('Browserless page metrics do not match the requested viewport and 2x device scale');
      let cdp;
      let bytes, plan;
      try {
        if (full_page === true) cdp = await context.newCDPSession(page);
        const cssContentSize = cdp ? (await cdp.send('Page.getLayoutMetrics')).cssContentSize : undefined;
        plan = screenshotPixelPlan({width, height, fullPage: full_page === true, cssContentSize});
        // A fixed CDP-derived clip bounds Chromium's allocation even if page JS
        // expands the document after the metrics read. Playwright's clip can be
        // viewport-limited, so use CDP for the beyond-viewport capture itself.
        if (cdp) {
          let timer;
          const capture = await Promise.race([
            cdp.send('Page.captureScreenshot', fullPageCaptureParams(plan)),
            new Promise((_, reject) => { timer = setTimeout(() => reject(Error('Browserless screenshot timed out')), 30000); }),
          ]).finally(() => clearTimeout(timer));
          if (typeof capture.data !== 'string' || capture.data.length > Math.ceil(MAX_OUTPUT / 3) * 4) throw Error('Browserless screenshot exceeds output bound');
          bytes = Buffer.from(capture.data, 'base64');
          if (bytes.length > MAX_OUTPUT) throw Error('Browserless screenshot exceeds output bound');
        } else bytes = await page.screenshot({type: 'png', fullPage: false, timeout: 30000});
        if (cdp) {
          const latest = (await cdp.send('Page.getLayoutMetrics')).cssContentSize;
          const current = screenshotPixelPlan({width, height, fullPage: true, cssContentSize: latest});
          if (current.clipWidth > plan.clipWidth || current.clipHeight > plan.clipHeight) throw Error('Browserless content grew during capture; retry');
        }
      } finally { await cdp?.detach(); }
      const [actualWidth, actualHeight] = pngDimensions(bytes);
      verifyScreenshotPixels(actualWidth, actualHeight, plan);
      checkRequest();
      return {bytes, width: actualWidth, height: actualHeight, evidence: {...evidence, console_errors: consoleErrors}};
    } finally { await context.close(); }
  });
}
function workspaceUrl(source) {
  assertRelative(source);
  return `http://design-tools:9002/files/${source.split('/').map(encodeURIComponent).join('/')}`;
}
export async function browserScreenshot(args) {
  await assertOutputReady(args);
  const tokens = await preflight(args);
  let url;
  if (args.source) { await workspacePath(args.source); url = workspaceUrl(args.source); }
  else {
    const candidate = new URL(args.url);
    if (candidate.protocol !== 'http:' || !['design-tools:9002', 'penpot-frontend:8080'].includes(candidate.host)) throw Error('Browserless URL must be an owned local workspace or Penpot page');
    url = candidate.toString();
  }
  const shot = await browserCapture(url, {...args, tokensCss: tokens.outputs['tokens.css']});
  const artifact = await publish(args.output, shot.bytes, outputOptions(args));
  const diff = args.baseline ? await odiffCompare({baseline: args.baseline, candidate: args.output}) : null;
  return {tokens, artifact, width: shot.width, height: shot.height, scale: 2, evidence: shot.evidence, qa: diff ? {status: diff.match ? 'match' : 'changed_pixels', odiff: diff} : {status: 'captured', inspected_pixels: true}};
}
export async function browserInspect(source, {baseline} = {}) {
  await workspacePath(source);
  if (path.extname(source).toLowerCase() === '.png') pngDimensions(await readFile(source));
  const shot = await browserCapture(workspaceUrl(source), browserInspectCaptureOptions);
  const output = `.mirket-design/qa/${crypto.randomUUID()}-inspect.png`;
  await workspacePath('.mirket-design/qa', {kind: 'directory', missing: true, createParents: true});
  await fs.mkdir(path.join(workspace, '.mirket-design/qa'), {recursive: true});
  const artifact = await publish(output, shot.bytes);
  const diff = baseline ? await odiffCompare({baseline, candidate: output}) : null;
  return {status: diff ? diff.match ? 'match' : 'changed_pixels' : 'captured', screenshot: artifact, width: shot.width, height: shot.height, scale: 2, evidence: shot.evidence, odiff: diff};
}
async function inspectSaved(source, baseline) {
  try { return await browserInspect(source, {baseline}); }
  catch (error) { return {status: 'failed', error: redactSecrets(error.message || error), artifact_retained: source}; }
}
export async function inspectHyperframesKeyframes(snapshots, baseline, inspect = inspectSaved) {
  if (!snapshots.length) throw Error('HyperFrames CLI snapshot produced no PNG frames');
  const keyframes = [];
  for (const [index, snapshot] of snapshots.entries()) {
    checkRequest();
    keyframes.push({source: snapshot.artifact.path, inspection: await inspect(snapshot.artifact.path, index === 0 ? baseline : undefined)});
  }
  const statuses = keyframes.map(item => item.inspection.status);
  return {
    status: statuses.includes('failed') ? 'failed' : statuses.includes('changed_pixels') ? 'changed_pixels' : baseline ? 'match' : 'captured',
    keyframes,
  };
}

export async function copyBoundedProjectTree(origin, stage, {label, onFile}) {
  if (typeof label !== 'string' || !label || typeof onFile !== 'function') throw Error('Bounded project traversal options are invalid');
  let files = 0, entries = 0, total = 0;
  async function copy(from, to, relative = '', depth = 0) {
    checkRequest();
    const directory = await fs.opendir(from, {bufferSize: 1});
    for await (const entry of directory) {
      checkRequest();
      entries++;
      const local = relative ? `${relative}/${entry.name}` : entry.name;
      if (entry.isFile()) {
        files++;
        if (files > MAX_PROJECT_FILES) throw Error(`${label} exceeds ${MAX_PROJECT_FILES} files`);
      }
      if (entries > MAX_PROJECT_ENTRIES) throw Error(`${label} exceeds ${MAX_PROJECT_ENTRIES} filesystem entries`);
      if (entry.name.startsWith('.') || ['node_modules', 'renders', 'snapshots'].includes(entry.name)) continue;
      const source = path.join(from, entry.name), dest = path.join(to, entry.name);
      if (entry.isSymbolicLink()) throw Error(`${label} symlink refused: ${local}`);
      if (entry.isDirectory()) {
        const childDepth = depth + 1;
        if (childDepth > MAX_PROJECT_DEPTH) throw Error(`${label} exceeds directory depth ${MAX_PROJECT_DEPTH}`);
        checkRequest();
        await fs.mkdir(dest);
        await copy(source, dest, local, childDepth);
        continue;
      }
      if (!entry.isFile()) throw Error(`Unsupported ${label} project asset: ${local}`);
      checkRequest();
      const bytes = await readBoundedProjectAsset(source, MAX_PROJECT_BYTES - total, label);
      total += bytes.length;
      checkRequest();
      await onFile({source, dest, local, bytes});
    }
  }
  await copy(origin, stage);
  return {files, entries, bytes: total};
}

async function stageProject(sourceDir, stage) {
  const origin = await workspacePath(sourceDir, {kind: 'directory'});
  await copyBoundedProjectTree(origin, stage, {label: 'Composition', onFile: async ({dest, local, bytes}) => {
    const ext = path.extname(local).toLowerCase();
    if (ext === '.svg') await fs.writeFile(dest, svgBytes(bytes.toString('utf8')));
    else if (['.png', '.jpg', '.jpeg', '.webp', '.avif'].includes(ext)) {
      const dims = ext === '.png' ? pngDimensions(bytes) : [2048, 2048];
      const format = ext === '.png' ? 'png' : ext === '.webp' ? 'webp' : ext === '.avif' ? 'avif' : 'jpeg';
      const processed = await imgproxyBytes({source: `${sourceDir}/${local}`, width: Math.min(dims[0], 4096), height: Math.min(dims[1], 4096), mode: 'fit', format});
      await fs.writeFile(dest, processed);
    } else await fs.writeFile(dest, bytes);
  }});
  const index = path.join(stage, 'index.html');
  let html = await fs.readFile(index, 'utf8');
  if (!/<head(?:\s|>)/i.test(html)) throw Error('HyperFrames project requires an HTML head');
  html = html.replace(/<head([^>]*)>/i, '<head$1><link rel="stylesheet" href="/__mirket_tokens.css">');
  await fs.writeFile(index, html);
}

export async function readBoundedProjectAsset(file, remaining, label) {
  if (!Number.isSafeInteger(remaining) || remaining < 0) throw Error(`${label} has an invalid remaining byte budget`);
  checkRequest();
  const handle = await fs.open(file, fsConstants.O_RDONLY | fsConstants.O_NOFOLLOW | fsConstants.O_NONBLOCK);
  try {
    const stat = await handle.stat();
    if (!stat.isFile()) throw Error(`${label} asset is not a regular file`);
    if (stat.size > remaining) throw Error(`${label} exceeds 64 MiB`);
    const chunks = []; let size = 0;
    for (;;) {
      checkRequest();
      const chunk = Buffer.allocUnsafe(Math.min(64 * 1024, remaining - size + 1));
      const {bytesRead} = await handle.read(chunk, 0, chunk.length, null);
      checkRequest();
      if (!bytesRead) break;
      size += bytesRead;
      if (size > remaining) throw Error(`${label} exceeds 64 MiB`);
      chunks.push(chunk.subarray(0, bytesRead));
    }
    return Buffer.concat(chunks, size);
  } finally { await handle.close(); }
}

export function parseHyperframesLint(stdout) {
  let report;
  try { report = JSON.parse(stdout); }
  catch { throw Error('HyperFrames linter returned invalid JSON'); }
  if (!report || typeof report !== 'object' || Array.isArray(report) ||
      typeof report.ok !== 'boolean' || !Number.isSafeInteger(report.errorCount) || report.errorCount < 0) {
    throw Error('HyperFrames linter returned an invalid verdict');
  }
  if (!report.ok || report.errorCount > 0) throw Error('HyperFrames linter reported errors');
  return report;
}
export function imageBatchBudget() {
  let bytes = 0, pixels = 0, count = 0;
  return image => {
    const dimensions = pngDimensions(image);
    bytes += image.length; pixels += dimensions[0] * dimensions[1]; count++;
    if (count > 64 || bytes > 64 * 1024 * 1024 || pixels > 64 * 1024 * 1024) throw Error('Image QA batch exceeds aggregate count, byte or pixel budget');
    return dimensions;
  };
}
export async function hyperframesRender(args) {
  await assertOutputReady(args);
  const tokens = await preflight(args);
  const sourceDir = args.project;
  await workspacePath(sourceDir, {kind: 'directory'});
  const format = args.format || 'mp4';
  if (!['mp4', 'webm'].includes(format) || !args.output?.endsWith(`.${format}`)) throw Error('HyperFrames output extension must match MP4 or WebM format');
  const stage = await temporaryDirectory('hyperframes');
  try {
    await stageProject(sourceDir, stage);
    await fs.copyFile(path.join(workspace, tokens.outputs['tokens.css']), path.join(stage, '__mirket_tokens.css'));
    const env = {HYPERFRAMES_NO_UPDATE_CHECK: '1', HYPERFRAMES_NO_TELEMETRY: '1', HYPERFRAMES_BROWSER_PATH: process.env.HYPERFRAMES_BROWSER_PATH || '/usr/bin/chromium', HYPERFRAMES_FFMPEG_PATH: process.env.HYPERFRAMES_FFMPEG_PATH || '/usr/bin/ffmpeg', HYPERFRAMES_FFPROBE_PATH: process.env.HYPERFRAMES_FFPROBE_PATH || '/usr/bin/ffprobe'};
    const lint = await run(process.execPath, [hf, 'lint', stage, '--json'], {env, timeoutMs: 45000});
    const lintReport = parseHyperframesLint(lint.stdout);
    const snapshotsDir = path.join(stage, 'snapshots');
    await run(process.execPath, [hf, 'snapshot', stage, '--frames', '3', '--output', snapshotsDir], {env, timeoutMs: 120000, maxStdout: 65536});
    const snapshots = [], acceptImage = imageBatchBudget();
    const snapshotRoot = `.mirket-design/qa/${crypto.randomUUID()}-hyperframes`;
    await fs.mkdir(path.join(workspace, snapshotRoot), {recursive: true});
    for (const name of (await fs.readdir(snapshotsDir)).sort()) {
      if (!name.endsWith('.png')) continue;
      const bytes = await regularOutput(path.join(snapshotsDir, name), 16 * 1024 * 1024);
      const dimensions = acceptImage(bytes);
      const artifact = await publish(`${snapshotRoot}/${name}`, bytes);
      snapshots.push({artifact, dimensions});
    }
    const qa = await inspectHyperframesKeyframes(snapshots, args.baseline);
    if (qa.status === 'failed' || qa.status === 'changed_pixels') {
      return {tokens, artifact: null, lint: lintReport, snapshots, qa, phases: ['lint', 'snapshot'], render_skipped: 'keyframe_qa', engine: 'hyperframes-cli'};
    }
    const renderFile = path.join(stage, `render.${format}`);
    await run(process.execPath, [hf, 'render', stage, '--format', format, '--output', renderFile], {env, timeoutMs: 300000, maxStdout: 65536});
    const bytes = await regularOutput(renderFile);
    const probe = await run(process.env.HYPERFRAMES_FFPROBE_PATH || '/usr/bin/ffprobe', ['-v', 'error', '-show_streams', '-show_format', '-of', 'json', renderFile], {timeoutMs: 30000});
    const media = JSON.parse(probe.stdout);
    if (!media.streams?.some(stream => stream.codec_type === 'video' && stream.width > 0 && stream.height > 0) || !(Number(media.format?.duration) > 0)) throw Error('HyperFrames CLI produced unusable video metadata');
    const artifact = await publish(args.output, bytes, outputOptions(args));
    return {tokens, artifact, lint: lintReport, snapshots, media: {duration: Number(media.format.duration), video: media.streams.filter(stream => stream.codec_type === 'video').map(stream => ({codec: stream.codec_name, width: stream.width, height: stream.height}))}, qa, phases: ['lint', 'snapshot', 'render'], engine: 'hyperframes-cli'};
  } finally { await fs.rm(stage, {recursive: true, force: true}); }
}

export async function typstCompile(args) {
  await assertOutputReady(args);
  const tokens = await preflight(args);
  await workspacePath(args.source);
  if (!args.source.endsWith('.typ') || !args.output?.endsWith('.pdf')) throw Error('Typst input must be .typ and output .pdf');
  const stage = await temporaryDirectory('typst');
  try {
    const parent = path.posix.dirname(args.source);
    const origin = parent === '.' ? workspace : await workspacePath(parent, {kind: 'directory'});
    await copyBoundedProjectTree(origin, stage, {label: 'Typst project', onFile: async ({dest, local, bytes}) => {
      const ext = path.extname(local).toLowerCase();
      if (ext === '.svg') await fs.writeFile(dest, svgBytes(bytes.toString('utf8')));
      else if (['.png', '.jpg', '.jpeg', '.webp', '.avif'].includes(ext)) {
        const dims = ext === '.png' ? pngDimensions(bytes) : [2048, 2048];
        const format = ext === '.png' ? 'png' : ext === '.webp' ? 'webp' : ext === '.avif' ? 'avif' : 'jpeg';
        await fs.writeFile(dest, await imgproxyBytes({source: parent === '.' ? local : `${parent}/${local}`, width: Math.min(dims[0], 4096), height: Math.min(dims[1], 4096), mode: 'fit', format}));
      } else await fs.writeFile(dest, bytes);
    }});
    const stageSource = path.join(stage, path.basename(args.source));
    const typstSource = await fs.readFile(stageSource, 'utf8');
    const tokenImport = '/.mirket-canonical-tokens.typ';
    await fs.copyFile(await workspacePath(tokens.outputs['tokens.typ']), path.join(stage, tokenImport.slice(1)));
    await fs.writeFile(stageSource, `#import "${tokenImport}": tokens\n${typstSource}`);
    const pdf = path.join(stage, 'document.pdf');
    await run('/usr/local/bin/typst', ['compile', '--root', stage, stageSource, pdf], {timeoutMs: 90000});
    const bytes = await regularOutput(pdf);
    if (!bytes.subarray(0, 5).equals(Buffer.from('%PDF-'))) throw Error('Typst did not produce a PDF');
    const artifact = await publish(args.output, bytes, outputOptions(args));
    const pageTemplate = path.join(stage, 'page-{p}.png');
    await run('/usr/local/bin/typst', ['compile', '--root', stage, '--format', 'png', '--pages', '1-65', stageSource, pageTemplate], {timeoutMs: 90000});
    await fs.mkdir(path.join(workspace, '.mirket-design/qa'), {recursive: true});
    const pageFiles = (await fs.readdir(stage)).filter(name => /^page-\d+\.png$/.test(name)).sort((a, b) => Number(a.match(/\d+/)[0]) - Number(b.match(/\d+/)[0]));
    if (!pageFiles.length || pageFiles.length > 64) throw Error('Typst did not render a bounded set of page PNGs');
    const pages = [], checks = [], acceptImage = imageBatchBudget();
    for (const [index, name] of pageFiles.entries()) {
      const pageBytes = await regularOutput(path.join(stage, name), 16 * 1024 * 1024);
      acceptImage(pageBytes);
      const page = await publish(`.mirket-design/qa/${crypto.randomUUID()}-typst-${name}`, pageBytes);
      pages.push(page);
      checks.push(await inspectSaved(page.path, index === 0 ? args.baseline : undefined));
    }
    const qa = {status: checks.some(item => item.status === 'failed') ? 'failed' : checks.some(item => item.status === 'changed_pixels') ? 'changed_pixels' : 'captured', pages: checks};
    return {tokens, artifact, pages, qa, engine: 'typst'};
  } finally { await fs.rm(stage, {recursive: true, force: true}); }
}

const forbiddenComfyType = /3[\s_-]*d|cad|mesh|splat|voxel|point[\s_-]*cloud|modeling|geometry|glb|gltf/i;
function comfyNodeMetadata(classType, info) {
  if (!info || typeof info !== 'object' || Array.isArray(info) ||
      typeof info.category !== 'string' || !Array.isArray(info.output) ||
      !info.output.every(type => typeof type === 'string') ||
      !info.input || typeof info.input !== 'object' || Array.isArray(info.input)) {
    throw Error(`ComfyUI class ${classType} has missing or malformed live metadata`);
  }
  if (info.name != null && info.name !== classType) throw Error(`ComfyUI class ${classType} has mismatched live metadata`);
  const signals = [classType, info.category, info.display_name, info.python_module, ...info.output];
  for (const group of Object.values(info.input)) {
    if (!group || typeof group !== 'object' || Array.isArray(group)) throw Error(`ComfyUI class ${classType} has malformed input metadata`);
    for (const spec of Object.values(group)) {
      if (typeof spec === 'string') signals.push(spec);
      else if (Array.isArray(spec) && typeof spec[0] === 'string') signals.push(spec[0]);
    }
  }
  if (signals.some(value => typeof value === 'string' && (forbiddenComfyType.test(value) || /^(?:OBJ|PLY|STL|FILE3D)$/i.test(value)))) {
    throw Error(`ComfyUI class ${classType} is a 3D/CAD/modeling node and is refused by the bitmap workflow route`);
  }
}
export async function submitComfyGraph(workflow, objectInfo, postPrompt) {
  if (!workflow || typeof workflow !== 'object' || Array.isArray(workflow) || !Object.keys(workflow).length || Object.keys(workflow).length > 128) {
    throw Error('ComfyUI workflow must be a nonempty API graph with at most 128 nodes');
  }
  if (!objectInfo || typeof objectInfo !== 'object' || Array.isArray(objectInfo)) throw Error('ComfyUI live object_info is malformed');
  for (const [nodeId, node] of Object.entries(workflow)) {
    if (nodeId.length > 128 || !node || typeof node !== 'object' || Array.isArray(node) ||
        typeof node.class_type !== 'string' || !node.class_type.length || node.class_type.length > 256) {
      throw Error(`ComfyUI graph node ${nodeId.slice(0, 128)} has no valid class_type`);
    }
    if (!Object.hasOwn(objectInfo, node.class_type)) throw Error(`ComfyUI graph node ${nodeId} uses unavailable class ${node.class_type}`);
    comfyNodeMetadata(node.class_type, objectInfo[node.class_type]);
  }
  checkRequest();
  return postPrompt({prompt: workflow, client_id: crypto.randomUUID()});
}
export async function comfyuiSubmit(args) {
  await assertOutputReady(args);
  const tokens = await preflight(args);
  const workflow = await readJson(args.workflow, 1024 * 1024);
  const liveInfo = await fetchBytes('http://comfyui:8188/object_info', {max: 16 * 1024 * 1024, timeoutMs: 20000});
  const objectInfo = JSON.parse(liveInfo.bytes.toString('utf8'));
  const queued = await submitComfyGraph(workflow, objectInfo, request => fetchBytes('http://comfyui:8188/prompt', {
    method: 'POST', body: JSON.stringify(request), headers: {'Content-Type': 'application/json'}, max: 65536,
  }));
  const job = JSON.parse(queued.bytes.toString('utf8'));
  if (!/^[a-z0-9-]{20,64}$/i.test(job.prompt_id || '')) throw Error('ComfyUI did not return a prompt ID');
  try {
  const deadline = Date.now() + 180000;
  let result;
  while (Date.now() < deadline) {
    checkRequest();
    await new Promise(resolve => setTimeout(resolve, 1000));
    const history = await fetchBytes(`http://comfyui:8188/history/${job.prompt_id}`, {max: 1024 * 1024});
    result = JSON.parse(history.bytes.toString('utf8'))[job.prompt_id];
    if (result) break;
  }
  if (!result || result.status?.status_str !== 'success') throw Error('ComfyUI workflow did not complete successfully before deadline');
  const first = Object.values(result.outputs || {}).flatMap(value => value.images || [])[0];
  if (!first?.filename) throw Error('ComfyUI workflow has no saved image output');
  const view = new URL('http://comfyui:8188/view');
  for (const [key, value] of Object.entries({filename: first.filename, subfolder: first.subfolder || '', type: first.type || 'output'})) view.searchParams.set(key, value);
  const image = await fetchBytes(view.toString(), {max: 32 * 1024 * 1024});
  pngDimensions(image.bytes);
  const artifact = await publish(args.output, image.bytes, outputOptions(args));
  const qa = await inspectSaved(args.output, args.baseline);
  return {tokens, artifact, qa, service: 'comfyui', prompt_id: job.prompt_id, workflow: args.workflow};
  } catch (error) {
    throw new Error(`ComfyUI prompt_id ${job.prompt_id} was accepted and may continue in the background: ${redactSecrets(error.message || error)}`, {cause: error});
  }
}
