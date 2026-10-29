import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {
  MIRKET_TOOL_CLASSES, classifyMirketTool, authorized, grant, needsAuthorization,
} from '../dist/extension/authorization.js';

const expectedClasses = {
  mcp__mirket__runtime_status: 'O0',
  mcp__mirket__code_inspect: 'O1',
  mcp__mirket__code_navigate: 'O1',
  mcp__mirket__diagram_render_prepare: 'P1',
  mcp__mirket__diagram_render_start: 'F1',
  mcp__mirket__design_prepare: 'P1',
  mcp__mirket__design_start: 'F1',
  mcp__mirket__video_prepare: 'P1',
  mcp__mirket__video_start: 'F1',
  mcp__mirket__video_status: 'O2',
  mcp__mirket__video_cancel: 'K1',
  mcp__mirket__video_result: 'O2',
  mcp__mirket__job_status: 'O2',
  mcp__mirket__job_cancel: 'K1',
  mcp__mirket__memory_prepare: 'P2',
  mcp__mirket__memory_commit: 'M1',
  mcp__mirket__memory_index_refresh: 'I1',
  mcp__mirket__memory_status: 'O3',
  mcp__mirket__memory_forget_prepare: 'P3',
  mcp__mirket__memory_forget_commit: 'X1',
  mcp__mirket__memory_forget_reconcile: 'X2',
  mcp__mirket__memory_search: 'O4',
  mcp__mirket__memory_list: 'O3',
  mcp__mirket__memory_read: 'O3',
  mcp__mirket__memory_history: 'O3',
};

test('the frozen MCP taxonomy has exactly 25 classified names', () => {
  assert.equal(Object.keys(expectedClasses).length, 25);
  assert.deepEqual(MIRKET_TOOL_CLASSES, expectedClasses);
  for (const [tool, effectClass] of Object.entries(expectedClasses)) {
    assert.equal(classifyMirketTool(tool), effectClass);
  }
});

test('known routes retain the existing project and shared extra-grant selector', () => {
  const project = {scope: {kind: 'project', project_root: '/project'}};
  const shared = {scope: {kind: 'shared_preferences'}};
  for (const tool of Object.keys(expectedClasses)) {
    const forget = tool === 'mcp__mirket__memory_forget_commit' || tool === 'mcp__mirket__memory_forget_reconcile';
    assert.equal(needsAuthorization(tool, project), forget, `${tool} project`);
    assert.equal(needsAuthorization(tool, shared), tool.startsWith('mcp__mirket__memory_'), `${tool} shared`);
  }
});

test('unknown Mirket names cannot be classified, selected or granted', () => {
  const tool = 'mcp__mirket__future_effect';
  const args = {scope: {kind: 'project', project_root: '/project'}};
  const home = fs.mkdtempSync('/private/tmp/mirket-authorization-');
  fs.chmodSync(home, 0o700);
  try {
    assert.throws(() => classifyMirketTool(tool), /Unclassified Mirket tool/);
    assert.throws(() => needsAuthorization(tool, args), /Unclassified Mirket tool/);
    assert.throws(() => grant({home, cwd: '/project', binding: {releaseId: 'release-one'}}, tool, args, 1000), /Unclassified Mirket tool/);
    assert.deepEqual(fs.readdirSync(home), []);
  } finally { fs.rmSync(home, {recursive: true, force: true}); }
});

test('exact grants survive reload, are reusable, expire and reject changed arguments, project or release', () => {
  const home = fs.mkdtempSync('/private/tmp/mirket-authorization-'); fs.chmodSync(home, 0o700);
  const runtime = {home, cwd: '/project', binding: {releaseId: 'release-one'}};
  const tool = 'mcp__mirket__memory_forget_commit';
  const args = {scope: {kind: 'project', project_root: '/project'}, operation_id: 'intent-one'};
  try {
    assert.equal(authorized(runtime, tool, args, 1000), false);
    grant(runtime, tool, args, 1000);
    assert.equal(authorized({...runtime}, tool, {operation_id: 'intent-one', scope: args.scope}, 1001), true);
    assert.equal(authorized(runtime, tool, args, 1002), true);
    assert.equal(authorized(runtime, tool, {...args, operation_id: 'intent-two'}, 1001), false);
    assert.equal(authorized({...runtime, cwd: '/other'}, tool, args, 1001), false);
    assert.equal(authorized({...runtime, binding: {releaseId: 'release-two'}}, tool, args, 1001), false);
    assert.equal(authorized(runtime, tool, args, 301000), false);
    const saved = fs.readFileSync(home+'/authorizations.json', 'utf8');
    fs.mkdirSync(home+'/authorizations.lock');
    assert.throws(() => grant(runtime, tool, args, 1002));
    assert.equal(fs.readFileSync(home+'/authorizations.json', 'utf8'), saved);
  } finally { fs.rmSync(home, {recursive: true, force: true}); }
});

