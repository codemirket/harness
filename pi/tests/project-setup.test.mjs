import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {syncBuiltinESMExports} from 'node:module';
import {pathToFileURL} from 'node:url';
import {initializeProject, recordProjectUse, projectSyncStatus, readProjectGuidance} from '../dist/extension/project-setup.js';
import {prepareProfile, applyProfile} from '../dist/extension/profiles.js';

const START = '<!-- mirket-project:start -->', END = '<!-- mirket-project:end -->';
function fixture() {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'mirket-project-')));
  const pkg = path.join(root, 'package'), home = path.join(root, 'private-runtime'), project = path.join(root, 'project');
  for (const directory of [pkg, home, project]) fs.mkdirSync(directory, {mode: 0o700});
  fs.mkdirSync(path.join(pkg, 'skills'));
  fs.writeFileSync(path.join(pkg, 'skills/catalog.json'), JSON.stringify({global: [{id: 'core'}], project: [{id: 'design'}, {id: 'conditional'}], profiles: {visual: {skills: ['design'], conditional: ['conditional']}}}));
  const runtime = {home, cwd: project, command: path.join(home, 'bin/mirket'), binding: {releaseId: 'sha256:' + 'a'.repeat(64)}};
  const guidance = path.join(project, 'AGENTS.md'), settings = path.join(project, '.pi/settings.json');
  return {root, pkg, home, project, runtime, guidance, settings, close() {fs.rmSync(root, {recursive: true, force: true});}};
}
function observations(file) {
  const match = fs.readFileSync(file, 'utf8').match(/<!-- mirket-project-data (.*?) -->/u);
  assert.ok(match); return JSON.parse(match[1]);
}
function consentPath(f) {return path.join(f.home, 'project-setup', fs.readdirSync(path.join(f.home, 'project-setup')).find(file => file.endsWith('.json')));}
function writeSettings(f, packages) {fs.mkdirSync(path.dirname(f.settings), {recursive: true}); fs.writeFileSync(f.settings, JSON.stringify({theme: 'keep', packages}));}

test('project init creates baseline Pi selection and durable opt-in while preserving manual source bytes', () => {
  const f = fixture();
  try {
    const original = '\ufeff# Project rules\r\n\r\nKeep this exact text.\r\n';
    fs.writeFileSync(f.guidance, original, {mode: 0o644});
    assert.equal(projectSyncStatus(f.runtime).status, 'disabled');
    assert.equal(recordProjectUse(f.runtime, {kind: 'tool', id: 'mcp__mirket__code_inspect'}), false);
    assert.equal(fs.existsSync(path.join(f.home, 'project-setup')), false);
    const initialized = initializeProject(f.pkg, f.runtime, [], []);
    assert.equal(initialized.optedIn, true); assert.equal(initialized.changed, true);
    assert.deepEqual(initialized.selectedSkills, ['skills/core/SKILL.md']);
    assert.equal(initialized.project, f.project); assert.equal(initialized.guidanceFile, f.guidance);
    const saved = fs.readFileSync(f.guidance, 'utf8');
    assert.ok(saved.startsWith(original)); assert.ok(saved.includes(START + '\r\n'));
    assert.equal(fs.statSync(f.guidance).mode & 0o777, 0o644);
    assert.equal(fs.statSync(consentPath(f)).mode & 0o777, 0o600);
    assert.deepEqual(JSON.parse(fs.readFileSync(f.settings)).packages, [{source: f.pkg, skills: ['skills/core/SKILL.md'], extensions: ['+./dist/extension/index.js']}]);
    assert.equal(projectSyncStatus(f.runtime).status, 'ready');
    const consent = fs.readFileSync(consentPath(f));
    assert.equal(initializeProject(f.pkg, f.runtime, [], []).changed, false);
    assert.equal(fs.readFileSync(f.guidance, 'utf8'), saved);
    assert.deepEqual(fs.readFileSync(consentPath(f)), consent);
    assert.equal(fs.existsSync(path.join(f.project, '.agents')), false);
  } finally {f.close();}
});

