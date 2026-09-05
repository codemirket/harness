import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { connect, call } from './support.mjs';

test('real stdio MCP clients share catalog changes and expose tools/resources/prompts', async t => {
  const root = await mkdtemp(join(tmpdir(), 'development-protocol-'));
  const first = await connect(join(root, 'state'), join(root, 'volumes'));
  const second = await connect(join(root, 'state'), join(root, 'volumes'));
  t.after(async () => { await first.client.close(); await second.client.close(); await rm(root, { recursive: true, force: true }); });
  const tools = await first.client.listTools();
  assert.equal(tools.tools.length, 10);
  const catalog = await call(first.client, 'list_catalog');
  assert.equal(catalog.catalog.length, 3);
  await call(first.client, 'add_catalog_entry', { id: 'mail', description: 'Mail', image: 'axllent/mailpit:v1.27', dataPath: '/data', ports: [{ name: 'smtp', container: 1025 }] });
  assert.ok((await call(second.client, 'list_catalog')).catalog.some(x => x.id === 'mail'));
  await call(second.client, 'remove_catalog_entry', { id: 'mail' });
  assert.equal((await call(first.client, 'list_catalog')).catalog.length, 3);
  assert.equal((await first.client.readResource({ uri: 'containers://catalog' })).contents.length, 1);
  assert.equal((await first.client.getPrompt({ name: 'use-local-service', arguments: { service: 'postgres', project: 'app-one' } })).messages.length, 1);
  const invalid = await first.client.callTool({ name: 'ensure_service', arguments: { template: 'postgres', alias: '../escape' } });
  assert.ok(invalid.isError);
  assert.equal(first.stderr(), '');
});
