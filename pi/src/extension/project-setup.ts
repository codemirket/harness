// Explicit project onboarding authorizes only this bounded source-guidance
// section. Pi settings remain the sole skill-selection source; observations do
// not provision capabilities or establish qualification. These path checks and
// cooperating locks do not contain hostile concurrent filesystem replacement.
import {createHash, randomUUID} from 'node:crypto';
import {lstatSync, mkdirSync, realpathSync, renameSync, rmdirSync, unlinkSync, writeFileSync} from 'node:fs';
import {dirname, isAbsolute, join, resolve} from 'node:path';
import {TextDecoder} from 'node:util';
import {MIRKET_TOOL_CLASSES} from './authorization.js';
import {BROWSER_TOOLS} from './browser-policy.js';
import {prepareProfile, applyProfile, readProjectProfile} from './profiles.js';
import {regularBytes, type VerifiedRuntime} from './runtime.js';

const START = '<!-- mirket-project:start -->', END = '<!-- mirket-project:end -->';
const DATA = '<!-- mirket-project-data ', MAX_SOURCE = 1024 * 1024, MAX_STATE = 65536;
const hash = (value: string | Buffer) => createHash('sha256').update(value).digest('hex');
interface Uses {schema: 1; skills: string[]; tools: string[]}
interface Consent {schema: 1; project: string; rootIdentity: string; releaseId: string; packageRoot: string; sectionSha256: string}
interface Source {bytes: Buffer | undefined; text: string; section: string | undefined; uses: Uses; newline: string; mode: number}
export interface ProjectSyncStatus {status: 'disabled' | 'ready' | 'changed'; project: string; releaseId: string; guidanceFile?: string; error?: string}
export interface ProjectInitialization {project: string; guidanceFile: string; settingsFile: string; selectedSkills: string[] | null; changed: boolean; optedIn: true}

