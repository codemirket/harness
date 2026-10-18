// Derive an isolated production importer from the frozen upstream lock.
// No version resolution, install hooks or network access occurs here.
import fs from 'node:fs';
import {createRequire} from 'node:module';
const require = createRequire('/opt/penpot/mcp/packages/server/package.json');
const yaml = require('js-yaml');
const root = '/opt/penpot/mcp';
const pkg = JSON.parse(fs.readFileSync(`${root}/packages/server/package.json`));
const documents = yaml.loadAll(fs.readFileSync(`${root}/pnpm-lock.yaml`, 'utf8'));
const candidates = documents.filter(doc => doc?.importers?.['packages/server']);
if (candidates.length !== 1) throw Error('Expected exactly one pinned server importer');
const lock = candidates[0];
const importer = lock.importers['packages/server'];
if (!importer?.dependencies || Object.keys(pkg.dependencies).sort().join() !== Object.keys(importer.dependencies).sort().join()) throw Error('Pinned production dependencies differ from importer');
delete pkg.devDependencies;
delete pkg.scripts;
lock.importers = {'.': {dependencies: importer.dependencies}};
fs.mkdirSync('/opt/server');
fs.writeFileSync('/opt/server/package.json', JSON.stringify(pkg));
fs.writeFileSync('/opt/server/pnpm-lock.yaml', '---\n' + documents.map(doc => yaml.dump(doc)).join('---\n'));
const config = yaml.load(fs.readFileSync(`${root}/pnpm-workspace.yaml`, 'utf8'));
delete config.packages;
config.storeDir = '/opt/penpot/.pnpm-store';
fs.writeFileSync('/opt/server/pnpm-workspace.yaml', yaml.dump(config));
for (const part of ['dist', 'data']) fs.cpSync(`${root}/packages/server/${part}`, `/opt/server/${part}`, {recursive: true});
