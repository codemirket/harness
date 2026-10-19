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
