# HyperFrames boundary research — 2026-10-27

## Decision

Keep HyperFrames as the video renderer. The evidence does not justify replacing
it, and the requested failures are not one homogeneous renderer defect. The
largest demonstrated failures are at the host-permission, authoring/skill,
wrapper-policy, and independent-QA boundaries. HyperFrames does have real
version-specific failure classes (stalled BeginFrame capture, media-engine
fallback, timeline readiness, preview/render disagreement, and child cleanup),
but the local record does not show that the pinned 0.8.197 renderer is the
cause of the observed Mirket failures. Those classes need a controlled
version comparison, not an unreviewed dependency upgrade.

This is a research and selection result, not release admission. It does not
change the pinned HyperFrames package, install anything, or alter the live
`~/.mirket` installation.

## Scope and evidence

The local candidate is HyperFrames producer **0.8.197**, source tag commit
`f51650d8a72ed472effff89da087c7e1cf6b2c98`, on macOS 26.5.2 ARM64. The current
upstream release page lists 0.8.207, released 2026-10-27; that is comparison
evidence only. Repository metadata was captured on 2026-10-27 in the private
task state file
`~/.codex/task-state/harness-design-research-20261027/repo-metadata.json`:
`heygen-com/hyperframes`, Apache-2.0, 73,571 stars, 6,666 forks, 175 open
issues. Stars and forks are context, not quality evidence.

Primary sources:

