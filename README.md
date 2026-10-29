# Mirket

Mirket is a **pi-only harness** for product development. Pi owns
models, authentication, sessions, native tools and MCP lifecycle. Mirket supplies
scoped project memory, selected JavaScript/TypeScript navigation, finite design
and HyperFrames video operations, and selected workflow guidance.

The qualification target is **macOS 26.5.2 ARM64** with external Pi 1.4.0 /
Undici 8.12.2. The current packaging/onboarding candidate passed its functional
local checks but missed the first-tool startup timing gate. Earlier package
results remain separate. See the [qualification record](docs/qualification.md)
for exact package identity, measured outcomes and limits.
See [architecture](docs/architecture.md) for the current product contract.

Memory retains canonical SQLite facts, FTS, corrections, supersession, expiry,
forgetting and recovery, with lazy local MiniLM retrieval. Navigation operates on
explicit revision-bound sources. Design supplies tokens, editable vectors and
D2 diagrams with SVG or bounded direct PNG output. Raster/Sharp is omitted
while native rights obligations remain open.
Video bundles HyperFrames and FFmpeg/ffprobe with source/notices; Chrome is an
explicit onboarding choice with verified installation or executable binding.
Successful exports require separate visual
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
Optional browser QA uses an external-only Playwright MCP peer and a separately
selected browser executable. Neither is in the default package. Pi TUI grants
one scoped local origin through `/mirket-browser open`; the selected video Chrome
binding is never reused silently. See [setup](docs/setup.md) for the choices and
[qualification](docs/qualification.md) for the exact tested scope.

Mirket state belongs under `<pi-agent-dir>/mirket/`. Setup uses a fresh private
home and preserves existing installations and unrelated settings. It does not
import an earlier installation or copy skills into host directories. Follow
[setup](docs/setup.md), then use `/mirket setup` and `/mirket doctor` in Pi.

Skills must justify retention with a useful workflow, safeguard or measured
benefit. Catalog cases are planned fixtures, not completed acceptance trials.
Source methods, code, fonts and runtime assets have separate provenance and
rights: see [skill provenance](skills/PROVENANCE.md), the short
[runtime table](packaging/README.md#runtime-choices) and delivered
[third-party scope](packaging/THIRD-PARTY.md). Historical review evidence remains
recoverable via [history](docs/history.md).
The [tool landscape](docs/tool-landscape-20261029.md) and
[bounded pilots](docs/tool-pilots-20261029.md) record candidate comparisons
without turning them into package admissions.

For development, run `python3 scripts/check-build-inputs.py` before Rust
compilation, then the locked offline Rust tests/clippy, formatting, locked
TypeScript/process tests and explicit-binary MCP smoke. Use disposable private
homes and projects. See [native build requirements](docs/native-build.md) and
[implementation contract](docs/architecture.md).
