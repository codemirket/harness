# Local design qualification

> **Dated qualification scope.** The installed fixture below still included the
> former `documents` component; document/Office work is now host-owned. The
> described HTML/CSS inspection experiment and its package inventory do not
> imply an admitted private browser. Current design-token consumer evidence is
> in [the token qualification](m4-token-consumption-qualification.md), and the
> later browser installation is in [browser acquisition](m3-browser-setup-acquisition.md).

The macOS ARM64 design component packages Style Dictionary 5.7.1, SVGO 4.1.0,
SAX 1.6.1, an exact private Node 24.22.1 runtime and a private Noto Sans font.
SVG-to-PNG rendering uses the Rust `resvg` 0.49.0 dependency in a bounded
same-binary child. The native `@resvg/resvg-js` package is excluded because its
published archive does not provide a reproducible native-source closure for this
release. This fixture used the [original design inventory](../packaging/admission/m4-design-macos.runtime.json)
of 6,128 production package files. The current
[Chromium-only selection](m4-design-chromium-file-selection.md) retains 6,024
package files while preserving every non-Playwright pin, the font, notices and
the [frozen design lock](../packaging/admission/m4-design-package-lock.json).
[`assemble-design-runtime.py`](../scripts/assemble-design-runtime.py)
copies only verified files; it does not invoke npm or upstream hooks.
Adding HyperFrames to the shared development lock did not change the retained
design files or resolved package entries. The release validator compares the
original lock entries against the current lock and packages only the active
Design selection. HyperFrames and its added packages
remain outside the admitted design component and require their own full
inventory and rights check before a video release can include them.
The shared adapter lock also includes `playwright-core` 1.64.1 and the pinned
HTML/CSS inspection libraries, with their embedded and supplemental notices.
The HTML/CSS MCP route passed its separate [installed qualification](m3-web-inspection-transport.md).
Packaging the browser adapter does not establish an admitted native browser engine.

`design_prepare` binds an explicit project, saved source revision, finite
operation and absent output path. `design_start` uses the shared admission,
receipt, cancellation and artifact owners. It supports JSON design tokens to
CSS or nested JSON, bounded SVG optimization, and SVG-to-PNG rendering up to
2048 × 2048 pixels. Inputs are capped at 1 MiB and outputs at 16 MiB. The Node
adapter and raster child receive private home/temp paths, cleared environments
and deadlines. SVG validation rejects active content and external resources;
the raster renderer loads only the admitted private font and does not resolve
system fonts or remote assets. Rendered output preserves source aspect ratio
inside the requested canvas. These are input and process limits, not an OS
sandbox or proof that arbitrary SVG artwork is visually correct.

On macOS ARM64, the disposable installed release
`sha256:a2b699db684ccf60c0eb2e579365416a1df4f9ceef0e09281213c5983fc0d13c`
passed [`design-installed-smoke.mjs`](../tests/design-installed-smoke.mjs) with
all four current components present: documents, diagram, design and semantic.
The MCP test created CSS, optimized SVG and a 400 × 160 PNG with visible private
font glyphs; it reopened saved bytes and matched their receipt revisions. It
also refused an external SVG resource in both optimization and rasterization,
left no destination after either refusal, reported complete scratch cleanup,
and rejected a stale release. The installed home and project were disposable;
the user installation and client settings were unchanged.

A later [token-consumption check](m4-token-consumption-qualification.md) compiled Style Dictionary CSS through the MCP route and used it in one unchanged responsive HTML component. Changing a source color changed saved browser pixels, while omitting that token produced a detected visible failure. The browser snapshot remains development-only and unadmitted.
The [current Chromium-only Design selection](m4-design-chromium-file-selection.md)
also passed a separate disposable installed release on macOS 26.5.2 ARM64.
Its manifest contains 6,024 selected package files and the unchanged installed
Design and JS/TS navigation tests passed after guarded assembly, activation
and healthy doctor checks. The saved release identity is
`sha256:896a1b6e710f8de9fb0dcc97fcf4b46529090c8fa80f719e7ed0f3aae40fd03e`.
The historical four-component fixture above is not the current release layout.
At the initial design qualification, 404 Rust tests passed across unit and integration
targets, with four opt-in tests skipped; 38 adapter tests passed with one native
fixture skipped, and Clippy was warning-free on the same platform. The PNG was
visually inspected in a separate direct renderer fixture. This qualifies the
finite local design route, not editable OpenPencil files, raster preparation,
browser-assisted interface design, motion/video, Windows, actual Codex Desktop
activation or release signing. Those remain separate implementation and native
qualification work.
