// Domain outcomes through the actual loaded pi package and its native MCP client.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
const revision=bytes=>'sha256:'+createHash('sha256').update(bytes).digest('hex');
export async function domainJourneys({session,call,save,cwd,release,root}){
 const runId=Date.now().toString(),work=join(cwd,'domain-'+runId);await fs.mkdir(work,{mode:0o700});
 const common={expected_release_id:release,project_root:cwd},evidence=[];
 await call(session,'tool_search',{query:'code_navigate design_prepare design_start job_status',limit:8});
 const text='const café = 17;\r\nconsole.log(café);\r\n',file=join(work,'navigation.ts');await fs.writeFile(file,text);
 const request={sources:[{path:file,language:'ts',expected_revision:revision(text)}],source:0,position_byte:Buffer.byteLength(text.slice(0,text.lastIndexOf('café'))),operation:'definitions',limit:10,typescript_model:{imports:[]}};
 const navigation=await call(session,'mcp__mirket__code_navigate',{...common,request});const outcome=navigation.navigation.outcome;
 assert.equal(outcome.status,'locations');assert.equal(outcome.model_complete,false);assert.equal(outcome.locations[0].revision,revision(text));assert.equal(outcome.locations[0].location.start_byte,Buffer.byteLength('const '));
 await fs.appendFile(file,'// changed source');const changed=await call(session,'mcp__mirket__code_navigate',{...common,request});assert.equal(changed.navigation.outcome.status,'changed');assert.equal(changed.navigation.outcome.locations,undefined);evidence.push({navigation,changed});
 await save('pi selected-source navigation resolves UTF-8/CRLF definition and refuses stale revision');
 async function design(name,bytes,destination,operation){
  const source=join(work,name),output=join(work,destination);await fs.writeFile(source,bytes);
  const prepared=await call(session,'mcp__mirket__design_prepare',{...common,source_path:source,expected_revision:revision(bytes),destination_path:output,operation});
  await call(session,'mcp__mirket__design_start',{...common,operation_id:prepared.operation_id});const deadline=Date.now()+65000;let result;
  while(Date.now()<deadline){
   try{result=await call(session,'mcp__mirket__job_status',{...common,operation_id:prepared.operation_id});}
   catch(error){if(!String(error).includes('operation store is busy'))throw error;evidence.push({busyRead:prepared.operation_id});await new Promise(r=>setTimeout(r,100));continue;}
   if(['succeeded','failed','cancelled'].includes(result.status)&&!result.scratch_cleanup?.startsWith('retained: terminal confirmation'))break;await new Promise(r=>setTimeout(r,100));
  }
  assert.ok(['succeeded','failed','cancelled'].includes(result.status),JSON.stringify(result));evidence.push({prepared,result,output});return {result,output};
 }
 const css=await design('tokens.json',JSON.stringify({color:{brand:{value:'#1463ff'}}}),'tokens.css',{kind:'tokens_compile',format:'css'});assert.equal(css.result.status,'succeeded',JSON.stringify(css.result));const cssBytes=await fs.readFile(css.output);assert.match(cssBytes.toString(),/--color-brand: #1463ff;/);assert.equal(css.result.artifact_revision,revision(cssBytes));
 const brand=cssBytes.toString().match(/--color-brand: (#[a-f0-9]+);/)[1];
 const svg=`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 120"><rect width="320" height="120" fill="${brand}"/><text x="24" y="76" fill="white" font-family="Noto Sans" font-size="40">Mirket</text></svg>`;
 const vector=await design('art.svg',svg,'art.min.svg',{kind:'svg_optimize'});assert.equal(vector.result.status,'succeeded',JSON.stringify(vector.result));assert.match((await fs.readFile(vector.output)).toString(),/<svg/);
 const unsafe=await design('unsafe.svg','<svg xmlns="http://www.w3.org/2000/svg"><image href="https://example.test/remote.png"/></svg>','unsafe.min.svg',{kind:'svg_optimize'});assert.equal(unsafe.result.status,'failed');assert.equal(await fs.stat(unsafe.output).then(()=>true,()=>false),false);
 const png=await design('raster.svg',svg,'art.png',{kind:'svg_render_png',width:640,height:240});assert.equal(png.result.status,'succeeded',JSON.stringify(png.result));const pixels=await fs.readFile(png.output);assert.equal(pixels.readUInt32BE(16),640);assert.equal(pixels.readUInt32BE(20),240);assert.equal(png.result.artifact_revision,revision(pixels));
 const changedPng=await design('changed.svg',svg.replace(brand,'#b42318'),'changed.png',{kind:'svg_render_png',width:640,height:240});assert.equal(changedPng.result.status,'succeeded',JSON.stringify(changedPng.result));assert.notDeepEqual(await fs.readFile(changedPng.output),pixels);
 await fs.writeFile(join(root,'domain-report-'+runId+'.json'),JSON.stringify({work,evidence,images:[png.output,changedPng.output],limits:['Saved pixels require separate image-capable inspection; image delivery to a pi model and video export not established']},null,2));
 await save('pi token compilation and SVG publication produce verified files; remote-asset SVG refuses without output');
 await save('pi saved raster exports bind revisions; changed SVG produces different PNG bytes (pixel inspection separate)');
}
