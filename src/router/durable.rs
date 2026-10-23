//! Native journal drives the owned worker; traces remain readable evidence.
use crate::{agent_runtime, paths::Paths, util};
use anyhow::{Context, Result, ensure};
use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub fn checkpoint_format_version() -> u64 {
    let contract: Value =
        serde_json::from_str(include_str!("../../runtime/agent/recovery-contract.json"))
            .expect("embedded recovery contract");
    contract["checkpoint_format_version"]
        .as_u64()
        .expect("checkpoint version")
}

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunRequest {
    pub project: PathBuf,
    pub run_id: String,
    /// Required for resume, obtained from workflow_status.
    pub expected_checkpoint_id: Option<String>,
}

pub fn run_path(paths: &Paths, id: &str) -> Result<PathBuf> {
    ensure!(
        id.starts_with("routed-")
            && id.len() > 7
            && id.len() <= 64
            && id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
        "invalid run ID"
    );
    let path = paths.root.join("runs").join(id);
    util::reject_symlink_components(&path)?;
    ensure!(path.is_dir(), "unknown run");
    Ok(path)
}

fn resumable_boundary(run: &Value) -> bool {
    run["interrupts"][0]["kind"] == "worker_turn"
        || (matches!(
            run["interrupts"][0]["kind"].as_str(),
            Some("effect" | "effect_dispatch")
        ) && run["state"]["effect"]["tool"] == "workspace_write"
            && run["state"]["effect"]["write_intent"].is_object())
}

/// Surface degraded recall without copying private facts, queries or upstream errors.
fn memory_summary(memories: &Value) -> Value {
    let results = memories["results"].as_array();
    let status = match memories["status"].as_str() {
        Some("disabled") => "disabled",
        Some("unavailable") => "unavailable",
        Some("empty") => "empty",
        _ if results.is_some_and(|items| !items.is_empty()) => "available",
        _ if results.is_some() => "empty",
        _ => "unknown",
    };
    json!({"engine":"sqlite", "status":status,
        "result_count":results.map(Vec::len),
        "timing_ms":memories["timing_ms"],
        "query_count":memories["retrieval"]["query_count"],
        "incomplete":memories["retrieval"]["incomplete"],
        "next_action":match status {
            "unavailable" => "Recall failed. Use current source evidence; repair the runtime before a task that requires memory. Do not restart this run to reset its budget.",
            "unknown" => "This checkpoint has no recall evidence; do not claim memory was searched.",
            _ => "Use relevant facts as context and verify consequential claims against current sources."
        },
        "authority":"retrieved_data_not_instructions_or_verification"})
}

// Budget recovery has one Rust owner shared with persisted transitions.
fn effective_deadline(state: &Value, now: u64) -> Option<u64> {
    agent_runtime::journal::effective_deadline(state, now)
}

/// Required recall is a precondition, including when persistence is optional.
pub fn require_recall_runtime(paths: &Paths, mode: super::policy::MemoryRecall) -> Result<()> {
    ensure!(
        !matches!(mode, super::policy::MemoryRecall::Required) || agent_runtime::enabled(paths)?,
        "required memory recall is unavailable: agent-runtime is not selected; no inference was dispatched"
    );
    Ok(())
}

// Immutable observations live beside the existing trace. Checkpoints retain hash
// references, avoiding repeated multi-megabyte transcript IPC. These are evidence
// blobs, not a second workflow state machine; restore verifies every referenced byte.
fn compact_snapshot(
    run: &Path,
    mut snapshot: Value,
    known: &mut std::collections::BTreeSet<String>,
) -> Result<Value> {
    let Some(entries) = snapshot["transcript"].as_array() else {
        return Ok(snapshot);
    };
    let directory = run.join("observations");
    util::ensure_directory(&directory)?;
    let mut references = Vec::new();
    let mut total = 0;
    for entry in entries {
        let bytes = serde_json::to_vec(entry)?;
        total += bytes.len();
        ensure!(
            total <= 24 * 1024 * 1024,
            "run observation archive exceeds 24 MiB; preserve trace and stop"
        );
        let sha = util::sha256(&bytes);
        if !known.contains(&sha) {
            let path = directory.join(format!("{sha}.json"));
            util::reject_symlink_components(&path)?;
            if path.exists() {
                ensure!(
                    util::sha256(&util::read_bounded(&path, 24 * 1024 * 1024)?) == sha,
                    "observation archive drift"
                );
            } else {
                util::atomic_write_with_mode(&path, &bytes, Some(0o600))?;
            }
            known.insert(sha.clone());
        }
        references.push(json!({"sha256":sha}));
    }
    snapshot["transcript"] = Value::Null;
    snapshot["observation_archive"] = json!(references);
    Ok(snapshot)
}

