# Synthetic memory retrieval evaluation contract

Status: root accepted the numeric qualification gates before any model scores
on 2026-10-24. Corpus SHA-256 is frozen as
`cc56a0141c41fec880bd8b528d3cd81c496e193719d5ec1dc6687bd6751d3a4c`.
Root independently reviewed 25 selected gold/context/confounder cases with no
annotation defect found. The evaluator was frozen and the first authorized native
model run failed three hybrid gate groups; measured evidence appears below. These
gates qualify this public synthetic corpus, not unseen generalization or
abstention behavior. No answer generator or abstention classifier was evaluated.

## Corpus and intended use

`tests/fixtures/memory-retrieval.json` contains 73 synthetic record revisions and
122 manually authored queries. Three fictional projects and a separate shared
preferences scope exercise lexical and semantic retrieval without user data or
external article excerpts. Every source uses a `fixture://` identity. The records,
quoted commands and questions are evaluation data, never executable instructions,
authorization or a prompt to ingest user transcripts.

| Query group | Count |
| --- | ---: |
| English question, English evidence, answerable current recall | 24 |
| Turkish question, Turkish evidence, answerable current recall | 24 |
| English question, Turkish evidence, answerable current recall | 16 |
| Turkish question, English evidence, answerable current recall | 16 |
| Missing eligible evidence, English / Turkish | 16 / 16 |
| Explicit ambiguity or unresolved conflict | 8 |
| Exact historical revision read, excluded from current-recall scores | 2 |

The four language groups use varied questions, including different consequences
of the same fact; they are not four copies of a translated question list. Some
facts recur to test language transfer, so observations are correlated. Report
counts and uncertainty, and do not describe this as 122 independent user tasks.
The public qualification set is inspectable by implementers and is not a blind
held-out test. A later unseen corpus is needed for a generalization claim.

Examples include exact Rust/API identifiers; Turkish `İ`, `ı`, accents and paths;
production versus sandbox currency; one service's timeout versus another's;
repository-wide knowledge versus checkout-local tasks; conflicting license,
temperature and pagination observations; and an explicit Podman-to-host-tools
policy reversal. Current recall must find the replacement policy while exact
history can still inspect its old revision. Preservation of history must not
make the former policy current again.

## Fixture schema and eligibility

`records[].id` identifies one fixture revision. `record_id` identifies its logical
lineage, and `revision` is its canonical revision. `is_current_revision` is
separate from the content's `status` (`current` or `unresolved`). The latter does
not make an old revision eligible. Records also carry source identity/revision,
locator/capture time, basis, type, scope, applicability, validity and supersession.
The frozen evaluation instant is `1800000000`; use it instead of the wall clock.

A future runner must seed through the canonical store and retain an explicit map
from fixture lineage/revision IDs to issued runtime IDs. It must preserve
history, status, validity and applicability, rather than concatenating every
revision into one embedding document. Fixture `route` and `partition` are test
metadata, not additional public tool parameters. The ignored manual integration test `tests/memory_retrieval.rs` performs this
seeding and comparison in a dedicated fresh process.

For `current_recall`, the eligibility oracle requires exact scope, the canonical
head, selected status, and `valid_from <= at_time < valid_until` where a bound
exists. Every non-null applicability field must match supplied context exactly.
Missing context is not a wildcard. A mismatched or absent checkout does not
permit another checkout's task record. Shared preferences never join a project
query implicitly. Apply canonical eligibility before ranking and recheck the
resolved canonical revision before returning results, including when a derived
index is stale. `exact_history` tests use the named revision separately.

Each query specifies gold revision IDs and relevance grades: 3 directly answers
the question; 2 supplies necessary evidence about an unresolved question. All
unlisted records have relevance 0 for that query. A conflicting pair has two
gold records; one disputed record may itself describe both unresolved choices.
No gold answer prose is synthesized from retrieval output. Negative queries have
empty gold because the scoped, eligible corpus lacks the requested information;
a fact that directly disproves a premise is answer evidence, not a negative.

