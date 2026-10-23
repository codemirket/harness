//! Bounded native-versus-retired recall benchmark.
//!
//! The retired side runs in its pinned baseline container with a private
//! in-container temporary store and no host mounts. Both sides receive the
//! same synthetic facts and three-query batches. This is an evidence gate for
//! the approved migration threshold, not a general product benchmark.
use anyhow::Context;
use mirket::{agent_runtime::Session, paths::Paths, podman::Engine};
use serde_json::{Value, json};
use std::{fs, time::Instant};

const QUERIES: [&str; 3] = ["MIRKET-4827", "SQLite", "Türkçe"];
const EXPECTED_KEYS: [&str; 3] = ["project-code", "db", "language-tr"];

fn facts() -> [Value; 3] {
    [
        json!({"key":"project-code","text":"The release identifier is MIRKET-4827.","kind":"fact","provenance":"synthetic comparative benchmark","expires_at":null}),
        json!({"key":"db","text":"The project database is SQLite.","kind":"decision","provenance":"synthetic comparative benchmark","expires_at":null}),
        json!({"key":"language-tr","text":"Yanıt dili Türkçe olmalı.","kind":"preference","provenance":"synthetic comparative benchmark","expires_at":null}),
    ]
}

async fn native_warm_p95(
    paths: &Paths,
    project: &std::path::Path,
) -> anyhow::Result<(f64, Vec<f64>)> {
    let scope = mirket::agent_runtime::scope(paths, project)?;
    let mut session = Session::open(paths, Some(project)).await?;
    for item in facts() {
        let response = session.call(Some(project), "memory_put", item).await?;
        anyhow::ensure!(
            response["result"]["memory"]["memory"].is_string(),
            "native helper put omitted acknowledged fact"
        );
    }
    let request = json!({"request":{"memory_recall":"required","memory_queries":QUERIES}});
    let batch = |response: &Value| -> anyhow::Result<()> {
        let items = response["result"]["state"]["memories"]["results"]
            .as_array()
            .context("native recall omitted results")?;
        anyhow::ensure!(
            items.len() >= 3,
            "native recall returned fewer than three eligible synthetic facts"
        );
        for expected in EXPECTED_KEYS {
            anyhow::ensure!(
                items
                    .iter()
                    .any(|item| { item["metadata"]["request_key"].as_str() == Some(expected) }),
                "native recall omitted expected fact {expected}"
            );
        }
        Ok(())
    };
    batch(
        &session
            .call(Some(project), "worker_recall", request.clone())
            .await?,
    )?;
    batch(
        &session
            .call(Some(project), "worker_recall", request.clone())
            .await?,
    )?;
    let mut samples = Vec::with_capacity(20);
    for _ in 0..20 {
        let started = Instant::now();
        batch(
            &session
                .call(Some(project), "worker_recall", request.clone())
                .await?,
        )?;
        samples.push(started.elapsed().as_secs_f64() * 1000.);
    }
    session.close().await?;
    let _ = fs::remove_dir_all(mirket::packages::agent::data(paths).join(scope));
    samples.sort_by(f64::total_cmp);
    let p95 = samples[((samples.len() * 95).saturating_sub(1) / 100).min(samples.len() - 1)];
    Ok((p95, samples))
}

fn baseline_script() -> String {
    let facts = serde_json::to_string(&serde_json::to_string(&facts()).unwrap()).unwrap();
    let queries = serde_json::to_string(&serde_json::to_string(&QUERIES).unwrap()).unwrap();
    let expected_keys =
        serde_json::to_string(&serde_json::to_string(&EXPECTED_KEYS).unwrap()).unwrap();
    format!(
        r#"
import json, tempfile, time
from pathlib import Path
from memory import put, recall_many
root=Path(tempfile.mkdtemp(prefix='mirket-recall-perf-'))
scope='b'*64
facts=json.loads({facts})
queries=json.loads({queries})
expected_keys=json.loads({expected_keys})
for item in facts:
    put(root, scope, {{'key':item['key'],'text':item['text'],'kind':item['kind'],'provenance':item['provenance'],'expires_at':None}})
def batch():
    result=recall_many(root, scope, queries)
    assert len(result)==3
    for page, expected_key in zip(result, expected_keys):
        assert any(item.get('metadata',{{}}).get('request_key') == expected_key for item in page.get('results',[])), (expected_key, page)
    return result
batch()
batch()
samples=[]
for _ in range(20):
    started=time.perf_counter()
    batch()
    samples.append((time.perf_counter()-started)*1000)
samples.sort()
p95=samples[min((len(samples)*95-1)//100,len(samples)-1)]
print(json.dumps({{'p95_ms':p95,'samples_ms':samples,'sample_count':len(samples),'batch_size':len(queries)}}))
"#
    )
}

#[tokio::test]
#[ignore = "explicit baseline image and prepared engine; warm recall migration gate"]
async fn native_recall_p95_beats_retired_baseline() {
    let image =
        std::env::var("MIRKET_BASELINE_AGENT_IMAGE").expect("explicit MIRKET_BASELINE_AGENT_IMAGE");
    assert!(
        image.starts_with("sha256:")
            && image.len() == 71
            && image[7..].bytes().all(|b| b.is_ascii_hexdigit())
    );
    let home =
        std::env::var_os("MIRKET_TEST_ENGINE_HOME").expect("explicit MIRKET_TEST_ENGINE_HOME");
    let paths = Paths::new(std::path::PathBuf::from(home).canonicalize().unwrap()).unwrap();
    let engine = Engine::resolve(&paths).unwrap();
    let script = baseline_script();
    let output = engine
        .run(
            &[
                "run".into(),
                "--rm".into(),
                "--network=none".into(),
                "--read-only".into(),
                "--cap-drop=ALL".into(),
                "--security-opt=no-new-privileges".into(),
                "--memory=1536m".into(),
                "--cpus=2".into(),
                "--pids-limit=128".into(),
                "--tmpfs=/tmp:rw,nosuid,nodev,noexec,size=256m,mode=1777".into(),
                "--entrypoint=python".into(),
                image.clone(),
                "-B".into(),
                "-c".into(),
                script,
            ],
            std::time::Duration::from_secs(300),
        )
        .unwrap();
    let retired: Value = serde_json::from_str(output.lines().last().unwrap()).unwrap();
    let retired_p95 = retired["p95_ms"].as_f64().unwrap();
    let project_dir = tempfile::tempdir().unwrap();
    let project = project_dir.path().canonicalize().unwrap();
    let (native_p95, native_samples) = native_warm_p95(&paths, &project).await.unwrap();
    assert!(retired_p95.is_finite() && retired_p95 > 0.);
    assert!(native_p95.is_finite() && native_p95 > 0.);
    let ratio = native_p95 / retired_p95;
    assert!(
        ratio <= 0.75,
        "native warm recall p95 {native_p95:.3}ms / retired {retired_p95:.3}ms = {ratio:.3}, threshold is 0.75"
    );
    let report = json!({"fixture":"recall-comparative-v1","queries":QUERIES,"batch_size":3,"warm_samples":20,"retired_baseline_p95_ms":retired_p95,"retired_baseline_samples_ms":retired["samples_ms"],"native_p95_ms":native_p95,"native_samples_ms":native_samples,"native_to_retired_p95_ratio":ratio,"threshold":0.75,"baseline_image":image,"host_mounts":false});
    if let Some(path) = std::env::var_os("MIRKET_RECALL_PERF_OUTPUT") {
        fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    eprintln!("{report}");
}
