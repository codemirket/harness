# Remaining native model evaluations — 2026-10-28

The bounded continuation finished with useful research/reviewer and mechanical
coordination evidence, but **product-skill output remains unqualified**. Both
valid product baselines scored 12/12 independently; their skill-enabled partners
timed out without artifacts. A concrete artifact-layout conflict was corrected
in six skills. That correction did not demonstrate a latency improvement.

This supplements the [previous closed evaluation](model-evaluation-20261028.md)
and [approved plan](../planning/research-improvements.md). It does not replace
earlier failures, claim broad skill uplift, or reopen the accepted D6 design.

## Protocol and accounting

Private evidence root:
`/Users/nazmiyilmaz/.codex/task-state/harness-supplemental-evaluation-20261028/`.
`protocol.json`, `seal.json`, `budget.json` and two explicit addenda retain the
inputs, runtime, attempts and original absolute deadline. The initial ceiling
was 11 lead attempts plus three coordination children, 180 seconds per cell or
coordination arm, and 2,400 seconds shared execution time. Independent groups
could continue after a failed cell; shared provider/authentication errors,
interruption, input drift or the absolute deadline would stop dependent work.

Two declared addenda added two lead attempts each: correction of a materialized
input defect, then one fresh held-out product pair after a guidance correction.
The total became **15 lead attempts and three child turns**. The deadline was
never reset. All earlier failed attempts remain counted. There were zero
automatic retries and zero author-artifact repairs, one skill-guidance repair,
and one declared new specimen after failure. Twelve lead turns and all three
children completed; three lead turns timed out. The initial product pair's two
attempts are diagnostic-only because the intended brief was overwritten.
No additional latency retry was made after the final timeout.

All leads and children requested Astra/high on Codex CLI 0.164.0-alpha.11.1,
macOS 26.5.2 ARM64. Runtime SHA-256:
`a04752e4d46a4d89e53e6f61c59c5349a638c8ce172ffbf391fd482b4d902a4e`.
Disposable processes used `SSL_CERT_FILE=/etc/ssl/cert.pem`, SHA-256
`9dae8d76e55cb08991f2b672d58999ea15560d910759c16b544f843bdffbb994`;
certificate verification stayed enabled. No model-cache seed was supplied.
CLI execution used the sole `--approve-for-me` flag. App-server used
`on-request` with `auto_review`. No live configuration was changed.

Each cell had disposable HOME, CODEX_HOME and project state, a minimal inherited
environment, and temporary auth removed after execution. The independent audit
found no auth-copy residue. Oracles were withheld from prompts/projects, not
OS read-isolated. Scope checks and observed commands are evidence, not a
security sandbox. Model settings do not attest provider identity.

Matched arms had the same prompt, input, requested model and time allowance;
the installed skill tree was the intended difference. Fixed order, small
fixtures and one observation per arm preclude a causal or general uplift claim.
Parent preparation and independent reviewers are outside trial token totals.

## Saved outcomes

| Case | No-skill seconds | With-skill seconds | Independently observed result |
| --- | ---: | ---: | --- |
| Reviewer controls | 29.310 | 33.501 | Both correctly handled all five clean, swapped, stale, partial and inaccessible-source controls, including original claim/source locators |
| Browser near miss | 19.502 | 21.977 | Both preserved meaning and uncertainty; neither browsed, and the enabled arm did not read the browser skill |
| Live-web research | 66.741 | 97.729 | Both covered all five requested topics with current official sources and appropriate limits |
| Initial product pair | 143.642 | 180.035, timeout | Invalid intended brief in both arms; excluded from product qualification |
| Corrected product inputs | 133.820 | 180.028, timeout | Baseline 12/12; enabled arm produced no artifact |
| Fresh product scope test | 120.346 | 180.029, timeout | Baseline 12/12; enabled arm produced no artifact after the skill correction |

