// Actual admitted Pi CLI/RPC project registration and observation with an offline fixture provider.
// Run with the private Node: tests/pi-project-setup.mjs PEERS BINARY NODE LICENSE.
// Equivalent explicit inputs: MIRKET_PI_PEERS, MIRKET_BINARY, MIRKET_NODE, MIRKET_NODE_LICENSE.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawn, execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {join, isAbsolute, resolve, dirname} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';
import {release as osRelease} from 'node:os';
import {createInterface} from 'node:readline';

const [peers=process.env.MIRKET_PI_PEERS, binary=process.env.MIRKET_BINARY,
  node=process.env.MIRKET_NODE, license=process.env.MIRKET_NODE_LICENSE] = process.argv.slice(2);
assert.ok([peers,binary,node,license].every(path=>path && isAbsolute(path)),
  'Supply absolute admitted peer tree, release binary, private Node and Node LICENSE paths');
assert.equal(process.execPath,node,'Run under the selected private Node');
const repository=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const root=await fs.mkdtemp('/private/tmp/mirket-pi-project-');
await fs.chmod(root,0o700);
console.log('Fixture:',root);
const agentDir=join(root,'agent'), project=join(root,'project'), home=join(root,'home'), sessions=join(root,'sessions');
for(const directory of [agentDir,project,home,sessions]) await fs.mkdir(directory,{mode:0o700});
const env=Object.fromEntries(Object.entries(process.env).filter(([key])=>!key.startsWith('PI_')
  && !key.startsWith('MIRKET_') && !key.startsWith('NODE_') && !key.startsWith('OPENAI_') && !key.startsWith('ANTHROPIC_')));
Object.assign(env,{HOME:home,PI_CODING_AGENT_DIR:agentDir,PI_CODING_AGENT_SESSION_DIR:sessions,
  PI_SKIP_UPDATE_CHECK:'1',PI_TELEMETRY_DISABLED:'1',PATH:'/nonexistent'});
const report={root,platform:process.platform,architecture:process.arch,osRelease:osRelease(),node:process.version,
  ...(process.platform==='darwin'?{macOSVersion:execFileSync('/usr/bin/sw_vers',['-productVersion'],{encoding:'utf8'}).trim(),
    macOSBuild:execFileSync('/usr/bin/sw_vers',['-buildVersion'],{encoding:'utf8'}).trim()}:{}),
  candidate:'current first-party source and selected native core assembled into a disposable minimal release; not released package qualification',
  startedAt:new Date().toISOString(),results:[],limits:[
    'Offline deterministic provider drives actual CLI tool calls; no paid model, visible TUI, image delivery, or other-platform qualification.',
    'Actual idle Pi slash expansion is exercised; queued streaming/follow-up skill invocation is outside this integration fixture.'
  ]};
