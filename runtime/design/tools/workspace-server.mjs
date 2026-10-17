import http from 'node:http';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawn} from 'node:child_process';
import {workspacePath} from './paths.mjs';
import {redactSecrets} from './request.mjs';

const contentTypes = new Map(Object.entries({
  '.html': 'text/html; charset=utf-8', '.css': 'text/css; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.mjs': 'text/javascript; charset=utf-8',
  '.json': 'application/json', '.svg': 'image/svg+xml', '.png': 'image/png', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', '.webp': 'image/webp', '.avif': 'image/avif',
  '.woff': 'font/woff', '.woff2': 'font/woff2', '.ttf': 'font/ttf', '.otf': 'font/otf', '.pdf': 'application/pdf', '.mp4': 'video/mp4', '.webm': 'video/webm',
}));
let sessionReady = false, sessionExit = false;
const server = http.createServer(async (request, response) => {
  try {
    if (request.method !== 'GET' && request.method !== 'HEAD') { response.writeHead(405); response.end(); return; }
    const url = new URL(request.url, 'http://design-tools:9002');
    if (url.pathname === '/health') {
      response.writeHead(sessionReady && !sessionExit ? 200 : 503, {'Content-Type': 'application/json'});
      response.end(JSON.stringify({workspace: true, penpot_session: sessionReady && !sessionExit}));
      return;
    }
    if (!url.pathname.startsWith('/files/') || url.search || url.hash) { response.writeHead(404); response.end(); return; }
    const raw = url.pathname.slice('/files/'.length);
    const relative = raw.split('/').map(decodeURIComponent).join('/');
    const file = await workspacePath(relative);
    const stat = await fs.lstat(file);
    if (stat.size > 64 * 1024 * 1024) { response.writeHead(413); response.end(); return; }
    const type = contentTypes.get(path.extname(file).toLowerCase());
    if (!type) { response.writeHead(415); response.end(); return; }
    const bytes = request.method === 'HEAD' ? null : await fs.readFile(file);
    response.writeHead(200, {'Content-Type': type, 'Content-Length': stat.size, 'X-Content-Type-Options': 'nosniff', 'Cache-Control': 'no-store', 'Content-Security-Policy': "default-src 'self' data: blob: 'unsafe-inline'"});
    response.end(bytes);
  } catch (error) {
    response.writeHead(400, {'Content-Type': 'text/plain'});
    response.end(redactSecrets(error.message).slice(0, 300));
  }
});

const child = spawn(process.execPath, ['/opt/design/penpot-session.mjs'], {stdio: ['ignore', 'pipe', 'pipe'], env: process.env});
let lineBuffer = '';
child.stderr.on('data', chunk => {
  lineBuffer += chunk.toString('utf8');
  for (;;) {
    const index = lineBuffer.indexOf('\n');
    if (index < 0) break;
    const line = lineBuffer.slice(0, index); lineBuffer = lineBuffer.slice(index + 1);
    try { const event = JSON.parse(line); if (event.ready === true || event.event === 'ready') sessionReady = true; } catch {}
    process.stderr.write(`[penpot-session] ${redactSecrets(line)}\n`);
  }
});
child.stdout.on('data', chunk => process.stderr.write(`[penpot-session] ${redactSecrets(chunk.toString('utf8'))}`));
child.on('exit', (code, signal) => {
  sessionExit = true;
  process.stderr.write(`Penpot session exited unexpectedly (${code ?? signal})\n`);
  server.close(() => process.exit(1));
});
for (const signal of ['SIGTERM', 'SIGINT']) process.once(signal, () => { child.kill(signal); server.close(() => process.exit(0)); });
server.listen(9002, '0.0.0.0', () => process.stderr.write('Workspace server listening on 9002\n'));
