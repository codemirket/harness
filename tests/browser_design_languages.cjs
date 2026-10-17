/* Optional real-browser evidence for the installed design-inspiration library.
 * Requires the managed Browserless service and its installed playwright-core client.
 * node tests/browser_design_languages.cjs LOOPBACK_BASE OUTPUT_DIR LIBRARY_JSON [ID,...]
 * BROWSERLESS_WS selects its WebSocket endpoint; the helper uses native Playwright. Screenshots require actual craft review. */
const { connect } = require('./browserless.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const base = process.argv[2];
const output = path.resolve(process.argv[3] || '.mirket/evidence/design-languages');
const manifestPath = process.argv[4];
const selection = process.argv[5]?.split(',');
const baseUrl = new URL(base || 'about:blank');
if (baseUrl.protocol !== 'http:' || !['127.0.0.1','localhost'].includes(baseUrl.hostname)
    || !baseUrl.port || baseUrl.username || baseUrl.password || baseUrl.search || baseUrl.hash
    || !baseUrl.pathname.endsWith('/')) throw new Error('Provide a loopback base ending in /.');
if (!manifestPath) throw new Error('Provide the installed library.json path.');
const report = { browser:'', base, manifest:manifestPath, scope:'Real Chromium rendering; automated access checks are bounded and do not establish human visual acceptance or assistive-technology compatibility.', examples:[], failures:[] };
async function geometry(page) {
  const size = await page.evaluate(() => ({viewport:innerWidth, width:document.documentElement.scrollWidth}));
  assert.ok(size.width <= size.viewport + 1, `horizontal overflow ${JSON.stringify(size)}`);
  return size;
}
async function capture(page, file) {
  await page.screenshot({path:path.join(output,file),fullPage:true});
  return file;
}
async function inspect(browser, instance, viewport, variant = '') {
  const id = instance.id;
  const tag = `${id}-${viewport.width}${variant ? '-misleading' : ''}`;
  const context = await browser.newContext({viewport, reducedMotion:'reduce',deviceScaleFactor:2});
  const page = await context.newPage();
  const errors = [], external = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.route('**/*', route => {
    if (new URL(route.request().url()).origin === baseUrl.origin) return route.continue();
    external.push(route.request().url()); return route.abort();
  });
  const row = {id,tag,viewport,variant:variant || 'purposeful',errors,external};
  try {
    await page.goto(`${base}${id}/${variant ? '?variant=misleading' : ''}`,{waitUntil:'load'});
    await page.locator('main').waitFor();
    assert.equal(await page.locator('html').getAttribute('lang'),'en');
    assert.equal(await page.locator('html').getAttribute('data-design-language'),id);
    assert.equal(await page.locator('main').count(),1);
    assert.ok((await page.locator('main').innerText()).length > 80,'specimen must convey real content');
    assert.equal(await page.locator('main .lesson, main .lesson-note, main aside[aria-label*="lesson" i]').count(),0,'teaching notes must not replace the actual specimen');
    row.geometry = await geometry(page);
    row.screenshot = await capture(page,`${tag}.png`);

    if (!variant) {
      const controls = page.locator('main button, main input, main select, main summary, main a[href]');
      // Disabled history commands are legitimately unfocusable in their initial
      // state. Inspect the first visible, enabled control instead of treating a
      // disabled Undo button as a failed keyboard journey.
      let focusable;
      for (let i=0; i<await controls.count(); i++) {
        const candidate=controls.nth(i);
        if (await candidate.isVisible() && await candidate.isEnabled()) { focusable=candidate; break; }
      }
      if (focusable) {
        await focusable.focus();
        row.focus = await focusable.evaluate(node => ({tag:node.tagName,focused:document.activeElement===node,outline:getComputedStyle(node).outlineStyle}));
        assert.equal(row.focus.focused,true);
        row.focus_screenshot = await capture(page,`${tag}-focus.png`);
      }
      // Enlarged text must reflow without clipping the document, not be hidden.
      if (viewport.width < 500) {
        await page.evaluate(() => {
          // Emulate text-only enlargement even for specimens using pixel type.
          // SVG label resizing is inspected separately at the actual use size.
          const nodes=[...document.querySelectorAll('main *:not(svg):not(svg *)')];
          const sizes=nodes.map(node=>[node,parseFloat(getComputedStyle(node).fontSize)]);
          for(const [node,size] of sizes) node.style.fontSize=`${size*2}px`;
        });
        row.enlarged_geometry = await geometry(page);
      }
    }
    assert.deepEqual(errors,[]); assert.deepEqual(external,[],'standalone examples may not request remote media');
  } catch (error) { row.failure = String(error); report.failures.push({tag,error:String(error)}); }
  await context.close(); return row;
}
(async()=>{
  await fs.mkdir(output,{recursive:true});
  const library=JSON.parse(await fs.readFile(manifestPath,'utf8'));
  const browser=await connect();
  report.browser=browser.version();
  for(const instance of library.instances.filter(item=>!selection || selection.includes(item.id))) {
    for(const viewport of [{width:1280,height:900},{width:390,height:844}]) report.examples.push(await inspect(browser,instance,viewport));
    report.examples.push(await inspect(browser,instance,{width:1280,height:900},'misleading'));
    const context=await browser.newContext({javaScriptEnabled:false,viewport:{width:390,height:844}});
    const page=await context.newPage();
    try {
      await page.goto(`${base}${instance.id}/`,{waitUntil:'load'});
      assert.ok((await page.locator('main').innerText()).length > 80);
      await geometry(page);
      report.examples.push({id:instance.id,tag:`${instance.id}-nojs`,screenshot:await capture(page,`${instance.id}-nojs.png`),nojs_readable:true});
    }catch(error){report.failures.push({tag:`${instance.id}-nojs`,error:String(error)});}
    await context.close();
  }
  await browser.close();
  await fs.writeFile(path.join(output,'report.json'),JSON.stringify(report,null,2)+'\n');
  console.log(JSON.stringify({examples:report.examples.length,failures:report.failures,report:path.join(output,'report.json')}));
  if(report.failures.length) process.exitCode=1;
})().catch(error=>{console.error(error);process.exitCode=1;});
