# HyperFrames revised-skill and model-repair follow-up — 2026-10-27

The user authorized this bounded follow-up after the earlier pilot concluded.
Two fresh Astra/high authors avoided the earlier intermediate-color contrast
failure. A third turn repaired the preserved real failure. Independent exports
and individual-frame review found no remaining demonstrated defects in these
three artifacts. Actual MP4 playback completed at 1× in Codex's in-app browser.
This is narrow outcome evidence, not a general design-quality or model ranking.

## Frozen scope and results

The follow-up used the revised `motion-direction` and existing `video-production`
skills, the same fictional Harborline brief and local font, and a 640×360,
12-fps, 36-frame MP4 profile without audio/captions. Both fresh authors saw the
updated paragraph about foreground/background interpolation in their recorded
file-read outputs. The repair author saw the same paragraph and the independent
finding identifying the original frame-24 defect. No target implementation was
prescribed in that finding.

The two fresh authors each kept a readable button palette and transferred focus
with a separate ring while softening the card emphasis. The repair author
changed only `film.js`: it switched paired action colors together and retained
the ring as a separate smoothly animated property. Layout, copy, font, brief,
protected skills and diagnostics stayed unchanged in the repair project.

| Trial | Model turn | Independent outcome |
| --- | --- | --- |
| Fresh author r1 | 165.067 s; completed without approval refusal | All 36 decoded frames reviewed; C2/T2; no repair required. |
| Fresh author r2 | 154.839 s; completed without approval refusal | All 36 decoded frames reviewed; C2/T2; no repair required. |
| Preserved-failure repair | 183.645 s; completed without approval refusal | Before frame 24 reproduces unreadable action text; all 36 repaired frames reviewed; C2/T2, no observed regression. |

C = copy/layout and T = card entry, reading hold and action-focus transfer,
each 0–2. Intentional entrance opacity is not automatically a defect. A ring
can communicate focus; the brief does not require changing button fill. Only
three strings are exact requirements; additional fictional descriptive copy
is allowed, while invented metrics are not.

All three authors completed actual Mirket runtime/prepare/start/status/result
calls using Codex automatic approval review. Each independent export was made
from a separate copy of the saved source through the real Mirket/HyperFrames
route. All 36 model-repair PNGs matched the author's decoded PNG bytes; all
independent fresh-author frame hashes were also present in their saved decodes.
These matches establish correspondence, not that the authors looked at pixels.
Lead and peer saved-pixel inspection supplies the separate visual evidence.

There was one shared follow-up limit: 30 minutes, 1800 aggregate model seconds,
two repair turns, no retries. Actual model time was **503.551 seconds**; one
repair turn was used. Artifact production/review/playback finished in **645.91
seconds**, within the deadline. The prior pilot's exhausted four-repair ledger
was left unchanged; this was the expressly authorized fresh follow-up, not a
new specimen silently resetting that earlier comparison. Both fresh final
snapshots equal their initial snapshots. No unnecessary repair was made.

## Playback evidence and its limits

Codex's ordinary in-app browser played the saved exports, using a local-only
HTML video page and visible controls. Mirket's private Chrome rendered them;
it was not used for browsing or playback review. Player telemetry was displayed
in the page and downloaded through its visible evidence control.

| Saved export | Playback rate | Wall time | Reported dropped / corrupted frames |
| --- | --- | --- | --- |
| Fresh r1 | 1× | 3003.2 ms | 0 / 0 |
| Model repair | 1× | 3004.5 ms | 0 / 0 |
| Fresh r2 | 1× | 3007.9 ms | 0 / 0 |

Every player run reached `ended` at media time 3s. The initial seek briefly
reported `waiting`, then `playing`; no subsequent waiting/stalled event was
observed. The browser reported 36 total video frames. Frame callbacks captured
35,35 and34 observations respectively; callback capture was not exhaustive,
so it is not used as proof of every presented frame. Independent decoding and
pixel review covered all 36 saved frames separately.

