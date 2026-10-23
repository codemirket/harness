// Native media fixtures exercise the pinned producers and inspect their saved
// bytes. Receipts and compiler exit codes alone do not establish semantic
// output, especially for tagged PDF.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import zlib from 'node:zlib';
import {SaxesParser} from 'saxes';

const workspace = await fs.mkdtemp(path.join(os.tmpdir(), 'mirket-native-media-'));
process.env.MIRKET_DESIGN_WORKSPACE = workspace;
const {d2Render, typstCompile} = await import('../operations.mjs');

test.after(async () => fs.rm(workspace, {recursive: true, force: true}));

async function requireBinary(t, binary, label) {
  try {
    await fs.access(binary);
  } catch (error) {
    if (process.env.MIRKET_ISOLATED_TEST === '1') throw error;
    t.skip(`${label} requires the explicit design runtime`);
    return false;
  }
  return true;
}

function inspectSvg(bytes) {
  assert.ok(bytes.length > 100, 'D2 SVG is unexpectedly empty');
  const source = bytes.toString('utf8');
  assert.match(source, /^\s*(?:<\?xml[^>]*>\s*)?<svg\b/i);
  assert.match(source, /(?:viewBox|width|height)=['"][^'"]+['"]/i);
  const parser = new SaxesParser({xmlns: true});
  let root = false;
  let text = '';
  let groups = 0;
  parser.on('opentag', tag => {
    if (tag.local === 'svg') root = true;
    if (tag.local === 'g') groups++;
  });
  parser.on('text', value => { text += value; });
  parser.write(source).close();
  assert.equal(root, true, 'D2 output has no SVG root');
  assert.ok(groups >= 2, 'D2 output has no retained diagram groups');
  return {source, text};
}

function pdfUnicodeMaps(text) {
  const map = new Map();
  const unicode = hex => {
    const bytes = Buffer.from(hex, 'hex');
    if (bytes.length % 2) return '';
    return Buffer.from(bytes).swap16().toString('utf16le');
  };
  for (const block of text.matchAll(/beginbfchar([\s\S]*?)endbfchar/g)) {
    for (const pair of block[1].matchAll(/<([0-9a-fA-F]+)>\s*<([0-9a-fA-F]+)>/g)) map.set(pair[1].toLowerCase(), unicode(pair[2]));
  }
  for (const block of text.matchAll(/beginbfrange([\s\S]*?)endbfrange/g)) {
    for (const range of block[1].matchAll(/<([0-9a-fA-F]+)>\s*<([0-9a-fA-F]+)>\s*<([0-9a-fA-F]+)>/g)) {
      const start = Number.parseInt(range[1], 16);
      const finish = Number.parseInt(range[2], 16);
      const base = Number.parseInt(range[3], 16);
      for (let code = start; code <= finish && code - start < 4096; code++) {
        const key = code.toString(16).padStart(range[1].length, '0');
        const value = (base + code - start).toString(16).padStart(range[3].length, '0');
        map.set(key.toLowerCase(), unicode(value));
      }
    }
  }
  return map;
}

function pdfTextCandidates(text) {
  const maps = [...text.matchAll(/begincmap([\s\S]*?)endcmap/g)].map(m => pdfUnicodeMaps(m[1]));
  const values = [];
  // Decode text-showing runs with each retained font CMap independently. Font
  // subsets reuse glyph IDs, so merging their CMaps corrupts the extracted text.
  for (const run of text.matchAll(/\bBT\b([\s\S]*?)\bET\b/g)) {
    const literals = [...run[1].matchAll(/\((?:\\[\s\S]|[^()\\])*\)/g)].map(m =>
      m[0].slice(1,-1).replace(/\\([0-7]{1,3}|\r\n|[\s\S])/g, (_, code) =>
        /^[0-7]/.test(code) ? String.fromCharCode(Number.parseInt(code,8)) :
        ({n:'\n',r:'\r',t:'\t',b:'\b',f:'\f','\n':'','\r':'','\r\n':''}[code] ?? code)));
    for (const map of maps) {
      let decoded = '';
      for (const literal of literals) {
        const hex = Buffer.from(literal, 'latin1').toString('hex');
        const width = [...map.keys()].every(key => key.length === 4) ? 4 : 2;
        for (let i=0;i<hex.length;i+=width) decoded += map.get(hex.slice(i,i+width)) ?? '\uFFFD';
      }
      values.push(decoded);
    }
  }
  return values.join('\n');
}

