# Sampled native skill outcome checks — 2026-10-27

This is a bounded Codex CLI/model check of three project skills, a catalog-wide
trivial nontrigger, and the installed Video skill-to-tool route. It advances the
[native skill evaluation contract](m4-guide-evaluation.md) but does not accept
all 67 guides or establish a general benefit. The user waived the separate
Desktop GUI test; this record does not claim GUI activation.

## Frozen setup

The evaluator created new, disposable, non-Git projects and froze their source,
prompts and rubrics before the turns. Trial data, exact JSONL, saved artifacts,
stderr, pre/post manifests and post-run checks are outside shipped source at
`~/.codex/task-state/harness-rewrite-implementation/skill-acceptance-20261027/`.
`frozen/cases.json` is SHA-256
`98fbc1aa1caefc8a13b0fe70f571d60ea8e9f1a3a6f807f6b8d3bf9c8e81d790`.
The separate installed-Video rubric is SHA-256
`951e1cfe9ca20562c129b37f0fcee6a60211e6c360a5cee8d3160179724744fd`.
The fresh art-direction brief and rubric are SHA-256
`f44839de05c470295524bdd6dd3fcab0737b7a568ba5e48dc019025ce394f39c`.
These are fresh evaluator-authored diagnostics, not the five public examples in
the catalog. The rubric was hidden from the model; the post-run qualitative
review was by the initiating Codex agent and was not blinded or human approval.

The client was the Desktop-bundled `codex-cli 0.164.0-alpha.11.1`, SHA-256
`a04752e4d46a4d89e53e6f61c59c5349a638c8ce172ffbf391fd482b4d902a4e`,
on macOS 26.5.2 ARM64. The exact backend model revision and cost were not
visible. Every turn used the same client defaults, `--ignore-user-config`,
`--ignore-rules`, `--ephemeral`, `--skip-git-repo-check`, a workspace-write
sandbox and a fresh fake `HOME`, with real `CODEX_HOME` only for existing
authentication. No credential bytes were copied or read by the evaluator.
The real Codex config SHA-256 was
`a24841bc83964a74a4c138a338609c88cd43bb7604d1040bd9efd85e920062bf`
before and after every turn. Prompts disallowed agents, new dependencies,
network use, messages, commits and unrelated effects. One art-direction retry
nevertheless wrote two PNGs to `/tmp` outside its disposable project; their
exact bytes were retained privately and the scratch was cleared. The sandbox
and prompt are not a general containment claim.

The catalog SHA-256 was
`8cd75a20c8472a58552f7d0ef62f32e5d17f37736ab44ca9fb17a67593b9025a`.
The exact selected `test-design`, `threat-model` and `video-production` bodies
matched their catalog hashes. Earlier installed Codex app-server checks
discovered all nine global and 58 selected project skills; discovery is separate
from the model outcomes below.

## Observed outcomes

| Trial | Budget / elapsed | Observed result |
| --- | ---: | --- |
| Invoice baseline | 90 / 27.827 s | Added five tests; three failed against the documented SKU-deduplication bug and all six passed against a separately substituted per-line-sum oracle. |
| Invoice with `test-design` | 90 / 29.833 s | Read the complete guide, added five tests; two failed against the bug and all six passed against the same independent oracle. Original source and contract stayed unchanged. Both conditions met the frozen test outcome, so this pair shows no measured guide benefit. |
| Package-import baseline and `threat-model`, first pair | 90 / 90.018 and 90.025 s | Both timed out after reading the architecture, before saving `THREAT_MODEL.md`. Failures are retained; they are not counted as successful reports. |
| Package-import baseline and `threat-model`, fresh exact-input repeat | 180 / 141.732 and 135.522 s | Both saved prioritized reports, preserved the source and identified the major trust, extraction, replay, rollback and executable-identity boundaries. The guided turn read the complete skill and assigned enforcement to named components for each major control; the baseline mostly left ownership as an open action. This is one narrow observed improvement, not a general security benefit. |
| Eight-skill nontrigger | 90 / 15.105 s | With architecture, threat model, video, art direction, data, technical docs, customer research and CI skills installed, Codex changed only `Status: draft` to `Status: ready` in `README.md` and read none of the eight bodies. |
| Two-skill nontrigger | 90 / 15.370 s | With `test-design` and `threat-model` installed, the same trivial edit changed only `README.md` and read neither body. |
| Installed Video skill | 300 / 30.068 s | Read the exact guide; called `runtime_status` on local release `sha256:5a1e9002c4bd2ba4362675c237d6b0830bde9535af133bdb7e84080ca36f2a68`, which reported Video configured. It selected the five local input revisions and attempted a 640×360, 12 fps, 24-frame MP4 `video_prepare`. The host refused the effect before server execution: `MCP tool call requires approval, but approval policy is never`. Codex stopped, reported no export, and left every source file unchanged. No model-driven render or decoded QA occurred. |
| Art-direction baseline / initial guide | 150 / 76.028 and 59.224 s | Both saved editable 1200×800 SVG posters with all six exact text strings and rendered final PNGs. The guided turn read the exact body and made a specimen first. Both reported visual inspection; their JSONL does not expose an image-viewing item. Later controls show that this absence cannot establish a false claim. An independent evaluator viewed both renders after the turns and found no obvious clipping. |
| Art-direction guide repair trials | 150 / 50.296 and 71.358 s | The same frozen brief was retried after two focused guidance edits. Both read the revised body and reported pixel inspection. The first saved its render outside the requested project in `/tmp`, a real scope violation; the evaluator retained those exact outputs privately and cleared that scratch. The second rendered its final SVG in project and made one placement repair. Its claimed viewing remains unobservable in JSONL. No further poster trial was launched to reset the budget. |

