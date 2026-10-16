#!/usr/bin/env node
/* MIT. Real-browser regression for the additive actual-viewport collector.
 * node evals/design-quality/test_viewport_proof.cjs [NEW_EVIDENCE_DIRECTORY]
 * Uses the existing PLAYWRIGHT_MODULE and CHROMIUM_PATH overrides from observe.cjs.
 * VIEWPORT_PROOF_MODULE selects a negative-control module for test calibration.
 * No installs, model outputs, screenshot rewrites or screenshot stitching.
 * Evidence (including failures) is retained in a new directory, never overwritten.
 */
'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const os = require('node:os');
const http = require('node:http');
const { createHash } = require('node:crypto');

const DEFAULT_PLAYWRIGHT = 'playwright';
const DEFAULT_CHROME = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const MODULE = path.resolve(process.env.VIEWPORT_PROOF_MODULE || path.join(__dirname, 'viewport-proof.cjs'));
const { captureViewportProof } = require(MODULE);
// Literal, hand-worked coverage for the fixture's 2,337 CSS px document and 128px overlap.
const SIZES = [
  { width: 1280, height: 900, scan: [0, 772, 1437] },
  { width: 390, height: 844, scan: [0, 716, 1432, 1493] },
  { width: 320, height: 800, scan: [0, 672, 1344, 1537] },
];
const COLORS = { skip: [255, 0, 213], obstruction: [221, 49, 49], sticky: [23, 169, 100], indicator: [33, 70, 244] };
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const json = value => JSON.stringify(value, null, 2) + '\n';
const binding = async file => { const bytes = await fs.readFile(file); return { path: path.resolve(file), bytes: bytes.length, sha256: hash(bytes) }; };
const freshJson = (file, value) => fs.writeFile(file, json(value), { flag: 'wx' });

function fixture(mode, growth = '') {
  const hidden = mode === 'transform' ? 'top:8px;transform:translateY(-140%)' : 'top:-90px';
  return `<!doctype html><html lang="en"><meta charset="utf-8"><title>Synthetic viewport evidence</title>
<style>
*{box-sizing:border-box}html{scrollbar-width:none;scroll-behavior:auto}body{margin:0;background:#f4f6f8;font:16px sans-serif}
#sticky{position:sticky;top:0;z-index:9;margin-left:210px;width:100px;height:32px;background:rgb(23,169,100)}
main{height:2305px;width:calc(100vw + 80px);background:linear-gradient(#f4f6f8,#e1e6eb)}
#skip{position:fixed;left:8px;${hidden};z-index:20;display:block;width:176px;height:48px;background:rgb(255,0,213);color:#000}
#skip:focus{top:8px;transform:none}
#action,#obstruction{position:fixed;top:150px;left:24px;width:200px;height:80px}
#action{z-index:5;background:#123276}#obstruction{z-index:6;background:rgb(221,49,49);pointer-events:none}
#focus-wrap{position:fixed;left:12px;bottom:20px;padding:9px;background:#fff;z-index:7}
#focus-wrap:focus-within{outline:4px solid rgb(33,70,244)}#choice{outline:none}
</style><body><a id="skip" href="#content">Skip content</a><header id="sticky" aria-label="Sticky marker"></header>
<main id="content" tabindex="-1"></main><button id="action">Covered action</button><div id="obstruction" aria-hidden="true"></div>
<label id="focus-wrap"><input id="choice" type="checkbox" style="">Native choice</label>
<script>
let sawBottom=false, grew=false;
addEventListener('scroll',()=>{
  if(scrollY>1400)sawBottom=true;
  if(!grew && (${growth === 'scan' ? 'scrollY>900' : growth === 'restore' ? 'sawBottom && scrollY===487' : 'false'})){
    grew=true;document.querySelector('main').style.height='3205px';
  }
});
</script></body></html>`;
}

