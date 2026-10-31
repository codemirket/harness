import { createServer, request, type IncomingHttpHeaders } from 'node:http';
import type { Socket } from 'node:net';
import { browserOrigin } from './browser-policy.js';

// This application proxy constrains the tested loopback HTTP route. It is not
// an OS network sandbox and does not make Chromium's other protocols safe.
export async function createBrowserProxy(origin: string, allowed: (method: string, url: string) => boolean) {
  browserOrigin(origin);
  const target = new URL(origin), sockets = new Set<Socket>(), upstream = new Set<ReturnType<typeof request>>();
  let closed = false, requests = 0, total = 0;
  const server = createServer({maxHeaderSize: 16384, requestTimeout: 10000, headersTimeout: 10000}, (req, res) => {
    const deny = (status = 403) => { res.writeHead(status, {'Connection': 'close', 'Content-Type': 'text/plain'}).end('Browser request refused'); req.resume(); };
    let url: URL;
    try { url = new URL(req.url ?? ''); } catch { deny(); return; }
    if (closed || ++requests > 256 || url.origin !== origin || url.username || url.password || !['GET', 'HEAD'].includes(req.method ?? '') || !allowed(req.method ?? '', url.href) || req.headers.upgrade || req.headers['transfer-encoding'] || (req.headers['content-length'] !== undefined && req.headers['content-length'] !== '0') || req.headers.authorization || req.headers.cookie) { deny(); return; }
    let body = 0;
    req.on('data', (chunk: Buffer) => { body += chunk.length; if (body) { pending.destroy(); res.destroy(); } });
    const headers: IncomingHttpHeaders = {host: target.host, connection: 'close'};
    for (const name of ['accept', 'accept-language', 'user-agent', 'range', 'if-none-match', 'if-modified-since']) if (req.headers[name] !== undefined) headers[name] = req.headers[name];
    headers['accept-encoding'] = 'identity';
    const pending = request({hostname: '127.0.0.1', port: Number(target.port), path: url.pathname + url.search, method: req.method, headers, agent: false, maxHeaderSize: 16384}, response => {
      const status = response.statusCode ?? 502;
      if (status >= 300 && status < 400 && status !== 304 || response.headers.location || response.headers['content-disposition']?.toLowerCase().includes('attachment') || response.headers['content-encoding'] && response.headers['content-encoding'] !== 'identity') { response.destroy(); deny(); return; }
      const chunks: Buffer[] = []; let bytes = 0;
      response.on('data', (chunk: Buffer) => {
        bytes += chunk.length; total += chunk.length;
        if (bytes > 16 * 1024 * 1024 || total > 64 * 1024 * 1024) { response.destroy(); pending.destroy(); res.destroy(); }
        else chunks.push(chunk);
      });
      response.on('error', () => { if (!res.headersSent) deny(502); else res.destroy(); });
      response.on('end', () => {
        if (closed || !allowed(req.method ?? '', url.href)) { deny(); return; }
        const output: IncomingHttpHeaders = {'Connection': 'close'};
        // HEAD and 304 describe the selected representation, not their empty
        // wire body. A fabricated zero length corrupts cache metadata. 204
        // forbids Content-Length altogether.
        if (status !== 204) {
          if (req.method === 'HEAD' || status === 304) {
            if (response.headers['content-length'] !== undefined) output['Content-Length'] = response.headers['content-length'];
          } else output['Content-Length'] = String(bytes);
        }
        for (const name of ['content-type', 'content-encoding', 'content-language', 'content-range', 'accept-ranges', 'etag', 'last-modified', 'cache-control', 'expires', 'vary', 'content-security-policy', 'x-content-type-options']) if (response.headers[name] !== undefined) output[name] = response.headers[name];
        res.writeHead(status, output); res.end(Buffer.concat(chunks));
      });
    });
    upstream.add(pending);
    const lifetime = setTimeout(() => pending.destroy(new Error('Browser response lifetime exceeded')), 10000);
    pending.setTimeout(10000, () => pending.destroy(new Error('Browser upstream timeout')));
    pending.on('error', () => { if (!res.headersSent) deny(502); else res.destroy(); });
    pending.on('close', () => { upstream.delete(pending); clearTimeout(lifetime); });
    res.on('close', () => pending.destroy());
    pending.end();
  });
  server.on('connection', socket => { sockets.add(socket); socket.on('error', () => {}); socket.on('close', () => sockets.delete(socket)); socket.setTimeout(10000, () => socket.destroy()); });
  const rejectTunnel = (_req: unknown, socket: Socket) => { socket.on('error', () => {}); socket.end('HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n'); };
  server.on('connect', rejectTunnel); server.on('upgrade', rejectTunnel);
  server.on('clientError', (_error, socket) => socket.destroy());
  await new Promise<void>((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', () => { server.off('error', reject); resolve(); }); });
  const address = server.address();
  if (!address || typeof address === 'string') throw new Error('Browser proxy has no literal loopback address');
  return {
    url: `http://127.0.0.1:${address.port}`,
    async close() { closed = true; for (const pending of upstream) pending.destroy(); for (const socket of sockets) socket.destroy(); await new Promise<void>(resolve => server.close(() => resolve())); },
  };
}