The independent invoice check ran saved tests unchanged in copied projects:
the buggy implementation failed and a separate simple per-line-sum oracle
passed in both conditions. The package-import post-run review applied the
frozen rubric to saved documents; both covered the main risks, while the
guided output was more explicit about component owners. It did not test an
implemented importer. Complete guide reads are evidenced by command outputs
containing the exact UTF-8 skill bodies, rather than the model's claims.

The Video refusal is a host permission result, not a Mirket runtime failure.
The exact denied argument object was then replayed through a separate direct
MCP client against the same disposable installed release. `video_prepare`
returned `status: prepared` with operation ID
`op-18e2548638c49d58-14166-0`; no `video_start` was sent and no export was
created. This checks the model's request shape and selected file revisions at
the server boundary, but it does not turn the refused model call into a
completed model-driven effect. The direct check is retained as
`video-denied-intent-direct-check.json` in the private trial directory.
Direct installed MCP Video exports and decoded output checks passed separately
in the [local Mac qualification](simplification-qualification-2026-10-27.md).
The refusal cannot be bypassed by changing Mirket's tool code or by treating
the user's task authorization as a host permission. No output file was
created, and the model correctly declined to claim FFmpeg/frame verification.

The art-direction review is retained as `art-direction-independent-review.json`
(SHA-256 `d841bc490010cbe3eaa5b3bb13920a9cafd1203c8b12276804aa54d43c205656`).
It checked saved SVG sources and PNG dimensions, and the initiating evaluator
viewed the renders with an image-capable tool after the model turns. The
initial guide body was loaded from the catalog frozen for that trial. The
current revised body is SHA-256
`0591e59891f86d82ad37820128160283ceb47b21850f65b43317564bbf7411dd`;
it explicitly requires rendering the final saved artifact and naming the
image-capable view at handoff. The first repair trial's `/tmp` write remains a
failure; guide benefit is not established by these four variable outputs. The
current catalog SHA-256 is
`6d9dd6bcb2bbae078b41b6ca2699f7c421d7ace745962a89517907206256b30d`.

### Image-view trace calibration

The initial review incorrectly treated the absence of an image-view item in
Codex CLI JSONL as proof that the model had not seen the posters. Five separate
read-only controls corrected that inference. With a PNG as the only project
file, Codex identified the orange circle on the right, then on the left in
a mirrored image, and read an unpredictable six-digit image code (`386260`).
In a stricter trial, the answer `478585` existed only in the generated PNG
before the turn: no criterion file or SVG source was saved, and Codex returned
the exact number. When the PNG was absent in a separate control, it reported
the number unverified. Each trace contained
only agent-message items, including the turns that said they used `view_image`.
Every project and the real Codex config remained unchanged. The exact controls
and hashes are in `image-view-probe-review.json` (SHA-256
`429ba1e81456f77cde60d4375ad366de4f1e18d6d244ab5de986588a8571542b`).
This strongly supports an image-access path whose calls are not represented
as JSONL items here; its internal mechanism and the exact viewing action in
each poster turn remain unobserved. The poster claims therefore cannot be
accepted or rejected from JSONL alone.

### Catalog-wide trivial nontrigger

The frozen plan `catalog-nontrigger-plan.json` (SHA-256
`850bdb5e5ac36e7bcbf6dac798ad129271feb9ef3dd8cd9301d5ae557b27b408`)
placed all 58 project skill bodies in six fresh projects of 8–10 skills and
all nine global bodies under a seventh trial's disposable user home. Every
body matched its catalog hash. With the same one-word README task and a
90-second limit per turn, all seven turns made exactly the requested edit,
left every other project file and the real Codex config unchanged, and showed
no skill-body read in the visible command trace. Total elapsed time was
98.854 seconds; the independent file/trace check is
`catalog-nontrigger-review.json` (SHA-256
`47c9751c9318f31eb470e5944b93846d8e8a718938f2f595bf3714640c88878b`).
This is one deliberately easy negative task for each catalog entry, not a
claim that every skill is correctly triggered or beneficial. The global case
used a fake `HOME` while retaining the real `CODEX_HOME` solely for existing
authentication, so this turn alone does not establish global skill discovery;
the separate installed app-server journey did that. The JSONL visibility
limit above also means "no visible body read" is narrower than proof that the
client never consulted a skill internally.

## Acceptance boundary

This sample demonstrates correct activation for four selected bodies, a
catalog-wide trivial nontrigger, discriminating saved tests and one detailed threat-model
comparison. The art-direction outside-project render is an explicit scope
defect; per-turn pixel viewing lacks direct trace evidence. The sample does not
establish broader representative nontrigger coverage, matched benefit across
the specialist catalog, model-verified creative/video output
quality, or model-driven effect completion. The initial 90-second threat-model timeouts
show that a 90-second budget was too short for this case; the fresh 180-second
repeat was matched across conditions and preserved both initial failures.
Model/provider drift between turns was not observable beyond the same client
version and settings, so causal benefit claims stay narrow. Windows and the
Desktop GUI remain untested under their separate decisions.
