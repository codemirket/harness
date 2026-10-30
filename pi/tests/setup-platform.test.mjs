import test from 'node:test';
import assert from 'node:assert/strict';
import {supportsMacOSVersion} from '../scripts/macos-version.mjs';

test('package setup admits the macOS 26.0 binary floor and newer major/minor releases', () => {
  for (const version of ['26.0', '26.5.2', '26.7.2', '27.0', '27.1.3', '28.0']) {
    assert.equal(supportsMacOSVersion(version), true, version);
  }
});

test('package setup refuses older, malformed or unknown macOS versions', () => {
  for (const version of ['25.9.9', '13.5', '26', '27.0-beta', '27.0.1.2', '', '9999999999999999999999.0', null]) {
    assert.equal(supportsMacOSVersion(version), false, String(version));
  }
});
