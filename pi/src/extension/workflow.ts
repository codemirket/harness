import type {ExtensionAPI} from '@earendil-works/pi-coding-agent';
import {Type} from 'typebox';
import {createHash} from 'node:crypto';
import {realpathSync} from 'node:fs';
import type {VerifiedRuntime} from './runtime.js';
import {beginTask,readTask,type TaskBrief} from './task-ledger.js';
import {runManagedChild} from './managed-child.js';

export function registerWorkflow(pi:ExtensionAPI,packageRoot:string,currentRuntime:()=>VerifiedRuntime|undefined):void {
 if(process.env.MIRKET_MANAGED_CHILD)return;
 let task:string|undefined;
 const running=new Set<AbortController>();
 pi.on('session_shutdown',()=>{for(const controller of running)controller.abort();task=undefined;});
 pi.on('session_start',(_event,ctx)=>{
  task=undefined;
  for(const entry of ctx.sessionManager.getBranch())if(entry.type==='custom'&&entry.customType==='mirket-task'){
   const data=entry.data as {task?:unknown;project?:unknown};if(typeof data?.task==='string'&&data.project===realpathSync(ctx.cwd))task=data.task;
  }
 });
 pi.registerCommand('mirket-task',{description:'Freeze task, outputs, rubric and finite shared budget from a JSON object',handler:async(args,ctx)=>{
  if(!ctx.isIdle())throw new Error('Start a task while the agent is idle');
  const runtime=currentRuntime();if(!runtime||realpathSync(ctx.cwd)!==runtime.cwd)throw new Error('Task needs an active Mirket project');
  const input=JSON.parse(args) as Record<string,unknown>;
  if(!input||typeof input!=='object'||Array.isArray(input)||Object.keys(input).some(k=>!['task','deliverable','outputs','rubric','repairs','childStarts','provider','model','effort'].includes(k)))throw new Error('Invalid task fields');
  const provider=input.provider??ctx.model?.provider,modelId=input.model??ctx.model?.id,effort=input.effort??ctx.thinkingLevel;
  if(typeof provider!=='string'||typeof modelId!=='string'||typeof effort!=='string'||typeof input.task!=='string')throw new Error('Task/model identity is missing');
  const model=ctx.modelRegistry.find(provider,modelId);if(!model||!ctx.modelRegistry.hasConfiguredAuth(model))throw new Error('Requested child model or authentication unavailable');
  const brief:TaskBrief={deliverable:input.deliverable as string,outputs:input.outputs as string[],rubric:input.rubric as string[],provider,model:modelId,effort};
  const digest=createHash('sha256').update(JSON.stringify(brief)).digest('hex');
  const ledger=beginTask(runtime.home,runtime.cwd,input.task,digest,{repairs:(input.repairs??2) as number,childStarts:(input.childStarts??6) as number,concurrency:2},brief);
  task=input.task;pi.appendEntry('mirket-task',{task,project:runtime.cwd,briefSha256:digest});
  pi.sendMessage({customType:'mirket-task',content:JSON.stringify({task,brief,limits:ledger.limits,reservations:ledger.reservations}),display:true});
 }});
 pi.registerTool({name:'mirket_delegate',label:'Managed pi child',description:'Run one finite independent child against the user-frozen task. Use disjoint output ownership. Child process completion is not independent quality verification.',parameters:Type.Object({prompt:Type.String({minLength:1,maxLength:16000}),outputs:Type.Array(Type.String(),{maxItems:32}),role:Type.Union([Type.Literal('mechanical'),Type.Literal('creative-author'),Type.Literal('creative-reviewer')]),repair:Type.Boolean()}),async execute(_id,args,signal,_update,ctx){
  const runtime=currentRuntime();if(!runtime||!task||runtime.cwd!==realpathSync(ctx.cwd))throw new Error('Use /mirket-task to freeze the task before delegation');
  const ledger=readTask(runtime.home,runtime.cwd,task),brief=ledger.brief;if(!brief)throw new Error('Task brief missing');
  const model=ctx.modelRegistry.find(brief.provider,brief.model);if(!model||!ctx.modelRegistry.hasConfiguredAuth(model))throw new Error('Frozen model/authentication unavailable');
  if(args.role.startsWith('creative')&&!model.input.includes('image'))throw new Error('Creative role requires image input capability');
  const controller=new AbortController();running.add(controller);
  try{const details=await runManagedChild(runtime,packageRoot,{task,...args},signal?AbortSignal.any([signal,controller.signal]):controller.signal);return {content:[{type:'text',text:JSON.stringify(details)}],details};}
  finally{running.delete(controller);}
 }});
 pi.registerTool({name:'mirket_task_status',label:'Task budget',description:'Read the durable task brief and consumed child/repair budget. Does not declare artifacts verified.',parameters:Type.Object({}),async execute(){
  const runtime=currentRuntime();if(!runtime||!task)throw new Error('No selected task');const details=readTask(runtime.home,runtime.cwd,task);return {content:[{type:'text',text:JSON.stringify(details)}],details};
 }});
}
