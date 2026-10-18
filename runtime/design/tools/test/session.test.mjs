import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {spawn, spawnSync} from 'node:child_process';
import {once} from 'node:events';
import {randomUUID} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {Server} from '@modelcontextprotocol/sdk/server/index.js';
import {StreamableHTTPServerTransport} from '@modelcontextprotocol/sdk/server/streamableHttp.js';
import {LATEST_PROTOCOL_VERSION, ListToolsRequestSchema} from '@modelcontextprotocol/sdk/types.js';

const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
async function within(promise, ms, label) {
  let timer;
  try {
    return await Promise.race([promise, new Promise((_, reject) => {
      timer = setTimeout(() => reject(Error(`${label} exceeded ${ms} ms`)), ms);
    })]);
  } finally { clearTimeout(timer); }
}

test('discovery avoids rendering imports and EOF closes the Penpot HTTP session', {timeout: 10000}, async () => {
  const sessions = new Map();
  let openGets = 0;
  let sawGet;
  const gotGet = new Promise(resolve => { sawGet = resolve; });
  const listener = http.createServer((request, response) => {
    if (request.url !== '/mcp') { response.writeHead(404).end(); return; }
    void (async () => {
      let session = sessions.get(request.headers['mcp-session-id']);
      if (!session && request.method === 'POST') {
        const transport = new StreamableHTTPServerTransport({
          sessionIdGenerator: randomUUID,
          onsessioninitialized: id => sessions.set(id, {transport, server: upstream}),
        });
        const upstream = new Server({name: 'loopback-penpot', version: '1.0.0'}, {capabilities: {tools: {listChanged: false}}});
        upstream.setRequestHandler(ListToolsRequestSchema, async () => ({tools: [{
          name: 'penpot_api_info', description: 'Synthetic Penpot fixture', inputSchema: {type: 'object', properties: {}},
        }]}));
        await upstream.connect(transport);
        session = {transport, server: upstream};
      }
      if (!session) { response.writeHead(404).end(); return; }
      if (request.method === 'GET') {
        openGets++;
        response.once('close', () => { openGets--; });
        sawGet();
      }
      await session.transport.handleRequest(request, response);
    })().catch(error => {
      if (!response.headersSent) response.writeHead(500).end(String(error));
      else response.destroy(error);
    });
  });
  let child;
  const fixture = await fs.mkdtemp(path.join(os.tmpdir(), 'mirket-discovery-imports-'));
  const loader = path.join(fixture, 'loader.mjs');
  await fs.writeFile(loader, `
    export async function resolve(specifier, context, next) {
      if (['style-dictionary', 'satori', '@resvg/resvg-js', 'svgo', 'playwright-core'].includes(specifier)) {
        throw Error('Discovery loaded renderer: ' + specifier);
      }
      return next(specifier, context);
    }
  `);
  try {
    listener.listen(0, '127.0.0.1');
    await once(listener, 'listening');
    const endpoint = `http://127.0.0.1:${listener.address().port}/mcp`;
    child = spawn(process.execPath, ['--loader', loader, fileURLToPath(new URL('../server.mjs', import.meta.url))], {
      stdio: ['pipe', 'pipe', 'pipe'],
      env: {...process.env, MIRKET_DESIGN_GROUPS: 'core,penpot', MIRKET_PENPOT_MCP_URL: endpoint},
    });
    let stdout = '';
    let stderr = '';
    let nextId = 0;
    const pending = new Map();
    child.stdout.on('data', chunk => {
      stdout += chunk;
      for (;;) {
        const end = stdout.indexOf('\n');
        if (end < 0) break;
        const line = stdout.slice(0, end);
        stdout = stdout.slice(end + 1);
        const message = JSON.parse(line);
        const settle = pending.get(message.id);
        if (settle) { pending.delete(message.id); settle(message); }
      }
    });
    child.stderr.on('data', chunk => { stderr += chunk.toString(); });
    function request(method, params) {
      const id = ++nextId;
      const result = new Promise(resolve => pending.set(id, resolve));
      child.stdin.write(JSON.stringify({jsonrpc: '2.0', id, method, params}) + '\n');
      return result;
    }
    const initialized = await within(request('initialize', {
      protocolVersion: LATEST_PROTOCOL_VERSION, capabilities: {}, clientInfo: {name: 'eof-test', version: '1.0.0'},
    }), 3000, 'initialization');
    assert.ok(initialized.result, JSON.stringify(initialized));
    child.stdin.write(JSON.stringify({jsonrpc: '2.0', method: 'notifications/initialized'}) + '\n');
    const listed = await within(request('tools/list', {}), 3000, 'tool discovery');
    assert.ok(listed.result?.tools.some(tool => tool.name === 'penpot_api_info'), JSON.stringify(listed));
    await within(gotGet, 3000, 'persistent upstream GET');
    assert.equal(openGets, 1);

    const exited = once(child, 'exit');
    child.stdin.end();
    const [code, signal] = await within(exited, 1800, 'worker EOF exit');
    assert.equal(code, 0, `signal=${signal}; stderr=${stderr.slice(-1000)}`);
    await within((async () => { while (openGets !== 0) await delay(10); })(), 1000, 'upstream stream close');
  } finally {
    child?.stdin.destroy();
    if (child && child.exitCode === null && child.signalCode === null) {
      child.kill('SIGKILL');
      await within(once(child, 'exit'), 1000, 'fixture child cleanup').catch(() => {});
    }
    await Promise.allSettled([...sessions.values()].map(session => session.server.close()));
    listener.closeAllConnections();
    if (listener.listening) await new Promise(resolve => listener.close(resolve));
    await fs.rm(fixture, {recursive: true, force: true});
  }
});

