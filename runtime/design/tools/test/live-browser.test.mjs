import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';

// Opt in only inside a running tools service with its real workspace HTTP server.
test('real Chromium captures viewport and full page at exactly 2x and seeks reproducibly', {skip: process.env.MIRKET_LIVE_BROWSER_TEST !== '1', timeout: 90000}, async () => {
  const {workspace} = await import('../paths.mjs');
  const {browserScreenshot, odiffCompare} = await import('../operations.mjs');
  const relative = `qa/browser-regression-${crypto.randomUUID()}`;
  const root = path.join(workspace, relative);
  await fs.mkdir(root, {recursive: true});
  try {
    await fs.writeFile(path.join(root, 'index.html'), '<!doctype html><html><head><style>body{margin:0}#token{display:none}main{height:500px;background:#123456}.orb{width:30px;height:30px;background:coral;animation:move 1s linear both}@keyframes move{to{transform:translateX(200px)}}</style></head><body><main><div class="orb"></div></main><span id="token"></span><script>document.querySelector("#token").textContent=getComputedStyle(document.documentElement).getPropertyValue("--color-accent").trim()</script></body></html>');
    const args = {source: `${relative}/index.html`, width:400, height:240, animation_time_ms:500, inspect_selectors:['#token']};
    const viewport = await browserScreenshot({...args, output:`${relative}/viewport.png`});
    assert.ok(viewport.evidence.selectors['#token'].text, 'canonical token stylesheet failed to load');
    assert.deepEqual(viewport.evidence.console_errors, []);
    assert.deepEqual([viewport.width, viewport.height], [800,480]);
    const full = await browserScreenshot({...args, full_page:true, output:`${relative}/full.png`});
    assert.deepEqual([full.width, full.height], [800,1000]);
    await browserScreenshot({...args, output:`${relative}/repeat.png`});
    assert.equal((await odiffCompare({baseline:`${relative}/viewport.png`, candidate:`${relative}/repeat.png`})).match,true);
  } finally { await fs.rm(root, {recursive:true,force:true}); }
});

test('HTML inspection handles literal head tags and contains route failures without killing the session', {skip: process.env.MIRKET_LIVE_BROWSER_TEST !== '1', timeout: 90000}, async () => {
  const {workspace} = await import('../paths.mjs');
  const {browserScreenshot} = await import('../operations.mjs');
  const relative = `qa/html-routing-${crypto.randomUUID()}`;
  const root = path.join(workspace, relative);
  await fs.mkdir(root, {recursive:true});
  try {
    await fs.writeFile(path.join(root, 'invalid.html'), '<script>window.early=true</script><html><head></head><body>Invalid prologue</body></html>');
    await assert.rejects(browserScreenshot({source:`${relative}/invalid.html`, output:`${relative}/invalid.png`, width:400, height:240}), /before the canonical token stylesheet/);
    await assert.rejects(fs.access(path.join(root, 'invalid.png')));
    await fs.writeFile(path.join(root, 'implicit.html'), '<!doctype html><html lang="en"><meta charset="utf-8"><title>Implicit head</title><div id="result"></div><script>const markup="<head>";document.querySelector("#result").textContent=markup+getComputedStyle(document.documentElement).getPropertyValue("--color-accent").trim()</script></html>');
    const captured = await browserScreenshot({source:`${relative}/implicit.html`, output:`${relative}/implicit.png`, width:400, height:240, inspect_selectors:['#result']});
    assert.ok(captured.evidence.selectors['#result'].text.startsWith('<head>'));
    assert.ok(captured.evidence.selectors['#result'].text.length > '<head>'.length, 'tokens must load before the script');
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
    await fs.writeFile(path.join(root, 'own/design/tokens.json'), JSON.stringify({color:{accent:{$type:'color',$value:'#345678'}}}));
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