test('init preserves exact existing filters and unrelated Pi settings; explicit selection replaces only the Mirket filter', () => {
  const f = fixture();
  try {
    writeSettings(f, [{source: '/other', skills: ['other']}, {source: f.pkg, skills: ['!skills/conditional/**', 'skills/design/SKILL.md'], extensions: [], autoload: false}]);
    const first = initializeProject(f.pkg, f.runtime, [], []);
    assert.deepEqual(first.selectedSkills, ['!skills/conditional/**', 'skills/design/SKILL.md']);
    const result = initializeProject(f.pkg, f.runtime, ['visual'], ['conditional']);
    assert.deepEqual(result.selectedSkills, ['!skills/**', '+skills/conditional/SKILL.md', '+skills/core/SKILL.md', '+skills/design/SKILL.md']);
    const settings = JSON.parse(fs.readFileSync(f.settings));
    assert.equal(settings.theme, 'keep'); assert.deepEqual(settings.packages[0], {source: '/other', skills: ['other']});
    assert.deepEqual(settings.packages[1].extensions, ['+./dist/extension/index.js']); assert.equal(settings.packages[1].autoload, false);
  } finally {f.close();}
});

test('existing unfiltered package remains honestly unfiltered and string entries become project package objects', () => {
  for (const asString of [false, true]) {
    const f = fixture();
    try {
      writeSettings(f, [asString ? f.pkg : {source: f.pkg, autoload: false}]);
      assert.equal(initializeProject(f.pkg, f.runtime, [], []).selectedSkills, null);
      const entry = JSON.parse(fs.readFileSync(f.settings)).packages[0];
      assert.equal(typeof entry, 'object'); assert.equal(entry.source, f.pkg); assert.equal(Object.hasOwn(entry, 'skills'), false);
    } finally {f.close();}
  }
});

test('exact successful-use observations are sorted, persistent and idempotent without changing selection or manual source edits', async () => {
  const f = fixture();
  try {
    initializeProject(f.pkg, f.runtime, [], []);
    fs.appendFileSync(f.guidance, '\nManual note added later.\n');
    const settings = fs.readFileSync(f.settings);
    assert.equal(recordProjectUse(f.runtime, {kind: 'tool', id: 'mcp__mirket__code_inspect'}), true);
    assert.equal(recordProjectUse(f.runtime, {kind: 'skill', id: 'design'}), true);
    assert.equal(recordProjectUse(f.runtime, {kind: 'skill', id: 'core'}), true);
    const saved = fs.readFileSync(f.guidance);
    assert.equal(recordProjectUse(f.runtime, {kind: 'skill', id: 'design'}), false);
    assert.deepEqual(fs.readFileSync(f.guidance), saved);
    assert.deepEqual(observations(f.guidance), {schema: 1, skills: ['core', 'design'], tools: ['mcp__mirket__code_inspect']});
    assert.ok(saved.toString().endsWith('\nManual note added later.\n'));
    assert.deepEqual(fs.readFileSync(f.settings), settings);
    const reloaded = await import('../dist/extension/project-setup.js?reload=' + Date.now());
    assert.equal(reloaded.projectSyncStatus(f.runtime).status, 'ready');
    assert.equal(reloaded.recordProjectUse(f.runtime, {kind: 'skill', id: 'design'}), false);
  } finally {f.close();}
});

test('unknown and injected IDs refuse with no source or consent effects', () => {
  const f = fixture();
  try {
    initializeProject(f.pkg, f.runtime, [], []);
    const before = fs.readFileSync(f.guidance), consent = fs.readFileSync(consentPath(f));
    for (const use of [{kind: 'skill', id: 'missing'}, {kind: 'tool', id: 'mcp__mirket__invented'}, {kind: 'tool', id: 'x\n' + END}, {kind: 'bad', id: 'design'}]) {
      assert.throws(() => recordProjectUse(f.runtime, use), /Unknown|Invalid/u);
      assert.deepEqual(fs.readFileSync(f.guidance), before); assert.deepEqual(fs.readFileSync(consentPath(f)), consent);
    }
  } finally {f.close();}
});

