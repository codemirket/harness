// A private, active Chromium session runs the real permission-checked Penpot plugin.
import fs from 'node:fs/promises';import path from 'node:path';import{pathToFileURL}from'node:url';import{randomBytes}from'node:crypto';
let browser;const MAX_OUTPUT=32*1024*1024;
// Retain 256 random bits while guaranteeing this pinned editor's four classes.
const privatePassword=()=>randomBytes(32).toString('base64url')+'aA0!';
async function clean(){if(browser){await browser.close().catch(()=>{});browser=undefined;}}
for(const s of ['SIGTERM','SIGINT'])process.once(s,()=>{void clean().finally(()=>process.exit(143));});
async function regular(file,max){const s=await fs.lstat(file);if(!s.isFile()||s.isSymbolicLink()||s.size>max)throw Error('Runtime file is missing, linked or oversized.');return fs.readFile(file);}
async function main(){
 const root=process.env.MIRKET_PENPOT_ROOT,configuredUrl=process.env.MIRKET_PENPOT_URL,data=process.env.MIRKET_PENPOT_DATA;
 if(!root||!configuredUrl||!data||!process.env.MIRKET_CHROMIUM)throw Error('Penpot service and pinned browser must be configured; run mirket package start penpot.');
 const origin=new URL(configuredUrl),port=Number(origin.port);if(!['127.0.0.1','localhost'].includes(origin.hostname)||origin.protocol!=='http:'||origin.username||origin.password||origin.pathname!=='/'||origin.search||origin.hash||!Number.isInteger(port)||port<=1024||port>65535)throw Error('Penpot requires Mirket’s loopback HTTP service URL with an explicit port above 1024 and no credentials, path, query or fragment.');
 const url=origin.href;
 const job=JSON.parse((await regular('job.json',256*1024)).toString('utf8'));if(!['probe','create','inspect','update','export'].includes(job.operation))throw Error('Unsupported Penpot operation.');
 const {chromium}=await import(pathToFileURL(path.join(root,'node_modules/playwright-core/index.mjs')).href);
 browser=await chromium.launch({executablePath:process.env.MIRKET_CHROMIUM,headless:true,chromiumSandbox:true});
 const context=await browser.newContext({viewport:{width:1600,height:1200},serviceWorkers:'block',acceptDownloads:false});const page=await context.newPage();
 const rpc=async(name,payload)=>{const endpoint=new URL('api/main/methods/'+name,origin).href;const r=payload===undefined?await context.request.get(endpoint,{headers:{Accept:'application/json'}}):await context.request.post(endpoint,{data:payload,headers:{Accept:'application/json'}});const t=await r.text();if(t.length>8*1024*1024)throw Error('Oversized Penpot RPC result.');let value;try{value=JSON.parse(t);}catch{}if(!r.ok()||!r.headers()['content-type']?.includes('application/json')){const code=typeof value?.code==='string'&&/^[a-z0-9-]{1,80}$/.test(value.code)?value.code:undefined;const error=Error('Penpot '+name+' failed ('+r.status()+(code?'; '+code:'')+').');error.penpotCode=code;error.httpStatus=r.status();throw error;}if(value===undefined)throw Error('Penpot '+name+' returned malformed JSON.');return value;};
 if(job.operation==='probe'){const p=await rpc('get-profile');if(p.id!=='00000000-0000-0000-0000-000000000000')throw Error('Unexpected anonymous readiness identity.');await clean();console.log(JSON.stringify({engine:'penpot',operation:'probe',version:'2.18.3',readiness:'service_rpc',plugin_invocation:false}));return;}
 if(!/^[a-f0-9]{64}$/.test(job.project_scope))throw Error('Expected canonical project scope identity.');
 const accounts=path.join(data,'accounts');await fs.mkdir(accounts,{recursive:true,mode:0o700});const cs=await fs.lstat(accounts);if(!cs.isDirectory()||cs.isSymbolicLink())throw Error('Unsafe Penpot account directory.');
 const credentialsFile=path.join(accounts,job.project_scope+'.json');let credentials;
 try{credentials=JSON.parse((await regular(credentialsFile,4096)).toString('utf8'));}catch(e){if(e.code!=='ENOENT')throw e;credentials={email:job.project_scope.slice(0,32)+'@mirket.invalid',password:privatePassword()};await fs.writeFile(credentialsFile,JSON.stringify(credentials),{mode:0o600,flag:'wx'});}
 let profile;try{profile=await rpc('login-with-password',credentials);}catch(loginError){
  if(loginError.httpStatus!==400||loginError.penpotCode!=='wrong-credentials')throw loginError;
  const prepare=account=>rpc('prepare-register-profile',{...account,fullname:'Mirket local design','accept-newsletter-updates':false});let prepared,candidate=credentials;
  try{prepared=await prepare(candidate);}catch(e){
   if(e.httpStatus!==400||e.penpotCode!=='weak-password')throw e;
   candidate={...credentials,password:privatePassword()};prepared=await prepare(candidate);
  }
  await rpc('register-profile',{token:prepared.token,'accept-newsletter-updates':false});profile=await rpc('login-with-password',candidate);
  if(candidate!==credentials){
   // Only authenticated recovery may replace a pending private credential.
   // Existing active/inactive accounts and unrelated failures leave it intact.
   if(profile.email!==candidate.email)throw Error('Registration returned a different account; private credentials were preserved.');
   const current=JSON.parse((await regular(credentialsFile,4096)).toString('utf8'));if(current.email!==credentials.email||current.password!==credentials.password)throw Error('Private account credentials changed during registration; retry.');
   const pending=credentialsFile+'.pending-'+randomBytes(16).toString('hex');try{await fs.writeFile(pending,JSON.stringify(candidate),{mode:0o600,flag:'wx'});await fs.rename(pending,credentialsFile);}finally{await fs.unlink(pending).catch(e=>{if(e.code!=='ENOENT')throw e;});}
   credentials=candidate;
  }
 }
 if(!profile.defaultTeamId||!profile.defaultProjectId)profile=await rpc('get-profile');
 let file;
 if(job.operation==='create')file=await rpc('create-file',{'project-id':profile.defaultProjectId,name:job.document.name});
 else {
  const handle=JSON.parse((await regular('input.penpot.json',16384)).toString('utf8'));
  if(handle.version!==1||handle.engine!=='penpot'||handle.project_scope!==job.project_scope||handle.project_id!==profile.defaultProjectId||handle.team_id!==profile.defaultTeamId||!/^[a-f0-9-]{36}$/.test(handle.file_id))throw Error('This handle belongs to a different project or service account.');
  file=await rpc('get-file?id='+handle.file_id);if(file.projectId!==profile.defaultProjectId)throw Error('Penpot file is outside the project account.');
  if(job.operation==='update'){
   if(file.revn!==job.expected_revision)throw Error('Penpot file revision changed; inspect again before refinement.');
   file=await rpc('duplicate-file',{'file-id':file.id,name:file.name+' — refinement'});
  }
 }
 const pluginSource=(await regular(path.join(root,'plugin.mjs'),128*1024)).toString('utf8');
 let activeJob=job.operation==='export'&&['penpot','svg'].includes(job.format)?{operation:'inspect'}:job;const pluginName='Mirket Design Tools';const pluginPath='/__mirket/';
 let resultResolve,resultReject;const resultPromise=new Promise((resolve,reject)=>{resultResolve=resolve;resultReject=reject;});
 page.on('console',async message=>{if(!message.text().startsWith('__MIRKET_PENPOT__'))return;try{const v=await message.args()[1].jsonValue();if(v.error)resultReject(Error(v.error));else resultResolve(v);}catch(e){resultReject(e);}});
 const failures=[];page.on('pageerror',e=>failures.push(e.message.slice(0,500)));
 await context.route('**/*',async route=>{const request=route.request(),u=new URL(request.url());if(u.origin!==origin.origin)return route.abort('blockedbyclient');
  if(u.pathname===pluginPath+'manifest.json')return route.fulfill({contentType:'application/json',body:JSON.stringify({name:pluginName,description:'Typed local design operations',code:'plugin.js',version:2,permissions:['content:read','content:write','library:read','library:write']})});
  if(u.pathname===pluginPath+'plugin.js')return route.fulfill({contentType:'application/javascript',body:pluginSource+'\nmirketPenpot('+JSON.stringify(activeJob)+').then(v=>console.log("__MIRKET_PENPOT__",v)).catch(e=>console.log("__MIRKET_PENPOT__",{error:e.message}));'});
  return route.continue();
 });
 await page.goto(new URL('#/workspace?team-id='+profile.defaultTeamId+'&file-id='+file.id,origin).href);
 await page.waitForSelector('[data-testid=viewport]');await page.locator('[data-tool=plugins]').click();
 const existing=page.getByText(pluginName,{exact:true});if(await existing.count()===0){await page.getByPlaceholder('Write a plugin URL').fill(new URL(pluginPath+'manifest.json',origin).href);await page.getByRole('button',{name:'Install',exact:true}).click();const accept=page.getByRole('button',{name:/Allow|Accept|Install/,exact:false}).last();await accept.click();await existing.waitFor();}
 const entry=existing.locator('..');let open=entry.getByRole('button',{name:'Open',exact:true});if(await open.count()===0)open=page.getByRole('button',{name:'Open',exact:true}).last();await open.click();
 const timer=setTimeout(()=>resultReject(Error('Penpot plugin exceeded 30 seconds.')),30000);let result;try{result=await resultPromise;}finally{clearTimeout(timer);}
 if(failures.length)throw Error('Penpot browser error: '+failures.join('; '));
 if(job.operation==='create'||job.operation==='update') {
  // Observe server persistence, then reopen in an independent browser page and inspect.
  const start=Date.now();let saved;
  do{saved=await rpc('get-file?id='+file.id);if(saved.revn>file.revn)break;await new Promise(r=>setTimeout(r,150));}while(Date.now()-start<15000);
  if(saved.revn<=file.revn)throw Error('Penpot did not persist the requested change; no handle was published.');
  const originalNodes=result.inspection.nodes;
  const reopened=await context.newPage();await reopened.goto(new URL('#/workspace?team-id='+profile.defaultTeamId+'&file-id='+file.id,origin).href);await reopened.waitForSelector('[data-testid=viewport]');await reopened.waitForFunction(()=>typeof globalThis.ɵloadPlugin==='function');
  activeJob={operation:'inspect'};
  const check=new Promise((resolve,reject)=>{reopened.on('console',async m=>{if(m.text().startsWith('__MIRKET_PENPOT__'))try{const v=await m.args()[1].jsonValue();if(v.error)reject(Error(v.error));else resolve(v);}catch(e){reject(e)}});});
  await reopened.locator('[data-tool=plugins]').click();await reopened.getByRole('button',{name:'Open',exact:true}).last().click();
  let timeout;let rr;try{rr=await Promise.race([check,new Promise((_,reject)=>{timeout=setTimeout(()=>reject(Error('Saved Penpot readback timed out.')),10000)})]);}finally{clearTimeout(timeout);}
  const signature=nodes=>{const parents=new Map(nodes.map(n=>[n.id,n.key]));return JSON.stringify(nodes.map(n=>({...n,id:undefined,parent:undefined,parent_key:parents.get(n.parent)||null})).sort((a,b)=>String(a.key).localeCompare(String(b.key))));};
  if(signature(originalNodes)!==signature(rr.inspection.nodes))throw Error('Persisted Penpot geometry differs from the requested result.');
  result.inspection=rr.inspection;result.inspection.revision=saved.revn;
  const handle={version:1,engine:'penpot',project_scope:job.project_scope,team_id:profile.defaultTeamId,project_id:profile.defaultProjectId,file_id:file.id,revision:saved.revn};await fs.writeFile('output.penpot.json',JSON.stringify(handle,null,2),{flag:'wx'});
 }
 if(job.operation==='export'&&job.format==='penpot') {
  const response=await context.request.post(new URL('api/main/methods/export-binfile',origin).href,{data:{'file-id':file.id,'include-libraries':false,'embed-assets':true},headers:{Accept:'text/event-stream'}});
  if(!response.ok()||!response.headers()['content-type']?.includes('text/event-stream'))throw Error('Native Penpot export endpoint failed.');
  const stream=await response.text();if(stream.length>1024*1024)throw Error('Oversized native export status.');
  const blocks=stream.split(/\r?\n\r?\n/);let asset;
  for(const block of blocks){const lines=block.split(/\r?\n/),event=lines.find(s=>s.startsWith('event:'))?.slice(6).trim(),body=lines.filter(s=>s.startsWith('data:')).map(s=>s.slice(5).trim()).join('\n');if(event==='error')throw Error('Native Penpot export failed; no archive was published.');if(event==='end'){const value=JSON.parse(body);asset=typeof value==='string'?value:value['~#uri'];}}
  if(!asset)throw Error('Native Penpot export returned no usable asset.');
  const location=new URL(asset);if(location.origin!==origin.origin||!/^\/assets\/by-id\/[a-f0-9-]{36}$/.test(location.pathname)||location.search||location.hash)throw Error('Native export returned an unexpected artifact URL.');
  const cookies=await context.cookies(url);const cookie=cookies.map(c=>c.name+'='+c.value).join('; ');
  const exported=await fetch(location,{headers:{Cookie:cookie},redirect:'error',signal:AbortSignal.timeout(15000)});
  if(!exported.ok||!exported.body)throw Error('Native Penpot archive is unavailable.');
  const chunks=[];let size=0;for await(const chunk of exported.body){size+=chunk.length;if(size>MAX_OUTPUT)throw Error('Native Penpot archive exceeds 32 MiB.');chunks.push(chunk);}
  result.bytes=Buffer.concat(chunks);
  if(result.bytes.length<4||result.bytes[0]!==0x50||result.bytes[1]!==0x4b)throw Error('Native Penpot export is not a ZIP archive.');
 }
 if(job.operation==='export'&&job.format==='svg') {
  const object=result.inspection.nodes.find(n=>n.key===job.object_key);if(!object)throw Error('SVG export requires an inspected object key.');
  const scale=job.scale||1;if(object.width*scale>2048||object.height*scale>2048)throw Error('SVG export exceeds 2048 pixels per side.');
  const render=await context.newPage();const rendererErrors=[];render.on('pageerror',e=>rendererErrors.push(e.message.slice(0,500)));
  const params=new URLSearchParams({'file-id':file.id,'page-id':result.inspection.page_id,'object-id':object.id,route:'objects','render-embed':'true'});
  await render.goto(new URL('render.html?'+params,origin).href);await render.waitForSelector('#screenshot-'+object.id);
  await render.evaluate(async()=>{for(const node of document.querySelectorAll('text')){const css=getComputedStyle(node);const faces=await document.fonts.load(css.fontWeight+' '+css.fontSize+' '+css.fontFamily,node.textContent);if(!faces.length)throw Error('Native SVG renderer did not load the requested bundled font.');}});
  if(rendererErrors.length)throw Error('Native SVG renderer failed: '+rendererErrors.join('; '));
  let svg=await render.locator('#screenshot-'+object.id).evaluate((element,scale)=>{element.setAttribute('width',String(Number(element.getAttribute('width'))*scale));element.setAttribute('height',String(Number(element.getAttribute('height'))*scale));return element.outerHTML;},scale);if(svg.length>8*1024*1024)throw Error('Native SVG renderer output exceeds 8 MiB.');
  let embedded=0;result.inspection.svg_renderer='native Penpot render.html DOM';
  const urls=[...new Set([...svg.matchAll(/url\((https?:[^)]*)\)/g)].map(m=>m[1]))];
  for(const resource of urls){const font=new URL(resource);if(font.origin!==origin.origin||!/^\/fonts\/sourcesanspro-(regular|bold|semibold)\.woff$/.test(font.pathname)||font.search||font.hash)throw Error('SVG export contains an unsupported linked resource.');const loaded=await fetch(font,{redirect:'error',signal:AbortSignal.timeout(5000)});if(!loaded.ok||!loaded.body)throw Error('Required bundled SVG font is unavailable.');let size=0;const chunks=[];for await(const chunk of loaded.body){size+=chunk.length;if(size>2*1024*1024)throw Error('SVG font exceeds 2 MiB.');chunks.push(chunk);}const bytes=Buffer.concat(chunks);if(bytes.toString('ascii',0,4)!==(font.pathname.endsWith('.woff2')?'wOF2':'wOFF'))throw Error('Font response is not WOFF.');svg=svg.replaceAll(resource,'data:font/'+(font.pathname.endsWith('.woff2')?'woff2':'woff')+';base64,'+bytes.toString('base64'));embedded++;}
  if(Buffer.byteLength(svg)>8*1024*1024)throw Error('Self-contained SVG exceeds 8 MiB.');result.bytes=Buffer.from(svg);result.inspection.embedded_fonts=embedded;
 }
 if(result.bytes){const bytes=Buffer.from(result.bytes);if(!bytes.length||bytes.length>MAX_OUTPUT)throw Error('Empty or oversized exported artifact.');await fs.writeFile('output.'+job.format,bytes,{flag:'wx'});delete result.bytes;}
 await clean();console.log(JSON.stringify({engine:'penpot',operation:job.operation,...result,notes:['Uses a real active headless Chromium workspace and registered Plugin API. Editable design resides in the local Penpot service; the project JSON is its scoped handle.']}));
}
main().catch(async e=>{await clean();process.stderr.write(String(e.message).slice(0,4000)+'\n');process.exitCode=1;});
