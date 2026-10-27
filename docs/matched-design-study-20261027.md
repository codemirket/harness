# Matched design study runner and native route calibration

D3 of the [remaining design plan](../planning/remaining-design-work.md) now has
an executable matched-study mode. It uses the existing native Codex process
runner; it is not a production Mirket model router or second agent orchestrator.
The four-brief creative cohort is still not run. The calibration below checks
the execution path, not aesthetic quality or a skill advantage.

## Comparison controls

Invoke the extension through `tests/motion-feedback-pilot.py matched`.
Its `prepare`, `verify`, `initial`, `repair`, `review-pack` and `finalize`
commands take explicit disposable root, ready manifest and external seal paths.
Preparation prints a seal digest; subsequent commands require that exact digest
through `--seal-sha256`. Preparation starts no model and copies no credentials.
`--calibration` selects the separate two-cell route check; omit it for the
eight-cell creative cohort.

Within each pair, the brief, prompt, common production contract, rubric, assets,
allowed output paths, model/effort and enabled MCP tools match. Tool config
differs only in the absolute private state path needed to isolate each process.
The treatment installs materialized copies of selected specialist skills; the
control has no project specialist directory. Both conditions can render and
inspect outputs. This differs from the historical source-only/feedback pilot,
which intentionally varied capability access and cannot measure skill benefit.

The ready manifest binds explicit Codex, Mirket, Node, design/video adapter,
Chrome, FFmpeg and ffprobe files, plus complete adapter/browser directory
hashes. Runtime-tree symlinks must resolve inside their bound tree. The study
stores the current Git HEAD, runner snapshots/hashes, skills and common inputs.
The external seal is retained outside author projects; rewriting an inner
manifest does not update that seal or the lead-supplied digest. This detects
drift under the trusted-lead workflow, not an attacker who can rewrite the whole
machine and the caller's arguments.

Disposable author roots must be outside the repository and have no ancestor
`AGENTS.md`. The usual `.codex/task-state` location inherited global guidance,
so the actual author projects ran under `/private/tmp`. `HOME`, `CODEX_HOME`
and temporary directories are isolated. The runner uses the isolated strict
Codex config, `--ignore-rules`, `--ephemeral`, the explicitly selected
`gpt-6-astra`/`high`, and user-authorized `--approve-for-me`. It does not use
approval-bypass flags or change live configuration. `--ignore-user-config`
would discard the deliberately supplied MCP config and is therefore not used.
Only the actual launch stages an owner-readable credential; exclusive no-follow
creation and directory-FD cleanup prevent following/replacing a credential link
or deleting live auth if a disposable pathname changes. Abrupt host death is
not a proven cleanup path; a stuck reservation is not automatically retried.

The creative cohort has four briefs × two conditions, a 240-second ceiling per
initial turn, 180 seconds per repair, four repairs total, 2700 aggregate model
seconds and one 45-minute wall window including review. Full turn time plus a
15-second shutdown allowance is reserved before launch. Actual invocation time,
including termination, is charged; independent file hashing is not mislabeled
as model time. Excessive time remains charged and marks an overrun. Ceilings
do not promise every possible repair will fit. The runner permits only frozen
cells, in frozen alternating condition order, with one initial/repair attempt
per cell. Timeouts, failed launches, interruptions and permission refusals stay
recorded. A native `turn.completed` event is required even for an author-turn
completion claim; it still does not establish artifact quality.

Repair allocation waits for all initial results and reviews of eligible
completed cells. Blocking findings rank before major findings, then frozen
cell order. Reviews bind the phase, rubric and unchanged saved artifact; each
finding names an inspected artifact, location and consequence. The current
project must still match the reviewed initial snapshot before a repair starts.
Minor-only feedback does not trigger an automatic repair. No new specimen,
delegate, empty review template or phase retry resets the budget.

Neutral packs contain the brief, rubric, requested editable outputs, local
font and saved image/video evidence. They exclude specialist files, producer
instructions, model/config fields, event logs and author review prose. A review
must bind an unchanged generated pack, identify inspected saved artifacts and
have a reported timestamp after pack creation within the shared window.
Authored source/comments/style may still reveal treatment; this is not
authenticated blindness. D4 needs independent capable reviewers with these
packs; the lead remains responsible for integration and suspect findings.

Finalization retains initial and post-repair findings, failures and unrun cells;
it requires review records for completed author turns and closes the study.
Its result is `recorded_not_quality_certified`. It does not silently equate
an empty finding list, a receipt, a model claim or a reviewer identity with
visual truth. D2's richer repair-evidence checker remains separately available;
the matched runner does not claim to synthesize or validate a complete D2
packet from model traces.

## Actual native calibration

