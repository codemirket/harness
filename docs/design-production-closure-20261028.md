# Bounded design-production closure — 2026-10-28

The approved [R1–R4 plan](../planning/remaining-design-work.md) produced a checked
maintenance variant using the existing HyperFrames route. No engine, dependency,
or orchestration change was needed for this journey. This is one maintenance
result, not evidence that arbitrary creative work is solved or that a model or
skill outperforms another. The earlier failed cohorts remain failed.

## Scope and outcome

The shared clock began at **11:38:53 UTC**, with a **12:38:53 UTC** deadline.
Execution, repair and verification finished by **12:01:57 UTC** (23m04s);
documentation, local commit and final handoff stayed inside the same window.
The frozen change was `Find your first run` → `Choose your next run`, in a
disposable copy of the accepted Daybreak source at baseline `eddf469`.
Profile: silent H.264 MP4, 1280×720, 30fps, 360 frames / 12 seconds.
The original composition, assets, export and acceptance record were preserved;
only its README changed. No live installation, publication or push occurred.

| Step | Finding / correction | Observed outcome |
| --- | --- | --- |
| R1 | Existing tools had separate instructions but no concise ordered production handoff. Added [saved source to checked export](../skills/video-production/references/hyperframes-0.8.197-authoring.md#saved-source-to-checked-export) and [Daybreak maintenance instructions](../specimens/daybreak/README.md#focused-maintenance). | Static source checking, preview path constraints, one-job reconciliation, decoded review and acceptance boundaries are explicit. Catalog resource hashes refreshed through repository tooling; no new reference adoption. |
| R2 | Rebuilt the changed page sections with the existing builder and selected seven exact render inputs. | Disposable MCP `runtime_status` reported video configured; prepare/start/status/result completed operation `op-18e2af09e0238770-bbcc-0`. The actual export contains the changed CTA. |
| R3 | Video checks do not cover live-page hit targets. At 1280×720, the absolute wordmark covered the CTA and its center intercepted the click. | Before repair, wordmark/CTA bounding-box gap was −77.48px and the hit target was `DIV.wordmark`; navigation did not occur. Existing film CSS already separated them. |
| R4 | Used the one shared repair batch in the disposable page's head: normal flex flow reserves room for the decorative wordmark. Raising stacking order alone would leave a visual collision. | Desktop gap is 32px and the center hits the link. Desktop and 390×844 mobile clicks reach `#sessions`; mobile has no horizontal overflow. Saved pixels and independent review found no residual actionable defect in these states. |

The initial CTA-only criterion did **not** pass without repair. `repair.json`
records the additional live-page layout change; the freeze was not rewritten.
It stays in `index.html`'s head, which `build-reel.py` does not copy into the
film. After rebuilding, all seven selected render inputs remained byte-identical
to the exported inputs. No second export was needed and no repair budget reset.

At this trial's close, the historical page in `specimens/daybreak` retained the
reproduced live-page limitation because the trial required preserving it. The
subsequent [release-readiness work](release-readiness-20261028.md) integrated
the head-only repair into the maintained page and rechecked desktop/mobile
clicks and pixels against the original failing control. The accepted film bytes
and this frozen trial's acceptance record remain unchanged.

## Actual artifact and review

The new MP4 is 2,124,736 bytes, SHA-256
`1c57c88b833908fda2e711123c6e951c4e7135e4aef009523e9261c25f92ea37`.
FFprobe confirmed one video stream, 1280×720, 30fps and 360 decoded frames.
All 360 frames decoded successfully. Saved-pixel inspection covered frozen
frames 21, 54, 75, 90, 102, 120, 177, 264, 321, 330 and 357, plus the independent
reviewer's samples at 0, 45, 90, 150, 240, 330 and 359. The CTA is readable in
the hero and return states; sampled club, sessions and transition states show
no actionable regression. This is sampled visual judgment, not exhaustive
perceptual inspection of every frame.

The existing independent glyph controls measured 283/285 expected y-descender
pixels and 80/83 k-edge pixels. CTA probe intrusion and full-height stage gutter
counts were both zero. Thresholds were unchanged. Codex's browser played the
actual MP4 at rate 1 from 0.2627 seconds to `ended=true` at 12 seconds without a
media error. Saved player pixels were inspected. This establishes playback
completion and discrete visual checks, not continuous human viewing or audio
quality. Audio/captions are intentionally absent in this task.

The original review-pack control passed, including decoded-frame correspondence.
The stale manifest in the changed copy correctly failed on `film.html`; it was
not regenerated to transfer acceptance. A separate browser regression check
retained the failing pre-repair hit target and passing repaired geometry/click.
The independent reviewer inspected both film samples and repaired desktop/mobile
pixels; the lead integrated the review and owned the single repair.

## Verification and platform

Actual host: **macOS 26.5.2 ARM64**, private Node **24.22.1**, locked TypeScript
**5.9.3**, HyperFrames **0.8.197**, explicit private Chrome/FFmpeg/FFprobe paths.
The current development binary embeds the refreshed skill catalog. The isolated
fixture's release label is `motion-authoring-20261027`; that label is not an
installed release qualification. Runtime paths, executable hashes and versions
are retained in `runtime-paths.json` and `runtime-identity.json`.

| Gate | Result |
| --- | --- |
| Build-input guard before Rust compilation | Passed |
| `cargo build --locked --offline` | Passed; copied tested binary used for export and smoke |
| `cargo test --locked --offline` | 506 passed, 5 ignored, no failures |
| All-target locked/offline Clippy with warnings denied; formatting | Passed |
| Adapter compilation with locked local TypeScript; adapter tests | Passed; 68 passed, 1 skipped |
| Authoring/preview/review-pack tests | 21 passed |
| Catalog refresh tests; skill assembly tests | 6 and 8 passed |
| Explicit-private-Node/current-binary MCP smoke | Passed inventory/runtime, stale release, absent payload, unknown job, pairing and stale source refusal |
| Final source preflight, actual export checks, original/stale pack controls | Passed with expected stale-pack refusal |
| Portable patch applied to a fresh disposable baseline | Passed; resulting page and film bytes match the delivered source |

The optional generic skill-creator `quick_validate.py` could not run because
PyYAML is absent from both available Python environments. No dependency was
installed to satisfy it. Skill frontmatter was unchanged; repository catalog
and assembly validation passed. This unavailable auxiliary check is not counted
as a pass.

No new CLI author/model comparison was run. No native model-selection enforcement
was added. Windows, macOS 15 floor, waived Desktop GUI activation, public
redistribution/source delivery, broad catalog acceptance and the remaining
six-area audit retain their separate unqualified status.

## Local handoff

Evidence root:
`/Users/nazmiyilmaz/.codex/task-state/harness-design-closure-20261028`.
It retains the frozen protocol, before/after sources, repair record, exact
render inputs, runtime receipt, all decoded frames, saved browser pixels,
positive/negative checks, review notes and gate logs. These are local evidence,
not a public recipient package or shipped runtime assets.

Working copy:
`/Users/Shared/mirket-design-closure-20261028/daybreak`.
Use `index.html` for the repaired page and `maintenance-player.html` /
`maintenance.mp4` for the changed export. The copied old `daybreak.mp4` and
review manifest remain historical controls; they do not describe the new film.
The copy's README records final source revisions, operation ID and budget.
`maintenance.patch` in the evidence root reproduces the copy's two source-file
changes against the preserved baseline. There is no automatic next cohort.
