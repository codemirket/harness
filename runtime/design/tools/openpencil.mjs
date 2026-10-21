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
export function validateOpenPencilTool(name, args, effect) {
  const tool = tools.get(name);
  if (!tool || tool.effect !== effect) throw Error(`Unsupported OpenPencil ${effect} tool: ${name}`);
  if (!validators.get(name)(args)) throw Error(`Invalid OpenPencil ${name} arguments: ${ajv.errorsText(validators.get(name).errors)}`);
  return tool;
}
const metadata = {engine:'OpenPencil', version:manifest.version, mode:'headless-file'};
async function cli(stage, args, json = false) {
  // Node's permission model limits upstream file access to immutable installed
  // dependencies/fonts and this operation's scratch directory. No caller paths
  // are passed to the CLI. Child processes/addons/worker threads remain denied.
  const result = await run(process.execPath, ['--permission',
    `--allow-fs-read=${root}`, '--allow-fs-read=/usr/share/fonts',
    `--allow-fs-read=${stage}`, `--allow-fs-write=${stage}`,
    path.join(root,'openpencil-cli.mjs'), ...args], {cwd:stage,timeoutMs:90000,maxStdout:1024*1024});
  if (!json) return result.stdout;
  const value=JSON.parse(result.stdout);
  if (value?.error || value?.success === false || (Array.isArray(value?.errors) && value.errors.length)) throw Error(`OpenPencil operation failed: ${JSON.stringify(value)}`);
  return value;
}
async function staged(source, fn) {
  const stage = await temporaryDirectory('openpencil');
  try {
    let input;
    if (source) {
      if (!/\.(fig|pen)$/.test(source)) throw Error('OpenPencil input must be a .fig or .pen document');
      input = path.join(stage,`input${path.extname(source)}`);
      await fs.writeFile(input,await readFile(source),{mode:0o600});
    }
    return await fn(stage,input);
  } finally { await fs.rm(stage,{recursive:true,force:true}); }
}
async function tokensFor(args) {
  const compiled = await compileTokens(args.tokens || process.env.MIRKET_DESIGN_TOKENS || 'design/tokens.json');
  return {compiled, text:(await readFile(compiled.outputs['tokens.dtcg.json'],512*1024)).toString('utf8')};
}
async function synchronize(stage,input,tokens) {
  const output=path.join(stage,'tokens.fig');
  const imported = await cli(stage,['tool','call','import_design_tokens',input,'--args',JSON.stringify({files:[{path:'Mirket/tokens.json',text:tokens.text}]}),'--output',output,'--json'],true);
  if (!Array.isArray(imported.skipped) || !Array.isArray(imported.issues) || imported.skipped.length || imported.issues.length) throw Error(`OpenPencil could not import all canonical tokens: ${JSON.stringify(imported)}`);
  tokens.imported = imported;
  return output;
}
export async function openpencilInspect(args) {
  if (!args.source) {
    if (args.tool) {
      const tool=tools.get(args.tool); if(!tool) throw Error('Unsupported OpenPencil tool');
      return {...metadata,tool};
    }
    return {...metadata,tools:manifest.tools.map(({name,effect,description})=>({name,effect,description}))};
  }
  const name=args.tool || 'get_page_tree';
  validateOpenPencilTool(name,args.arguments || {},'read');
  return staged(args.source,async(stage,input)=>({...metadata,source:args.source,source_sha256:sha256(await regularOutput(input)),
    result:await cli(stage,['tool','call',name,input,'--args',JSON.stringify(args.arguments || {}),'--json'],true)}));
}
export async function openpencilDocument(args) {
  if (!args.output.endsWith('.fig')) throw Error('Editable OpenPencil output must end in .fig');
  await assertOutputReady(args);
  const tokens=await tokensFor(args);
  return staged(args.source,async(stage,input)=>{
    if(!input) {input=path.join(stage,'blank.pen');await fs.writeFile(input,JSON.stringify({version:'2.7',children:[]}));}
    const file=await synchronize(stage,input,tokens);
    const artifact=await publish(args.output,await regularOutput(file),args);
    return {...metadata,artifact,tokens:tokens.compiled,token_import:tokens.imported,qa:{status:'passed',scope:'Editable file written; inspect exported pixels before acceptance'}};
  });
}
export async function openpencilEdit(args) {
  if (!args.output.endsWith('.fig')) throw Error('Editable OpenPencil output must end in .fig');
  validateOpenPencilTool(args.tool,args.arguments || {},'write');
  await assertOutputReady(args);
  const tokens=await tokensFor(args);
  return staged(args.source,async(stage,input)=>{
    const sourceHash=sha256(await regularOutput(input));
    const output=path.join(stage,'edited.fig');
    const operationResult=await cli(stage,['tool','call',args.tool,input,'--args',JSON.stringify(args.arguments || {}),'--output',output,'--json'],true);
    // .fig serialization remaps IDs. Return the reopened tree as the usable
    // result; pre-save IDs from an upstream mutation must never be reused.
    const result=await cli(stage,['tool','call','get_page_tree',output,'--args','{}','--json'],true);
    const artifact=await publish(args.output,await regularOutput(output),args);
    return {...metadata,artifact,result,operation_result:operationResult,id_notice:'Use IDs from result (the saved document), not operation_result (before serialization).',source:args.source,source_sha256:sourceHash,tokens:tokens.compiled,qa:{status:'passed',scope:'Editable file written; inspect exported pixels before acceptance'}};
  });
}
export async function openpencilExport(args) {
  const format=args.format || 'png';
  if(!['png','svg'].includes(format)||!args.output.endsWith(`.${format}`)) throw Error('OpenPencil export format and extension must match');
  await assertOutputReady(args);
  return staged(args.source,async(stage,input)=>{
    const output=path.join(stage,`export.${format}`);
    // Export the saved document unchanged.
    await cli(stage,['export',input,'--output',output,'--format',format,'--scale','2',...(args.node?['--node',args.node]:[]),...(args.page?['--page',args.page]:[])]);
    const sourceHash=sha256(await regularOutput(input));
    let bytes=await regularOutput(output);
    if(format==='svg') bytes=(await import('./operations.mjs')).optimizeSvgBytes(bytes.toString('utf8'));
    let dimensions;
    if(format==='png') {
      if(bytes.length<24||!bytes.subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10])))throw Error('Invalid OpenPencil PNG');
      dimensions={width:bytes.readUInt32BE(16),height:bytes.readUInt32BE(20)};
      if(!dimensions.width||!dimensions.height||dimensions.width*dimensions.height>16*1024*1024)throw Error('OpenPencil PNG exceeds pixel budget');
    }
    const artifact=await publish(args.output,bytes,args);
    return {...metadata,artifact,source:args.source,source_sha256:sourceHash,dimensions,
      qa:{status:'passed',scope:'File exported; independent pixel inspection is required'}};
  });
}
