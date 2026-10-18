import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';

const workspace = await fs.mkdtemp(path.join(os.tmpdir(), 'mirket-operations-bounds-'));
process.env.MIRKET_DESIGN_WORKSPACE = workspace;

const {
  copyBoundedProjectTree,
  readBoundedProjectAsset,
  withBrowserCaptureSession,
} = await import('../operations.mjs');
const {withRequest} = await import('../request.mjs');

test.after(async () => fs.rm(workspace, {recursive: true, force: true}));

test('Playwright cancellation before connect or before the operation microtask prevents work', async () => {
  const before = new AbortController();
  before.abort(Error('already cancelled'));
  await assert.rejects(withBrowserCaptureSession(async () => assert.fail('must not connect'), '', {}, async () => {}, {signal: before.signal}), /already cancelled/);
  const controller = new AbortController();
  let operations = 0, closes = 0;
  await assert.rejects(withBrowserCaptureSession(async () => {
    queueMicrotask(() => queueMicrotask(() => controller.abort(Error('cancelled before operation'))));
    return {close: async () => { closes++; }};
  }, '', {}, async () => { operations++; }, {signal: controller.signal}), /cancelled before operation/);
  assert.equal(operations, 0);
  assert.equal(closes, 1);
});

async function temporaryPair(name) {
  const root = await fs.mkdtemp(path.join(workspace, `${name}-`));
  const origin = path.join(root, 'origin');
  const stage = path.join(root, 'stage');
  await fs.mkdir(origin);
  await fs.mkdir(stage);
  return {root, origin, stage};
}

test('Playwright cancellation arriving during connect closes the eventual browser before context work', async () => {
  const controller = new AbortController();
  let releaseConnect;
  const connected = new Promise(resolve => { releaseConnect = resolve; });
  let closes = 0, operations = 0;
  const browser = {close: async () => { closes++; }};
  const capture = withBrowserCaptureSession(
    async () => { await connected; return browser; },
    'owned-child',
    {timeout: 15000},
    async () => { operations++; },
    {signal: controller.signal, timeoutMs: 1000},
  );
  const rejected = assert.rejects(capture, /cancelled while connecting/);
  controller.abort(Error('cancelled while connecting'));
  releaseConnect();
  await rejected;
  assert.equal(operations, 0);
  assert.ok(closes >= 1, 'the remotely connected browser was closed');
});

test('Playwright capture operation deadline rejects stalled protocol work and closes the browser', async () => {
  let closes = 0;
  const browser = {close: async () => { closes++; }};
  await assert.rejects(withBrowserCaptureSession(
    async () => browser,
    'owned-child',
    {timeout: 15000},
    async () => new Promise(() => {}),
    {timeoutMs: 20},
  ), /capture operation timed out/);
  assert.ok(closes >= 1, 'deadline closed the remote browser');
});

test('bounded traversal counts empty directories instead of allowing file-limit bypass', async () => {
  const {root, origin, stage} = await temporaryPair('empty-directories');
  try {
    for (let index = 0; index < 129; index++) await fs.mkdir(path.join(origin, `empty-${String(index).padStart(3, '0')}`));
    await assert.rejects(copyBoundedProjectTree(origin, stage, {
      label: 'Test project',
      onFile: async () => assert.fail('empty directories must not invoke the file copier'),
    }), /exceeds 128 filesystem entries/);
  } finally {
    await fs.rm(root, {recursive: true, force: true});
  }
});

test('bounded traversal rejects excessive depth before creating or descending into it', async () => {
  const {root, origin, stage} = await temporaryPair('directory-depth');
  try {
    let current = origin;
    for (let depth = 1; depth <= 33; depth++) {
      current = path.join(current, `d${depth}`);
      await fs.mkdir(current);
    }
    await assert.rejects(copyBoundedProjectTree(origin, stage, {
      label: 'Test project',
      onFile: async () => {},
    }), /exceeds directory depth 32/);
    let staged = stage;
    for (let depth = 1; depth <= 32; depth++) staged = path.join(staged, `d${depth}`);
    await fs.stat(staged);
    await assert.rejects(fs.stat(path.join(staged, 'd33')), {code: 'ENOENT'});
  } finally {
    await fs.rm(root, {recursive: true, force: true});
  }
});

test('bounded project asset reads observe cancellation between descriptor reads', async () => {
  const file = path.join(workspace, 'cancelled-asset.bin');
  await fs.writeFile(file, Buffer.alloc(128 * 1024, 7));
  const controller = new AbortController();
  const originalOpen = fs.open;
  fs.open = async (...args) => {
    const handle = await originalOpen(...args);
    if (args[0] === file) {
      const originalRead = handle.read.bind(handle);
      let reads = 0;
      handle.read = async (...parameters) => {
        const result = await originalRead(...parameters);
        if (++reads === 1) controller.abort(Error('asset read cancelled'));
        return result;
      };
    }
    return handle;
  };
  try {
    await assert.rejects(
      withRequest(controller.signal, () => readBoundedProjectAsset(file, 256 * 1024, 'Test project')),
      /asset read cancelled/,
    );
  } finally {
    fs.open = originalOpen;
  }
});

test('nonblocking no-follow project asset reader promptly rejects a FIFO replacement', {skip: process.platform !== 'linux'}, async t => {
  const fifo = path.join(workspace, 'replacement-fifo');
  const made = spawnSync('mkfifo', [fifo], {encoding: 'utf8', timeout: 1000});
  if (made.error?.code === 'ENOENT') { t.skip('mkfifo is unavailable'); return; }
  assert.equal(made.status, 0, made.stderr);
  // A regression must fail without leaving a libuv thread blocked on open().
  const script = `import assert from 'node:assert/strict';
    import {readBoundedProjectAsset} from ${JSON.stringify(new URL('../operations.mjs', import.meta.url).href)};
    await assert.rejects(readBoundedProjectAsset(${JSON.stringify(fifo)}, 1024, 'Test project'), /not a regular file/);`;
  const result = spawnSync(process.execPath, ['--input-type=module', '-e', script], {encoding: 'utf8', timeout: 5000, killSignal: 'SIGKILL'});
  assert.equal(result.status, 0, result.error?.message || result.stderr);
});

test('nonblocking descriptor reads preserve stable normal project assets', async () => {
  const file = path.join(workspace, 'stable-asset.txt');
  const expected = Buffer.from('stable normal asset\n'.repeat(4096));
  await fs.writeFile(file, expected);
  for (let attempt = 0; attempt < 3; attempt++) {
    const actual = await readBoundedProjectAsset(file, expected.length, 'Test project');
    assert.deepEqual(actual, expected);
  }
});
