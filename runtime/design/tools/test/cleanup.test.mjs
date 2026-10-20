import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import sharp from 'sharp';
const workspace = await fs.mkdtemp(path.join(os.tmpdir(), 'mirket-cleanup-'));
process.env.MIRKET_DESIGN_WORKSPACE = workspace;
const {rasterTransform, svgOptimize, vectorRender, browserInspect, validateBrowserActions, hyperframesSnapshotArgs, optimizeSvgBytes, stageProject, stageDesignAsset} = await import('../operations.mjs');
const {withRenderSlot} = await import('../paths.mjs');
const {withRequest} = await import('../request.mjs');
test.after(() => fs.rm(workspace, {recursive:true, force:true}));

test('mechanical raster transforms need no tokens; all output codecs decode with bounded dimensions', async () => {
  await sharp({create:{width:80,height:40,channels:4,background:{r:20,g:60,b:120,alpha:0.5}}}).png().toFile(path.join(workspace,'source.png'));
  for (const format of ['png','jpeg','webp','avif']) {
    const result = await rasterTransform({source:'source.png',output:`out.${format}`,width:40,height:20,mode:'force',format});
    assert.equal(result.service,'sharp');
    const image = await sharp(path.join(workspace,result.artifact.path)).metadata();
    assert.equal(image.width,40); assert.equal(image.height,20);
  }
  const kept = await rasterTransform({source:'source.png',output:'not-enlarged.png',width:400,height:200});
  assert.equal((await sharp(path.join(workspace,kept.artifact.path)).metadata()).width,80);
  const enlarged = await rasterTransform({source:'source.png',output:'enlarged.png',width:400,height:200,enlarge:true});
  assert.equal((await sharp(path.join(workspace,enlarged.artifact.path)).metadata()).width,400);
  await sharp(path.join(workspace,'source.png')).gif().toFile(path.join(workspace,'source.gif'));
  await rasterTransform({source:'source.gif',output:'gif-first.png',width:40,height:20});
  await fs.writeFile(path.join(workspace,'not-raster.svg'),'<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"/>');
  await assert.rejects(rasterTransform({source:'not-raster.svg',output:'bad.png',width:20,height:20}), /Only PNG/);
  await assert.rejects(rasterTransform({source:'source.png',output:'bad.png',width:4097,height:20}), /dimensions/);
  await assert.rejects(fs.access(path.join(workspace,'bad.png')));
});

test('native PNG QA returns original pixels without a browser or duplicate screenshot', async () => {
  await sharp({create:{width:12,height:9,channels:3,background:'#123456'}}).png().toFile(path.join(workspace,'native.png'));
  const qa = await browserInspect('native.png');
  assert.equal(qa.screenshot.path,'native.png');
  assert.equal(qa.width,12); assert.equal(qa.height,9); assert.equal(qa.scale,1);
  assert.equal(qa.status,'ready_for_inspection');
});

test('composition staging retains original raster size, encoding and metadata', async () => {
  const source = path.join(workspace, 'composition'), stage = path.join(workspace, 'staged');
  await fs.mkdir(source); await fs.mkdir(stage);
  await fs.writeFile(path.join(source, 'index.html'), '<html><head></head><body><img src="hero.jpeg"></body></html>');
  const originals = new Map();
  for (const format of ['jpeg', 'png', 'webp', 'avif']) {
    const bytes = await sharp({create:{width:3000,height:12,channels:4,background:{r:20,g:60,b:120,alpha:0.5}}})
      .withMetadata({orientation:6}).toFormat(format).toBuffer();
    originals.set(format, bytes);
    await fs.writeFile(path.join(source, `hero.${format}`), bytes);
  }
  await stageProject('composition', stage);
  for (const [format, bytes] of originals) {
    const actual = await fs.readFile(path.join(stage, `hero.${format}`));
    assert.deepEqual(actual, bytes, `${format} changed during staging`);
    const metadata = await sharp(actual).metadata();
    assert.equal(Math.max(metadata.width, metadata.height), 3000);
  }
});

test('shared video and paged asset staging rejects corrupt rasters before writing', async () => {
  const dest = path.join(workspace, 'invalid-staged.png');
  const png = await sharp({create:{width:40,height:40,channels:3,background:'#123456'}}).png().toBuffer();
  await assert.rejects(stageDesignAsset({dest, local:'broken.png', bytes:png.subarray(0, 50)}));
  await assert.rejects(fs.access(dest));
  await assert.rejects(stageDesignAsset({dest, local:'fake.png', bytes:Buffer.from('<svg/>')}), /Only PNG/);
  await assert.rejects(fs.access(dest));
});

