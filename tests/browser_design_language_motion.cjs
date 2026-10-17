/* Behavioral acceptance for the four original design-language motion studies.
 * Use an managed Browserless and installed playwright-core client; never installs either.
 *
 * node tests/browser_design_language_motion.cjs LOOPBACK_BASE OUTPUT_DIR [ID]
 * BASE serves design-inspiration/assets/design-languages/ and ends with /.
 * BROWSERLESS_WS selects the managed Browserless WebSocket endpoint; the helper uses native Playwright.
 * CAPTURE_FRAMES=0 skips PNGs; CAPTURE_VIDEO=1 records wide normal
 * purposeful runs. Neither screenshots nor a passing probe are visual approval.
 */
const { connect } = require('./browserless.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');

const base = process.argv[2];
const outputArgument = process.argv[3];
const selection = process.argv[4];
const languages = [
  'material-responsive-shape',
  'fluent-spatial',
  'apple-liquid-glass',
  'duolingo-performance'
];
if (!base || (!/^http:\/\/(127\.0\.0\.1|localhost):\d+\/[^?#]*$/.test(base) || !base.endsWith('/'))) {
  throw new Error('Provide a loopback HTTP base URL ending in / for the design-language assets.');
}
if (!outputArgument) throw new Error('Provide an output directory for the JSON result and optional media.');
if (selection && !languages.includes(selection)) throw new Error(`Unknown motion study: ${selection}`);
const output = path.resolve(outputArgument);
const captureFrames = process.env.CAPTURE_FRAMES !== '0';
const captureVideo = process.env.CAPTURE_VIDEO === '1';
const result = {
  scope: 'Local Chromium, original web teaching studies. Ordinary playback is not sought or accelerated. Geometry, meaning and state are asserted; captured frames still require visual inspection. No native material parity, real device or assistive-technology claim.',
  scenarios: []
};
const pause = (page, ms) => page.waitForTimeout(ms);

async function snapshot(page, row, label, fullPage = false) {
  if (!captureFrames) return;
  const file = `${row.tag}-${label}.png`;
  const before = await page.evaluate(() => performance.now());
  await page.screenshot({ path: path.join(output, file), fullPage });
  const after = await page.evaluate(() => performance.now());
  row.frames.push({ file, before_ms: before, after_ms: after });
}

async function dimensions(page) {
  const size = await page.evaluate(() => ({ viewport: innerWidth, content: document.documentElement.scrollWidth }));
  assert.ok(size.content <= size.viewport + 1, `Horizontal overflow: ${JSON.stringify(size)}`);
  return size;
}

async function runningAnimations(page) {
  return page.evaluate(() => document.getAnimations()
    .filter(animation => animation.playState === 'running')
    .map(animation => ({ target: animation.effect?.target?.id, property: animation.transitionProperty })));
}

async function instrumentation(context) {
  await context.addInitScript(() => {
    const original = window.requestAnimationFrame.bind(window);
    window.__motionProbe = { callbacks: 0, preferenceChanges: [] };
    matchMedia('(prefers-reduced-motion: reduce)').addEventListener('change', event => {
      window.__motionProbe.preferenceChanges.push({ reduced: event.matches, time: performance.now() });
    });
    window.requestAnimationFrame = callback => original(time => {
      window.__motionProbe.callbacks++;
      callback(time);
    });
  });
}

async function keyboardActivate(page, selector) {
  const control = page.locator(selector);
  await control.focus();
  await page.keyboard.press('Enter');
  assert.notEqual(await control.evaluate(node => getComputedStyle(node).outlineStyle), 'none', 'Keyboard focus must be visible');
}

async function material(page, row) {
  const initialTransform = await page.locator('#shape').evaluate(node => getComputedStyle(node).transform);
  await keyboardActivate(page, '#sound-toggle');
  await pause(page, 90);
  const moving = await page.evaluate(() => {
    const marker = document.querySelector('#meter-marker');
    const track = document.querySelector('.meter-line').getBoundingClientRect();
    return {
      spatial: Number(document.querySelector('#shape').dataset.spatial),
      transform: getComputedStyle(document.querySelector('#shape')).transform,
      reading: Number(marker.dataset.visualReading),
      marker_x: marker.getBoundingClientRect().x,
      track_x: track.x,
      track_width: track.width
    };
  });
  assert.equal(await page.locator('#sound-toggle').getAttribute('aria-pressed'), 'true');
  assert.notEqual(moving.transform, initialTransform, 'The actual control geometry must change');
  assert.ok(Math.abs(moving.marker_x - (moving.track_x + moving.track_width * moving.reading / 100)) < 1, 'The measured marker position must match its exposed reading');
  if (row.variant === 'purposeful' || row.reduced) assert.equal(moving.reading, 42);
  else assert.notEqual(moving.reading, 42, 'The misleading counterexample must actually invent a reading during normal playback');
  row.first_motion = moving;
  await snapshot(page, row, '90ms');
  await pause(page, 240);
  await snapshot(page, row, '330ms');
  await pause(page, 1100);
  assert.equal(await page.locator('#shape').getAttribute('data-spatial'), '1.0000');
  await page.locator('#sound-toggle').click();
  await pause(page, 45);
  await page.locator('#sound-toggle').click();
  await pause(page, 35);
  await page.locator('#sound-toggle').click();
  await pause(page, 1300);
  assert.equal(await page.locator('#sound-toggle').getAttribute('aria-pressed'), 'false');
  assert.equal(await page.locator('#shape').getAttribute('data-spatial'), '0.0000');
  assert.equal(Number(await page.locator('#meter-marker').getAttribute('data-visual-reading')), 42);
  assert.equal(await page.locator('#room-status').textContent(), 'Sound cushion is off.');
  row.checks.push('Actual spatial geometry changes; the purposeful calibrated marker stays at 42; rapid reversal settles on the latest off state');
}

async function fluent(page, row) {
  const before = await page.locator('#inspector').boundingBox();
  await keyboardActivate(page, '#expand');
  assert.equal(await page.locator('#expand').getAttribute('aria-expanded'), 'true');
  await pause(page, 100);
  const middle = await page.locator('#inspector').boundingBox();
  await snapshot(page, row, 'expand-100ms');
  await pause(page, 400);
  const after = await page.locator('#inspector').boundingBox();
  assert.ok(after.width > before.width + 20, 'The actual note container must grow');
  if (!row.reduced) assert.ok(middle.width > before.width && middle.width <= after.width + 1, 'Normal playback must expose an intermediate container width');
  assert.equal(await page.locator('#inspector').getAttribute('data-object-id'), 'FN-028');
  assert.match(await page.locator('#note-title').textContent(), /Willow at/);
  assert.ok(await page.locator('#note-detail').isVisible());
  await page.locator('button[data-destination="collections"]').click();
  await pause(page, 90);
  await snapshot(page, row, 'peer-90ms');
  assert.equal(await page.locator('#workspace').getAttribute('data-hierarchy'), row.variant === 'purposeful' ? 'peer' : 'child');
  assert.equal(await page.locator('#workspace-crumb').textContent(), row.variant === 'purposeful' ? 'Workspace / Collections' : 'Notes / Collections');
  await page.locator('button[data-destination="notes"]').click();
  await pause(page, 25);
  await page.locator('button[data-destination="collections"]').click();
  await pause(page, 450);
  assert.equal(await page.locator('#destination-title').textContent(), 'River collection');
  assert.equal(await page.locator('.workspace-nav button[aria-pressed="true"]').count(), 1);
  await page.locator('#expand').click();
  await pause(page, 50);
  await page.locator('#expand').click();
  await pause(page, 450);
  assert.equal(await page.locator('#expand').getAttribute('aria-expanded'), 'true');
  row.container_geometry = { before, middle, after };
  row.checks.push('FN-028 grows as the same container; breadcrumb detects false peer hierarchy; rapid navigation and size reversal retain the latest state');
}

async function apple(page, row) {
  const before = await page.locator('#article').boundingBox();
  const dockBefore = await page.locator('#control-dock').boundingBox();
  await keyboardActivate(page, '#view-tools');
  await pause(page, 95);
  await snapshot(page, row, 'open-95ms');
  await pause(page, 400);
  assert.equal(await page.locator('#view-tools').getAttribute('aria-expanded'), 'true');
  assert.equal(await page.locator('#article').getAttribute('data-layer'), row.variant === 'purposeful' ? 'content' : 'transient');
  const after = await page.locator('#article').boundingBox();
  const dockAfter = await page.locator('#control-dock').boundingBox();
  assert.ok(dockAfter.height > dockBefore.height + 20, 'The actual control group must acquire room');
  if (row.variant === 'purposeful') assert.deepEqual(after, before, 'Reading content must keep its bounds');
  else assert.notEqual(after.y, before.y, 'The misleading route must actually move content into a floating surface');
  await page.locator('#labels').click();
  assert.equal(await page.locator('#specimen').getAttribute('data-labels'), 'true');
  assert.ok(await page.locator('.map-label').isVisible());
  await page.locator('#save').click();
  assert.equal(await page.locator('#save').getAttribute('aria-pressed'), 'true');
  await page.locator('#view-tools').click();
  await pause(page, 35);
  await page.locator('#view-tools').click();
  await pause(page, 400);
  await page.locator('#close-tools').focus();
  await page.keyboard.press('Escape');
  assert.equal(await page.locator('#view-tools').getAttribute('aria-expanded'), 'false');
  assert.equal(await page.evaluate(() => document.activeElement.id), 'view-tools');
  assert.equal(await page.locator('#toolset').isVisible(), false);
  assert.equal(await page.locator('#save').getAttribute('aria-pressed'), 'true');
  row.layer_geometry = { before, after, dockBefore, dockAfter };
  row.checks.push('The control layer expands; purposeful article bounds remain fixed; labels/save work; rapid reopening and Escape preserve state and restore focus');
}

async function character(page, row) {
  await keyboardActivate(page, '[data-answer="incorrect"]');
  await pause(page, 100);
  assert.equal(await page.locator('#specimen').getAttribute('data-count'), '0');
  assert.equal(await page.locator('#character-stage').getAttribute('data-reaction'), row.variant === 'purposeful' ? 'think' : 'celebrate');
  await snapshot(page, row, 'incorrect-100ms');
  const before = await page.locator('#actor').evaluate(node => getComputedStyle(node).transform);
  await page.locator('[data-answer="correct"]').click();
  await pause(page, 190);
  const during = await page.locator('#actor').evaluate(node => getComputedStyle(node).transform);
  if (!row.reduced) {
    assert.notEqual(during, before, 'The actual character group must move');
    assert.ok((await runningAnimations(page)).length > 0, 'Normal character playback must be in flight');
  } else assert.deepEqual(await runningAnimations(page), []);
  await snapshot(page, row, 'correct-190ms');
  await page.locator('[data-answer="incorrect"]').click();
  await pause(page, 40);
  await page.locator('[data-answer="correct"]').click();
  await pause(page, 65);
  await page.locator('[data-answer="incorrect"]').click();
  await pause(page, 1150);
  assert.equal(await page.locator('#specimen').getAttribute('data-event'), 'incorrect');
  assert.equal(await page.locator('#specimen').getAttribute('data-count'), '0');
  assert.match(await page.locator('#answer-status').textContent(), /^Not yet/);
  assert.equal(await page.locator('body').getAttribute('data-phase'), 'rest');
  assert.equal(await page.locator('#character-stage').getAttribute('data-reaction'), row.variant === 'purposeful' ? 'think' : 'celebrate');
  row.character_geometry = { before, during };
  row.checks.push('Actual character groups perform; error/count remain truthful; misleading error celebrates; interrupted acting rests on the latest event');
}

// Each call starts a real new transition from the current settled state.
async function triggerMotion(page, id) {
  if (id === 'material-responsive-shape') await page.locator('#sound-toggle').click();
  if (id === 'fluent-spatial') {
    const destination = await page.locator('#workspace').getAttribute('data-destination');
    await page.locator(`button[data-destination="${destination === 'notes' ? 'collections' : 'notes'}"]`).click();
  }
  if (id === 'apple-liquid-glass') await page.locator('#view-tools').click();
  if (id === 'duolingo-performance') await page.locator('#replay').click();
}

async function preferenceChange(page, row) {
  await triggerMotion(page, row.id);
  await pause(page, 45);
  const phase = await page.locator('body').getAttribute('data-phase');
  assert.ok(phase === 'moving' || phase === 'performing', 'The live-change probe must interrupt active motion');
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.waitForFunction(() => document.body.dataset.reduced === 'true');
  await pause(page, 80);
  assert.deepEqual(await runningAnimations(page), [], 'Reduced motion must stop the actual effects');
  assert.equal(await page.locator('body').getAttribute('data-phase'), 'rest');
  await snapshot(page, row, 'live-reduced');
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  await page.waitForFunction(() => document.body.dataset.reduced === 'false');
  await pause(page, 80);
  assert.deepEqual(await runningAnimations(page), [], 'Restoring preference must not replay obsolete motion');
  await triggerMotion(page, row.id);
  await pause(page, 45);
  const restored = await page.locator('body').getAttribute('data-phase');
  assert.ok(restored === 'moving' || restored === 'performing', 'A new action must animate after preference restore');
  if (row.id !== 'material-responsive-shape') assert.ok((await runningAnimations(page)).length > 0);
  await pause(page, 1350);
  assert.equal(await page.locator('body').getAttribute('data-phase'), 'rest');
  row.checks.push('An actual in-flight effect stops on live reduce; restore does not replay; the next action animates and rests normally');
}

async function runScene(browser, id, variant, reduced) {
  const tag = `${id}-${variant}-${reduced ? 'reduced' : 'normal'}`;
  const context = await browser.newContext({
    viewport: { width: 1280, height: 1000 },
    reducedMotion: reduced ? 'reduce' : 'no-preference',
    ...(captureVideo && variant === 'purposeful' && !reduced ? {
      recordVideo: { dir: path.join(output, 'video'), size: { width: 1280, height: 1000 } }
    } : {})
  });
  await instrumentation(context);
  const page = await context.newPage();
  const row = { id, tag, variant, reduced, frames: [], checks: [], errors: [] };
  result.scenarios.push(row);
  page.on('pageerror', error => row.errors.push(String(error)));
  try {
    await page.goto(`${base}${id}/?variant=${variant}`, { waitUntil: 'load' });
    assert.equal(await page.locator('html').getAttribute('lang'), 'en');
    assert.equal(await page.locator('html').getAttribute('data-design-language'), id);
    assert.equal(await page.locator('main aside').count(), 0, 'Teaching chrome must sit outside the specimen');
    assert.equal(await page.locator('body').getAttribute('data-reduced'), String(reduced));
    await dimensions(page);
    await snapshot(page, row, 'initial');
    if (id === 'material-responsive-shape') await material(page, row);
    if (id === 'fluent-spatial') await fluent(page, row);
    if (id === 'apple-liquid-glass') await apple(page, row);
    if (id === 'duolingo-performance') await character(page, row);
    await snapshot(page, row, 'settled', true);
    if (!reduced) await preferenceChange(page, row);
    await pause(page, 120);
    const framesBefore = await page.evaluate(() => window.__motionProbe.callbacks);
    await pause(page, 200);
    assert.equal(await page.evaluate(() => window.__motionProbe.callbacks), framesBefore, 'Settled art must stop requesting frames');
    assert.deepEqual(await runningAnimations(page), []);
    row.checks.push('Finite playback reaches rest without a continuing animation-frame loop');
    await page.setViewportSize({ width: 320, height: 740 });
    await pause(page, 120);
    row.narrow = await dimensions(page);
    await triggerMotion(page, id);
    await pause(page, 90);
    await snapshot(page, row, '320-playback', true);
    await pause(page, 1350);
    await dimensions(page);
    assert.deepEqual(await runningAnimations(page), []);
    await snapshot(page, row, '320-rest', true);
    row.checks.push('320px reflow and a narrow replay retain complete content and settle');
    row.motion_probe = await page.evaluate(() => window.__motionProbe);
    assert.deepEqual(row.errors, []);
    row.status = 'passed';
    console.log(`PASS ${tag}`);
  } catch (error) {
    row.status = 'failed';
    row.error = String(error.stack || error);
    throw error;
  } finally {
    const video = page.video();
    await context.close();
    if (video) {
      const file = path.join(output, 'video', `${tag}.webm`);
      await video.saveAs(file);
      await video.delete();
      row.video = path.relative(output, file);
    }
  }
}

async function fallback(browser, id) {
  const context = await browser.newContext({ viewport: { width: 320, height: 740 }, javaScriptEnabled: false });
  const row = { id, tag: `${id}-nojs`, frames: [], checks: [] };
  result.scenarios.push(row);
  try {
    const page = await context.newPage();
    await page.goto(`${base}${id}/`, { waitUntil: 'load' });
    assert.ok(await page.locator('h1').isVisible());
    await dimensions(page);
    assert.equal(await page.locator('main button:visible').count(), 0, 'Inactive controls must not pretend to work');
    assert.ok(await page.locator('main noscript').isVisible(), 'A readable static explanation must remain');
    await snapshot(page, row, 'static', true);
    row.checks.push('The question/content and static artwork remain; inactive controls are hidden; no 320px overflow');
    row.status = 'passed';
  } catch (error) {
    row.status = 'failed';
    row.error = String(error.stack || error);
    throw error;
  } finally {
    await context.close();
  }
}

(async () => {
  await fs.mkdir(output, { recursive: true });
  const browser = await connect();
  result.browser = browser.version();
  result.platform = `${process.platform}/${process.arch}`;
  try {
    for (const id of languages.filter(id => !selection || id === selection)) {
      for (const variant of ['purposeful', 'misleading']) {
        for (const reduced of [false, true]) await runScene(browser, id, variant, reduced);
      }
      await fallback(browser, id);
    }
    result.status = 'passed';
  } catch (error) {
    result.status = 'failed';
    result.error = String(error.stack || error);
    process.exitCode = 1;
    console.error(error);
  } finally {
    await browser.close();
    await fs.writeFile(path.join(output, 'results.json'), `${JSON.stringify(result, null, 2)}\n`);
  }
})();