function directory(path: string, privateMode = false): string {
  if (!isAbsolute(path) || resolve(path) !== path || realpathSync(path) !== path) throw new Error('Project setup requires canonical absolute directories');
  const stat = lstatSync(path);
  if (!stat.isDirectory() || stat.isSymbolicLink() || (privateMode && ((stat.mode & 0o077) !== 0 || (process.getuid && stat.uid !== process.getuid())))) throw new Error('Project setup state must be a private owned directory');
  return hash(JSON.stringify([stat.dev, stat.ino, stat.birthtimeMs]));
}
function paths(runtime: VerifiedRuntime) {
  const identity = directory(runtime.cwd);
  directory(runtime.home, true);
  if (typeof runtime.binding.releaseId !== 'string' || !runtime.binding.releaseId.length || runtime.binding.releaseId.length > 256) throw new Error('Missing project runtime release');
  const state = join(runtime.home, 'project-setup');
  return {identity, state, consent: join(state, hash(runtime.cwd) + '.json'), guidance: join(runtime.cwd, 'AGENTS.md')};
}
function optionalBytes(path: string, max: number): Buffer | undefined {
  try { return regularBytes(path, max); }
  catch (error) { if ((error as NodeJS.ErrnoException).code === 'ENOENT') return undefined; throw error; }
}
function text(bytes: Buffer): string {return new TextDecoder('utf-8', {fatal: true, ignoreBOM: true}).decode(bytes);}
function ids(value: unknown): value is string[] {
  return Array.isArray(value) && value.length <= 256 && value.every(id => typeof id === 'string' && /^[a-z][a-z0-9_-]*(?:\.[a-z0-9_-]+)*$/u.test(id) && id.length <= 160) && new Set(value).size === value.length;
}
function source(file: string): Source {
  const bytes = optionalBytes(file, MAX_SOURCE), original = bytes === undefined ? '' : text(bytes);
  const newline = original.includes('\r\n') ? '\r\n' : '\n';
  const starts = original.split(START).length - 1, ends = original.split(END).length - 1;
  if (starts > 1 || ends > 1 || starts !== ends || original.split('<!-- mirket-project:').length - 1 !== starts + ends || original.split(DATA).length - 1 !== starts) throw new Error('Malformed or duplicate Mirket project markers');
  let section: string | undefined, uses: Uses = {schema: 1, skills: [], tools: []};
  if (starts) {
    const begin = original.indexOf(START), end = original.indexOf(END);
    if (end < begin) throw new Error('Malformed Mirket project markers');
    section = original.slice(begin, end + END.length);
    const lines = section.split(/\r?\n/u), data = lines.filter(line => line.startsWith(DATA));
    if (data.length !== 1 || !data[0]!.endsWith(' -->')) throw new Error('Malformed Mirket project data');
    const parsed = JSON.parse(data[0]!.slice(DATA.length, -4)) as Uses;
    if (!parsed || parsed.schema !== 1 || Object.keys(parsed).some(key => !['schema', 'skills', 'tools'].includes(key)) || !ids(parsed.skills) || !ids(parsed.tools)) throw new Error('Invalid Mirket project data');
    uses = parsed;
  } else if (original.includes(DATA)) throw new Error('Orphaned Mirket project data');
  return {bytes, text: original, section, uses, newline, mode: bytes === undefined ? 0o600 : lstatSync(file).mode & 0o777};
}
function render(uses: Uses, newline: string): string {
  return [START, DATA + JSON.stringify(uses) + ' -->', '## Mirket project setup', '',
    'Pi owns models, authentication, sessions, native tools and MCP lifecycle. Mirket owns explicit scoped authorization for its operations.',
    'The exact IDs below record observed use, not current readiness or qualified output. Recheck the running release and required capability with mirket_doctor after setup, reconnect or a relevant change.',
    'Use project toolchains for application builds, tests and format-specific exports. Skill guidance does not authorize dependency installation, account setup or restricted effects.', '',
    'Observed skill IDs: ' + (uses.skills.length ? uses.skills.map(id => '`' + id + '`').join(', ') : 'none yet') + '.',
    'Observed tool IDs: ' + (uses.tools.length ? uses.tools.map(id => '`' + id + '`').join(', ') : 'none yet') + '.', END].join(newline);
}
function nextSource(previous: Source, section: string): string {
  if (previous.section !== undefined) return previous.text.replace(previous.section, () => section);
  const separator = !previous.text.length ? '' : previous.text.endsWith(previous.newline + previous.newline) ? '' : previous.text.endsWith(previous.newline) ? previous.newline : previous.newline + previous.newline;
  return previous.text + separator + section + previous.newline;
}
function sameBytes(a: Buffer | undefined, b: Buffer | undefined): boolean {return a === undefined ? b === undefined : b !== undefined && a.equals(b);}
function replace(file: string, expected: Buffer | undefined, next: string, mode: number, max: number): boolean {
  if (Buffer.byteLength(next) > max) throw new Error('Project setup output exceeds its byte bound');
  if (expected?.equals(Buffer.from(next))) return false;
  directory(dirname(file));
  const temporary = file + '.mirket-' + randomUUID() + '.next';
  let created = false;
  try {
    writeFileSync(temporary, next, {flag: 'wx', mode}); created = true;
    if (!sameBytes(expected, optionalBytes(file, max))) throw new Error('Project setup file changed during update');
    renameSync(temporary, file); created = false;
    return true;
  } finally {if (created) unlinkSync(temporary);}
}
function readConsent(runtime: VerifiedRuntime, selected: ReturnType<typeof paths>): {value: Consent; bytes: Buffer} | undefined {
  try {directory(selected.state, true);} catch (error) {if ((error as NodeJS.ErrnoException).code === 'ENOENT') return undefined; throw error;}
  const bytes = optionalBytes(selected.consent, MAX_STATE);
  if (!bytes) return undefined;
  const value = JSON.parse(text(bytes)) as Consent;
  if (!value || value.schema !== 1 || Object.keys(value).some(key => !['schema', 'project', 'rootIdentity', 'releaseId', 'packageRoot', 'sectionSha256'].includes(key)) || value.project !== runtime.cwd || value.rootIdentity !== selected.identity || typeof value.packageRoot !== 'string' || !/^[a-f0-9]{64}$/u.test(value.sectionSha256)) throw new Error('Project setup consent identity changed');
  directory(value.packageRoot);
  return {value, bytes};
}
function checkedSource(runtime: VerifiedRuntime, selected: ReturnType<typeof paths>, consent: Consent): Source {
  if (consent.releaseId !== runtime.binding.releaseId) throw new Error('Project setup consent belongs to another release; run /mirket-project init');
  readProjectProfile(consent.packageRoot, join(runtime.cwd, '.pi/settings.json'));
  const current = source(selected.guidance);
  if (current.section === undefined || hash(current.section) !== consent.sectionSha256) throw new Error('Mirket project section changed; inspect it before initializing again');
  return current;
}
function withLock<T>(selected: ReturnType<typeof paths>, action: () => T): T {
  try {mkdirSync(selected.state, {mode: 0o700});} catch (error) {if ((error as NodeJS.ErrnoException).code !== 'EEXIST') throw error;}
  directory(selected.state, true);
  const lock = selected.consent + '.lock';
  mkdirSync(lock, {mode: 0o700});
  try {return action();} finally {rmdirSync(lock);}
}
function catalogSkills(packageRoot: string): Set<string> {
  directory(packageRoot);
  directory(join(packageRoot, 'skills'));
  const catalog = JSON.parse(text(regularBytes(join(packageRoot, 'skills/catalog.json'), 2 * 1024 * 1024))) as {global: {id: string}[]; project: {id: string}[]};
  const values = [...catalog.global, ...catalog.project].map(entry => entry.id);
  if (!ids(values)) throw new Error('Invalid project skill catalog');
  return new Set(values);
}
function verifyUse(packageRoot: string, use: {kind: 'skill' | 'tool'; id: string}): void {
  if (!use || !['skill', 'tool'].includes(use.kind) || !ids([use.id])) throw new Error('Invalid project capability observation');
  if (use.kind === 'skill') {
    if (!catalogSkills(packageRoot).has(use.id)) throw new Error('Unknown Mirket skill observation');
  } else if (!Object.hasOwn(MIRKET_TOOL_CLASSES, use.id) && !['mirket_doctor', 'mirket_delegate', 'mirket_task_status', ...BROWSER_TOOLS.map(id => 'mcp__mirket_browser__' + id)].includes(use.id)) throw new Error('Unknown Mirket tool observation');
}

