//! Manual, synthetic scale probe for the current SQLite/FTS + Rust cosine path.
//! It uses the public canonical/index APIs and never loads a model or user data.

use mirket::{
    embedding::{
        self, ChunkEmbedding, EmbeddingModel, PassageEmbedding, QueryEmbedding,
        VerifiedEmbeddingPayload,
    },
    memory::{
        self, Applicability, ClaimBasis, MemoryStore, RecordInput, RecordKind, RecordRef,
        RecordStatus, Scope, SearchRequest, Source,
    },
};
use serde_json::json;
use std::{
    env, fs,
    path::PathBuf,
    sync::atomic::AtomicBool,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

#[test]
#[ignore = "manual 10k/100k record performance qualification"]
fn measure_scoped_semantic_recall() -> Result<(), Box<dyn std::error::Error>> {
    let count: usize = env::var("MIRKET_SCALE_COUNT")?.parse()?;
    assert!((1_000..=100_000).contains(&count));
    let report = PathBuf::from(env::var_os("MIRKET_SCALE_REPORT").ok_or("report path required")?);
    assert!(report.is_absolute() && !report.exists());
    let parent = report
        .parent()
        .ok_or("report parent required")?
        .canonicalize()?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let state = parent.join(format!("memory-scale-{}-{nonce}", std::process::id()));
    fs::create_dir(&state)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&state, fs::Permissions::from_mode(0o700))?;
    }
    let run = (|| -> Result<_, Box<dyn std::error::Error>> {
        let mut store = MemoryStore::open(&state, Scope::Project("scale-fixture".into()))?;
        let identity = embedding::expected_model_identity()?;
        let query_text = "observatory telescope battery";
        let cancelled = AtomicBool::new(false);
        let model_package = env::var_os("MIRKET_SCALE_EMBEDDING_PACKAGE");
        let model_started = Instant::now();
        let mut model = if let Some(path) = model_package {
            assert_eq!(env::var("ORT_DISABLE_TELEMETRY")?, "1");
            Some(EmbeddingModel::load(VerifiedEmbeddingPayload::open(
                &PathBuf::from(path),
            )?)?)
        } else {
            None
        };
        let model_load_seconds = model
            .as_ref()
            .map(|_| model_started.elapsed().as_secs_f64());
        let query = if let Some(model) = model.as_mut() {
            model.embed_query(query_text, &cancelled)?
        } else {
            let mut vector = vec![0.0_f32; embedding::DIMENSIONS];
            vector[0] = 1.0;
            QueryEmbedding {
                model_identity: identity.clone(),
                source_sha256: embedding::source_digest(query_text),
                input_bytes: query_text.len(),
                input_tokens: 6,
                vector,
            }
        };
        let target = query.vector.clone();
        let mut other = vec![0.0_f32; embedding::DIMENSIONS];
        let least_aligned = target
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
            .map(|(axis, _)| axis)
            .unwrap();
        other[least_aligned] = 1.0;
        let started = Instant::now();
        let mut seed_end = None;
        let mut expected_record = None;
        for i in 0..count {
            let content = format!("Observatory telescope battery policy entry {i:06}");
            let record = store.create(
                &format!("op-scale-{i:06}"),
                RecordInput {
                    kind: RecordKind::Fact,
                    basis: ClaimBasis::Observed,
                    content: content.clone(),
                    sources: vec![Source {
                        source_id: "fixture:memory-scale".into(),
                        revision: "fixture-1".into(),
                        locator: format!("record:{i}"),
                        captured_at: 123,
                    }],
                    status: RecordStatus::Current,
                    critical: false,
                    applicability: Applicability::default(),
                    valid_from: None,
                    valid_until: None,
                },
                None,
            )?;
            if i == 0 {
                expected_record = Some(record.record_id.clone());
            }
            let vector = if i == 0 { &target } else { &other };
            store.apply_index(
                &RecordRef {
                    record_id: record.record_id,
                    revision: record.revision,
                },
                &memory::index::source_digest(&content),
                &PassageEmbedding {
                    model_identity: identity.clone(),
                    input_bytes: content.len(),
                    content_tokens: 5,
                    covered_bytes: content.len(),
                    complete: true,
                    chunks: vec![ChunkEmbedding {
                        start_byte: 0,
                        end_byte: content.len(),
                        input_tokens: 7,
                        vector: vector.clone(),
                    }],
                },
            )?;
            if i + 1 == count {
                seed_end = Some(started.elapsed());
            }
        }
        let coverage_started = Instant::now();
        let coverage = store.index_coverage()?;
        let coverage_seconds = coverage_started.elapsed().as_secs_f64();
        assert_eq!(coverage.current_heads, count as u64);
        assert_eq!(coverage.indexed_heads, count as u64);
        assert_eq!(coverage.missing_heads, 0);
        let request = SearchRequest {
            query: query_text.into(),
            limit: 5,
            ..SearchRequest::default()
        };
        let mut samples_ms = Vec::new();
        for _ in 0..30 {
            let begin = Instant::now();
            let result = store.search_semantic(request.clone(), &query)?;
            samples_ms.push(begin.elapsed().as_secs_f64() * 1000.0);
            assert_eq!(result.coverage.eligible_heads, count as u64);
            assert_eq!(result.coverage.semantic_indexed_eligible, count as u64);
            assert_eq!(result.coverage.semantic_missing_eligible, 0);
            assert_eq!(
                result.hits[0].record.record_id,
                expected_record.as_ref().unwrap().as_str()
            );
        }
        samples_ms.sort_by(f64::total_cmp);
        let mut lexical_samples_ms = Vec::new();
        for _ in 0..10 {
            let begin = Instant::now();
            let result = store.search(request.clone())?;
            lexical_samples_ms.push(begin.elapsed().as_secs_f64() * 1000.0);
            assert_eq!(result.coverage.eligible_heads, count as u64);
        }
        lexical_samples_ms.sort_by(f64::total_cmp);
        let mut end_to_end_samples_ms = Vec::new();
        if let Some(model) = model.as_mut() {
            for _ in 0..30 {
                let begin = Instant::now();
                let fresh_query = model.embed_query(query_text, &cancelled)?;
                let result = store.search_hybrid(request.clone(), &fresh_query)?;
                end_to_end_samples_ms.push(begin.elapsed().as_secs_f64() * 1000.0);
                assert_eq!(result.coverage.semantic_indexed_eligible, count as u64);
                assert_eq!(
                    result.hits[0].record.record_id,
                    expected_record.as_ref().unwrap().as_str()
                );
            }
            end_to_end_samples_ms.sort_by(f64::total_cmp);
        }
        let database_bytes = fs::read_dir(state.join("memory"))?
            .filter_map(Result::ok)
            .filter_map(|entry| entry.metadata().ok())
            .filter(|meta| meta.is_file())
            .map(|meta| meta.len())
            .sum::<u64>();
        Ok(json!({
            "schema": 1,
            "status": "passed",
            "fixture": "synthetic public-API canonical writes and one checked vector per head",
            "record_count": count,
            "os": env::consts::OS,
            "architecture": env::consts::ARCH,
            "profile": "test (debug)",
            "seed_and_index_seconds": seed_end.unwrap().as_secs_f64(),
            "validated_coverage_seconds": coverage_seconds,
            "model_load_seconds": model_load_seconds,
            "database_bytes": database_bytes,
            "warm_search_samples_ms": samples_ms,
            "warm_search_p50_ms": samples_ms[14],
            "warm_search_p95_ms": samples_ms[28],
            "warm_lexical_samples_ms": lexical_samples_ms,
            "warm_lexical_p95_ms": lexical_samples_ms[9],
            "warm_full_samples_ms": end_to_end_samples_ms,
            "warm_full_p50_ms": end_to_end_samples_ms.get(14),
            "warm_full_p95_ms": end_to_end_samples_ms.get(28),
            "limits": [
                "Synthetic stored vectors; optional real model query inference tests latency, not answer-support quality",
                "One scope, one chunk per head, warm page cache, sequential requests",
                "No client, concurrent writes, backup, pressure or process RSS measurement"
            ]
        }))
    })();
    match run {
        Ok(value) => {
            fs::write(&report, serde_json::to_vec_pretty(&value)?)?;
            fs::remove_dir_all(&state)?;
            Ok(())
        }
        Err(error) => {
            fs::write(
                &report,
                serde_json::to_vec_pretty(&json!({
                    "schema": 1, "status": "failed", "error": error.to_string(),
                    "fixture_state": state,
                }))?,
            )?;
            Err(error)
        }
    }
}
