// Deterministic tool-call fixture; makes no model or network request.
export function makeFixtureProvider(ai) {
 return function fixtureProvider(pi){pi.registerProvider('offline-fixture',{api:'offline-fixture',baseUrl:'http://invalid.local',apiKey:'fixture',
 models:[{id:'deterministic',name:'Offline fixture',reasoning:false,input:['text'],cost:{input:0,output:0,cacheRead:0,cacheWrite:0},contextWindow:200000,maxTokens:20000}],
 streamSimple(model,context){const stream=ai.createAssistantMessageEventStream();const last=context.messages.findLastIndex(m=>m.role==='user');
 const user=context.messages[last];const text=typeof user.content==='string'?user.content:user.content.filter(x=>x.type==='text').map(x=>x.text).join('');
 const request=JSON.parse(typeof user.content==='string'?user.content:text);const issued=context.messages.slice(last+1).some(m=>m.role==='assistant');
 const output={role:'assistant',api:model.api,provider:model.provider,model:model.id,content:!issued?[{type:'toolCall',id:'fixture-'+Date.now(),name:request.tool,arguments:request.args??{}}]:[{type:'text',text:'Fixture complete'}],usage:{input:0,output:0,cacheRead:0,cacheWrite:0,totalTokens:0,cost:{input:0,output:0,cacheRead:0,cacheWrite:0,total:0}},stopReason:!issued?'toolUse':'stop',timestamp:Date.now()};
 queueMicrotask(()=>{stream.push({type:'start',partial:output});stream.push({type:'done',reason:output.stopReason,message:output});stream.end(output);});return stream;
 }});}
}
