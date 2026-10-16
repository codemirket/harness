#!/usr/bin/env node
/* MIT. Neutral browser evidence collector, never a semantic/craft grader.
 * Existing runtime only; no install, downloads or external artifact requests.
 * node observe.cjs ARTIFACT_DIR OBSERVATION_DIR [PLAN.json | --actions PLAN.json]
 * See controls/observer-contract.md for the frozen action interface. */
'use strict';
const fs = require('node:fs/promises');
const path = require('node:path');
const http = require('node:http');
const crypto = require('node:crypto');
const assert = require('node:assert/strict');

const DEFAULT_PLAYWRIGHT = 'playwright';
const DEFAULT_CHROME = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const VIEWPORTS = [{ width: 1280, height: 900 }, { width: 390, height: 844 }, { width: 320, height: 800 }];
const TARGET_KEYS = ['selector', 'role', 'label', 'testId', 'text'];
const ACTIONS = new Set(['click', 'focus', 'fill', 'check', 'uncheck', 'select', 'press', 'read_text', 'read_value', 'read_attr', 'read_count', 'read_storage', 'read_active', 'expect_text', 'expect_value', 'expect_count', 'expect_visible', 'expect_attr', 'expect_storage', 'expect_active', 'wait_for', 'reload', 'capture']);
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const normalize = value => value.replace(/\s+/g, ' ').trim();
const errorData = e => ({ name: e.name || 'Error', message: e.message || String(e), stack: e.stack || null });
const safeName = value => String(value).replace(/[^A-Za-z0-9_.-]/g, '-').replace(/^\.+/, '').slice(0, 100) || 'capture';
const inside = (root, file) => file === root || file.startsWith(root + path.sep);

