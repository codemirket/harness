# Remotion as a reference for HyperFrames authoring

Research date: 2026-10-27 (Europe/Istanbul). Scope is the Remotion authoring
model and its public agent guidance, not a proposal to replace HyperFrames or
to add a second renderer. Sources were discovered and checked as public
primary material. Dated pages are used for dated claims; current API docs are
explicitly background, not adoption evidence. No Remotion installer, package,
or upstream code was run or added.

## Executive conclusion

Remotion's most transferable idea is an explicit temporal model: every frame
is a deterministic function of finite composition metadata and input props,
with scene-relative timing and a preview/sample loop before export. Its agent
guidance exposes named, validated design controls and routes specialized work
to small skills. The live design-system probe also shows a concrete authoring
UX that Mirket does not currently provide: source-located composition context,
seekable frames, and reusable animated assets with visible controls.

That evidence supports a project-owned seekable preview, stable source context,
and reusable animated-asset library while preserving HyperFrames; it does not
establish that Mirket needs the Remotion engine, React, Studio, Zod, remote
assets, or a second capture path. Other bounded opportunities are explicit
frame units, a small frame-sample manifest, readiness gates, and post-package
reference checks. The current skills already require saved-pixel and decoded
output evidence, local revision-bound assets, a bundled font, caption safe
area, and one shared repair budget; those remain stricter requirements.

## Research ledger and source findings

Each summary below is deliberately limited to the source's relevant method,
observed limit, and adoption signal. “Observed” means present in the reviewed
page or immutable source; it is not an independent quality or uplift result.

### 1. Current Remotion Agent Skills source

