import fs from "node:fs/promises";
import { constants, type Stats } from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { spawn } from "node:child_process";

export const MAX_INPUT = 16 * 1024 * 1024;
export const MAX_OUTPUT = 2 * 1024 * 1024 * 1024;
// Frame tree + completed archive/output + 1 GiB of finite producer overhead.
export const MAX_WORK = 5 * 1024 * 1024 * 1024;
export const MAX_TIMEOUT = 30 * 60 * 1000;
export type VideoFormat = "mp4" | "webm" | "gif" | "png_sequence_tar";
export interface Profile { width: number; height: number; fps: { numerator: number; denominator: number }; frame_count: number; format: VideoFormat; captions_path: string | null }
interface Source { path: string; revision: string; bytes: number }
export interface VideoRequest { protocol_version: number; release_id: string; job_id: string; job_root: string; input_root: string; entry_file: string; sources: Source[]; output_path: string; profile: Profile; timeout_ms: number; max_output_bytes: number }
export interface Paths { chromium: string; ffmpeg: string; ffprobe: string }
export class VideoError extends Error { constructor(public readonly code: string) { super(code); } }
function fail(code: string): never { throw new VideoError(code); }
const record = (v: unknown): v is Record<string, unknown> => !!v && typeof v === "object" && !Array.isArray(v);
export function exact(value: unknown, keys: string[]): value is Record<string, unknown> { return record(value) && Object.keys(value).length === keys.length && keys.every((key) => Object.hasOwn(value,key)); }
export const digest = (bytes: Uint8Array): string => "sha256:" + createHash("sha256").update(bytes).digest("hex");
export const relative = (v: unknown): v is string => typeof v === "string" && v.length > 0 && v.length <= 1024 && !/[\\:\0]/.test(v) && v.split("/").every((part) => part !== "" && part !== "." && part !== "..");
const absolute = (v: unknown): v is string => typeof v === "string" && path.isAbsolute(v) && path.normalize(v) === v && v.length <= 4096 && !v.includes("\0");
const integer = (v: unknown, min: number, max: number): v is number => Number.isSafeInteger(v) && (v as number) >= min && (v as number) <= max;
export function validateRequest(value: unknown, release: string): VideoRequest {
  if (!exact(value,["protocol_version","release_id","job_id","job_root","input_root","entry_file","sources","output_path","profile","timeout_ms","max_output_bytes"])) fail("invalid_request");
  const r = value as unknown as VideoRequest, p = r.profile;
  if (r.protocol_version !== 1 || r.release_id !== release || !/^[a-zA-Z0-9_.:-]{1,128}$/.test(r.job_id) || !absolute(r.job_root) || r.job_root !== process.cwd()
    || r.input_root !== path.join(r.job_root,"source") || !relative(r.entry_file) || !r.entry_file.endsWith(".html") || !absolute(r.output_path) || path.dirname(r.output_path) !== r.job_root
    || !integer(r.timeout_ms,100,MAX_TIMEOUT) || !integer(r.max_output_bytes,1,MAX_OUTPUT)
    || !exact(p,["width","height","fps","frame_count","format","captions_path"]) || !exact(p.fps,["numerator","denominator"])
    || !integer(p.width,2,1920) || !integer(p.height,2,1920) || p.width % 2 !== 0 || p.height % 2 !== 0 || p.width*p.height > 1920*1080
    || !integer(p.fps.numerator,1,60000) || !integer(p.fps.denominator,1,1001) || p.fps.numerator < p.fps.denominator || p.fps.numerator > 60*p.fps.denominator
    || !integer(p.frame_count,1,18000) || p.frame_count*p.fps.denominator > 600*p.fps.numerator
    || !["mp4","webm","gif","png_sequence_tar"].includes(p.format) || (p.captions_path !== null && !relative(p.captions_path))
    || !Array.isArray(r.sources) || r.sources.length < 1 || r.sources.length > 128) fail("invalid_request");
  const ext = p.format === "png_sequence_tar" ? "tar" : p.format;
  if (r.output_path !== path.join(r.job_root,`result.${ext}`)) fail("invalid_output_stage");
  let total = 0; const names = new Set<string>();
  for (const s of r.sources) {
    if (!exact(s,["path","revision","bytes"]) || !relative(s.path) || !/^sha256:[a-f0-9]{64}$/.test(s.revision) || !integer(s.bytes,0,MAX_INPUT) || names.has(s.path)) fail("invalid_source");
    names.add(s.path); total += s.bytes;
  }
  if (total > MAX_INPUT || !names.has(r.entry_file) || (p.captions_path !== null && !names.has(p.captions_path))) fail("invalid_source");
  return r;
}
export async function readFile(file: string, limit: number): Promise<Buffer> {
  if (await fs.realpath(file) !== file) fail("source_symlink_or_ancestry_changed");
  const before = await fs.lstat(file);
  if (!before.isFile() || before.isSymbolicLink() || before.size > limit) fail("source_type_or_size");
  const handle = await fs.open(file, constants.O_RDONLY | constants.O_NOFOLLOW);
  try {
    const opened = await handle.stat(); if (opened.ino !== before.ino || opened.dev !== before.dev) fail("source_changed");
    const buffer = Buffer.alloc(before.size + 1); let offset = 0;
    while (offset < buffer.length) { const n = await handle.read(buffer,offset,buffer.length-offset,offset); if (!n.bytesRead) break; offset += n.bytesRead; }
    const after = await fs.lstat(file);
    if (offset !== before.size || after.ino !== before.ino || after.dev !== before.dev || after.mtimeMs !== before.mtimeMs || after.size !== before.size) fail("source_changed");
    return buffer.subarray(0,offset);
  } finally { await handle.close(); }
}
// Unlike readFile, this video-body reader retains only a fixed 64 KiB buffer
// and a small prefix. Metadata/ancestry are rechecked on both path and handle.
export async function fileRevision(file: string, limit: number, signal?: AbortSignal, consume?: (chunk: Buffer) => Promise<void>): Promise<{revision:string;bytes:number;prefix:Buffer}> {
  if (!integer(limit,1,MAX_OUTPUT)) fail("invalid_file_budget");
  if (await fs.realpath(file) !== file) fail("source_symlink_or_ancestry_changed");
  const before = await fs.lstat(file);
  if (!before.isFile() || before.isSymbolicLink() || before.size > limit) fail("source_type_or_size");
  const handle = await fs.open(file,constants.O_RDONLY | constants.O_NOFOLLOW);
  const same = (value: Stats) => value.isFile() && value.ino===before.ino && value.dev===before.dev && value.size===before.size && value.mtimeMs===before.mtimeMs && value.ctimeMs===before.ctimeMs;
  try {
    if (!same(await handle.stat())) fail("source_changed");
    const hash=createHash("sha256"),buffer=Buffer.alloc(65536),prefix=Buffer.alloc(Math.min(33,before.size));let total=0;
    for (;;) {
      if (signal?.aborted) fail("cancelled");
      const {bytesRead:n}=await handle.read(buffer,0,buffer.length,null);if(!n)break;
      if(total+n>limit || total+n>before.size)fail("source_type_or_size");
      if(total<prefix.length)buffer.copy(prefix,total,0,Math.min(n,prefix.length-total));
      hash.update(buffer.subarray(0,n));
      if(consume)await consume(buffer.subarray(0,n));
      total+=n;
    }
    if(total!==before.size || await fs.realpath(file)!==file || !same(await fs.lstat(file)) || !same(await handle.stat()))fail("source_changed");
    return {revision:"sha256:"+hash.digest("hex"),bytes:total,prefix};
  } finally { await handle.close(); }
}

