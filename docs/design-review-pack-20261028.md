# D6 review pack and remaining release gates

The [Daybreak review pack](../specimens/daybreak/review.html) is delivered. On
2026-10-28 the user answered **“Ready for this design direction”** after being
presented with the pack and asked to judge it at 1× against the Granger target.
That acceptance is recorded for the current corrected Daybreak candidate,
not as release authorization, universal defect freedom or a model comparison.

The film SHA-256 is
`395f48430985797e6a56353f5e0946eb85791d81e9ccacb15b739494e523d2bb`.
The film and all its composition inputs remain unchanged from D6's starting
revision, `3076fd5`. No new render, artwork or creative repair was performed;
earlier creative budgets remained closed. This D6 packaging/verification pass
had one 30-minute window, including independent read-only review.

## Reviewable output

- One local page contains the actual saved MP4, native playback controls,
  five chapter buttons, full-size linked PNGs and three focused questions about
  art direction, motion/reading time and overall craft against the user's target.
- The opening comparison clearly labels the preserved defective export and
  corrected candidate. The other saved frames show hero, club, runs and return.
  Granger is an external reference link; no reference media is redistributed.
- The live responsive product page remains a separate linked artifact.
  The pack presents evidence without imposing reel transitions on normal UI.
- The user's acceptance is bound to the export hash in
  `qa/review-pack.json`. The page does not submit, infer or collect acceptance
  itself. A future video hash cannot inherit that acceptance silently.

Current local URL: <http://127.0.0.1:8768/review.html>, while the existing preview
server is running. The source page and assets remain in the repository for
later local serving; no public site or release was published.

## Actual verification

Fresh FFprobe reports **1280×720, 30/1fps, 360 decoded frames, 12.000000s**,
with no audio stream. The entire saved MP4 decodes without errors. All four
existing pixel regressions pass: full y descender, full k edge, hero CTA
separation and no exposed full-height stage gutter. This repeats the affected
artifact checks; it does not reopen the prior production experiment.

Codex's in-app browser exercised all five chapter controls. Each paused at its
declared time (0.7, 3.0, 5.9, 8.8 and 11.9 seconds). A normal-speed play began
at 0.2447s with `playbackRate: 1`, then was observed at 12s with `ended: true`
and no media error. A replay also reached the end. These are discrete timed
observations, not a claim of continuously observed perceptual playback.

The review surface was inspected at 1280×720 and 390×844. All six saved images
loaded at their native 1280×720 dimensions; the mobile document width was 390px
with no horizontal overflow. An initial interaction left part of the film
outside the viewport. The review page now fits the video within the viewport
and scrolls it into view when playback/chapter controls are activated; the
rechecked desktop bounds were approximately y=42–510 in a 720px viewport.
Temporary viewport overrides were reset.

[check-review-pack.mjs](../specimens/daybreak/qa/check-review-pack.mjs) has two
explicit evidence levels. Its basic mode checks 15 frozen identities. With
explicit private FFmpeg/FFprobe paths it also probes the actual profile and
compares decoded RGB pixels of every current PNG against the MP4 at its stated
timestamp. All five correspond exactly. The historical before frame has its
own preserved hash and prior review evidence, not a false current-film match.
Neither mode scores visual quality.

Discriminating checks retained:

| Perturbation | Actual result |
| --- | --- |
| Change the video, displayed opening PNG or composition JS in a disposable copy | Identity check refuses each changed file. |
| Change shared `site.css` | Regression check refuses it; the real dependency was also matched to the retained export source and included in the pack manifest. |
| Relabel the opening PNG as 5.9s while leaving all image/video hashes intact | Identity-only mode passes; actual decoded correspondence fails. This is why a snapshot receipt cannot establish frame provenance by itself. |
| Retain acceptance for a different video, introduce a symlink, or point outside the pack | Automated negative cases refuse. |

