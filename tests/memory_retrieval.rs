//! Manual, fresh-process qualification. Never loaded by MCP or an ordinary test run.
//! See docs/memory-evaluation-contract.md; do not tune this runner on its scores.
use mirket::{
    embedding::{EmbeddingModel, VerifiedEmbeddingPayload},
    memory::{
        MemoryStore, RecordInput, RecordRef, Scope, SearchContext, SearchRequest, SearchResult,
        StatusFilter,
    },
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::{Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const CORPUS: &str = include_str!("fixtures/memory-retrieval.json");
const CORPUS_SHA256: &str = "cc56a0141c41fec880bd8b528d3cd81c496e193719d5ec1dc6687bd6751d3a4c";
const RUNNER: &str = include_str!("memory_retrieval.rs");
const RUNTIME_PINS: &str = include_str!("../packaging/admission/m2-semantic-minilm.runtime.json");
const GATES: &str = r#"{"recall5_overall":0.90,"recall5_direction":0.85,"ndcg5_overall":0.85,"ndcg5_direction":0.80,"identifier_path_top1":18,"ambiguous_complete5":8,"history_exact":2,"illegal_results":0,"critical_coverage_errors":0,"false_support_max_if_observable":1,"negative_count":32,"invented_resolution_max_if_observable":0,"rrf_k":60,"fusion_weights":[1,1],"candidate_limit":20}"#;

#[derive(Clone, Deserialize)]
struct FixtureRecord {
    id: String,
    record_id: String,
    revision: u64,
    scope: Scope,
    is_current_revision: bool,
    supersedes: Option<RecordRef>,
    #[serde(flatten)]
    fields: BTreeMap<String, Value>,
}
impl FixtureRecord {
    fn input(&self) -> Result<RecordInput> {
        let names = [
            "kind",
            "basis",
            "content",
            "sources",
            "status",
            "critical",
            "applicability",
            "valid_from",
            "valid_until",
        ];
        let object = names
            .into_iter()
            .map(|key| {
                self.fields
                    .get(key)
                    .cloned()
                    .map(|v| (key.to_owned(), v))
                    .ok_or_else(|| format!("{} missing {key}", self.id))
            })
            .collect::<std::result::Result<serde_json::Map<String, Value>, _>>()?;
        Ok(serde_json::from_value(Value::Object(object))?)
    }
}
#[derive(Clone, Deserialize)]
struct Gold {
    record_revision_id: String,
    grade: u32,
}
#[derive(Clone, Deserialize)]
struct Query {
    id: String,
    route: String,
    direction: String,
    scope: Scope,
    query: String,
    context: SearchContext,
    status_filter: StatusFilter,
    limit: usize,
    max_bytes: usize,
    expected_outcome: String,
    gold: Vec<Gold>,
    tags: Vec<String>,
    expected_critical_revision_ids: Vec<String>,
    expected_unknown_context_critical_revision_ids: Vec<String>,
}
impl Query {
    fn request(&self) -> SearchRequest {
        SearchRequest {
            query: self.query.clone(),
            status: self.status_filter,
            context: self.context.clone(),
            limit: self.limit,
            max_bytes: self.max_bytes,
        }
    }
}
#[derive(Deserialize)]
struct Corpus {
    records: Vec<FixtureRecord>,
    queries: Vec<Query>,
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn scope_key(scope: &Scope) -> String {
    serde_json::to_string(scope).expect("scope serialization")
}
fn reference_key(reference: &RecordRef) -> String {
    format!("{}:{}", reference.record_id, reference.revision)
}
fn require(condition: bool, message: impl Into<String>) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into().into())
    }
}
fn explicit_path(name: &str) -> Result<PathBuf> {
    let path = PathBuf::from(env::var_os(name).ok_or_else(|| format!("explicit {name} required"))?);
    require(path.is_absolute(), format!("{name} must be absolute"))?;
    Ok(path)
}
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Result<Self> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = env::temp_dir()
            .canonicalize()?
            .join(format!("mirket-retrieval-{}-{nonce}", std::process::id()));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&path)?;
        Ok(Self(path))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn save_report(file: &mut fs::File, report: &Value) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(report)?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(&bytes)?;
    file.write_all(b"\n")?;
    file.set_len(bytes.len() as u64 + 1)?;
    file.sync_all()?;
    Ok(())
}

