// Actual pi MCP invocation, with independent decoding of the saved export.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {join,isAbsolute} from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
const revision=bytes=>'sha256:'+createHash('sha256').update(bytes).digest('hex');
export async function videoJourney({session,call,save,cwd,release,root,fixture}){
 assert.ok(isAbsolute(fixture));
 const work=join(cwd,'video-'+Date.now());await fs.mkdir(work,{mode:0o700});
 const names=['index.html','anime.umd.min.js','NotoSans.ttf','voice.wav','music.wav'];
 const revisions={};for(const name of names){const bytes=await fs.readFile(join(fixture,name));await fs.writeFile(join(work,name),bytes);revisions[name]=revision(bytes);}
 const common={expected_release_id:release,project_root:cwd};
 async function status(job){try{return await call(session,'mcp__mirket__video_status',job);}catch(error){if(!String(error).includes('operation store is busy'))throw error;return undefined;}}
 async function terminal(job){const deadline=Date.now()+90000;while(Date.now()<deadline){const value=await status(job);if(value&&['succeeded','failed','cancelled'].includes(value.status)&&['not_started','removed_verified_owned_files'].includes(value.scratch_cleanup))return value;await new Promise(r=>setTimeout(r,200));}throw new Error('Video terminal cleanup deadline exceeded');}
 await call(session,'tool_search',{query:'video_prepare video_start video_status video_result',limit:8});
 const destination=join(work,'result.mp4');
 const intent={...common,source_path:join(work,'index.html'),expected_revision:revisions['index.html'],destination_path:destination,
  assets:names.slice(1).map(path=>({path,expected_revision:revisions[path]})),
  profile:{width:640,height:360,fps:{numerator:12,denominator:1},frame_count:24,format:'mp4',captions_path:null}};
 const prepared=await call(session,'mcp__mirket__video_prepare',intent);assert.equal(prepared.status,'prepared');
 const job={...common,operation_id:prepared.operation_id};await call(session,'mcp__mirket__video_start',job);
 const deadline=Date.now()+180000;let result;
 while(Date.now()<deadline){
  try{result=await call(session,'mcp__mirket__video_status',job);}catch(error){if(!String(error).includes('operation store is busy'))throw error;await new Promise(r=>setTimeout(r,200));continue;}
  if(['succeeded','failed','cancelled'].includes(result.status))break;await new Promise(r=>setTimeout(r,250));
 }
 await fs.writeFile(join(work,'receipt.json'),JSON.stringify({intent,prepared,result},null,2));
 assert.equal(result?.status,'succeeded',JSON.stringify(result));
 const native=join(root,'agent/mirket/releases','r-'+release.slice(7),'runtime/video/bin');
 const probe=JSON.parse(execFileSync(join(native,'ffprobe'),['-v','error','-show_streams','-show_format','-of','json',destination],{encoding:'utf8',timeout:10000}));
 const video=probe.streams.find(s=>s.codec_type==='video');assert.equal(video.width,640);assert.equal(video.height,360);assert.equal(Number(video.nb_frames),24);assert.ok(probe.streams.some(s=>s.codec_type==='audio'));assert.ok(Math.abs(Number(probe.format.duration)-2)<0.1);
 const frames=[];for(const [name,time] of [['first',0],['middle',1],['last',1.9]]){
  const image=join(work,name+'.png');execFileSync(join(native,'ffmpeg'),['-v','error','-nostdin','-i',destination,'-ss',String(time),'-frames:v','1',image],{timeout:10000});frames.push(image);
 }
 assert.notDeepEqual(await fs.readFile(frames[0]),await fs.readFile(frames[2]));
 const wavPath=join(work,'decoded-audio.wav');
 execFileSync(join(native,'ffmpeg'),['-v','error','-nostdin','-i',destination,'-map','0:a:0','-ac','1','-ar','48000','-c:a','pcm_s16le','-f','wav',wavPath],{timeout:10000,maxBuffer:1024*1024});
 const wav=await fs.readFile(wavPath);assert.equal(wav.toString('ascii',0,4),'RIFF');assert.equal(wav.toString('ascii',8,12),'WAVE');let pcm,validFormat=false;
 for(let offset=12;offset+8<=wav.length;){const kind=wav.toString('ascii',offset,offset+4),length=wav.readUInt32LE(offset+4);assert.ok(offset+8+length<=wav.length);if(kind==='fmt '){assert.ok(length>=16);assert.equal(wav.readUInt16LE(offset+8),1);assert.equal(wav.readUInt16LE(offset+10),1);assert.equal(wav.readUInt32LE(offset+12),48000);assert.equal(wav.readUInt16LE(offset+22),16);validFormat=true;}if(kind==='data')pcm=wav.subarray(offset+8,offset+8+length);offset+=8+length+(length%2);}
 assert.ok(validFormat&&pcm);
 assert.equal(pcm.length,2*96000);
 const amplitude=(start,hz)=>{const first=Math.round(start*48000),count=4800;let sine=0,cosine=0;for(let i=0;i<count;i++){const value=pcm.readInt16LE((first+i)*2)/32768,phase=2*Math.PI*hz*i/48000;sine+=value*Math.sin(phase);cosine+=value*Math.cos(phase);}return 2*Math.hypot(sine,cosine)/count;};
 const audio={voice:amplitude(.3,440),music:amplitude(1.3,660),mixedVoice:amplitude(.8,440),mixedMusic:amplitude(.8,660)};
 assert.ok(amplitude(.05,440)<.001&&amplitude(.05,660)<.001);assert.ok(audio.voice>.12&&amplitude(.3,660)<.001);assert.ok(audio.music>.06&&amplitude(1.3,440)<.001);assert.ok(amplitude(1.8,440)<.001&&amplitude(1.8,660)<.001);assert.ok(audio.mixedVoice/audio.mixedMusic>1.8&&audio.mixedVoice/audio.mixedMusic<2.2);
 for(const name of names)assert.equal(revision(await fs.readFile(join(work,name))),revisions[name]);
 await fs.writeFile(join(work,'decoded-evidence.json'),JSON.stringify({release,destination,probe,frames,audio,limits:['Pixel inspection, caption accuracy and playback require separate review.']},null,2));
 await save('actual pi video export decodes to 24 frames; independent PCM oracle verifies audio timing/gain; editable sources are preserved');
 const source=await fs.readFile(join(work,'index.html'),'utf8');assert.match(source,/<\/head>/i);
 const changed=source.replace('background:white','background:#dce9ff').replace('Mirket revised','Mirket updated');assert.notEqual(changed,source);
 const changedPath=join(work,'changed.html');await fs.writeFile(changedPath,changed);
 const staleAt=session.session.messages.length;
 await session.session.prompt(JSON.stringify({tool:'mcp__mirket__video_prepare',args:{...intent,source_path:changedPath,expected_revision:revisions['index.html'],destination_path:join(work,'stale.mp4')}}));
 const stale=session.session.messages.slice(staleAt).findLast(message=>message.role==='toolResult');assert.equal(stale?.isError,true);assert.match(JSON.stringify(stale),/revision|changed|stale/i);
 assert.equal(await fs.stat(join(work,'stale.mp4')).then(()=>true,()=>false),false);
 const changedIntent={...intent,source_path:changedPath,expected_revision:revision(changed),destination_path:join(work,'changed.mp4')};
 const changedPrepared=await call(session,'mcp__mirket__video_prepare',changedIntent),changedJob={...common,operation_id:changedPrepared.operation_id};
 await call(session,'mcp__mirket__video_start',changedJob);const changedResult=await terminal(changedJob);assert.equal(changedResult.status,'succeeded',JSON.stringify(changedResult));
 const changedFrame=join(work,'changed-middle.png');execFileSync(join(native,'ffmpeg'),['-v','error','-nostdin','-i',changedIntent.destination_path,'-ss','1','-frames:v','1',changedFrame],{timeout:10000});
 assert.notEqual(revision(await fs.readFile(changedFrame)),revision(await fs.readFile(frames[1])));
 await fs.writeFile(join(work,'changed-source-evidence.json'),JSON.stringify({changedIntent,changedResult,changedFrame,originalFrame:frames[1],staleRefusal:stale},null,2));
 await save('changed video source produces changed decoded pixels; stale source revision refuses before export');
 const unsafe=source.replace(/<\/head>/i,'<script src="https://example.invalid/forbidden.js"></script></head>');await fs.writeFile(join(work,'unsafe.html'),unsafe);
 const refusedIntent={...intent,source_path:join(work,'unsafe.html'),expected_revision:revision(unsafe),destination_path:join(work,'forbidden.mp4')};
 const refusal=await call(session,'mcp__mirket__video_prepare',refusedIntent),refusedJob={...common,operation_id:refusal.operation_id};await call(session,'mcp__mirket__video_start',refusedJob);const refused=await terminal(refusedJob);
 assert.equal(refused.status,'failed');assert.match(refused.error,/remote_or_absolute_asset_refused/);assert.equal(refused.effect,'not_attempted');assert.equal(await fs.stat(refusedIntent.destination_path).then(()=>true,()=>false),false);
 await call(session,'tool_search',{query:'video_cancel',limit:2});
 const cancelIntent={...intent,destination_path:join(work,'cancelled.mp4')};const pending=await call(session,'mcp__mirket__video_prepare',cancelIntent),cancelJob={...common,operation_id:pending.operation_id};await call(session,'mcp__mirket__video_start',cancelJob);
 const activeDeadline=Date.now()+15000;let running=false;while(Date.now()<activeDeadline){const value=await status(cancelJob);if(value?.status==='running'&&value.scratch_path){running=true;break;}assert.ok(!value||['queued','running'].includes(value.status),JSON.stringify(value));await new Promise(r=>setTimeout(r,100));}assert.equal(running,true);
 await new Promise(r=>setTimeout(r,400));await call(session,'mcp__mirket__video_cancel',cancelJob);const cancelled=await terminal(cancelJob);assert.equal(cancelled.status,'cancelled');assert.equal(cancelled.effect,'not_attempted');assert.equal(await fs.stat(cancelIntent.destination_path).then(()=>true,()=>false),false);
 await fs.writeFile(join(work,'failure-evidence.json'),JSON.stringify({refused,cancelled},null,2));
 await save('actual pi remote video asset refusal and in-flight cancellation leave no published output and confirm owned scratch cleanup');
 console.log('VIDEO_EVIDENCE',work);
}
