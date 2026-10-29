# Local candidate qualification

## Packaging and onboarding candidate, 2026-10-29

This records fresh evidence for the packaging/onboarding change set on macOS
**26.5.2 (25F84), ARM64**. It does not extend to Windows or another
native platform. The test used disposable HOME, Pi agent/session directories and
projects under `/private/tmp/mirket-onboarding-qualification.MNBJjZ`; it did
not touch the user's installation. The tested local package was assembled from
the final runtime and notice inputs without Chrome or Raster in its default
payload. Pi 1.4.0 remained the external product host.

- Native release ID: `sha256:1e42a96ef7c776646969f1c3d0c2f57c9852efe62afb8b6b981d6c6cbe478708`.
- Package inventory SHA-256: `1b7066dec5a4f58e183607b6f56feb7a991e6786960bb062aa09b478d8efd40d`.
- Private Node 24.22.1 executable SHA-256: `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`.
- Selected and pinned Chrome for Testing 156.0.8078.73 executable SHA-256:
  `9c5ab4582272b583d209b854fd7c494c50f11026c5728f84b4ffb7001a9e5e45`.
- Publisher Chrome ZIP SHA-256:
  `a46be48ee2e33b3edc1a98a39668b9f0abb8f23f031bc4df195e2388e6de28d1`.
- Rust 1.99.1; private Node v24.22.1. The assembly fixture contains 13,347
  files and occupies about 852 MiB. Its inventory has no Chrome or Raster
  payload. The shipped license, third-party notices, Video packet and FFmpeg
  source packet were verified in the assembled artifact.

The build guard passed before Rust compilation. `cargo test --locked --offline`
passed (435 library tests, 3 binary tests, 1 memory fixture; 2 library and 1
scale/manual cases ignored), as did `cargo clippy --locked --offline
--all-targets -- -D warnings`, `cargo fmt --check`, `cargo build --locked
--offline`, the locked TypeScript build, 96 TypeScript process tests (1
skipped), 124 Python tests, and `tests/mcp-smoke.mjs` using the explicit private
Node and native paths. Source/package manifest verification and a fresh setup
of this final package passed; an earlier setup of the same native release also
passed. The installed Pi CLI/RPC/TUI checked
doctor, reload, fork/resume, abort/recovery, `/mcp` and Ctrl-D shutdown.
An offline image-capable Pi provider fixture received the exact PNG bytes from
an installed design output through Pi's `read` tool (SHA-256
`11ad05a70be560cb79243cff441867307294d3a13d99ead599c404a5036118a4`).
This checks image transport, not a live provider's visual interpretation.

Actual Pi TUI `/mirket setup` exercised **Skip**, **Use installed executable**
and **Install pinned version**. The installed choice ran a local JavaScript DOM
capability probe; ten independent fresh bindings passed. The pinned choice
downloaded the exact Google ZIP from the publisher, verified its archive and
every member/link, then passed the same capability probe. Pi doctor reported
Chrome `missing` after no selection and `ready` after the pinned selection.
Process tests covered cancellation, bad or corrupted download, changed bound
bytes, Retry and no PATH fallback. The changed-byte prepare/start refusal also
passed in Rust. The original Chrome probe intermittently failed during process
teardown; the final probe configuration passed ten fresh bindings. Both the
failed and passing receipts are retained in the disposable evidence directory.

The default package passed installed D2 Dagre/ELK/TALA and Pi navigation,
token, SVG and decoded resvg PNG journeys without Chrome onboarding, including
after Skip. A separate installed Pi journey with Chrome explicitly skipped
passed cross-language semantic recall, independent SQLite backup inspection,
correction, authorized forgetting and managed backup restore/reindex. With an
explicitly selected installed Chrome binding, an actual Pi video export produced
a decoded 24-frame MP4. An independent ffprobe/PCM oracle
checked audio timing and gain; changed source changed decoded pixels. Stale
revision, remote asset and in-flight cancellation checks passed with no
published failed output. This qualifies those observed paths, not all possible
media content or provider interpretation.

**Startup timing failed.** Three fresh installed Pi CLI first-tool samples of
the final package were **3.867, 3.682 and 3.793 seconds**. A quiet repeat after
setup I/O stopped measured **4.004, 3.687 and 3.740 seconds**. Both cohorts exceed
the recorded 1.5-second gate (and the earlier 2.5-second full-package gate).
The CLI/RPC/TUI functions passed, but this candidate does not meet that timing
target. Profiling found approximately 0.85 seconds in Pi initialization,
0.71 seconds verifying the external Pi host and 2.96 seconds in full native
payload verification in a run with competing setup I/O; these are diagnostic
samples, not a controlled
attribution. The new descriptor/selection/Chrome identity status check averaged
0.489 ms over 20 runs (1.900 ms cold). No startup gate was waived.

The delivered FFmpeg replay recipe was transcribed from the exact build records
but has **not** been freshly rebuilt; generated-source correspondence for some
bundled wrapper bytes remains conservatively attributed rather than
independently reconstructed. Raster/Sharp remains excluded pending its native
rights/source closure. Windows packaging, ownership and capability fidelity,
other platforms, live installation, larger corpora and unexercised routes need
separate qualification. Mirket-owned processes are not an OS sandbox.

The raw local receipts are outside the shipped source, principally at
`/private/tmp/mirket-onboarding-qualification.MNBJjZ`,
including its `memory-domain-report-1793289743087.json` and decoded video
operation files and `image-delivery-report.json`,
`/private/tmp/mirket-onboarding-final-network-evidence.json`,
`/private/tmp/mirket-stable-bind-evidence.json`,
`/private/tmp/mirket-chrome-process-trial.json`, and
`/private/tmp/mirket-packaging-evidence-oyazpoc1/checks.json`. These are
disposable local paths; the hashes and results above retain the identity and
limits of this trial if those paths are removed.

## Earlier Pi package candidate

The refactored Pi package was tested on **macOS 26.5.2 (build 25F84), ARM64**
on 2026-10-29. The tested working tree was HEAD
`9ddf25cef144b680aa741fef03f340315f88c81b` plus uncommitted changes;
the package identity below binds the tested artifact more precisely than HEAD.
Private receipts are retained outside the repository. The later repository
cleanup and subsequent onboarding/package changes are separate revisions; the
results below do not qualify them.

## Identity

- Package: `@codemirket/pi-harness` 0.0.1; one Rust crate and one compiled
  TypeScript package. External unbundled host: Pi 1.4.0 with Undici 8.12.2;
  exact historical host review is recoverable via [historical evidence](history.md).
- Native release: `sha256:6a80efe8b79381b2c26d7d10f4a02169e44820f860a10969573dd73c09accd11`.
- Source and installed Rust executable SHA-256:
  `c79475802ccc5a0c1afa6bf3798bacc2d3c4e4b44d7c045fb93c2985c8c8a746`.
- Assembled package inventory SHA-256:
  `c64393f4ebf97c13830abb3cc10924a6835369ab65b34e6cc7404673bff91066`.
- Private Node 24.22.1 SHA-256:
  `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`.

## Earlier results

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