export function initializeProject(packageRoot: string, runtime: VerifiedRuntime, profiles: string[], skills: string[]): ProjectInitialization {
  if (process.env.MIRKET_MANAGED_CHILD) throw new Error('Managed children cannot initialize project setup');
  const selected = paths(runtime);
  directory(packageRoot);
  const available = catalogSkills(packageRoot);
  return withLock(selected, () => {
    const existing = readConsent(runtime, selected), previous = source(selected.guidance);
    if (existing && (previous.section === undefined || hash(previous.section) !== existing.value.sectionSha256)) throw new Error('Mirket project section changed; inspect it before initializing again');
    for (const id of previous.uses.skills) if (!available.has(id)) throw new Error('Project source refers to an unknown Mirket skill');
    for (const id of previous.uses.tools) verifyUse(packageRoot, {kind: 'tool', id});
    const section = render(previous.uses, previous.newline), next = nextSource(previous, section);
    if (Buffer.byteLength(next) > MAX_SOURCE) throw new Error('Project setup output exceeds its byte bound');
    const settingsFile = join(runtime.cwd, '.pi/settings.json');
    try {directory(dirname(settingsFile));} catch (error) {if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error;}
    const edit = prepareProfile(packageRoot, settingsFile, profiles, skills);
    if (!profiles.length && !skills.length) edit.preserveSelection = true;
    edit.ensureExtension = true;
    const profile = applyProfile(edit);
    if (directory(runtime.cwd) !== selected.identity) throw new Error('Project identity changed during setup');
    const changed = replace(selected.guidance, previous.bytes, next, previous.mode, MAX_SOURCE);
    const consent: Consent = {schema: 1, project: runtime.cwd, rootIdentity: selected.identity, releaseId: runtime.binding.releaseId, packageRoot, sectionSha256: hash(section)};
    replace(selected.consent, existing?.bytes, JSON.stringify(consent) + '\n', 0o600, MAX_STATE);
    return {project: runtime.cwd, guidanceFile: selected.guidance, settingsFile, selectedSkills: profile.skills, changed: changed || profile.changed, optedIn: true};
  });
}

export function recordProjectUse(runtime: VerifiedRuntime, use: {kind: 'skill' | 'tool'; id: string}): boolean {
  if (process.env.MIRKET_MANAGED_CHILD) return false;
  const selected = paths(runtime), first = readConsent(runtime, selected);
  if (!first) return false;
  return withLock(selected, () => {
    const consent = readConsent(runtime, selected);
    if (!consent) throw new Error('Project setup consent disappeared');
    const previous = checkedSource(runtime, selected, consent.value);
    verifyUse(consent.value.packageRoot, use);
    const list = use.kind === 'skill' ? previous.uses.skills : previous.uses.tools;
    if (list.includes(use.id)) return false;
    if (list.length >= 256) throw new Error('Project observation limit reached');
    list.push(use.id); list.sort();
    const section = render(previous.uses, previous.newline);
    if (directory(runtime.cwd) !== selected.identity) throw new Error('Project identity changed during observation');
    replace(selected.guidance, previous.bytes, nextSource(previous, section), previous.mode, MAX_SOURCE);
    replace(selected.consent, consent.bytes, JSON.stringify({...consent.value, sectionSha256: hash(section)}) + '\n', 0o600, MAX_STATE);
    return true;
  });
}

export function projectSyncStatus(runtime: VerifiedRuntime): ProjectSyncStatus {
  const result = {project: runtime.cwd, releaseId: runtime.binding.releaseId};
  try {
    const selected = paths(runtime), consent = readConsent(runtime, selected);
    if (!consent) return {...result, status: 'disabled'};
    checkedSource(runtime, selected, consent.value);
    return {...result, status: 'ready', guidanceFile: selected.guidance};
  } catch (error) {return {...result, status: 'changed', error: error instanceof Error ? error.message : String(error)};}
}

/** Refresh only the source explicitly enrolled by this project and release. */
export function readProjectGuidance(runtime: VerifiedRuntime): {file: string; content: string} | undefined {
  const selected = paths(runtime), consent = readConsent(runtime, selected);
  if (!consent) return undefined;
  const current = checkedSource(runtime, selected, consent.value);
  return {file: selected.guidance, content: current.text};
}
