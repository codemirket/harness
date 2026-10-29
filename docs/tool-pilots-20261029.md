# Tool pilot results — 2026-10-29

These are bounded experiments following the [tool landscape](tool-landscape-20261029.md),
not a new package admission or installed-product qualification. All work used
disposable private state outside the real home ancestry. The trial hosts were
macOS **26.5.2 (25F84), ARM64** and private Node **24.22.1**. No upstream
installer or hook ran, and no production dependency, lockfile, or user
installation changed. Raw commands, artifacts, failed attempts and byte
manifests remain outside shipped source at the paths below. The existing
[package qualification](qualification.md) remains scoped to its own bytes and
revision; it does not qualify these candidates.

| Trial | Observed result | Decision |
| --- | --- | --- |
| diagram-design methods with D2 | Two of three initial diagrams read more clearly; the JobState diagram lost visible outcome connections until a separate repair. | Keep the existing D2 route and add one small semantic safeguard to the diagrams skill. Do not import the upstream catalog, fonts or helpers. |
| Optional Playwright MCP as a Pi peer | Ten local interaction/accessibility/responsive/lifecycle cases passed in a disposable Pi SDK fixture after correcting a timing assertion and using Pi unregister/re-register for cancellation recovery. | Promising optional browser peer, but no production binding or admission yet. Direct same-peer close after abort hangs; exact notices/assets, scoped operation controls and browser identity need completion. |
| Excalidraw editable canvas | Five saved-scene edit/reopen/export cases passed. D2 produced clearer routing on the same content; OpenPencil lacks several tested shapes. | Do not add or replace the current routes yet. Font/WASM grants and a large dependency closure remain unresolved, and visual output needs improvement. |
| Project-supplied Anime.js with HyperFrames | Three diagnostic ten-second exports passed after one source timing repair. A success receipt in the first run concealed an early-moving subject. | Keep HyperFrames and its existing author-supplied Anime path; document timeline offsets and decoded timing checks. This does not justify Motion or another renderer. |

## Diagram method with the existing D2 executable

