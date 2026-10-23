import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';

test('shipped Chromium meets the reviewed security floor and retired video dependencies are absent', {skip: process.env.MIRKET_LIVE_BROWSER_TEST !== '1'}, async () => {
  const {execFile} = await import('node:child_process');
  const {promisify} = await import('node:util');
  const run = promisify(execFile);
  const manifest = JSON.parse(await fs.readFile(new URL('../browser.json', import.meta.url), 'utf8'));
  const key = process.arch === 'arm64' ? 'arm64' : 'amd64';
  const {stdout} = await run(path.join('/opt/design/chrome', manifest.archives[key].executable), ['--version']);
  assert.ok(stdout.includes(manifest.version), stdout);
  const playwright = JSON.parse(await fs.readFile(new URL('../node_modules/playwright-core/package.json', import.meta.url), 'utf8'));
  assert.equal(playwright.version, process.env.MIRKET_PLAYWRIGHT_VERSION);
  const bundled = JSON.parse(await fs.readFile(new URL('../node_modules/playwright-core/browsers.json', import.meta.url), 'utf8')).browsers.find(x=>x.name==='chromium');
  assert.deepEqual(bundled.browserVersion.split('.').slice(0,3), manifest.version.split('.').slice(0,3));
  assert.ok(Number(manifest.version.split('.')[3]) >= Number(bundled.browserVersion.split('.')[3]), 'Never downgrade a security patch to the SDK bundled revision');
  assert.equal(manifest.version, process.env.MIRKET_PLAYWRIGHT_CHROMIUM_VERSION);
  assert.equal(bundled.revision, process.env.MIRKET_PLAYWRIGHT_CHROMIUM_REVISION);
  for (const name of ['esbuild','hyperframes']) {
    await assert.rejects(import(name), {code:'ERR_MODULE_NOT_FOUND'});
  }
  await assert.rejects(fs.access('/usr/bin/ffmpeg'));
  await assert.rejects(fs.access('/usr/bin/ffprobe'));
});

// The development gate owns a fresh workspace and loopback HTTP server inside
// its disposable container. Existing service-based invocation stays supported.
let isolatedServer;
if (process.env.MIRKET_ISOLATED_TEST === '1') {
  test.before(async () => {
    const {workspace} = await import('../paths.mjs');
    await fs.mkdir(path.join(workspace, 'design'), {recursive:true});
    await fs.writeFile(path.join(workspace, 'design/tokens.json'), JSON.stringify({color:{accent:{$type:'color',$value:'#345678'}}}));
    const {handleWorkspaceRequest} = await import('../workspace-server.mjs');
    const {createServer} = await import('node:http');
    isolatedServer = createServer(handleWorkspaceRequest);
    await new Promise((resolve,reject) => {
      isolatedServer.once('error', reject);
      isolatedServer.listen(9002, '0.0.0.0', resolve);
    });
  });
  test.after(async () => {
    if (isolatedServer) await new Promise(resolve => isolatedServer.close(resolve));
  });
}

// Opt in only inside a running tools service with its real workspace HTTP server.
test('D2 SVG renders and receives visual QA without an HTML accessibility claim', {skip: process.env.MIRKET_LIVE_BROWSER_TEST !== '1', timeout: 90000}, async () => {
  const {workspace} = await import('../paths.mjs');
  const {d2Render} = await import('../operations.mjs');
  const relative = `qa/d2-svg-${crypto.randomUUID()}`;
  const root = path.join(workspace, relative);
  let screenshot;
  await fs.mkdir(root, {recursive:true});
  try {
    await fs.writeFile(path.join(root, 'diagram.d2'), 'codex -> mirket: routes\nmirket -> design: renders\n');
    const result = await d2Render({source:`${relative}/diagram.d2`, output:`${relative}/diagram.svg`});
    assert.equal(result.qa.status, 'captured');
    assert.equal(result.qa.accessibility.status, 'not_applicable');
    assert.equal(result.qa.evidence.accessibility.status, 'not_applicable');
    const svg = await fs.readFile(path.join(root, 'diagram.svg'), 'utf8');
    assert.match(svg, /codex/);
    assert.match(svg, /design/);
    screenshot = result.qa.screenshot.path;
    assert.equal(crypto.createHash('sha256').update(await fs.readFile(path.join(workspace, screenshot))).digest('hex'), result.qa.screenshot.sha256);
  } finally {
    if (screenshot) await fs.rm(path.join(workspace, screenshot), {force:true});
    await fs.rm(root, {recursive:true,force:true});
  }
});

