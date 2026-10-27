import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { captionSidecars, cues, digest, fileRevision, MAX_OUTPUT, MAX_WORK, preflight, tar, treeBytes, validateRequest, writeTar } from '../dist/video-export.js';
import { localUrl, mirketPreparePage, mirketSourceFile, mirketVideoFetch } from '../dist/video-policy.js';

test('caption cue times derive from rational frame positions and preserve Unicode',()=>{
  const input=cues([{start_frame:3,end_frame:21,text:'Hazır · Türkçe ğüşİı\n世界 🌍'}],24);
  const result=captionSidecars(input,{numerator:12,denominator:1});
  assert.match(result.srt.toString(),/00:00:00,250 --> 00:00:01,750/);
  assert.match(result.vtt.toString(),/^WEBVTT\n/);assert.match(result.vtt.toString(),/世界 🌍/);
  assert.throws(()=>cues([{start_frame:3,end_frame:21,text:'a'},{start_frame:20,end_frame:24,text:'b'}],24),/invalid_caption_cues/);
  assert.throws(()=>cues([{start_frame:0,end_frame:24,text:'\ud800'}],24),/invalid_caption_cues/);
});

test('PNG tar is deterministic and refuses duplicate, traversal and nested entries',()=>{
  const entries=[{path:'frame_000001.png',bytes:Buffer.from('frame')},{path:'manifest.json',bytes:Buffer.from('{}\n')}];
  const archive=tar(entries);assert.deepEqual(archive,tar(entries));assert.equal(archive.length%512,0);assert.equal(archive.subarray(257,263).toString(),'ustar\0');
  for(const name of ['../outside','/outside','frames/a.png','C:\\outside'])assert.throws(()=>tar([{path:name,bytes:Buffer.alloc(0)}]),/invalid_archive_entry/);
  assert.throws(()=>tar([entries[0],entries[0]]),/invalid_archive_entry/);assert.throws(()=>tar(entries,1024),/output_budget/);
});

test('work-tree scan tolerates disappearing Chromium child directories but not a missing root',async()=>{
  const root=await fs.realpath(await fs.mkdtemp(path.join(os.tmpdir(),'mirket-video-scan-')));
  const child=path.join(root,'transient-profile');
  await fs.mkdir(child);
  const original=fs.readdir;
  try{
    fs.readdir=async(file,options)=>{
      if(file===child){const error=new Error('profile removed during traversal');error.code='ENOENT';throw error;}
      return original(file,options);
    };
    assert.equal(await treeBytes(root),0);
    await assert.rejects(treeBytes(path.join(root,'missing-root')),{code:'ENOENT'});
  }finally{fs.readdir=original;await fs.rm(root,{recursive:true,force:true});}
});