test('malformed markers, invalid UTF-8 and oversized guidance refuse before Pi settings are edited', () => {
  for (const body of [START, END + '\n' + START, START + '\n' + START + '\n' + END + '\n' + END, '<!-- mirket-project:other -->', Buffer.from([0xff]), 'x'.repeat(1024 * 1024 + 1)]) {
    const f = fixture();
    try {
      fs.writeFileSync(f.guidance, body); const original = fs.readFileSync(f.guidance);
      assert.throws(() => initializeProject(f.pkg, f.runtime, [], []));
      assert.deepEqual(fs.readFileSync(f.guidance), original); assert.equal(fs.existsSync(f.settings), false);
      assert.equal(fs.readdirSync(path.join(f.home, 'project-setup')).length, 0);
    } finally {f.close();}
  }
});

test('managed section drift refuses observations and reinitialization, while project/release consent is scoped', () => {
  const f = fixture();
  try {
    initializeProject(f.pkg, f.runtime, [], []);
    const oldSource = fs.readFileSync(f.guidance, 'utf8');
    fs.writeFileSync(f.guidance, oldSource.replace('none yet', 'user change'));
    assert.equal(projectSyncStatus(f.runtime).status, 'changed');
    assert.throws(() => recordProjectUse(f.runtime, {kind: 'skill', id: 'design'}), /section changed/u);
    assert.throws(() => initializeProject(f.pkg, f.runtime, [], []), /section changed/u);
    fs.writeFileSync(f.guidance, oldSource);
    const updated = {...f.runtime, binding: {releaseId: 'sha256:' + 'b'.repeat(64)}};
    assert.equal(projectSyncStatus(updated).status, 'changed');
    assert.throws(() => recordProjectUse(updated, {kind: 'skill', id: 'design'}), /another release/u);
    initializeProject(f.pkg, updated, [], []); assert.equal(projectSyncStatus(updated).status, 'ready');
    const other = path.join(f.root, 'other'); fs.mkdirSync(other);
    assert.equal(recordProjectUse({...updated, cwd: other}, {kind: 'skill', id: 'design'}), false);
    const moved = path.join(f.root, 'moved'); fs.renameSync(f.project, moved); fs.mkdirSync(f.project);
    assert.equal(projectSyncStatus(updated).status, 'changed');
  } finally {f.close();}
});

test('aliased project, settings, guidance, catalog and private consent paths refuse without touching targets', () => {
  for (const kind of ['project', 'settings', 'guidance-symlink', 'guidance-hardlink', 'catalog', 'consent', 'state']) {
    const f = fixture();
    try {
      let runtime = f.runtime;
      const outside = path.join(f.root, 'outside'); fs.mkdirSync(outside);
      const canary = path.join(outside, 'canary'); fs.writeFileSync(canary, 'untouched\n');
      if (kind === 'project') {const alias = path.join(f.root, 'alias'); fs.symlinkSync(f.project, alias); runtime = {...runtime, cwd: alias};}
      if (kind === 'settings') fs.symlinkSync(outside, path.dirname(f.settings));
      if (kind === 'guidance-symlink') fs.symlinkSync(canary, f.guidance);
      if (kind === 'guidance-hardlink') fs.linkSync(canary, f.guidance);
      if (kind === 'catalog') {fs.renameSync(path.join(f.pkg, 'skills'), path.join(outside, 'skills')); fs.symlinkSync(path.join(outside, 'skills'), path.join(f.pkg, 'skills'));}
      if (kind === 'consent') {initializeProject(f.pkg, runtime, [], []); const file = consentPath(f); fs.unlinkSync(file); fs.symlinkSync(canary, file);}
      if (kind === 'state') fs.symlinkSync(outside, path.join(f.home, 'project-setup'));
      assert.throws(() => initializeProject(f.pkg, runtime, [], []), undefined, kind);
      assert.equal(fs.readFileSync(canary, 'utf8'), 'untouched\n', kind);
      if (kind === 'settings') assert.deepEqual(fs.readdirSync(outside), ['canary']);
    } finally {f.close();}
  }
});

