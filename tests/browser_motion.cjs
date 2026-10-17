/* Optional rendered acceptance probe. Uses an managed Browserless;
 * never installs a browser or package. Serve an installed advanced-motion/assets
 * directory first. Usage: node tests/browser_motion.cjs BASE_URL OUTPUT_DIRECTORY
 * This captures evidence for visual review; a pass is not aesthetic acceptance. */
const { connect } = require('./browserless.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const { createHash } = require('node:crypto');

const base = process.argv[2];
const output = path.resolve(process.argv[3] || '.mirket/evidence/motion-browser');
const selection = process.argv[4]; // Optional scenario prefix for a focused rerun.
if (!base || !/^http:\/\/(127\.0\.0\.1|localhost):\d+\//.test(base)) {
  throw new Error('Provide a loopback HTTP URL ending in / for installed example assets.');
}
const result = { browser:'', scope:'Local Chromium, recorded natural playback; frame captures and assertions need visual review.', scenarios:[] };
const fingerprint = value => createHash('sha256').update(value).digest('hex');
const pause = (page, ms) => page.waitForTimeout(ms);
async function snapshot(page, name, fullPage = false) {
  const before = await page.evaluate(() => performance.now());
  const file = `${name}.png`;
  await page.screenshot({ path:path.join(output,file), fullPage });
  const after = await page.evaluate(() => performance.now());
  return { file, before_ms:before, after_ms:after };
}
async function scroll(page, y) {
  await page.evaluate(value => scrollTo({top:value, behavior:'instant'}), y);
  await pause(page,100);
}
async function dimensions(page) {
  const size = await page.evaluate(() => ({ viewport:innerWidth, content:document.documentElement.scrollWidth }));
  assert.ok(size.content <= size.viewport + 1, `horizontal overflow: ${JSON.stringify(size)}`);
  return size;
}
async function solarState(page) {
  return page.evaluate(() => ({
    phase:document.body.dataset.phase,
    title:document.querySelector('#observation-title').textContent,
    matrix:getComputedStyle(document.querySelector('[data-solar-motion]')).transform,
    selected:[...document.querySelectorAll('.phase-picker button[aria-pressed="true"]')].map(b => b.dataset.phase)
  }));
}
async function selectSolar(page, phase) {
  await page.locator(`.phase-picker button[data-phase="${phase}"]`).dispatchEvent('click');
}
async function selectSignal(page, mode) {
  await page.locator(`button[data-mode="${mode}"]`).dispatchEvent('click');
}
async function probeInstrumentation(context) {
  await context.addInitScript(() => {
    const original = window.requestAnimationFrame.bind(window);
    // The probe is injected into this test context only, never into shipped UI.
    window.__motionProbe = { callbacks:0, timestamps:[], preferenceChanges:[] };
    matchMedia('(prefers-reduced-motion: reduce)').addEventListener('change', event => {
      window.__motionProbe.preferenceChanges.push({reduced:event.matches, time:performance.now()});
    });
    window.requestAnimationFrame = callback => original(time => {
      window.__motionProbe.callbacks++;
      if (window.__motionProbe.timestamps.length < 600) window.__motionProbe.timestamps.push(time);
      callback(time);
    });
  });
}
async function runScene(browser, scene, viewport, reduced) {
  const tag = `${scene}-${viewport.width}-${reduced ? 'reduced' : 'normal'}`;
  const context = await browser.newContext({viewport, deviceScaleFactor:2, reducedMotion:reduced ? 'reduce' : 'no-preference', recordVideo:{dir:path.join(output,'video'),size:viewport}});
  await probeInstrumentation(context);
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', error => errors.push(String(error)));
  const row = {tag,frames:[],checks:[],errors};
  await page.goto(`${base}${scene}/`, {waitUntil:'load'});
  await page.locator('h1').waitFor();
  // Ordinary playback progresses on the browser clock; no animation seeking.
  for (const [label, wait] of [['early',100],['overlap',240],['settled',1150]]) {
    await pause(page,wait);
    row.frames.push(await snapshot(page,`${tag}-${label}`));
  }
  row.dimensions = await dimensions(page);
  assert.equal(await page.locator('h1').evaluate(node => getComputedStyle(node).opacity), '1');
  row.frames.push(await snapshot(page,`${tag}-full`,true));
  row.checks.push('entrance settles; no horizontal overflow');
  await page.locator(scene === 'solstice' ? '.hero .text-link' : '.hero .button').hover();
  await pause(page,250);
  row.frames.push(await snapshot(page,`${tag}-hover`));

  if (scene === 'solstice') {
    await page.locator('.phase-picker').scrollIntoViewIfNeeded();
    await selectSolar(page,'noon');
    row.frames.push(await snapshot(page,`${tag}-sun-start`));
    await pause(page,270);
    row.frames.push(await snapshot(page,`${tag}-sun-middle`));
    await pause(page,650);
    const noon = await solarState(page);
    assert.equal(noon.title,'The clear hour.');
    assert.match(noon.matrix, /27, -63\)/);
    await selectSolar(page,'dusk'); await pause(page,70);
    await selectSolar(page,'noon'); await pause(page,55);
    await selectSolar(page,'dawn'); await pause(page,950);
    const end = await solarState(page);
    assert.deepEqual(end.selected,['dawn']);
    assert.equal(end.phase,'dawn'); assert.equal(end.title,'The first light.');
    assert.match(end.matrix, /-35, 32\)/);
    row.frames.push(await snapshot(page,`${tag}-sun-final`));
    row.checks.push('real SVG translation and copy commit; rapid reversal settles latest state');
    await page.locator('.phase-picker button[data-phase="dusk"]').focus();
    await page.keyboard.press('Enter');
    assert.equal((await solarState(page)).phase,'dusk');
    row.frames.push(await snapshot(page,`${tag}-focus`));
    const studyY = await page.locator('.study-window').evaluate(n => n.getBoundingClientRect().top + scrollY - innerHeight * .35);
    await scroll(page,studyY);
    const initial = await page.locator('.study-orb').evaluate(n => getComputedStyle(n).transform);
    await scroll(page,studyY + 130);
    const forward = await page.locator('.study-orb').evaluate(n => getComputedStyle(n).transform);
    await scroll(page,studyY);
    const reverse = await page.locator('.study-orb').evaluate(n => getComputedStyle(n).transform);
    assert.equal(initial,reverse,'scroll reversal must recover the same pose');
    if (!reduced) assert.notEqual(initial,forward,'normal scroll must visibly change the art');
    else assert.equal(initial,'none');
    row.frames.push(await snapshot(page,`${tag}-scroll`));
    await page.locator('.note summary').first().click();
    assert.equal(await page.locator('.note details').first().getAttribute('open'),'');
    row.checks.push('keyboard control, native disclosure, reversible scroll and static reduced composition');
  } else {
    assert.equal(await page.locator('.field').getAttribute('data-canvas'),'ready');
    const initialHash = fingerprint(await page.locator('canvas').evaluate(n => n.toDataURL()));
    row.initial_canvas_hash = initialHash;
    await scroll(page,170);
    const scrollHash = fingerprint(await page.locator('canvas').evaluate(n => n.toDataURL()));
    await scroll(page,0);
    const reverseHash = fingerprint(await page.locator('canvas').evaluate(n => n.toDataURL()));
    assert.equal(initialHash,reverseHash,'scroll reversal must recover identical Canvas pixels');
    if (!reduced) assert.notEqual(initialHash,scrollHash);
    else assert.equal(initialHash,scrollHash);
    await page.locator('.choices').scrollIntoViewIfNeeded();
    await selectSignal(page,'explore');
    await pause(page,130);
    row.frames.push(await snapshot(page,`${tag}-gradient-middle`));
    await pause(page,450);
    row.frames.push(await snapshot(page,`${tag}-gradient-settled`));
    await selectSignal(page,'focus'); await pause(page,40);
    await selectSignal(page,'explore'); await pause(page,40);
    await selectSignal(page,'focus'); await pause(page,600);
    assert.equal(await page.locator('.session-card').getAttribute('data-mode'),'focus');
    assert.match(await page.locator('#mode-title').textContent(),/One thing/);
    assert.equal(await page.locator('button[aria-pressed="true"]').count(),1);
    await page.locator('button[data-mode="explore"]').focus();
    await page.keyboard.press('Enter');
    assert.match(await page.locator('#mode-title').textContent(),/Follow a spark/);
    row.frames.push(await snapshot(page,`${tag}-focus`));
    row.checks.push('real Canvas pixels and reversible scroll; gradient/card state survives rapid input; keyboard choice');
    await scroll(page,0); await pause(page,1650);
    const before = await page.evaluate(() => window.__motionProbe.callbacks);
    await pause(page,350);
    const after = await page.evaluate(() => window.__motionProbe.callbacks);
    assert.equal(before,after,'settled Canvas must not keep requesting frames');
    row.canvas_idle_callbacks = after - before;
  }
  await dimensions(page);
  // Interrupt a real effect with a live preference change, not just a fresh load.
  if (!reduced) {
    if (scene === 'solstice') { await page.locator('.phase-picker').scrollIntoViewIfNeeded(); await selectSolar(page,'noon'); }
    else { await page.locator('.choices').scrollIntoViewIfNeeded(); await selectSignal(page,'focus'); }
    await pause(page,60);
    await page.emulateMedia({reducedMotion:'reduce'});
    await pause(page,100);
    const running = await page.evaluate(() => document.getAnimations().filter(a => a.playState === 'running').map(a => ({type:a.constructor.name, target:a.effect?.target?.className, property:a.transitionProperty, time:a.currentTime})));
    const preference = await page.evaluate(() => ({matches:matchMedia('(prefers-reduced-motion: reduce)').matches, events:window.__motionProbe.preferenceChanges}));
    assert.deepEqual(running,[],`running after reduced motion: ${JSON.stringify({running,preference})}`);
    row.frames.push(await snapshot(page,`${tag}-live-reduced`));
    row.checks.push('live reduced-motion interruption cancels all running decoration');
    if (scene === 'signal') {
      await scroll(page,0);
      row.live_reduced_canvas_hash = fingerprint(await page.locator('canvas').evaluate(n => n.toDataURL()));
      assert.notEqual(row.initial_canvas_hash,row.live_reduced_canvas_hash,'live reduced preference must change the drawn composition');
    }
    await page.emulateMedia({reducedMotion:'no-preference'});
    await pause(page,100);
  }
  await page.setViewportSize({width:320,height:740});
  await pause(page,180);
  await dimensions(page);
  row.frames.push(await snapshot(page,`${tag}-320`));
  // Explicit lifecycle events are a disposal/remount probe, not proof of native bfcache.
  await page.evaluate(() => dispatchEvent(new PageTransitionEvent('pagehide',{persisted:true})));
  await pause(page,60);
  const suspended = await page.evaluate(() => window.__motionProbe.callbacks);
  await pause(page,150);
  assert.equal(suspended,await page.evaluate(() => window.__motionProbe.callbacks));
  await page.evaluate(() => dispatchEvent(new PageTransitionEvent('pageshow',{persisted:true})));
  if (scene === 'solstice') { await selectSolar(page,'dusk'); assert.equal((await solarState(page)).phase,'dusk'); }
  else { await selectSignal(page,'explore'); assert.match(await page.locator('#mode-title').textContent(),/Follow a spark/); }
  row.checks.push('320px reflow, explicit disposal/remount events, no script errors');
  assert.deepEqual(errors,[]);
  row.rAF = await page.evaluate(() => window.__motionProbe);
  const video = page.video();
  await context.close();
  const videoFile = path.join(output,'video',`${tag}.webm`);
  await video.saveAs(videoFile); await video.delete();
  row.video = path.relative(output,videoFile);
  result.scenarios.push(row);
  console.log(`PASS ${tag}`);
}
async function fallbacks(browser, scene) {
  const context = await browser.newContext({viewport:{width:390,height:844},javaScriptEnabled:false});
  const page = await context.newPage();
  await page.goto(`${base}${scene}/`);
  assert.ok(await page.locator('h1').isVisible()); await dimensions(page);
  assert.ok(await page.locator(scene === 'solstice' ? '.solar-instrument' : '.field-fallback').isVisible());
  assert.equal(await page.locator(scene === 'solstice' ? '.phase-picker' : '.choices').isVisible(),false);
  const frame = await snapshot(page,`${scene}-no-script`,true);
  result.scenarios.push({tag:`${scene}-no-script`,frames:[frame],checks:['readable static art/content; inactive controls hidden']});
  await context.close();
}
async function failureProbe(browser) {
  const context = await browser.newContext();
  const page = await context.newPage();
  await page.route('**/motion.js', route => route.abort());
  const response = await page.goto(`${base}solstice/`);
  assert.equal(response.status(),200); assert.ok(await page.locator('h1').isVisible());
  // Even directly dispatching to a hidden control cannot fake the expected effect.
  await selectSolar(page,'noon');
  const state = await solarState(page);
  assert.notEqual(state.phase,'noon'); assert.notEqual(state.title,'The clear hour.');
  result.scenarios.push({tag:'blocked-controller-failure-probe',checks:['HTTP 200 and a rendered page are insufficient: missing controller prevents the required state/art change'],state});
  await context.close();
  const canvasContext = await browser.newContext();
  await canvasContext.addInitScript(() => { HTMLCanvasElement.prototype.getContext = () => null; });
  const canvasPage = await canvasContext.newPage();
  await canvasPage.goto(`${base}signal/`); await pause(canvasPage,1500);
  assert.ok(await canvasPage.locator('.field-fallback').isVisible());
  await selectSignal(canvasPage,'explore');
  assert.match(await canvasPage.locator('#mode-title').textContent(),/Follow a spark/);
  result.scenarios.push({tag:'canvas-unavailable',checks:['SVG poster remains visible; DOM controls still work'],frames:[await snapshot(canvasPage,'signal-no-canvas')]});
  await canvasContext.close();
}
(async () => {
  await fs.mkdir(output,{recursive:true});
  const browser = await connect();
  result.browser = browser.version(); result.platform = `${process.platform}/${process.arch}`;
  try {
    if (selection === 'craft') {
      await require('./browser_design_craft.cjs')({browser,base,output,result,snapshot});
    }
    for (const scene of ['solstice','signal']) {
      for (const viewport of [{width:1440,height:1000},{width:390,height:844}]) {
        for (const reduced of [false,true]) {
          const tag = `${scene}-${viewport.width}-${reduced ? 'reduced' : 'normal'}`;
          if (!selection || tag.startsWith(selection)) await runScene(browser,scene,viewport,reduced);
        }
      }
      if (!selection) await fallbacks(browser,scene);
    }
    if (!selection) await failureProbe(browser);
    assert.ok(result.scenarios.length > 0,'scenario selection matched no checks');
    for (const normal of result.scenarios.filter(s => s.live_reduced_canvas_hash)) {
      const reduced = result.scenarios.find(s => s.tag === normal.tag.replace(/normal$/, 'reduced'));
      if (reduced) assert.equal(normal.live_reduced_canvas_hash,reduced.initial_canvas_hash,'live and initial reduced preference must choose identical Canvas pixels');
    }
    result.status = 'passed';
  } catch (error) {
    result.status = 'failed'; result.error = String(error.stack || error); process.exitCode = 1;
    console.error(error);
  } finally {
    await browser.close();
    await fs.writeFile(path.join(output,'results.json'),JSON.stringify(result,null,2)+'\n');
  }
})();
