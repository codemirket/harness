# Small role workflow evaluation

Eight original synthetic scenarios: six development cases and two separately
assigned held-out cases. These are public fixtures. A separate split is not a
blind test: anyone who has read these graders is ineligible for a blind claim.
Keep worker inputs separate from graders and reports. Do not give workers the
rubric or expected answers; host permissions, not this folder layout, control access.

Run `mirket dev role-eval suite --project . --suite evals/role-workflow/suite.json`
to inspect the fixed fixture identities. This does not execute any agent.
`mirket dev role-eval check --project . --suite evals/role-workflow/suite.json --report PATH`
checks a saved report and recomputes its aggregates. Paths are project-relative.

## Running a real pilot

1. Copy the suite into task evidence and create one configuration file per arm.
   Replace the template's unknown host/model/settings with actual values. Fix the
   harness/skill source revision, authority, tools, budgets and trial count before
   dispatch. Pin their SHA-256 values in your copied suite. Use comparable values
   across arms except the intentional change; the checker checks identities, not
   the scientific adequacy of the experiment configuration.
2. Dispatch actual host agents against `inputs/` only. Preserve all raw responses
   and relevant host tool traces. For the explicit-skill case, observe retrieval;
   simply saying the skill was used is insufficient.
3. An independent grader inspects the saved output and each `graders/` assertion.
   Save rationale and citations to output/trace evidence for every verdict.
   The deterministic checker checks rubric verdict consistency, not semantic truth.
4. Record every planned case/configuration/trial cell, including failures. A model
   that finishes with an incorrect answer is `task_failed`; inability to start or
   collect a run is `infrastructure_failed`, with null score and empty verdicts.
   Unknown cost and latency remain null; known measurements must be finite,
   nonnegative and at most 1e15 to keep aggregation bounded. Do not retry secretly or remove poor runs;
   create a new explicitly planned experiment for additional trials.
5. Check the report. Inspect disagreements and subgroups before drawing conclusions.
   Repeat with fresh held-out scenarios for real blinded evaluation. This small
   fixture suite cannot establish general quality gains or human acceptance.

## Report schema (version 1)

Top-level: `schema_version`, `provenance` (`host_observed` or `synthetic`),
`suite_sha256`, `checker_sha256`, `trials`, `limitations` (nonempty).
Each trial: `case_id`, `configuration`, `trial` (one-based), `input_sha256`,
`rubric_sha256`, `configuration_sha256`, `outcome` (`passed`, `task_failed`, or
`infrastructure_failed`), `score`, `verdicts` (criterion ID to boolean), `evidence`
(array of `{path, sha256}`), `rationale`, `cost_usd`, `latency_ms`.
A scored trial must bind saved evidence. Score equals passed criteria / all
criteria; passed requires every criterion. Aggregates are separated by arm and
split; infrastructure failures are explicitly excluded from scored means and
retained in counts. Costs are totaled only when every cost is known.

`tests/role_evaluation.rs` constructs an explicitly synthetic example report
solely for checker regression tests. It is not a model run or role-quality result.
The suite pins the checker source bytes and checks against the compiled checker;
source edits require intentional suite re-pinning and a freshly compiled binary.
Confirmation suites may use an empty `development` array and only fresh held-out
cases. The held-out set must remain nonempty, every case must belong to its stated
split, and every planned arm/repeat cell must still be recorded. A confirmation
does not need invented development runs to satisfy the record format.
No providers, model dispatch, automatic graders or new dependencies are included.
