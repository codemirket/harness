// Run domain outcomes against an already installed disposable full package.
import assert from 'node:assert/strict';import fs from 'node:fs/promises';import {join,isAbsolute} from 'node:path';import {pathToFileURL} from 'node:url';
import {makeFixtureProvider} from './pi-offline-provider.mjs';import {domainJourneys} from './pi-domain-journeys.mjs';
import {videoJourney} from './pi-video-journey.mjs';
import {memoryJourney} from './pi-memory-journey.mjs';
import {editableJourney} from './pi-editable-journey.mjs';
import {seedTuiAuthorization} from './pi-tui-auth-seed.mjs';
const [peers,root,packageName='package']=process.argv.slice(2);assert.ok(isAbsolute(peers)&&root.startsWith('/private/tmp/mirket-pi-package-'));assert.ok(['package','package-verified'].includes(packageName));
const cwd=process.env.MIRKET_PI_MEMORY_TEST==='1'?join(root,'memory-project-'+Date.now()):join(root,'project'),agentDir=join(root,'agent'),packed=join(root,packageName);process.env.HOME=join(root,'home');process.env.PI_CODING_AGENT_DIR=agentDir;process.env.PI_CODING_AGENT_SESSION_DIR=join(root,'sessions');process.env.PI_SKIP_UPDATE_CHECK='1';process.env.PI_TELEMETRY_DISABLED='1';
await fs.mkdir(cwd,{recursive:true,mode:0o700});
const sdk=await import(pathToFileURL(join(peers,'@earendil-works/pi-coding-agent/dist/index.js')));const ai=await import(pathToFileURL(join(peers,'@earendil-works/pi-ai/dist/index.js')));
const settingsFile=join(agentDir,'settings.json'),settings=JSON.parse(await fs.readFile(settingsFile));settings.defaultProvider='offline-fixture';settings.defaultModel='deterministic';settings.defaultTools=['+codemode','+tool_search'];settings.enableInstallTelemetry=false;await fs.writeFile(settingsFile,JSON.stringify(settings));
const settingsManager=sdk.SettingsManager.create(cwd,agentDir);const loader=new sdk.DefaultResourceLoader({cwd,agentDir,settingsManager,noContextFiles:true,extensionFactories:[{name:'codemode',factory:sdk.createCodemodeExtension(),builtin:true,replaceable:true},{name:'tool-search',factory:sdk.createToolSearchExtension(),builtin:true,replaceable:true},{name:'mcp',factory:sdk.createMcpExtension(),builtin:true,replaceable:true},makeFixtureProvider(ai)]});await loader.reload();assert.deepEqual(loader.getExtensions().errors,[]);
assert.ok(loader.getExtensions().extensions.some(extension=>extension.path===join(packed,'dist/extension/index.js')),'Expected assembled extension was not actually loaded');
const {session}=await sdk.createAgentSession({cwd,agentDir,settingsManager,resourceLoader:loader,sessionManager:sdk.SessionManager.create(cwd,join(root,'sessions'))});const e={session};const errors=[],notifications=[];
const report={root,package:packed,results:[],errors,notifications};const save=async name=>{report.results.push(name);await fs.writeFile(join(root,'domain-session-report.json'),JSON.stringify(report,null,2));console.log('PASS',name)};
async function call(entry,tool,args={}){const at=entry.session.messages.length;await entry.session.prompt(JSON.stringify({tool,args}));const result=entry.session.messages.slice(at).findLast(m=>m.role==='toolResult'&&m.toolName===tool);assert.ok(result,JSON.stringify(errors));assert.equal(result.isError,false,JSON.stringify(result));const text=result.content.filter(c=>c.type==='text').map(c=>c.text).join('\n');try{return JSON.parse(text)}catch{return text}}
try{
 await session.bindExtensions({mode:'print',uiContext:{notify:(text,type)=>notifications.push({text,type})},onError:error=>errors.push(error)});await session.setModel(session.modelRuntime.getModel('offline-fixture','deterministic'));
 const doctor=await call(e,'mirket_doctor');assert.equal(doctor.status,'ready',JSON.stringify(doctor));const release=doctor.releaseId;report.releaseId=release;await save('full installed package and actual pi runtime identity');
 if(process.env.MIRKET_PI_TUI_AUTH_SEED==='1')await seedTuiAuthorization({session:e,call,cwd,release,root});
 else if(process.env.MIRKET_PI_EDITABLE_TEST==='1')await editableJourney({session:e,call,save,cwd,release,root});
 else if(process.env.MIRKET_PI_MEMORY_TEST==='1')await memoryJourney({session:e,call,save,cwd,release,root});
 else if(process.env.MIRKET_PI_VIDEO_FIXTURE)await videoJourney({session:e,call,save,cwd,release,root,fixture:process.env.MIRKET_PI_VIDEO_FIXTURE});
 else await domainJourneys({session:e,call,save,cwd,release,root});
}finally{await session.extensionRunner.emit({type:'session_shutdown',reason:'quit'});session.dispose();await fs.writeFile(join(root,'domain-session-report.json'),JSON.stringify(report,null,2));}
