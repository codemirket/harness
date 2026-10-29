import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {prepareProfile,applyProfile} from '../dist/extension/profiles.js';
function fixture(){const root=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'mirket-profile-')));const pkg=path.join(root,'package');fs.mkdirSync(path.join(pkg,'skills'),{recursive:true});fs.writeFileSync(path.join(pkg,'skills/catalog.json'),JSON.stringify({global:[{id:'core'}],project:[{id:'a'},{id:'b'},{id:'conditional'}],profiles:{one:{skills:['a'],conditional:['conditional']},two:{skills:['a','b'],conditional:[]}}}));const file=path.join(root,'settings.json');return {root,pkg,file,close(){fs.rmSync(root,{recursive:true,force:true})}};}
test('profile union preserves unrelated settings and owned extension filters, without enabling conditional skills',()=>{
 const f=fixture();try{fs.writeFileSync(f.file,JSON.stringify({theme:'preserved',packages:[{source:'/other',skills:['other']},{source:f.pkg,extensions:[],autoload:false}]}));applyProfile(prepareProfile(f.pkg,f.file,['one','two'],[]));const result=JSON.parse(fs.readFileSync(f.file));assert.equal(result.theme,'preserved');assert.deepEqual(result.packages[0],{source:'/other',skills:['other']});assert.deepEqual(result.packages[1],{source:f.pkg,extensions:[],autoload:false,skills:['skills/a/SKILL.md','skills/b/SKILL.md','skills/core/SKILL.md']});assert.equal(fs.existsSync(path.join(f.root,'.agents')),false);}finally{f.close();}
});
test('stale settings, duplicate package ownership and symlink target refuse without overwriting user data',()=>{
 const f=fixture();try{fs.writeFileSync(f.file,'{}');const edit=prepareProfile(f.pkg,f.file,['one'],[]);fs.writeFileSync(f.file,'{"theme":"concurrent"}');assert.throws(()=>applyProfile(edit),/Settings changed/);assert.equal(fs.readFileSync(f.file,'utf8'),'{"theme":"concurrent"}');fs.writeFileSync(f.file,JSON.stringify({packages:[f.pkg,{source:f.pkg}]}));assert.throws(()=>applyProfile(prepareProfile(f.pkg,f.file,['one'],[])),/Duplicate/);fs.unlinkSync(f.file);const user=path.join(f.root,'user.json');fs.writeFileSync(user,'{}');fs.symlinkSync(user,f.file);assert.throws(()=>prepareProfile(f.pkg,f.file,['one'],[]));assert.equal(fs.readFileSync(user,'utf8'),'{}');}finally{f.close();}
});
test('pi-written relative package source updates in place and alias duplicates refuse',()=>{
 const f=fixture();try{
 fs.writeFileSync(f.file,JSON.stringify({packages:[{source:'./package',extensions:[],autoload:false}]}));
 applyProfile(prepareProfile(f.pkg,f.file,['one'],[]));const result=JSON.parse(fs.readFileSync(f.file));
 assert.equal(result.packages.length,1);assert.equal(result.packages[0].source,f.pkg);assert.deepEqual(result.packages[0].extensions,[]);assert.equal(result.packages[0].autoload,false);
 fs.symlinkSync(f.pkg,path.join(f.root,'alias'));fs.writeFileSync(f.file,JSON.stringify({packages:['./package','./alias']}));const before=fs.readFileSync(f.file,'utf8');
 assert.throws(()=>applyProfile(prepareProfile(f.pkg,f.file,['one'],[])),/Duplicate/);assert.equal(fs.readFileSync(f.file,'utf8'),before);
 }finally{f.close();}
});
