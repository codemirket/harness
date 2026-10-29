import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {createHash} from 'node:crypto';
import {createOnboarding} from '../dist/extension/onboarding.js';
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
function fixture(selections,confirmations=[],paths=[]){
 const root=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'mirket-onboarding-ui-'))),home=path.join(root,'home'),pkg=path.join(root,'package');fs.mkdirSync(home,{mode:0o700});fs.mkdirSync(pkg);
 const specs={schema:1,tools:[{id:'chromium',target:'aarch64-apple-darwin',title:'Chrome',version:'156.0.8078.73',terms:'Google terms fixture'}]},descriptor=JSON.stringify(specs),packageBytes=JSON.stringify({name:'@codemirket/pi-harness',version:'0.0.1'}),releaseId='sha256:'+'1'.repeat(64);
 fs.writeFileSync(path.join(pkg,'package.json'),packageBytes);fs.writeFileSync(path.join(pkg,'runtime-setup.json'),descriptor);fs.writeFileSync(path.join(pkg,'native-binding.json'),JSON.stringify({schema:1,packageVersion:'0.0.1',hostVersion:'1.4.0',releaseId,coreSha256:'2'.repeat(64),nodeSha256:'3'.repeat(64),hostInventorySha256:'4'.repeat(64),packageSha256:hash(packageBytes),runtimeSetupSha256:hash(descriptor)}));
 const messages=[],notices=[],confirms=[],widgets=[];let raw;
 const pi={sendMessage:value=>messages.push(value)},ctx={mode:'tui',hasUI:true,ui:{select:async()=>selections.shift(),input:async()=>paths.shift(),confirm:async(title,body)=>{confirms.push({title,body});return confirmations.shift();},notify:(...args)=>notices.push(args),onTerminalInput:callback=>{raw=callback;return()=>{};},setStatus:()=>{},setWidget:(...args)=>widgets.push(args)}};
 const onboarding=createOnboarding(pi,pkg,()=>({home,cwd:root,binding:{releaseId}}));
 return {root,home,pkg,releaseId,onboarding,ctx,messages,notices,confirms,widgets,raw:()=>raw,close:()=>fs.rmSync(root,{recursive:true,force:true})};
}
test('one native flow exposes both install and installed choices, skip keeps core configuration intact',async()=>{
 const f=fixture(['Skip']);try{await f.onboarding.setup('',f.ctx);const choice=JSON.parse(fs.readFileSync(path.join(f.home,'runtime-tools.json'))).tools.chromium;assert.deepEqual(choice,{choice:'skip'});assert.equal(f.messages.length,1);assert.equal(f.confirms.length,0);assert.equal(fs.existsSync(path.join(f.home,'runtimes')),false);}finally{f.close();}
});
test('denied installed-path authorization never probes the executable',async()=>{
 const f=fixture(['Use an installed executable','Skip'],[false],['/not/executed']);try{await f.onboarding.setup('',f.ctx);assert.equal(f.confirms.length,1);assert.match(f.confirms[0].body,/not sandboxed/);assert.deepEqual(JSON.parse(fs.readFileSync(path.join(f.home,'runtime-tools.json'))).tools.chromium,{choice:'skip'});assert.equal(f.notices.length,0);}finally{f.close();}
});
test('failure offers retry, then skip; cancel exits without persisting a selection',async()=>{
 const f=fixture(['Install pinned 156.0.8078.73 (unavailable)','Retry','Skip']);try{await f.onboarding.setup('',f.ctx);assert.match(f.notices[0][0],/No verified publisher archive/);assert.equal(JSON.parse(fs.readFileSync(path.join(f.home,'runtime-tools.json'))).tools.chromium.choice,'skip');}finally{f.close();}
 const cancelled=fixture(['Cancel setup']);try{await cancelled.onboarding.setup('',cancelled.ctx);assert.equal(fs.existsSync(path.join(cancelled.home,'runtime-tools.json')),false);assert.match(cancelled.notices[0][0],/cancelled/);}finally{cancelled.close();}
});