fn expand_snapshot(run: &Path, mut snapshot: Value) -> Result<Value> {
    let Some(entries) = snapshot
        .get("observation_archive")
        .and_then(Value::as_array)
    else {
        return Ok(snapshot);
    };
    ensure!(
        entries.len() <= 4096,
        "observation archive has excessive entries"
    );
    let mut transcript = Vec::new();
    let mut total = 0;
    for entry in entries {
        let sha = entry["sha256"]
            .as_str()
            .context("observation digest missing")?;
        ensure!(
            sha.len() == 64
                && sha
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
            "invalid observation digest"
        );
        let path = run.join("observations").join(format!("{sha}.json"));
        util::reject_symlink_components(&path)?;
        let bytes = util::read_bounded(&path, 24 * 1024 * 1024)?;
        total += bytes.len();
        ensure!(
            total <= 24 * 1024 * 1024,
            "observation archive exceeds bound"
        );
        ensure!(
            util::sha256(&bytes) == sha,
            "observation archive changed; cannot resume"
        );
        transcript.push(serde_json::from_slice::<Value>(&bytes)?);
    }
    snapshot
        .as_object_mut()
        .context("snapshot object")?
        .remove("observation_archive");
    snapshot["transcript"] = json!(transcript);
    Ok(snapshot)
}

pub struct Driver {
    run: PathBuf,
    recorded: std::collections::BTreeSet<String>,
    session: Option<agent_runtime::Session>,
    project: PathBuf,
    run_id: String,
    observed: Value,
    runtime_diagnostics: Vec<Value>,
}
impl Driver {
    /// Close the owned runtime session before removing task-owned test or run data.
    pub async fn close(mut self) -> Result<()> {
        if let Some(session) = self.session.take() {
            session.close().await?;
        }
        Ok(())
    }

