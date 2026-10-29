// Exactly one frozen author cell. Invoked by the controller in a disposable root.
import assert from 'node:assert/strict';import fs from 'node:fs/promises';import {join,resolve,sep} from 'node:path';import {pathToFileURL} from 'node:url';import {createHash} from 'node:crypto';
import {renderLocal} from './pi-cohort-render.mjs';
const [peers,root,cohort,taskId,condition,authFile,catalogFile,browserPeers,chrome]=process.argv.slice(2);
assert.ok(root.startsWith('/private/tmp/mirket-pi-package-'));assert.ok(['vanilla','mirket'].includes(condition));
const specBytes=await fs.readFile(join(cohort,'frozen.json')),spec=JSON.parse(specBytes),task=spec.tasks.find(t=>t.id===taskId);assert.ok(task);
const manifest=JSON.parse(await fs.readFile(join(cohort,'manifest.json')));assert.equal(createHash('sha256').update(specBytes).digest('hex'),manifest.specSha256);
const cell=join(cohort,taskId+'-'+condition),cwd=join(cell,'project'),agentDir=join(root,'agent'),packed=join(root,'package');await fs.mkdir(cell,{mode:0o700});await fs.mkdir(cwd,{mode:0o700});await fs.mkdir(join(cell,'evidence'),{mode:0o700});
for(const [name,bytes]of Object.entries(task.files))await fs.writeFile(join(cwd,name),bytes,{flag:'wx'});
for(const asset of task.assets??[]){assert.ok(!asset.name.includes('/')&&!asset.name.includes('..'));const bytes=await fs.readFile(asset.source);assert.equal(createHash('sha256').update(bytes).digest('hex'),asset.sha256);await fs.writeFile(join(cwd,asset.name),bytes,{flag:'wx'});}
Object.assign(process.env,{HOME:join(root,'home'),PI_CODING_AGENT_DIR:agentDir,PI_CODING_AGENT_SESSION_DIR:join(cell,'sessions'),PI_SKIP_UPDATE_CHECK:'1',PI_TELEMETRY_DISABLED:'1',PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD:'1'});
const sdk=await import(pathToFileURL(join(peers,'@earendil-works/pi-coding-agent/dist/index.js'))),{AuthStorage,ReadOnlyAuthStorage}=await import(pathToFileURL(join(peers,'@earendil-works/pi-coding-agent/dist/core/auth-storage.js'))),{Type}=await import(pathToFileURL(join(peers,'typebox/build/index.mjs')).href);
// Read-only by default: never rotate a refresh token then discard its replacement.
// The opt-in requires the user's narrow live-auth-store exception.
const credentials=process.env.MIRKET_COHORT_AUTH_REFRESH==='authorized'?AuthStorage.create(authFile):new ReadOnlyAuthStorage(authFile);await fs.copyFile(catalogFile,join(cell,'models.json'));
const runtime=await sdk.ModelRuntime.create({credentials,modelsPath:null,modelsStorePath:join(cell,'models.json'),allowModelNetwork:false});const model=runtime.getModel('openai','gpt-6-astra');assert.ok(model);
const binding=JSON.parse(await fs.readFile(join(packed,'native-binding.json'))),home=join(agentDir,'mirket');
const report={taskId,condition,started:new Date().toISOString(),platform:process.platform,arch:process.arch,node:process.version,specSha256:manifest.specSha256,releaseId:binding.releaseId,requested:spec.model,notices:[],toolCalls:[],renders:[],usage:{input:0,output:0,cacheRead:0,cacheWrite:0,totalTokens:0,cost:0},repairTurns:0};
let session,count=0,preview=0,budgetAbort=false;
const policy=pi=>{
 if(condition==='vanilla')pi.on('session_start',()=>pi.registerMcpServer('mirket',{command:join(home,'bin/mirket'),args:['serve','--home',home,'--expected-release-id',binding.releaseId],cwd,exposure:'deferred',toolExposure:{runtime_status:'direct'}}));
 pi.on('before_provider_request',e=>({...e.payload,max_output_tokens:8192}));
 pi.on('tool_call',e=>{
  report.toolCalls.push({name:e.toolName,input:e.input});if(++count>spec.limits.toolCalls)return {block:true,reason:'Frozen tool budget exhausted'};
  if(['write','edit'].includes(e.toolName)){const path=resolve(cwd,e.input.path??'');if(!task.outputs.some(name=>path===join(cwd,name)))return {block:true,reason:'Frozen output ownership only'};}
  if(e.toolName==='read'){const path=resolve(cwd,e.input.path??'');if(!(path.startsWith(cwd+sep)||path.startsWith(join(packed,'skills')+sep)||path.startsWith(join(cell,'evidence')+sep)))return {block:true,reason:'Frozen project inputs/skills/evidence only'};}
  if(e.toolName.startsWith('mcp__mirket__')){const input=e.input,project=input.project_root??input.scope?.project_root;if(project&&project!==cwd)return {block:true,reason:'Frozen project scope only'};
   if(/forget|commit|prepare|start|cancel|index_refresh/.test(e.toolName))return {block:true,reason:'Author cohort uses saved inputs and local previews; native effects qualified separately'};}
  if(e.toolName==='mirket_delegate')return {block:true,reason:'These frozen cells explicitly prohibit delegation'};
 });
 pi.registerTool({name:'local_preview',label:'Local saved preview',description:'Render a saved project HTML or SVG with the shared isolated local-only Chrome renderer. Returns saved pixels and observed layout/keyboard state. No external requests are served.',parameters:Type.Object({file:Type.String(),width:Type.Integer({minimum:320,maximum:1000}),height:Type.Integer({minimum:240,maximum:800}),time:Type.Number({minimum:0,maximum:2}),reduced:Type.Boolean()}),async execute(_id,args){
  if(!task.outputs.includes(args.file)||!['.html','.svg'].some(x=>args.file.endsWith(x)))throw Error('Only frozen visual outputs');
  const output=join(cell,'evidence',`preview-${++preview}.png`);const result=await renderLocal({cwd,...args,browserPeers,chrome,output});report.renders.push(result);return {content:[{type:'text',text:JSON.stringify(result)},{type:'image',mimeType:'image/png',data:(await fs.readFile(output)).toString('base64')}],details:result};
 }});
};
const skills=task.skills.map(id=>`skills/${id}/SKILL.md`);
const settingsManager=sdk.SettingsManager.inMemory({packages:condition==='mirket'?[{source:packed,skills}]:[],enableInstallTelemetry:false});
const loader=new sdk.DefaultResourceLoader({cwd,agentDir,settingsManager,noContextFiles:true,noSkills:condition==='vanilla',noPromptTemplates:true,noThemes:true,extensionFactories:[{name:'mcp',factory:sdk.createMcpExtension(),builtin:true,replaceable:true},{name:'tool-search',factory:sdk.createToolSearchExtension(),builtin:true,replaceable:true},policy]});
await loader.reload();assert.deepEqual(loader.getExtensions().errors,[]);report.availableSkills=loader.getSkills().skills.map(s=>({name:s.name,filePath:s.filePath}));if(condition==='vanilla')assert.equal(report.availableSkills.length,0);
({session}=await sdk.createAgentSession({cwd,agentDir,settingsManager,resourceLoader:loader,modelRuntime:runtime,model,thinkingLevel:'medium',tools:['read','write','edit','tool_search','local_preview','mirket_doctor','mirket_task_status'],sessionManager:sdk.SessionManager.create(cwd,join(cell,'sessions'))}));
let timer;const start=performance.now();
try{
 await session.bindExtensions({mode:'print',uiContext:{notify:(text,type)=>report.notices.push({text,type})},onError:e=>report.notices.push({error:String(e)})});assert.equal(session.model.provider,'openai');assert.equal(session.model.id,'gpt-6-astra');assert.equal(session.thinkingLevel,'medium');report.effective={provider:session.model.provider,id:session.model.id,effort:session.thinkingLevel};
 session.subscribe(event=>{if(event.type==='message_end'&&event.message.role==='assistant'){const u=event.message.usage??{};for(const key of ['input','output','cacheRead','cacheWrite','totalTokens'])report.usage[key]+=u[key]??0;report.usage.cost+=u.cost?.total??0;if(report.usage.totalTokens>spec.limits.observedTokens){budgetAbort=true;void session.abort();}}});
 timer=setTimeout(()=>{budgetAbort=true;void session.abort();},spec.limits.wallSeconds*1000);
 if(process.env.PI_COHORT_PREFLIGHT==='1'){report.status='preflight';}else await session.prompt(spec.commonPrompt+'\n\n'+task.prompt+'\n\nInput files: '+[...Object.keys(task.files),...(task.assets??[]).map(a=>a.name)].join(', ')+'\nRequired outputs: '+task.outputs.join(', '));const final=session.messages.findLast(message=>message.role==='assistant');
 if(process.env.PI_COHORT_PREFLIGHT!=='1'&&!budgetAbort&&final?.stopReason!=='stop')throw new Error(final?.errorMessage??'Author did not finish normally');
 report.status=process.env.PI_COHORT_PREFLIGHT==='1'?'preflight':budgetAbort?'budget_exhausted':'completed';
}catch(error){report.status='failed';report.error=String(error?.stack??error);}finally{
 clearTimeout(timer);report.elapsedMs=performance.now()-start;report.messages=session.messages;report.outputs=[];
 for(const name of task.outputs){try{const bytes=await fs.readFile(join(cwd,name));report.outputs.push({name,bytes:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex')});}catch{report.outputs.push({name,missing:true});}}
 await session.extensionRunner.emit({type:'session_shutdown',reason:'quit'});session.dispose();await fs.writeFile(join(cell,'report.json'),JSON.stringify(report,null,2),{mode:0o600});console.log(JSON.stringify({taskId,condition,status:report.status,elapsedMs:report.elapsedMs,usage:report.usage,outputs:report.outputs}));
}
