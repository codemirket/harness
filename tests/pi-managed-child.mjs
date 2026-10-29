// Opt-in actual CLI child proof; parent tool selection is deterministic.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {join,isAbsolute} from 'node:path';
import {pathToFileURL} from 'node:url';
import {makeFixtureProvider} from './pi-offline-provider.mjs';
const [peers,root,authFile,catalogFile,imageSource]=process.argv.slice(2);assert.ok([peers,root,authFile,catalogFile,imageSource].every(isAbsolute)&&root.startsWith('/private/tmp/mirket-pi-package-'));
process.env.HOME=join(root,'home');process.env.PI_CODING_AGENT_DIR=join(root,'agent');process.env.PI_SKIP_UPDATE_CHECK='1';process.env.PI_TELEMETRY_DISABLED='1';
const agentDir=join(root,'agent'),cwd=join(root,'project');
const auth=join(agentDir,'auth.json'),previousAuth=await fs.readFile(auth).catch(error=>{if(error.code!=='ENOENT')throw error;return undefined;});
if(previousAuth)assert.deepEqual(JSON.parse(previousAuth),{},'Disposable test must not replace existing credentials');
await fs.copyFile(authFile,auth);await fs.chmod(auth,0o600);
let session;
const report={root,results:[],errors:[]};
try{
 await fs.copyFile(catalogFile,join(agentDir,'models-store.json'));
 const sdk=await import(pathToFileURL(join(peers,'@earendil-works/pi-coding-agent/dist/index.js'))),ai=await import(pathToFileURL(join(peers,'@earendil-works/pi-ai/dist/index.js')));
 const settingsManager=sdk.SettingsManager.create(cwd,agentDir),loader=new sdk.DefaultResourceLoader({cwd,agentDir,settingsManager,noContextFiles:true,extensionFactories:[{name:'mcp',factory:sdk.createMcpExtension(),builtin:true,replaceable:true},makeFixtureProvider(ai)]});await loader.reload();assert.deepEqual(loader.getExtensions().errors,[]);
 ({session}=await sdk.createAgentSession({cwd,agentDir,settingsManager,resourceLoader:loader,sessionManager:sdk.SessionManager.create(cwd,join(root,'managed-proof-sessions'))}));
 await session.bindExtensions({mode:'print',uiContext:{notify:()=>{}},onError:error=>report.errors.push(String(error))});await session.setModel(session.modelRuntime.getModel('offline-fixture','deterministic'));
 const task='managed-'+Date.now(),file='managed-proof-'+Date.now()+'.txt',cancelFile='cancel-'+file;
 await session.prompt('/mirket-task '+JSON.stringify({task,deliverable:'Prove a scoped independent child can save an exact artifact',outputs:[file,cancelFile],rubric:['File contains exactly Managed child proof followed by one newline.'],provider:'openai',model:'gpt-6-astra',effort:'medium',repairs:1,childStarts:2}));
 const at=session.messages.length;
 await session.prompt(JSON.stringify({tool:'mirket_delegate',args:{prompt:'Run mirket_doctor. Then write '+file+' containing exactly "Managed child proof" followed by one newline. Stop after saving the file; report its path.',outputs:[file],role:'mechanical',repair:false}}));
 const result=session.messages.slice(at).findLast(m=>m.role==='toolResult'&&m.toolName==='mirket_delegate');assert.ok(result);assert.equal(result.isError,false,JSON.stringify(result));const detail=JSON.parse(result.content.find(p=>p.type==='text').text);report.child=detail;assert.equal(detail.status,'complete',JSON.stringify(detail));assert.equal(await fs.readFile(join(cwd,file),'utf8'),'Managed child proof\n');assert.equal(detail.bootstrap.model,'gpt-6-astra');assert.equal(detail.bootstrap.effort,'medium');assert.equal(detail.ownedProcessesRemaining,0);report.results.push('Actual unbundled CLI child, verified bootstrap, exact saved output and zero observed descendants');console.log(JSON.stringify({status:'passed',evidence:detail.evidence,model:detail.bootstrap.model,effort:detail.bootstrap.effort}));
 const children=join(agentDir,'mirket/children'),before=new Set(await fs.readdir(children));
 const pending=session.prompt(JSON.stringify({tool:'mirket_delegate',args:{prompt:'Run mirket_doctor, then inspect '+file+' before writing '+cancelFile+'.',outputs:[cancelFile],role:'mechanical',repair:false}}));
 let cancelledDirectory;const until=Date.now()+15000;
 while(Date.now()<until){const name=(await fs.readdir(children)).find(name=>!before.has(name));if(name){cancelledDirectory=join(children,name);if(await fs.stat(join(cancelledDirectory,'bootstrap.json')).then(()=>true,()=>false))break;}await new Promise(r=>setTimeout(r,30));}
 assert.ok(cancelledDirectory);await session.abort();await pending.catch(error=>{report.cancelError=String(error);});
 const cancelResult=JSON.parse(await fs.readFile(join(cancelledDirectory,'result.json')));assert.equal(cancelResult.status,'cancelled',JSON.stringify(cancelResult));assert.equal(cancelResult.ownedProcessesRemaining,0);assert.equal(await fs.stat(join(cwd,cancelFile)).then(()=>true,()=>false),false);report.cancel=cancelResult;report.results.push('Actual child cancellation after bootstrap terminates observed descendants and publishes no output');
 const budgetAt=session.messages.length;await session.prompt(JSON.stringify({tool:'mirket_delegate',args:{prompt:'Attempt a third child after cancellation.',outputs:[cancelFile],role:'mechanical',repair:false}}));const refused=session.messages.slice(budgetAt).findLast(m=>m.role==='toolResult'&&m.toolName==='mirket_delegate');assert.equal(refused?.isError,true);assert.match(JSON.stringify(refused),/budget exhausted/);report.results.push('Cancellation consumes the shared child budget; third actual delegation refuses');
 console.log('PASS managed cancellation and exhausted budget');
 const image='review-'+Date.now()+'.png';await fs.copyFile(imageSource,join(cwd,image));
 await session.prompt('/mirket-task '+JSON.stringify({task:'review-'+Date.now(),deliverable:'Independent visual review',outputs:[image],rubric:['The background must be blue.','The white Mirket text must be complete and unclipped.'],provider:'openai',model:'gpt-6-astra',effort:'medium',repairs:1,childStarts:1}));
 const reviewAt=session.messages.length;await session.prompt(JSON.stringify({tool:'mirket_delegate',args:{prompt:'Inspect the saved pixels against every frozen rubric item. Report any mismatch; do not modify the artifact.',outputs:[],role:'creative-reviewer',repair:false}}));
 const reviewResult=session.messages.slice(reviewAt).findLast(m=>m.role==='toolResult'&&m.toolName==='mirket_delegate');assert.equal(reviewResult?.isError,false,JSON.stringify(reviewResult));const review=JSON.parse(reviewResult.content.find(p=>p.type==='text').text);assert.equal(review.status,'complete',JSON.stringify(review));assert.equal(review.reviewTargets.length,1);assert.match(JSON.stringify(review.final),/red|crimson|brick/i);report.review=review;report.results.push('Independent Astra Medium reviewer actually receives saved pixels and identifies the deliberate background mismatch');console.log('PASS actual creative reviewer image delivery and defect finding');
 const missing='missing-'+Date.now()+'.txt';await session.prompt('/mirket-task '+JSON.stringify({task:'missing-'+Date.now(),deliverable:'Missing artifact counterexample',outputs:[missing],rubric:['Declared output exists'],provider:'openai',model:'gpt-6-astra',effort:'medium',repairs:0,childStarts:1}));
 const missingAt=session.messages.length;await session.prompt(JSON.stringify({tool:'mirket_delegate',args:{prompt:'Run mirket_doctor only. This is a negative qualification fixture: do not create any file. Report the doctor result and stop.',outputs:[missing],role:'mechanical',repair:false}}));
 const missingResult=session.messages.slice(missingAt).findLast(m=>m.role==='toolResult'&&m.toolName==='mirket_delegate');assert.equal(missingResult?.isError,false,JSON.stringify(missingResult));const missingEvidence=JSON.parse(missingResult.content.find(p=>p.type==='text').text);assert.equal(missingEvidence.status,'incomplete');assert.match(missingEvidence.reason,/output is missing/);report.missingOutput=missingEvidence;report.results.push('A normally completed model turn with missing declared output is incomplete');console.log('PASS missing artifact refuses false completion');
}finally{
 if(session){await session.extensionRunner.emit({type:'session_shutdown',reason:'quit'});session.dispose();}
 if(previousAuth)await fs.writeFile(auth,previousAuth,{mode:0o600});else await fs.unlink(auth);await fs.writeFile(join(root,'managed-child-proof.json'),JSON.stringify(report,null,2),{mode:0o600});
}
