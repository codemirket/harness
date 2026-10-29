// Installed semantic memory and SQLite recovery artifacts through actual pi.
import {execFileSync} from 'node:child_process';import assert from 'node:assert/strict';import fs from 'node:fs/promises';import {join} from 'node:path';
export async function memoryJourney({session,call,save,cwd,release,root}){
 await call(session,'tool_search',{query:'memory_prepare memory_commit memory_search memory_history memory_read',limit:8});
 const common={expected_release_id:release,scope:{kind:'project',project_root:cwd}},evidence=[];
 const record=content=>({kind:'fact',basis:'observed',content,sources:[{source_id:'fixture:pi-semantic',revision:'v1',locator:'fixture://semantic-test',captured_at:123}],status:'current',critical:false,applicability:{task_key:null,checkout_id:null,operation:null},valid_from:null,valid_until:null});
 const search=query=>call(session,'mcp__mirket__memory_search',{...common,request:{query,status:'both',context:{task_key:null,checkout_id:null,operation:null,at_time:null},limit:20,max_bytes:32768}});
 const original='Gözlemevi teleskobunun bataryası dokuz saat çalışır; ölçüm soğuk gecede yapılmıştır.';
 const prepared=await call(session,'mcp__mirket__memory_prepare',{...common,mutation:{kind:'create',record:record(original),supersedes:null}});
 const committed=await call(session,'mcp__mirket__memory_commit',{...common,operation_id:prepared.operation_id});evidence.push({prepared,committed});
 assert.equal(committed.status,'committed');assert.equal(committed.semantic_index.status,'indexed',JSON.stringify(committed));assert.equal(committed.backup.status,'eligible');
 const snapshot=committed.backup.snapshot;assert.match(snapshot.snapshot_id,/^[a-zA-Z0-9-]+$/);const backup=join(root,'agent/mirket/backups',snapshot.snapshot_id+'.sqlite3');assert.equal((await fs.stat(backup)).size,snapshot.byte_len);
 const {DatabaseSync}=await import('node:sqlite');const db=new DatabaseSync(backup,{readOnly:true});try{assert.equal(db.prepare('PRAGMA quick_check').get().quick_check,'ok');assert.ok(db.prepare('SELECT content FROM revisions WHERE record_id=?').all(committed.record.record_id).some(row=>row.content===original));}finally{db.close();}
 const recalled=await search('How many hours can the observatory telescope battery operate?');evidence.push({recalled});assert.equal(recalled.coverage.semantic_available,true);assert.equal(recalled.coverage.lexical_only,false);assert.ok(recalled.hits.some(hit=>hit.record.record_id===committed.record.record_id&&hit.record.revision===1));
 await save('actual pi semantic recall crosses languages and independently opened SQLite backup contains the canonical fact');
 const correction=record('Gözlemevi teleskobunun yeni bataryası on iki saat çalışır; dokuz saatlik eski karar değiştirilmiştir.');
 const revise=await call(session,'mcp__mirket__memory_prepare',{...common,mutation:{kind:'revise',record_id:committed.record.record_id,expected_revision:1,record:correction}});
 const corrected=await call(session,'mcp__mirket__memory_commit',{...common,operation_id:revise.operation_id});assert.equal(corrected.record.revision,2);assert.equal(corrected.semantic_index.status,'indexed');
 const current=await search('How long does the replacement telescope battery run?');assert.ok(current.hits.some(hit=>hit.record.record_id===committed.record.record_id&&hit.record.revision===2));assert.ok(!current.hits.some(hit=>hit.record.record_id===committed.record.record_id&&hit.record.revision===1));
 const history=await call(session,'mcp__mirket__memory_history',{...common,record_id:committed.record.record_id,before_revision:null,limit:20,max_bytes:32768});assert.deepEqual(history.records.map(record=>record.revision),[2,1]);evidence.push({corrected,current,history});
 await call(session,'tool_search',{query:'memory_forget_prepare memory_forget_commit',limit:4});
 const forget=await call(session,'mcp__mirket__memory_forget_prepare',{...common,record_id:committed.record.record_id,expected_revision:2});const intent={...common,operation_id:forget.operation_id};
 await session.session.prompt('/mirket-authorize '+JSON.stringify({tool:'mcp__mirket__memory_forget_commit',args:intent}));
 const forgotten=await call(session,'mcp__mirket__memory_forget_commit',intent);assert.equal(forgotten.status,'forgotten');assert.equal(forgotten.forget.cleanup_complete,true);assert.equal(await fs.stat(backup).then(()=>true,()=>false),false);
 const after=await search('telescope battery hours');assert.ok(!after.hits.some(hit=>hit.record.record_id===committed.record.record_id));assert.equal(after.coverage.semantic_missing_eligible,0);evidence.push({forgotten,after});
 const recoveryPrepared=await call(session,'mcp__mirket__memory_prepare',{...common,mutation:{kind:'create',record:record('Quartz telescope calibration is twelve seconds.'),supersedes:null}});
 const recoverySaved=await call(session,'mcp__mirket__memory_commit',{...common,operation_id:recoveryPrepared.operation_id});assert.equal(recoverySaved.backup.status,'eligible');
 const laterPrepared=await call(session,'mcp__mirket__memory_prepare',{...common,mutation:{kind:'revise',record_id:recoverySaved.record.record_id,expected_revision:1,record:record('Quartz telescope calibration is now eighteen seconds.')}});
 await call(session,'mcp__mirket__memory_commit',{...common,operation_id:laterPrepared.operation_id});
 // Maintenance is deliberately outside model tool exposure and requires the
 // actual pi MCP session to release its native client before restoration.
 const processRows=()=>execFileSync('/bin/ps',['-axo','pid=,ppid=,lstart=,command='],{encoding:'utf8'}).split('\n').flatMap(line=>{const match=line.match(/^\s*(\d+)\s+(\d+)\s+(.{24})\s+(.*)$/);return match?[{pid:+match[1],parent:+match[2],start:match[3],command:match[4]}]:[]});
 const beforeShutdown=processRows(),ids=new Set([process.pid]);let changed=true;while(changed){changed=false;for(const row of beforeShutdown)if(ids.has(row.parent)&&!ids.has(row.pid)){ids.add(row.pid);changed=true;}}
 const owned=beforeShutdown.filter(row=>row.pid!==process.pid&&ids.has(row.pid)&&!row.command.startsWith('/bin/ps'));
 await session.session.extensionRunner.emit({type:'session_shutdown',reason:'quit'});
 const stopDeadline=Date.now()+3000;let survivors;
 do{survivors=processRows().filter(row=>owned.some(old=>old.pid===row.pid&&old.start===row.start));if(!survivors.length)break;await new Promise(resolve=>setTimeout(resolve,50));}while(Date.now()<stopDeadline);
 assert.deepEqual(survivors,[],'Maintenance requires observed native client/worker shutdown, not just a closed MCP transport');evidence.push({beforeMaintenance:{owned,survivors}});
 const home=join(root,'agent/mirket'),binary=join(home,'bin/mirket');
 const admin=(command,...extra)=>JSON.parse(execFileSync(binary,['dev','memory',command,'--state-dir',join(home,'state'),'--backup-dir',join(home,'backups'),'--release-id',release,'--project-root',cwd,...extra],{encoding:'utf8',timeout:90000,maxBuffer:1024*1024}));
 const restore=admin('restore-prepare','--snapshot-id',recoverySaved.backup.snapshot.snapshot_id);assert.equal(restore.complete,false);
 const restored=admin('restore-commit','--operation-id',restore.operation_id);assert.equal(restored.complete,true);assert.equal(admin('restore-reconcile','--operation-id',restore.operation_id).complete,true);
 await session.session.reload();await session.session.setModel(session.session.modelRuntime.getModel('offline-fixture','deterministic'));
 assert.equal((await call(session,'mirket_doctor')).status,'ready');
 await call(session,'tool_search',{query:'memory_read memory_search memory_index_refresh',limit:6});
 const recovered=await call(session,'mcp__mirket__memory_read',{...common,record_id:recoverySaved.record.record_id,revision:null});assert.equal(recovered.record.revision,1);assert.match(recovered.record.data.content,/twelve/);
 const incomplete=await search('Quartz telescope calibration');assert.ok(incomplete.coverage.semantic_missing_eligible>0);assert.ok(!incomplete.hits.some(hit=>hit.record.record_id===recoverySaved.record.record_id&&hit.record.revision===2));assert.ok(!incomplete.hits.some(hit=>hit.record.record_id===committed.record.record_id));
 const rebuilt=await call(session,'mcp__mirket__memory_index_refresh',{...common,after_row_id:null,limit:32,max_bytes:262144});assert.equal(rebuilt.status,'page_complete');assert.equal(rebuilt.coverage.missing_heads,0);
 const finalRecall=await search('Quartz telescope calibration');assert.equal(finalRecall.coverage.semantic_missing_eligible,0);assert.ok(finalRecall.hits.some(hit=>hit.record.record_id===recoverySaved.record.record_id&&hit.record.revision===1));assert.ok(!finalRecall.hits.some(hit=>hit.record.record_id===committed.record.record_id));
 evidence.push({restore,restored,recovered,incomplete,rebuilt,finalRecall});
 await save('pi releases the client for managed restore, reloads canonical revision, reports incomplete derived coverage and rebuilds without resurrecting forgotten facts');
 await fs.writeFile(join(root,'memory-domain-report-'+Date.now()+'.json'),JSON.stringify({release,evidence},null,2));await save('pi correction excludes stale vectors; authorized forgetting removes semantic recall and the affected recovery snapshot');
}
