import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import net from 'node:net';
import {createBrowserProxy} from '../dist/extension/browser-proxy.js';

test('real proxy forwards only the granted HTTP origin and GET/HEAD, refusing redirects and tunnels', async () => {
  const received = [];
  const fixture = http.createServer((req, res) => {
    received.push({method: req.method, url: req.url});
    if (req.url === '/redirect') return res.writeHead(302, {Location: '/ok'}).end();
    if (req.url === '/attachment') return res.writeHead(200, {'Content-Disposition': 'attachment; filename=x'}).end('download');
    if (req.url === '/compressed') return res.writeHead(200, {'Content-Encoding': 'gzip'}).end('fake');
    if (req.url === '/cached' && req.headers['if-none-match'] === 'fixture-v1') return res.writeHead(304, {ETag: 'fixture-v1'}).end();
    res.writeHead(200, {'Content-Type': 'text/plain', 'Set-Cookie': 'secret=x'}).end('forwarded');
  });
  await new Promise(resolve => fixture.listen(0, '127.0.0.1', resolve));
  const origin = `http://127.0.0.1:${fixture.address().port}`;
  let permitted = true;
  const proxy = await createBrowserProxy(origin, (method, target) => permitted && ['GET', 'HEAD'].includes(method) && new URL(target).origin === origin && !new URL(target).username);
  const port = Number(new URL(proxy.url).port);
  const call = (path, method = 'GET', headers = {}) => new Promise((resolve, reject) => {
    const req = http.request({hostname: '127.0.0.1', port, path, method, headers, agent: false}, res => { const parts = []; res.on('data', chunk => parts.push(chunk)); res.on('end', () => resolve({status: res.statusCode, text: Buffer.concat(parts).toString(), headers: res.headers})); });
    req.on('error', reject); req.end();
  });
  const raw = value => new Promise((resolve, reject) => { const socket = net.connect(port, '127.0.0.1'); let result = ''; socket.on('connect', () => socket.write(value)); socket.on('data', bytes => { result += bytes; }); socket.on('end', () => resolve(result)); socket.on('error', reject); });
  try {
    const accepted = await call(origin + '/ok?x=1'); assert.equal(accepted.status, 200); assert.equal(accepted.text, 'forwarded'); assert.equal(accepted.headers['set-cookie'], undefined);
    assert.equal((await call(origin + '/ok', 'HEAD')).status, 200);
    const revalidated = await call(origin + '/cached', 'GET', {'If-None-Match': 'fixture-v1'});
    assert.equal(revalidated.status, 304); assert.equal(revalidated.headers.etag, 'fixture-v1'); assert.equal(revalidated.text, '');
    const before = received.length;
    for (const [url, method, headers] of [[origin + '/post', 'POST', {}], ['http://127.0.0.1:1/escape', 'GET', {}], ['http://example.org/', 'GET', {}], [origin + '/auth', 'GET', {Authorization: 'secret'}], [origin + '/cookie', 'GET', {Cookie: 'secret'}], [origin + '/', 'GET', {'Content-Length': '1'}]]) assert.equal((await call(url, method, headers)).status, 403);
    assert.equal(received.length, before);
    assert.match(await raw('CONNECT 127.0.0.1:1 HTTP/1.1\r\nHost: 127.0.0.1:1\r\n\r\n'), /403/);
    assert.match(await raw(`GET ${origin}/ HTTP/1.1\r\nHost: localhost\r\nConnection: Upgrade\r\nUpgrade: websocket\r\n\r\n`), /403/);
    for (const path of ['/redirect', '/attachment', '/compressed']) assert.equal((await call(origin + path)).status, 403);
    permitted = false; assert.equal((await call(origin + '/revoked')).status, 403);
    assert.equal(received.some(request => request.url === '/revoked'), false);
  } finally { await proxy.close(); fixture.closeAllConnections(); await new Promise(resolve => fixture.close(resolve)); }
});

test('proxy caps response bytes before exposing a response to the browser', async () => {
  const fixture = http.createServer((_req, res) => res.end(Buffer.alloc(16 * 1024 * 1024 + 1, 'x')));
  await new Promise(resolve => fixture.listen(0, '127.0.0.1', resolve));
  const origin = `http://127.0.0.1:${fixture.address().port}`, proxy = await createBrowserProxy(origin, () => true);
  try {
    await assert.rejects(new Promise((resolve, reject) => { const req = http.get({hostname: '127.0.0.1', port: Number(new URL(proxy.url).port), path: origin + '/', agent: false}, res => { res.resume(); res.on('end', () => resolve(res.statusCode)); }); req.on('error', reject); }));
  } finally { await proxy.close(); fixture.closeAllConnections(); await new Promise(resolve => fixture.close(resolve)); }
});
