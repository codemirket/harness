# Motion authoring trials — 2026-10-27

This is a private, finite held-out authoring comparison. It is not a renderer
qualification and does not claim a skill-quality uplift until the lead renders
and independently reviews the saved outputs. The private trial root is:

`/Users/nazmiyilmaz/.codex/task-state/harness-motion-authoring-20261027/trials`

## Frozen protocol

The frozen protocol is `frozen/protocol.json`, with source briefs in
`frozen/cases.json` and the baseline skill snapshot created from
`git show HEAD` at commit `8d4f3923822ee67317863a97cd60d69f737d06f8`.
The two matched specimens are:

1. A Harborlight Tide Pass three-beat product demo.
2. A Beacon Loop editable vector illustration loop.

Both briefs require a HyperFrames 0.8.197 no-timeline root, 640×360, 12 fps,
36 frames / 3 seconds, CSS/WAAPI-only authoring, a bundled local Noto Sans
font, no packages/network, stable IDs, one animation owner, editable sources,
and zero-based captions covering frames 0–35. Each specimen permits at most
one repair; the shared task budget is four repairs. At initial authoring freeze,
no repair or review turn had run; the bounded repair amendment below records the
later independent runtime finding and its one-turn repairs.

The current updated motion-resource snapshot is frozen at
`frozen/current-skills/skills`, including
`motion-direction/references/editable-motion.md`; its hashes are in
`frozen/current-skill-hashes.json`. The baseline and current outputs are still
not a causal quality comparison because the backend model/reasoning identity is
unavailable and no independent visual review has scored the artifacts.

## Runtime and model-turn identity

- Host: macOS 26.5.2 ARM64.
- Codex CLI: Desktop bundled, SHA-256
  `a04752e4d46a4d89e53e6f61c59c5349a638c8ce172ffbf391fd482b4d902a4e`.
- Config sentinel before/after each turn:
  `58519db0c8d95e39a5566f7660867333ab37543a3bfce6e676fc99a3cc66aeb3`.
- Matching args: `exec --strict-config --ignore-user-config --ignore-rules
  --ephemeral --skip-git-repo-check --sandbox workspace-write --color never
  --json`, disposable `HOME`, and real `/Users/nazmiyilmaz/.codex` reused as
  `CODEX_HOME` for authentication.
- Backend model identity and reasoning setting are not exposed by these JSONL
  records and remain unknown; no model uplift claim is inferred.
- Local bundled Noto Sans SHA-256:
  `bfb7bb691513f12e734dc346c03a03f784912432d7e3fa8e56efcf906fe86b3d`.

## Baseline turns

| Specimen | Wall time | Exit | Outputs | Independent status |
|---|---:|---:|---|---|
| Product demo / baseline | 90.427 s | 0 | `index.html`, `film.css`, `film.js`, `captions.json` | no rendering or pixels; root review pending |
| Vector loop / baseline | 119.669 s | 0 | `index.html`, `film.css`, `film.js`, `illustration.svg`, `captions.json` | no rendering or pixels; root review pending |

## Current-resource turns

| Specimen | Wall time | Exit | Outputs | Loaded resource / independent status |
|---|---:|---:|---|---|
| Product demo / current | 86.932 s | 0 | `index.html`, `film.css`, `film.js`, `captions.json` | `editable-motion.md` loaded; no rendering or pixels |
| Vector loop / current | 96.480 s | 0 | `index.html`, `film.css`, `film.js`, `illustration.svg`, `captions.json` | `editable-motion.md` loaded; no rendering or pixels |

Each trial retains `prompt.txt`, `budget.json`, `initial-manifest.json`,
`events.jsonl`, `stderr.log`, `result.json`, and the complete project tree.
Both turns preserved BRIEF.md and NotoSans.ttf, passed their model-reported
static source checks, and did not create extra project files. Model claims are
retained as claims only; no visual or playback outcome is inferred.

The current turns likewise preserved BRIEF.md and NotoSans.ttf, retained the
frozen current skill/reference snapshot in their final manifests, and reported
the new editable-motion contract as loaded. No repair was performed during the
authoring turns; the later bounded repair turns are recorded below.

The product turn’s final manifest retains the frozen baseline skill hashes.
The vector turn likewise retains the same baseline skill snapshot. The product
turn included a failed local static-test assertion in its trace before its
final source checks passed; this remains in `events.jsonl` and is not silently
treated as a pass.

## Bounded repair turns

The independent runtime qualification refused all four initial projects with
`authored_root_profile_mismatch`; inspection found that each composition root
lacked a literal `data-no-timeline` attribute despite explanatory comments. A
subsequent oracle showed native WAAPI motion in exported frames, so the initial
over-broad suggestion to replace native animation was corrected before any
repair process started. The final feedback required only the literal root
marker and preservation of the existing dimensions, three-second duration,
facts, local font, captions, editable files and output scope.

Each repair ran exactly once in an immutable copy of its corresponding initial
project. The four-turn shared repair allowance is fully consumed; there were no
retries or budget resets. `repair-before-manifest.json`,
`repair-after-manifest.json`, `repair-result.json`, `events.jsonl`, and the
complete repaired project tree are retained in each directory.

| Repair copy | Wall time | Exit | Changed paths | Root marker |
|---|---:|---:|---|---|
| Product demo / baseline | 15.048 s | 0 | `index.html` only | literal marker present |
| Product demo / current | 12.701 s | 0 | `index.html` only | literal marker present |
| Vector loop / baseline | 13.571 s | 0 | `index.html` only | literal marker present |
| Vector loop / current | 18.518 s | 0 | `index.html` only | literal marker present |

The original four projects remain unchanged and still lack the marker, as
expected for an immutable before/after comparison. These repair turns have not
been rendered or independently scored yet.

## Repaired-source runtime oracle

The final repaired copies were each passed to the real Mirket video MCP route
sequentially with explicit private paths for Node, the adapter, Chromium, FFmpeg
and FFprobe. Runtime identity was `darwin-arm64`, release
`motion-authoring-20261027`, and `runtime_status.video.configured` was true in
all four receipts. All four jobs reached `prepared` and `queued`, then failed
before export with the same observable adapter refusal:
`invalid_caption_cues` (`effect: not_attempted`; private stage may exist).
No MP4 or decoded frames were produced, so this is a runtime failure finding,
not visual evidence and not a basis for another repair.

Receipts and source manifests are retained at:

- `runtime/repaired/product-demo-baseline/runtime-receipt.json`
- `runtime/repaired/product-demo-current/runtime-receipt.json`
- `runtime/repaired/vector-loop-baseline/runtime-receipt.json`
- `runtime/repaired/vector-loop-current/runtime-receipt.json`

## Qualification limits and amendments

The lead will render all initial and repaired outputs with the explicit private
Chromium/FFmpeg route and independently decode the output. The independent
oracle observed native WAAPI motion at frames 32, 80, 128, 224 and 312, so the
repair does not claim that native animation is categorically unsupported. The
pinned producer’s deterministic seek behavior still requires saved-pixel and
decoded-output review, and the renderer must distinguish supported native WAAPI
motion from CSS `@keyframes` output. The frozen briefs were not rewritten after
the baseline turns.

No model-driven export, pixel inspection, playback claim, or final visual
judgment is made for the authoring or repair turns in this report. Windows and
the waived Desktop GUI check remain unqualified.

