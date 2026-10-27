# Video authoring diagnostic pilot — 2026-10-27

This is the record of a newly frozen, finite diagnostic pilot. It is
independent of the earlier motion-authoring trials and does not alter their
evidence or repair budgets. The first draft root is retained as a
pre-execution revision; it used placeholder diagnostics, received zero model
calls, and is not qualification evidence:

`/Users/nazmiyilmaz/.codex/task-state/harness-video-diagnostics-20261027/pilot`

That draft’s frozen protocol SHA-256 is
`22b7edf66cbb6163aaa8eb8c3f4ec8ab7b10d38b6201dcf591b54f587a516573` and its
placeholder seed SHA-256 is
`2e0ca2488dfd1985a77cae087f11ed6f8c2eba7185706b6eb4b8426eb0c603c5`.
Neither is used for the corrected final pilot.

The corrected final freeze and completed authoring run are at:

`/Users/nazmiyilmaz/.codex/task-state/harness-video-diagnostics-20261027/pilot-qualified`

An intermediate corrected-preflight root (`pilot-final`) also received zero
model calls and is retained only as pre-execution history. The final 24-cell
manifest is
`/Users/nazmiyilmaz/.codex/task-state/harness-video-diagnostics-20261027/pilot-qualified/final-24-cell-manifest.json`.
The export-oriented supplemental manifest, including expected files, font and
caption selection, per-cell snapshots, corrected JSONL usage, and captured
scope/visual-message audits, is
`/Users/nazmiyilmaz/.codex/task-state/harness-video-diagnostics-20261027/pilot-qualified/final-24-cell-export-manifest.json`.

## Corrected pre-execution design

The corrected protocol is frozen after the lead supplied the actual read-only
production checker and valid seed fixture. It covers six new tasks, two
conditions, and two repeats per cell: 24 authoring runs total.

Preparation takes explicit `--root`, `--diagnostics-checker`, `--seed-fixture`,
`--config`, and `--font` paths. The runner has no required personal-home or
installation path; its default root is only a configurable convenience and
execution still requires an approved lead manifest.

| Task | Independent structural target |
|---|---|
| `glasshouse-product` | new Glasshouse facts, three beats, editable HTML/CSS/JS, captions |
| `pebble-motion` | new Pebble marker enter/pause/settle motion and resting state |
| `cinder-vector` | new Cinder editable SVG groups, contact point and reversal |
| `malformed-root-cues` | repair an authored missing root marker and malformed two-cue source |
| `valid-control` | preserve an already-valid source without unnecessary repairs |
| `refusal-local-asset` | refuse a missing/unselected local asset and proprietary remote font |

The baseline condition uses the skill snapshots from repository HEAD
(`0fdad609fccfc58b97c1dda68f373fa0dd2d3c91`, as returned by `git rev-parse
HEAD` at freeze time) and no executable diagnostic or
seed. The diagnostics+seed condition uses the current working motion/video
skill resources (including the updated authoring reference), the actual
read-only production checker, its diagnostic reference, and the valid seed
fixture. This is an intervention bundle, not an isolated checker-only or
reference-only variable. Seed SHA-256 and checker SHA-256 are recorded in the final
`frozen/` manifest; no model or validator may mutate them. Both conditions
share the same task contract hash; only the explicitly named diagnostics+seed
resources differ.

Both conditions use the same task briefs, output contracts, local Noto Sans
fixture, and deterministic values where applicable. Captions use production
semantics: `end_frame` is exclusive, adjacent cues may share a boundary, and
coverage is 0 through 36. The malformed-root task begins with a real two-cue
authored input; the valid-control task begins with a valid source and tests
unnecessary-repair behavior. The independent oracle calls the same shared
production preflight in both conditions, then applies separate task-specific
assertions. Baseline feedback records the checker’s actual stable error code,
while treatment feedback retains the same full findings plus detailed
diagnostics. Model self-report is never used as success evidence.

## Model and budget freeze

The configured model settings were read from the current private Codex config
before the draft preparation and are carried into the final freeze in
`frozen/model-settings.json`:

- model: `gpt-6-astra`
- reasoning effort: `high`
- config SHA-256: `58519db0c8d95e39a5566f7660867333ab37543a3bfce6e676fc99a3cc66aeb3`

