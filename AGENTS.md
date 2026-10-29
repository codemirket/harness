# Mirket

Follow `docs/architecture.md` and `docs/skills.md`. Pi is the sole product host.
Preserve source provenance and truthful qualification evidence. Preserve the user installation and unrelated client settings. No
compatibility layers or publication without authorization.

Keep one Rust crate and one compiled TypeScript package under `pi/`. Pi owns models,
authentication, sessions, native tools and MCP lifecycle. Mirket owns explicit
scoped authorization for its effect-bearing operations. Mirket-owned processes are not a sandbox. Use explicit,
verified private executable paths where packaging is manageable. For a
burdensome native tool, setup may bind an explicitly selected host executable
after capability and identity checks. Installed jobs must recheck that binding
and must not search PATH or silently fall back to another executable.

Ask before new production dependencies unless their exact source/version/rights
batch is already approved. Current byte and source pins live in `packaging/runtime`;
inventory files do not grant permission. Never run upstream installers/hooks incidentally.
Before compiling Rust dependencies run `python3 scripts/check-build-inputs.py`.
This checks the admitted SDK build-script condition that could otherwise change
Git hook configuration. Use the locked Rust toolchain and dependency closure.

Keep file owners disjoint when delegating, integrate their actual results, and
retain test evidence with platform/runtime identity. Temporary coordination history
belongs outside shipped source. Preserve existing work; no commits or pushes
without authorization.

Use disposable private state directories and projects for integration tests.
Never use the user's existing `.mirket`, `.pi`, `.agents` or client settings.
Use disposable HOME, pi agent/session directories and projects outside the
real home ancestry. Do not import a previous installation.
Current checks include `cargo test --locked --offline`,
`cargo clippy --locked --offline --all-targets -- -D warnings`, the adapter's
locked TypeScript compilation/process tests, and `tests/mcp-smoke.mjs` with
explicit binary/private Node paths. Run the build guard first. Pi CLI, TUI, image delivery and cancellation require fresh evidence. Windows
ownership, packaging and capability fidelity have separate milestone gates;
passing local source or protocol tests does not establish them.

Skill retention is evidence-led: keep guidance only when a concrete workflow,
quality safeguard or measured performance benefit justifies its context and
maintenance cost. Remove redundant or non-contributing skills instead of
preserving catalog size. A tied small trial does not prove universal uselessness;
repeated failure can justify retiring the tested bundle without blaming each
member. Preserve failed evidence and host-owned workflows when reducing scope.
The local candidate qualification scope is macOS 26.5.2 ARM64. See
`docs/qualification.md` for its exact identity and limits. Other platforms
require separate native qualification.
