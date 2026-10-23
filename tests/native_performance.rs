//! Fixed synthetic native-store measurements. This is an ignored gate by
//! design: it records reproducible behavior on the invoking host and never
//! turns a local timing into a claim about the retired runtime.
use mirket::{agent_runtime::journal::Journal, memory::store::FactStore};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn vector(axis: usize) -> Vec<f32> {
    let mut v = vec![0.; 384];
    v[axis % 384] = 1.;
    v
}
fn fact(index: usize) -> Value {
    json!({"key":format!("perf-{index}"),"text":format!("synthetic retrieval needle-{index} stable corpus fact"),"kind":"fact","provenance":"native performance fixture","expires_at":null})
}
fn scope() -> String {
    "c".repeat(64)
}

fn journal_bytes_fixture(dir: &std::path::Path) -> (u64, u64) {
    let path = dir.canonicalize().unwrap().join("execution.sqlite");
    let mut journal = Journal::open(&path, &scope()).unwrap();
    let assignment = "fixed long-run assignment ".repeat(4096);
    let mut state = json!({"turn":0,"assignment":assignment,"observations":[]});
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs();
    let deadline = now + Duration::from_secs(7_000).as_secs();
    let start = journal.execute("workflow_start", &json!({"run_id":"perf-run","envelope":{"checkpoint_format_version":1,"host_verification_required":false,"provider":"fixture"},"snapshot":state,"deadline":deadline}), now, json!({"results":[]})).unwrap();
    let mut current = start;
    let mut full_snapshot_bytes = serde_json::to_vec(&state).unwrap().len() as u64;
    for turn in 1..=96 {
        state["turn"] = json!(turn);
        state["observations"] = json!({"turn":turn,"payload":"fixed observation"});
        full_snapshot_bytes += serde_json::to_vec(&state).unwrap().len() as u64;
        current = journal.execute("workflow_resume", &json!({"run_id":"perf-run","expected_checkpoint_id":current["checkpoint_id"],"key":format!("step-{turn}"),"reply":{"decision":"continue","snapshot":state}}), now + turn as u64, Value::Null).unwrap();
    }
    drop(journal);
    let conn = rusqlite::Connection::open(path).unwrap();
    let native: u64 = conn
        .query_row("SELECT bytes FROM runs WHERE id='perf-run'", [], |row| {
            row.get(0)
        })
        .unwrap();
    (native, full_snapshot_bytes)
}

