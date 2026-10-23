import fs from 'node:fs/promises';
import {readFileSync} from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import Ajv from 'ajv';
import {compileTokens} from './tokens.mjs';
import {readFile, publish, run, temporaryDirectory, regularOutput, sha256, assertOutputReady} from './paths.mjs';
const root = path.dirname(fileURLToPath(import.meta.url));
const manifest = JSON.parse(readFileSync(new URL('./openpencil-tools.json', import.meta.url)));
const tools = new Map(manifest.tools.map(tool => [tool.name, tool]));
const ajv = new Ajv({strict:false, allErrors:true});
const validators = new Map(manifest.tools.map(tool => [tool.name, ajv.compile(tool.schema)]));
export const OPENPENCIL_LIMITS = Object.freeze({operations:64, request_bytes:1024*1024, output_bytes:64*1024*1024, readback_bytes:1024*1024, timeout_ms:90000, heap_mb:512});
export function validateOpenPencilTool(name, args, effect) {
  const tool = tools.get(name);
  if (!tool || tool.effect !== effect) throw Error(`Unsupported OpenPencil ${effect} tool: ${name}`);
  if (!validators.get(name)(args)) throw Error(`Invalid OpenPencil ${name} arguments: ${ajv.errorsText(validators.get(name).errors)}`);
  return tool;
}
const batchProps = {};
for (const name of ['update_node','set_layout','set_layout_child','set_text_properties']) Object.assign(batchProps,tools.get(name).schema.properties);
delete batchProps.id;
// Layout and text tools share this key with different enums. Preserve both
// instead of silently replacing layout directions during the property merge.
batchProps.direction = {anyOf:[tools.get('set_layout').schema.properties.direction,tools.get('set_text_properties').schema.properties.direction]};
const validateUpdates = ajv.compile({type:'array',minItems:1,maxItems:64,items:{type:'object',additionalProperties:false,required:['id','props'],properties:{id:{type:'string',minLength:1},props:{type:'object',additionalProperties:false,properties:batchProps}}}});
export function validateOpenPencilBatch(operations) {
  if (!Array.isArray(operations) || !operations.length || operations.length > OPENPENCIL_LIMITS.operations) throw Error('OpenPencil batch requires 1–64 operations');
  const text=JSON.stringify(operations);
  if (Buffer.byteLength(text)>OPENPENCIL_LIMITS.request_bytes) throw Error('OpenPencil batch exceeds request byte budget');
  // Snapshot JSON data before asynchronous work; callers cannot mutate it after validation.
  const copy=JSON.parse(text);
  let count=0;
  for (const operation of copy) {
    if (!operation || typeof operation!=='object' || Array.isArray(operation) || Object.keys(operation).some(key=>!['tool','arguments'].includes(key)) || !Object.hasOwn(operation,'arguments')) throw Error('Invalid OpenPencil batch operation');
    validateOpenPencilTool(operation.tool,operation.arguments,'write');
    count++;
    if (operation.tool==='batch_update') {
      const updates=JSON.parse(operation.arguments.operations);
      if (!validateUpdates(updates)) throw Error(`Invalid OpenPencil batch_update operations: ${ajv.errorsText(validateUpdates.errors)}`);
      count+=updates.length-1;
    }
    if (count>OPENPENCIL_LIMITS.operations) throw Error('OpenPencil batch exceeds total operation budget');
  }
  return copy;
}
export function assertOpenPencilSuccess(value) {
  if (value?.error || value?.success===false || (Array.isArray(value?.errors)&&value.errors.length) || (Array.isArray(value?.issues)&&value.issues.length) || (Array.isArray(value?.skipped)&&value.skipped.length)) throw Error('OpenPencil operation failed; staged document was not published');
  return value;
}
const metadata = {engine:'OpenPencil', version:manifest.version, mode:'headless-file', contract_version:1};
function evidence(artifact,kind) {
  return {contract_version:1,artifact_sha256:artifact.sha256,local:{status:'passed',scope:kind},figma:{status:'unverified',reason:'No Figma consumer fixture was executed; local serialization/export is not Figma acceptance.'},visual:{status:'unverified',reason:'Independent pixel inspection required'},native_media:{browser:'DOM/CSS',vector:'SVG',paged:'Typst/PDF'}};
}
async function cli(stage,args,json=false,deadline=performance.now()+OPENPENCIL_LIMITS.timeout_ms) {
  const remaining=Math.floor(deadline-performance.now());
  if(remaining<=0) throw Error('OpenPencil transaction deadline exceeded');
  const result=await run(process.execPath,['--max-old-space-size=512','--permission',`--allow-fs-read=${root}`,'--allow-fs-read=/usr/share/fonts',`--allow-fs-read=${stage}`,`--allow-fs-write=${stage}`,path.join(root,'openpencil-cli.mjs'),...args],{cwd:stage,timeoutMs:remaining,maxStdout:OPENPENCIL_LIMITS.readback_bytes});
  return json ? assertOpenPencilSuccess(JSON.parse(result.stdout)) : result.stdout;
}
async function staged(source,fn) {
  const stage=await temporaryDirectory('openpencil');
  try {
    let input;
    if(source) {
      if(!/\.(fig|pen)$/.test(source)) throw Error('OpenPencil input must be a .fig or .pen document');
      input=path.join(stage,`input${path.extname(source)}`);
      await fs.writeFile(input,await readFile(source),{mode:0o600});
    }
    return await fn(stage,input);
  } finally {await fs.rm(stage,{recursive:true,force:true});}
}
async function tokensFor(args) {
  const compiled=await compileTokens(args.tokens || process.env.MIRKET_DESIGN_TOKENS || 'design/tokens.json');
  return {compiled,text:(await readFile(compiled.outputs['tokens.dtcg.json'],512*1024)).toString('utf8')};
}
async function transaction(stage,input,operations,deadline) {
  const output=path.join(stage,'edited.fig');
  const request=path.join(stage,'request.json');
  await fs.writeFile(request,JSON.stringify(validateOpenPencilBatch(operations)),{mode:0o600});
  const operationResult=await cli(stage,['transaction',input,output,request],true,deadline);
  // Never publish a file whose reopened tree could not be returned in full.
  const result=await cli(stage,['tool','call','get_page_tree',output,'--args','{}','--json'],true,deadline);
  const bytes=await regularOutput(output,OPENPENCIL_LIMITS.output_bytes);
  if(performance.now()>=deadline) throw Error('OpenPencil transaction deadline exceeded');
  return {bytes,result,operationResult};
}
export async function openpencilInspect(args) {
  if(!args.source) {
    if(args.tool) {const tool=tools.get(args.tool);if(!tool)throw Error('Unsupported OpenPencil tool');return {...metadata,tool};}
    return {...metadata,batch:manifest.batch,limits:OPENPENCIL_LIMITS,tools:manifest.tools.map(({name,effect,description})=>({name,effect,description}))};
  }
  const name=args.tool || 'get_page_tree';
  validateOpenPencilTool(name,args.arguments || {},'read');
  return staged(args.source,async(stage,input)=>({...metadata,source:args.source,source_sha256:sha256(await regularOutput(input)),result:await cli(stage,['tool','call',name,input,'--args',JSON.stringify(args.arguments || {}),'--json'],true)}));
}
export async function openpencilDocument(args) {
  if(!args.output.endsWith('.fig'))throw Error('Editable OpenPencil output must end in .fig');
  if(args.source===args.output)throw Error('Imports must preserve the original; use a distinct output');
  await assertOutputReady(args);
  const tokens=await tokensFor(args);
  const deadline=performance.now()+OPENPENCIL_LIMITS.timeout_ms;
  return staged(args.source,async(stage,input)=>{
    const sourceHash=input?sha256(await regularOutput(input)):undefined;
    if(!input){input=path.join(stage,'blank.pen');await fs.writeFile(input,JSON.stringify({version:'2.7',children:[]}));}
    const saved=await transaction(stage,input,[{tool:'import_design_tokens',arguments:{files:[{path:'Mirket/tokens.json',text:tokens.text}]}}],deadline);
    const imported=saved.operationResult.operation_results[0];
    if(!Array.isArray(imported.skipped)||!Array.isArray(imported.issues))throw Error('OpenPencil token import receipt is missing');
    const artifact=await publish(args.output,saved.bytes,args);
    return {...metadata,artifact,result:saved.result,source:args.source,source_sha256:sourceHash,tokens:tokens.compiled,token_import:imported,evidence:evidence(artifact,'editable-document-reopened'),qa:{status:'passed',scope:'Local editable document reopened; consumer and visual acceptance remain unverified'}};
  });
}
export async function openpencilEdit(args) {
  if(!args.output.endsWith('.fig'))throw Error('Editable OpenPencil output must end in .fig');
  if(!args.source)throw Error('OpenPencil edit requires a source');
  if(Object.hasOwn(args,'tool')||Object.hasOwn(args,'arguments'))throw Error('Use operations:[{tool,arguments}] for an OpenPencil batch');
  const operations=validateOpenPencilBatch(args.operations);
  await assertOutputReady(args);
  // Compile once; canonical token import is an operation in the same transaction,
  // not a separately serialized revision with remapped IDs.
  const tokens=await tokensFor(args);
  const all=validateOpenPencilBatch([{tool:'import_design_tokens',arguments:{files:[{path:'Mirket/tokens.json',text:tokens.text}]}},...operations]);
  const deadline=performance.now()+OPENPENCIL_LIMITS.timeout_ms;
  return staged(args.source,async(stage,input)=>{
    const sourceHash=sha256(await regularOutput(input));
    const saved=await transaction(stage,input,all,deadline);
    const artifact=await publish(args.output,saved.bytes,args);
    return {...metadata,artifact,result:saved.result,operation_results:saved.operationResult.operation_results.slice(1),id_notice:'Use IDs from result (reopened saved document); operation_results contain transient pre-save IDs.',source:args.source,source_sha256:sourceHash,tokens:tokens.compiled,limits:OPENPENCIL_LIMITS,evidence:evidence(artifact,'editable-document-reopened'),qa:{status:'passed',scope:'Batch published after reopened readback; consumer and visual acceptance remain unverified'}};
  });
}
export async function openpencilExport(args) {
  const format=args.format || 'png';
  if(!['png','svg'].includes(format)||!args.output.endsWith(`.${format}`))throw Error('OpenPencil export format and extension must match');
  if(args.consumer!==undefined&&!['local','figma'].includes(args.consumer))throw Error('Unsupported OpenPencil consumer');
  await assertOutputReady(args);
  return staged(args.source,async(stage,input)=>{
    const output=path.join(stage,`export.${format}`);
    await cli(stage,['export',input,'--output',output,'--format',format,'--scale','2',...(args.node?['--node',args.node]:[]),...(args.page?['--page',args.page]:[])]);
    const sourceHash=sha256(await regularOutput(input));
    let bytes=await regularOutput(output);
    if(format==='svg')bytes=(await import('./operations.mjs')).optimizeSvgBytes(bytes.toString('utf8'));
    let dimensions;
    if(format==='png'){
      if(bytes.length<24||!bytes.subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10])))throw Error('Invalid OpenPencil PNG');
      dimensions={width:bytes.readUInt32BE(16),height:bytes.readUInt32BE(20)};
      if(!dimensions.width||!dimensions.height||dimensions.width*dimensions.height>16*1024*1024)throw Error('OpenPencil PNG exceeds pixel budget');
    }
    const artifact=await publish(args.output,bytes,args);
    return {...metadata,artifact,source:args.source,source_sha256:sourceHash,dimensions,consumer:args.consumer || 'local',evidence:evidence(artifact,`local-${format}-export`),qa:{status:args.consumer==='figma'?'unverified':'passed',scope:'Local export only; no Figma consumer acceptance or independent pixel inspection'}};
  });
}
