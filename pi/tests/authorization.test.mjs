import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';import {authorized,grant,needsAuthorization} from '../dist/extension/authorization.js';
test('exact grants survive reload state, expire and cannot authorize a changed project, release or operation',()=>{
 const home=fs.mkdtempSync('/private/tmp/mirket-authorization-');fs.chmodSync(home,0o700);
 const runtime={home,cwd:'/project',binding:{releaseId:'release-one'}},tool='mcp__mirket__memory_forget_commit',args={scope:{kind:'project',project_root:'/project'},operation_id:'intent-one'};
 try{
 assert.equal(needsAuthorization(tool,args),true);assert.equal(authorized(runtime,tool,args,1000),false);grant(runtime,tool,args,1000);
 assert.equal(authorized({...runtime},tool,{operation_id:'intent-one',scope:args.scope},1001),true);
 assert.equal(authorized(runtime,tool,{...args,operation_id:'intent-two'},1001),false);
 assert.equal(authorized({...runtime,cwd:'/other'},tool,args,1001),false);
 assert.equal(authorized({...runtime,binding:{releaseId:'release-two'}},tool,args,1001),false);
 assert.equal(authorized(runtime,tool,args,301000),false);
 assert.equal(needsAuthorization('mcp__mirket__memory_read',{scope:{kind:'shared_preferences'}}),true);
 assert.equal(needsAuthorization('mcp__mirket__memory_commit',args),false);
 const saved=fs.readFileSync(home+'/authorizations.json','utf8');fs.mkdirSync(home+'/authorizations.lock');assert.throws(()=>grant(runtime,tool,args,1002));assert.equal(fs.readFileSync(home+'/authorizations.json','utf8'),saved);
 }finally{fs.rmSync(home,{recursive:true,force:true});}
});
