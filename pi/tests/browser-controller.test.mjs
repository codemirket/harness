import test from 'node:test';
import assert from 'node:assert/strict';
import {createBrowserController, BROWSER_PREFIX} from '../dist/extension/browser.js';
import {mkdtempSync, mkdirSync, writeFileSync, rmSync, realpathSync} from 'node:fs';
import {join} from 'node:path';

test('optional browser controller stays lazy and cannot grant from a model tool', async () => {
  const handlers = new Map(), effects = [];
  const pi = {on(name, handler) { handlers.set(name, handler); }, unregisterMcpServer(name) { effects.push(['unregister', name]); }, registerMcpServer() { throw Error('must not register'); }};
  const controller = createBrowserController(pi, () => { throw Error('ordinary startup must not inspect the runtime'); }, () => { throw Error('must not read user agent state'); });
  assert.equal(controller.status().status, 'inactive');
  assert.deepEqual(effects, []);
  const gate = handlers.get('tool_call');
  assert.equal(gate({toolName: 'read', input: {}}, {cwd: '/private/tmp'}), undefined);
  assert.deepEqual(gate({toolName: BROWSER_PREFIX + 'browser_snapshot', input: {}}, {cwd: '/private/tmp'}), {block: true, reason: 'Browser session has not been authorized'});
  await controller.revoke(); await controller.cancel();
  assert.deepEqual(effects, []);
  await assert.rejects(controller.open('https://example.org', {cwd: '/private/tmp'}), /literal/);
  assert.deepEqual(effects, []);
});

test('browser controller refuses exact and hyphen alias namespaces in global/project MCP settings and peer registrations', {skip: process.platform !== 'darwin' || process.arch !== 'arm64'}, async () => {
  const root = realpathSync(mkdtempSync('/private/tmp/mirket-browser-alias-'));
  const project = join(root, 'project'), agent = join(root, 'agent');
  mkdirSync(project, {mode: 0o700}); mkdirSync(agent, {mode: 0o700}); mkdirSync(join(project, '.pi'), {mode: 0o700});
  let registrations = [], registered = false;
  const pi = {on() {}, unregisterMcpServer() { throw Error('must preserve the other owner'); }, registerMcpServer() { registered = true; }, getMcpServers() { return registrations; }, getCommands() { return [{name: 'mcp', sourceInfo: {path: 'builtin:mcp'}}]; }};
  const runtime = {cwd: project, home: join(root, 'mirket'), binding: {releaseId: 'sha256:test'}};
  const controller = createBrowserController(pi, () => runtime, () => agent);
  try {
    for (const directory of [agent, join(project, '.pi')]) for (const name of ['mirket_browser', 'mirket-browser']) {
      const file = join(directory, 'mcp.json');
      writeFileSync(file, JSON.stringify({mcpServers: {[name]: {command: '/nonexistent'}}}), {mode: 0o600});
      await assert.rejects(controller.open('http://127.0.0.1:8123', {cwd: project}), /Same-namespace browser MCP override/);
      assert.equal(registered, false); rmSync(file);
    }
    registrations = [{name: 'mirket-browser', config: {command: '/nonexistent'}}];
    await assert.rejects(controller.open('http://127.0.0.1:8123', {cwd: project}), /namespace is already owned/);
    assert.equal(registered, false);
  } finally { rmSync(root, {recursive: true, force: true}); }
});
