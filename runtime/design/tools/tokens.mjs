import fs from 'node:fs/promises';
import path from 'node:path';
import {workspace, workspacePath, readFile, sha256, temporaryDirectory, readBoundedFile} from './paths.mjs';
import {checkRequest} from './request.mjs';

const DEFAULT_SOURCE = process.env.MIRKET_DESIGN_TOKENS || 'design/tokens.json';
const OUTPUT_NAMES = ['tokens.css', 'tokens.js', 'tokens.ts', 'tokens.typ', 'penpot.tokens.json'];

function flat(dictionary) {
  return Object.fromEntries(dictionary.allTokens.map(token => [token.path.join('.'), token.$value ?? token.value]));
}
function nested(dictionary) {
  const root = {};
  for (const token of dictionary.allTokens) {
    let at = root;
    for (const segment of token.path.slice(0, -1)) at = at[segment] ||= {};
    at[token.path.at(-1)] = {$type: token.$type ?? token.type, $value: token.$value ?? token.value};
  }
  return root;
}
function typstValue(value) {
  if (value === null) return 'none';
  if (typeof value === 'number') {
    if (!Number.isFinite(value)) throw Error('Token contains a non-finite number');
    return String(value);
  }
  if (typeof value === 'boolean') return String(value);
  if (typeof value === 'string') return JSON.stringify(value);
  if (Array.isArray(value)) return `(${value.map(typstValue).join(', ')}${value.length === 1 ? ',' : ''})`;
  if (typeof value === 'object') {
    const entries = Object.entries(value);
    if (!entries.length) return '(:)';
    const pairs = entries.map(([key, item]) => `(${JSON.stringify(key)}, ${typstValue(item)})`);
    return `(${pairs.join(', ')}${pairs.length === 1 ? ',' : ''}).to-dict()`;
  }
  throw Error('Token contains a value Typst cannot represent');
}
let compiler;
function tokenCompiler() {
  return compiler ||= import('style-dictionary').then(({default: StyleDictionary}) => {
    for (const [name, format] of Object.entries({
      'mirket/js': ({dictionary}) => `export const tokens = ${JSON.stringify(flat(dictionary), null, 2)};\n`,
      'mirket/ts': ({dictionary}) => `export const tokens = ${JSON.stringify(flat(dictionary), null, 2)} as const;\n`,
      'mirket/typst': ({dictionary}) => `#let tokens = ${typstValue(flat(dictionary))}\n`,
      'mirket/penpot': ({dictionary}) => `${JSON.stringify(nested(dictionary), null, 2)}\n`,
    })) StyleDictionary.registerFormat({name, format});
    return StyleDictionary;
  });
}

export async function compileTokens(source = DEFAULT_SOURCE) {
  checkRequest();
  const input = await readFile(source, 512 * 1024);
  const digest = sha256(input);
  // Formats/configuration live in this module; the locked closure identifies the
  // compiler and its transforms. Preserve source identity separately for consumers.
  const recipe = sha256(Buffer.concat([
    await fs.readFile(new URL('./tokens.mjs', import.meta.url)),
    await fs.readFile(new URL('./package-lock.json', import.meta.url)),
  ]));
  const generation = sha256(Buffer.from(`${digest}:${recipe}`));
  const root = await workspacePath('.mirket-design/tokens', {kind: 'directory', missing: true, createParents: true});
  await fs.mkdir(root, {recursive: true, mode: 0o700});
  const dest = path.join(root, generation);
  const existing = await cachedGeneration();
  if (existing) return existing;
  async function cachedGeneration() {
    try {
      const stat = await fs.lstat(dest);
      if (!stat.isDirectory() || stat.isSymbolicLink()) throw Error('Token generation path is not a real directory');
      const manifest = JSON.parse((await readBoundedFile(path.join(dest, 'manifest.json'), 16384, 'Token manifest')).toString('utf8')); if (!manifest.output_sha256 || typeof manifest.output_sha256 !== 'object') throw Error('Invalid token manifest');
      if (manifest.source_sha256 !== digest || manifest.compiler_sha256 !== recipe) throw Error('Existing token generation has a different source or compiler identity');
      const outputs = {};
      for (const name of OUTPUT_NAMES) {
        const file = path.join(dest, name);
        const item = await fs.lstat(file);
        if (!item.isFile() || item.isSymbolicLink() || item.size === 0) throw Error('Existing token generation is incomplete');
        if (sha256(await readBoundedFile(file, 4 * 1024 * 1024, 'Compiled tokens')) !== manifest.output_sha256[name]) throw Error('Existing token generation has drifted');
        outputs[name] = path.relative(workspace, file);
      }
      checkRequest();
      return {source, source_sha256: digest, compiler_sha256: recipe, outputs, reused: true};
    } catch (error) {
      if (error.code !== 'ENOENT') throw error;
      if (await fs.lstat(dest).catch(() => null)) throw Error('Existing token generation is incomplete');
      return null;
    }
  }
  const tokens = JSON.parse(input.toString('utf8'));
  if (!tokens || typeof tokens !== 'object' || Array.isArray(tokens)) throw Error('Canonical tokens must be a DTCG JSON object');
  const stage = await temporaryDirectory('tokens');
  try {
    const StyleDictionary = await tokenCompiler();
    const sd = new StyleDictionary({
      usesDtcg: true,
      log: {verbosity: 'silent', warnings: 'error'},
      tokens,
      platforms: {
        css: {transformGroup: 'css', buildPath: `${stage}/`, files: [{destination: 'tokens.css', format: 'css/variables'}]},
        js: {transformGroup: 'js', buildPath: `${stage}/`, files: [{destination: 'tokens.js', format: 'mirket/js'}]},
        ts: {transformGroup: 'js', buildPath: `${stage}/`, files: [{destination: 'tokens.ts', format: 'mirket/ts'}]},
        typst: {transformGroup: 'js', buildPath: `${stage}/`, files: [{destination: 'tokens.typ', format: 'mirket/typst'}]},
        penpot: {transformGroup: 'js', buildPath: `${stage}/`, files: [{destination: 'penpot.tokens.json', format: 'mirket/penpot'}]},
      },
    });
    await sd.buildAllPlatforms();
    const outputs = {}, output_sha256 = {};
    for (const name of OUTPUT_NAMES) {
      const file = path.join(stage, name);
      const stat = await fs.lstat(file);
      if (!stat.isFile() || !stat.size) throw Error(`Style Dictionary did not build ${name}`);
      output_sha256[name] = sha256(await readBoundedFile(file, 4 * 1024 * 1024, 'Compiled tokens'));
      outputs[name] = path.relative(workspace, path.join(dest, name));
    }
    await fs.writeFile(path.join(stage, 'manifest.json'), JSON.stringify({source_sha256: digest, compiler_sha256: recipe, output_sha256}));
    checkRequest();
    try { await fs.rename(stage, dest); }
    catch (error) {
      if (!['EEXIST', 'ENOTEMPTY'].includes(error.code)) throw error;
      const winner = await cachedGeneration();
      if (!winner) throw error;
      return winner;
    }
    return {source, source_sha256: digest, compiler_sha256: recipe, outputs, reused: false};
  } finally { await fs.rm(stage, {recursive: true, force: true}).catch(() => {}); }
}