async function binding(file, relativeTo) {
  const bytes = await fs.readFile(file);
  return { path: relativeTo ? path.relative(relativeTo, file).split(path.sep).join('/') : path.resolve(file), sha256: sha(bytes), bytes: bytes.length };
}
async function treeBindings(root) {
  const files = [];
  async function walk(dir) {
    for (const item of (await fs.readdir(dir, { withFileTypes: true })).sort((a, b) => a.name.localeCompare(b.name))) {
      const file = path.join(dir, item.name);
      assert.equal(item.isSymbolicLink(), false, `artifact symlink refused: ${path.relative(root, file)}`);
      if (item.isDirectory()) await walk(file);
      else if (item.isFile()) files.push(await binding(file, root));
      else throw new Error(`non-regular artifact refused: ${path.relative(root, file)}`);
    }
  }
  await walk(root);
  return { files, tree_sha256: sha(JSON.stringify(files)) };
}
function validateViewport(v) {
  assert.ok(v && Number.isInteger(v.width) && Number.isInteger(v.height) && v.width >= 240 && v.width <= 2560 && v.height >= 240 && v.height <= 2160, 'viewport must use integer dimensions in supported range');
}
function validateTarget(t) {
  assert.ok(t && typeof t === 'object' && !Array.isArray(t), 'target must be an object');
  const keys = TARGET_KEYS.filter(key => Object.hasOwn(t, key));
  assert.equal(keys.length, 1, 'target needs exactly one locator kind');
  assert.equal(typeof t[keys[0]], 'string', 'target locator must be a string');
  if (keys[0] === 'role') assert.equal(typeof t.name, 'string', 'role target needs a name');
}
function validatePlan(plan) {
  assert.equal(plan.schema_version, 1, 'unsupported probe schema');
  assert.ok(typeof plan.id === 'string' && plan.id.trim(), 'probe id required');
  assert.ok(Array.isArray(plan.scenarios), 'scenarios array required');
  const ids = new Set();
  for (const scenario of plan.scenarios) {
    assert.ok(typeof scenario.id === 'string' && scenario.id.trim() && !ids.has(scenario.id), 'scenario ids must be unique strings'); ids.add(scenario.id);
    if (scenario.viewport) validateViewport(scenario.viewport);
    assert.ok(Array.isArray(scenario.steps), 'scenario steps array required');
    const stepIds = new Set();
    for (const step of scenario.steps) {
      assert.ok(typeof step.id === 'string' && step.id.trim() && !stepIds.has(step.id), 'step ids must be unique strings'); stepIds.add(step.id);
      assert.ok(ACTIONS.has(step.action), `unknown action: ${step.action}`);
      if (step.target) validateTarget(step.target);
      if (!['read_storage', 'expect_storage', 'read_active', 'reload', 'capture', 'press'].includes(step.action)) validateTarget(step.target);
      if (step.action === 'expect_text') assert.equal(Number(Object.hasOwn(step, 'equals')) + Number(Object.hasOwn(step, 'contains')), 1, 'expect_text needs exactly one of equals or contains');
      if (step.action.startsWith('expect_') && !['expect_text', 'expect_active'].includes(step.action)) assert.ok(Object.hasOwn(step, 'equals'), `${step.action} needs equals`);
      if (step.action.endsWith('_storage')) {
        assert.equal(typeof step.key, 'string', 'storage key required');
        if (step.json_path !== undefined) assert.ok(Array.isArray(step.json_path) && step.json_path.every(k => typeof k === 'string' || Number.isInteger(k)), 'json_path must be an array of keys or indices');
      }
    }
  }
  return plan;
}
async function serve(root, errors) {
  const actualRoot = await fs.realpath(root);
  const types = { '.html': 'text/html; charset=utf-8', '.css': 'text/css; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.mjs': 'text/javascript; charset=utf-8', '.json': 'application/json; charset=utf-8', '.svg': 'image/svg+xml', '.png': 'image/png', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', '.webp': 'image/webp', '.woff': 'font/woff', '.woff2': 'font/woff2', '.ico': 'image/x-icon' };
  const server = http.createServer(async (req, res) => {
    try {
      assert.ok(['GET', 'HEAD'].includes(req.method), 'only static reads are served');
      const requested = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
      const candidate = path.resolve(actualRoot, `.${requested === '/' ? '/index.html' : requested}`);
      assert.ok(inside(actualRoot, candidate) && candidate !== actualRoot, 'path escape refused');
      let cursor = actualRoot;
      for (const part of path.relative(actualRoot, candidate).split(path.sep)) {
        cursor = path.join(cursor, part);
        assert.equal((await fs.lstat(cursor)).isSymbolicLink(), false, 'symlink refused');
      }
      assert.ok((await fs.stat(candidate)).isFile(), 'only regular files served');
      const bytes = await fs.readFile(candidate);
      res.writeHead(200, { 'Content-Type': types[path.extname(candidate)] || 'application/octet-stream', 'Cache-Control': 'no-store', 'X-Content-Type-Options': 'nosniff' });
      res.end(req.method === 'HEAD' ? undefined : bytes);
    } catch (error) {
      errors.push({ method: req.method, url: req.url, error: errorData(error) });
      res.writeHead(404, { 'Content-Type': 'text/plain' }); res.end('Not found');
    }
  });
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
  return { origin: `http://127.0.0.1:${server.address().port}`, close: () => new Promise(resolve => server.close(resolve)) };
}
function locator(page, target) {
  validateTarget(target);
  if (target.selector !== undefined) return page.locator(target.selector);
  if (target.role !== undefined) return page.getByRole(target.role, { name: target.name, exact: target.exact ?? true });
  if (target.label !== undefined) return page.getByLabel(target.label, { exact: target.exact ?? true });
  if (target.testId !== undefined) return page.getByTestId(target.testId);
  return page.getByText(target.text, { exact: target.exact ?? true });
}
async function settle(page) {
  await page.evaluate(async () => {
    await Promise.race([document.fonts.ready, new Promise((_, reject) => setTimeout(() => reject(new Error('font readiness timed out')), 3500))]);
    await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
  });
}
async function activeElement(page) {
  return page.evaluate(() => {
    const el = document.activeElement;
    if (!el) return null;
    const r = el.getBoundingClientRect(), s = getComputedStyle(el);
    return { tag: el.tagName.toLowerCase(), id: el.id || null, role: el.getAttribute('role'), name_hint: (el.getAttribute('aria-label') || el.labels?.[0]?.textContent || el.textContent || '').replace(/\s+/g, ' ').trim().slice(0, 300), rect: { x: r.x, y: r.y, width: r.width, height: r.height }, outline: { style: s.outlineStyle, width: s.outlineWidth, color: s.outlineColor, offset: s.outlineOffset }, box_shadow: s.boxShadow, opacity: s.opacity, visibility: s.visibility, in_viewport: r.bottom > 0 && r.right > 0 && r.top < innerHeight && r.left < innerWidth, note: 'Style and geometry are hints; inspect pixels for actual focus visibility, contrast and occlusion.' };
  });
}
async function snapshot(page) {
  return page.evaluate(() => {
    const visible = el => { const r = el.getBoundingClientRect(), s = getComputedStyle(el); return !!(r.width && r.height && s.display !== 'none' && s.visibility !== 'hidden'); };
    const rect = el => { const r = el.getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height }; };
    const short = el => ({ tag: el.tagName.toLowerCase(), id: el.id || null, role: el.getAttribute('role'), text: (el.getAttribute('aria-label') || el.textContent || '').replace(/\s+/g, ' ').trim().slice(0, 300), rect: rect(el) });
    const controls = [...document.querySelectorAll('button,input,select,textarea,a[href],[tabindex]')].map(el => ({ ...short(el), label: [...(el.labels || [])].map(l => l.textContent.replace(/\s+/g, ' ').trim()).join('; '), type: el.getAttribute('type'), disabled: !!el.disabled, value: 'value' in el ? el.value : null, checked: 'checked' in el ? el.checked : null, tab_index: el.tabIndex, visible: visible(el) }));
    const candidates = [...document.querySelectorAll('body *')].filter(visible);
    const outside = candidates.filter(el => { const r = el.getBoundingClientRect(); return r.left < -1 || r.right > innerWidth + 1; }).slice(0, 120).map(short);
    const clipped = candidates.filter(el => { const s = getComputedStyle(el); return ((el.scrollWidth > el.clientWidth + 1 && ['hidden', 'clip'].includes(s.overflowX)) || (el.scrollHeight > el.clientHeight + 1 && ['hidden', 'clip'].includes(s.overflowY))); }).slice(0, 120).map(el => ({ ...short(el), scroll_width: el.scrollWidth, client_width: el.clientWidth, scroll_height: el.scrollHeight, client_height: el.clientHeight, overflow: getComputedStyle(el).overflow }));
    return { title: document.title, lang: document.documentElement.lang, dir: document.documentElement.dir || 'ltr', url: location.pathname, viewport: { width: innerWidth, height: innerHeight, device_pixel_ratio: devicePixelRatio }, document: { scroll_width: document.documentElement.scrollWidth, scroll_height: document.documentElement.scrollHeight, body_scroll_width: document.body?.scrollWidth, horizontal_overflow: document.documentElement.scrollWidth > innerWidth + 1, root_overflow_x: getComputedStyle(document.documentElement).overflowX, body_overflow_x: document.body ? getComputedStyle(document.body).overflowX : null }, text: document.body?.innerText || '', headings: [...document.querySelectorAll('h1,h2,h3,h4,h5,h6')].map(short), landmarks: [...document.querySelectorAll('main,nav,header,footer,aside,[role="main"],[role="navigation"],[role="status"],[role="alert"],[role="dialog"]')].map(short), controls, outside_viewport_candidates: outside, clipped_content_candidates: clipped, images: [...document.images].map(el => ({ src: el.getAttribute('src'), alt: el.alt, complete: el.complete, natural_width: el.naturalWidth, natural_height: el.naturalHeight, ...rect(el) })), fonts: [...document.fonts].map(f => ({ family: f.family, status: f.status, weight: f.weight })), note: 'Geometry candidates require inspection. Document width alone does not establish content visibility or craft.' };
  });
}
async function motionState(page) {
  return page.evaluate(() => ({ preference: matchMedia('(prefers-reduced-motion: reduce)').matches ? 'reduce' : 'no-preference', animations: document.getAnimations().map(a => ({ name: a.animationName || null, play_state: a.playState, current_time: a.currentTime, target: a.effect?.target ? { tag: a.effect.target.tagName.toLowerCase(), id: a.effect.target.id || null } : null, timing: a.effect?.getComputedTiming() || null })), declared_motion: [...document.querySelectorAll('body *')].map(el => { const s = getComputedStyle(el); return { tag: el.tagName.toLowerCase(), id: el.id || null, animation_name: s.animationName, animation_duration: s.animationDuration, transition_property: s.transitionProperty, transition_duration: s.transitionDuration, scroll_behavior: s.scrollBehavior }; }).filter(s => s.animation_name !== 'none' || s.transition_duration.split(',').some(t => parseFloat(t) > 0) || s.scroll_behavior === 'smooth') }));
}
async function storageValue(page, step) {
  const raw = await page.evaluate(key => localStorage.getItem(key), step.key);
  if (step.json_path === undefined) return raw;
  assert.notEqual(raw, null, `localStorage item is absent: ${step.key}`);
  let value = JSON.parse(raw);
  for (const key of step.json_path) {
    assert.ok(value !== null && typeof value === 'object' && Object.hasOwn(value, key), `localStorage JSON path is absent at ${String(key)}`);
    value = value[key];
  }
  return value;
}
async function selectOptions(target, requested) {
  if (typeof requested !== 'string') return { selected: await target.selectOption(requested), requested, resolution: 'explicit_playwright_option' };
  await target.waitFor({ state: 'attached', timeout: 4000 });
  const options = await target.locator('option').evaluateAll(nodes => nodes.map(n => ({ value: n.value, label: n.label, disabled: n.disabled })));
  const exactValue = options.filter(o => o.value === requested && !o.disabled);
  let choice, resolution;
  if (exactValue.length === 1) { choice = { value: exactValue[0].value }; resolution = 'exact_value'; }
  else {
    const exactLabel = options.filter(o => o.label === requested && !o.disabled);
    const caseOnly = options.filter(o => !o.disabled && (o.value.toLocaleLowerCase('en') === requested.toLocaleLowerCase('en') || o.label.toLocaleLowerCase('en') === requested.toLocaleLowerCase('en')));
    if (exactLabel.length === 1) { choice = { label: exactLabel[0].label }; resolution = 'exact_visible_label_fallback'; }
    else if (!exactValue.length && !exactLabel.length && caseOnly.length === 1) { choice = { value: caseOnly[0].value }; resolution = 'unique_case_only_fallback'; }
    else { const error = new Error('select string is absent or ambiguous in enabled option values/labels'); error.selection_observation = { requested, options }; throw error; }
  }
  return { selected: await target.selectOption(choice), requested, resolution, options };
}
async function executeStep(page, step, capture) {
  const target = () => locator(page, step.target);
  let actual, expected;
  try { switch (step.action) {
    case 'click': await target().click(); actual = await activeElement(page); break;
    case 'focus': await target().focus(); actual = await activeElement(page); break;
    case 'fill': await target().fill(step.value); actual = await target().inputValue(); break;
    case 'check': await target().check(); actual = await target().isChecked(); break;
    case 'uncheck': await target().uncheck(); actual = await target().isChecked(); break;
    case 'select': actual = await selectOptions(target(), step.value); break;
    case 'press': if (step.target) await target().press(step.key); else await page.keyboard.press(step.key); actual = await activeElement(page); break;
    case 'read_text': actual = normalize(await target().innerText()); break;
    case 'read_value': actual = await target().inputValue(); break;
    case 'read_attr': actual = await target().getAttribute(step.attribute); break;
    case 'read_count': actual = await target().count(); break;
    case 'read_storage': actual = await storageValue(page, step); break;
    case 'read_active': actual = await activeElement(page); break;
    case 'expect_text': actual = normalize(await target().innerText()); expected = Object.hasOwn(step, 'equals') ? normalize(step.equals) : { contains: normalize(step.contains) }; if (Object.hasOwn(step, 'equals')) assert.equal(actual, expected); else assert.ok(actual.includes(expected.contains), 'rendered text lacks expected literal'); break;
    case 'expect_value': actual = await target().inputValue(); expected = step.equals; assert.equal(actual, expected); break;
    case 'expect_attr': actual = await target().getAttribute(step.attribute); expected = step.equals; assert.equal(actual, expected); break;
    case 'expect_count': actual = await target().count(); expected = step.equals; assert.equal(actual, expected); break;
    case 'expect_visible': actual = await target().isVisible(); expected = step.equals; assert.equal(actual, expected); break;
    case 'expect_storage': expected = step.equals; actual = await storageValue(page, step); assert.deepEqual(actual, expected); break;
    case 'expect_active': actual = await target().evaluate(el => document.activeElement === el); expected = true; assert.equal(actual, expected); break;
    case 'wait_for': await target().waitFor({ state: step.state, timeout: 4000 }); actual = step.state; break;
    case 'reload': await page.reload({ waitUntil: 'load' }); await settle(page); actual = { path: new URL(page.url()).pathname, active: await activeElement(page) }; break;
    case 'capture': actual = await capture(step.name || step.id); break;
    default: throw new Error(`unknown action: ${step.action}`);
  } } catch (error) { error.observer_actual = actual; error.observer_expected = expected; throw error; }
  return { status: step.action.startsWith('expect_') ? 'passed' : 'observed', actual, ...(expected === undefined ? {} : { expected }) };
}

