import type { ExtensionAPI, ExtensionContext } from '@earendil-works/pi-coding-agent';
import { randomUUID } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { join } from 'node:path';
import { lstatSync, mkdirSync, mkdtempSync, realpathSync, rmSync, writeFileSync } from 'node:fs';
import { regularBytes, type VerifiedRuntime } from './runtime.js';
import { executableIdentity, runtimeBrowserBindings } from './runtime-tools.js';
import { BROWSER_TOOLS, browserOrigin, createBrowserPolicy, assertBrowserBinding, type BrowserGrant, type BrowserBindingIdentity } from './browser-policy.js';
import { createBrowserProxy } from './browser-proxy.js';

export const BROWSER_PEER = 'mirket_browser';
export const BROWSER_PREFIX = `mcp__${BROWSER_PEER}__`;
interface ProcessIdentity {pid: number; parent: number; started: string; command: string}
const delay = (ms: number) => new Promise<void>(resolve => setTimeout(resolve, ms));
// Pi 1.4.0 maps hyphens to underscores in MCP tool namespaces.
const ownsBrowserNamespace = (name: string) => name.replace(/-/gu, '_') === BROWSER_PEER;
function processes(): ProcessIdentity[] {
  return execFileSync('/bin/ps', ['-axo', 'pid=,ppid=,lstart=,command='], {encoding: 'utf8', timeout: 3000, maxBuffer: 4 * 1024 * 1024}).split('\n').flatMap(line => {
    const match = line.match(/^\s*(\d+)\s+(\d+)\s+(.{24})\s+(.*)$/u);
    return match ? [{pid: Number(match[1]), parent: Number(match[2]), started: match[3]!, command: match[4]!}] : [];
  });
}
function privateDirectory(path: string) {
  const stat = lstatSync(path);
  if (!stat.isDirectory() || stat.isSymbolicLink() || stat.mode & 0o077 || realpathSync(path) !== path) throw new Error('Browser state directory is not private and canonical');
}
function noOverride(agentDir: string, project: string) {
  for (const file of [join(agentDir, 'mcp.json'), join(project, '.pi', 'mcp.json')]) {
    try { if (Object.keys(JSON.parse(regularBytes(file, 1024 * 1024).toString()).mcpServers ?? {}).some(ownsBrowserNamespace)) throw new Error(`Same-namespace browser MCP override: ${file}`); }
    catch (error) { if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error; }
  }
}
type BrowserPolicy = ReturnType<typeof createBrowserPolicy>;
interface ActiveSession {
  runtime: VerifiedRuntime; grant: BrowserGrant; policy: BrowserPolicy; work: string; socketDirectory: string;
  proxy: Awaited<ReturnType<typeof createBrowserProxy>>; config: Extract<Parameters<ExtensionAPI['registerMcpServer']>[1], {command: string}>;
  observed: Map<number, ProcessIdentity>; observer: ReturnType<typeof setInterval>; expiry: ReturnType<typeof setTimeout>;
}
export interface BrowserLimits {expiresInSeconds?: number; maxCalls?: number; expected?: BrowserBindingIdentity}

