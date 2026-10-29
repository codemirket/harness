// Offline installed Pi authorization lifecycle fixture. No provider network calls.
// Run through pi-session-authorization-runner.py for bounded group cleanup.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {join, isAbsolute, resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {makeFixtureProvider} from './pi-offline-provider.mjs';

const [peers,binary,node,license]=process.argv.slice(2);
assert.ok([peers,binary,node,license].every(path=>path && isAbsolute(path)), 'Supply absolute admitted peer, release binary, private Node and Node LICENSE paths');
assert.equal(process.execPath,node,'Run under the selected private Node');
assert.equal(process.env.MIRKET_SESSION_SUPERVISED,'1','Use pi-session-authorization-runner.py');
const root=await fs.mkdtemp('/private/tmp/mirket-pi-package-');
await fs.chmod(root,0o700);
console.log('Fixture:',root);
const started=Date.now();
const agentDir=join(root,'agent'), project=join(root,'project'), home=join(root,'home'), sessions=join(root,'sessions');
for(const dir of [agentDir,project,home,sessions]) await fs.mkdir(dir,{mode:0o700});
Object.assign(process.env,{HOME:home,PI_CODING_AGENT_DIR:agentDir,PI_CODING_AGENT_SESSION_DIR:sessions,PI_SKIP_UPDATE_CHECK:'1',PI_TELEMETRY_DISABLED:'1'});
const report={root,platform:process.platform,arch:process.arch,node:process.version,startedAt:new Date().toISOString(),results:[],limits:[]};
const evidence=join(root,'session-cases.json');
const save=async(label,data={})=>{report.results.push({label,...data});await fs.writeFile(evidence,JSON.stringify(report,null,2)+'\n');console.log('PASS',label);};
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const sha=async path=>hash(await fs.readFile(path));
const native=join(root,'native'),packed=join(root,'package');
for(const dir of [join(native,'bin'),join(native,'runtime/node/bin')]) await fs.mkdir(dir,{recursive:true,mode:0o700});
for(const [source,destination,mode] of [[binary,'bin/mirket',0o700],[node,'runtime/node/bin/node',0o700],[license,'runtime/node/LICENSE',0o600]]) {
 const file=join(native,destination);await fs.copyFile(source,file);await fs.chmod(file,mode);
}
const release=JSON.parse(execFileSync(binary,['dev','manifest','--directory',native],{encoding:'utf8',timeout:60000})).release_id;
execFileSync('/usr/bin/python3',[resolve('scripts/assemble-pi-package.py'),'--mirket',binary,'--source',native,'--output',packed],{encoding:'utf8',timeout:60000});
const settingsFile=join(agentDir,'settings.json');
await fs.writeFile(settingsFile,JSON.stringify({theme:'untouched-fixture-setting',packages:[]}));
const setup=JSON.parse(execFileSync(node,[join(packed,'scripts/setup.mjs'),'--agent-dir',agentDir],{encoding:'utf8',timeout:660000}));
assert.equal(setup.releaseId,release);
const settings={theme:'untouched-fixture-setting',enableInstallTelemetry:false,packages:[packed],defaultTools:['+codemode','+tool_search'],defaultProvider:'offline-fixture',defaultModel:'deterministic'};
await fs.writeFile(settingsFile,JSON.stringify(settings));
const settingsHash=await sha(settingsFile);
await save('fresh private package installed',{release,binarySha256:await sha(binary),nodeSha256:await sha(node),settingsHash});

const sdk=await import(pathToFileURL(join(peers,'@earendil-works/pi-coding-agent/dist/index.js')).href);
const ai=await import(pathToFileURL(join(peers,'@earendil-works/pi-ai/dist/index.js')).href);
const settingsManager=sdk.SettingsManager.create(project,agentDir);
const loader=new sdk.DefaultResourceLoader({cwd:project,agentDir,settingsManager,noContextFiles:true,extensionFactories:[
 {name:'codemode',factory:sdk.createCodemodeExtension(),builtin:true,replaceable:true},
 {name:'tool-search',factory:sdk.createToolSearchExtension(),builtin:true,replaceable:true},
 {name:'mcp',factory:sdk.createMcpExtension(),builtin:true,replaceable:true},makeFixtureProvider(ai)]});
