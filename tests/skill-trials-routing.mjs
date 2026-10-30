// Focused paired routing evidence, not a model-utility benchmark.
// Run using explicit private Node: FILE PEERS PACKAGE [EXACT_PEER_SOURCE CHROME_UNPACK_ROOT [CASE_ID [SHARED_PRIVATE_AGENT_DIR]]].
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {readFileSync} from 'node:fs';
import http from 'node:http';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {join, resolve, dirname, isAbsolute} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';
import {makeFixtureProvider} from './pi-offline-provider.mjs';

const [peers, packed, peerSource, chromeRoot, onlyCase, sharedAgent] = process.argv.slice(2);
assert.ok([peers, packed, process.execPath].every(isAbsolute));
assert.ok(!peerSource || [peerSource, chromeRoot].every(isAbsolute));
assert.ok(!sharedAgent || (isAbsolute(sharedAgent)&&sharedAgent.startsWith('/private/tmp/mirket-skill-trials-routing-')),
  'An explicitly shared installation must belong to a disposable routing trial, never live state');
const repository = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const root = await fs.mkdtemp('/private/tmp/mirket-skill-trials-routing-');
await fs.chmod(root, 0o700);
console.log('Evidence:', root);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const fileSha = async path => sha(await fs.readFile(path));
const binding = JSON.parse(await fs.readFile(join(packed, 'native-binding.json')));
assert.equal(await fileSha(process.execPath), binding.nodeSha256);
const report = {root, startedAt:new Date().toISOString(), identity:{platform:process.platform,
  architecture:process.arch, node:process.version, nodePath:process.execPath,
  nodeSha256:binding.nodeSha256, piVersion:JSON.parse(await fs.readFile(join(peers,
    '@earendil-works/pi-coding-agent/package.json'))).version,
  platformVersion:execFileSync('/usr/bin/sw_vers', {encoding:'utf8'}), release:binding.releaseId,
  packageInventorySha256:await fileSha(join(packed,'package-inventory.json')),
  runnerSha256:await fileSha(fileURLToPath(import.meta.url)),
  sourceCheckerSha256:await fileSha(join(repository,'scripts/check-video-source.mjs')),
  sourceAdapterSha256:await fileSha(join(repository,'pi/dist/runtime/video-export.js'))},
  method:{provider:'offline-fixture', model:'deterministic', effort:'off',
    nativeInstallation:sharedAgent?{kind:'explicit shared private installation',agentDir:sharedAgent,
      scope:'separate HOME, project, project settings and sessions for each arm'}:{kind:'fresh installation per native arm'},
    utility:'unavailable: no authorized model credentials; scripted tool calls do not measure model judgment',
    matching:'same case brief, fixture bytes, provider, tools and budget; selected arm invokes its exact skill body',
    limits:['No live model, token benefit, visual quality or human acceptance claim.',
      'Browser fixture grant is test-only policy/MCP evidence, not a Pi TUI authorization qualification.',
      'All 305 catalog cases remain planned.']}, cases:[], failures:[]};
const save = () => fs.writeFile(join(root,'report.json'), JSON.stringify(report,null,2)+'\n');
await save();

// Set private process state before importing Pi; each arm replaces all paths again.
process.env.HOME = join(root,'bootstrap-home');
process.env.PI_CODING_AGENT_DIR = join(root,'bootstrap-agent');
process.env.PI_CODING_AGENT_SESSION_DIR = join(root,'bootstrap-sessions');
process.env.TMPDIR = join(root,'bootstrap-tmp');
await fs.mkdir(process.env.TMPDIR,{mode:0o700});
process.env.PI_SKIP_UPDATE_CHECK = '1';
process.env.PI_TELEMETRY_DISABLED = '1';
process.env.PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = '1';
for (const key of Object.keys(process.env)) if (/^(OPENAI|ANTHROPIC|GEMINI|GOOGLE|AZURE|AWS|PI_API|MIRKET_)/.test(key)) delete process.env[key];
const sdk = await import(pathToFileURL(join(peers,'@earendil-works/pi-coding-agent/dist/index.js')));
const ai = await import(pathToFileURL(join(peers,'@earendil-works/pi-ai/dist/index.js')));
const policyModule = await import(pathToFileURL(join(packed,'dist/extension/browser-policy.js')));
const profiles = await import(pathToFileURL(join(packed,'dist/extension/profiles.js')));
const catalog = JSON.parse(await fs.readFile(join(packed,'skills/catalog.json')));
const {createVideoSeed} = await import(pathToFileURL(join(repository,'scripts/create-video-seed.mjs')));
const {checkVideoSource} = await import(pathToFileURL(join(repository,'scripts/check-video-source.mjs')));
const font = join(packed,'native/runtime/design/fonts/NotoSans.ttf');
const fontNotice = join(repository,'packaging/notices/NotoSans-OFL.txt');

