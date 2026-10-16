# Synthetic judge calibration

The seven standalone original MIT controls in `../controls/` contain zero model
executions. Two CSS-only composition contrasts have identical content and state
logic within each pair and different visual languages across pairs. The extra
controls test an identical duplicate, a source-order perturbation, and a polished
false save. These are teaching/measurement controls, not real product work or
evidence that a model improved. No human calibration or acceptance is claimed.

`control-spec.json` freezes independent expected relations and located anchors.
The craft dimensions and weights are the experiment's `../craft-rubric.json`;
scores remain continuous from 0 to 100. No individual target score is prescribed.
The two composition positives must each exceed their paired negative by ten
points, and the exact duplicate must differ by no more than five points. The
order/label checks and false-state check stay separately visible. Do not lower
thresholds or reinterpret an initial score after reading source or identity.

Run the controls with the already available Node, Playwright and Chrome:

```sh
node evals/design-quality/controls/run-controls.cjs /absolute/private-qa/controls
node evals/design-quality/calibration/pack.cjs /absolute/private-qa/controls /absolute/private-qa/calibration
```

The first command saves raw errors, per-step outcomes, DOM snapshots and full-page
plus viewport images. `controls-check.json` checks CSS-only identity, a true local
write after reload, a convincing false success, exact duplicate pixels and stable
identity under content-order changes. A localStorage test proves that local state
only. It does not establish server persistence, real dispatch or collaboration.

The second command creates two render-only judge packets with neutral labels,
counterbalanced pair placement and independently permuted orders. It copies PNGs
and raw briefs only, not source, decision, observer assertions, source paths or
arm labels. The mapping key and source bindings are saved outside those packets.
Keep judges within their assigned packet until render-only judgments are locked.
Each judge should inspect wide and narrow full-page/viewport views and saved/focus
states. It must record two located observations for every dimension, then lock
scores before source/functional review or peer discussion. A packet has five
pairs: two composition families, exact duplicate, content order, and initial/
post-save visual parity between a true effect and the polished false effect.

`judge-template.json` describes the rating format. Save all original judgments.
Then run:

```sh
node evals/design-quality/calibration/validate-judgment.cjs /absolute/judgment.json /absolute/private-qa/calibration/key.json
```

The validator checks completeness, numeric ranges, actual packet image references
and literal hash identity, then computes frozen weighted comparisons. It cannot
verify that a claimed visual observation is perceptually correct. The lead must
inspect the cited region. Pair placement and score differences are retained so
order/label instability is inspectable. If a judge is asked to score a reversed,
relabeled packet later, lock its first judgment and compare same-source scores;
do not replace the first score. Cross-judge counterbalancing alone cannot isolate
an individual judge's order effect.

After that lock, functional inspection should follow the false save through its
separate owner read and reload. The synthetic false control passes the immediate
success text and visible counter while failing owner-storage/reload checks. This
demonstrates a convincing fake that a screenshot-only or success-only evaluator
would accept. A render-only judge must not invent this unseen defect.

If calibration fails, retain the failure and follow the experiment's one-repeat
contingency. No third family is supplied here. Authoring a new family after
failure requires freezing its source and relations before a repeat judgment and
keeping the first failure; it is not an independent preauthored reserve.
