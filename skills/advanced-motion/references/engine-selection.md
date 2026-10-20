# Choose an animation owner

Use when selecting or repairing the animation layer. First inspect the installed
versions, current property owners and actual behavior. Keep a working project
stack unless a demonstrated requirement justifies migration. A registry demo or
source listing does not establish accessibility, performance or reuse rights.

| Requirement | Candidate and decision boundary |
| --- | --- |
| Finite hover, disclosure, entry or feedback | CSS/WAAPI; native layout and semantic state own meaning. Avoid another runtime for a few transitions. |
| React presence, layout and gestures | Motion when its component/lifecycle model fits the app; exercise interruption and mount/unmount behavior. |
| Framework-independent DOM/SVG choreography | Anime.js when its timeline, scope and SVG tools cover the scene. |
| Elaborate coordinated timeline, measured scroll or pinned scene | GSAP when the actual scene warrants it; use the existing GSAP integration reference for cleanup and scroll math. |
| Focus, keyboard, dialog/menu state and semantics | Existing accessible primitives such as Radix own behavior; the animation engine only visualizes state. |
| Offline frame-based video | Keep the proven renderer. Compare HyperFrames with Remotion only on a representative deterministic scene, seeking, fonts, encode and editability; changing source format alone is not a quality win. |

Animate.css can suit an existing class-based effect vocabulary. ScrollReveal is
narrowly focused on entry effects and has distinct license choices. mo.js is a
specialized burst/shape tool. These are candidates for particular needs, not a
reason to accumulate overlapping engines. Inspect current package licenses and
redistribution needs before adopting; source availability is not permission.

## One property, one owner

Avoid CSS, a gesture library and a timeline writing the same transform or opacity.
Use separate wrappers when layout and artwork need independent transforms. Define
which event interrupts motion, where it settles, whether it reverses, and who
releases observers/listeners/RAF/timers. Timeline cleanup alone does not cancel
unrelated asynchronous callbacks. Cancel delayed work before it can recreate a
scene after teardown. Exercise two instances and remove one while the other runs.

For exit transitions, coordinate the semantic primitive's presence lifecycle with
the animation completion/cancellation contract. Do not infer that arbitrary JS
motion will delay a primitive's unmount. If retaining DOM (for example forceMount),
verify focus restoration, hidden/inert behavior and accessibility-tree state;
invisible focusable controls are a defect. Test rapid open-close-open and removal
mid-exit. Animation must not be the only mechanism that commits application state.

## Reduced motion and initial visibility

Choose the usable reduced-motion composition first: final content and primary
actions stay available. Check initial preference and a preference change during
playback. A hook or CSS duration override may not cancel running timelines,
delays, gesture inertia or scroll pinning. Verify the installed implementation;
subscribe/clean up the preference change where necessary. Restore the settled
state and clear pending delays. No-JS/failed-load content must remain readable;
apply initial hiding only after a working animation owner has mounted.

For split text, preserve a complete accessible label and visual word wrapping.
Split grapheme clusters rather than code units; keep Arabic joining and bidi
runs intact unless the shaping system supports the intended treatment. Avoid
announcing every animated letter separately.

## Prove the choice on the actual consumer

Use one representative scene: trigger, reverse/interrupt, resize, navigate away,
return, toggle reduced motion and run two instances. Inspect meaningful intermediate
poses and settled state, not just screenshots of endpoints. Measure dropped frames,
layout work and shipped bytes on the target device/browser before claiming gains.
A different engine is justified by a reproduced gap and a better measured result,
not by the number of features in its README. A passing build is not playback proof.

## Source boundaries

Independently authored selection criteria informed by the reviewed implementations:
[Anime.js](https://github.com/juliangarnier/anime/tree/01b81be1df6843ccfe0a71c0699a746bf740dd77),
[Motion](https://github.com/motiondivision/motion/tree/c5fab238ab4f8d8132d1cb05d6607439b3904058),
[Radix](https://github.com/radix-ui/primitives/tree/eb3a2b7091293e27df8f9ba906632d8daa3d81a3)
and the existing pinned GSAP integration guidance. No runtime or demo code is
included. API and licensing decisions must be checked against the chosen installed
version. Remotion and template/asset rights need separate review before adoption.
