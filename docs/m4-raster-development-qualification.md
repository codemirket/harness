# Finite raster design jobs — development qualification

> **Historical native candidate.** The active closed Raster selection now uses
> [upstream prebuilt archives](m4-raster-prebuilt-qualification.md). The job
> contract and smoke test remain applicable to that selection.

Observed 2026-10-26 on macOS ARM64. This is an owned application path for the
reduced Sharp 0.35.5/libvips 8.18.7 candidate, **not a release admission**. Its
native source and rights evidence is in
[the source closure](video-native-source-closure.md). The published design
component still contains only its previously admitted token/SVG dependencies;
installed releases report `design.raster_configured=false` until a separately
admitted private raster package is supplied. No host Sharp lookup is used by an
installed operation.

`design_prepare` now accepts `raster_transform` with explicit PNG, JPEG,
WebP or AVIF source/output formats, a `fill` or no-enlargement `inside` fit and
target width/height of 1–2048 pixels. The operation JSON names JPEG `jpeg`;
the source and destination filename extension is `.jpg`. Preparation requires an exact saved source
revision and an absent destination in a selected project. Raster inputs and
outputs are each bounded to 16 MiB; token/SVG inputs retain their 1 MiB limit.
The adapter rejects corrupt or mislabeled images, animated/multipage images,
source dimensions above 8192 or 16,777,216 input pixels. It applies EXIF
orientation, bounds output dimensions, converts to sRGB and uses fixed
PNG/JPEG/WebP/AVIF encoder settings. JPEG flattens transparency on white.
Metadata is not retained in this first finite transform. The worker owns a
private scratch snapshot, one child process, a 30-second job deadline, output
hash validation and new-copy publication. It does not sandbox untrusted native
code.

Development qualification selects the exact source-built candidate module with
`mirket dev serve --design-adapter ... --design-font ... --raster-module ...`.
The module path must be an existing canonical absolute `.mjs` file; the Rust
owner passes that same path to the private Node adapter. The adapter advertises
`raster.transform` only when this explicit module is present. The normal
installed manifest does not supply it. Windows child ownership and the private
Sharp package are not release-qualified.

The direct adapter test used the candidate's `install-candidate` module, with
its sibling native dylib SHA-256
`1801c6f1cb12363a8d1815d224840001031ed96f6f986e9e91e03dce35ee453a`.
It transformed a two-pixel RGBA source into WebP, decoded the saved pixels and
alpha, and refused wrong source-format claims and corrupt pixels without a
stage. The disposable MCP test then performed `design_prepare`, `design_start`
and terminal receipt inspection through the Rust worker and private Node
24.22.1. It proved saved source/output revisions, 8×8 RGBA output, stale
revision refusal, JPEG output with white-flattened alpha, and a 3,151,914-byte PNG source above the former 1 MiB
design-input bound, downsampled to a 256×256 WebP copy. The resulting 104-byte
WebP had SHA-256
`216ab42f4809255844e3007b0c1a27103a2abec293bb6eb676a6c1263687bb50`.
The initial MCP fixture failed before initialization because its disposable
state directory was group-readable; the fixture was corrected to mode 0700 and
passed. That failed scratch directory was removed after the pass.

After integration, the build-input guard passed; `cargo test --locked --offline`
passed 495 library, three CLI and one integration tests (four explicit manual
ignores); a later fourth CLI argument test and all-target strict Clippy passed;
adapter TypeScript compiled; and
`MIRKET_RASTER_MODULE=<candidate> npm test --offline` passed 90 adapter tests
with one optional live-browser test skipped. The direct and MCP raster tests
are in `adapters/tests/design-main.test.mjs` and
`tests/raster-mcp-smoke.mjs`.

Before release, assemble an exact separate raster payload and reconcile its
shipped bytes with the source kit, notices and applicable LGPL delivery terms;
exercise recipient source access and modification/import/rollback under the
normal integrity path; then qualify actual Codex client use, visual output,
minimum macOS and native Windows x64/ARM64. The current development result
does not satisfy those gates.
