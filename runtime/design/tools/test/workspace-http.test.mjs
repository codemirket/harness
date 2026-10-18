import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';

const workspace = await fs.mkdtemp(path.join(os.tmpdir(), 'mirket-http-test-'));
process.env.MIRKET_DESIGN_WORKSPACE = workspace;
const {handleWorkspaceRequest} = await import('../workspace-server.mjs');

test.after(async () => fs.rm(workspace, {recursive: true, force: true}));

async function withServer(hooks, run) {
  const server = http.createServer((request, response) => handleWorkspaceRequest(request, response, hooks));
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  try { await run(`http://127.0.0.1:${server.address().port}`); }
  finally { await new Promise(resolve => server.close(resolve)); }
}

test('workspace HTTP handles empty files and HEAD using the validated descriptor', async () => {
  await fs.writeFile(path.join(workspace, 'empty.txt.html'), '');
  await fs.writeFile(path.join(workspace, 'head.html'), 'visible');
  await withServer({}, async base => {
    const empty = await fetch(`${base}/files/empty.txt.html`);
    assert.equal(empty.status, 200);
    assert.equal(empty.headers.get('content-length'), '0');
    assert.equal(await empty.text(), '');
    const head = await fetch(`${base}/files/head.html`, {method: 'HEAD'});
    assert.equal(head.status, 200);
    assert.equal(head.headers.get('content-length'), '7');
    assert.equal(await head.text(), '');
  });
});

test('workspace HTTP silences implicit favicon but keeps missing files at 404', async () => {
  await withServer({}, async base => {
    for (const method of ['GET', 'HEAD']) {
      const icon = await fetch(`${base}/favicon.ico`, {method});
      assert.equal(icon.status, 204);
      assert.equal(await icon.text(), '');
    }
    assert.equal((await fetch(`${base}/missing.ico`)).status, 404);
    assert.equal((await fetch(`${base}/files/missing.png`)).status, 404);
  });
});

test('workspace HTTP rejects a final-file symlink replacement between validation and open', async () => {
  const file = path.join(workspace, 'final-swap.png');
  const secret = path.join(workspace, 'final-secret.png');
  await fs.writeFile(file, 'original');
  await fs.writeFile(secret, 'replacement secret');
  let swapped = false;
  await withServer({afterValidation: async ({relative}) => {
    if (relative !== 'final-swap.png' || swapped) return;
    swapped = true;
    await fs.rm(file);
    await fs.symlink(secret, file);
  }}, async base => {
    const response = await fetch(`${base}/files/final-swap.png`);
    assert.equal(response.status, 400);
    assert.doesNotMatch(await response.text(), /replacement secret/);
  });
  assert.equal(swapped, true);
});

test('workspace HTTP rejects an ancestor replacement between validation and descriptor traversal', async () => {
  const parent = path.join(workspace, 'ancestor-swap');
  const displaced = path.join(workspace, 'ancestor-displaced');
  await fs.mkdir(parent);
  await fs.writeFile(path.join(parent, 'asset.png'), 'original');
  let swapped = false;
  await withServer({afterValidation: async ({relative}) => {
    if (relative !== 'ancestor-swap/asset.png' || swapped) return;
    swapped = true;
    await fs.rename(parent, displaced);
    await fs.mkdir(parent);
    await fs.writeFile(path.join(parent, 'asset.png'), 'replacement secret');
  }}, async base => {
    const response = await fetch(`${base}/files/ancestor-swap/asset.png`);
    assert.equal(response.status, 400);
    assert.doesNotMatch(await response.text(), /replacement secret/);
  });
  assert.equal(swapped, true);
});

test('workspace HTTP streams only the fstat byte range when a file grows afterward', async () => {
  const file = path.join(workspace, 'growing.png');
  await fs.writeFile(file, 'bounded');
  let grew = false;
  await withServer({afterStat: async ({relative}) => {
    if (relative !== 'growing.png' || grew) return;
    grew = true;
    await fs.truncate(file, 64 * 1024 * 1024 + 1);
  }}, async base => {
    const response = await fetch(`${base}/files/growing.png`);
    assert.equal(response.status, 200);
    assert.equal(response.headers.get('content-length'), '7');
    assert.equal(response.headers.get('x-content-type-options'), 'nosniff');
    assert.equal(await response.text(), 'bounded');
  });
  assert.equal(grew, true);
  assert.equal((await fs.stat(file)).size, 64 * 1024 * 1024 + 1);
});