async function observe(artifactDir, outputDir, planPath = null, provenance = {}, options = {}) {
  const root = await fs.realpath(path.resolve(artifactDir)), out = path.resolve(outputDir);
  assert.ok(!inside(root, out), 'observation output must be outside artifact write scope');
  await fs.mkdir(out, { recursive: true });
  assert.equal((await fs.lstat(out)).isSymbolicLink(), false, 'output symlink refused');
  const actualOut = await fs.realpath(out);
  assert.ok(!inside(root, actualOut), 'real observation output must be outside artifact write scope');
  const result = { schema_version: 1, kind: 'design_quality_observations', provenance: { artifact_origin: 'unverified', model_executions: null, ...provenance }, started_at: new Date().toISOString(), source_bindings: {}, runtime: {}, generic: { captures: [], layouts: [], focus: [], zoom: null, reduced_motion: null }, scenarios: [], console_errors: [], page_errors: [], network_errors: [], blocked_requests: [], server_errors: [], errors: [], limits: ['No automatic semantic/craft score, human visual acceptance or accessibility conformance is inferred.', 'Headless desktop Chrome only; narrow widths do not test physical touch, safe areas, screen readers or native clients.', 'Static loopback artifact only; localStorage evidence is local browser state, not a server write or real collaboration.', 'Network routing and file checks are collector precautions, not an operating-system sandbox.', 'Tab trace/style hints, geometry candidates, zoom approximation and computed motion require interpretation.'] };
  let browser, server;
  let plan = { schema_version: 1, id: 'generic-only', scenarios: [] };
  const recordError = (phase, e, extra = {}) => result.errors.push({ phase, ...extra, ...errorData(e) });
  const save = async () => { result.finished_at = new Date().toISOString(); await fs.writeFile(path.join(actualOut, 'observations.json'), JSON.stringify(result, null, 2) + '\n'); };
  try {
    result.source_bindings.artifact = await treeBindings(root);
    result.source_bindings.observer = await binding(__filename);
    if (planPath) { plan = validatePlan(JSON.parse(await fs.readFile(planPath, 'utf8'))); result.source_bindings.probe = await binding(path.resolve(planPath)); }
    result.probe_id = plan.id;
    const executablePath = process.env.CHROMIUM_PATH || DEFAULT_CHROME;
    const modulePath = process.env.PLAYWRIGHT_MODULE || DEFAULT_PLAYWRIGHT;
    const { chromium } = require(modulePath);
    server = await serve(root, result.server_errors);
    browser = await chromium.launch({ headless: true, executablePath, args: ['--disable-background-networking', '--disable-component-update', '--disable-domain-reliability', '--disable-sync', '--no-first-run'] });
    result.runtime = { node: process.version, platform: process.platform, arch: process.arch, browser: await browser.version(), browser_executable: executablePath, playwright_module: modulePath, device_pixel_ratio: 1, locale: 'en-GB', timezone: 'UTC', color_scheme: 'light', origin: server.origin };
    async function newPage(label, viewport, options = {}) {
      const context = await browser.newContext({ viewport, deviceScaleFactor: options.deviceScaleFactor || 1, acceptDownloads: false, serviceWorkers: 'block', locale: 'en-GB', timezoneId: 'UTC', colorScheme: 'light', reducedMotion: options.reducedMotion || 'no-preference' });
      await context.route('**/*', route => {
        const url = route.request().url();
        if (new URL(url).origin === server.origin || url.startsWith('data:') || url.startsWith('blob:')) return route.continue();
        result.blocked_requests.push({ context: label, url, method: route.request().method(), resource_type: route.request().resourceType() }); return route.abort('blockedbyclient');
      });
      if (typeof context.routeWebSocket === 'function') await context.routeWebSocket('**/*', ws => { const url = ws.url(); if (new URL(url).origin === server.origin.replace(/^http/, 'ws')) ws.connectToServer(); else { result.blocked_requests.push({ context: label, url, resource_type: 'websocket' }); ws.close({ code: 1008, reason: 'external requests blocked' }); } });
      else result.limits.push('This Playwright runtime lacks WebSocket routing; WebSocket blocking is unverified.');
      const page = await context.newPage(); page.setDefaultTimeout(4000); page.setDefaultNavigationTimeout(8000);
      page.on('console', msg => { if (msg.type() === 'error') result.console_errors.push({ context: label, text: msg.text(), location: msg.location() }); });
      page.on('pageerror', error => result.page_errors.push({ context: label, ...errorData(error) }));
      page.on('requestfailed', req => result.network_errors.push({ context: label, url: req.url(), resource_type: req.resourceType(), failure: req.failure() }));
      page.on('response', response => { if (response.status() >= 400) result.network_errors.push({ context: label, url: response.url(), status: response.status() }); });
      page.on('download', download => { result.errors.push({ phase: 'artifact_download', context: label, suggested_filename: download.suggestedFilename() }); void download.cancel(); });
      page.on('popup', popup => { result.errors.push({ phase: 'artifact_popup', context: label, url: popup.url(), note: 'Popup closed; inspect artifact intent separately.' }); void popup.close(); });
      page.on('crash', () => result.errors.push({ phase: 'page_crash', context: label }));
      await page.goto(server.origin + '/', { waitUntil: 'load' }); await settle(page);
      return { context, page };
    }
    const captureNames = new Map();
    async function capture(page, label, collection) {
      const baseStem = safeName(label), n = (captureNames.get(baseStem) || 0) + 1; captureNames.set(baseStem, n);
      const stem = n === 1 ? baseStem : `${baseStem}-${n}`, record = { id: label, viewport: page.viewportSize(), files: [], errors: [] };
      const snapFile = path.join(actualOut, stem + '.dom.json');
      try { await fs.writeFile(snapFile, JSON.stringify(await snapshot(page), null, 2) + '\n'); record.files.push({ role: 'dom_snapshot', ...await binding(snapFile, actualOut) }); } catch (e) { record.errors.push({ phase: 'snapshot', ...errorData(e) }); }
      for (const fullPage of [false, true]) {
        const file = path.join(actualOut, `${stem}.${fullPage ? 'fullpage' : 'viewport'}.png`);
        try { await page.screenshot({ path: file, fullPage, animations: 'allow', caret: 'initial', timeout: 8000 }); record.files.push({ role: fullPage ? 'fullpage' : 'viewport', ...await binding(file, actualOut) }); } catch (e) { record.errors.push({ phase: 'screenshot', full_page: fullPage, ...errorData(e) }); }
      }
      collection.push(record);
      return record;
    }
    const viewports = [...VIEWPORTS];
    if (options.wideWidth !== undefined) { validateViewport({ width: options.wideWidth, height: 900 }); if (!viewports.some(v => v.width === options.wideWidth)) viewports.splice(1, 0, { width: options.wideWidth, height: 900 }); }
    for (const viewport of viewports) {
      const label = `default-${viewport.width}`; let context;
      try {
        const opened = await newPage(label, viewport); context = opened.context;
        const dom = await snapshot(opened.page); result.generic.layouts.push({ viewport, document: dom.document, clipped_content_candidates: dom.clipped_content_candidates, outside_viewport_candidates: dom.outside_viewport_candidates });
        await capture(opened.page, label, result.generic.captures);
      } catch (e) { recordError('default_capture', e, { context: label }); } finally { if (context) await context.close().catch(e => recordError('context_close', e)); }
    }
    for (const viewport of [VIEWPORTS[0], VIEWPORTS[2]]) {
      const label = `keyboard-${viewport.width}`, focus = { viewport, trace: [], captures: [], note: 'First 24 Tabs or first repeated focus cycle, whichever comes first. Does not prove every control or modal path.' }; result.generic.focus.push(focus); let context;
      try {
        const opened = await newPage(label, viewport); context = opened.context; const page = opened.page;
        const seen = new Set();
        for (let i = 0; i < 24; i++) {
          await page.keyboard.press('Tab'); const active = await activeElement(page); focus.trace.push({ tab: i + 1, ...active });
          if (i < 3) await capture(page, `${label}-tab-${i + 1}`, focus.captures);
          const signature = JSON.stringify([active?.tag, active?.id, active?.name_hint, active?.rect]);
          if (seen.has(signature) && i > 1) { focus.stop_reason = 'repeated_focus_signature'; break; } seen.add(signature);
        }
      } catch (e) { focus.error = errorData(e); recordError('keyboard_trace', e, { context: label }); } finally { if (context) await context.close().catch(e => recordError('context_close', e)); }
    }
    {
      const zoom = { requested_scale: 2, method: '200_percent_reflow_approximation', css_viewport: { width: 640, height: 450 }, device_scale_factor: 2, captures: [], note: 'Half CSS dimensions at DPR2 approximates 200% reflow in a 1280×900 window. Browser chrome zoom and text-only enlargement are unverified.' }; result.generic.zoom = zoom; let context;
      try { const opened = await newPage('zoom-200-reflow', zoom.css_viewport, { deviceScaleFactor: 2 }); context = opened.context; zoom.layout = (await snapshot(opened.page)).document; await capture(opened.page, 'zoom-200-reflow', zoom.captures); } catch (e) { zoom.error = errorData(e); recordError('zoom_reflow', e); } finally { if (context) await context.close().catch(e => recordError('context_close', e)); }
    }
    {
      const motion = { viewport: VIEWPORTS[1], samples: [], captures: [], note: 'Computed state and 250ms samples can expose remaining motion. Zero animations does not prove interaction motion quality or correct interruption.' }; result.generic.reduced_motion = motion;
      for (const preference of ['no-preference', 'reduce']) {
        let context; try { const opened = await newPage(`motion-${preference}`, VIEWPORTS[1], { reducedMotion: preference }); context = opened.context; const before = await motionState(opened.page); await opened.page.waitForTimeout(250); const after = await motionState(opened.page); motion.samples.push({ requested_preference: preference, before, after }); await capture(opened.page, `motion-${preference}-390`, motion.captures); } catch (e) { motion.samples.push({ requested_preference: preference, error: errorData(e) }); recordError('motion_sample', e, { preference }); } finally { if (context) await context.close().catch(e => recordError('context_close', e)); }
      }
    }
    for (const scenario of plan.scenarios) {
      const record = { id: scenario.id, viewport: scenario.viewport || VIEWPORTS[0], steps: [], captures: [], errors: [] }; result.scenarios.push(record); let context; let priorFailure = false;
      try {
        const label = `scenario-${safeName(scenario.id)}`, opened = await newPage(label, record.viewport); context = opened.context; const page = opened.page;
        const shot = name => capture(page, `${label}-${safeName(name)}`, record.captures);
        await shot('initial');
        for (const step of scenario.steps) {
          const stepRecord = { id: step.id, action: step.action, request: step, prior_failure: priorFailure }; record.steps.push(stepRecord);
          try { Object.assign(stepRecord, await executeStep(page, step, shot)); }
          catch (e) { Object.assign(stepRecord, { status: 'failed', ...(Object.hasOwn(e, 'observer_actual') ? { actual: e.observer_actual } : {}), ...(Object.hasOwn(e, 'observer_expected') ? { expected: e.observer_expected } : {}), ...(e.selection_observation ? { selection_observation: e.selection_observation } : {}), error: errorData(e) }); priorFailure = true; await shot(`failure-${step.id}`); }
        }
        await shot('final'); record.completed = true;
      } catch (e) { record.errors.push(errorData(e)); recordError('scenario', e, { scenario: scenario.id }); } finally { if (context) await context.close().catch(e => recordError('context_close', e)); }
    }
    result.source_bindings.artifact_after = await treeBindings(root);
    result.artifact_unchanged = result.source_bindings.artifact.tree_sha256 === result.source_bindings.artifact_after.tree_sha256;
    if (!result.artifact_unchanged) result.errors.push({ phase: 'artifact_changed_during_observation', note: 'Source identity changed. Do not reuse captures as one frozen artifact.' });
    result.collection_completed = true;
  } catch (e) { result.collection_completed = false; recordError('collector', e); }
  finally { if (browser) await browser.close().catch(e => recordError('browser_close', e)); if (server) await server.close().catch(e => recordError('server_close', e)); await save(); }
  return result;
}
if (require.main === module) {
  const [artifactDir, outputDir, ...args] = process.argv.slice(2); let planPath = null, wideWidth; let valid = true;
  for (let i = 0; i < args.length; i++) { if (args[i] === '--actions' && args[i + 1]) planPath = args[++i]; else if (args[i] === '--wide-width' && args[i + 1]) wideWidth = Number(args[++i]); else if (!args[i].startsWith('--') && planPath === null) planPath = args[i]; else valid = false; }
  if (!artifactDir || !outputDir || !valid) { console.error('Usage: node observe.cjs ARTIFACT_DIR OBSERVATION_DIR [PLAN.json | --actions PLAN.json] [--wide-width 1440]'); process.exitCode = 1; }
  else observe(artifactDir, outputDir, planPath, {}, { wideWidth }).then(result => { console.log(JSON.stringify({ collection_completed: result.collection_completed, artifact_unchanged: result.artifact_unchanged, scenarios: result.scenarios.map(s => ({ id: s.id, failed_steps: s.steps.filter(st => st.status === 'failed').map(st => st.id) })), errors: result.errors.length, observation_file: path.resolve(outputDir, 'observations.json'), semantic_or_craft_grade: null })); if (!result.collection_completed) process.exitCode = 1; }).catch(e => { console.error(e.stack); process.exitCode = 1; });
}
module.exports = { observe, validatePlan, VIEWPORTS };
