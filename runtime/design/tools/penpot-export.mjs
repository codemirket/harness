import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import {workspace, workspacePath, readFile, publish, assertRelative, MAX_OUTPUT} from './paths.mjs';
import {optimizeSvgBytes, rasterBytes} from './operations.mjs';
import {checkRequest, combinedSignal} from './request.mjs';

// Native Penpot SVG exports keep font-face URLs on their own frontend. Embed
// those resources before the normal SVG hygiene boundary so the saved file is
// portable without permitting arbitrary external SVG resources.
function countOccurrences(source, needle) {
  let count = 0, index = 0;
  while ((index = source.indexOf(needle, index)) !== -1) { count++; index += needle.length; }
  return count;
}

export async function embedPenpotFonts(source) {
  const resources = new Map();
  const prepared = source.replace(/@font-face\s*\{[^{}]*\}/gi, block => block.replace(/url\(\s*(['"]?)([^'"\s)]+)\1\s*\)/gi, (match, quote, address) => {
    if (address.startsWith('data:')) {
      if (!/^data:font\/(?:woff2?|ttf|otf);base64,[a-zA-Z0-9+/=]+$/.test(address) || address.length > 5600000) throw Error('Penpot SVG embedded font is invalid or exceeds four MiB');
    } else {
      const url = new URL(address);
      const ownedPath = /^\/fonts\/[a-zA-Z0-9_-]+\.(?:woff2?|ttf|otf)$/.test(url.pathname) ||
        /^\/assets\/by-id\/[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}$/.test(url.pathname) ||
        /^\/internal\/gfonts\/font\/(?:[a-zA-Z0-9_-]+\/){2}[a-zA-Z0-9_-]+\.(?:woff2?|ttf|otf)$/.test(url.pathname);
      if (url.href !== address || url.origin !== 'http://penpot-frontend:8080' || url.username || url.password || url.search || url.hash || !ownedPath) throw Error('Penpot SVG font resource is outside its owned frontend');
    }
    if (!resources.has(address)) resources.set(address, {marker: Buffer.from(`mirket-font-${resources.size}`).toString('base64')});
    return `url(data:font/woff;base64,${resources.get(address).marker})`;
  }));
  // Reject scripts, DTDs, and every other external resource before any request.
  optimizeSvgBytes(prepared);
  if (resources.size > 8) throw Error('Penpot SVG exceeds eight font resources');
  let result = prepared, resultBytes = Buffer.byteLength(prepared), total = 0;
  for (const [address, {marker}] of resources) {
    checkRequest();
    let bytes;
    if (address.startsWith('data:')) bytes = Buffer.from(address.slice(address.indexOf(',') + 1), 'base64');
    else {
      const response = await fetch(address, {redirect: 'error', signal: combinedSignal(15000)});
      if (!response.ok || Number(response.headers.get('content-length')) > 4 * 1024 * 1024) {
        await response.body?.cancel();
        throw Error('Penpot SVG font response failed or exceeds four MiB');
      }
      const chunks = []; let size = 0;
      for await (const chunk of response.body) {
        size += chunk.length;
        if (size > 4 * 1024 * 1024 || total + size > 16 * 1024 * 1024) throw Error('Penpot SVG fonts exceed their byte bound');
        chunks.push(chunk);
      }
      bytes = Buffer.concat(chunks, size);
    }
    if (bytes.length > 4 * 1024 * 1024 || total + bytes.length > 16 * 1024 * 1024) throw Error('Penpot SVG fonts exceed their byte bound');
    const signature = bytes.subarray(0, 4).toString('hex');
    const mime = {'774f4646': 'font/woff', '774f4632': 'font/woff2', '4f54544f': 'font/otf', '00010000': 'font/ttf'}[signature];
    if (!mime || bytes.length < 12) throw Error('Penpot SVG font response is not a font');
    total += bytes.length;
    const placeholder = `data:font/woff;base64,${marker}`;
    const occurrences = countOccurrences(result, placeholder);
    const replacementBytes = Buffer.byteLength(`data:${mime};base64,`) + 4 * Math.ceil(bytes.length / 3);
    const prospectiveBytes = resultBytes + occurrences * (replacementBytes - Buffer.byteLength(placeholder));
    if (prospectiveBytes > MAX_OUTPUT) throw Error('Penpot SVG exceeds the output bound');
    const replacement = `data:${mime};base64,${bytes.toString('base64')}`;
    result = result.replaceAll(placeholder, replacement);
    resultBytes = prospectiveBytes;
  }
  if (resultBytes > MAX_OUTPUT) throw Error('Penpot SVG exceeds the output bound');
  return result;
}

function requestedWorkspacePath(filePath) {
  if (typeof filePath !== 'string' || !path.isAbsolute(filePath)) throw Error('Penpot filePath must be an absolute /workspace path');
  const relative = path.relative(workspace, filePath).split(path.sep).join('/');
  assertRelative(relative);
  return relative;
}

export async function exportPenpotShape(client, args, signal) {
  const relative = requestedWorkspacePath(args.filePath);
  await workspacePath(relative, {missing: true});
  try { await readFile(relative, 32 * 1024 * 1024); throw Error(`Penpot export destination already exists: ${relative}`); }
  catch (error) { if (!String(error.message).startsWith('Missing workspace path:')) throw error; }
  const format = args.format || 'png';
  if (!['png', 'svg'].includes(format) || !relative.endsWith(`.${format}`)) throw Error('Penpot export extension must match png or svg format');
  await workspacePath('.mirket-design/penpot', {kind: 'directory', missing: true, createParents: true});
  await fs.mkdir(path.join(workspace, '.mirket-design/penpot'), {recursive: true});
  const stageRelative = `.mirket-design/penpot/${crypto.randomUUID()}.${format}`;
  const stage = path.join(workspace, stageRelative);
  try {
    const response = await client.callTool({name: 'export_shape', arguments: {...args, filePath: stage}}, undefined, {signal});
    if (response.isError) return response;
    const raw = await readFile(stageRelative, 32 * 1024 * 1024);
    const bytes = format === 'svg' ? optimizeSvgBytes(await embedPenpotFonts(raw.toString('utf8'))) : raw;
    if (format === 'png' && !bytes.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]))) throw Error('Penpot export did not produce PNG bytes');
    await publish(relative, bytes);
    return {...response, content: (response.content || []).map(item => item.type === 'text' ? {...item, text: item.text.replaceAll(stage, args.filePath)} : item)};
  } finally { await fs.rm(stage, {force: true}).catch(() => {}); }
}

export async function importPenpotImage(client, args, signal) {
  const relative = requestedWorkspacePath(args.filePath);
  if (!/\.(?:png|jpe?g|gif|webp)$/i.test(relative)) throw Error('Penpot import_image supports PNG, JPEG, GIF or WebP raster inputs');
  await workspacePath(relative);
  const bytes = await rasterBytes({source: relative, width: 4096, height: 4096, mode: 'fit', format: 'png'});
  await workspacePath('.mirket-design/penpot', {kind: 'directory', missing: true, createParents: true});
  await fs.mkdir(path.join(workspace, '.mirket-design/penpot'), {recursive: true});
  const stageRelative = `.mirket-design/penpot/${crypto.randomUUID()}.png`;
  const stage = path.join(workspace, stageRelative);
  await publish(stageRelative, bytes);
  try { return await client.callTool({name: 'import_image', arguments: {...args, filePath: stage}}, undefined, {signal}); }
  finally { await fs.rm(stage, {force: true}).catch(() => {}); }
}
