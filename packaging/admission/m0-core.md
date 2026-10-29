# Core dependency admission

The active dependency selections are [Cargo.toml](../../Cargo.toml),
[Cargo.lock](../../Cargo.lock), [TypeScript package.json](../../pi/package.json)
and [TypeScript lock](../../pi/package-lock.json). Do not copy an earlier
proposed manifest or treat an old inventory as the complete current closure.

The original approved core batch is documented at
[its immutable source revision](https://github.com/codemirket/harness/blob/b3068a181a674f2aff751de4ff254f8120e7f96e/packaging/admission/m0-core.md).
Its retained [Rust inventory](m0-core.inventory.json),
[development Node inventory](m0-core.node-dev-inventory.json) and
[advisory snapshot](m0-core.advisories.json) record that batch's exact inputs and
review date; they are not a fresh vulnerability assessment or an authorization
for new packages. Later SQLite, semantic and design/video admissions extend it.

Run the [build-input guard](../../scripts/check-build-inputs.py) before Cargo.
In particular, the admitted rmcp source layout must not trigger Git-hook
configuration. Use locked/offline Rust inputs, verify adapter package identities,
and never invoke upstream installers/hooks incidentally. Current output and
platform qualification is in the [release record](../../docs/first-release-qualification-20261028.md).