`excluded_confounders` documents tempting nonanswers and why they do not establish
the requested fact. It is not a blanket ban on returning a current, eligible but
irrelevant nearest neighbor. Scope, currentness and applicability violations are
separate correctness failures; an eligible irrelevant neighbor instead lowers
ranking quality or must remain explicitly unsupported. Preserve exact spelling
of identifiers and paths: do not lowercase paths, strip Turkish diacritics,
normalize away underscores or replace near-match IDs before scoring. Do not
translate the stored evidence or queries as a hidden preprocessing aid. If a
product explicitly adds query translation later, measure and label it as another
pipeline, including its latency and errors.

`expected_critical_revision_ids` lists applicable current critical constraints.
`expected_unknown_context_critical_revision_ids` lists otherwise relevant critical
constraints whose applicability cannot be decided because required context is
absent, excluding known mismatches. The latter requires an observable coverage
gap, not application of that constraint. Critical constraints are evaluated
independently of query relevance and must not improve semantic ranking scores
merely by appearing in a mandatory coverage section.

## Accepted gates frozen before model evaluation

Hybrid is the qualifying shipped mode. Lexical and semantic-only pipelines are
measured diagnostic baselines, not separate required ranking passes. Every
pipeline must pass canonical eligibility and critical-coverage checks.

Root accepted the following numeric gates before the first model result. Freeze
the runner and pipeline parameters by digest before authorized execution. Do not weaken a threshold or rewrite gold to accommodate a failed
model. A justified annotation repair requires a new corpus version, recorded
reason and rerun of every compared pipeline.

| Measurement | Accepted gate | Denominator |
| --- | --- | --- |
| Mean Recall@5 | At least 0.90 overall and 0.85 in each direction | 80 answerable current queries; 24/24/16/16 by direction |
| Mean nDCG@5 | At least 0.85 overall and 0.80 in each direction | Same 80 queries |
| Exact identifier/path relevant result at rank 1 | 18 of 18 | Answerable current queries tagged `identifier` or `path` |
| Complete unresolved-evidence coverage@5 | 8 of 8 include all gold | Ambiguous queries |
| False supported result on missing-information queries | At most 1 of 32 | Negative queries, only if the pipeline exposes a support/abstention decision |
| Invented resolution of an unresolved choice | 0 of 8 | Answer stage, if any; retrieval alone cannot pass this gate |
| Wrong scope, non-head current recall, known-inapplicable or excluded lineage leak | Zero | All returned current results, plus separate deletion/index-staleness fixtures |
| Exact historical revision identity/provenance | 2 of 2 | Historical queries, outside ranking aggregates |
| Applicable critical coverage and unknown-context reporting | All expected constraints returned or accurately reported as omitted; all unknown-context gaps reported | Every current query plus constrained-budget variants |

Recall@5 is retrieved gold count divided by gold count, averaged per query.
nDCG@5 uses gain `2^grade - 1` and discount `log2(rank + 1)`, with ideal ordering
from that query's gold. Report MRR, Recall@1/3/5 and per-direction scores as
additional diagnostics. Rankings must deduplicate canonical revision identity;
duplicate hits cannot earn extra credit. Measure query-ranked results separately
from mandatory critical coverage, and also report the actual bounded merged
result so the ranking score does not hide critical items consuming its budget.
A ranker that only exposes merged results must report that limitation; do not
infer an unobservable pre-merge ranking.

A nearest-neighbor list is not an answer assertion: raw semantic retrieval will
often return candidates for a negative query. Do not label every candidate a
false supported answer, or count a mandatory critical constraint as one. The
negative gate measures a separately observable acceptance/abstention decision.
If the product has no such stage, report the gate as unmeasured, report negative
candidate-score distributions, and do not claim trustworthy no-answer handling.
Any acceptance score threshold must be calibrated on a separate development
set, not fitted on these 32 negatives. Answer-stage judgments require source-ID
attribution and explicit human adjudication of uncertain cases; an LLM judge
alone is not evidence of correct grounding.