const textExtensions = new Set([".html",".css",".js",".mjs",".json",".svg",".vtt",".srt"]);
const assetExtensions = new Set([...textExtensions,".png",".jpg",".jpeg",".webp",".gif",".ttf",".otf",".woff",".woff2",".wav",".mp3",".m4a",".aac",".opus",".mp4",".webm",".ogg"]);
function references(text: string, ext: string): string[] {
  const refs: string[] = [];
  // These unsupported dynamic mechanisms are explicit refusals. JavaScript is
  // authored source, not hostile code executed inside a security sandbox.
  if (/\b(?:fetch|XMLHttpRequest|WebSocket|EventSource|importScripts|SharedWorker)\s*\(|\bimport\s*\(|\bserviceWorker\b|\bsrcset\s*=|\bdata-requires-webgpu\b|\bdata-composition-src\b/i.test(text)) fail("unsupported_dynamic_dependency_or_effect");
  const patterns = ext === ".html" || ext === ".svg"
    ? [/(?:\bsrc|\bhref|\bposter|\bxlink:href)\s*=\s*["']([^"']+)["']/gi,/url\(\s*["']?([^'"\s)]+)["']?\s*\)/gi,/\bfrom\s*["']([^"']+)["']/g]
    : ext === ".css" ? [/url\(\s*["']?([^'"\s)]+)["']?\s*\)/gi,/@import\s*["']([^"']+)["']/gi]
    : ext === ".js" || ext === ".mjs" ? [/\b(?:from|import)\s*["']([^"']+)["']/g] : [];
  for (const pattern of patterns) for (const match of text.matchAll(pattern)) refs.push(match[1]!);
  return refs;
}
interface Cue { start_frame: number; end_frame: number; text: string }
export function cues(value: unknown, frames: number): Cue[] {
  if (!Array.isArray(value) || value.length > 1000) fail("invalid_caption_cues");
  let previous = 0;
  return value.map((v): Cue => {
    if (!exact(v,["start_frame","end_frame","text"]) || !integer(v.start_frame,previous,frames-1) || !integer(v.end_frame,(v.start_frame as number)+1,frames)
      || typeof v.text !== "string" || !v.text.length || Buffer.byteLength(v.text)>4096 || /[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/.test(v.text) || /[\uD800-\uDFFF]/u.test(v.text)) fail("invalid_caption_cues");
    previous = v.end_frame as number; return v as unknown as Cue;
  });
}
function timestamp(frame: number, fps: Profile["fps"], separator: string): string {
  const ms = Math.round(frame*fps.denominator*1000/fps.numerator), h = Math.floor(ms/3600000), m = Math.floor(ms/60000)%60, s = Math.floor(ms/1000)%60;
  return `${String(h).padStart(2,"0")}:${String(m).padStart(2,"0")}:${String(s).padStart(2,"0")}${separator}${String(ms%1000).padStart(3,"0")}`;
}
export function captionSidecars(items: Cue[], fps: Profile["fps"]): { srt: Buffer; vtt: Buffer } {
  const body = (separator: string, numbered: boolean) => items.map((c,i)=>`${numbered?`${i+1}\n`:""}${timestamp(c.start_frame,fps,separator)} --> ${timestamp(c.end_frame,fps,separator)}\n${c.text}\n`).join("\n");
  return { srt: Buffer.from(body(",",true)), vtt: Buffer.from("WEBVTT\n\n"+body(".",false)) };
}
const escape = (value: string) => value.replaceAll("&","&amp;").replaceAll("<","&lt;").replaceAll(">","&gt;").replaceAll('"',"&quot;");
export async function preflight(request: VideoRequest): Promise<{ entry: Buffer; captions: Cue[]; sources_revision: string; expected_audio: boolean }> {
  const selected = new Map<string,Buffer>();
  for (const source of request.sources) {
    if (!assetExtensions.has(path.extname(source.path).toLowerCase())) fail("unsupported_asset_type");
    const bytes = await readFile(path.join(request.input_root,source.path),MAX_INPUT);
    if (bytes.length !== source.bytes || digest(bytes) !== source.revision) fail("source_revision_changed");
    selected.set(source.path,bytes);
  }
  const decoder = new TextDecoder("utf-8",{fatal:true});
  for (const [name,bytes] of selected) {
    const ext = path.extname(name); if (!textExtensions.has(ext)) continue;
    const text = decoder.decode(bytes); if (text.includes("\0")) fail("invalid_source_text");
    for (let ref of references(text,ext)) {
      if (ref.startsWith("#") || ref.startsWith("data:")) continue;
      if (/[?#]/.test(ref)) fail("asset_query_or_fragment_refused");
      try { ref = decodeURIComponent(ref); } catch { fail("invalid_asset_reference"); }
      if (/^(?:[a-z][a-z0-9+.-]*:|\/)/i.test(ref) || ref.includes("\\")) fail("remote_or_absolute_asset_refused");
      const local = path.posix.normalize(path.posix.join(path.posix.dirname(name),ref));
      if (!relative(local) || !selected.has(local)) fail("missing_or_unselected_asset");
    }
  }
  const entry = selected.get(request.entry_file)!; const html = decoder.decode(entry), p = request.profile;
  const root = html.match(/<[^>]*\bdata-composition-id\s*=\s*["'][^"']+["'][^>]*>/i)?.[0];
  const attr = (name: string) => root?.match(new RegExp(`\\b${name}\\s*=\\s*["']([^"']+)["']`,"i"))?.[1];
  const duration = p.frame_count*p.fps.denominator/p.fps.numerator;
  if (!root || !/\bdata-no-timeline(?:\s|=|>)/i.test(root) || Number(attr("data-width"))!==p.width || Number(attr("data-height"))!==p.height || Math.abs(Number(attr("data-duration"))-duration)>1e-6) fail("authored_root_profile_mismatch");
  if (![...selected.keys()].some((name)=>/\.(ttf|otf|woff2?)$/.test(name)) || !/@font-face\s*\{/.test([...selected].filter(([n])=>/\.(html|css)$/.test(n)).map(([,b])=>decoder.decode(b)).join("\n"))) fail("explicit_local_font_required");
  let captions: Cue[] = [];
  if (p.captions_path !== null) { try { captions = cues(JSON.parse(decoder.decode(selected.get(p.captions_path)!)),p.frame_count); } catch (error) { if(error instanceof VideoError)throw error; fail("invalid_caption_cues"); } }
  return {entry,captions,sources_revision:digest(Buffer.from(JSON.stringify(request.sources))),expected_audio:/<audio\b[^>]*\bsrc\s*=/i.test(html)};
}
function burnedCaptions(html: string, items: Cue[], profile: Profile): string {
  if (!items.length) return html;
  const duration = profile.frame_count*profile.fps.denominator/profile.fps.numerator;
  const styles = items.map((c,i)=>`@keyframes mirketCue${i}{${c.start_frame===0?"":"0%{opacity:0}"}${c.start_frame/profile.frame_count*100}%{opacity:1}${c.end_frame/profile.frame_count*100}%{opacity:0}${c.end_frame===profile.frame_count?"":"100%{opacity:0}"}}`).join("");
  const overlay = `<style>${styles}.mirket-caption{position:fixed;z-index:2147483647;left:5%;right:5%;bottom:5%;margin:0;text-align:center;white-space:pre-wrap;color:white;background:rgba(0,0,0,.75);font-size:${Math.max(16,Math.round(profile.height/22))}px;line-height:1.3;pointer-events:none}</style>`+items.map((c,i)=>`<p class="mirket-caption" style="animation:mirketCue${i} ${duration}s step-end both">${escape(c.text)}</p>`).join("");
  if(!/<\/body\s*>/i.test(html))fail("caption_injection_requires_body");return html.replace(/<\/body\s*>/i,overlay+"</body>");
}
export async function runTool(file: string, args: string[], signal: AbortSignal, maxBytes = 65536): Promise<Buffer> {
  if (signal.aborted) fail("cancelled");
  return await new Promise((resolve,reject)=>{
    const child=spawn(file,args,{stdio:["ignore","pipe","pipe"],detached:false,windowsHide:true,env:process.env});
    const chunks:Buffer[]=[];let out=0,errors=0,reason:string|undefined;
    const stop=()=>{reason="cancelled";child.kill("SIGKILL");};signal.addEventListener("abort",stop,{once:true});
    child.stdout.on("data",(chunk:Buffer)=>{out+=chunk.length;if(out>maxBytes){reason="media_output_budget";child.kill("SIGKILL");}else chunks.push(chunk);});
    child.stderr.on("data",(chunk:Buffer)=>{errors+=chunk.length;if(errors>65536){reason="media_diagnostic_budget";child.kill("SIGKILL");}});
    child.once("error",()=>{signal.removeEventListener("abort",stop);reject(new VideoError("private_media_launch_failed"));});
    child.once("close",(code)=>{signal.removeEventListener("abort",stop);if(reason||code!==0)reject(new VideoError(reason??"media_command_failed"));else resolve(Buffer.concat(chunks));});
  });
}
async function probe(file:string,paths:Paths,signal:AbortSignal,frames=false):Promise<Record<string,unknown>>{
  const args=["-v","error",...(frames?["-count_frames"]:[]),"-show_streams","-show_format","-of","json",file];
  try{const value:unknown=JSON.parse((await runTool(paths.ffprobe,args,signal)).toString("utf8"));if(!record(value))fail("invalid_media_probe");return value;}catch(error){if(error instanceof VideoError)throw error;return fail("invalid_media_probe");}
}
async function verifyMedia(file:string,request:VideoRequest,paths:Paths,signal:AbortSignal,expectedAudio:boolean):Promise<Record<string,unknown>>{
  const info=await probe(file,paths,signal,true),streams=info.streams;
  if(!Array.isArray(streams))fail("missing_media_streams");
  const video=streams.filter((s)=>record(s)&&s.codec_type==="video"),audio=streams.filter((s)=>record(s)&&s.codec_type==="audio");
  if(video.length!==1||video[0].width!==request.profile.width||video[0].height!==request.profile.height||Number(video[0].nb_read_frames)!==request.profile.frame_count)fail("media_frame_contract_mismatch");
  const expected=request.profile.format==="mp4"?"h264":request.profile.format==="webm"?"vp9":"gif";
  if(video[0].codec_name!==expected||audio.length>1||(request.profile.format==="gif"&&audio.length))fail("media_codec_contract_mismatch");
  if(expectedAudio&&request.profile.format!=="gif"&&audio.length!==1)fail("supplied_audio_stream_missing");
  if(audio.length&&audio[0].codec_name!==(request.profile.format==="mp4"?"aac":"opus"))fail("media_audio_codec_mismatch");
  const duration=request.profile.frame_count*request.profile.fps.denominator/request.profile.fps.numerator;
  const actual=record(info.format)?Number(info.format.duration):NaN;
  if(!Number.isFinite(actual)||Math.abs(actual-duration)>Math.max(.1,request.profile.fps.denominator/request.profile.fps.numerator))fail("media_duration_mismatch");
  // Complete independent decode uses the exact private media binary, distinct
  // from the producer's success result and container metadata.
  await runTool(paths.ffmpeg,["-v","error","-nostdin","-i",file,"-map","0","-f","null","-"],signal);
  return {video_codec:expected,audio_codec:audio[0]?.codec_name??null,frame_count:Number(video[0].nb_read_frames),duration_seconds:actual,complete_decode:true,alpha:"not_independently_qualified",player_playback:"not_exercised"};
}
export interface ArchiveEntry { path: string; bytes: Buffer }
// Small fixture helper only; production PNG archives use writeTar below.
export function tar(entries: ArchiveEntry[],maxBytes=20*1024*1024):Buffer{
  if(!integer(maxBytes,1,20*1024*1024))fail("output_budget");
  const seen=new Set<string>(),parts:Buffer[]=[];let total=1024;
  for(const entry of entries){if(!relative(entry.path)||entry.path.includes("/")||Buffer.byteLength(entry.path)>100||seen.has(entry.path))fail("invalid_archive_entry");seen.add(entry.path);
    const header=Buffer.alloc(512);header.write(entry.path,0,100,"utf8");header.write("0000600\0",100,8);header.write("0000000\0",108,8);header.write("0000000\0",116,8);header.write(entry.bytes.length.toString(8).padStart(11,"0")+"\0",124,12);header.write("00000000000\0",136,12);header.fill(32,148,156);header[156]=48;header.write("ustar\0",257,6);header.write("00",263,2);
    const sum=header.reduce((a,b)=>a+b,0);header.write(sum.toString(8).padStart(6,"0")+"\0 ",148,8);
    const padding=Buffer.alloc((512-entry.bytes.length%512)%512);total+=512+entry.bytes.length+padding.length;if(total>maxBytes)fail("output_budget");parts.push(header,entry.bytes,padding);
  }return Buffer.concat([...parts,Buffer.alloc(1024)],total);
}
export interface ArchiveFileEntry { path:string; source:string|Buffer; bytes:number; revision:string }
function archiveHeader(name:string,size:number):Buffer {
  const header=Buffer.alloc(512);header.write(name,0,100,"utf8");header.write("0000600\0",100,8);header.write("0000000\0",108,8);header.write("0000000\0",116,8);header.write(size.toString(8).padStart(11,"0")+"\0",124,12);header.write("00000000000\0",136,12);header.fill(32,148,156);header[156]=48;header.write("ustar\0",257,6);header.write("00",263,2);
  const sum=header.reduce((a,b)=>a+b,0);header.write(sum.toString(8).padStart(6,"0")+"\0 ",148,8);return header;
}
export async function writeTar(entries:ArchiveFileEntry[],output:string,maxBytes:number,signal:AbortSignal):Promise<void>{
  if(!integer(maxBytes,1,MAX_OUTPUT)||entries.length>18004)fail("output_budget");
  const seen=new Set<string>();let total=1024;
  // Refuse the complete extent before creating an output or copying any body.
  for(const entry of entries){
    if(!relative(entry.path)||entry.path.includes("/")||Buffer.byteLength(entry.path)>100||seen.has(entry.path)||!integer(entry.bytes,0,MAX_OUTPUT)||!/^sha256:[a-f0-9]{64}$/.test(entry.revision))fail("invalid_archive_entry");
    seen.add(entry.path);total+=512+Math.ceil(entry.bytes/512)*512;if(total>maxBytes)fail("output_budget");
  }
  const handle=await fs.open(output,constants.O_WRONLY|constants.O_CREAT|constants.O_EXCL|constants.O_NOFOLLOW,0o600);
  const write=async(bytes:Buffer)=>{let offset=0;while(offset<bytes.length){if(signal.aborted)fail("cancelled");const n=await handle.write(bytes,offset,bytes.length-offset,null);if(!n.bytesWritten)fail("archive_write_failed");offset+=n.bytesWritten;}};
  try{
    for(const entry of entries){
      await write(archiveHeader(entry.path,entry.bytes));
      if(typeof entry.source==="string"){
        const actual=await fileRevision(entry.source,maxBytes,signal,write);
        if(actual.bytes!==entry.bytes||actual.revision!==entry.revision)fail("archive_source_changed");
      }else{
        if(entry.source.length!==entry.bytes||digest(entry.source)!==entry.revision)fail("archive_source_changed");
        await write(entry.source);
      }
      await write(Buffer.alloc((512-entry.bytes%512)%512));
    }
    // Node's permission model denies fsync even for an allowed private file.
    // This is disposable render scratch: Rust verifies the completed archive,
    // then syncs its separate publication stage before exposing the artifact.
    await write(Buffer.alloc(1024));
  }finally{await handle.close();}
}

export async function treeBytes(root:string):Promise<number>{
  let total=0,count=0;const pending=[root];
  while(pending.length){const dir=pending.pop()!;let entries;
    try{entries=await fs.readdir(dir,{withFileTypes:true});}
    catch(error){if(dir!==root&&(error as NodeJS.ErrnoException).code==="ENOENT")continue;throw error;}
    for(const entry of entries){if(++count>25000)fail("work_tree_budget");const file=path.join(dir,entry.name);
    // Chromium creates transient singleton symlinks and sockets. Count entries
    // but never follow them. Source and final-artifact readers independently
    // require canonical regular files; this scan only measures owned disk use.
    if(entry.isSymbolicLink())continue;
    if(entry.isDirectory())pending.push(file);else if(entry.isFile()){try{total+=(await fs.lstat(file)).size;}catch(error){if((error as NodeJS.ErrnoException).code==="ENOENT")continue;throw error;}if(total>MAX_WORK)fail("work_tree_budget");}}}
  return total;
}
export async function exportVideo(request:VideoRequest,paths:Paths,signal:AbortSignal):Promise<{output_revision:string;output_bytes:number;evidence:Record<string,unknown>}>{
  const prepared=await preflight(request),p=request.profile;
  // Every tool is explicitly selected before producer import, encoder discovery
  // or browser launch. No inherited HF/Puppeteer/provider configuration exists.
  const encoders=(await runTool(paths.ffmpeg,["-hide_banner","-encoders"],signal,131072)).toString();
  const formats=(await runTool(paths.ffmpeg,["-hide_banner","-formats"],signal,131072)).toString();
  for(const name of (p.format==="mp4"?["libx264","aac"]:p.format==="webm"?["libvpx-vp9","libopus"]:p.format==="gif"?["gif","png"]:["png","aac"]))if(!new RegExp(`\\b${name}\\b`).test(encoders))fail("required_encoder_unavailable");
  if(request.sources.some((source)=>/\.(wav|mp3|m4a|aac|opus|mp4|webm|ogg)$/.test(source.path))&&(!/\bpcm_f32le\b/.test(encoders)||!/^\s*.E.\s+ipod\s/m.test(formats)))fail("required_audio_intermediate_profile_unavailable");
  for(const source of request.sources.filter((s)=>/\.(wav|mp3|m4a|aac|opus|mp4|webm|ogg)$/.test(s.path))){const info=await probe(path.join(request.input_root,source.path),paths,signal);if(!Array.isArray(info.streams)||!info.streams.length)fail("input_media_unreadable");}
  if(prepared.captions.length)await fs.writeFile(path.join(request.input_root,request.entry_file),burnedCaptions(prepared.entry.toString("utf8"),prepared.captions,p));
  const producer=await import("@hyperframes/producer");
  const fps={num:p.fps.numerator,den:p.fps.denominator};
  // The exact candidate's EngineConfig narrows fps to the distributed route's
  // preset union, while its local Fps API accepts rational rates. Rendering uses
  // the rational job field; the engine config receives the numeric value.
  const producerConfig=producer.resolveConfig({fps:(p.fps.numerator/p.fps.denominator) as 24,quality:"draft",format:"png",chromePath:paths.chromium,forceScreenshot:true,useDrawElement:false,enableStreamingEncode:false,concurrency:1,enableBrowserPool:false,disableGpu:true,browserGpuMode:"software",extractCacheDir:path.join(request.job_root,"cache")});
  const logger={debug:()=>{},info:()=>{},warn:()=>{},error:()=>{}};
  const job=producer.createRenderJob({fps,quality:"draft",format:p.format==="png_sequence_tar"?"png-sequence":p.format,workers:1,useGpu:false,strictness:"strict",hdrMode:"force-sdr",entryFile:request.entry_file,producerConfig,logger});
  const rendered=p.format==="png_sequence_tar"?path.join(request.job_root,"frames"):request.output_path;
  await producer.executeRenderJob(job,request.input_root,rendered,()=>{},signal);
  if(job.status!=="complete")fail("producer_did_not_complete");
  await treeBytes(request.job_root);
  let evidence:Record<string,unknown>;
  if(p.format==="png_sequence_tar"){
    const names=(await fs.readdir(rendered)).sort(),entries:ArchiveFileEntry[]=[];
    for(let n=1;n<=p.frame_count;n++){
      const name=`frame_${String(n).padStart(6,"0")}.png`;if(!names.includes(name))fail("missing_png_frame");
      const source=path.join(rendered,name),actual=await fileRevision(source,request.max_output_bytes,signal),head=actual.prefix;
      if(head.length<33||!head.subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10]))||head.readUInt32BE(8)!==13||head.subarray(12,16).toString()!=="IHDR"||head.readUInt32BE(16)!==p.width||head.readUInt32BE(20)!==p.height)fail("png_frame_profile_mismatch");
      entries.push({path:name,source,bytes:actual.bytes,revision:actual.revision});
    }
    if(names.some((n)=>!/^frame_\d{6}\.png$/.test(n)&&n!=="audio.m4a")||names.filter((n)=>n.endsWith(".png")).length!==p.frame_count)fail("unexpected_png_sequence_entry");
    await runTool(paths.ffmpeg,["-v","error","-nostdin","-framerate",`${p.fps.numerator}/${p.fps.denominator}`,"-i",path.join(rendered,"frame_%06d.png"),"-f","null","-"],signal);
    if(prepared.expected_audio&&!names.includes("audio.m4a"))fail("supplied_audio_sidecar_missing");
    if(names.includes("audio.m4a")){
      const source=path.join(rendered,"audio.m4a");await runTool(paths.ffmpeg,["-v","error","-nostdin","-i",source,"-f","null","-"],signal);
      const actual=await fileRevision(source,request.max_output_bytes,signal);entries.push({path:"audio.m4a",source,bytes:actual.bytes,revision:actual.revision});
    }
    if(prepared.captions.length){const sidecars=captionSidecars(prepared.captions,p.fps);for(const [name,source]of [["captions.srt",sidecars.srt],["captions.vtt",sidecars.vtt]] as const)entries.push({path:name,source,bytes:source.length,revision:digest(source)});}
    const manifest={schema:1,format:p.format,width:p.width,height:p.height,fps:p.fps,frame_count:p.frame_count,entries:entries.map((e)=>({path:e.path,bytes:e.bytes,revision:e.revision}))};
    const source=Buffer.from(JSON.stringify(manifest)+"\n");if(source.length>4*1024*1024)fail("archive_manifest_budget");
    entries.push({path:"manifest.json",source,bytes:source.length,revision:digest(source)});
    await writeTar(entries,request.output_path,request.max_output_bytes,signal);
    await treeBytes(request.job_root);
    evidence={frame_count:p.frame_count,ordered_frame_decode:true,archive_manifest:true,audio_sidecar:names.includes("audio.m4a"),alpha:"not_independently_qualified"};
  }else evidence=await verifyMedia(rendered,request,paths,signal,prepared.expected_audio);
  const output=await fileRevision(request.output_path,request.max_output_bytes,signal);
  return {output_revision:output.revision,output_bytes:output.bytes,evidence:{...evidence,output_budget_bytes:request.max_output_bytes,scratch_budget_bytes:MAX_WORK,producer:"0.8.197",policy_version:1,source_revision:prepared.sources_revision,caption_cues:prepared.captions.length,source_editability:"project_owned_html_css_js_and_selected_assets",semantic_motion:"requires_visual_review",audio_gain_sync:"requires_content_review",direct_child_exit_status:"not_asserted_owned_group_is_reaped"}};
}
