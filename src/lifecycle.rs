//! Project lifecycle uses the same runtime and scope as workers and memory.
use crate::{agent_runtime, paths::Paths, router::durable, util};
use anyhow::{Context, Result, ensure};
use rmcp::schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub expired_before: u64,
    pub completed_before: u64,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Export,
    Plan,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Memory,
    Workflow,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Cursor {
    pub phase: Phase,
    pub after: Option<String>,
    /// Export revision fingerprint; absent for retention previews.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MemoryRef {
    pub id: String,
    pub sha256: String,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkflowRef {
    pub run_id: String,
    pub checkpoint_id: String,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetentionPlan {
    pub scope: String,
    pub policy: Policy,
    pub memories: Vec<MemoryRef>,
    pub workflows: Vec<WorkflowRef>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Inspect {
    pub project: PathBuf,
    pub action: Action,
    pub cursor: Option<Cursor>,
    pub policy: Option<Policy>,
}
impl Inspect {
    pub fn project(&self) -> &Path {
        &self.project
    }
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Apply {
    pub project: PathBuf,
    /// Exact plan returned by runtime_data; not a new selection of records.
    pub plan: RetentionPlan,
    pub expected_sha256: String,
}
pub async fn inspect(paths: &Paths, request: Inspect) -> Result<Value> {
    let (operation, args) = match request.action {
        Action::Export => {
            ensure!(
                request.policy.is_none(),
                "export does not accept a retention policy"
            );
            (
                "data_export",
                json!({"cursor":request.cursor,"page_bytes":48 * 1024}),
            )
        }
        Action::Plan => (
            "data_plan",
            json!({"policy":request.policy.as_ref().context("retention plan requires a policy")?,"cursor":request.cursor}),
        ),
    };
    agent_runtime::call(paths, Some(request.project()), operation, args).await
}
pub async fn apply(paths: &Paths, request: Apply) -> Result<Value> {
    let _maintenance = crate::maintenance::shared(paths)?;
    let runs = &request.plan.workflows;
    ensure!(
        runs.len() <= 100 && request.plan.memories.len() <= 100,
        "retention batch exceeds 100 entries"
    );
    let mut locks = Vec::new();
    for run in runs {
        let id = run.run_id.as_str();
        // Same execution lock as resume, spawn and forget. Never collect a live
        // worker even if its checkpoint currently looks terminal.
        let path = durable::run_path(paths, id)?;
        locks.push(util::FileLock::acquire(&path.join("execution.lock"))?);
        let trace: Value = serde_json::from_slice(&util::read_bounded(
            &path.join("trace.json"),
            32 * 1024 * 1024,
        )?)?;
        ensure!(
            matches!(trace["status"].as_str(), Some("complete" | "host_verified")),
            "run trace still reports unresolved work; retention refused"
        );
        if trace["host_verification"].is_object() {
            let checked = crate::router::handoff::record(
                paths,
                &crate::router::handoff::Request {
                    project: request.project.clone(),
                    run_id: id.into(),
                    checks: None,
                    expected_trace_sha256: None,
                    begin: None,
                    attempt_id: None,
                },
            )?;
            ensure!(
                checked["status"] == "host_verified",
                "host verification is unresolved; retention refused"
            );
        }
    }
    agent_runtime::call(
        paths,
        Some(&request.project),
        "data_apply",
        json!({"plan":request.plan,"expected_sha256":request.expected_sha256}),
    )
    .await
}

/// Publish only a complete consistent export; refuse overwriting any destination.
pub async fn export(paths: &Paths, project: &Path, output: &Path) -> Result<Value> {
    ensure!(output.is_absolute(), "export output must be absolute");
    util::reject_symlink_components(output)?;
    ensure!(!output.exists(), "export destination already exists");
    let parent = output.parent().context("export parent")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    let mut session = agent_runtime::Session::open(paths, Some(project)).await?;
    let mut cursor = Value::Null;
    let mut count = 0;
    let mut bytes = 0;
    let mut fingerprint = Value::Null;
    loop {
        let page = session
            .call(Some(project), "data_export", json!({"cursor":cursor}))
            .await?;
        let result = &page["result"];
        if fingerprint.is_null() {
            fingerprint = result["sha256"].clone();
        }
        ensure!(
            fingerprint == result["sha256"],
            "export changed between pages"
        );
        let mut encoded = serde_json::to_vec(&page)?;
        encoded.push(b'\n');
        bytes += encoded.len();
        ensure!(bytes <= 512 * 1024 * 1024, "export exceeds 512 MiB");
        temporary.write_all(&encoded)?;
        count += result["items"].as_array().context("export items")?.len();
        cursor = result["next_cursor"].clone();
        if cursor.is_null() {
            ensure!(
                Some(count as u64) == result["total"].as_u64(),
                "incomplete export"
            );
            break;
        }
    }
    session.close().await?;
    temporary.as_file().sync_all()?;
    temporary
        .persist_noclobber(output)
        .context("publish complete export")?;
    Ok(
        json!({"exported":true,"path":output,"records":count,"bytes":bytes,"sha256":fingerprint,
        "coverage":"Logical facts and latest snapshots, not a restorable database backup or historical traces."}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn retention_refuses_active_worker_before_runtime_or_deletion() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
        let run = paths.root.join("runs/routed-active");
        std::fs::create_dir_all(&run).unwrap();
        let _lock = util::FileLock::acquire(&run.join("execution.lock")).unwrap();
        let error = apply(
            &paths,
            Apply {
                project: home.path().into(),
                plan: serde_json::from_value(json!({"scope":"a".repeat(64),"policy":{"expired_before":0,"completed_before":0},"memories":[],"workflows":[{"run_id":"routed-active","checkpoint_id":"fixture"}]})).unwrap(),
                expected_sha256: "unused".into(),
            },
        )
        .await
        .unwrap_err();
        assert!(!format!("{error:#}").contains("not selected"));
        assert!(!paths.root.join("execution").exists());
    }

    #[tokio::test]
    async fn export_preserves_existing_destination() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
        let output = home.path().canonicalize().unwrap().join("existing.jsonl");
        std::fs::write(&output, b"user evidence").unwrap();
        let error = export(&paths, home.path(), &output).await.unwrap_err();
        assert!(format!("{error:#}").contains("already exists"));
        assert_eq!(std::fs::read(&output).unwrap(), b"user evidence");
    }
}
