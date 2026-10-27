# Six-area review and correction record — 2026-10-27

Review base: `678c3c608a0d20cc643693fb9a4d51c0dcdd1acb`, current `harness`
checkout. The approved product boundary remains unchanged: Codex owns agents,
models, permissions, browsing, documents, shell, Git and SQL; Mirket owns its
scoped memory, selected JS/TS navigation, design, video and native skill
distribution. All integration state is disposable. The live installation was
not updated, and no dependency, release publication or push was introduced.

The later user-approved design adoption is recorded separately in
[design-adoption-20261027.md](design-adoption-20261027.md). Its method-adapted
provenance and supporting resources supersede this pass's original no-adaptation
and 67-body-only payload descriptions; the results below remain historical.

## Finding-to-fix matrix

P2 denotes an incorrect or ambiguous result affecting a realistic consumer;
P3 denotes an evidence/guidance/documentation gap. A reviewed area with no
demonstrated runtime defect is not represented as a code fix.

| Area | Finding and consequence | Correction and source | Outcome evidence / limits |
| --- | --- | --- | --- |
| 1. Authoring and references — P3 | Original authoring did not fetch the reference bodies. Catalog associations therefore could not establish source fidelity. Asset rights guidance did not explicitly preserve required notices through optimization/handoff. | Reviewed 58 families at the selected locators; retained exact retrieval identities and focused technique/rights analyses. Added receipt-backed [fetch](../scripts/fetch-skill-reference-evidence.py) and deterministic [refresh tooling](../scripts/refresh-skill-catalog.py), including a regression for extension-bearing license discovery; updated [catalog](../skills/catalog.json)/[provenance](../skills/PROVENANCE.md), and notice-preservation guidance in [art-direction](../skills/art-direction/SKILL.md) and [vector-art](../skills/vector-art/SKILL.md). | [Reference audit](../planning/reviews/2026-10-27/skill-reference-audit.md), [engineering methods](../planning/reviews/2026-10-27/source-methods.md), [visual methods](../planning/reviews/2026-10-27/source-visual.md), [product/reference library](../planning/reviews/2026-10-27/source-product.md). Retrieval, license observations, independent synthesis and model acceptance remain separate. No upstream prose/code/assets were copied. The subsequent independently worded method adaptations are recorded in [the adoption ledger](../planning/reviews/2026-10-27/source-design-adoption.md). |
| 2. Skill/tool pairing — P3 | Catalog capability IDs had no direct runtime-status counterpart; callers had to infer readiness from differently named configured fields or the stable implemented-method list. | Added payload-backed `skill_capabilities` in [server.rs](../src/server.rs). Preserved stable `capabilities` and `tools/list`; configured does not mean successfully executed. Added configured/unconfigured and stale-source MCP probes in [mcp-smoke.mjs](../tests/mcp-smoke.mjs). | [Pairing review](deep-review-pairing.md), actual stdio inventory/refusal checks, payload-combination unit tests, and positive disposable runtime output checks. Host routes remain host-owned. |
| 3. Project memory — P2 | A complete-looking semantic index row with a corrupt source digest was counted as indexed even though recall rejected it. Refresh could overstate usable coverage. | [memory_index.rs](../src/memory_index.rs) now uses the same validated chunk reader as recall for coverage. Added corrupt-digest regression, real-model [cross-project/forgetting oracle](../tests/deep-review-semantic.mjs), and [coverage timing](../tests/memory_scale.rs). | [Memory review](deep-review-memory.md). Canonical correction/recovery/forgetting tests, held-out ranked recall, and two-client idle observations. Validating 100,000 synthetic heads took 1.585 s in a debug build; this adds work to each refresh page and is not a constant-time claim. SQLite/FTS and lazy MiniLM remain. |
| 4. Capability matrix — P3 | No single matrix joined workflow, skill, owning tool, observed output, failure probe and evidence; README/plan still described an old checkout/installation coexisting. | Added [67-skill/eight-profile matrix](deep-review-capability-matrix.md) and realistic mixed-project combinations. Corrected [README](../README.md), [current plan](../planning/implementation-plan.md) and current-state note in [decisions](../planning/decisions.md). | All 67 IDs were checked statically. No missing subject justified adding another global/project skill or duplicating a Codex capability. Profile unions are selection aids, not runtime qualification. |
| 5. Coordination — P2 guidance | A guided held-out disagreement scenario proposed another delegate despite adequate existing reviewer evidence. Scope, denial and shared-budget guidance needed clearer operational boundaries. | [Coordination skill](../skills/agent-coordination-strategy/SKILL.md) now resolves existing evidence before adding a reviewer, checks permitted destinations and shares the budget. [Managed guidance](../src/codex_guidance.rs) preserves requested/inherited settings, stops the denied action while allowing independent authorized work, and explicitly states its enforcement limits. | [Scenario decisions](deep-review-scenario-observations.md): initial guided partial retained; one post-repair turn resolved it with no observed regression in the other nine decisions. [Artifact trials](deep-review-coordination.md) chose no delegation. Actual native spawn/model/permission enforcement is not established. |
| 6. Feedback/output quality — P2 guidance/evidence | Prior creative retry wrote outside its requested project; prior model-driven video export was host-denied. A renderer exit, a generated test, or missing CLI image events could mislead acceptance. | Kept those failures visible; froze new matched inputs/rubrics/budgets, independently checked saved code and viewed saved pixels. Added optional retained output in [video smoke](../tests/video-mcp-smoke.mjs) for decoded inspection. | [Feedback evidence](deep-review-feedback.md): seven independent black-box migration cases passed both conditions and rejected a plausible wrong deduplicator. Both posters were independently viewed. Direct video edit changed 3,434 decoded heading pixels; caption region changed 11,993 pixels during the cue and zero afterward. This does not turn the denied model export into success or prove general skill benefit. |

