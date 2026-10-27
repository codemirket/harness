# Mirket

The consolidated plan in `planning/implementation-plan.md` was approved by the
user on 2026-10-25. Execute it and preserve the current source and qualification
evidence. Preserve the user installation and unrelated client settings. No
compatibility layers or publication without authorization.

Keep one Rust crate and one compiled TypeScript adapter package. Host clients own
agents and permissions. Mirket-owned processes are not a sandbox. Use explicit,
verified private executable paths where packaging is manageable. For a
burdensome native tool, setup may bind an explicitly selected host executable
after capability and identity checks. Installed jobs must recheck that binding
and must not search PATH or silently fall back to another executable.

Ask before new production dependencies unless their exact admission batch is
already approved. Current admissions live in `packaging/admission`; proposed
inventories do not grant permission. Never run upstream installers/hooks incidentally.
Before compiling Rust dependencies run `python3 scripts/check-build-inputs.py`.
This checks the admitted SDK build-script condition that could otherwise change
Git hook configuration. Use the locked Rust toolchain and dependency closure.

Keep file owners disjoint when delegating, integrate their actual results, and
retain test evidence with platform/runtime identity. Temporary coordination history
belongs outside shipped source. Preserve existing work; no commits or pushes
without authorization.

Use disposable private state directories and projects for integration tests.
Never point qualification commands at the user's existing `.mirket` home.
Current checks include `cargo test --locked --offline`,
`cargo clippy --locked --offline --all-targets -- -D warnings`, the adapter's
locked TypeScript compilation/process tests, and `tests/mcp-smoke.mjs` with
explicit binary/private Node paths. Run the build guard first. The user waived
the Codex Desktop GUI check for this release; record it as untested. Windows
ownership, packaging and capability fidelity have separate milestone gates;
passing local source or protocol tests does not establish them.
