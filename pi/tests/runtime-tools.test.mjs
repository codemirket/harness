import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {bindRuntime,installRuntime,runtimeToolStatus,skipRuntime,assertRuntimeTools} from '../dist/extension/runtime-tools.js';
const hash=data=>createHash('sha256').update(data).digest('hex');
const spec={id:'chromium',target:'aarch64-apple-darwin',title:'Chrome for video rendering',version:'156.0.8078.73',terms:'Test fixture only'};
function fixture(){const home=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'mirket-runtime-tools-')));fs.chmodSync(home,0o700);return {home,close(){fs.rmSync(home,{recursive:true,force:true});}};}
function fakeChrome(file,wait=false){
 // A fixture exercises ownership/protocol; actual Chrome evidence is a separate integration gate.
 const source=`#!${process.execPath}\nimport fs from 'node:fs';\nif(process.argv.includes('--version')){console.log('Google Chrome 156.0.8078.73');process.exit(0);}\n${wait?"setTimeout(()=>process.exit(0),60000);":"console.log("+JSON.stringify('<div id="probe">mirket-runtime-ready</div>')+");"}\n`;
 fs.writeFileSync(file,source,{mode:0o700});return Buffer.from(source);
}
function zip(name,data){
 const local=Buffer.alloc(30);local.writeUInt32LE(0x04034b50);local.writeUInt16LE(20,4);local.writeUInt32LE(data.length,18);local.writeUInt32LE(data.length,22);local.writeUInt16LE(Buffer.byteLength(name),26);
 const central=Buffer.alloc(46);central.writeUInt32LE(0x02014b50);central.writeUInt16LE(20,4);central.writeUInt16LE(20,6);central.writeUInt32LE(data.length,20);central.writeUInt32LE(data.length,24);central.writeUInt16LE(Buffer.byteLength(name),28);
 const body=Buffer.concat([local,Buffer.from(name),data]),directory=Buffer.concat([central,Buffer.from(name)]),end=Buffer.alloc(22);end.writeUInt32LE(0x06054b50);end.writeUInt16LE(1,8);end.writeUInt16LE(1,10);end.writeUInt32LE(directory.length,12);end.writeUInt32LE(body.length,16);return Buffer.concat([body,directory,end]);
}
function archiveSpec(bytes,executable){const archive=zip('chrome/test-chrome',bytes);return {archive,selected:{...spec,format:'zip',archive:{url:'https://fixture.invalid/chrome.zip',bytes:archive.length,sha256:hash(archive)},executable:'runtime/browser/chrome/test-chrome',files:[{path:'runtime/browser/chrome/test-chrome',bytes:bytes.length,sha256:hash(bytes),mode:0o700}],links:[]}};}
test('installed choice tests capability, persists exact path and catches changed bytes before jobs',async()=>{
 const f=fixture();try{const executable=path.join(f.home,'selected-chrome.mjs');fakeChrome(executable);await bindRuntime(f.home,'release-1',spec,executable,new AbortController().signal,()=>{});
 assert.equal(runtimeToolStatus(f.home,'release-1',[spec])[0].status,'ready');assert.equal(JSON.parse(fs.readFileSync(path.join(f.home,'runtime-tools.json'))).tools.chromium.path,executable);
 const runtime={home:f.home,binding:{releaseId:'release-1'}};assertRuntimeTools(runtime,['chromium']);fs.appendFileSync(executable,'// changed\n');assert.equal(runtimeToolStatus(f.home,'release-1',[spec])[0].status,'changed');assert.throws(()=>assertRuntimeTools(runtime,['chromium']),/missing or changed/);
 assert.deepEqual(fs.readdirSync(f.home).filter(name=>name.startsWith('.runtime-probe')),[]);
 }finally{f.close();}
});
test('pinned choice verifies archive and members, retains completed payload and progress',async()=>{
 const f=fixture();try{const input=path.join(f.home,'fixture.mjs'),bytes=fakeChrome(input),{archive,selected}=archiveSpec(bytes);const messages=[];await installRuntime(f.home,'release-1',selected,new AbortController().signal,message=>messages.push(message),async()=>new Response(archive));
 const status=runtimeToolStatus(f.home,'release-1',[spec])[0];assert.equal(status.status,'ready');assert.equal(status.choice,'pinned');assert.ok(status.path.startsWith(path.join(f.home,'runtimes')));assert.ok(messages.includes('Archive SHA-256 verified'));assert.ok(messages.some(message=>message.includes('100%')));assert.equal(fs.readdirSync(path.join(f.home,'runtimes')).some(name=>name.endsWith('.partial')),false);
 }finally{f.close();}
});
test('skip marks missing without invoking or installing any tool',()=>{const f=fixture();try{skipRuntime(f.home,'release-1','chromium');assert.equal(runtimeToolStatus(f.home,'release-1',[spec])[0].status,'missing');assert.equal(fs.existsSync(path.join(f.home,'runtimes')),false);}finally{f.close();}});
test('bad download hash leaves prior selection untouched and permits retry',async()=>{
 const f=fixture();try{const bytes=fakeChrome(path.join(f.home,'fixture.mjs')),{archive,selected}=archiveSpec(bytes);skipRuntime(f.home,'release-1','chromium');const before=fs.readFileSync(path.join(f.home,'runtime-tools.json'),'utf8');
 await assert.rejects(installRuntime(f.home,'release-1',{...selected,archive:{...selected.archive,sha256:'0'.repeat(64)}},new AbortController().signal,()=>{},async()=>new Response(archive)),/SHA-256/);assert.equal(fs.readFileSync(path.join(f.home,'runtime-tools.json'),'utf8'),before);assert.deepEqual(fs.readdirSync(path.join(f.home,'runtimes')),[]);
 await installRuntime(f.home,'release-1',selected,new AbortController().signal,()=>{},async()=>new Response(archive));assert.equal(runtimeToolStatus(f.home,'release-1',[spec])[0].status,'ready');
 }finally{f.close();}
});
test('cancelled download and capability process leave no unfinished payload or selection',async()=>{
 const f=fixture();try{const controller=new AbortController(),bytes=fakeChrome(path.join(f.home,'fixture.mjs')),{archive,selected}=archiveSpec(bytes);
 await assert.rejects(installRuntime(f.home,'release-1',selected,controller.signal,()=>controller.abort(),async()=>new Response(archive)));assert.equal(fs.existsSync(path.join(f.home,'runtime-tools.json')),false);assert.deepEqual(fs.readdirSync(path.join(f.home,'runtimes')),[]);
 const executable=path.join(f.home,'waiting-chrome.mjs');fakeChrome(executable,true);const probeController=new AbortController();const timer=setTimeout(()=>probeController.abort(),1000);try{await assert.rejects(bindRuntime(f.home,'release-1',spec,executable,probeController.signal,()=>{}),/cancelled/);}finally{clearTimeout(timer);}assert.equal(fs.existsSync(path.join(f.home,'runtime-tools.json')),false);assert.deepEqual(fs.readdirSync(f.home).filter(name=>name.startsWith('.runtime-probe')),[]);
 }finally{f.close();}
});
test('unsafe archive member refuses before extraction',async()=>{
 const f=fixture();try{const archive=zip('../escape',Buffer.from('bad'));const selected={...spec,format:'zip',archive:{url:'https://fixture.invalid/bad.zip',bytes:archive.length,sha256:hash(archive)},executable:'runtime/browser/escape',files:[{path:'runtime/browser/escape',bytes:3,sha256:hash(Buffer.from('bad')),mode:0o700}],links:[]};await assert.rejects(installRuntime(f.home,'release-1',selected,new AbortController().signal,()=>{},async()=>new Response(archive)),/Unsafe/);assert.deepEqual(fs.readdirSync(path.join(f.home,'runtimes')),[]);}finally{f.close();}
});

