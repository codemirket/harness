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

test('workspace HTTP silences implicit favicon but keeps missing files at 404', async () => {
  const server = http.createServer(handleWorkspaceRequest);
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  try {
    const base = `http://127.0.0.1:${server.address().port}`;
    for (const method of ['GET', 'HEAD']) {
      const icon = await fetch(`${base}/favicon.ico`, {method});
      assert.equal(icon.status, 204);
      assert.equal(await icon.text(), '');
    }
    assert.equal((await fetch(`${base}/missing.ico`)).status, 404);
    assert.equal((await fetch(`${base}/files/missing.png`)).status, 404);
  } finally { await new Promise(resolve => server.close(resolve)); }
});
