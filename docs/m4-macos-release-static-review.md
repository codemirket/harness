# macOS installed-release review — 2026-10-26

The focused review found and repaired two update/activation defects in
`src/release_store.rs`. Both were reproduced through the existing built CLI in
private disposable four-file payload fixtures before source changes. The
source fixes pass all 22 release-store unit tests. This is additional evidence
for the local update owner; it does not qualify a complete release or actual
Codex Desktop activation.

## Scope and frozen reproduction

The review followed source admission, installed manifest verification, stable
launcher activation, installed recipient selection and deferred MCP startup in
`distribution.rs`, `release_store.rs`, `installed_payload.rs`,
`installation.rs` and `deferred_server.rs`, with maintenance/session consumers
and their tests. It used the approved consolidated implementation plan. The
already documented Chromium, Raster, video, Windows, recipient and native
client gates remain separate.

The reproduction ran on macOS 26.5.2 ARM64 using the existing
`target/debug/mirket`, SHA-256
`d338f769e0ce95d45741dfa20dcb57d6512a9af232d11a3a003962ea668cf1de`.
Its tiny payloads contain inert core/Node bytes; the development CLI verifies
and copies them but never executes them. The test selected only private homes
under a new temporary directory. No real user installation, native client
configuration, upstream installer, publication, commit or push was changed.

Exact pre-fix CLI arguments, results and fixture bytes are retained at
`~/.codex/task-state/harness-rewrite-implementation/macos-release-static-review-20261026/pre-fix-reproduction/`.
The saved `evidence.json` records the original temporary paths and binary hash.

## P1 — A pending committed activation could delete the next candidate

After A → B, an interrupted recovery can leave a committed B activation
journal after removing its staged record. The prior `stage_verified` admitted
a new C without inspecting that journal. `activate_with_launcher_locked` then
recovered B and returned early. Recovery's `prune_retired` deleted C because C
was outside B's active/predecessor pair, while the C staged pointer remained.

The disposable reproduction persisted that exact committed journal state and
ran `dev update-core` for C. The command returned success with `staged=C` and
`activation=B`; the C release directory was absent afterward and `staged.json`
still pointed to C. This is an incorrect explicit update result and broken
retained state, not merely a missing qualification receipt.

The repair refuses staging another candidate while the original activation
journal exists. Recovery also refuses a different pre-existing staged intent
before launcher changes or retired-payload cleanup. Both intents and payloads
remain available for inspection. Explicit retry of the original candidate
continues through the existing recovery owner; the fix does not replay another
candidate or discard uncertain records.

The discriminating regression covers both admission and recovery: preserve
active/staged/journal/launcher/backup bytes, release inventory and candidate
manifest after refusal, then verify the original same-candidate retry finishes
and keeps A. Existing committed-journal and interrupted-launcher recovery tests
also pass.

## P2 — Repeating the active release discarded its predecessor

The launcher-aware activation path always set `previous_release_id` to the
current active ID. After A → B, repeating B therefore wrote
`active=B, previous=B` and pruned A. The separate development activation path
already preserved the earlier predecessor for this same-ID case.

The pre-fix CLI reproduction confirmed that a successful repeated B update
changed the previous pointer from A to B and left only B on disk. The repair
applies the existing same-ID rule to launcher-aware activation: retain the
recorded predecessor, otherwise record the formerly active release.

The regression performs A → B → B and verifies identical active-record bytes,
unchanged A manifest, both retained identities, correct B launcher bytes and no
leftover staged or activation record.

## Verification and limits

Before Rust compilation, `python3 scripts/check-build-inputs.py` passed for
`aarch64-apple-darwin`. After the source changes,
`cargo test --locked --offline --lib release_store::tests:: -- --nocapture`
passed: **22 passed, 0 failed, 532 filtered**. The compile took 16.17 seconds
and the selected tests 5.39 seconds on this run; these are observations, not
performance benchmarks. Only the owned Rust file was formatted with the
locked toolchain's rustfmt.

No additional concrete high-severity defect was found in the inspected
admission/serve control flow. Installed opening still verifies the finite
inventory, exact file bytes/modes and active release identity. A connection
retains maintenance exclusion while deferred verification runs; every tool
call waits for successful verification and running-core comparison before
opening the engine. Catalog discovery alone does not establish health.

This review assumes the documented private immutable-release filesystem model;
the path-based checks do not establish containment against hostile concurrent
replacement. It did not exercise actual power loss, the macOS 15 floor,
Windows, full capability payload assembly, post-fix installed binary/GUI
activation, general native-owner crash recovery or performance/resource gates.
The initiating coordinator owns the subsequent full integration check.

## Coordinator integration — 2026-10-26

The final build-input guard passed on macOS ARM64. The full locked offline suite
passed: **548 library tests, 11 CLI tests and one active integration test**;
seven cases remain deliberately ignored. Strict all-target Clippy with warnings
denied and `cargo fmt --all -- --check` also passed. The rebuilt development CLI
has SHA-256
`b14034bc5693e3a8039179cc444b13a87090af593412a3a029ecb0790a216324`.

A fresh disposable installed CLI fixture then exercised A → B → B with that
binary. The active pointer retained A as B's predecessor, A's manifest remained
unchanged, and the staged/journal records were absent. In a separate fresh home,
a committed B activation journal caused a requested C update to refuse before
any active, staged, journal, launcher, backup or release-inventory change. An
explicit B recovery cleared the journal; a subsequent C update activated C
with B retained. Both final core doctors returned `healthy` with verified
manifests and launchers. The fixture uses inert four-file development payloads,
so it verifies the installed release-store/CLI transition rather than execution
of a real Node adapter or complete product package.

The saved runner and exact result are under
`~/.codex/task-state/harness-rewrite-implementation/release-store-review-20261026/`.
`post-fix-installed-1/result.json` has SHA-256
`5593035012f12c9885fb2c96a4a8d5c758e3a2c92d7b6ff901c98fcccf0577d9`.
The repaired `src/release_store.rs` has SHA-256
`9f4bbcdc7eee30c0ff523bb39946fbf5fd6fd7b9eb3e9630a746e78f68543d99`.