test('real Chromium captures viewport and full page at exactly 2x and seeks reproducibly', {skip: process.env.MIRKET_LIVE_BROWSER_TEST !== '1', timeout: 90000}, async () => {
  const {workspace} = await import('../paths.mjs');
  const {browserScreenshot, odiffCompare} = await import('../operations.mjs');
  const relative = `qa/browser-regression-${crypto.randomUUID()}`;
  const root = path.join(workspace, relative);
  await fs.mkdir(root, {recursive: true});
  try {
    const {compileTokens} = await import('../tokens.mjs');
    const compiled = await compileTokens();
    await fs.writeFile(path.join(root, 'index.html'), '<!doctype html><html><head><link rel="stylesheet" href="/files/'+compiled.outputs['tokens.css']+'" ><style>body{margin:0}#token{display:none}main{height:500px;background:#123456}.orb{width:30px;height:30px;background:coral;animation:move 1s linear both}@keyframes move{to{transform:translateX(200px)}}</style></head><body><main><div class="orb"></div></main><span id="token"></span><script>document.querySelector("#token").textContent=getComputedStyle(document.documentElement).getPropertyValue("--color-accent").trim()</script></body></html>');
    const args = {source: `${relative}/index.html`, width:400, height:240, animation_time_ms:500, inspect_selectors:['#token']};
    const viewport = await browserScreenshot({...args, output:`${relative}/viewport.png`});
    assert.ok(viewport.evidence.selectors['#token'].text, 'canonical token stylesheet failed to load');
    assert.deepEqual(viewport.evidence.console_errors, []);
    assert.deepEqual([viewport.width, viewport.height], [800,480]);
    if(process.env.MIRKET_ISOLATED_TEST==='1') console.log('BROWSER_EVIDENCE '+JSON.stringify({png:(await fs.readFile(path.join(root,'viewport.png'))).toString('base64'),evidence:viewport.evidence}));
    const full = await browserScreenshot({...args, full_page:true, output:`${relative}/full.png`});
    assert.deepEqual([full.width, full.height], [800,1000]);
    await browserScreenshot({...args, output:`${relative}/repeat.png`});
    assert.equal((await odiffCompare({baseline:`${relative}/viewport.png`, candidate:`${relative}/repeat.png`})).match,true);
  } finally { await fs.rm(root, {recursive:true,force:true}); }
});

test('capture observes unmodified HTML without a token source', {skip: process.env.MIRKET_LIVE_BROWSER_TEST !== '1', timeout: 90000}, async () => {
  const {workspace} = await import('../paths.mjs');
  const {browserScreenshot} = await import('../operations.mjs');
  const relative = `qa/html-routing-${crypto.randomUUID()}`;
  const root = path.join(workspace, relative);
  await fs.mkdir(root, {recursive:true});
  try {
    await fs.writeFile(path.join(root, 'invalid.html'), '<script>window.early=true</script><html><head></head><body>Invalid prologue</body></html>');
    await browserScreenshot({source:`${relative}/invalid.html`, output:`${relative}/invalid.png`, width:400, height:240, tokens:'missing.json'});
    await fs.writeFile(path.join(root, 'implicit.html'), '<!doctype html><html lang="en"><meta charset="utf-8"><title>Implicit head</title><div id="result"></div><script>const markup="<head>";document.querySelector("#result").textContent=markup+getComputedStyle(document.documentElement).getPropertyValue("--color-accent").trim()</script></html>');
    const captured = await browserScreenshot({source:`${relative}/implicit.html`, output:`${relative}/implicit.png`, width:400, height:240, inspect_selectors:['#result']});
    assert.ok(captured.evidence.selectors['#result'].text.startsWith('<head>'));
    assert.equal(captured.evidence.selectors['#result'].text, '<head>', 'capture must not inject a stylesheet');
    assert.equal(captured.tokens, undefined);
    assert.deepEqual(captured.evidence.console_errors, []);
  } finally { await fs.rm(root, {recursive:true,force:true}); }
});

test('scoped Chromium and file operations cannot read a sibling project', {skip: process.env.MIRKET_LIVE_BROWSER_TEST !== '1', timeout:90000}, async () => {
  const {execFile} = await import('node:child_process');
  const {promisify} = await import('node:util');
  const {workspace} = await import('../paths.mjs');
  const relative = `qa/scope-regression-${crypto.randomUUID()}`;
  const root = path.join(workspace, relative);
  await fs.mkdir(path.join(root, 'own/design'), {recursive:true});
  await fs.mkdir(path.join(root, 'sibling'));
  try {
    await fs.writeFile(path.join(root, 'sibling/secret.txt'), 'SIBLING_PRIVATE_SENTINEL');
    const sibling = `/files/${relative}/sibling/secret.txt`;
    await fs.writeFile(path.join(root, 'own/index.html'), `<!doctype html><html><head></head><body><div id="result">pending</div><script>fetch(${JSON.stringify(sibling)}).then(r=>r.text()).then(t=>{document.querySelector('#result').textContent=t}).catch(()=>{document.querySelector('#result').textContent='blocked'}).finally(()=>{document.querySelector('#result').setAttribute('data-done','yes')})</script></body></html>`);
    const operations = new URL('../operations.mjs', import.meta.url).href;
    const paths = new URL('../paths.mjs', import.meta.url).href;
    const script = `import assert from 'node:assert/strict';
      const {browserScreenshot} = await import(${JSON.stringify(operations)});
      const {readFile} = await import(${JSON.stringify(paths)});
      await assert.rejects(readFile('../sibling/secret.txt'), /traversal/);
      await assert.rejects(browserScreenshot({url:${JSON.stringify('http://design-tools:9002'+sibling)},output:'rejected.png'}), /outside the assigned/);
      const result = await browserScreenshot({source:'index.html',output:'scoped.png',width:400,height:240,wait_for:'#result[data-done]',inspect_selectors:['#result']});
      assert.equal(result.evidence.selectors['#result'].text,'blocked');
      assert.equal(result.width,800); assert.equal(result.height,480);
      console.log(JSON.stringify({scope:process.env.MIRKET_DESIGN_WORKSPACE,sibling_blocked:true,pixels:[result.width,result.height]}));`;
    const result = await promisify(execFile)(process.execPath, ['--input-type=module','-e',script], {env:{...process.env,MIRKET_DESIGN_WORKSPACE:path.join(root,'own'),MIRKET_SCOPED_WORKER:'1'},timeout:75000,maxBuffer:1024*1024});
    assert.ok(result.stdout.includes('"sibling_blocked":true'));
  } finally { await fs.rm(root, {recursive:true,force:true}); }
});


