// Keep the official Penpot MCP plugin connected in a real Browserless tab.
// This process is supervised by workspace-server.mjs and writes diagnostics to stderr only.
import { chromium } from 'playwright-core';
import fs from 'node:fs/promises';

const frontend = 'http://penpot-frontend:8080';
const manifest = 'http://penpot-mcp:4400/manifest.json';
const browserless = new URL('ws://browserless:3000');
browserless.searchParams.set('token', process.env.MIRKET_BROWSERLESS_TOKEN || '');
// The plugin requires Web Crypto on both its frame and ancestor origin.
// These HTTP origins are confined to the owned, private design network.
browserless.searchParams.set('launch', JSON.stringify({args: [
  `--unsafely-treat-insecure-origin-as-secure=${frontend},${new URL(manifest).origin}`,
]}));
const email = process.env.MIRKET_PENPOT_EMAIL;
const password = process.env.MIRKET_PENPOT_PASSWORD;
const savedFile = '/workspace/.mirket/penpot-session.json';
if (!email || !password || !process.env.MIRKET_BROWSERLESS_TOKEN) {
  throw Error('Missing private Penpot account or Browserless token');
}

let browser;
let closed = false;
for (const signal of ['SIGINT', 'SIGTERM']) {
  process.once(signal, () => { closed = true; void browser?.close().finally(() => process.exit(0)); });
}

async function rpc(context, name, data) {
  const url = `${frontend}/api/main/methods/${name}`;
  const response = data === undefined
    ? await context.request.get(url, { headers: { Accept: 'application/json' } })
    : await context.request.post(url, { data, headers: { Accept: 'application/json' } });
  const contentType = response.headers()['content-type'] || '';
  const raw = await response.body();
  if (raw.length > 8 * 1024 * 1024 || !contentType.includes('application/json')) throw Error(`Penpot ${name} returned an invalid response`);
  const value = JSON.parse(raw.toString('utf8'));
  if (!response.ok()) {
    const error = Error(`Penpot ${name} failed (${response.status()}; ${value.code || 'unknown'})`);
    error.status = response.status();
    error.code = value.code;
    throw error;
  }
  return value;
}

async function account(context) {
  let profile;
  try {
    profile = await rpc(context, 'login-with-password', { email, password });
  } catch (error) {
    if (error.status !== 400 || error.code !== 'wrong-credentials') throw error;
    const prepared = await rpc(context, 'prepare-register-profile', {
      email, password, fullname: 'Mirket local design', 'accept-newsletter-updates': false,
    });
    await rpc(context, 'register-profile', { token: prepared.token, 'accept-newsletter-updates': false });
    profile = await rpc(context, 'login-with-password', { email, password });
  }
  if (!profile.defaultTeamId || !profile.defaultProjectId) profile = await rpc(context, 'get-profile');
  if (!profile.defaultTeamId || !profile.defaultProjectId) throw Error('Penpot profile has no default team/project');
  return profile;
}

async function designFile(context, profile) {
  let saved;
  try { saved = JSON.parse(await fs.readFile(savedFile, 'utf8')); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  if (saved) {
    if (!/^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$/i.test(saved.fileId || '') || saved.teamId !== profile.defaultTeamId) {
      throw Error('Persisted Penpot file identity is invalid');
    }
    return saved.fileId;
  }
  const file = await rpc(context, 'create-file', { 'project-id': profile.defaultProjectId, name: 'Mirket design workspace' });
  if (!/^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$/i.test(file.id || '')) throw Error('Penpot returned an invalid file ID');
  await fs.mkdir('/workspace/.mirket', {recursive: true, mode: 0o700});
  const temporary = `${savedFile}.${process.pid}.tmp`;
  await fs.writeFile(temporary, JSON.stringify({schemaVersion: 1, fileId: file.id, teamId: profile.defaultTeamId}) + '\n', {mode: 0o600, flag: 'wx'});
  await fs.rename(temporary, savedFile);
  return file.id;
}

async function waitForPluginFrame(page) {
  const deadline = Date.now() + 30000;
  while (Date.now() < deadline) {
    for (const frame of page.frames()) {
      if (await frame.locator('#connect-btn').count().catch(() => 0)) return frame;
    }
    await page.waitForTimeout(250);
  }
  throw Error('Official Penpot MCP plugin UI did not open');
}

async function main() {
  browser = await chromium.connectOverCDP(browserless.href, { timeout: 30000 });
  const context = await browser.newContext({ viewport: null, serviceWorkers: 'block' });
  const profile = await account(context);
  const fileId = await designFile(context, profile);
  const page = await context.newPage();
  await page.goto(`${frontend}/#/workspace?team-id=${profile.defaultTeamId}&file-id=${fileId}`, { waitUntil: 'domcontentloaded' });
  await page.waitForSelector('[data-testid=viewport]', { timeout: 30000 });
  await page.locator('[data-tool=plugins]').click();
  const pluginName = 'Penpot MCP Plugin';
  if (await page.getByText(pluginName, { exact: true }).count() === 0) {
    await page.getByPlaceholder('Write a plugin URL').fill(manifest);
    await page.getByRole('button', { name: 'Install', exact: true }).click();
    await page.getByRole('button', { name: /Allow|Accept|Install/, exact: false }).last().click();
  }
  const rowOpen = page.getByText(pluginName, { exact: true }).locator('xpath=../..').getByRole('button', { name: 'Open', exact: true });
  await rowOpen.click();
  const frame = await waitForPluginFrame(page);
  await frame.locator('#connect-btn').click();
  const status = frame.locator('#connection-status');
  await frame.locator('#connection-status[data-status="connected"], #connection-status[data-status="error"]').waitFor({ timeout: 30000 });
  if (await status.getAttribute('data-status') !== 'connected') {
    const detail = (await frame.locator('#status-text').innerText()).trim().slice(0, 500);
    throw Error(`Official Penpot MCP plugin failed: ${detail}`);
  }
  // Set the canvas viewport after Browserless initializes the plugin iframe.
  await page.setViewportSize({ width: 1600, height: 1000 });
  process.stderr.write(JSON.stringify({ ready: true, service: 'penpot-session', file_id: fileId, origin: frontend }) + '\n');
  while (!closed) {
    if (page.isClosed() || !(await frame.locator('#connection-status[data-status="connected"]').count().catch(() => 0))) {
      throw Error('Official Penpot MCP plugin session disconnected');
    }
    await page.waitForTimeout(2000);
  }
}

main().catch(async error => {
  let message = String(error?.message ?? error);
  for (const secret of [process.env.MIRKET_BROWSERLESS_TOKEN, password]) {
    if (secret) message = message.replaceAll(secret, '[redacted]');
  }
  process.stderr.write(`penpot-session: ${message.slice(0, 2000)}\n`);
  try { await browser?.close(); } catch {}
  process.exitCode = 1;
});
