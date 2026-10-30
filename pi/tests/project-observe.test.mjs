import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdtempSync, realpathSync, mkdirSync, writeFileSync, rmSync, symlinkSync} from 'node:fs';
import {join, relative} from 'node:path';
import {pathToFileURL} from 'node:url';
import {registerProjectObservation} from '../dist/extension/project-observe.js';

function fixture(record) {
  const root=realpathSync(mkdtempSync('/private/tmp/mirket-project-observe-'));
  const pkg=join(root,'package'), project=join(root,'project'), home=join(root,'agent/mirket');
  for (const directory of [pkg,project,home,join(pkg,'skills/fixture'),join(pkg,'skills/second'),join(pkg,'dist/extension')]) mkdirSync(directory,{recursive:true,mode:0o700});
  const skill=join(pkg,'skills/fixture/SKILL.md'), second=join(pkg,'skills/second/SKILL.md');
  const body='Use a bounded fixture and inspect its result.';
  const source=`---\nname: fixture\ndescription: Fixture guidance\n---\n\n${body}\n`;
  writeFileSync(skill,source);writeFileSync(second,source.replace('fixture','second'));
  writeFileSync(join(pkg,'dist/extension/index.js'),'// fixture owner');
  const entry=(id,path,bytes)=>({id,path,source_bytes:Buffer.byteLength(bytes),source_sha256:createHash('sha256').update(bytes).digest('hex')});
  writeFileSync(join(pkg,'skills/catalog.json'),JSON.stringify({global:[entry('fixture','skills/fixture/SKILL.md',source)],project:[entry('second','skills/second/SKILL.md',source.replace('fixture','second'))]}));
  const runtime={cwd:project,home,command:join(home,'bin/mirket'),binding:{releaseId:'sha256:fixture'}};
  let selectedRuntime=runtime, idle=true, signal;
  const tools=[{name:'read',sourceInfo:{path:'builtin:read'}},{name:'mcp__mirket__code_inspect',sourceInfo:{path:'builtin:mcp'}},{name:'mcp__mirket__memory_search',sourceInfo:{path:'builtin:mcp'}},{name:'mirket_delegate',sourceInfo:{path:join(pkg,'dist/extension/index.js')}}];
  const commands=[{name:'skill:fixture',source:'skill',sourceInfo:{path:skill}}];
  const servers=[{name:'mirket',config:{command:runtime.command,cwd:project,args:['serve','--home',home,'--expected-release-id',runtime.binding.releaseId]}}];
  const handlers=new Map(), uses=[], warnings=[];
  const pi={on(name,handler){const list=handlers.get(name)??[];list.push(handler);handlers.set(name,list);},getCommands:()=>commands,getAllTools:()=>tools,getMcpServers:()=>servers};
  const ctx={cwd:project,isIdle:()=>idle,get signal(){return signal;},ui:{notify:(message,level)=>warnings.push({message,level})}};
  registerProjectObservation(pi,pkg,()=>selectedRuntime,(value,use)=>{assert.equal(value,runtime);if(record)return record(value,use);uses.push(use);return true;});
  async function emit(name,event={}) {for(const handler of handlers.get(name)??[])assert.equal(await handler({type:name,...event},ctx),undefined,'Observation changed the host event/result');}
  const result=(toolName,input={},extra={})=>emit('tool_result',{toolName,toolCallId:'fixture-call',input,isError:false,content:[{type:'text',text:'success'}],...extra});
  const expanded=(id='fixture',args='')=>`<skill name="${id}" location="${join(pkg,'skills',id,'SKILL.md')}">\nReferences are relative to ${join(pkg,'skills',id)}.\n\n${body}\n</skill>${args?'\n\n'+args:''}`;
  const message=text=>emit('message_end',{message:{role:'user',content:[{type:'text',text}],timestamp:1}});
  return {root,pkg,project,home,skill,second,source,runtime,commands,tools,servers,handlers,uses,warnings,ctx,emit,result,expanded,message,setIdle:value=>{idle=value;},setRuntime:value=>{selectedRuntime=value;},setSignal:value=>{signal=value;},close:()=>rmSync(root,{recursive:true,force:true})};
}

