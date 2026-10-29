// Real macOS Chrome qualification. All writes stay in one disposable private root.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {bindRuntime,installRuntime,skipRuntime,runtimeToolStatus,executableIdentity} from '../pi/dist/extension/runtime-tools.js';
const [chrome,archivePath]=process.argv.slice(2);
assert.ok(chrome&&archivePath&&path.isAbsolute(chrome)&&path.isAbsolute(archivePath),'Supply explicit Chrome executable and exact pinned publisher ZIP paths');
const root=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'mirket-real-runtime-')));fs.chmodSync(root,0o700);
const pin=JSON.parse(fs.readFileSync(new URL('../packaging/runtime/chrome-macos.json',import.meta.url)));
const spec={id:'chromium',target:pin.target,title:'Chrome for video rendering',version:pin.version,terms:'Google Chrome terms apply. Test replays independently acquired publisher archive.',archive:pin.archive,format:'zip',executable:pin.executable,files:pin.files,links:pin.links};
const home=path.join(root,'home'),home2=path.join(root,'pinned');fs.mkdirSync(home,{mode:0o700});fs.mkdirSync(home2,{mode:0o700});
const evidence={schema:1,platform:process.platform,arch:process.arch,node:process.version,nodeSha256:executableIdentity(process.execPath),observedAt:new Date().toISOString(),root,archiveReplay:!process.argv.includes('--network'),checks:[]};
try{
 await bindRuntime(home,'fixture-real',spec,chrome,new AbortController().signal,()=>{});assert.equal(runtimeToolStatus(home,'fixture-real',[spec])[0].status,'ready');evidence.checks.push('selected-installed-Chrome-local-JavaScript-DOM');
 skipRuntime(home,'fixture-real','chromium');assert.equal(runtimeToolStatus(home,'fixture-real',[spec])[0].status,'missing');evidence.checks.push('skip-missing');
 const bytes=fs.readFileSync(archivePath);assert.equal(createHash('sha256').update(bytes).digest('hex'),pin.archive.sha256);
 await installRuntime(home2,'fixture-real',spec,new AbortController().signal,()=>{},process.argv.includes('--network')?undefined:async()=>new Response(bytes));assert.equal(runtimeToolStatus(home2,'fixture-real',[spec])[0].status,'ready');evidence.checks.push('pinned-publisher-archive-and-every-file-link-verified');evidence.checks.push('pinned-Chrome-local-JavaScript-DOM');
 const status=runtimeToolStatus(home2,'fixture-real',[spec])[0];evidence.selectedExecutableSha256=status.sha256;evidence.archiveSha256=pin.archive.sha256;
 const real=path.join(root,'real-operation');fs.mkdirSync(real,{mode:0o700});
 // A fresh rendered artifact after onboarding proves the bound executable works beyond setup.
 const {execFileSync}=await import('node:child_process');const page=path.join(real,'render.html'),png=path.join(real,'render.png');fs.writeFileSync(page,'<!doctype html><body style="background:#164e63;color:white"><h1>Onboarded runtime</h1></body>');
 execFileSync(status.path,['--headless','--disable-gpu','--use-mock-keychain','--password-store=basic','--disable-background-networking','--disable-component-update','--disable-sync','--disable-extensions','--no-first-run','--no-default-browser-check',`--user-data-dir=${path.join(real,'profile')}`,`--screenshot=${png}`,'--window-size=320,240',new URL('file://'+page).href],{cwd:real,env:{HOME:real,TMPDIR:real,PATH:'',LANG:'C'},timeout:30000,stdio:'pipe'});const output=fs.readFileSync(png);assert.equal(output.subarray(0,8).toString('hex'),'89504e470d0a1a0a');assert.equal(output.readUInt32BE(16),320);assert.equal(output.readUInt32BE(20),240);assert.ok(output.length>500);evidence.checks.push('post-onboarding-real-Chrome-320x240-render');evidence.operationOutputSha256=createHash('sha256').update(output).digest('hex');
 console.log(JSON.stringify(evidence,null,2));
}finally{fs.rmSync(root,{recursive:true,force:true});}
