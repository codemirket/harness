// Snapshot identity, with optional decoded correspondence. Neither judges aesthetics.
import {readFile,lstat} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {resolve,relative,isAbsolute,sep} from 'node:path';
import {fileURLToPath} from 'node:url';
const root=resolve(process.argv[2] ?? fileURLToPath(new URL('..',import.meta.url)));
const manifest=JSON.parse(await readFile(resolve(root,'qa/review-pack.json'),'utf8'));
const files=[manifest.source_video,...manifest.frames,...Object.entries(manifest.composition_inputs).map(([path,sha256])=>({path,sha256}))];
const failures=[];
if(manifest.human_review && manifest.human_review.video_sha256!==manifest.source_video.sha256){
 failures.push({path:'human_review.video_sha256',reason:'acceptance belongs to another video'});
}
for(const {path,sha256} of files){
 const target=resolve(root,path),rel=relative(root,target);
 if(isAbsolute(path)||rel==='..'||rel.startsWith(`..${sep}`)){failures.push({path,reason:'outside pack'});continue;}
 try{
  let cursor=root;for(const part of rel.split(sep)){cursor=resolve(cursor,part);if((await lstat(cursor)).isSymbolicLink())throw Error('symlink');}
  const actual=createHash('sha256').update(await readFile(target)).digest('hex');
  if(actual!==sha256)failures.push({path,reason:'snapshot changed'});
 }catch(error){failures.push({path,reason:error.code??error.message});}
}
const [ffmpeg,ffprobe]=process.argv.slice(3),frameChecks=[];
const identityMatches=failures.length===0;
if(ffmpeg||ffprobe){
 if(!ffmpeg||!ffprobe||!isAbsolute(ffmpeg)||!isAbsolute(ffprobe))throw Error('Supply both explicit absolute FFmpeg and FFprobe paths');
 if(identityMatches){
  const run=(command,args,maxBuffer=4*1024*1024)=>{
   const r=spawnSync(command,args,{timeout:30000,maxBuffer});
   if(r.status!==0)throw Error(r.stderr?.toString()||r.error?.message||'Decode failed');return r.stdout;
  };
  const probe=(file)=>JSON.parse(run(ffprobe,['-v','error','-count_frames','-show_streams','-of','json',file]));
  const rgb=(file,time)=>run(ffmpeg,['-v','error','-nostdin','-i',file,...(time===undefined?[]:['-ss',String(time)]),'-frames:v','1','-f','rawvideo','-pix_fmt','rgb24','pipe:1']);
  try{
   const video=resolve(root,manifest.source_video.path),profile=manifest.profile,streams=probe(video).streams,v=streams.find(s=>s.codec_type==='video');
   const [n,d]=v.r_frame_rate.split('/').map(Number);
   frameChecks.push({name:'saved video profile',pass:v.width===profile.width&&v.height===profile.height&&n/d===profile.fps&&Number(v.nb_read_frames)===profile.frames&&Math.abs(Number(v.duration)-profile.duration_seconds)<.001&&streams.every(s=>s.codec_type!=='audio')});
   const current=manifest.frames.filter(f=>f.source===manifest.source_video.path);
   if(current.length!==5)throw Error('Expected the five current Daybreak review frames');
   for(const frame of current){
    if(!Number.isFinite(frame.seconds)||frame.seconds<0||frame.seconds>=profile.duration_seconds)throw Error('Invalid frame timestamp');
    const path=resolve(root,frame.path),p=probe(path).streams[0];
    frameChecks.push({name:frame.path,seconds:frame.seconds,pass:p.width===profile.width&&p.height===profile.height&&rgb(video,frame.seconds).equals(rgb(path))});
   }
  }catch(error){frameChecks.push({name:'decoded correspondence',pass:false,reason:error.message});}
 }
}
console.log(JSON.stringify({checked:files.length,failures,identity_matches:identityMatches,frame_checks:frameChecks,frame_correspondence_checked:frameChecks.length>0,historical_before_frame:'identity only; original export evidence retained separately',visual_quality:'not established by this check'},null,2));
if(failures.length||frameChecks.some(c=>!c.pass))process.exitCode=1;