test('staging refuses animated WebP instead of silently flattening its frames', async () => {
  const pixels = Buffer.from([...Array(4).fill([255,0,0,255]).flat(), ...Array(4).fill([0,0,255,255]).flat()]);
  const bytes = await sharp(pixels, {raw:{width:2,height:4,channels:4,pageHeight:2}})
    .webp({loop:0,delay:[100,100]}).toBuffer();
  assert.equal((await sharp(bytes).metadata()).pages, 2);
  const dest = path.join(workspace, 'animated-staged.webp');
  await assert.rejects(stageDesignAsset({dest, local:'animated.webp', bytes}), /explicit frame sequence or video/);
  await assert.rejects(fs.access(dest));
});

test('editable vector output retains semantic groups and yields a matching native preview', async () => {
  const svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 80 40"><g id="scene"><g id="subject"><rect x="8" y="8" width="64" height="24" fill="#13579b"/></g></g></svg>';
  await fs.writeFile(path.join(workspace,'art.svg'),svg);
  const result = await vectorRender({source:'art.svg',output:'final.svg',width:160,height:80});
  const saved = await fs.readFile(path.join(workspace,'final.svg'),'utf8');
  assert.match(saved,/id="scene"/); assert.match(saved,/id="subject"/);
  const image = await sharp(path.join(workspace,result.preview.path)).metadata();
  assert.equal(image.width,160); assert.equal(image.height,80);
  assert.equal(result.qa.screenshot.path,result.preview.path);
  await svgOptimize({source:'art.svg',output:'optimized.svg'});
  assert.throws(() => optimizeSvgBytes('<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>'),/Unsafe SVG/);
});

test('editable vector output preserves hidden states and empty attachment groups', async () => {
  const svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 80 40"><g id="rest"><rect width="40" height="40"/></g><g id="expanded" display="none"><circle cx="20" cy="20" r="10"/></g><g id="label-anchor"/></svg>';
  await fs.writeFile(path.join(workspace, 'states.svg'), svg);
  const result = await vectorRender({source:'states.svg',output:'states-editable.svg',width:80,height:40});
  const saved = await fs.readFile(path.join(workspace,result.artifact.path),'utf8');
  for (const id of ['rest', 'expanded', 'label-anchor']) assert.ok(saved.includes(`id="${id}"`), `${id} was deleted`);
  const before = await sharp(Buffer.from(svg)).ensureAlpha().raw().toBuffer();
  const after = await sharp(path.join(workspace,result.preview.path)).ensureAlpha().raw().toBuffer();
  assert.deepEqual(after, before, 'preserving an inactive state must not reveal it');
});

test('native render admission serializes competing work and releases after failures', async () => {
  let release, entered;
  const order = [];
  const ready = new Promise(r => {entered=r;});
  const first = withRenderSlot(async () => { order.push('first'); entered(); await new Promise(r => {release=r;}); order.push('released'); });
  await ready;
  const second = withRenderSlot(async () => { order.push('second'); return 17; });
  const timer = setTimeout(release, 100);
  try { assert.equal(await second, 17); }
  finally { clearTimeout(timer); release(); await first; }
  assert.deepEqual(order, ['first', 'released', 'second']);
  await assert.rejects(withRenderSlot(async () => {throw Error('renderer failure');}),/renderer failure/);
  assert.equal(await withRenderSlot(async () => 42),42);
});

test('native vector previews retain ordinary text without caller-supplied fonts', async () => {
  const families = ['Georgia, serif', 'serif', 'sans-serif', 'monospace'];
  const source = `<svg xmlns="http://www.w3.org/2000/svg" width="300" height="160"><rect width="300" height="160" fill="white"/>${families.map((family, i) => `<text x="10" y="${30 + i * 40}" font-family="${family}" font-size="24" fill="black">I II III 123</text>`).join('')}</svg>`;
  await fs.writeFile(path.join(workspace, 'text.svg'), source);
  const result = await vectorRender({source:'text.svg', output:'text-preview.svg', width:300, height:160});
  const {data, info} = await sharp(path.join(workspace, result.preview.path)).removeAlpha().raw().toBuffer({resolveWithObject:true});
  for (let row = 0; row < families.length; row++) {
    let ink = 0;
    for (let y = row * 40; y < (row + 1) * 40; y++) {
      for (let x = 0; x < info.width; x++) if (data[(y * info.width + x) * info.channels] < 128) ink++;
    }
    assert.ok(ink > 100, `${families[row]} text disappeared from the native preview`);
  }
  assert.match(await fs.readFile(path.join(workspace, 'text-preview.svg'), 'utf8'), /<text/);
});

