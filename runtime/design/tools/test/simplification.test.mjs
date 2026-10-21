// Bounded replacement-contract probes: a shared renderer is not automatically
// an equivalent authoring/export interface. Real dependencies, no model calls.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
const workspace=await fs.mkdtemp(path.join(os.tmpdir(),'mirket-simplification-'));
process.env.MIRKET_DESIGN_WORKSPACE=workspace;
const {svgOptimize,vectorRender,staticRender,typstCompile}=await import('../operations.mjs');
test.after(()=>fs.rm(workspace,{recursive:true,force:true}));

async function evidence(name, artifact) {
  if(process.env.MIRKET_ISOLATED_TEST==='1') console.log('SIMPLIFICATION_EVIDENCE '+JSON.stringify({name,png:(await fs.readFile(path.join(workspace,artifact))).toString('base64')}));
}

test('standalone SVG optimization preserves a useful input outside the vector preview limit',async()=>{
  const source='<svg xmlns="http://www.w3.org/2000/svg" width="80" height="40"><!--'+ 'x'.repeat(2*1024*1024) +'--><g id="retained"><rect width="40" height="40" fill="#123456"/></g><g id="hidden" display="none"><circle r="8"/></g></svg>';
  await fs.writeFile(path.join(workspace,'large.svg'),source);
  await svgOptimize({source:'large.svg',output:'optimized.svg'});
  assert.match(await fs.readFile(path.join(workspace,'optimized.svg'),'utf8'),/id="hidden"/);
  await assert.rejects(vectorRender({source:'large.svg',output:'direct.svg'}),/limit|large|bound|exceed/i);
  const rendered=await vectorRender({source:'optimized.svg',output:'preview.svg',width:160,height:80});
  await evidence('large-svg-after-optimization',rendered.preview.path);
});

test('Satori supplies wrapping layout while authored SVG retains a distinct vector contract',async()=>{
  await fs.mkdir(path.join(workspace,'design'));
  await fs.writeFile(path.join(workspace,'design/tokens.json'),JSON.stringify({color:{accent:{$type:'color',$value:'#123456'}}}));
  await fs.copyFile('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf',path.join(workspace,'font.ttf'));
  const cases=[
    {name:'wrapping',text:'A long heading wraps into readable lines inside this narrow card.'},
    {name:'multilingual',text:'İstanbul — Größe — información'},
    {name:'data-card',text:'Queue capacity 24 · Accepted 18 · Rejected 6'},
  ];
  for(const {name,text} of cases) {
    const result=await staticRender({width:360,height:200,fonts:[{path:'font.ttf',name:'DejaVu'}],output:`${name}.png`,tree:{type:'div',props:{style:{width:360,height:200,padding:24,backgroundColor:'#f4f1ea',color:'#123456',fontSize:24,fontFamily:'DejaVu'}},children:[text]}});
    assert.equal(result.engine,'satori+resvg-js');
    assert.equal(result.qa.width,360);assert.equal(result.qa.height,200);
    await evidence(name,result.artifact.path);
  }
});

test('paged output retains two pages rather than passing on a PDF header alone',async t=>{
  try {await fs.access('/usr/local/bin/typst');} catch(error) {if(process.env.MIRKET_ISOLATED_TEST==='1')throw error;t.skip('Typst requires the explicit design runtime');return;}
  await fs.mkdir(path.join(workspace,'paged'));
  await fs.writeFile(path.join(workspace,'paged/document.typ'),'#set page(width: 180pt, height: 120pt, margin: 12pt)\n#text(size: 18pt)[First page]\n#pagebreak()\n#text(size: 18pt)[Second page]');
  const result=await typstCompile({source:'paged/document.typ',output:'pages.pdf'});
  assert.equal(result.pages.length,2);
  for(const [i,page] of result.pages.entries())await evidence(`typst-page-${i+1}`,page.path);
});

