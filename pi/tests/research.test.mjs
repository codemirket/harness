import test from 'node:test';
import assert from 'node:assert/strict';
import {researchPayload,observeResearch,researchLinks} from '../dist/extension/research.js';
const evidence=()=>({schema:1,id:'fixture',project:'/fixture',release:'fixture',provider:'openai',model:'gpt-6-astra',query:'question',startedAt:'2026-10-29',status:'running',actions:[],answers:[]});
test('native research preserves existing tools and source includes without mutating the provider request',()=>{
 const before={tools:[{type:'function',name:'read'},{type:'web_search_preview'}],include:['reasoning.encrypted_content']};
 const after=researchPayload(before);assert.deepEqual(after.tools,[{type:'function',name:'read'},{type:'web_search'}]);assert.deepEqual(after.include,['reasoning.encrypted_content','web_search_call.action.sources']);assert.equal(before.tools[1].type,'web_search_preview');assert.throws(()=>researchPayload(null));
});
test('source evidence uses provider annotations, deduplicates events and refuses unsafe clickable URLs',()=>{
 const record=evidence();const item={id:'one',type:'message',content:[{type:'output_text',text:'Invented https://unobserved.example is not a citation.',annotations:[{type:'url_citation',url:'https://example.org/source'},{type:'url_citation',url:'javascript:alert(1)'},{type:'url_citation',url:'https://secret@example.org/source'}]}]};
 observeResearch(record,{type:'response.output_item.done',item});observeResearch(record,{type:'response.output_item.done',item});assert.equal(record.answers.length,1);assert.deepEqual(researchLinks(record),['https://example.org/source']);
});
test('research evidence has finite action and byte budgets',()=>{
 const record=evidence();for(let n=0;n<8;n++)observeResearch(record,{type:'response.output_item.done',item:{type:'web_search_call',id:String(n)}});
 assert.throws(()=>observeResearch(record,{type:'response.output_item.done',item:{type:'web_search_call',id:'overflow'}}),/budget/);assert.equal(record.actions.length,8);
 assert.throws(()=>observeResearch(record,{type:'response.output_item.done',item:{type:'message',id:'large',content:'x'.repeat(256*1024)}}),/byte budget/);assert.equal(record.answers.length,0);
});
