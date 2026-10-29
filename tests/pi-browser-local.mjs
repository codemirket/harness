// Disposable, local-only qualification. This is not production browser admission.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import http from 'node:http';
import {join,isAbsolute} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {makeFixtureProvider} from './pi-offline-provider.mjs';
const [peers,root,browserRoot]=process.argv.slice(2);
assert.ok(isAbsolute(peers)&&root.startsWith('/private/tmp/mirket-pi-browser-')&&isAbsolute(browserRoot));
const sha=b=>createHash('sha256').update(b).digest('hex');
const inventory=JSON.parse(await fs.readFile(new URL('../packaging/admission/m3-browser-cft-macos-arm64.json',import.meta.url)));
for(const f of inventory.files)assert.equal(sha(await fs.readFile(join(browserRoot,f.path))),f.sha256);
for(const l of inventory.links)assert.equal(await fs.readlink(join(browserRoot,l.relative_path)),l.target);
const install=JSON.parse(await fs.readFile(join(root,'installation-evidence.json')));
for(const pkg of install.packages)for(const f of pkg.files)assert.equal(sha(await fs.readFile(join(root,f.path))),f.sha256);
const run=await fs.mkdtemp(join(root,'run-'));const cwd=join(run,'project'),agentDir=join(run,'agent'),output=join(run,'output');
for(const p of [cwd,agentDir,output,join(run,'home')])await fs.mkdir(p,{mode:0o700});
Object.assign(process.env,{HOME:join(run,'home'),PI_CODING_AGENT_DIR:agentDir,PI_CODING_AGENT_SESSION_DIR:join(run,'sessions'),PI_SKIP_UPDATE_CHECK:'1',PI_TELEMETRY_DISABLED:'1',PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD:'1'});
const report={run,platform:process.platform,arch:process.arch,node:process.version,scope:'local fixtures only; test-only admission',checks:[],requests:[],errors:[]};
const html=`<!doctype html><html lang="en"><meta charset="utf-8"><title>Local Mirket fixture</title><style>*{box-sizing:border-box}body{font:20px system-ui;margin:0;padding:24px;background:#f3f5fa;color:#18223b}main{max-width:680px;margin:auto}button{font:inherit;padding:12px 24px;background:#193e9b;color:white;border:0;border-radius:8px}button:focus-visible{outline:4px solid #ef8c00;outline-offset:4px}.cards{display:grid;grid-template-columns:1fr 1fr;gap:16px}.card{padding:20px;background:white;border-radius:12px}@media(max-width:480px){.cards{grid-template-columns:1fr}}.motion{animation:pulse 1s infinite}@keyframes pulse{50%{opacity:.5}}@media(prefers-reduced-motion:reduce){.motion{animation:none}}</style><main><h1>Mirket local UI</h1><p>Keyboard, narrow layout and Turkish glyphs: İğüş.</p><div class="cards"><section class="card"><h2>Preferences</h2><button id="toggle" aria-pressed="false" onclick="this.setAttribute('aria-pressed',this.getAttribute('aria-pressed')==='false');document.querySelector('#status').textContent=this.getAttribute('aria-pressed')==='true'?'Enabled':'Disabled'">Enable reminders</button><p id="status" role="status">Disabled</p></section><section class="card"><h2>Preview</h2><p class="motion">Motion preference: <span id="motion"></span></p><p id="viewport"></p></section></div></main><script>document.querySelector('#motion').textContent=matchMedia('(prefers-reduced-motion: reduce)').matches?'reduced':'normal';function size(){document.querySelector('#viewport').textContent=innerWidth<=480?'Narrow layout':'Wide layout'}size();addEventListener('resize',size)</script></html>`;
const origin='http://fixture.invalid';
// Proxy serves only this synthetic origin itself. It never opens outbound sockets.
const proxy=http.createServer((req,res)=>{report.requests.push({method:req.method,url:req.url});let url;try{url=new URL(req.url)}catch{res.writeHead(403).end();return}if(url.origin!==origin||req.method!=='GET'){res.writeHead(403).end('local-only');return}if(url.pathname==='/redirect'){res.writeHead(302,{Location:'https://outside.invalid/blocked'}).end();return}res.writeHead(200,{'Content-Type':'text/html','Content-Security-Policy':"default-src 'self' 'unsafe-inline'; connect-src 'none'; img-src 'none'"}).end(html)});
proxy.on('connect',(req,socket)=>{report.requests.push({method:'CONNECT',url:req.url,blocked:true});socket.end('HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n')});
await new Promise(resolve=>proxy.listen(0,'127.0.0.1',resolve));const proxyUrl=`http://127.0.0.1:${proxy.address().port}`;
const configFile=join(run,'browser.json');
await fs.writeFile(configFile,JSON.stringify({browser:{browserName:'chromium',isolated:true,launchOptions:{executablePath:join(browserRoot,inventory.executable),headless:true,proxy:{server:proxyUrl,bypass:'<-loopback>'},args:['--disable-quic','--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE 127.0.0.1']},contextOptions:{viewport:{width:1000,height:700},reducedMotion:'reduce',serviceWorkers:'block'}},network:{allowedOrigins:[origin]},outputDir:output,timeouts:{action:5000,navigation:10000}},null,2));
const observed=new Map();
function processRows(){return execFileSync('/bin/ps',['-axo','pid=,ppid=,lstart=,command='],{encoding:'utf8'}).split('\n').flatMap(line=>{const m=line.match(/^\s*(\d+)\s+(\d+)\s+(.{24})\s+(.*)$/);return m?[{pid:Number(m[1]),parent:Number(m[2]),start:m[3],command:m[4]}]:[]})}
function observe(){const rows=processRows();const owned=new Set([process.pid]);let changed=true;while(changed){changed=false;for(const row of rows)if(owned.has(row.parent)&&!owned.has(row.pid)){owned.add(row.pid);changed=true;if(!row.command.startsWith('/bin/ps'))observed.set(row.pid,row)}}}
const observer=setInterval(observe,150);observer.unref();
const sdk=await import(pathToFileURL(join(peers,'@earendil-works/pi-coding-agent/dist/index.js')));const ai=await import(pathToFileURL(join(peers,'@earendil-works/pi-ai/dist/index.js')));
const allowed=['browser_navigate','browser_snapshot','browser_take_screenshot','browser_resize','browser_press_key','browser_close'];
const prefix='mcp__mirket_browser_test__';
const settingsManager=sdk.SettingsManager.create(cwd,agentDir);settingsManager.setDefaultProvider('offline-fixture');settingsManager.setDefaultModel('deterministic');
const fixture=pi=>{pi.on('session_start',()=>pi.registerMcpServer('mirket_browser_test',{command:process.execPath,args:[join(root,'node_modules/@playwright/mcp/cli.js'),'--config',configFile,'--no-webmcp','--block-service-workers'],cwd,env:{HOME:join(run,'home'),PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD:'1',NO_PROXY:'',HTTP_PROXY:proxyUrl,HTTPS_PROXY:proxyUrl},exposure:'hidden',toolExposure:Object.fromEntries(allowed.map(n=>[n,'direct'])),timeout:20}));pi.on('tool_call',e=>{if(e.toolName===prefix+'browser_navigate'&&new URL(e.input.url).origin!==origin)return {block:true,reason:'Local qualification origin only'};});};
const loader=new sdk.DefaultResourceLoader({cwd,agentDir,settingsManager,noContextFiles:true,noSkills:true,extensionFactories:[{name:'mcp',factory:sdk.createMcpExtension(),builtin:true,replaceable:true},makeFixtureProvider(ai),fixture]});await loader.reload();assert.deepEqual(loader.getExtensions().errors,[]);
const {session}=await sdk.createAgentSession({cwd,agentDir,settingsManager,resourceLoader:loader,sessionManager:sdk.SessionManager.create(cwd,join(run,'sessions'))});
const save=async name=>{report.checks.push(name);await fs.writeFile(join(run,'report.json'),JSON.stringify(report,null,2));console.log('PASS',name)};
async function call(name,args={},error=false){let at=session.messages.length;await session.prompt(JSON.stringify({tool:prefix+name,args}));const result=session.messages.slice(at).findLast(m=>m.role==='toolResult');assert.ok(result);await fs.writeFile(join(run,`${report.checks.length}-${name}.json`),JSON.stringify(result,null,2));assert.equal(!!result.isError,error,JSON.stringify(result));let text=result.content.filter(x=>x.type==='text').map(x=>x.text).join('\n');for(const match of text.matchAll(/\[Snapshot\]\(\.\.\/output\/([^/)]+\.yml)\)/g)){text+='\n'+await fs.readFile(join(output,match[1]),'utf8')}return text}
try{await session.bindExtensions({mode:'print',onError:e=>report.errors.push(e)});await session.setModel(session.modelRuntime.getModel('offline-fixture','deterministic'));
 const text=await call('browser_navigate',{url:origin});assert.match(text,/Mirket local UI/);assert.match(text,/reduced/);await save('actual pi MCP navigation, accessibility snapshot and reduced-motion fixture');
 await call('browser_press_key',{key:'Tab'});const key=await call('browser_press_key',{key:'Enter'});assert.match(key,/Enabled/);await save('keyboard focus and activation change actual accessible state');
 await call('browser_take_screenshot',{type:'png',scale:'css',filename:'wide.png',fullPage:true});await save('saved wide screenshot');
 await call('browser_resize',{width:360,height:740});const narrow=await call('browser_snapshot');assert.match(narrow,/Narrow layout/);await call('browser_take_screenshot',{type:'png',scale:'css',filename:'narrow.png',fullPage:true});await save('responsive narrow viewport and saved screenshot');
 await call('browser_navigate',{url:'https://outside.invalid/'},true);await save('out-of-scope navigation denied before execution');
 for(const name of ['browser_install','browser_evaluate','browser_run_code','browser_file_upload'])await call(name,{},true);await save('installation, arbitrary execution and upload tools unreachable');
 // Allowed-origin redirects must still be contained by the local proxy.
 await call('browser_navigate',{url:origin+'/redirect'},true);await save('redirect to nonlocal origin cannot escape deny-all proxy');
 await call('browser_close');await save('browser close completed');
}finally{await session.extensionRunner.emit({type:'session_shutdown',reason:'quit'});session.dispose();clearInterval(observer);proxy.closeAllConnections();await new Promise(resolve=>proxy.close(resolve));await fs.writeFile(join(run,'report.json'),JSON.stringify(report,null,2));await fs.writeFile(join(root,'latest-run'),run);}
let survivors=[];const deadline=Date.now()+3000;do{survivors=processRows().filter(row=>observed.get(row.pid)?.start===row.start);if(!survivors.length)break;await new Promise(resolve=>setTimeout(resolve,50))}while(Date.now()<deadline);
report.observedProcesses=[...observed.values()];report.survivors=survivors;
assert.deepEqual(survivors,[],'observed owned descendant survived shutdown');await save('zero observed owned descendants after pi shutdown');
console.log(run);
