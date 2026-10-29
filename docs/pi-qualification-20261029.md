# Pi local candidate qualification — 2026-10-29

Status: pi-only implementation and the scoped local technical candidate are validated.
Scope: private, disposable macOS 26.5.2 ARM64 candidate. This record does not authorize live installation,
publication, redistribution, or production browser dependencies.

## Architecture and changes

| Finding | Implemented boundary | Owning source |
| --- | --- | --- |
| Mirket must have one agent host | Removed Codex configuration, guidance, installation aliases and copying workflows; pi owns agents, authentication and permissions | `src/main.rs`, `src/installation.rs`, `pi/` |
| Native MCP already works in pi | Pi registers and owns the Rust MCP process; deferred/codemode/direct exposure uses the native host | `pi/src/extension/index.ts`, `src/deferred_server.rs` |
| Package and executable identity must agree | Exact external-host closure and entry checks; bound private Node/core; every installed native file verified before native effects/readiness | `pi/src/extension/host.ts`, `runtime.ts`, `src/pi_host.rs`, `src/distribution.rs` |
| Repeating native verification increased startup | Removed the extension's redundant standalone doctor pass; MCP verification remains mandatory | `pi/src/extension/runtime.ts`, `src/deferred_server.rs` |
| Darwin stdout EOF can precede child-exit notification | Bounded owned-child reap, then read-only group-absence probe; never signal a reaped/recycled identity | `src/process.rs` |
| Desktop tools are not inherited | Provider-native search, actual model vision, explicit optional browser/toolchain prerequisites | `pi/src/extension/research.ts`, `skills/`, `planning/pi-skill-routes.md` |
| Managed delegation must stay finite | Exact model/effort bootstrap, disjoint outputs, persistent shared budget, cancellation and explicit authorization | `pi/src/extension/workflow.ts`, `managed-child.ts`, `task-ledger.ts`, `authorization.ts` |

One Rust crate and one compiled TypeScript package remain. No compatibility import
or dual-host path was added. Deleted/replaced implementation is visible in the
uncommitted Git diff; no commit or push was authorized or performed. Historical
qualification records remain labeled historical and are not fresh pi evidence.

## Exact candidate

- Durable package: `/Users/nazmiyilmaz/.codex/task-state/pi-migration-candidate-20261029/package`.
  An APFS clone was rehashed against all 13,167 inventory entries after copying.
- Actually installed/tested source: `/private/tmp/mirket-pi-package-cleanup-vsju74ht/package-verified`.
- Disposable installed home: that root's `agent/mirket`.
- Native release: `sha256:a88906e210d7874e10722fd6add4c86654e88dbb3021da273b1029e0279be5f2`.
- Core SHA-256: `45e361e25c39e7136c22d617d67d44f27ed5395a5da11a80d55d1bbe9b6e8df8`.
- Node: 24.22.1; SHA-256 `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`.
- External host: pi 1.4.0 unbundled CLI with Undici 8.12.2; 122 package instances,
  15,249 unique files. Frozen inventory SHA-256
  `ea4fcb77d656728a46561fdad2071bf9f4eed91864fb5c1a8dfb3324a4e60662`.
- Admission: `packaging/admission/pi-local-candidate.json`. Earlier proposal
  inventories retain their original bytes; the later approval is separate.
- Model comparisons: `openai/gpt-6-astra`, Medium. Existing pi authentication was
  used only as authorized, including normal token-refresh writes to its auth store.

The separate fresh core-only candidate is `/private/tmp/mirket-pi-package-YCgz32`.
It uses the same compiled extension and repaired core with a smaller native payload.
The host is external, not redistributed inside either package.

## Actual verification

| Gate | Result and evidence |
| --- | --- |
| Locked Rust | 431 library tests, 3 binary tests and 1 semantic integration test passed; 4 manual/expensive tests ignored. Clippy passed. `/tmp/mirket-pi-cleanup-rust-full.log`, `-clippy.log` |
| TypeScript | Locked build passed; 78 tests passed, 1 skipped. `/tmp/mirket-pi-dedup-ts-tests.log` |
| Python packaging/evidence | 149 tests passed; `/tmp/mirket-pi-final-python-149.log` |
| Fresh core setup/lifecycle | Actual installed SDK/CLI/RPC/TUI, package filters/removal, two clients, reload, project scope, same-name override and stale identities tested; `/private/tmp/mirket-pi-package-YCgz32` |
| Full native tamper/recovery | Same-length changed file refuses actual doctor and native tool; restoring bytes and reload recover. `native-verification-report.json` in full root |
| Navigation/design | UTF-8/CRLF resolution, stale refusal, tokens/SVG/raster outputs, remote refusal, editable GUID preservation and reopen/export passed. `/tmp/mirket-pi-cleanup-domain.log`, `-editable.log` |
| Memory | Actual semantic recall, correction/forget, incomplete coverage, managed backup restore and rebuild passed. `/tmp/mirket-pi-cleanup-memory.log` |
| Video | 24 decoded frames, independent PCM timing/gain, changed-source/changed-pixels, stale refusal, remote refusal and cancellation cleanup passed. `/tmp/mirket-pi-cleanup-video.log` |
| Cleanup regression | Original EOF race failed iteration 19; corrected 256-iteration regression and live-group counterexample pass. Same four held-out sources now export unchanged |
| Combined load | Semantic/render/navigation overlap passed, two heavy slots, 10 overlap samples; 1,341,344 KiB maximum sampled aggregate RSS; EOF cleanup 156 ms. Full root `combined-load-TqUvnQ` |
| Protocol | Explicit final binary/private Node MCP smoke passed; `/tmp/mirket-pi-cleanup-mcp-smoke.log` |
| Real model/search/vision | Earlier installed proofs with the exact repaired external host and unchanged provider integrations passed; `model-vision-proof.json` and `model-research-proof.json` under `/private/tmp/mirket-pi-package-wjKGau` |
| Managed children and TUI authorization | Actual concurrent Astra Medium children, requested/effective effort, shared budget across reload, cancellation and negative scope/effort checks passed. TUI denial preserved a canonical record; acceptance removed it. Retained pi fixture evidence is referenced in the migration plan |