test('startup, discovery, errors, host tools and administrative tools do not record use',async()=>{
  const f=fixture();try {
    await f.emit('session_start');
    await f.emit('before_agent_start',{messageType:'user',prompt:'ordinary task'});
    for(const name of ['mcp__mirket__runtime_status','mcp__mirket__memory_status','mcp__mirket__job_status','mcp__mirket__job_cancel','mcp__mirket__video_status','mcp__mirket__video_cancel','mirket_doctor','mirket_task_status','tool_search','bash']) {
      f.tools.push({name,sourceInfo:{path:'builtin:mcp'}});await f.result(name,{command:`cat ${f.skill}`});
    }
    await f.result('read',{path:f.skill},{isError:true});
    await f.result('mcp__mirket__code_inspect',{}, {isError:true});
    await f.message(f.expanded());
    assert.deepEqual(f.uses,[]);assert.deepEqual(f.warnings,[]);
  }finally{f.close();}
});

test('successful selected built-in skill reads record one ID with source-bound ownership',async()=>{
  const f=fixture();try {
    await f.result('read',{path:'@'+f.skill});
    await f.result('read',{path:relative(f.project,f.skill)});
    await f.result('read',{path:pathToFileURL(f.skill).href},{parentToolCallId:'outer'});
    assert.deepEqual(f.uses,[{kind:'skill',id:'fixture'}]);
    await f.emit('session_start');
    f.commands.push({name:'skill:second',source:'skill',sourceInfo:{path:f.second}});
    await f.result('read',{path:f.second});
    assert.deepEqual(f.uses.at(-1),{kind:'skill',id:'second'});
  }finally{f.close();}
});

test('unselected, colliding, replaced, changed and aliased skill resources are refused',async()=>{
  const f=fixture();try {
    await f.result('read',{path:f.second});
    f.tools[0].sourceInfo.path='/foreign/read.js';await f.result('read',{path:f.skill});f.tools[0].sourceInfo.path='builtin:read';
    f.commands.push({...f.commands[0],source:'extension'});await f.result('read',{path:f.skill});f.commands.pop();
    const prefixCollision=f.pkg+'-foreign/skills/fixture/SKILL.md';f.commands[0].sourceInfo.path=prefixCollision;await f.result('read',{path:prefixCollision});f.commands[0].sourceInfo.path=f.skill;
    writeFileSync(f.skill,f.source+'Changed instruction.');await f.result('read',{path:f.skill});writeFileSync(f.skill,f.source);
    const saved=join(f.root,'aliased-body');writeFileSync(saved,f.source);rmSync(f.skill);symlinkSync(saved,f.skill);await f.result('read',{path:f.skill});
    assert.deepEqual(f.uses,[]);
  }finally{f.close();}
});

test('slash activation waits for the exact host-expanded user message, including queued inputs',async()=>{
  const f=fixture();try {
    await f.emit('input',{text:'/skill:fixture perform the task',source:'interactive'});
    await f.emit('before_agent_start',{messageType:'user',prompt:f.expanded('fixture','perform the task')});
    assert.deepEqual(f.uses,[]);
    await f.message(f.expanded('fixture','perform the task'));
    assert.deepEqual(f.uses,[{kind:'skill',id:'fixture'}]);
    await f.emit('session_start');f.setIdle(false);
    await f.emit('input',{text:'/skill:fixture queued task',source:'rpc',streamingBehavior:'followUp'});
    await f.message('An unrelated preceding user message');
    await f.message(f.expanded('fixture','queued task'));
    assert.equal(f.uses.length,2);
    await f.message(f.expanded('fixture','queued task'));assert.equal(f.uses.length,2);
  }finally{f.close();}
});

test('pasted tags, failed expansion, changed selection and bounded stale pending inputs do not activate',async()=>{
  const f=fixture();try {
    await f.emit('input',{text:f.expanded(),source:'interactive'});await f.message(f.expanded());
    await f.emit('input',{text:'/skill:fixture',source:'interactive'});await f.message('/skill:fixture');
    await f.emit('input',{text:'ordinary next task',source:'interactive'});await f.message(f.expanded());
    f.setIdle(false);await f.emit('input',{text:'/skill:fixture',source:'interactive'});
    await f.emit('input',{text:f.expanded(),source:'interactive'});await f.message(f.expanded());
    await f.emit('input',{text:'/skill:fixture',source:'interactive'});f.commands.length=0;await f.message(f.expanded());
    f.commands.push({name:'skill:fixture',source:'skill',sourceInfo:{path:f.skill}});
    for(let n=0;n<17;n++)await f.emit('input',{text:'/skill:fixture task '+n,source:'interactive'});
    await f.message(f.expanded('fixture','task 0'));
    assert.deepEqual(f.uses,[]);
    await f.message(f.expanded('fixture','task 16'));assert.deepEqual(f.uses,[{kind:'skill',id:'fixture'}]);
  }finally{f.close();}
});

