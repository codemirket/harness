#!/usr/bin/env node
/** Render an editable SVG at multiple sizes/backgrounds without changing its source. */
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { createRequire } from 'node:module';
import { spawnSync } from 'node:child_process';

const args = process.argv.slice(2);
if (args.includes('--help')) {
  console.log('vector INPUT.svg --output NEW_DIR\nRequires installed sharp (HARNESS_NODE_MODULES). Produces editable source copy, 24/64/256/1024px PNGs, light/dark contact sheet, report.json. Static rendering only; no aesthetic acceptance.');
  process.exit(0);
}
const hash = b => crypto.createHash('sha256').update(b).digest('hex');
try {
  if (args.length !== 3 || args[1] !== '--output') throw new Error('Use vector INPUT.svg --output NEW_DIR');
  const input = path.resolve(args[0]), output = path.resolve(args[2]);
  if (fs.existsSync(output)) throw new Error('Output already exists');
  for (let p = output; ; p = path.dirname(p)) {
    if (fs.existsSync(p) && fs.lstatSync(p).isSymbolicLink()) throw new Error('Linked output path');
    if (p === path.dirname(p)) break;
  }
  if (!fs.statSync(path.dirname(output)).isDirectory()) throw new Error('Output parent must exist');
  const raw = fs.readFileSync(input);
  if (raw.length > 4 * 1024 * 1024) throw new Error('SVG exceeds 4 MiB limit');
  const source = raw.toString('utf8');
  // Reject active/external content before passing bytes to a rasterizer. This is
  // a local review tool for trusted authored assets, not an upload sanitizer.
  const py = process.env.HARNESS_PYTHON || 'python3';
  const validate = spawnSync(py, ['-c', String.raw`
import json,sys,xml.etree.ElementTree as E
s=sys.stdin.read()
if '<!DOCTYPE' in s.upper() or '<!ENTITY' in s.upper(): raise ValueError('DTD/entity unsupported')
r=E.fromstring(s)
if r.tag.split('}')[-1]!='svg': raise ValueError('Expected SVG root')
v=r.get('viewBox','').replace(',',' ').split()
if len(v)!=4 or any(not __import__('math').isfinite(float(x)) for x in v) or float(v[2])<=0 or float(v[3])<=0: raise ValueError('Positive finite viewBox required')
ids=[]
for e in r.iter():
 tag=e.tag.split('}')[-1]
 if tag in ('script','foreignObject','image','feImage','style'): raise ValueError('Active/raster/embedded style unsupported: '+tag)
 for k,x in e.attrib.items():
  key=k.split('}')[-1]
  if key.lower().startswith('on'): raise ValueError('Event handler unsupported')
  if key=='href' and not x.startswith('#'): raise ValueError('External resource unsupported')
  if key=='style': raise ValueError('Inline CSS unsupported; use SVG presentation attributes')
  if 'url(' in x.lower() and not __import__('re').fullmatch(r'url\(\s*[\x27\x22]?#[\w.-]+[\x27\x22]?\s*\)',x): raise ValueError('External CSS resource unsupported')
 if e.get('id'):
  if e.get('id') in ids: raise ValueError('Duplicate SVG id')
  ids.append(e.get('id'))
print(json.dumps({'viewBox':v,'groups':sum(e.tag.split('}')[-1]=='g' for e in r.iter()),'paths':sum(e.tag.split('}')[-1]=='path' for e in r.iter()),'text_elements':sum(e.tag.split('}')[-1]=='text' for e in r.iter()),'ids':ids}))
`], {input: source, encoding:'utf8', timeout:15000, maxBuffer:1024*1024});
  if (validate.status !== 0) throw new Error('SVG preflight: ' + (validate.stderr || validate.error?.message || 'failed'));
  const structure = JSON.parse(validate.stdout);
  const req = createRequire(path.resolve(process.env.HARNESS_NODE_MODULES || './node_modules', '__workbench__.cjs'));
  let sharp;
  try { sharp = req('sharp'); } catch { throw new Error('sharp is missing; configure an existing Node module directory (no auto-install)'); }
  fs.mkdirSync(output);
  fs.writeFileSync(path.join(output, 'source.svg'), raw);
  const artifacts = [{path:'source.svg',sha256:hash(raw)}];
  const backgrounds = [{name:'light',color:'#f5f3ed',ink:'#202824'},{name:'dark',color:'#202824',ink:'#f5f3ed'}];
  const sizes = [24,64,256,1024];
  let cells='';
  for (const [row,bg] of backgrounds.entries()) {
    for (const [col,size] of sizes.entries()) {
      const image = await sharp(raw,{density:144,limitInputPixels:16*1024*1024}).resize(size,size,{fit:'contain',background:{r:0,g:0,b:0,alpha:0}}).png().toBuffer();
      const name=`${bg.name}-${size}.png`;
      const composed=await sharp(image).flatten({background:bg.color}).png().toBuffer();
      fs.writeFileSync(path.join(output,name),composed); artifacts.push({path:name,sha256:hash(composed)});
      const display=Math.min(size,220), x=col*260, y=row*310;
      cells+=`<rect x="${x}" y="${y}" width="260" height="310" fill="${bg.color}"/><image x="${x+(260-display)/2}" y="${y+30+(220-display)/2}" width="${display}" height="${display}" href="data:image/png;base64,${image.toString('base64')}"/><text x="${x+20}" y="${y+284}" font-family="sans-serif" font-size="14" fill="${bg.ink}">${size}px · ${bg.name}</text>`;
    }
  }
  const sheet=await sharp(Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" width="1040" height="620">${cells}</svg>`)).png().toBuffer();
  fs.writeFileSync(path.join(output,'contact-sheet.png'),sheet);artifacts.push({path:'contact-sheet.png',sha256:hash(sheet)});
  const report={schema_version:1,kind:'vector',input,source_sha256:hash(raw),structure,renderer:sharp.versions,artifacts,checks_passed:true,
    review_required:['Inspect silhouette and optical balance at actual small sizes.','Inspect light/dark contrast, clipped effects and editable groups.','Test repeated inline instances and motion in the actual browser integration.'],
    limits:['Static rasterization is not aesthetic, accessibility or animation acceptance.','Text rendering depends on installed fonts. Active/raster/CSS imports are rejected; this is not a general SVG sanitizer.']};
  fs.writeFileSync(path.join(output,'report.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
} catch(error) { console.error(error.message); process.exitCode=2; }
