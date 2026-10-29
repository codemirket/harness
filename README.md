# Mirket

Mirket is a **pi-only harness** for product development. Pi owns
models, authentication, sessions, native tools and MCP lifecycle. Mirket supplies
scoped project memory, selected JavaScript/TypeScript navigation, finite design
and HyperFrames video operations, and selected workflow guidance.

The scoped local candidate is validated on **macOS 26.5.2 ARM64** with the exact
external pi 1.4.0 / Undici 8.12.2 host. It is not published or installed into the
user's live environment. See the [qualification record](docs/pi-qualification-20261029.md)
for package identity, reproducible checks, measured limits and unsupported paths.
The [migration plan](planning/pi-migration.md) retains decisions and failed evidence.

Memory retains canonical SQLite facts, FTS, corrections, supersession, expiry,
forgetting and recovery, with lazy local MiniLM retrieval. Navigation operates on
explicit revision-bound sources. Design supports tokens, editable vectors,
raster operations and diagrams. Video uses HyperFrames with verified private
rendering Chrome and FFmpeg/ffprobe. Successful exports require separate visual
and decoded-output review.

The package uses one Rust crate and one compiled TypeScript package. Pi's native
MCP client owns the verified Rust service. Native tools use explicit verified
paths; loading an extension must not download dependencies or launch heavy
workers. Mirket-owned processes are not an OS sandbox.

The candidate's `/mirket-research QUESTION` command uses provider-native search
through pi's existing `openai/gpt-6-astra` authentication. It enables one research
turn, retains search actions and citation spans, and displays clickable sources.
Missing provider access fails explicitly; it does not install a search service.
The installed research/vision preflight passed at Medium effort. Browser automation is qualified only for disposable local-fixture testing;
no production browser dependency is admitted. Broader providers remain unsupported.
The [bounded comparison](docs/pi-heldout-comparison-20261029.md) found no general
quality or speed advantage, and preserves its failed/budget-limited runs.

Fresh pi state belongs under `<pi-agent-dir>/mirket/`. Existing installations and
unrelated settings are preserved; no old configuration or skill-copy migration
is supported. The local package and disposable setup procedure are recorded in the qualification
record. Current source changes do not update a live installation.

Skills must justify retention with a concrete useful workflow, safeguard or
measured benefit. Existing catalog cases are planned fixtures, not completed
acceptance trials. Source methods, code, fonts and runtime assets retain separate
licensing obligations. See [provenance](skills/PROVENANCE.md) and exact records in
`packaging/admission`; a proposed inventory is not dependency admission.

For development, run `python3 scripts/check-build-inputs.py` before Rust
compilation, then the locked offline Rust tests/clippy, formatting, locked
TypeScript/process tests and explicit-binary MCP smoke. Use disposable private
homes and projects. See [native build requirements](docs/native-build.md) and
[implementation contract](planning/implementation-plan.md).