async function fixture(){
  const directory=await fs.realpath(await fs.mkdtemp(path.join(os.tmpdir(),'mirket-video-source-')));await fs.mkdir(path.join(directory,'source'));
  const html=Buffer.from('<!doctype html><html><head><style>@font-face{font-family:Local;src:url("./font.ttf")}</style></head><body><div data-composition-id="local" data-no-timeline data-width="640" data-height="360" data-duration="2"><img src="./image.png"></div></body></html>');
  const sources=[['index.html',html],['font.ttf',Buffer.from('qualified elsewhere')],['image.png',Buffer.from('asset')]];
  for(const [name,bytes]of sources)await fs.writeFile(path.join(directory,'source',name),bytes);
  const request={protocol_version:1,release_id:'test:video',job_id:'video-1',job_root:directory,input_root:path.join(directory,'source'),entry_file:'index.html',sources:sources.map(([name,bytes])=>({path:name,revision:digest(bytes),bytes:bytes.length})),output_path:path.join(directory,'result.mp4'),profile:{width:640,height:360,fps:{numerator:12,denominator:1},frame_count:24,format:'mp4',captions_path:null},timeout_ms:30000,max_output_bytes:20*1024*1024};
  return {directory,request,async html(text){const bytes=Buffer.from(text);await fs.writeFile(path.join(directory,'source/index.html'),bytes);request.sources[0]={path:'index.html',revision:digest(bytes),bytes:bytes.length};}};
}
test('selected source preflight refuses missing, network, dynamic and stale inputs before renderer import',async()=>{
  const f=await fixture();try{
    assert.ok((await preflight(f.request)).entry.length);
    await fs.writeFile(path.join(f.directory,'source/image.png'),'changed');await assert.rejects(preflight(f.request),/source_revision_changed/);
    await fs.writeFile(path.join(f.directory,'source/image.png'),'asset');
    const base=await fs.readFile(path.join(f.directory,'source/index.html'),'utf8');
    for(const [before,after,reason]of [['./image.png','./missing.png','missing_or_unselected_asset'],['./image.png','https://example.com/private.png','remote_or_absolute_asset_refused'],['./image.png','../outside.png','missing_or_unselected_asset']]){
      await f.html(base.replace(before,after));await assert.rejects(preflight(f.request),new RegExp(reason));
    }
    await f.html(base.replace('</body>','<script>fetch("/api")</script></body>'));await assert.rejects(preflight(f.request),/unsupported_dynamic_dependency_or_effect/);
  }finally{await fs.rm(f.directory,{recursive:true,force:true});}
});
test('wire request binds finite work to the owned source/output and exact profile',async()=>{
  const f=await fixture(),previous=process.cwd();try{process.chdir(f.directory);assert.equal(validateRequest(f.request,'test:video').profile.frame_count,24);
    for(const mutate of [r=>r.profile.fps.denominator=0,r=>r.output_path=path.join(f.directory,'outside.mp4'),r=>r.sources.push({...r.sources[0]})]){const value=structuredClone(f.request);mutate(value);assert.throws(()=>validateRequest(value,'test:video'));}
  }finally{process.chdir(previous);await fs.rm(f.directory,{recursive:true,force:true});}
});
test('pre-navigation policy permits only literal same-origin loopback and refuses escaped files',async()=>{
  const f=await fixture(),previous=process.cwd(),old=process.env.MIRKET_VIDEO_POLICY_VERSION;
  try{process.chdir(f.directory);process.env.MIRKET_VIDEO_POLICY_VERSION='1';
    assert.equal(localUrl('http://127.0.0.1:31001/index.html').hostname,'127.0.0.1');
    for(const value of ['http://localhost:31001/','https://example.com/','file:///etc/passwd','http://127.0.0.1:31001@evil.example/'])assert.throws(()=>localUrl(value),/mirket_video_remote_dependency/);
    await assert.rejects(mirketVideoFetch('http://127.0.0.1:9999/private-service'),/mirket_video_unowned_loopback_origin/);
    const events=[],handlers={};const page={async setBypassServiceWorker(){events.push('bypass')},async setRequestInterception(){events.push('interception')},on(name,fn){handlers[name]=fn},async evaluateOnNewDocument(){events.push('before-navigation-script')}};
    await mirketPreparePage(page,'http://127.0.0.1:31001');assert.deepEqual(events,['bypass','interception','before-navigation-script']);
    for(const [url,expected]of [['http://127.0.0.1:31001/image.png','continue'],['http://127.0.0.1:9999/secret','abort'],['https://example.com/tracker','abort']]){
      let action;handlers.request({url:()=>url,isInterceptResolutionHandled:()=>false,continue:async()=>{action='continue'},abort:async()=>{action='abort'}});assert.equal(action,expected);
    }
    assert.equal(mirketSourceFile(path.join(f.directory,'source/index.html'),path.join(f.directory,'source')),true);
    assert.equal(mirketSourceFile(path.join(f.directory,'source/../secret'),path.join(f.directory,'source')),false);
    await fs.symlink('/etc/passwd',path.join(f.directory,'source/escape'));
    assert.equal(mirketSourceFile(path.join(f.directory,'source/escape'),path.join(f.directory,'source')),false);
  }finally{process.chdir(previous);if(old===undefined)delete process.env.MIRKET_VIDEO_POLICY_VERSION;else process.env.MIRKET_VIDEO_POLICY_VERSION=old;await fs.rm(f.directory,{recursive:true,force:true});}
});


