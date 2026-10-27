// Explicit development/assembly step, never an npm lifecycle hook. Assemble
// exact lock-bound upstream bytes or verify an unchanged prior assembly.
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import ts from 'typescript';
const root = path.resolve(import.meta.dirname, '..');
const inputs = JSON.parse(fs.readFileSync(path.join(import.meta.dirname, 'video-policy-inputs.json'), 'utf8'));
const hash = (text) => createHash('sha256').update(text).digest('hex');
const modificationNotice = '// Modified by Mirket: private video launch, network and source-ownership policy hooks.\n';
const receiptPath = path.join(root, 'node_modules/.mirket-video-policy.json');
const previous = fs.existsSync(receiptPath) ? JSON.parse(fs.readFileSync(receiptPath, 'utf8')) : null;
const planned = [];
for (const input of inputs.files) {
  const file = path.join(root, 'node_modules', input.path);
  const original = fs.readFileSync(file, 'utf8');
  if (hash(original) !== input.sha256) {
    const prior = previous?.schema === 1 && previous.policy_version === 1 && previous.development_candidate === true
      && Array.isArray(previous.files) && previous.files.find(item => item.path === input.path);
    if (!prior || prior.original_sha256 !== input.sha256 || prior.patched_sha256 !== hash(original)
      || !original.startsWith(modificationNotice)) throw new Error(`Exact upstream input differs: ${input.path}`);
    planned.push({file,source:original,path:input.path,original_sha256:input.sha256,patched_sha256:prior.patched_sha256});
    continue;
  }
  let source = original;
  const imports = new Set();
  if (input.fetch) {
    const ast = ts.createSourceFile(file, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.JS);
    const offsets = [];
    const visit = (node) => {
      if (ts.isCallExpression(node) && ts.isIdentifier(node.expression) && node.expression.text === 'fetch') offsets.push(node.expression.getStart(ast));
      ts.forEachChild(node, visit);
    };
    visit(ast);
    if (offsets.length !== input.fetch) throw new Error(`Fetch call count differs: ${input.path}`);
    for (const at of offsets.sort((a,b)=>b-a)) source = source.slice(0,at) + 'mirketVideoFetch' + source.slice(at+5);
    imports.add('mirketVideoFetch');
  }
  if (input.launch) {
    // Wrap exact object argument without rewriting vendor logic or a runtime
    // monkey patch. The pinned AST supplies the matching object extent.
    const ast = ts.createSourceFile(file, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.JS);
    const calls = [];
    const visit = node => { if (ts.isCallExpression(node) && ts.isPropertyAccessExpression(node.expression) && node.expression.expression.getText(ast)==='ppt' && node.expression.name.text==='launch' && node.arguments.length===1) calls.push(node.arguments[0]); ts.forEachChild(node,visit); };
    visit(ast);
    if(calls.length!==input.launch)throw new Error('Browser launch AST differs');
    for(const arg of calls.sort((a,b)=>b.getStart(ast)-a.getStart(ast)))source=source.slice(0,arg.getStart(ast))+'mirketLaunchOptions('+source.slice(arg.getStart(ast),arg.end)+')'+source.slice(arg.end);
    imports.add('mirketLaunchOptions');
  }
  if (input.pages) {
    for (const [before, after] of [
      ['    onPageCreated(page);','    onPageCreated(page);\n    await mirketPreparePage(page, serverUrl);'],
      ['    const page = await session.browser.newPage();','    const page = await session.browser.newPage();\n    await mirketPreparePage(page, session.serverUrl);'],
    ]) {
      if(source.split(before).length!==2)throw new Error('Page hook input differs');source=source.replace(before,after);
    }
    imports.add('mirketPreparePage');
  }
  if(input.detached){
    const before="opts.detached ??= process.platform !== 'win32';";
    if(source.split(before).length!==2)throw new Error('Puppeteer ownership input differs');
    source=source.replace(before,"opts.detached = process.env.MIRKET_VIDEO_POLICY_VERSION === '1' ? false : (opts.detached ?? process.platform !== 'win32');");
  }
  if(input.server){
    for(const [before,after] of [
      ['if (isPathInside(candidate, root)) file = openRegularFile(candidate);','if (mirketSourceFile(candidate, root)) file = openRegularFile(candidate);'],
      ['url: `http://localhost:${info.port}`','url: `http://127.0.0.1:${info.port}`'],
      ['      resolve12({\n        url: `http://127.0.0.1:${info.port}`','      mirketRegisterServer(`http://127.0.0.1:${info.port}`);\n      resolve12({\n        url: `http://127.0.0.1:${info.port}`'],
    ]){if(source.split(before).length!==2)throw new Error('Private file server input differs');source=source.replace(before,after);}
    imports.add('mirketSourceFile');
    imports.add('mirketRegisterServer');
  }
  if(input.fontFetch){
    const before='const fetchImpl = options.fetchImpl ?? fetch;';
    if(source.split(before).length!==2)throw new Error('Font fetch input differs');
    source=source.replace(before,'const fetchImpl = options.fetchImpl ?? mirketVideoFetch;');
    imports.add('mirketVideoFetch');
  }
  if(imports.size){let relative=path.relative(path.dirname(file),path.join(root,'dist/video-policy.js')).split(path.sep).join('/');if(!relative.startsWith('.'))relative='./'+relative;source=`import { ${[...imports].join(', ')} } from ${JSON.stringify(relative)};\n`+source;}
  source=modificationNotice+source;
  planned.push({file,source,path:input.path,original_sha256:input.sha256,patched_sha256:hash(source)});
}
// Validate every input before the first mutation; a missing/different input
// preserves the entire unmodified candidate for inspection.
for(const item of planned)if(fs.readFileSync(item.file,'utf8')!==item.source)fs.writeFileSync(item.file,item.source);
const receipt={schema:1,policy_version:1,development_candidate:true,files:planned.map(({path,original_sha256,patched_sha256})=>({path,original_sha256,patched_sha256}))};
const receiptText=JSON.stringify(receipt,null,2)+'\n';
if(!fs.existsSync(receiptPath)||fs.readFileSync(receiptPath,'utf8')!==receiptText)fs.writeFileSync(receiptPath,receiptText);
process.stdout.write(JSON.stringify({policy_version:1,patched_files:planned.length,release_admitted:false})+'\n');