The six reviewer/near-miss/live artifacts received one independent semantic
review from an A/B-labelled packet with the condition mapping stored separately. Both valid product baselines received one review
against their respective frozen brief, corpus, oracle and 12-point rubric.
Those reviews were not blinded because no enabled artifact existed. Root
integration inspected the saved decisions, funnel arithmetic and evidence
limits. No author output was repaired by a reviewer.

The live topic was GitHub Actions concurrency: default replacement of the one
pending run, `queue: max` capacity of 100 pending runs, overflow behavior,
incompatibility with `cancel-in-progress: true`, and waiting-start order versus
dispatch order. Both authors performed a two-query search and one open
operation. Its action summary names the main documentation page; its results
list all three pages with separate reference IDs and line counts. The reviewer
separately checked the actual answers against current
[concurrency documentation](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency),
[limits](https://docs.github.com/en/actions/reference/limits) and the
[dated May 7 changelog](https://github.blog/changelog/2026-05-07-github-actions-concurrency-groups-now-allow-larger-queues/).
Snapshots, access dates and hashes are retained. Neither answer establishes
actual GitHub workflow execution, Enterprise Server behavior or broad research
coverage across platforms.

The enabled reviewer read `critical-review`. The enabled live author read
`browser-research` and its coverage reference. Each enabled product author read
all six relevant bodies and resources. Successful reads and model statements
about applying a skill are not independent native-activation evidence or proof
of benefit. The product timeouts occurred after completed reads; no provider
error was recorded. Their cause is unknown. They must not be described as a
proven model, provider, context-size or instruction-conflict diagnosis.

Both valid product baselines separated synthetic praise, expired metrics and
future competitors from current proof, calculated the actual weakest funnel
stage, and saved useful audience-specific copy. The fresh case changed the
product, audience, alternatives, evidence IDs and weakest stage from activation
to retention. All data were evaluation fixtures, including records labelled
“real” inside the exercise; no real customer demand or conversion lift follows.

## Findings and corrections

| Reproducible consequence | Source / correction | Verification and limit |
| --- | --- | --- |
| Six skill instructions prescribe extra sidecars despite a task requesting one file | `skills/{customer-research,product-strategy,positioning,conversion,marketing-copy,experiments-launch}/SKILL.md` and each corresponding reference now treat paths as defaults, honor the caller's layout and share evidence records | Independent source review, catalog/assembly and full repository gates passed. Fresh enabled model still timed out; performance benefit is unproved |
| Marketing guidance could imply an installed checker exists | `skills/marketing-copy/SKILL.md` now qualifies the checker as repository qualification tooling when available | No new installed tool or capability promise |
| `BRIEF.md` overwrote `brief.md` on case-insensitive macOS, invalidating intended inputs before execution | Private `product-fixture-repair/supplemental_runner.py` removes the redundant file and checks byte equality and recursive case-fold path uniqueness | A no-provider regression reproduces/refuses the alias; both corrected projects match frozen inputs. Initial pair remains diagnostic |
| Coordination finalization looked up `result_json` instead of the frozen `result.json` key and falsely labelled completed turns as setup errors | Private `coordination-recovery/frozen-runner-fixed.mjs`; recovery reconciles retained native rollouts and actual artifacts | Private regression checks exact key and evidence preservation on grading failure. Recovery made no model calls; missing original app-server response records remain missing |
| Product enabled arms fail the 180-second useful-output contract | Retained results and independent baseline review; no unsupported production performance fix | Two valid paired tasks failed on the enabled arm. Do not qualify R7/R8 usefulness or claim uplift from source checks |

Repository-owned `scripts/refresh-skill-catalog.py --write` refreshed catalog
hashes and `planning/reviews/2026-10-27/reference-audit.json`. Provenance was
processed but unchanged because no new source or reuse method was admitted.
The nine globals, 58 project IDs, dependencies and ownership boundary remain
unchanged. The archived pre-change catalog matches the original trial seal;
the current catalog intentionally differs in the six corrected entries.
Do not assert that every original external source path still has its old hash.

## Coordination and cost

The same `[3,3,8,-2]` task produced sum 12, count 4 and lead-owned integration in
all three arms. Actual command traces show disjoint child writes, unchanged
sources and only the permitted outputs. The collector failed after execution;
these outcomes were recovered from retained native evidence, not inferred from
its erroneous summary or rerun to obtain cleaner records.

| Arm | Children | Lead elapsed seconds | Aggregate host-reported tokens |
| --- | ---: | ---: | ---: |
| No delegate | 0 | 20.763 | 48,448 |
| One worker | 1 | 39.135 | 174,860 |
| Two workers | 2 | 61.000 | 317,943 |

Elapsed time is native lead task-start to completion; overlapping child time is
not added. Latest cumulative usage is counted once per thread. This mechanical
specimen favors no delegation; it does not establish a universal policy or say
anything about creative reviewers. All six native contexts recorded Astra/high
and automatic approval review. No approval-request records were found, but the
lost app-server protocol means exact request/response reconstruction is limited.
Mirket cannot hard-enforce native spawning or model choices.

The 12 ordinary cell attempts consumed 1,206.660 summed seconds, including all
three timeouts and the invalid initial product pair. Nine completed ordinary
turns reported 837,773 input tokens (752,640 cached subset) and 21,332 output
tokens. Coordination reported 541,251 aggregate tokens across leads/children.
These are host measurements, not billing; cached input must not be added again.
Timed-out turn usage is unknown, not zero, so these are not complete spend totals.

## Verification and retained evidence

The independent final audit reconciled all 12 ordinary result/event/output
joins and manifests, both addenda, recovered coordination ownership, archived
source identities and auth cleanup. No project scope drift was observed.
`reviews/joined-outcomes.json`, `reviews/research-independent-review.json`,
`reviews/product-independent-review.json`, `reviews/final-audit.json` and
`coordination-recovery/{recovery.json,commands.json,hash-manifest.json}` retain
the detailed results. Original erroneous ledgers remain unchanged; recovery
and addenda explain their defects instead of silently rewriting history.
`reviews/closure-manifest.json` binds those records and protocols at SHA-256
`7b085545cbf9ced1087ca673bc754493da27ae44fd459e0903825dcd0b8f7ae6`.

After the skill changes, on macOS 26.5.2 ARM64:

| Repository gate | Actual result |
| --- | --- |
| Build-input guard before Rust compilation | Passed |
| Locked/offline Rust tests | 509 passed, 5 ignored |
| Locked/offline all-target Clippy, warnings denied | Passed |
| Rust formatting | Passed |
| Adapter lock audit | 209 installed entries matched; 50 absent optional entries; no missing required entries or mismatches |
| TypeScript compilation with explicit private Node 24.22.1 | Passed |
| Adapter process tests | 68 passed, 1 skipped |
| Python tests | 228 run: 227 passed, 1 skipped |
| Explicit-binary/private-Node MCP smoke | Passed inventory/status, stale/absent/unknown routes, configured pairing, stale-source/unsupported/wrong-project refusals |
| Catalog refresh, independent catalog/assembly checks, diff check | Passed |

Smoke used `/Users/nazmiyilmaz/Projects/harness/target/debug/mirket` and
`/Users/nazmiyilmaz/.codex/task-state/harness-deep-review-20261027/release-source/runtime/node/bin/node`.
Logs live under the evidence root. Private runner regressions additionally
checked CLI arguments/auth cleanup, materialization collisions and coordination
finalization without provider calls.

Remaining qualification is explicit: product-skill useful output within the
budget, general research/creative uplift, Windows, macOS 15 floor and the waived
Desktop GUI check. A future product performance investigation should isolate
bundle/resource loading and artifact size under a separately frozen comparison;
it must retain these failures and must not simply extend deadlines until a pass.
No further current production defect is established by the timeout traces.
All 335 catalog development acceptance cases retain their recorded status.
No live installation, new dependency, push or publication was changed.
