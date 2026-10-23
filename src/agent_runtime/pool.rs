//! Process-local project sessions, with cross-process admission owned by EmbeddingSession.
use super::EmbeddingSession;
use crate::paths::Paths;
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

type Entry = Arc<Mutex<Option<(EmbeddingSession, Instant)>>>;
type Key = (std::path::PathBuf, std::path::PathBuf);
static REQUESTS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(16);
static POOL: OnceLock<Mutex<BTreeMap<Key, Entry>>> = OnceLock::new();
const IDLE: Duration = Duration::from_secs(60);

pub async fn embed(
    paths: &Paths,
    project: &Path,
    texts: Vec<String>,
    fit_auto_queries: bool,
) -> Result<(Vec<Vec<f32>>, Vec<String>, Value)> {
    let _request = REQUESTS
        .try_acquire()
        .map_err(|_| anyhow::anyhow!("embedding request queue is full; no inference dispatched"))?;
    let key = (
        paths.root.clone(),
        crate::paths::canonical_project(project)?,
    );
    let pool = POOL.get_or_init(Default::default);
    let entry = {
        let mut entries = pool.lock().await;
        if let Some(entry) = entries.get(&key) {
            entry.clone()
        } else {
            ensure!(
                entries.len() < 16,
                "embedding project queue is full; no inference dispatched"
            );
            let entry = Arc::new(Mutex::new(None));
            entries.insert(key.clone(), entry.clone());
            let owned = entry.clone();
            let paths = paths.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                    let Ok(mut held) = owned.try_lock() else {
                        continue;
                    };
                    if held.as_ref().is_none_or(|(_, last)| {
                        last.elapsed() >= IDLE || super::has_waiters(&paths)
                    }) {
                        if let Some((session, _)) = held.take() {
                            let _ = session.close().await;
                        }
                        let mut entries = POOL.get().unwrap().lock().await;
                        if Arc::strong_count(&owned) == 2 {
                            entries.remove(&key);
                            break;
                        }
                    }
                }
            });
            entry
        }
    };
    let queued = Instant::now();
    let mut held = entry.lock().await;
    let queue_ms = queued.elapsed().as_secs_f64() * 1000.0;
    let cold = held.is_none();
    let startup = Instant::now();
    if cold {
        *held = Some((
            EmbeddingSession::open(paths, Some(project)).await?,
            Instant::now(),
        ));
    }
    let startup_ms = if cold {
        startup.elapsed().as_secs_f64() * 1000.0
    } else {
        0.0
    };
    // Keep the in-flight transport out of the pool. Cancellation drops it,
    // closing its input and releasing admission instead of reusing unread replies.
    let mut session = held.take().unwrap().0;
    let result = session
        .call(
            Some(project),
            "embed",
            json!({"texts":texts,"fit_auto_queries":fit_auto_queries}),
        )
        .await;
    match result {
        Ok(value) => {
            let decoded = (|| -> Result<(Vec<Vec<f32>>, Vec<String>)> {
                let vectors: Vec<Vec<f32>> =
                    serde_json::from_value(value["result"]["vectors"].clone())?;
                ensure!(
                    vectors.len() == texts.len()
                        && vectors.iter().all(|v| v.len() == 384
                            && v.iter().all(|x| x.is_finite())
                            && v.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>() > 0.),
                    "invalid helper vectors"
                );
                let effective: Vec<String> =
                    serde_json::from_value(value["result"]["effective_texts"].clone())?;
                ensure!(
                    effective.len() == vectors.len(),
                    "effective query count mismatch"
                );
                Ok((vectors, effective))
            })();
            let (vectors, effective) = match decoded {
                Ok(result) => result,
                Err(error) => {
                    held.take();
                    return Err(error);
                }
            };
            *held = Some((session, Instant::now()));
            Ok((
                vectors,
                effective,
                json!({"queue":queue_ms,"startup":startup_ms,"model":value["result"]["timing_ms"],"transport":value["transport"],"resources":value["result"]["resources"]}),
            ))
        }
        Err(error) => {
            held.take();
            Err(error)
        }
    }
}