The small corpus cannot qualify storage scale, deletion, backup or crash safety.
Add separate deterministic variants that shorten count/byte budgets, omit
applicability context, change canonical heads after indexing, forget a lineage,
and load a mismatched index generation. An overflow must be observable and may
not silently drop critical constraints. No scope/head/deletion leak is tolerated
regardless of average retrieval quality. The existing canonical MCP and lifecycle
tests remain required, rather than being replaced by a semantic score.

## Reproducible comparison and reporting

Run the existing lexical baseline, the proposed semantic ranker and the proposed
fusion pipeline against the same canonical snapshots, query texts and eligibility
oracle. Keep tokenization, model/tokenizer digests, dimensions, truncation,
normalization, pooling, distance metric, top-k, fusion constants and tie-breaking
explicit. Ties should be stable by canonical identity. Report failures by query
ID with returned fixture IDs, rank, score, eligibility and source evidence; do
not retain unrelated user content. Compare lexical versus semantic failures to
verify that cross-language gains do not hide exact-identifier regressions.

The artifact should include the corpus and runner digests, release/binary and
model/tokenizer identities, OS/architecture, runtime versions, warm/cold state,
per-query results, aggregates and raw counts. Report Wilson intervals for binary
hit rates and seeded, lineage-clustered bootstrap intervals for aggregate ranking
metrics; do not assert statistical significance from the small shared-fact set.
Preserve failed attempts and parameter changes. A later tuning run is not the
first frozen qualification run.

Measure cold initialization, warm end-to-end recall latency, peak and idle RSS,
index bytes and indexing time independently from quality. Report p50/p95 and
sample count, without mixing model startup into some queries but not others.
The project's proposed warm focused-recall p95 under one second at 10,000 records
requires a separate measured scale fixture; this 73-revision corpus cannot prove
it. A 100,000-record measurement must be reported separately if undertaken. Report
physical hardware, thread settings and concurrency. Offline local inference has
no external API charge but still consumes CPU, memory and disk; record those costs
rather than claiming zero cost. Neither a passing score nor a file hash establishes
native-client activation, permission enforcement or user acceptance.

## Review evidence

The author reviewed gold against the synthetic source records and corrected
negative cases that were actually answerable by contradiction. Structural checks
confirmed unique IDs, all gold/confounder references, current gold eligibility,
language-direction labels, valid shared-preference types, exact history references
and critical applicability labels. Turkish characters remain present in the saved
UTF-8 JSON. These checks establish fixture consistency only. Root independently reviewed the 25 requested subset cases (policy reversal and
history, exact Turkish/path identity, checkout tasks, corrected negatives and
unresolved evidence) and found no annotation defect. Evaluator qualification and
actual model scores were subsequently measured without annotation changes; the
first frozen result is recorded below. Numeric gates remain unchanged.

## Manual evaluator freeze and execution boundary

The evaluator source is `tests/memory_retrieval.rs`, frozen before model execution
with SHA-256
`aab656c4f5c1d2b3b46ad3ba022bd80063262f482c3889f4d91f0da19e65fcc6`.
Its canonical numeric gate JSON has SHA-256
`7da9698ee965dfbaee365eed77d30bd71f4c82fd805ef3ae471c714a073a8dcc`.
The gate JSON is embedded in that source; the runner checks its compiled corpus
digest and requires the externally supplied reviewed runner digest. Subsequent
source changes require a new explicit review/freeze before execution.