struct Seeded {
    stores: BTreeMap<String, MemoryStore>,
    references: BTreeMap<String, RecordRef>,
    reverse: BTreeMap<String, String>,
    records: BTreeMap<String, FixtureRecord>,
}
fn seed(corpus: &Corpus, state: &Path) -> Result<Seeded> {
    let mut seeded = Seeded {
        stores: BTreeMap::new(),
        references: BTreeMap::new(),
        reverse: BTreeMap::new(),
        records: BTreeMap::new(),
    };
    let mut lineages: BTreeMap<(String, String), RecordRef> = BTreeMap::new();
    for (i, fixture) in corpus.records.iter().enumerate() {
        let scope = scope_key(&fixture.scope);
        if !seeded.stores.contains_key(&scope) {
            seeded.stores.insert(
                scope.clone(),
                MemoryStore::open(state, fixture.scope.clone())?,
            );
        }
        let data = fixture.input()?;
        let store = seeded.stores.get_mut(&scope).unwrap();
        let lineage = (scope.clone(), fixture.record_id.clone());
        let operation_id = format!("op-retrieval-fixture-{i:04}");
        let record = if let Some(previous) = lineages.get(&lineage) {
            require(
                fixture.revision == previous.revision + 1,
                "fixture revision gap",
            )?;
            require(
                fixture.supersedes.as_ref().is_some_and(|r| {
                    r.record_id == fixture.record_id && r.revision == previous.revision
                }),
                "revision lacks exact supersession reference",
            )?;
            store.revise(&operation_id, &previous.record_id, previous.revision, data)?
        } else {
            require(
                fixture.revision == 1,
                "new fixture lineage must start at revision one",
            )?;
            let supersedes = fixture
                .supersedes
                .as_ref()
                .map(|r| {
                    let previous = lineages
                        .get(&(scope.clone(), r.record_id.clone()))
                        .ok_or("superseded lineage missing")?;
                    require(
                        previous.revision == r.revision,
                        "superseded revision mismatch",
                    )?;
                    Ok::<_, Box<dyn std::error::Error>>(previous.clone())
                })
                .transpose()?;
            store.create(&operation_id, data, supersedes)?
        };
        let reference = RecordRef {
            record_id: record.record_id,
            revision: record.revision,
        };
        require(
            seeded
                .reverse
                .insert(reference_key(&reference), fixture.id.clone())
                .is_none(),
            "runtime reference duplicated",
        )?;
        require(
            seeded
                .references
                .insert(fixture.id.clone(), reference.clone())
                .is_none(),
            "fixture revision duplicated",
        )?;
        seeded.records.insert(fixture.id.clone(), fixture.clone());
        lineages.insert(lineage, reference);
    }
    // Verify saved canonical state, not merely mutation responses.
    for fixture in &corpus.records {
        let reference = &seeded.references[&fixture.id];
        let actual = seeded
            .stores
            .get_mut(&scope_key(&fixture.scope))
            .unwrap()
            .read(&reference.record_id, Some(reference.revision))?;
        require(
            actual.data == fixture.input()?
                && actual.is_current_revision == fixture.is_current_revision,
            format!("seed differs at {}", fixture.id),
        )?;
    }
    Ok(seeded)
}
fn eligible(record: &FixtureRecord, query: &Query) -> Result<bool> {
    let input = record.input()?;
    let at = query
        .context
        .at_time
        .ok_or("fixture requires frozen evaluation time")?;
    let status = match query.status_filter {
        StatusFilter::Both => true,
        StatusFilter::Current => matches!(input.status, mirket::memory::RecordStatus::Current),
        StatusFilter::Unresolved => {
            matches!(input.status, mirket::memory::RecordStatus::Unresolved)
        }
    };
    let pairs = [
        (&input.applicability.task_key, &query.context.task_key),
        (&input.applicability.checkout_id, &query.context.checkout_id),
        (&input.applicability.operation, &query.context.operation),
    ];
    Ok(record.scope == query.scope
        && record.is_current_revision
        && status
        && input.valid_from.is_none_or(|v| at >= v)
        && input.valid_until.is_none_or(|v| at < v)
        && pairs
            .iter()
            .all(|(needed, actual)| needed.is_none() || needed == actual))
}
fn metrics(query: &Query, ranked: &[String]) -> Value {
    let gold: BTreeMap<&str, u32> = query
        .gold
        .iter()
        .map(|g| (g.record_revision_id.as_str(), g.grade))
        .collect();
    let hits = |k| {
        ranked
            .iter()
            .take(k)
            .filter(|id| gold.contains_key(id.as_str()))
            .count()
    };
    let recall = |k| {
        if gold.is_empty() {
            0.0
        } else {
            hits(k) as f64 / gold.len() as f64
        }
    };
    let dcg: f64 = ranked
        .iter()
        .take(5)
        .enumerate()
        .map(|(i, id)| {
            ((2u32.pow(*gold.get(id.as_str()).unwrap_or(&0)) - 1) as f64) / ((i + 2) as f64).log2()
        })
        .sum();
    let mut ideal: Vec<u32> = gold.values().copied().collect();
    ideal.sort_by(|a, b| b.cmp(a));
    let idcg: f64 = ideal
        .iter()
        .take(5)
        .enumerate()
        .map(|(i, g)| ((2u32.pow(*g) - 1) as f64) / ((i + 2) as f64).log2())
        .sum();
    json!({"recall1":recall(1),"recall3":recall(3),"recall5":recall(5),"ndcg5":if idcg == 0.0 {0.0} else {dcg/idcg},"mrr":ranked.iter().position(|id| gold.contains_key(id.as_str())).map_or(0.0, |i| 1.0/(i+1) as f64),"all_gold_at5":!gold.is_empty() && hits(5)==gold.len(),"relevant_top1":hits(1)>0})
}

