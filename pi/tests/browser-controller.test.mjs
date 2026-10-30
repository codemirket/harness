import test from 'node:test';
import assert from 'node:assert/strict';
import {createBrowserController, BROWSER_PREFIX} from '../dist/extension/browser.js';
import {mkdtempSync, mkdirSync, writeFileSync, rmSync, realpathSync} from 'node:fs';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
import {BROWSER_TOOLS} from '../dist/extension/browser-policy.js';

test('optional browser controller stays lazy and cannot grant from a model tool', async () => {
  const handlers = new Map(), effects = [];
  const pi = {on(name, handler) { handlers.set(name, handler); }, unregisterMcpServer(name) { effects.push(['unregister', name]); }, registerMcpServer() { throw Error('must not register'); }};
  const controller = createBrowserController(pi, () => { throw Error('ordinary startup must not inspect the runtime'); }, () => { throw Error('must not read user agent state'); });
  assert.equal(controller.status().status, 'inactive');
  assert.deepEqual(effects, []);
  const gate = handlers.get('tool_call');
  assert.equal(gate({toolName: 'read', input: {}}, {cwd: '/private/tmp'}), undefined);
  assert.deepEqual(gate({toolName: BROWSER_PREFIX + 'browser_snapshot', input: {}}, {cwd: '/private/tmp'}), {block: true, reason: 'Browser session has not been authorized'});
  await controller.revoke(); await controller.cancel();
  assert.deepEqual(effects, []);
  await assert.rejects(controller.open('https://example.org', {cwd: '/private/tmp'}), /literal/);
  assert.deepEqual(effects, []);
});

test('browser controller refuses exact and hyphen alias namespaces in global/project MCP settings and peer registrations', {skip: process.platform !== 'darwin' || process.arch !== 'arm64'}, async () => {
  const root = realpathSync(mkdtempSync('/private/tmp/mirket-browser-alias-'));
  const project = join(root, 'project'), agent = join(root, 'agent');
  mkdirSync(project, {mode: 0o700}); mkdirSync(agent, {mode: 0o700}); mkdirSync(join(project, '.pi'), {mode: 0o700});
  let registrations = [], registered = false;
  const pi = {on() {}, unregisterMcpServer() { throw Error('must preserve the other owner'); }, registerMcpServer() { registered = true; }, getMcpServers() { return registrations; }, getCommands() { return [{name: 'mcp', sourceInfo: {path: 'builtin:mcp'}}]; }};
  const runtime = {cwd: project, home: join(root, 'mirket'), binding: {releaseId: 'sha256:test'}};
  const controller = createBrowserController(pi, () => runtime, () => agent);
  try {
    for (const directory of [agent, join(project, '.pi')]) for (const name of ['mirket_browser', 'mirket-browser']) {
      const file = join(directory, 'mcp.json');
      writeFileSync(file, JSON.stringify({mcpServers: {[name]: {command: '/nonexistent'}}}), {mode: 0o600});
      await assert.rejects(controller.open('http://127.0.0.1:8123', {cwd: project}), /Same-namespace browser MCP override/);
      assert.equal(registered, false); rmSync(file);
    }
    registrations = [{name: 'mirket-browser', config: {command: '/nonexistent'}}];
    await assert.rejects(controller.open('http://127.0.0.1:8123', {cwd: project}), /namespace is already owned/);
    assert.equal(registered, false);
  } finally { rmSync(root, {recursive: true, force: true}); }
});

