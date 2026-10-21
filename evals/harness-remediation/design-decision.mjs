// Bounded, offline encoded-video opportunity; uses the actual owned adapter.
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
process.env.MIRKET_DESIGN_WORKSPACE='/workspace';
const {hyperframesRender}=await import('../operations.mjs');
await fs.mkdir('/workspace/design');
await fs.writeFile('/workspace/design/tokens.json',JSON.stringify({color:{accent:{$type:'color',$value:'#123456'}}}));
await fs.mkdir('/workspace/video');
await fs.writeFile('/workspace/video/index.html',`<!doctype html><html><head><meta charset="utf-8"><style>
*{box-sizing:border-box}body{margin:0;background:#f4f1ea;font-family:sans-serif}
#scene{width:320px;height:180px;position:relative;overflow:hidden;background:#f4f1ea}
h1{position:absolute;left:20px;top:12px;font-size:22px;color:#123456}
#box{position:absolute;left:20px;top:100px;width:40px;height:40px;background:#123456;animation:travel 1s linear both}
@keyframes travel{from{transform:translateX(0)}to{transform:translateX(220px)}}
</style></head><body><div id="scene" data-composition-id="trial" data-no-timeline data-width="320" data-height="180" data-duration="1" data-fps="24"><h1>Queue accepted</h1><div id="box" class="clip" data-start="0" data-duration="1"></div></div><script>window.__timelines={};</script></body></html>`);
const start=Date.now();
try {
 const result=await hyperframesRender({project:'video',output:'trial.mp4',snapshot_times:[0,0.5,0.9]});
 assert.deepEqual(result.phases,['lint','snapshot','render']);
 const bytes=await fs.readFile('/workspace/trial.mp4');
 assert.ok(bytes.length>1000);
 const frames=[];
 for(const item of result.snapshots) frames.push({path:item.artifact.path,png:(await fs.readFile('/workspace/'+item.artifact.path)).toString('base64')});
 console.log(JSON.stringify({ok:true,elapsed_ms:Date.now()-start,media:result.media,qa:result.qa,frames,video:bytes.toString('base64'),limits:'One synthetic encoded-video opportunity; named adapter invocation, no natural adoption or general quality claim.'}));
} catch(error) {
 console.log(JSON.stringify({ok:false,elapsed_ms:Date.now()-start,error:String(error),limits:'Retain failed invocation; no replacement or quality conclusion.'}));
}
