import assert from 'node:assert/strict';
import test from 'node:test';
import {mkdtemp,mkdir,writeFile,rm,symlink} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
const checker=fileURLToPath(new URL('./check-review-pack.mjs',import.meta.url));
const hash=(text)=>createHash('sha256').update(text).digest('hex');
test('snapshot identity refuses changed video, frame, source and links; never certifies quality',async()=>{
 const root=await mkdtemp(join(tmpdir(),'mirket-review-pack-'));
 try{
  await mkdir(join(root,'qa'));
  for(const name of ['video.mp4','frame.png','film.js','site.css'])await writeFile(join(root,name),'frozen');
  const manifest={source_video:{path:'video.mp4',sha256:hash('frozen')},frames:[{path:'frame.png',sha256:hash('frozen')}],composition_inputs:{'film.js':hash('frozen'),'site.css':hash('frozen')}};
  const save=()=>writeFile(join(root,'qa/review-pack.json'),JSON.stringify(manifest));await save();
  const run=()=>{const r=spawnSync(process.execPath,[checker,root],{encoding:'utf8'});return {code:r.status,value:JSON.parse(r.stdout)};};
  let result=run();assert.equal(result.code,0);assert.equal(result.value.visual_quality,'not established by this check');
  manifest.human_review={video_sha256:hash('older candidate')};await save();result=run();assert.equal(result.code,1);assert.equal(result.value.failures[0].reason,'acceptance belongs to another video');delete manifest.human_review;await save();
  for(const name of ['video.mp4','frame.png','film.js','site.css']){
   await writeFile(join(root,name),'changed');result=run();assert.equal(result.code,1);assert.deepEqual(result.value.failures,[{path:name,reason:'snapshot changed'}]);await writeFile(join(root,name),'frozen');
  }
  await rm(join(root,'frame.png'));await symlink(join(root,'video.mp4'),join(root,'frame.png'));result=run();assert.equal(result.code,1);assert.equal(result.value.failures[0].reason,'symlink');
  manifest.frames=[{path:'../outside.png',sha256:hash('frozen')}];await save();result=run();assert.equal(result.code,1);assert.equal(result.value.failures[0].reason,'outside pack');
 }finally{await rm(root,{recursive:true,force:true});}
});