Two native Codex turns ran on macOS 26.5.2 ARM64 using the pinned local CLI and
existing private runtime. Both received the same task: editable 160×100 SVG,
paper `#f4efe4` background, teal `#176b65` circle centered at `(80,50)`, radius
30, plus an actual Mirket PNG at `.evidence/preview.png`.

| Condition / neutral artifact | Native turn | Actual outcome |
| --- | --- | --- |
| Specialist / `artifact-caed9e80` | 61.282 s | Read project `vector-art`; real runtime status, prepare/start/job status; saved SVG and PNG |
| No specialist / `artifact-674f10d8` | 65.416 s | Same real tool path and saved SVG/PNG; no project specialist files supplied |

Both native turns completed without recorded approval/unavailable signals. Both
initial `design_prepare` calls supplied a bare hex digest and were refused as
a changed source revision; both corrected it to a `sha256:` revision token
within the same initial turn. These refused calls remain in the traces. This
shared producer-contract gap was fixed after the closed calibration by making
the token format explicit in the common contract for both future conditions.
The revised sentence was not tested by another model call; the actual successful
calls already demonstrate the required format. No extra calibration turn or
creative-cohort retry was run to erase the first refusals.

The lead opened both saved neutral-pack PNGs with an image-capable tool. An
independent FFmpeg RGB extraction checked 2,472 circle-interior and 12,772
background pixels per image with a two-channel-value tolerance and excluded
the two-pixel radial antialias band. Both had zero mismatches. Dimensions were
independently read with ffprobe. Both SVG and PNG files were byte-identical:

- SVG SHA-256: `8c520cb1357ac2d89561566921ba7d6ed225a1615f2fffd66fefb931ff232f4c`.
- PNG SHA-256: `11d24f490636fecbad71944cb8aae6e587d2a7bc6c7754c0300cf3f4d1e9d4b7`.

These hashes identify the inspected files; the geometry/color oracle and actual
pixel views support the result. A simple circle is not an aesthetic benchmark.
The lead knew condition labels; this calibration did not test blinded creative
judgment. Missing CLI image events were not used to infer lack of inspection.

The closed calibration used 126.699 aggregate invocation seconds, zero repairs,
and approximately 201 wall seconds including review. Its separate ceiling was
two 120-second turns, 270 model seconds including shutdown capacity, a ten-minute
wall window and no repairs. Both staged credentials were removed. A superseded
preparation had zero model launches and an untouched budget; it is retained as
unstarted preparation, not counted as a trial or used to reset a running cohort.

## Evidence, checks and limits

Private evidence is retained at
`/Users/nazmiyilmaz/.codex/task-state/harness-matched-design-20261027/`:
`calibration-binding.json`, the external `calibration-v2-seal.json`, pair equality,
runtime binding, trace summary, both pixel oracles, calibration summary and the
exact executed runner source. The author study, snapshots, neutral packs,
reviews and `study-result.json` remain at
`/private/tmp/mirket-matched-design-20261027-v2/`.
`calibration-evidence/` under the durable task-state directory also retains
the frozen inputs, artifacts, traces, packs and reviews, excluding session
homes and credentials; it is an archive, not a relocated runnable study.
The retained seal SHA-256 is
`fbb178a6b9093950f1326e807c9f698ae9ef64e1381a0e2b9b2910982e52b8d7`.
The post-calibration producer clarification creates a new runner revision;
resuming that closed old freeze with changed code is intentionally refused.

All 111 affected Python tests passed, including 24 matched-study tests and 18
shared motion-runner tests. Tests cover pair equality, credentials, path
traversal, seal/closure/guidance drift, neutral-pack bypass, stale reviews,
budget exhaustion/overruns, failed launch, interruption and no retries.
Simulated denial/unavailability tests establish failure-record behavior; they
are not a new native host-approval-denial qualification.

`tests/mcp-smoke.mjs` passed using explicit private Node and debug Mirket paths:
actual MCP inventory/status, stale release, absent private payloads, unknown
jobs, configured pairing and stale source refusal on Darwin ARM64. Historical
[native design approval denial](m4-codex-design-skill-tool-qualification.md)
remains separately scoped to its earlier `never` policy. This calibration used
automatic review and did not deliberately provoke a host denial. The running
CLI/model configuration is recorded; Mirket does not enforce provider identity
or intercept native delegation.

No Rust, adapter, production dependency, native skill or catalog input changed.
Catalog refresh reported no drift and diff checks passed. Rust/adapter build
gates were not rerun or represented as fresh evidence for this Python-only
runner change. No live installation, release or remote Git state changed.

Remaining: the eight-cell creative cohort, independent UI interactions and
visual/motion scoring, bounded repairs with retained residual defects, and
native agent-choice/interruption/denial qualification. GUI remains waived and
unverified, Windows unqualified, and catalog `planned-not-run` acceptance
unchanged. The active video engine remains HyperFrames; no host-browser render
switch or Remotion dependency was introduced.