let peerPath, browserPath, peerFiles=[];
if (peerSource) {
  const pin = JSON.parse(await fs.readFile(join(repository,'packaging/runtime/playwright-mcp-external.json')));
  const chrome = JSON.parse(await fs.readFile(join(repository,'packaging/runtime/chrome-macos.json')));
  const payload = join(root,'external-peer');
  for (const file of pin.files) {
    const bytes = await fs.readFile(join(peerSource,file.path));
    assert.equal(sha(bytes),file.sha256);
    await fs.mkdir(dirname(join(payload,file.path)),{recursive:true,mode:0o700});
    await fs.writeFile(join(payload,file.path),bytes,{mode:0o600});
    peerFiles.push({path:join(payload,file.path),sha256:file.sha256});
  }
  const chromePath = path => join(chromeRoot,path.replace(/^runtime\/browser\//,''));
  for (const file of chrome.files) assert.equal(await fileSha(chromePath(file.path)),file.sha256);
  for (const link of chrome.links) assert.equal(await fs.readlink(chromePath(link.relative_path)),link.target);
  peerPath = join(payload,pin.executable); browserPath = chromePath(chrome.executable);
  report.externalBrowser = {peerPath, browserPath, peerPinSha256:await fileSha(join(repository,
    'packaging/runtime/playwright-mcp-external.json')), browserPinSha256:await fileSha(join(repository,
    'packaging/runtime/chrome-macos.json')), peerSha256:await fileSha(peerPath),
    browserSha256:await fileSha(browserPath), verification:'all selected peer files and all Chrome files/links matched retained pins'};
}

const cases = [
  {id:'source-only-hyperframes', skill:'video-production', brief:'Hand off editable local HyperFrames source with a local font and editable captions. Do not render or claim playback proof.'},
  {id:'public-research-no-route', skill:'browser-research', native:true, brief:'Research public official sources with citations; report a missing configured research route truthfully.'},
  {id:'local-browser-viewport', skill:'browser-research', browser:true, brief:'Inspect the local page at wide and narrow viewport sizes; save viewport PNGs and report observed layout.'},
  {id:'account-form-outside-peer', skill:'browser-automation', brief:'Enter account form text on a public site using the local peer; report the unavailable text-entry/public route.'},
  {id:'missing-raster', skill:'raster-art', native:true, brief:'Resize a supplied raster image; report the missing design.raster capability without installing a substitute.'},
  {id:'inherited-autoload-exclusion', skill:'browser-research', inherited:true, brief:'Select library-cli under autoload:false and exclude globally inherited conditional browser/raster skills.'},
];

assert.ok(!onlyCase || cases.some(spec=>spec.id===onlyCase),'Unknown case ID');
for (const spec of cases.filter(spec=>!onlyCase || spec.id===onlyCase)) {
  const caseReport = {id:spec.id, brief:spec.brief, utility:'unavailable', arms:[]};
  report.cases.push(caseReport); await save();
  for (const selected of [false,true]) {
    const arm = {selected, status:'running', startedAt:new Date().toISOString(), checks:[], contexts:[]};
    caseReport.arms.push(arm); await save();
    const work = join(root,`${spec.id}-${selected?'selected':'baseline'}`);
    const home=join(work,'home'),agentDir=spec.native&&sharedAgent?sharedAgent:join(work,'agent'),cwd=join(work,'project'),sessions=join(work,'sessions'),tmp=join(work,'tmp');
    for (const directory of [home,agentDir,cwd,sessions,tmp]) await fs.mkdir(directory,{recursive:true,mode:0o700});
    Object.assign(process.env,{HOME:home,TMPDIR:tmp,PI_CODING_AGENT_DIR:agentDir,PI_CODING_AGENT_SESSION_DIR:sessions});
    let session, server, sockets;
    const contexts=[];
    try {
      if (spec.native&&!sharedAgent) {
        const installation = JSON.parse(execFileSync(process.execPath,[join(packed,'scripts/setup.mjs'),'--agent-dir',agentDir],
          {env:process.env,cwd,encoding:'utf8',timeout:660000,maxBuffer:4*1024*1024}));
        assert.equal(installation.releaseId,binding.releaseId); arm.installation=installation;
      }
      const globalSettings = {enableInstallTelemetry:false,defaultProvider:'offline-fixture',defaultModel:'deterministic',defaultTools:['+codemode','+tool_search'],
        packages:[{source:packed,extensions:spec.native?undefined:[],skills:selected?[`skills/${spec.skill}/SKILL.md`]:[]}]};
      if(spec.inherited) globalSettings.packages[0].skills=['skills/browser-research/SKILL.md','skills/raster-art/SKILL.md'];
      if(spec.native&&sharedAgent) {
        arm.sharedPrivateAgentDir=sharedAgent;arm.sharedGlobalSettingsSha256=await fileSha(join(agentDir,'settings.json'));
        await fs.mkdir(join(cwd,'.pi'),{mode:0o700});
        await fs.writeFile(join(cwd,'.pi/settings.json'),JSON.stringify(globalSettings));
      } else await fs.writeFile(join(agentDir,'settings.json'),JSON.stringify(globalSettings));
      if(spec.inherited) {
        await fs.mkdir(join(cwd,'.pi'),{mode:0o700});
        await fs.writeFile(join(cwd,'.pi/settings.json'),JSON.stringify({theme:'preserve-routing-trial',
          packages:[{source:packed,autoload:false,extensions:[],skills:[]}]}));
        if(selected) profiles.applyProfile(profiles.prepareProfile(packed,join(cwd,'.pi/settings.json'),['library-cli'],[]));
      }
      const fixture=pi=>makeFixtureProvider(ai)({registerProvider(name,config){pi.registerProvider(name,{...config,
        streamSimple(model,context,...rest){
          contexts.push(structuredClone(context));
          const index=context.messages.findLastIndex(message=>message.role==='user'),user=context.messages[index];
          const text=typeof user.content==='string'?user.content:user.content.filter(p=>p.type==='text').map(p=>p.text).join('');
          if(text.startsWith('<skill ')&&text.includes('</skill>')) {
            const messages=[...context.messages];messages[index]={...user,content:text.slice(text.indexOf('</skill>')+8).trim()};
            context={...context,messages};
          }
          return config.streamSimple(model,context,...rest);
        }});}});
      const factories=[{name:'codemode',factory:sdk.createCodemodeExtension(),builtin:true,replaceable:true},
        {name:'tool-search',factory:sdk.createToolSearchExtension(),builtin:true,replaceable:true},
        {name:'mcp',factory:sdk.createMcpExtension(),builtin:true,replaceable:true},fixture];
      let origin, policy;
      if(spec.browser && peerPath) {
        sockets=await fs.mkdtemp('/private/tmp/mirket-st-');await fs.chmod(sockets,0o700);arm.socketDirectory=sockets;
        const html='<!doctype html><meta charset="utf-8"><title>Held-out viewport</title><style>body{font:20px sans-serif}.cards{display:grid;grid-template-columns:1fr 1fr}@media(max-width:480px){.cards{grid-template-columns:1fr}}</style><h1>Local viewport İğüş</h1><div class="cards"><p>First card</p><p>Second card</p></div><p id="state"></p><script>function size(){document.querySelector("#state").textContent=(innerWidth<=480?"Narrow":"Wide")+" layout; motion="+(matchMedia("(prefers-reduced-motion: reduce)").matches?"reduced":"normal")}size();addEventListener("resize",size)</script>';
        server=http.createServer((req,res)=>res.writeHead(200,{'content-type':'text/html'}).end(html));
        await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));origin=`http://127.0.0.1:${server.address().port}`;
        const output=join(work,'output');await fs.mkdir(output,{mode:0o700});
        const config=join(work,'browser.json');await fs.writeFile(config,JSON.stringify({browser:{browserName:'chromium',isolated:true,
          launchOptions:{executablePath:browserPath,headless:true},contextOptions:{viewport:{width:1000,height:700},reducedMotion:'reduce',serviceWorkers:'block'}},
          network:{allowedOrigins:[origin]},outputDir:output,timeouts:{action:5000,navigation:10000}}));
        const grant={project:cwd,release:binding.releaseId,peerPath,browserPath,peerIdentity:report.externalBrowser.peerSha256,
          browserIdentity:report.externalBrowser.browserSha256,origin,output,expires:Date.now()+300000,maxCalls:64,methods:['GET','HEAD'],tools:policyModule.BROWSER_TOOLS};
        policy=policyModule.createBrowserPolicy(grant,()=>{
          for(const file of peerFiles)assert.equal(sha(readFileSync(file.path)),file.sha256);
          assert.equal(sha(readFileSync(browserPath)),report.externalBrowser.browserSha256);
        });
        factories.push(pi=>{
          pi.on('session_start',()=>pi.registerMcpServer('routing_browser',{command:process.execPath,args:[peerPath,'--config',config,'--no-webmcp','--block-service-workers'],cwd,
            env:{HOME:home,TMPDIR:tmp,PWTEST_SOCKETS_DIR:sockets,PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD:'1'},exposure:'hidden',toolExposure:Object.fromEntries(policyModule.BROWSER_TOOLS.map(name=>[name,'direct'])),timeout:20}));
          pi.on('tool_call',event=>{if(!event.toolName.startsWith('mcp__routing_browser__'))return;
            try {Object.assign(event.input,policy.check(event.toolName.slice('mcp__routing_browser__'.length),event.input,{project:cwd,release:binding.releaseId}));}
            catch(error){return {block:true,reason:String(error)};}
          });
        });
      }
      const manager=sdk.SettingsManager.create(cwd,agentDir),loader=new sdk.DefaultResourceLoader({cwd,agentDir,settingsManager:manager,noContextFiles:true,extensionFactories:factories});
      await loader.reload();assert.deepEqual(loader.getExtensions().errors,[]);
      ({session}=await sdk.createAgentSession({cwd,agentDir,settingsManager:manager,resourceLoader:loader,thinkingLevel:'off',sessionManager:sdk.SessionManager.create(cwd,sessions)}));
      const errors=[];await session.bindExtensions({mode:'print',uiContext:{notify:()=>{}},onError:error=>errors.push(typeof error==='string'?error:JSON.stringify(error))});
      await session.setModel(session.modelRuntime.getModel('offline-fixture','deterministic'));
      arm.skills=loader.getSkills().skills.map(skill=>skill.name).sort();arm.tools=session.getAllTools().map(tool=>tool.name).sort();
      if(!spec.inherited) assert.deepEqual(arm.skills,selected?[spec.skill]:[]);
      async function call(tool,args={},useBody=false) {
        const request=JSON.stringify({brief:spec.brief,tool,args});const before=session.messages.length;
        await session.prompt(selected&&useBody?`/skill:${spec.skill} ${request}`:request);
        const result=session.messages.slice(before).findLast(message=>message.role==='toolResult'&&message.toolName===tool);
        assert.ok(result,JSON.stringify({tool,errors}));return result;
      }
      const fixturePath=join(cwd,'case.json');await fs.writeFile(fixturePath,JSON.stringify({brief:spec.brief,limits:report.method.limits})+'\n');
      assert.equal(!!(await call('read',{path:fixturePath},!spec.inherited)).isError,false);
      if(spec.id==='source-only-hyperframes') {
        const source=join(cwd,'handoff');await createVideoSeed({destination:source,font,fontNotice,withCaptions:true});
        const valid=await checkVideoSource({project:source,manifest:join(source,'video-manifest.json')});
        assert.equal(valid.status,'valid',JSON.stringify(valid));
        const refusedSource=join(cwd,'handoff-missing-root');await fs.cp(source,refusedSource,{recursive:true});
        const manifest=JSON.parse(await fs.readFile(join(refusedSource,'video-manifest.json')));
        const htmlPath=join(refusedSource,'index.html'),html=await fs.readFile(htmlPath,'utf8');
        await fs.writeFile(htmlPath,html.replace(' data-no-timeline',''));
        const entry=manifest.sources.find(item=>item.path==='index.html');entry.revision='sha256:'+await fileSha(htmlPath);entry.bytes=(await fs.stat(htmlPath)).size;
        await fs.writeFile(join(refusedSource,'video-manifest.json'),JSON.stringify(manifest));
        const invalid=await checkVideoSource({project:refusedSource,manifest:join(refusedSource,'video-manifest.json')});
        assert.notEqual(invalid.status,'valid');
        arm.sourceOracle={valid,invalid,validProject:source,refusedProject:refusedSource,producer:'scripted existing createVideoSeed, not model output',coverage:'root/font/assets/captions static contract only'};
        arm.checks.push('Source-only fixture passes static oracle; missing literal root marker refuses. No rendering capability required by selected guidance.');
      } else if(spec.id==='public-research-no-route') {
        await session.prompt('/mirket-research '+spec.brief);
        assert.ok(errors.some(error=>error.includes('Research is qualified only')),JSON.stringify(errors));
        assert.ok(!session.getAllTools().some(tool=>/web_search|browser_navigate/.test(tool.name)));
        arm.checks.push('Actual installed research command refuses unconfigured offline provider; no public browser/search tool exists.');
      } else if(spec.id==='local-browser-viewport') {
        if(!peerPath) arm.checks.push('Actual viewport capture untested: exact external peer/browser paths were not supplied.');
        else {
          const prefix='mcp__routing_browser__';
          assert.equal(!!(await call(prefix+'browser_navigate',{url:origin})).isError,false);
          assert.equal(!!(await call(prefix+'browser_wait_for',{text:'Wide layout; motion=reduced'})).isError,false);
          const wide=await call(prefix+'browser_snapshot');assert.equal(!!wide.isError,false);assert.match(JSON.stringify(wide),/Wide layout; motion=reduced/);
          const screen=await call(prefix+'browser_take_screenshot',{type:'png',scale:'css',filename:'wide.png',fullPage:false});assert.equal(!!screen.isError,false);
          assert.equal(!!(await call(prefix+'browser_resize',{width:360,height:740})).isError,false);
          assert.equal(!!(await call(prefix+'browser_wait_for',{text:'Narrow layout; motion=reduced'})).isError,false);
          const narrow=await call(prefix+'browser_snapshot');assert.equal(!!narrow.isError,false);assert.match(JSON.stringify(narrow),/Narrow layout; motion=reduced/);
          assert.equal(!!(await call(prefix+'browser_take_screenshot',{type:'png',scale:'css',filename:'narrow.png',fullPage:false})).isError,false);
          const pngs=[];for(const name of ['wide.png','narrow.png']){const bytes=await fs.readFile(join(work,'output',name));assert.equal(bytes.subarray(1,4).toString(),'PNG');pngs.push({name,width:bytes.readUInt32BE(16),height:bytes.readUInt32BE(20),sha256:sha(bytes)});}
          assert.deepEqual(pngs.map(({width,height})=>[width,height]),[[1000,700],[360,740]]);arm.pngs=pngs;
          assert.equal(!!(await call(prefix+'browser_navigate',{url:'https://example.org/'})).isError,true);
          assert.equal(!!(await call(prefix+'browser_type',{element:'Account',ref:'e1',text:'synthetic'})).isError,true);
          assert.equal(!!(await call(prefix+'browser_take_screenshot',{type:'png',scale:'css',fullPage:true})).isError,true);
          await call(prefix+'browser_close');
          arm.checks.push('Actual Pi MCP wide/narrow snapshots and viewport PNG dimensions pass; public navigation, text-entry tool and full-page capture refuse.');
        }
      } else if(spec.id==='account-form-outside-peer') {
        const context={project:cwd,release:binding.releaseId};
        const grant={...context,origin:'http://127.0.0.1:8123',output:join(work,'output'),peerPath:peerPath??join(work,'peer.js'),browserPath:browserPath??join(work,'chrome'),peerIdentity:'test',browserIdentity:'test',expires:Date.now()+300000,maxCalls:64,methods:['GET','HEAD'],tools:policyModule.BROWSER_TOOLS};
        const scoped=policyModule.createBrowserPolicy(grant,()=>{}),denials=[];
        for(const [tool,args] of [['browser_type',{text:'synthetic'}],['browser_fill_form',{fields:[]}],['browser_press_key',{key:'synthetic text'}],['browser_navigate',{url:'https://example.org/account'}]]) {
          assert.throws(()=>scoped.check(tool,args,context));denials.push({tool,args});
        }
        assert.equal(scoped.network('POST',grant.origin+'/account'),false);arm.policyDenials=denials;
        arm.checks.push('Exact packaged seven-tool policy refuses public navigation, form/text entry and POST. No account state was used.');
      } else if(spec.id==='missing-raster') {
        const doctor=await call('mirket_doctor');assert.equal(!!doctor.isError,false);
        const parsed=JSON.parse(doctor.content.filter(part=>part.type==='text').map(part=>part.text).join('\n'));
        assert.equal(parsed.status,'ready');assert.equal(parsed.skillReadiness.status,selected?'missing':'ready');
        assert.deepEqual(parsed.skillReadiness.unavailableSkills,selected?[{skill:'raster-art',missing:['design.raster']}]:[]);
        arm.doctor=parsed;arm.checks.push('Actual installed doctor preserves runtime ready; only selected raster guidance reports missing design.raster.');
      } else if(spec.inherited) {
        if(selected) {assert.ok(!arm.skills.includes('browser-research'));assert.ok(!arm.skills.includes('raster-art'));
          assert.deepEqual(arm.skills,[...new Set([...catalog.global.map(skill=>skill.id),...catalog.profiles['library-cli'].skills])].sort());}
        else assert.deepEqual(arm.skills,['browser-research','raster-art']);
        const settings=JSON.parse(await fs.readFile(join(cwd,'.pi/settings.json')));assert.equal(settings.theme,'preserve-routing-trial');arm.projectSettings=settings;
        arm.checks.push(selected?'Actual Pi resolver excludes both inherited conditional skills after explicit profile selection.':'Actual Pi resolver reproduces inherited conditional skills before explicit profile selection.');
      }
      await fs.writeFile(join(work,'contexts.json'),JSON.stringify(contexts,null,2));
      await fs.writeFile(join(work,'messages.json'),JSON.stringify(session.messages,null,2));
      arm.contexts=contexts.map(context=>({bytes:Buffer.byteLength(JSON.stringify(context)),sha256:sha(JSON.stringify(context))}));
      arm.contextBytes=arm.contexts[0]?.bytes;arm.contextPath=join(work,'contexts.json');arm.errors=errors;
      if(spec.id!=='public-research-no-route') assert.deepEqual(errors,[]);
      arm.status=spec.browser&&!peerPath?'partial':'passed';arm.elapsedMs=Date.now()-Date.parse(arm.startedAt);
      console.log(arm.status.toUpperCase(),spec.id,selected?'selected':'baseline');
    } catch(error) {
      arm.status='failed';arm.error=String(error?.stack??error);report.failures.push({id:spec.id,selected,error:arm.error});
      console.log('FAIL',spec.id,selected?'selected':'baseline',String(error));
    } finally {
      if(session){
        await fs.writeFile(join(work,'messages.json'),JSON.stringify(session.messages,null,2));
        await fs.writeFile(join(work,'contexts.json'),JSON.stringify(contexts,null,2));
        await session.extensionRunner.emit({type:'session_shutdown',reason:'quit'});session.dispose();
      }
      if(spec.native&&sharedAgent) assert.equal(await fileSha(join(agentDir,'settings.json')),arm.sharedGlobalSettingsSha256,'Shared global settings changed');
      if(server){server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
      if(sockets) await fs.rm(sockets,{recursive:true,force:true});
      await save();
    }
  }
  caseReport.contextDeltaBytes=caseReport.arms[1].contextBytes-caseReport.arms[0].contextBytes;
  caseReport.status=caseReport.arms.some(arm=>arm.status==='failed')?'failed':caseReport.arms.every(arm=>arm.status==='passed')?'passed':'partial';await save();
}
report.finishedAt=new Date().toISOString();report.status=report.failures.length?'failed':report.cases.every(row=>row.status==='passed')?'passed':'partial';await save();
console.log(JSON.stringify({status:report.status,evidence:join(root,'report.json'),cases:report.cases.length,arms:report.cases.flatMap(row=>row.arms).length,failures:report.failures.length}));
if(report.failures.length)process.exitCode=1;
