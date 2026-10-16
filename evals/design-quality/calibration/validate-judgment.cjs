#!/usr/bin/env node
/* MIT. Structural/literal validator; it cannot establish perceptual correctness. */
'use strict';
const fs=require('node:fs/promises'),path=require('node:path'),crypto=require('node:crypto'),assert=require('node:assert/strict');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
async function validate(judgmentFile,keyFile){
  const judgment=JSON.parse(await fs.readFile(judgmentFile,'utf8')),key=JSON.parse(await fs.readFile(keyFile,'utf8'));
  assert.equal(judgment.schema_version,1);assert.equal(judgment.kind,'synthetic_craft_calibration_judgment');assert.equal(judgment.synthetic_controls,true);assert.equal(judgment.human_acceptance,'not_collected');
  assert.ok(typeof judgment.judge_id==='string'&&judgment.judge_id.trim(),'actual judge identity required');assert.ok(Number.isFinite(Date.parse(judgment.locked_at)),'render-only lock timestamp required');
  const packet=key.packets[judgment.packet_id];assert.ok(packet,'known packet required');
  const packetDir=path.join(path.dirname(path.resolve(keyFile)),judgment.packet_id);
  assert.equal(sha(await fs.readFile(path.join(packetDir,'packet.json'))),packet.packet_sha256,'packet changed');
  const expected=packet.pairs.flatMap(p=>p.artifacts.map(a=>({pair_id:p.id,...a}))),ratings=new Map();
  assert.ok(Array.isArray(judgment.ratings));assert.equal(judgment.ratings.length,expected.length,'all displayed artifacts must be rated');
  for(const rating of judgment.ratings){
    const entry=expected.find(e=>e.pair_id===rating.pair_id&&e.label===rating.artifact_label);assert.ok(entry,'rating must identify a displayed pair/label');
    const id=`${rating.pair_id}/${rating.artifact_label}`;assert.ok(!ratings.has(id),'duplicate rating');assert.equal(rating.presentation_position,entry.presentation_position,'presentation position must be preserved');
    assert.ok(Array.isArray(rating.dimensions));assert.equal(rating.dimensions.length,key.dimensions.length,'every dimension required');
    const dimIds=new Set();let composite=0;
    for(const dimension of rating.dimensions){
      const configured=key.dimensions.find(d=>d.id===dimension.id);assert.ok(configured&&!dimIds.has(dimension.id),'unique known dimension');dimIds.add(dimension.id);
      assert.ok(typeof dimension.score==='number'&&Number.isFinite(dimension.score)&&dimension.score>=0&&dimension.score<=100,'continuous score must be between 0 and 100');
      assert.ok(Array.isArray(dimension.observations)&&dimension.observations.length>=2,'two located observations per dimension');
      for(const observation of dimension.observations){
        assert.ok(typeof observation.image==='string'&&observation.image.startsWith(entry.label+'/'),'observation must cite this artifact image');
        const bound=packet.images[observation.image];assert.ok(bound,'image must exist in frozen packet');assert.equal(sha(await fs.readFile(path.join(packetDir,observation.image))),bound.sha256,'image changed');
        assert.ok(typeof observation.region==='string'&&observation.region.trim().length>=6,'specific located region required');assert.ok(typeof observation.claim==='string'&&observation.claim.trim().length>=15,'located visual fact/consequence required');
      }
      composite+=configured.weight*dimension.score;
    }
    ratings.set(id,{...entry,composite,dimensions:rating.dimensions.map(d=>({id:d.id,score:d.score}))});
  }
  const pairs=packet.pairs.map(pair=>{
    const scored=pair.artifacts.map(a=>ratings.get(`${pair.id}/${a.label}`));
    const result={id:pair.id,kind:pair.kind,family:pair.family,ratings:scored,difference_first_minus_second:scored[0].composite-scored[1].composite};
    if(pair.kind==='composition'){
      const positive=scored.find(s=>['C101','C106'].includes(s.control)),negative=scored.find(s=>['C102','C107'].includes(s.control));result.positive_minus_negative=positive.composite-negative.composite;result.calibration_check_passed=result.positive_minus_negative>=key.thresholds.positive_minus_negative_minimum;
    }else if(pair.kind==='identical'){result.absolute_difference=Math.abs(result.difference_first_minus_second);result.calibration_check_passed=result.absolute_difference<=key.thresholds.identical_pair_maximum_absolute_difference;}
    return result;
  });
  const consistency=[];
  for(const control of new Set([...ratings.values()].map(r=>r.control))){const same=[...ratings.values()].filter(r=>r.control===control);if(same.length>1)consistency.push({control,ratings:same.map(r=>({label:r.label,pair_id:r.pair_id,presentation_position:r.presentation_position,composite:r.composite})),range:Math.max(...same.map(r=>r.composite))-Math.min(...same.map(r=>r.composite)),interpretation:'Descriptive within-judge label/placement repeat; do not replace initial scores.'});}
  return{schema_version:1,kind:'synthetic_calibration_validation',judge_id:judgment.judge_id,packet_id:judgment.packet_id,synthetic_controls:true,control_model_executions:0,judgment_sha256:sha(await fs.readFile(judgmentFile)),key_sha256:sha(await fs.readFile(keyFile)),calibrated:pairs.filter(p=>p.calibration_check_passed!==undefined).every(p=>p.calibration_check_passed),pairs,label_order_consistency:consistency,limits:['Structural checks and expected contrasts do not establish that located observations are perceptually correct.','No human acceptance or model quality estimate is produced.','Same-control score variation is descriptive; cross-judge counterbalancing does not identify an individual causal order effect.']};
}
if(require.main===module){const[a,b]=process.argv.slice(2);if(!a||!b){console.error('Usage: node validate-judgment.cjs JUDGMENT.json KEY.json');process.exitCode=1}else validate(a,b).then(r=>console.log(JSON.stringify(r,null,2))).catch(e=>{console.error(e.stack);process.exitCode=1})}
module.exports={validate};
