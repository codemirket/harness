// Loaded explicitly only by the managed CLI child. No model-authored bootstrap
// text is accepted as proof of effective configuration.
import {getAgentDir,VERSION,type ExtensionAPI} from '@earendil-works/pi-coding-agent';
import {writeFileSync,realpathSync,lstatSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {dirname,resolve,relative,sep} from 'node:path';
import {regularBytes,verifyRuntime} from './runtime.js';
import {readTask} from './task-ledger.js';
export interface ChildContract {
 schema:1;task:string;reservation:string;project:string;home:string;packageRoot:string;
 release:string;provider:string;model:string;effort:string;outputs:string[];tools:string[];
 receipt:string;briefSha256:string;
}
export default function childBootstrap(pi:ExtensionAPI):void {
 const path=process.env.MIRKET_CHILD_CONTRACT;if(!path||process.env.MIRKET_MANAGED_CHILD!=='1')throw new Error('Managed child contract missing');
 const bytes=regularBytes(path,32768);const contract=JSON.parse(bytes.toString()) as ChildContract;
 if(contract.schema!==1||realpathSync(path)!==path||dirname(contract.receipt)!==dirname(path))throw new Error('Invalid child contract');
 let started=false,verified=false;
 pi.on('before_agent_start',async(_event,ctx)=>{
  if(started)return;started=true;
  try{
  if(realpathSync(ctx.cwd)!==contract.project||ctx.model?.provider!==contract.provider||ctx.model.id!==contract.model||ctx.thinkingLevel!==contract.effort)throw new Error('Managed child effective model, effort or project differs');
  const runtime=await verifyRuntime(contract.packageRoot,getAgentDir(),ctx.cwd,VERSION);
  if(runtime.home!==contract.home||runtime.binding.releaseId!==contract.release)throw new Error('Managed child runtime differs');
  const ledger=readTask(runtime.home,runtime.cwd,contract.task),reservation=ledger.reservations.find(r=>r.id===contract.reservation);
  if(ledger.briefSha256!==contract.briefSha256||reservation?.state!=='active'||JSON.stringify(reservation.outputs)!==JSON.stringify(contract.outputs))throw new Error('Managed child shared budget differs');
  const tools=pi.getActiveTools();if(tools.some(tool=>!contract.tools.includes(tool))||!tools.includes('mirket_doctor'))throw new Error('Managed child tool exposure differs');
  const registration=pi.getMcpServers().find(server=>server.name==='mirket');
  if(!registration||!('command' in registration.config)||registration.config.command!==runtime.command||registration.config.cwd!==runtime.cwd)throw new Error('Managed child MCP binding differs');
  writeFileSync(contract.receipt,JSON.stringify({schema:1,task:contract.task,reservation:contract.reservation,project:runtime.cwd,release:runtime.binding.releaseId,provider:ctx.model.provider,model:ctx.model.id,effort:ctx.thinkingLevel,tools,briefSha256:ledger.briefSha256,contractSha256:createHash('sha256').update(bytes).digest('hex'),hostNodeSha256:createHash('sha256').update(regularBytes(process.execPath,256*1024*1024)).digest('hex'),hostCliSha256:createHash('sha256').update(regularBytes(process.argv[1]!,4*1024*1024)).digest('hex'),pid:process.pid})+'\n',{flag:'wx',mode:0o600});
  verified=true;
  }catch(error){ctx.abort();throw error;}
 });
 pi.on('tool_call',event=>{
  if(!verified||!contract.tools.includes(event.toolName))return {block:true,reason:'Tool outside verified managed child exposure'};
  if(['read','write','edit'].includes(event.toolName)){
   try{
    const input=event.input as Record<string,unknown>,value=input.path??input.file_path;
    if(typeof value!=='string')throw new Error('Missing child file path');
    const file=resolve(contract.project,value),local=relative(contract.project,file);
    if(local==='..'||local.startsWith('..'+sep)||local.startsWith(sep))throw new Error('Child file leaves project');
    let existing=file;for(;;){try{lstatSync(existing);break;}catch(error){if((error as NodeJS.ErrnoException).code!=='ENOENT')throw error;existing=dirname(existing);}}
    if(realpathSync(existing)!==existing)throw new Error('Child file follows an alias');
    if(event.toolName!=='read'&&!contract.outputs.includes(local))throw new Error('Child does not own this output');
   }catch(error){return {block:true,reason:String(error)};}
  }
 });
 // Access through the managed built-in tools is scoped. This is not an OS sandbox.
}
