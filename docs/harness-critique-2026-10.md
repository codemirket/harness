# Harness critique — 20 October 2026

**The harness has stronger controls for identity, isolation and record consistency than for capability usability, operational diagnosis and measured task quality.** The next investment should close those gaps before expanding the skill/role inventory. Several restrictions are deliberate and useful; the defects arise where discovery promises a route that execution rejects, an explicit requirement is ignored, or useful evidence disappears before it can improve the system.

This review uses the [harness map](harness-map.md), source revision `d8013f53969db336d813274d1fbfba53a89ad0e2`, Mirket 1.0.11, and the matching installed source fingerprint. It adds local discriminating probes and public dependency queries. It does not change production behavior, install dependencies, invoke paid models, launch containers, modify providers or publish anything.

## Priorities

P1 means address or explicitly disposition before the next release; P2 means a material defect/gap for the next reliability cycle; P3 means a measured design improvement after those repairs. These are remediation priorities, not CVSS scores. No exploited vulnerability, cross-project escape or data loss was demonstrated.

| ID | Priority | Finding | Evidence strength | Smallest complete fix |
| --- | --- | --- | --- | --- |
| F01 | P1 | Normal CI can package releases without running the owned JS suite or contained runtime acceptance | Source-confirmed gate gap | Add JS checks and require a candidate-bound contained acceptance receipt before promotion |
| F02 | P1 triage | Four locked packages match nine advisories; the compiler pin also misses a regression-fix release | Live advisory/version queries; reachability qualified below | Patch/upgrade or record bounded exceptions, then automate all-closure review |
| F03 | P2 | `memory_recall: required` is ignored when the runtime package is absent | Reproduced through installed CLI | Reject before inference when required recall cannot be provided |
| F04 | P2 | `editable-interface` is advertised as an owned workflow but cannot be dispatched | Reproduced route rejection | Mark the workflow host-only/unavailable to workers in the shared contract |
| F05 | P2 | Failed worker traces discard the specific stopping error | Reproduced malformed-response failure | Persist a bounded, redacted typed failure receipt |
| F06 | P2 | Usage receipts disappear; default model selection cannot be evaluated for efficiency | Reproduced lost usage and T3-first ranking | Preserve usage/timing and compare eligible routing policies on fixed tasks |
| F07 | P2 | Worker source tools cannot inspect large files or search a repository | Reproduced read failure; actual oversized repository inputs | Add bounded range/search tools with file-identity binding |
| F08 | P2 | Resume depends on every old candidate, including unused models | Source-confirmed rejection condition; contained journey not rerun | Bind the selected execution identity; revalidate unused alternatives only if needed |
| F09 | P2 | Automatic memory recall ignores task text after the first 256 characters | Exact-function probe | Add explicit bounded recall queries or a structured task-derived query |
| F10 | P2 | The multilingual retrieval baseline has no relevance regression gate | Exact-function probe passes with all answerable searches empty | Add frozen per-case checks and a separate representative evaluation |
| F11 | P2 | Quality evaluation has retained failures but no validated replacement evidence of benefit | Recorded study; current tooling inspected | End-to-end packet preflight, immutable failure collection, fresh blinded trials |
| F12 | P3 | Wall-clock budgets limit the practical value of durable recovery | Source-confirmed documented tradeoff | Separate active budget from bounded pause/retention policy |
| F13 | P2 investigation | The standard gate timed out during MCP startup, then passed that test in isolation | One full-run failure and one focused pass | Instrument startup phases and isolate the timing cause before changing limits |

## F01 — Release confidence stops short of the production runtimes

The standard owner, [`src/dev.rs`](../src/dev.rs), runs formatting, Clippy and locked all-target Rust tests. [Native CI](../.github/workflows/check.yml) invokes that gate and then builds distributions. The website workflow tests website tooling. Neither shown workflow runs `npm test` in `runtime/design/tools`, the optional browser journeys, or `mirket dev acceptance`.