test('actual axe checks captured interaction state, keyboard focus and reduced motion', {skip:process.env.MIRKET_LIVE_BROWSER_TEST!=='1', timeout:90000}, async()=>{
  const {workspace}=await import('../paths.mjs');
  const {browserScreenshot}=await import('../operations.mjs');
  const relative=`qa/accessibility-${crypto.randomUUID()}`;
  const root=path.join(workspace,relative);
  await fs.mkdir(root,{recursive:true});
  try {
    await fs.writeFile(path.join(root,'index.html'), `<!doctype html><html lang="en"><head><title>Keyboard fixture</title><style>body{color:#111;background:#fff}button:focus{outline:3px solid #036}.orb{animation:move 1s infinite}@keyframes move{to{transform:translateX(100px)}}@media(prefers-reduced-motion:reduce){.orb{animation:none}}</style></head><body><main><h1>Keyboard fixture</h1><button id="control" onclick="this.removeAttribute('aria-label');this.textContent=''">Show details</button><p class="orb" id="motion">Motion fixture</p></main></body></html>`);
    const good=await browserScreenshot({source:`${relative}/index.html`,output:`${relative}/good.png`,width:400,height:240,reduced_motion:'reduce',actions:[{kind:'press',selector:'body',key:'Tab'}],inspect_selectors:['#motion']});
    assert.equal(good.evidence.active_element.id,'control');
    assert.equal(good.evidence.selectors['#motion'].transform,'none');
    assert.equal(good.evidence.accessibility.version,'4.14.0');
    assert.deepEqual(good.evidence.accessibility.violations,[]);
    const bad=await browserScreenshot({source:`${relative}/index.html`,output:`${relative}/bad.png`,width:400,height:240,actions:[{kind:'click',selector:'#control'}]});
    assert.ok(bad.evidence.accessibility.violations.some(x=>x.id==='button-name'));
    assert.equal(bad.qa.accessibility.status,'failed');
    assert.equal(bad.qa.status,'failed');
  } finally { await fs.rm(root,{recursive:true,force:true}); }
});


test('malicious HTML cannot read host files, contact companions or write job inputs', {skip:process.env.MIRKET_LIVE_BROWSER_TEST !== '1',timeout:90000}, async()=>{
  const {workspace}=await import('../paths.mjs');
  const {browserScreenshot}=await import('../operations.mjs');
  const relative=`qa/hostile-${crypto.randomUUID()}`;
  const root=path.join(workspace,relative);await fs.mkdir(root,{recursive:true});
  const html=`<!doctype html><html lang="en"><head><title>Isolation probe</title></head><body><div id="result">pending</div><script>
    const urls=['file:///etc/passwd','http://untrusted.internal:8000/health','http://host.containers.internal/','http://127.0.0.1:8000/'];
    Promise.all(urls.map(url=>fetch(url).then(r=>r.ok?'LEAK':'blocked').catch(()=>'blocked'))).then(async results=>{
      const write=await fetch(location.href,{method:'PUT',body:'MODIFIED'}).then(r=>r.ok?'WRITE':'blocked').catch(()=>'blocked');
      document.querySelector('#result').textContent=results.concat(write).join(',');document.querySelector('#result').dataset.done='yes';
    });
  </script></body></html>`;
  await fs.writeFile(path.join(root,'index.html'),html);
  try {
    const result=await browserScreenshot({source:`${relative}/index.html`,output:`${relative}/result.png`,width:400,height:240,wait_for:'#result[data-done]',inspect_selectors:['#result']});
    assert.equal(result.evidence.selectors['#result'].text,'blocked,blocked,blocked,blocked,blocked');
    assert.match(result.evidence.sandbox,/Layer 1 Sandbox\s+Namespace/);
    assert.match(result.evidence.sandbox,/Seccomp-BPF sandbox\s+Yes/);
    assert.equal(await fs.readFile(path.join(root,'index.html'),'utf8'),html);
  } finally {await fs.rm(root,{recursive:true,force:true});}
});