// Exercise the actual protocol boundary: no retired renderer or disabled format dispatch.
test('retired video and unselected Excalidraw fail explicitly without changing saved artifacts',async()=>{
  const {Client}=await import('@modelcontextprotocol/sdk/client/index.js');
  const {StdioClientTransport}=await import('@modelcontextprotocol/sdk/client/stdio.js');
  await fs.writeFile(path.join(workspace,'retained-scene.html'),'user scene');
  await fs.writeFile(path.join(workspace,'retained.mp4'),'user output');
  const client=new Client({name:'retirement-test',version:'1'});
  try {
    await client.connect(new StdioClientTransport({command:process.execPath,args:[new URL('../server.mjs',import.meta.url).pathname],env:{PATH:process.env.PATH,HOME:workspace,MIRKET_DESIGN_WORKSPACE:workspace,MIRKET_DESIGN_GROUPS:'core,diagrams'},stderr:'pipe'}));
    const listed=await client.listTools();
    assert.equal(listed.tools.length,13);
    assert.ok(!listed.tools.some(t=>t.name==='hyperframes_render'));
    for (const name of ['constructor', '__proto__', 'toString', 'execute_code']) {
      const unknown=await client.callTool({name,arguments:{output:'absent.svg'}});
      assert.equal(unknown.isError,true);
      assert.match(JSON.stringify(unknown),/Unknown design tool/);
    }
    const retired=await client.callTool({name:'hyperframes_render',arguments:{project:'retained-scene.html',output:'retained.mp4'}});
    assert.equal(retired.isError,true);
    assert.match(JSON.stringify(retired),/retired.*host encoder/);
    const companion=await client.callTool({name:'kroki_render',arguments:{source:'missing.excalidraw',diagram_type:'excalidraw',output:'absent.svg'}});
    assert.equal(companion.isError,true);
    assert.equal(companion.structuredContent.execution.state,'not_started');
    assert.equal(companion.structuredContent.execution.code,'disabled_group');
    assert.match(JSON.stringify(companion),/--group excalidraw/);
    assert.equal(await fs.readFile(path.join(workspace,'retained-scene.html'),'utf8'),'user scene');
    assert.equal(await fs.readFile(path.join(workspace,'retained.mp4'),'utf8'),'user output');
    await assert.rejects(fs.access(path.join(workspace,'absent.svg')));
  } finally {await client.close();}
});


test('completed negative QA remains inspectable and supports a corrected render',async()=>{
  const {Client}=await import('@modelcontextprotocol/sdk/client/index.js');
  const {StdioClientTransport}=await import('@modelcontextprotocol/sdk/client/stdio.js');
  const {createHash}=await import('node:crypto');
  const sharp=(await import('sharp')).default;
  await sharp({create:{width:80,height:40,channels:4,background:'#ffffff'}}).png().toFile(path.join(workspace,'qa-baseline.png'));
  await fs.writeFile(path.join(workspace,'qa-source.svg'),'<svg xmlns="http://www.w3.org/2000/svg" width="80" height="40"><rect width="80" height="40" fill="#123456"/></svg>');
  const client=new Client({name:'qa-repair-test',version:'1'});
  try {
    await client.connect(new StdioClientTransport({command:process.execPath,args:[new URL('../server.mjs',import.meta.url).pathname],env:{PATH:process.env.PATH,HOME:workspace,MIRKET_DESIGN_WORKSPACE:workspace},stderr:'pipe'}));
    const args={source:'qa-source.svg',output:'qa-output.svg',width:80,height:40,baseline:'qa-baseline.png'};
    const failed=await client.callTool({name:'vector_render',arguments:args});
    assert.equal(failed.isError,true);
    assert.equal(failed.structuredContent.qa.status,'changed_pixels');
    assert.deepEqual(failed.structuredContent.execution,{tool:'vector_render',state:'completed'});
    for (const receipt of [failed.structuredContent.artifact,failed.structuredContent.preview]) {
      const bytes=await fs.readFile(path.join(workspace,receipt.path));
      assert.equal(createHash('sha256').update(bytes).digest('hex'),receipt.sha256);
    }
    await fs.writeFile(path.join(workspace,'qa-source.svg'),'<svg xmlns="http://www.w3.org/2000/svg" width="80" height="40"><rect width="80" height="40" fill="#ffffff"/></svg>');
    const repaired=await client.callTool({name:'vector_render',arguments:{...args,overwrite:true,expected_sha256:failed.structuredContent.artifact.sha256}});
    assert.equal(repaired.isError,false);
    assert.equal(repaired.structuredContent.qa.status,'match');
    const invalid=await client.callTool({name:'vector_render',arguments:{...args,unknown:true}});
    assert.equal(invalid.structuredContent.execution.state,'not_started');
    assert.equal(invalid.structuredContent.execution.code,'invalid_arguments');
  } finally {await client.close();}
});