fn inspect(query: &Query, result: &SearchResult, seeded: &Seeded) -> Result<Value> {
    let mut violations = Vec::new();
    if result.scope != query.scope {
        violations.push("response scope differs".to_owned());
    }
    if result.ranking.len() > 20 {
        violations.push("ranked metadata exceeds candidate20".to_owned());
    }
    let mut seen = BTreeSet::new();
    let mut ranked = Vec::new();
    let mut ranked_evidence = Vec::new();
    for candidate in &result.ranking {
        let key = reference_key(&candidate.reference);
        let id = seeded
            .reverse
            .get(&key)
            .ok_or("unmapped ranked reference")?;
        if !seen.insert(id) {
            violations.push(format!("duplicate ranked revision {id}"));
        }
        let valid = eligible(&seeded.records[id], query)?;
        if !valid {
            violations.push(format!("ineligible ranked revision {id}"));
        }
        if !candidate.score.is_finite() {
            violations.push(format!("nonfinite score {id}"));
        }
        ranked.push(id.clone());
        ranked_evidence
            .push(json!({"fixture_revision_id":id,"score":candidate.score,"eligible":valid}));
    }
    let mut merged = Vec::new();
    let mut critical = BTreeSet::new();
    let mut merged_seen = BTreeSet::new();
    for hit in &result.hits {
        let reference = RecordRef {
            record_id: hit.record.record_id.clone(),
            revision: hit.record.revision,
        };
        let id = seeded
            .reverse
            .get(&reference_key(&reference))
            .ok_or("unmapped merged reference")?;
        if !merged_seen.insert(id) {
            violations.push(format!("duplicate merged revision {id}"));
        }
        if !eligible(&seeded.records[id], query)? {
            violations.push(format!("ineligible merged revision {id}"));
        }
        if hit.record.data != seeded.records[id].input()? || !hit.record.is_current_revision {
            violations.push(format!("canonical body/head mismatch {id}"));
        }
        if matches!(hit.reason, mirket::memory::MatchReason::CriticalConstraint) {
            critical.insert(id.clone());
        }
        merged.push(id.clone());
    }
    let expected: BTreeSet<_> = query
        .expected_critical_revision_ids
        .iter()
        .cloned()
        .collect();
    let missing: Vec<_> = expected.difference(&critical).cloned().collect();
    let unexpected: Vec<_> = critical.difference(&expected).cloned().collect();
    let mut coverage_errors = Vec::new();
    if result.coverage.applicable_current_critical != expected.len() as u64 {
        coverage_errors.push("applicable count mismatch");
    }
    if result.coverage.unclassified_current_critical
        != query.expected_unknown_context_critical_revision_ids.len() as u64
    {
        coverage_errors.push("unknown-context count mismatch");
    }
    if result.coverage.returned_critical != critical.len() {
        coverage_errors.push("returned critical count mismatch");
    }
    if !unexpected.is_empty() {
        coverage_errors.push("unrequested/inapplicable critical insertion");
    }
    if result.coverage.critical_overflow != !missing.is_empty() {
        coverage_errors.push("critical overflow mismatch");
    }
    if result.hits.len() > query.limit || serde_json::to_vec(result)?.len() > query.max_bytes {
        violations.push("merged response exceeds budget".into());
    }
    Ok(
        json!({"ranking":ranked_evidence,"merged_fixture_ids":merged,"mandatory_critical_fixture_ids":critical,"missing_critical_fixture_ids":missing,"coverage":result.coverage,"eligibility_violations":violations,"coverage_errors":coverage_errors,"metrics":metrics(query,&ranked)}),
    )
}
fn percentile(values: &mut [f64], fraction: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    Some(
        values[((values.len() as f64 * fraction).ceil() as usize)
            .saturating_sub(1)
            .min(values.len() - 1)],
    )
}
fn wilson95(hits: usize, count: usize) -> Value {
    if count == 0 {
        return Value::Null;
    }
    let n = count as f64;
    let p = hits as f64 / n;
    let z: f64 = 1.959963984540054;
    let denominator = 1.0 + z * z / n;
    let center = (p + z * z / (2.0 * n)) / denominator;
    let margin = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt() / denominator;
    json!({"low":center-margin,"high":center+margin})
}
fn clustered_interval(rows: &[&Value], metric: &str) -> Value {
    let mut clusters: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for row in rows {
        // Current answerable queries have one gold canonical head. Repeated
        // language questions about that lineage share a bootstrap cluster.
        let key = row["gold"][0]["fixture_revision_id"]
            .as_str()
            .unwrap()
            .to_owned();
        clusters
            .entry(key)
            .or_default()
            .push(row["result"]["metrics"][metric].as_f64().unwrap());
    }
    let groups: Vec<_> = clusters.values().collect();
    if groups.is_empty() {
        return Value::Null;
    }
    let mut state = 42u64;
    let mut means = Vec::with_capacity(2000);
    for _ in 0..2000 {
        let mut sum = 0.0;
        let mut count = 0;
        for _ in 0..groups.len() {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let group = groups[(state >> 32) as usize % groups.len()];
            sum += group.iter().sum::<f64>();
            count += group.len();
        }
        means.push(sum / count as f64);
    }
    json!({"low":percentile(&mut means,0.025),"high":percentile(&mut means,0.975),"clusters":groups.len(),"replicates":2000,"seed":42})
}
fn aggregate(rows: &[Value], pipeline: &str, direction: Option<&str>) -> Value {
    let selected: Vec<_> = rows
        .iter()
        .filter(|r| {
            r["pipeline"] == pipeline
                && r["outcome"] == "answerable"
                && r["variant"] == "normal"
                && direction.is_none_or(|d| r["direction"] == d)
        })
        .collect();
    let mean = |key: &str| {
        if selected.is_empty() {
            0.0
        } else {
            selected
                .iter()
                .map(|r| r["result"]["metrics"][key].as_f64().unwrap())
                .sum::<f64>()
                / selected.len() as f64
        }
    };
    let mut latencies: Vec<_> = selected
        .iter()
        .map(|r| r["warm_end_to_end_ms"].as_f64().unwrap())
        .collect();
    json!({"count":selected.len(),"recall1":mean("recall1"),"recall3":mean("recall3"),"recall5":mean("recall5"),"ndcg5":mean("ndcg5"),"mrr":mean("mrr"),"recall5_cluster_bootstrap95":clustered_interval(&selected,"recall5"),"ndcg5_cluster_bootstrap95":clustered_interval(&selected,"ndcg5"),"warm_end_to_end_p50_ms":percentile(&mut latencies,0.50),"warm_end_to_end_p95_ms":percentile(&mut latencies,0.95)})
}
fn evaluate_gates(rows: &[Value], history: usize) -> (Value, Vec<String>) {
    let mut summary = serde_json::Map::new();
    let mut failures = Vec::new();
    if history != 2 {
        failures.push(format!("exact history {history}/2"));
    }
    for pipeline in ["lexical", "semantic", "hybrid"] {
        let quality_required = pipeline == "hybrid";
        let overall = aggregate(rows, pipeline, None);
        let directions: BTreeMap<_, _> = ["en_en", "tr_tr", "en_tr", "tr_en"]
            .into_iter()
            .map(|d| (d, aggregate(rows, pipeline, Some(d))))
            .collect();
        if quality_required {
            if overall["recall5"].as_f64().unwrap() < 0.90 {
                failures.push(format!("{pipeline}: Recall@5 below .90"));
            }
            if overall["ndcg5"].as_f64().unwrap() < 0.85 {
                failures.push(format!("{pipeline}: nDCG@5 below .85"));
            }
            for (direction, scores) in &directions {
                if scores["recall5"].as_f64().unwrap() < 0.85
                    || scores["ndcg5"].as_f64().unwrap() < 0.80
                {
                    failures.push(format!("{pipeline}/{direction}: directional quality gate"));
                }
            }
        }
        let selected: Vec<_> = rows.iter().filter(|r| r["pipeline"] == pipeline).collect();
        let identifier_hits = selected
            .iter()
            .filter(|r| {
                r["variant"] == "normal"
                    && r["exact_identifier_path"] == true
                    && r["outcome"] == "answerable"
                    && r["result"]["metrics"]["relevant_top1"] == true
            })
            .count();
        let ambiguous_complete = selected
            .iter()
            .filter(|r| {
                r["variant"] == "normal"
                    && r["outcome"] == "ambiguous"
                    && r["result"]["metrics"]["all_gold_at5"] == true
            })
            .count();
        let illegal: usize = selected
            .iter()
            .map(|r| {
                r["result"]["eligibility_violations"]
                    .as_array()
                    .unwrap()
                    .len()
            })
            .sum();
        let coverage: usize = selected
            .iter()
            .map(|r| r["result"]["coverage_errors"].as_array().unwrap().len())
            .sum();
        if quality_required && identifier_hits != 18 {
            failures.push(format!(
                "{pipeline}: identifier/path top1 {identifier_hits}/18"
            ));
        }
        if quality_required && ambiguous_complete != 8 {
            failures.push(format!(
                "{pipeline}: unresolved evidence {ambiguous_complete}/8"
            ));
        }
        if illegal != 0 || coverage != 0 {
            failures.push(format!(
                "{pipeline}: {illegal} eligibility and {coverage} coverage errors"
            ));
        }
        summary.insert(pipeline.into(),json!({"quality_gates_required":quality_required,"overall":overall,"directions":directions,"identifier_path_top1":{"hits":identifier_hits,"total":18,"wilson95":wilson95(identifier_hits,18)},"unresolved_complete5":{"hits":ambiguous_complete,"total":8,"wilson95":wilson95(ambiguous_complete,8)},"eligibility_errors":illegal,"coverage_errors":coverage}));
    }
    (Value::Object(summary), failures)
}

