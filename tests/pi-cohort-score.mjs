// Post-author independent checks and neutral artifact packets. Never edits authors.
import fs from 'node:fs/promises';import {join} from 'node:path';import {randomUUID,createHash} from 'node:crypto';
import {checkCode,checkStructured} from './pi-cohort-oracle.mjs';import {renderLocal} from './pi-cohort-render.mjs';
const [cohort,browserPeers,chrome]=process.argv.slice(2);const spec=JSON.parse(await fs.readFile(join(cohort,'frozen.json')));
const root=join(cohort,'blind');await fs.mkdir(root,{mode:0o700});const mapping=[],results=[];
for(const task of spec.tasks){
 const candidates=[];
 for(const condition of ['vanilla','mirket']){
  const cell=join(cohort,task.id+'-'+condition),cwd=join(cell,'project'),id=randomUUID();const out=join(root,id);await fs.mkdir(out,{mode:0o700});
  const result={task:task.id,condition,id,checks:[],images:[]};
  try{
   const report=JSON.parse(await fs.readFile(join(cell,'report.json')));result.authorStatus=report.status;result.usage=report.usage;result.elapsedMs=report.elapsedMs;result.toolCount=report.toolCalls.length;result.skillReads=report.toolCalls.filter(t=>t.name==='read'&&String(t.input.path).includes('/skills/')).map(t=>t.input.path.split('/skills/')[1]);
   for(const name of task.outputs){const bytes=await fs.readFile(join(cwd,name));const bound=report.outputs.find(o=>o.name===name);if(createHash('sha256').update(bytes).digest('hex')!==bound.sha256)throw Error('Saved output changed since author report');await fs.writeFile(join(out,name),bytes);}
   if(task.area==='code')result.checks.push(await checkCode(task,cwd));
   else if(!['ui','motion'].includes(task.area))result.checks.push(await checkStructured(task,cwd));
   else{
    const states=task.id==='localized-ui'?[{label:'wide',width:1000,height:700,time:0,reduced:false},{label:'narrow-reduced',width:360,height:740,time:0,reduced:true}]:task.id==='vector-alert'?[{label:'vector',width:480,height:240,time:0,reduced:true}]:[0,1,1.9].map(time=>({label:'t-'+time,width:640,height:360,time,reduced:false}));
    for(const state of states){const output=join(out,state.label+'.png'),render=await renderLocal({cwd,file:task.outputs[0],...state,browserPeers,chrome,output});const safe={label:state.label,state:render.state,focus:render.focus,after:render.after};result.images.push(output);result.checks.push(safe);}
    if(task.area==='motion'){
     const exports=JSON.parse(await fs.readFile(join(cohort,'native-video-results.json'))),record=exports.find(r=>r.task===task.id&&r.condition===condition);
     result.checks.push({nativeExport:{status:record?.status,error:record?.error,streams:record?.probe?.streams?.map(s=>({type:s.codec_type,width:s.width,height:s.height,frames:s.nb_frames}))}});
     result.executablePassed=record?.status==='exported';
     for(const [index,path]of (record?.images??[]).entries()){const destination=join(out,'export-'+index+'.png');await fs.copyFile(path,destination);result.images.push(destination);}
    }
    if(task.id==='localized-ui')result.executablePassed=result.checks.every(c=>c.state.scrollWidth<=c.state.width&&c.focus.tag==='BUTTON'&&c.focus.outline!=='none'&&c.after.pressed==='true'&&c.after.text.includes('Açık'));
    if(task.id==='vector-alert'){const source=await fs.readFile(join(cwd,'alert.svg'),'utf8');result.executablePassed=/<text[\s>]/.test(source)&&!/<image[\s>]|https?:\/\/(?!www.w3.org\/2000\/svg)/.test(source);}
   }
  }catch(error){result.error=String(error.stack??error);result.executablePassed=false;}
  await fs.writeFile(join(out,'checks.json'),JSON.stringify(result.checks,null,2));mapping.push({id,task:task.id,condition});results.push(result);const blinded=JSON.parse(JSON.stringify({id,files:task.outputs,images:result.images.map(p=>p.split('/').at(-1)),checks:result.checks,error:result.error}).replaceAll(task.id+'-'+condition,id).replaceAll(cohort,'<cohort>'));candidates.push(blinded);
 }
 // Candidate order is randomized independently of condition and never described
 // to the reviewer. All initial artifacts and failures remain in the packet.
 if(Math.random()<.5)candidates.reverse();
 const packet={task:task.id,prompt:task.prompt,inputs:task.files,rubric:spec.rubric[task.area],candidates};
 await fs.writeFile(join(root,task.id+'.json'),JSON.stringify(packet,null,2));
 await fs.writeFile(join(cohort,'oracle-results.json'),JSON.stringify(results,null,2));await fs.writeFile(join(cohort,'blind-mapping.json'),JSON.stringify(mapping,null,2));
 console.log('CHECKED',task.id);
}