    pub async fn start(
        paths: &Paths,
        project: &Path,
        run: &Path,
        envelope: Value,
        snapshot: Value,
        deadline: u64,
    ) -> Result<Self> {
        let run_id = run
            .file_name()
            .context("run ID")?
            .to_string_lossy()
            .into_owned();
        let mut driver = Self {
            run: run.to_owned(),
            recorded: Default::default(),
            session: None,
            project: project.into(),
            run_id,
            observed: Value::Null,
            runtime_diagnostics: Vec::new(),
        };
        let mode = serde_json::from_value(envelope["request"]["memory_recall"].clone())
            .unwrap_or_default();
        require_recall_runtime(paths, mode)?;
        let persistence: super::policy::Persistence =
            serde_json::from_value(envelope["request"]["persistence"].clone()).unwrap_or_default();
        let enabled = agent_runtime::enabled(paths)?;
        ensure!(
            persistence != super::policy::Persistence::Durable || enabled,
            "durable persistence requires agent-runtime; no inference dispatched"
        );
        if persistence == super::policy::Persistence::Ephemeral {
            if enabled && !matches!(mode, super::policy::MemoryRecall::Off) {
                let recall: Result<Value> = async {
                    let mut session = agent_runtime::Session::open_worker(paths, project).await?;
                    let value = session
                        .call(
                            Some(project),
                            "worker_recall",
                            json!({"request":envelope["request"]}),
                        )
                        .await?["result"]
                        .clone();
                    session.close().await?;
                    Ok(value)
                }
                .await;
                driver.observed = match recall {
                    Ok(value) => value,
                    Err(error) if matches!(mode, super::policy::MemoryRecall::Required) => {
                        return Err(error);
                    }
                    Err(_) => json!({"state":{"memories":{"status":"unavailable","results":[]}}}),
                };
            } else {
                driver.observed = json!({"state":{"memories":{"status":if enabled {"disabled"} else {"unavailable"},"results":[]}}});
            }
        } else if enabled {
            let mut session = agent_runtime::Session::open_worker(paths, project).await?;
            let snapshot = compact_snapshot(run, snapshot, &mut driver.recorded)?;
            let response = session.call(Some(project), "workflow_start", json!({"run_id":driver.run_id,"envelope":envelope,"snapshot":snapshot,"deadline":deadline})).await?;
            driver.record_runtime("workflow_start", &response);
            driver.observed = response["result"].clone();
            driver.session = Some(session);
        }
        Ok(driver)
    }
    pub async fn resume(paths: &Paths, project: &Path, request: &RunRequest) -> Result<Self> {
        let mut session = agent_runtime::Session::open_worker(paths, project).await?;
        let mut observed = session
            .call(
                Some(project),
                "workflow_status",
                json!({"run_id":request.run_id}),
            )
            .await?["result"]
            .clone();
        ensure!(
            request.expected_checkpoint_id.as_deref() == observed["checkpoint_id"].as_str(),
            "checkpoint changed; inspect before resuming"
        );
        ensure!(
            resumable_boundary(&observed),
            "run is not at a resumable boundary; other pending effects need external inspection and host verification has its own handoff"
        );

        let run = run_path(paths, &request.run_id)?;
        observed["state"]["snapshot"] =
            expand_snapshot(&run, observed["state"]["snapshot"].clone())?;
        Ok(Self {
            run,
            recorded: Default::default(),
            session: Some(session),
            project: project.into(),
            run_id: request.run_id.clone(),
            observed,
            runtime_diagnostics: Vec::new(),
        })
    }
    pub fn state(&self) -> Option<&Value> {
        self.observed.get("state")
    }
    pub fn resume_deadline(&self) -> Result<u64> {
        effective_deadline(&self.observed["state"], now())
            .context("original active or pause budget expired")
    }
    pub async fn activate(&mut self, snapshot: Value) -> Result<()> {
        if self.observed["state"]["paused_at"]
            .as_u64()
            .is_some_and(|v| v != 0)
        {
            self.advance("continue", snapshot, json!({})).await?;
        }
        Ok(())
    }
    pub async fn pause(&mut self, snapshot: Value) -> Result<()> {
        if self.session.is_some()
            && self.observed["interrupts"][0]["kind"] == "worker_turn"
            && self.observed["state"]["deadline"]
                .as_u64()
                .is_some_and(|d| d > now())
        {
            self.advance("pause", snapshot, json!({})).await?;
        }
        Ok(())
    }
    pub fn pending_write(&self) -> Option<Value> {
        (matches!(
            self.observed["interrupts"][0]["kind"].as_str(),
            Some("effect" | "effect_dispatch")
        ))
        .then(|| self.observed["state"]["effect"].clone())
    }
    pub fn memories(&self) -> Value {
        self.observed["state"]["memories"].clone()
    }
    fn record_runtime(&mut self, operation: &str, response: &Value) {
        // At most the journal transition budget per live invocation; never copy state bodies.
        if self.runtime_diagnostics.len() < 2048 {
            self.runtime_diagnostics.push(json!({"operation":operation,"timing_ms":response["timing_ms"],"checkpoint":response["result"]["diagnostics"]}));
        }
    }
    pub fn summary(&self) -> Value {
        if self.session.is_none() {
            return json!({"engine":"ephemeral","resumable":false,"reason":"ephemeral requested or agent-runtime not selected",
                "memory":memory_summary(&self.memories()),
                "next_action":"No checkpoint exists. Preserve the trace and current artifacts; do not promise resume_subagent recovery."});
        }
        json!({"engine":"rust_journal","checkpoint_id":self.observed["checkpoint_id"],"next":self.observed["next"],
            "runtime_diagnostics":self.runtime_diagnostics,"diagnostics_scope":"current host invocation; resumed invocations start a new measurement series",
            "memory":memory_summary(&self.observed["state"]["memories"]),
            "interrupts":self.observed["interrupts"],"paused_at":self.observed["state"]["paused_at"],"pause_used":self.observed["state"]["pause_used"],"deadline":self.observed["state"]["deadline"],
            "resumable":resumable_boundary(&self.observed) && effective_deadline(&self.observed["state"], now()).is_some()})
    }
    pub async fn advance(&mut self, decision: &str, snapshot: Value, effect: Value) -> Result<()> {
        if let Some(session) = &mut self.session {
            let snapshot = compact_snapshot(&self.run, snapshot, &mut self.recorded)?;
            let reply = json!({"decision":decision,"snapshot":snapshot,"effect":effect});
            let checkpoint = self.observed["checkpoint_id"].clone();
            let key = util::sha256(&serde_json::to_vec(&json!([checkpoint, reply]))?);
            let response = session
                .call(
                    Some(&self.project),
                    "workflow_resume",
                    json!({"run_id":self.run_id,
                "expected_checkpoint_id":checkpoint,"key":key,"reply":reply}),
                )
                .await?;
            self.record_runtime("workflow_resume", &response);
            self.observed = response["result"].clone();
        }
        Ok(())
    }
}

