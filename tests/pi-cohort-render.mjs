// Test-only shared local renderer. The proxy never opens an outbound connection.
import fs from 'node:fs/promises';
import http from 'node:http';
import {join,resolve,sep,extname} from 'node:path';
import {pathToFileURL} from 'node:url';
export async function renderLocal({cwd,file,width=640,height=360,time=0,reduced=false,browserPeers,chrome,output}){
 const root=await fs.realpath(cwd),target=resolve(root,file);if(!target.startsWith(root+sep))throw new Error('Preview outside fixture');
 const requests=[];const proxy=http.createServer(async(req,res)=>{
  requests.push(req.url);try{const url=new URL(req.url);if(url.origin!=='http://cohort.invalid'||req.method!=='GET')throw Error('Nonlocal');
   const path=await fs.realpath(resolve(root,'.'+decodeURIComponent(url.pathname)));if(!path.startsWith(root+sep))throw Error('Escaped root');
   const type={'.html':'text/html','.svg':'image/svg+xml','.css':'text/css','.js':'text/javascript','.ttf':'font/ttf'}[extname(path)]??'application/octet-stream';
   res.writeHead(200,{'Content-Type':type,'Content-Security-Policy':"default-src 'self' 'unsafe-inline'; connect-src 'none'; frame-src 'none'; object-src 'none'"});res.end(await fs.readFile(path));
  }catch{res.writeHead(403).end('local-only');}
 });proxy.on('connect',(_req,socket)=>socket.end('HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n'));
 await new Promise(r=>proxy.listen(0,'127.0.0.1',r));let browser;
 try{
  const {chromium}=await import(pathToFileURL(join(browserPeers,'playwright/index.mjs')));
  browser=await chromium.launch({executablePath:chrome,headless:true,proxy:{server:`http://127.0.0.1:${proxy.address().port}`,bypass:'<-loopback>'},args:['--disable-quic','--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE 127.0.0.1']});
  const context=await browser.newContext({viewport:{width,height},reducedMotion:reduced?'reduce':'no-preference',serviceWorkers:'block'}),page=await context.newPage();
  await page.goto('http://cohort.invalid/'+file,{waitUntil:'load',timeout:10000});await page.evaluate(async()=>document.fonts.ready);
  await page.evaluate(t=>{for(const a of document.getAnimations()){a.pause();a.currentTime=t*1000;}},time);
  const state=await page.evaluate(()=>({text:document.body?.innerText??document.documentElement.textContent,scrollWidth:document.documentElement.scrollWidth,width:innerWidth,animations:document.getAnimations().length}));
  await page.screenshot({path:output,scale:'css'});
  await page.keyboard.press('Tab');const focus=await page.evaluate(()=>({tag:document.activeElement?.tagName,outline:document.activeElement?getComputedStyle(document.activeElement).outlineStyle:null}));await page.keyboard.press('Enter');
  const after=await page.evaluate(()=>({text:document.body?.innerText,pressed:document.querySelector('[aria-pressed]')?.getAttribute('aria-pressed')}));
  return {output,state,focus,after,requests};
 }finally{await browser?.close();proxy.closeAllConnections();await new Promise(r=>proxy.close(r));}
}
