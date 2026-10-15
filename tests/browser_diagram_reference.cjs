/* Probe the installed relay reference in an existing Playwright/Chromium runtime.
 * Usage: node tests/browser_diagram_reference.cjs URL OUTPUT_DIRECTORY
 * URL points directly to the installed relay-diagram/ directory. */
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { chromium } = require('playwright');

const url = new URL(process.argv[2] || 'http://127.0.0.1:8766/');
const output = path.resolve(process.argv[3] || '.mirket/evidence/diagram-reference');
fs.mkdirSync(path.join(output, 'video'), {recursive:true});
const report = {url:String(url),scenarios:[],limitation:'Browser checks do not grade visual craft. Inspect saved normal-speed video and frames.'};
const pause = ms => new Promise(resolve => setTimeout(resolve,ms));
function contrast(foreground,background) {
  const luminance = color => {
    const channels=(color.match(/[\d.]+/g)||[]).slice(0,3).map(Number).map(value=>{
      const normalized=value/255;
      return normalized<=0.04045 ? normalized/12.92 : ((normalized+0.055)/1.055)**2.4;
    });
    return channels[0]*0.2126+channels[1]*0.7152+channels[2]*0.0722;
  };
  const a=luminance(foreground),b=luminance(background);
  return (Math.max(a,b)+0.05)/(Math.min(a,b)+0.05);
}

(async () => {
  const browser = await chromium.launch({headless:true,
    ...(process.env.MIRKET_BROWSER_EXECUTABLE ? {executablePath:process.env.MIRKET_BROWSER_EXECUTABLE} : {})});
  try {
    for (const width of [1080,390,320]) {
      const context = await browser.newContext({viewport:{width,height:960},reducedMotion:'no-preference',
        recordVideo:{dir:path.join(output,'video'),size:{width,height:960}}});
      const page = await context.newPage();
      const errors=[];
      page.on('pageerror',error=>errors.push(String(error)));
      try {
        await page.goto(String(url));
        const row={width,frames:[],checks:[]};
        assert.equal(await page.locator('svg:visible').count(),1);
        assert.match(await page.locator('svg:visible').textContent(),/Observe[\s\S]*Decide[\s\S]*Act/);
        assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
        if(width===320) {
          const labels=await page.locator('svg:visible .node-caption, svg:visible .route-label').evaluateAll(nodes=>nodes.map(n=>n.getBoundingClientRect().height));
          assert.ok(labels.every(height=>height>=12),`small-screen labels too small: ${labels}`);
        }
        const labelColors=await page.locator('svg:visible .node-caption, svg:visible .route-label').evaluateAll(nodes=>nodes.map(node=>({
          fill:getComputedStyle(node).fill,
          background:node.classList.contains('route-label')?'rgb(247, 248, 245)':node.closest('.node-center')?'rgb(23, 62, 83)':'rgb(255, 255, 255)',
        })));
        assert.ok(labelColors.every(pair=>contrast(pair.fill,pair.background)>=4.5),`diagram label contrast is low: ${JSON.stringify(labelColors)}`);
        row.checks.push('readable structure present and no horizontal overflow');
        for (const [name,delay] of [['first',550],['second',1000],['settled',1150]]) {
          await pause(delay);
          const frame=path.join(output,`relay-${width}-${name}.png`);
          await page.screenshot({path:frame,fullPage:true});
          row.frames.push(frame);
        }
        assert.equal((await page.evaluate(()=>window.diagramStudy.state())).running,false);
        await page.locator('[data-replay]').click();
        await pause(80);
        await page.locator('[data-replay]').click();
        assert.equal((await page.evaluate(()=>window.diagramStudy.state())).running,true);
        await page.emulateMedia({reducedMotion:'reduce'});
        await page.waitForTimeout(350);
        await page.waitForFunction(()=>window.diagramStudy.state().reduced && !window.diagramStudy.state().running && document.querySelector('[data-replay]').disabled,null,{polling:100});
        assert.equal(await page.locator('[data-replay]').isDisabled(),true);
        assert.equal(await page.locator('svg:visible [data-packet="first"]').evaluate(n=>getComputedStyle(n).opacity),'0');
        row.checks.push('ordinary playback settles; replay replaces prior motion; live reduced motion cancels packets');
        assert.deepEqual(errors,[]);
        report.scenarios.push(row);
      } finally { await context.close(); }
    }
    for (const mode of ['reduced','no-script']) {
      const context=await browser.newContext({viewport:{width:320,height:900},reducedMotion:'reduce',javaScriptEnabled:mode!=='no-script'});
      const page=await context.newPage();
      try {
        await page.goto(String(url));
        assert.equal(await page.locator('svg:visible').count(),1);
        assert.match(await page.locator('svg:visible').textContent(),/Observe[\s\S]*Decide[\s\S]*Act/);
        assert.equal(await page.locator('[data-replay]').isDisabled(),true);
        if(mode==='reduced') assert.equal((await page.evaluate(()=>window.diagramStudy.state())).running,false);
        const frame=path.join(output,`relay-${mode}.png`);
        await page.screenshot({path:frame,fullPage:true});
        report.scenarios.push({mode,frames:[frame],checks:['static meaning, routes and labels survive']});
      } finally {await context.close();}
    }
    const page=await browser.newPage();
    try {
      await page.goto(String(url));
      await page.evaluate(()=>window.diagramStudy.seek(650));
      const comparison=[];
      for(const [name,ms] of [['mid',650],['rest',2400]]) {
        await page.evaluate(ms=>window.diagramStudy.seek(ms),ms);
        const frame=path.join(output,`relay-comparison-a-${name}.png`);
        await page.locator('[data-diagram]').screenshot({path:frame});
        comparison.push(frame);
      }
      const awkward=new URL(String(url)); awkward.searchParams.set('variant','awkward');
      await page.goto(String(awkward));
      await page.evaluate(()=>window.diagramStudy.seek(650));
      const active=await page.locator('svg:visible [data-route]').evaluateAll(nodes=>nodes.map(n=>getComputedStyle(n).opacity));
      assert.deepEqual(active,['1','1']);
      for(const [name,ms] of [['mid',650],['rest',2400]]) {
        await page.evaluate(ms=>window.diagramStudy.seek(ms),ms);
        const frame=path.join(output,`relay-comparison-b-${name}.png`);
        await page.locator('[data-diagram]').screenshot({path:frame});
        comparison.push(frame);
      }
      assert.equal(await page.locator('svg:visible [data-ring="observe"]').getAttribute('stroke-width'),'6');
      report.scenarios.push({mode:'awkward-counterexample',frames:comparison,checks:['simultaneous handoffs and wrong final emphasis are present; visual verdict is external']});
    } finally {await page.close();}
    fs.writeFileSync(path.join(output,'report.json'),JSON.stringify(report,null,2));
    console.log(JSON.stringify({ok:true,scenarios:report.scenarios.length,output}));
  } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
