// Frozen Daybreak pixel regression checks, not a general aesthetic evaluator.
// Usage: PRIVATE_NODE check-export.mjs ABS_FFMPEG ABS_VIDEO
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {isAbsolute} from 'node:path';
import {fileURLToPath} from 'node:url';
const [ffmpeg,video]=process.argv.slice(2);
for(const path of [ffmpeg,video])assert.ok(path&&isAbsolute(path),'Explicit absolute executable/video paths required');
function rgb(file,seconds){
 const args=['-v','error','-nostdin','-i',file];
 if(seconds!==undefined)args.push('-ss',String(seconds));
 args.push('-frames:v','1','-f','rawvideo','-pix_fmt','rgb24','pipe:1');
 const result=spawnSync(ffmpeg,args,{maxBuffer:4*1024*1024,timeout:30000});
 assert.equal(result.status,0,result.stderr?.toString());
 assert.equal(result.stdout.length,1280*720*3,'Expected frozen 1280×720 raster');
 return result.stdout;
}
function count(bytes,box,predicate){
 let result=0;for(let y=box[1];y<box[3];y++)for(let x=box[0];x<box[2];x++){
 const offset=(y*1280+x)*3;if(predicate(bytes[offset],bytes[offset+1],bytes[offset+2]))result++;
 }return result;
}
const lime=(r,g,b)=>r>150&&g>160&&b<160&&g-b>50;
const cream=(r,g,b)=>r>225&&g>225&&b>210;
// Independent control: same Mirket renderer/time/font with only the opening clip-path disabled.
const control=rgb(fileURLToPath(new URL('./intro-unmasked-control.png',import.meta.url)));
const opening=rgb(video,.7),hero=rgb(video,3);
const checks=[];
for(const [name,box] of [['complete y descender',[495,444,555,457]],['complete k edge',[939,330,953,421]]]){
 const expected=count(control,box,lime),actual=count(opening,box,lime);
 checks.push({name,expected,actual,pass:expected>50&&actual>=expected*.85});
}
// The original hero painted 421px down, virtually touching the CTA at418px.
// Reserve the first18px below the button; background is dark in this region.
const intrusion=count(hero,[70,420,195,438],cream);
checks.push({name:'clear space below hero CTA',intrusion,pass:intrusion<10});
const transition=rgb(video,4);let run=0,longest=0;
for(let x=0;x<1280;x++){
 let matching=0;for(let y=10;y<710;y++){
  const i=(y*1280+x)*3;
  if(Math.abs(transition[i]-48)<=5&&Math.abs(transition[i+1]-35)<=5&&Math.abs(transition[i+2]-46)<=5)matching++;
 }
 run=matching/700>.97?run+1:0;longest=Math.max(longest,run);
}
checks.push({name:'no exposed full-height stage gutter',pixels:longest,pass:longest<=4});
console.log(JSON.stringify({video,checks,passed:checks.every(c=>c.pass)},null,2));
if(checks.some(c=>!c.pass))process.exitCode=1;
