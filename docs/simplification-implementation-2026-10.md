# Harness simplification implementation

This records implementation decisions from the October 22 critique. It does not
claim better model quality, lower total cost, smaller images or client activation.

| Boundary | Implementation and source owner |
| --- | --- |
| Packages | Additive install, explicit set, preserved settings; `src/cli.rs`. |
| Readiness | Separate installation/readiness aggregation with unknowns explicit; `src/packages/mod.rs`. |
| Models | Marketing morphology removed; verified capability envelopes, quality unmeasured; `src/router/policy.rs`. |
| Search | Bounded literal/regex source discovery, project-local Git ignore rules, hashes/lines and explicit coverage; `src/router/search.rs`. |
| Budgets/context | Per-run limits, numeric provider usage, hash/index observation recall and retained audit trace; `src/router/execution.rs`. |
| Evaluations | Token/intervention metrics and matched three-arm/five-case protocol; `src/role_eval.rs`, `evals/role-workflow/simplification-suite.json`. |
| No-change | Explicit source assessment permits no write but still requires full reads and host checks; `tests/model_routing.rs`. |
| Pure operations | Native trim/sort/deduplicate, unsupported rename rejected, other conversions remain inference; `src/router/policy.rs`. |
| Expertise | Advisory explicit guidance, no alternative-role matrix, tool scope unchanged; `src/router/assignment.rs`, `registry/capabilities.json`. |
| Design review | Applicable axes, required interactive/motion checks, optional host review; `src/router/design_gate.rs`. |
| Persistence | Independent persistence/recall choices and shared pause/format contract; `src/router/durable.rs`, `runtime/agent/recovery-contract.json`. |
| Updates | Resumable activation journal using existing setup/service owners, clients pending; `src/update/activation.rs`. |
| Dependencies | Expanded offline retrieval experiment; no speculative production removals; `tests/runtime_experiments.rs`. |
| Compatibility | Legacy fields hidden from schemas; router tests and presentation code excluded from execution identity; `src/source_identity.rs`. |
| Guidance | Shorter shared guide and honest discovery categories; `src/mcp.rs`, `src/catalog/workflows.rs`. |

## Conditional decisions and limits

- Search uses the approved `regex` and `ignore` dependencies. It applies bounded
  project-local `.gitignore` files with nested overrides, records their hashes,
  and excludes common generated directories. Global excludes and `.git/info/exclude`
  remain outside the assigned source scope. Results do not assert completeness.
- Matched model campaigns require fixed models/settings, actual artifacts and a
  separate blinded reviewer. Templates and mocked protocol tests are not quality
  results; no paid campaign is implied by suite validation.
- The SQLite journal prototype checks limited storage/CAS/retry behavior. It lacks
  effect reconciliation, migration, host-handoff and retention/export parity.
  LangGraph remains the production checkpoint owner.
- The no-NLP arm instruments a disposable index. It does not prove supported
  import/lock removal, cold-start savings or migration. Keep the dependency closure
  until those gates and representative retrieval/abstention quality pass.
- Exact selected guidance, tools, providers and artifacts remain recovery bindings.
  Other shared production modules are conservatively included in compatibility.
- Host review records evidence, not authenticated human identity or semantic proof.
  No-change source reads still require named host project checks.
- Native in-memory operations have no resumable/host-review session. Explicit
  durable, required-memory or host-review combinations are rejected; use a source
  workflow when those requirements apply.

Final verification is recorded in the task evidence directory. No commit or live
installation update is included in this implementation request.

The fresh offline retrieval comparison used 26 synthetic facts and 20 queries.
All four arms achieved answerable recall@3 of 1.0, but each returned nothing for
only one of four unanswerable queries. This does not establish reliable abstention
or safe NLP removal. Shared caches and instrumentation prevent interpreting the
elapsed-time difference as production latency or image-size savings.

Checkpoint IPC now uses immutable observation references. A regression verifies
that a 128 KiB observation produces a checkpoint payload below 512 bytes and that
archive tampering prevents restoration. This measures serialization volume only,
not end-to-end latency. Full audit traces remain available.

Verified on macOS ARM64 with the existing Linux ARM64 runtime: formatting,
Clippy, 444 standard Rust tests (zero failures; nine opt-in tests excluded),
72 isolated design tests, and the explicit contained acceptance gate covering
runtime, desktop contracts, parallel execution, design output and backup recovery.
Disposable all-target setup, doctor, catalog and generated-map checks passed.
The regex/ignore fixture matched ripgrep's expected paths and lines. These checks
establish regression and installation behavior, not comparative model quality.