`cargo check --locked --test memory_retrieval` and
`cargo clippy --locked --test memory_retrieval -- -D warnings` passed without
loading the model. The additional ordinary test
`frozen_fixture_seeds_and_gold_is_canonically_eligible_without_model` exercises
canonical seeding, retained history and gold eligibility without inference. It
passed in 0.18 seconds through the freshly built test executable, SHA-256
`3caf37cd7d85363047bde239b9bb2d5d7598db88b48f1a5cba18e51c8b79ba12`.
The Cargo command had built that executable but failed on a concurrent
`Server::new` call-site migration in the separate main binary; the seed-only
test was therefore selected directly from the test executable. No model was
loaded. Integration-test compilation may rebuild binary targets, so it must not
overlap transport tests that own `target/debug/mirket`.

After root explicitly permits the model run and transport tests finish, use a
fresh process with one selected ignored test and an explicit private package:

```sh
ORT_DISABLE_TELEMETRY=1 \
MIRKET_TEST_EMBEDDING_PACKAGE=/absolute/verified/embedding-package \
MIRKET_RETRIEVAL_REPORT=/absolute/new-private-report.json \
MIRKET_RETRIEVAL_RUNNER_SHA256=aab656c4f5c1d2b3b46ad3ba022bd80063262f482c3889f4d91f0da19e65fcc6 \
MIRKET_RETRIEVAL_EXECUTE=frozen-qualified-run \
cargo test --locked --test memory_retrieval qualify_synthetic_memory_retrieval \
  -- --ignored --exact --test-threads=1 --nocapture
```

The report path must be new and absolute; the runner never overwrites a prior
report. It creates task-owned private scratch, seeds through canonical mutation
APIs and checks the saved records, loads one verified model, selects index batches
of at most 32 records/256 KiB, and publishes passages only against the selected
reference and recomputed source digest. It never fetches a model or runs MCP
inference. It retains incremental JSON reports on ordinary errors and failed
gates, and removes only its own disposable scratch. An abrupt process death can
leave scratch; the runner does not claim crash cleanup.

Reports separate the unmerged ranked candidate IDs/scores from bounded public
hits and mandatory critical coverage. They include two constrained-budget
variants, per-query source mappings, raw counts, lineage-clustered bootstrap
intervals (2,000 resamples, seed 42), Wilson intervals for binary subset gates,
model/runtime pins, source/executable digests and timing methodology. Passage
indexing warms the model before query timing. One query embedding is shared for
the controlled comparison and charged in full to each semantic pipeline; lexical
timing has no embedding charge. Pipelines run sequentially in fixed lexical,
semantic, hybrid order without resetting OS or SQLite caches. Those timings are
honest component sums, not independent native-client latency samples. Hardware
SKU, RSS, full index storage bytes and scale qualification require separately
recorded measurements; they are not inferred from this corpus run.

The evaluator does not invent a support classifier or an answer-generation stage.
Abstention and generated conflict resolution remain explicitly unmeasured even
if the observable retrieval gates pass. It also does not replace the independent
index-staleness, deletion, backup or concurrency tests.

Pre-score correction: the native pipe test observed ORT `.ses` initialization
even with the API telemetry option disabled. Before any corpus model scores, the
runner was changed to require `ORT_DISABLE_TELEMETRY=1` at process startup, before
loading the native runtime. The execution example includes that environment
setting. This is a transport/privacy correction, not tuning based on retrieval
results; the corpus and numeric gate digests remain unchanged. The superseded
runner digest was
`ff5f2acb39bfd0cb81e7361afbade2200df19dd259dbe3429d886c2235975844`.

## First frozen native result: qualification failed

On 2026-10-24 at approximately 18:44 UTC the authorized frozen runner completed
in a fresh process under macOS `sandbox-exec` network denial, with
`ORT_DISABLE_TELEMETRY=1` and `TOKENIZERS_PARALLELISM=false`. The approved local
package was `releases/m2-semantic-aarch64-apple-darwin` in task state. The test
executable was rebuilt from current sources before the run; its SHA-256 was
`72945cb4f2e4b13a37daa3e4001bd1df45e7a0911f21c8c7ff0ed70d511f72c6`.
Corpus, gates and corrected runner digests exactly matched the frozen values
above. No corpus annotation, ranking parameter or threshold changed after scores.

