import {spawn,execFileSync} from 'node:child_process';
import {getPackageDir} from '@earendil-works/pi-coding-agent';
import {createHash,randomUUID} from 'node:crypto';
import {mkdirSync,writeFileSync,realpathSync,lstatSync} from 'node:fs';
import {dirname,join,resolve} from 'node:path';
import {regularBytes,type VerifiedRuntime} from './runtime.js';
import {readTask,reserveTask,finishTaskReservation} from './task-ledger.js';
import type {ChildContract} from './child-bootstrap.js';

interface Identity {pid:number;parent:number;started:string}
function processes():Map<number,Identity> {
 const rows=execFileSync('/bin/ps',['-axo','pid=,ppid=,lstart='],{encoding:'utf8',timeout:3000,maxBuffer:1024*1024});
 const result=new Map<number,Identity>();for(const row of rows.trim().split('\n')){const fields=row.trim().split(/\s+/);const pid=Number(fields.shift()),parent=Number(fields.shift());if(pid&&parent>=0)result.set(pid,{pid,parent,started:fields.join(' ')});}return result;
}
const delay=(ms:number)=>new Promise<void>(resolve=>setTimeout(resolve,ms));
export interface ManagedChildRequest {task:string;prompt:string;outputs:string[];role:'mechanical'|'creative-author'|'creative-reviewer';repair:boolean}
export async function runManagedChild(runtime:VerifiedRuntime,packageRoot:string,request:ManagedChildRequest,signal?:AbortSignal):Promise<Record<string,unknown>> {
 if(process.env.MIRKET_MANAGED_CHILD)throw new Error('Managed delegation has one level');
 if(process.platform!=='darwin'||process.arch!=='arm64')throw new Error('Managed child cleanup is qualified only on macOS ARM64');
 if(!request.prompt.trim()||request.prompt.length>16000)throw new Error('Child deliverable must be finite');
 const ledger=readTask(runtime.home,runtime.cwd,request.task);if(!ledger.brief)throw new Error('Task has no frozen workflow brief');
 if(request.role==='creative-reviewer'&&request.outputs.length)throw new Error('Creative reviewer cannot own author outputs');
 if(signal?.aborted)throw new Error('Child request already cancelled');
 const id=randomUUID();
 const directory=join(runtime.home,'children');try{mkdirSync(directory,{mode:0o700});}catch(error){if((error as NodeJS.ErrnoException).code!=='EEXIST')throw error;}
 if(realpathSync(directory)!==directory||(lstatSync(directory).mode&0o077)!==0)throw new Error('Managed child directory is not private and canonical');
 const work=join(directory,id);mkdirSync(work,{mode:0o700});
 const cli=join(getPackageDir(),'dist/cli.js'),bootstrap=join(packageRoot,'dist/extension/child-bootstrap.js');
 const identities={node:createHash('sha256').update(regularBytes(process.execPath,256*1024*1024)).digest('hex'),cli:createHash('sha256').update(regularBytes(cli,4*1024*1024)).digest('hex'),bootstrap:createHash('sha256').update(regularBytes(bootstrap,1024*1024)).digest('hex')};
 const tools=['read','mirket_doctor','mcp__mirket__runtime_status',...(request.role==='creative-reviewer'?[]:['write','edit'])];
 const reviewTargets=request.role==='creative-reviewer'?ledger.brief.outputs.map(path=>{const file=join(runtime.cwd,path);if(realpathSync(file)!==file)throw new Error('Review input is aliased');const bytes=regularBytes(file,16*1024*1024);return {path,sha256:createHash('sha256').update(bytes).digest('hex')};}):[];
 if(request.role==='creative-reviewer'&&!reviewTargets.some(item=>/\.(png|jpe?g|webp)$/i.test(item.path)))throw new Error('Creative review needs a saved raster artifact in the frozen outputs');
 const contract:ChildContract={schema:1,task:request.task,reservation:id,project:runtime.cwd,home:runtime.home,packageRoot,release:runtime.binding.releaseId,provider:ledger.brief.provider,model:ledger.brief.model,effort:ledger.brief.effort,outputs:request.outputs,tools,receipt:join(work,'bootstrap.json'),briefSha256:ledger.briefSha256};
 const contractPath=join(work,'contract.json');writeFileSync(contractPath,JSON.stringify(contract)+'\n',{flag:'wx',mode:0o600});
 const prompt=`Role: ${request.role}. Complete only this independent deliverable: ${request.prompt}\nOwned output files: ${JSON.stringify(request.outputs)}. Review inputs (read all when present): ${JSON.stringify(reviewTargets)}. Frozen task rubric: ${JSON.stringify(ledger.brief.rubric)}. Run mirket_doctor first. Do not delegate, change the task budget, or claim independent verification you did not perform. Report saved outputs, findings and remaining defects.`;
 const args=[cli,'--offline','--mode','json','--print','--no-extensions','-e','builtin:mcp','-e',join(packageRoot,'dist/extension/index.js'),'-e',bootstrap,'--no-skills','--no-prompt-templates','--no-themes','--no-context-files','--provider',contract.provider,'--model',contract.model,'--thinking',contract.effort,'--tools',tools.join(','),'--session-dir',join(work,'sessions'),prompt];
 writeFileSync(join(work,'launch.json'),JSON.stringify({identities,args,contract})+'\n',{flag:'wx',mode:0o600});
 reserveTask(runtime.home,runtime.cwd,request.task,'child',id,request.outputs);
 if(request.repair)try{reserveTask(runtime.home,runtime.cwd,request.task,'repair','repair-'+id);}catch(error){finishTaskReservation(runtime.home,runtime.cwd,request.task,id,'incomplete');throw error;}
 const child=spawn(process.execPath,args,{cwd:runtime.cwd,env:{...process.env,PI_CODING_AGENT_DIR:dirname(runtime.home),PI_CODING_AGENT_SESSION_DIR:join(work,'sessions'),PI_SKIP_UPDATE_CHECK:'1',PI_TELEMETRY_DISABLED:'1',MIRKET_MANAGED_CHILD:'1',MIRKET_CHILD_CONTRACT:contractPath},stdio:['ignore','pipe','pipe']});
 let stdout='',stderr='',reason:string|undefined,closed=false;
 const owned=new Map<number,Identity>();
 const sample=()=>{const all=processes();let added=true;while(added){added=false;for(const process of all.values())if(process.parent===child.pid||(owned.has(process.parent)&&all.get(process.parent)?.started===owned.get(process.parent)?.started)){if(!owned.has(process.pid)){owned.set(process.pid,process);added=true;}}}return all;};
 const stop=(why:string)=>{reason??=why;if(!closed)child.kill('SIGTERM');};
 const abort=()=>stop('cancelled');signal?.addEventListener('abort',abort,{once:true});
 child.stdout.on('data',bytes=>{if(Buffer.byteLength(stdout)+bytes.length>2*1024*1024)stop('Child output budget exhausted');else stdout+=bytes.toString();});
 child.stderr.on('data',bytes=>{if(Buffer.byteLength(stderr)+bytes.length>65536)stop('Child stderr budget exhausted');else stderr+=bytes.toString();});
 const watcher=setInterval(()=>{try{sample();}catch{stop('Could not inspect child ownership');}},200);
 const timeout=setTimeout(()=>stop('Child exceeded 120-second limit'),120000);
 const escalation=setInterval(()=>{if(reason&&!closed)child.kill('SIGKILL');},2000);
 let exitCode:number|null=null,spawnError:string|undefined;
 await new Promise<void>(resolve=>{child.once('error',error=>{spawnError=String(error);});child.once('close',code=>{exitCode=code;closed=true;resolve();});});
 clearInterval(watcher);clearInterval(escalation);clearTimeout(timeout);signal?.removeEventListener('abort',abort);
 writeFileSync(join(work,'events.jsonl'),stdout,{flag:'wx',mode:0o600});writeFileSync(join(work,'stderr.txt'),stderr,{flag:'wx',mode:0o600});
 const living=()=>{const all=processes();return [...owned.values()].filter(known=>all.get(known.pid)?.started===known.started);};
 for(const item of living())try{process.kill(item.pid,'SIGTERM');}catch(error){if((error as NodeJS.ErrnoException).code!=='ESRCH')throw error;}
 const cleanupDeadline=Date.now()+2000;while(living().length&&Date.now()<cleanupDeadline)await delay(50);
 for(const item of living())try{process.kill(item.pid,'SIGKILL');}catch(error){if((error as NodeJS.ErrnoException).code!=='ESRCH')throw error;}
 const hardDeadline=Date.now()+1000;while(living().length&&Date.now()<hardDeadline)await delay(50);
 if(living().length)throw new Error('Managed descendants remain; reservation stays active for recovery');
 let bootstrapRecord:Record<string,unknown>|undefined;
 try{bootstrapRecord=JSON.parse(regularBytes(contract.receipt,32768).toString());}catch{reason??='Missing child bootstrap evidence';}
 if(bootstrapRecord&&(bootstrapRecord.task!==contract.task||bootstrapRecord.reservation!==id||bootstrapRecord.project!==runtime.cwd||bootstrapRecord.release!==runtime.binding.releaseId||bootstrapRecord.model!==contract.model||bootstrapRecord.provider!==contract.provider||bootstrapRecord.effort!==contract.effort||bootstrapRecord.briefSha256!==ledger.briefSha256))reason??='Child bootstrap evidence differs';
 if(bootstrapRecord&&(bootstrapRecord.hostNodeSha256!==identities.node||bootstrapRecord.hostCliSha256!==identities.cli||bootstrapRecord.contractSha256!==createHash('sha256').update(regularBytes(contractPath,32768)).digest('hex')))reason??='Child executable or contract identity differs';
 let final:Record<string,unknown>|undefined,doctor=false;
 const readCalls=new Map<string,string>(),readInputs=new Map<string,boolean>();
 try{for(const line of stdout.split('\n').filter(Boolean)){const event=JSON.parse(line);if(event.type==='message_end'&&event.message?.role==='assistant')final=event.message;if(event.type==='tool_execution_end'&&event.toolName==='mirket_doctor'&&!event.isError)doctor=JSON.parse(event.result.content.find((p:{type:string})=>p.type==='text').text).status==='ready';if(event.type==='tool_execution_start'&&event.toolName==='read'&&typeof event.args?.path==='string')readCalls.set(event.toolCallId,resolve(runtime.cwd,event.args.path));if(event.type==='tool_execution_end'&&event.toolName==='read'&&!event.isError&&readCalls.has(event.toolCallId))readInputs.set(readCalls.get(event.toolCallId)!,event.result.content.some((p:{type:string})=>p.type==='image'));}}catch{reason??='Invalid child event stream';}
 if(!doctor)reason??='Child did not prove actual MCP readiness';
 if(spawnError||exitCode!==0||final?.stopReason!=='stop')reason??=spawnError??'Child did not finish normally';
 const artifacts:{path:string;bytes:number;sha256:string}[]=[];
 if(!reason)for(const path of request.outputs){try{const file=join(runtime.cwd,path);if(realpathSync(file)!==file)throw new Error('Output is aliased');const bytes=regularBytes(file,16*1024*1024);artifacts.push({path,bytes:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex')});}catch{reason='Declared child output is missing, aliased or exceeds the authored-source bound';break;}}
 if(!reason)for(const target of reviewTargets){try{const file=join(runtime.cwd,target.path);if(!readInputs.has(file)||createHash('sha256').update(regularBytes(file,16*1024*1024)).digest('hex')!==target.sha256||(/\.(png|jpe?g|webp)$/i.test(target.path)&&!readInputs.get(file)))throw new Error('Review evidence missing or changed');}catch{reason='Review inputs changed or were not actually read; saved pixels must reach the reviewer';break;}}
 const outcome=reason==='cancelled'?'cancelled':reason?'incomplete':'complete';
 finishTaskReservation(runtime.home,runtime.cwd,request.task,id,outcome);
 if(request.repair)finishTaskReservation(runtime.home,runtime.cwd,request.task,'repair-'+id,outcome);
 const report={status:outcome,reason,task:request.task,reservation:id,evidence:work,bootstrap:bootstrapRecord,artifacts,reviewTargets,final,ownedProcessesRemaining:0,limits:'Managed tool scope, saved artifact identity and observed process cleanup; not an OS sandbox or a quality verdict.'};
 writeFileSync(join(work,'result.json'),JSON.stringify(report)+'\n',{flag:'wx',mode:0o600});return report;
}