The repository test covers changed video/frame/JS/CSS, stale acceptance,
symlinks and path escape. Actual positive and wrong-timestamp decode journeys
ran using private Node v24.22.1 and the already selected private FFmpeg pair on
macOS 26.5.2 ARM64. Syntax, local asset-link, diff and catalog checks pass.
Rust, runtime, adapter and native skill inputs did not change: the complete
[D5 source gates](native-coordination-qualification-20261028.md#verification)
remain the preceding qualification and are not described as freshly rerun.

Private evidence:
`/Users/nazmiyilmaz/.codex/task-state/harness-d6-review-pack-20261028/`, including
the baseline protocol, probe/decode output, pixel checks, identity/decode
checks, wrong-timestamp negative case, test log, desktop/mobile screenshots
and discrete playback observations. The connected Mirket runtime was read
passively: release `sha256:551dbc9d6754ba9003734c5b171c96878f4e09483421f56b602cf625e122d5d9`
reported configured finite video jobs. No operation on the live installation
was started, and no new engine or production dependency was introduced.

## Current gate handoff

| Gate / owner | Current status | Required next evidence |
| --- | --- | --- |
| Current candidate art direction / user | **Accepted for this candidate** | A materially changed candidate needs its own review; this is not a general quality benchmark. |
| Local macOS Video / Mirket | Exact private candidate accepted by current source; installed four-format and cancellation evidence retained | Maintain pinned inputs and requalify affected behavior after changes. [Current source](../src/distribution/video_candidate.rs), [local integration](simplification-qualification-2026-10-27.md#final-local-mac-integration). |
| HyperFrames source/notices / distributor | Exact 0.8.197 publisher/source mappings and font notices retained | Bind the complete applicable source/notices to the actual combined recipient package and selected delivery route. [Recipient record](m3-video-recipient-closure.md). Local use is distinct from public release admission. |
| FFmpeg recipient closure / distributor | Neutral private pair and local source-delivery fixture qualified | Exercise actual recipient availability, final package/source identity, relevant aggregate terms and applicable organizational/legal review; finish authentication/advisory checks. [Source gate](ffmpeg-recipient-source-closure.md#remaining-release-gates). No public offer or release channel was selected here. |
| Browser / setup and host owners | Private Chrome is setup-acquired for rendering; Codex owns general browsing | Preserve pinned acquisition/identity and applicable recipient obligations. [Acquisition record](m3-browser-setup-acquisition.md). Optional Raster rights are separate from the Sharp-free Video tree. |
| macOS floor / native qualification owner | Current execution is macOS 26.5.2 ARM64; binary deployment metadata names macOS 15 | Actually run the supported setup/runtime/output and recovery gates on macOS 15. Metadata cannot establish OS-floor support. |
| Windows / native qualification owner | x64 and ARM64 remain unqualified | Actual process-tree ownership, file identity/ACLs, installed payloads, Codex integration and output journeys on each intended target. [Windows gates](windows-qualification-status.md). |
| Desktop GUI / user waiver | Waived and untested | No pass inferred from CLI or browser work; only explicit future GUI qualification changes this status. |
| Native interruption / Codex host | D5 observed a command continuing after turn interruption | Observe owner-level command/job cessation before cancellation claims; no Mirket orchestrator or host interception added. [D5 result](native-coordination-qualification-20261028.md). |

An independent gate reviewer initially read the historical Video refusal as a
current blocker. It was rejected against current source and the existing
historical-document banner. Local Video installation is **not** the remaining
blocker. The browser acquisition document's old general-browsing wording and
GUI release-gate implication were clarified to match the accepted boundary.

## Six-area scope and changed files

| Original audit area | D6 contribution / boundary |
| --- | --- |
| Skill authoring and references | Existing direction retained; no skill body/reference change or catalog acceptance promotion. |
| Skill/tool pairing | Mirket rendering boundary retained; Codex browser playback and project QA tools exercised. No new render/tool pairing claimed. |
| Project memory | Unchanged; no new memory qualification. |
| Capability matrix | Above handoff distinguishes local route, human acceptance, public recipient delivery and actual native platform gates. |
| Agent coordination | One independent read-only reviewer, lead-owned integration; stale gate interpretation corrected from source. No new creative model trial. |
| Feedback/output quality | Current candidate accepted by user; whole-frame review controls, actual frame correspondence and stale-evidence negative cases delivered. |

Changed files are the new `review.html`/`review.css`, six `qa/review/*.png`
frames, `qa/review-pack.json`, its checker and regression test, the Daybreak
README/reel link, browser acquisition clarification, remaining plan, root
README and this record. Catalog/provenance hashes required no refresh because
their source bodies/resources did not change; the repository checker reports
no drift. The local commit ID is provided in the delivery message. Public
release, Windows and OS-floor qualification remain open; the original six-area
audit is not declared complete from D6.
