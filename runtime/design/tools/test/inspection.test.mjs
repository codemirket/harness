import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import sharp from 'sharp';
const workspace = await fs.mkdtemp(path.join(os.tmpdir(), 'mirket-inspection-'));
process.env.MIRKET_DESIGN_WORKSPACE = workspace;
const {prepareInspection, IMAGE_BYTES} = await import('../inspection.mjs');
const {sha256} = await import('../paths.mjs');
test.after(() => fs.rm(workspace, {recursive:true, force:true}));

test('large raster retains original bytes and has complete lossless bounded inspection evidence', async () => {
  const width=2048, height=1024, pixels=crypto.randomBytes(width*height*4);
  const png = await sharp(pixels,{raw:{width,height,channels:4}}).png().toBuffer();
  assert.ok(png.length > IMAGE_BYTES);
  await fs.writeFile(path.join(workspace,'large.png'),png);
  const source={path:'large.png',sha256:sha256(png),bytes:png.length};
  const result=await prepareInspection({artifact:source});
  const plan=result.inspection[0];
  assert.deepEqual(plan.source,source);
  assert.deepEqual(await fs.readFile(path.join(workspace,'large.png')),png);
  const reconstructed=Buffer.alloc(pixels.length);
  let area=0;
  for (const tile of plan.tiles) {
    const bytes=await fs.readFile(path.join(workspace,tile.path));
    assert.ok(bytes.length <= IMAGE_BYTES);
    assert.equal(sha256(bytes),tile.sha256);
    const raw=await sharp(bytes).ensureAlpha().raw().toBuffer();
    for (let row=0;row<tile.height;row++) raw.copy(reconstructed,((tile.top+row)*width+tile.left)*4,row*tile.width*4,(row+1)*tile.width*4);
    area+=tile.width*tile.height;
  }
  assert.equal(area,width*height);
  assert.deepEqual(reconstructed,pixels);
  assert.ok((await fs.stat(path.join(workspace,plan.overview.path))).size <= IMAGE_BYTES);
  await fs.writeFile(path.join(workspace,'large.png'),'changed');
  await assert.rejects(prepareInspection({artifact:source}),/changed after rendering/);
});

test('small images stay direct and AVIF receives PNG evidence even below the byte limit',async()=>{
  const small=await sharp({create:{width:24,height:16,channels:4,background:'#abcdef'}}).png().toBuffer();
  await fs.writeFile(path.join(workspace,'small.png'),small);
  const receipt={artifact:{path:'small.png',sha256:sha256(small)}};
  assert.equal(await prepareInspection(receipt),receipt);
  const avif=await sharp(small).avif().toBuffer();
  await fs.writeFile(path.join(workspace,'small.avif'),avif);
  assert.equal((await prepareInspection({artifact:{path:'small.avif',sha256:sha256(avif)}})).inspection[0].tiles.length,1);
});
