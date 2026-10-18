// One bounded native decoder per child; the parent owns cancellation and cleanup.
import sharp from 'sharp';
import fs from 'node:fs/promises';
sharp.cache(false);
sharp.concurrency(1);
const [input, output, json] = process.argv.slice(2);
const {width, height, mode, format, gravity, quality, enlarge} = JSON.parse(json);
const signature = Buffer.alloc(32);
const handle = await fs.open(input, 'r');
try { await handle.read(signature, 0, signature.length, 0); } finally { await handle.close(); }
const raster = signature.subarray(0, 8).equals(Buffer.from([137,80,78,71,13,10,26,10]))
  || (signature[0] === 255 && signature[1] === 216 && signature[2] === 255)
  || /^GIF8[79]a$/.test(signature.toString('ascii', 0, 6))
  || (signature.toString('ascii', 0, 4) === 'RIFF' && signature.toString('ascii', 8, 12) === 'WEBP')
  || (signature.toString('ascii', 4, 8) === 'ftyp' && /avif|avis/.test(signature.toString('ascii', 8, 32)));
if (!raster) throw Error('Only PNG, JPEG, GIF, WebP and AVIF raster sources are supported');
const image = sharp(input, {limitInputPixels: 17_000_000, sequentialRead: true, animated: false, failOn: 'warning'});
const metadata = await image.metadata();
if (!metadata.width || !metadata.height || metadata.width * metadata.height > 17_000_000) throw Error('Raster exceeds source pixel budget');
const fit = {fit: 'inside', fill: 'cover', force: 'fill'}[mode];
const position = {sm: sharp.strategy.attention, ce: 'centre', no: 'north', ea: 'east', so: 'south', we: 'west'}[gravity];
const prepared = image.rotate().resize({width, height, fit, position, withoutEnlargement: !enlarge});
if (format === 'jpeg') prepared.flatten({background: '#ffffff'});
await prepared.toFormat(format, {quality}).toFile(output);
if ((await fs.stat(output)).size > 64 * 1024 * 1024) throw Error('Raster exceeds output byte budget');
