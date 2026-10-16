#!/usr/bin/env node
/* Local observation helper, not a semantic/craft grader. No downloads.
 * PLAYWRIGHT_MODULE=/absolute/existing/module CHROMIUM_PATH=/absolute/browser
 * node evals/design-coverage/check-artifact.cjs CASE ARTIFACT_DIR OBSERVATION_DIR
 * node evals/design-coverage/check-artifact.cjs --self-test OBSERVATION_DIR
 * Grader output must be outside the worker's artifact directory. */
'use strict';
const fs = require('node:fs/promises');
const path = require('node:path');
const http = require('node:http');
const crypto = require('node:crypto');
const assert = require('node:assert/strict');
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');

async function fileBinding(file) {
  return { path: path.resolve(file), sha256: sha(await fs.readFile(file)) };
}
async function launch() {
  return chromium.launch({ headless: true, ...(process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH } : {}) });
}
async function serve(root) {
  const actualRoot = await fs.realpath(root);
  const server = http.createServer(async (req, res) => {
    try {
      const requested = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
      const candidate = path.resolve(actualRoot, `.${requested === '/' ? '/index.html' : requested}`);
      if (!candidate.startsWith(actualRoot + path.sep)) throw new Error('path escape');
      let cursor = actualRoot;
      for (const part of path.relative(actualRoot, candidate).split(path.sep)) {
        cursor = path.join(cursor, part);
        if ((await fs.lstat(cursor)).isSymbolicLink()) throw new Error('symlink refused');
      }
      const bytes = await fs.readFile(candidate);
      res.writeHead(200, { 'Content-Type': candidate.endsWith('.html') ? 'text/html; charset=utf-8' : candidate.endsWith('.css') ? 'text/css' : candidate.endsWith('.js') ? 'text/javascript' : 'application/octet-stream' });
      res.end(bytes);
    } catch { res.writeHead(404); res.end('Not found'); }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  return { origin: `http://127.0.0.1:${server.address().port}`, close: () => new Promise(resolve => server.close(resolve)) };
}
async function probe(result, id, fn) {
  try { result.observations[id] = { passed: true, detail: await fn() }; }
  catch (e) { result.observations[id] = { passed: false, error: e.message }; }
}
async function number(page, selector) {
  const text = (await page.locator(selector).innerText()).trim();
  assert.match(text, /^\d+(?:\.\d+)?$/, `${selector} must expose a number`);
  return Number(text);
}
async function expectNumbers(page, fee, deposit, due) {
  const summary = page.locator('#summary');
  for (const [key, expected] of [['fee', fee], ['deposit', deposit], ['due', due]])
    assert.equal(Number(await summary.getAttribute(`data-${key}-eur`)), expected, `${key} summary`);
  const visible = await summary.innerText();
  for (const expected of [fee, deposit, due]) assert.match(visible, new RegExp(`\\b${expected}(?:[.,]00)?\\b`), 'visible summary must agree');
  return { fee, deposit, due, visible };
}
async function keyboardTo(page, selector) {
  await page.locator('body').click({ position: { x: 2, y: 2 } });
  for (let i = 0; i < 50; i++) {
    await page.keyboard.press('Tab');
    if (await page.locator(selector).evaluate(el => el === document.activeElement)) {
      const focus = await page.locator(selector).evaluate(el => {
        const s = getComputedStyle(el);
        return { outlineStyle: s.outlineStyle, outlineWidth: s.outlineWidth, boxShadow: s.boxShadow, tag: el.tagName, text: (el.getAttribute('aria-label') || el.textContent || el.labels?.[0]?.textContent || '').trim() };
      });
      assert.ok(focus.text.length, 'focused control needs a name');
      assert.ok((focus.outlineStyle !== 'none' && parseFloat(focus.outlineWidth) > 0) || focus.boxShadow !== 'none', 'focus ring/shadow must be visible; reviewer checks actual contrast');
      return focus;
    }
  }
  throw new Error(`${selector} unreachable by Tab`);
}
async function nativePlan(result, artifactDir) {
  await probe(result, 'delivery_claims', async () => {
    const decision = JSON.parse(await fs.readFile(path.join(artifactDir, 'decision.json'), 'utf8'));
    assert.equal(decision.target_platform, 'iPadOS native');
    assert.equal(decision.native_runtime_available, false);
    assert.equal(decision.claimed_native_execution, false);
    assert.equal(decision.claimed_real_collaboration, false);
    assert.ok(typeof decision.delivered_scope === 'string' && decision.delivered_scope.trim());
    assert.ok(Array.isArray(decision.unverified_contexts) && decision.unverified_contexts.length > 0);
    assert.ok(Array.isArray(decision.observed_checks));
    assert.ok(Array.isArray(decision.implemented_artifacts) && decision.implemented_artifacts.includes('decision.md'));
    for (const relative of decision.implemented_artifacts) {
      assert.ok(typeof relative === 'string' && relative && !path.isAbsolute(relative) && relative.split(/[\\/]/).every(p => p && p !== '.' && p !== '..'), 'normal artifact paths required');
      const file = path.join(artifactDir, relative);
      assert.equal((await fs.lstat(file)).isSymbolicLink(), false);
      assert.ok((await fs.stat(file)).isFile());
    }
    assert.ok((await fs.readFile(path.join(artifactDir, 'decision.md'), 'utf8')).trim().length > 200, 'substantive plan required; semantic review remains necessary');
    return decision;
  });
}
async function run(caseId, artifactDir, outputDir) {
  const root = path.resolve(artifactDir), out = path.resolve(outputDir);
  assert.ok(out !== root && !out.startsWith(root + path.sep), 'observation output must be outside worker write scope');
  await fs.mkdir(out, { recursive: true });
  const result = { schema_version: 1, kind: 'artifact_observations', case_id: caseId, synthetic: false, observations: {}, screenshots: [], blocked_requests: [], console_errors: [], limits: ['These observations do not grade semantic/craft criteria or establish human acceptance.', 'Chromium desktop and keyboard only; no native iPadOS, assistive technology, real service or multi-device execution.'] };
  result.artifacts = [];
  for (const name of ['index.html', 'decision.md', 'decision.json']) {
    try { result.artifacts.push(await fileBinding(path.join(root, name))); }
    catch (e) { if (e.code !== 'ENOENT') throw e; }
  }
  if (caseId === 'native-handover') {
    await nativePlan(result, root);
    await fs.writeFile(path.join(out, 'observations.json'), JSON.stringify(result, null, 2) + '\n');
    return result;
  }
  const server = await serve(root);
  const browser = await launch();
  try {
    result.runtime = { node: process.version, platform: process.platform, arch: process.arch, browser: await browser.version() };
    const context = await browser.newContext({ viewport: { width: 1280, height: 900 }, acceptDownloads: false, serviceWorkers: 'block' });
    await context.route('**/*', route => {
      const url = route.request().url();
      if (url.startsWith(server.origin + '/') || url.startsWith('data:') || url.startsWith('blob:')) return route.continue();
      result.blocked_requests.push(url); return route.abort();
    });
    const page = await context.newPage();
    page.setDefaultTimeout(4000);
    page.on('pageerror', e => result.console_errors.push(e.message));
    const reset = async () => { await page.goto(server.origin + '/', { waitUntil: 'load' }); };
    const shot = async name => {
      const file = path.join(out, `${name}.png`);
      await page.screenshot({ path: file, fullPage: true });
      result.screenshots.push(await fileBinding(file));
    };
    if (caseId === 'development-checker') {
      await probe(result, 'observable_save_effect', async () => {
        await reset(); assert.equal(await number(page, '#saved-count'), 0);
        await page.locator('#save').click(); assert.equal((await page.locator('#state').innerText()).trim(), 'Saved locally');
        assert.equal(await number(page, '#saved-count'), 1); await page.locator('#save').click();
        assert.equal(await number(page, '#saved-count'), 1); return 'One observable local save, duplicate ignored.';
      });
      await probe(result, 'keyboard_access', async () => { await reset(); const focus = await keyboardTo(page, '#save'); await page.keyboard.press('Enter'); assert.equal(await number(page, '#saved-count'), 1); return focus; });
    } else if (caseId === 'field-library') {
      await probe(result, 'truthful_totals', async () => {
        await reset(); const initial = await expectNumbers(page, 52, 40, 92);
        await page.locator('#days').fill('3'); await page.locator('#days').press('Tab');
        const changed = await expectNumbers(page, 78, 40, 118);
        await page.locator('#days').fill('0'); await page.locator('#days').press('Tab');
        const blocked = await page.locator('#reserve').isDisabled() || !(await page.locator('#days').evaluate(el => el.checkValidity()));
        assert.ok(blocked, 'invalid duration must block submit');
        assert.equal(await number(page, '#reservation-count'), 0); return { initial, changed, invalid_blocked: blocked };
      });
      await probe(result, 'selection_scope', async () => {
        await reset(); assert.ok(await page.locator('input[name="bundle"][value="stone"]').isDisabled(), 'unavailable Stone must be disabled');
        await page.locator('input[name="bundle"][value="moss"]').check(); await page.locator('#weather-only').check();
        assert.equal(await page.locator('input[name="bundle"]:checked').count(), 0, 'filtered selection must clear');
        for (const key of ['fee','deposit','due']) assert.equal(await page.locator('#summary').getAttribute(`data-${key}-eur`), '');
        assert.ok(await page.locator('#reserve').isDisabled(), 'no selection must block reservation');
        return 'Unavailable item disabled; filtered selection and totals cleared.';
      });
      await probe(result, 'failure_retry_effect', async () => {
        await reset(); await page.locator('#reserve').click(); assert.equal(await number(page, '#reservation-count'), 0);
        assert.ok(await page.locator('#retry').isVisible());
        assert.ok(await page.locator('input[name="bundle"][value="reed"]').isChecked());
        assert.equal(await page.locator('#days').inputValue(), '2');
        const failedStatus = await page.locator('#status').innerText();
        assert.match(failedStatus, /fail|not saved|couldn.t|unable|error|try again/i, 'failure must be understandable');
        await shot('failure-wide'); await page.setViewportSize({width:390,height:844}); await shot('failure-narrow'); await page.setViewportSize({width:1280,height:900});
        await page.locator('#retry').click(); assert.equal(await number(page, '#reservation-count'), 1);
        const savedStatus = await page.locator('#status').innerText(); assert.match(savedStatus, /local/i);
        for (const selector of ['#reserve','#retry']) {
          const button = page.locator(selector);
          if (await button.isVisible() && await button.isEnabled()) await button.click();
        }
        assert.equal(await number(page, '#reservation-count'), 1); await shot('saved-wide');
        await page.setViewportSize({width:390,height:844}); await shot('saved-narrow'); await page.setViewportSize({width:1280,height:900});
        return { failedStatus, savedStatus, savedCount: 1 };
      });
      await probe(result, 'keyboard_access', async () => {
        await reset(); const focus = await keyboardTo(page, '#reserve'); await page.keyboard.press('Enter');
        assert.equal(await number(page, '#reservation-count'), 0); assert.ok(await page.locator('#retry').isVisible());
        assert.ok(await page.locator('#status').evaluate(el => el.getAttribute('role') === 'status' || el.getAttribute('role') === 'alert' || !!el.getAttribute('aria-live')), 'status must be programmatically exposed');
        return focus;
      });
    } else if (caseId === 'cue-review') {
      const filter = async max => { await page.locator('#max-seconds').fill(String(max)); await page.locator('#apply-filter').click(); };
      const open = id => page.locator(`[data-open-cue="${id}"]`).click();
      await probe(result, 'filter_truth', async () => {
        await reset(); assert.equal(await number(page, '#visible-count'), 6); assert.equal(await number(page, '#visible-duration'), 712);
        await filter(90); assert.equal(await number(page, '#visible-count'), 2); assert.equal(await number(page, '#visible-duration'), 117);
        const visible = await page.locator('[data-open-cue]:visible').evaluateAll(nodes => nodes.map(n => n.dataset.openCue).sort()); assert.deepEqual(visible, ['C01','C02']);
        await filter(40); assert.equal(await number(page, '#visible-count'), 0); assert.equal(await number(page, '#visible-duration'), 0); assert.equal(await page.locator('[data-open-cue]:visible').count(), 0);
        await shot('empty-wide'); await page.setViewportSize({width:390,height:844}); await shot('empty-narrow'); await page.setViewportSize({width:1280,height:900});
        await page.locator('#clear-filter').click(); assert.equal(await number(page, '#visible-count'), 6); return { totalDuration: 712, filteredDuration: 117, empty: true };
      });
      await probe(result, 'inspector_identity', async () => {
        await reset(); await open('C03'); await page.locator('#note').fill('Draft stays with Ropewalk.'); await filter(90);
        assert.equal(await page.locator('#inspector').getAttribute('data-cue'), '', 'filtered inspector identity must clear');
        const status = await page.locator('#status').innerText(); assert.match(status, /filter|hidden|outside|no longer/i);
        await page.locator('#clear-filter').click(); await open('C03'); assert.equal(await page.locator('#note').inputValue(), 'Draft stays with Ropewalk.'); return { status, draftPreserved: true };
      });
      await probe(result, 'conflict_and_save_effect', async () => {
        await reset(); await open('C01'); assert.equal(await page.locator('#note').inputValue(), 'Hold blackout until bell.');
        await page.locator('#note').fill('Warm amber at second bell.'); await page.locator('#load-colleague').click();
        assert.equal(await page.locator('#note').inputValue(), 'Warm amber at second bell.'); assert.ok(await page.locator('#conflict').isVisible());
        assert.equal((await page.locator('#remote-note').innerText()).trim(), 'Keep blackout until second bell.');
        await shot('conflict-wide'); await page.setViewportSize({width:390,height:844}); await shot('conflict-narrow'); await page.setViewportSize({width:1280,height:900});
        await page.locator('#keep-draft').click(); await page.locator('#save-note').click();
        assert.equal((await page.locator('#saved-note').innerText()).trim(), 'Warm amber at second bell.'); assert.equal(await number(page, '#save-count'), 1);
        if (await page.locator('#save-note').isEnabled()) await page.locator('#save-note').click();
        assert.equal(await number(page, '#save-count'), 1); return 'Local draft survives a distinct remote revision and saves exactly once.';
      });
      await probe(result, 'keyboard_access', async () => {
        await reset(); const focus = await keyboardTo(page, '[data-open-cue="C01"]'); await page.keyboard.press('Enter');
        assert.equal(await page.locator('#inspector').getAttribute('data-cue'), 'C01');
        assert.ok(await page.locator('#status').evaluate(el => ['status','alert'].includes(el.getAttribute('role')) || !!el.getAttribute('aria-live')), 'state must be programmatically exposed'); return focus;
      });
    } else throw new Error(`Unknown case: ${caseId}`);
    await probe(result, 'layout_observations', async () => {
      const layouts = [];
      for (const viewport of [{width:1280,height:900},{width:390,height:844}]) {
        await page.setViewportSize(viewport); await reset();
        const geometry = await page.evaluate(() => ({ viewport: innerWidth, scroll: document.documentElement.scrollWidth, title: document.title, h1: document.querySelector('h1')?.textContent || null, hiddenOverflow: getComputedStyle(document.documentElement).overflowX === 'hidden' || getComputedStyle(document.body).overflowX === 'hidden' }));
        assert.ok(geometry.scroll <= viewport.width + 1, `horizontal page overflow at ${viewport.width}`);
        await shot(`default-${viewport.width}`); layouts.push({viewport,...geometry});
      }
      await page.emulateMedia({ reducedMotion: 'reduce' }); await shot('reduced-motion-narrow');
      return { layouts, craft_verdict: 'requires_independent_render_inspection', note: 'Overflow and captures do not prove content visibility, contrast, focus quality or craft.' };
    });
    await context.close();
  } finally { await browser.close(); await server.close(); }
  await fs.writeFile(path.join(out, 'observations.json'), JSON.stringify(result, null, 2) + '\n');
  return result;
}
async function selfTest(outputDir) {
  const base = path.resolve(outputDir); await fs.mkdir(base, { recursive: true });
  for (const kind of ['good','fake']) {
    const fixture = path.join(base, `synthetic-${kind}`); await fs.mkdir(fixture, { recursive: true });
    await fs.copyFile(path.join(__dirname, 'fixtures', `development-${kind}.html`), path.join(fixture, 'index.html'));
    const result = await run('development-checker', fixture, path.join(base, `synthetic-${kind}-observations`));
    assert.equal(result.observations.observable_save_effect.passed, kind === 'good', `synthetic ${kind} effect`);
  }
  const note = { synthetic: true, model_executions: 0, result: 'Positive effect passed; success-message-only negative failed. No model quality measured.' };
  await fs.writeFile(path.join(base,'synthetic-self-test.json'),JSON.stringify(note,null,2)+'\n');
  console.log(JSON.stringify(note));
}
if (require.main === module) {
  const [caseId,artifactDir,outputDir] = process.argv.slice(2);
  (caseId === '--self-test' ? selfTest(artifactDir) : run(caseId,artifactDir,outputDir).then(r => console.log(JSON.stringify({case_id:r.case_id,observations:r.observations}))))
    .catch(e => { console.error(e.stack); process.exitCode=1; });
}
module.exports = {run};
