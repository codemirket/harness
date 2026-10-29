import { createHash, randomUUID } from 'node:crypto';
import { constants, lstatSync, realpathSync, mkdirSync, writeFileSync, renameSync, rmSync, rmdirSync, openSync, closeSync, readSync, fstatSync, chmodSync, symlinkSync, mkdtempSync, readdirSync, readlinkSync } from 'node:fs';
import { join, dirname, isAbsolute, resolve, posix } from 'node:path';
import { spawn } from 'node:child_process';
import { inflateRawSync, gunzipSync } from 'node:zlib';
import { regularBytes, readBinding, type VerifiedRuntime } from './runtime.js';
export type ToolId = 'chromium' | 'browser-chromium' | 'playwright-mcp';
interface Archive {
    url: string;
    bytes: number;
    sha256: string;
}
interface ZipFile {
    path: string;
    bytes: number;
    sha256: string;
    mode: number;
}
export interface ToolSpec {
    id: ToolId;
    target: string;
    title: string;
    version: string;
    terms: string;
    archive?: Archive;
    format?: 'zip' | 'tar.gz';
    archives?: (Archive & { prefix: string })[];
    executable?: string;
    files?: ZipFile[];
    links?: {
        relative_path: string;
        target: string;
    }[];
    unavailable?: string;
}
export interface ToolBinding {
    choice: 'installed' | 'pinned' | 'skip';
    path?: string;
    sha256?: string;
    version?: string;
    tree?: { root: string; identity: string; entries: TreeEntry[] };
}
interface TreeEntry { path: string; kind: 'file' | 'directory' | 'link'; mode: number; sha256?: string; target?: string }
interface Choices {
    schema: 1;
    releaseId: string;
    tools: Partial<Record<ToolId, ToolBinding>>;
}
export type Progress = (message: string) => void;
const ids = ['chromium', 'browser-chromium', 'playwright-mcp'];
const sha = (data: Uint8Array) => createHash('sha256').update(data).digest('hex');
const hex = (value: unknown): value is string => typeof value === 'string' && /^[a-f0-9]{64}$/.test(value);
function safe(relative: string): boolean { return !!relative && !relative.includes('\\') && !relative.includes('\0') && !posix.isAbsolute(relative) && relative.split('/').every(p => p !== '.' && p !== '..' && p !== ''); }
function privateHome(home: string): void {
    const stat = lstatSync(home);
    if (!isAbsolute(home) || !stat.isDirectory() || stat.isSymbolicLink() || (stat.mode & 0o077) || realpathSync(home) !== home)
        throw new Error('Setup requires the active private Mirket home');
}
function requirePlatform(spec: ToolSpec): void {
    const target = process.platform === 'darwin' && process.arch === 'arm64' ? 'aarch64-apple-darwin' : undefined;
    if (!target || spec.target !== target) throw new Error(`Runtime target ${spec.target} does not match the qualified platform ${process.platform}/${process.arch}`);
}
export function executableIdentity(file: string, requireExecutable = true): string {
    if (!isAbsolute(file) || resolve(file) !== file || realpathSync(file) !== file)
        throw new Error('Select an absolute executable without symlink ancestry');
    const fd = openSync(file, constants.O_RDONLY | constants.O_NOFOLLOW);
    try {
        const before = fstatSync(fd);
        if (!before.isFile() || before.nlink !== 1 || before.size <= 0 || before.size > 512 * 1024 * 1024 || (requireExecutable && !(before.mode & 0o111)))
            throw new Error('Selected file is not a regular executable');
        const hash = createHash('sha256'), buffer = Buffer.alloc(1024 * 1024);
        let count;
        while ((count = readSync(fd, buffer, 0, buffer.length, null)))
            hash.update(buffer.subarray(0, count));
        const after = lstatSync(file);
        if (after.ino !== before.ino || after.dev !== before.dev || after.size !== before.size || after.mtimeMs !== before.mtimeMs || after.isSymbolicLink())
            throw new Error('Executable changed during verification');
        return hash.digest('hex');
    }
    finally {
        closeSync(fd);
    }
}
export function readRuntimeSpecs(packageRoot: string): ToolSpec[] {
    const binding = readBinding(packageRoot), bytes = regularBytes(join(packageRoot, 'runtime-setup.json'), 1024 * 1024);
    if (!binding.runtimeSetupSha256 || sha(bytes) !== binding.runtimeSetupSha256)
        throw new Error('Runtime setup descriptor identity changed');
    const value = JSON.parse(bytes.toString());
    if (value.schema !== 1 || !Array.isArray(value.tools) || value.tools.length > 4)
        throw new Error('Invalid runtime setup descriptor');
    const seen = new Set<string>();
    for (const tool of value.tools) {
        if (!ids.includes(tool.id) || seen.has(tool.id) || typeof tool.title !== 'string' || typeof tool.version !== 'string' || typeof tool.terms !== 'string' || typeof tool.target !== 'string')
            throw new Error('Invalid runtime tool');
        seen.add(tool.id);
        for (const archive of [...(tool.archive ? [tool.archive] : []), ...(tool.archives ?? [])]) {
            if (!hex(archive.sha256) || !Number.isSafeInteger(archive.bytes) || archive.bytes <= 0 || archive.bytes > 512 * 1024 * 1024 || typeof archive.url !== 'string' || !archive.url.startsWith('https://')) throw new Error('Invalid pinned runtime archive');
        }
        if (tool.archives && (!Array.isArray(tool.archives) || tool.archives.length > 4 || tool.archives.some((a: {prefix: string}) => !safe(a.prefix?.replace(/\/$/u, ''))))) throw new Error('Invalid pinned runtime archives');
        if (tool.files && (!Array.isArray(tool.files) || tool.files.length > 4096 || tool.files.some((f: ZipFile) => !safe(f.path) || !hex(f.sha256) || !Number.isSafeInteger(f.bytes) || f.bytes < 0 || ![0o600, 0o700, 0o644, 0o755].includes(f.mode)) || new Set(tool.files.map((f: ZipFile) => f.path)).size !== tool.files.length)) throw new Error('Invalid selected runtime members');
    }
    return value.tools;
}
function readChoices(home: string, releaseId: string, reset = false): Choices {
    try {
        const value = JSON.parse(regularBytes(join(home, 'runtime-tools.json'), 1024 * 1024).toString());
        if (value.schema !== 1 || typeof value.releaseId !== 'string' || !value.tools || typeof value.tools !== 'object' || Array.isArray(value.tools) || Object.keys(value.tools).some(k => !ids.includes(k)))
            throw new Error('Invalid runtime selection');
        // An update requires fresh choices. Preserve the old file until an explicit setup action.
        if (value.releaseId !== releaseId) {
            if (reset)
                return { schema: 1, releaseId, tools: {} };
            throw new Error('Runtime selection belongs to another release; run /mirket setup');
        }
        for (const binding of Object.values(value.tools) as ToolBinding[]) {
            if (!binding || !['skip', 'installed', 'pinned'].includes(binding.choice) || (binding.choice !== 'skip' && (typeof binding.path !== 'string' || !hex(binding.sha256) || typeof binding.version !== 'string')))
                throw new Error('Invalid runtime binding');
            if (binding.tree && (typeof binding.tree.root !== 'string' || !isAbsolute(binding.tree.root) || !hex(binding.tree.identity) || !Array.isArray(binding.tree.entries) || binding.tree.entries.length > 4096)) throw new Error('Invalid runtime tree binding');
        }
        return value;
    }
    catch (error) {
        if ((error as NodeJS.ErrnoException).code === 'ENOENT')
            return { schema: 1, releaseId, tools: {} };
        throw error;
    }
}
function saveChoice(home: string, releaseId: string, id: ToolId, binding: ToolBinding): void {
    privateHome(home);
    const lock = join(home, 'runtime-tools.lock');
    mkdirSync(lock, { mode: 0o700 });
    const next = join(home, `runtime-tools-${randomUUID()}.next`);
    try {
        const choices = readChoices(home, releaseId, true);
        choices.tools[id] = binding;
        writeFileSync(next, JSON.stringify(choices) + '\n', { flag: 'wx', mode: 0o600 });
        renameSync(next, join(home, 'runtime-tools.json'));
    }
    finally {
        rmSync(next, { force: true });
        rmdirSync(lock);
    }
}
function run(file: string, args: string[], home: string, signal: AbortSignal): Promise<string> {
    signal.throwIfAborted();
    return new Promise((resolve, reject) => {
        const child = spawn(file, args, { cwd: home, env: { HOME: home, TMPDIR: home, PATH: '', LANG: 'C', LC_ALL: 'C' }, detached: process.platform !== 'win32', stdio: ['ignore', 'pipe', 'pipe'] });
        const parts: Buffer[] = [];
        let size = 0, failure: Error | undefined, closed = false;
        const kill = (error: Error) => { failure ??= error; if (!closed && child.pid) {
            try {
                process.kill(process.platform === 'win32' ? child.pid : -child.pid, 'SIGKILL');
            }
            catch (error) {
                if ((error as NodeJS.ErrnoException).code !== 'ESRCH')
                    failure = new Error('Owned capability process cleanup failed');
            }
        } };
        const abort = () => kill(new Error('Runtime setup cancelled'));
        signal.addEventListener('abort', abort, { once: true });
        const timer = setTimeout(() => kill(new Error('Capability check timed out')), 20000);
        for (const stream of [child.stdout, child.stderr])
            stream.on('data', (data: Buffer) => { size += data.length; if (size > 512 * 1024)
                kill(new Error('Capability output exceeded its limit'));
            else
                parts.push(data); });
        child.on('error', error => { failure = error; });
        child.on('close', code => { closed = true; clearTimeout(timer); signal.removeEventListener('abort', abort);
        // Reap any owned same-group descendant even when the probe leader exited early.
        if (process.platform !== 'win32' && child.pid) {
            try { process.kill(-child.pid, 'SIGKILL'); }
            catch (error) { if ((error as NodeJS.ErrnoException).code !== 'ESRCH') failure = new Error('Owned capability process cleanup failed'); }
        }
        if (failure)
            reject(failure);
        else if (code !== 0)
            reject(new Error(`Capability check failed (${code}): ${Buffer.concat(parts).toString('utf8').slice(-2048)}`));
        else
            resolve(Buffer.concat(parts).toString('utf8')); });
    });
}
export async function checkCapability(spec: ToolSpec, file: string, home: string, signal: AbortSignal, node?: string, browser?: string): Promise<ToolBinding> {
    const before = executableIdentity(file, spec.id !== 'playwright-mcp'), flag = '--version';
    if (spec.id === 'playwright-mcp' && !node) throw new Error('Verified private Node is required');
    const version = (await run(node ?? file, node ? [file, flag] : [flag], home, signal)).trim().split('\n')[0]!.slice(0, 256);
    if (spec.id === 'chromium' || spec.id === 'browser-chromium') {
        if (!/Chrome|Chromium/i.test(version))
            throw new Error('Selected executable does not identify as Chrome or Chromium');
        if (version.match(/\b(\d+\.\d+\.\d+\.\d+)\b/u)?.[1] !== spec.version)
            throw new Error(`Chrome ${spec.version} is required; selected ${version}`);
        const page = join(home, 'probe.html');
        writeFileSync(page, '<!doctype html><div id="probe">pending</div><script>document.querySelector("#probe").textContent=["mirket","runtime","ready"].join("-")</script>', { flag: 'wx', mode: 0o600 });
        // Prevent unrelated background services and real keychain access in the fresh probe profile.
        // This also avoids the observed macOS headless teardown watchdog; clean exit remains required.
        const dom = await run(file, ['--headless', '--disable-gpu', '--use-mock-keychain', '--password-store=basic', '--disable-background-networking', '--disable-component-update', '--disable-sync', '--disable-extensions', '--no-first-run', '--no-default-browser-check', `--user-data-dir=${join(home, 'chrome-profile')}`, '--dump-dom', '--virtual-time-budget=1000', '--window-size=640,480', new URL('file://' + page).href], home, signal);
        if (!dom.includes('<div id="probe">mirket-runtime-ready</div>')) throw new Error('Chrome did not execute the local JavaScript capability probe');
    }
    if (spec.id === 'playwright-mcp') {
        if (!version.includes(spec.version)) throw new Error(`Playwright MCP ${spec.version} is required`);
        const help = await run(node!, [file, '--help'], home, signal);
        if (!help.includes('--executable-path') || !help.includes('--isolated') || !help.includes('--config')) throw new Error('Selected Playwright server lacks required configuration capabilities');
        if (!browser) throw new Error('Select the separate browser executable before the Playwright server');
        const sockets = mkdtempSync('/private/tmp/mk-pw-');
        chmodSync(sockets, 0o700);
        try {
            const result = await run(node!, ['--input-type=module', '-e', peerProbe, file, browser, sockets], home, signal);
            if (!result.includes('mirket-playwright-capability-ready')) throw new Error('Playwright browser capability probe did not complete');
        } finally { rmSync(sockets, { recursive: true, force: true }); }
    }
    if (executableIdentity(file, spec.id !== 'playwright-mcp') !== before)
        throw new Error('Selected executable changed during capability checks');
    return { choice: 'installed', path: file, sha256: before, version };
}
export function runtimeToolStatus(home: string, releaseId: string, specs: ToolSpec[]): Record<string, unknown>[] {
    let choices: Choices;
    try {
        choices = readChoices(home, releaseId);
    }
    catch (error) {
        return specs.map(spec => ({ id: spec.id, status: 'changed', error: String(error) }));
    }
    return specs.map(spec => {
        const binding = choices.tools[spec.id];
        if (!binding || binding.choice === 'skip')
            return { id: spec.id, status: 'missing', choice: binding?.choice ?? 'unconfigured', pinnedInstall: (spec.archive || spec.archives?.length) ? 'available' : spec.unavailable ?? 'No pinned publisher archive is configured' };
        let status = 'ready';
        try {
            if (bindingIdentity(binding, spec.id) !== binding.sha256)
                status = 'changed';
        }
        catch {
            status = 'changed';
        }
        const { tree, ...selection } = binding;
        return { id: spec.id, status, ...selection, ...(tree ? { tree: { root: tree.root, identity: tree.identity, members: tree.entries.length } } : {}) };
    });
}
export function skipRuntime(home: string, releaseId: string, id: ToolId): void { saveChoice(home, releaseId, id, { choice: 'skip' }); }
export async function bindRuntime(home: string, releaseId: string, spec: ToolSpec, file: string, signal: AbortSignal, progress: Progress, options: { treeRoot?: string; node?: string } = {}): Promise<void> {
    privateHome(home);
    requirePlatform(spec);
    progress(`Checking ${spec.title} identity and capabilities`);
    const probe = join(home, `.runtime-probe-${randomUUID()}`);
    mkdirSync(probe, { mode: 0o700 });
    try {
        const selected = spec.id === 'playwright-mcp' ? join(file, spec.executable!) : file;
        const tree = setupTree(spec, selected, spec.id === 'playwright-mcp' ? file : options.treeRoot);
        const binding = await checkCapability(spec, selected, probe, signal, options.node, spec.id === 'playwright-mcp' ? selectedBrowser(home, releaseId) : undefined);
        if (tree && treeSnapshot(tree.root).identity !== tree.identity) throw new Error('Runtime tree changed during capability check');
        signal.throwIfAborted();
        saveChoice(home, releaseId, spec.id, { ...binding, ...(tree ? { tree } : {}) });
    }
    finally {
        rmSync(probe, { recursive: true, force: true });
    }
}
async function download(archive: Archive, signal: AbortSignal, progress: Progress, fetcher: typeof fetch): Promise<Buffer> {
    const response = await fetcher(archive.url, { signal, redirect: 'error' });
    if (!response.ok || !response.body)
        throw new Error(`Pinned download failed (${response.status})`);
    const declared = response.headers.get('content-length');
    if (declared && Number(declared) !== archive.bytes)
        throw new Error('Pinned download size differs');
    const reader = response.body.getReader(), parts: Uint8Array[] = [];
    let bytes = 0, last = -1;
    try {
        for (;;) {
            signal.throwIfAborted();
            const chunk = await reader.read();
            if (chunk.done)
                break;
            bytes += chunk.value.length;
            if (bytes > archive.bytes)
                throw new Error('Pinned download exceeds its expected size');
            parts.push(chunk.value);
            const percent = Math.floor(bytes / archive.bytes * 100);
            if (percent !== last) {
                progress(`Downloading ${percent}% (${bytes}/${archive.bytes} bytes)`);
                last = percent;
            }
        }
    }
    finally {
        await reader.cancel().catch(() => { });
    }
    const result = Buffer.concat(parts);
    if (bytes !== archive.bytes || sha(result) !== archive.sha256)
        throw new Error('Pinned download SHA-256 or size mismatch');
    progress('Archive SHA-256 verified');
    return result;
}
function writeSelected(stage: string, name: string, bytes: Buffer, expected: {
    bytes: number;
    sha256: string;
    mode: number;
}): void {
    if (!safe(name) || bytes.length !== expected.bytes || sha(bytes) !== expected.sha256)
        throw new Error('Pinned archive member identity mismatch');
    const output = join(stage, name);
    mkdirSync(dirname(output), { recursive: true, mode: 0o700 });
    writeFileSync(output, bytes, { flag: 'wx', mode: expected.mode });
    chmodSync(output, expected.mode);
}
function extractZip(data: Buffer, stage: string, spec: ToolSpec, signal: AbortSignal): string {
    if (!spec.files?.length || !spec.executable)
        throw new Error('ZIP member inventory is missing');
    let eocd = -1;
    for (let i = data.length - 22; i >= Math.max(0, data.length - 65557); i--)
        if (data.readUInt32LE(i) === 0x06054b50) {
            eocd = i;
            break;
        }
    if (eocd < 0 || data.readUInt16LE(eocd + 4) || data.readUInt16LE(eocd + 6))
        throw new Error('Unsupported pinned ZIP structure');
    const files = new Map(spec.files.map(file => [file.path, file])), links = new Map((spec.links ?? []).map(link => [link.relative_path, link.target])), seen = new Set<string>();
    let offset = data.readUInt32LE(eocd + 16);
    for (let entry = 0; entry < data.readUInt16LE(eocd + 10); entry++) {
        signal.throwIfAborted();
        if (data.readUInt32LE(offset) !== 0x02014b50)
            throw new Error('Invalid ZIP directory');
        const flags = data.readUInt16LE(offset + 8), method = data.readUInt16LE(offset + 10), compressed = data.readUInt32LE(offset + 20), size = data.readUInt32LE(offset + 24), nameLength = data.readUInt16LE(offset + 28), extra = data.readUInt16LE(offset + 30), comment = data.readUInt16LE(offset + 32), local = data.readUInt32LE(offset + 42);
        const name = data.subarray(offset + 46, offset + 46 + nameLength).toString('utf8');
        offset += 46 + nameLength + extra + comment;
        if (!safe(name.replace(/\/$/u, '')) || flags & 1 || ![0, 8].includes(method) || size > 512 * 1024 * 1024 || seen.has(name))
            throw new Error('Unsafe pinned ZIP member');
        seen.add(name);
        if (name.endsWith('/'))
            continue;
        const relative = 'runtime/browser/' + name, file = files.get(relative), link = links.get(relative);
        if (!file && link === undefined)
            throw new Error(`Unselected ZIP member: ${name}`);
        if (data.readUInt32LE(local) !== 0x04034b50)
            throw new Error('Invalid ZIP local header');
        const start = local + 30 + data.readUInt16LE(local + 26) + data.readUInt16LE(local + 28);
        const compressedBytes = data.subarray(start, start + compressed);
        if (compressedBytes.length !== compressed)
            throw new Error('Truncated ZIP member');
        const bytes = method === 0 ? compressedBytes : inflateRawSync(compressedBytes, { maxOutputLength: Math.max(1, size) });
        if (file)
            writeSelected(stage, relative, bytes, file);
        else if (bytes.toString() !== link || bytes.length !== size)
            throw new Error('ZIP link identity mismatch');
    }
    for (const file of spec.files)
        if (!seen.has(file.path.replace(/^runtime\/browser\//u, '')))
            throw new Error('Pinned ZIP is incomplete');
    for (const [name, target] of links) {
        if (!seen.has(name.replace(/^runtime\/browser\//u, '')) || !safe(target) || !posix.normalize(posix.join(posix.dirname(name), target)).startsWith('runtime/browser/'))
            throw new Error('Unsafe pinned ZIP link');
        const output = join(stage, name);
        mkdirSync(dirname(output), { recursive: true, mode: 0o700 });
        symlinkSync(target, output);
    }
    return spec.executable;
}
export async function installRuntime(home: string, releaseId: string, spec: ToolSpec, signal: AbortSignal, progress: Progress, fetcher: typeof fetch = fetch, node?: string): Promise<void> {
    privateHome(home);
    requirePlatform(spec);
    if (!spec.archive && !spec.archives?.length)
        throw new Error(spec.unavailable ?? 'A pinned publisher archive is not configured for this tool');
    const parent = join(home, 'runtimes');
    try {
        mkdirSync(parent, { mode: 0o700 });
    }
    catch (error) {
        if ((error as NodeJS.ErrnoException).code !== 'EEXIST')
            throw error;
    }
    if (realpathSync(parent) !== parent || (lstatSync(parent).mode & 0o077))
        throw new Error('Runtime acquisition directory changed');
    const stage = join(parent, `.${spec.id}-${randomUUID()}.partial`), final = join(parent, `${spec.id}-${randomUUID()}`), probe = join(home, `.runtime-probe-${randomUUID()}`);
    mkdirSync(stage, { mode: 0o700 });
    mkdirSync(probe, { mode: 0o700 });
    let published = false;
    try {
        let executable: string;
        if (spec.format === 'zip' && spec.archive) {
            const bytes = await download(spec.archive, signal, progress, fetcher);
            progress('Verifying and extracting selected runtime files');
            executable = extractZip(bytes, stage, spec, signal);
        } else if (spec.format === 'tar.gz' && spec.archives?.length && spec.executable) {
            for (const archive of spec.archives) {
                const bytes = await download(archive, signal, progress, fetcher);
                extractTar(bytes, stage, spec, archive.prefix, signal);
            }
            executable = spec.executable;
        } else throw new Error('Unsupported pinned runtime archive');
        signal.throwIfAborted();
        const selected = join(stage, executable);
        const appEnd = selected.indexOf('.app/');
        const tree = setupTree(spec, selected, spec.id === 'playwright-mcp' ? stage : appEnd >= 0 ? selected.slice(0, appEnd + 4) : undefined, true);
        const binding = await checkCapability(spec, selected, probe, signal, node, spec.id === 'playwright-mcp' ? selectedBrowser(home, releaseId) : undefined);
        if (tree && treeSnapshot(tree.root).identity !== tree.identity) throw new Error('Runtime tree changed during capability check');
        signal.throwIfAborted();
        renameSync(stage, final);
        published = true;
        saveChoice(home, releaseId, spec.id, { ...binding, choice: 'pinned', path: join(final, executable), ...(tree ? {tree: {...tree, root: join(final, tree.root.slice(stage.length + 1))}} : {}) });
        progress(`${spec.title} ready`);
    }
    catch (error) {
        if (published)
            rmSync(final, { recursive: true, force: true });
        throw error;
    }
    finally {
        rmSync(stage, { recursive: true, force: true });
        rmSync(probe, { recursive: true, force: true });
    }
}
export function assertRuntimeTools(runtime: VerifiedRuntime, tools: ToolId[]): void {
    const choices = readChoices(runtime.home, runtime.binding.releaseId);
    for (const id of tools) {
        const binding = choices.tools[id];
        if (!binding || binding.choice === 'skip' || bindingIdentity(binding, id) !== binding.sha256)
            throw new Error(`${id} is missing or changed; run /mirket setup`);
    }
}

/** Snapshot every regular byte, directory, permission and contained symlink; never follows links while walking. */
function treeSnapshot(root: string): NonNullable<ToolBinding['tree']> {
    if (!isAbsolute(root) || resolve(root) !== root || realpathSync(root) !== root || !lstatSync(root).isDirectory()) throw new Error('Select an exact absolute real runtime tree directory');
    const entries: TreeEntry[] = [];
    let total = 0;
    const walk = (directory: string) => {
        for (const name of readdirSync(directory).sort()) {
            const file = join(directory, name), relative = file.slice(root.length + 1), stat = lstatSync(file);
            if (!safe(relative) || entries.length >= 4096) throw new Error('Runtime tree membership exceeds its limit');
            const mode = stat.mode & 0o777;
            if (stat.isSymbolicLink()) {
                const target = readlinkSync(file), resolved = realpathSync(file);
                if (isAbsolute(target) || !resolved.startsWith(root + '/')) throw new Error('Runtime tree link escapes the selected directory');
                entries.push({ path: relative, kind: 'link', mode, target });
            } else if (stat.isDirectory()) {
                entries.push({ path: relative, kind: 'directory', mode }); walk(file);
            } else if (stat.isFile()) {
                total += stat.size;
                if (total > 1024 * 1024 * 1024) throw new Error('Runtime tree exceeds its byte limit');
                entries.push({ path: relative, kind: 'file', mode, sha256: executableIdentity(file, false) });
            } else throw new Error('Runtime tree contains a special file');
        }
    };
    walk(root);
    return { root, entries, identity: sha(Buffer.from(JSON.stringify(entries))) };
}
function setupTree(spec: ToolSpec, executable: string, root?: string, exactOnly = true): ToolBinding['tree'] {
    if (spec.id === 'chromium') return undefined;
    if (!root || !executable.startsWith(root + '/')) throw new Error('Explicit runtime tree root must contain the selected executable');
    const tree = treeSnapshot(root);
    if (spec.id === 'playwright-mcp') {
        if (!spec.files?.length || executable !== join(root, spec.executable!)) throw new Error('Selected Playwright tree does not contain the pinned CLI');
        const allowed = new Set<string>();
        for (const file of spec.files) {
            if (!safe(file.path)) throw new Error('Unsafe selected peer path');
            let parent = posix.dirname(file.path);
            while (parent !== '.') { allowed.add(parent); parent = posix.dirname(parent); }
            allowed.add(file.path);
            const entry = tree.entries.find(entry => entry.path === file.path);
            if (!entry || entry.kind !== 'file' || entry.sha256 !== file.sha256 || lstatSync(join(root, file.path)).size !== file.bytes) throw new Error(`Pinned peer member changed: ${file.path}`);
        }
        if (exactOnly && tree.entries.some(entry => !allowed.has(entry.path) || entry.kind === 'link')) throw new Error('Selected peer tree has unselected members');
    }
    return tree;
}
function bindingIdentity(binding: ToolBinding, id: ToolId): string {
    if (id !== 'chromium' && !binding.tree) throw new Error('Runtime tree identity is missing');
    if (binding.tree && (!hex(binding.tree.identity) || treeSnapshot(binding.tree.root).identity !== binding.tree.identity || !binding.path?.startsWith(binding.tree.root + '/'))) throw new Error('Runtime tree identity changed');
    return executableIdentity(binding.path!, id !== 'playwright-mcp');
}
export interface BrowserRuntimeBindings { peerPath: string; peerIdentity: string; browserPath: string; browserIdentity: string }
/** Call immediately before each browser operation and launch; this does not authorize effects. */
export function runtimeBrowserBindings(runtime: VerifiedRuntime): BrowserRuntimeBindings {
    const choices = readChoices(runtime.home, runtime.binding.releaseId);
    const peer = choices.tools['playwright-mcp'], browser = choices.tools['browser-chromium'];
    if (!peer || !browser || peer.choice === 'skip' || browser.choice === 'skip') throw new Error('Optional browser server and separate browser are required; run /mirket setup');
    if (bindingIdentity(peer, 'playwright-mcp') !== peer.sha256 || bindingIdentity(browser, 'browser-chromium') !== browser.sha256) throw new Error('Browser runtime bytes changed; run /mirket setup');
    return { peerPath: peer.path!, peerIdentity: peer.tree!.identity, browserPath: browser.path!, browserIdentity: browser.tree!.identity };
}
export function privateRuntimeNode(runtime: VerifiedRuntime): string {
    const node = join(runtime.home, 'releases', `r-${runtime.binding.releaseId.slice(7)}`, 'runtime/node/bin/node');
    if (executableIdentity(node) !== runtime.binding.nodeSha256) throw new Error('Private Node identity changed');
    return node;
}
function extractTar(data: Buffer, stage: string, spec: ToolSpec, prefix: string, signal: AbortSignal): void {
    const expanded = gunzipSync(data, { maxOutputLength: 64 * 1024 * 1024 });
    const selected = new Map((spec.files ?? []).filter(file => file.path.startsWith(prefix)).map(file => [file.path, file]));
    if (!selected.size || !safe(prefix.replace(/\/$/u, ''))) throw new Error('Pinned TAR selection is missing');
    const seen = new Set<string>();
    let offset = 0;
    for (; offset + 512 <= expanded.length;) {
        signal.throwIfAborted();
        const header = expanded.subarray(offset, offset + 512);
        if (header.every(byte => byte === 0)) break;
        const string = (start: number, length: number) => header.subarray(start, start + length).toString('utf8').replace(/\0.*$/su, '');
        const name = string(0, 100), parent = string(345, 155), type = string(156, 1), sizeText = string(124, 12).trim();
        const size = /^[0-7]+$/u.test(sizeText) ? parseInt(sizeText, 8) : NaN;
        const checksumText = string(148, 8).trim(), checksum = /^[0-7]+$/u.test(checksumText) ? parseInt(checksumText, 8) : NaN;
        let sum = 0; for (let i = 0; i < 512; i++) sum += i >= 148 && i < 156 ? 32 : header[i]!;
        const full = parent ? parent + '/' + name : name;
        if (sum !== checksum || !safe(full.replace(/\/$/u, '')) || !Number.isSafeInteger(size) || size < 0 || size > 64 * 1024 * 1024 || offset + 512 + size > expanded.length || seen.has(full)) throw new Error('Unsafe pinned TAR member');
        seen.add(full);
        if (!['', '0', '5'].includes(type)) throw new Error('Unsupported pinned TAR member type');
        if (type !== '5') {
            if (!full.startsWith('package/')) throw new Error('Unexpected pinned TAR root');
            const relative = prefix + full.slice('package/'.length), expected = selected.get(relative);
            if (expected) { writeSelected(stage, relative, expanded.subarray(offset + 512, offset + 512 + size), expected); selected.delete(relative); }
        }
        offset += 512 + Math.ceil(size / 512) * 512;
    }
    if (selected.size) throw new Error('Pinned TAR selection is incomplete');
}

function selectedBrowser(home: string, releaseId: string): string {
    const browser = readChoices(home, releaseId).tools['browser-chromium'];
    if (!browser || browser.choice === 'skip' || bindingIdentity(browser, 'browser-chromium') !== browser.sha256) throw new Error('Configure the separate browser executable before the Playwright server');
    return browser.path!;
}
// Executed only after explicit setup consent, in the disposable capability directory.
// Native process-group cleanup in run() owns this probe and all its descendants.
const peerProbe = String.raw`
import http from 'node:http';
import {spawn} from 'node:child_process';
import {createInterface} from 'node:readline';
const [cli,browser,sockets]=process.argv.slice(1);
const server=http.createServer((_req,res)=>{res.setHeader('Content-Type','text/html');res.end('<!doctype html><title>Mirket capability</title><button id="probe">pending</button><script>document.querySelector("#probe").textContent="mirket-playwright-capability-ready"</script>');});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin='http://127.0.0.1:'+server.address().port;
const child=spawn(process.execPath,[cli,'--headless','--isolated','--executable-path',browser,'--allowed-origins',origin],{env:{...process.env,PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD:'1',PWTEST_SOCKETS_DIR:sockets},stdio:['pipe','pipe','pipe']});
const stopped=new Promise(resolve=>child.once('close',resolve));
let sequence=0;const pending=new Map();let stderr='';
child.stderr.on('data',data=>{stderr=(stderr+data).slice(-2048);});
child.on('error',error=>{for(const item of pending.values()) item.reject(error);});
child.on('close',()=>{for(const item of pending.values()) item.reject(new Error('Peer closed: '+stderr));});
createInterface({input:child.stdout}).on('line',line=>{let message;try{message=JSON.parse(line);}catch{return;}const item=pending.get(message.id);if(item){pending.delete(message.id);message.error?item.reject(new Error(JSON.stringify(message.error))):item.resolve(message.result);}});
const request=(method,params)=>new Promise((resolve,reject)=>{const id=++sequence;pending.set(id,{resolve,reject});child.stdin.write(JSON.stringify({jsonrpc:'2.0',id,method,params})+'\n');});
try {
 await request('initialize',{protocolVersion:'2024-11-05',capabilities:{},clientInfo:{name:'mirket-setup',version:'1'}});
 child.stdin.write(JSON.stringify({jsonrpc:'2.0',method:'notifications/initialized'})+'\n');
 const navigation=await request('tools/call',{name:'browser_navigate',arguments:{url:origin}});
 if(navigation.isError) throw new Error(JSON.stringify(navigation));
 const snapshot=await request('tools/call',{name:'browser_snapshot',arguments:{}});
 if(snapshot.isError||!JSON.stringify(snapshot).includes('mirket-playwright-capability-ready')) throw new Error('Browser JavaScript/snapshot capability failed');
 const close=await request('tools/call',{name:'browser_close',arguments:{}});
 if(close.isError) throw new Error('Browser close capability failed');
 child.stdin.end();
 const code=await stopped;if(code!==0)throw new Error('Peer teardown failed: '+code);
 console.log('mirket-playwright-capability-ready');
} finally { child.kill('SIGKILL');server.closeAllConnections();await new Promise(resolve=>server.close(resolve)); }
`;