function inspectPdf(bytes, markers) {
  assert.ok(bytes.length > 256, 'PDF is unexpectedly small');
  assert.equal(bytes.subarray(0, 5).toString(), '%PDF-', 'missing PDF header');
  assert.match(bytes.toString('latin1').slice(-1024), /%%EOF\s*$/);
  const raw = bytes.toString('latin1');
  const views = [raw];
  let streamCount = 0;
  // Match each stream's own object dictionary and exact byte length. Looking
  // backwards across objects can misclassify raw font/image streams as Flate.
  const streams = /\d+\s+\d+\s+obj\s*(<<(?:(?!endobj)[\s\S])*?>>)\s*stream(?:\r\n|\n|\r)/g;
  let match;
  while ((match = streams.exec(raw)) !== null && streamCount < 256) {
    const dictionary = match[1];
    const length = /\/Length\s+(\d+)\s*(?!\d|\s+\d+\s+R)/.exec(dictionary);
    assert.ok(length, 'fixture PDF stream requires a direct length');
    const start = streams.lastIndex;
    const finish = start + Number(length[1]);
    assert.ok(finish <= bytes.length, 'PDF stream exceeds saved bytes');
    if (/\/FlateDecode\b/.test(dictionary)) {
      const inflated = zlib.inflateSync(bytes.subarray(start, finish), {maxOutputLength: 8 * 1024 * 1024});
      views.push(inflated.toString('latin1'));
    }
    streams.lastIndex = finish;
    streamCount++;
  }
  const inspected = views.join('\n');
  const readable = pdfTextCandidates(inspected);
  const hasLanguage = /\/Lang\s*(?:\([^)]{1,32}\)|\/[A-Za-z-]{2,16})/.test(inspected);
  const hasHeading = /\/S\s*\/H[1-6](?:\b|\s)/.test(inspected);
  const hasAlt = /\/Alt\s*(?:\([^)]{1,512}\)|<[0-9a-fA-F]{2,1024}>)/.test(inspected);
  const markerPositions = markers.map(marker => readable.indexOf(marker));
  const readingOrder = markerPositions.every(position => position >= 0) && markerPositions.every((position, index) => index === 0 || position > markerPositions[index - 1]);
  return {hasLanguage, hasHeading, hasAlt, readingOrder, markerPositions, streamCount, readable};
}

async function writeTypstFixture() {
  await fs.mkdir(path.join(workspace, 'design'), {recursive: true});
  await fs.writeFile(path.join(workspace, 'design/tokens.json'), JSON.stringify({color: {accent: {$type: 'color', $value: '#135724'}}}));
  await fs.mkdir(path.join(workspace, 'good'), {recursive: true});
  await fs.mkdir(path.join(workspace, 'bad'), {recursive: true});
  await fs.writeFile(path.join(workspace, 'good/fixture.svg'), '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><rect width="24" height="24" fill="#135724"/></svg>');
  await fs.copyFile(path.join(workspace, 'good/fixture.svg'), path.join(workspace, 'bad/fixture.svg'));
  await fs.writeFile(path.join(workspace, 'good/document.typ'), `#set document(title: [Native media accessibility fixture])
#set text(lang: "en")
#set page(width: 240pt, height: 220pt, margin: 18pt)
#text(size: 14pt)[READING-ONE]

= Accessible heading

This paragraph follows the heading and precedes the figure.

#figure(image("fixture.svg", alt: "A solid green square"), caption: [A green square])

#text(size: 14pt)[READING-TWO]
`);
  // This document is intentionally authored with a non-semantic bold span and
  // an image without alt text. PDF/UA must reject it; the test proves the
  // negative fixture is meaningful rather than merely checking a receipt.
  await fs.writeFile(path.join(workspace, 'bad/document.typ'), `#set document(title: [Bad accessibility fixture])
#set text(lang: "en")
#set page(width: 240pt, height: 180pt, margin: 18pt)
#text(weight: "bold")[Stylistic text presented as a heading]
#image("fixture.svg")
`);
}

test('real D2/TALA output is a valid labeled SVG', async t => {
  if (!await requireBinary(t, '/usr/local/bin/d2', 'D2/TALA')) return;
  await fs.writeFile(path.join(workspace, 'topology.d2'), `direction: right
client: Client
service: Service
database: Database
client -> service: calls
service -> database: stores
`);
  const result = await d2Render({source: 'topology.d2', output: 'topology.svg', format: 'svg'});
  assert.equal(result.engine, 'd2');
  assert.equal(result.layout, 'tala');
  assert.equal(result.version, '0.9.0');
  const artifact = await fs.readFile(path.join(workspace, result.artifact.path));
  const {text} = inspectSvg(artifact);
  assert.match(text, /Client/);
  assert.match(text, /Service/);
  assert.match(text, /Database/);
  assert.match(text, /calls/);
});

test('PDF/UA fixture contains language, heading, alt text and source reading order', async t => {
  if (!await requireBinary(t, '/usr/local/bin/typst', 'Typst')) return;
  await writeTypstFixture();
  const result = await typstCompile({source: 'good/document.typ', output: 'good.pdf', pdf_standard: 'ua-1'});
  assert.equal(result.pdf_standard, 'ua-1');
  assert.ok(result.pages.length >= 1);
  const pdf = await fs.readFile(path.join(workspace, result.artifact.path));
  const semantics = inspectPdf(pdf, ['READING-ONE', 'Accessible heading', 'READING-TWO']);
  assert.equal(semantics.hasLanguage, true, 'PDF/UA output lacks a document language');
  assert.equal(semantics.hasHeading, true, 'PDF/UA output lacks a tagged heading');
  assert.equal(semantics.hasAlt, true, 'PDF/UA output lacks alternative text');
  assert.equal(semantics.readingOrder, true, `content markers are not in source order: ${semantics.markerPositions}`);
  assert.equal(inspectPdf(pdf, ['READING-TWO', 'Accessible heading', 'READING-ONE']).readingOrder, false, 'reversed source order must fail');
  assert.ok(semantics.streamCount > 0, 'PDF inspection did not inspect any content streams');
});

test('PDF/UA negative fixture rejects missing alternative text', async t => {
  if (!await requireBinary(t, '/usr/local/bin/typst', 'Typst')) return;
  await writeTypstFixture();
  await assert.rejects(
    typstCompile({source: 'bad/document.typ', output: 'bad.pdf', pdf_standard: 'ua-1'}),
    /alt|alternative|accessib|PDF\/UA/i,
  );
  await assert.rejects(fs.access(path.join(workspace, 'bad.pdf')));
});

