// Serve the immutable official plugin build and supervise the compiled MCP server.
import http from 'node:http';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawn} from 'node:child_process';
const types = {'.html':'text/html; charset=utf-8', '.js':'text/javascript; charset=utf-8', '.css':'text/css', '.json':'application/json', '.svg':'image/svg+xml', '.png':'image/png', '.jpg':'image/jpeg', '.jpeg':'image/jpeg', '.woff2':'font/woff2'};
const root = '/opt/plugin';
const server = http.createServer(async (req, res) => {
  try {
    // Public immutable plugin assets are fetched by the separate Penpot origin.
    res.setHeader('Access-Control-Allow-Origin', '*');
    if (req.method === 'OPTIONS') { res.writeHead(204, {'Access-Control-Allow-Methods':'GET, HEAD, OPTIONS'}).end(); return; }
    if (!['GET','HEAD'].includes(req.method)) { res.writeHead(405).end(); return; }
    const url = new URL(req.url, 'http://penpot-mcp:4400');
    const relative = decodeURIComponent(url.pathname === '/' ? '/index.html' : url.pathname);
    const file = path.resolve(root, `.${relative}`);
    if (!file.startsWith(root + '/') || relative.includes('\0') || relative.includes('\\')) { res.writeHead(400).end(); return; }
    const real = await fs.realpath(file);
    if (!real.startsWith(root + '/')) { res.writeHead(400).end(); return; }
    const stat = await fs.stat(real);
    if (!stat.isFile() || stat.size > 16 * 1024 * 1024) { res.writeHead(404).end(); return; }
    res.writeHead(200, {'Content-Type': types[path.extname(real)] || 'application/octet-stream', 'Content-Length': stat.size, 'X-Content-Type-Options':'nosniff'});
    res.end(req.method === 'HEAD' ? undefined : await fs.readFile(real));
  } catch { if (!res.headersSent) res.writeHead(404); res.end(); }
});
const child = spawn(process.execPath, ['/opt/server/dist/index.js'], {cwd:'/opt/server', stdio:'inherit', detached:true});
let stopping = false;
function stop(code = 0) {
  if (stopping) return;
  stopping = true;
  server.close();
  try { process.kill(-child.pid, 'SIGTERM'); } catch {}
  const timer = setTimeout(() => { try { process.kill(-child.pid, 'SIGKILL'); } catch {} process.exit(code); }, 3000);
  child.once('close', () => { clearTimeout(timer); process.exit(code); });
}
child.once('error', () => stop(1));
child.once('exit', code => stop(code || 1));
server.once('error', () => stop(1));
for (const signal of ['SIGINT','SIGTERM']) process.once(signal, () => stop());
server.listen(4400, '0.0.0.0');
