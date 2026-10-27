# Memory deep review (2026-10-27)

This review covers the canonical SQLite/FTS store, revision and scope
filtering, the rebuildable semantic index, the embedding session boundary, and
backup/forgetting lifecycle paths at the current checkout. All writes used
disposable temporary roots; the user installation was not opened.

## Reproducible finding and fix

`MemoryStore::index_coverage` counted a semantic revision as indexed from SQL
row shape alone. A restored or damaged row with a stale `source_sha256`, an
invalid vector, or invalid UTF-8 chunk coverage could therefore report
`indexed_heads=1` even though semantic recall correctly discarded it as a
cache miss. That made the MCP index-refresh receipt claim more coverage than
the retrieval path could use. Coverage now runs the same `ChunkReader` and
passage validation used by semantic recall, including source digest, complete
chunk coverage, finite vectors, and L2 normalization. A regression test
mutates a complete row's digest and checks `current_heads=1,
indexed_heads=0,missing_heads=1`.

## Journeys exercised

The canonical MCP fixture was run with the rebuilt checkout binary on macOS
ARM64 and Node v24.18.1, using private state and project roots. It passed
prepare/commit/replay and lost-response reconciliation, stale corrections and
supersession, expiry and currentness filters, lexical recall, shared versus
project and linked-worktree isolation, exact history/provenance, concurrent
client shutdown, forgetting of all revisions and derived vectors, stale-vector
rejection, explicit transfer/import restore, and cross-scope preservation. It also
exercised stale-release, absent-model lexical degradation, incomplete semantic
index rows, and occupied/drifted transfer destinations.

The command was:

```text
node tests/memory-smoke.mjs --mirket /Users/nazmiyilmaz/Projects/harness/target/debug/mirket --node /Users/nazmiyilmaz/.nvm/versions/node/v24.18.1/bin/node
```

The fixture printed all canonical, forgetting, restore, and isolation passes.
The installed two-client semantic smoke also passed with the explicit private
MiniLM/ONNX payload and private Node at
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/simplification-20261027/local-mac-candidate-20261027/full-source/runtime/node/bin/node`.
On macOS ARM64, summed descendant RSS was 2,207,744 KiB after both clients
used the model, 2,183,024 KiB at 30 seconds, and 60,640 KiB at 75 seconds;
both model leaves had exited while both MCP servers remained. These are
sampled process-tree sums, not physical-memory peaks. The same payload passed
the independent held-out oracle in
[`tests/deep-review-semantic.mjs`](../tests/deep-review-semantic.mjs): six
same-project distractors filled the top-five budget, a paraphrase ranked the
expected fact first, a semantically similar fact in another project did not
leak, and forgetting removed the expected record while the other project still
recalled its fact.

The synthetic public-API scale probe measured full validated coverage at
10,000 one-chunk heads in 0.149 s and at 100,000 heads in 1.585 s. The
100,000-head debug run seeded/indexed in 538.7 s, with warm semantic-vector p95
1,959.7 ms and lexical p95 125.3 ms. These are all-terms-match cache/SQLite
timing probes, not model-quality or physical-RSS claims.

Raw retained evidence is under
`/Users/nazmiyilmaz/.codex/task-state/harness-deep-review-20261027/memory/`:
`heldout-semantic-final.json` (the earlier `heldout-semantic.json` is retained
too), `semantic-installed-idle.json`, `scale-10k.json`,
`scale-100k.json`, `binary.sha256`, and `failed-concurrent-save.json`. The
10k and 100k commands used `cargo test --locked --offline --test memory_scale
measure_scoped_semantic_recall -- --ignored --nocapture`; the semantic runs
used the explicit private Node and package paths recorded beside those files.
That held-out run used the rebuilt binary whose SHA-256 is recorded in
`binary.sha256`. The final integrated rerun also passed canonical MCP and held-out
semantic journeys using the explicit private Node v24.22.1. Its binary hash,
commands and results are in `../final-gates/results.json` relative to that
evidence directory. The one-sample end-to-end cold first save, including index
startup, took 6,526 ms; warm recall took 221 ms. These debug-journey observations
are not isolated inference benchmarks or latency distributions.
The earlier canonical smoke command using `~/.nvm` Node is retained as an
earlier lexical-only run; the real-model evidence above uses the private
release Node.

## Verification limits

The source and unit tests cover canonical CAS, provenance, expiry, context
matching, stale revision refusal, backup authority, restore exclusion,
forgetting precedence over retained snapshots, and semantic publication CAS.
Managed backup/restore authority is covered by the dedicated Rust backup
tests; native model quality remains bounded by the frozen and held-out oracles
above.
Windows ownership and native execution remain unqualified by the approved
plan. Codex Desktop GUI activation is intentionally untested under the user
waiver. No receipt, hash, or process exit was treated as proof of semantic
answer usefulness; absent or invalid payloads remain explicitly degraded.
