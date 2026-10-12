// Fixed local accessibility runner. No project modules, hooks or arbitrary scripts.
import { chromium } from 'playwright-core';
import fs from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { buildReport } from './report.mjs';
const axePath = fileURLToPath(new URL('./node_modules/axe-core/axe.min.js', import.meta.url));
const bounded = (value, max = 1024) => String(value ?? '').slice(0, max);
let browser;
try {
  browser = await chromium.launch({ executablePath: process.env.MIRKET_CHROMIUM, headless: true, downloadsPath: process.cwd(), args: ['--disable-background-networking'] });
  const probe = process.argv[2] === '--probe';
  const options = probe ? {width:640,height:360} : JSON.parse(await fs.readFile(process.argv[3], 'utf8'));
  const context = await browser.newContext({viewport:{width:options.width ?? 1280,height:options.height ?? 800}, acceptDownloads:false, serviceWorkers:'block'});
  const page = await context.newPage();
  page.setDefaultTimeout(5000); page.setDefaultNavigationTimeout(15000);
  const errors = [], blocked = [];
  page.on('pageerror', error => errors.push(bounded(error.message)));
  page.on('popup', child => { errors.push('unexpected popup'); child.close().catch(()=>{}); });
  page.on('download', download => { errors.push('unexpected download'); download.cancel().catch(()=>{}); });
  if (probe) {
    await page.setContent('<!doctype html><html lang="en"><title>Runtime probe</title><main><h1>Probe</h1><button></button></main></html>');
  } else {
    const origin = new URL(options.url).origin;
    await context.route('**/*', async route => {
      const url = new URL(route.request().url());
      if (url.origin === origin || ['data:','blob:'].includes(url.protocol)) await route.continue();
      else { blocked.push(bounded(url.origin)); await route.abort('blockedbyclient'); }
    });
    const response = await page.goto(options.url, {waitUntil:'load'});
    if (!response || !response.ok()) throw Error(`Local page returned ${response?.status() ?? 'no response'}`);
    if (new URL(page.url()).origin !== origin) throw Error('Page redirected outside the selected project origin');
    for (const action of options.actions ?? []) {
      const locator = page.locator(action.selector);
      if (action.action === 'click') await locator.click();
      else if (action.action === 'fill') await locator.fill(action.value);
      else if (action.action === 'press') await locator.press(action.key);
      else throw Error('Unsupported typed action');
      if (new URL(page.url()).origin !== origin) throw Error('Interaction navigated outside the selected project origin');
    }
    await page.evaluate(async () => { await document.fonts.ready; });
  }
  if (errors.length || blocked.length) throw Error(`Audit state is incomplete: page errors=${errors.join('; ')}, blocked external origins=${[...new Set(blocked)].join(', ')}`);
  const frames = page.frames();
  if (frames.length > 8) throw Error('More than eight frames; split this page audit');
  for (const frame of frames) await frame.addScriptTag({path:axePath});
  const visible = await page.evaluate(() => Array.from(document.body.querySelectorAll('*')).filter(el => {
    const rect=el.getBoundingClientRect(); const style=getComputedStyle(el);
    return rect.width>0 && rect.height>0 && style.visibility!=='hidden' && style.display!=='none';
  }).length);
  if (!visible) throw Error('Rendered page has no visible content to audit');
  if (options.selector && !(await page.locator(options.selector).isVisible())) throw Error('Selected audit region is absent or hidden');
  const result = await page.evaluate(async ({selector,tags}) => {
    const context=selector ? {include:[selector]} : document;
    return await window.axe.run(context,{runOnly:{type:'tag',values:tags?.length ? tags : ['wcag2a','wcag2aa','wcag21a','wcag21aa','wcag22aa','best-practice']}});
  }, options);
  if (probe) {
    if (!result.violations.some(rule=>rule.id==='button-name')) throw Error('Engine failed its inaccessible button probe');
    process.stdout.write(JSON.stringify({engine:'axe-core',version:result.testEngine.version,browser_version:browser.version(),failure_probe:'button-name'}));
  } else {
    const report = buildReport(result, {
      url: options.url,
      final_url: page.url(),
      viewport: page.viewportSize(),
      visible_elements: visible,
      actions: options.actions ?? [],
    });
    const bytes=JSON.stringify(report,null,2);
    if (Buffer.byteLength(bytes)>1024*1024) throw Error('Audit report exceeds 1 MiB; select a smaller page region');
    await fs.writeFile(process.argv[4], bytes, {flag:'wx'});
    process.stdout.write(JSON.stringify({engine:'axe-core',version:result.testEngine.version,violations:report.violations.length,incomplete:report.incomplete.length}));
  }
} catch(error) { process.stderr.write(`Accessibility audit failed: ${bounded(error.stack ?? error,4096)}\n`); process.exitCode=1; }
finally { await browser?.close(); }
