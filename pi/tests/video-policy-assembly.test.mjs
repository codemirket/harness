import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const adapterRoot=fileURLToPath(new URL('../',import.meta.url));
const sha=source=>createHash('sha256').update(source).digest('hex');
async function fixture(){
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'mirket-video-policy-assembly-'));
  await fs.mkdir(path.join(root,'scripts'));await fs.mkdir(path.join(root,'node_modules/vendor'),{recursive:true});
  await fs.symlink(path.join(adapterRoot,'node_modules/typescript'),path.join(root,'node_modules/typescript'),process.platform==='win32'?'junction':'dir');
  await fs.copyFile(path.join(adapterRoot,'scripts/prepare-video-policy.mjs'),path.join(root,'scripts/prepare-video-policy.mjs'));
  const sources=['export const load = () => fetch("local");\n',"const opts = {}; opts.detached ??= process.platform !== 'win32';\n"];
  const files=sources.map((source,i)=>({path:`vendor/${i}.js`,sha256:sha(source),...(i===0?{fetch:1}:{detached:true})}));
  for(let i=0;i<sources.length;i++)await fs.writeFile(path.join(root,'node_modules',files[i].path),sources[i]);
  await fs.writeFile(path.join(root,'scripts/video-policy-inputs.json'),JSON.stringify({schema:1,files}));
  return {root,files,sources,run:()=>spawnSync(process.execPath,[path.join(root,'scripts/prepare-video-policy.mjs')],{encoding:'utf8'}),read:i=>fs.readFile(path.join(root,'node_modules',files[i].path),'utf8')};
}

test('assembly labels every modified vendor file and an unchanged rerun preserves exact bytes',async()=>{
  const f=await fixture();try{
    const first=f.run();assert.equal(first.status,0,first.stderr);
    const receipt=await fs.readFile(path.join(f.root,'node_modules/.mirket-video-policy.json'),'utf8');
    const outputs=await Promise.all(f.files.map((_,i)=>f.read(i)));
    for(const output of outputs)assert.match(output,/^\/\/ Modified by Mirket: /);
    assert.match(outputs[0],/mirketVideoFetch\("local"\)/);assert.match(outputs[1],/opts.detached = process.env.MIRKET_VIDEO_POLICY_VERSION/);
    const second=f.run();assert.equal(second.status,0,second.stderr);
    assert.deepEqual(await Promise.all(f.files.map((_,i)=>f.read(i))),outputs);
    assert.equal(await fs.readFile(path.join(f.root,'node_modules/.mirket-video-policy.json'),'utf8'),receipt);
    await fs.unlink(path.join(f.root,'node_modules/.mirket-video-policy.json'));
    assert.notEqual(f.run().status,0);assert.deepEqual(await Promise.all(f.files.map((_,i)=>f.read(i))),outputs);
    await fs.writeFile(path.join(f.root,'node_modules/.mirket-video-policy.json'),receipt);
    await fs.appendFile(path.join(f.root,'node_modules',f.files[1].path),'// drift\n');
    assert.notEqual(f.run().status,0);assert.equal(await f.read(0),outputs[0]);
  }finally{await fs.rm(f.root,{recursive:true,force:true});}
});

test('a changed final input refuses the complete assembly before any vendor file is written',async()=>{
  const f=await fixture();try{
    await fs.appendFile(path.join(f.root,'node_modules',f.files[1].path),'// unexpected upstream change\n');
    const result=f.run();assert.notEqual(result.status,0);assert.match(result.stderr,/Exact upstream input differs/);
    assert.equal(await f.read(0),f.sources[0]);
    await assert.rejects(fs.stat(path.join(f.root,'node_modules/.mirket-video-policy.json')),/ENOENT/);
  }finally{await fs.rm(f.root,{recursive:true,force:true});}
});
