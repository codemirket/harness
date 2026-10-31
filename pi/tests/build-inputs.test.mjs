import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {checkDependencies} from '../scripts/check-dependencies.mjs';

test('offline build guard refuses absent, mismatched and unlocked direct dependencies', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'mirket-build-inputs-'));
  const write = (file, value) => { const target = path.join(root, file); fs.mkdirSync(path.dirname(target), {recursive: true}); fs.writeFileSync(target, JSON.stringify(value)); };
  const manifest = {dependencies: {typescript: '5.9.3'}, devDependencies: {'@types/node': '24.19.2'}};
  const lock = {lockfileVersion: 3, packages: {'': manifest, 'node_modules/typescript': {version: '5.9.3'}, 'node_modules/@types/node': {version: '24.19.2'}}};
  try {
    write('package.json', manifest); write('package-lock.json', lock);
    write('node_modules/typescript/package.json', {name: 'typescript', version: '5.9.3'});
    assert.throws(() => checkDependencies(root), /ENOENT/);
    write('node_modules/@types/node/package.json', {name: '@types/node', version: '24.19.2'});
    assert.doesNotThrow(() => checkDependencies(root));
    write('node_modules/typescript/package.json', {name: 'typescript', version: '5.9.2'});
    assert.throws(() => checkDependencies(root), /Installed dependency differs/);
    write('node_modules/typescript/package.json', {name: 'typescript', version: '5.9.3'});
    lock.packages['node_modules/typescript'].version = '5.9.2'; write('package-lock.json', lock);
    assert.throws(() => checkDependencies(root), /not exactly locked/);
    lock.packages['node_modules/typescript'].version = '5.9.3'; write('package-lock.json', lock);
    write('package.json', {...manifest, dependencies: {...manifest.dependencies, extra: '1.0.0'}});
    assert.throws(() => checkDependencies(root), /Package lock differs/);
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
});