test('duplicate package ownership and invalid existing filters refuse before guidance or consent is written', () => {
  for (const packages of [f => [f.pkg, {source: f.pkg}], f => [{source: f.pkg, skills: 'bad'}], () => ['npm:@codemirket/pi-harness@0.0.1']]) {
    const f = fixture();
    try {
      writeSettings(f, packages(f)); const before = fs.readFileSync(f.settings);
      assert.throws(() => initializeProject(f.pkg, f.runtime, [], []), /Duplicate|Invalid|Conflicting/u);
      assert.deepEqual(fs.readFileSync(f.settings), before); assert.equal(fs.existsSync(f.guidance), false);
      assert.equal(fs.readdirSync(path.join(f.home, 'project-setup')).length, 0);
    } finally {f.close();}
  }
});

test('cooperating lock contention and observed source drift preserve newer external content', () => {
  const f = fixture(), originalWrite = fs.writeFileSync;
  try {
    initializeProject(f.pkg, f.runtime, [], []);
    const lock = consentPath(f) + '.lock'; fs.mkdirSync(lock);
    const before = fs.readFileSync(f.guidance);
    assert.throws(() => recordProjectUse(f.runtime, {kind: 'skill', id: 'design'}));
    assert.deepEqual(fs.readFileSync(f.guidance), before); fs.rmdirSync(lock);
    fs.writeFileSync = function(file, ...args) {
      const result = originalWrite.call(this, file, ...args);
      if (String(file).startsWith(f.guidance + '.mirket-')) originalWrite(f.guidance, 'Newer external edit\n');
      return result;
    };
    syncBuiltinESMExports();
    assert.throws(() => recordProjectUse(f.runtime, {kind: 'skill', id: 'design'}), /changed during update/u);
    assert.equal(fs.readFileSync(f.guidance, 'utf8'), 'Newer external edit\n');
    assert.equal(fs.readdirSync(f.project).some(file => file.endsWith('.next')), false);
  } finally {fs.writeFileSync = originalWrite; syncBuiltinESMExports(); f.close();}
});

test('managed children cannot create opt-in or mutate project source', () => {
  const f = fixture(), previous = process.env.MIRKET_MANAGED_CHILD;
  try {
    initializeProject(f.pkg, f.runtime, [], []); const before = fs.readFileSync(f.guidance);
    process.env.MIRKET_MANAGED_CHILD = '1';
    assert.throws(() => initializeProject(f.pkg, f.runtime, [], []), /Managed children/u);
    assert.equal(recordProjectUse(f.runtime, {kind: 'skill', id: 'design'}), false);
    assert.deepEqual(fs.readFileSync(f.guidance), before);
  } finally {if (previous === undefined) delete process.env.MIRKET_MANAGED_CHILD; else process.env.MIRKET_MANAGED_CHILD = previous; f.close();}
});

test('guidance refresh requires scoped opt-in and rechecks current managed source', () => {
  const f = fixture();
  try {
    fs.writeFileSync(f.guidance, 'Manual rules before enrollment.\n');
    assert.equal(readProjectGuidance(f.runtime), undefined);
    assert.equal(fs.existsSync(path.join(f.home, 'project-setup')), false);
    initializeProject(f.pkg, f.runtime, [], []);
    recordProjectUse(f.runtime, {kind: 'skill', id: 'design'});
    fs.appendFileSync(f.guidance, '\nCurrent manual rules.\n');
    assert.deepEqual(readProjectGuidance(f.runtime), {file: f.guidance, content: fs.readFileSync(f.guidance, 'utf8')});
    const changedRelease = {...f.runtime, binding: {releaseId: 'sha256:' + 'b'.repeat(64)}};
    assert.throws(() => readProjectGuidance(changedRelease), /another release/u);
    fs.writeFileSync(f.guidance, fs.readFileSync(f.guidance, 'utf8').replace('Observed skill IDs:', 'Edited skill IDs:'));
    assert.throws(() => readProjectGuidance(f.runtime), /section changed/u);
  } finally {f.close();}
});

