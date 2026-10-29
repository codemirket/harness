// Actual CLI + compiled child policy, using a deterministic no-network provider.
import assert from 'node:assert/strict';import fs from 'node:fs/promises';import {join,isAbsolute} from 'node:path';import {spawnSync} from 'node:child_process';
import {beginTask,reserveTask,finishTaskReservation} from '../pi/dist/extension/task-ledger.js';
const [node,peers,root,packageName='package']=process.argv.slice(2);assert.ok([node,peers,root].every(isAbsolute)&&root.startsWith('/private/tmp/mirket-pi-package-'));
const agent=join(root,'agent'),home=join(agent,'mirket'),project=join(root,'project'),packed=join(root,packageName),binding=JSON.parse(await fs.readFile(join(packed,'native-binding.json')));
const base=join(home,'child-boundaries-'+Date.now());await fs.mkdir(base,{mode:0o700});
const report=[];
for(const scenario of ['wrong-effort','unowned-write','dangling-link','recursive-tool']){
 const work=join(base,scenario);await fs.mkdir(work,{mode:0o700});const task='boundary-'+scenario+'-'+Date.now();
 const owned='boundary-'+scenario+'-'+Date.now()+'.txt',outside=join(root,'escaped-'+task+'.txt');
 const brief={deliverable:'Scope counterexample',outputs:[owned],rubric:['No forbidden output'],provider:'offline-fixture',model:'deterministic',effort:scenario==='wrong-effort'?'medium':'off'};
 beginTask(home,project,task,'d'.repeat(64),{repairs:0,childStarts:1,concurrency:1},brief);reserveTask(home,project,task,'child','child',[owned]);
 const tools=['read','write','mirket_doctor','mcp__mirket__runtime_status'];
 const contract={schema:1,task,reservation:'child',project,home,packageRoot:packed,release:binding.releaseId,provider:brief.provider,model:brief.model,effort:brief.effort,outputs:[owned],tools,receipt:join(work,'bootstrap.json'),briefSha256:'d'.repeat(64)};
 const contractFile=join(work,'contract.json');await fs.writeFile(contractFile,JSON.stringify(contract),{mode:0o600});
 if(scenario==='dangling-link')await fs.symlink(outside,join(project,owned));
 const request=scenario==='recursive-tool'?{tool:'mirket_delegate',args:{prompt:'recursive attempt',outputs:[],role:'mechanical',repair:false}}:{tool:'write',args:{path:scenario==='unowned-write'?outside:join(project,owned),content:'forbidden'}};
 const bootstrap=new URL('../pi/dist/extension/child-bootstrap.js',import.meta.url).pathname;
 const result=spawnSync(node,[join(peers,'@earendil-works/pi-coding-agent/dist/cli.js'),'--offline','--no-extensions','-e','builtin:mcp','-e',join(packed,'dist/extension/index.js'),'-e',bootstrap,'-e',join(root,'child-provider.mjs'),'--no-skills','--no-prompt-templates','--no-themes','--no-context-files','--provider','offline-fixture','--model','deterministic','--thinking','off','--mode','json','--print','--tools',tools.join(','),JSON.stringify(request)],{cwd:project,env:{...process.env,HOME:join(root,'home'),PI_CODING_AGENT_DIR:agent,PI_CODING_AGENT_SESSION_DIR:join(work,'sessions'),PI_SKIP_UPDATE_CHECK:'1',PI_TELEMETRY_DISABLED:'1',MIRKET_MANAGED_CHILD:'1',MIRKET_CHILD_CONTRACT:contractFile},encoding:'utf8',timeout:20000,maxBuffer:2*1024*1024});
 await fs.writeFile(join(work,'stdout.jsonl'),result.stdout);await fs.writeFile(join(work,'stderr.txt'),result.stderr);
 assert.equal(await fs.stat(outside).then(()=>true,()=>false),false);assert.equal(await fs.stat(join(project,owned)).then(()=>true,()=>false),false);
 if(scenario==='wrong-effort'){assert.equal(await fs.stat(contract.receipt).then(()=>true,()=>false),false);assert.match(result.stdout+result.stderr,/effective model, effort or project differs|verified managed child/);}
 else{assert.ok(await fs.stat(contract.receipt));assert.match(result.stdout,/"isError":true/);}
 if(scenario==='dangling-link')await fs.unlink(join(project,owned));finishTaskReservation(home,project,task,'child','incomplete');report.push({scenario,evidence:work,status:'passed'});console.log('PASS',scenario);
}
await fs.writeFile(join(root,'child-boundaries-proof.json'),JSON.stringify(report,null,2));
