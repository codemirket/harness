import test from 'node:test';
import assert from 'node:assert/strict';
import {browserOrigin, createBrowserPolicy, assertBrowserBinding, BROWSER_TOOLS} from '../dist/extension/browser-policy.js';
import {createHash} from 'node:crypto';
import {mkdtempSync, writeFileSync, readFileSync, realpathSync, rmSync} from 'node:fs';
import {join} from 'node:path';

const context = {project: '/private/tmp/browser-test/project', release: 'sha256:test'};
function fixture(overrides = {}) {
  let time = 1000, checks = 0;
  const grant = {...context, peerPath: '/private/tmp/browser-test/peer/cli.js', peerIdentity: 'peer', browserPath: '/private/tmp/browser-test/browser/chrome', browserIdentity: 'browser', origin: 'http://127.0.0.1:8123', output: '/private/tmp/browser-test/output', expires: 301000, maxCalls: 64, methods: ['GET', 'HEAD'], tools: BROWSER_TOOLS, ...overrides};
  const policy = createBrowserPolicy(grant, () => { checks++; }, () => time);
  return {policy, advance(value) { time = value; }, get checks() { return checks; }};
}
test('browser grant requires an exact literal IPv4 loopback HTTP origin', () => {
  assert.equal(browserOrigin('http://127.0.0.1:8123'), 'http://127.0.0.1:8123');
  for (const origin of ['http://localhost:8123', 'https://127.0.0.1:8123', 'http://127.1:8123', 'http://[::1]:8123', 'http://127.0.0.1', 'http://127.0.0.1:80', 'http://127.0.0.1:8123/', 'http://user@127.0.0.1:8123', 'http://127.0.0.1:8123/?x']) assert.throws(() => browserOrigin(origin), undefined, origin);
});
test('browser policy binds project/release, rechecks identity and enforces expiry/calls', () => {
  const f = fixture({maxCalls: 2});
  assert.throws(() => f.policy.check('browser_snapshot', {}, {...context, project: '/other'}));
  assert.throws(() => f.policy.check('browser_snapshot', {}, {...context, release: 'other'}));
  assert.deepEqual(f.policy.check('browser_snapshot', {}, context), {});
  assert.equal(f.policy.remaining, 1); assert.equal(f.checks, 1);
  f.policy.check('browser_close', {}, context); assert.equal(f.policy.active, false);
  assert.throws(() => f.policy.check('browser_close', {}, context));
  const expired = fixture(); expired.advance(301000);
  assert.equal(expired.policy.network('GET', 'http://127.0.0.1:8123/'), false);
  assert.throws(() => expired.policy.check('browser_snapshot', {}, context));
  assert.throws(() => fixture({expires: 301001})); assert.throws(() => fixture({maxCalls: 65}));
  assert.throws(() => fixture({methods: ['GET', 'POST']})); assert.throws(() => fixture({tools: [...BROWSER_TOOLS, 'browser_evaluate']}));
  const revoked = fixture(); revoked.policy.revoke(); assert.throws(() => revoked.policy.check('browser_snapshot', {}, context));
  const changed = createBrowserPolicy({...context, peerPath: '/peer/cli.js', peerIdentity: 'p', browserPath: '/browser/chrome', browserIdentity: 'b', origin: 'http://127.0.0.1:1', output: '/out', expires: 1001, maxCalls: 1, methods: ['GET', 'HEAD'], tools: BROWSER_TOOLS}, () => { throw Error('identity drift'); }, () => 1000);
  assert.throws(() => changed.check('browser_snapshot', {}, context), /identity drift/); assert.equal(changed.remaining, 1);
});
test('identical executable bytes at alternate paths cannot substitute for the authorized peer/browser paths', () => {
  const root = realpathSync(mkdtempSync('/private/tmp/mirket-browser-paths-'));
  try {
    const paths = ['peer-cli.js', 'alternate-peer-cli.js', 'chrome', 'alternate-chrome'].map(name => join(root, name));
    for (const path of paths) writeFileSync(path, 'identical test executable bytes', {mode: 0o700});
    const sha = path => createHash('sha256').update(readFileSync(path)).digest('hex');
    assert.equal(sha(paths[0]), sha(paths[1])); assert.equal(sha(paths[2]), sha(paths[3]));
    const grant = {...context, peerPath: paths[0], peerIdentity: sha(paths[0]), browserPath: paths[2], browserIdentity: sha(paths[2]), origin: 'http://127.0.0.1:8123', output: join(root, 'output'), expires: 2000, maxCalls: 4, methods: ['GET', 'HEAD'], tools: BROWSER_TOOLS};
    let selected = {...grant};
    const policy = createBrowserPolicy(grant, () => assertBrowserBinding(grant, selected), () => 1000);
    policy.check('browser_snapshot', {}, context);
    for (const [key, alternate] of [['peerPath', paths[1]], ['browserPath', paths[3]]]) {
      selected = {...grant, [key]: alternate};
      assert.equal(selected.peerIdentity, grant.peerIdentity); assert.equal(selected.browserIdentity, grant.browserIdentity);
      assert.throws(() => assertBrowserBinding(grant, selected), new RegExp(key));
      assert.throws(() => policy.check('browser_snapshot', {}, context), new RegExp(key));
      assert.equal(policy.remaining, 3);
    }
  } finally { rmSync(root, {recursive: true, force: true}); }
});
test('seven-tool policy rejects executable, file, credential and argument escapes', () => {
  assert.equal(BROWSER_TOOLS.length, 7);
  const {policy} = fixture();
  for (const name of ['browser_evaluate', 'browser_run_code', 'browser_file_upload', 'browser_install', 'browser_tabs']) assert.throws(() => policy.check(name, {}, context));
  for (const url of ['file:///etc/passwd', 'http://127.0.0.1:8124/', 'http://user:pass@127.0.0.1:8123/', 'https://example.org/']) assert.throws(() => policy.check('browser_navigate', {url}, context));
  assert.throws(() => policy.check('browser_navigate', {url: 'http://127.0.0.1:8123/', code: 'x'}, context));
  assert.throws(() => policy.check('browser_snapshot', {filename: '/tmp/out'}, context));
  for (const filename of ['../x.png', '/tmp/x.png', 'x.jpg', 'x\\a.png', '.png']) assert.throws(() => policy.check('browser_take_screenshot', {type: 'png', scale: 'css', filename}, context));
  assert.deepEqual(policy.check('browser_take_screenshot', {type: 'png', scale: 'css', filename: 'wide.png'}, context), {type: 'png', scale: 'css', filename: '/private/tmp/browser-test/output/wide.png'});
  assert.deepEqual(policy.check('browser_take_screenshot', {type: 'png', scale: 'css', fullPage: false}, context), {type: 'png', scale: 'css', fullPage: false});
  assert.throws(() => policy.check('browser_take_screenshot', {type: 'png', scale: 'css', fullPage: true}, context), /Only viewport.*fullPage/);
  assert.throws(() => policy.check('browser_press_key', {key: 'Control+L'}, context));
  assert.throws(() => policy.check('browser_resize', {width: 10000, height: 700}, context));
  assert.throws(() => policy.check('browser_wait_for', {time: 30}, context));
  assert.throws(() => policy.check('browser_wait_for', {text: 'x', textGone: 'y'}, context));
  assert.equal(policy.network('POST', 'http://127.0.0.1:8123/'), false);
  assert.equal(policy.network('HEAD', 'http://127.0.0.1:8123/x'), true);
});
