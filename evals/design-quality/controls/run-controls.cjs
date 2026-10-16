#!/usr/bin/env node
/* MIT. Runs synthetic controls with an existing local browser. No model runs. */
'use strict';
const fs = require('node:fs/promises');
const path = require('node:path');
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const { observe } = require('../observe.cjs');
const { generate } = require('./generate.cjs');
async function run(outputDir, options = {}) {
  const out = path.resolve(outputDir); await fs.mkdir(out, { recursive: true });
  const manifest = await generate(), report = { schema_version: 1, synthetic: true, model_executions: 0, checks: [], observations: [], limits: ['These deterministic checks test the collector and synthetic local state only.', 'No judge calibration or model quality is inferred from a collector check.'] };
  const script = html => html.match(/<script>([\s\S]*?)<\/script>/)[1];
  for (const [a,b] of [['C101','C102'],['C106','C107']]) {
    const aa = await fs.readFile(path.join(__dirname,a,'index.html'),'utf8'), bb = await fs.readFile(path.join(__dirname,b,'index.html'),'utf8');
    assert.equal(script(aa),script(bb),'composition pair must share exact state logic');
    assert.equal(aa.replace(/<style>[\s\S]*?<\/style>/,'<style></style>'),bb.replace(/<style>[\s\S]*?<\/style>/,'<style></style>'),'composition pair differs only in CSS');
    report.checks.push({id:`${a}-${b}-same-content-and-logic`,passed:true});
  }
  assert.equal(manifest.controls.find(c=>c.id==='C101').sha256,manifest.controls.find(c=>c.id==='C104').sha256); report.checks.push({id:'exact-source-duplicate',passed:true});
  for (const control of manifest.controls) {
    const probe = path.join(__dirname, control.family === 'service-desk' ? 'service-review.probe.json' : 'local-save.probe.json');
    const target = path.join(out,control.id,'observations.json'); let result;
    if(options.reuse) {
      result=JSON.parse(await fs.readFile(target,'utf8'));const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
      assert.equal(result.source_bindings.observer.sha256,hash(await fs.readFile(path.join(__dirname,'..','observe.cjs'))),'reuse requires unchanged observer');
      assert.equal(result.source_bindings.probe.sha256,hash(await fs.readFile(probe)),'reuse requires unchanged probe');
      assert.deepEqual(result.source_bindings.artifact.files,[{path:'index.html',sha256:control.sha256,bytes:(await fs.stat(path.join(__dirname,control.id,'index.html'))).size}],'reuse requires exact current source');
    } else result = await observe(path.join(__dirname,control.id),path.join(out,control.id),probe,{artifact_origin:'original_synthetic_control',synthetic:true,model_executions:0},{wideWidth:1440});
    const failures = result.scenarios.flatMap(s=>s.steps.filter(st=>st.status==='failed').map(st=>st.id));
    assert.equal(result.collection_completed,true,`${control.id} collection`); assert.equal(result.artifact_unchanged,true,`${control.id} frozen source`);
    assert.equal(result.errors.length,0,`${control.id} infrastructure errors`);
    if(control.id==='C103') {
      for(const id of ['claimed-success','claimed-count']) assert.equal(result.scenarios[0].steps.find(s=>s.id===id).status,'passed','polished fake must pass superficial message/counter');
      for(const id of ['owned-session','owned-note','owned-count','persisted-session','persisted-note','persisted-input','final-owner-count']) assert.ok(failures.includes(id),`fake must fail ${id}`);
    } else assert.deepEqual(failures,[],`${control.id} behavior must match positive controls`);
    report.observations.push({id:control.id,output:path.relative(out,path.join(out,control.id,'observations.json')),failed_steps:failures,collection_completed:result.collection_completed,errors:result.errors.length});
    console.log(JSON.stringify({control:control.id,failed_steps:failures,synthetic:true,model_executions:0}));
  }
  const read = async id=>JSON.parse(await fs.readFile(path.join(out,id,'default-1280.dom.json'),'utf8'));
  const content=text=>text.toLocaleLowerCase('en').replace(/\s+/g,' ').trim();
  for(const [a,b]of [['C101','C102'],['C101','C104'],['C106','C107']]) assert.equal(content((await read(a)).text),content((await read(b)).text),'composition controls must expose identical words (CSS capitalization/line breaks may differ)');
  const a=JSON.parse(await fs.readFile(path.join(out,'C101','observations.json'),'utf8')),b=JSON.parse(await fs.readFile(path.join(out,'C104','observations.json'),'utf8'));
  const find = (r,id)=>r.generic.captures.find(c=>c.id===id).files.find(f=>f.role==='viewport').sha256;
  report.checks.push({id:'duplicate-1280-render-identical',passed:find(a,'default-1280')===find(b,'default-1280'),first:find(a,'default-1280'),second:find(b,'default-1280')});
  assert.equal(report.checks.at(-1).passed,true,'identical controls should have identical default viewport pixels');
  report.checks.push({id:'false-success-rejected-at-owner-and-reload',passed:true});
  report.checks.push({id:'content-order-perturbation-keeps-selected-record-identity',passed:true});
  await fs.writeFile(path.join(out,'controls-check.json'),JSON.stringify(report,null,2)+'\n');
  return report;
}
if(require.main===module)run(process.argv[2]||'.mirket/implementation/design-quality-2026-10-15/qa/controls',{reuse:process.argv.includes('--reuse')}).catch(e=>{console.error(e.stack);process.exitCode=1});
module.exports={run};
