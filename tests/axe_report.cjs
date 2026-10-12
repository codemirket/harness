// Exercise the production reducer without a browser or optional runtime install.
const assert = require('node:assert/strict');
const path = require('node:path');
const {pathToFileURL} = require('node:url');

(async () => {
  const helper = pathToFileURL(path.resolve(__dirname, '../runtime/packages/axe-core/report.mjs'));
  const {buildReport} = await import(helper.href);
  const node = () => ({target: ['#subject'], html: '<button></button>', impact: 'critical',
    failureSummary: 'Add an accessible name.', any: [{id: 'button-name', message: 'Missing name.'}], all: [], none: []});
  const rule = () => ({id: 'button-name', impact: 'critical', description: 'Buttons need names.',
    help: 'Add a button name.', helpUrl: 'https://dequeuniversity.com/rules/axe/4.14/button-name',
    tags: ['wcag2a'], nodes: [node()]});
  const results = () => ({testEngine: {name: 'axe-core', version: '4.14.0'}, testEnvironment: {},
    violations: [rule()], incomplete: [], passes: [], inapplicable: []});
  const context = {url: 'http://localhost:3000', final_url: 'http://localhost:3000',
    viewport: {width: 640, height: 360}, visible_elements: 1, actions: []};
  let probes = 0;
  const check = (edit, inspect, expected = true) => {
    const input = results();
    edit(input);
    const snapshot = JSON.stringify(input);
    const output = buildReport(input, context);
    assert.equal(output.truncated, expected);
    assert.equal(JSON.stringify(input), snapshot, 'reducing diagnostics must preserve source results');
    assert.equal(output.manual_review_required, true);
    assert.equal(output.url, context.url);
    inspect(output);
    probes++;
  };
  check(() => {}, output => assert.deepEqual(output.violations[0], rule()), false);
  for (const [field, limit] of [['description', 1024], ['help', 1024], ['helpUrl', 1024]]) {
    check(input => {input.violations[0][field] = 'x'.repeat(limit);}, output =>
      assert.equal(output.violations[0][field].length, limit), false);
    check(input => {input.violations[0][field] = 'x'.repeat(limit + 1);}, output =>
      assert.equal(output.violations[0][field].length, limit));
  }
  for (const [field, limit] of [['html', 1024], ['failureSummary', 2048]]) {
    check(input => {input.violations[0].nodes[0][field] = 'x'.repeat(limit);}, output =>
      assert.equal(output.violations[0].nodes[0][field].length, limit), false);
    check(input => {input.violations[0].nodes[0][field] = 'x'.repeat(limit + 1);}, output =>
      assert.equal(output.violations[0].nodes[0][field].length, limit));
  }
  for (const kind of ['any', 'all', 'none']) {
    check(input => {input.violations[0].nodes[0][kind] = Array.from({length: 16}, () => ({id: kind, message: 'x'.repeat(1024)}));}, output =>
      assert.equal(output.violations[0].nodes[0][kind].length, 16), false);
    check(input => {input.violations[0].nodes[0][kind] = [{id: kind, message: 'x'.repeat(1600)}];}, output =>
      assert.equal(output.violations[0].nodes[0][kind][0].message.length, 1024));
    check(input => {input.violations[0].nodes[0][kind] = Array.from({length: 17}, () => ({id: kind, message: 'check'}));}, output =>
      assert.equal(output.violations[0].nodes[0][kind].length, 16));
  }
  check(input => {input.violations[0].nodes[0].target = ['x'.repeat(512)];}, output =>
    assert.equal(output.violations[0].nodes[0].target[0].length, 512), false);
  check(input => {input.violations[0].nodes[0].target = ['x'.repeat(513)];}, output =>
    assert.equal(output.violations[0].nodes[0].target[0].length, 512));
  check(input => {input.violations[0].nodes[0].target = Array(9).fill('#item');}, output =>
    assert.equal(output.violations[0].nodes[0].target.length, 8));
  check(input => {input.violations[0].nodes[0].target = [Array(9).fill('#frame')];}, output =>
    assert.equal(output.violations[0].nodes[0].target[0].length, 8));
  check(input => {input.violations[0].nodes[0].target = [['x'.repeat(513)]];}, output =>
    assert.equal(output.violations[0].nodes[0].target[0][0].length, 512));
  check(input => {input.violations[0].nodes = Array.from({length: 32}, node);}, output =>
    assert.equal(output.violations[0].nodes.length, 32), false);
  check(input => {input.violations[0].nodes = Array.from({length: 33}, node);}, output =>
    assert.equal(output.violations[0].nodes.length, 32));
  for (const kind of ['violations', 'incomplete', 'passes', 'inapplicable']) {
    check(input => {input[kind] = Array.from({length: 200}, rule);}, output =>
      assert.equal(output[kind].length, 200), false);
    check(input => {input[kind] = Array.from({length: 201}, rule);}, output =>
      assert.equal(output[kind].length, 200));
  }
  process.stdout.write(`${JSON.stringify({ok: true, probes, helper: 'runtime/packages/axe-core/report.mjs',
    missing_name_probe: 'synthetic report reduction; real browser runtime probe remains separate'})}\n`);
})().catch(error => {console.error(error); process.exitCode = 1;});
