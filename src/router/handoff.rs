//! Host verification extends the existing run trace; workers cannot dispatch it.
use super::{
    policy::SpawnRequest,
    tools::{ArtifactRef, current_digest},
};
use crate::{paths::Paths, util};
use anyhow::{Context, Result, ensure};
use rmcp::schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

pub fn default_checks() -> Vec<String> {
    vec!["project_checks".into()]
}

pub fn validate_checks(request: &SpawnRequest) -> Result<()> {
    let checks = &request.host_verification_checks;
    ensure!(
        !checks.is_empty() && checks.len() <= 16,
        "declare 1..=16 host verification checks"
    );
    let unique: BTreeSet<_> = checks.iter().collect();
    ensure!(
        unique.len() == checks.len(),
        "duplicate host verification check"
    );
    for name in checks {
        ensure!(
            !name.is_empty()
                && name.len() <= 64
                && name
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-'),
            "invalid host verification check name"
        );
    }
    Ok(())
}

pub(crate) fn required(request: &SpawnRequest) -> bool {
    request.task_needs.iter().any(|need| {
        matches!(
            need.workflow.as_str(),
            "source-implementation" | "web-interface" | "interactive-motion" | "editable-interface"
        )
    })
}

pub(crate) fn create(
    paths: &Paths,
    request: &SpawnRequest,
    project: &Path,
    artifacts: &[ArtifactRef],
) -> Result<Option<Value>> {
    let design_scope = super::tools::design_scope(paths, project)?;
    Ok(required(request).then(|| json!({
        "project":project, "required_checks":request.host_verification_checks,
        "artifacts":artifacts.iter().filter(|a| a.path.starts_with(project)).map(|a| json!({"path":a.path.strip_prefix(project).expect("project artifact"), "sha256":a.sha256})).collect::<Vec<_>>(),
        "design_artifacts":artifacts.iter().filter(|a| !a.path.starts_with(project)).filter_map(|a| a.path.strip_prefix(&design_scope).ok().map(|path| json!({"path":path,"sha256":a.sha256}))).collect::<Vec<_>>(),
        "status":"awaiting_host_verification",
        "instruction":"Begin a verification attempt with input paths and named commands before executing checks. Run the project's required build/tests and relevant failure probes through authorized host tools. Save logs under this project and submit every required named check with evidence using verification_handoff. Readback alone is not runtime verification.",
        "assurance":"Host-reported checks with hash-bound evidence; not independent semantic verification or human approval."
    })))
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Passed,
    Failed,
    NotRun,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub name: String,
    pub status: CheckStatus,
    /// Actual check and located result; no commands are executed by this tool.
    pub summary: String,
    pub evidence: Option<Evidence>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Begin {
    /// Unique retry key for this check attempt; same input/key reuses it.
    pub key: String,
    /// Relevant source, tests, configuration and locks. Directories include additions/deletions.
    pub inputs: Vec<PathBuf>,
    /// Optional inputs relative to this project's assigned design workspace.
    #[serde(default)]
    pub design_inputs: Vec<PathBuf>,
    /// Exact command/runner identity for each required check; recorded, never executed here.
    pub commands: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub project: PathBuf,
    /// The routed-* directory name returned by spawn_subagent, never a path.
    pub run_id: String,
    /// Omit begin/checks to inspect and reconcile freshness; new attempts/submissions require the observed digest.
    pub checks: Option<Vec<Check>>,
    pub expected_trace_sha256: Option<String>,
    /// Prepare an input-bound attempt before running checks. Mutually exclusive with checks.
    pub begin: Option<Begin>,
    /// Attempt ID returned by begin; required when submitting checks.
    pub attempt_id: Option<String>,
}

fn manifest(project: &Path, inputs: &[PathBuf]) -> Result<BTreeMap<PathBuf, String>> {
    ensure!(
        inputs.len() <= 64,
        "declare at most 64 verification input paths per workspace"
    );
    let mut pending = inputs.to_vec();
    let mut files = BTreeMap::new();
    let mut visited = BTreeSet::new();
    let mut bytes = 0u64;
    while let Some(relative) = pending.pop() {
        util::safe_relative(&relative)?;
        ensure!(
            !relative.components().any(|c| c
                .as_os_str()
                .to_str()
                .is_some_and(|v| v.eq_ignore_ascii_case(".git"))),
            "repository internals are not verification inputs"
        );
        if !visited.insert(relative.clone()) {
            continue;
        }
        ensure!(
            visited.len() <= 20000,
            "verification input manifest exceeds 20000 entries; narrow the inputs"
        );
        let path = project.join(&relative);
        util::reject_symlink_components(&path)?;
        let metadata = std::fs::symlink_metadata(&path)?;
        if metadata.is_dir() {
            files.insert(relative.clone(), "directory".into());
            for entry in std::fs::read_dir(path)? {
                pending.push(relative.join(entry?.file_name()));
            }
        } else {
            ensure!(
                metadata.is_file(),
                "verification input is not a regular file"
            );
            bytes += metadata.len();
            ensure!(
                bytes <= 256 * 1024 * 1024,
                "verification inputs exceed 256 MiB; narrow the inputs"
            );
            files.insert(relative, current_digest(&path)?);
        }
    }
    Ok(files)
}

fn verify_file(project: &Path, evidence: &Evidence) -> Result<()> {
    util::safe_relative(&evidence.path)?;
    ensure!(
        current_digest(&project.join(&evidence.path))? == evidence.sha256,
        "verification input or evidence changed: {}",
        evidence.path.display()
    );
    Ok(())
}

/// Small MCP/checkpoint projection; complete immutable manifests remain in the trace.
pub(crate) fn projection(handoff: &Value) -> Value {
    let mut value = handoff.clone();
    let object = value.as_object_mut().expect("handoff object");
    object.remove("artifacts");
    object.remove("design_artifacts");
    if let Some(attempts) = object.get_mut("attempts").and_then(Value::as_array_mut) {
        for attempt in attempts {
            let files = json!({"project":attempt["files"],"design":attempt["design_files"]});
            *attempt = json!({"id":attempt["id"],"started_at":attempt["started_at"],"finished_at":attempt["finished_at"],
                "input_entries":attempt["files"].as_object().map_or(0, |v| v.len()) + attempt["design_files"].as_object().map_or(0, |v| v.len()),
                "manifest_sha256":util::sha256(serde_json::to_string(&files).expect("JSON manifest").as_bytes()),
                "checks_recorded":attempt.get("checks").is_some()});
        }
    }
    if let Some(checks) = object.get_mut("checks").and_then(Value::as_array_mut) {
        for check in checks {
            if let Some(summary) = check["summary"].as_str() {
                check["summary"] = json!(summary.chars().take(512).collect::<String>());
            }
        }
    }
    value
}

pub fn record(paths: &Paths, request: &Request) -> Result<Value> {
    let _maintenance = crate::maintenance::shared(paths)?;
    ensure!(
        request.run_id.starts_with("routed-")
            && request.run_id.len() <= 64
            && request
                .run_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
        "invalid run ID"
    );
    let project = crate::paths::canonical_project(&request.project)?;
    let run = paths.root.join("runs").join(&request.run_id);
    util::reject_symlink_components(&run)?;
    ensure!(run.is_dir(), "unknown run");
    let _lock = util::FileLock::acquire(&run.join("verification.lock"))?;
    let trace_path = run.join("trace.json");
    util::reject_symlink_components(&trace_path)?;
    let bytes = util::read_bounded(&trace_path, 32 * 1024 * 1024)?;
    let mut digest = util::sha256(&bytes);
    let mut trace: Value = serde_json::from_slice(&bytes)?;
    let handoff = trace
        .get("host_verification")
        .filter(|v| v.is_object())
        .context("run has no host verification handoff")?;
    ensure!(
        handoff["project"].as_str() == project.to_str(),
        "handoff belongs to a different project"
    );
    ensure!(
        !(request.begin.is_some() && request.checks.is_some()),
        "begin and checks are separate operations"
    );
    let required: BTreeSet<String> = serde_json::from_value(handoff["required_checks"].clone())?;
    let original = trace.clone();
    if let Some(begin) = &request.begin {
        ensure!(
            !begin.key.is_empty() && begin.key.len() <= 128 && !begin.key.contains('\0'),
            "attempt key requires 1..=128 bytes"
        );
        ensure!(
            begin.commands.keys().cloned().collect::<BTreeSet<_>>() == required,
            "declare the command/runner for every required check"
        );
        ensure!(
            begin
                .commands
                .values()
                .all(|c| !c.trim().is_empty() && c.len() <= 2048 && !c.contains('\0')),
            "invalid verification command identity"
        );
        let id = format!("attempt-{}", &util::sha256(begin.key.as_bytes())[..24]);
        let attempts = trace["host_verification"]
            .get("attempts")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if let Some(previous) = attempts.iter().find(|a| a["id"] == id) {
            ensure!(
                previous["begin"] == serde_json::to_value(begin)?,
                "attempt key reused with different inputs"
            );
            ensure!(
                trace["host_verification"]["active_attempt"] == id,
                "attempt has been superseded; inspect the active attempt"
            );
        } else {
            ensure!(
                request.expected_trace_sha256.as_deref() == Some(&digest),
                "trace changed; inspect before beginning verification"
            );
            ensure!(
                attempts.len() < 64,
                "run has 64 verification attempts; start a new run preserving this history"
            );
            let mut inputs = begin.inputs.clone();
            let artifacts: Vec<Evidence> =
                serde_json::from_value(trace["host_verification"]["artifacts"].clone())?;
            inputs.extend(artifacts.into_iter().map(|a| a.path));
            inputs.sort();
            inputs.dedup();
            let roots = inputs.clone();
            inputs.retain(|input| {
                !roots
                    .iter()
                    .any(|parent| input != parent && input.starts_with(parent))
            });
            let mut design_inputs = begin.design_inputs.clone();
            let design_artifacts: Vec<Evidence> = serde_json::from_value(
                trace["host_verification"]
                    .get("design_artifacts")
                    .cloned()
                    .unwrap_or_else(|| json!([])),
            )?;
            design_inputs.extend(design_artifacts.into_iter().map(|a| a.path));
            design_inputs.sort();
            design_inputs.dedup();
            ensure!(
                !inputs.is_empty() || !design_inputs.is_empty(),
                "verification requires input paths"
            );
            let files = manifest(&project, &inputs)?;
            let design_files = manifest(
                &super::tools::design_scope(paths, &project)?,
                &design_inputs,
            )?;
            let mut attempts = attempts;
            attempts.push(json!({"id":id,"begin":begin,"inputs":inputs,"files":files,
                "design_inputs":design_inputs,"design_files":design_files,"started_at":super::durable::now()}));
            trace["host_verification"]["attempts"] = json!(attempts);
            trace["host_verification"]["active_attempt"] = json!(id);
            trace["host_verification"]
                .as_object_mut()
                .unwrap()
                .remove("checks");
        }
    }
    let active = trace["host_verification"]["active_attempt"]
        .as_str()
        .map(str::to_owned);
    let index = active.as_ref().and_then(|id| {
        trace["host_verification"]["attempts"]
            .as_array()?
            .iter()
            .position(|a| a["id"].as_str() == Some(id))
    });
    let mut stale = None;
    if let Some(index) = index {
        let attempt = &trace["host_verification"]["attempts"][index];
        let inputs: Vec<PathBuf> = serde_json::from_value(attempt["inputs"].clone())?;
        match manifest(&project, &inputs) {
            Ok(files) if serde_json::to_value(&files)? == attempt["files"] => {}
            _ => {
                stale = Some(
                    "verification inputs changed; begin a new attempt before rerunning checks"
                        .to_owned(),
                )
            }
        }
        let design_inputs: Vec<PathBuf> = serde_json::from_value(
            attempt
                .get("design_inputs")
                .cloned()
                .unwrap_or_else(|| json!([])),
        )?;
        let design_expected = attempt
            .get("design_files")
            .cloned()
            .unwrap_or_else(|| json!({}));
        match manifest(
            &super::tools::design_scope(paths, &project)?,
            &design_inputs,
        ) {
            Ok(files) if serde_json::to_value(&files)? == design_expected => {}
            _ => stale = Some("design inputs changed; begin a new verification attempt".into()),
        }
        if request.checks.is_none()
            && let Some(checks) = attempt.get("checks")
        {
            for check in serde_json::from_value::<Vec<Check>>(checks.clone())? {
                if let Some(evidence) = check.evidence
                    && let Err(error) = verify_file(&project, &evidence)
                {
                    stale = Some(error.to_string());
                    break;
                }
            }
        }
        if let Some(checks) = &request.checks {
            ensure!(
                request.attempt_id.as_deref() == active.as_deref(),
                "submit the active attempt ID returned by begin"
            );
            let names: BTreeSet<_> = checks.iter().map(|c| c.name.clone()).collect();
            ensure!(
                checks.len() <= 16 && names.len() == checks.len() && names == required,
                "submit exactly the required checks, without duplicates"
            );
            for check in checks {
                ensure!(
                    !check.summary.trim().is_empty()
                        && check.summary.len() <= 2048
                        && !check.summary.contains('\0'),
                    "check summary requires 1..=2048 bytes and no NUL"
                );
                ensure!(
                    matches!(check.status, CheckStatus::NotRun) || check.evidence.is_some(),
                    "executed checks require evidence"
                );
                if let Some(evidence) = &check.evidence {
                    verify_file(&project, evidence)?;
                }
            }
            let submitted = serde_json::to_value(checks)?;
            if let Some(previous) = attempt.get("checks") {
                ensure!(
                    *previous == submitted,
                    "completed attempts are immutable; begin a new attempt"
                );
            } else if stale.is_none() {
                ensure!(
                    request.expected_trace_sha256.as_deref() == Some(&digest),
                    "trace changed; inspect before submitting"
                );
                trace["host_verification"]["attempts"][index]["checks"] = submitted.clone();
                trace["host_verification"]["attempts"][index]["finished_at"] =
                    json!(super::durable::now());
            }
            // Bind both sides of evidence validation; source may change while logs are read.
            if stale.is_none()
                && serde_json::to_value(manifest(&project, &inputs)?)?
                    != trace["host_verification"]["attempts"][index]["files"]
            {
                stale = Some(
                    "verification inputs changed during submission; begin a new attempt".into(),
                );
            }
            if stale.is_none()
                && serde_json::to_value(manifest(
                    &super::tools::design_scope(paths, &project)?,
                    &design_inputs,
                )?)? != design_expected
            {
                stale = Some("design inputs changed during submission; begin a new attempt".into());
            }
            if stale.is_none() {
                trace["host_verification"]["checks"] = submitted;
            }
        }
    } else {
        ensure!(
            request.checks.is_none(),
            "begin an input-bound attempt before executing and submitting checks"
        );
    }
    let status = if stale.is_some() {
        "verification_stale"
    } else if index.is_none() {
        "verification_unbound"
    } else if trace["host_verification"]
        .get("checks")
        .and_then(Value::as_array)
        .is_some_and(|checks| !checks.is_empty() && checks.iter().all(|c| c["status"] == "passed"))
    {
        "host_verified"
    } else {
        "awaiting_host_verification"
    };
    trace["status"] = json!(status);
    trace["host_verification"]["status"] = json!(status);
    trace["host_verification"]["stale_reason"] = json!(stale);
    if trace != original {
        let updated = serde_json::to_vec_pretty(&trace)?;
        ensure!(updated.len() <= 32 * 1024 * 1024, "run trace exceeds limit");
        ensure!(
            util::sha256(&util::read_bounded(&trace_path, 32 * 1024 * 1024)?) == digest,
            "trace changed during verification"
        );
        util::atomic_write_with_mode(&trace_path, &updated, Some(0o600))?;
        digest = util::sha256(&updated);
    }
    Ok(
        json!({"run_id":request.run_id,"trace_sha256":digest,"host_verification":projection(&trace["host_verification"]),"trace_path":trace_path,
        "status":trace["status"],"accepted":request.checks.is_some() && stale.is_none()}),
    )
}
