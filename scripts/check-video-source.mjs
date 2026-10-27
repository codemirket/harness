#!/usr/bin/env node
// Static authoring check for one explicit video source manifest.
// This imports the compiled adapter contract; it does not copy, write, spawn,
// render, browse, or scan a project tree.
import { lstat, realpath } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const COVERAGE = 'static_only; no_render_no_pixels_no_audio_playback';
const SHA = /^sha256:[a-f0-9]{64}$/;
const PROFILE_KEYS = ['width', 'height', 'fps', 'frame_count', 'format', 'captions_path'];
const FPS_KEYS = ['numerator', 'denominator'];
const SOURCE_KEYS = ['path', 'revision', 'bytes'];
const MANIFEST_KEYS = ['schema_version', 'entry_file', 'profile', 'sources'];

const object = value => value !== null && typeof value === 'object' && !Array.isArray(value);
const exact = (value, keys) => object(value) && Object.keys(value).length === keys.length
  && keys.every(key => Object.hasOwn(value, key));
const boundedText = value => typeof value === 'string' && value.length > 0 && value.length <= 1024;
const relative = value => boundedText(value) && !/[\\:\0]/.test(value)
  && value.split('/').every(part => part !== '' && part !== '.' && part !== '..');
const code = value => typeof value === 'string' && /^[a-zA-Z0-9_]{1,80}$/.test(value)
  ? value : 'static_check_refused';

class StaticCheckError extends Error {
  constructor(publicCode) {
    super(publicCode);
    this.publicCode = publicCode;
  }
}

async function regularCanonical(file, publicCode) {
  try {
    const canonical = await realpath(file);
    const metadata = await lstat(file);
    if (!metadata.isFile() || metadata.isSymbolicLink() || canonical !== path.resolve(file)) throw new Error();
    return canonical;
  } catch {
    throw new StaticCheckError(publicCode);
  }
}

async function canonicalProject(value) {
  if (typeof value !== 'string' || !path.isAbsolute(value)) throw new StaticCheckError('invalid_project');
  try {
    const canonical = await realpath(value);
    const metadata = await lstat(value);
    if (!metadata.isDirectory() || metadata.isSymbolicLink() || canonical !== path.resolve(value)) throw new Error();
    return canonical;
  } catch {
    throw new StaticCheckError('invalid_project');
  }
}

function parseManifest(value) {
  if (!exact(value, MANIFEST_KEYS) || value.schema_version !== 1
    || !relative(value.entry_file) || !exact(value.profile, PROFILE_KEYS)
    || !exact(value.profile.fps, FPS_KEYS) || !Array.isArray(value.sources)
    || value.sources.length < 1 || value.sources.length > 128) {
    throw new StaticCheckError('invalid_manifest');
  }
  const seen = new Set();
  for (const source of value.sources) {
    if (!exact(source, SOURCE_KEYS) || !relative(source.path) || seen.has(source.path)
      || !SHA.test(source.revision) || !Number.isSafeInteger(source.bytes)
      || source.bytes < 0 || source.bytes > 16 * 1024 * 1024) {
      throw new StaticCheckError('invalid_manifest_sources');
    }
    seen.add(source.path);
  }
  if (!seen.has(value.entry_file)
    || (value.profile.captions_path !== null && (!relative(value.profile.captions_path)
      || !seen.has(value.profile.captions_path)))) {
    throw new StaticCheckError('invalid_manifest_sources');
  }
  return value;
}

async function adapter() {
  try {
    return await import(pathToFileURL(fileURLToPath(new URL('../adapters/dist/video-export.js', import.meta.url))).href);
  } catch {
    throw new StaticCheckError('adapter_unavailable');
  }
}

function diagnostic(publicCode, phase = 'static') {
  return [{ phase, code: code(publicCode) }];
}

