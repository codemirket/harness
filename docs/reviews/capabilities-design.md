# Design capability review

Reviewed 2026-10-09 against current source bodies, catalog adaptations and a
bounded Fieldwork brand exercise. This is evidence of specific workflows and
tested artifacts, not a claim that every future design task will meet its brief.
Source files were read directly; catalog presence alone was not counted as use.

## Capability map and acceptance boundaries

| Outcome | Exact skill IDs / source | Realistic acceptance evidence |
| --- | --- | --- |
| Web screen and public-page design | Authored `interface-design`; `frontend-engineering` supports behavior. Application/public-page/art-direction references distinguish composition needs. | Inspect the real page at target widths with real content, states, keyboard and task completion. Compare a relevant reference; human visual acceptance remains separate. |
| Mobile product design | New authored `mobile-design`; adapted `emil-mobile-native` (invocation `mobile-native`) remains a mobile-web defect recipe. Native profiles select actual engineering skills below. | Browser reflow plus platform navigation/input contract; native build/device/assistive checks only where actually run. CSS viewport emulation is not native-device evidence. |
| Brand guidelines | New authored `brand-guidelines` with Fieldwork worked reference and standard-library contrast helper. | Extract source and rendered tokens, apply them in a second specimen, inspect actual mark/assets/copy, measure specific pairs and document observed/proposed/approved distinctions. |
| Editable vector illustration and assets | Authored `svg-creation`, `scripts/audit_svg.py` and vector-craft reference. | Editable geometry; multi-size/background render; clipping/optical/instance-ID review; actual embedding. XML audit cannot certify aesthetics or sanitize uploads. |
| Raster illustration and image assets | Available native image-generation capability; `interface-design` workbench illustration recipe for placement; `brand-guidelines` for asset roles. No portable image-generation runtime is supplied. | Inspect original pixels, intended crop, text contrast, dimensions, provenance and real placement. Generated output is not approval; bitmap in SVG is not vector source. |
| Ordinary interface animation | Authored/adapted `motion-design` and interaction recipes. | Actual input, enter/exit, reversal, rapid repeat, cleanup, reduced motion and playback/frames; profile when claiming performance. |
| Navigation animation | Authored `page-transitions` with navigation-craft reference. | Real back/forward, deep entry, late data, skipped visual path, focus/scroll and exactly one semantic commit; normal/reduced playback. |
| Complex DOM choreography | Authored `advanced-motion` with scene-mechanics reference. | Timeline/layout/scroll/3D states, resize/invalidation, interruption, cleanup and intermediate geometry; a final frame does not validate choreography. |
| SVG animation | Authored `svg-animation`, supported by `svg-creation`. | Actual embedding, static meaning, meaningful beats, replay/pause, reduced motion at load/live change, two instances and disposal. Video/export needs its own runtime and checks. |
| Design revision | Global `interface-design` leads; optional `taste-redesign-skill` (invocation `redesign-existing-projects`) has an explicit opt-in adaptation. | Baseline and revised renders under matching conditions; preserve approved brand/content/behavior; defects versus preferences; exercise affected interactions. No mandatory style replacement. |
| Interface QA | `test-design` with quality-assurance reference, supported by `interface-design`; adapted `emil-break-ui`; `openai-playwright` only with a ready approved runtime. | Follow changed state through reload or second read, failure and affected input/roles. Located defects with reproduction. Screen capture is not a full accessibility or release audit. |

Native engineering profiles already contain concrete IDs:

- Apple: `ecc-swiftui-patterns`, `ecc-swift-protocol-di-testing`, `test-design`.
- Android: `ecc-android-clean-architecture`, `ecc-compose-multiplatform-patterns`,
  `ecc-kotlin-coroutines-flows`, `test-design`.
- React Native: `ecc-react-native-patterns`, `test-design`, `motion-design`;
  browser motion recipes are not native animation API evidence.
- Flutter: `ecc-flutter-dart-code-review`, `test-design`, `architecture-review`;
  the profile explicitly limits complex implementation examples.

These catalog/profile observations do not establish native SDK readiness or a
successful native app build. Source-level routes for mobile design must select
the actual platform implementation, not call the browser recipe universal.

## Gaps corrected in source guidance

`taste-brandkit` and `ui-ux-pro-max-brand` remain `manual` review candidates.
They were not promoted merely to fill a capability list. The new authored brand
workflow supplies an inspectable token/mark/voice/export contract and a bounded
calculation helper, without external image services or brand-script dependencies.

The adapted `emil-mobile-native` specifically scopes itself to mobile web; a
new mobile product-design workflow now covers journey/back behavior, target
units, native controls, keyboards/insets, scaling, accessibility and state
restoration. Its worked reference explicitly distinguishes iOS, Android and web.