The complete failed JSON report and stdout/stderr are retained under
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/memory-retrieval-20261024T184340Z/`.
`report.json` SHA-256 is
`bfd51650dd799d5a7d3822058e55bdd0a12e7308f123d620e604164d298703f4`.
It contains 366 pipeline rows (120 current queries and two budget variants for
three pipelines), both history checks, per-query IDs/scores/coverage, intervals,
raw counts, runtime/model pins, timings and the explicit gate failures. The
process exited with failed-test status; the report is not a passing receipt.

| Hybrid group | Recall@5 | nDCG@5 | Result |
| --- | ---: | ---: | --- |
| All answerable current queries | 75/80 = 0.9375 | 0.870512 | Overall gates pass |
| EN → EN | 24/24 = 1.0000 | 1.000000 | Pass |
| TR → TR | 24/24 = 1.0000 | 1.000000 | Pass |
| EN → TR | 13/16 = 0.8125 | 0.632701 | Both direction gates fail |
| TR → EN | 14/16 = 0.8750 | 0.719861 | Direction nDCG gate fails |
| Exact identifier/path subset, top 1 | 15/18 | — | Required 18/18; fails |
| Complete unresolved evidence, top 5 | 8/8 | — | Pass |
| Exact historical identity/provenance | 2/2 | — | Pass |

The five missing top-five answers were q050 (payment callback authenticity,
correct evidence rank 9), q058 (leave-letter template, rank 6), q062 (PDF password
retention in logs, rank 7), q075 (paragraph identity after insertion, rank 7) and
q076 (Unicode comment offset units, rank 13). The exact identifier/path top-one
failures were q052 (Turkish CSV path, rank 3), q058 (template path, rank 6) and
q066 (queue-cancellation callable, rank 3). These remain failures even though
the gold records existed deeper in the candidate list. The full report preserves
which confounders ranked ahead of each answer.

All three pipelines had **zero detected eligibility violations and zero
critical-coverage errors** across this run's normal and constrained-budget
queries. This includes scope, canonical-head identity, applicability, serialized
count/byte limits and observable missing/unknown critical coverage. It does not
replace the separate lifecycle/index-staleness tests. Same-language hybrid
answers all ranked first, but that result on short public synthetic facts is not
evidence of unseen multilingual generalization.

The literal-whitespace AND lexical baseline returned no query-ranked results
for these natural-language questions: 0/80 answerable Recall@5, 0/18 exact-subset
top-one hits and 0/8 unresolved full coverage. Mandatory critical records were
still returned correctly, and were deliberately not credited as lexical ranking.
Semantic-only and hybrid rankings were identical here because there were no
lexical candidates to fuse. This is a measured limitation of the frozen pipeline
and query interface, not a reason to silently change lexical tokenization or
reduce the thresholds. The report remains the first frozen comparison.

All 32 missing-evidence queries had semantic/hybrid nearest candidates and none
had lexical candidates. No support/abstention stage exists in this evaluation,
so those candidates are not automatically counted as false answers or accepted
as evidence. No abstention gate, generated-answer ambiguity gate or negative
answerability claim was established.

The verified payload check took 1.746 seconds; model loading took 6.479 seconds.
Indexing 70 canonical heads from the 73 revision entries took 33.059 seconds.
For the 80 answerable queries, hybrid warm component-sum p50 was 250.720 ms and
p95 was 255.764 ms; semantic-only p95 was 255.695 ms, lexical p95 was 0.893 ms.
These are the documented fixed-order, shared-embedding, debug-build measurements
on this tiny corpus, not the 10,000-record/native-client performance gate.

`/usr/bin/time -lp` measured 76.44 seconds elapsed, 78.19 user CPU seconds and
1.77 system CPU seconds for the test process, with maximum RSS 1,309,179,904 bytes
and peak memory footprint 976,815,760 bytes. RSS after model drop and release-build
performance were not measured. The qualified host is macOS 26.5.2 ARM64, Apple
M1 Pro with 16 GiB RAM, as recorded by the prior native qualification. No `.ses`
file was present directly in the repository working directory or report directory
after this run; that narrow inspection is not a complete filesystem-write audit.

Hybrid semantic memory has **not passed this qualification**. Fixes or new
pipeline experiments require their own documented change and new frozen runner
review, while retaining this failed report and the unchanged accepted gates.

## Second frozen comparison: OR/BM25 hybrid also failed

Root ran the next explicitly reviewed comparison after changing lexical candidate
generation from all-term AND to quoted OR/BM25, with no stopword dictionary,
translation, weight change or corpus edit. The same runner, corpus, candidate20,
equal RRF60 and accepted gates were retained. Memory source was pinned before
that run to SHA-256
`a12dfd7bd0387a7549ba7b24feacdc66ee6df1ad12c37712e889bb6d8a3e57a0`.
The report is retained at
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/memory-retrieval-or-20261024T185252Z/report.json`,
SHA-256 `9feacdb46d6d96b4a3ff780e8408780561b816fdafea6ec905eacbcaed7af676`.
Its test-executable SHA-256 is
`e2e1811cea9813ac83e6b7fc8a35bf59c70520683cc05c4842d216cdf85f9b55`.

