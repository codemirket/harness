import test from 'node:test';
import assert from 'node:assert/strict';
import {embedPenpotFonts} from '../penpot-export.mjs';
import {optimizeSvgBytes} from '../operations.mjs';

const svg = address => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 50"><style>@font-face{font-family:probe;src:url(${address}) format('woff')}</style><text x="0" y="30" font-family="probe">Portable text</text></svg>`;
const font = Buffer.concat([Buffer.from('wOFF'), Buffer.alloc(40)]);

test('native Penpot font routes become self-contained before SVG optimization', async () => {
  const original = global.fetch;
  const requested = [];
  global.fetch = async (url, options) => {
    requested.push(url);
    assert.equal(options.redirect, 'error');
    assert.ok(options.signal instanceof AbortSignal);
    return new Response(font);
  };
  try {
    for (const route of ['fonts/sourcesanspro-regular.woff', 'assets/by-id/12345678-1234-1234-1234-123456789abc', 'internal/gfonts/font/roboto/v30/abc_123.woff2']) {
      const result = optimizeSvgBytes(await embedPenpotFonts(svg(`http://penpot-frontend:8080/${route}`))).toString();
      assert.match(result, /data:font\/woff;base64,d09GRg/);
      assert.doesNotMatch(result, /http:\/\/penpot-frontend/);
      assert.match(result, /Portable text/);
    }
    assert.equal(requested.length, 3);
  } finally { global.fetch = original; }
});

test('font embedding refuses external, normalized and malicious SVG before requests', async () => {
  const original = global.fetch;
  let requests = 0;
  global.fetch = async () => { requests++; throw Error('Unexpected request'); };
  try {
    for (const address of ['https://example.com/font.woff', 'http://penpot-frontend:8080/fonts/../fonts/probe.woff', '/fonts/probe.woff', 'file:///tmp/font.woff', 'http://penpot-frontend:8080/assets/by-id/not-a-uuid']) {
      await assert.rejects(embedPenpotFonts(svg(address)));
    }
    await assert.rejects(embedPenpotFonts(svg('http://penpot-frontend:8080/fonts/probe.woff').replace('</svg>', '<image href="/private.png"/></svg>')), /external resources/);
    assert.equal(requests, 0);
  } finally { global.fetch = original; }
});

test('font response magic and streaming size are checked without trusting headers', async () => {
  const original = global.fetch;
  try {
    global.fetch = async () => new Response('this is not a font');
    await assert.rejects(embedPenpotFonts(svg('http://penpot-frontend:8080/fonts/probe.woff')), /not a font/);
    global.fetch = async () => new Response(new ReadableStream({start(controller) {
      controller.enqueue(Buffer.alloc(4 * 1024 * 1024));
      controller.enqueue(Buffer.alloc(1));
      controller.close();
    }}));
    await assert.rejects(embedPenpotFonts(svg('http://penpot-frontend:8080/fonts/probe.woff')), /byte bound/);
    await assert.rejects(embedPenpotFonts(svg('data:font/woff;base64,bm90LWEtZm9udA==')), /not a font/);
    const embedded = await embedPenpotFonts(svg(`data:font/woff;base64,${font.toString('base64')}`));
    assert.match(embedded, /data:font\/woff;base64,d09GRg/);
  } finally { global.fetch = original; }
});

test('SVG hygiene rejects relative and escaped CSS resources while retaining fragment fills', () => {
  for (const address of ['/fonts/probe.woff', '../probe.woff', 'https://example.com/font.woff']) assert.throws(() => optimizeSvgBytes(svg(address)), /external style resource/);
  assert.throws(() => optimizeSvgBytes(svg('data:font/woff;base64,d09GRg==').replace('src:url', 'src:u\\72l')), /escaped CSS/);
  assert.match(optimizeSvgBytes('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"><defs><linearGradient id="paint"><stop stop-color="red"/></linearGradient></defs><rect width="20" height="20" fill="url(#paint)"/></svg>').toString(), /url\(#/);
});