## Evidence scope

Observed host: macOS 26.5.2 (25F84), ARM64. Rust 1.99.1; private Node
v24.22.1, SHA-256
`6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`.
Matched model trials used Desktop-bundled CLI 0.164.0-alpha.11.1, SHA-256
`a04752e4d46a4d89e53e6f61c59c5349a638c8ce172ffbf391fd482b4d902a4e`.
Backend model revision and monetary cost were opaque. The client configuration
hash stayed unchanged across the recorded turns.

Raw logs, frozen cases, exact skill snapshots, model traces, independent
oracles and decoded/viewed outputs are retained under
`/Users/nazmiyilmaz/.codex/task-state/harness-deep-review-20261027/`.
The source reviews also identify private raw reference receipts. Upstream raw
bodies are not included in the release skill payload, which remains 67 bodies
plus catalog and provenance. Two intermediate full Rust attempts ran while
embedded catalog/provenance inputs were changing and failed; their logs remain
retained. Only the final frozen-source gate results establish integration.

## Final integrated verification

The frozen source passed `python3 scripts/check-build-inputs.py` before Rust
compilation, `cargo test --locked --offline` (502 passed, five manual tests
ignored), all-target locked/offline Clippy with warnings denied, and formatting.
The locked TypeScript 5.9.3 adapter build passed; its process suite passed 59
of 60 tests, with the opt-in real-raster case skipped. The Python suite passed
24 tests after the final collector license-discovery and deterministic-order corrections. The catalog
refresh dry-run reported no drift, and `git diff --check` passed.

Actual stdio MCP and canonical-memory smoke tests passed using the explicit
private Node and rebuilt binary. The real MiniLM held-out paraphrase ranked the
expected fact first among six same-project distractors, preserved cross-project
isolation and excluded the forgotten fact while retaining the other project's
fact. One end-to-end cold first save including index startup took 6,526 ms;
warm recall took 221 ms (one sample each, debug build). The final binary SHA-256
is `88c920470d567ebacae0a455667761f5918694b91e2b4ba4c814fcfca57541b5`.
Exact commands, durations, platform and logs are retained in
`final-gates/results.json`, `final-gates/supplemental-results.json` and
`final-gates/handoff-results.json` under the
private evidence root above. The five ignored tests are opt-in raster fixture,
real embedding package, final-launcher-unlink recovery, synthetic retrieval,
and scale probes; separate actual model and 10k/100k scale evidence is recorded
here without pretending the default invocation ran those ignored cases.