The selected method came from retained diagram-design commit
[`550fd66edff058bc73056c3416b76d69cfe865ba`](https://github.com/cathrynlavery/diagram-design/tree/550fd66edff058bc73056c3416b76d69cfe865ba),
used only as guidance. A single author made baseline and selected versions of
three source-grounded Mirket diagrams: Pi/Mirket ownership, the actual JobState
lifecycle, and record/revision/source provenance. Both arms used verified
private D2 **0.9.0** (`e8ba501a413bac33a4d93014c475b804de10fb2863a391583e23a8d88160e311`),
ELK and the same content and size targets. The lead inspected six initial
1280/768-width PNGs. The selected ownership/provenance layouts had clearer
grouping and labels. The initial selected lifecycle left Failed and Interrupted
visually orphaned and placed transition exceptions in tiny prose. One later
repair restored those relationships, but made the diagram tall and left
connectors crossing a group heading. That repair is separate from the matched
initial comparison; this unblinded pilot establishes no causal quality gain.

All six final D2 sources remained editable, and changing their palette changed
decoded PNG pixels. D2's direct PNG exporter produced all twelve initial
width variants using its explicit executable and no browser. At this pilot's
revision, Mirket's operation exposed **SVG only**. All twelve attempts to feed the raw D2 SVG into
Mirket's bounded resvg path failed: the SVG embeds data-URL font CSS, which the
resource validator rejects. The policy was not relaxed. A later direct D2 PNG
route has its own [installed qualification](qualification.md); this pilot is
not evidence for that package.

Raw report: `/private/tmp/mirket-diagram-method-pilot-8simymc7/REPORT.md`
(SHA-256 `8e8285f046e4c6ea23cc9ea9b27005fa0ccd5a92c18d08f75ba60757dea097fd`).
Its `SHA256SUMS.json` binds the source, images and failures. The resulting
small skill safeguard requires visible guards and connected outcomes when
simplifying repeated arrows; it does not claim the upstream method was adopted.

## Playwright MCP in a disposable Pi fixture

The exact candidate was [`@playwright/mcp` 0.0.83](https://github.com/microsoft/playwright-mcp/releases/tag/v0.0.83)
plus `playwright` and `playwright-core`
**1.64.0-alpha-1790635538000**. The three directly fetched npm archives
had SHA-256 values, respectively:

- `f0014a13361b1ee2d8607a4e187868ab9a6e8ac637dbcd66e4af43f738d0c8cb`
- `436022a21e7a7c34436970fe04ab3fccae87a5045d3ed5af3f66c6e3ba377ee8`
- `54ad1d38174d6ef56d1f165e52547ece8e6c24b865f2a072d9ebaacfbad1ca47`

Pi **1.4.0** registered the peer against a private profile and an explicit
Chrome for Testing **156.0.8078.73** executable; `PATH=/nonexistent`. The
fixture exposed six bounded browser tools. Ten final cases passed: accessible
Turkish text, reduced motion, keyboard focus/action, saved screenshot delivery
through Pi, narrow and wide layout, profile storage isolation after restart,
origin/redirect containment, absence of install/evaluation/upload tools, and
abort followed by process termination and a working fresh registration. No
model was called, so token benefit was not measured. A first narrow-layout
assertion sampled the old state immediately after resize; an observable-state
wait fixed the test, and the failed receipt was retained.

The direct `browser_close` after abort hung on both tested Chrome builds.
Inspection of this exact Playwright bundle found that `browser_navigate`
does not pass the cancellation signal through, while close reads page title
during pending navigation. The final fixture used Pi unregister to end all
nine observed old descendants in **122.50 ms**, then registered a fresh peer
and navigated successfully by **834.24 ms**. This is a lifecycle workaround,
not proof that an in-flight navigation stopped when the abort returned.
Origin filtering was paired with a local deny-all network fixture; it is not a
security boundary. A production route would also need Mirket's scoped effect
authorization and an explicit browser onboarding/binding distinct from video.

The roots declare Apache-2.0, but the supplied bundles inline **163 distinct
npm versions**. The inventory includes a CC-BY-4.0 `caniuse-lite` section,
BlueOak-1.0.0 `minimatch`, Codicon font bytes and WebP WASM with their own
attribution/source requirements. An exact selected-file notice and ancillary
asset audit is still needed. Both host Chrome and the copied Chrome for Testing
failed strict full-app codesign resource checks in this scratch setup, although
the CfT archive and file identity checks passed; neither is a production
browser binding. The final fixture used a separate private copy, not the
installed video-only Chrome binding.

Raw summary: `/private/tmp/mirket-playwright-pilot-tamfbfda/SUMMARY.md`
(SHA-256 `7d595dec31491ab6576e24069b28f620ae00048658f06d5837c716befd7b0b91`).
The final ten-case receipt is `run-jdeZVX/report.json` (SHA-256
`a2c3441bda7d3a3f913277b6dc67c00e260827f087ee92772a5e04755e5eed0e`);
failed close, resize and recovery attempts remain alongside it.

A **2026-10-30 selected-file follow-up** narrowed the peer to twelve unchanged
publisher files (6,781,681 bytes), with six runtime files observed in use and
79 embedded dependency versions. Twelve scoped Pi fixture tasks passed. The
remaining rights gap is `proxy-agent-negotiate@1.1.0`: its required bundle
declares MIT but supplies no license text. Omitting or tree-shaking that module
did not preserve the tested contract; an older core without it failed the
current screenshot schema. Full redistribution clearance was not established.
An external, explicitly acquired peer would still need its own browser binding,
scoped authorization, real loopback forwarding policy and installed Pi
qualification. The exact rights batch, failed alternatives and fixture receipts
remain outside shipped source at
`/private/tmp/mirket-playwright-rights-20261030-81_u46f3/REPORT.md` and
`/private/tmp/mirket-browser-candidate-20261030-min-03i2940m/IMPLEMENTATION.md`.

## Excalidraw against D2 and OpenPencil

The exact [`@excalidraw/excalidraw` 0.18.1](https://github.com/excalidraw/excalidraw/releases/tag/v0.18.1)
npm archive was **31,234,066 bytes**, SHA-256
`b280d4b364b65cba264c5aa4e7435cc2ce6421eabbbef9765a56997a0fadf534`;
the tagged source archive was SHA-256
`be44d1d10865ef0ab2474b59e1430eb12e55eb84a3663e7d11818b8ff6a5c93c`.
Five scratch React component cases saved `.excalidraw` scenes, reopened them
in a fresh page, changed text or arrow geometry, exported changed SVG/PNG,
and reopened the changed scene. Independent checks saw preserved semantic
bindings, changed arrowheads/text and **2,091–3,857** changed decoded pixels
per case. Turkish glyphs and locally loaded fonts were visible. This tested
component API edits, not human pointer/keyboard editing or a Mirket operation.

D2 produced changed-source/changed-export output on all five equivalent
content cases, with clearer straight routing; its existing sketch mode covers
the hand-drawn look. Excalidraw preserved canvas positions when a response
arrow was reversed, whereas D2 reordered the boxes. Excalidraw's manually
bent connectors formed conspicuous V shapes and left visible gaps. The finite
OpenPencil 0.16.0 route correctly refused arrow, diamond and multiline cases;
its supported single-line Turkish rectangle/text case passed edit/export.

The scratch install occupied **261 package locations**. The execution bundle
contains **150 distinct npm package/version pairs**, about **12.8 MB** of JS,
**234 WOFF2 files**, and embedded HarfBuzz/WOFF2 WASM. The root source is MIT,
but its npm archive omits that license file. The selected closure still needs
complete notices and build/source correspondence. In particular, the exact
Cascadia font embeds mixed grant language, old Liberation 1.05 points to an
unspecified license agreement, and the embedded WASM has underlying
HarfBuzz/WOFF2/Brotli/Emscripten components. These issues were left explicit,
not inferred clear from the root MIT grant. No browser installation or remote
font fallback was used.

Raw report: `/private/tmp/mirket-excalidraw-pilot-20261029/REPORT.md`
(SHA-256 `540c4ea4122c1309d279b5fd0a2d3a0d3c89de49143cead176325d7cef15a169`).
Its adjacent files preserve exact archive/font/WASM/selected-package hashes,
five exports, comparison images and failed attempts.

## Project-supplied Anime.js with HyperFrames

The default Mirket package has **no Anime.js bytes or dependency entry**.
The tested `animejs` **4.5.0** UMD file was an explicitly selected project
asset fetched into scratch from the npm archive (SHA-256
`f1bf4be22ae739bbfefd3ec3651a37a7d178dfa8ce782b0ebe5e39347fcb1706`,
UMD SHA-256 `8d5b3a58a1f64023a04a4cedeef135a2263b18da04b35177ac13438a0bad033b`),
with registry integrity and MIT license retained. The prior test file named
`anime.umd.min.js` was a 54-byte no-op around native WAAPI, so it did not
qualify Anime.js behavior. This pilot used the final package's native release
`sha256:1e42a96ef7c776646969f1c3d0c2f57c9852efe62afb8b6b981d6c6cbe478708`
through fresh MCP prepare/start/status/result/cancel operations. It was not
an installed Pi CLI or model journey.

Three source-grounded, simple 640×360 diagnostic briefs each requested a
ten-second, 120-frame export with one-second opening and final holds, an
eight-second subject translation, three captions and a local timing tone.
All three first-run receipts reported success, yet decoded frames showed the
subject at x=178/315/452 at 2.5/5/7.5 seconds instead of the expected
approximately 122/260/398. A direct Anime API probe found that seeking an
`animate({delay:1000})` instance excludes that delay from seek time. One shared
authoring repair registered a timeline with an explicit 1000 ms placement.
All three repaired exports then matched the expected five positions within
one decoded pixel, including opening/final holds. This is a concrete
source-authoring pitfall, not evidence that HyperFrames itself is broken.

The repaired MP4s independently decoded to 120 H.264/AAC frames over ten
seconds. Caption transitions matched the requested frame boundaries; decoded
PCM put the tone at 0.50004–1.49996 seconds. A changed title changed 3,602
heading pixels. Saved-video forward/backward seeks matched the decoded subject
positions, and normal-speed playback ended at 10.076, 10.057 and 10.073
seconds. In-flight cancellation published no destination and verified owned
cleanup. An initial file-origin player attempt and then a loopback player
without byte-range support failed; the corrected private player served ranges
and those failures remain in the packet. The lead inspected initial and
repaired decoded frames. These simple cards show timing fidelity after one
repair, not polished creative quality or a measured advantage over Motion.

Raw report: `/private/tmp/mirket-anime-benchmark-goaom_3g/REPORT.md`
(SHA-256 `d530cefda7225f3b552eeb55dae748a4b02e273ec1e57b4b391f88b57a90bdb5`).
`SUMMARY.json`, `COMMANDS.txt`, `identity.json`, the initial and repaired
decoded frames, failure reports and source assets retain the exact scope.

## Decision at the pilot revision

The existing package remained the supported path at this pilot revision. The
diagram and Anime authoring safeguards were small source changes; none of these
trials admitted a new production runtime.
Browser-facing skills now describe Playwright MCP as conditional on an
explicitly configured and qualified Pi peer, rather than claiming that the
scratch pilot or a stable tool name makes it available in the package.
Playwright was the first optional integration candidate. Its later external-only
implementation, unresolved notice boundary and distinct installed-package
results are recorded in [packaging](../packaging/README.md),
[architecture](architecture.md) and [qualification](qualification.md); this
pilot is not counted as evidence for those later bytes. Excalidraw remains
deferred until its exact
font/WASM rights and visual-routing issues are resolved and its additional
editable-canvas value warrants the package cost. Other platforms require
their own native tests. The repository's `AGENTS.md` requires asking before a
new production dependency unless its **exact admission batch** was already
approved; the earlier landscape review did not provide that approval.
