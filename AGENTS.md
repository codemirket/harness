# Mirket repository

Read `docs/architecture.md`, `docs/skills.md` and `docs/qualification.md`.
Keep one Rust crate and one compiled TypeScript package under `pi/`. Pi is the
sole agent host and owns MCP lifecycle; Mirket owns scoped authorization for
its effects. Mirket processes are not a sandbox. Preserve provenance, failed
evidence, existing user settings and unrelated work; no new dependencies,
publication, commits or pushes without applicable authorization.

Choose the smallest complete change across callers and tests. Use explicit,
verified private executables (or a setup-verified installed binding); never
search PATH, silently fall back or run upstream installers/hooks. Before any
Rust compilation run `python3 scripts/check-build-inputs.py` to check native
inputs and the admitted dependency build scripts. Use the locked toolchain and
offline closure. Run `cargo fmt --check`, `cargo test --locked --offline`,
`cargo clippy --locked --offline --all-targets -- -D warnings`, and the locked
Pi adapter build/tests and MCP smoke with explicit private Node/binary paths;
see `docs/native-build.md` for commands and further gates.

Integration tests use disposable private HOME, Pi agent/session directories and
projects outside the real home ancestry, never the user's `.mirket`, `.pi`,
`.agents` or client settings. Give delegates disjoint ownership and integrate
their actual results. Keep coordination outside shipped source. Report tested
platform/runtime identities and limits: 0.0.1 was qualified on macOS 26.5.2
ARM64; the macOS 26.0+ ARM64 install floor does not qualify macOS 27, Windows or
other platforms. Retain skills only for concrete workflows or measured benefit.

<!-- mirket-project:start -->
<!-- mirket-project-data {"schema":1,"skills":["agent-coordination-strategy","ci-release","code-change","critical-review","task-planning"],"tools":["mcp__mirket__code_inspect"]} -->
## Mirket project setup

Pi owns models, authentication, sessions, native tools and MCP lifecycle. Mirket owns explicit scoped authorization for its operations.
The exact IDs below record observed use, not current readiness or qualified output. Recheck the running release and required capability with mirket_doctor after setup, reconnect or a relevant change.
Use project toolchains for application builds, tests and format-specific exports. Skill guidance does not authorize dependency installation, account setup or restricted effects.

Observed skill IDs: `agent-coordination-strategy`, `ci-release`, `code-change`, `critical-review`, `task-planning`.
Observed tool IDs: `mcp__mirket__code_inspect`.
<!-- mirket-project:end -->