test('removed, changed, duplicate and malformed project package ownership stops all enrolled source access without mutation', () => {
  for (const change of [
    f => fs.unlinkSync(f.settings),
    f => fs.writeFileSync(f.settings, '{"packages":[]}'),
    f => fs.writeFileSync(f.settings, JSON.stringify({packages: [{source: path.join(f.root, 'other-package')}]})),
    f => fs.writeFileSync(f.settings, JSON.stringify({packages: [{source: f.pkg}, {source: f.pkg}]})),
    f => fs.writeFileSync(f.settings, JSON.stringify({packages: [{source: f.pkg}, 'npm:@codemirket/pi-harness@0.0.1']})),
    f => fs.writeFileSync(f.settings, JSON.stringify({packages: [f.pkg]})),
    f => fs.writeFileSync(f.settings, JSON.stringify({packages: [{source: f.pkg, skills: 'bad'}]})),
    f => fs.writeFileSync(f.settings, JSON.stringify({packages: [{source: f.pkg, skills: [7]}]})),
    f => {const outside = path.join(f.root, 'external-settings.json'); fs.renameSync(f.settings, outside); fs.symlinkSync(outside, f.settings);},
    f => fs.linkSync(f.settings, path.join(f.root, 'hardlinked-settings.json')),
    f => {const outside = path.join(f.root, 'external-pi'); fs.renameSync(path.dirname(f.settings), outside); fs.symlinkSync(outside, path.dirname(f.settings));},
  ]) {
    const f = fixture();
    try {
      initializeProject(f.pkg, f.runtime, [], []);
      const guidance = fs.readFileSync(f.guidance), consent = fs.readFileSync(consentPath(f));
      change(f);
      const settings = fs.existsSync(f.settings) ? fs.readFileSync(f.settings) : undefined;
      assert.equal(projectSyncStatus(f.runtime).status, 'changed');
      assert.throws(() => recordProjectUse(f.runtime, {kind: 'tool', id: 'mcp__mirket__code_inspect'}));
      assert.throws(() => readProjectGuidance(f.runtime));
      assert.deepEqual(fs.readFileSync(f.guidance), guidance);
      assert.deepEqual(fs.readFileSync(consentPath(f)), consent);
      if (settings) assert.deepEqual(fs.readFileSync(f.settings), settings);
      else assert.equal(fs.existsSync(f.settings), false);
    } finally {f.close();}
  }
});

test('legitimate project skill filter changes keep enrollment valid without freezing the selection', () => {
  const f = fixture();
  try {
    initializeProject(f.pkg, f.runtime, [], []);
    applyProfile(prepareProfile(f.pkg, f.settings, ['visual'], []));
    assert.equal(projectSyncStatus(f.runtime).status, 'ready');
    assert.equal(recordProjectUse(f.runtime, {kind: 'skill', id: 'design'}), true);
    assert.deepEqual(readProjectGuidance(f.runtime), {file: f.guidance, content: fs.readFileSync(f.guidance, 'utf8')});
    const settings = JSON.parse(fs.readFileSync(f.settings)); delete settings.packages[0].skills;
    fs.writeFileSync(f.settings, JSON.stringify(settings));
    assert.equal(projectSyncStatus(f.runtime).status, 'ready');
    assert.equal(recordProjectUse(f.runtime, {kind: 'tool', id: 'mcp__mirket__code_inspect'}), true);
  } finally {f.close();}
});

