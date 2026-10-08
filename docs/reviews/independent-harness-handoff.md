# Ready handoff: finish the independent Opus review

Continue in this harness repository. The user requested two independent reviews
at maximum effort: Claude Code CLI **Opus 5.5 (`claude-opus-5-5`, `max`)** and native
Codex **Astra (`gpt-6-astra`, `ultra`)**. The user chose a ready handoff; **do not
schedule a follow-up**.

Astra has approved the final implementation and README with limits. Opus completed
R1 and R2 with `APPROVE_WITH_LIMITS`, but has not approved the final repairs. Its
final calls reached the Claude session limit. The official CLI reported a reset
at **2026-10-08 19:30 Europe/Istanbul**; check actual availability rather than
assuming the reset grants access. No substitute model, lower effort or billing
fallback is authorized by this handoff.

Read [review status](independent-harness-review.md),
[provenance](../evidence/independent-harness-review-2026-10-08.json) and
[verification](../evidence/independent-harness-verification-2026-10-08.json).
They are primary-agent context and **must not be passed to Opus if they reveal the
other reviewer's opinions**. No commit or push was performed in this review task.
Preserve the existing uncommitted work and follow current user authorization.

## Frozen assignment

The ready review input is `build/dual-independent-review/opus-r5`, with 202 source
files and aggregate SHA-256:

`4f0e8ec6540686a1a95978b03897ef0b4c1d2efe842d78bfc6051b26428362b2`

It contains the shared acceptance criteria, neutral revision facts, primary
execution evidence and **only Opus's own R2 report**. It excludes the final status
records and all Astra reports. Preserve this isolation. Review code and guidance,
not an expected verdict. Keep the completed all-domain assessment where unchanged.

Before invoking the reviewer, verify every source hash against its `SNAPSHOT.json`
and compare current implementation inputs with the durable verification record.
The frozen R5 inputs should match current source; later-added review reports and
status records are intentionally outside the snapshot. If implementation has
changed, first establish the new diff and relevant checks, then give both reviewers
identical updated source independently. Do not silently reuse an old approval.

If ignored build files have been removed, reconstruct an isolated snapshot using
`final_snapshot.files` from the provenance JSON, requiring every copied current
file to match its recorded hash. Add the preserved acceptance criteria, the exact
Opus R2 report and the durable **verification-only** JSON. Do not copy the review
status/provenance record itself or any Astra report. Recreate a neutral prompt
covering the final diagnostic, alias, export, evidence and README deltas. If hashes
no longer match, treat this as new source requiring reassessment.

## Run once when the exact CLI model is available

The prepared driver verifies explicit effort forwarding and retains allowlisted
model metadata. It uses the production bounded runner, the existing subscription
route, Read/Glob/Grep only, a 1 MiB output cap and a 1,200-second timeout:

```sh
python3 build/dual-independent-review/run_opus_r5.py
```

The script writes new `opus-max-r5-{result.json,model.json,report.md}` files under
that ignored task directory. Inspect an existing result before another run; never
overwrite or discard an already accepted report. The production CLI alternative
is below if the observational driver is unavailable; it does not retain the same
allowlisted model-usage metadata:

```sh
python3 ai.py delegate claude \
  --project build/dual-independent-review/opus-r5 \
  --model claude-opus-5-5 --effort max --timeout 1200 \
  --prompt-file build/dual-independent-review/r5-prompt.txt \
  --output build/dual-independent-review/opus-max-r5-result.json
```

If the CLI still reports a limit or failure, record it and stop. Do not infer a
verdict, loop retries, change authentication, or fall back to another provider.
Reviewer restrictions are CLI/context controls, not an OS sandbox.

## Completion

Inspect the actual opinion. For a concrete defect, reproduce it, repair only the
necessary source, run affected gates and obtain independent reassessments from
both requested models against the same final source. Never share the other
reviewer's opinion to induce agreement. Optional suggestions and unverified
platform/outcome limits are distinct from implementation blockers.

If the final opinion approves with limits and no required repair remains, preserve
its report and model evidence under `docs/reviews/independent-harness/` and update
the review status/provenance with the actual verdict and approved fingerprint.
Do not replace `APPROVE_WITH_LIMITS` with an unconditional claim. Retain the full
history, including failed/cancelled runs and the session-limit interruption.

The current checks are **471 tests in 35.759 seconds**, 30 normal-plus-specialist
and 24 portable exact copies, preserving migration/permission checks, export
checks and one compatible pinned payload for each of 14 eligible archive sources.
These do not prove automatic skill invocation, perfect design, production safety,
physical-device readiness or untested domain outcomes. The completed reports
contain the all-domain capability matrices and these limits.