test('stdio EOF reaps a TERM-resistant renderer before a late write', {timeout: 12000}, async () => {
  const workspace = await fs.mkdtemp(path.join(os.tmpdir(), 'mirket-eof-child-'));
  const pidFile = path.join(workspace, 'child.pid');
  const readyFile = path.join(workspace, 'server.ready');
  const lateFile = path.join(workspace, 'late-write');
  const marker = `mirket-owned-eof-${randomUUID()}`;
  const childCode = `
    const fs = require('node:fs');
    const marker = ${JSON.stringify(marker)};
    process.on('SIGTERM', () => {});
    fs.writeFileSync(${JSON.stringify(pidFile)}, String(process.pid));
    setTimeout(() => fs.writeFileSync(${JSON.stringify(lateFile)}, marker), 7000);
    setInterval(() => {}, 1000);
  `;
  const pathsUrl = new URL('../paths.mjs', import.meta.url).href;
  const serverUrl = new URL('../server.mjs', import.meta.url).href;
  const wrapper = `
    import {run} from ${JSON.stringify(pathsUrl)};
    import {writeFileSync} from 'node:fs';
    void run(process.execPath, ['-e', ${JSON.stringify(childCode)}], {timeoutMs: 30000}).catch(() => {});
    await import(${JSON.stringify(serverUrl)});
    writeFileSync(${JSON.stringify(readyFile)}, 'ready');
  `;
  let worker;
  let ownedPid;
  try {
    worker = spawn(process.execPath, ['--input-type=module', '-e', wrapper], {
      stdio: ['pipe', 'pipe', 'pipe'],
      env: {...process.env, MIRKET_DESIGN_WORKSPACE: workspace},
    });
    let stderr = '';
    worker.stderr.on('data', chunk => { stderr += chunk.toString(); });
    worker.stdout.resume();
    await within((async () => {
      for (;;) {
        if ((await fs.stat(readyFile).catch(() => null)) && (await fs.stat(pidFile).catch(() => null))) return;
        if (worker.exitCode !== null || worker.signalCode !== null) throw Error(`worker exited before ready: ${stderr.slice(-1000)}`);
        await delay(10);
      }
    })(), 3000, 'active renderer and MCP worker readiness');
    const observedReadyAt = Date.now();
    ownedPid = Number(await fs.readFile(pidFile, 'utf8'));
    assert.ok(Number.isSafeInteger(ownedPid) && ownedPid > 0);
    const running = spawnSync('ps', ['-ww', '-p', String(ownedPid), '-o', 'command='], {encoding: 'utf8', timeout: 1000});
    assert.equal(running.status, 0);
    assert.ok(running.stdout.includes(marker), 'PID belongs to this test renderer');

    const exited = once(worker, 'exit');
    worker.stdin.end();
    const [code, signal] = await within(exited, 5500, 'active worker EOF exit');
    assert.equal(code, 0, `signal=${signal}; stderr=${stderr.slice(-1000)}`);
    await within((async () => {
      for (;;) {
        try { process.kill(ownedPid, 0); }
        catch (error) { if (error.code === 'ESRCH') return; throw error; }
        await delay(20);
      }
    })(), 1000, 'resistant renderer reap');
    await delay(Math.max(0, observedReadyAt + 7250 - Date.now()));
    await assert.rejects(fs.stat(lateFile), {code: 'ENOENT'});
  } finally {
    worker?.stdin.destroy();
    if (worker && worker.exitCode === null && worker.signalCode === null) {
      worker.kill('SIGKILL');
      await within(once(worker, 'exit'), 1000, 'worker fixture cleanup').catch(() => {});
    }
    if (ownedPid) {
      const identity = spawnSync('ps', ['-ww', '-p', String(ownedPid), '-o', 'command='], {encoding: 'utf8', timeout: 1000});
      if (identity.status === 0 && identity.stdout.includes(marker)) {
        try { process.kill(-ownedPid, 'SIGKILL'); } catch (error) { if (error.code !== 'ESRCH') throw error; }
      }
    }
    await fs.rm(workspace, {recursive: true, force: true});
  }
});
