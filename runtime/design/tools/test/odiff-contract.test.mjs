import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {Resvg} from '@resvg/resvg-js';

const workspace = await fs.mkdtemp(path.join(os.tmpdir(), 'mirket-odiff-contract-'));
process.env.MIRKET_DESIGN_WORKSPACE = workspace;
const {odiffCompare} = await import('../operations.mjs');
test.after(async () => fs.rm(workspace, {recursive: true, force: true}));

test('Odiff publishes only actual diagnostics and reports dimension mismatches', async () => {
  for (const [name, color, width] of [['baseline', 'white', 16], ['changed', 'black', 16], ['resized', 'white', 8]]) {
    const bytes = new Resvg(`<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="16"><rect width="100%" height="100%" fill="${color}"/></svg>`).render().asPng();
    await fs.writeFile(path.join(workspace, `${name}.png`), bytes);
  }
  const same = await odiffCompare({baseline: 'baseline.png', candidate: 'baseline.png'});
  assert.equal(same.match, true);
  assert.equal(same.diff, null);
  await assert.rejects(fs.stat(path.join(workspace, '.mirket-design/qa')), {code: 'ENOENT'});
  const dimensions = await odiffCompare({baseline: 'baseline.png', candidate: 'resized.png'});
  assert.equal(dimensions.match, false);
  const changed = await odiffCompare({baseline: 'baseline.png', candidate: 'changed.png'});
  assert.equal(changed.match, false);
  assert.equal(typeof changed.diff?.sha256, 'string');
  assert.ok((await fs.stat(path.join(workspace, changed.diff.path))).size > 0);
});