test('queued render cancellation and admission deadline never start a renderer', async () => {
  let release, entered;
  const ready = new Promise(r => { entered = r; });
  const first = withRenderSlot(async () => { entered(); await new Promise(r => { release = r; }); });
  await ready;
  let dispatched = 0;
  try {
    await assert.rejects(withRenderSlot(() => { dispatched++; }, {waitMs:50}), /timed out before operation started/);
    const controller = new AbortController();
    const waiting = withRequest(controller.signal, () => withRenderSlot(() => { dispatched++; }));
    const timer = setTimeout(() => controller.abort(), 50);
    try { await assert.rejects(waiting, /cancelled before operation started|abort/i); }
    finally { clearTimeout(timer); }
    assert.equal(dispatched, 0);
  } finally { release(); await first; }
  // A cancelled waiter must not later acquire and occupy the slot.
  assert.equal(await withRenderSlot(async () => 'available', {waitMs:1000}), 'available');
});

test('browser actions and timing reject unbounded or executable instructions', () => {
  validateBrowserActions([{kind:'press',selector:'#dialog',key:'Escape'}],500,'reduce');
  assert.throws(() => validateBrowserActions([{kind:'evaluate',selector:'body'}],0,'reduce'),/Invalid browser action/);
  assert.throws(() => validateBrowserActions(Array(9).fill({kind:'click',selector:'button'}),0,'reduce'),/eight/);
  assert.throws(() => validateBrowserActions([],60001,'reduce'),/time/);
  assert.throws(() => validateBrowserActions([],0,'invalid'),/preference/);
});


test('motion checkpoint selection is bounded and preserves exact requested times', () => {
  assert.deepEqual(hyperframesSnapshotArgs([1.5,0,0.5]), ['--at','0,0.5,1.5','--no-end']);
  assert.throws(() => hyperframesSnapshotArgs([1,1]), /unique/);
  assert.throws(() => hyperframesSnapshotArgs([3601]), /3600/);
  assert.throws(() => hyperframesSnapshotArgs([]), /1..8/);
});

test('SVG styles validate combined text and CDATA before and after optimization', () => {
  for (const css of ['<![CDATA[@import url("https://example.test/x.css");]]>', '<![CDATA[rect{fill:url(https://example.test/x.svg)}]]>', '@im<![CDATA[port "https://example.test/x.css";]]>']) {
    assert.throws(() => optimizeSvgBytes(`<svg xmlns="http://www.w3.org/2000/svg"><style>${css}</style></svg>`), /SVG external/);
  }
  assert.ok(optimizeSvgBytes('<svg xmlns="http://www.w3.org/2000/svg"><style><![CDATA[rect{fill:red}]]></style><rect width="10" height="10"/></svg>').length);
});

test('raster preparation honors EXIF orientation and flattens JPEG alpha onto white', async () => {
  await sharp({create:{width:80,height:40,channels:3,background:'#123456'}}).jpeg().withMetadata({orientation:6}).toFile(path.join(workspace,'oriented.jpg'));
  const rotated = await rasterTransform({source:'oriented.jpg',output:'upright.png',width:100,height:100});
  const metadata = await sharp(path.join(workspace,rotated.artifact.path)).metadata();
  assert.deepEqual([metadata.width,metadata.height],[40,80]);
  await sharp({create:{width:12,height:12,channels:4,background:{r:0,g:0,b:0,alpha:0}}}).png().toFile(path.join(workspace,'transparent.png'));
  const flattened = await rasterTransform({source:'transparent.png',output:'white.jpg',width:12,height:12,format:'jpeg'});
  const pixels = await sharp(path.join(workspace,flattened.artifact.path)).raw().toBuffer();
  assert.ok(pixels.every(value=>value>=253));
});
