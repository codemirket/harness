import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
const workspace = await fs.mkdtemp(path.join(os.tmpdir(),'mirket-openpencil-'));
process.env.MIRKET_DESIGN_WORKSPACE=workspace;
const {openpencilDocument,openpencilEdit,openpencilInspect,openpencilExport,validateOpenPencilTool,validateOpenPencilBatch,assertOpenPencilSuccess,OPENPENCIL_LIMITS}=await import('../openpencil.mjs');
const {sha256}=await import('../paths.mjs');
test.after(()=>fs.rm(workspace,{recursive:true,force:true}));
const operation=(tool,args)=>({tool,arguments:args});
const rectangle=name=>operation('create_shape',{type:'RECTANGLE',name,x:0,y:0,width:240,height:120});

test('batch validation bounds work, validates nested updates and excludes code/network capabilities',()=>{
  for(const tool of ['eval','stock_photo','insert_icon','save_file','unknown'])
    assert.throws(()=>validateOpenPencilBatch([operation(tool,{})]),/Unsupported/);
  assert.throws(()=>validateOpenPencilTool('create_shape',{type:'INVENTED'},'write'),/Invalid/);
  for(const value of [undefined,null,{},[],Array.from({length:65},()=>rectangle('Too many'))])
    assert.throws(()=>validateOpenPencilBatch(value),/1–64/);
  assert.throws(()=>validateOpenPencilBatch([operation('get_page_tree',{})]),/Unsupported/);
  assert.throws(()=>validateOpenPencilBatch([{...rectangle('Extra'),code:'not permitted'}]),/Invalid/);
  assert.throws(()=>validateOpenPencilBatch([rectangle('x'.repeat(OPENPENCIL_LIMITS.request_bytes))]),/byte budget/);
  const updates=items=>operation('batch_update',{operations:JSON.stringify(items)});
  assert.throws(()=>validateOpenPencilBatch([operation('batch_update',{operations:'not JSON'})]));
  assert.throws(()=>validateOpenPencilBatch([updates([{id:'1:1',props:{unreviewed_property:true}}])]),/Invalid/);
  assert.throws(()=>validateOpenPencilBatch([updates([{id:'1:1',props:{visible:'yes'}}])]),/Invalid/);
  assert.throws(()=>validateOpenPencilBatch([updates(Array.from({length:64},()=>({id:'1:1',props:{name:'Valid'}}))),rectangle('Over budget')]),/total operation budget/);
  assert.equal(validateOpenPencilBatch([updates([{id:'1:1',props:{name:'Valid',visible:true}}])]).length,1);
  const original=[rectangle('Snapshot')];
  const validated=validateOpenPencilBatch(original);
  original[0].arguments.name='Changed';
  assert.equal(validated[0].arguments.name,'Snapshot');
  for(const result of [{error:'failure'},{success:false},{errors:['failure']},{issues:['failure']},{skipped:['failure']}])
    assert.throws(()=>assertOpenPencilSuccess(result),/operation failed/);
});

