import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import sharp from 'sharp';
const workspace = await fs.mkdtemp(path.join(os.tmpdir(), 'mirket-cleanup-'));
process.env.MIRKET_DESIGN_WORKSPACE = workspace;
const {rasterTransform, svgOptimize, vectorRender, browserInspect, validateBrowserActions, hyperframesSnapshotArgs, optimizeSvgBytes} = await import('../operations.mjs');
const {withRenderSlot} = await import('../paths.mjs');
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

test('native render admission refuses competing work and releases after failures', async () => {
  let release, entered;
  const ready = new Promise(r => {entered=r;});
  const first = withRenderSlot(async () => { entered(); await new Promise(r => {release=r;}); });
  await ready;
  try { await assert.rejects(withRenderSlot(() => assert.fail('second renderer started')), /busy/); }
  finally { release(); await first; }
  await assert.rejects(withRenderSlot(async () => {throw Error('renderer failure');}),/renderer failure/);
  assert.equal(await withRenderSlot(async () => 42),42);
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
