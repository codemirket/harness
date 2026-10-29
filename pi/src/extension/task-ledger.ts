// Shared durable accounting for managed work. Session branches never own counters.
import {createHash,randomUUID} from 'node:crypto';
import {mkdirSync,writeFileSync,renameSync,unlinkSync,rmdirSync,realpathSync,lstatSync} from 'node:fs';
import {join} from 'node:path';
import {regularBytes} from './runtime.js';
export interface TaskLimits {repairs:number;childStarts:number;concurrency:number}
export interface TaskBrief {deliverable:string;outputs:string[];rubric:string[];provider:string;model:string;effort:string}
interface Reservation {id:string;kind:'repair'|'child';state:'active'|'finished';outputs?:string[];outcome?:'complete'|'incomplete'|'cancelled'}
export interface TaskLedger {schema:1;task:string;project:string;briefSha256:string;limits:TaskLimits;brief?:TaskBrief;reservations:Reservation[]}
export function validOutput(path:string):boolean {return typeof path==='string'&&path.length>0&&path.length<=512&&!path.startsWith('/')&&!path.includes('\\')&&path.split('/').every(part=>part!==''&&part!=='.'&&part!=='..'&&part!=='.git'&&part!=='.pi'&&part!=='.agents');}
function validBrief(brief:TaskBrief):boolean {return !!brief&&typeof brief.deliverable==='string'&&brief.deliverable.length>0&&brief.deliverable.length<=8000&&Array.isArray(brief.outputs)&&brief.outputs.length<=32&&brief.outputs.every(validOutput)&&new Set(brief.outputs).size===brief.outputs.length&&Array.isArray(brief.rubric)&&brief.rubric.length>0&&brief.rubric.length<=16&&brief.rubric.every(item=>typeof item==='string'&&item.length>0&&item.length<=1000)&&[brief.provider,brief.model,brief.effort].every(item=>typeof item==='string'&&item.length>0&&item.length<=128);}
function validLimits(limits:TaskLimits):boolean {
 return !!limits && Number.isInteger(limits.repairs)&&limits.repairs>=0&&limits.repairs<=8&&Number.isInteger(limits.childStarts)&&limits.childStarts>=0&&limits.childStarts<=32&&Number.isInteger(limits.concurrency)&&limits.concurrency>=1&&limits.concurrency<=2;
}
function location(home:string,project:string,task:string):string {
 if(!/^[a-zA-Z0-9][a-zA-Z0-9_-]{0,63}$/.test(task))throw new Error('Invalid task identity');
 if(realpathSync(home)!==home||(lstatSync(home).mode&0o077)!==0)throw new Error('Task ledger needs private real runtime home');
 const folder=join(home,'tasks');try{mkdirSync(folder,{mode:0o700});}catch(error){if((error as NodeJS.ErrnoException).code!=='EEXIST')throw error;}
 if(realpathSync(folder)!==folder||(lstatSync(folder).mode&0o077)!==0)throw new Error('Invalid task ledger directory');
 return join(folder,createHash('sha256').update(JSON.stringify([realpathSync(project),task])).digest('hex')+'.json');
}
function read(file:string):TaskLedger|undefined {
 try { const value=JSON.parse(regularBytes(file,128*1024).toString());
  if(value.schema!==1||!validLimits(value.limits)||(value.brief!==undefined&&!validBrief(value.brief))||!Array.isArray(value.reservations)||value.reservations.length>128||value.reservations.some((r:Reservation)=>!r||!['child','repair'].includes(r.kind)||!['active','finished'].includes(r.state)||typeof r.id!=='string'||(r.outputs!==undefined&&(!Array.isArray(r.outputs)||!r.outputs.every(validOutput)))||(r.state==='finished'&&!['complete','incomplete','cancelled'].includes(r.outcome!)))||new Set(value.reservations.map((r:Reservation)=>r.id)).size!==value.reservations.length)throw new Error('Invalid task ledger');return value;
 }catch(error){if((error as NodeJS.ErrnoException).code==='ENOENT')return;throw error;}
}
function edit<T>(file:string,change:(ledger:TaskLedger|undefined)=>{ledger:TaskLedger;result:T}):T {
 const lock=file+'.lock';mkdirSync(lock,{mode:0o700});const temporary=file+'.'+randomUUID()+'.next';let created=false;
 try{const {ledger,result}=change(read(file));writeFileSync(temporary,JSON.stringify(ledger)+'\n',{flag:'wx',mode:0o600});created=true;renameSync(temporary,file);created=false;return result;}
 finally{if(created)unlinkSync(temporary);rmdirSync(lock);}
}
export function beginTask(home:string,project:string,task:string,briefSha256:string,limits:TaskLimits,brief?:TaskBrief):TaskLedger {
 if(!/^[a-f0-9]{64}$/.test(briefSha256)||!validLimits(limits)||(brief!==undefined&&!validBrief(brief)))throw new Error('Invalid finite task limits or brief');
 const canonical=realpathSync(project);
 return edit(location(home,canonical,task),existing=>{
  if(existing){if(existing.project!==canonical||existing.task!==task||existing.briefSha256!==briefSha256||JSON.stringify(existing.brief)!==JSON.stringify(brief)||existing.limits.repairs!==limits.repairs||existing.limits.childStarts!==limits.childStarts||existing.limits.concurrency!==limits.concurrency)throw new Error('Task brief and budget are frozen');return {ledger:existing,result:existing};}
  const ledger:TaskLedger={schema:1,task,project:canonical,briefSha256,limits,reservations:[]};if(brief)ledger.brief=brief;return {ledger,result:ledger};
 });
}
export function readTask(home:string,project:string,task:string):TaskLedger {
 const ledger=read(location(home,project,task));if(!ledger||ledger.project!==realpathSync(project)||ledger.task!==task)throw new Error('Task identity missing or changed');return ledger;
}
export function reserveTask(home:string,project:string,task:string,kind:'repair'|'child',id:string,outputs?:string[]):TaskLedger {
 if(kind!=='repair'&&kind!=='child')throw new Error('Invalid reservation kind');
 if(!/^[a-zA-Z0-9][a-zA-Z0-9_-]{0,63}$/.test(id))throw new Error('Invalid reservation identity');
 return edit(location(home,project,task),ledger=>{
  if(!ledger)throw new Error('Task must be explicitly started');
  if(ledger.project!==realpathSync(project)||ledger.task!==task)throw new Error('Task identity changed');
  if(outputs!==undefined){
   if(kind!=='child'||!ledger.brief||!Array.isArray(outputs)||outputs.some(path=>!validOutput(path)||!ledger.brief!.outputs.includes(path))||new Set(outputs).size!==outputs.length)throw new Error('Child outputs differ from the frozen task');
   const overlaps=(a:string,b:string)=>a===b||a.startsWith(b+'/')||b.startsWith(a+'/');
   if(ledger.reservations.some(r=>r.kind==='child'&&r.state==='active'&&r.outputs?.some(a=>outputs.some(b=>overlaps(a,b)))))throw new Error('Active child file ownership overlaps');
  }
  if(ledger.reservations.some(r=>r.id===id))throw new Error('Reservation already exists; inspect rather than replay');
  const previous=ledger.reservations.filter(r=>r.kind===kind);
  if(previous.length>=(kind==='repair'?ledger.limits.repairs:ledger.limits.childStarts))throw new Error('Task budget exhausted');
  if(kind==='child'&&previous.filter(r=>r.state==='active').length>=ledger.limits.concurrency)throw new Error('Managed child concurrency exhausted');
  const reservation:Reservation={id,kind,state:'active'};if(outputs!==undefined)reservation.outputs=outputs;ledger.reservations.push(reservation);return {ledger,result:ledger};
 });
}
export function finishTaskReservation(home:string,project:string,task:string,id:string,outcome:'complete'|'incomplete'|'cancelled'):TaskLedger {
 if(!['complete','incomplete','cancelled'].includes(outcome))throw new Error('Invalid reservation outcome');
 // The managed caller must observe process cleanup before releasing a child.
 // A timeout never frees a slot automatically or refunds consumed task budget.
 return edit(location(home,project,task),ledger=>{
  if(!ledger||ledger.project!==realpathSync(project)||ledger.task!==task)throw new Error('Unknown or changed task');const reservation=ledger.reservations.find(r=>r.id===id);if(!reservation)throw new Error('Unknown reservation');
  if(reservation.state==='finished'&&reservation.outcome!==outcome)throw new Error('Reservation outcome already recorded');
  reservation.state='finished';reservation.outcome=outcome;return {ledger,result:ledger};
 });
}
