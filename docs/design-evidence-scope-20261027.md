# Design evidence scope and chronology

This implements D1 of the [remaining design plan](../planning/remaining-design-work.md)
after baseline commit `dba8873`. It changes the optional repository checker,
not the installed runtime, native skill payloads or catalog acceptance status.

## Demonstrated defect and correction

The prior checker accepted an unlisted root file even when a packet declared an
output scope. It also accepted a visual report dated before its capture. The
earlier model-repair trial exposed why scope must follow the actual condition:
its prompt permitted in-project `evidence/`, while a fresh-author check assumed
`.feedback/`. A folder name cannot establish an out-of-scope write without the
corresponding frozen permission. See the [trial reconciliation](hyperframes-followup-qualification-20261027.md).

`scripts/check-design-evidence.py` now accepts this optional schema:

```json
"artifact_scope": {
  "files": ["index.html", "film.css", "film.js"],
  "directories": ["evidence"]
}
```

Bind it with `contract.artifact_scope_sha256`, the SHA-256 of UTF-8 JSON using
sorted keys, `ensure_ascii=False` and compact separators `(',', ':')`. Freeze
the policy before the run. Files are exact relative paths; directories permit
descendants. Entries grant permission, so unused optional outputs need not
exist. Required source files, states and captures remain separate requirements.
Existing entries must match their declared file/directory type. Traversal,
ambiguous paths, symlinks, special files, hidden extras and directory-prefix
lookalikes refuse. Only the actual packet file supplied by the caller is omitted
from the inventory; sibling JSON files are ordinary outputs.

A passing result includes every permitted regular file and its hash, plus a
canonical inventory hash. With no scope, the result explicitly says `unrecorded`
and supplies no inventory. Deleting scope while retaining its frozen hash
refuses. Captures, findings and visual reports must occur inside the shared
budget window; findings and reports cannot precede their referenced capture.

This remains trust-based record validation. Rewriting an entire packet and its
hashes is not detectable. Retain the original freeze outside the author's output
tree when running comparisons. The inventory observes one root at check time;
it is neither an OS sandbox nor an audit of writes elsewhere. Reviewer identity,
timestamps and visual statements remain attestations. Repair/source lineage and
finding-resolution evidence are D2, not established by this change.

## Actual artifact journey

On macOS 26.5.2 ARM64, Python 3.15.1, the lead froze a new zero-repair record-check
window at `2026-10-27T20:14:44.012595Z`. This was an unchanged re-export of the
previously repaired composition, not a new author/model trial or a reset of a
closed creative cohort. `tests/motion-authoring-runtime.mjs` used explicit private
Node, Chrome, FFmpeg and ffprobe paths with the existing debug Mirket binary and
compiled adapter. It initialized the real MCP server, checked runtime status,
prepared/started/polled/read the video job and removed its disposable home/state.
The release label was `motion-authoring-20261027`.

The export completed at `20:14:53.726968Z`; ffprobe established 640×360, 12 fps,
36 frames. The runner decoded frames 0, 6, 12, 24 and 35. The lead opened saved
frame 24 with an image-capable tool before recording the review at
`20:15:21.722190Z`: “Log a reading” and its arrow are legible on the light button.
Only that frame was visually inspected in this run. No new continuous playback,
audio, caption, aesthetic-comparison or Windows claim follows.

| Probe | Observed outcome |
| --- | --- |
| Real source/export packet, scope `inputs/` and `export/` | Accepted; 21 regular files inventoried |
| Add `unexpected.txt` at the root | Refused: `file outside artifact scope: unexpected.txt` |
| Report one microsecond before its capture, within the budget | Refused: `visual_report predates its capture` |
| Remove both controlled probes and recheck | Accepted; no artifact repair |

The exact checker from `dba8873` accepted both negative probes against this same
real packet. `baseline-probe-results.json` retains that before/after discriminator.

Private evidence is retained at
`/Users/nazmiyilmaz/.codex/task-state/harness-design-scope-20261027/`:
`frozen-inputs.json`, `export-completed.json`, `visual-observation.json`,
`checker-results.json`, replay helper `check-packet.py`, and
`packet-root/review.json` with `packet-root/export/result.mp4` and decoded pixels.
The export SHA-256 is
`6d81087091f3e5188764ad2cda7582edc84e822ae1be5b0d49c8c68f00b81de8`;
the restored inventory SHA-256 is
`ef8b243365ad18fae93541a7e3c6c80cfffd79082d2db3e75d6031f2a1c8e7e7`.
These locate evidence; neither hash establishes visual quality.

## Regression gates and remaining work

`python3 -m unittest -q tests/test_design_evidence.py tests/test_design_adoption_trials.py tests/test_motion_feedback_pilot.py tests/test_refresh_skill_catalog.py`
passed all 64 tests, including 35 checker tests. The repository-owned catalog
refresh reported no changes. An independent read-only reviewer reran the 64
tests and diff check, reviewed packet exclusion, symlinks, declared types and
chronology, and found no concrete D1 regression or blocker. No production dependency, runtime implementation
or skill reference changed. Rust/adapter compilation gates from the baseline
qualification were not rerun or represented as fresh evidence for this
Python-only slice; the actual existing MCP video path was exercised above.

D2 repair lineage is next, followed by the matched comparison runner and finite
UI/illustration/vector/motion cohort. Catalog `planned-not-run` acceptance,
broader design quality, Desktop GUI (waived), Windows and distribution gates
remain open. The private renderer worked in this run; no host-browser renderer
switch was justified. No live installation, release or remote Git state changed.