The separate contained gate does exist and exercises real runtime behavior with prepared ARM64 packages. [CONTRIBUTING](../CONTRIBUTING.md) explicitly requires it before deployment. That is useful process guidance, but the checked-in packaging path has no machine-readable requirement tying a successful contained run to the exact binary, runtime recipes and locks being promoted. External branch protection and unpublished release procedures were not inspected, so this is a repository automation gap rather than a claim that every past release skipped verification.

**Impact:** changes to Python runtime behavior, JavaScript tool contracts, native modules or container identities can pass normal CI while their actual consumers fail. The [existing harness review](harness-review-2026-10.md) records real concurrency defects found only during contained desktop calls, illustrating why Rust mock/contract tests cannot substitute for those journeys.

**Fix:** add an explicit Node 22 job for the owned JS suite, with dependency provisioning confined to CI. Add a manual or dedicated ARM64 acceptance job that emits a receipt bound to candidate binary digest, compiled source identity, both runtime recipes and dependency locks. Make artifact promotion verify that receipt; do not force expensive containers into every fast unit test. Keep desktop reconnection checks separate.

**Acceptance:** an intentional JS behavior regression must fail the JS job; a changed runtime recipe must invalidate an older acceptance receipt; missing contained evidence must prevent promotion while still allowing ordinary development checks. Owner: `dev`, release workflow and package maintainers. Scope: medium.

## F02 — Dependency identity is strong; dependency maintenance is incomplete

A fresh OSV batch query covered **679 unique public package coordinates: 233 Cargo, 349 npm and 97 Python**. All queries returned; nine advisory IDs matched four packages. No Python coordinate matched an advisory in this response. This is not a statement that the Python runtime is vulnerability-free. Exact queries, versions, advisory bodies and 32 direct-dependency version comparisons are saved in [dependency-audit.json](harness-critique/dependency-audit.json).

| Locked dependency | Advisory / patched range | Reachability assessment in this harness |
| --- | --- | --- |
| `rustls 0.23.45` | RUSTSEC-2026-0346/0347/0348/0349; patched at `0.23.46` | TLS is used through reqwest. The observed build enables `ring`, `std`, `tls12`, with no Brotli feature. No custom session Debug logging or ECH configuration was found. The key-share issue depends on session/group/server behavior. No reachable exploit was demonstrated. |
| `zip 6.0.0` | RUSTSEC-2026-0367/0368/0369; patched range `>=9.0.0` | The only inspected ZIP consumer, `catalog::select_archive`, manually reads selected bounded regular-file bytes. It does not call the affected `ZipArchive::extract`/stream extraction APIs or preserve archive permissions. These scanner matches do **not** establish a Mirket extraction escape. |
| `ajv 8.17.1` | GHSA-2g4f-4pwh-qvx6; fixed in `8.18.0` | Owned `server.mjs` constructs Ajv without `$data`, the option required by the advisory. Full third-party use was not traced. |
| `fflate 0.7.3` | GHSA-px8p-9vwx-vf98; fixed within its branch at `0.7.5` | Pulled by Satori and `@shuding/opentype.js`. The advisory concerns `unzipSync`; a reachable call from the inspected owned tools was not established. Patch compatibility needs font/render tests. |

