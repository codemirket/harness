# Illustration quality evaluation

Eight original briefs span painterly scenes, geometric families, conceptual
editorial work, accurate diagrams and branded composition. This is an executable
fixture inventory for the existing `mirket dev role-eval` checker, not a report
of model execution or measured quality. No single reference defines success.

This suite is opt-in evaluation material. Ordinary design work follows the
[image-to-code workflow and bounded quality gates](../../docs/visual-quality.md);
it does not start a sample campaign or require running this suite.

Inspect the suite with:

```sh
mirket dev role-eval suite --project . --suite evals/illustration-quality/suite.json
```

Four development and four transfer cases have two trials per arm, 32 planned
executions. All public inputs are inspectable; the `held_out` split does not make
them blind. Use fresh privately authored briefs before claiming unseen transfer.
Keep private outputs, reference captures, configuration and task history outside
shipped source. The repository license covers these original briefs.

## Freeze before dispatch

Copy the suite to task evidence. Replace configuration templates with actual
host, model identity, effort, source/binary and guidance hashes, tools, production
capabilities, reference assets/provenance, budget and stopping rule; update the
suite's bindings. Keep the user's selected model and effort constant. If a router
cannot pin that choice, record the limitation or use a host that preserves it;
do not attribute another model's output to the selected model.

First compare one intervention at a time: reference retention, direction/critique
guidance, or medium selection. Both arms must have comparable tool availability;
a new producer is a separately labelled capability experiment. Do not conflate
an image model comparison with a reasoning-effort comparison. Freeze any relevant
reference set identically across arms. Several references may contribute different
principles; they are not a template to imitate. A task can need no external image.

Supply the same raw brief and ordinary host instructions. Give workers only their
own output directory and assigned context. Keep graders and other outputs out of
their context. Counterbalance arm order per brief/repeat. Preserve every started
trial, including timeouts, failed production and rejected results. A budget-ended
unfinished asset is a task failure, not a missing observation. Missing runtime
infrastructure remains a separately recorded infrastructure failure.

## Inspect and compare

Capture final source, exports and full/use-size/detail views, plus producer/tool
traces and any revision history that explains the result. Open the actual files;
do not grade a creator's description. Use the JSON rubrics for binary correctness
and delivery guards in the existing version-1 report. Bind craft ratings and
review packets as evidence, using [craft review](craft-review.md). Check reports
with the existing role-eval `check` command; it validates identity/consistency,
not visual appeal or reviewer independence.

Report per-brief craft changes and defects, judge disagreement, repeated-trial
variation, observed time/cost and all incomplete cells. Average repeats within
brief before comparing arms; repeats and judges are not independent briefs.
Do not select the best run, silently retry failures, change frozen criteria or
use a single high-scoring image to claim broad improvement. User preference is
`not_collected` until the user actually compares the results. A release claiming
quality gains needs fresh transfer evidence, not only fixture/test validity.
