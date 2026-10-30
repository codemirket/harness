import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync, renameSync, unlinkSync, rmdirSync, existsSync, lstatSync, realpathSync } from 'node:fs';
import { dirname, isAbsolute, join, resolve } from 'node:path';
import { homedir } from 'node:os';
import { regularBytes } from './runtime.js';
interface Skill { id: string }
interface Catalog { global: Skill[]; project: Skill[]; profiles: Record<string, {skills: string[]; conditional: string[]}> }
export interface ProfileEdit { file: string; expected: string; packageRoot: string; skills: string[]; preserveSelection?: boolean; ensureExtension?: boolean }
export interface AppliedProfile { skills: string[] | null; changed: boolean }
const PROJECT_EXTENSION = '+./dist/extension/index.js';
const digest = (value: string | undefined) => createHash('sha256').update(value === undefined ? 'missing' : `present:${value}`).digest('hex');
export function profileSelection(catalog: Catalog, names: string[], ids: string[]): string[] {
  const available = new Set([...catalog.global, ...catalog.project].map(s => s.id));
  const selected = new Set(catalog.global.map(s => s.id));
  for (const name of names) {
    if (!Object.hasOwn(catalog.profiles, name)) throw new Error(`Unknown profile: ${name}`);
    for (const id of catalog.profiles[name]!.skills) selected.add(id);
  }
  for (const id of ids) selected.add(id);
  for (const id of selected) if (!available.has(id)) throw new Error(`Unknown skill: ${id}`);
  return [...selected].sort().map(id => `skills/${id}/SKILL.md`);
}
function read(file: string): string | undefined {
  try { return regularBytes(file, 2 * 1024 * 1024).toString(); }
  catch (error) { if ((error as NodeJS.ErrnoException).code === 'ENOENT') return undefined; throw error; }
}
export function prepareProfile(packageRoot: string, file: string, profiles: string[], skills: string[]): ProfileEdit {
  if (!isAbsolute(file) || !isAbsolute(packageRoot)) throw new Error('Profile paths must be absolute');
  const catalog = JSON.parse(regularBytes(join(packageRoot, 'skills/catalog.json'), 2 * 1024 * 1024).toString()) as Catalog;
  return {file, packageRoot: realpathSync(packageRoot), expected: digest(read(file)), skills: profileSelection(catalog, profiles, skills)};
}
function packageSettings(original: string | undefined, packageRoot: string, parent: string) {
  const settings = JSON.parse(original ?? '{}') as Record<string, unknown>;
  if (!settings || Array.isArray(settings) || typeof settings !== 'object') throw new Error('Invalid pi settings');
  const packages = settings.packages ?? [];
  if (!Array.isArray(packages)) throw new Error('Invalid pi package settings');
  const matches = packages.flatMap((entry, index) => {
    const source = typeof entry === 'string' ? entry : entry?.source;
    if (typeof source !== 'string') throw new Error('Invalid package source');
    const value = source.trim();
    if (/^npm:@codemirket\/pi-harness(?:@|$)/.test(value)) throw new Error('Conflicting registry package ownership');
    if (/^[a-z]+:|^git@/i.test(value)) return [];
    // Pi resolves local entries relative to the settings directory.
    const local = value === '~' ? homedir() : value.startsWith('~/') ? join(homedir(), value.slice(2)) : resolve(parent, value);
    let canonical = local;
    try { canonical = realpathSync(local); }
    catch (error) { if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error; }
    return canonical === packageRoot ? [index] : [];
  });
  if (matches.length > 1) throw new Error('Duplicate Mirket package objects');
  const index = matches[0];
  return {settings, packages, index, existing: index === undefined ? {} : packages[index]};
}
function validateFilter(filter: unknown): string[] | null {
  if (filter === undefined) return null;
  if (!Array.isArray(filter) || filter.some(value => typeof value !== 'string')) throw new Error('Invalid Mirket skill filter');
  return filter;
}
/** Inspect active project ownership without creating directories or rewriting settings. */
export function readProjectProfile(packageRoot: string, file: string): string[] | null {
  if (!isAbsolute(file) || !isAbsolute(packageRoot)) throw new Error('Profile paths must be absolute');
  const parent = dirname(file);
  if (realpathSync(parent) !== resolve(parent) || !lstatSync(parent).isDirectory()) throw new Error('Aliased settings parent');
  const {index, existing} = packageSettings(read(file), packageRoot, parent);
  if (index === undefined || !existing || typeof existing !== 'object' || Array.isArray(existing)) throw new Error('Project Mirket package object missing or changed');
  // Init pins one exact force-include, which works for normal filters and Pi's
  // autoload:false delta. Do not recreate Pi's glob matcher here or attest a
  // subsequently ambiguous filter. An omitted normal filter loads the manifest.
  const extensions = existing.extensions;
  const exactPaths = ['dist/extension/index.js', './dist/extension/index.js', join(packageRoot, 'dist/extension/index.js')];
  const exact = Array.isArray(extensions) && extensions.length === 1 && typeof extensions[0] === 'string'
    && (extensions[0].startsWith('+') ? exactPaths.includes(extensions[0].slice(1)) : exactPaths.filter(value => !value.startsWith('./')).includes(extensions[0]));
  if (!(extensions === undefined && existing.autoload !== false) && !exact) throw new Error('Project Mirket extension filter changed or disabled; run /mirket-project init');
  return validateFilter(existing.skills);
}
export function applyProfile(edit: ProfileEdit): AppliedProfile {
  const parent = dirname(edit.file);
  // Do not follow a project-controlled settings directory or non-regular file.
  if (!existsSync(parent)) mkdirSync(parent, {mode: 0o700});
  if (realpathSync(parent) !== resolve(parent) || !lstatSync(parent).isDirectory()) throw new Error('Aliased settings parent');
  const lock = `${edit.file}.lock`, temporary = `${edit.file}.mirket-${process.pid}.next`;
  mkdirSync(lock, {mode: 0o700}); // Shares pi's lock-directory exclusion; busy means refuse.
  let created = false;
  try {
    const original = read(edit.file);
    if (digest(original) !== edit.expected) throw new Error('Settings changed; recompute the profile selection');
    const {settings, packages, index, existing} = packageSettings(original, edit.packageRoot, parent);
    const preserve = edit.preserveSelection && index !== undefined;
    const filter = preserve && typeof existing === 'object' ? existing.skills : preserve ? undefined : edit.skills;
    validateFilter(filter);
    const replacement = {...(typeof existing === 'object' ? existing : {}), source: edit.packageRoot, ...(filter === undefined ? {} : {skills: filter}), ...(edit.ensureExtension ? {extensions: [PROJECT_EXTENSION]} : {})};
    if (index !== undefined && JSON.stringify(existing) === JSON.stringify(replacement)) return {skills: filter ?? null, changed: false};
    if (index === undefined) packages.push(replacement); else packages[index] = replacement;
    settings.packages = packages;
    writeFileSync(temporary, JSON.stringify(settings, null, 2) + '\n', {flag:'wx',mode:0o600});created = true;
    // Lock-cooperating pi edits are excluded. Refuse a non-cooperating write
    // observed before replacement; arbitrary hostile writers are not sandboxed.
    if (digest(read(edit.file)) !== edit.expected) throw new Error('Settings changed during profile edit');
    renameSync(temporary, edit.file);created = false;
    return {skills: filter ?? null, changed: true};
  } finally { if (created) unlinkSync(temporary); rmdirSync(lock); }
}
