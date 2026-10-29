# Mirket

Mirket is a **pi-only harness** for product development. Pi owns
models, authentication, sessions, native tools and MCP lifecycle. Mirket supplies
scoped project memory, selected JavaScript/TypeScript navigation, finite design
and HyperFrames video operations, and selected workflow guidance.

The refactored local candidate passed the scoped checks on **macOS 26.5.2
ARM64** with external Pi 1.4.0 / Undici 8.12.2. See the
[qualification record](docs/qualification.md) for exact package identity,
measured outcomes and limits.
See [architecture](docs/architecture.md) for the current product contract.

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

The `/mirket-research QUESTION` command uses provider-native search
through pi's existing `openai/gpt-6-astra` authentication. It enables one research
turn, retains search actions and citation spans, and displays clickable sources.
Missing provider access fails explicitly; it does not install a search service.
Provider-dependent research and vision require configured access.
Browser automation was tested only with disposable local fixtures; no
production browser dependency is admitted.

Mirket state belongs under `<pi-agent-dir>/mirket/`. Setup uses a fresh private
home and preserves existing installations and unrelated settings. It does not
import an earlier installation or copy skills into host directories.

Skills must justify retention with a useful workflow, safeguard or measured
benefit. Catalog cases are planned fixtures, not completed acceptance trials.
Source methods, code, fonts and runtime assets have separate provenance and
admission records: see [skill provenance](skills/PROVENANCE.md) and
[dependency admission](packaging/admission/README.md).

For development, run `python3 scripts/check-build-inputs.py` before Rust
compilation, then the locked offline Rust tests/clippy, formatting, locked
TypeScript/process tests and explicit-binary MCP smoke. Use disposable private
homes and projects. See [native build requirements](docs/native-build.md) and
[implementation contract](docs/architecture.md).
