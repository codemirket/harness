/* Additional scenarios for browser_motion.cjs. BASE_URL serves installed skills.
 * Uses the existing browser harness; no runtime or dependency installation. */
const assert = require('node:assert/strict');
const path = require('node:path');

module.exports = async function craft({browser, base, output, result, snapshot}) {
  const timing = `${base}motion-design/assets/timing-specimen.html`;
  const character = `${base}graphic-design/assets/procedural-character/`;
  for (const width of [1000, 390]) {
    const context = await browser.newContext({viewport:{width,height:900}, reducedMotion:'no-preference',
      recordVideo:{dir:path.join(output,'video'),size:{width,height:900}}});
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    try {
      await page.goto(timing);
      const row = {tag:`craft-${width}`,checks:[],frames:[]};
      // Observe ordinary unpaused playback. This also rejects the upstream
      // mislabeled 1650ms motion; it is not a rendering performance benchmark.
      row.timing = await page.evaluate(async () => {
        const root = document.querySelector('[data-motion-specimen]');
        root.motionSpecimen.replay();
        const animation = root.querySelector('[data-marker]').getAnimations()[0];
        const duration = animation.effect.getTiming().duration;
        const rate = animation.playbackRate;
        const start = performance.now();
        const frames = [];
        while (animation.playState !== 'finished' && animation.playState !== 'idle') {
          frames.push({elapsed:performance.now()-start,currentTime:animation.currentTime,
            transform:getComputedStyle(root.querySelector('[data-marker]')).transform});
          await new Promise(requestAnimationFrame);
        }
        return {duration,rate,elapsed:performance.now()-start,frames};
      });
      assert.equal(row.timing.duration,180);
      assert.equal(row.timing.rate,1);
      assert.ok(row.timing.frames.some(f => f.currentTime > 0 && f.currentTime < 180));
      assert.ok(row.timing.elapsed < 1000,'180ms specimen must not be a multi-second loop');
      await page.selectOption('[data-rate]','0.25');
      assert.match(await page.locator('[data-timing]').textContent(),/720ms/);
      await page.locator('[data-replay]').focus();
      await page.keyboard.press('Enter');
      await page.keyboard.press('Tab');
      await page.keyboard.press('Enter');
      assert.match(await page.locator('[data-status]').textContent(),/Paused/);
      assert.deepEqual(await page.locator('[data-marker]').evaluate(n => {
        const a = n.getAnimations()[0];
        return {duration:a.effect.getTiming().duration,rate:a.playbackRate};
      }),{duration:180,rate:0.25});
      const paused = await page.locator('[data-marker]').evaluate(n => n.getAnimations()[0].currentTime);
      await page.waitForTimeout(100);
      assert.equal(await page.locator('[data-marker]').evaluate(n => n.getAnimations()[0].currentTime),paused);
      assert.notEqual(await page.locator(':focus').evaluate(n => getComputedStyle(n).outlineStyle),'none');
      row.frames.push(await snapshot(page,`timing-${width}-paused`));
      await page.keyboard.press('Enter');
      await page.emulateMedia({reducedMotion:'reduce'});
      await page.waitForFunction(() => document.querySelector('[data-marker]').getAnimations().length === 0);
      assert.equal(await page.locator('[data-marker]').evaluate(n => getComputedStyle(n).transform),'none');
      assert.ok(await page.getByText('Selection and useful information').isVisible());
      await page.emulateMedia({reducedMotion:'no-preference'});
      await page.evaluate(() => {
        const root = document.querySelector('[data-motion-specimen]');
        root.motionSpecimen.replay(); root.motionSpecimen.replay(); root.motionSpecimen.replay();
        root.motionSpecimen.dispose(); root.remove();
      });
      assert.equal(await page.evaluate(() => document.getAnimations().length),0);
      row.checks.push('actual180ms, labeled720ms slow mode, keyboard pause/resume/focus, live reduction, rapid replay and explicit removal cleanup');

      await page.goto(character);
      await page.waitForFunction(() => document.querySelector('#warm').visible === true);
      const coolBefore = await page.locator('#cool').evaluate(n => ({state:n.dataset.pose,
        fill:n.querySelector('linearGradient stop').getAttribute('stop-color')}));
      await page.locator('#warm button[data-pose=working]').focus();
      await page.keyboard.press('Enter');
      assert.match(await page.locator('#warm [role=status]').textContent(),/^Working/);
      // Sample an intermediate pose during natural playback; capture timestamps
      // bound the observation rather than promising an exact animation time.
      await page.waitForTimeout(70);
      row.frames.push(await snapshot(page,`character-${width}-working`));
      await page.evaluate(() => {
        const n = document.querySelector('#warm'); n.select('rest'); n.select('working'); n.select('rest');
        n.querySelector('linearGradient stop').setAttribute('stop-color','#edc377');
      });
      await page.waitForTimeout(300);
      assert.match(await page.locator('#warm [role=status]').textContent(),/^Rest/);
      assert.deepEqual(await page.locator('#cool').evaluate(n => ({state:n.dataset.pose,
        fill:n.querySelector('linearGradient stop').getAttribute('stop-color')})),coolBefore);
      await page.evaluate(() => document.querySelector('#warm').select('working'));
      await page.emulateMedia({reducedMotion:'reduce'});
      await page.waitForFunction(() => document.getAnimations().length === 0);
      await page.evaluate(() => { document.querySelector('#warm').select('ready'); document.querySelector('#cool').select('rest'); });
      assert.equal(await page.evaluate(() => document.getAnimations().length),0);
      row.frames.push(await snapshot(page,`character-${width}-reduced`,true));
      await page.emulateMedia({reducedMotion:'no-preference'});
      const removal = await page.evaluate(() => {
        const n = document.querySelector('#warm');n.select('working'); n.remove();
        const cleared = n.motion.size === 0 && n.abort.signal.aborted;
        document.querySelector('.studies').prepend(n);
        return cleared && n.dataset.pose === 'working';
      });
      assert.ok(removal);
      await page.evaluate(() => {
        document.querySelector('#warm').select('rest');
        document.querySelector('main').style.paddingBottom = '1600px';
        scrollTo(0,document.body.scrollHeight);
      });
      await page.waitForFunction(() => document.querySelector('#warm').visible === false);
      assert.equal(await page.evaluate(() => document.getAnimations().length),0);
      await page.setViewportSize({width:width===390?420:900,height:900});
      await page.evaluate(() => scrollTo(0,0));
      assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
      assert.deepEqual(errors,[]);
      row.checks.push('two independent materials/statuses, rapid reversal/latest state, live reduction, disconnected cleanup/reconnect, offscreen stop, responsive bounds');
      result.scenarios.push(row);
    } finally { await context.close(); }
  }
  for (const mode of ['reduced','no-script']) {
    const context = await browser.newContext({reducedMotion:'reduce',javaScriptEnabled:mode!=='no-script'});
    const page = await context.newPage();
    try {
      await page.goto(timing);
      assert.ok(await page.getByRole('heading',{name:'Activity selected'}).isVisible());
      if (mode==='reduced') { await page.locator('[data-replay]').click(); assert.equal(await page.evaluate(() => document.getAnimations().length),0); }
      await page.goto(character);
      assert.equal(await page.locator('.character').count(),2);
      assert.equal(await page.locator('[role=status]').count(),2);
      if (mode==='reduced') { await page.locator('#warm button[data-pose=working]').click(); assert.equal(await page.evaluate(() => document.getAnimations().length),0); }
      else assert.equal(await page.locator('button:disabled').count(),6);
      result.scenarios.push({tag:`craft-${mode}`,checks:['readable static content and semantic status'],frames:[await snapshot(page,`character-${mode}`,true)]});
    } finally { await context.close(); }
  }
};
