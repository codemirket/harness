// Source checks only. This does not replace a rendered browser journey.
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';

const root = path.resolve(import.meta.dirname, '..');
const html = fs.readFileSync(path.join(root, 'index.html'), 'utf8');
const sourceBytes = fs.readFileSync(path.join(root, 'capabilities.json'));
const source = JSON.parse(sourceBytes).capabilities;
const appScript = html.match(/<script>([\s\S]*?)<\/script>/)[1];
const embedded = JSON.parse(html.match(/<script id="capability-data" type="application\/json">([\s\S]*?)<\/script>/)[1]);
const checks = [];
const pass = (name, details) => { checks.push({name, result:'pass', ...(details ? {details} : {})}); console.log(`PASS ${name}`); };

new vm.Script(appScript);
pass('Application JavaScript parses');
assert.deepEqual(embedded, source.map(({id,title,aliases,deliverable,acceptance}) => ({id,title,aliases,deliverable,acceptance})));
assert.equal(new Set(embedded.map(item => item.id)).size, source.length);
pass('Embedded capabilities exactly preserve all source role, deliverable, and acceptance fields', `${source.length} capabilities`);
assert.equal(embedded.every(item => item.aliases.length && item.acceptance.length && item.deliverable),true);
pass('All capabilities have display roles, deliverables, and acceptance evidence');

const ids = [...html.matchAll(/\bid="([^"]+)"/g)].map(match => match[1]);
assert.equal(ids.length,new Set(ids).size);
for (const match of html.matchAll(/\b(?:aria-labelledby|aria-describedby|aria-controls)="([^"]+)"/g)) {
  for (const id of match[1].split(' ')) assert.ok(ids.includes(id), `Missing ARIA target ${id}`);
}
pass('Unique element IDs and valid static ARIA reference targets');
assert.equal(/<(?:script|link|img|iframe|audio|video)[^>]+\b(?:src|href)\s*=/i.test(html),false);
assert.equal(/@import|url\s*\(\s*["']?https?:|\bfetch\s*\(|\bXMLHttpRequest\b|\bWebSocket\b|navigator\.sendBeacon/.test(html),false);
pass('No external assets, imports, fetches, or service connections in the artifact');
for (const token of ['schema_version','prerequisites','failure_probe','"lead"','"support"']) assert.equal(html.includes(token),false);
pass('Installer/schema/support metadata is absent from the embedded product data');
assert.match(html, /<label[^>]+for="search"/);
assert.match(html, /role="status" aria-live="polite" aria-atomic="true"/);
assert.match(html, /@media\(prefers-reduced-motion:reduce\)/);
assert.match(html, /<noscript>/);
pass('Search label, live result status, reduced-motion rule, and JavaScript-disabled guidance are present');

// Execute the shipped search expressions verbatim, without a DOM or network.
// This verifies the matching rules, not rendered controls or focus behavior.
const normalizeLine = appScript.match(/const normalize =[^\n]+/)[0];
const searchableLine = appScript.match(/const searchable =[^\n]+/)[0];
const tokenLine = appScript.match(/const tokens =[^\n]+/)[0];
const matchesLine = appScript.match(/matches = query &&[^\n]+/)[0];
const context = vm.createContext({capabilities:embedded});
vm.runInContext(`${normalizeLine}\n${searchableLine}\nfunction find(query) {${tokenLine}\nlet matches;${matchesLine}\nreturn matches.map(item => item.id);}`,context);
const find = query => vm.runInContext(`find(${JSON.stringify(query)})`,context);
let aliases = 0;
for (const item of source) for (const alias of item.aliases) {assert.ok(find(alias).includes(item.id),alias);aliases++;}
assert.ok(find('cFo').includes('financial-analysis'));
assert.ok(find('Chief Financial Officer').includes('financial-analysis'));
assert.ok(find('  web designer  ').includes('web-design'));
assert.ok(find('ui-designer').includes('web-design'));
assert.ok(find('cash timing').includes('financial-analysis'));
assert.equal(find('lunar pastry architect').length,0);
assert.equal(find('!!!').length,0);
assert.equal(find('').length,source.length);
pass('Shipped search expressions find every source alias and handle case, punctuation, outcome keywords, blanks, and no matches', `${aliases} aliases; expression-level check only`);

const colors = {ink:'#22372c',muted:'#5e685f',green:'#28533c',paper:'#f5f5ef',surface:'#fffefa',wash:'#eaf0df',list:'#f0f2e9',hover:'#e7ecdd',focus:'#8b451d',placeholder:'#657063',white:'#ffffff'};
const luminance = hex => {
  const rgb=hex.slice(1).match(/../g).map(x=>parseInt(x,16)/255).map(x=>x<=.04045?x/12.92:((x+.055)/1.055)**2.4);
  return .2126*rgb[0]+.7152*rgb[1]+.0722*rgb[2];
};
const pairs = [['ink','paper'],['ink','surface'],['ink','wash'],['muted','paper'],['muted','surface'],['muted','wash'],['muted','list'],['muted','hover'],['green','wash'],['white','green'],['placeholder','surface'],['focus','surface'],['focus','wash'],['focus','list']];
const contrast = pairs.map(([foreground,background]) => {
  const a=luminance(colors[foreground]),b=luminance(colors[background]);
  const ratio=(Math.max(a,b)+.05)/(Math.min(a,b)+.05);
  assert.ok(ratio>=4.5,`${foreground}/${background}: ${ratio}`);
  return {foreground,background,ratio:Number(ratio.toFixed(2))};
});
pass('Declared text and focus color combinations exceed 4.5:1', `Minimum ${Math.min(...contrast.map(x=>x.ratio))}:1; palette calculation, not rendered accessibility audit`);
const digest = data => crypto.createHash('sha256').update(data).digest('hex');
fs.writeFileSync(path.join(root,'qa','source-checks.json'),JSON.stringify({
  checkedAt:new Date().toISOString(),method:'Static/source checks and execution of shipped search expressions. No browser rendering.',
  files:{'index.html':digest(html),'capabilities.json':digest(sourceBytes)},checks,contrast,
  unverified:['Rendered desktop/mobile appearance','Actual browser search/selection/focus journey','Physical touch and on-screen keyboard','Screen-reader behavior']
},null,2)+'\n');