test('platform mismatch refuses before any download',async()=>{const f=fixture();try{let fetched=false;await assert.rejects(installRuntime(f.home,'release-1',{...spec,target:'x86_64-pc-windows-msvc',archive:{url:'https://fixture.invalid/archive',bytes:1,sha256:'0'.repeat(64)}},new AbortController().signal,()=>{},async()=>{fetched=true;return new Response('x');}),/does not match/);assert.equal(fetched,false);assert.equal(fs.existsSync(path.join(f.home,'runtimes')),false);}finally{f.close();}});
test('wrong Chrome version refuses before saving a binding',async()=>{const f=fixture();try{const executable=path.join(f.home,'old-chrome.mjs');fakeChrome(executable);await assert.rejects(bindRuntime(f.home,'release-1',{...spec,version:'155.0.0.0'},executable,new AbortController().signal,()=>{}),/Chrome 155\.0\.0\.0 is required/);assert.equal(fs.existsSync(path.join(f.home,'runtime-tools.json')),false);assert.deepEqual(fs.readdirSync(f.home).filter(name=>name.startsWith('.runtime-probe')),[]);}finally{f.close();}});

test('correct executed DOM with nonzero Chrome teardown still refuses the binding',async()=>{const f=fixture();try{const executable=path.join(f.home,'teardown-failure.mjs');fakeChrome(executable);fs.appendFileSync(executable,'process.exitCode=2;\n');await assert.rejects(bindRuntime(f.home,'release-1',spec,executable,new AbortController().signal,()=>{}),/Capability check failed \(2\)/);assert.equal(fs.existsSync(path.join(f.home,'runtime-tools.json')),false);assert.deepEqual(fs.readdirSync(f.home).filter(name=>name.startsWith('.runtime-probe')),[]);}finally{f.close();}});

