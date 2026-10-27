# Skill-to-tool pairing review

Reviewed at the current checkout on 2026-10-27 (macOS Darwin ARM64). The
catalog declares seven private capability IDs. `Installation::skill_catalog`
derives those IDs only from verified release components; `SkillSync` refuses a
required capability that is absent and treats conditional capabilities as
optional guidance branches. Project selections are pinned under
`.mirket/project.json`, and a sync whose pin, receipt, source digest, release
identity, or project root changes is refused.

| Catalog capability | Owner and route | Activation condition | Observed refusal/limit |
| --- | --- | --- | --- |
| `navigation.js-ts` | Mirket `code_inspect`/`code_navigate` | configured verified `design` component (the component carries the fixed TS 5.9.3 adapter) | no design payload: both calls report the private runtime is not configured; SQL and other languages remain Codex/project-owned |
| `design.tokens` | Mirket `design_prepare` + `design_start` | verified `design` component | source revision, project root, output extension, and adapter identity are checked |
| `design.openpencil` | Mirket OpenPencil branch of `design_prepare` | verified `openpencil` component | works independently of the standard design payload; absent payload refuses before a job is issued |
| `design.vector` | Mirket SVG optimize/render operations | verified `design` component | external assets/imports and stale revisions are refused |
| `design.raster` | Mirket Sharp raster operation | verified `design` and `raster` components together | a design-only release reports Sharp unavailable; no host/global Sharp fallback |
| `design.diagrams` | Mirket `diagram_render_prepare`/`diagram_render_start` | verified `diagram` component | a skills-only release reports D2 unavailable |
| `video` | Mirket finite video prepare/start/status/cancel/result | verified `video` component with exact Chrome/HyperFrames/FFmpeg payload | host approval can deny the effect; refusal leaves sources unchanged and does not imply an export |

The runtime status response now separates the stable implemented operation
surface (`capabilities`) from exact payload-backed `skill_capabilities`. A
skills-only server therefore does not claim navigation, design, diagram, or
video readiness even though those MCP method schemas remain in `tools/list` and
the operation names remain in `capabilities`. Clients receive a stable
protocol surface, while a call to an unavailable operation returns a concrete
private-payload refusal. A tool listing or an operation name is not treated as
proof that the selected runtime is ready.

Evidence:

* `python3 scripts/check-build-inputs.py` passed for the admitted macOS ARM64
  inputs.
* `cargo test --locked --offline server::tests` passed the payload matrix,
  including OpenPencil without standard design and design/raster/diagram/video
  combinations.
* After `cargo build --locked --offline`, the fresh binary SHA-256 was
  `beff0da90c361267f8bd9de489f29ef0474bf0e45eea34dae42ab8d53f60f41a`.
  `node tests/mcp-smoke.mjs --mirket /Users/nazmiyilmaz/Projects/harness/target/debug/mirket --node /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/codex-gui-core-20261026/release-source/runtime/node/bin/node`
  passed the real stdio inventory and exact empty-payload capability assertion,
  then exercised stale-release, unavailable D2/design/video payloads, and
  unknown-operation receipt refusal in disposable state. The explicit private
  Node SHA-256 was
  `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`. It
  observed Darwin/arm64. No user home or project was used.
  The same run also started a configured disposable design payload, reported
  the three exact design/navigation skill IDs, and refused a stale source
  revision before issuing a job or destination file.
* Existing `skill_sync` tests (25 cases) cover capability refusal, wrong
  project pins, managed collisions, source/receipt drift, interrupted
  transactions, and exact rollback. The model-facing host permission boundary
  remains outside Mirket: a Codex approval denial is recorded as a host result,
  not converted into a Mirket success or a fallback route.

The final integrated stdio rerun passed using binary SHA-256
`88c920470d567ebacae0a455667761f5918694b91e2b4ba4c814fcfca57541b5`
and private Node v24.22.1. See [the integrated record](deep-review-20261027.md)
for final disposable installed-operation results. Windows native execution,
Codex Desktop GUI activation and catalog-wide acceptance remain unqualified;
specific successful runtime fixtures do not establish those broader gates.
