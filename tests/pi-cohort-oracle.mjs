// Frozen independent checks, run after each initial author artifact is saved.
import fs from 'node:fs/promises';import {join} from 'node:path';import vm from 'node:vm';import assert from 'node:assert/strict';
export async function checkCode(task,cwd){
 const file=task.id==='intervals'?'merge.mjs':'decoder.mjs',name=task.id==='intervals'?'mergeWindows':'decodeLines';
 const source=await fs.readFile(join(cwd,file),'utf8'),context=vm.createContext({TextDecoder,Uint8Array});
 const module=new vm.SourceTextModule(source,{context});await module.link(()=>{throw Error('No imports required for these pure functions')});await module.evaluate({timeout:1000});context.fn=module.namespace[name];assert.equal(typeof context.fn,'function');
 const checks=[];
 const run=(label,input,expected,error=false)=>{context.input=input;let result,thrown;try{result=new vm.Script('fn(input)').runInContext(context,{timeout:1000});}catch(e){thrown=e;}if(error)assert.equal(thrown?.name,'TypeError',label);else{if(thrown)throw thrown;assert.deepEqual(JSON.parse(JSON.stringify(result)),expected,label);}checks.push(label);};
 if(task.id==='intervals'){
  run('empty',[],[]);run('unsorted-touch-overlap',[[7,9],[1,3],[2,4],[4,7]],[[1,9]]);run('negative-fractional',[[-2,-.5],[0,2],[8,9]],[[-2,-.5],[0,2],[8,9]]);
  const original=[[4,6],[1,2]],before=JSON.stringify(original);run('preserve-input',original,[[1,2],[4,6]]);assert.equal(JSON.stringify(original),before);
  for(const value of [null,{},[[1]],[[1,1]],[[3,2]],[[1,Infinity]],[[NaN,2]],[[1,'2']],[[1,2,3]]])run('malformed-'+JSON.stringify(value),value,null,true);
 }else{
  const encode=s=>new TextEncoder().encode(s);run('empty',[],[]);run('plain',[encode('a\nb\n')],['a','b']);run('interior-empty',[encode('a\n\nb')],['a','','b']);
  const bytes=encode('İğüş\r\n世界\n');run('split-every-byte',[...bytes].map(b=>new Uint8Array([b])),['İğüş','世界']);run('no-final-lf',[encode('last')],['last']);run('single-newline',[encode('\n')],['']);run('invalid',[new Uint8Array([0xc3,0x28])],null,true);
 }
 return {passed:true,checks};
}
export async function checkStructured(task,cwd){
 const value=JSON.parse(await fs.readFile(join(cwd,task.outputs[0]),'utf8'));
 if(['research','memory'].includes(task.area)){assert.equal(typeof value.answer,'string');assert.ok(Array.isArray(value.citations)&&Array.isArray(value.limitations));return {passed:true,shapeOnly:true,requiresBlindContentReview:true,value};}
 if(task.id==='coord-no-delegate'){assert.equal(value.approved,false);assert.match(JSON.stringify(value),/keyboard/i);assert.ok(value.remainingChecks.length);}
 else{assert.equal(typeof value.decision,'string');assert.ok(value.ownership&&typeof value.ownership==='object'&&!Array.isArray(value.ownership));assert.ok(Object.values(value.ownership).every(x=>typeof x==='string'));assert.ok(Array.isArray(value.remainingChecks));}
 return {passed:true,requiresBlindContentReview:true,value};
}
