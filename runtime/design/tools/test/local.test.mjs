import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {spawn, spawnSync} from 'node:child_process';

const workspace = await fs.mkdtemp(path.join(os.tmpdir(), 'mirket-design-test-'));
process.env.MIRKET_DESIGN_WORKSPACE = workspace;
const {publish, readFile, run, readBoundedFile, regularOutput, sha256} = await import('../paths.mjs');
const {compileTokens} = await import('../tokens.mjs');
const {svgOptimize, staticRender, prepareSatoriTree, odiffCompare, imageBatchBudget, injectTokenStylesheet, inspectHyperframesKeyframes, optimizeSvgBytes, hyperframesRender, typstCompile, readBoundedProjectAsset, parseHyperframesLint, browserInspectCaptureOptions, screenshotPixelPlan, verifyScreenshotPixels, fullPageCaptureParams} = await import('../operations.mjs');
const {withRequest} = await import('../request.mjs');
const {penpotTokenRecords, synchronizePenpotTokens} = await import('../penpot-tokens.mjs');
const {exportPenpotShape} = await import('../penpot-export.mjs');

test.after(async () => fs.rm(workspace, {recursive: true, force: true}));

test('simultaneous absent-output publication has one atomic winner', async () => {
  const results = await Promise.allSettled([
    publish('race.txt', Buffer.from('alpha')),
    publish('race.txt', Buffer.from('bravo')),
  ]);
  assert.equal(results.filter(item => item.status === 'fulfilled').length, 1);
  assert.equal(results.filter(item => item.status === 'rejected').length, 1);
  assert.ok(['alpha', 'bravo'].includes((await readFile('race.txt')).toString()));
});

test('canonical token generation is complete, reused and detects drift', async () => {
  await fs.mkdir(path.join(workspace, 'design'));
  await fs.writeFile(path.join(workspace, 'design/tokens.json'), JSON.stringify({color: {accent: {$type: 'color', $value: '#123456'}}}));
  const first = await compileTokens('design/tokens.json');
  assert.deepEqual(Object.keys(first.outputs).sort(), ['penpot.tokens.json', 'tokens.css', 'tokens.js', 'tokens.ts', 'tokens.typ']);
  assert.equal((await compileTokens('design/tokens.json')).reused, true);
  const loader = path.join(workspace, 'no-compiler-loader.mjs');
  await fs.writeFile(loader, `export async function resolve(specifier, context, next) {
    if (specifier === 'style-dictionary') throw Error('Cached tokens loaded the compiler');
    return next(specifier, context);
  }`);
  const cached = spawnSync(process.execPath, ['--loader', loader, '--input-type=module', '-e', `
    const {compileTokens} = await import(${JSON.stringify(new URL('../tokens.mjs', import.meta.url).href)});
    if (!(await compileTokens('design/tokens.json')).reused) throw Error('Expected verified cache');
  `], {encoding: 'utf8', timeout: 5000});
  assert.equal(cached.status, 0, cached.stderr);
  await fs.writeFile(path.join(workspace, first.outputs['tokens.css']), 'drift');
  await assert.rejects(compileTokens('design/tokens.json'), /drifted/);
  await fs.writeFile(path.join(workspace, 'design/tokens.json'), JSON.stringify({color: {accent: {$type: 'color', $value: '#654321'}}}));
});