Source: [remotion-dev/skills commit 30d4553](https://github.com/remotion-dev/skills/commit/30d4553ecd46fbc5212fd0ff006ae43af522158b), pushed 2026-10-26. GitHub's [commit API](https://api.github.com/repos/remotion-dev/skills/commits/30d4553ecd46fbc5212fd0ff006ae43af522158b) resolves the full immutable SHA to `30d4553ecd46fbc5212fd0ff006ae43af522158b`; the reviewed files report version 4.0.539. The current router now points to `REFERENCE.md` subskills, asks an agent to open Studio before editing, and defers rendering unless explicitly requested. Current markup keeps `useCurrentFrame()` in the smallest frame-dependent component, uses `fps` for `from`/`trimBefore`/`durationInFrames`, keeps editable interpolations inline, and routes interactive timeline structure to separate references. Current render guidance supports stills and `render --frames=...` samples. These are useful methods, not measured superiority; the current source is the primary adoption reference.

The [repository API](https://api.github.com/repos/remotion-dev/skills) identifies
this commit as current `main` at review time. The source is pinned here so
future changes can be distinguished from current behavior; a moving URL is not
treated as a stable contract.

### 2. Remotion's dated release and tool-maintenance evidence

The [v4.0.539 release](https://github.com/remotion-dev/remotion/releases/tag/v4.0.539) is dated 2026-10-26 and contains fixes for spring overshoot clamping, hue angles, multi-keyframe loops, `prefetch()` cleanup, animated-image caching, and capture behavior; it matches the current skills source version. Remotion's [blog](https://www.remotion.dev/blog) dates Media Parser to 2025-05-20, Editor Starter to 2025-08-21, and Mediabunny sponsorship to 2025-09-01; a later note says Media Parser/WebCodecs are being phased out in favor of Mediabunny. This is a concrete version-drift/tool-maintenance signal, not proof that Mediabunny is better for Mirket. HyperFrames should retain its pinned producer and independently requalify any replacement rather than following moving docs or adding a dependency.

### 3. Current frame and composition documentation (background)

The current [frame model](https://www.remotion.dev/docs/idea) describes a video as images indexed by finite frame number, with width, height, duration, and fps; frame zero is the first frame and `durationInFrames - 1` the last. [`useCurrentFrame`](https://www.remotion.dev/docs/use-current-frame) is parent-relative inside a timed sequence, while [`Sequence`](https://www.remotion.dev/docs/sequence) shifts children by `from` and bounds them by `durationInFrames`. In the pinned markup guidance, `from`, `trimBefore`, `durationInFrames`, and `premountFor` are all frame-based values derived from `fps`; that explicit unit boundary is the useful detail. This is current API background observed 2026-10-27, not dated adoption evidence. The transferable portion is explicit frame arithmetic and scene boundaries. The non-transferable portion is React's component tree and Remotion's renderer. HyperFrames should express the same invariants in its existing HTML root, finite fps/frame-count contract, and decoded frame checks.

### 4. Props, schemas, and visual editing (background)

The current [parameterized rendering](https://www.remotion.dev/docs/parameterized-rendering), [schemas](https://www.remotion.dev/docs/schemas), and [visual editing](https://www.remotion.dev/docs/visual-editing) docs describe defaults as an editable shape, optional Zod validation/constraints/descriptions, dynamic metadata calculation, timeline transforms/keyframes, undo, and render-dialog input props. This is a useful control-surface pattern: expose only controls a producer can validate and retain defaults so a specimen is reproducible. It does not establish that Studio edits improve final quality. Mirket should keep controls in project-owned HTML/JSON/source and validate finite dimensions, frame ranges, local assets, and captions at its existing preflight boundary; it should not add Studio, Zod, or a new editor merely to resemble Remotion.

### 5. Preview, fonts, captions, and design systems (background)

Current [Studio guidance](https://www.remotion.dev/docs/studio), [font guidance](https://www.remotion.dev/docs/fonts), [layout best practices](https://www.remotion.dev/docs/layout-utils/best-practices), [captions](https://www.remotion.dev/docs/captions), and [design systems](https://www.remotion.dev/docs/design-systems) show a preview-first workflow, waiting for fonts before text measurement, cancellation on failed readiness, first-class SRT/VTT import/export, and reusable parameterized compositions. The [Showcase](https://www.remotion.dev/showcase) is a curated gallery, not a controlled quality comparison; no causal uplift or independent visual oracle is supplied. Transfer the readiness gates and preview/sample discipline. Keep Mirket's stricter local-font and local-asset policy: Remotion examples can use network/remote resources, which is not an acceptable production assumption for the HyperFrames route.

A separate live probe of Remotion's design-system WebMCP surface (recorded in
[the comparison report](research-20261027-motion-authoring-comparison.md), not treated as a Remotion vendor claim) did more than list
tools: it selected a composition, inspected sequences, sought frame 45, and
returned a playback state for a 450-frame 1080-square/30-fps animated logo.
Live screen inspection saw frame 0's transparent checkerboard and frame 45's
logo/wordmark, with a user-facing theme-prop form visible. This supports
source-located preview and seekable design-system inspection as a useful
method. It did not establish export correctness, continuous playback quality,
or an art-direction uplift; those remain unqualified.

### 6. Agent routing failures that are useful counterevidence

The public [skills issue #24](https://github.com/remotion-dev/skills/issues/24), opened 2026-07-25, reports a historical mismatch between `REFERENCE.md` links and installed `SKILL.md` names. [Issue #20](https://github.com/remotion-dev/skills/issues/20), opened 2026-07-02, reports an example that used hooks and helpers without importing them. [Issue #19](https://github.com/remotion-dev/skills/issues/19), opened 2026-07-01, reports `trimBefore`/`trimAfter` being described as seconds although the API expects frames, and a non-destructured config value producing `NaN`. Current HEAD now uses `REFERENCE.md` routing and includes a complete import example, so these issues are historical regression tests rather than proof that current behavior remains broken. They still justify post-assembly link checks, compile/lint checks, and visible frame/seconds units in Mirket's own resources.

### License and reuse boundary

Remotion core's [LICENSE.md](https://github.com/remotion-dev/remotion/blob/main/LICENSE.md) is the custom “Remotion License”, not MIT, Apache, or an OSI-approved license. It has eligibility and commercial limits, including a paid-license boundary for larger for-profit organizations. The [skills repository API](https://api.github.com/repos/remotion-dev/skills) reports `license: null`; its [main tree](https://api.github.com/repos/remotion-dev/skills/git/trees/main?recursive=1) has no `LICENSE`, `LICENSE.md`, `COPYING`, or `NOTICE` entry at the time of review. A public repository, a raw URL, or a source-available package is not permission to copy skill prose, code, fonts, or brand assets. The blog's statement that Mediabunny is MPL-2.0 applies to that separate project and does not alter Remotion's license. This report therefore records methods and links only; no upstream assets or text were incorporated.

## Transferable methods mapped to Mirket

| Remotion method | HyperFrames application | Observable evidence | Non-transfer constraint |
| --- | --- | --- | --- |
| Frame-first deterministic clock | Define finite fps, frame count, and scene/beat boundaries before authoring. Keep timing conversions explicit and review first/middle/final samples. | Root metadata, decoded frame count/order/timing, and an independent pixel oracle. | Do not import React hooks or assume arbitrary CSS transitions are seekable. |
| Parent-relative temporal composition | Give each beat/track one timing owner and state its enter/hold/exit/resting frames; treat nested offsets as a written mapping. | Timed storyboard plus decoded samples at scene boundaries and interruption/replay checks where interactive. | HyperFrames remains an HTML/CSS/JS source under its pinned producer; no second timeline engine. |
| Validated design props | Expose only named controls that affect the specimen (copy, color, timing, asset, dimensions); retain defaults and reject invalid frame ranges/assets. | Source manifest, preflight receipt plus actual output checks; a changed prop must change the intended output. | No Studio/Zod dependency or claim that an editor's undo is evidence of quality. |
| Preview before export | Use the qualified private Chromium route for a bounded specimen; inspect representative frames before spending a full export. | Saved pixels independently viewed, then decoded MP4/PNG/audio/caption checks. | Render success/hash/receipt remains insufficient; Mirket's existing approval and path rules stay in force. |
| Font/media/caption readiness | Block until selected local fonts and caption sources are present, loaded, in range, and safe-area tested. | Explicit local font declaration + selected binary; decoded captioned export and audio/metadata checks. | No network Google fonts, remote media, or dynamic fetch/import in the local-only route. |
| Small specialized agent routes | Keep authoring, preview, render, caption, and upgrade guidance separate and route only to the capability that exists. | Resource-link smoke tests, runtime status, actual operation outcome, and failure receipt. | Host owns agents, model choice, permissions, browsing, and Git; Mirket cannot intercept native spawning. |
| Version/pin discipline | Record producer/reference pin and requalify when release behavior or media stack changes. | Pin/hash, platform/runtime identity, and fresh decoded evidence. | Do not follow current Remotion docs as if they describe HyperFrames 0.8.197. |

## Observed opportunities and bounded recommendations

These are evidence-backed opportunities found by comparing the methods above with
`skills/video-production/SKILL.md`, `skills/motion-direction/SKILL.md`, the
pinned HyperFrames authoring contract, and
`docs/design-adoption-video-20261027.md`.

1. **Make frame-unit errors harder to author.** The Mirket contract is
   explicit about finite fps/frame count, but its prose mostly gives duration
   in seconds and leaves the conversion to the author. Remotion's issue #19
   demonstrates a real failure mode. Add a compact authoring example/table in
   the existing reference that maps seconds → rounded frame index, states
   zero-based/end-exclusive ranges, and labels every timing field. Add a
   discriminating fixture where a seconds-as-frames error would shift a beat;
   verify decoded boundary frames. Do not add a new runtime.

2. **Add a frame-sample manifest to the existing video workflow.** The
   current skill requires representative inspection and the retained native
   evidence already inspects first/middle/final frames, but the workflow does
   not standardize a small named sample set. A manifest such as
   `first`, each scene boundary, `middle`, and `last`, with expected subject
   or caption state, would make preview-to-export comparison reproducible.
   It must remain evidence, not a quality claim; saved pixels and decoded
   exports still require independent inspection.

3. **Retain the strict local readiness gate.** The frozen model trials in
   `design-adoption-video-20261027.md` refused two roots missing
   `data-no-timeline` and one root missing a selected font binary. Remotion's
   font-measurement guidance supports the underlying readiness rationale, but
   its remote/Google-font examples must not weaken Mirket's local policy.
   Improve the error text/examples only if needed to distinguish “font loaded”
   from “font selected and revision-bound”; no fallback to a host font.

4. **Test reference packaging, not just source hashes.** Issues #24 and
   #20 show that a receipt or source file can exist while routing or examples
   are unusable. The repository's skill/resource checks should resolve every
   shipped relative reference after packaging and compile/lint executable
   snippets where a skill promises code. A successful catalog hash is not
   evidence that the linked body exists or that an example runs.

5. **State the control-surface boundary.** The current skills correctly
   expose a small set of controls but do not explicitly say that a control
   must have a source-level owner and an observable validation rule. Add one
   sentence to the existing guidance and a fixture with two controls targeting
   the same property; reject ambiguous ownership before export. This supports
   the existing motion-direction rule rather than introducing a UI editor.

6. **Pin and label current references.** The current skills source is pinned at
   full SHA `30d4553ecd46fbc5212fd0ff006ae43af522158b`, while the HyperFrames
   route remains pinned at 0.8.197. Keep comparison links labelled as
   background or source locators; a moving documentation URL must not silently
   change the producer contract or be treated as a qualification result.

## Non-transferable or rejected changes

- **React/Studio/second renderer:** Remotion's component tree, Studio timeline,
  and CLI renderer are an architecture reference only. Mirket's accepted
  boundary is a pinned HyperFrames producer through qualified private
  Chromium/FFmpeg; adding a parallel route would invalidate the evidence model.
- **Remote assets and Google fonts:** upstream convenience is incompatible with
  revision-bound local assets, explicit bundled fonts, and deterministic
  disposable-project qualification.
- **Copying Remotion skills or assets:** the skills repository has no stated
  license; core has a custom license. Retain only independently worded
  principles and links.
- **Replacing SQLite/FTS or HyperFrames for novelty:** this research supplies
  no performance or consistency evidence for such a change.
- **Showcase claims:** attractive gallery examples do not establish visual
  inspection, accessibility, playback, or output correctness. The existing
  pixel/frame/audio/caption oracles remain the required evidence.

## Relation to current HyperFrames evidence

The retained native run in
`docs/design-adoption-video-20261027.md` is stronger than a Remotion receipt:
independent decoded pixels show a registered Anime object moving at sampled
positions while an otherwise similar unregistered object remains frozen. The
same record reports three frozen model specimens refused before rendering
(two missing `data-no-timeline`; one missing a selected local font binary), so
there is no honest model-export or decoded-frame claim for those inputs. The
Remotion methods above therefore reinforce, rather than replace, Mirket's
existing refusal and evidence rules. They suggest better manifests and
reference packaging checks, not a conclusion that HyperFrames or Mirket's
current route is visually equivalent to Remotion.

## Evidence limits and unqualified paths

- No independent benchmark compared Remotion output quality with HyperFrames;
  no uplift, speed, or adoption claim is made.
- Remotion Showcase pages were not treated as visual QA, and no upstream package was installed and no source/assets were copied into the harness.
- Current Remotion docs were read for API meaning only; their “updated” dates
  do not prove dated user adoption.
- Windows qualification and the waived Codex Desktop GUI check remain
  unverified under the project plan.
- This report is research only. It makes no production, catalog, installation,
  release, or live `~/.mirket` change.

