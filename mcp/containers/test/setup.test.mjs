import { test } from 'node:test';
import assert from 'node:assert/strict';
import { registrationArgs, isOwnedRegistration } from '../setup.mjs';

test('registration uses absolute executable paths and Claude user scope', () => {
  const args = registrationArgs('claude', '/opt/node', '/runtime/server/src/index.mjs', '/opt/docker');
  assert.ok(args.includes('user'));
  assert.ok(args.includes('stdio'));
  assert.deepEqual(args.slice(-2), ['/opt/node', '/runtime/server/src/index.mjs']);
  assert.ok(args.includes('DEVELOPMENT_MCP_DOCKER=/opt/docker'));
});

test('registration refuses to overwrite an unrelated server with the same name', () => {
  const prior = { entry: '/runtime/old/src/index.mjs', node: '/opt/node' };
  assert.equal(isOwnedRegistration({ command: '/opt/node', args: ['/unrelated/index.mjs'] }, prior), false);
  assert.equal(isOwnedRegistration({ command: '/opt/node', args: [prior.entry] }, prior), true);
});

test('Desktop registration preserves unrelated settings and rejects a name collision', async t => {
  const { mkdtemp, writeFile, readFile, rm } = await import('node:fs/promises');
  const { tmpdir } = await import('node:os');
  const { join } = await import('node:path');
  const { registerDesktop } = await import('../setup.mjs');
  const directory = await mkdtemp(join(tmpdir(), 'development-desktop-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const file = join(directory, 'claude_desktop_config.json');
  await writeFile(file, JSON.stringify({ theme: 'dark', mcpServers: { other: { command: 'other' }, development: { command: '/old/node', args: ['/old/server'] } } }));
  const registration = { command: '/node', args: ['/runtime/src/index.mjs'], env: { DEVELOPMENT_MCP_DOCKER: '/docker' } };
  await registerDesktop(file, registration, { node: '/old/node', entry: '/old/server' });
  const saved = JSON.parse(await readFile(file, 'utf8'));
  assert.equal(saved.theme, 'dark');
  assert.equal(saved.mcpServers.development, undefined);
  assert.equal(saved.mcpServers.other.command, 'other');
  assert.deepEqual(saved.mcpServers.containers, registration);
  await assert.rejects(registerDesktop(file, { command: '/unrelated', args: ['server'] }), /unrelated/);
});

test('Codex timeout update uses a versioned native config mutation', async t => {
  const { configureCodexTimeout } = await import('../setup-codex.mjs');
  const { mkdtemp, writeFile, rm } = await import('node:fs/promises');
  const { tmpdir } = await import('node:os');
  const { join } = await import('node:path');
  const directory = await mkdtemp(join(tmpdir(), 'development-codex-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const fixture = join(directory, 'app-server.mjs');
  await writeFile(fixture, `import { createInterface } from 'node:readline';
    for await (const line of createInterface({input:process.stdin})) {
      const request=JSON.parse(line);
      if (request.id == null) continue;
      let result={};
      if(request.method==='config/read') result={layers:[{name:{type:'user',file:'/user/config.toml'},version:'expected-version'}]};
      if(request.method==='config/value/write') {
        if(request.params.expectedVersion!=='expected-version'||request.params.keyPath!=='mcp_servers.containers.tool_timeout_sec'||request.params.value!==1800) {process.stdout.write(JSON.stringify({id:request.id,error:{code:-1}})+'\\n');continue;}
        result={status:'ok',version:'next-version',filePath:'/user/config.toml'};
      }
      process.stdout.write(JSON.stringify({id:request.id,result})+'\\n');
    }`);
  await configureCodexTimeout({ command: process.execPath, args: [fixture] });
});