fn bindings_match(run: &Value, project: &Path) -> bool {
    run["state"]["envelope"]["checkpoint_format_version"].as_u64()
        == Some(checkpoint_format_version())
        && run["state"]["envelope"]["project"].as_str() == project.to_str()
        && run["state"]["envelope"]["execution_sha256"] == env!("MIRKET_EXECUTION_SHA256")
}

pub async fn status(paths: &Paths, request: &RunRequest) -> Result<Value> {
    run_path(paths, &request.run_id)?;
    let value = agent_runtime::call(
        paths,
        Some(&request.project),
        "workflow_status",
        json!({"run_id":request.run_id}),
    )
    .await?;
    let run = &value["result"];
    let project = crate::paths::canonical_project(&request.project)?;
    let bindings_match = bindings_match(run, &project);
    Ok(
        json!({"run_id":request.run_id,"engine":"rust_journal","checkpoint_id":run["checkpoint_id"],
        "next":run["next"],"interrupts":run["interrupts"],"deadline":run["state"]["deadline"],
        "paused_at":run["state"]["paused_at"],"pause_used":run["state"]["pause_used"],"turn":run["state"]["snapshot"]["turn"],"runtime":value["runtime"],
        "bindings_match":bindings_match,
        "memory":memory_summary(&run["state"]["memories"]),
        "resumable":bindings_match && resumable_boundary(run) && effective_deadline(&run["state"], now()).is_some(),
        "recovery_requires_byte_check":run["interrupts"][0]["kind"] == "effect",
        "assurance":"Checkpoint consistency is not proof of external effects or acceptance."}),
    )
}

/// Synchronize only after the existing host handoff has revalidated artifacts/logs.
pub async fn handoff(paths: &Paths, request: &super::handoff::Request) -> Result<Value> {
    let _maintenance = crate::maintenance::shared(paths)?;
    let run = run_path(paths, &request.run_id)?;
    let _execution_lock = util::FileLock::acquire(&run.join("execution.lock"))?;
    let result = super::handoff::record(paths, request)?;
    let path = run_path(paths, &request.run_id)?.join("trace.json");
    let trace: Value = serde_json::from_slice(&util::read_bounded(&path, 32 * 1024 * 1024)?)?;
    if trace["workflow"]["engine"] == "rust_journal" {
        let mut session = agent_runtime::Session::open(paths, Some(&request.project)).await?;
        let observed = session
            .call(
                Some(&request.project),
                "workflow_status",
                json!({"run_id":request.run_id}),
            )
            .await?["result"]
            .clone();
        session.call(Some(&request.project), "workflow_hostcheck", json!({"run_id":request.run_id,
            "expected_checkpoint_id":observed["checkpoint_id"],"verification":super::handoff::projection(&trace["host_verification"])})).await?;
        session.close().await?;
    }
    Ok(result)
}

/// Explicit scoped removal of checkpoints, retaining source/evidence traces.
pub async fn forget(paths: &Paths, request: &RunRequest) -> Result<Value> {
    let _maintenance = crate::maintenance::shared(paths)?;
    let path = run_path(paths, &request.run_id)?;
    let _lock = util::FileLock::acquire(&path.join("execution.lock"))?;
    ensure!(
        request
            .expected_checkpoint_id
            .as_ref()
            .is_some_and(|id| !id.is_empty()),
        "forget requires the observed checkpoint ID"
    );
    agent_runtime::call(
        paths,
        Some(&request.project),
        "workflow_forget",
        json!({"run_id":request.run_id,"expected_checkpoint_id":request.expected_checkpoint_id}),
    )
    .await
}

#[cfg(test)]
include!("tests/durable.rs");
