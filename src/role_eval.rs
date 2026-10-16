//! Bounded role-evaluation records. These checks do not launch or grade a model.
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub path: String,
    pub sha256: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    pub id: String,
    pub file: Binding,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub split: String,
    pub input: Binding,
    pub rubric: Binding,
    pub criteria: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Suite {
    pub schema_version: u32,
    pub id: String,
    pub checker: Binding,
    pub configurations: Vec<Configuration>,
    pub trials_per_case: u32,
    pub development: Vec<String>,
    pub held_out: Vec<String>,
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Trial {
    pub case_id: String,
    pub configuration: String,
    pub trial: u32,
    pub input_sha256: String,
    pub rubric_sha256: String,
    pub configuration_sha256: String,
    pub outcome: String,
    pub score: Option<f64>,
    pub verdicts: BTreeMap<String, bool>,
    pub evidence: Vec<Binding>,
    pub rationale: String,
    pub cost_usd: Option<f64>,
    pub latency_ms: Option<f64>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunReport {
    pub schema_version: u32,
    pub provenance: String,
    pub suite_sha256: String,
    pub checker_sha256: String,
    pub trials: Vec<Trial>,
    pub limitations: Vec<String>,
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn file_identity(path: &Path) -> Result<String> {
    let metadata = fs::metadata(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(format!("{}:{}", metadata.dev(), metadata.ino()))
    }
    #[cfg(not(unix))]
    {
        Ok(format!("{:?}", metadata.created()?))
    }
}
fn bounded_file(project: &Path, path: &Path) -> Result<Vec<u8>> {
    ensure!(
        !path.as_os_str().is_empty()
            && path.components().all(|c| matches!(c, Component::Normal(_))),
        "file paths must be normal project-relative components"
    );
    let current = project.canonicalize()?.join(path);
    crate::util::reject_symlink_components(&current)?;
    let metadata = fs::metadata(&current)?;
    let identity = file_identity(&current)?;
    let bytes = crate::util::read_bounded(&current, 1024 * 1024)?;
    let after = fs::metadata(&current)?;
    crate::util::reject_symlink_components(&current)?;
    ensure!(
        identity == file_identity(&current)?
            && metadata.len() == after.len()
            && metadata.modified()? == after.modified()?,
        "evaluation file changed while reading"
    );
    Ok(bytes)
}
fn verify_binding(project: &Path, binding: &Binding) -> Result<Vec<u8>> {
    ensure!(
        binding.sha256.len() == 64
            && binding
                .sha256
                .bytes()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "invalid SHA-256: {}",
        binding.path
    );
    let bytes = bounded_file(project, Path::new(&binding.path))?;
    ensure!(
        digest(&bytes) == binding.sha256,
        "stale evaluation binding: {}",
        binding.path
    );
    Ok(bytes)
}
fn nonempty_unique(values: &[String], label: &str) -> Result<BTreeSet<String>> {
    let set: BTreeSet<_> = values.iter().cloned().collect();
    ensure!(
        !set.is_empty() && set.len() == values.len() && set.iter().all(|v| !v.trim().is_empty()),
        "empty or duplicate {label}"
    );
    Ok(set)
}
fn validate_suite(suite: &Suite) -> Result<()> {
    ensure!(
        suite.schema_version == 1 && !suite.id.trim().is_empty(),
        "unsupported or unnamed suite"
    );
    ensure!(
        (1..=100).contains(&suite.trials_per_case)
            && (1..=32).contains(&suite.cases.len())
            && (1..=8).contains(&suite.configurations.len()),
        "suite exceeds bounded trial/case/configuration limits"
    );
    let dev = if suite.development.is_empty() {
        BTreeSet::new()
    } else {
        nonempty_unique(&suite.development, "development cases")?
    };
    let held = nonempty_unique(&suite.held_out, "held-out cases")?;
    ensure!(dev.is_disjoint(&held), "development/held-out split overlap");
    let ids = nonempty_unique(
        &suite.cases.iter().map(|c| c.id.clone()).collect::<Vec<_>>(),
        "case IDs",
    )?;
    ensure!(
        ids == dev.union(&held).cloned().collect(),
        "split IDs do not match case IDs"
    );
    nonempty_unique(
        &suite
            .configurations
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>(),
        "configuration IDs",
    )?;
    let dev_hashes: BTreeSet<_> = suite
        .cases
        .iter()
        .filter(|c| dev.contains(&c.id))
        .map(|c| &c.input.sha256)
        .collect();
    ensure!(
        !suite
            .cases
            .iter()
            .any(|c| held.contains(&c.id) && dev_hashes.contains(&c.input.sha256)),
        "development/held-out input overlap"
    );
    for case in &suite.cases {
        ensure!(
            case.split
                == if dev.contains(&case.id) {
                    "development"
                } else {
                    "held_out"
                },
            "case split mismatch: {}",
            case.id
        );
        ensure!(case.criteria.len() <= 32, "too many criteria");
        nonempty_unique(&case.criteria, "rubric criteria")?;
    }
    Ok(())
}
fn load_suite(project: &Path, path: &Path) -> Result<(Suite, String)> {
    let bytes = bounded_file(project, path)?;
    let suite: Suite = serde_json::from_slice(&bytes).context("invalid evaluation suite")?;
    validate_suite(&suite)?;
    verify_binding(project, &suite.checker)?;
    ensure!(
        suite.checker.sha256 == digest(include_bytes!("role_eval.rs")),
        "suite checker differs from compiled checker"
    );
    for config in &suite.configurations {
        verify_binding(project, &config.file)?;
    }
    for case in &suite.cases {
        verify_binding(project, &case.input)?;
        let rubric: Value = serde_json::from_slice(&verify_binding(project, &case.rubric)?)?;
        let criteria: Vec<String> = serde_json::from_value(
            rubric
                .get("criteria")
                .cloned()
                .context("rubric has no criteria")?,
        )?;
        ensure!(
            criteria == case.criteria,
            "rubric criteria mismatch: {}",
            case.id
        );
    }
    Ok((suite, digest(&bytes)))
}
/// Inspect pinned fixtures; no host/model execution or mutation.
pub fn suite(project: &Path, suite_path: &Path) -> Result<Value> {
    let (suite, hash) = load_suite(project, suite_path)?;
    Ok(
        json!({"schema_version":1,"suite_sha256":hash,"expected_cells":suite.cases.len()*suite.configurations.len()*suite.trials_per_case as usize,"suite":suite,"execution":"not_dispatched","held_out_visibility":"public fixtures; split separation does not establish blindness"}),
    )
}
/// Pure record validator and deterministic aggregator. Hashes must already be checked by the caller.
pub fn validate(suite: &Suite, suite_sha256: &str, report: &RunReport) -> Result<Value> {
    validate_suite(suite)?;
    ensure!(report.schema_version == 1, "unsupported report version");
    ensure!(
        ["synthetic", "host_observed"].contains(&report.provenance.as_str()),
        "unknown report provenance"
    );
    ensure!(
        report.suite_sha256 == suite_sha256 && report.checker_sha256 == suite.checker.sha256,
        "suite/checker identity mismatch"
    );
    ensure!(
        !report.limitations.is_empty(),
        "report must state limitations"
    );
    let expected = suite.cases.len() * suite.configurations.len() * suite.trials_per_case as usize;
    ensure!(report.trials.len() <= expected, "too many trial cells");
    let mut seen = BTreeSet::new();
    let mut groups: BTreeMap<(String, String), Vec<&Trial>> = BTreeMap::new();
    for trial in &report.trials {
        let case = suite
            .cases
            .iter()
            .find(|c| c.id == trial.case_id)
            .context("unknown case ID")?;
        let config = suite
            .configurations
            .iter()
            .find(|c| c.id == trial.configuration)
            .context("unknown configuration")?;
        ensure!(
            (1..=suite.trials_per_case).contains(&trial.trial),
            "trial index outside planned range"
        );
        ensure!(
            seen.insert((&trial.case_id, &trial.configuration, trial.trial)),
            "duplicate trial cell"
        );
        ensure!(
            trial.input_sha256 == case.input.sha256
                && trial.rubric_sha256 == case.rubric.sha256
                && trial.configuration_sha256 == config.file.sha256,
            "input/rubric/configuration identity mismatch"
        );
        ensure!(
            !trial.rationale.trim().is_empty(),
            "trial requires rationale"
        );
        for value in [trial.cost_usd, trial.latency_ms].into_iter().flatten() {
            ensure!(
                value.is_finite() && (0.0..=1e15).contains(&value),
                "invalid cost or latency (expected 0..1e15)"
            );
        }
        ensure!(trial.evidence.len() <= 16, "too many evidence bindings");
        match trial.outcome.as_str() {
            "infrastructure_failed" => ensure!(
                trial.score.is_none() && trial.verdicts.is_empty(),
                "infrastructure failure must remain unscored"
            ),
            "passed" | "task_failed" => {
                ensure!(
                    !trial.evidence.is_empty() && trial.evidence.len() <= 16,
                    "scored trial requires bounded saved evidence"
                );
                ensure!(
                    trial.verdicts.keys().cloned().collect::<BTreeSet<_>>()
                        == case.criteria.iter().cloned().collect(),
                    "missing or unknown rubric verdict"
                );
                let score = trial.verdicts.values().filter(|v| **v).count() as f64
                    / case.criteria.len() as f64;
                ensure!(
                    trial.score.is_some_and(|v| v.is_finite()
                        && (0.0..=1.0).contains(&v)
                        && (v - score).abs() < 1e-12),
                    "invalid or inconsistent score"
                );
                ensure!(
                    (trial.outcome == "passed") == (score == 1.0),
                    "outcome inconsistent with rubric verdicts"
                );
            }
            _ => bail!("unknown trial outcome"),
        }
        groups
            .entry((config.id.clone(), case.split.clone()))
            .or_default()
            .push(trial);
    }
    ensure!(
        seen.len() == expected,
        "missing trial cells: expected {expected}, got {}",
        seen.len()
    );
    let aggregates: Vec<Value> = groups.into_iter().map(|((configuration, split), trials)| {
        let scored: Vec<f64> = trials.iter().filter_map(|t| t.score).collect();
        let costs: Vec<f64> = trials.iter().filter_map(|t| t.cost_usd).collect();
        let latency: Vec<f64> = trials.iter().filter_map(|t| t.latency_ms).collect();
        json!({"configuration":configuration,"split":split,"trials":trials.len(),"passed":trials.iter().filter(|t| t.outcome=="passed").count(),"task_failed":trials.iter().filter(|t| t.outcome=="task_failed").count(),"infrastructure_failed":trials.len()-scored.len(),"scored_trials":scored.len(),"mean_score":if scored.is_empty(){None}else{Some(scored.iter().sum::<f64>()/scored.len() as f64)},"known_cost_usd":costs.iter().sum::<f64>(),"unknown_cost_trials":trials.len()-costs.len(),"total_cost_usd":if costs.len()==trials.len(){Some(costs.iter().sum::<f64>())}else{None},"known_latency_trials":latency.len(),"mean_known_latency_ms":if latency.is_empty(){None}else{Some(latency.iter().sum::<f64>()/latency.len() as f64)}})
    }).collect();
    Ok(
        json!({"schema_version":1,"valid":true,"provenance":report.provenance,"aggregates":aggregates,"limits":"Verifies record consistency and current declared file identities; does not authenticate execution, grade semantics, or establish general role-quality improvement."}),
    )
}
/// Check current file identities before applying the pure record validator.
pub fn check(project: &Path, suite_path: &Path, report_path: &Path) -> Result<Value> {
    let (suite, hash) = load_suite(project, suite_path)?;
    let report: RunReport = serde_json::from_slice(&bounded_file(project, report_path)?)
        .context("invalid evaluation report")?;
    let result = validate(&suite, &hash, &report)?;
    if report.provenance == "host_observed" {
        for config in &suite.configurations {
            let contents: Value = serde_json::from_slice(&verify_binding(project, &config.file)?)?;
            ensure!(
                contents.get("status").and_then(Value::as_str) != Some("template_not_run"),
                "host-observed report cannot use an unfilled template configuration"
            );
            for field in ["host", "model", "authority", "harness_revision"] {
                ensure!(
                    contents
                        .get(field)
                        .and_then(Value::as_str)
                        .is_some_and(|s| !s.trim().is_empty()),
                    "host-observed configuration requires {field}"
                );
            }
            for field in ["model_settings", "budget", "guidance_hashes"] {
                ensure!(
                    contents.get(field).is_some_and(Value::is_object),
                    "host-observed configuration requires explicit {field} object"
                );
            }
            ensure!(
                contents.get("tools").is_some_and(Value::is_array),
                "host-observed configuration requires tool inventory"
            );
        }
    }
    let mut cache = BTreeMap::new();
    let mut total_bytes = 0usize;
    for trial in &report.trials {
        for evidence in &trial.evidence {
            if let Some(hash) = cache.get(&evidence.path) {
                ensure!(hash == &evidence.sha256, "conflicting evidence identities");
            } else {
                let bytes = verify_binding(project, evidence)?;
                total_bytes += bytes.len();
                ensure!(
                    total_bytes <= 64 * 1024 * 1024,
                    "evaluation evidence exceeds 64 MiB unique bytes"
                );
                cache.insert(evidence.path.clone(), evidence.sha256.clone());
            }
        }
    }
    Ok(result)
}
