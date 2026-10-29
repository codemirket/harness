import { createHash, randomUUID } from 'node:crypto';
import { constants, lstatSync, realpathSync, mkdirSync, writeFileSync, renameSync, rmSync, rmdirSync, openSync, closeSync, readSync, fstatSync, chmodSync, symlinkSync } from 'node:fs';
import { join, dirname, isAbsolute, resolve, posix } from 'node:path';
import { spawn } from 'node:child_process';
import { inflateRawSync } from 'node:zlib';
import { regularBytes, readBinding, type VerifiedRuntime } from './runtime.js';
export type ToolId = 'chromium';
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
    format?: 'zip';
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
}
interface Choices {
    schema: 1;
    releaseId: string;
    tools: Partial<Record<ToolId, ToolBinding>>;
}
export type Progress = (message: string) => void;
const ids = ['chromium'];
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
export function executableIdentity(file: string): string {
    if (!isAbsolute(file) || resolve(file) !== file || realpathSync(file) !== file)
        throw new Error('Select an absolute executable without symlink ancestry');
    const fd = openSync(file, constants.O_RDONLY | constants.O_NOFOLLOW);
    try {
        const before = fstatSync(fd);
        if (!before.isFile() || before.nlink !== 1 || before.size <= 0 || before.size > 512 * 1024 * 1024 || !(before.mode & 0o111))
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
    const binding = readBinding(packageRoot), bytes = regularBytes(join(packageRoot, 'runtime-setup.json'), 512 * 1024);
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
        if (tool.archive && (!hex(tool.archive.sha256) || !Number.isSafeInteger(tool.archive.bytes) || tool.archive.bytes <= 0 || tool.archive.bytes > 512 * 1024 * 1024 || !tool.archive.url.startsWith('https://')))
            throw new Error('Invalid pinned runtime archive');
    }
    return value.tools;
}
function readChoices(home: string, releaseId: string, reset = false): Choices {
    try {
        const value = JSON.parse(regularBytes(join(home, 'runtime-tools.json'), 65536).toString());
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
        child.on('close', code => { closed = true; clearTimeout(timer); signal.removeEventListener('abort', abort); if (failure)
            reject(failure);
        else if (code !== 0)
            reject(new Error(`Capability check failed (${code}): ${Buffer.concat(parts).toString('utf8').slice(-2048)}`));
        else
            resolve(Buffer.concat(parts).toString('utf8')); });
    });
}
export async function checkCapability(spec: ToolSpec, file: string, home: string, signal: AbortSignal): Promise<ToolBinding> {
    const before = executableIdentity(file), flag = '--version';
    const version = (await run(file, [flag], home, signal)).trim().split('\n')[0]!.slice(0, 256);
    if (spec.id === 'chromium') {
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
    if (executableIdentity(file) !== before)
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
            return { id: spec.id, status: 'missing', choice: binding?.choice ?? 'unconfigured', pinnedInstall: spec.archive ? 'available' : spec.unavailable ?? 'No pinned publisher archive is configured' };
        let status = 'ready';
        try {
            if (executableIdentity(binding.path!) !== binding.sha256)
                status = 'changed';
        }
        catch {
            status = 'changed';
        }
        return { id: spec.id, status, ...binding };
    });
}
export function skipRuntime(home: string, releaseId: string, id: ToolId): void { saveChoice(home, releaseId, id, { choice: 'skip' }); }
export async function bindRuntime(home: string, releaseId: string, spec: ToolSpec, file: string, signal: AbortSignal, progress: Progress): Promise<void> {
    privateHome(home);
    requirePlatform(spec);
    progress(`Checking ${spec.title} identity and capabilities`);
    const probe = join(home, `.runtime-probe-${randomUUID()}`);
    mkdirSync(probe, { mode: 0o700 });
    try {
        const binding = await checkCapability(spec, file, probe, signal);
        signal.throwIfAborted();
        saveChoice(home, releaseId, spec.id, binding);
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
export async function installRuntime(home: string, releaseId: string, spec: ToolSpec, signal: AbortSignal, progress: Progress, fetcher: typeof fetch = fetch): Promise<void> {
    privateHome(home);
    requirePlatform(spec);
    if (!spec.archive)
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
        const bytes = await download(spec.archive, signal, progress, fetcher);
        signal.throwIfAborted();
        progress('Verifying and extracting selected runtime files');
        if (spec.format !== 'zip')
            throw new Error('Unsupported pinned runtime archive');
        const executable = extractZip(bytes, stage, spec, signal);
        const binding = await checkCapability(spec, join(stage, executable), probe, signal);
        signal.throwIfAborted();
        renameSync(stage, final);
        published = true;
        saveChoice(home, releaseId, spec.id, { ...binding, choice: 'pinned', path: join(final, executable) });
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
        if (!binding || binding.choice === 'skip' || executableIdentity(binding.path!) !== binding.sha256)
            throw new Error(`${id} is missing or changed; run /mirket setup`);
    }
}
