// Mirket's service boundary; parser implementations remain upstream Kroki.
import http from 'node:http';
import {spawn} from 'node:child_process';
const types = new Set(['plantuml','c4plantuml','vegalite','excalidraw','d2','graphviz']);
export function permitted(method, url) {
  return method === 'GET' && url === '/health' || method === 'POST' && /^\/([a-z0-9]+)\/(svg|png)$/.test(url) && types.has(url.split('/')[1]);
}
export async function handle(req, res) {
  if (!permitted(req.method,req.url)) {res.writeHead(404);res.end();return;}
  const controller = new AbortController();
  const timer = setTimeout(()=>controller.abort(),60000);
  res.on('close',()=>controller.abort());
  try {
    const chunks=[];let size=0;
    for await (const chunk of req) {size+=chunk.length;if(size>262144)throw Error('Input exceeds limit');chunks.push(chunk);}
    const reply=await fetch('http://127.0.0.1:8001'+req.url,{method:req.method,redirect:'error',
      headers:{'content-type':'text/plain; charset=utf-8'},body:req.method==='POST'?Buffer.concat(chunks):undefined,signal:controller.signal});
    const output=[];size=0;
    for await (const chunk of reply.body) {size+=chunk.length;if(size>16*1024*1024)throw Error('Output exceeds limit');output.push(chunk);}
    res.writeHead(reply.status,{'Content-Type':reply.headers.get('content-type')||'application/octet-stream','X-Content-Type-Options':'nosniff'});
    res.end(Buffer.concat(output));
  } catch {if(!res.headersSent)res.writeHead(502);res.end();}
  finally {clearTimeout(timer);}
}
if (process.argv[1]?.endsWith('/gateway.mjs')) {
  const backend=spawn('/usr/bin/java',['-XX:MaxRAMPercentage=60','-jar','/opt/kroki/kroki-server.jar'],{
    stdio:['ignore','inherit','inherit'],env:{...process.env,KROKI_LISTEN:'127.0.0.1:8001',KROKI_SAFE_MODE:'SECURE',KROKI_ENABLE_COMPANION_DISCOVERY:'false'}});
  const server=http.createServer({requestTimeout:65000,headersTimeout:10000,maxHeaderSize:8192},handle);
  backend.once('error',()=>process.exit(1));backend.once('exit',()=>server.close(()=>process.exit(1)));
  for(const signal of ['SIGTERM','SIGINT'])process.once(signal,()=>{backend.kill(signal);server.close();});
  server.listen(8000,'0.0.0.0');
}