test('official OpenPencil batches reopen saved IDs, preserve failure atomicity and qualify consumer evidence',async()=>{
  await fs.mkdir(path.join(workspace,'design'));
  await fs.writeFile(path.join(workspace,'design/tokens.json'),JSON.stringify({color:{accent:{$type:'color',$value:'#123456'}}}));
  const created=await openpencilDocument({output:'blank.fig'});
  assert.equal(created.engine,'OpenPencil');
  assert.equal(created.evidence.figma.status,'unverified');
  assert.equal(created.evidence.artifact_sha256,created.artifact.sha256);
  const schema=await openpencilInspect({tool:'create_shape'});
  assert.ok(schema.tool.schema.required.includes('type'));
  const edited=await openpencilEdit({source:'blank.fig',operations:[rectangle('Editable proof'),rectangle('Second layer')],output:'proof.fig'});
  assert.equal(edited.operation_results.length,2);
  const saved=edited.result.children.find(node=>node.name==='Editable proof');
  assert.ok(saved?.id,'return usable IDs from the reopened saved tree');
  assert.equal(edited.result.children.filter(node=>['Editable proof','Second layer'].includes(node.name)).length,2);
  const sourceBefore=await fs.readFile(path.join(workspace,'proof.fig'));
  const hash=sha256(sourceBefore);
  assert.equal(edited.artifact.sha256,hash);
  const failure=[operation('rename_node',{id:saved.id,name:'Would change'}),operation('set_fill',{id:'missing-node',color:'#123456'})];
  await assert.rejects(openpencilEdit({source:'proof.fig',operations:failure,output:'failed.fig'}),/operation failed/);
  await assert.rejects(fs.access(path.join(workspace,'failed.fig')));
  await assert.rejects(openpencilEdit({source:'proof.fig',operations:failure,output:'proof.fig',overwrite:true,expected_sha256:hash}),/operation failed/);
  assert.deepEqual(await fs.readFile(path.join(workspace,'proof.fig')),sourceBefore);
  await fs.writeFile(path.join(workspace,'existing.fig'),sourceBefore);
  await assert.rejects(openpencilEdit({source:'proof.fig',operations:failure,output:'existing.fig',overwrite:true,expected_sha256:hash}),/operation failed/);
  assert.deepEqual(await fs.readFile(path.join(workspace,'existing.fig')),sourceBefore);
  await assert.rejects(openpencilEdit({source:'proof.fig',operations:[operation('batch_update',{operations:JSON.stringify([{id:saved.id,props:{name:'Would change'}},{id:'missing',props:{name:'Fails'}}])})],output:'partial.fig'}),/operation failed/);
  await assert.rejects(fs.access(path.join(workspace,'partial.fig')));
  const filled=await openpencilEdit({source:'proof.fig',operations:[operation('set_fill',{id:saved.id,color:'#123456'})],output:'proof.fig',overwrite:true,expected_sha256:hash});
  const reopened=filled.result.children.find(node=>node.name==='Editable proof');
  assert.ok(reopened?.id);
  const inspectedNode=await openpencilInspect({source:'proof.fig',tool:'get_node',arguments:{id:reopened.id}});
  assert.match(JSON.stringify(inspectedNode.result),/Editable proof/);
  const vars=await openpencilInspect({source:'proof.fig',tool:'list_variables'});
  assert.match(JSON.stringify(vars.result),/accent/);
  const before=await fs.readFile(path.join(workspace,'proof.fig'));
  await fs.rename(path.join(workspace,'design/tokens.json'),path.join(workspace,'design/saved-tokens.json'));
  let png;
  try {
    png=await openpencilExport({source:'proof.fig',node:reopened.id,output:'proof.png',consumer:'figma'});
    assert.equal(png.tokens,undefined,'export must not mutate/reimport tokens');
  } finally {await fs.rename(path.join(workspace,'design/saved-tokens.json'),path.join(workspace,'design/tokens.json'));}
  assert.deepEqual(png.dimensions,{width:480,height:240});
  assert.equal(png.source_sha256,sha256(before));
  assert.equal(png.evidence.local.status,'passed');
  assert.equal(png.evidence.figma.status,'unverified');
  assert.equal(png.qa.status,'unverified');
  assert.equal(png.evidence.artifact_sha256,png.artifact.sha256);
  const {default:sharp}=await import('sharp');
  const pixel=await sharp(path.join(workspace,'proof.png')).extract({left:240,top:120,width:1,height:1}).removeAlpha().raw().toBuffer();
  assert.deepEqual([...pixel],[0x12,0x34,0x56]);
  if(process.env.MIRKET_ISOLATED_TEST==='1')console.log('OPENPENCIL_EVIDENCE '+JSON.stringify({png:(await fs.readFile(path.join(workspace,'proof.png'))).toString('base64'),fig:before.toString('base64'),receipt:png}));
  const svg=await openpencilExport({source:'proof.fig',output:'proof.svg',format:'svg'});
  assert.equal(svg.evidence.figma.status,'unverified');
  assert.equal(svg.evidence.native_media.vector,'SVG');
  assert.match(await fs.readFile(path.join(workspace,'proof.svg'),'utf8'),/<svg/);
  assert.deepEqual(await fs.readFile(path.join(workspace,'proof.fig')),before);
  await assert.rejects(openpencilExport({source:'proof.fig',output:'proof.pdf',format:'pdf'}),/format and extension/);
  await assert.rejects(fs.access(path.join(workspace,'proof.pdf')));
  await assert.rejects(openpencilDocument({source:'proof.fig',output:'proof.fig',overwrite:true,expected_sha256:sha256(before)}),/preserve the original/);
  const imported=await openpencilDocument({source:'proof.fig',output:'imported.fig'});
  assert.equal(imported.source_sha256,sha256(before));
  assert.deepEqual(await fs.readFile(path.join(workspace,'proof.fig')),before);
  const rename={source:'proof.fig',operations:[operation('rename_node',{id:reopened.id,name:'Revised'})],output:'proof.fig',overwrite:true};
  await assert.rejects(openpencilEdit({...rename,expected_sha256:'0'.repeat(64)}),/revision mismatch/);
  await openpencilEdit({...rename,expected_sha256:sha256(before)});
  assert.match(JSON.stringify((await openpencilInspect({source:'proof.fig'})).result),/Revised/);
});