The final focused lead review found no further actionable defect in the changed
startup/cleanup paths. Its scope and limitations are saved in `focused-review.json`.
It is not an independent review or deployment authorization.

## Performance decisions

The user approved 1.5 seconds for core startup and a separate 2.5 seconds for the
full native package. Three quiet full-package samples were 1882.89, 1808.80 and
1776.06 ms. All host/native files remain verified; no metadata-only trust cache
was introduced. Earlier failing measurements are retained.

Idle limits: aggregate 450 MiB; Rust 32 MiB core-only or user-approved 40 MiB
full-package; growth from 30 to 120 seconds at most 16 MiB; shutdown within
3 seconds with no owned children. Full-package observation before approval was
317.11 MiB aggregate and 33.16 MiB Rust at 30/60/120 seconds, no growth, and
0.08-second shutdown. Final explicit-profile reruns passed. Core startup was 1104.36, 1086.43 and
1076.65 ms. Core warm idle peaked at 419.72 MiB aggregate / 18.77 MiB Rust;
full warm idle peaked at 432.66 MiB aggregate / 32.94 MiB Rust. Neither grew
from 30 to 120 seconds. Shutdown took 0.077 / 0.079 seconds with zero remaining
children. Startup RSS at zero seconds is retained separately from the frozen
warm-idle gate.

## Quality findings and boundaries

[The held-out report](pi-heldout-comparison-20261029.md) preserves all 24 original
author runs and the one two-pair expansion. Code oracles pass in both conditions;
most other scores tie and Mirket often costs more tokens. There is no demonstrated
general quality or speed advantage. Original motion failures and budget stops
remain failures. The narrower source-only export workflow produced four artifacts;
the native cleanup repair exported those same bytes successfully. This does not
qualify the original broad motion loadout or all 305 catalog cases.

Saved UI/vector pixels, editable design exports, changed video pixels and decoded
motion frames were inspected. The caption is visible at 0.5/1.5 seconds and hidden
in the next frame and at 1.9 seconds. An initial batched-image review misread was
corrected without changing source/runtime; `caption-review-correction.json`
preserves the correction. Secondary percentage-label defects in the held-out
progress videos remain. An isolated local Chrome player subsequently played the complete two-second
caption export to its ended event without media errors (640×360, 2.003 seconds
wall time, eight advancing time observations). Playback was muted; audio has
separate PCM evidence. Both progress sources also produce identical, meaningful
reduced-motion browser states at 0/1.9 seconds, with zero active animations;
saved pixels were inspected. No reduced-motion-specific video export is claimed.

Browser automation passed only disposable local-fixture testing under the exact
three-package test admission. No production browser batch is admitted. Provider
search works without that browser; interactive remote browsing, accounts and
external writes are not part of this candidate. Image generation and document/
spreadsheet/SQL operations require an explicitly available authorized provider or
project toolchain; a skill alone does not provide those capabilities.

The 61 retained skills have source/route/safeguard records, not 305 completed
acceptance cases or universal benefit claims. Other macOS versions, Windows and
Linux remain unqualified. Ignored Rust performance cases, larger corpora and
untested visual states remain visible. Mirket processes are not an OS sandbox.

The 13,167 package files (924,642,139 bytes) and 318 recipient files
(125,606,766 bytes) were independently rehashed against their inventories;
compiled extension modules also match the checkout build. The final native
manifest has a matching private recipient notice/source fixture
at `video-recipient` in the full root. It lists 21 supplementary upstream source
archives unavailable offline; no complete redistribution or legal clearance is
claimed. Publication and live installation are not included in completion.

## Reproduction

Use the exact external peer tree and explicit private Node documented in the
migration plan. Run `python3 scripts/check-build-inputs.py` before any Rust build,
then `cargo test --locked --offline`, `cargo clippy --locked --offline --all-targets
-- -D warnings`, and `cargo fmt --check`. Build TypeScript with
`MIRKET_PI_PEERS` set to that peer tree and run its process tests. No package
manager hooks, downloads or incidental installers are required.

`tests/pi-integration.mjs` takes absolute peers, core, Node and Node-license paths
and creates a fresh private root. `tests/pi-domain-session.mjs` takes peers,
existing disposable root and `package-verified`; its memory, editable and video
environment switches select installed journeys. `tests/pi-cli-lifecycle.py` takes
`--node`, `--peers`, `--state-root`; `tests/pi-idle.py` additionally selects
`--payload core|full`. Never target the user's existing Mirket home.
