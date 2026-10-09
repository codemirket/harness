#!/usr/bin/env node
/** Deterministic local browser evidence; no installs, model calls, or baseline writes. */
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import { createRequire } from 'node:module';

const HELP = `Browser evidence collector
Usage: node browser.mjs --scenario FILE --output NEW_DIR
Optional environment: HARNESS_NODE_MODULES (absolute node_modules directory),
HARNESS_BROWSER (absolute Chromium executable). No runtime downloads occur.
Scenario JSON (unknown keys are rejected):
 {"schema_version":1,"url":"http://127.0.0.1:3000",
  "viewports":[{"name":"desktop","width":1440,"height":1000}],
  "motion":["no-preference","reduce"], "allowed_origins":[],
  "timeout_ms":60000,
  "actions":[{"op":"click","selector":"button"},
             {"op":"fill","selector":"input","value":"hello"},
             {"op":"capture","label":"opening","times_ms":[0,150,400]}],
  "assertions":[{"op":"visible","selector":"dialog"},
                {"op":"text","selector":"h1","expected":"Hello"}]}
Actions: click/fill/press/check/uncheck/hover (selector; fill/press require value),
wait (ms), capture (label, increasing times_ms offsets from start of capture).
Assertions: visible/hidden/text/value/count; text/value compare exact strings,
count compares a nonnegative integer. Visibility requires a unique element;
hidden permits zero matches. Final screenshot is always captured.
Defaults: desktop 1440x1000 + mobile 390x844, normal + reduced motion.
Limits: 4 viewports, 40 actions, 40 assertions, 80 screenshots, 90 seconds.
Initial URL must be loopback HTTP(S); allowed_origins explicitly permits other
HTTP(S) origins for resources/navigation. New isolated contexts, no saved auth.
Request/WebSocket controls are browser guards, not an OS network sandbox.
Exit 0: completed checks (may be limited with no assertions); 1: observed failure;
2: invalid input or unavailable runtime. report.json always requires visual review.
`;
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const fail = message => { throw new Error(message); };
const object = x => x && typeof x === 'object' && !Array.isArray(x);
function keys(value, allowed, name) {
  if (!object(value) || Object.keys(value).some(k => !allowed.includes(k))) fail(`Invalid ${name} fields`);
}
function string(value, name, max = 512) {
  if (typeof value !== 'string' || !value.length || value.length > max) fail(`Invalid ${name}`);
  return value;
}
function integer(value, min, max, name) {
  if (!Number.isInteger(value) || value < min || value > max) fail(`Invalid ${name}`);
  return value;
}
function local(url) {
  // Exact loopback names only: lookalikes such as localhost.example are excluded.
  return url.hostname === 'localhost' || url.hostname === '[::1]' || /^127\.\d{1,3}\.\d{1,3}\.\d{1,3}$/.test(url.hostname);
}
function webURL(value) {
  const url = new URL(string(value, 'URL', 2048));
  if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password) fail('Only HTTP(S) URLs without credentials are supported');
  return url;
}
function validate(input) {
  keys(input, ['schema_version','url','viewports','motion','allowed_origins','timeout_ms','actions','assertions'], 'scenario');
  if (input.schema_version !== 1) fail('Unsupported scenario schema_version');
  const url = webURL(input.url);
  if (!local(url)) fail('Initial URL must use a loopback host');
  const result = {...input, url: url.href,
    viewports: input.viewports ?? [{name:'desktop',width:1440,height:1000},{name:'mobile',width:390,height:844}],
    motion: input.motion ?? ['no-preference','reduce'], allowed_origins: input.allowed_origins ?? [],
    actions: input.actions ?? [], assertions: input.assertions ?? [], timeout_ms: input.timeout_ms ?? 60000};
  for (const [field, max] of [['viewports',4],['motion',2],['allowed_origins',16],['actions',40],['assertions',40]]) {
    if (!Array.isArray(result[field]) || result[field].length > max) fail(`Invalid ${field} list`);
  }
  if (!result.viewports.length || !result.motion.length) fail('At least one viewport and motion mode required');
  const names = new Set();
  for (const viewport of result.viewports) {
    keys(viewport, ['name','width','height'], 'viewport');
    string(viewport.name,'viewport name',31);
    if (!/^[a-z][a-z0-9-]{0,30}$/.test(viewport.name) || names.has(viewport.name)) fail('Invalid or duplicate viewport name');
    names.add(viewport.name); integer(viewport.width,240,2560,'width'); integer(viewport.height,240,2000,'height');
  }
  if (new Set(result.motion).size !== result.motion.length || result.motion.some(x => !['no-preference','reduce'].includes(x))) fail('Invalid motion modes');
  result.allowed_origins = result.allowed_origins.map(value => {
    const origin = webURL(value); if (origin.href !== origin.origin + '/') fail('allowed_origins requires origins without paths');
    return origin.origin;
  });
  integer(result.timeout_ms,1000,90000,'timeout_ms');
  let captures = 1;
  for (const action of result.actions) {
    if (!object(action)) fail('Invalid action');
    if (action.op === 'wait') { keys(action,['op','ms'],'wait'); integer(action.ms,0,5000,'wait ms'); }
    else if (action.op === 'capture') {
      keys(action,['op','label','times_ms'],'capture');
      string(action.label,'capture label',31);
      if (!/^[a-z][a-z0-9-]{0,30}$/.test(action.label)) fail('Invalid capture label');
      if (!Array.isArray(action.times_ms) || !action.times_ms.length || action.times_ms.length > 8) fail('Invalid capture times');
      action.times_ms.forEach((value,i) => { integer(value,0,5000,'capture time'); if(i && value <= action.times_ms[i-1]) fail('Capture times must increase'); });
      captures += action.times_ms.length;
    } else {
      if (!['click','fill','press','check','uncheck','hover'].includes(action.op)) fail('Unsupported action');
      keys(action,['op','selector',...(['fill','press'].includes(action.op)?['value']:[])],'action');
      string(action.selector,'selector');
      if (['fill','press'].includes(action.op) && (typeof action.value !== 'string' || action.value.length > 4096)) fail('Invalid action value');
    }
  }
  if (captures * result.viewports.length * result.motion.length > 80) fail('Screenshot limit exceeded');
  for (const assertion of result.assertions) {
    if (!object(assertion) || !['visible','hidden','text','value','count'].includes(assertion.op)) fail('Unsupported assertion');
    keys(assertion,['op','selector',...(['text','value','count'].includes(assertion.op)?['expected']:[])],'assertion');
    string(assertion.selector,'selector');
    if (assertion.op === 'count') integer(assertion.expected,0,10000,'expected count');
    if (['text','value'].includes(assertion.op) && (typeof assertion.expected !== 'string' || assertion.expected.length > 4096)) fail('Invalid assertion expected value');
  }
  return result;
}