await loader.reload();assert.deepEqual(loader.getExtensions().errors,[]);
const {session}=await sdk.createAgentSession({cwd:project,agentDir,settingsManager,resourceLoader:loader,sessionManager:sdk.SessionManager.create(project,sessions)});
const errors=[];
await session.bindExtensions({mode:'print',uiContext:{notify:()=>{}},onError:e=>errors.push(e)});
await session.setModel(session.modelRuntime.getModel('offline-fixture','deterministic'));
async function invoke(tool,args={},expectedError){
 const before=session.messages.length;
 await session.prompt(JSON.stringify({tool,args}));
 const result=session.messages.slice(before).findLast(m=>m.role==='toolResult'&&m.toolName===tool);
 assert.ok(result,JSON.stringify(errors));
 assert.equal(result.isError,Boolean(expectedError),JSON.stringify(result));
 const raw=result.content.filter(c=>c.type==='text').map(c=>c.text).join('\n');
 if(expectedError){assert.match(raw,expectedError);return raw;}
 try{return JSON.parse(raw);}catch{return raw;}
}
const stateSnapshot=()=>JSON.parse(execFileSync('/usr/bin/python3',['-c',`
import hashlib,json,pathlib,sqlite3,sys
root=pathlib.Path(sys.argv[1]); tables={}; files={}
for file in sorted((root/'state').rglob('*.sqlite3')):
 db=sqlite3.connect(file.as_uri()+'?mode=ro',uri=True)
 try:
  names=[x[0] for x in db.execute("select name from sqlite_master where type='table' and name not like 'sqlite_%' order by name")]
  tables[file.name]={name:sorted(db.execute('select * from "'+name+'"').fetchall(),key=repr) for name in names}
 finally: db.close()
for file in sorted((root/'state'/'operations').rglob('*')):
 if file.is_file() and not any(file.name.endswith(x) for x in ('.sqlite3','.sqlite3-wal','.sqlite3-shm')):
  files[str(file.relative_to(root))]=hashlib.sha256(file.read_bytes()).hexdigest()
print(json.dumps({'tables':tables,'receiptFiles':files},sort_keys=True,default=str))
`,join(agentDir,'mirket')],{encoding:'utf8',timeout:5000}));
const grantFile=join(agentDir,'mirket/authorizations.json');
const grantBytes=()=>fs.readFile(grantFile);
try {
 const doctor=await invoke('mirket_doctor');assert.equal(doctor.status,'ready',JSON.stringify(doctor));
 await invoke('tool_search',{query:'memory_prepare memory_commit memory_read',limit:8});
 const scope={kind:'shared_preferences'},common={expected_release_id:release,scope};
 const record={kind:'preference',basis:'observed',content:'Installed shared-read fixture',sources:[{source_id:'fixture:shared-read',revision:'v1',locator:'fixture://offline',captured_at:123}],status:'current',critical:false,applicability:{task_key:null,checkout_id:null,operation:null},valid_from:null,valid_until:null};
 const prepareArgs={...common,mutation:{kind:'create',record,supersedes:null}};
 await session.prompt('/mirket-authorize '+JSON.stringify({tool:'mcp__mirket__memory_prepare',args:prepareArgs}));
 const prepared=await invoke('mcp__mirket__memory_prepare',prepareArgs);
 const commitArgs={...common,operation_id:prepared.operation_id};
 await session.prompt('/mirket-authorize '+JSON.stringify({tool:'mcp__mirket__memory_commit',args:commitArgs}));
 const committed=await invoke('mcp__mirket__memory_commit',commitArgs);
 assert.equal(committed.status,'committed');
 const readArgs={...common,record_id:committed.record.record_id};
 await invoke('mcp__mirket__memory_read',readArgs,/Explicit authorization required/);
 await session.prompt('/mirket-authorize '+JSON.stringify({tool:'mcp__mirket__memory_read',args:readArgs}));
 const grant=JSON.parse(await grantBytes());assert.ok(grant.grants.some(g=>g.expires>Date.now() && g.expires<=Date.now()+300000));
 const beforeRead=stateSnapshot(),grantBeforeRead=await grantBytes();
 await fs.writeFile(join(root,'read-before.json'),JSON.stringify(beforeRead,null,2)+'\n');
 assert.equal((await invoke('mcp__mirket__memory_read',readArgs)).record.data.content,record.content);
 assert.deepEqual(stateSnapshot(),beforeRead,'Authorized native read changed canonical state or receipt files');
 const oldRunner=session.extensionRunner;
 await session.reload();
 assert.notEqual(session.extensionRunner,oldRunner,'Pi did not replace its extension runner');
 await session.setModel(session.modelRuntime.getModel('offline-fixture','deterministic'));
 assert.equal((await invoke('mirket_doctor')).status,'ready');
 await invoke('tool_search',{query:'memory_read memory_prepare',limit:8});
 assert.equal((await invoke('mcp__mirket__memory_read',readArgs)).record.data.content,record.content);
 assert.deepEqual(await grantBytes(),grantBeforeRead,'Matching Pi reload changed the stored grant');
 const afterReadReload=stateSnapshot();
 assert.deepEqual(afterReadReload,beforeRead,'Reloaded native read changed canonical state or receipt files');
 await fs.writeFile(join(root,'read-after-reload.json'),JSON.stringify(afterReadReload,null,2)+'\n');
 await save('exact unexpired shared read grant survives actual Pi reload and rebind',{recordId:committed.record.record_id,grantCount:grant.grants.length,canonicalAndReceiptsUnchanged:true,stateSha256:hash(JSON.stringify(beforeRead))});

 // The real installed runner enters the TUI hook; the deferred UI response is
 // controlled here. Pi reload emits a real shutdown and replaces that runner.
 const pendingArgs={...common,mutation:{kind:'create',record:{...record,content:'Must not be prepared after shutdown'},supersedes:null}};
 const beforePending=stateSnapshot(),authBeforePending=await grantBytes();
 await fs.writeFile(join(root,'pending-before.json'),JSON.stringify(beforePending,null,2)+'\n');
 let accept,entered;
 const enteredPromise=new Promise(resolve=>entered=resolve);
 const acceptedPromise=new Promise(resolve=>accept=resolve);
 const runner=session.extensionRunner,originalCreateContext=runner.createContext.bind(runner);
 runner.createContext=()=>{
  const ctx=originalCreateContext();
  Object.defineProperties(ctx,{mode:{value:'tui',configurable:true},hasUI:{value:true,configurable:true},ui:{value:{confirm:()=>{entered();return acceptedPromise;}},configurable:true}});
  return ctx;
 };
 const pending=runner.emitToolCall({type:'tool_call',toolName:'mcp__mirket__memory_prepare',input:pendingArgs});
 await enteredPromise;
 await session.reload();
 assert.notEqual(session.extensionRunner,runner,'Shutdown did not replace pending runner');
 accept(true);
 const blocked=await pending;
 assert.equal(blocked?.block,true,JSON.stringify(blocked));
 assert.match(blocked.reason,/Session changed during authorization/);
 assert.deepEqual(await grantBytes(),authBeforePending,'Late acceptance stored a grant');
 assert.deepEqual(stateSnapshot(),beforePending,'Late acceptance changed canonical state or receipts');
 await session.setModel(session.modelRuntime.getModel('offline-fixture','deterministic'));
 await invoke('tool_search',{query:'memory_prepare',limit:8});
 await invoke('mcp__mirket__memory_prepare',pendingArgs,/Explicit authorization required/);
 const afterPending=stateSnapshot();
 assert.deepEqual(afterPending,beforePending,'Subsequent ungranted call changed canonical state or receipts');
 await fs.writeFile(join(root,'pending-after.json'),JSON.stringify(afterPending,null,2)+'\n');
 assert.equal(await sha(settingsFile),settingsHash,'Authorization flow changed unrelated Pi settings');
 await save('pending TUI confirmation blocked after actual Pi shutdown and reload',{blockReason:blocked.reason,canonicalAndReceiptsUnchanged:true,grantUnchanged:true,settingsUnchanged:true,stateSha256:hash(JSON.stringify(beforePending))});
 report.limits.push('Pending confirmation used the installed extension runner tool_call event with deferred TUI UI; it did not drive a visible terminal dialog or a true Pi session switch. Pi reload did emit session_shutdown and replaced the runner. Native non-dispatch is evidenced by unchanged canonical/receipt snapshots and a subsequent real prompt denial.');
} finally {
 await session.extensionRunner.emit({type:'session_shutdown',reason:'quit'});session.dispose();
 report.finishedAt=new Date().toISOString();report.elapsedMs=Date.now()-started;report.errors=errors;
 await fs.writeFile(evidence,JSON.stringify(report,null,2)+'\n');
 console.log('Evidence:',evidence);
}