| Pipeline | Answerable Recall@5 | nDCG@5 | Identifier/path top 1 | Complete unresolved evidence |
| --- | ---: | ---: | ---: | ---: |
| OR lexical | 53/80 = 0.6625 | 0.613789 | 10/18 | 5/8 |
| Semantic-only | 75/80 = 0.9375 | 0.870512 | 15/18 | 8/8 |
| OR hybrid | 71/80 = 0.8875 | 0.802703 | 11/18 | 7/8 |

Hybrid failed six reported gate groups: overall recall, overall nDCG, both
cross-language direction groups, exact identifier/path top-one and unresolved
complete coverage. Its EN→EN recall was 23/24 with nDCG 0.927577; TR→TR was 24/24
and 0.984622; EN→TR was 11/16 and 0.472600; TR→EN was 13/16 and 0.672616. All three
pipelines again had zero reported eligibility and critical-coverage errors.

OR made lexical candidate retrieval useful for some natural-language questions,
but **did not improve hybrid qualification**: weak same-language word matches
received full reciprocal-rank votes and degraded cross-language ordering. The
semantic-only metrics remained unchanged. This comparison is evidence against
blindly fusing every lexical match, not permission to lower the gates, choose
per-query weights or silently drop difficult cases. Both failed reports remain
part of the qualification record; semantic memory is still unqualified.

## Frozen confidence-guard comparison: small and base both fail

The independently reviewed retrieval implementation was frozen at `memory.rs`
SHA-256 `90834a3001cefe3505457a75a68b5e1f040f8ab9e166551074f4e0899cf104e8`.
A disposable workspace compared the admitted E5-small INT8 with official
E5-base INT8 at revision `d128750597153bb5987e10b1c3493a34e5a4502a`.
The runner, corpus, gates, candidate limit and fusion parameters remained unchanged.
Both runs used unoptimized Rust with debug symbols and incremental compilation
disabled, sequential fresh processes, denied network access and
`ORT_DISABLE_TELEMETRY=1` before native initialization. Production remained unchanged.

| Hybrid metric | Small INT8 | Base INT8 |
| --- | ---: | ---: |
| Overall Recall@5 / nDCG@5 | .9375 / .870512 | .9625 / .927629 |
| EN→TR Recall@5 / nDCG@5 | .8125 / .632701 | .9375 / .891366 |
| TR→EN Recall@5 / nDCG@5 | .8750 / .719861 | .8750 / .778029 |
| Identifier/path answer top one | 15/18 | 15/18 |
| Complete unresolved evidence | 8/8 | 8/8 |
| Eligibility / critical-coverage errors | 0 / 0 | 0 / 0 |
| Warm positive-query p95 | 230.107 ms | 234.735 ms |
| Whole-process maximum RSS | 1,281,654,784 bytes | 1,593,163,776 bytes |