test('successful substantive native and owned extension tools include nested execution',async()=>{
  const f=fixture();try {
    await f.result('mcp__mirket__code_inspect',{expected_release_id:f.runtime.binding.releaseId,project_root:f.project},{parentToolCallId:'codemode'});
    await f.result('mcp__mirket__code_inspect');await f.result('mirket_delegate');
    assert.deepEqual(f.uses,[{kind:'tool',id:'mcp__mirket__code_inspect'},{kind:'tool',id:'mirket_delegate'}]);
  }finally{f.close();}
});

test('foreign tool owners, changed registrations, overrides and cross-project/release arguments are refused',async()=>{
  const f=fixture();try {
    f.tools[1].sourceInfo.path='/foreign/mcp.js';await f.result('mcp__mirket__code_inspect');f.tools[1].sourceInfo.path='builtin:mcp';
    f.tools[3].sourceInfo.path=f.pkg+'-foreign/dist/extension/index.js';await f.result('mirket_delegate');
    f.servers[0].config.command='/foreign/mirket';await f.result('mcp__mirket__code_inspect');f.servers[0].config.command=f.runtime.command;
    f.servers[0].config.args.push('--foreign');await f.result('mcp__mirket__code_inspect');f.servers[0].config.args.pop();
    await f.result('mcp__mirket__code_inspect',{expected_release_id:'other'});
    await f.result('mcp__mirket__code_inspect',{project_root:f.root});
    await f.result('mcp__mirket__memory_search',{scope:{kind:'project',project_root:f.root}});
    writeFileSync(join(f.root,'agent/mcp.json'),JSON.stringify({mcpServers:{mirket:{command:'/foreign'}}}));await f.result('mcp__mirket__code_inspect');
    assert.deepEqual(f.uses,[]);
  }finally{f.close();}
});

test('missing runtime, cancellation, session shutdown and stale invocation prevent recording',async()=>{
  const f=fixture();try {
    f.setRuntime(undefined);await f.result('read',{path:f.skill});f.setRuntime(f.runtime);
    f.ctx.cwd=f.root;await f.result('read',{path:f.skill});f.ctx.cwd=f.project;
    const cancelled=new AbortController();cancelled.abort();f.setSignal(cancelled.signal);await f.result('read',{path:f.skill});f.setSignal(undefined);
    await f.emit('input',{text:'/skill:fixture',source:'interactive'});await f.emit('session_shutdown');
    await f.message(f.expanded());await f.result('mcp__mirket__code_inspect');
    await f.emit('session_start');await f.message(f.expanded());
    assert.deepEqual(f.uses,[]);
    await f.result('read',{path:f.skill});assert.deepEqual(f.uses,[{kind:'skill',id:'fixture'}]);
  }finally{f.close();}
});

test('sync failure warns without changing the successful result and can be retried',async()=>{
  let calls=0;
  const f=fixture(()=>{if(++calls===1)throw Error('managed block drift');return calls>=3;});try {
    await f.result('read',{path:f.skill});assert.equal(calls,1);
    assert.deepEqual(f.warnings,[{message:'Project guidance sync failed: managed block drift',level:'warning'}]);
    await f.result('read',{path:f.skill});await f.result('read',{path:f.skill});await f.result('read',{path:f.skill});assert.equal(calls,3);
  }finally{f.close();}
});

test('a disabled record does not suppress use after initialization in the same session',async()=>{
  let enabled=false,calls=0;
  const f=fixture(()=>{calls++;return enabled;});try {
    await f.result('read',{path:f.skill});assert.equal(calls,1);
    enabled=true;
    await f.result('read',{path:f.skill});await f.result('read',{path:f.skill});assert.equal(calls,2);
  }finally{f.close();}
});

test('managed children register no observation handlers',()=>{
  const before=process.env.MIRKET_MANAGED_CHILD;
  try {process.env.MIRKET_MANAGED_CHILD='1';registerProjectObservation({on(){throw Error('registered child hook');}},'/unused',()=>{throw Error('read child runtime');},()=>{throw Error('recorded child use');});}
  finally {if(before===undefined)delete process.env.MIRKET_MANAGED_CHILD;else process.env.MIRKET_MANAGED_CHILD=before;}
});
