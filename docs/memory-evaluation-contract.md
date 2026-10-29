# Synthetic memory retrieval evaluation contract

The numeric qualification gates were fixed before model scores on 2026-10-24.
Corpus SHA-256 is
`cc56a0141c41fec880bd8b528d3cd81c496e193719d5ec1dc6687bd6751d3a4c`.
An independent review found no annotation defect in 25 selected
gold/context/confounder cases. The first native alternative-model run failed
three hybrid gate groups; the selected MiniLM result appears below. These
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

The following numeric gates were fixed before the first model result. Freeze
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

## Manual evaluator freeze and execution boundary

The evaluator source is `tests/memory_retrieval.rs`, frozen before model execution
with SHA-256
`aab656c4f5c1d2b3b46ad3ba022bd80063262f482c3889f4d91f0da19e65fcc6`.
Its canonical numeric gate JSON has SHA-256
`7da9698ee965dfbaee365eed77d30bd71f4c82fd805ef3ae471c714a073a8dcc`.
The gate JSON is embedded in that source; the runner checks its compiled corpus
digest and requires the externally supplied reviewed runner digest. Subsequent
source changes require a new explicit review/freeze before execution.

Run the ignored evaluator deliberately in a fresh process with an explicitly
verified private package. Do not overlap it with transport tests that own the
same test binary.

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

## Selected runtime and limits

The admitted MiniLM ARM64 payload passed the frozen 122-query corpus with hybrid
Recall@5 0.9875, nDCG@5 0.9448, identifier/path top-1 18/18, ambiguous evidence
complete@5 8/8 and zero eligibility/coverage errors. Exact model inputs are in
`packaging/runtime/semantic.json`. This is bounded corpus
qualification, not unseen generalization, abstention or scale evidence. Failed
alternative-model experiments remain in Git history and private evidence.