test('init explicitly enables the sole Mirket extension for normal and delta project objects without changing their skill filter', () => {
  for (const autoload of [undefined, true, false]) {
    const f = fixture();
    try {
      const original = {source: f.pkg, skills: ['skills/design/SKILL.md'], extensions: [], prompts: [], ...(autoload === undefined ? {} : {autoload})};
      writeSettings(f, [{source: '/other', extensions: []}, original]);
      const result = initializeProject(f.pkg, f.runtime, [], []);
      assert.deepEqual(result.selectedSkills, original.skills);
      const settings = JSON.parse(fs.readFileSync(f.settings)), entry = settings.packages[1];
      assert.equal(settings.theme, 'keep'); assert.deepEqual(settings.packages[0], {source: '/other', extensions: []});
      assert.deepEqual(entry, {...original, extensions: ['+./dist/extension/index.js']});
      assert.equal(projectSyncStatus(f.runtime).status, 'ready');
      entry.extensions = []; fs.writeFileSync(f.settings, JSON.stringify(settings));
      const guidance = fs.readFileSync(f.guidance), consent = fs.readFileSync(consentPath(f));
      assert.equal(projectSyncStatus(f.runtime).status, 'changed');
      assert.throws(() => recordProjectUse(f.runtime, {kind: 'skill', id: 'design'}), /extension filter/u);
      assert.throws(() => readProjectGuidance(f.runtime), /extension filter/u);
      assert.deepEqual(fs.readFileSync(f.guidance), guidance); assert.deepEqual(fs.readFileSync(consentPath(f)), consent);
      initializeProject(f.pkg, f.runtime, [], []);
      assert.equal(projectSyncStatus(f.runtime).status, 'ready');
      const repaired = JSON.parse(fs.readFileSync(f.settings)); delete repaired.packages[1].extensions;
      fs.writeFileSync(f.settings, JSON.stringify(repaired));
      assert.equal(projectSyncStatus(f.runtime).status, autoload === false ? 'changed' : 'ready');
    } finally {f.close();}
  }
});

test('native Pi package filtering accepts the generated exact extension path in normal and delta modes', {skip: !process.env.MIRKET_PI_PEERS}, async () => {
  const modulePath = path.join(process.env.MIRKET_PI_PEERS, '@earendil-works/pi-coding-agent/dist/core/package-manager.js');
  const {DefaultPackageManager} = await import(pathToFileURL(modulePath).href);
  const f = fixture();
  try {
    const extension = path.join(f.pkg, 'dist/extension/index.js');
    fs.mkdirSync(path.dirname(extension), {recursive: true}); fs.writeFileSync(extension, 'export default function() {}\n');
    fs.writeFileSync(path.join(f.pkg, 'package.json'), JSON.stringify({name: '@codemirket/pi-harness', version: '0.0.1', type: 'module', pi: {extensions: ['./dist/extension/index.js']}}));
    const manager = new DefaultPackageManager({cwd: f.project, agentDir: path.join(f.root, 'unused-agent'), settingsManager: {}});
    const metadata = {source: f.pkg, scope: 'project', origin: 'package', baseDir: f.pkg, packageRoot: f.pkg};
    for (const autoload of [true, false]) {
      writeSettings(f, [{source: f.pkg, extensions: [], autoload}]);
      initializeProject(f.pkg, f.runtime, [], []);
      const entry = JSON.parse(fs.readFileSync(f.settings)).packages[0];
      const resources = manager.createAccumulator();
      manager.collectPackageResources(f.pkg, resources, entry, metadata);
      assert.equal(resources.extensions.get(extension)?.enabled, true, 'native Pi autoload=' + autoload);
      for (const pattern of ['+./dist/extension/index.js', '+dist/extension/index.js', '+' + extension, 'dist/extension/index.js', extension]) {
        const selected = {...entry, extensions: [pattern]};
        fs.writeFileSync(f.settings, JSON.stringify({packages: [selected]}));
        const exact = manager.createAccumulator();
        manager.collectPackageResources(f.pkg, exact, selected, metadata);
        assert.equal(exact.extensions.get(extension)?.enabled, true, 'native exact ' + pattern + ', autoload=' + autoload);
        assert.equal(projectSyncStatus(f.runtime).status, 'ready');
      }
      // Preserve the native empty-filter behavior that exposed this wiring gap.
      const disabled = manager.createAccumulator();
      manager.collectPackageResources(f.pkg, disabled, {...entry, autoload: true, extensions: []}, metadata);
      assert.equal(disabled.extensions.get(extension)?.enabled, false);
    }
  } finally {f.close();}
});
