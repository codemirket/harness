import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
const workspace = await fs.mkdtemp(path.join(os.tmpdir(),'mirket-openpencil-'));
process.env.MIRKET_DESIGN_WORKSPACE=workspace;
const {openpencilDocument,openpencilEdit,openpencilInspect,openpencilExport,validateOpenPencilTool}=await import('../openpencil.mjs');
const {sha256}=await import('../paths.mjs');
test.after(()=>fs.rm(workspace,{recursive:true,force:true}));

test('official OpenPencil tools create, edit, reopen and export a layered document with tokens',async()=>{
  await fs.mkdir(path.join(workspace,'design'));
  await fs.writeFile(path.join(workspace,'design/tokens.json'),JSON.stringify({color:{accent:{$type:'color',$value:'#123456'}}}));
  const created=await openpencilDocument({output:'blank.fig'});
  assert.equal(created.engine,'OpenPencil');
  const schema=await openpencilInspect({tool:'create_shape'});
  assert.ok(schema.tool.schema.required.includes('type'));
  const edited=await openpencilEdit({source:'blank.fig',tool:'create_shape',arguments:{type:'RECTANGLE',name:'Editable proof',x:0,y:0,width:240,height:120},output:'proof.fig'});
  assert.equal(edited.result.children[0].name,'Editable proof');
  const savedId=edited.result.children[0].id;
  await assert.rejects(openpencilEdit({source:'proof.fig',tool:'set_fill',arguments:{id:'missing-node',color:'#123456'},output:'missing-node.fig'}),/operation failed/);
  await assert.rejects(fs.access(path.join(workspace,'missing-node.fig')));
  await assert.rejects(openpencilEdit({source:'proof.fig',tool:'batch_update',arguments:{operations:JSON.stringify([{id:savedId,props:{name:'Would change'}},{id:'missing',props:{name:'Fails'}}])},output:'partial.fig'}),/operation failed/);
  await assert.rejects(fs.access(path.join(workspace,'partial.fig')));
  await openpencilEdit({source:'proof.fig',tool:'set_fill',arguments:{id:savedId,color:'#123456'},output:'proof.fig',overwrite:true,expected_sha256:sha256(await fs.readFile(path.join(workspace,'proof.fig')))});
  const tree=await openpencilInspect({source:'proof.fig'});
  assert.match(JSON.stringify(tree.result),/Editable proof/);
  const vars=await openpencilInspect({source:'proof.fig',tool:'list_variables'});
  assert.match(JSON.stringify(vars.result),/accent/);
  const before=await fs.readFile(path.join(workspace,'proof.fig'));
  const png=await openpencilExport({source:'proof.fig',output:'proof.png'});
  assert.deepEqual(png.dimensions,{width:480,height:240});
  assert.equal(png.source_sha256,sha256(before));
  const {default:sharp}=await import('sharp');
  const pixel=await sharp(path.join(workspace,'proof.png')).extract({left:240,top:120,width:1,height:1}).removeAlpha().raw().toBuffer();
  assert.deepEqual([...pixel],[0x12,0x34,0x56]);
  if(process.env.MIRKET_ISOLATED_TEST==='1') console.log('OPENPENCIL_EVIDENCE '+JSON.stringify({png:(await fs.readFile(path.join(workspace,'proof.png'))).toString('base64'),fig:before.toString('base64'),receipt:png}));
  await openpencilExport({source:'proof.fig',output:'proof.svg',format:'svg'});
  await assert.rejects(openpencilExport({source:'proof.fig',output:'proof.pdf',format:'pdf'}),/format and extension/);
  await assert.rejects(fs.access(path.join(workspace,'proof.pdf')));
  assert.match(await fs.readFile(path.join(workspace,'proof.svg'),'utf8'),/<svg/);
  assert.deepEqual(await fs.readFile(path.join(workspace,'proof.fig')),before);
  // Refuse stale concurrent edits, rather than silently replacing saved work.
  const rename={source:'proof.fig',tool:'rename_node',arguments:{id:savedId,name:'Revised'},output:'proof.fig',overwrite:true};
  await assert.rejects(openpencilEdit({...rename,expected_sha256:'0'.repeat(64)}),/revision mismatch/);
  await openpencilEdit({...rename,expected_sha256:sha256(before)});
  assert.match(JSON.stringify((await openpencilInspect({source:'proof.fig'})).result),/Revised/);
});

