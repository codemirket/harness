# Local candidate qualification

The refactored Pi package was tested on **macOS 26.5.2 (build 25F84), ARM64**
on 2026-10-29. The tested working tree was HEAD
`9ddf25cef144b680aa741fef03f340315f88c81b` plus uncommitted changes;
the package identity below binds the tested artifact more precisely than HEAD.
Private receipts are retained outside the repository. The later repository
cleanup was not retested at the user's request.

## Identity

- Package: `@codemirket/pi-harness` 0.0.1; one Rust crate and one compiled
  TypeScript package. External unbundled host: Pi 1.4.0 with Undici 8.12.2;
  exact admission is in [pi-local-candidate.json](../packaging/admission/pi-local-candidate.json).
- Native release: `sha256:6a80efe8b79381b2c26d7d10f4a02169e44820f860a10969573dd73c09accd11`.
- Source and installed Rust executable SHA-256:
  `c79475802ccc5a0c1afa6bf3798bacc2d3c4e4b44d7c045fb93c2985c8c8a746`.
- Assembled package inventory SHA-256:
  `c64393f4ebf97c13830abb3cc10924a6835369ab65b34e6cc7404673bff91066`.
- Private Node 24.22.1 SHA-256:
  `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`.

## Current results

The build guard, locked offline Rust tests, clippy, formatting and release build,
explicit-path MCP smoke, TypeScript process suite (**84 pass, 1 skip**) and
Python suite (**123 pass**) passed. Exact assembled-package Pi integration
passed **15/15 checkpoints**. Four sequential installed journeys checked
domain operations, semantic memory, editable design and video with file,
SQLite, decoded media and cancellation oracles. Package/source inventory and
native binding matched the installed fixture.

Three fresh full-package Pi CLI first-tool samples were **2.002, 2.066 and
2.059 seconds**, under the 2.5-second full startup gate. At 30, 60 and 120
seconds after doctor, sampled Pi-plus-Rust RSS was **291.42, 174.55 and
174.55 MiB**; Rust RSS was **31.13 MiB** throughout. These pass the 450 MiB
aggregate, 40 MiB Rust and at most 16 MiB growth gates for warm idle.
The startup 0-second aggregate sample was 467.91 MiB; it is outside the
30-second warm-idle gate. Pi EOF left no observed owned children after
**0.092 seconds**, under the 3-second shutdown gate. These are local samples,
not percentile or cross-platform guarantees. The separate 1.5-second core
startup gate was not measured in this full-package resource cohort.

Installed Pi consumer checks passed CLI/RPC doctor, reload, abort/recovery and
PTY shutdown; visible TUI denial/acceptance of a memory-forget operation; and
byte-exact PNG delivery to an offline image-capable provider fixture. The TUI
result qualifies that tested effect, and image transport does not establish
provider interpretation. These checks used disposable HOME, Pi agent/session
directories and project paths outside the real home ancestry.

## Limits

The offline image check covered transport, not provider-native image
interpretation. Individual effect routes not exercised in the current journeys,
larger memory corpora, ignored manual Rust cases and a separate reduced-motion
video export remain unqualified. All 305 catalog skill cases are planned, not
completed trials. The synthetic memory retrieval result has its own
[evaluation contract](memory-evaluation-contract.md).

The recipient source manifest is unrun. Windows ownership, packaging and native
capability fidelity need separate qualification; other platforms and live user
installation were not tested here. Mirket-owned processes are not an OS sandbox.

See [native build instructions](native-build.md) for reproducible checks. Do
not use the user's existing installation or settings as a qualification fixture.