async function state(page) {
  return page.evaluate(() => {
    const skip = document.querySelector('#skip'), r = skip.getBoundingClientRect();
    return { x: scrollX, y: scrollY, width: innerWidth, height: innerHeight,
      document_width: document.documentElement.scrollWidth, document_height: document.documentElement.scrollHeight,
      active: document.activeElement.id, skip: { y: r.y, bottom: r.bottom, focused: skip.matches(':focus') } };
  });
}
async function settle(page) { await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))); }
async function prepare(page, focused = 'action') {
  await page.evaluate(focused => { document.getElementById(focused).focus({ preventScroll: true }); scrollTo({ left: 37, top: 487, behavior: 'instant' }); }, focused);
  await settle(page);
}
function watchScreenshots(page) {
  const original = page.screenshot.bind(page), calls = [];
  page.screenshot = async options => {
    const bytes = await original(options);
    calls.push({ options: { ...options }, sha256: hash(bytes), bytes: bytes.length, buffer: Buffer.from(bytes) });
    return bytes;
  };
  return { calls, original, restore() { page.screenshot = original; }, summary() { return calls.map(({ buffer, ...call }) => call); } };
}
async function pixels(inspector, bytes) {
  assert.ok(bytes.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10])), 'saved artifact must be a PNG');
  const dimensions = { width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) };
  const counts = await inspector.evaluate(async ({ base64, colors }) => {
    const image = new Image();
    image.src = 'data:image/png;base64,' + base64;
    await image.decode();
    const canvas = document.createElement('canvas');
    canvas.width = image.naturalWidth; canvas.height = image.naturalHeight;
    const context = canvas.getContext('2d', { willReadFrequently: true });
    context.drawImage(image, 0, 0);
    const rgba = context.getImageData(0, 0, canvas.width, canvas.height).data;
    const colorEntries = Object.entries(colors);
    const result = Object.fromEntries(Object.keys(colors).map(key => [key, { count: 0, bounds: null }]));
    for (let offset = 0; offset < rgba.length; offset += 4) {
      for (const [name, color] of colorEntries) {
        if (rgba[offset] !== color[0] || rgba[offset + 1] !== color[1] || rgba[offset + 2] !== color[2] || rgba[offset + 3] !== 255) continue;
        const value = result[name], x = (offset / 4) % canvas.width, y = Math.floor(offset / 4 / canvas.width);
        value.count++;
        if (!value.bounds) value.bounds = { left: x, right: x, top: y, bottom: y };
        else { value.bounds.left = Math.min(value.bounds.left, x); value.bounds.right = Math.max(value.bounds.right, x); value.bounds.top = Math.min(value.bounds.top, y); value.bounds.bottom = Math.max(value.bounds.bottom, y); }
      }
    }
    return result;
  }, { base64: bytes.toString('base64'), colors: COLORS });
  return { ...dimensions, colors: counts };
}
async function entries(directory) {
  const values = [];
  for (const name of (await fs.readdir(directory)).sort()) {
    const file = path.join(directory, name), stat = await fs.lstat(file);
    values.push({ name, kind: stat.isSymbolicLink() ? 'link' : stat.isFile() ? 'file' : 'directory',
      identity: stat.isSymbolicLink() ? await fs.readlink(file) : stat.isFile() ? hash(await fs.readFile(file)) : null });
  }
  return values;
}