test('rejected OpenPencil inputs cannot dispatch code or overwrite user files',async()=>{
  for(const tool of ['eval','stock_photo','insert_icon','save_file','unknown'])
    assert.throws(()=>validateOpenPencilTool(tool,{},'write'),/Unsupported/);
  assert.throws(()=>validateOpenPencilTool('create_shape',{type:'INVENTED'},'write'),/Invalid/);
  await assert.rejects(openpencilInspect({source:'../outside.fig'}),/traversal/);
  const outside=path.join(workspace,'outside.fig');
  await fs.writeFile(outside,'not a document');
  await fs.symlink(outside,path.join(workspace,'link.fig'));
  await assert.rejects(openpencilInspect({source:'link.fig'}),/symlink/);
  await assert.rejects(openpencilDocument({output:'blank.fig'}),/Output exists/);
  await fs.writeFile(path.join(workspace,'corrupt.fig'),'bad');
  await assert.rejects(openpencilEdit({source:'corrupt.fig',tool:'create_shape',arguments:{type:'RECTANGLE',x:0,y:0,width:10,height:10},output:'never.fig'}));
  await assert.rejects(fs.access(path.join(workspace,'never.fig')));
  const leftovers=(await fs.readdir(path.join(workspace,'.mirket-design'))).filter(n=>n.startsWith('openpencil-')||n.startsWith('child-'));
  assert.deepEqual(leftovers,[]);
});

test('reviewed tool schemas match the pinned upstream definitions and omit external/code capabilities',async()=>{
  const {ALL_TOOLS,toolChangesDocument}=await import('@open-pencil/core/tools');
  const {toJsonSchema}=await import('@valibot/to-json-schema');
  const manifest=JSON.parse(await fs.readFile(new URL('../openpencil-tools.json',import.meta.url)));
  for(const tool of manifest.tools) {
    const upstream=ALL_TOOLS.find(def=>def.name===tool.name);
    assert.ok(upstream,tool.name);
    assert.deepEqual(tool.schema,toJsonSchema(upstream.input,{errorMode:'ignore'}),tool.name);
    assert.equal(tool.effect,toolChangesDocument(upstream)?'write':'read');
    assert.ok(upstream.capabilities.every(cap=>['document:read','document:write'].includes(cap)));
  }
});

test('actual design MCP discovery and calls expose OpenPencil with the scoped runtime identity',async()=>{
  const {Client}=await import('@modelcontextprotocol/sdk/client/index.js');
  const {StdioClientTransport}=await import('@modelcontextprotocol/sdk/client/stdio.js');
  const client=new Client({name:'openpencil-integration-test',version:'1'});
  const transport=new StdioClientTransport({command:process.execPath,args:[new URL('../server.mjs',import.meta.url).pathname],env:{PATH:process.env.PATH,HOME:workspace,MIRKET_DESIGN_WORKSPACE:workspace,MIRKET_DESIGN_GROUPS:'core'},stderr:'pipe'});
  try {
    await client.connect(transport);
    const listed=await client.listTools();
    assert.deepEqual(listed.tools.find(t=>t.name==='openpencil_export').inputSchema.properties.format.enum,['png','svg']);
    assert.equal(listed.tools.filter(t=>t.name.startsWith('openpencil_')).length,4);
    assert.ok(!listed.tools.some(t=>['execute_code','export_shape','import_image','penpot_api_info'].includes(t.name)));
    const call=await client.callTool({name:'openpencil_document',arguments:{output:'mcp.fig'}});
    assert.notEqual(call.isError,true,JSON.stringify(call));
    assert.equal(call.structuredContent.runtime.workspace,workspace);
    assert.ok(call.structuredContent.runtime.files['openpencil-tools.json']);
    const invalid=await client.callTool({name:'openpencil_edit',arguments:{source:'mcp.fig',tool:'eval',arguments:{code:'throw Error("should not execute")'},output:'eval.fig'}});
    assert.equal(invalid.isError,true);
    await assert.rejects(fs.access(path.join(workspace,'eval.fig')));
  } finally { await client.close(); }
});
