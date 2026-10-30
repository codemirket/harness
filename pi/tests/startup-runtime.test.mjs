import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdtempSync, realpathSync, mkdirSync, writeFileSync, symlinkSync, linkSync, rmSync} from 'node:fs';
import {join} from 'node:path';
import {regularSha256, verifyRuntime} from '../dist/extension/runtime.js';
import {verifyHost} from '../dist/extension/host.js';

const sha = bytes => createHash('sha256').update(bytes).digest('hex');
function fixture() {
  const root = realpathSync(mkdtempSync('/private/tmp/mirket-startup-runtime-'));
  return {root, close: () => rmSync(root, {recursive: true, force: true})};
}

test('bootstrap digest covers full executable bytes and rechecks changed bytes without a cache', () => {
  const f = fixture();
  try {
    const file = join(f.root, 'executable'), bytes = Buffer.alloc(3 * 1024 * 1024 + 17, 0x31);
    writeFileSync(file, bytes);
    assert.equal(regularSha256(file, bytes.length), sha(bytes));
    bytes[bytes.length - 1] = 0x32;
    writeFileSync(file, bytes);
    assert.equal(regularSha256(file, bytes.length), sha(bytes));
    writeFileSync(file, Buffer.alloc(0));
    assert.equal(regularSha256(file, 0), sha(Buffer.alloc(0)));
  } finally { f.close(); }
});

test('bootstrap digest refuses symlinks, hard links, directories and oversized bytes', () => {
  const f = fixture();
  try {
    const file = join(f.root, 'executable');
    writeFileSync(file, 'identity');
    assert.throws(() => regularSha256(file, 7), /Invalid or aliased file/);
    assert.throws(() => regularSha256(f.root, 1024), /Invalid or aliased file/);
    const alias = join(f.root, 'alias');
    symlinkSync(file, alias);
    assert.throws(() => regularSha256(alias, 1024));
    rmSync(alias); linkSync(file, alias);
    assert.throws(() => regularSha256(file, 1024), /Invalid or aliased file/);
    assert.throws(() => regularSha256(alias, 1024), /Invalid or aliased file/);
  } finally { f.close(); }
});

test('runtime bootstrap verifies launcher and private Node anew for each call', async () => {
  const f = fixture();
  try {
    const pkg = join(f.root, 'package'), agent = join(f.root, 'agent'), project = join(f.root, 'project'), home = join(agent, 'mirket');
    for (const directory of [pkg, agent, project, home, join(home, 'bin')]) mkdirSync(directory, {mode: 0o700});
    const core = Buffer.from('launcher-v1'), node = Buffer.from('private-node-v1'), release = 'a'.repeat(64);
    const privateNode = join(home, 'releases', `r-${release}`, 'runtime/node/bin');
    mkdirSync(privateNode, {recursive: true, mode: 0o700});
    const packageBytes = JSON.stringify({name: '@codemirket/pi-harness', version: '0.0.1'});
    writeFileSync(join(pkg, 'package.json'), packageBytes);
    writeFileSync(join(pkg, 'native-binding.json'), JSON.stringify({schema: 1, packageVersion: '0.0.1', hostVersion: '1.4.0', hostInventorySha256: 'b'.repeat(64), releaseId: `sha256:${release}`, coreSha256: sha(core), nodeSha256: sha(node), packageSha256: sha(packageBytes)}));
    const launcher = join(home, 'bin/mirket'), executable = join(privateNode, 'node');
    writeFileSync(launcher, core); writeFileSync(executable, node);
    assert.equal((await verifyRuntime(pkg, agent, project, '1.4.0')).command, launcher);
    writeFileSync(launcher, 'launcher-v2');
    await assert.rejects(verifyRuntime(pkg, agent, project, '1.4.0'), /Native launcher identity mismatch/);
    writeFileSync(launcher, core); writeFileSync(executable, 'private-node-v2');
    await assert.rejects(verifyRuntime(pkg, agent, project, '1.4.0'), /Private Node identity mismatch/);
    writeFileSync(executable, node);
    assert.equal((await verifyRuntime(pkg, agent, project, '1.4.0')).cwd, project);
  } finally { f.close(); }
});

test('host bootstrap checks executable digests before invoking its verifier', async () => {
  const f = fixture();
  try {
    const hostRoot = join(f.root, 'node_modules'), hostPackage = join(hostRoot, '@earendil-works/pi-coding-agent'), entry = join(hostPackage, 'dist/cli.js');
    mkdirSync(join(hostPackage, 'dist'), {recursive: true}); writeFileSync(entry, 'host fixture');
    const verifier = join(f.root, 'verifier');
    const output = JSON.stringify({root: hostRoot, entry, lockSha256: 'c'.repeat(64), verifiedFiles: 15249});
    const script = `#!/bin/sh\nprintf '%s\\n' '${output}'\n`;
    writeFileSync(verifier, script, {mode: 0o700});
    const nodeHash = regularSha256(process.execPath, 256 * 1024 * 1024), coreHash = sha(script);
    await assert.rejects(verifyHost(f.root, hostPackage, 'd'.repeat(64), '0'.repeat(64), verifier, coreHash, entry), /qualified Node executable/);
    await assert.rejects(verifyHost(f.root, hostPackage, 'd'.repeat(64), nodeHash, verifier, '0'.repeat(64), entry), /Host verifier identity mismatch/);
    assert.deepEqual(await verifyHost(f.root, hostPackage, 'd'.repeat(64), nodeHash, verifier, coreHash, entry), JSON.parse(output));
    writeFileSync(verifier, script + '# changed\n');
    await assert.rejects(verifyHost(f.root, hostPackage, 'd'.repeat(64), nodeHash, verifier, coreHash, entry), /Host verifier identity mismatch/);
  } finally { f.close(); }
});
