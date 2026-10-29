// Interrupt the actual assembled setup parent, observe its child, then resume.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {join,isAbsolute} from 'node:path';
import {spawn,execFileSync} from 'node:child_process';
const [packed]=process.argv.slice(2);assert.ok(isAbsolute(packed));
const root=await fs.mkdtemp('/private/tmp/mirket-pi-package-recovery-');await fs.chmod(root,0o700);
for(const name of ['agent','home','project','sessions'])await fs.mkdir(join(root,name),{mode:0o700});
execFileSync('/bin/cp',['-cR',packed,join(root,'package')]);
const agentDir=join(root,'agent'),home=join(agentDir,'mirket'),setup=join(root,'package/scripts/setup.mjs');
const env={...process.env,HOME:join(root,'home'),PI_CODING_AGENT_DIR:agentDir,PI_SKIP_UPDATE_CHECK:'1',PI_TELEMETRY_DISABLED:'1'};
const rows=()=>execFileSync('/bin/ps',['-axo','pid=,ppid=,lstart=,command='],{encoding:'utf8'}).split('\n').flatMap(line=>{const m=line.match(/^\s*(\d+)\s+(\d+)\s+(.{24})\s+(.*)$/);return m?[{pid:+m[1],parent:+m[2],start:m[3],command:m[4]}]:[]});
const report={root,package:join(root,'package'),checks:[]};const save=async name=>{report.checks.push(name);await fs.writeFile(join(root,'setup-recovery-report.json'),JSON.stringify(report,null,2));console.log('PASS',name)};
console.log('Fixture:',root);
function launch(resume){const child=spawn(process.execPath,[setup,'--agent-dir',agentDir,...(resume?['--resume']:[])],{env,stdio:['ignore','pipe','pipe']});let out='',err='';child.stdout.on('data',b=>out+=b.toString());child.stderr.on('data',b=>err+=b.toString());const done=new Promise(resolve=>child.once('close',(code,signal)=>resolve({code,signal,out,err})));return {child,done}}
const first=launch(false);let owned;const deadline=Date.now()+30000;
while(Date.now()<deadline&&first.child.exitCode===null){owned=rows().find(row=>row.parent===first.child.pid&&row.command.includes('setup --home '+home));if(owned&&await fs.stat(home).then(()=>true,()=>false))break;await new Promise(r=>setTimeout(r,25))}
if(!owned){first.child.kill('SIGTERM');await first.done;}
assert.ok(owned,'Actual native setup child was not observed');report.interruptedChild=owned;
first.child.kill('SIGTERM');const result=await first.done;await fs.writeFile(join(root,'interrupted-setup.stdout'),result.out);await fs.writeFile(join(root,'interrupted-setup.stderr'),result.err);assert.notEqual(result.code,0);assert.ok(!rows().some(row=>row.pid===owned.pid&&row.start===owned.start));assert.equal(await fs.stat(join(agentDir,'settings.json')).then(()=>true,()=>false),false);await save('SIGTERM to setup parent terminates observed native child and does not register an incomplete package');
const resumed=await launch(true).done;await fs.writeFile(join(root,'resumed-setup.stdout'),resumed.out);await fs.writeFile(join(root,'resumed-setup.stderr'),resumed.err);assert.equal(resumed.code,0,resumed.err);report.installed=JSON.parse(resumed.out);const binding=JSON.parse(await fs.readFile(join(root,'package/native-binding.json')));assert.equal(report.installed.releaseId,binding.releaseId);const settings=JSON.parse(await fs.readFile(join(agentDir,'settings.json')));assert.equal(settings.packages[0].source,join(root,'package'));await save('Explicit resume completes the same interrupted full native payload and registers its core profile');console.log(root);
