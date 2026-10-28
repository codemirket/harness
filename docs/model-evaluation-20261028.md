# Post-connectivity native model evaluation

Later evidence: [supplemental evaluation](supplemental-model-evaluation-20261028.md).
This record preserves its original cohort; the later report supplies current
results and remaining qualification limits.

The user requested this evaluation after the authenticated CLI connectivity
prerequisite passed. Earlier timed-out, rate-limited and procedurally invalid
cohorts remain closed and are retained in their qualification records. This is
one explicitly frozen post-connectivity batch, not a retry hidden by a new root.

## Protocol and interpretation

Evidence root:
`/Users/nazmiyilmaz/.codex/task-state/harness-model-evaluation-20261028/`.
Its `protocol.json`, `seal.json`, `budget.json` and execution ledger bind the
inputs, runtime and one absolute deadline. Preparation corrections happened
before the first model call. Private protocol scripts are evaluation artifacts,
not installed Mirket tools or a second product agent controller.

The declared ceiling is 3,600 execution seconds, 23 lead cells and three native
coordination child turns; 90 seconds per ordinary cell, 120 per coordination arm,
zero retries and zero author repairs. An incomplete turn, timeout, interruption
or provider limit stops the batch. Cleanup after a deadline does not permit a
new call. Parent preparation and independent review are not included in trial
model-token totals. Independent semantic review gets one pass; it cannot revise
the initial artifacts.

All trial leads and children request `gpt-6-astra` with `high` reasoning. The
runtime is Codex CLI `0.164.0-alpha.11.1` on macOS 26.5.2 ARM64. CLI SHA-256:
`a04752e4d46a4d89e53e6f61c59c5349a638c8ce172ffbf391fd482b4d902a4e`.
The process-local `SSL_CERT_FILE=/etc/ssl/cert.pem` bundle is frozen at SHA-256
`9dae8d76e55cb08991f2b672d58999ea15560d910759c16b544f843bdffbb994`.
Certificate verification stays enabled. No model-cache seed is used.

Each cell has a disposable HOME, CODEX_HOME and project. Auth is copied only for
execution and removed afterward. The parent removes inherited environment
variables except its declared runtime/proxy allowlist. Codex owns workspace-write
and automatic approval review. No live installation or user configuration is
changed. The hidden oracle is withheld from prompts/projects, **not isolated from
all filesystem reads**; trace review is required. Host model settings do not
independently attest provider identity.

The six repeated synthetic research questions are development calibration,
paired with `evidence-research`. `browser-research` explicitly skips research fully
answered by supplied text, so it is reserved for the live-web and near-miss pair.
No skill name or forced AGENTS instruction is given to these authors. Installed
skill presence and a body read remain distinct from native activation or causal
benefit. The product pair evaluates a six-skill bundle, and its explicit brief
already contains substantial method guidance; it cannot isolate a skill effect.

The native reviewer pair uses five frozen clean/defective/unavailable claims.
The product pair must preserve real/synthetic/inferred distinctions, reject an
expired metric and a future-only alternative, identify the observed funnel
bottleneck, and save useful grounded copy. The near miss rewrites supplied text
without research. The live-web pair investigates current GitHub Actions pending
queues, limits, cancellation, ordering and historical advice using official
sources. One live topic cannot establish broad research coverage.

The coordination comparison uses identical `[3,3,8,-2]` input in no-delegate,
one-worker and two-worker arms. Exact sum/count files and lead-owned integration
are checked. It is a small mechanical cost comparison, not a creative model or
general parallel speedup benchmark.

## Scoring corrections

`tests/research-quality-trials.py` now records the frozen research skill ID and
accepts the appropriate `evidence-research` corpus pairing. Its lexical grader
can accept a false negated restriction containing the expected words, so
`mechanical_checks_passed` is separate from semantic outcome. A mechanical pass
leaves `outcome_observed` null pending independent semantic review. A new
regression demonstrates this with “not limited to paid workspaces.” No keyword
heuristic is presented as a general entailment evaluator.

## Actual results and finite stop

**Twelve native corpus turns completed. All twelve saved answers passed a
condition-blinded independent semantic review.** Both conditions correctly
handled all six questions, including missing evidence and wrong-product
exclusion. The with-skill body was read in all six enabled cells; none of the
six baselines read it. Native activation remains unknown. This is a tie on a
small reused calibration set, not evidence of causal skill uplift.

| Condition | Completed cells | Semantic passes | Summed seconds | Input tokens | Cached input subset | Output tokens |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| No skill | 6 | 6 | 143.870 | 365,149 | 337,408 | 2,538 |
| Evidence-research available | 6 | 6 | 167.278 | 405,386 | 372,736 | 3,549 |

