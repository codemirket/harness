// Render the unchanged authored motion sources through installed native pi MCP.
import fs from 'node:fs/promises';import {join} from 'node:path';import {createHash} from 'node:crypto';import {execFileSync} from 'node:child_process';
export async function cohortVideoJourney({session,call,save,cwd,release,root,cohort}){
 const spec=JSON.parse(await fs.readFile(join(cohort,'frozen.json'))),results=[];
 await call(session,'tool_search',{query:'video_prepare video_start video_status',limit:5});
 const common={expected_release_id:release,project_root:cwd};const native=join(root,'agent/mirket/releases','r-'+release.slice(7),'runtime/video/bin');
 for(const task of spec.tasks.filter(t=>t.area==='motion'))for(const condition of ['vanilla','mirket']){
  const name=task.id+'-'+condition,work=join(cwd,'cohort-'+name);await fs.mkdir(work,{mode:0o700});const report={task:task.id,condition,work};
  try{
   const bytes=await fs.readFile(join(cohort,name,'project/index.html'));await fs.writeFile(join(work,'index.html'),bytes);report.sourceSha256=createHash('sha256').update(bytes).digest('hex');const assets=[];for(const asset of task.assets??[]){const data=await fs.readFile(join(cohort,name,'project',asset.name));const sha=createHash('sha256').update(data).digest('hex');if(sha!==asset.sha256)throw Error('Changed supplied asset');await fs.writeFile(join(work,asset.name),data);assets.push({path:asset.name,expected_revision:'sha256:'+sha});}
   const destination=join(work,'result.mp4');const intent={...common,source_path:join(work,'index.html'),expected_revision:'sha256:'+report.sourceSha256,destination_path:destination,assets,profile:{width:640,height:360,fps:{numerator:12,denominator:1},frame_count:24,format:'mp4',captions_path:null}};
   const prepared=await call(session,'mcp__mirket__video_prepare',intent),job={...common,operation_id:prepared.operation_id};report.intent=intent;report.prepared=prepared;await call(session,'mcp__mirket__video_start',job);
   const deadline=Date.now()+180000;let status;
   while(Date.now()<deadline){try{status=await call(session,'mcp__mirket__video_status',job);}catch(error){if(!String(error).includes('operation store is busy'))throw error;}if(status&&['succeeded','failed','cancelled'].includes(status.status))break;await new Promise(r=>setTimeout(r,250));}
   report.result=status;if(status?.status!=='succeeded')throw Error('Native export did not succeed: '+JSON.stringify(status));
   report.probe=JSON.parse(execFileSync(join(native,'ffprobe'),['-v','error','-show_streams','-show_format','-of','json',destination],{encoding:'utf8',timeout:10000}));report.images=[];
   for(const time of [0,1,1.9]){const output=join(work,'decoded-'+time+'.png');execFileSync(join(native,'ffmpeg'),['-v','error','-nostdin','-i',destination,'-ss',String(time),'-frames:v','1',output],{timeout:10000});report.images.push(output);}
   report.status='exported';
  }catch(error){report.status='failed';report.error=String(error.stack??error);}
  results.push(report);await fs.writeFile(join(cohort,'native-video-results.json'),JSON.stringify(results,null,2));console.log('COHORT_EXPORT',name,report.status);
 }
}
