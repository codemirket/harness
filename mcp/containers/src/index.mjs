#!/usr/bin/env node
import { homedir } from 'node:os';
import { join, resolve } from 'node:path';
import { serveStdio } from '@modelcontextprotocol/server/stdio';
import { Store } from './store.mjs';
import { Docker } from './docker.mjs';
import { Manager } from './manager.mjs';
import { createServer } from './server.mjs';

process.umask(0o077);
const state = resolve(process.env.DEVELOPMENT_MCP_HOME ?? join(homedir(), '.dev.mcp'));
const volumes = resolve(process.env.DEVELOPMENT_MCP_VOLUMES ?? join(homedir(), '.dev.volumes'));
const store = new Store(state);
const manager = new Manager(store, new Docker(), volumes);
if (process.argv.includes('--check')) {
  await manager.local();
  console.log(JSON.stringify({ ok: true, state, volumes, catalog: manager.listCatalog().map(t => t.id) }));
  store.close();
} else {
  serveStdio(() => createServer(manager), { onerror: () => console.error('Containers MCP transport error.') });
}