test('separate browser binding preserves video selection and refuses whole-app byte/member/link drift',async()=>{
 const f=fixture();try{
 const app=path.join(f.home,'Chosen.app');fs.mkdirSync(app);const executable=path.join(app,'browser.mjs');fakeChrome(executable);const resource=path.join(app,'resource');fs.writeFileSync(resource,'initial');
 skipRuntime(f.home,'release-1','chromium');const browserSpec={...spec,id:'browser-chromium'};
 await bindRuntime(f.home,'release-1',browserSpec,executable,new AbortController().signal,()=>{},{treeRoot:app});
 assert.deepEqual(JSON.parse(fs.readFileSync(path.join(f.home,'runtime-tools.json'))).tools.chromium,{choice:'skip'});
 const runtime={home:f.home,binding:{releaseId:'release-1'}};assertRuntimeTools(runtime,['browser-chromium']);
 fs.writeFileSync(resource,'changed');assert.throws(()=>assertRuntimeTools(runtime,['browser-chromium']),/identity changed/);fs.writeFileSync(resource,'initial');
 fs.writeFileSync(path.join(app,'added'),'unexpected');assert.equal(runtimeToolStatus(f.home,'release-1',[browserSpec])[0].status,'changed');fs.unlinkSync(path.join(app,'added'));
 fs.symlinkSync('/etc/passwd',path.join(app,'escape'));await assert.rejects(bindRuntime(f.home,'release-1',browserSpec,executable,new AbortController().signal,()=>{},{treeRoot:app}),/link escapes/);
 }finally{f.close();}
});

test('non-executable peer source uses explicit Node, joint MCP capability and selected-tree identity',async()=>{
 const f=fixture();try{
 const app=path.join(f.home,'Browser.app');fs.mkdirSync(app);const browser=path.join(app,'browser.mjs');fakeChrome(browser);
 await bindRuntime(f.home,'release-1',{...spec,id:'browser-chromium'},browser,new AbortController().signal,()=>{},{treeRoot:app});
 const tree=path.join(f.home,'peer');fs.mkdirSync(tree);const source=`if(process.argv.includes('--version')){console.log('Version 0.0.83');process.exit(0);}if(process.argv.includes('--help')){console.log('--executable-path --isolated --config');process.exit(0);}const {createInterface}=require('node:readline');const input=createInterface({input:process.stdin});input.on('line',line=>{const message=JSON.parse(line);if(!message.id)return;const result=message.method==='initialize'?{protocolVersion:'2024-11-05',capabilities:{tools:{}},serverInfo:{name:'fixture',version:'0.0.83'}}:{content:[{type:'text',text:'mirket-playwright-capability-ready'}]};console.log(JSON.stringify({jsonrpc:'2.0',id:message.id,result}));});`;
 const relative='node_modules/@playwright/mcp/cli.cjs',cli=path.join(tree,relative);fs.mkdirSync(path.dirname(cli),{recursive:true});fs.writeFileSync(cli,source,{mode:0o600});
 const selected={id:'playwright-mcp',target:spec.target,title:'Protocol fixture server, not upstream qualification',version:'0.0.83',terms:'fixture',executable:relative,files:[{path:relative,bytes:Buffer.byteLength(source),sha256:hash(Buffer.from(source)),mode:0o600}]};
 const shadow=path.join(tree,'node_modules/@playwright/mcp/node_modules/playwright-core');fs.mkdirSync(shadow,{recursive:true});fs.writeFileSync(path.join(shadow,'package.json'),'{"version":"1.64.0-alpha-1790635538000"}');
 await assert.rejects(bindRuntime(f.home,'release-1',selected,tree,new AbortController().signal,()=>{},{node:process.execPath}),/unselected members/);assert.equal(JSON.parse(fs.readFileSync(path.join(f.home,'runtime-tools.json'))).tools['playwright-mcp'],undefined);
 fs.rmSync(path.join(tree,'node_modules/@playwright/mcp/node_modules'),{recursive:true});
 await bindRuntime(f.home,'release-1',selected,tree,new AbortController().signal,()=>{},{node:process.execPath});
 const runtime={home:f.home,binding:{releaseId:'release-1'}};assertRuntimeTools(runtime,['playwright-mcp']);
 fs.writeFileSync(path.join(tree,'extra.txt'),'unselected addition');assert.throws(()=>assertRuntimeTools(runtime,['playwright-mcp']),/tree identity changed/);
 }finally{f.close();}
});