Primary advisories: [rustls secret Debug output](https://rustsec.org/advisories/RUSTSEC-2026-0346.html), [ECH handling](https://rustsec.org/advisories/RUSTSEC-2026-0347.html), [Brotli allocation](https://rustsec.org/advisories/RUSTSEC-2026-0348.html), [key-share prediction](https://rustsec.org/advisories/RUSTSEC-2026-0349.html), [ZIP permissions](https://rustsec.org/advisories/RUSTSEC-2026-0367.html), [ZIP link targets](https://rustsec.org/advisories/RUSTSEC-2026-0368.html), [ZIP chmod through links](https://rustsec.org/advisories/RUSTSEC-2026-0369.html), [Ajv](https://github.com/advisories/GHSA-2g4f-4pwh-qvx6), [fflate](https://github.com/advisories/GHSA-px8p-9vwx-vf98).

Separately, [`rust-toolchain.toml`](../rust-toolchain.toml) pins **1.99.0**. The official **1.99.1** announcement dated 15 October lists fixes for x86-64 FFI boolean return miscompilation, missing `wcslen` symbols on some targets, and macOS linker loading. Mirket distributes x86-64 binaries, but this review did not reproduce those compiler failures in Mirket. Updating the compiler is justified by the supported target surface, not a claim that a particular Mirket binary is miscompiled. [Rust release announcement](https://blog.rust-lang.org/2026/10/15/Rust-1.99.1/).

Other newer direct releases observed include rmcp 3.6.0, HyperFrames 0.8.181, Satori 0.63.1, spaCy 3.8.16, serde_json 1.0.154 and Tokio 1.54.0. Reqwest, rusqlite, sha2, toml_edit and zip have newer releases across compatibility boundaries. A newer release alone is not a defect or permission for a bulk upgrade. Playwright and the main Mem0/LangGraph/FastEmbed pins matched the queried latest versions.

**Fix:** first move rustls to a fixed compatible patch, Ajv to a tested fixed version, and Rust to 1.99.1. Resolve fflate through a tested compatible parent/override; avoid assuming a 0.8 override preserves 0.7 APIs. For zip, retain the safe extraction design, assess the major upgrade, and record the current non-reachability exception until validated. Refresh notices and package recipes alongside locks. These are proposals, not changes made by this review.

Add a scheduled/admission dependency review covering Cargo, npm, Python, base OS/browser packages and the forwarded Penpot closure. The current repository has no checked-in advisory job or Renovate/Dependabot configuration. Existing Python audit/dated OSV evidence is useful but covers only one closure and one date. Exceptions should record affected symbols/configuration, owner, expiry and regression evidence.

**Acceptance:** rescan exact locks; preserve archive-link/mode rejection tests; exercise TLS client paths and actual font/render output; run native packaging and contained acceptance. No blanket `audit ignore`, and no CVSS label copied into a Mirket exploit claim. Owner: dependency/package/release maintainers. Scope: small patch changes plus medium compatibility work.

## F03 — Required memory recall can become no recall

[`Driver::start`](../src/router/durable.rs) checks whether `agent-runtime` is enabled and otherwise creates an ephemeral driver. It does not inspect `request.memory_recall`. The request enum exposes `Required`, and [runtime documentation](agent-runtime.md) says required recall propagates failure.

**Reproducer:** the `required-memory-without-runtime` case in [local-probes.json](harness-critique/local-probes.json) uses a disposable home without the package, asks for `memory_recall: required`, and runs against a synthetic localhost model. It exits successfully with `status: complete`, `workflow.engine: ephemeral`, and no memory operation. The response does disclose ephemeral execution; the defect is that the explicit recall requirement is not honored.

**Impact:** a caller can require project memory for an instruction-sensitive task and still receive a completed answer produced without it. This is distinct from an available but empty store, which may legitimately return no facts.

**Fix:** preflight required recall before discovery/inference and reject if its runtime is unavailable. Keep `best_effort` and `off` semantics explicit; do not automatically install or enable the package. Put readiness/required-memory enforcement in the shared owned-run path, not only the Python recall node.

**Acceptance:** absent runtime + required → error and zero inference calls; absent runtime + best_effort → explicit unavailable/disabled receipt; available empty store + required → empty but successful recall; runtime failure + required → failure. Owner: router/durable and runtime bridge. Scope: small.

## F04 — The catalog offers an impossible worker route

`editable-interface` in [`registry/capabilities.json`](../registry/capabilities.json) requires Penpot operations. `capability_plan` advertises it for Web Design. [`tools::definitions`](../src/router/tools.rs) rejects all five Penpot object tools before discovery because their objects are not scoped safely to a worker project.

**Reproducer:** `editable-interface-route` calls `mirket models route` with that declared need and a native design-document specimen. It fails with the explicit host-connection requirement, without inference. Catalog validation still passes because [`workflows::validate`](../src/catalog/workflows.rs) validates known tool names and coverage, not worker eligibility. [Probe evidence](harness-critique/local-probes.json).

The refusal is the right authorization boundary. The defect is the shared discovery contract presenting it as a worker workflow without an execution-owner/availability distinction.

**Fix:** add an execution owner/status to the workflow contract and expose `host_only` with its reason and host handoff in plans. Exclude it from owned worker selection until object-level scoping exists. Do not remove the dispatcher protection or use a role override to bypass it.

**Acceptance:** iterate all workflow IDs through contract preparation; each must either yield an eligible owned plan or explicitly report host-only/unavailable before suggesting a spawn. The Penpot host path should remain available. Owner: catalog/workflows, assignment and tools. Scope: small to medium.

## F05 — The saved trace loses the cause of failure

[`finish_run`](../src/router/execution.rs) replaces every persisted failure with the same generic string. The CLI error chain contains the actual cause, but it is not retained in `trace.json`. Tool errors caught inside the loop can be retained; this finding concerns failures that escape the loop, including malformed model responses and policy/identity rejection.

**Reproducer:** a synthetic provider returns an incomplete JSON action. The caller receives `model action is invalid JSON: expected ident at line 1 column 2`. The saved incomplete trace contains neither the parser error, the failure class, nor a sanitized rejected-response identity. The `specific_error_retained` probe is false. [Evidence](harness-critique/local-probes.json).

**Impact:** once the host output is unavailable, different failure classes look identical. Recovery advice, incident grouping and evaluation failure accounting lose the observation that would distinguish a provider failure from a harness defect.

**Fix:** persist a bounded typed receipt containing stage, error code, selected target/model, retryability, effect certainty and a redacted diagnostic. For invalid output, retain a hash/size and optionally an explicitly permitted truncated sample. Do not dump raw provider messages, headers or credentials into traces.

**Acceptance:** malformed output, model identity mismatch, deadline, transport failure and uncertain mutation produce distinct durable failure codes; injected secrets stay redacted; no failed tool action is replayed. Owner: execution/adapters. Scope: small to medium.

## F06 — Routing has neither usage feedback nor measured efficiency

HTTP [`complete`](../src/router/execution.rs) parses the provider envelope and returns only the action. The run records requested completion limits, not actual token usage or per-attempt elapsed time. The local provider supplied 1,234 prompt tokens and 56 completion tokens; the successful trace discarded that receipt. This is **synthetic metadata plumbing evidence**, not a measured real-model cost.

[`rank_candidates`](../src/router/policy.rs) sorts primarily by highest tier, frontier/full-size signals, reasoning strength and capacities. The deterministic `Sort lines` probe has a T1 floor and two eligible candidates, but ranks the synthetic T3 frontier candidate first. `spawn` then raises the failover floor to the selected candidate’s tier. Name/description morphology also affects classification. These are intentional policy choices, not proof of superior task performance or malicious model metadata.

**Impact:** even mechanically simple work defaults toward the strongest eligible class, while the harness lacks the retained usage/latency data needed to assess the tradeoff. This review cannot quantify wasted spend or claim that a smaller model would succeed equally well.

**Fix:** preserve normalized provider usage with an explicit unavailable state, provider/model identity, wall-clock duration per attempt and retry/probe accounting. Keep pricing optional and versioned. Use fixed task families to compare the existing policy against explicit candidate policies; preserve user-selected model choices. Change defaults only after quality/latency/cost results justify it.

**Acceptance:** known synthetic usage survives to traces and evaluation export; missing usage stays null; retries are not double-counted; deterministic routing benchmarks include both failures and successful outputs. Owner: adapters/router and role evaluation. Scope: medium.

## F07 — Bounded source tools lack a way to recover from their bounds

`workspace_read` accepts only a path and rejects files over 128 KiB. `workspace_list` accepts one directory and rejects more than 1,000 entries. There is no owned range-read or text-search tool and no general shell escape. The limits are defensible; having no bounded continuation path makes normal repository investigation impossible for some inputs.

**Reproducer:** `source-over-read-limit` asks the assigned read tool for a 131,073-byte synthetic file and receives the byte-limit failure. The current registry catalog is 710,541 bytes and the design npm lock is 172,718 bytes, both beyond that boundary. The role named “analysis” or “source-review” does not supply another way to inspect them. [Tools](../src/router/tools.rs), [probe](harness-critique/local-probes.json).

**Fix:** add offset/line-range reads with full-file identity, bounded output and continuation; add literal/regex search with bounded matches and location references. Keep paths scoped and symlink/internal-directory checks intact. Distinguish a fragment-read receipt from full-file verification so reading one line cannot attest an entire changed artifact.

**Acceptance:** a worker can locate and inspect a fact beyond 128 KiB without receiving the whole file; changed bytes invalidate a continuation; directory pagination works; escaped paths and misleading partial verification fail. Owner: owned file tools and workflow registry. Scope: medium.

## F08 — Recovery is coupled to unused model candidates

In [`execution::resume`](../src/router/execution.rs), the code reloads the entire saved candidate list, refreshes discovery, and requires every candidate to exist and match its full serialized `Model` metadata. This check runs before reading `state.selected`. A disappearance or changed description/pricing/capability record on an unused fallback blocks resumption even when the actual selected model and task bindings are unchanged.

There is an additional conditional failure path: `verify_vision` can upgrade a Claude candidate from unknown vision to `Some(true)` and append a conformance-evidence string. A forced discovery refresh reconstructs models from provider metadata, while `resume` does not reapply that conformance receipt before full-object equality. If the provider still omits vision metadata, even the selected model can fail equality despite unchanged actual capability.

This rejection condition is directly visible in source; it was not exercised end-to-end against LangGraph in this review. It is stricter than needed to preserve the selected execution identity and can make recovery depend on unrelated provider catalog churn.

**Fix:** preserve strict task, tool, source and selected-provider identity bindings. Validate the selected model’s execution-relevant contract; validate an unused fallback if/when failover is required. Record exclusion of a changed fallback rather than blocking the primary continuation. Preserve or revalidate conformance evidence against explicit executable/model bindings instead of comparing augmented and raw metadata as if they were the same identity. Preserve explicit model-selection restrictions and original floors.

**Acceptance:** checkpoint with A selected and B unused; remove/rename B → A can resume; alter A’s execution contract or source/tool binding → resume fails; an unavailable fallback is never silently substituted below policy; a previously conformance-verified vision model can resume under the same verified contract. Owner: resume/discovery. Scope: medium.

## F09 — Automatic recall uses a lossy objective prefix

[`workflow.recall_node`](../runtime/agent/workflow.py) takes `request.objective[:256]`, then repeatedly halves that prefix if tokenizer overflow remains, and retrieves at most five facts. Task needs, later requirements and selected file context do not participate in that query.

**Reproducer:** two tasks with the same 256-character preamble but different critical Turkish/offline and German/reporting requirements produce identical queries. The [exact-function probe](harness-critique/feedback-probes.json) uses a controlled tokenizer and recall function; it establishes query construction, not real retrieval quality.

**Impact:** a long, carefully bounded brief can systematically omit the most relevant requirement from recall. The excerpt and objective hash are exposed, which helps diagnosis, but there is no request field to provide a better recall query through this path.

**Fix:** allow explicit bounded recall queries with provenance, or derive them from structured task needs using a deterministic policy. Keep input facts and retrieved text non-authoritative. If multiple queries are allowed, bound combined candidate count and deduplicate before the final context limit. Do not introduce an unmeasured LLM query-rewriter by default.

**Acceptance:** distinct requirements after character 256 yield distinct query coverage; Turkish/German examples include relevant facts; total recall remains bounded; the full brief is never silently treated as wholly searched. Owner: worker request contract and workflow recall. Scope: small to medium.

## F10 — The multilingual baseline reports relevance but does not enforce it

`test_retrieval_baseline_and_export_pagination` computes `recall_at_3` and prints it. Its assertions check export identity/pagination and result authority labels. It never asserts that expected facts were retrieved or that the unanswerable query stays empty.

**Discriminating probe:** executing the exact test body against a controlled memory fake that preserves pagination but returns zero search results passes all its assertions. All six answerable cases have recall@3 = 0. [Evidence](harness-critique/feedback-probes.json). A separate English semantics test does assert one successful retrieval; this finding is specifically about the multilingual/contradiction baseline and must not be read as “no retrieval assertions exist.”

**Fix:** freeze a small set of task-relevant per-case expectations and make the contained regression gate fail on their violation. Keep exploratory observations separate. Add representative consented project queries, tail-position requirements, stale/superseded facts, unanswerable cases and instruction-like retrieved text. Add answer-generation evaluation separately; good retrieval does not establish safe use of the facts.

The existing active-fact filtering also stops at 256 candidates and explicitly reports incomplete recall. Its regression test confirms an active fact can remain hidden behind 300 superseded candidates. This is an honest bounded limitation, not silent success. Evaluate indexing/filtering improvements on representative stores rather than simply raising the bound.

**Acceptance:** empty retrieval fails the relevance gate; wrong-language, wrong-current-fact and false-positive unanswerable cases fail independently; reports retain failures and incomplete-recall status. Owner: agent runtime tests and AI evaluation. Scope: medium.

## F11 — The quality loop is not yet a reliable basis for expanding guidance

The [16 October report](../evals/design-quality/RESULTS-2026-10-16.md) retained important negative evidence: no demonstrated design-quality gain, one improved and one regressed brief, an invalid primary analysis, overwritten failed self-check reports, and incompatible score-lock fields/path bindings. Its descriptive +0.1125 result did not meet its advancement thresholds. This is a strength in reporting, not a successful validation of the revised guidance.

Current packet tooling includes structural validators, score forms and a self-test. Therefore the fix is not “add a validator”; it is to exercise the **actual collector → score lock → packet verifier → primary analyzer** path before spending on a campaign. This review did not reopen private study assets or run a replacement trial, and it does not assume every historical serializer defect remains in the current helper.

The 347 skills, 41 roles and 63 profiles provide a large declared surface, but registration, exact delivery and role descriptions do not establish marginal usefulness. Existing authored guidance explicitly recognizes that distinction. There is no new measured gain established by this critique.

**Fix:** preflight one synthetic end-to-end packet using the same serializers/paths as the planned run; reject incompatible records before dispatch. Collect failed attempts in append-only, host-owned evidence outside worker write scope. Run a small fresh blinded comparison on the most common actual tasks, with independent artifact checks, fixed criteria and usage receipts from F06. Compare no-extra-guidance, targeted guidance and the proposed change; keep cases and failures rather than tuning on the confirmation set.

**Acceptance:** the production packet round-trip reaches the frozen analyzer; deliberate failure-report overwrite is detected at collection time; every planned cell survives; contradictory judge/task results remain visible. Expansion of profiles/references should be driven by those results. Owner: evaluation tooling and guidance owners. Scope: medium to large, with no quality-gain claim until measured.

## F12 — Durable recovery retains state but charges downtime

`ExecutionState::revalidate`, `Driver::summary` and Python `receive` compare the original absolute deadline with current time. A blocked worker or closed client can exhaust its 15/30/45-minute budget while no useful execution happens. Resuming does not reset it. Host-verification handling has a distinct path; this finding concerns continued worker execution, not a claim that all later host checks expire.

This is documented and prevents unlimited retries, so it is a product tradeoff rather than a hidden correctness failure. Nevertheless, it reduces the usefulness of durable state for interruptions longer than a short session.

**Fix:** consider an accumulated active-execution budget, conservative charging for uncertain in-flight work, and a separate maximum pause/retention interval. Preserve counts and evidence across resumes. Do not implement a simple deadline reset, which would defeat the current limit.

**Acceptance:** a known inactive pause does not consume active budget; a crashed uncertain call is charged conservatively; repeated resumes cannot reset accumulated time/turns; stale source/model/tool bindings still reject. Owner: durable execution. Scope: medium; validate need with real interruption traces first.

## F13 — A standard-gate startup timeout needs diagnosis

The full `mirket dev check --project .` run passed formatting and Clippy, then failed `motion_payload_installed_mcp_delivers_typed_sources_and_rejects_escaped_resources` at `tests/motion_payload.rs:305`: the 15-second MCP initialization timeout elapsed before payload assertions. The same test passed in a focused rerun without source changes. The earlier map turn also had a passing full gate on the unchanged production source. [Preserved logs](harness-critique/check-results.json).

This establishes a varying local test outcome, not its cause or a production startup SLA violation. The failing target ran multiple installation/payload tests together; the passing rerun isolated one. Resource contention is a hypothesis. Startup captures an executable digest, and the tested debug binary was about 79 MiB versus about 15 MiB for the installed optimized binary, but size alone does not attribute the timeout.

**Fix:** retain phase timing and bounded child diagnostics for spawn, identity capture, catalog construction and MCP initialization. Reproduce under controlled parallelism, explicitly isolate ambient native discovery where it is not part of the test, and measure the optimized path separately. Repair the identified blocking work or test isolation. Do not raise the deadline or add automatic retries merely to hide this failure.

**Acceptance:** the original scenario passes in bounded repeated concurrent and isolated runs; logs identify where time was spent; actual optimized-client startup is measured separately. Owner: MCP startup and integration-test harness. Scope: small diagnostic change first; remediation depends on its evidence.

## Missing tooling, in delivery order

| Need | Existing partial mechanism | Concrete addition |
| --- | --- | --- |
| Release assurance across languages | Rust gate, optional contained gate, manual JS tests | CI JS job and candidate/recipe-bound acceptance receipt |
| Dependency freshness/reachability | Pins, notices, Python audit and this one-time query | Recurring all-closure inventory/advisory review with expiring exceptions |
| Honest capability discovery | Role/workflow metadata and runtime refusal | Shared execution owner, availability and reason in plans |
| Durable diagnosis | Traces and caller errors | Typed redacted failure receipts and export |
| Efficiency evidence | Requested token caps and report cost fields | Actual usage/timing normalization, per-attempt accounting |
| Repository investigation | Whole-file read and one-directory list | Scoped range reads, bounded search and paginated listing |
| Reliable retrieval quality | Seven-query observation baseline | Per-case regression gate and representative evaluation |
| Valid evaluation campaigns | Forms, validators and separate analyzers | Same-path preflight plus immutable collection |

The role/tool mismatch also needs a clear product presentation. Registry inspection gives these concrete examples:

| Expertise selected | Current owned workflow coverage | External capability still needed for the full deliverable |
| --- | --- | --- |
| Deep Research | Analysis and structured response | Web/document retrieval and source capture through the host |
| Spreadsheet Analysis | Analysis and structured response | Native spreadsheet read/write/recalculation and saved-workbook inspection |
| Office Production | Paged media, raster/vector preparation, analysis | Editable DOCX/PPTX/XLSX authoring and native consumer verification; Typst PDF production is a narrower output |
| Database or Mobile Engineering | Source implementation/review and analysis | Actual database/device/test runtime through host verification |
| Marketing | Analysis and structured response | Authorized channel/analytics connectors when the requested work needs them |

These are **coverage boundaries, not five broken implementations**. The current plans already say `runtime_status: not_checked`; preserve that honesty and add concrete host-handoff/prerequisite receipts. More role names or a new profile cannot supply a missing producer. Identical skill membership also is not automatically redundant: `project-foundation` and `portable-foundation` have different delivery intent. Consolidation should follow observed selection conflicts or task failures, not raw inventory size.

A general shell in every worker, a second state store, a new vector database or another planning/profile system would not directly resolve these findings. Keep host-owned code execution, browser research, raster generation and external applications under their existing permissions; provide clear handoffs where the worker cannot complete the deliverable.

## Recommended implementation sequence

1. **Before the next release:** triage F02 and land the compatible security/compiler updates; bind release promotion to F01 evidence; investigate the observed F13 gate failure before treating the current check run as green. Preserve reachability exceptions where an upgrade requires more work.
2. **Repair misleading or lost contracts:** F03 required recall, F04 workflow availability, F05 failure receipts. These are bounded changes with deterministic regression tests.
3. **Make work inspectable and recoverable:** F06 usage/timing, F07 range/search, F08 selected-model recovery. This supplies the evidence and tooling needed for meaningful policy changes.
4. **Repair the learning loop:** F09 recall query selection, F10 relevance gates and F11 collector preflight. Then run representative comparisons before expanding guidance or changing routing defaults.
5. **Measure recovery demand:** use traces to decide whether F12 active-time accounting merits its complexity.

Each fix should retain its failing reproducer and be verified at the owning boundary. Do not batch all dependency major upgrades, all routing changes and all guidance edits into one experiment: attribution and rollback would become unnecessarily difficult.

## What is working and should be preserved

The review found real controls worth keeping: safe manual ZIP member selection; explicit host/worker authority boundaries; scoped file tools; refusal of unsafe shared Penpot object operations; exact-revision and idempotency checks; separation of readback from host verification; explicit incomplete-recall reporting; and public preservation of negative quality results. A larger tool surface should extend those controls rather than erase them.

## Evidence, reproducibility and limits

- [Local CLI probes](harness-critique/local-probes.json): six cases against the installed matching CLI, a localhost synthetic inference endpoint and disposable state. No external inference. Includes requests' observable results, saved traces and receipt checks.
- [Feedback probes](harness-critique/feedback-probes.json): exact selected owned function bodies with controlled fakes; not contained-runtime integration tests.
- [Dependency snapshot](harness-critique/dependency-audit.json): 679 OSV results, nine advisory bodies and 32 direct-version queries, dated 20 October 2026. Advisory matching is not exploitability proof.
- [Reproduction tools and scope](../evals/harness-critique/README.md): standard-library Python utilities. Current-defect assertions are diagnostic probes; they should be replaced by desired-behavior regressions when fixes land.
- Source locations above name owning functions as well as files so the review survives line movement. The map's original source fingerprint remains applicable: this review changes documentation/evaluation utilities only.

Not exercised here: real model quality/cost, the contained service matrix, native desktop activation, Windows/Linux native execution, full installed OS/container SBOMs, complete Penpot transitive reachability, all 47 upstream skill source pins against current heads, or a semantic review of every catalog entry. Existing private evidence and external CI protection were not assumed. Those limits do not weaken the reproduced local contract failures, but they constrain broader security, performance and quality claims.

Final verification status: catalog validation, report links/JSON, probe syntax, source hashes, six local CLI probes, two feedback component probes, and the focused source-contract tests passed. The full standard gate recorded 342 passed, 1 failed and 6 ignored tests before Cargo stopped; the isolated failed test subsequently passed. See the saved logs for exact per-target results. The full gate is **not reported as green**. No production fix was made to conceal or resolve the timing failure.