Existing interface, SVG, control/navigation/advanced motion and redesign routes
already specify useful composition and behavioral checks. Their unchanged bodies
were reviewed read-only. No unsupported new runtime or mandatory ceremony was
added. `vercel-web-design-guidelines` remains manual because its wrapper fetches
an unpinned remote guideline; do not count that listing as deterministic QA.

## Worked brand evidence

Task: document the real Fieldwork identity and apply it in a compact brand sheet,
while preserving the source application. Ran against
`examples/craft-lab/index.html`, SHA256
`d64bfa6c9e7a5b0d3557d34596c39b882f153184ce18256d0f02f7b2364b2a1e`.

Ignored local evidence is under `build/capability-design/`:

- `brand-spec.md`: usable mini specification with observed/proposed decisions,
  concrete inconsistencies and limitations.
- `brand-specimen.html` / `brand-specimen.png`: actual second application of the
  extracted palette, mark, type, voice, material family and measured pair table.
- `observations.json` / `tokens-observed.json`: computed styles, control bounds,
  source hash, actual labels and browser version.
- `fieldwork-desktop.png` / `fieldwork-mobile.png`: actual source application;
  mobile capture includes focus returned after preview dismissal.
- `fieldwork-mark-forest.svg`, six `material-N-master.svg`: editable exports
  from actual DOM, not newly fabricated brand assets.
- `contrast.json`: opaque sRGB measurements. `probe.cjs` records the bounded
  read-only extraction/render operation through the existing configured runtime.

Chrome 154.0.8037.99 rendered desktop 1440×1000 and mobile 390×844. Inspected the
brand sheet, the actual mobile screen and original asset family. Horizontal
overflow measured 0. Text-pair ratios were 12.7002 (Ink/Paper), 5.1363
(Muted/Surface), 7.8771 (white/Forest); focus/Paper was 3.9855. The numerical
thresholds do not establish whole-page accessibility.

The guide produced specific findings: differing card/dialog save vocabulary;
two focus outlines on a focused card; 10–11px supporting type needing actual
scaling/device review; and low-contrast rule/input-boundary pairs 1.2248/1.3981
that should not become the sole required indicator. Low contrast on a decorative
separator is not itself a WCAG violation. Mobile targets measured 34px favorite,
38px filter height and 42px search height; these are CSS measurements, not native
44pt/48dp checks. These findings were recorded without changing the specimen.

The contrast helper passed black/white 21:1, identical-color 1:1, symmetry and
invalid-format rejection probes. The standalone mark passed the existing SVG
structural audit. Bundled skill-creator `quick_validate.py` could not execute
because PyYAML is absent in both available Python runtimes; no dependency was
installed. Standard-library frontmatter/reference/script checks are recorded in
the local validation artifact. Catalog registration is integrated separately by
the primary task; this review does not claim that files on disk prove activation.

This was a self-applied worked exercise by the author, not an independent blinded
skill benchmark. It demonstrates usable artifacts and located findings. Existing
`build/browser-proof/report.json` separately records four browser scenario runs
and 24 PNGs with passing checks; it does not establish native mobile behavior,
screen-reader operation, brand approval or subjective motion quality.

## Independent contract review

Read the visual entries in `registry/capabilities.json` separately from authoring
the skills. Its scope explicitly describes workflow coverage, not certification
or automatic activation; deliverables are acceptance contracts rather than claims
that every domain has already passed a live task. No broad runtime proof follows
from that registry.

Integration feedback sent to the primary author:

- `visual-assets` should make the raster-provider branch as explicit as
  `illustration`; a fixed SVG lead cannot supply a requested bitmap workflow.
- `mobile-design` should distinguish simulator build/layout evidence from
  physical input/gesture claims requiring matching hardware. The new skill
  already makes this distinction.
- `web-design` should permit supplied or proposed direction/content for a new
  product, rather than treating an existing design system as a prerequisite.

Other visual contracts correctly separate state/behavior, source/editability,
rendering and acceptance. The animation prerequisite acknowledges native/video
as different deliverables; no current browser test certifies those engines.
These are source-contract corrections, not a request to install more runtimes.

## Sources and remaining limits

Primary sources were read on 2026-10-09: [Apple's UI design
tips](https://developer.apple.com/design/tips/), [Android accessibility API
defaults](https://developer.android.com/develop/ui/compose/accessibility/api-defaults),
[adaptive navigation](https://developer.android.com/develop/ui/compose/layouts/adaptive/build-adaptive-navigation),
[window insets](https://developer.android.com/develop/ui/compose/system/insets),
[W3C target size](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html),
[text contrast](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html),
[non-text contrast](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html),
and [Primer color roles](https://primer.style/product/getting-started/foundations/color-usage/).

No native build, simulator/device, mobile keyboard, VoiceOver/TalkBack, print
proof, dark-theme audit, generated raster image or human visual acceptance was
performed in this bounded exercise. Those are explicit evidence gaps, not
completed capabilities. The new skills make the next test concrete without
pretending to supply runtimes, asset rights or guaranteed aesthetic judgment.
