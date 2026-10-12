// Preserve useful axe diagnostics within the report contract, marking every cut.
export function buildReport(result, context) {
  let truncated = false;
  const text = (value, limit = 1024) => {
    const source = String(value ?? '');
    if (source.length > limit) truncated = true;
    return source.slice(0, limit);
  };
  const list = (values, limit) => {
    if (values.length > limit) truncated = true;
    return values.slice(0, limit);
  };
  const checks = values => list(values, 16).map(check => ({id: check.id, message: text(check.message)}));
  const simplify = rules => list(rules, 200).map(rule => ({
    id: rule.id,
    impact: rule.impact,
    description: text(rule.description),
    help: text(rule.help),
    helpUrl: text(rule.helpUrl),
    tags: rule.tags,
    nodes: list(rule.nodes, 32).map(node => ({
      target: list(node.target, 8).map(item => Array.isArray(item)
        ? list(item, 8).map(selector => text(selector, 512))
        : text(item, 512)),
      html: text(node.html),
      impact: node.impact,
      failureSummary: text(node.failureSummary, 2048),
      any: checks(node.any),
      all: checks(node.all),
      none: checks(node.none),
    })),
  }));
  const report = {
    testEngine: result.testEngine,
    testEnvironment: result.testEnvironment,
    ...context,
    violations: simplify(result.violations),
    incomplete: simplify(result.incomplete),
    passes: simplify(result.passes),
    inapplicable: simplify(result.inapplicable),
    manual_review_required: true,
  };
  report.truncated = truncated;
  return report;
}
