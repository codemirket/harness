#!/usr/bin/env node
/* MIT. Blinded render packets; source identity key stays outside judge packets. */
'use strict';
const fs=require('node:fs/promises'),path=require('node:path'),crypto=require('node:crypto'),assert=require('node:assert/strict');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const definitions=[
  {id:'P01',kind:'composition',family:'workshop-register',controls:['C101','C102']},
  {id:'P02',kind:'composition',family:'service-desk',controls:['C106','C107']},
  {id:'P03',kind:'identical',family:'workshop-register',controls:['C101','C104']},
  {id:'P04',kind:'content-order',family:'workshop-register',controls:['C101','C105']},
  {id:'P05',kind:'polished-false-state',family:'workshop-register',controls:['C101','C103']}
];
const layouts={
  'judge-a':[{pair:'P02',order:[1,0],labels:['L17','L63']},{pair:'P04',order:[1,0],labels:['L42','L08']},{pair:'P01',order:[0,1],labels:['L91','L24']},{pair:'P05',order:[1,0],labels:['L56','L33']},{pair:'P03',order:[0,1],labels:['L72','L19']}],
  'judge-b':[{pair:'P03',order:[1,0],labels:['L38','L82']},{pair:'P01',order:[1,0],labels:['L14','L67']},{pair:'P05',order:[0,1],labels:['L95','L21']},{pair:'P02',order:[0,1],labels:['L49','L76']},{pair:'P04',order:[0,1],labels:['L53','L06']}]
};
async function pack(observationDir,outputDir){
  const observations=path.resolve(observationDir),out=path.resolve(outputDir);await fs.mkdir(out,{recursive:true});
  const spec=JSON.parse(await fs.readFile(path.join(__dirname,'control-spec.json'),'utf8'));
  const rubric=JSON.parse(await fs.readFile(path.join(__dirname,'..','craft-rubric.json'),'utf8'));
  const key={schema_version:1,synthetic:true,model_executions:0,created_at:new Date().toISOString(),thresholds:spec.thresholds,dimensions:rubric.dimensions.map(d=>({id:d.id,weight:d.weight})),packets:{}};
  for(const[packetId,layout]of Object.entries(layouts)){
    const packetDir=path.join(out,packetId);await fs.mkdir(packetDir,{recursive:true});
    const packet={schema_version:1,kind:'render_only_synthetic_calibration_packet',id:packetId,synthetic_controls:true,model_executions:0,human_acceptance:'not_collected',rubric,pairs:[],instructions:['Inspect only this assigned packet before locking every rating. Do not read the mapping key, control source, specification relations, observer results or peer ratings.','Use the raw brief and located rendered evidence for each score; two located observations per dimension.','Labels and presentation positions are arbitrary. Similar artifacts may be identical. Do not infer quality from order.','Initial and post-action captures show visible states. A displayed saved claim alone does not establish persistence. Keep functional conclusions for the later stage.','Record uncertainties and actual blinding exceptions. Do not revise locked render-only scores after inspecting functional results.']};
    key.packets[packetId]={pairs:[],images:{}};
    for(const presentation of layout){
      const definition=definitions.find(d=>d.id===presentation.pair),family=spec.families.find(f=>f.id===definition.family);
      const pair={id:definition.id,presentation_order:packet.pairs.length+1,brief:family.brief,artifacts:[]};
      const keyPair={id:definition.id,kind:definition.kind,family:definition.family,artifacts:[]};
      for(let pos=0;pos<2;pos++){
        const control=definition.controls[presentation.order[pos]],label=presentation.labels[pos];
        const record=JSON.parse(await fs.readFile(path.join(observations,control,'observations.json'),'utf8'));
        assert.equal(record.collection_completed,true,`${control} collection required`);assert.equal(record.artifact_unchanged,true,`${control} source must be unchanged`);
        const art={label,presentation_position:pos+1,images:[]};const dir=path.join(packetDir,label);await fs.mkdir(dir,{recursive:true});
        const names=['default-1280','default-390','default-320'];
        const actions=definition.family==='service-desk'?'scenario-filter-open-and-owned-review-final':'scenario-separate-storage-read-and-reload-post-save-claim';names.push(actions);
        for(const name of names)for(const view of ['viewport','fullpage']){
          const filename=`${name}.${view}.png`,bytes=await fs.readFile(path.join(observations,control,filename)),relative=`${label}/${filename}`;
          await fs.writeFile(path.join(dir,filename),bytes);const bound={path:relative,sha256:sha(bytes),bytes:bytes.length};art.images.push(bound);key.packets[packetId].images[relative]=bound;
        }
        pair.artifacts.push(art);keyPair.artifacts.push({label,presentation_position:pos+1,control,source_tree_sha256:record.source_bindings.artifact.tree_sha256});
      }
      packet.pairs.push(pair);key.packets[packetId].pairs.push(keyPair);
    }
    await fs.writeFile(path.join(packetDir,'packet.json'),JSON.stringify(packet,null,2)+'\n');
    await fs.writeFile(path.join(packetDir,'README.md'),`# Render-only calibration packet ${packetId}\n\nRead packet.json and only the images listed there. Keep every score on the 0–100 continuous craft rubric and provide two located visual observations per dimension. Lock all render-only scores before source, observer outcomes, mapping key or peer ratings. This packet contains original synthetic controls, zero model trial executions, and no human acceptance.\n`);
    key.packets[packetId].packet_sha256=sha(await fs.readFile(path.join(packetDir,'packet.json')));
  }
  await fs.writeFile(path.join(out,'key.json'),JSON.stringify(key,null,2)+'\n');
  return key;
}
if(require.main===module){const[a,b]=process.argv.slice(2);if(!a||!b){console.error('Usage: node pack.cjs OBSERVATIONS_DIR OUTPUT_DIR');process.exitCode=1}else pack(a,b).then(k=>console.log(JSON.stringify({packets:Object.keys(k.packets),synthetic:true,model_executions:0,key:path.resolve(b,'key.json')}))).catch(e=>{console.error(e.stack);process.exitCode=1})}
module.exports={pack};