// Synthetic binding bytes and mocked MCP registration exercise the controller's
// real grant/result lifecycle. No browser, peer or Node payload is executed.
async function observationFixture(recordUse, maxCalls=4) {
  const root=realpathSync(mkdtempSync('/private/tmp/mirket-browser-observe-'));
  const project=join(root,'project'),agent=join(root,'agent'),home=join(agent,'mirket');
  for(const directory of [project,home])mkdirSync(directory,{recursive:true,mode:0o700});
  const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
  function binding(name,mode) {
    const treeRoot=join(root,name),file=join(treeRoot,'fixture');
    mkdirSync(treeRoot,{mode:0o700});writeFileSync(file,name,{mode});
    const sha256=hash(name),entries=[{path:'fixture',kind:'file',mode,sha256}];
    return {choice:'installed',path:file,sha256,version:'fixture',tree:{root:treeRoot,entries,identity:hash(JSON.stringify(entries))}};
  }
  const runtime={cwd:project,home,binding:{releaseId:'sha256:fixture',nodeSha256:hash('synthetic-node')}};
  const node=join(home,'releases/r-fixture/runtime/node/bin/node');
  mkdirSync(join(home,'releases/r-fixture/runtime/node/bin'),{recursive:true,mode:0o700});writeFileSync(node,'synthetic-node',{mode:0o700});
  writeFileSync(join(home,'runtime-tools.json'),JSON.stringify({schema:1,releaseId:runtime.binding.releaseId,tools:{'playwright-mcp':binding('peer',0o600),'browser-chromium':binding('browser',0o700)}}),{mode:0o600});
  const handlers=new Map(),warnings=[],uses=[],effects=[];
  let registrations=[];
  const tools=BROWSER_TOOLS.map(name=>({name:BROWSER_PREFIX+name,exposure:'direct',sourceInfo:{path:'builtin:mcp'}}));
  const pi={on(name,handler){handlers.set(name,handler);},getCommands:()=>[{name:'mcp',sourceInfo:{path:'builtin:mcp'}}],getMcpServers:()=>registrations,getAllTools:()=>tools,getActiveTools:()=>tools.map(tool=>tool.name),registerMcpServer(name,config){registrations.push({name,config});},unregisterMcpServer(name){effects.push(['unregister',name]);registrations=registrations.filter(server=>server.name!==name);}};
  const controller=createBrowserController(pi,()=>runtime,()=>agent,(value,name)=>{assert.equal(value,runtime);uses.push(name);return recordUse?.(value,name)??true;});
  const abort=new AbortController(),ctx={cwd:project,signal:abort.signal,ui:{notify:(message,level)=>warnings.push({message,level})}};
  async function call(name='browser_snapshot',id='call') {
    const event={toolName:BROWSER_PREFIX+name,toolCallId:id,input:{}};
    assert.equal(await handlers.get('tool_call')(event,ctx),undefined);
    return event;
  }
  const result=(event,extra={})=>handlers.get('tool_result')({...event,isError:false,content:[{type:'text',text:'fixture success'}],...extra},ctx);
  async function close(){try{await controller.revoke();}finally{rmSync(root,{recursive:true,force:true});}}
  try{await controller.open('http://127.0.0.1:8123',ctx,{maxCalls});}
  catch(error){await close();throw error;}
  return {controller,runtime,handlers,pi,ctx,abort,uses,warnings,effects,call,result,close};
}

const qualifiedPlatform={skip:process.platform!=='darwin'||process.arch!=='arm64'};
test('verified successful browser use records its exact ID before the last-call revoke',qualifiedPlatform,async()=>{
  let f;
  f=await observationFixture(()=>{assert.equal(f.pi.getMcpServers().length,1,'recording must precede budget revoke');return true;},1);
  try {
    const event=await f.call('browser_snapshot');
    assert.equal(await f.result(event),undefined);
    assert.deepEqual(f.uses,[BROWSER_PREFIX+'browser_snapshot']);
    assert.deepEqual(f.warnings,[]);assert.equal(f.controller.status().status,'inactive');
    assert.deepEqual(f.effects,[['unregister','mirket_browser']]);
  }finally{await f.close();}
});

test('failed, cancelled and stale browser results never record project use',qualifiedPlatform,async()=>{
  for(const cause of ['failure','cancel','revoke','unverified']) {
    const f=await observationFixture();try {
      const event=cause==='unverified'?{toolName:BROWSER_PREFIX+'browser_snapshot',toolCallId:'unverified',input:{}}:await f.call();
      if(cause==='cancel')f.abort.abort();
      if(cause==='revoke')await f.controller.revoke('fixture session changed');
      const result=await f.result(event,{isError:cause==='failure'});
      if(cause!=='failure')assert.equal(result.isError,true);
      assert.deepEqual(f.uses,[],cause);assert.deepEqual(f.warnings,[]);
    }finally{await f.close();}
  }
});

test('browser guidance callback failures warn without replacing successful results; disabled callback stays quiet',qualifiedPlatform,async()=>{
  for(const disabled of [false,true]) {
    const f=await observationFixture(()=>{if(!disabled)throw Error('managed block drift');return false;});try {
      const event=await f.call('browser_snapshot');
      assert.equal(await f.result(event),undefined);
      assert.deepEqual(f.uses,[BROWSER_PREFIX+'browser_snapshot']);
      assert.equal(f.controller.status().status,'authorized');
      assert.deepEqual(f.warnings,disabled?[]:[{message:'Project guidance sync failed: managed block drift',level:'warning'}]);
    }finally{await f.close();}
  }
});