Live player screenshots were inspected along with saved frames. This qualifies
actual timed playback, replayable files and the observed rendered states on this
host. It does **not** establish continuous human-like perception of pacing or
smoothness from an uninterrupted video stream; these tools expose screenshots
and playback telemetry. Broad aesthetic quality, audio sync, interaction,
responsive/reduced-motion modes and other player/platform combinations remain
unqualified. Audio and interactions were excluded by the frozen brief.

The user also permitted host-browser rendering if the private browser proves
impractical. No such practical failure occurred here, so the renderer was not
changed during the frozen trial. That conditional permission is retained; it
does not itself qualify a host-browser capture backend.

## Review disagreements and evaluation corrections

One reviewer initially reported jumps at frames 08/32 of fresh-r1. Lead inspection
of the exact images and their neighbours did not reproduce them. Frames32–35
were also byte-identical, disproving the alleged frame 32-only anomaly. The
reviewer withdrew both findings; a second reviewer independently confirmed
stable geometry. Another proposed penalty required changing button fill or
preserving unrequested surrounding copy. Those criteria were rejected against
the actual brief, and the reviewers corrected their scores. No artifact was
changed on the basis of these unsupported findings.

The repair control exposed an evaluation mismatch: the reused fresh-author
validator expects `.feedback/` and an exact root-file allowlist, whereas the
frozen control prompt permitted in-project evidence and an exported artifact
without prescribing that directory. The author saved `evidence/` and
`harborline-repaired.mp4` inside its authorized project. The original validator
findings remain retained. A separate reconciliation checks the actual prompt,
all changed existing files, protected inputs, and the independent export;
this is not retroactive relaxation of the fresh-author contracts. No full-machine
write audit or Mirket sandbox claim is made.

## Evidence and verification

Private root:
`/Users/nazmiyilmaz/.codex/task-state/harness-motion-followup-20261027`.

- `protocol.json`, `frozen-followup.py`, `pilot/frozen/`: frozen scope, rubric,
  exact runner, revised skills, runtime hashes and model settings.
- `pilot/trials/`: fresh prompts, model JSONL, usage, initial/final sources,
  source checks, author exports and decoded evidence.
- `pilot/known-failure-control/`: unchanged seed, finding, model repair,
  original validator result and `control-scope-reconciliation.json`.
- `oracle/{fresh-r1,fresh-r2,repair-seed,model-repair}/`: independent selected
  source, real MCP result, metadata, all 36 decoded frames and pixel reviews.
- `review-adjudication.json`: rejected findings and evidence-based resolution.
- `playback/browser-evidence.json`: downloaded browser playback events/counters.
- `playback/`: local viewer and the exact three MP4s; temporary HTTP service
  and browser tab were closed after verification.
- `summary.json`: timing, budgets and zero remaining disposable auth copies.

Actual platform: macOS ARM64. Desktop Codex CLI 0.164.0-alpha.11.1, private
Node 24.22.1, HyperFrames 0.8.197, Chrome for Testing 156.0.8078.47 and private
FFmpeg/ffprobe 9.0.2. Exact identities are frozen and were rechecked unchanged.
Codex's in-app browser was the playback target; its separate browser build was
not identified by the tool. Astra/high is the configured host choice, not a
provider-side identity attestation or Mirket-enforced model selection.

No Rust, adapter, runtime, skill or catalog implementation changed in this
follow-up. The previous final checks therefore remain applicable: build-input
guard, 505 passing Rust tests (5 ignored), locked/offline Clippy with warnings
denied, formatting, locked adapter compilation, 64 passing adapter tests (one
opt-in skip), and explicit-binary/private-Node MCP smoke. The pilot's 18 regression
tests were rerun; catalog refresh reports no drift. No compilation or dependency
installation was needed for these documentation-only repository changes.

Changed repository files in this follow-up: this report, `README.md`, and a
cross-reference in `docs/hyperframes-feedback-qualification-20261027.md`.
There are no new local commits, pushes, releases, live installation changes or
new dependencies. Earlier uncommitted work is preserved.

Remaining: broader creative/no-skill comparisons, native child-model control,
continuous perceptual pacing review, distribution source/notice release gates,
Windows execution and the waived Desktop GUI qualification. The original broad
six-area audit and catalog `planned-not-run` acceptance are not declared complete.