#[test]
#[ignore = "manual: explicit verified model package, new report path, frozen runner and separate process admission required"]
fn qualify_synthetic_memory_retrieval() -> Result<()> {
    require(
        env::var("ORT_DISABLE_TELEMETRY").as_deref() == Ok("1"),
        "ORT_DISABLE_TELEMETRY=1 is required before the first native runtime load",
    )?;
    require(
        env::var("MIRKET_RETRIEVAL_EXECUTE").as_deref() == Ok("frozen-qualified-run"),
        "set explicit execution marker only after root authorizes the frozen runner",
    )?;
    require(
        digest(CORPUS.as_bytes()) == CORPUS_SHA256,
        "frozen corpus changed",
    )?;
    let runner_sha = digest(RUNNER.as_bytes());
    require(
        env::var("MIRKET_RETRIEVAL_RUNNER_SHA256").as_deref() == Ok(runner_sha.as_str()),
        "runner digest must match explicitly reviewed source",
    )?;
    let package = explicit_path("MIRKET_TEST_EMBEDDING_PACKAGE")?;
    let report_path = explicit_path("MIRKET_RETRIEVAL_REPORT")?;
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut report_file = options.open(&report_path)?;
    let gates: Value = serde_json::from_str(GATES)?;
    let mut report = json!({"schema":1,"status":"running","corpus_sha256":CORPUS_SHA256,"runner_sha256":runner_sha,"gates_sha256":digest(GATES.as_bytes()),"gates":gates,"process_id":std::process::id(),"os":env::consts::OS,"architecture":env::consts::ARCH,"test_executable_sha256":digest(&fs::read(env::current_exe()?)?),"package":package,"runtime_pins_sha256":digest(RUNTIME_PINS.as_bytes()),"runtime_pins":serde_json::from_str::<Value>(RUNTIME_PINS)?,"timing_method":"Sequential lexical/semantic/hybrid search; one query embedding shared, charged in full to each semantic pipeline; model already warmed by indexing; OS/SQLite caches are not reset between pipelines.","pipeline_parameters":{"rrf_k":60,"equal_weights":true,"candidate_limit":20,"query_limit":5,"query_bytes":32768},"independent_review":{"reviewer":"root","reviewed_query_count":25,"annotation_defects_found":0},"unmeasured":["abstention/false support","generated answer ambiguity","unseen generalization","10k/100k scale","peak/idle RSS","native desktop activation","power-loss durability"],"queries":[],"history":[]});
    save_report(&mut report_file, &report)?;
    let outcome = (|| -> Result<()> {
        let corpus: Corpus = serde_json::from_str(CORPUS)?;
        require(
            corpus.records.len() == 73 && corpus.queries.len() == 122,
            "fixture cardinality changed",
        )?;
        let scratch = Scratch::new()?;
        let mut seeded = seed(&corpus, &scratch.0)?;
        let started = Instant::now();
        let payload = VerifiedEmbeddingPayload::open(&package)?;
        report["payload_verification_ms"] = json!(started.elapsed().as_secs_f64() * 1000.0);
        let started = Instant::now();
        let mut model = EmbeddingModel::load(payload)?; // Exactly one model/session load in this process.
        report["cold_model_load_ms"] = json!(started.elapsed().as_secs_f64() * 1000.0);
        report["model_identity"] = serde_json::to_value(model.model_identity())?;
        let cancel = AtomicBool::new(false);
        let started = Instant::now();
        let mut indexed = 0;
        for store in seeded.stores.values_mut() {
            let mut cursor = None;
            loop {
                let batch = store.select_index_batch(cursor, 32, 256 * 1024)?;
                require(
                    batch.items.len() <= 32
                        && batch.items.iter().map(|i| i.content.len()).sum::<usize>() <= 256 * 1024,
                    "index batch exceeds bound",
                )?;
                let texts: Vec<&str> = batch.items.iter().map(|i| i.content.as_str()).collect();
                if !texts.is_empty() {
                    let passages = model.embed_passages(&texts, &cancel)?;
                    require(
                        passages.len() == batch.items.len(),
                        "embedding result count differs",
                    )?;
                    for (item, passage) in batch.items.iter().zip(&passages) {
                        require(
                            mirket::memory::index::source_digest(&item.content)
                                == item.source_sha256,
                            "selection digest differs",
                        )?;
                        store.apply_index(&item.reference, &item.source_sha256, passage)?;
                        indexed += 1;
                    }
                }
                if !batch.has_more {
                    break;
                }
                require(
                    batch.next_after_row_id.is_some() && batch.next_after_row_id != cursor,
                    "index cursor stalled",
                )?;
                cursor = batch.next_after_row_id;
            }
            require(
                store.index_coverage()?.missing_heads == 0,
                "index coverage incomplete",
            )?;
        }
        report["indexing"] = json!({"heads":indexed,"elapsed_ms":started.elapsed().as_secs_f64()*1000.0,"batch_max_records":32,"batch_max_bytes":256*1024});
        save_report(&mut report_file, &report)?;
        for query in &corpus.queries {
            if query.route == "exact_history" {
                for gold in &query.gold {
                    let reference = &seeded.references[&gold.record_revision_id];
                    let actual = seeded
                        .stores
                        .get_mut(&scope_key(&query.scope))
                        .unwrap()
                        .read(&reference.record_id, Some(reference.revision))?;
                    require(
                        actual.data == seeded.records[&gold.record_revision_id].input()?
                            && !actual.is_current_revision,
                        "exact historical evidence differs",
                    )?;
                }
                report["history"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"query_id":query.id,"passed":true}));
                continue;
            }
            let started = Instant::now();
            let embedding = model.embed_query(&query.query, &cancel)?;
            let embed_ms = started.elapsed().as_secs_f64() * 1000.0;
            for pipeline in ["lexical", "semantic", "hybrid"] {
                let started = Instant::now();
                let store = seeded.stores.get_mut(&scope_key(&query.scope)).unwrap();
                let result = match pipeline {
                    "lexical" => store.search(query.request())?,
                    "semantic" => store.search_semantic(query.request(), &embedding)?,
                    _ => store.search_hybrid(query.request(), &embedding)?,
                };
                let search_ms = started.elapsed().as_secs_f64() * 1000.0;
                let inspected = inspect(query, &result, &seeded)?;
                let embedding_ms = if pipeline == "lexical" { 0.0 } else { embed_ms };
                report["queries"].as_array_mut().unwrap().push(json!({"query_id":query.id,"query":query.query,"scope":query.scope,"context":query.context,"direction":query.direction,"outcome":query.expected_outcome,"gold":query.gold.iter().map(|g|json!({"fixture_revision_id":g.record_revision_id,"grade":g.grade})).collect::<Vec<_>>(),"pipeline":pipeline,"variant":"normal","exact_identifier_path":query.tags.iter().any(|t|t=="identifier" || t=="path"),"embedding_ms":embedding_ms,"search_ms":search_ms,"warm_end_to_end_ms":embedding_ms+search_ms,"result":inspected}));
            }
            // Deterministic small result budgets; ranking quality is scored only on normal rows.
            if query.id == "q073" || query.id == "q001" {
                let mut constrained = query.clone();
                constrained.limit = 1;
                constrained.max_bytes = 1024;
                for pipeline in ["lexical", "semantic", "hybrid"] {
                    let store = seeded.stores.get_mut(&scope_key(&query.scope)).unwrap();
                    let result = match pipeline {
                        "lexical" => store.search(constrained.request())?,
                        "semantic" => store.search_semantic(constrained.request(), &embedding)?,
                        _ => store.search_hybrid(constrained.request(), &embedding)?,
                    };
                    let inspected = inspect(&constrained, &result, &seeded)?;
                    report["queries"].as_array_mut().unwrap().push(json!({"query_id":query.id,"pipeline":pipeline,"variant":"limit1_bytes1024","outcome":query.expected_outcome,"result":inspected}));
                }
            }
            save_report(&mut report_file, &report)?;
        }
        let (summary, failures) = evaluate_gates(
            report["queries"].as_array().unwrap(),
            report["history"].as_array().unwrap().len(),
        );
        report["summary"] = summary;
        report["gate_failures"] = json!(failures);
        require(
            failures.is_empty(),
            format!(
                "{} qualification gates failed; report retained",
                failures.len()
            ),
        )?;
        Ok(())
    })();
    report["status"] = json!(if outcome.is_ok() {
        "passed_observable_gates"
    } else {
        "failed"
    });
    if let Err(error) = &outcome {
        report["error"] = json!(error.to_string());
    }
    save_report(&mut report_file, &report)?;
    eprintln!("Retrieval qualification report: {}", report_path.display());
    outcome
}