test('invalid input, corrupt documents and symlinks never publish user files',async()=>{
  await assert.rejects(openpencilEdit({source:'blank.fig',tool:'create_shape',arguments:{},output:'legacy.fig'}),/Use operations/);
  await assert.rejects(openpencilInspect({source:'../outside.fig'}),/traversal/);
  const outside=path.join(workspace,'outside.fig');
  await fs.writeFile(outside,'not a document');
  await fs.symlink(outside,path.join(workspace,'link.fig'));
  await assert.rejects(openpencilInspect({source:'link.fig'}),/symlink/);
  await assert.rejects(openpencilDocument({output:'blank.fig'}),/Output exists/);
  await fs.writeFile(path.join(workspace,'corrupt.fig'),'bad');
  await assert.rejects(openpencilEdit({source:'corrupt.fig',operations:[rectangle('Never saved')],output:'never.fig'}));
  await assert.rejects(fs.access(path.join(workspace,'never.fig')));
  const leftovers=(await fs.readdir(path.join(workspace,'.mirket-design'))).filter(n=>n.startsWith('openpencil-')||n.startsWith('child-'));
  assert.deepEqual(leftovers,[]);
});

test('reviewed schemas match the pinned upstream definitions and omit external/code capabilities',async()=>{
  const {ALL_TOOLS,toolChangesDocument}=await import('@open-pencil/core/tools');
  const {toJsonSchema}=await import('@valibot/to-json-schema');
  const manifest=JSON.parse(await fs.readFile(new URL('../openpencil-tools.json',import.meta.url)));
  for(const tool of manifest.tools){
    const upstream=ALL_TOOLS.find(def=>def.name===tool.name);
    assert.ok(upstream,tool.name);
    assert.deepEqual(tool.schema,toJsonSchema(upstream.input,{errorMode:'ignore'}),tool.name);
    assert.equal(tool.effect,toolChangesDocument(upstream)?'write':'read');
    assert.ok(upstream.capabilities.every(cap=>['document:read','document:write'].includes(cap)));
  }
  assert.equal(manifest.batch.max_total_operations,OPENPENCIL_LIMITS.operations);
  assert.equal(manifest.batch.max_request_bytes,OPENPENCIL_LIMITS.request_bytes);
});

