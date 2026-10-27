# D2 diagram qualification

The macOS ARM64 release inventory admits D2 v0.9.0 as one private diagram
renderer. Its executable, MPL-2.0 license, third-party notices and SPDX receipt
are pinned in [`m4-d2.manifest.json`](../packaging/admission/m4-d2.manifest.json).
[`assemble-d2.py`](../scripts/assemble-d2.py) accepts the exact archive and
separately verified SPDX document, copies only four selected files and does not
run upstream scripts. The executable SHA-256 is
`e8ba501a413bac33a4d93014c475b804de10fb2863a391583e23a8d88160e311`.

An installed release exposes `diagram_render_prepare` and
`diagram_render_start`. Preparation binds one saved UTF-8 `.d2` source revision,
an explicit project root, a new `.svg` destination and Dagre, ELK or TALA. It
rejects sources above 1 MiB, imports, external URLs, image/icon/font references
and links. Start runs D2 in the existing finite worker with shared admission,
receipt, cancellation and private scratch ownership. D2 receives a cleared
environment, a private home/temp directory, no watch mode, and a 30-second render
deadline. Output is capped at 16 MiB, checked for a local SVG structure and
external/active references, then published as a new copy only. The host owns
permissions; source filtering and environment clearing do not constitute an OS
network sandbox or a complete D2 language parser.

On macOS ARM64, the disposable installed MCP journey in
[`diagram-installed-smoke.mjs`](../tests/diagram-installed-smoke.mjs) passed for
all three layouts using the exact admitted D2 binary, private Node 24.22.1,
and the rebuilt Rust core. Each layout produced a distinct SVG, a confirmed
saved SHA-256 revision, a successful receipt and complete scratch cleanup.
Stale source/release identities and an import attempt were refused. The test
used a disposable home and project; it did not alter Codex settings or the
existing user installation. The assembler's two synthetic refusal tests and
the Rust request/output validation tests also passed.

This is a functional local qualification of the bounded SVG route. The first
public release still needs Mirket signing/notarization, native Windows x64 and
ARM64 D2 packages and tests, broader visual fixtures, and cancellation/large
output probes for the actual renderer. A successful SVG hash or process exit
does not prove that a diagram communicates the intended architecture; inspect
the rendered image in its target context.
