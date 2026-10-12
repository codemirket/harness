// Mirket's bounded adapter for the byte-pinned HyperFrames 0.8.149 producer.
// This invokes upstream capture sessions, seeking, compilation, media mixing and encoding.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {pathToFileURL} from 'node:url';
import {spawn} from 'node:child_process';
import http from 'node:http';
import https from 'node:https';
import {verifyDecodedFrames} from './verification.mjs';
const root=process.env.MIRKET_HYPERFRAMES_ROOT;
const chromium=process.env.MIRKET_CHROMIUM;
const ffmpeg=process.env.MIRKET_FFMPEG;
const ffprobe=process.env.MIRKET_FFPROBE;
for (const [name,value] of Object.entries({root,chromium,ffmpeg,ffprobe})) {
  if (!value || !path.isAbsolute(value) || !fs.existsSync(value)) throw Error(`Missing pinned runtime ${name}`);
}
const log=(...args)=>process.stderr.write(args.map(v=>typeof v==='string'?v:JSON.stringify(v)).join(' ')+'\n');
console.log=log;console.warn=log;console.error=log;
const failures=[];
const allowedOrigins=new Set();
const browsers=new Set();
// Scratch is private and finite, including producer media extractions and browser profiles.
function scratchBytes(dir){let total=0;for(const e of fs.readdirSync(dir,{withFileTypes:true})){const p=path.join(dir,e.name);if(e.isSymbolicLink())throw Error('Scratch symlink refused');if(e.isDirectory())total+=scratchBytes(p);else if(e.isFile())total+=fs.statSync(p).size;}return total;}
const abort=new AbortController();
const scratchWatch=setInterval(()=>{try{if(scratchBytes(process.cwd())>256*1024*1024)throw Error('HyperFrames scratch exceeds 256 MiB');}catch(e){if(e.code==='ENOENT')return;if(!abort.signal.aborted){failures.push(e.message);abort.abort(e);for(const browser of browsers)browser.process()?.kill('SIGKILL');}}},250);scratchWatch.unref();
if(process.env.MIRKET_CONTAINER!=='1')throw Error('HyperFrames requires Mirket’s verified Podman runtime');
let browserSandbox=true;
function assertClean(){if(failures.length)throw Error(failures.slice(0,8).join('; '));}
function localUrl(value){const u=new URL(value);return u.protocol==='http:'&&allowedOrigins.has(u.origin);}
const realFetch=globalThis.fetch;
globalThis.fetch=(url,...args)=>{if(!localUrl(typeof url==='string'?url:url.url??String(url))) throw Error('Remote Node fetch refused');return realFetch(url,...args);};
// Producer remote localization can use Node HTTP as well as fetch. No external services.
for(const mod of [http,https])for(const method of ['request','get']){
 const original=mod[method].bind(mod);mod[method]=function(url,...args){
   const host=typeof url==='object'&&!(url instanceof URL)?url.hostname??url.host:new URL(String(url)).hostname;
   const port=typeof url==='object'&&!(url instanceof URL)?url.port??80:new URL(String(url)).port||80;
   if(mod===https||!allowedOrigins.has(`http://${host}:${port}`))throw Error('Unmapped or remote Node HTTP refused');
   return original(url,...args);
 };
}
process.env.HYPERFRAMES_BROWSER_PATH=chromium;
process.env.PRODUCER_HEADLESS_SHELL_PATH=chromium;
process.env.HYPERFRAMES_FFMPEG_PATH=ffmpeg;
process.env.HYPERFRAMES_FFPROBE_PATH=ffprobe;
process.env.HYPERFRAMES_EXTRACT_CACHE_DIR='off';
process.env.HYPERFRAMES_TELEMETRY_DISABLED='1';
process.env.PUPPETEER_SKIP_DOWNLOAD='true';
const puppeteer=(await import(pathToFileURL(path.join(root,'node_modules/puppeteer-core/lib/puppeteer/puppeteer-core.js')))).default;
const launch=puppeteer.launch.bind(puppeteer);
puppeteer.launch=async options=>{
 const isolationFeatures=new Set(['IsolateOrigins','site-per-process']);
 const args=(options.args??[]).filter(a=>!['--no-sandbox','--disable-setuid-sandbox'].includes(a))
   .flatMap(a=>{if(!a.startsWith('--disable-features='))return[a];const features=a.slice('--disable-features='.length).split(',').filter(f=>f&&!isolationFeatures.has(f));return features.length?[`--disable-features=${features.join(',')}`]:[];});
 // Re-rasterize complete tiles: partial SVG raster reuse differed by up to two
 // channel values when the same Solstice frame was reached in reverse order.
 args.push('--disable-partial-raster','--disable-background-networking','--disable-component-update','--no-first-run','--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE localhost, EXCLUDE 127.0.0.1','--proxy-server=direct://','--proxy-bypass-list=*');
 args.push('--no-sandbox');
 const browser=await launch({...options,args,pipe:true,executablePath:chromium,userDataDir:path.join(process.cwd(),`browser-${browsers.size}`),headless:true});
 browsers.add(browser);
 const actualArgs=browser.process()?.spawnargs??[];
 browserSandbox&&=!actualArgs.includes('--no-sandbox')&&!actualArgs.includes('--disable-setuid-sandbox');
 if(actualArgs.length===0||browserSandbox)throw Error('Browser isolation flags do not match the selected runtime');
 if(actualArgs.some(a=>a==='--disable-web-security'||a.startsWith('--disable-features=')&&a.slice('--disable-features='.length).split(',').some(f=>isolationFeatures.has(f))))throw Error('Browser sandbox/security was disabled');
 log('[Mirket browser flags]',actualArgs.filter(a=>a.startsWith('--disable-features=')||a.startsWith('--remote-debugging-')||a==='--disable-partial-raster'));
 const newPage=browser.newPage.bind(browser);
 browser.newPage=async()=>{
  const page=await newPage();await page.setRequestInterception(true);
  page.on('request',request=>{
   if(request.isInterceptResolutionHandled())return;
   const raw=request.url();let allow=false;
   try{const u=new URL(raw);allow=allowedOrigins.has(u.origin)&&!u.username&&!u.password&&!u.search&&!u.pathname.split('/').some(x=>decodeURIComponent(x)==='..');
     if(u.protocol==='data:'||u.protocol==='blob:')allow=!request.isNavigationRequest();
   }catch{}
   if(request.isNavigationRequest()&&request.frame()!==page.mainFrame())allow=false;
   if(allow)request.continue().catch(e=>failures.push(String(e)));else{failures.push(`Unmapped or remote browser request: ${raw.slice(0,200)}`);request.abort().catch(()=>{});}
  });
  page.on('pageerror',e=>failures.push(`Page error: ${e.message}`));
  page.on('response',r=>{if(r.status()>=400&&!r.url().endsWith('/favicon.ico'))failures.push(`Asset HTTP ${r.status()}: ${r.url()}`);});
  page.on('popup',p=>{failures.push('Popups are unsupported');p.close().catch(()=>{});});
  // Block browsing capabilities before any authored script executes.
  await page.evaluateOnNewDocument(()=>{
   for(const name of ['Worker','SharedWorker','WebSocket','EventSource','WebTransport','RTCPeerConnection','webkitRTCPeerConnection'])Object.defineProperty(window,name,{value:class{constructor(){throw Error(name+' is unsupported by the local composition profile');}},writable:false,configurable:false});
   Object.defineProperty(window,'open',{value:()=>{throw Error('Popups are unsupported');},writable:false,configurable:false});
   navigator.serviceWorker?.register&&(navigator.serviceWorker.register=()=>Promise.reject(Error('Service workers are unsupported')));
  });
  const cdp=await page.createCDPSession();await cdp.send('Page.setDownloadBehavior',{behavior:'deny'});
  return page;
 };
 return browser;
};
const hf=await import(pathToFileURL(path.join(root,'node_modules/hyperframes/dist/src-PTIV7IVV.js')));
const config=hf.resolveConfig({chromePath:chromium,expectedChromiumMajor:155,concurrency:1,enableBrowserPool:false,browserGpuMode:'software',forceScreenshot:true,useDrawElement:false,enablePageSideCompositing:false,enableStreamingEncode:true,enableChunkedEncode:false,staticFrameDedup:false,browserTimeout:10000,protocolTimeout:15000,playerReadyTimeout:5000,renderReadyTimeout:5000,pageNavigationTimeout:7000,ffmpegEncodeTimeout:70000,ffmpegProcessTimeout:20000,ffmpegStreamingTimeout:70000,extractCacheMaxBytes:33554432,frameDataUriCacheBytesLimitMb:32,frameDataUriCacheLimit:16});
async function command(executable,args,max=256*1024){return new Promise((resolve,reject)=>{
 const child=spawn(executable,args,{stdio:['ignore','pipe','pipe']});let out=Buffer.alloc(0),err=Buffer.alloc(0);const timer=setTimeout(()=>child.kill('SIGKILL'),20000);
 child.stdout.on('data',b=>{out=Buffer.concat([out,b]);if(out.length>max)child.kill('SIGKILL');});child.stderr.on('data',b=>{err=Buffer.concat([err,b]);if(err.length>131072)child.kill('SIGKILL');});
 child.on('error',reject);child.on('close',code=>{clearTimeout(timer);if(code!==0||out.length>max||err.length)reject(Error(`Native verification failed: ${err.toString().slice(-3000)}`));else resolve(out);});
 });}
