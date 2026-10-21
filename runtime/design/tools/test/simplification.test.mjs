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