// Only a trusted explicit user command may call open. No browser/model tool can
// create or renew a grant. Pi owns MCP transport and process registration.
export function createBrowserController(pi: ExtensionAPI, getRuntime: () => VerifiedRuntime | undefined, getAgentDir: () => string, recordUse?: (runtime: VerifiedRuntime, toolName: string) => boolean) {
  let active: ActiveSession | undefined, generation = 0, opening = false;
  let cleanup: Promise<void> | undefined, cleanupFailure: string | undefined, lastReason = 'Browser session has not been authorized';
  const pendingCalls = new Map<string, {generation: number; removeAbort: () => void}>();
  function observe(session: ActiveSession): ProcessIdentity[] {
    const rows = processes();
    const roots = rows.filter(row => row.parent === process.pid && row.command.includes(join(session.work, 'browser.json')));
    for (const row of roots) session.observed.set(row.pid, row);
    let added = true;
    while (added) { added = false; for (const row of rows) {
      const owner = session.observed.get(row.parent);
      if (owner && rows.some(parent => parent.pid === owner.pid && parent.started === owner.started) && !session.observed.has(row.pid)) { session.observed.set(row.pid, row); added = true; }
    } }
    return rows.filter(row => session.observed.get(row.pid)?.started === row.started);
  }
  async function reap(session: ActiveSession) {
    let living = observe(session), deadline = Date.now() + 2000;
    while (living.length && Date.now() < deadline) { await delay(50); living = observe(session); }
    for (const signal of ['SIGTERM', 'SIGKILL'] as const) {
      for (const row of living) {
        if (!processes().some(current => current.pid === row.pid && current.started === row.started)) continue;
        try { process.kill(row.pid, signal); } catch (error) { if ((error as NodeJS.ErrnoException).code !== 'ESRCH') throw error; }
      }
      deadline = Date.now() + 1500;
      while (living.length && Date.now() < deadline) { await delay(50); living = observe(session); }
    }
    if (living.length) throw new Error('Observed old browser descendants remain; replacement refused');
  }
  function revoke(reason = 'Browser authorization revoked'): Promise<void> {
    lastReason = reason;
    ++generation;
    for (const call of pendingCalls.values()) call.removeAbort();
    pendingCalls.clear();
    const old = active;
    active = undefined;
    if (!old) return cleanup ?? Promise.resolve();
    old.policy.revoke();
    clearTimeout(old.expiry); clearInterval(old.observer);
    try { observe(old); } catch (error) { cleanupFailure = String(error); }
    pi.unregisterMcpServer(BROWSER_PEER);
    cleanup = (async () => {
      await old.proxy.close();
      await reap(old);
      if (cleanupFailure) throw new Error(cleanupFailure);
      for (const path of ['home', 'tmp', 'profile']) rmSync(join(old.work, path), {recursive: true, force: true});
      rmSync(old.socketDirectory, {recursive: true, force: true});
    })().catch(error => { cleanupFailure = String(error); throw error; });
    // Abort callbacks cannot await; the rejection remains available to the next
    // explicit revoke/open and is reported in status, without an unhandled promise.
    void cleanup.catch(() => {});
    return cleanup;
  }
  function verify(session: ActiveSession, project: string, tool?: string) {
    const runtime = getRuntime();
    if (!runtime || runtime.cwd !== session.grant.project || realpathSync(project) !== session.grant.project || runtime.binding.releaseId !== session.grant.release || runtime.home !== session.runtime.home) throw new Error('Browser project or release changed');
    noOverride(getAgentDir(), runtime.cwd);
    const binding = runtimeBrowserBindings(runtime);
    assertBrowserBinding(session.grant, binding);
    if (executableIdentity(session.config.command) !== runtime.binding.nodeSha256) throw new Error('Private browser Node identity changed');
    const mcp = pi.getCommands().find(command => command.name === 'mcp');
    if (mcp?.sourceInfo.path !== 'builtin:mcp') throw new Error('Pi MCP command is missing or replaced');
    const registrations = pi.getMcpServers();
    if (registrations.some(server => server.name !== BROWSER_PEER && ownsBrowserNamespace(server.name))) throw new Error('Browser MCP namespace has another owner');
    const registration = registrations.find(server => server.name === BROWSER_PEER);
    if (!registration || JSON.stringify(registration.config) !== JSON.stringify(session.config)) throw new Error('Browser MCP registration changed');
    if (tool) {
      const owned = pi.getAllTools().find(candidate => candidate.name === tool);
      if (owned?.sourceInfo.path !== 'builtin:mcp' || owned.exposure !== 'direct' || !pi.getActiveTools().includes(tool)) throw new Error('Browser tool is unavailable, filtered or has another owner');
    }
    privateDirectory(session.grant.output);
    privateDirectory(session.work);
  }
  pi.on('tool_call', (event, ctx) => {
    if (!event.toolName.startsWith(BROWSER_PREFIX)) return;
    try {
      const session = active;
      if (!session) throw new Error(lastReason);
      verify(session, ctx.cwd, event.toolName);
      const args = session.policy.check(event.toolName.slice(BROWSER_PREFIX.length), event.input, {project: realpathSync(ctx.cwd), release: getRuntime()!.binding.releaseId});
      Object.assign(event.input, args);
      const abort = () => { void revoke('Browser call cancelled').catch(() => {}); };
      if (ctx.signal?.aborted) { abort(); throw new Error('Browser call already cancelled'); }
      const signal = ctx.signal;
      signal?.addEventListener('abort', abort, {once: true});
      pendingCalls.set(event.toolCallId, {generation, removeAbort: () => signal?.removeEventListener('abort', abort)});
    } catch (error) {
      return {block: true, reason: error instanceof Error ? error.message : String(error)};
    }
  });
  pi.on('tool_result', async (event, ctx) => {
    if (!event.toolName.startsWith(BROWSER_PREFIX)) return;
    const call = pendingCalls.get(event.toolCallId);
    pendingCalls.delete(event.toolCallId); call?.removeAbort();
    if (!call || call.generation !== generation || !active) return {isError: true, content: [{type: 'text', text: 'Browser result discarded after cancellation, revocation or session change'}]};
    if (event.isError) { await revoke('Browser operation failed; authorize a fresh session'); return; }
    try { recordUse?.(active.runtime, event.toolName); }
    catch (error) {
      try { ctx.ui.notify(`Project guidance sync failed: ${error instanceof Error ? error.message : String(error)}`, 'warning'); } catch { /* Guidance maintenance must not change a successful browser result. */ }
    }
    if (!active.policy.active) await revoke('Browser call budget exhausted or grant expired');
  });
  return {
    async open(origin: string, ctx: Pick<ExtensionContext, 'cwd'>, limits: BrowserLimits = {}) {
      browserOrigin(origin);
      if (process.platform !== 'darwin' || process.arch !== 'arm64') throw new Error('Optional browser cleanup is qualified only on macOS ARM64');
      if (opening) throw new Error('Browser authorization is already opening');
      opening = true;
      let prepared: {work: string; socketDirectory: string; proxy?: Awaited<ReturnType<typeof createBrowserProxy>>} | undefined;
      try {
        await revoke('Replacing browser authorization');
        if (cleanupFailure) throw new Error(`Prior browser cleanup failed: ${cleanupFailure}`);
        const current = generation, runtime = getRuntime();
        if (!runtime || realpathSync(ctx.cwd) !== runtime.cwd) throw new Error('No verified runtime bound to this browser project');
        noOverride(getAgentDir(), runtime.cwd);
        if (pi.getMcpServers().some(server => ownsBrowserNamespace(server.name))) throw new Error('Browser MCP namespace is already owned');
        if (pi.getCommands().find(command => command.name === 'mcp')?.sourceInfo.path !== 'builtin:mcp') throw new Error('Pi builtin MCP is unavailable or replaced');
        const binding = runtimeBrowserBindings(runtime);
        if (limits.expected) assertBrowserBinding(limits.expected, binding);
        const lifetime = limits.expiresInSeconds ?? 300, calls = limits.maxCalls ?? 64;
        if (!Number.isSafeInteger(lifetime) || lifetime < 1 || lifetime > 300 || !Number.isSafeInteger(calls) || calls < 1 || calls > 64) throw new Error('Browser authorization must be 1–300 seconds and 1–64 calls');
        const node = join(runtime.home, 'releases', `r-${runtime.binding.releaseId.slice(7)}`, 'runtime/node/bin/node');
        if (executableIdentity(node) !== runtime.binding.nodeSha256) throw new Error('Private browser Node identity changed');
        privateDirectory(runtime.home);
        const parent = join(runtime.home, 'browser');
        try { mkdirSync(parent, {mode: 0o700}); } catch (error) { if ((error as NodeJS.ErrnoException).code !== 'EEXIST') throw error; }
        privateDirectory(parent);
        const work = join(parent, randomUUID()); mkdirSync(work, {mode: 0o700});
        for (const name of ['home', 'tmp', 'profile', 'output']) mkdirSync(join(work, name), {mode: 0o700});
        const socketDirectory = realpathSync(mkdtempSync('/private/tmp/mb-'));
        prepared = {work, socketDirectory};
        const grant: BrowserGrant = {project: runtime.cwd, release: runtime.binding.releaseId, ...binding, origin, output: join(work, 'output'), expires: Date.now() + lifetime * 1000, maxCalls: calls, methods: ['GET', 'HEAD'], tools: BROWSER_TOOLS};
        // verify() rechecks the complete bindings before every policy check.
        const policy = createBrowserPolicy(grant, () => {});
        const proxy = await createBrowserProxy(origin, (method, url) => policy.network(method, url));
        prepared.proxy = proxy;
        if (current !== generation) throw new Error('Browser authorization changed while opening');
        const configFile = join(work, 'browser.json');
        writeFileSync(configFile, JSON.stringify({browser: {browserName: 'chromium', isolated: true, launchOptions: {executablePath: binding.browserPath, headless: true, proxy: {server: proxy.url, bypass: '<-loopback>'}, args: ['--disable-quic', '--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE 127.0.0.1', '--disable-background-networking', '--disable-component-update', '--disable-sync', '--disable-extensions', '--no-first-run', '--no-default-browser-check', '--use-mock-keychain', '--password-store=basic', '--force-webrtc-ip-handling-policy=disable_non_proxied_udp']}, contextOptions: {viewport: {width: 1000, height: 700}, reducedMotion: 'reduce', serviceWorkers: 'block', acceptDownloads: false}}, network: {allowedOrigins: [origin]}, outputDir: grant.output, timeouts: {action: 5000, navigation: 10000}}), {flag: 'wx', mode: 0o600});
        const config: ActiveSession['config'] = {command: node, args: [binding.peerPath, '--config', configFile, '--no-webmcp', '--block-service-workers'], cwd: runtime.cwd, env: {HOME: join(work, 'home'), TMPDIR: join(work, 'tmp'), PATH: '/nonexistent', NODE_OPTIONS: '', NODE_PATH: '', LD_PRELOAD: '', DYLD_INSERT_LIBRARIES: '', PWTEST_SOCKETS_DIR: socketDirectory, PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD: '1', PLAYWRIGHT_BROWSERS_PATH: '0', NO_PROXY: '', HTTP_PROXY: proxy.url, HTTPS_PROXY: proxy.url}, exposure: 'hidden', toolExposure: Object.fromEntries(BROWSER_TOOLS.map(tool => [tool, 'direct' as const])), timeout: 15};
        const session = {runtime, grant, policy, work, socketDirectory, proxy, config, observed: new Map<number, ProcessIdentity>()} as ActiveSession;
        active = session;
        prepared = undefined;
        session.observer = setInterval(() => { try { observe(session); } catch { void revoke('Browser ownership observation failed').catch(() => {}); } }, 100);
        session.expiry = setTimeout(() => { void revoke('Browser authorization expired').catch(() => {}); }, Math.max(1, grant.expires - Date.now()));
        session.observer.unref(); session.expiry.unref();
        pi.registerMcpServer(BROWSER_PEER, config);
        const deadline = Date.now() + 15000;
        // Unregister hides old definitions but leaves their names in getAllTools.
        // Wait for fresh direct exposure rather than accepting a stale name.
        const ready = () => pi.getAllTools().some(tool => tool.name === BROWSER_PREFIX + 'browser_navigate' && tool.exposure === 'direct') && pi.getActiveTools().includes(BROWSER_PREFIX + 'browser_navigate');
        while (active === session && Date.now() < deadline && !ready()) await delay(50);
        if (active !== session) throw new Error(lastReason);
        verify(session, ctx.cwd, BROWSER_PREFIX + 'browser_navigate');
        lastReason = '';
        return this.status();
      } catch (error) {
        await revoke('Browser session could not open');
        if (prepared) { await prepared.proxy?.close(); rmSync(prepared.work, {recursive: true, force: true}); rmSync(prepared.socketDirectory, {recursive: true, force: true}); }
        throw error;
      }
      finally { opening = false; }
    },
    revoke,
    cancel() { return revoke('Browser session cancelled'); },
    status() {
      let status = cleanupFailure ? 'cleanup_failed' : active?.policy.active ? 'authorized' : 'inactive', reason = cleanupFailure ?? lastReason;
      if (active) { try { verify(active, active.grant.project); } catch (error) { status = 'changed'; reason = error instanceof Error ? error.message : String(error); } }
      return {status, reason, ...(active ? {project: active.grant.project, release: active.grant.release, origin: active.grant.origin, peerPath: active.grant.peerPath, peerIdentity: active.grant.peerIdentity, browserPath: active.grant.browserPath, browserIdentity: active.grant.browserIdentity, expires: active.grant.expires, remainingCalls: active.policy.remaining, methods: active.grant.methods, tools: active.grant.tools, output: active.grant.output} : {}), scope: 'One literal loopback HTTP origin, GET/HEAD only; application proxy and observed process cleanup, not an OS sandbox'};
    },
  };
}
