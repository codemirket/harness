// One job per private user/mount/PID/network namespace. Only a bounded snapshot
// is readable; this worker never receives the live project or service network.
import fs from 'node:fs/promises';
import http from 'node:http';
import path from 'node:path';

const request = JSON.parse(await fs.readFile('/request.json', 'utf8'));
process.env.MIRKET_BROWSER_JOB_PREFIX=request.jobPrefix;
process.env.MIRKET_BROWSER_TOKEN_PATHS=JSON.stringify(request.tokenPaths);
const {browserCaptureLocal}=await import('./operations.mjs');
const types = {'.html':'text/html','.css':'text/css','.js':'text/javascript','.mjs':'text/javascript','.svg':'image/svg+xml','.png':'image/png','.jpg':'image/jpeg','.webp':'image/webp','.woff':'font/woff','.woff2':'font/woff2','.ttf':'font/ttf','.json':'application/json'};
const server = http.createServer(async (req, res) => {
  try {
    const url = new URL(req.url, 'http://design-tools:9002');
    const relative = decodeURIComponent(url.pathname.slice('/files/'.length));
    if (!['GET','HEAD'].includes(req.method) || !url.pathname.startsWith('/files/') || relative.split('/').some(x => !x || x === '.' || x === '..' || x.includes('\\'))) throw Error('Invalid request');
    const target = await fs.realpath(path.join('/inputs', relative));
    if (!target.startsWith('/inputs/')) throw Error('Outside inputs');
    const stat = await fs.stat(target);
    if (!stat.isFile() || stat.size > 64 * 1024 * 1024) throw Error('Invalid file');
    const type = types[path.extname(target)];
    if (!type) throw Error('Unsupported file');
    res.writeHead(200, {'Content-Type':type,'X-Content-Type-Options':'nosniff','Content-Security-Policy':"default-src 'self' data: blob: 'unsafe-inline'"});
    res.end(req.method === 'HEAD' ? undefined : await fs.readFile(target));
  } catch { res.writeHead(404);res.end(); }
});
await new Promise((resolve,reject) => {server.once('error',reject);server.listen(9002,'127.0.0.1',resolve);});
try {
  const shot = await browserCaptureLocal(request.url, request.options);
  process.stdout.write(JSON.stringify({...shot,bytes:shot.bytes.toString('base64')}));
} finally { await new Promise(resolve => server.close(resolve)); }
