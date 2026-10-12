// Fixed offline job launcher. Host paths, commands and project mounts never enter this contract.
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawn} from 'node:child_process';
const key=process.argv[2];
if(!/^[A-Za-z0-9-]{1,64}$/.test(key||''))throw Error('Invalid private job key');
const manifest=JSON.parse(await fs.readFile(`/input/${key}/launch.json`,'utf8'));
for(const name of await fs.readdir('/work'))await fs.rm(`/work/${name}`,{recursive:true,force:true});
for(const name of await fs.readdir('/output'))await fs.rm(`/output/${name}`,{recursive:true,force:true});
await fs.mkdir(`/output/${key}`);
const work=`/work/${key}`;
if(!['motion-canvas','hyperframes'].includes(manifest.package)||!Array.isArray(manifest.args)||manifest.args.length>16)throw Error('Invalid fixed job contract');
await fs.mkdir('/tmp/home',{recursive:true});
async function copy(input,output){
 await fs.mkdir(output,{recursive:true});
 for(const entry of await fs.readdir(input,{withFileTypes:true})){
  if(entry.name==='launch.json'&&input===`/input/${key}`)continue;
  const from=path.join(input,entry.name),to=path.join(output,entry.name);
  if(entry.isDirectory())await copy(from,to);
  else if(entry.isFile())await fs.copyFile(from,to,fs.constants.COPYFILE_EXCL);
  else throw Error('Job links and special files are refused');
 }
}
await copy(`/input/${key}`,work);
const child=spawn('/opt/nodejs/bin/node',[`/opt/${manifest.package}/runner.mjs`,...manifest.args],{cwd:work,stdio:['ignore','inherit','inherit']});
for(const signal of ['SIGTERM','SIGINT'])process.once(signal,()=>child.kill(signal));
const code=await new Promise((resolve,reject)=>{child.once('error',reject);child.once('close',resolve)});
if(code!==0)process.exit(code||1);
for(const name of ['output.png','output.mp4','output.webm','output.svg','report.json']){
 const stat=await fs.lstat(`${work}/${name}`).catch(()=>null);if(!stat)continue;
 if(!stat.isFile()||stat.isSymbolicLink()||stat.size===0||stat.size>32*1024*1024)throw Error('Output is not usable bounded regular data');
 await fs.copyFile(`${work}/${name}`,`/output/${key}/${name}`,fs.constants.COPYFILE_EXCL);
}
