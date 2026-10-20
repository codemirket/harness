// Uses an explicitly installed Playwright module and browser; installs nothing.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
const { chromium } = await import(process.env.MIRKET_PLAYWRIGHT_MODULE || 'playwright-core');
const [url, output] = process.argv.slice(2);
if (!url || !output || !process.env.MIRKET_CHROMIUM_PATH)
  throw Error('Provide URL, evidence directory and MIRKET_CHROMIUM_PATH');
const expected = (await fs.readFile(new URL('../public/install-prompt.txt', import.meta.url), 'utf8')).trim();
await fs.mkdir(output, {recursive:true});
const browser = await chromium.launch({executablePath:process.env.MIRKET_CHROMIUM_PATH,headless:true});
const report=[];
try {
  for (const width of [1440, 768, 390, 320]) {
    const context=await browser.newContext({viewport:{width,height:1000},deviceScaleFactor:2,reducedMotion:'reduce'});
    try {
      const page=await context.newPage();
      const errors=[];const failed=[];
      page.on('pageerror',e=>errors.push(e.message));
      page.on('response',r=>{if(r.status()>=400)failed.push(r.url());});
      await page.goto(url,{waitUntil:'networkidle'});
      assert.equal(await page.locator('h1').count(),1);
      assert.match(await page.locator('#release-status').textContent(),/Mirket 1\.0\.11/);
      assert.match(await page.locator('#continuity').textContent(),/optional agent runtime/);
      assert.equal(await page.locator('#install-prompt').inputValue(),expected);
      assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),`Overflow at ${width}`);
      await page.keyboard.press('Tab');
      assert.equal(await page.locator(':focus').textContent(),'Skip to content');
      await page.screenshot({path:path.join(output,`${width}-entry.png`)});
      await page.keyboard.press('Enter');
      assert.equal(await page.locator(':focus').getAttribute('id'),'main');
      await page.locator('.hero a[href="#install"]').click();
      assert.equal(new URL(page.url()).hash,'#install');
      assert.ok(await page.locator('#install').evaluate(e=>Math.abs(e.getBoundingClientRect().top-28)<2));
      for (const link of await page.locator('header nav a[href^="#"]').all()) {
        const hash=await link.getAttribute('href');
        await link.click();
        assert.equal(new URL(page.url()).hash,hash);
        assert.ok(await page.locator(hash).isVisible());
      }
      assert.equal(await page.evaluate(()=>getComputedStyle(document.documentElement).colorScheme),'dark');
      for (const stage of ['build','verify','scope']) {
        const control=page.locator(`[data-stage="${stage}"]`);
        await control.focus();await page.keyboard.press('Enter');
        assert.equal(await control.getAttribute('aria-pressed'),'true');
        assert.ok(await page.locator(`[data-panel="${stage}"]`).isVisible());
        assert.equal(await page.locator('[data-panel]:visible').count(),1);
        await page.locator('#how-it-works').scrollIntoViewIfNeeded();
        await page.screenshot({path:path.join(output,`${width}-workflow-${stage}.png`)});
      }
      await page.evaluate(()=>Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async text=>{window.copiedText=text;}}}));
      await page.locator('#copy').focus();
      await page.keyboard.press('Enter');
      assert.equal(await page.evaluate(()=>window.copiedText),expected);
      assert.match(await page.locator('#copy-status').textContent(),/^Copied\./);
      await page.evaluate(()=>Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async()=>{throw Error('denied');}}}));
      await page.locator('#copy').click();
      assert.match(await page.locator('#copy-status').textContent(),/Copy unavailable/);
      assert.equal(await page.locator(':focus').getAttribute('id'),'install-prompt');
      assert.equal(await page.locator('#install-prompt').evaluate(e=>e.selectionEnd-e.selectionStart),expected.length);
      const summary=page.locator('summary').first();
      await summary.focus();await page.keyboard.press('Enter');
      assert.equal(await page.locator('details').first().getAttribute('open'),'');
      await page.keyboard.press('Enter');
      await page.locator('#quality-gates summary').click();
      assert.ok(await page.locator('#code-fallback').isVisible());
      assert.equal(await page.locator('#quality-gates dt').count(),6);
      for(const selector of ['#install','#project-setup','#how-it-works','#daily-use','#architecture','#packages','#continuity','#image-to-code','#code-fallback','#quality-gates','#reference','.faq']) {
        await page.locator(selector).first().scrollIntoViewIfNeeded();
        await page.screenshot({path:path.join(output,`${width}-${selector.replace(/[.#]/g,'')}.png`)});
      }
      await page.evaluate(()=>{const sizes=[...document.querySelectorAll('body, body *')].map(e=>[e,parseFloat(getComputedStyle(e).fontSize)*2]);for(const [e,size] of sizes)e.style.fontSize=`${size}px`;});
      // Browser zoom-equivalent reflow is separately covered by a 320 CSS-pixel viewport.
      assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),`Enlarged text overflow at ${width}`);
      await page.locator('#install').scrollIntoViewIfNeeded();
      await page.screenshot({path:path.join(output,`${width}-enlarged.png`)});
      assert.deepEqual(errors,[]);assert.deepEqual(failed,[]);
      report.push({width,deviceScale:2,copy:'success and denial',keyboard:'passed',overflow:false,consoleErrors:errors});
    } finally {await context.close();}
  }
  const context=await browser.newContext({javaScriptEnabled:false,viewport:{width:390,height:844}});
  try {
    const page=await context.newPage();await page.goto(url);
    assert.equal(await page.locator('#install-prompt').inputValue(),expected);
    assert.ok(await page.locator('noscript').isVisible());
    assert.equal(await page.locator('#quality-gates dt').count(),6);
    assert.equal(await page.locator('[data-panel]:visible').count(),3);
    assert.ok(await page.locator('.workflow-controls').isHidden());
    await page.locator('#quality-gates summary').click();
    assert.ok(await page.locator('#image-to-code').isVisible());
    assert.ok(await page.locator('#code-fallback').isVisible());
    await page.locator('#project-setup summary').click();
    assert.ok(await page.locator('#project-setup pre').isVisible());
    assert.ok(await page.locator('#copy').isHidden());
    const text=await context.request.get(new URL('install-prompt.txt',url).href);
    assert.ok(text.ok());assert.equal((await text.text()).trim(),expected);
    assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
    report.push({javascript:false,promptReadable:true,plainText:'matches'});
  } finally {await context.close();}
  await fs.writeFile(path.join(output,'browser-report.json'),JSON.stringify(report,null,2)+'\n');
  console.log(JSON.stringify(report));
} finally {await browser.close();}