function adapterDiagnostics(value) {
  if (!Array.isArray(value) || !value.length) return undefined;
  const result = [];
  for (const item of value) {
    if (result.length >= 16 || !object(item)) break;
    const allowed = ['phase', 'path', 'field', 'reason', 'expected', 'cue_index'];
    if (Object.keys(item).some(key => !allowed.includes(key))
      || !['source', 'reference', 'root', 'font', 'caption'].includes(item.phase)
      || !relative(item.path) || !/^[A-Za-z0-9_.-]{1,64}$/.test(item.field)
      || !/^[A-Za-z0-9_.-]{1,100}$/.test(item.reason)
      || typeof item.expected !== 'string' || item.expected.length < 1 || item.expected.length > 256
      || /[\u0000-\u001f\u007f]/.test(item.expected) || Buffer.byteLength(item.expected, 'utf8') > 256
      || (item.cue_index !== undefined && item.phase !== 'caption')
      || (item.cue_index !== undefined && (!Number.isSafeInteger(item.cue_index) || item.cue_index < 0 || item.cue_index > 999))) continue;
    const next = { phase: item.phase, path: item.path, field: item.field, reason: item.reason,
      expected: item.expected, ...(item.cue_index === undefined ? {} : { cue_index: item.cue_index }) };
    if (Buffer.byteLength(JSON.stringify([...result, next]), 'utf8') > 8192) break;
    result.push(next);
  }
  return result.length ? result : undefined;
}

/**
 * Check exactly the files named by a manifest. The temporary structural request
 * uses cwd because validateRequest intentionally requires job_root === cwd;
 * preflight then receives a second request whose input_root is the canonical
 * project. This is an internal adapter-contract mapping, not a project copy.
 */
export async function checkVideoSource({ project, manifest }) {
  try {
    const root = await canonicalProject(project);
    const manifestPath = await regularCanonical(manifest, 'invalid_manifest');
    const { readFile: adapterReadFile, validateRequest, preflight } = await adapter();
    let parsed;
    try {
      const bytes = await adapterReadFile(manifestPath, 65536);
      parsed = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes));
    } catch {
      throw new StaticCheckError('invalid_manifest');
    }
    parsed = parseManifest(parsed);
    const extension = parsed.profile.format === 'png_sequence_tar' ? 'tar' : parsed.profile.format;
    const structural = {
      protocol_version: 1,
      release_id: 'authoring-check',
      job_id: 'authoring-check',
      job_root: process.cwd(),
      input_root: path.join(process.cwd(), 'source'),
      entry_file: parsed.entry_file,
      sources: parsed.sources,
      output_path: path.join(process.cwd(), `result.${extension}`),
      profile: parsed.profile,
      timeout_ms: 1000,
      max_output_bytes: 2 * 1024 * 1024 * 1024,
    };
    validateRequest(structural, 'authoring-check');
    await preflight({ ...structural, job_root: root, input_root: root, output_path: path.join(root, `result.${extension}`) });
    return { status: 'valid', coverage: COVERAGE };
  } catch (error) {
    const publicCode = error instanceof StaticCheckError ? error.publicCode : code(error?.code);
    const diagnostics = adapterDiagnostics(error?.diagnostics) ?? diagnostic(publicCode);
    return { status: 'refused', code: publicCode, diagnostics, coverage: COVERAGE };
  }
}

function cliOptions(args) {
  if (args.length !== 4 || args[0] !== '--project' || args[2] !== '--manifest'
    || !path.isAbsolute(args[1]) || !path.isAbsolute(args[3])) throw new StaticCheckError('usage');
  return { project: args[1], manifest: args[3] };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  try {
    const result = await checkVideoSource(cliOptions(process.argv.slice(2)));
    process.stdout.write(`${JSON.stringify(result)}\n`);
    if (result.status !== 'valid') process.exitCode = 1;
  } catch (error) {
    process.stdout.write(`${JSON.stringify({ status: 'refused', code: code(error?.publicCode), diagnostics: diagnostic(error?.publicCode), coverage: COVERAGE })}\n`);
    process.exitCode = 1;
  }
}