test('unchanged tokens rebuild after compiler recipe or dependency identity changes', async () => {
  const fixture = await fs.mkdtemp(path.join(os.tmpdir(), 'mirket-token-recipe-'));
  try {
    for (const name of ['tokens.mjs', 'paths.mjs', 'request.mjs', 'package-lock.json'])
      await fs.copyFile(new URL(`../${name}`, import.meta.url), path.join(fixture, name));
    await fs.symlink(new URL('../node_modules', import.meta.url).pathname, path.join(fixture, 'node_modules'), 'dir');
    await fs.writeFile(path.join(workspace, 'recipe-tokens.json'), JSON.stringify({color: {accent: {$type:'color', $value:'#abcdef'}}}));
    const module = new URL(`file://${fixture}/tokens.mjs`);
    const first = await (await import(`${module}?v=1`)).compileTokens('recipe-tokens.json');
    const original = await fs.readFile(path.join(fixture, 'tokens.mjs'), 'utf8');
    await fs.writeFile(path.join(fixture, 'tokens.mjs'), original.replace('export const tokens =', '/* revised format */ export const tokens ='));
    const secondCompiler = (await import(`${module}?v=2`)).compileTokens;
    const second = await secondCompiler('recipe-tokens.json');
    assert.equal(second.source_sha256, first.source_sha256);
    assert.notEqual(second.compiler_sha256, first.compiler_sha256);
    assert.notEqual(second.outputs['tokens.js'], first.outputs['tokens.js']);
    assert.equal(second.reused, false);
    assert.match(await fs.readFile(path.join(workspace, second.outputs['tokens.js']), 'utf8'), /revised format/);
    assert.equal((await secondCompiler('recipe-tokens.json')).reused, true);
    await fs.appendFile(path.join(fixture, 'package-lock.json'), '\n');
    const third = await (await import(`${module}?v=3`)).compileTokens('recipe-tokens.json');
    assert.equal(third.source_sha256, second.source_sha256);
    assert.notEqual(third.compiler_sha256, second.compiler_sha256);
    assert.equal(third.reused, false);
  } finally { await fs.rm(fixture, {recursive:true, force:true}); }
});

test('concurrent cold token compilation shares one complete generation', async () => {
  await fs.mkdir(path.join(workspace, 'design'), {recursive:true});
  await fs.writeFile(path.join(workspace, 'design/concurrent.json'), JSON.stringify({color:{race:{$type:'color',$value:'#135724'}}}));
  const results = await Promise.allSettled(Array.from({length:4}, () => compileTokens('design/concurrent.json')));
  assert.deepEqual(results.filter(x => x.status === 'rejected').map(x => x.reason.message), []);
  assert.equal(new Set(results.map(x => x.value.source_sha256)).size, 1);
  assert.equal((await compileTokens('design/concurrent.json')).reused, true);
});