async function main() {
  const args = process.argv.slice(2);
  if (args.length === 1 && ['--help','-h'].includes(args[0])) { console.log(HELP); return 0; }
  const parsed = {};
  for (let i=0;i<args.length;i+=2) {
    if (!['--scenario','--output'].includes(args[i]) || !args[i+1] || parsed[args[i]]) fail('Use --scenario FILE --output NEW_DIR; see --help');
    parsed[args[i]] = args[i+1];
  }
  if (!parsed['--scenario'] || !parsed['--output']) fail('Use --scenario FILE --output NEW_DIR; see --help');
  const output = path.resolve(parsed['--output']);
  // Do not follow linked parents or overwrite an existing result directory.
  for (let parent=path.dirname(output);;parent=path.dirname(parent)) {
    const stat = await fs.lstat(parent);
    if (stat.isSymbolicLink() || !stat.isDirectory()) fail('Output parents must be existing real directories');
    if (path.dirname(parent) === parent) break;
  }
  await fs.mkdir(output,{mode:0o700});
  const report = {schema_version:1, started_at:new Date().toISOString(), status:'blocked',
    checks:'not_run', visual_review:'required', scenario_file:path.resolve(parsed['--scenario']),
    runtime:{node:process.version}, runs:[], artifacts:[], limits:[
      'Screenshots and diagnostics require visual review; no design quality score is computed.',
      'Browser request guards are not an operating-system network sandbox.',
      'No comprehensive accessibility audit or business-logic test coverage is implied.',
      'Motion captures are sampled frames; full animation correctness remains unverified.']};
  let browser, timer, deadline=0, cancelled=false, exitCode=2;
  const remaining = () => { if(cancelled || Date.now() >= deadline) fail('Scenario runtime limit exceeded'); return Math.max(1,Math.min(5000,deadline-Date.now())); };
  try {
    const sourceInfo=await fs.lstat(report.scenario_file);
    if(!sourceInfo.isFile() || sourceInfo.size>65536)fail('Scenario must be a regular file of at most 64 KiB');
    const bytes = await fs.readFile(report.scenario_file);
    if (bytes.length > 65536) fail('Scenario file exceeds 64 KiB');
    report.scenario_sha256 = hash(bytes);
    const scenario = validate(JSON.parse(bytes.toString('utf8')));
    report.scenario = scenario;
    const require = createRequire(import.meta.url);
    const modules = process.env.HARNESS_NODE_MODULES;
    if (modules && !path.isAbsolute(modules)) fail('HARNESS_NODE_MODULES must be absolute');
    let playwright;
    try { playwright = require(modules ? path.join(modules,'playwright') : 'playwright'); }
    catch { fail('Playwright runtime unavailable; configure HARNESS_NODE_MODULES. No installation attempted.'); }
    const executablePath = process.env.HARNESS_BROWSER;
    if (executablePath && !path.isAbsolute(executablePath)) fail('HARNESS_BROWSER must be absolute');
    report.runtime.playwright = require(modules ? path.join(modules,'playwright/package.json') : 'playwright/package.json').version;
    deadline = Date.now() + scenario.timeout_ms;
    browser = await playwright.chromium.launch({headless:true,timeout:Math.min(15000,scenario.timeout_ms),executablePath,
      args:['--force-webrtc-ip-handling-policy=disable_non_proxied_udp']});
    report.runtime.browser = browser.version();
    const allowed = value => {
      try { const u=new URL(value); if(u.protocol==='ws:')u.protocol='http:'; if(u.protocol==='wss:')u.protocol='https:';
        return ['http:','https:'].includes(u.protocol) && (local(u) || scenario.allowed_origins.includes(u.origin)); }
      catch { return false; }
    };
    const runScenario = async () => {
      for (const viewport of scenario.viewports) for (const reducedMotion of scenario.motion) {
        remaining();
        const run = {viewport, motion:reducedMotion, started_at:new Date().toISOString(), assertions:[], actions:[],
          console_errors:[], page_errors:[], http_errors:[], failed_requests:[], blocked_requests:[], overflow:[], status:'running'};
        report.runs.push(run);
        const recordEvent = (field,value) => {
          if(run[field].length<200)run[field].push(value);
          else {run.truncated_events ??= {};run.truncated_events[field]=(run.truncated_events[field]??0)+1;}
        };
        const context = await browser.newContext({viewport:{width:viewport.width,height:viewport.height},reducedMotion,
          colorScheme:'light',serviceWorkers:'block',acceptDownloads:false});
        try {
          await context.route('**/*', route => {
            if (allowed(route.request().url())) return route.continue();
            recordEvent('blocked_requests',route.request().url()); return route.abort('blockedbyclient');
          });
          if (typeof context.routeWebSocket !== 'function') fail('Runtime lacks WebSocket policy support');
          await context.routeWebSocket('**/*', route => {
            if (allowed(route.url())) route.connectToServer();
            else {recordEvent('blocked_requests',route.url());route.close();}
          });
          await context.addInitScript(() => {
            for (const name of ['RTCPeerConnection','webkitRTCPeerConnection']) Object.defineProperty(globalThis,name,{value:undefined,configurable:false});
          });
          const page=await context.newPage();
          page.on('console', msg => {if(msg.type()==='error')recordEvent('console_errors',msg.text().slice(0,4096));});
          page.on('pageerror', error => recordEvent('page_errors',String(error).slice(0,4096)));
          page.on('requestfailed', req => recordEvent('failed_requests',{url:req.url(),error:req.failure()?.errorText}));
          page.on('response', response => {if(response.status()>=400)recordEvent('http_errors',{url:response.url(),status:response.status()});});
          page.on('popup', popup => {recordEvent('page_errors','Unexpected popup blocked');void popup.close();});
          page.on('dialog', dialog => {recordEvent('page_errors','Unexpected browser dialog dismissed');void dialog.dismiss();});
          const readOverflow = () => page.evaluate(() => {
            const width=document.documentElement.clientWidth;
            return [...document.querySelectorAll('body *')].filter(e=>{const r=e.getBoundingClientRect();return r.width>0&&r.height>0&&(r.right>width+1||r.left < -1);})
              .slice(0,100).map(e=>{const r=e.getBoundingClientRect();return {tag:e.tagName,id:e.id,testid:e.getAttribute('data-testid'),left:r.left,right:r.right,viewport_width:width};});
          });
          let screenshotIndex=0;
          const capture = async(label, times=[0]) => {
            const start=Date.now();
            for(const time of times) {
              const wait=time-(Date.now()-start);if(wait>0)await page.waitForTimeout(Math.min(wait,remaining()));
              const name=`${viewport.name}-${reducedMotion}-${String(++screenshotIndex).padStart(2,'0')}-${label}-${time}.png`;
              if(report.artifacts.length>=80)fail('Screenshot limit exceeded');
              const overflow=await readOverflow();
              const actualOffset=Date.now()-start;
              const raw=await page.screenshot({animations:'allow',timeout:remaining()});
              await fs.writeFile(path.join(output,name),raw,{flag:'wx',mode:0o600});
              report.artifacts.push({file:name,sha256:hash(raw),bytes:raw.length,width:raw.readUInt32BE(16),height:raw.readUInt32BE(20),viewport:viewport.name,motion:reducedMotion,
                label,overflow_candidates:overflow,requested_offset_ms:time,actual_offset_ms:actualOffset,completed_offset_ms:Date.now()-start,url:page.url()});
            }
          };
          try {
            await page.goto(scenario.url,{waitUntil:'domcontentloaded',timeout:remaining()});
            await page.evaluate(() => document.fonts.ready);
            for(const action of scenario.actions) {
              remaining();const actionRecord={...action,started_at:new Date().toISOString()};run.actions.push(actionRecord);
              if(action.op==='wait')await page.waitForTimeout(Math.min(action.ms,remaining()));
              else if(action.op==='capture')await capture(action.label,action.times_ms);
              else {
                const loc=page.locator(action.selector),options={timeout:remaining()};
                if(['fill','press'].includes(action.op))await loc[action.op](action.value,options);
                else await loc[action.op](options);
              }
              actionRecord.finished_at=new Date().toISOString();
            }
            for(const assertion of scenario.assertions) {
              const record={...assertion,pass:false};run.assertions.push(record);
              try {
                const loc=page.locator(assertion.selector);remaining();
                if(assertion.op==='visible'){await loc.waitFor({state:'visible',timeout:remaining()});record.actual=true;record.pass=true;}
                else if(assertion.op==='hidden'){await loc.waitFor({state:'hidden',timeout:remaining()});record.actual=false;record.pass=true;}
                else {
                  const assertionDeadline=Date.now()+remaining();
                  do {
                    record.actual=assertion.op==='count'?await loc.count():assertion.op==='value'?await loc.inputValue({timeout:remaining()}):await loc.innerText({timeout:remaining()});
                    record.pass=record.actual===assertion.expected;
                    if(record.pass || Date.now()>=assertionDeadline)break;
                    await page.waitForTimeout(Math.min(75,remaining()));
                  } while(true);
                }
              } catch(error){record.error=String(error).slice(0,4096);}
            }
            run.overflow=await readOverflow();
            await capture('final');
          } catch(error) {
            run.error=String(error).slice(0,4096);
            if(!cancelled && Date.now()<deadline)try{await capture('failure');}catch{/* Retain primary error. */}
          }
          run.url=page.url();
          run.status=run.error || run.assertions.some(x=>!x.pass) || run.console_errors.length || run.page_errors.length || run.http_errors.length || run.failed_requests.length || run.blocked_requests.length ? 'fail':'pass';
        } finally {run.finished_at=new Date().toISOString();await context.close();}
      }
    };
    await Promise.race([runScenario(),new Promise((_,reject)=>{timer=setTimeout(()=>{cancelled=true;reject(new Error('Scenario runtime limit exceeded'));},Math.max(1,deadline-Date.now()));})]);
    const failed=report.runs.some(run=>run.status!=='pass');
    report.status=failed?'fail':'complete';report.checks=failed?'fail':scenario.assertions.length?'pass':'limited';
    if(!scenario.assertions.length)report.limits.push('No state assertions declared; this run is capture/diagnostics only, not QA passed.');
    exitCode=failed?1:0;
  } catch(error) {report.error=String(error).slice(0,4096);if(browser){report.status='fail';report.checks='fail';exitCode=1;}}
  finally {
    clearTimeout(timer);cancelled=true;
    if(browser)try{await browser.close();}catch(error){report.cleanup_error=String(error);exitCode=1;report.status='fail';report.checks='fail';}
    for(const run of report.runs)if(run.status==='running'){run.status='fail';run.error ??= report.error ?? 'Run interrupted';}
    report.finished_at=new Date().toISOString();
    report.duration_ms=Date.parse(report.finished_at)-Date.parse(report.started_at);
    await fs.writeFile(path.join(output,'report.json'),JSON.stringify(report,null,2)+'\n',{flag:'wx',mode:0o600});
  }
  console.log(JSON.stringify({status:report.status,checks:report.checks,visual_review:report.visual_review,report:path.join(output,'report.json')}));
  return exitCode;
}
main().then(code=>{process.exitCode=code;}).catch(error=>{console.error(String(error));process.exitCode=2;});