#[test]
fn frozen_fixture_seeds_and_gold_is_canonically_eligible_without_model() -> Result<()> {
    require(
        digest(CORPUS.as_bytes()) == CORPUS_SHA256,
        "frozen corpus changed",
    )?;
    let corpus: Corpus = serde_json::from_str(CORPUS)?;
    let scratch = Scratch::new()?;
    let seeded = seed(&corpus, &scratch.0)?;
    let mut ids = BTreeSet::new();
    let mut outcomes = BTreeMap::new();
    for query in &corpus.queries {
        require(ids.insert(&query.id), "duplicate query ID")?;
        *outcomes
            .entry(query.expected_outcome.as_str())
            .or_insert(0usize) += 1;
        require(
            query.gold.is_empty() == (query.expected_outcome == "no_answer"),
            "missing gold or nonempty negative gold",
        )?;
        for gold in &query.gold {
            let record = seeded
                .records
                .get(&gold.record_revision_id)
                .ok_or("unknown gold revision")?;
            if query.route == "current_recall" {
                require(
                    eligible(record, query)?,
                    format!("ineligible gold {}", query.id),
                )?;
            } else {
                require(
                    record.scope == query.scope && !record.is_current_revision,
                    "historical gold mismatch",
                )?;
            }
        }
    }
    require(
        outcomes.get("answerable") == Some(&82)
            && outcomes.get("no_answer") == Some(&32)
            && outcomes.get("ambiguous") == Some(&8),
        "frozen outcome counts differ",
    )?;
    let ambiguity = corpus.queries.iter().find(|q| q.id == "q113").unwrap();
    let partial = metrics(ambiguity, &[ambiguity.gold[0].record_revision_id.clone()]);
    require(
        partial["recall5"] == 0.5 && partial["all_gold_at5"] == false,
        "partial conflict evidence must not pass complete coverage",
    )?;
    Ok(())
}
