# Local candidate qualification

The pi-only candidate at commit `9455496` was validated on **macOS 26.5.2 ARM64**.
Documentation/provenance cleanup does not retroactively alter that artifact.
Other platforms, live installation, publication and redistribution are not qualified
or authorized by the local candidate checks.

## Identity

- Package: `@codemirket/pi-harness` 0.0.1; one Rust crate and one compiled TypeScript package.
- External host: pi 1.4.0, unbundled CLI, Undici 8.12.2; exact admission in
  [pi-local-candidate.json](../packaging/admission/pi-local-candidate.json).
- Native release: `sha256:a88906e210d7874e10722fd6add4c86654e88dbb3021da273b1029e0279be5f2`.
- Core SHA-256: `45e361e25c39e7136c22d617d67d44f27ed5395a5da11a80d55d1bbe9b6e8df8`.
- Private Node: 24.22.1, SHA-256 `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`.
- Durable local package/evidence: `/Users/nazmiyilmaz/.codex/task-state/pi-migration-candidate-20261029/`.
  Full experiment evidence and the original qualification report remain outside
  shipped source and in the checkpoint commit.

## Verified behavior

Locked Rust tests and clippy, TypeScript build/process tests, Python packaging
checks, explicit-binary MCP smoke, fresh pi SDK/CLI/TUI/RPC setup, native resource
filters, reload, two clients, wrong-project/stale identity refusal and shutdown
passed. Same-length installed payload tampering blocks actual tools and readiness;
restoration and reload recover. Every external-host/native file is verified.

Actual installed journeys covered semantic recall, corrections/forgetting, incomplete
coverage, backup recovery/rebuild; UTF-8/CRLF navigation and stale source refusal;
tokens, editable design reopen/export, changed raster outputs; decoded video/audio,
caption timing, playback, changed-source/changed-pixels, remote-asset refusal and
cancellation cleanup. Saved pixels were inspected. Managed children exercised
exact Astra Medium bootstrap, shared budgets, disjoint output ownership, cancellation,
model/effort/scope failures and actual TUI effect authorization.

Performance gates are core/full startup **1.5/2.5 seconds**, aggregate warm idle
**450 MiB**, Rust warm idle **32/40 MiB**, at most **16 MiB growth** from 30 to 120
seconds, and shutdown within **3 seconds** with no owned children. Measured core
startup was 1.08–1.10 seconds; full startup 1.78–1.88 seconds. Core/full warm idle
peaks were 419.72/432.66 MiB aggregate and 18.77/32.94 MiB Rust. No idle growth;
shutdown took under 0.08 seconds. Semantic/render/navigation overlap also passed.
These are local observations, not percentile or cross-platform guarantees.

## Limits

The frozen 24-run vanilla/Mirket comparison and one two-pair expansion found no
general quality or speed advantage. Most scores tied; treatment often used more
tokens. Original motion failures, budget stops and secondary progress-label defects
remain in checkpoint/private evidence. The narrower source-only workflow exported
four unchanged artifacts after the native cleanup repair. This does not qualify
the original broad motion loadout or all 305 planned skill cases.

Provider-native search and vision passed using configured `openai/gpt-6-astra` at
Medium. Browser automation is admitted only for disposable local-fixture testing;
production remote-browser dependencies are not approved. Image generation, documents,
spreadsheets and SQL require an authorized provider or installed project toolchain.
Reduced-motion browser states passed; a separate reduced-motion video export was
not exercised. Larger memory corpora and ignored manual Rust cases remain untested.

The 318-file private recipient fixture retains exact notices/source materials and
explicitly lists 21 supplementary source archives unavailable offline. This is not
redistribution clearance. Mirket-owned processes are not an OS sandbox.

Use [native build instructions](native-build.md) and the installed integration
fixtures under `tests/`. Never target existing user installations for qualification.
