// Preserve production bytes; oversized rasters get lossless, identity-bound
// tiles plus an overview. Each image fits the worker's 4 MiB image boundary.
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {readFile, publish, temporaryDirectory, regularOutput, run, sha256, workspacePath, MAX_OUTPUT} from './paths.mjs';

export const IMAGE_BYTES = 4 * 1024 * 1024;
export async function prepareInspection(result) {
  const inspection = [];
  const candidates = [result.artifact, result.preview, ...(result.pages || [])];
  for (const source of candidates) {
    if (!source?.path || !/\.(png|jpe?g|webp|avif)$/i.test(source.path)) continue;
    const bytes = await readFile(source.path, MAX_OUTPUT);
    if (sha256(bytes) !== source.sha256) throw Error('Inspection source changed after rendering');
    if (bytes.length <= IMAGE_BYTES && !/\.avif$/i.test(source.path)) continue;
    const temp = await temporaryDirectory('inspection');
    try {
      const input = path.join(temp, 'input'), manifest = path.join(temp, 'manifest.json');
      await fs.writeFile(input, bytes, {mode:0o600});
      await run(process.execPath, ['--max-old-space-size=192', fileURLToPath(new URL('./raster-worker.mjs', import.meta.url)), input, manifest, JSON.stringify({operation:'inspection'})], {timeoutMs:90000});
      const plan = JSON.parse((await regularOutput(manifest, 32 * 1024)).toString('utf8'));
      if (!Array.isArray(plan.tiles) || !plan.tiles.length || plan.tiles.length > 64) throw Error('Invalid inspection tile plan');
      const prefix = `.mirket-design/inspection/${crypto.randomUUID()}`;
      await fs.mkdir(await workspacePath('.mirket-design/inspection', {kind:'directory', missing:true, createParents:true}), {recursive:true});
      const save = async item => {
        if (!/^(overview|tile-\d+)\.png$/.test(item.file)) throw Error('Invalid inspection output');
        const saved = await publish(`${prefix}-${item.file}`, await regularOutput(path.join(temp, item.file), IMAGE_BYTES));
        const {file, ...region} = item;
        return {...saved, ...region};
      };
      const overview = await save(plan.overview);
      const tiles = [];
      for (const tile of plan.tiles) tiles.push(await save(tile));
      // Retain the original and verify it again after producing the evidence.
      if (sha256(await readFile(source.path, MAX_OUTPUT)) !== source.sha256) throw Error('Inspection source changed during tiling');
      inspection.push({source, width:plan.width, height:plan.height, overview, tiles,
        rule:'Inspect the overview for composition and every native-resolution tile for detail. Original bytes remain unchanged; tile coordinates are decoded pixel coordinates.'});
    } finally { await fs.rm(temp, {recursive:true, force:true}); }
  }
  return inspection.length ? {...result, inspection} : result;
}
