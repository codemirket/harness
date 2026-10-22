// Existing installed browser only; this does not establish legal compliance.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
const {chromium} = await import(process.env.MIRKET_PLAYWRIGHT_MODULE || 'playwright-core');
const [url, output] = process.argv.slice(2);
if (!url || !output || !process.env.MIRKET_CHROMIUM_PATH) throw Error('Provide URL, evidence directory and MIRKET_CHROMIUM_PATH');
await fs.mkdir(output, {recursive:true});
const browser = await chromium.launch({executablePath:process.env.MIRKET_CHROMIUM_PATH,headless:true});
const report = [];
try {
  for (const width of [1440,390,320]) {
    const context = await browser.newContext({viewport:{width,height:1000},deviceScaleFactor:2,javaScriptEnabled:false});
    try {
      const page = await context.newPage();
      const requests = []; const failed = [];
      page.on('request', request => requests.push(request.url()));
      page.on('response', response => {if(response.status()>=400) failed.push(response.url());});
      await page.goto(url);
      for (const file of ['legal.html']) {
        await page.locator(`footer a[href="./${file}"]`).click();
        assert.equal(new URL(page.url()).pathname,`/${file}`);
        assert.equal(await page.locator('h1').count(),1);
        assert.equal(await page.locator('main').count(),1);
        assert.ok(await page.locator('a[href="mailto:hello@mirket.dev"]').count()>0);
        assert.equal(await page.locator('[data-legal-draft]').count(),0);
        assert.equal(await page.locator('meta[name="robots"][content="noindex"]').count(),0);
        for (const link of await page.locator('header nav a,footer nav a').all()) assert.ok((await link.boundingBox()).height>=44);
        await page.goto(new URL(file,url).href);
        await page.keyboard.press('Tab');
        assert.equal(await page.locator(':focus').textContent(),'Skip to content');
        await page.keyboard.press('Enter');
        assert.equal(await page.locator(':focus').getAttribute('id'),'main');
        assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
        await page.screenshot({path:path.join(output,`${width}-${file}.png`),fullPage:true});
        await page.evaluate(()=>{const sizes=[...document.querySelectorAll('body,body *')].map(e=>[e,parseFloat(getComputedStyle(e).fontSize)*2]);for(const [e,size] of sizes)e.style.fontSize=`${size}px`;});
        assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),`Enlarged text overflow: ${file} at ${width}`);
        assert.deepEqual(await context.cookies(),[]);
        assert.equal(await page.evaluate(()=>localStorage.length+sessionStorage.length),0);
        report.push({width,page:file,javascript:false,deviceScale:2,keyboard:true,reflow:true,enlargedText:true,storageIdentifiers:false});
        await page.goto(url);
      }
      assert.deepEqual(failed,[]);
      assert.ok(requests.every(request=>new URL(request).origin===new URL(url).origin),'Unexpected third-party request');
    } finally {await context.close();}
  }
  await fs.writeFile(path.join(output,'report.json'),JSON.stringify(report,null,2)+'\n');
  console.log(JSON.stringify(report));
} finally {await browser.close();}
