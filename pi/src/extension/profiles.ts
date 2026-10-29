import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync, renameSync, unlinkSync, rmdirSync, existsSync, lstatSync, realpathSync } from 'node:fs';
import { dirname, isAbsolute, join, resolve } from 'node:path';
import { homedir } from 'node:os';
import { regularBytes } from './runtime.js';
interface Skill { id: string }
interface Catalog { global: Skill[]; project: Skill[]; profiles: Record<string, {skills: string[]; conditional: string[]}> }
export interface ProfileEdit { file: string; expected: string; packageRoot: string; skills: string[] }
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
export function applyProfile(edit: ProfileEdit): void {
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
      // Pi resolves local package entries relative to the settings directory,
      // including entries written by its own package manager.
      const local = value === '~' ? homedir() : value.startsWith('~/') ? join(homedir(), value.slice(2)) : resolve(parent, value);
      let canonical = local;
      try { canonical = realpathSync(local); }
      catch (error) { if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error; }
      return canonical === edit.packageRoot ? [index] : [];
    });
    if (matches.length > 1) throw new Error('Duplicate Mirket package objects');
    const index = matches[0];
    const existing = index === undefined ? {} : packages[index];
    const replacement = {...(typeof existing === 'object' ? existing : {}), source: edit.packageRoot, skills: edit.skills};
    if (index === undefined) packages.push(replacement); else packages[index] = replacement;
    settings.packages = packages;
    writeFileSync(temporary, JSON.stringify(settings, null, 2) + '\n', {flag:'wx',mode:0o600});created = true;
    // Lock-cooperating pi edits are excluded. Refuse a non-cooperating write
    // observed before replacement; arbitrary hostile writers are not sandboxed.
    if (digest(read(edit.file)) !== edit.expected) throw new Error('Settings changed during profile edit');
    renameSync(temporary, edit.file);created = false;
  } finally { if (created) unlinkSync(temporary); rmdirSync(lock); }
}