test('project memory has no extra grant while shared memory reads still do', () => {
  const home = fs.mkdtempSync('/private/tmp/mirket-authorization-'); fs.chmodSync(home, 0o700);
  const runtime = {home, cwd: '/project', binding: {releaseId: 'release-one'}};
  try {
    assert.equal(needsAuthorization('mcp__mirket__memory_commit', {scope: {kind: 'project'}}), false);
    assert.throws(() => grant(runtime, 'mcp__mirket__memory_commit', {scope: {kind: 'project'}}, 1000), /needs no extra authorization/);
    const sharedRead = {scope: {kind: 'shared_preferences'}, record_id: 'record-one'};
    assert.equal(needsAuthorization('mcp__mirket__memory_read', sharedRead), true);
    grant(runtime, 'mcp__mirket__memory_read', sharedRead, 1000);
    assert.equal(authorized(runtime, 'mcp__mirket__memory_read', sharedRead, 1001), true);
    assert.equal(authorized(runtime, 'mcp__mirket__memory_read', {...sharedRead, record_id: 'record-two'}, 1001), false);
  } finally { fs.rmSync(home, {recursive: true, force: true}); }
});

test('an unexpired shared read grant survives a matching authorization helper reload', async () => {
  const home = fs.mkdtempSync('/private/tmp/mirket-authorization-'); fs.chmodSync(home, 0o700);
  const runtime = {home, cwd: '/project', binding: {releaseId: 'release-one'}};
  const tool = 'mcp__mirket__memory_read';
  const args = {scope: {kind: 'shared_preferences'}, record_id: 'record-one'};
  const statePath = home+'/authorizations.json';
  try {
    assert.equal(needsAuthorization(tool, args), true);
    assert.equal(authorized(runtime, tool, args, 1000), false);
    grant(runtime, tool, args, 1000);
    const saved = fs.readFileSync(statePath);
    const diskState = JSON.parse(saved.toString('utf8'));
    assert.equal(diskState.schema, 1);
    assert.equal(diskState.grants.length, 1);
    assert.equal(diskState.grants[0].expires, 301000);

    const moduleUrl = new URL('../dist/extension/authorization.js', import.meta.url);
    moduleUrl.searchParams.set('unit-reload', String(Date.now()));
    const reloaded = await import(moduleUrl.href);
    const sameBinding = {home, cwd: '/project', binding: {releaseId: 'release-one'}};
    assert.equal(reloaded.authorized(sameBinding, tool, args, 1001), true);
    assert.equal(reloaded.authorized(sameBinding, tool, args, 1002), true);
    assert.equal(reloaded.authorized({...sameBinding, cwd: '/other'}, tool, args, 1001), false);
    assert.equal(reloaded.authorized({...sameBinding, binding: {releaseId: 'release-two'}}, tool, args, 1001), false);
    assert.equal(reloaded.authorized(sameBinding, tool, args, 301000), false);
    assert.deepEqual(fs.readFileSync(statePath), saved);
    assert.deepEqual(fs.readdirSync(home), ['authorizations.json']);
  } finally { fs.rmSync(home, {recursive: true, force: true}); }
});

test('malformed persisted authorization state refuses reads and grants without state effects', () => {
  const home = fs.mkdtempSync('/private/tmp/mirket-authorization-'); fs.chmodSync(home, 0o700);
  const runtime = {home, cwd: '/project', binding: {releaseId: 'release-one'}};
  const tool = 'mcp__mirket__memory_read';
  const args = {scope: {kind: 'shared_preferences'}, record_id: 'record-one'};
  const statePath = home+'/authorizations.json';
  const canaryPath = home+'/canary';
  try {
    fs.writeFileSync(canaryPath, 'unchanged canary\n');
    for (const [malformed, expectedError] of [
      ['{invalid json', SyntaxError],
      [JSON.stringify({schema: 2, grants: []}), /Invalid authorization state/],
      [JSON.stringify({schema: 1, grants: [{key: 'not-a-hash', expires: 301000}]}), /Invalid authorization state/],
      [JSON.stringify({schema: 1, grants: [{key: 'a'.repeat(64), expires: '301000'}]}), /Invalid authorization state/],
    ]) {
      fs.writeFileSync(statePath, malformed);
      const beforeState = fs.readFileSync(statePath);
      const beforeCanary = fs.readFileSync(canaryPath);
      const beforeNames = fs.readdirSync(home).sort();
      assert.throws(() => authorized(runtime, tool, args, 1001), expectedError, malformed);
      assert.throws(() => grant(runtime, tool, args, 1001), expectedError, malformed);
      assert.deepEqual(fs.readFileSync(statePath), beforeState, malformed);
      assert.deepEqual(fs.readFileSync(canaryPath), beforeCanary, malformed);
      assert.deepEqual(fs.readdirSync(home).sort(), beforeNames, malformed);
    }
  } finally { fs.rmSync(home, {recursive: true, force: true}); }
});