const containerInput=['-protocol_whitelist','file,pipe','-format_whitelist','mov,matroska,webm,wav,mp3'];
// Complete MP4/WebM packets are demuxed directly. This avoids the pinned FFmpeg's
// Opus AVParser diagnostic (also reproduced on FFmpeg's own sine fixture);
// strict codec decoding still rejects malformed packets and any error diagnostic.
const packetInput=['-fflags','+noparse+nofillin'];
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
let server,session;
async function run(job){
 const dir=path.join(process.cwd(),'input');
 fs.mkdirSync(path.join(dir,'__mirket'),{recursive:true});fs.copyFileSync(path.join(root,'node_modules/gsap/dist/gsap.min.js'),path.join(dir,'__mirket/gsap.min.js'));
 // The capture API uses index.html; the selected entry is copied into this private alias.
 if(job.entry!=='index.html')fs.copyFileSync(path.join(dir,job.entry),path.join(dir,'index.html'));
 const html=fs.readFileSync(path.join(dir,'index.html'),'utf8');
 if(/<(iframe|object|embed)\b|\b(?:http[s]?:)?\/\//i.test(html.replace(/https?:\/\/www\.w3\.org\/2000\/svg/g,'')))throw Error('Remote URLs and embedded browsing contexts are unsupported');
 // Inspect compressed media before upstream extraction can allocate frame surfaces.
 for(const relative of job.media??[]){
  const file=path.join(dir,relative);const probe=JSON.parse((await command(ffprobe,['-v','error',...containerInput,...(relative.endsWith('.webm')?packetInput:[]),'-show_streams','-show_format','-of','json',file])).toString());
  const duration=Number(probe.format.duration);
  if(!Number.isFinite(duration)||duration<=0||duration>30||probe.streams.length>2)throw Error('Source media must be <=30 seconds and at most two streams');
  for(const stream of probe.streams){
   if(stream.codec_type==='video'){
    const [n,d]=String(stream.avg_frame_rate).split('/').map(Number);
    if(!['h264','vp8','vp9'].includes(stream.codec_name)||!['yuv420p','yuvj420p','yuva420p'].includes(stream.pix_fmt)||['smpte2084','arib-std-b67'].includes(stream.color_transfer)||stream.width<1||stream.height<1||stream.width>1920||stream.height>1920||stream.width*stream.height>2073600||!(n/d>0&&n/d<=60))throw Error('Source video exceeds the bounded SDR H.264/VP8/VP9 profile');
   }else if(stream.codec_type==='audio'){
    if(!['pcm_s16le','pcm_s24le','pcm_f32le','mp3','aac','opus','vorbis'].includes(stream.codec_name)||![1,2].includes(stream.channels)||!(Number(stream.sample_rate)>0&&Number(stream.sample_rate)<=48000))throw Error('Source audio exceeds the stereo/48kHz profile');
   }else throw Error('Unsupported source-media stream');
  }
 }
 const fps={num:job.fps,den:1};
 server=await hf.createFileServer({projectDir:dir,fps});
 allowedOrigins.add(new URL(server.url).origin);allowedOrigins.add(`http://127.0.0.1:${server.port}`);
 // The producer creates additional loopback servers during render; those are still private staged files.
 const oldListen=http.Server.prototype.listen;
 http.Server.prototype.listen=function(...args){this.once('listening',()=>{const a=this.address();if(a&&typeof a==='object'){allowedOrigins.add(`http://localhost:${a.port}`);allowedOrigins.add(`http://127.0.0.1:${a.port}`);}});return oldListen.apply(this,args);};
 session=await hf.createCaptureSession(server.url,path.join(process.cwd(),'capture'),{width:job.width,height:job.height,fps,format:'png',deviceScaleFactor:1,compositionDurationSeconds:job.frames/job.fps,requiresWebGpu:false},null,config);
 await hf.initializeSession(session);
 const state=await session.page.evaluate(()=>{const el=document.querySelector('[data-composition-id]');return{duration:window.__hf?.duration,seek:typeof window.__hf?.seek==='function',timelineIds:Object.keys(window.__timelines??{}),composition:el?{id:el.dataset.compositionId,width:Number(el.dataset.width),height:Number(el.dataset.height),duration:Number(el.dataset.duration)}:null,title:document.title,media:Array.from(document.querySelectorAll('video,audio')).map(e=>({id:e.id,kind:e.tagName.toLowerCase(),src:e.getAttribute('src'),hasAudio:e.dataset.hasAudio==='true'}))};});
 assertClean();
 if(!state.seek||!state.composition||state.composition.width!==job.width||state.composition.height!==job.height||Math.abs(state.duration-job.frames/job.fps)>1e-6||Math.abs(state.composition.duration-job.frames/job.fps)>1e-6)throw Error('HyperFrames root dimensions/duration must match the manifest');
 const result={engine:'hyperframes',version:'0.8.149',operation:job.operation,width:job.width,height:job.height,fps:job.fps,frames:job.frames,browser_sandbox:browserSandbox,isolation:'podman',composition:state};
 const capture=async frame=>{const v=await hf.captureFrameToBuffer(session,frame,frame/job.fps);assertClean();if(session.pageErrors.length||session.scriptLoadFailures.length)throw Error('HyperFrames capture diagnostics report a page or script failure');return v.buffer;};
 if(job.operation==='frame'){
  const bytes=await capture(job.frame);fs.writeFileSync(job.output,bytes);
  Object.assign(result,{frame:job.frame,output_bytes:bytes.length,sha256:sha(bytes)});
 }
 if(['verify','render','probe'].includes(job.operation)){
  const frames=[...new Set([0,Math.floor((job.frames-1)/2),job.frames-1])];const hashes=new Map();
  for(const frame of frames)hashes.set(frame,sha(await capture(frame)));
  for(const frame of [...frames].reverse())if(sha(await capture(frame))!==hashes.get(frame))throw Error('Forward/reverse HyperFrames frame pixels differ');
  result.verification={deterministic:true,unique_frames:new Set(hashes.values()).size,samples:frames.map(frame=>({frame,sha256:hashes.get(frame)}))};
 }
 await hf.closeCaptureSession(session);session=null;server.close();server=null;
 if(job.operation==='render'){
  const request=hf.createRenderRequest({projectDir:dir,outputPath:path.resolve(job.output),engineConfig:config,options:{fps,quality:'standard',format:job.format,workers:1,useGpu:false,hdrMode:'force-sdr',entryFile:'index.html',strictness:'strict',debug:false}});
  const render=hf.createRenderJob(hf.renderConfigFromRequest(request,{logger:{info:log,warn:log,error:log,debug:()=>{}}}));
  await hf.executeRenderJob(render,dir,path.resolve(job.output),undefined,abort.signal,assertClean);assertClean();
  if(render.status!=='complete'||render.warnings.length)throw Error(`HyperFrames render incomplete or warned: ${JSON.stringify(render.warnings)}`);
  const metadata=JSON.parse((await command(ffprobe,['-v','error',...containerInput,...(job.format==='webm'?packetInput:[]),'-show_streams','-show_format','-of','json',job.output])).toString());
  const video=metadata.streams.filter(s=>s.codec_type==='video'),audio=metadata.streams.filter(s=>s.codec_type==='audio');
  if(video.length!==1||video[0].width!==job.width||video[0].height!==job.height||video[0].avg_frame_rate!==`${job.fps}/1`||video[0].codec_name!==(job.format==='mp4'?'h264':'vp9'))throw Error('Encoded video metadata differs from manifest');
  if(audio.length>1||audio.some(s=>s.codec_name!==(job.format==='mp4'?'aac':'opus')))throw Error('Unexpected encoded audio profile');
  // The full decode establishes actual count, timing and changing saved pixels.
  // Metadata probing does not need a second decode just to count the frames.
  // The built-in VP9 decoder can discard alpha. Include it in the frame hashes
  // so alpha-only choreography is not mislabeled as a frozen video.
  const decoded=await command(ffmpeg,['-nostdin','-v','error','-xerror','-err_detect','explode',...containerInput,...(job.format==='webm'?[...packetInput,'-c:v','libvpx-vp9']:[]),'-i',job.output,'-map','0:v:0','-an',...(job.format==='webm'?['-pix_fmt','rgba']:[]),'-fps_mode','passthrough','-f','framehash','-hash','sha256','pipe:1']);
  Object.assign(result,verifyDecodedFrames(decoded.toString('utf8'),job,result.verification.unique_frames),{codec:video[0].codec_name,audio_streams:audio.length,audio_codec:audio[0]?.codec_name??null,duration:Number(metadata.format.duration),format:job.format});
  const expectedAudio=state.media.some(m=>m.kind==='audio')||state.media.some(m=>m.kind==='video'&&m.hasAudio);
  if(expectedAudio&&audio.length!==1)throw Error('Authored audio was lost in the producer output');
  if(!Number.isFinite(result.duration)||Math.abs(result.duration-job.frames/job.fps)>1/job.fps+.025)throw Error('Encoded duration differs from manifest');
  if(audio.length)await command(ffmpeg,['-nostdin','-v','error','-xerror','-err_detect','explode',...containerInput,...(job.format==='webm'?packetInput:[]),'-i',job.output,'-map','0:a:0','-f','null','-'],1);
  if(job.format==='webm'){
   const rgba=await command(ffmpeg,['-nostdin','-v','error','-xerror','-err_detect','explode',...containerInput,...packetInput,'-c:v','libvpx-vp9','-i',job.output,'-frames:v','1','-pix_fmt','rgba','-f','rawvideo','-'],job.width*job.height*4);
   if(rgba.length!==job.width*job.height*4)throw Error('WebM alpha decode dimensions differ');
   let transparent=0,opaque=0;for(let i=3;i<rgba.length;i+=4){if(rgba[i]<255)transparent++;if(rgba[i]===255)opaque++;}
   result.alpha={decoded:true,transparent_pixels:transparent,opaque_pixels:opaque};
  }
 }
 assertClean();return result;
}
try{
 let job;
 if(process.argv[2]==='--probe'){
  fs.mkdirSync('input',{recursive:true});fs.writeFileSync('input/index.html','<!doctype html><html><head><style>html,body{margin:0;width:64px;height:64px}#mark{width:32px;height:32px;background:red}</style><script src="/__mirket/gsap.min.js"></script></head><body><div data-composition-id="main" data-width="64" data-height="64" data-duration="0.1"><div id="mark"></div></div><script>window.__timelines={main:gsap.timeline({paused:true}).to("#mark",{x:20,duration:.1,ease:"none"})};</script></body></html>');
  job={entry:'index.html',width:64,height:64,fps:30,frames:3,operation:'probe'};
 }else job=JSON.parse(fs.readFileSync(process.argv[2],'utf8'));
 const result=await run(job);process.stdout.write(JSON.stringify(result)+'\n');
}catch(e){log(e.stack??String(e));process.exitCode=1;}
finally{clearInterval(scratchWatch);if(session)await hf.closeCaptureSession(session).catch(log);server?.close();for(const browser of browsers)await browser.close().catch(()=>{});}
