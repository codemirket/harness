import fs from 'node:fs/promises';import {join} from 'node:path';
export async function seedTuiAuthorization({session,call,cwd,release,root}){
 const common={expected_release_id:release,scope:{kind:'project',project_root:cwd}};
 await call(session,'tool_search',{query:'memory_prepare memory_commit memory_forget_prepare',limit:4});
 const record={kind:'fact',basis:'observed',content:'Disposable interactive authorization control '+Date.now(),sources:[{source_id:'fixture:tui-auth',revision:'v1',locator:'fixture://tui-auth',captured_at:123}],status:'current',critical:false,applicability:{task_key:null,checkout_id:null,operation:null},valid_from:null,valid_until:null};
 const prepared=await call(session,'mcp__mirket__memory_prepare',{...common,mutation:{kind:'create',record,supersedes:null}}),committed=await call(session,'mcp__mirket__memory_commit',{...common,operation_id:prepared.operation_id});

 await fs.writeFile(join(root,'tui-auth-intent.json'),JSON.stringify({recordId:committed.record.record_id,prepare:{tool:'mcp__mirket__memory_forget_prepare',args:{...common,record_id:committed.record.record_id,expected_revision:1}},request:{tool:'mcp__mirket__memory_forget_commit',args:{...common,operation_id:'prepared-in-tui'}}},null,2));
}