const evidence=join(root,'project-setup-report.json');
const writeReport=()=>fs.writeFile(evidence,JSON.stringify(report,null,2)+'\n');
const save=async(label,data={})=>{report.results.push({label,...data});await writeReport();console.log('PASS',label);};
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const sha=async file=>hash(await fs.readFile(file));
const native=join(root,'native'), packed=join(root,'package');
for(const directory of ['bin','runtime/node/bin']) await fs.mkdir(join(native,directory),{recursive:true,mode:0o700});
for(const [source,destination,mode] of [[binary,'bin/mirket',0o700],[node,'runtime/node/bin/node',0o700],[license,'runtime/node/LICENSE',0o600]]) {
  await fs.copyFile(source,join(native,destination));await fs.chmod(join(native,destination),mode);
}
const run=(executable,args,timeout=60000)=>execFileSync(executable,args,{encoding:'utf8',timeout,env,cwd:repository});
const release=JSON.parse(run(binary,['dev','manifest','--directory',native])).release_id;
run('/usr/bin/python3',[join(repository,'scripts/assemble-pi-package.py'),'--mirket',binary,'--source',native,'--output',packed]);
const settingsFile=join(agentDir,'settings.json');
await fs.writeFile(settingsFile,JSON.stringify({theme:'preserved-project-fixture',packages:[]}));
const installed=JSON.parse(run(node,[join(packed,'scripts/setup.mjs'),'--agent-dir',agentDir],660000));
assert.equal(installed.releaseId,release);
const globalSettings=JSON.parse(await fs.readFile(settingsFile,'utf8'));
Object.assign(globalSettings,{enableInstallTelemetry:false,defaultProvider:'offline-fixture',defaultModel:'deterministic',defaultTools:['+codemode','+tool_search']});
await fs.writeFile(settingsFile,JSON.stringify(globalSettings));
const globalSettingsHash=await sha(settingsFile), manual='# Fixture project\n\nPreserve these instructions exactly.\n';
const suffix='\n## Manual instructions after the managed section\n\nPreserve this trailing content exactly.\n';
await fs.writeFile(join(project,'AGENTS.md'),manual);
const fixtureProvider=join(root,'provider.mjs'), contextFile=join(root,'model-contexts.jsonl');
await fs.writeFile(fixtureProvider,
  `import {appendFileSync} from 'node:fs';\nimport * as ai from ${JSON.stringify(pathToFileURL(join(peers,'@earendil-works/pi-ai/dist/index.js')).href)};\n`
  +`import {makeFixtureProvider} from ${JSON.stringify(pathToFileURL(join(repository,'tests/pi-offline-provider.mjs')).href)};\n`
  +`export default function(pi){pi.registerCommand('fixture-reload',{description:'Test-only actual Pi session reload',handler:async(_args,ctx)=>{await ctx.reload();}});\n`
  +`makeFixtureProvider(ai)({registerProvider(name,config){pi.registerProvider(name,{...config,streamSimple(model,context,...rest){\n`
  +`const index=context.messages.findLastIndex(message=>message.role==='user'), user=context.messages[index];\n`
  +`const text=typeof user.content==='string'?user.content:user.content.filter(part=>part.type==='text').map(part=>part.text).join('');\n`
  +`const systemMessages=context.messages.filter(message=>message.role==='system');\n`
  +`const projectContext=systemMessages.findLast(message=>Object.hasOwn(message.sections??{},'project_context'))?.sections.project_context;\n`
  +`appendFileSync(${JSON.stringify(contextFile)},JSON.stringify({systemMessages,projectContext,lastUserText:text})+'\\n');\n`
  +`if(text.startsWith('<skill ')&&text.includes('</skill>')){const messages=[...context.messages];messages[index]={...user,content:text.slice(text.indexOf('</skill>')+8).trim()};context={...context,messages};}\n`
  +`return config.streamSimple(model,context,...rest);}});}});}\n`);
const cli=join(peers,'@earendil-works/pi-coding-agent/dist/cli.js');
report.identity={release,entry:cli,binarySha256:await sha(binary),nodeSha256:await sha(node),
  hostVersion:JSON.parse(await fs.readFile(join(peers,'@earendil-works/pi-coding-agent/package.json'),'utf8')).version,
  hostInventorySha256:await sha(join(packed,'host-inventory.json')),packageInventorySha256:await sha(join(packed,'package-inventory.json'))};
await save('fresh disposable package installed with explicit private executables');

const child=spawn(node,[cli,'--offline','--provider','offline-fixture','--model','deterministic','-e',fixtureProvider,'--mode','rpc'],
  {cwd:project,env,stdio:['pipe','pipe','pipe'],detached:true});