#[test]
#[ignore = "host-scale performance gate; records native behavior without comparing retired runtime"]
fn native_retrieval_scales_and_checkpoint_bytes_are_measured() {
    let dir = tempfile::tempdir().unwrap();
    let mut measurements = Vec::new();
    for count in [0usize, 100, 1_000, 10_000, 100_000] {
        let started = Instant::now();
        let path = dir
            .path()
            .canonicalize()
            .unwrap()
            .join(format!("facts-{count}.sqlite"));
        let mut store = FactStore::open(&path, &scope(), "fixture-384").unwrap();
        for index in 0..count {
            store
                .put(&fact(index), &vector(index), index as u64 + 100)
                .unwrap();
        }
        let insert_ms = started.elapsed().as_secs_f64() * 1000.;
        let query_indices = if count == 0 {
            vec![0]
        } else {
            vec![0, count / 2, count - 1]
        };
        let query_started = Instant::now();
        for index in &query_indices {
            let result = store
                .search(
                    &format!("needle-{index}"),
                    Some(&vector(*index)),
                    5,
                    count as u64 + 1000,
                )
                .unwrap();
            let items = result["results"].as_array().unwrap();
            if count == 0 {
                assert!(items.is_empty());
            } else {
                assert!(
                    !items.is_empty(),
                    "eligible fact retrieval returned empty at count {count}, query {index}"
                );
                assert!(
                    items.iter().any(|item| item["memory"]
                        == format!("synthetic retrieval needle-{index} stable corpus fact")),
                    "expected eligible fact missing at count {count}, query {index}"
                );
            }
        }
        let single_ms = query_started.elapsed().as_secs_f64() * 1000.;
        drop(store);
        let mut client_measurements = Vec::new();
        for clients in [1usize, 2, 8] {
            let clients_started = Instant::now();
            let shared = Arc::new(path.clone());
            let mut joins = Vec::new();
            for client in 0..clients {
                let path = shared.clone();
                joins.push(thread::spawn(move || {
                    let cold_started = Instant::now();
                    let mut store = FactStore::open(&path, &scope(), "fixture-384").unwrap();
                    let expected = |index: usize, result: &Value| {
                        let items = result["results"].as_array().unwrap();
                        (count == 0 && items.is_empty())
                            || (count > 0
                                && items.iter().any(|item| {
                                    item["memory"]
                                        == format!(
                                            "synthetic retrieval needle-{index} stable corpus fact"
                                        )
                                }))
                    };
                    let first_index = if count == 0 { 0 } else { client % count };
                    let cold = store
                        .search(
                            &format!("needle-{first_index}"),
                            Some(&vector(first_index)),
                            5,
                            count as u64 + 1000,
                        )
                        .unwrap();
                    assert!(
                        expected(first_index, &cold),
                        "cold retrieval must retain eligible facts for {clients} clients at count {count}"
                    );
                    let cold_ms = cold_started.elapsed().as_secs_f64() * 1000.;
                    // Warm the SQLite connection and statement/page cache before
                    // collecting the comparable samples below.
                    for warm in 0..2 {
                        let index = if count == 0 { 0 } else { (client + warm + 1) % count };
                        let result = store
                            .search(
                                &format!("needle-{index}"),
                                Some(&vector(index)),
                                5,
                                count as u64 + 1000,
                            )
                            .unwrap();
                        assert!(expected(index, &result));
                    }
                    let mut samples = Vec::with_capacity(20);
                    for sample in 0..20 {
                        let batch_started = Instant::now();
                        for offset in 0..3 {
                            let index = if count == 0 {
                                0
                            } else {
                                (client + sample * 3 + offset) % count
                            };
                            let result = store
                                .search(
                                    &format!("needle-{index}"),
                                    Some(&vector(index)),
                                    5,
                                    count as u64 + 1000,
                                )
                                .unwrap();
                            assert!(
                                expected(index, &result),
                                "warm retrieval must retain eligible facts for {clients} clients at count {count}, sample {sample}, query {offset}"
                            );
                        }
                        samples.push(batch_started.elapsed().as_secs_f64() * 1000.);
                    }
                    samples.sort_by(f64::total_cmp);
                    let p95 = samples[((samples.len() * 95).saturating_sub(1) / 100)
                        .min(samples.len() - 1)];
                    (cold_ms, p95, samples)
                }));
            }
            let results: Vec<_> = joins.into_iter().map(|join| join.join().unwrap()).collect();
            for (cold_ms, p95, samples) in results {
                client_measurements.push(json!({
                    "clients": clients,
                    "cold_open_query_ms": cold_ms,
                    "warm_query_batch_size": 3,
                    "warm_sample_count": samples.len(),
                    "p95_query_batch_ms": p95,
                    "warm_query_batch_samples_ms": samples,
                }));
            }
            // Keep one record per client cardinality while exposing every
            // client distribution and its aggregate worst p95.
            let group_start = client_measurements.len() - clients;
            let group = &client_measurements[group_start..];
            let aggregate_p95 = group
                .iter()
                .map(|entry| entry["p95_query_batch_ms"].as_f64().unwrap())
                .fold(0., f64::max);
            client_measurements.push(json!({
                "clients": clients,
                "aggregate_worst_p95_query_batch_ms": aggregate_p95,
                "client_distributions": group,
                "wall_ms": clients_started.elapsed().as_secs_f64() * 1000.,
            }));
        }
        measurements.push(json!({"facts":count,"insert_ms":insert_ms,"single_query_ms":single_ms,"clients":client_measurements}));
    }
    let (native_bytes, full_snapshot_bytes) = journal_bytes_fixture(dir.path());
    assert!(
        native_bytes.saturating_mul(2) <= full_snapshot_bytes,
        "native journal bytes {native_bytes} must be <= 50% of repeated full snapshots {full_snapshot_bytes}"
    );
    let report = json!({"fixture":"native-facts-v1","dimensions":384,"counts":[0,100,1000,10000,100000],"measurements":measurements,"checkpoint":{"native_bytes":native_bytes,"full_snapshot_reference_bytes":full_snapshot_bytes},"comparison":"No retired-runtime timing or speedup claim is made."});
    if let Some(path) = std::env::var_os("MIRKET_NATIVE_PERF_OUTPUT") {
        fs::write(
            PathBuf::from(path),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
    eprintln!("{report}");
}
