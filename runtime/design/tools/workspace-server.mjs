import http from 'node:http';
import fs from 'node:fs/promises';
import {constants} from 'node:fs';
import {pipeline} from 'node:stream/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {workspace, workspacePath} from './paths.mjs';
import {redactSecrets} from './request.mjs';

const contentTypes = new Map(Object.entries({
  '.html': 'text/html; charset=utf-8', '.css': 'text/css; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.mjs': 'text/javascript; charset=utf-8',
  '.json': 'application/json', '.svg': 'image/svg+xml', '.png': 'image/png', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', '.webp': 'image/webp', '.avif': 'image/avif',
  '.woff': 'font/woff', '.woff2': 'font/woff2', '.ttf': 'font/ttf', '.otf': 'font/otf', '.pdf': 'application/pdf', '.mp4': 'video/mp4', '.webm': 'video/webm',
}));
const MAX_HTTP_BYTES = 64 * 1024 * 1024;
const directoryOpenFlags = constants.O_RDONLY | constants.O_DIRECTORY | constants.O_NOFOLLOW | constants.O_NONBLOCK;
const fileOpenFlags = constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK;

function sameFile(left, right) { return left.dev === right.dev && left.ino === right.ino; }

async function openWorkspaceFile(relative, afterValidation) {
  const file = await workspacePath(relative);
  const parts = relative.split('/');
  const expected = [await fs.lstat(workspace)];
  let current = workspace;
  for (const part of parts) { current = path.join(current, part); expected.push(await fs.lstat(current)); }
  await afterValidation?.({file, relative});

  let directory, handle;
  try {
    directory = await fs.open(workspace, directoryOpenFlags);
    let actual = await directory.stat();
    if (!actual.isDirectory() || !sameFile(actual, expected[0])) throw Error('Workspace changed during request validation');
    for (let index = 0; index < parts.length - 1; index++) {
      const next = await fs.open(`/proc/self/fd/${directory.fd}/${parts[index]}`, directoryOpenFlags);
      try {
        actual = await next.stat();
        if (!actual.isDirectory() || !sameFile(actual, expected[index + 1])) throw Error('Workspace parent changed during request validation');
      } catch (error) { await next.close().catch(() => {}); throw error; }
      await directory.close();
      directory = next;
    }
    handle = await fs.open(`/proc/self/fd/${directory.fd}/${parts.at(-1)}`, fileOpenFlags);
    actual = await handle.stat();
    if (!actual.isFile() || !sameFile(actual, expected.at(-1))) throw Error('Workspace file changed during request validation');
    return {file, handle, stat: actual};
  } catch (error) {
    await handle?.close().catch(() => {});
    throw error;
  } finally { await directory?.close().catch(() => {}); }
}

export async function handleWorkspaceRequest(request, response, hooks = {}) {
  try {
    if (request.method !== 'GET' && request.method !== 'HEAD') { response.writeHead(405); response.end(); return; }
    const url = new URL(request.url, 'http://design-tools:9002');
    if (url.pathname === '/health') {
      response.writeHead(200, {'Content-Type': 'application/json'});
      response.end(JSON.stringify({workspace: true, runtime: true}));
      return;
    }
    if (url.pathname === '/favicon.ico' && !url.search && !url.hash) {
      response.writeHead(204, {'Cache-Control': 'no-store'});
      response.end();
      return;
    }
    if (!url.pathname.startsWith('/files/') || url.search || url.hash) { response.writeHead(404); response.end(); return; }
    const raw = url.pathname.slice('/files/'.length);
    const relative = raw.split('/').map(decodeURIComponent).join('/');
    const {file, handle, stat} = await openWorkspaceFile(relative, hooks.afterValidation);
    try {
      if (stat.size > MAX_HTTP_BYTES) { response.writeHead(413); response.end(); return; }
      const type = contentTypes.get(path.extname(file).toLowerCase());
      if (!type) { response.writeHead(415); response.end(); return; }
      await hooks.afterStat?.({file, relative, size: stat.size});
      response.writeHead(200, {'Content-Type': type, 'Content-Length': stat.size, 'X-Content-Type-Options': 'nosniff', 'Cache-Control': 'no-store', 'Content-Security-Policy': "default-src 'self' data: blob: 'unsafe-inline'"});
      if (request.method === 'HEAD' || stat.size === 0) response.end();
      else await pipeline(handle.createReadStream({autoClose: false, start: 0, end: stat.size - 1}), response);
    } finally { await handle.close().catch(() => {}); }
  } catch (error) {
    if (response.headersSent) response.destroy();
    else {
      const missing = error.code === 'ENOENT' || String(error.message).startsWith('Missing workspace path:');
      response.writeHead(missing ? 404 : 400, {'Content-Type': 'text/plain'});
      response.end(redactSecrets(error.message).slice(0, 300));
    }
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
const server = http.createServer(handleWorkspaceRequest);
for (const signal of ['SIGTERM', 'SIGINT']) process.once(signal, () => server.close(() => process.exit(0)));
server.listen(9002, '0.0.0.0', () => process.stderr.write('Workspace server listening on 9002\n'));
}