The runner passed these exact values explicitly; it did not choose a model.
Each run had a fixed 180-second model-phase wall cap: 120 seconds for initial
authoring and, only when an independent finding existed, 60 seconds for one
repair. This is not an end-to-end latency measurement. The shared repair ceiling is 24, with no retries,
resets, or new-specimen budget resets. Each run retains initial result,
independent findings, optional repair/final result, scoped file deltas,
elapsed time, token usage from JSONL usage events, and approval/blocked signals.
The JSONL stream may not expose a provider-side model identity; if that remains
unavailable, results will report the exact configured selection and mark backend
identity as unverified rather than claiming an isolated model comparison.

Every trial freezes prompt, budget, metadata, dependency and protected-input
hashes. It retains a pre-turn project snapshot, initial authored snapshot, and
final snapshot; protected brief/font/skill/diagnostic files are rechecked before
each phase. The ledger records initial, repair and final phases temporally and
rejects duplicate phases or retries. `valid-control` must finish with zero
scoped file changes.

## Launch gate and execution

Execution was gated on an approved diagnostics manifest
containing explicit absolute paths for Codex CLI, private Codex config/auth
home, Mirket, Node, adapter, Chromium, FFmpeg and FFprobe, plus the exact
frozen seed digest. The corrected runner rejects missing paths, seed mismatch,
non-approved manifests, altered frozen inputs/dependencies, completed trial
directories, caption contract regressions, duplicate ledger phases, and any
attempt to exceed the fixed budgets. The lead supplied that manifest, and all
24 cells ran in the frozen order with no retries.

## Authoring outcomes

The approved run completed all 12 baseline/treatment pairs (24 cells) with no
retry. It recorded 24 initial results, 24 final results, and 5 one-repair
records against the shared ceiling of 24. The final manifest reports 17 cells
passing the complete authoring contract, 19 passing the static contract, and 7
failed cells; failures are retained rather than retried. The configured model
was `gpt-6-astra` with `high` reasoning. Initial JSONL usage was observed for
17 cells and unavailable for 7 timed-out cells. The observed initial totals
were 2,112,132 input, 1,838,464 cached-input, 34,836 output, and 4,840
reasoning-output tokens (cache-write input was 0); unavailable cells are not
treated as zero. The supplemental manifest retains the per-cell derivation.
The original runner already read top-level usage events, including
`turn.completed`. The first aggregate incorrectly described all usage as
unavailable; the supplemental derivation corrects that summary. The post-run
parser/test now distinguishes absent usage from zero and includes cache-write
tokens. It does not rewrite the frozen raw event records or change the
executed pilot version. The exact executed runner
is preserved at
`/Users/nazmiyilmaz/.codex/task-state/harness-video-diagnostics-20261027/pilot-qualified/executed-video-authoring-pilot.py`
with SHA-256
`c75ce007389c513d55c3e7e2d981c4a234a7720c773a312437b6c6b06ee7415e`, matching
the pre-execution record. The post-run parser source hash is
`57aa85d9c20865955544fafd4a1bae3984445810c6349bd0df256d4ab13519bc`; its only
purpose is corrected audit reporting.

All five repair phases timed out before a completion usage event. Their usage
is unavailable, not zero, so the initial totals above are a partial lower bound
on total pilot token cost.

No positive model claim of visual inspection was captured. The event audit
retains explicit negative/unavailable statements and 13 cells with captured
file-change events; those events do not prove that no transient shell write
occurred outside a project. The malformed-root-cues task disclosed its needed
repairs in the brief and all four specimens solved them initially. The five
later repair phases were creative missing-deliverable/timeouts, so this pilot
does not establish improved repair efficiency or isolate the diagnostic
bundle’s individual contributions.

The production preflight ran in both conditions and is static-only. The authoring
runner did not render video, decode frames, inspect pixels, or establish motion
semantics. The subsequent [independent export and pixel review](video-authoring-pilot-exports-20261027.md)
exported all 15 available video sources, reviewed 75 decoded samples and
retained four refusals plus five missing-deliverable failures. Two exported
sources still count as authoring timeouts. The lead confirmed the visual
observations against 41 byte-distinct saved PNGs, withdrew an incorrect review
finding and recorded the minimal controls' unqualified font use.
Cells with missing deliverables remain authoring failures, not pending passes.
Windows qualification and the waived Desktop GUI check remain unqualified.