These are host-reported usage totals from completed-turn events, not billing or
provider attestation. Cached input is included in input totals; do not add it
again. The fixed baseline-first order and one sample per question cannot
establish a reliable latency or token effect. Native captured execution totals
311.148 seconds; shared elapsed time through the final stop was 584.360 seconds,
including evaluator diagnosis/corrections. The original deadline never moved.

The original mechanical grader passed only **6/12**, while the independent
semantic reviewer passed **12/12**. This disagreement was investigated against
the actual sources:

- Both compound-claim answers cited Atlas v1 and v2, correctly establishing
  export and preserving the absence of import evidence. The exact citation
  list rejected relevant history. The future grader now allows v1 additionally
  for this case while still requiring v2 and rejecting unrelated/duplicate IDs.
- Both unavailable-source answers used `insufficient`, recorded
  `route_status: unavailable`, and cited the consulted Atlas corpus while
  explicitly abstaining. The old prompt did not define the verdict distinction
  or whether `sources` meant supporting versus consulted evidence.
- Both wrong-project answers used descriptive strings such as `Atlas scheduled
  exports` and `Atlas v2, paid workspaces`. They did not transfer Beacon's
  capability, but the oracle expected the literal `Atlas` in both fields.

The future prompt now states exact product-name fields, the verdict distinction
and supporting-source semantics. Its new wording is **not natively qualified by
these old answers**. A separately labelled post-hoc regrade of the citation
allowance alone passes 8/12; it does not replace the frozen score or convert
this into a clean preregistered pass. Semantic reviews remain bound to the
original saved answers and corpus hashes.

The outer supervisor had two false stops before the real terminal failure:

1. It matched `429` in the completed-turn token count `62429`. The repository
   detector now inspects diagnostic error events/messages, not usage or ordinary
   tool output. Tests reject the false positive and detect genuine HTTP/API
   rate-limit errors. `evaluator-correction/` seals the corrected detector and
   resumes only unstarted cells, with the original six still counted.
2. Its full metadata comparison rejected `/etc/ssl/cert.pem` versus the canonical
   `/private/etc/ssl/cert.pem`. `samefile` and the frozen content hash established
   identity. `path-equivalence-correction/` records the correction before any
   supplemental call. Neither false stop reset a deadline or repeated a turn.

The first supplemental invocation then exited **2 in 0.019 seconds**, with no
model events or artifact: this CLI rejects `--approve-for-me` together with
`-s workspace-write`. That is a real failed invocation, not a model-quality
failure or a provider limit. The frozen stop rule ended the batch permanently.
The unexecuted repair in `invocation-repair/` removes the redundant sandbox
option and has a no-provider argument/cleanup regression check. The twelve
successful corpus invocations already demonstrate the sole automatic-review
flag on this same runtime; the supplemental repair itself has not been rerun.

Final ledger: **12 completed model turns, one failed local CLI invocation,
10 unstarted lead cells, zero coordination child turns, zero repeated cells and
zero author repairs.** Native reviewer-control, product/marketing, near-miss,
live-web and no-delegate comparisons therefore remain unqualified. Source
snapshots, planned prompts and a fixed runner do not count as executed evidence.
The independent scope audit inspected 38 command actions and 12 root answer-file
writes. It found no observed oracle reads, external sources, network access,
credential access or out-of-scope writes. This is a trace observation, not OS
read isolation. The auth-copy scan found no copied auth files remaining.

## Evidence and checks

`reviews/corpus-independent-review.json` retains the one-pass blinded findings
and 29 packet hashes. `reviews/joined-outcomes.json` joins exact answer/event
hashes, original mechanical scores, independent semantics and the labelled
post-hoc regrade. `reviews/corpus-usage.json` retains per-cell usage; `reviews/final-audit.json`
records the independent action audit and verification of all 222 original sealed
inputs. The original
`ledger.json` and `stop.json`, both correction addenda, and the failed
`supplemental/reviewer-no-skill/` directory remain unchanged. The active final
ledger is `path-equivalence-correction/ledger.json`; no earlier attempt is erased.

Current repository checks: 228 Python tests ran, with 227 passes and one existing skip;
22 focused research tests passed after the final prompt clarification. Catalog
refresh verification reports no changed hashes (9 global / 58 project skills).
Whitespace/diff checks pass. The private corrected invocation passes its
no-provider regression. No Rust, adapter, dependency, skill body or runtime
implementation changed; their earlier locked/offline Rust, Clippy, formatting,
locked adapter and explicit-binary/private-Node MCP gates were not rerun for
these evaluator/document changes.

The batch makes no claim about broad web coverage, product-market demand,
creative/model superiority, Windows, the macOS 15 floor, or the waived Desktop
GUI check. All 335 catalog development cases retain their recorded planned
status. No live installation, push or publication occurred.