test('file-backed PNG archive exceeds 20 MiB with bounded reads and refuses stale input and over-budget extent',async()=>{
  const root=await fs.realpath(await fs.mkdtemp(path.join(os.tmpdir(),'mirket-video-file-tar-'))),source=path.join(root,'frame.png'),output=path.join(root,'frames.tar');
  const handle=await fs.open(source,'wx');await handle.truncate(21*1024*1024);await handle.close();
  try{
    const controller=new AbortController(),frame=await fileRevision(source,MAX_OUTPUT,controller.signal);
    const entries=[{path:'frame_000001.png',source,bytes:frame.bytes,revision:frame.revision}];
    const originalOpen=fs.open;let largestRead=0;
    fs.open=async(...args)=>{const file=await originalOpen(...args),read=file.read.bind(file);file.read=async(buffer,...rest)=>{largestRead=Math.max(largestRead,buffer.length);assert.ok(buffer.length<=65536);return read(buffer,...rest);};return file;};
    try{await writeTar(entries,output,MAX_OUTPUT,controller.signal);await fileRevision(output,MAX_OUTPUT,controller.signal);}finally{fs.open=originalOpen;}
    assert.ok(largestRead>0);assert.ok((await fs.stat(output)).size>20*1024*1024);
    assert.equal(MAX_OUTPUT,2*1024*1024*1024);assert.equal(MAX_WORK,5*1024*1024*1024);
    await assert.rejects(writeTar(entries,path.join(root,'too-small.tar'),1024,controller.signal),/output_budget/);
    await assert.rejects(fs.stat(path.join(root,'too-small.tar')),{code:'ENOENT'});
    await fs.writeFile(source,'changed');await assert.rejects(writeTar(entries,path.join(root,'stale.tar'),MAX_OUTPUT,controller.signal),/archive_source_changed/);
    const stopped=new AbortController();stopped.abort();await assert.rejects(fileRevision(source,MAX_OUTPUT,stopped.signal),/cancelled/);
    await fs.symlink(source,path.join(root,'link'));await assert.rejects(fileRevision(path.join(root,'link'),MAX_OUTPUT),/symlink/);
  }finally{await fs.rm(root,{recursive:true,force:true});}
});

test('PNG archive succeeds when private Node permits writes but denies fsync',async()=>{
  const root=await fs.realpath(await fs.mkdtemp(path.join(os.tmpdir(),'mirket-video-no-fsync-'))),output=path.join(root,'frames.tar');
  const originalOpen=fs.open;let opened=false;
  try{
    fs.open=async(...args)=>{
      const handle=await originalOpen(...args);
      if(args[0]===output){opened=true;handle.sync=async()=>{throw new Error('fsync API is disabled when Permission Model is enabled');};}
      return handle;
    };
    const bytes=Buffer.from('frame bytes'),entries=[{path:'frame_000001.png',source:bytes,bytes:bytes.length,revision:digest(bytes)}];
    await writeTar(entries,output,4096,new AbortController().signal);
    assert.equal(opened,true);
    const saved=await fs.readFile(output);
    assert.equal(saved.subarray(257,263).toString(),'ustar\0');
    assert.deepEqual(saved.subarray(512,512+bytes.length),bytes);
    assert.equal(saved.length%512,0);
  }finally{fs.open=originalOpen;await fs.rm(root,{recursive:true,force:true});}
});

test('archive disk-write failure retains partial private stage and preserves its source',async()=>{
  const root=await fs.realpath(await fs.mkdtemp(path.join(os.tmpdir(),'mirket-video-enospc-'))),source=path.join(root,'input'),output=path.join(root,'partial.tar');
  const originalOpen=fs.open;await fs.writeFile(source,'preserve input');
  try{
    const input=await fileRevision(source,100),entries=[{path:'audio.m4a',source,bytes:input.bytes,revision:input.revision}];
    fs.open=async(...args)=>{const handle=await originalOpen(...args);if(args[0]===output){const write=handle.write.bind(handle);let writes=0;handle.write=async(...values)=>{if(++writes>1){const error=new Error('simulated disk full');error.code='ENOSPC';throw error;}return write(...values);};}return handle;};
    await assert.rejects(writeTar(entries,output,4096,new AbortController().signal),{code:'ENOSPC'});
    assert.equal((await fs.stat(output)).size,512);assert.equal(await fs.readFile(source,'utf8'),'preserve input');
  }finally{fs.open=originalOpen;await fs.rm(root,{recursive:true,force:true});}
});
