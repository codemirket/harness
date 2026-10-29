// Actual concurrent CLI children and durable budgets across pi fork/reload.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {join,isAbsolute} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {makeFixtureProvider} from './pi-offline-provider.mjs';
const [peers,root,authFile,catalogFile]=process.argv.slice(2);assert.ok([peers,root,authFile,catalogFile].every(isAbsolute)&&root.startsWith('/private/tmp/mirket-pi-package-'));
const agentDir=join(root,'agent'),cwd=join(root,'project'),auth=join(agentDir,'auth.json');
Object.assign(process.env,{HOME:join(root,'home'),PI_CODING_AGENT_DIR:agentDir,PI_SKIP_UPDATE_CHECK:'1',PI_TELEMETRY_DISABLED:'1'});
const original=await fs.readFile(auth);assert.deepEqual(JSON.parse(original),{});await fs.copyFile(authFile,auth);await fs.chmod(auth,0o600);
const active=[],report={results:[],errors:[]};
try{
 await fs.copyFile(catalogFile,join(agentDir,'models-store.json'));
 const sdk=await import(pathToFileURL(join(peers,'@earendil-works/pi-coding-agent/dist/index.js'))),ai=await import(pathToFileURL(join(peers,'@earendil-works/pi-ai/dist/index.js')));
 async function open(manager){const settingsManager=sdk.SettingsManager.create(cwd,agentDir),loader=new sdk.DefaultResourceLoader({cwd,agentDir,settingsManager,noContextFiles:true,extensionFactories:[{name:'mcp',factory:sdk.createMcpExtension(),builtin:true,replaceable:true},makeFixtureProvider(ai)]});await loader.reload();assert.deepEqual(loader.getExtensions().errors,[]);const {session}=await sdk.createAgentSession({cwd,agentDir,settingsManager,resourceLoader:loader,sessionManager:manager??sdk.SessionManager.create(cwd,join(root,'shared-budget-sessions'))});active.push(session);await session.bindExtensions({mode:'print',uiContext:{notify:()=>{}},onError:e=>report.errors.push(String(e))});await session.setModel(session.modelRuntime.getModel('offline-fixture','deterministic'));return session;}
 async function call(s,tool,args={},error=false){const at=s.messages.length;await s.prompt(JSON.stringify({tool,args}));const result=s.messages.slice(at).findLast(m=>m.role==='toolResult'&&m.toolName===tool);assert.ok(result);assert.equal(!!result.isError,error,JSON.stringify(result));const text=result.content.filter(x=>x.type==='text').map(x=>x.text).join('\n');try{return JSON.parse(text)}catch{return text}}
 const a=await open(),task='shared-'+Date.now(),files=[0,1,2].map(n=>task+'-'+n+'.txt');
 const brief={task,deliverable:'Two independent saved files and one bounded repair',outputs:files,rubric:['Each assigned file contains its exact filename followed by a newline.'],provider:'openai',model:'gpt-6-astra',effort:'medium',repairs:1,childStarts:3};
 await a.prompt('/mirket-task '+JSON.stringify(brief));await call(a,'mirket_task_status');
 const parent=a.sessionManager.getSessionFile();assert.ok(parent);
 const b=await open(sdk.SessionManager.forkFrom(parent,cwd,join(root,'shared-budget-forks'))),c=await open(sdk.SessionManager.forkFrom(parent,cwd,join(root,'shared-budget-forks')));
 assert.equal((await call(b,'mirket_task_status')).task,task);assert.equal((await call(c,'mirket_task_status')).task,task);
 const ledgerPath=join(agentDir,'mirket/tasks',createHash('sha256').update(JSON.stringify([cwd,task])).digest('hex')+'.json');let peak=0;
 const readLedger=async()=>JSON.parse(await fs.readFile(ledgerPath));
 const request=n=>({prompt:`Run mirket_doctor, then write ${files[n]} with exactly its filename and one newline. Do not read other outputs.`,outputs:[files[n]],role:'mechanical',repair:false});
 const first=call(a,'mirket_delegate',request(0)),second=call(b,'mirket_delegate',request(1));
 const until=Date.now()+15000;while(Date.now()<until){const ledger=await readLedger();peak=Math.max(peak,ledger.reservations.filter(r=>r.kind==='child'&&r.state==='active').length);if(peak===2)break;await new Promise(r=>setTimeout(r,25))}assert.equal(peak,2);
 const refused=await call(c,'mirket_delegate',request(2),true);assert.match(JSON.stringify(refused),/concurrency exhausted/);
 report.children=await Promise.all([first,second]);for(let n=0;n<2;n++){assert.equal(report.children[n].status,'complete',JSON.stringify(report.children[n]));assert.equal(report.children[n].ownedProcessesRemaining,0);assert.equal(await fs.readFile(join(cwd,files[n]),'utf8'),files[n]+'\n')}
 report.results.push('Two actual Astra Medium children overlap in time, own disjoint files, save exact outputs and clean up; a third fork cannot exceed concurrency');console.log('PASS concurrent children and third-child refusal');
 await a.reload();await a.setModel(a.modelRuntime.getModel('offline-fixture','deterministic'));assert.equal((await call(a,'mirket_task_status')).reservations.length,2);
 const repair=await call(a,'mirket_delegate',{...request(2),repair:true});assert.equal(repair.status,'complete',JSON.stringify(repair));report.repair=repair;
 await c.reload();await c.setModel(c.modelRuntime.getModel('offline-fixture','deterministic'));const ledger=await call(c,'mirket_task_status');assert.equal(ledger.reservations.filter(r=>r.kind==='child').length,3);assert.equal(ledger.reservations.filter(r=>r.kind==='repair').length,1);
 const exhausted=await call(c,'mirket_delegate',{...request(2),repair:true},true);assert.match(JSON.stringify(exhausted),/budget exhausted/);report.ledger=ledger;report.results.push('Reload and fork preserve the same consumed child/repair budget; new output does not reset it');console.log('PASS fork/reload shared repair accounting');
}finally{for(const s of active){await s.extensionRunner.emit({type:'session_shutdown',reason:'quit'});s.dispose()}await fs.writeFile(auth,original,{mode:0o600});await fs.writeFile(join(root,'child-shared-budget-proof.json'),JSON.stringify(report,null,2),{mode:0o600});}
