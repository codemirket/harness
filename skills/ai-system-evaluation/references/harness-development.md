# Evaluate harness changes through delivered work

Use this workflow when changing shared routing, skill content or agent execution
guidance. Registration checks prove availability; they cannot establish that an
agent read the right advice, used the tools correctly or delivered a better result.

## Prepare comparable runs

Use the installed `skill-catalog/scripts/harness.py` engine, or `python3 ai.py` from
the source checkout. The foundation plugin carries the same engine and fixtures.
Run `eval list` to inspect seven development cases: frontend hierarchy, refund
totals, technical CLI documentation, paginated HTTP integration, source-based
research, rollout analysis and resumable database backfill. Each has a task, seed workspace, verifier and rubric.

```sh
python3 ai.py eval list
mkdir -p build/evaluation-runs
python3 ai.py eval prepare --case engineering-ledger-total \
  --output build/evaluation-runs/ledger-candidate \
  --model 'exact model/version or explicit unavailable label' \
  --condition candidate --settings 'reasoning, tools and budget'
```

The destination must be absent. Preparation records case/control-file hashes and
a fingerprint of the supplied harness source. `--harness-source /absolute/snapshot`
can identify retained baseline instructions. The recorded model, settings and
source fingerprint are provenance labels, not proof that a runtime used them.

Assign the task to the authorized agent with ownership confined to `workspace/`.
Keep `run.json`, the verifier, task and rubric outside its assignment. Read only
the baseline or candidate guidance assigned to that trial. Record actual skill
reads, tool observations and failures separately from the agent's selection claims.
Do not revise the case or immutable fixture inputs to make an output pass. Changed
case definitions require fresh preparation; rechecking an old output is a replay,
not a new independent trial.

## Verify and inspect the output

```sh
python3 ai.py eval check --run build/evaluation-runs/ledger-candidate
python3 ai.py eval report --run build/evaluation-runs/ledger-candidate
```

`check` executes the copied verifier and candidate code with normal host
permissions. It bounds duration and output; the scratch directory and process
limits are not an OS sandbox. Use only the authorized fixtures and outputs. No
dependency installation, runtime provisioning, model call or account connection
is performed by this CLI. Review code before running unfamiliar candidate code.

The frontend, documentation, research, analysis and database cases require qualitative review after
their checks. Inspect the actual required output files against `rubric.md`; for
frontend work view every required browser screenshot and exercise the behaviors.
An existing PNG proves neither good composition nor usable interaction. The
frontend verifier checks dimensions, presence and source preservation, not beauty.

Use `eval review --help` to record the reviewer, `human` or `agent`, decision,
rationale and each required evidence path relative to the workspace. Record a
human review only when a human actually performed it. Evidence must include the
case's required outputs and a new or changed artifact. Final acceptance needs a
passing check as well as the required review. Output or
run-metadata changes make prior checks/reviews stale.

## Interpret results

Inspect checks and reviews separately. Report which task and failure each check
covers, the reviewer kind and any missing evidence. Local JSON records are editable;
consistency/hash validation prevents accidental stale or inconsistent claims but
does not attest runtime execution, reviewer identity or approval.

`eval report` exposes the current `review_kind`, `reviewer` and `accepted_by`.
`accepted` retains the combined check/review outcome; it is not human acceptance.
`accepted_by: none` includes passing cases that require no review. Stale or invalid
reviews cannot supply acceptance identity. Kind counts include only valid current
reviews; decision counts remain separate.

These public cases are for development and regression checks. Both conditions
passing a small repair establishes neither superiority nor quality on real
projects. Keep inputs, models, settings, tool access and budgets comparable; use
repeated trials and authorized held-out tasks before claiming a general gain.
Record visual owner assessment, failure categories and effort alongside success.
Keep failed cases, fix the responsible layer and rerun affected checks.
