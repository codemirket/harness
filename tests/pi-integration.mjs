// Offline assembled-package integration. Explicit admitted peers and binaries only.
import assert from 'node:assert/strict';
import {makeFixtureProvider} from './pi-offline-provider.mjs';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {join, isAbsolute, resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
const [peers, binary, node, license, nativeSource] = process.argv.slice(2);
assert.ok([peers,binary,node,license].every(x=>x&&isAbsolute(x)), 'Supply absolute peers, Rust binary, private Node and Node LICENSE paths');
const root = await fs.mkdtemp('/private/tmp/mirket-pi-package-');
console.log('Fixture:',root);
await fs.chmod(root,0o700);
const agentDir=join(root,'agent'), cwd=join(root,'project'), other=join(root,'other'), userHome=join(root,'home');
for (const path of [agentDir,cwd,other,userHome,join(root,'sessions')]) await fs.mkdir(path,{mode:0o700});
process.env.HOME=userHome; process.env.PI_CODING_AGENT_DIR=agentDir;
process.env.PI_CODING_AGENT_SESSION_DIR=join(root,'sessions');
process.env.PI_SKIP_UPDATE_CHECK='1'; process.env.PI_TELEMETRY_DISABLED='1';
const report={platform:process.platform,architecture:process.arch,node:process.version,root,results:[]};
const save=async name=>{report.results.push(name);await fs.writeFile(join(root,'report.json'),JSON.stringify(report,null,2));console.log('PASS',name);};
const native=join(root,'native'),packed=join(root,'package');
if(nativeSource){
 assert.ok(isAbsolute(nativeSource));
 await fs.cp(nativeSource,native,{recursive:true,dereference:false,verbatimSymlinks:true});
 await fs.unlink(join(native,'manifest.json')); // New tuple for copied first-party changes.
 // Reassemble only first-party code/resources; admitted third-party bytes stay exact.
 for(const file of await fs.readdir(resolve('pi/dist/runtime'))){
  if(!file.endsWith('.js'))continue;
  const target=join(native,file.startsWith('video-')?'runtime/video/dist':'adapters',file);
  await fs.copyFile(resolve('pi/dist/runtime',file),target);
 }
 await fs.rm(join(native,'skills'),{recursive:true});await fs.cp(resolve('skills'),join(native,'skills'),{recursive:true});
}
await fs.mkdir(join(native,'bin'),{recursive:true,mode:0o700});
await fs.mkdir(join(native,'runtime/node/bin'),{recursive:true,mode:0o700});
for (const [source,destination,mode] of [[binary,'bin/mirket',0o700],[node,'runtime/node/bin/node',0o700],[license,'runtime/node/LICENSE',0o600]]) {
 await fs.copyFile(source,join(native,destination));await fs.chmod(join(native,destination),mode);
}
const cli=(...args)=>JSON.parse(execFileSync(binary,args,{encoding:'utf8',timeout:60000}));
const release=cli('dev','manifest','--directory',native).release_id;
execFileSync('/usr/bin/python3',[resolve('scripts/assemble-pi-package.py'),'--mirket',binary,'--source',native,'--output',packed],{stdio:'pipe',timeout:60000});
const setup=()=>JSON.parse(execFileSync(node,[join(packed,'scripts/setup.mjs'),'--agent-dir',agentDir],{encoding:'utf8',timeout:660000,stdio:['ignore','pipe','pipe']}));
const extensionPath=join(packed,'dist/extension/index.js'),extensionBytes=await fs.readFile(extensionPath);
await fs.appendFile(extensionPath,'\n// altered after assembly\n');assert.throws(setup,/differs from inventory/);
assert.equal(await fs.stat(join(agentDir,'mirket')).then(()=>true,()=>false),false);await fs.writeFile(extensionPath,extensionBytes);
await fs.writeFile(join(agentDir,'settings.json'),JSON.stringify({theme:'preserved',packages:[]}));
const installed=setup();assert.equal(installed.releaseId,release);assert.throws(setup,/existing home is preserved/);
const initialSettings=JSON.parse(await fs.readFile(join(agentDir,'settings.json')));
assert.equal(initialSettings.theme,'preserved');assert.deepEqual(initialSettings.packages,[{source:packed,skills:installed.selectedSkills}]);
await save('fresh assembled local package setup; repeated setup preserves existing home');
await save('altered extension refuses before native setup; setup applies core filter and preserves unrelated settings');
const sdk=await import(pathToFileURL(join(peers,'@earendil-works/pi-coding-agent/dist/index.js')).href);
const ai=await import(pathToFileURL(join(peers,'@earendil-works/pi-ai/dist/index.js')).href);
const fixtureProvider=makeFixtureProvider(ai);
const settings={enableInstallTelemetry:false,packages:[packed],defaultTools:['+codemode','+tool_search'],defaultProvider:'offline-fixture',defaultModel:'deterministic'};
await fs.writeFile(join(agentDir,'settings.json'),JSON.stringify(settings));
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
const processes=()=>execFileSync('/bin/ps',['-axo','pid=,ppid=,command='],{encoding:'utf8'}).split('\n').filter(line=>line.includes(join(agentDir,'mirket/bin/mirket')+' serve'));
let active=[];
async function open({project=cwd,disabled=false,excludeTools}={}){
 const before=processes().length;
 const settingsManager=sdk.SettingsManager.create(project,agentDir);
 const loader=new sdk.DefaultResourceLoader({cwd:project,agentDir,settingsManager,noContextFiles:true,extensionFactories:[{name:'codemode',factory:sdk.createCodemodeExtension(),builtin:true,replaceable:true},{name:'tool-search',factory:sdk.createToolSearchExtension(),builtin:true,replaceable:true},...(disabled?[]:[{name:'mcp',factory:sdk.createMcpExtension(),builtin:true,replaceable:true}]),fixtureProvider]});
 await loader.reload();assert.deepEqual(loader.getExtensions().errors,[]);assert.equal(processes().length,before,'factory spawned service');
 const {session}=await sdk.createAgentSession({cwd:project,agentDir,settingsManager,resourceLoader:loader,excludeTools,sessionManager:sdk.SessionManager.create(project,join(root,'sessions'))});
 const notifications=[],errors=[];
 await session.bindExtensions({mode:'print',uiContext:{notify:(text,type)=>notifications.push({text,type})},onError:e=>errors.push(e)});
 await session.setModel(session.modelRuntime.getModel('offline-fixture','deterministic'));
 const entry={session,loader,notifications,errors};active.push(entry);return entry;
}
async function close(e){await e.session.extensionRunner.emit({type:'session_shutdown',reason:'quit'});e.session.dispose();active=active.filter(x=>x!==e);await sleep(150);}
async function call(e,tool,args={}){const from=e.session.messages.length;await e.session.prompt(JSON.stringify({tool,args}));const result=e.session.messages.slice(from).findLast(m=>m.role==='toolResult'&&m.toolName===tool);assert.ok(result,JSON.stringify(e.errors));assert.equal(result.isError,false,JSON.stringify(result));const text=result.content.filter(c=>c.type==='text').map(c=>c.text).join('\n');try{return JSON.parse(text);}catch{return text;}}
try {
 const e=await open();
 await e.session.prompt('/mcp');
 const status=await call(e,'mcp__mirket__runtime_status');assert.equal(status.release_id,release);
 const doctor=await call(e,'mirket_doctor');assert.equal(doctor.status,'ready',JSON.stringify({doctor,errors:e.errors,notifications:e.notifications,tools:e.session.getAllTools().filter(t=>t.name==='mcp__mirket__runtime_status')}));
 await save('native built-in MCP registration, actual status and nested doctor identity');
 if(nativeSource){const {domainJourneys}=await import('./pi-domain-journeys.mjs');await domainJourneys({session:e,call,save,cwd,release,root});}
 await call(e,'tool_search',{query:'memory_prepare memory_commit memory_read',limit:8});
 const common={expected_release_id:release,scope:{kind:'project',project_root:cwd}};
 const record={kind:'fact',basis:'observed',content:'Assembled pi package saved this fact',sources:[{source_id:'fixture:pi-package',revision:'v1',locator:'fixture://offline',captured_at:123}],status:'current',critical:false,applicability:{task_key:null,checkout_id:null,operation:null},valid_from:null,valid_until:null};
 const prepared=await call(e,'mcp__mirket__memory_prepare',{...common,mutation:{kind:'create',record,supersedes:null}});
 const committed=await call(e,'mcp__mirket__memory_commit',{...common,operation_id:prepared.operation_id});assert.equal(committed.status,'committed');
 const read={...common,record_id:committed.record.record_id};assert.equal((await call(e,'mcp__mirket__memory_read',read)).record.data.content,record.content);
 await save('deferred discovery and installed memory prepare/commit/read');
 const before=e.session.messages.length;
 await e.session.prompt(JSON.stringify({tool:'mcp__mirket__memory_read',args:{...read,scope:{kind:'project',project_root:other}}}));
 const wrong=e.session.messages.slice(before).findLast(m=>m.role==='toolResult');assert.equal(wrong.isError,true);assert.match(JSON.stringify(wrong),/Operation project differs/);
 const cm=await call(e,'codemode',{code:`try{await tools.mcp__mirket__memory_read(${JSON.stringify({...read,scope:{kind:'project',project_root:other}})});text({blocked:false});}catch(error){text({blocked:true,error:String(error)});}`});
 assert.match(JSON.stringify(cm),/blocked.*true/);assert.match(JSON.stringify(cm),/Operation project differs/);
 await save('wrong-project direct and codemode calls are refused by session scope');

 await call(e,'tool_search',{query:'memory_forget_prepare memory_forget_commit',limit:8});
 const disposable=await call(e,'mcp__mirket__memory_prepare',{...common,mutation:{kind:'create',record:{...record,content:'Explicitly disposable authorization test record'},supersedes:null}});
 const saved=await call(e,'mcp__mirket__memory_commit',{...common,operation_id:disposable.operation_id});
 const forget=await call(e,'mcp__mirket__memory_forget_prepare',{...common,record_id:saved.record.record_id,expected_revision:saved.record.revision});
 const forgetArgs={...common,operation_id:forget.operation_id};
 const deniedAt=e.session.messages.length;await e.session.prompt(JSON.stringify({tool:'mcp__mirket__memory_forget_commit',args:forgetArgs}));
 const denied=e.session.messages.slice(deniedAt).findLast(m=>m.role==='toolResult');assert.equal(denied.isError,true);assert.match(JSON.stringify(denied),/Explicit authorization required/);
 assert.equal((await call(e,'mcp__mirket__memory_read',{...common,record_id:saved.record.record_id})).record.data.content,'Explicitly disposable authorization test record');
 const nestedDenied=await call(e,'codemode',{code:`try{await tools.mcp__mirket__memory_forget_commit(${JSON.stringify(forgetArgs)});text({blocked:false});}catch(error){text({blocked:true,error:String(error)});}`});assert.match(JSON.stringify(nestedDenied),/Explicit authorization required/);
 await e.session.prompt('/mirket-authorize '+JSON.stringify({tool:'mcp__mirket__memory_forget_commit',args:forgetArgs}));
 const forgotten=await call(e,'mcp__mirket__memory_forget_commit',forgetArgs);assert.equal(forgotten.status,'forgotten');
 await save('headless destructive forgetting denied direct and nested; exact explicit command authorizes only its intent');

 await e.session.reload();await e.session.setModel(e.session.modelRuntime.getModel('offline-fixture','deterministic'));
 await call(e,'tool_search',{query:'memory_read',limit:8});assert.equal((await call(e,'mcp__mirket__memory_read',read)).record.data.content,record.content);
 await save('reload rebind and persistent canonical memory');
 const second=await open();assert.equal((await call(second,'mirket_doctor')).status,'ready');await close(second);await close(e);assert.equal(processes().length,0);
 const filtered=await open({excludeTools:['mcp__mirket__runtime_status']});assert.equal((await call(filtered,'mirket_doctor')).status,'not_ready');await close(filtered);
 const disabled=await open({disabled:true});assert.ok((await call(disabled,'mirket_doctor')).issues.includes('MCP disabled or missing'));await close(disabled);
 await save('two clients, filtered status, disabled MCP and orderly child cleanup');
 await fs.writeFile(join(agentDir,'mcp.json'),JSON.stringify({mcpServers:{mirket:{enabled:false,command:binary}}}));
 const overridden=await open();assert.match(JSON.stringify(await call(overridden,'mirket_doctor')),/Same-name MCP/);assert.equal(processes().length,0);await close(overridden);await fs.unlink(join(agentDir,'mcp.json'));
 const bindingPath=join(packed,'native-binding.json'),original=await fs.readFile(bindingPath);
 const binding=JSON.parse(original);binding.coreSha256='0'.repeat(64);await fs.writeFile(bindingPath,JSON.stringify(binding));
 const stale=await open();assert.match(JSON.stringify(await call(stale,'mirket_doctor')),/identity mismatch/);assert.equal(processes().length,0);await close(stale);await fs.writeFile(bindingPath,original);
 await save('same-name override and stale native identity refuse readiness and spawning');
 const currentCatalog=JSON.parse(await fs.readFile(join(packed,'skills/catalog.json'))),skillCount=currentCatalog.global.length+currentCatalog.project.length;
 const resources=await open();assert.equal(resources.loader.getSkills().skills.length,skillCount,'Unfiltered package exposes its current manifest');
 await resources.session.prompt('/mirket-profile project library-cli +js-ts-navigation');
 await resources.session.reload();await resources.session.setModel(resources.session.modelRuntime.getModel('offline-fixture','deterministic'));
 const selected=resources.loader.getSkills().skills.map(s=>s.name).sort();
 const catalog=JSON.parse(await fs.readFile(join(packed,'skills/catalog.json')));
 assert.deepEqual(selected,[...new Set([...catalog.global.map(s=>s.id),...catalog.profiles['library-cli'].skills,'js-ts-navigation'])].sort());
 assert.equal((await call(resources,'mirket_doctor')).status,'ready');await close(resources);
 const otherSession=await open({project:other});assert.equal(otherSession.loader.getSkills().skills.length,skillCount);await close(otherSession);
 await save('all manifest skills load unfiltered; project profile replaces global package only in selected project');

 const profileModule=await import(pathToFileURL(join(packed,'dist/extension/profiles.js')));
 const projectSettings=join(cwd,'.pi/settings.json');
 for (const names of [['web-product','backend-service'],['design-studio'],['library-cli','documents'],['data-ai','web-product']]) {
  const extra=names.includes('design-studio')?['video-production']:[];
  profileModule.applyProfile(profileModule.prepareProfile(packed,projectSettings,names,extra));
  const manager=sdk.SettingsManager.create(cwd,agentDir),loader=new sdk.DefaultResourceLoader({cwd,agentDir,settingsManager:manager,noContextFiles:true});await loader.reload();
  const expected=[...new Set([...catalog.global.map(s=>s.id),...names.flatMap(n=>catalog.profiles[n].skills),...extra])].sort();
  assert.deepEqual(loader.getSkills().skills.map(s=>s.name).sort(),expected);assert.equal(processes().length,0);
 }
 const globalFile=join(agentDir,'settings.json');
 await fs.writeFile(globalFile,JSON.stringify({...settings,packages:[{source:packed,skills:catalog.global.map(s=>`skills/${s.id}/SKILL.md`)}]}));
 await fs.writeFile(projectSettings,JSON.stringify({packages:[{source:packed,autoload:false,skills:['+skills/browser-research/SKILL.md']}]}));
 const delta=await open();assert.deepEqual(delta.loader.getSkills().skills.map(s=>s.name).sort(),[...catalog.global.map(s=>s.id),'browser-research'].sort());assert.equal((await call(delta,'mirket_doctor')).status,'ready');await close(delta);
 await fs.writeFile(projectSettings,'{}');await fs.writeFile(globalFile,JSON.stringify({...settings,packages:[]}));
 const removed=await open();assert.ok(!removed.session.getAllTools().some(t=>t.name==='mirket_doctor'||t.name.startsWith('mcp__mirket__')));assert.equal(processes().length,0);await close(removed);
 assert.ok((await fs.stat(join(agentDir,'mirket'))).isDirectory(),'Package removal must preserve canonical state');
 await fs.writeFile(globalFile,JSON.stringify(settings));
 await save('four mixed-project loadouts, autoload:false delta and package removal preserve state without workers');

 report.releaseId=release;report.binarySha256=createHash('sha256').update(await fs.readFile(binary)).digest('hex');
 await save('integration complete');
} finally {for(const e of [...active])await close(e);console.log('Evidence:',root);}