test('actual MCP discovery and calls expose the clean batch contract',async()=>{
  const {Client}=await import('@modelcontextprotocol/sdk/client/index.js');
  const {StdioClientTransport}=await import('@modelcontextprotocol/sdk/client/stdio.js');
  const client=new Client({name:'openpencil-integration-test',version:'1'});
  const transport=new StdioClientTransport({command:process.execPath,args:[new URL('../server.mjs',import.meta.url).pathname],env:{PATH:process.env.PATH,HOME:workspace,MIRKET_DESIGN_WORKSPACE:workspace,MIRKET_DESIGN_GROUPS:'core'},stderr:'pipe'});
  try{
    await client.connect(transport);
    const listed=await client.listTools();
    const editSchema=listed.tools.find(t=>t.name==='openpencil_edit').inputSchema;
    assert.ok(editSchema.required.includes('operations'),'server owner must publish the clean batch schema');
    assert.equal(editSchema.properties.operations.maxItems,63,'one operation is reserved for canonical tokens');
    assert.equal(editSchema.properties.tool,undefined);
    assert.deepEqual(listed.tools.find(t=>t.name==='openpencil_export').inputSchema.properties.consumer.enum,['local','figma']);
    assert.equal(listed.tools.filter(t=>t.name.startsWith('openpencil_')).length,4);
    assert.ok(!listed.tools.some(t=>['execute_code','export_shape','import_image','penpot_api_info'].includes(t.name)));
    const call=await client.callTool({name:'openpencil_document',arguments:{output:'mcp.fig'}});
    assert.notEqual(call.isError,true,JSON.stringify(call));
    assert.equal(call.structuredContent.runtime.workspace,workspace);
    assert.ok(call.structuredContent.runtime.files['openpencil-tools.json']);
    const edited=await client.callTool({name:'openpencil_edit',arguments:{source:'mcp.fig',operations:[rectangle('MCP batch')],output:'mcp-edited.fig'}});
    assert.notEqual(edited.isError,true,JSON.stringify(edited));
    assert.match(JSON.stringify(edited.structuredContent.result),/MCP batch/);
    const invalid=await client.callTool({name:'openpencil_edit',arguments:{source:'mcp.fig',operations:[operation('eval',{code:'throw Error("should not execute")'})],output:'eval.fig'}});
    assert.equal(invalid.isError,true);
    await assert.rejects(fs.access(path.join(workspace,'eval.fig')));
  }finally{await client.close();}
});

test('schema discovery writes nothing and remains available while the render slot is held',async()=>{
  const {Client}=await import('@modelcontextprotocol/sdk/client/index.js');
  const {StdioClientTransport}=await import('@modelcontextprotocol/sdk/client/stdio.js');
  const {withRenderSlot}=await import('../paths.mjs');
  const schemaRoot=await fs.mkdtemp(path.join(os.tmpdir(),'mirket-schema-'));
  async function connect(root){
    const client=new Client({name:'schema-inspection',version:'1'});
    await client.connect(new StdioClientTransport({command:process.execPath,args:[new URL('../server.mjs',import.meta.url).pathname],env:{PATH:process.env.PATH,HOME:root,MIRKET_DESIGN_WORKSPACE:root,MIRKET_DESIGN_GROUPS:'core'},stderr:'pipe'}));
    return client;
  }
  let client=await connect(schemaRoot);
  try{
    const result=await client.callTool({name:'openpencil_inspect',arguments:{tool:'create_shape'}});
    assert.notEqual(result.isError,true);
    assert.equal(result.structuredContent.tool.effect,'write');
    assert.deepEqual(await fs.readdir(schemaRoot),[]);
  }finally{await client.close();await fs.rm(schemaRoot,{recursive:true,force:true});}
  client=await connect(workspace);
  try{
    await withRenderSlot(async()=>{
      const result=await client.callTool({name:'openpencil_inspect',arguments:{}},undefined,{timeout:3000});
      assert.notEqual(result.isError,true);
      assert.equal(result.structuredContent.tools.length,87);
      assert.equal(result.structuredContent.batch.max_total_operations,64);
      const schema=await client.callTool({name:'openpencil_inspect',arguments:{tool:'get_node'}},undefined,{timeout:3000});
      assert.equal(schema.structuredContent.tool.effect,'read');
    });
  }finally{await client.close();}
});
