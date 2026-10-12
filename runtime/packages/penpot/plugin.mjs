// This fixed source runs in Penpot's registered, permission-checked plugin sandbox.
// The caller supplies typed data; no caller-controlled JavaScript is evaluated.
async function mirketPenpot(job) {
  penpot.flags.throwValidationErrors = true;
  penpot.flags.naturalChildOrdering = true;
  if (penpot.version !== '2.18.3') throw Error('Expected Penpot 2.18.3; received ' + penpot.version);
  function snapshot() {
    const nodes=[];
    function walk(shape,parent) {
      if(nodes.length>=256)throw Error('Penpot file exceeds the 256-node profile.');
      if(!['board','rectangle','ellipse','path','text'].includes(shape.type))throw Error('This document contains shapes outside the typed local profile.');
      if(shape.type==='text'&&(shape.fontId!=='sourcesanspro'||!['400','600','700'].includes(shape.fontWeight)||shape.fontStyle!=='normal'))throw Error('This profile supports bundled Source Sans Pro normal text at weights400,600,700.');
      if((shape.fills||[]).some(f=>f.fillImage||f.fillColorGradient||!/^#[a-f0-9]{6}$/i.test(f.fillColor||'')))throw Error('Linked, image or unsupported fills are outside the typed local profile.');
      nodes.push({id:shape.id,key:shape.getPluginData('mirket-key'),parent,type:shape.type,name:shape.name,x:shape.x,y:shape.y,width:shape.width,height:shape.height,rotation:shape.rotation,opacity:shape.opacity??1,fills:shape.fills,strokes:shape.strokes,text:shape.type==='text'?shape.characters:undefined,font_size:shape.type==='text'?shape.fontSize:undefined,font_weight:shape.type==='text'?shape.fontWeight:undefined,font_family:shape.type==='text'?shape.fontFamily:undefined,font_id:shape.type==='text'?shape.fontId:undefined,path:shape.type==='path'?shape.d:undefined});
      for(const child of shape.children||[])walk(child,shape.id);
    }
    for(const shape of penpot.currentPage.root.children)walk(shape,null);
    return {version:penpot.version,file_id:penpot.currentFile.id,page_id:penpot.currentPage.id,name:penpot.currentFile.name,revision:penpot.currentFile.revn,nodes};
  }
  if(job.operation==='create') {
    const keys=new Map();
    for(const node of job.document.nodes) {
      let shape;
      if(node.kind==='board')shape=penpot.createBoard();
      else if(node.kind==='rectangle')shape=penpot.createRectangle();
      else if(node.kind==='ellipse')shape=penpot.createEllipse();
      else if(node.kind==='path'){shape=penpot.createPath();shape.d=node.path;if(shape.commands.length<2||shape.width<=0||shape.height<=0)throw Error('SVG path has no usable geometry.');}
      else if(node.kind==='text'){shape=penpot.createText(node.text);if(!shape)throw Error('Text cannot be empty.');shape.fontSize=String(node.font_size||24);shape.fontWeight=String(node.font_weight||400);shape.growType='fixed';}
      else throw Error('Unsupported Penpot shape.');
      shape.name=node.key;shape.setPluginData('mirket-key',node.key);
      shape.resize(node.width,node.height);
      shape.x=node.x;shape.y=node.y;if(node.rotation!==undefined)shape.rotation=node.rotation;
      if(node.opacity!==undefined)shape.opacity=node.opacity;
      shape.fills=node.fills||[{fillColor:'#222222',fillOpacity:1}];
      shape.strokes=node.strokes||[];
      if(node.corner_radius!==undefined) {if(node.kind!=='rectangle')throw Error('Corner radius requires rectangle.');shape.borderRadius=node.corner_radius;}
      if(node.parent){const parent=keys.get(node.parent);if(!parent||parent.type!=='board')throw Error('Parent must be an earlier board.');parent.appendChild(shape);}
      if(node.layout){if(shape.type!=='board')throw Error('Layout requires board.');let layout=shape.addFlexLayout();layout.dir=node.layout.direction;layout.rowGap=node.layout.gap;layout.columnGap=node.layout.gap;layout.topPadding=node.layout.padding;layout.rightPadding=node.layout.padding;layout.bottomPadding=node.layout.padding;layout.leftPadding=node.layout.padding;}
      keys.set(node.key,shape);
    }
    await penpot.waitForLayoutUpdate(10000);
    const s=snapshot();if(s.nodes.length!==job.document.nodes.length)throw Error('Penpot did not retain all requested shapes.');
    for(const node of job.document.nodes){const found=s.nodes.find(s=>s.key===node.key);if(!found||found.name!==node.key)throw Error('Penpot did not apply a requested object identity.');if(!node.parent&&!node.layout&&node.kind!=='path'&&(Math.abs(found.width-node.width)>0.1||Math.abs(found.height-node.height)>0.1))throw Error('Penpot did not apply requested geometry.');}
    return {inspection:s};
  }
  if(job.operation==='update') {
    for(const patch of job.patches) {
      const shape=penpot.currentPage.getShapeById(snapshot().nodes.find(n=>n.key===patch.key)?.id);if(!shape)throw Error('Unknown shape ID: '+patch.key);
      const geometry={x:patch.x??shape.x,y:patch.y??shape.y,width:patch.width??shape.width,height:patch.height??shape.height};
      if(patch.text!==undefined){if(shape.type!=='text')throw Error('Text patch requires text shape.');shape.characters=patch.text;}
      if(patch.path!==undefined){if(shape.type!=='path')throw Error('Path patch requires path shape.');shape.d=patch.path;if(shape.commands.length<2||shape.width<=0||shape.height<=0)throw Error('SVG path has no usable geometry.');}
      if(patch.width!==undefined||patch.height!==undefined||patch.path!==undefined)shape.resize(geometry.width,geometry.height);
      shape.x=geometry.x;shape.y=geometry.y;
      if(patch.fills!==undefined)shape.fills=patch.fills;
      if(patch.rotation!==undefined)shape.rotation=patch.rotation;if(patch.opacity!==undefined)shape.opacity=patch.opacity;
    }
    await penpot.waitForLayoutUpdate(10000);return{inspection:snapshot()};
  }
  if(job.operation==='export') {
    let bytes;
    if(!['png','svg'].includes(job.format))throw Error('Unsupported object export format.');
    {
      const shape=penpot.currentPage.getShapeById(snapshot().nodes.find(n=>n.key===job.object_key)?.id);if(!shape)throw Error('Export requires an inspected object ID.');
      if(shape.width*(job.scale||1)>2048||shape.height*(job.scale||1)>2048)throw Error('Export exceeds 2048 pixels per side.');
      bytes=await shape.export({type:job.format,scale:job.scale||1});
    }
    if(!bytes||!bytes.length||bytes.length>32*1024*1024)throw Error('Penpot returned an empty or oversized export.');
    return{inspection:snapshot(),bytes:Array.from(bytes)};
  }
  return {inspection:snapshot()};
}
