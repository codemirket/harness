#!/usr/bin/env node
/* MIT. Check probe compatibility before dispatch, using the observer's own validator.
 * No model, browser, artifact execution or untrusted JavaScript is accepted.
 * The evaluated declaration prefix is trusted repository code, not a sandbox. */
'use strict';
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const vm = require('node:vm');
const assert = require('node:assert/strict');

const ROOT = path.resolve(__dirname, '../..');
const OBSERVER = path.join(__dirname, 'observe.cjs');
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');

function validator() {
  assert.equal(fs.lstatSync(OBSERVER).isSymbolicLink(), false, 'observer symlink refused');
  const bytes = fs.readFileSync(OBSERVER);
  const source = bytes.toString('utf8');
  const end = source.indexOf('\nasync function serve(');
  const start = source.indexOf('function validatePlan(');
  assert.ok(start >= 0 && end > start, 'observer declaration interface changed; review preflight');
  const prefix = source.slice(0, end);
  const context = vm.createContext({ require });
  vm.runInContext(prefix, context, { timeout: 1000, filename: OBSERVER });
  return {
    observer: { path: path.relative(ROOT, OBSERVER).split(path.sep).join('/'), sha256: sha(bytes) },
    declaration_sha256: sha(Buffer.from(prefix)),
    check(value) {
      context.__probe = value;
      try { vm.runInContext('validatePlan(__probe)', context, { timeout: 1000 }); }
      finally { delete context.__probe; }
    }
  };
}

function planParts(value) {
  assert.ok(typeof value === 'string' && value && !path.isAbsolute(value) && !value.includes('\\'), 'normal project-relative plan required');
  const parts = value.split('/');
  assert.ok(parts.every(x => x && x !== '.' && x !== '..'), 'normal project-relative plan required');
  return parts;
}

function planFile(value) {
  const parts = planParts(value);
  let cursor = ROOT;
  for (const part of parts) {
    cursor = path.join(cursor, part);
    assert.equal(fs.lstatSync(cursor).isSymbolicLink(), false, 'plan symlink component refused');
  }
  const stat = fs.statSync(cursor);
  assert.ok(stat.isFile() && stat.size > 0 && stat.size <= 128 * 1024, 'plan must be a nonempty regular JSON file, at most128KiB');
  return { path: value, bytes: fs.readFileSync(cursor) };
}

function selfTest(v) {
  const base = () => ({
    schema_version: 1, id: 'synthetic-only',
    scenarios: [{ id: 'table', viewport: { width: 390, height: 844 },
      steps: [{ id: 'visible', action: 'expect_visible', target: { role: 'table', name: '', exact: false }, equals: true }] }]
  });
  const checks = [];
  function test(name, mutate, valid) {
    const probe = base(); mutate(probe);
    let accepted = true;
    try { v.check(probe); } catch (_) { accepted = false; }
    checks.push({ name, expected_valid: valid, actual_valid: accepted, passed: accepted === valid });
  }
  test('amended role-only table contract accepted', () => {}, true);
  test('original nameless-role failure rejected before browser launch', p => { delete p.scenarios[0].steps[0].target.name; }, false);
  test('named button contract accepted', p => { p.scenarios[0].steps[0].target = { role: 'button', name: 'Save' }; }, true);
  test('unknown action rejected', p => { p.scenarios[0].steps[0].action = 'invented_action'; }, false);
  test('duplicate step identity rejected', p => { p.scenarios[0].steps.push({ ...p.scenarios[0].steps[0] }); }, false);
  test('two locator kinds rejected', p => { p.scenarios[0].steps[0].target.testId = 'table'; }, false);
  test('unsupported viewport rejected', p => { p.scenarios[0].viewport.width = 100; }, false);
  test('absent literal expectation rejected', p => { delete p.scenarios[0].steps[0].equals; }, false);
  test('ambiguous text expectations rejected', p => { p.scenarios[0].steps[0] = { id: 'text', action: 'expect_text', target: { testId: 'message' }, equals: 'A', contains: 'A' }; }, false);
  test('expression-like storage path rejected', p => { p.scenarios[0].steps[0] = { id: 'state', action: 'read_storage', key: 'state', json_path: 'records[0]' }; }, false);
  for (const value of ['../outside.json', '/absolute.json', '..\\outside.json']) {
    let rejected = false;
    try { planParts(value); } catch (_) { rejected = true; }
    checks.push({ name: 'non-normal plan path rejected: ' + value, passed: rejected });
  }
  return { kind: 'synthetic_probe_preflight_checks', checks, check_count: checks.length,
    ok: checks.every(x => x.passed), model_executions: 0, browser_launches: 0, artifact_reads: 0 };
}

function main() {
  const args = process.argv.slice(2);
  assert.ok(args.length && args.length <= 32, 'Usage: node evals/design-quality/preflight.cjs PLAN.json ... | --self-test');
  const v = validator();
  let report;
  if (args.length === 1 && args[0] === '--self-test') report = selfTest(v);
  else {
    const plans = [];
    for (const argument of args) {
      let entry = { path: argument, sha256: null, valid: false };
      try {
        const file = planFile(argument);
        entry.sha256 = sha(file.bytes);
        v.check(JSON.parse(file.bytes.toString('utf8')));
        entry.valid = true;
      } catch (error) { entry.error = { name: error.name, message: error.message }; }
      plans.push(entry);
    }
    report = { kind: 'probe_contract_preflight', plans, ok: plans.every(x => x.valid),
      model_executions: 0, browser_launches: 0, artifact_reads: 0 };
  }
  report.schema_version = 1;
  report.checker = { path: path.relative(ROOT, __filename).split(path.sep).join('/'),
    sha256: sha(fs.readFileSync(__filename)) };
  report.observer = v.observer;
  report.declaration_sha256 = v.declaration_sha256;
  report.limits = ['Compatibility with the current observer only; no semantic correctness, artifact success or design-quality grade.',
    'Probe content is JSON data. The declaration prefix is trusted project code; the VM is not a security sandbox.',
    'Preflight does not authenticate execution, prevent output leakage or establish human acceptance.'];
  process.stdout.write(JSON.stringify(report, null, 2) + '\n');
  if (!report.ok) process.exitCode = 1;
}
try { main(); }
catch (error) { process.stderr.write(error.name + ': ' + error.message + '\n'); process.exitCode = 2; }