async function main() {
  assert.ok(process.argv.length <= 3, 'Usage: node test_viewport_proof.cjs [NEW_EVIDENCE_DIRECTORY]');
  const out = process.argv[2] ? path.resolve(process.argv[2]) : await fs.mkdtemp(path.join(os.tmpdir(), 'mirket-viewport-proof-'));
  if (process.argv[2]) await fs.mkdir(out);
  const report = { schema_version: 1, kind: 'actual_viewport_regression', started_at: new Date().toISOString(),
    module: await binding(MODULE), checker: await binding(__filename), evidence_directory: out, tests: [],
    limits: ['Synthetic loopback pages and actual headless desktop Chrome only.', 'Pixel assertions establish fixture visibility and capture identity, not design quality, human acceptance or assistive-technology behavior.', 'Original PNG bytes are retained without rewriting, stitching or masking.'] };
  // Keep the exact checker and module for a failed probe even after a later repair.
  for (const [source, name, expected] of [[MODULE, 'module.cjs', report.module], [__filename, 'checker.cjs', report.checker]]) {
    const bytes = await fs.readFile(source);
    assert.equal(hash(bytes), expected.sha256, 'source changed while binding this run');
    await fs.writeFile(path.join(out, name), bytes, { flag: 'wx' });
  }
  const save = () => fs.writeFile(path.join(out, 'results.json'), json(report));
  const pages = new Map();
  const server = http.createServer((req, res) => { const source = pages.get(req.url); if (!source) { res.writeHead(404); res.end(); return; } res.setHeader('Content-Type', 'text/html; charset=utf-8'); res.end(source); });
  let browser, inspector;
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const origin = `http://127.0.0.1:${server.address().port}`;
  async function run(name, action) {
    const directory = path.join(out, name); await fs.mkdir(directory);
    const result = { name, directory, status: 'running' }; report.tests.push(result);
    try { result.observation = await action(directory); result.status = 'passed'; }
    catch (error) { result.status = 'failed'; result.error = { name: error.name, message: error.message, stack: error.stack }; }
    await save();
    process.stdout.write(`${result.status}: ${name}\n`);
  }
  async function pageFor(directory, size, mode = 'top', growth = '') {
    const source = fixture(mode, growth), route = '/' + path.basename(directory);
    pages.set(route, source);
    await fs.writeFile(path.join(directory, 'fixture.html'), source, { flag: 'wx' });
    const context = await browser.newContext({ viewport: { width: size.width, height: size.height }, deviceScaleFactor: size.dpr || 1, serviceWorkers: 'block', locale: 'en-GB', timezoneId: 'UTC', colorScheme: 'light', acceptDownloads: false });
    await context.route('**/*', route => new URL(route.request().url()).origin === origin ? route.continue() : route.abort('blockedbyclient'));
    const page = await context.newPage(); await page.goto(origin + route, { waitUntil: 'load' }); await prepare(page);
    return { page, context };
  }
  async function inspectCapture(directory, page, watch, result) {
    const facts = [];
    for (const frame of result.frames) {
      const bytes = await fs.readFile(path.join(directory, frame.file));
      facts.push({ file: frame.file, binding: await binding(path.join(directory, frame.file)), pixels: await pixels(inspector, bytes) });
    }
    await freshJson(path.join(directory, 'pixel-inspection.json'), { facts, screenshot_calls: watch.summary() });
    // Pixel assertions precede collector metadata so hiding real overlays cannot pass.
    for (const fact of facts) {
      assert.equal(fact.pixels.width, result.initial.width, 'PNG must be an actual CSS viewport width');
      assert.equal(fact.pixels.height, result.initial.height, 'PNG must be an actual CSS viewport height');
      assert.ok(fact.pixels.colors.obstruction.count >= 15500, 'real fixed obstruction must remain in every viewport PNG');
      assert.ok(fact.pixels.colors.sticky.count >= 2000, 'genuine sticky marker must remain in every viewport PNG');
    }
    assert.equal(watch.calls.length, result.frames.length, 'each saved frame must originate in one real screenshot');
    for (let i = 0; i < result.frames.length; i++) {
      const frame = result.frames[i], call = watch.calls[i], bytes = await fs.readFile(path.join(directory, frame.file));
      assert.equal(call.options.fullPage, false, 'collector must request actual viewport pixels');
      assert.equal(call.options.scale, 'css');
      assert.equal(frame.sha256, hash(bytes), 'manifest hash must bind saved PNG bytes');
      assert.equal(frame.bytes, bytes.length);
      assert.equal(call.sha256, frame.sha256);
      assert.deepEqual(bytes, call.buffer, 'saved screenshot must be the unmodified returned bytes');
    }
    assert.deepEqual(JSON.parse(await fs.readFile(path.join(directory, result.label + '-viewports.json'), 'utf8')), result, 'saved manifest must describe returned evidence');
    return facts;
  }
  try {
    const modulePath = process.env.PLAYWRIGHT_MODULE || DEFAULT_PLAYWRIGHT, executablePath = process.env.CHROMIUM_PATH || DEFAULT_CHROME;
    browser = await require(modulePath).chromium.launch({ headless: true, executablePath, args: ['--disable-background-networking', '--disable-component-update', '--disable-domain-reliability', '--disable-sync', '--no-first-run'] });
    inspector = await browser.newPage();
    report.runtime = { node: process.version, platform: process.platform, arch: process.arch, browser: await browser.version(), playwright_module: modulePath, browser_executable: executablePath, origin };
    for (const size of SIZES) for (const mode of ['top', 'transform']) for (const focused of [false, true]) {
      await run(`${size.width}-${mode}-${focused ? 'focused' : 'offscreen'}`, async directory => {
        const { page, context } = await pageFor(directory, size, mode);
        const watch = watchScreenshots(page);
        try {
          if (focused) await prepare(page, 'skip');
          const before = await state(page), html = await page.content();
          let fullPageDiagnostic = null;
          if (!focused) {
            const viewport = await watch.original({ fullPage: false, animations: 'disabled', scale: 'css' });
            const fullPage = await watch.original({ fullPage: true, animations: 'disabled', scale: 'css' });
            await fs.writeFile(path.join(directory, 'diagnostic-viewport.png'), viewport, { flag: 'wx' });
            await fs.writeFile(path.join(directory, 'diagnostic-fullpage.png'), fullPage, { flag: 'wx' });
            fullPageDiagnostic = { before, after: await state(page), viewport: await pixels(inspector, viewport), full_page: await pixels(inspector, fullPage) };
            fullPageDiagnostic.ghost_reproduced = fullPageDiagnostic.viewport.colors.skip.count === 0 && fullPageDiagnostic.full_page.colors.skip.count > 0;
            await freshJson(path.join(directory, 'fullpage-diagnostic.json'), fullPageDiagnostic);
            assert.equal(fullPageDiagnostic.viewport.colors.skip.count, 0, 'offscreen skiplink is absent in direct actual viewport control');
            assert.deepEqual(fullPageDiagnostic.after, before, 'diagnostic capture must preserve the starting state');
          }
          const result = await captureViewportProof(page, directory, 'proof', size.width === 1280 ? { maxFrames: 4 } : {});
          const facts = await inspectCapture(directory, page, watch, result);
          for (const fact of facts) assert.ok(focused ? fact.pixels.colors.skip.count >= 7000 : fact.pixels.colors.skip.count === 0, focused ? 'focused skiplink must be visible in every actual viewport' : 'hidden skiplink must be absent in every actual viewport');
          const scan = result.frames.filter(frame => frame.role === 'document_scan');
          assert.deepEqual(scan.map(frame => frame.before.scroll_y), size.scan, 'uniform document coverage must match independent worked positions');
          assert.equal(result.initial.document_height, 2337);
          assert.equal(result.frames[0].role, 'current'); assert.equal(result.frames[0].before.scroll_y, before.y);
          assert.equal(result.frames[0].before.scroll_x, before.x);
          assert.ok(scan.every((frame, i) => i === 0 || frame.before.scroll_y > scan[i - 1].before.scroll_y && frame.before.scroll_y <= scan[i - 1].before.scroll_y + size.height - 128));
          assert.ok(scan.at(-1).before.scroll_y + size.height >= 2337);
          assert.equal(result.document_scan_complete, true); assert.equal(result.stable_dimensions, true); assert.equal(result.scroll_restored, true);
          assert.deepEqual(await state(page), before, 'original horizontal/vertical scroll and focus must be preserved');
          assert.equal(await page.content(), html, 'collector must not alter fixture markup/styles');
          assert.ok(result.frames.every(frame => frame.before.active.id === (focused ? 'skip' : 'action') && frame.after.active.id === (focused ? 'skip' : 'action')));
          return { frames: facts.length, fullpage_ghost_reproduced: fullPageDiagnostic?.ghost_reproduced ?? null, source: await binding(path.join(directory, 'fixture.html')) };
        } finally { watch.restore(); await context.close(); }
      });
    }
    await run('390-dpr2-parent-focus-indicator', async directory => {
      const { page, context } = await pageFor(directory, { ...SIZES[1], dpr: 2 });
      const watch = watchScreenshots(page);
      try {
        await prepare(page, 'choice'); const before = await state(page);
        const result = await captureViewportProof(page, directory, 'proof');
        const facts = await inspectCapture(directory, page, watch, result);
        assert.ok(facts.every(fact => fact.pixels.colors.indicator.count > 300), 'containing-label focus indicator must remain in actual viewport pixels');
        for (const frame of result.frames) {
          const active = frame.before.focus_chain.find(item => item.id === 'choice'), parent = frame.before.focus_chain.find(item => item.id === 'focus-wrap');
          assert.equal(active.outline.style, 'none'); assert.equal(active.focused, true);
          assert.equal(parent.outline.style, 'solid'); assert.equal(parent.outline.width, '4px');
          assert.equal(parent.focused, false); assert.equal(typeof active.focus_visible, 'boolean');
        }
        assert.deepEqual(await state(page), before);
        return { frames: facts.length, device_pixel_ratio: 2 };
      } finally { watch.restore(); await context.close(); }
    });
    for (const growth of ['scan', 'restore']) await run('dynamic-dimensions-' + growth, async directory => {
      const { page, context } = await pageFor(directory, SIZES[0], 'top', growth);
      const watch = watchScreenshots(page);
      try {
        const before = await state(page), result = await captureViewportProof(page, directory, 'proof');
        const facts = await inspectCapture(directory, page, watch, result), after = await state(page);
        await freshJson(path.join(directory, 'independent-state.json'), { before, after });
        assert.equal(after.document_height, 3237, 'fixture must produce a real observed dimension change');
        assert.equal(result.document_scan_complete, false, 'observed dynamic dimensions must prevent complete evidence');
        assert.equal(result.stable_dimensions, false, 'observed dynamic dimensions must be disclosed');
        assert.equal(after.x, before.x); assert.equal(after.y, before.y); assert.equal(after.active, before.active);
        return { frames: facts.length, before, after };
      } finally { watch.restore(); await context.close(); }
    });
    const refusals = [
      { name: 'invalid-label', label: '../escape' },
      { name: 'negative-overlap', options: { overlap: -1 } },
      { name: 'overlap-at-viewport-height', options: { overlap: 900 } },
      { name: 'invalid-frame-cap', options: { maxFrames: 1 } },
      { name: 'over-limit-frame-cap', options: { maxFrames: 257 } },
      { name: 'predeclared-frame-cap', options: { maxFrames: 2 }, error: /frame cap/ },
      { name: 'existing-current', seed: 'blocked-current.png', error: /output exists/ },
      { name: 'existing-later-scan', seed: 'blocked-scroll-003.png', error: /output exists/ },
      { name: 'existing-manifest', seed: 'blocked-viewports.json', error: /output exists/ },
      { name: 'existing-output-symlink', seedLink: true, error: /output exists/ },
      { name: 'output-directory-symlink', symlink: true, error: /symlink refused/ },
      { name: 'non-directory-output', file: true },
      { name: 'missing-output', missing: true },
    ];
    for (const spec of refusals) await run('refusal-' + spec.name, async directory => {
      const { page, context } = await pageFor(directory, SIZES[0]);
      const watch = watchScreenshots(page), actual = path.join(directory, 'outputs');
      try {
        let destination = actual;
        if (spec.file) await fs.writeFile(actual, 'existing file\n', { flag: 'wx' });
        else if (!spec.missing) await fs.mkdir(actual);
        if (spec.seed) await fs.writeFile(path.join(actual, spec.seed), 'must remain unchanged\n', { flag: 'wx' });
        if (spec.seedLink) {
          const target = path.join(directory, 'original-target');
          await fs.writeFile(target, 'symlink target must remain unchanged\n', { flag: 'wx' });
          await fs.symlink(target, path.join(actual, 'blocked-current.png'));
        }
        if (spec.symlink) { destination = path.join(directory, 'output-link'); await fs.symlink(actual, destination); }
        const before = await state(page), filesBefore = await entries(directory), outputsBefore = !spec.file && !spec.missing ? await entries(actual) : null;
        await assert.rejects(captureViewportProof(page, destination, spec.label || 'blocked', spec.options), spec.error);
        assert.equal(watch.calls.length, 0, 'refusal must precede screenshots and output writes');
        assert.deepEqual(await state(page), before, 'preflight refusal must preserve starting scroll/focus');
        assert.deepEqual(await entries(directory), filesBefore, 'preflight must not leave partial output');
        if (outputsBefore) assert.deepEqual(await entries(actual), outputsBefore, 'existing artifacts must not be overwritten');
        return { screenshots: 0, state_preserved: true, existing_outputs_preserved: true };
      } finally { watch.restore(); await context.close(); }
    });
    await run('destination-race-refuses-overwrite', async directory => {
      const { page, context } = await pageFor(directory, SIZES[0]);
      const watch = watchScreenshots(page), target = path.join(directory, 'race-current.png');
      try {
        const before = await state(page), screenshot = page.screenshot;
        page.screenshot = async options => {
          const bytes = await screenshot(options);
          // A competing writer creates a destination after preflight, before save.
          await fs.writeFile(target, 'competing writer sentinel\n', { flag: 'wx' });
          return bytes;
        };
        await assert.rejects(captureViewportProof(page, directory, 'race'), error => error.code === 'EEXIST');
        assert.equal(await fs.readFile(target, 'utf8'), 'competing writer sentinel\n');
        const partial = JSON.parse(await fs.readFile(path.join(directory, 'race-viewports.json'), 'utf8'));
        assert.equal(partial.document_scan_complete, false); assert.equal(partial.frames.length, 0);
        assert.deepEqual(await state(page), before);
        return { competing_file_preserved: true, retained_frames: 0, screenshots: watch.calls.length };
      } finally { watch.restore(); await context.close(); }
    });
    await run('screenshot-failure-restores-state', async directory => {
      const { page, context } = await pageFor(directory, SIZES[0]);
      const watch = watchScreenshots(page);
      try {
        const before = await state(page), screenshot = page.screenshot;
        page.screenshot = async options => { if (watch.calls.length === 1) throw new Error('injected screenshot failure'); return screenshot(options); };
        await assert.rejects(captureViewportProof(page, directory, 'partial'), /injected screenshot failure/);
        const partial = JSON.parse(await fs.readFile(path.join(directory, 'partial-viewports.json'), 'utf8'));
        assert.equal(partial.document_scan_complete, false); assert.equal(partial.frames.length, 1);
        const bytes = await fs.readFile(path.join(directory, partial.frames[0].file));
        assert.deepEqual(bytes, watch.calls[0].buffer); assert.equal(partial.frames[0].sha256, hash(bytes));
        assert.equal(partial.scroll_restored, true); assert.deepEqual(await state(page), before);
        return { retained_frames: 1, error_propagated: true, state_preserved: true };
      } finally { watch.restore(); await context.close(); }
    });
  } catch (error) { report.fatal_error = { name: error.name, message: error.message, stack: error.stack }; }
  finally {
    if (browser) await browser.close();
    await new Promise(resolve => server.close(resolve));
    report.finished_at = new Date().toISOString(); report.module_after = await binding(MODULE); report.checker_after = await binding(__filename);
    report.module_unchanged = report.module_after.sha256 === report.module.sha256;
    report.checker_unchanged = report.checker_after.sha256 === report.checker.sha256;
    report.ok = !report.fatal_error && report.module_unchanged && report.checker_unchanged && report.tests.every(test => test.status === 'passed');
    await save();
  }
  process.stdout.write(json({ ok: report.ok, test_count: report.tests.length, passed: report.tests.filter(test => test.status === 'passed').length,
    failed: report.tests.filter(test => test.status !== 'passed').map(test => ({ name: test.name, message: test.error?.message })),
    result: await binding(path.join(out, 'results.json')), module: report.module, checker: report.checker }));
  if (!report.ok) process.exitCode = 1;
}
main().catch(error => { process.stderr.write(error.stack + '\n'); process.exitCode = 2; });