A rebuilt seven-component release was assembled and installed under the
private `installed-user` directory. Setup and doctor passed with release ID
`sha256:ec2eb6d9c5fa9c1f32b2276216358fcd2d9dee7ebf66a30b1f3f417787598428`.
Its actual MCP inventory exposed 25 tools and all seven configured catalog
capability IDs; all 67 packaged bodies and nine installed global bodies matched
the final source. These identity checks are separate from operation evidence.
Its installed MCP journeys passed selected-source JS/TS navigation, token CSS
and SVG rendering, PNG/JPEG/WebP/AVIF raster conversions, and OpenPencil
create/patch/export with changed saved semantics and source preservation.
The separate disposable D2 journey passed Dagre/ELK/TALA output, revision and
refusal checks. These are operation-output assertions, not additional visual
quality claims. Exact logs are in `installed-gates/`; the private runner's
initial doctor JSON lookup used the wrong nesting and was corrected before
continuing the operation tests. Setup itself had succeeded and was not repeated.

The matched artifact comparison comprised four model turns: baseline/guided
migration and baseline/guided poster production. Both code artifacts passed
seven independently chosen black-box cases; both saved posters were viewed and
had legible, uncropped required copy. Ten held-out decision scenarios ran in a
matched baseline/guided comparison; one guided disagreement partial prompted
one targeted repair turn, whose ten decisions passed independent review. The
initial invalid harness prompt, guided partial, exact skill snapshots and
post-repair trace remain retained. These are bounded observations, not a
statistical improvement claim or evidence of native-spawn enforcement.

## Remaining limits

- All 67 catalog acceptance records remain `planned-not-run`. The authored
  five-case sets are development examples; these representative held-out
  comparisons do not complete catalog-wide acceptance or establish a causal
  quality benefit.
- There are 45 immutable Git-pinned families and 13 moving/unpinned URL
  families. The retrieval collector has license receipts for 44 families and
  no standalone license receipt for 14; focused reviews retain additional
  component/site-rights distinctions. Missing rights evidence grants no reuse
  permission. No removed source or new production dependency was admitted.
- Windows x64/ARM64 native ownership, installation and capability fidelity
  remain unqualified. The macOS 15 floor was not tested on this macOS 26 host.
  Codex Desktop GUI activation remains untested under the user's waiver.
- Decision-table compliance and this review's native reviewers do not establish
  hard enforcement of future native spawning, model/reasoning choices,
  interruption, filesystem writes or approvals. Mirket does not intercept
  Codex's agent controls and no second orchestrator was added.
- Direct video runtime checks are distinct from the host-denied model-driven
  export. Sampled decoded frames establish the stated fixture observations,
  not full playback/player coverage, human acceptance, or arbitrary-video
  quality. Process RSS samples are not physical-memory peaks.

The local commit IDs and changed-file inventory are recorded at handoff. This
review does not promote the catalog or an unqualified release path.

## Changed files and local commits

Runtime corrections and their discriminating tests are in local commit
`0dbc192` (`fix: report usable memory coverage and configured skill capabilities`).
The accompanying guidance, reference evidence, tooling and review record are
committed separately; the handoff identifies that commit. No commit was pushed.

The complete 34-file inventory relative to the review base is:

```text
README.md
docs/deep-review-20261027.md
docs/deep-review-capability-matrix.md
docs/deep-review-coordination.md
docs/deep-review-feedback.md
docs/deep-review-memory-scale-100k.json
docs/deep-review-memory.md
docs/deep-review-pairing.md
docs/deep-review-scenario-observations.md
docs/deep-review-semantic-failed-concurrency.json
planning/decisions.md
planning/implementation-plan.md
planning/reviews/2026-10-27/reference-audit.json
planning/reviews/2026-10-27/reference-fetches.json
planning/reviews/2026-10-27/skill-reference-audit.md
planning/reviews/2026-10-27/source-methods.md
planning/reviews/2026-10-27/source-product.md
planning/reviews/2026-10-27/source-visual.md
scripts/fetch-skill-reference-evidence.py
scripts/refresh-skill-catalog.py
skills/PROVENANCE.md
skills/agent-coordination-strategy/SKILL.md
skills/art-direction/SKILL.md
skills/catalog.json
skills/vector-art/SKILL.md
src/codex_guidance.rs
src/memory_index.rs
src/server.rs
tests/deep-review-semantic.mjs
tests/deep-review-trials.py
tests/mcp-smoke.mjs
tests/memory_scale.rs
tests/test_refresh_skill_catalog.py
tests/video-mcp-smoke.mjs
```