const events=[],lines=createInterface({input:child.stdout}),started=Date.now();
let stderr='',childError,sequence=0,wake;
const exited=new Promise(resolve=>child.once('close',(code,signal)=>resolve({code,signal})));
child.once('error',error=>{childError=error;wake?.();});
child.stdin.on('error',error=>{childError=error;wake?.();});
child.stderr.on('data',bytes=>{stderr+=bytes;if(stderr.length>4*1024*1024){childError=Error('CLI stderr exceeds fixture budget');wake?.();}});
lines.on('line',line=>{
  try {events.push(JSON.parse(line));if(events.length>10000)throw Error('CLI event count exceeds fixture budget');}
  catch(error){childError=error;}
  wake?.();
});
child.once('close',()=>wake?.());
async function until(predicate,from=0,timeout=45000) {
  const deadline=Date.now()+timeout;
  while(true) {
    const result=events.slice(from).find(predicate);if(result)return result;
    if(childError)throw childError;
    if(child.exitCode!==null || child.signalCode!==null)throw Error(`Pi exited before expected event: ${stderr}`);
    const remaining=Math.min(deadline-Date.now(),300000-(Date.now()-started));
    assert.ok(remaining>0,`RPC deadline exceeded; stderr: ${stderr}`);
    await new Promise(resolve=>{const timer=setTimeout(resolve,Math.min(remaining,1000));wake=()=>{clearTimeout(timer);resolve();};});
    wake=undefined;
  }
}
async function request(type,fields={}) {
  const id=String(++sequence),from=events.length;
  child.stdin.write(JSON.stringify({id,type,...fields})+'\n');
  const response=await until(event=>event.type==='response' && event.id===id,from);
  assert.equal(response.success,true,JSON.stringify(response));return response.data;
}
async function prompt(message) {
  const from=events.length,response=await request('prompt',{message});
  if(response?.disposition!=='handled')await until(event=>event.type==='agent_settled',from);
  const observed=events.slice(from);
  assert.deepEqual(observed.filter(event=>event.type==='extension_error'),[],`Pi extension failed while handling ${message}`);
  assert.deepEqual(observed.filter(event=>event.type==='message_end' && event.message.role==='assistant' && event.message.stopReason==='error'),[],
    `Fixture provider failed while handling ${message}`);
  return observed;
}
async function tool(name,args={}) {
  const observed=await prompt(JSON.stringify({tool:name,args}));
  const result=observed.findLast(event=>event.type==='tool_execution_end' && event.toolName===name);
  assert.ok(result,JSON.stringify(observed));assert.equal(result.isError,false,JSON.stringify(result));
  const text=result.result.content.filter(part=>part.type==='text').map(part=>part.text).join('\n');
  try{return JSON.parse(text);}catch{return text;}
}
async function projectStatus() {
  const observed=await prompt('/mirket-project status');
  const message=observed.findLast(event=>event.type==='message_end' && event.message.role==='custom' && event.message.customType==='mirket-project');
  assert.ok(message,'Actual CLI status command returned no project status');
  return JSON.parse(message.message.content);
}
const guidance=()=>fs.readFile(join(project,'AGENTS.md'),'utf8');
function metadata(text) {
  assert.equal(text.split('<!-- mirket-project:start -->').length,2,'Expected one managed block');
  assert.equal(text.split('<!-- mirket-project:end -->').length,2,'Expected one managed block end');
  assert.ok(text.startsWith(manual),'Unmanaged project instructions changed');
  assert.ok(text.endsWith(suffix),'Trailing unmanaged project instructions changed');
  const match=text.match(/<!-- mirket-project-data (\{[^\n]*\}) -->/);
  assert.ok(match,'Managed guidance lacks observation metadata');return JSON.parse(match[1]);
}
const pause=milliseconds=>new Promise(resolve=>setTimeout(resolve,milliseconds));
function members() {
  return execFileSync('/bin/ps',['-axo','pid=,pgid=,stat=,command='],{encoding:'utf8'}).split('\n').flatMap(line=>{
    const match=line.trim().match(/^(\d+)\s+(\d+)\s+(\S+)\s+(.*)$/);
    return match && Number(match[2])===child.pid && !match[3].startsWith('Z')?[{pid:Number(match[1]),command:match[4]}]:[];
  });
}
try {
  await request('get_state');
  assert.ok((await request('get_commands')).commands.some(command=>command.name==='mirket-project' && command.source==='extension'));
  const catalog=JSON.parse(await fs.readFile(join(packed,'skills/catalog.json'),'utf8'));
  const selected=[...new Set([...catalog.global.map(skill=>skill.id),...catalog.profiles['library-cli'].skills,'js-ts-navigation'])].sort();
  const preservedSkills=selected.map(id=>`skills/${id}/SKILL.md`);
  await fs.mkdir(join(project,'.pi'),{mode:0o700});
  await fs.writeFile(join(project,'.pi/settings.json'),JSON.stringify({theme:'preserved-project-theme',
    packages:[{source:packed,extensions:[],skills:preservedSkills,prompts:[]}]}));
  await prompt('/mirket-project init');
  const projectSettings=JSON.parse(await fs.readFile(join(project,'.pi/settings.json'),'utf8'));
  const projectPackage=projectSettings.packages.find(entry=>entry.source===packed);
  assert.deepEqual(projectPackage.skills,preservedSkills,'Init replaced the existing selected skill filter');
  assert.ok(projectPackage.extensions.includes('+./dist/extension/index.js'),'Init did not explicitly enable Mirket extension');
  assert.deepEqual(projectPackage.prompts,[],'Init changed unrelated package resource filters');
  assert.equal(projectSettings.theme,'preserved-project-theme','Init changed unrelated project settings');
  await fs.appendFile(join(project,'AGENTS.md'),suffix);
  const initialized=await guidance();metadata(initialized);
  assert.equal(await sha(settingsFile),globalSettingsHash,'Project registration changed global Pi settings');
  await fs.writeFile(join(root,'AGENTS-initial.md'),initialized);
  await save('actual CLI project init enables a disabled Mirket extension and preserves selected skills, manual guidance and unrelated settings');

  // /reload belongs to Pi's TUI. RPC reaches the same host reload through a
  // fixture-only extension command calling the public command context method.
  await prompt('/fixture-reload');
  const commands=(await request('get_commands')).commands;
  assert.ok(commands.some(command=>command.name==='mirket-project' && command.source==='extension'),'Actual reload lost the Mirket project command');
  assert.deepEqual(commands.filter(command=>command.source==='skill').map(command=>command.name.slice('skill:'.length)).sort(),selected);
  await save('actual CLI reload retains the Mirket extension and exposes the exact selected native Pi skills',{selected,extensions:projectPackage.extensions});

  const status=await tool('mcp__mirket__runtime_status');assert.equal(status.release_id,release);
  assert.equal(await guidance(),initialized,'Administrative status call counted as substantive use');
  await tool('tool_search',{query:'memory_prepare memory_commit memory_read',limit:8});
  const common={expected_release_id:release,scope:{kind:'project',project_root:project}};
  const record={kind:'fact',basis:'observed',content:'Disposable project setup integration fact',
    sources:[{source_id:'fixture:project-setup',revision:'v1',locator:'fixture://offline',captured_at:123}],
    status:'current',critical:false,applicability:{task_key:null,checkout_id:null,operation:null},valid_from:null,valid_until:null};
  const prepared=await tool('mcp__mirket__memory_prepare',{...common,mutation:{kind:'create',record,supersedes:null}});
  const committed=await tool('mcp__mirket__memory_commit',{...common,operation_id:prepared.operation_id});
  assert.equal(committed.status,'committed');
  assert.equal((await tool('mcp__mirket__memory_read',{...common,record_id:committed.record.record_id})).record.data.content,record.content);
  const afterTool=await guidance();
  for(const id of ['memory_prepare','memory_commit','memory_read'])assert.ok(metadata(afterTool).tools.includes('mcp__mirket__'+id));
  assert.notEqual(afterTool,initialized,'Actual Mirket tool use did not update managed guidance');
  const skill='code-change';assert.ok(selected.includes(skill));
  const read=await tool('read',{path:join(packed,'skills',skill,'SKILL.md')});assert.match(typeof read==='string'?read:JSON.stringify(read),/code-change/);
  const afterSkill=await guidance();assert.ok(metadata(afterSkill).skills.includes(skill));
  assert.notEqual(afterSkill,afterTool,'Actual selected skill read did not update managed guidance');
  const slashSkill='critical-review';assert.ok(selected.includes(slashSkill));
  await prompt('/skill:'+slashSkill+' '+JSON.stringify({tool:'mcp__mirket__runtime_status'}));
  const afterSlash=await guidance();assert.ok(metadata(afterSlash).skills.includes(slashSkill),'Actual Pi slash skill expansion was not observed');
  await fs.writeFile(join(root,'AGENTS-after-use.md'),afterSlash);
  await tool('mcp__mirket__runtime_status');
  const contexts=(await fs.readFile(contextFile,'utf8')).trim().split('\n').map(line=>JSON.parse(line));
  assert.ok(contexts.some(context=>context.lastUserText.startsWith('<skill name="critical-review" ')),'Fixture did not consume an actual Pi-expanded slash skill');
  const section=afterSlash.slice(afterSlash.indexOf('<!-- mirket-project:start -->'),afterSlash.indexOf('<!-- mirket-project:end -->')+'<!-- mirket-project:end -->'.length);
  assert.ok(contexts.at(-1).projectContext?.includes(section),'Next actual model turn did not consume current managed project guidance');
  await prompt('/mirket-project init');
  assert.equal(await guidance(),afterSlash,'Repeated init changed existing selection or observations');
  assert.equal(await sha(settingsFile),globalSettingsHash,'Observed use changed unrelated global settings');
  assert.deepEqual(JSON.parse(await fs.readFile(join(project,'.pi/settings.json'),'utf8')),projectSettings);
  const sync=await projectStatus();
  assert.equal(sync.status,'ready');assert.equal(sync.guidanceFile,join(project,'AGENTS.md'));assert.equal(sync.project,project);
  assert.equal(await guidance(),afterSlash,'Status command changed source guidance');
  await save('actual native MCP tool, verified skill read and Pi slash expansion update only the managed block; repeated init preserves state',
    {observed:metadata(afterSlash),globalSettingsUnchanged:true,nextModelTurnContainsCurrentGuidance:true,sync});

  // Keep the already-bound host session alive: native calls can still succeed,
  // but removing the project package must revoke guidance maintenance immediately.
  await tool('tool_search',{query:'memory_search',limit:4});
  const projectSettingsFile=join(project,'.pi/settings.json'),originalSettings=await fs.readFile(projectSettingsFile);
  let changed,search;
  try {
    const removed={...projectSettings,packages:projectSettings.packages.filter(entry=>entry.source!==packed)};
    await fs.writeFile(projectSettingsFile,JSON.stringify(removed));
    changed=await projectStatus();assert.equal(changed.status,'changed');
    assert.ok(!metadata(afterSlash).tools.includes('mcp__mirket__memory_search'),'Negative case needs a previously unobserved substantive tool');
    search=await tool('mcp__mirket__memory_search',{...common,request:{query:'Disposable project setup integration fact',status:'both',
      context:{task_key:null,checkout_id:null,operation:null,at_time:null},limit:20,max_bytes:32768}});
    assert.ok(search.hits.some(hit=>hit.record.record_id===committed.record.record_id),'Successful native search did not retrieve the fixture fact');
    assert.equal(await guidance(),afterSlash,'Successful Mirket result changed guidance after project package removal');
    await fs.writeFile(join(root,'AGENTS-with-removed-wiring.md'),await guidance());
  } finally {
    await fs.writeFile(projectSettingsFile,originalSettings);
  }
  const restored=await projectStatus();assert.equal(restored.status,'ready');
  assert.equal(await guidance(),afterSlash,'Restoring project wiring changed source guidance');
  assert.equal(await sha(settingsFile),globalSettingsHash,'Project wiring failure changed global Pi settings');
  await save('removed project package changes sync status and blocks guidance updates after a successful new native tool; restoring exact settings recovers',
    {changed,restored,searchRecordId:committed.record.record_id,guidanceUnchanged:true,projectSettingsRestored:true,globalSettingsUnchanged:true});
} catch(error) {
  report.failure={message:error.message,stack:error.stack};throw error;
} finally {
  child.stdin.end();
  const normalExit=await Promise.race([exited,pause(5000)]);
  const before=members();
  const liveLeader=child.exitCode===null && child.signalCode===null && before.some(member=>member.pid===child.pid);
  const owned=!before.length || liveLeader || before.some(member=>member.command.includes(root));
  if(before.length && owned) {
    try{process.kill(-child.pid,'SIGTERM');}catch(error){if(error.code!=='ESRCH')throw error;}
    await Promise.race([exited,pause(1000)]);
    if(members().length) {try{process.kill(-child.pid,'SIGKILL');}catch(error){if(error.code!=='ESRCH')throw error;}}
    await Promise.race([exited,pause(2000)]);
  }
  const after=members();report.cleanup={before,after,refusedUnownedGroup:before.length>0&&!owned};
  lines.close();report.exit=normalExit;report.finishedAt=new Date().toISOString();report.cliElapsedMs=Date.now()-started;
  await fs.writeFile(join(root,'cli-events.jsonl'),events.map(event=>JSON.stringify(event)).join('\n')+'\n');
  await fs.writeFile(join(root,'cli.stderr'),stderr);await writeReport();
  console.log('Evidence:',evidence);
  assert.deepEqual(after,[],'Fixture CLI process group survived cleanup');
  assert.equal(report.cleanup.refusedUnownedGroup,false,'Refused cleanup of a changed process group');
  if(!report.failure)assert.equal(normalExit?.code,0,'Actual CLI did not exit gracefully after stdin closed');
}
