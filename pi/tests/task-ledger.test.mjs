import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';import {beginTask,reserveTask,finishTaskReservation} from '../dist/extension/task-ledger.js';
test('shared task counters survive reopening and cancellation does not refund or reset frozen budget',()=>{
 const home=fs.mkdtempSync('/private/tmp/mirket-ledger-');fs.chmodSync(home,0o700);const project=home,task='fixture',brief='a'.repeat(64),limits={repairs:2,childStarts:3,concurrency:2};
 try{
 beginTask(home,project,task,brief,limits);reserveTask(home,project,task,'child','one');reserveTask(home,project,task,'child','two');assert.throws(()=>reserveTask(home,project,task,'child','three'),/concurrency/);
 assert.equal(beginTask(home,project,task,brief,limits).reservations.length,2);assert.throws(()=>beginTask(home,project,task,'b'.repeat(64),limits),/frozen/);assert.throws(()=>beginTask(home,project,task,brief,{...limits,childStarts:4}),/frozen/);
 finishTaskReservation(home,project,task,'one','cancelled');reserveTask(home,project,task,'child','three');finishTaskReservation(home,project,task,'two','complete');assert.throws(()=>reserveTask(home,project,task,'child','four'),/budget/);
 reserveTask(home,project,task,'repair','round-one');finishTaskReservation(home,project,task,'round-one','incomplete');reserveTask(home,project,task,'repair','new-specimen');assert.throws(()=>reserveTask(home,project,task,'repair','fork-attempt'),/budget/);
 assert.throws(()=>reserveTask(home,project,task,'repair','round-one'),/already exists/);assert.throws(()=>finishTaskReservation(home,project,task,'one','complete'),/already recorded/);
 }finally{fs.rmSync(home,{recursive:true,force:true});}
});
test('corrupt persisted limits fail closed without replacing state',()=>{
 const home=fs.mkdtempSync('/private/tmp/mirket-ledger-invalid-');fs.chmodSync(home,0o700);
 try{beginTask(home,home,'fixture','a'.repeat(64),{repairs:2,childStarts:2,concurrency:2});const file=home+'/tasks/'+fs.readdirSync(home+'/tasks')[0];const data=JSON.parse(fs.readFileSync(file));data.limits.childStarts=999;const invalid=JSON.stringify(data);fs.writeFileSync(file,invalid);assert.throws(()=>reserveTask(home,home,'fixture','child','one'),/Invalid task ledger/);assert.equal(fs.readFileSync(file,'utf8'),invalid);}finally{fs.rmSync(home,{recursive:true,force:true});}
});
test('frozen output ownership rejects overlap and path escape across reopened task callers',()=>{
 const home=fs.mkdtempSync('/private/tmp/mirket-ledger-ownership-');fs.chmodSync(home,0o700);
 const brief={deliverable:'independent files',outputs:['a.txt','folder/b.txt'],rubric:['both correct'],provider:'openai',model:'gpt-6-astra',effort:'medium'};
 try{
  beginTask(home,home,'fixture','c'.repeat(64),{repairs:2,childStarts:4,concurrency:2},brief);
  reserveTask(home,home,'fixture','child','one',['a.txt']);assert.throws(()=>reserveTask(home,home,'fixture','child','overlap',['a.txt']),/ownership overlaps/);
  assert.throws(()=>reserveTask(home,home,'fixture','child','outside',['../escape']),/frozen task/);
  assert.throws(()=>reserveTask(home,home,'fixture','child','settings',['.pi/settings.json']),/frozen task/);
  reserveTask(home,home,'fixture','child','two',['folder/b.txt']);
  assert.throws(()=>beginTask(home,home,'fixture','c'.repeat(64),{repairs:2,childStarts:4,concurrency:2},{...brief,outputs:['other.txt']}),/frozen/);
 }finally{fs.rmSync(home,{recursive:true,force:true});}
});