Small fails both cross-language direction groups and identifier top-one. Base
fails Turkish→English nDCG and identifier top-one. Base improves q052 but introduces
an identifier regression at q029 (rank 1→3); q058 improves 6→2 and q066 worsens 3→5.
Base still misses the top five on q050 (rank 10), q075 (6) and q076 (8). These are
retained failures, not grounds to change gold or accepted thresholds.

Base verification/load took 2.027/7.366 seconds versus small's 1.098/4.632 seconds.
Indexing 70 heads took 29.897/28.941 seconds respectively. These are tiny-corpus,
fixed-order component measurements, not the 10k/100k scope, two-heavy-job, idle
reclamation or native Windows gates. Both models returned candidates for all 32
missing-information queries; no answerability or downstream decision stage was
measured. Base remains **unadopted**, and semantic acceptance remains open.

The immutable reports are under
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/dependencies/semantic/alternative/`:
`paired-small/report.json` SHA-256
`7693367135bc9090d94aa3bad5d77b50ebf7315dfc8c651f1e57b0669c57d5a5`,
and `paired-base-corrected-bound/report.json` SHA-256
`c8a0cdcf32534e6b5177a1f2682aa68d3ea7d2ec69c312bc9fcbe8cc3ace30a0`.
`paired-comparison.md` and `.json` retain full metrics, exact misses, native cost
and source/executable provenance. A separate initial base attempt was retained:
it stopped before loading ORT because the isolated verifier still had the small
model's 128 MiB file ceiling. Only that candidate ceiling was corrected to the exact
verified 278,686,411-byte model size before its first actual inference run; no
retrieval tuning occurred. No further model campaign followed this comparison.


## Bounded reranker cost screen: not adopted

A separate frozen cost probe evaluated
`cross-encoder/mmarco-mMiniLMv2-L12-H384-v1` at revision
`1427fd652930e4ba29e8149678df786c240d8825`, using its ARM64 INT8 graph,
the admitted ONNX Runtime, two CPU threads and five batches of four pairs.
Twenty warm rounds per padded length retained complete, untruncated pairs.
On the same 16 GiB M1 Pro, full-response p95 was **1,197 / 2,368 / 7,393 ms**
for widths 128 / 256 / 512, failing the frozen less-than-one-second screen.
The warm leaf reported an approximately **5.98 GiB** RSS high-water mark.

This standalone probe reused development-profile Rust dependencies. Its failed
cost screen does not establish optimized-release impossibility, native-only
inference timing, concurrent two-job fit or 10k-record retrieval performance.
All 66 completed responses had exactly twenty finite ordered scores; that is
protocol evidence, not relevance quality. Cancellation and parent EOF reaped
the owned child. The EOF-only exercise also emitted an ONNX Cast diagnostic;
its timing does not establish whether it preceded or resulted from shutdown.

Evidence is retained in the implementation task state under
`dependencies/semantic/reranker-research/probe/cost-01/`, including the frozen
report, child observations and independent interpretation. This candidate is
not in production. At the time of this cost screen, the earlier E5 retrieval gates remained unchanged.
The subsequent selected MiniLM result is recorded below.


## Selected MiniLM result, 2026-10-25

The production-source MiniLM ARM64 payload passed all frozen observable gates on
the same 122-query corpus, without changing its fixtures or thresholds. Hybrid
Recall@5 was 0.9875 and nDCG@5 was 0.9448; identifier/path top-1 was 18/18,
ambiguous evidence complete@5 was 8/8, and eligibility/coverage errors were
zero. [The current qualification](m2-semantic-qualification.md) binds the exact
model, runner, package and measured memory cost. This supersedes the failed E5
candidate for the macOS ARM64 private runtime. Unseen generalization,
abstention and scale remain separate gates.