* [Pinned producer README](https://github.com/heygen-com/hyperframes/blob/f51650d8a72ed472effff89da087c7e1cf6b2c98/packages/producer/README.md)
  describes BeginFrame capture, FFmpeg encoding/audio mixing, Node >=22,
  Chrome/Chromium and FFmpeg requirements, project entry-file/dimension
  conventions, and the upstream output formats.
* [Pinned troubleshooting guide](https://github.com/heygen-com/hyperframes/blob/f51650d8a72ed472effff89da087c7e1cf6b2c98/docs/guides/troubleshooting.mdx)
  requires a composition root with `data-composition-id`, width and height;
  documents local-asset and remote-media failures, animation registration,
  media timing rules, preview/render differences, the 45-second no-timeline
  wait in this pinned revision, and common expensive browser effects.
* [Current upstream releases](https://github.com/heygen-com/hyperframes/releases)
  show subsequent fixes in 0.8.206 and 0.8.207. The page is used only to
  identify known upstream failure classes, not to silently select a new
  version.

Local evidence was read from:

* `docs/m3-motion-video-qualification.md` (candidate qualification and the
  `data-no-timeline`, Anime registration, missing-media, crashpad and
  four-format observations);
* `docs/video-export-route.md` (Mirket wrapper contract, source/asset policy,
  bounded publisher, process cleanup, the intermittent scanner failure and its
  repair);
* `docs/m3-video-installed-route-preparation.md` (installed-route fixture
  omissions, source preflight and the local four-format run);
* `docs/m4-skill-acceptance-sample-20261027.md` (host approval refusal and the
  art-direction scope escape);
* private raw trial artifacts under
  `~/.codex/task-state/harness-rewrite-implementation/` (not copied into this
  repository). The private records are evidence locators, not release inputs.

The local trials used the same small authored HTML fixture, explicit private
Chrome/FFmpeg/ffprobe, one worker and disposable state. They are useful
diagnostics, not coverage of arbitrary compositions, long renders, media
codecs, player compatibility, Windows, or release rights.

## What actually failed, and where the cause belongs

| Observation | Reproducible consequence | Best-supported owner | HyperFrames conclusion |
| --- | --- | --- | --- |
| The model's `video_prepare` call was refused with `MCP tool call requires approval, but approval policy is never`. | No server execution, no render, no output, unchanged sources. A direct replay of the same request against the disposable MCP server returned `prepared`. | Codex host approval boundary. | Not a HyperFrames failure; the renderer was not called. Mirket cannot grant host permission. |
| One art-direction repair wrote two PNGs to `/tmp` outside the requested project. | A real out-of-scope write occurred during a non-video creative trial. | Model/skill compliance and harness artifact-scope enforcement. | HyperFrames was not involved. The correction is path ownership and independent artifact checks, not a renderer change. |
| An installed fixture referenced an unselected `checker.png`; another caption scenario lacked a pre-existing caption element. | The installed qualification initially failed before it exercised the intended composition. Including the selected image and handling the no-caption composition made the fixture pass. | Fixture/source authoring. | Explicitly recorded as fixture omissions, not a HyperFrames change. |
| Repeated MCP renders initially returned `video_export_failed`; the work-tree sampler raced a transient Chromium directory and received `ENOENT`. | Fifteen later complete runs passed after the sampler tolerated only disappearing children below its existing root. | Mirket wrapper race and cleanup policy. | The observed fix was in the wrapper scanner. It is not evidence that HyperFrames itself failed. |
| A missing local image was discovered by the producer after capture/encode work. | The destination was absent, but work was wasted. | Mirket preflight policy. | HyperFrames correctly reported the media failure; Mirket should preflight selected assets before launching an expensive render. |
| The first launch left Chromium crashpad handlers in the normal macOS crash directory despite a private `HOME`; HyperFrames passed `--no-sandbox`/`--disable-setuid-sandbox`. | Task-created handlers were identified and terminated; later launches supplied a private crash-dumps directory. | Mirket process/private-state launch contract, with an upstream launch behavior to account for. | This is a genuine integration limitation of the assembled route, not proof that the frame renderer is incorrect. The contract must keep explicit crash state and process ownership. |
| Without `data-no-timeline`, a non-GSAP fixture waited for `window.__timelines[id]` and failed with `sub_timeline_readiness_timeout`. | Adding the supported marker made the strict render proceed. | Authoring contract plus pinned producer readiness behavior. | This is a real pinned-version behavior, documented upstream. It is predictable and skill-fixable; it is not a reason to replace HyperFrames. |
| An Anime v4 instance not registered in `window.__hfAnime` rendered successfully but stayed at x=32 while CSS/WAAPI motion advanced. | A successful status and changing whole-frame hashes did not prove intended motion. | Authoring/skill and independent visual QA. | HyperFrames can produce a valid file with semantically wrong animation when the source omits registration. The skill must require registration and an oracle must inspect intended pixels. |
| Small MP4/WebM/GIF/PNG-sequence fixtures decoded and passed intended-motion checks. | One controlled Mac fixture worked, including audio and alpha checks in the native qualification. | Positive wrapper + renderer evidence, narrow scope. | This supports retaining HyperFrames, but does not qualify arbitrary assets, long renders, player compatibility, or Windows. |

The distinction matters: a successful receipt, output hash, process exit, or
model statement is not evidence that the intended frame content, scope, rights,
or saved output was correct. Conversely, no image-view event in Codex CLI JSONL
does not prove that a model failed to inspect an image; the saved-pixel oracle
is the appropriate test.

## HyperFrames' actual contract and limits

The pinned producer is an HTML/DOM renderer, not a general-purpose media
authoring system. Its documented contract is a project directory with an entry
file (default `index.html`), authored dimensions, finite FPS, and local runtime
requirements (Node >=22, Chrome/Chromium and FFmpeg). It captures browser frames
through Chrome BeginFrame, encodes through FFmpeg, and mixes audio. The pinned
docs list MP4, WebM, MOV, GIF, PNG sequence and HLS, while Mirket intentionally
exposes only the finite profiles admitted by its wrapper.

The troubleshooting guide itself calls out the following behavior: missing or
remote media can fail from path, permission, expiry or cross-origin issues;
browser effects and heavy layers can make preview stutter; a video element
should be wrapped rather than directly resizing its media box; composition code
must not call `play()`, `pause()` or set `currentTime`; a root duration should be
authored directly; timed elements need the required class/attributes; and the
render can differ from preview because of fonts, remote media, browser effects
or the exported file. These are authoring and runtime constraints, not claims
of an independent Mirket bug.

The pinned no-timeline behavior is especially important: no timeline marker
causes a fixed 45-second readiness poll in 0.8.197. Upstream 0.8.206 release
notes say this wait was removed. That is evidence of a real upstream limitation
in the pinned version, but Mirket's current authoring contract already requires
`data-no-timeline` for non-GSAP compositions. A version upgrade still requires
the repository's normal dependency admission, provenance, rights and
cross-platform gates.

Upstream's current 0.8.207 release notes also list fixes for stalled BeginFrame
capture resumption, fresh-session recovery after `drawElement` failures,
software decoding when a Mac media engine refuses a clip, child cleanup, media
source probing, and Windows setup. These are credible signals that this class
of renderer has moving edge cases. They do not identify a defect in Mirket's
0.8.197 run, and they do not authorize upgrading the pinned package.

## Mirket wrapper restrictions are deliberate

Mirket narrows the upstream surface to preserve Codex ownership and an
auditable project boundary. The wrapper requires explicit canonical Node,
Chromium, FFmpeg and ffprobe paths; a private home/profile/cache/crash area;
local selected sources and assets with revisions; no remote or dynamic source
references; bounded dimensions, frames, FPS, duration, output and sampled work
tree; one internal worker; stale-source/destination refusal; cancellable owned
process cleanup; and independently decoded output/archive checks. It exposes
MP4, WebM, GIF and `png_sequence_tar`, not every upstream format. These limits
are Mirket policy, not HyperFrames capabilities being falsely reported as
missing. They are also not an OS sandbox for hostile HTML.

The wrapper's source and destination revision checks, loopback file server,
request interception, executable identity checks and scratch cleanup are
correct places to constrain an authored project. They must not be described as
features supplied by HyperFrames. Conversely, HyperFrames' own requirement to
register animation/timeline state must not be hidden behind a generic
"render succeeded" receipt.

## Root-cause ranking and corrections within Mirket's control

1. **Host permission (critical, demonstrated):** retain the current behavior
   that stops after approval denial and reports no export. Add no bypass and no
   Mirket replacement tool. The direct prepared-intent check is useful protocol
   evidence only; it is not a model-driven export.
2. **Out-of-scope creative write (high, demonstrated):** keep the project path
   explicit, inspect saved manifests independently, and retain the failure as a
   model/skill regression. A renderer cannot repair this.
3. **False visual success (high, demonstrated):** require the authored
   `data-composition-id`, dimensions/duration and `data-no-timeline` or timeline
   registration; require Anime v4 `window.__hfAnime` registration; inspect
   first/middle/last intended elements from decoded frames. Hashes and changing
   neighbors are insufficient.
4. **Wrapper race (high, repaired, bounded):** retain the narrowly scoped
   disappearing-child-directory tolerance. Continue failing on a missing root,
   unrelated traversal errors, ownership uncertainty, or changed bytes. Do not
   broaden this to an unbounded ignore.
5. **Late media failure (medium, demonstrated):** preflight all selected local
   files and codec inputs before consuming a heavy slot. Keep producer errors
   and source paths in evidence.
6. **Private-state/process cleanup (medium, demonstrated integration risk):**
   keep explicit crash-dump/profile paths, process birth identities and cleanup
   evidence. Treat the route as a bounded integration, not a sandbox.
7. **Fixture omissions (medium, repaired):** freeze an asset manifest and
   composition fixture before running the model; distinguish fixture failure
   from renderer failure in the report.

## Smallest experiments that keep HyperFrames

These experiments are finite and do not change the selected package:

1. **Pinned-versus-upstream A/B.** Run the exact same frozen fixture, selected
   assets, private Node/Chrome/FFmpeg, wrapper, and independent pixel/audio
   oracle once with 0.8.197 and once with a disposable current candidate
   (0.8.207). Include one 0.8.197 no-timeline fixture and one longer/stall
   probe within existing limits. Record stage, frame index, stderr, child
   cleanup, decoded frame/audio evidence, and resource samples. This isolates
   an upstream fix from a wrapper or fixture change. Do not make the newer
   version production input from this experiment alone.
2. **Authoring matrix.** Use four otherwise identical compositions: valid
   timeline registration, valid `data-no-timeline`, missing marker, and wrong
   composition ID. Confirm preparation/strict readiness errors are distinct and
   that a successful export has intended motion at first/middle/last frames.
3. **Asset and scope matrix.** Compare a selected local asset, missing asset,
   external script/media reference, and stale source revision. Verify refusal
   occurs before launch where Mirket owns preflight, no destination appears,
   source remains unchanged, and no out-of-project file is written.
4. **Permission matrix.** Repeat `video_prepare` with host approval denied and
   with a permitted direct MCP client in a disposable home. The first must stop
   before server execution; the second must reach the existing wrapper and
   independent decode checks. This keeps host permission ownership explicit.
5. **QA matrix.** For each accepted small fixture, save first/middle/last
   decoded images, an audio window report, source/output manifests and an
   independent oracle result. Add a reviewer that cannot change the source
   while inspecting visual claims. Retain initial artifact, findings, repair,
   final artifact and remaining defects; a new specimen must not reset the
   shared review/repair budget.

## Unqualified paths and product question

The following remain unqualified: Windows native execution and packaging,
Desktop GUI activation (explicitly waived), arbitrary authored compositions,
long/high-resolution renders, player compatibility, all upstream formats not
exposed by Mirket, active process-death/lost-owner interleavings, complete
recipient source/notice/native-rights delivery, and a performance comparison.
The existing candidate docs also record no listening test with speech/music or
loudness-standard qualification.

There is no current product ambiguity requiring a replacement renderer. If a
future gate proves that Mirket needs hard control over native Codex spawning,
that would conflict with the accepted boundary that Codex owns agents; the
question would be whether to accept guidance-only coordination or change that
boundary. HyperFrames is not the cause of that ownership decision.