test('Style Dictionary composite shadow values retain object and array structure in Typst', async () => {
  await fs.mkdir(path.join(workspace, 'design'), {recursive: true});
  const shadow = color => ({color, offsetX: '2px', offsetY: '4px', blur: '6px', spread: '1px'});
  await fs.writeFile(path.join(workspace, 'design/composite-tokens.json'), JSON.stringify({
    effect: {
      card: {$type: 'shadow', $value: shadow('#123456')},
      stack: {$type: 'shadow', $value: [shadow('#abcdef'), shadow('#fedcba')]},
    },
  }));
  const compiled = await compileTokens('design/composite-tokens.json');
  const js = (await readFile(compiled.outputs['tokens.js'])).toString();
  const typst = (await readFile(compiled.outputs['tokens.typ'])).toString();
  assert.match(js, /"effect\.card": \{/);
  assert.match(js, /"effect\.stack": \[/);
  assert.match(typst, /\("effect\.card", \(\("color", "#123456"\)/);
  assert.match(typst, /\("effect\.stack", \(\(\("color", "#abcdef"\)/);
  assert.match(typst, /\("color", "#fedcba"\)/);
  assert.match(typst, /\.to-dict\(\)/);
  assert.doesNotMatch(typst, /\[object Object\]/);
});

test('Playwright HTML receives canonical CSS before head scripts run', () => {
  const css = 'http://design-tools:9002/files/.mirket-design/tokens/abc/tokens.css';
  const document = '<!doctype html><html><head><script>window.initial = getComputedStyle(document.documentElement).getPropertyValue("--color-accent")</script></head><body>Probe</body></html>';
  const injected = injectTokenStylesheet(document, css);
  assert.match(injected, /data-mirket-tokens/);
  assert.ok(injected.indexOf(`href="${css}"`) < injected.indexOf('<script>'));
  assert.ok(injected.startsWith('<!doctype html><html><head>'));
  assert.match(injectTokenStylesheet('<html><body>Probe</body></html>', css), /<html><head><link[^>]+><\/head><body>/);
  const implicit = '<!doctype html><html lang="en"><meta charset="utf-8"><script>const literal = "<head>";</script></html>';
  const implicitResult = injectTokenStylesheet(implicit, css);
  assert.ok(implicitResult.indexOf('data-mirket-tokens') < implicitResult.indexOf('<meta'));
  assert.ok(implicitResult.endsWith('<script>const literal = "<head>";</script></html>'));
  const comments = '<!doctype html><!-- <head><script>fake</script> --><html data-label="<head>"><!-- <script> --><head title=">">';
  assert.equal(injectTokenStylesheet(comments, css), `${comments}<link rel="stylesheet" data-mirket-tokens href="${css}">`);
  assert.throws(() => injectTokenStylesheet('<!-- unfinished', css), /unterminated/);
  assert.throws(() => injectTokenStylesheet('<script>window.initial = "too early"</script><html><head></head></html>', css), /before the canonical token stylesheet/);
});

test('full-page QA uses Chromium content size to enforce a physical-pixel budget', () => {
  assert.deepEqual(browserInspectCaptureOptions, {width: 800, height: 600, full_page: true});
  const plan = screenshotPixelPlan({width: 800, height: 600, fullPage: true, cssContentSize: {width: 800, height: 734}});
  assert.deepEqual(plan, {pixelWidth: 1600, pixelHeight: 1468, clipWidth: 800, clipHeight: 734});
  assert.deepEqual(fullPageCaptureParams(plan), {format: 'png', fromSurface: true, captureBeyondViewport: true,
    clip: {x: 0, y: 0, width: 800, height: 734, scale: 2}});
  verifyScreenshotPixels(1600, 1468, plan);
  assert.throws(() => verifyScreenshotPixels(1600, 1200, plan), /dimensions differ/);
  assert.throws(() => screenshotPixelPlan({width: 800, height: 600, fullPage: true, cssContentSize: {width: 800, height: 6000}}), /16 Mi pixel budget/);
  assert.throws(() => screenshotPixelPlan({width: 800, height: 600, fullPage: true, cssContentSize: {width: NaN, height: 734}}), /content metrics are invalid/);
  const viewport = screenshotPixelPlan({width: 800, height: 600, fullPage: false});
  assert.throws(() => verifyScreenshotPixels(1600, 1199, viewport), /dimensions differ/);
  assert.throws(() => verifyScreenshotPixels(4000, 5000, plan), /16 Mi pixel budget/);
});

test('HyperFrames inspects every keyframe before a failed QA gate can render', async () => {
  const snapshots = ['frame-00.png', 'frame-01.png', 'frame-02.png'].map(name => ({artifact: {path: name}}));
  const seen = [];
  const qa = await inspectHyperframesKeyframes(snapshots, 'baseline.png', async (source, baseline) => {
    seen.push([source, baseline]);
    return {status: source === 'frame-01.png' ? 'failed' : 'captured', artifact_retained: source};
  });
  assert.deepEqual(seen, [
    ['frame-00.png', 'baseline.png'], ['frame-01.png', undefined], ['frame-02.png', undefined],
  ]);
  assert.equal(qa.status, 'failed');
  assert.equal(qa.keyframes.length, 3);
  assert.deepEqual(qa.keyframes.map(item => item.source), snapshots.map(item => item.artifact.path));
});

test('HyperFrames lint requires a valid zero-error verdict before snapshots', () => {
  const good = {ok: true, errorCount: 0, warningCount: 0, findings: [], filesScanned: 1};
  assert.deepEqual(parseHyperframesLint(JSON.stringify(good)), good);
  assert.throws(() => parseHyperframesLint('warning before {"ok":true,"errorCount":0}'), /invalid JSON/);
  assert.throws(() => parseHyperframesLint('{"ok":true}'), /invalid verdict/);
  assert.throws(() => parseHyperframesLint('{"ok":true,"errorCount":"0"}'), /invalid verdict/);
  assert.throws(() => parseHyperframesLint('{"ok":false,"errorCount":0}'), /reported errors/);
  assert.throws(() => parseHyperframesLint('{"ok":true,"errorCount":1}'), /reported errors/);
});

test('oversized sparse assets are rejected before HyperFrames or Typst launches', async () => {
  const motion = path.join(workspace, 'oversized-motion');
  const paged = path.join(workspace, 'oversized-paged');
  await fs.mkdir(motion);
  await fs.mkdir(paged);
  await fs.writeFile(path.join(motion, 'index.html'), '<html><head></head><body>test</body></html>');
  await fs.writeFile(path.join(paged, 'document.typ'), '= Test\n');
  for (const folder of [motion, paged]) {
    const handle = await fs.open(path.join(folder, 'oversized.bin'), 'wx');
    try { await handle.truncate(64 * 1024 * 1024 + 1); }
    finally { await handle.close(); }
  }
  await assert.rejects(readBoundedProjectAsset(path.join(motion, 'oversized.bin'), 64 * 1024 * 1024, 'Composition'), /Composition exceeds 64 MiB/);
  await assert.rejects(hyperframesRender({project: 'oversized-motion', output: 'oversized.mp4'}), /Composition exceeds 64 MiB/);
  await assert.rejects(typstCompile({source: 'oversized-paged/document.typ', output: 'oversized.pdf'}), /Typst project exceeds 64 MiB/);
  await assert.rejects(fs.stat(path.join(workspace, 'oversized.mp4')), {code: 'ENOENT'});
  await assert.rejects(fs.stat(path.join(workspace, 'oversized.pdf')), {code: 'ENOENT'});
});

test('Typst cannot read an unprepared absolute workspace asset', async t => {
  try { await fs.access('/usr/local/bin/typst'); } catch { t.skip('Typst runtime unavailable'); return; }
  await fs.writeFile(path.join(workspace, 'outside-typst-stage.txt'), 'outside');
  await fs.mkdir(path.join(workspace, 'bounded-paged'));
  await fs.writeFile(path.join(workspace, 'bounded-paged/document.typ'), '#read("/outside-typst-stage.txt")\n');
  await assert.rejects(typstCompile({source: 'bounded-paged/document.typ', output: 'bounded.pdf'}), error => {
    assert.match(error.message, /outside-typst-stage\.txt/);
    return true;
  });
  await assert.rejects(fs.stat(path.join(workspace, 'bounded.pdf')), {code: 'ENOENT'});
});


test('malicious imported SVG is rejected before publication', async () => {
  await fs.writeFile(path.join(workspace, 'unsafe.svg'), '<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>');
  await assert.rejects(svgOptimize({source: 'unsafe.svg', output: 'clean.svg', tokens: 'design/tokens.json'}), /Unsafe SVG/);
  await assert.rejects(fs.stat(path.join(workspace, 'clean.svg')), {code: 'ENOENT'});
});

test('standard Graphviz SVG 1.1 header is stripped without resolving arbitrary DTDs', () => {
  const header = '<?xml version="1.0" encoding="UTF-8" standalone="no"?>\n<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN"\n "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">\n';
  const body = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 50"><text x="2" y="20">Graphviz</text></svg>';
  const optimized = optimizeSvgBytes(header + body).toString('utf8');
  assert.match(optimized, /<svg/);
  assert.match(optimized, /Graphviz/);
  assert.doesNotMatch(optimized, /DOCTYPE|ENTITY/);
  assert.throws(() => optimizeSvgBytes(header.replace('svg11.dtd', 'other.dtd') + body), /DTD and entity declarations are refused/);
  assert.throws(() => optimizeSvgBytes(header.replace('svg11.dtd">', 'svg11.dtd" [<!ENTITY xxe SYSTEM "file:\/\/\/etc\/passwd">]>') + body), /DTD and entity declarations are refused/);
  assert.throws(() => optimizeSvgBytes('<!DOCTYPE svg SYSTEM "file:///etc/passwd">' + body), /DTD and entity declarations are refused/);
});

test('known output conflicts stop HyperFrames before external work', async () => {
  await fs.writeFile(path.join(workspace, 'occupied.png'), 'existing-output');
  await fs.writeFile(path.join(workspace, 'occupied.mp4'), 'existing-output');
  await fs.writeFile(path.join(workspace, 'empty.png'), '');
  const originalFetch = global.fetch;
  let requests = 0;
  global.fetch = () => { requests++; throw Error('External service was touched'); };
  try {
    await assert.rejects(hyperframesRender({project: 'missing-project', output: 'occupied.mp4'}), /Output exists; pass overwrite=true/);
    assert.equal(requests, 0);
  } finally { global.fetch = originalFetch; }
  assert.equal((await fs.readFile(path.join(workspace, 'occupied.png'), 'utf8')), 'existing-output');
});

test('child timeout stops the process group and reports timeout', async () => {
  await assert.rejects(run(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], {timeoutMs: 100}), /timed out/);
});

test('TERM-resistant noisy child retains bounded diagnostics while both pipes drain', async () => {
  const code = 'process.on("SIGTERM",()=>{});const block=Buffer.alloc(65536,65);let count=0;const writer=setInterval(()=>{if(count++<192){process.stdout.write(block);process.stderr.write(block)}else clearInterval(writer)},1);setInterval(()=>{},1000);';
  const before = process.memoryUsage().external;
  let peak = before;
  const sample = setInterval(() => { peak = Math.max(peak, process.memoryUsage().external); }, 5);
  try {
    await assert.rejects(run(process.execPath, ['-e', code], {maxStdout: 1024, maxStderr: 1024, timeoutMs: 10000}), error => {
      assert.match(error.message, /Process (?:stdout|stderr) exceeded limit/);
      assert.ok(error.message.length < 1200, 'diagnostic itself remains bounded');
      return true;
    });
  } finally {
    peak = Math.max(peak, process.memoryUsage().external);
    clearInterval(sample);
  }
  assert.ok(peak - before < 64 * 1024 * 1024, 'parent retained more than the bounded pipe captures');
});

test('a cancelled tool request kills only its child process', async () => {
  const controller = new AbortController();
  const cancelled = withRequest(controller.signal, () => run(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], {timeoutMs: 5000}));
  setTimeout(() => controller.abort(), 50);
  await assert.rejects(cancelled, /cancelled/);
  const survivor = await run(process.execPath, ['-e', 'process.stdout.write("survived")']);
  assert.equal(survivor.stdout, 'survived');
});

test('cancellation removes a detached grandchild after its parent ignores SIGTERM', {skip: process.platform === 'win32'}, async () => {
  const marker = `escaped-${process.pid}-${Date.now()}`;
  const script = path.join(workspace, `${marker}.mjs`);
  const pidFile = path.join(workspace, `${marker}.pid`);
  await fs.writeFile(script, 'setInterval(() => {}, 1000);\n');
  const parentCode = `const {spawn}=require('node:child_process'); const fs=require('node:fs'); const child=spawn(process.execPath,[${JSON.stringify(script)}],{detached:true,stdio:'ignore'}); child.unref(); fs.writeFileSync(${JSON.stringify(pidFile)},String(child.pid)); process.on('SIGTERM',()=>{}); setInterval(()=>{},1000);`;
  const controller = new AbortController();
  const cancelled = withRequest(controller.signal, () => run(process.execPath, ['-e', parentCode], {timeoutMs: 10000}));
  let grandchildPid;
  try {
    for (let attempt = 0; attempt < 100; attempt++) {
      try { grandchildPid = Number(await fs.readFile(pidFile, 'utf8')); break; }
      catch (error) { if (error.code !== 'ENOENT') throw error; }
      await new Promise(resolve => setTimeout(resolve, 20));
    }
    assert.ok(Number.isInteger(grandchildPid) && grandchildPid > 1, 'detached grandchild started');
    await new Promise(resolve => setTimeout(resolve, 100));
    controller.abort();
    await assert.rejects(cancelled, /cancelled/);
    const status = spawnSync('/bin/ps', ['-p', String(grandchildPid), '-o', 'stat='], {encoding: 'utf8'});
    assert.ok(status.status !== 0 || !status.stdout.trim() || /^Z/.test(status.stdout.trim()), 'detached grandchild stopped');
  } finally {
    controller.abort();
    await cancelled.catch(() => {});
    if (grandchildPid) {
      const current = spawnSync('/bin/ps', ['-p', String(grandchildPid), '-o', 'command='], {encoding: 'utf8'}).stdout.trim();
      if (current.includes(script)) { try { process.kill(grandchildPid, 'SIGKILL'); } catch {} }
    }
  }
});

test('renderer child receives no service credentials', async () => {
  process.env.MIRKET_PENPOT_PASSWORD = 'test-secret-12345';
  try {
    const result = await run(process.execPath, ['-e', 'process.stdout.write(String(process.env.MIRKET_PENPOT_PASSWORD || "absent"))']);
    assert.equal(result.stdout, 'absent');
  } finally { delete process.env.MIRKET_PENPOT_PASSWORD; }
});

test('renderer child uses private workspace disk scratch and removes it after exit', async () => {
  const code = 'const fs=require("node:fs");const path=require("node:path");fs.writeFileSync(path.join(process.env.TMPDIR,"probe"),"temporary");process.stdout.write(JSON.stringify({home:process.env.HOME,tmp:process.env.TMPDIR,cache:process.env.XDG_CACHE_HOME,telemetry:process.env.HYPERFRAMES_NO_TELEMETRY}));';
  const result = await run(process.execPath, ['-e', code]);
  const child = JSON.parse(result.stdout);
  assert.match(child.tmp, /[\\/]\.mirket-design[\\/]child-[^\\/]+$/);
  assert.equal(child.home, child.tmp);
  assert.equal(child.cache, path.join(child.tmp, 'cache'));
  assert.equal(child.telemetry, '1');
  await assert.rejects(fs.stat(child.tmp), {code: 'ENOENT'});
});

test('child scratch refuses a symlinked workspace staging directory', {skip: process.platform === 'win32'}, async () => {
  const isolated = await fs.mkdtemp(path.join(os.tmpdir(), 'mirket-design-symlink-'));
  const outside = await fs.mkdtemp(path.join(os.tmpdir(), 'mirket-design-outside-'));
  try {
    await fs.symlink(outside, path.join(isolated, '.mirket-design'));
    const moduleUrl = new URL('../paths.mjs', import.meta.url).href;
    const probe = `const {run}=await import(${JSON.stringify(moduleUrl)});try{await run(process.execPath,['-e','process.stdout.write("spawned")']);process.exitCode=2}catch(error){process.stdout.write(error.message)}`;
    const child = spawnSync(process.execPath, ['--input-type=module', '-e', probe], {
      env: {...process.env, MIRKET_DESIGN_WORKSPACE: isolated}, encoding: 'utf8', timeout: 5000,
    });
    assert.equal(child.status, 0, child.stderr);
    assert.match(child.stdout, /symlink/i);
    assert.deepEqual(await fs.readdir(outside), []);
  } finally {
    await fs.rm(isolated, {recursive: true, force: true});
    await fs.rm(outside, {recursive: true, force: true});
  }
});

test('Satori resolves a canonical token into its SVG output', async t => {
  const fontSource = process.platform === 'linux'
    ? '/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf'
    : '/System/Library/Fonts/Supplemental/Arial.ttf';
  try { await fs.access(fontSource); } catch { t.skip('Host font fixture unavailable'); return; }
  await fs.copyFile(fontSource, path.join(workspace, 'font.ttf'));
  const result = await staticRender({
    tokens: 'design/tokens.json', fonts: [{path: 'font.ttf'}], width: 320, height: 120,
    format: 'svg', output: 'token-render.svg',
    tree: {type: 'div', props: {style: {color: {$token: 'color.accent'}, width: 320, height: 120}}, children: 'Token rendered'},
  });
  assert.equal(result.artifact.path, 'token-render.svg');
  assert.match((await readFile('token-render.svg')).toString(), /#654321/i);
});

test('Satori renders a real WOFF2 glyph and rejects a mislabeled font', async () => {
  const font = new URL('../node_modules/style-dictionary/examples/complete/assets/fonts/MaterialIcons-Regular.woff2', import.meta.url);
  await fs.copyFile(font, path.join(workspace, 'icons.woff2'));
  const args = {
    fonts: [{path:'icons.woff2',name:'Icons'}], width:64, height:64, format:'svg', output:'woff2.svg',
    tree:{type:'div',props:{style:{fontFamily:'Icons',fontSize:48,color:'#123456'}},children:'\ue88a'},
  };
  const rendered = await staticRender(args);
  assert.ok(rendered.artifact.bytes > 100);
  const svg = (await readFile('woff2.svg')).toString();
  assert.match(svg, /<path/);
  assert.match(svg, /#123456/);
  await fs.writeFile(path.join(workspace, 'mislabeled.woff2'), Buffer.from('wOFF-invalid-font'));
  await assert.rejects(staticRender({...args, fonts:[{path:'mislabeled.woff2'}], output:'invalid-woff2.svg'}), /font bytes do not match/);
  await assert.rejects(fs.access(path.join(workspace, 'invalid-woff2.svg')));
});

test('Penpot token preflight maps supported types and rejects unsupported composites', async () => {
  assert.deepEqual(penpotTokenRecords({color: {accent: {$type: 'color', $value: '#fff'}}, font: {size: {$type: 'fontSize', $value: 16}}}), [
    {name: 'color.accent', type: 'color', value: '#fff'}, {name: 'font.size', type: 'fontSizes', value: '16'},
  ]);
  assert.throws(() => penpotTokenRecords({motion: {$type: 'duration', $value: '1s'}}), /cannot map/);
  assert.throws(() => penpotTokenRecords({text: {$type: 'typography', $value: {fontFamily: 'Arial'}}}), /cannot map|scalar/);
});

test('Penpot sync uses official execute_code and verifies acknowledgement', async () => {
  const compiled = await compileTokens('design/tokens.json');
  let called;
  const client = {callTool: async request => {
    called = request;
    return {content: [{type: 'text', text: JSON.stringify({result: {marker: 'mirket-token-sync', source_sha256: compiled.source_sha256}})}]};
  }};
  const result = await synchronizePenpotTokens(client, compiled, undefined);
  assert.equal(called.name, 'execute_code');
  assert.match(called.arguments.code, /Mirket Canonical/);
  assert.equal(result.count, 1);
});

test('official Penpot SVG export is staged, optimized, and never clobbers an existing path', async () => {
  const target = path.join(workspace, 'penpot.svg');
  const client = {callTool: async request => {
    assert.equal(request.name, 'export_shape');
    assert.notEqual(request.arguments.filePath, target);
    await fs.writeFile(request.arguments.filePath, '<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10" fill="red"/></svg>');
    return {content: [{type: 'text', text: `The shape has been exported to ${request.arguments.filePath}`}]};
  }};
  const result = await exportPenpotShape(client, {shapeId: 'selection', format: 'svg', filePath: target});
  assert.match(result.content[0].text, /penpot\.svg$/);
  assert.match((await fs.readFile(target, 'utf8')), /<svg/);
  await assert.rejects(exportPenpotShape(client, {shapeId: 'selection', format: 'svg', filePath: target}), /already exists/);
});

test('corrupt official Penpot SVG export fails without publishing', async () => {
  const target = path.join(workspace, 'broken-penpot.svg');
  const client = {callTool: async request => {
    await fs.writeFile(request.arguments.filePath, Buffer.from([0, 0]));
    return {content: [{type: 'text', text: 'success'}]};
  }};
  await assert.rejects(exportPenpotShape(client, {shapeId: 'selection', format: 'svg', filePath: target}), /SVG|saxes|Unclosed|disallowed character/i);
  await assert.rejects(fs.stat(target), {code: 'ENOENT'});
});


test('concurrent overwrite publication has one revision winner', async () => {
  await publish('overwrite-race.txt', Buffer.from('before'));
  const options = {overwrite:true,expected_sha256:sha256(Buffer.from('before'))};
  const outcomes = await Promise.allSettled(Array.from({length:8}, (_,i) => publish('overwrite-race.txt', Buffer.from(`after-${i}`), options)));
  assert.equal(outcomes.filter(x => x.status === 'fulfilled').length, 1);
  assert.equal(outcomes.filter(x => x.status === 'rejected').length, 7);
});

test('Satori rejects wide trees before fetching and serializes/deduplicates image preparation', async () => {
  const img = n => ({type:'img',props:{src:`image-${n}.png`,width:100,height:100}});
  let calls=0,active=0,peak=0;
  const loader = async () => {calls++; peak=Math.max(peak,++active); await new Promise(r=>setTimeout(r,5)); active--; return Buffer.from('image');};
  await assert.rejects(prepareSatoriTree({type:'div',children:Array.from({length:17},(_,i)=>img(i))},{},loader),/aggregate/);
  assert.equal(calls,0);
  await prepareSatoriTree({type:'div',children:[img(1),img(2),img(1)]},{},loader);
  assert.equal(calls,2); assert.equal(peak,1);
  calls=0;
  await assert.rejects(prepareSatoriTree({type:'div',children:[img(1),img(2)]},{},async()=>{calls++;throw Error('injected image failure');}),/injected/);
  assert.equal(calls,1); assert.equal(active,0);
  await assert.rejects(prepareSatoriTree(img(1),{},async()=>Buffer.alloc(12*1024*1024+1)),/byte budget/);
});

test('bounded readers cap bytes even when a regular file grows after fstat', async () => {
  const file=path.join(workspace,'growing.bin');
  const open=fs.open;let read=0;
  await fs.writeFile(file,'small');
  fs.open=async (...args)=>{
    const handle=await open(...args);
    if(args[0]===file){
      const stat=handle.stat.bind(handle), originalRead=handle.read.bind(handle);
      handle.stat=async()=>{const before=await stat();await fs.writeFile(file,Buffer.alloc(1024*1024));return before;};
      handle.read=async(...params)=>{const result=await originalRead(...params);read+=result.bytesRead;return result;};
    }
    return handle;
  };
  try {await assert.rejects(readBoundedFile(file,1024),/exceeds/); assert.equal(read,1025);}
  finally {fs.open=open;}
  await assert.rejects(regularOutput(file,1024),/exceeds/);
});

test('successful renderer exit reaps detached stdio-independent descendants', async () => {
  const code=`const {spawn}=require('node:child_process');const c=spawn(process.execPath,['-e','setInterval(()=>{},1000)'],{detached:true,stdio:'ignore'});console.log(c.pid);c.unref();`;
  const result=await run(process.execPath,['-e',code]);
  const pid=Number(result.stdout.trim());assert.ok(pid>1);
  assert.throws(()=>process.kill(pid,0),{code:'ESRCH'});
});


test('cold token compilation is safe across independent processes and missing parents', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'mirket-token-processes-'));
  await fs.writeFile(path.join(root,'tokens.json'), JSON.stringify({color:{accent:{$type:'color',$value:'#765432'}}}));
  const code=`const {compileTokens}=await import(${JSON.stringify(new URL('../tokens.mjs',import.meta.url).href)});console.log(JSON.stringify(await compileTokens('tokens.json')));`;
  try {
    const results=await Promise.all(Array.from({length:3},()=>new Promise((resolve,reject)=>{
      const child=spawn(process.execPath,['--input-type=module','-e',code],{env:{...process.env,MIRKET_DESIGN_WORKSPACE:root},stdio:['ignore','pipe','pipe']});let out='',err='';
      child.stdout.on('data',c=>out+=c);child.stderr.on('data',c=>err+=c);child.once('error',reject);child.once('close',n=>n===0?resolve(JSON.parse(out)):reject(Error(err)));
    })));
    assert.equal(new Set(results.map(r=>r.source_sha256)).size,1);
  } finally {await fs.rm(root,{recursive:true,force:true});}
});

test('non-rendering operations do not load Satori, resvg or browser clients', async () => {
  const loader=path.join(workspace,'no-renderers.mjs');
  await fs.writeFile(loader,`export async function resolve(s,c,n){if(['satori','@resvg/resvg-js','playwright-core'].includes(s))throw Error('Eager renderer '+s);return n(s,c);}`);
  const result=spawnSync(process.execPath,['--loader',loader,'--input-type=module','-e',`const {optimizeSvgBytes}=await import(${JSON.stringify(new URL('../operations.mjs',import.meta.url).href)});optimizeSvgBytes('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1 1"><path d="M0 0h1v1z"/></svg>');`],{encoding:'utf8',timeout:5000});
  assert.equal(result.status,0,result.stderr);
});


test('Odiff rejects compressed-image dimension bombs before native decoding', async () => {
  const header=Buffer.alloc(24);Buffer.from([137,80,78,71,13,10,26,10]).copy(header);header.writeUInt32BE(100000,16);header.writeUInt32BE(100000,20);
  await fs.writeFile(path.join(workspace,'huge-pixels.png'),header);
  await assert.rejects(odiffCompare({baseline:'huge-pixels.png',candidate:'huge-pixels.png'}),/decoded pixel budget/);
});


test('page and frame QA enforce a shared decoded-pixel budget', () => {
  const header=Buffer.alloc(24);Buffer.from([137,80,78,71,13,10,26,10]).copy(header);header.writeUInt32BE(4096,16);header.writeUInt32BE(4096,20);
  const accept=imageBatchBudget();
  for(let i=0;i<4;i++) assert.deepEqual(accept(header),[4096,4096]);
  assert.throws(()=>accept(header),/aggregate/);
});


test('scoped browser routes reject sibling, encoded traversal and editor requests', async () => {
  const {scopedWorkspaceUrlAllowed} = await import('../operations.mjs');
  const prefix = 'projects/my project';
  assert.equal(scopedWorkspaceUrlAllowed('http://design-tools:9002/files/projects/my%20project/index.html', prefix), true);
  for (const url of [
    'http://design-tools:9002/files/projects/sibling/secret.txt',
    'http://design-tools:9002/files/projects/my%20project/%2e%2e/sibling/secret.txt',
    'http://design-tools:9002/files/projects/my%20project/%2e%2e%2fsibling/secret.txt',
    'http://design-tools:9002/files/projects/my%20project/a%5c..%5csecret.txt',
    'http://penpot-frontend:8080/', 'file:///workspace/projects/my%20project/index.html',
  ]) assert.equal(scopedWorkspaceUrlAllowed(url, prefix), false, url);
});
