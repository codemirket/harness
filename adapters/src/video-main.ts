import fs from "node:fs/promises";
import path from "node:path";
import { createHash } from "node:crypto";
import { format } from "node:util";
import { exact, exportVideo, readFile, treeBytes, validateRequest, VideoError, MAX_TIMEOUT, type Paths } from "./video-export.js";

process.umask(0o077);
const controller=new AbortController();
for(const signal of ["SIGINT","SIGTERM","SIGHUP"] as const)process.on(signal,()=>controller.abort());
process.stdout.on("error",()=>controller.abort());
// Producer/library logs cannot corrupt the bounded stdio protocol or expose
// source contents as diagnostics. Count and bound them without retaining text.
let diagnosticBytes=0;
for(const method of ["log","info","debug","warn","error"] as const)console[method]=(...args:unknown[])=>{diagnosticBytes+=Buffer.byteLength(format(...args));if(diagnosticBytes>16384)controller.abort();};
const emit=async(value:unknown)=>{const frame=JSON.stringify(value)+"\n";if(Buffer.byteLength(frame)>65536)throw new VideoError("response_budget");await new Promise<void>((resolve,reject)=>process.stdout.write(frame,(error)=>error?reject(error):resolve()));};
const sha=(b:Uint8Array)=>createHash("sha256").update(b).digest("hex");
async function policy():Promise<void>{
  const moduleRoot=path.resolve(import.meta.dirname,"../node_modules");
  const record:unknown=JSON.parse((await readFile(path.join(moduleRoot,".mirket-video-policy.json"),65536)).toString("utf8"));
  if(!exact(record,["schema","policy_version","development_candidate","files"])||record.schema!==1||record.policy_version!==1||record.development_candidate!==true||!Array.isArray(record.files)||record.files.length!==8)throw new VideoError("private_policy_patch_required");
  for(const item of record.files){if(!exact(item,["path","original_sha256","patched_sha256"])||typeof item.path!=="string"||item.path.includes("..")||typeof item.patched_sha256!=="string"||!/^[@a-zA-Z0-9_./-]+$/.test(item.path))throw new VideoError("invalid_private_policy_record");
    if(sha(await readFile(path.join(moduleRoot,item.path),2*1024*1024))!==item.patched_sha256)throw new VideoError("private_policy_bytes_changed");}
}
async function request():Promise<unknown>{
  return await new Promise((resolve,reject)=>{const chunks:Buffer[]=[];let size=0,settled=false;const timer=setTimeout(()=>reject(new VideoError("request_deadline")),10000);
    process.stdin.on("data",(chunk:Buffer)=>{if(settled){controller.abort();return;}size+=chunk.length;if(size>65536){clearTimeout(timer);reject(new VideoError("request_budget"));process.stdin.pause();return;}chunks.push(chunk);
      const bytes=Buffer.concat(chunks),newline=bytes.indexOf(10);if(newline>=0){clearTimeout(timer);settled=true;try{if(newline!==bytes.length-1)throw new Error();resolve(JSON.parse(bytes.subarray(0,newline).toString("utf8")));}catch{reject(new VideoError("invalid_request_frame"));}}});
    process.stdin.once("end",()=>{clearTimeout(timer);controller.abort();if(!settled)reject(new VideoError("owner_disconnected"));});
    process.stdin.once("error",()=>{clearTimeout(timer);reject(new VideoError("request_read_failed"));});
  });
}
let release="",job:string|null=null,started=false;
try{
  const args=process.argv.slice(2);if(args.length!==8||args[0]!=="--release-id"||args[2]!=="--chromium"||args[4]!=="--ffmpeg"||args[6]!=="--ffprobe"||!/^[a-zA-Z0-9_.:-]{1,256}$/.test(args[1]!))throw new VideoError("invalid_startup");
  release=args[1]!;const paths:Paths={chromium:args[3]!,ffmpeg:args[5]!,ffprobe:args[7]!};
  for(const file of Object.values(paths)){if(!path.isAbsolute(file)||await fs.realpath(file)!==file||(await fs.lstat(file)).isSymbolicLink())throw new VideoError("explicit_private_tool_required");}
  await policy();
  process.env.MIRKET_VIDEO_POLICY_VERSION="1";process.env.MIRKET_VIDEO_CHROMIUM=paths.chromium;
  process.env.HYPERFRAMES_FFMPEG_PATH=paths.ffmpeg;process.env.HYPERFRAMES_FFPROBE_PATH=paths.ffprobe;
  for(const name of ["profile","cache","config","crash"])await fs.mkdir(path.join(process.cwd(),name),{mode:0o700});
  await emit({type:"ready",protocol_version:1,release_id:release,operation:"video.export",policy_version:1,max_frame_bytes:65536});
  const value=validateRequest(await request(),release);job=value.job_id;started=true;
  process.stdin.on("close",()=>{});
  let workError:unknown;
  const timer=setTimeout(()=>controller.abort(),Math.min(value.timeout_ms,MAX_TIMEOUT));
  let scanning=false;
  const watch=setInterval(()=>{if(scanning)return;scanning=true;void treeBytes(process.cwd()).catch((error)=>{workError=error;controller.abort();}).finally(()=>{scanning=false;});},250);watch.unref();
  try{let result;try{result=await exportVideo(value,paths,controller.signal);}catch(error){throw workError??error;}if(workError)throw workError;if(controller.signal.aborted)throw new VideoError("cancelled");
    await emit({type:"result",protocol_version:1,release_id:release,job_id:job,status:"staged",output_path:value.output_path,...result});
  }finally{clearTimeout(timer);clearInterval(watch);}
}catch(error){
  process.exitCode=1;
  const code=error instanceof VideoError?error.code:"video_export_failed";
  const diagnostics=error instanceof VideoError?error.diagnostics:undefined;
  const extra=diagnostics?.length?{diagnostics}:{};
  if(started)await fs.writeFile(path.join(process.cwd(),"failure.json"),JSON.stringify({code,...extra,diagnostic:error instanceof Error?String(error.stack).slice(0,8192):"unknown"})+"\n",{flag:"wx",mode:0o600}).catch(()=>{});
  await emit({type:"error",protocol_version:1,release_id:release,job_id:job,code,output_may_exist:started,...extra}).catch(()=>{});
}finally{
  process.stdin.destroy();
  process.stdout.end();
}
