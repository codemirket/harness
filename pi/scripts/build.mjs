// Explicit offline build against the separately admitted host peer tree.
import fs from 'node:fs';
import path from 'node:path';
import ts from 'typescript';
const root = path.resolve(import.meta.dirname, '..');
const peers = process.env.MIRKET_PI_PEERS;
if (!peers || !path.isAbsolute(peers)) throw new Error('Set MIRKET_PI_PEERS to the exact admitted node_modules directory');
const paths = {};
for (const [name, version, entry] of [
  ['@earendil-works/pi-coding-agent', '1.4.0', 'dist/index.d.ts'],
  ['typebox', '1.3.27', 'build/typebox.d.mts'],
]) {
  const directory = path.join(peers, name);
  const pkg = JSON.parse(fs.readFileSync(path.join(directory, 'package.json'), 'utf8'));
  if (pkg.version !== version) throw new Error(`Unadmitted peer ${name}@${pkg.version}`);
  paths[name] = [path.join(directory, name === 'typebox' ? pkg.types : entry)];
}
const config = ts.readConfigFile(path.join(root, 'tsconfig.json'), ts.sys.readFile);
if (config.error) throw new Error(ts.flattenDiagnosticMessageText(config.error.messageText, '\n'));
config.config.compilerOptions.paths = paths;
config.config.compilerOptions.typeRoots = [path.join(root, 'node_modules/@types')];
const parsed = ts.parseJsonConfigFileContent(config.config, ts.sys, root);
const program = ts.createProgram(parsed.fileNames, parsed.options);
const emitted = program.emit();
const errors = [...parsed.errors, ...ts.getPreEmitDiagnostics(program), ...emitted.diagnostics];
if (errors.length) {
  console.error(ts.formatDiagnosticsWithColorAndContext(errors, {getCanonicalFileName:f=>f,getCurrentDirectory:()=>root,getNewLine:()=> '\n'}));
  process.exitCode = 1;
}
