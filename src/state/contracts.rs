//! Opt-in evidence formats. File identities and caller declarations are not approval or authentication.
use super::{State, bounded};
use crate::{catalog::Catalog, util};
use anyhow::{Context, Result, bail, ensure};
use rmcp::schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

const MAX_CONTRACT_BYTES: usize = 128 * 1024;
const MAX_BINDINGS: usize = 64;
const MAX_HASH_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContractKind {
    Verification,
    Contributions,
}

impl ContractKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Verification => "verification",
            Self::Contributions => "contributions",
        }
    }
}

impl std::str::FromStr for ContractKind {
    type Err = anyhow::Error;
    fn from_str(value: &str) -> Result<Self> {
        match value {
            "verification" => Ok(Self::Verification),
            "contributions" => Ok(Self::Contributions),
            _ => bail!("contract must be verification or contributions"),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    path: String,
    sha256: String,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Outcome {
    Passed,
    Failed,
    Error,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Verification {
    schema_version: u32,
    task_id: String,
    inputs: Vec<Binding>,
    checker: Binding,
    result: Binding,
    outcome: Outcome,
    runtime: Option<String>,
    limitations: Vec<String>,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Status {
    Planned,
    Running,
    Unknown,
    Completed,
    Reused,
    Failed,
    Interrupted,
    Waived,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Disposition {
    Pending,
    Accepted,
    Rejected,
    Waived,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contribution {
    role: String,
    attempt: u32,
    result_attempt: Option<u32>,
    status: Status,
    host_agent_id: Option<String>,
    inputs: Vec<Binding>,
    evidence: Vec<Binding>,
    depends_on: Vec<String>,
    validation: String,
    unverified_paths: Vec<String>,
    reason: Option<String>,
    disposition: Disposition,
}

impl Contribution {
    fn acceptable(&self) -> bool {
        (matches!(self.status, Status::Completed | Status::Reused)
            && self.disposition == Disposition::Accepted)
            || (self.status == Status::Waived && self.disposition == Disposition::Waived)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contributions {
    schema_version: u32,
    task_id: String,
    roster: Binding,
    assignments: Vec<Contribution>,
    limitations: Vec<String>,
}

#[derive(Default)]
struct Files {
    bindings: usize,
    bytes: u64,
    checked: BTreeMap<String, (String, u64)>,
}

impl Files {
    fn check(&mut self, root: &Path, binding: &Binding) -> Result<()> {
        self.bindings += 1;
        ensure!(
            self.bindings <= MAX_BINDINGS,
            "contract exceeds 64 file bindings"
        );
        bounded(&binding.path, "binding path", 512)?;
        ensure!(
            binding.sha256.len() == 64
                && binding
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
            "binding SHA-256 must contain 64 lowercase hexadecimal characters"
        );
        if let Some((hash, _)) = self.checked.get(&binding.path) {
            ensure!(
                hash == &binding.sha256,
                "conflicting digests for binding {}",
                binding.path
            );
            return Ok(());
        }
        util::safe_relative(Path::new(&binding.path))?;
        let path = root.join(&binding.path);
        util::reject_symlink_components(&path)?;
        let metadata = fs::metadata(&path)?;
        ensure!(
            metadata.is_file(),
            "binding must be a regular file: {}",
            binding.path
        );
        ensure!(
            metadata.len() <= 16 * 1024 * 1024,
            "binding exceeds 16 MiB: {}",
            binding.path
        );
        ensure!(
            self.bytes + metadata.len() <= MAX_HASH_BYTES,
            "contract exceeds 64 MiB of unique bound files"
        );
        let (hash, size) = State::file_evidence(root, &binding.path)?;
        ensure!(
            size == metadata.len(),
            "binding changed while validating: {}",
            binding.path
        );
        ensure!(
            hash == binding.sha256,
            "stale binding: {} (SHA-256 changed)",
            binding.path
        );
        self.bytes += size;
        self.checked.insert(binding.path.clone(), (hash, size));
        Ok(())
    }

    fn list(&mut self, root: &Path, bindings: &[Binding], name: &str) -> Result<()> {
        ensure!(bindings.len() <= 32, "{name} exceeds 32 bindings");
        for binding in bindings {
            self.check(root, binding)?;
        }
        Ok(())
    }
}

fn strings(values: &[String], name: &str, count: usize, bytes: usize) -> Result<()> {
    ensure!(values.len() <= count, "{name} exceeds {count} items");
    for value in values {
        bounded(value, name, bytes)?;
    }
    Ok(())
}

fn header(version: u32, task_id: &str, task: &Value, limitations: &[String]) -> Result<()> {
    ensure!(
        version == 1,
        "unsupported contract schema_version; expected 1"
    );
    bounded(task_id, "contract task id", 64)?;
    ensure!(
        task["id"].as_str() == Some(task_id),
        "contract task_id does not match task"
    );
    strings(limitations, "limitations", 16, 2048)
}

/// Validate a bounded contract and its explicit file identities. Never executes a checker.
pub fn validate(
    root: &Path,
    task: &Value,
    kind: ContractKind,
    bytes: &[u8],
    completion: bool,
) -> Result<Value> {
    ensure!(
        bytes.len() <= MAX_CONTRACT_BYTES,
        "contract exceeds 128 KiB"
    );
    let mut files = Files::default();
    let details = match kind {
        ContractKind::Verification => {
            let contract: Verification =
                serde_json::from_slice(bytes).context("invalid verification contract")?;
            header(
                contract.schema_version,
                &contract.task_id,
                task,
                &contract.limitations,
            )?;
            ensure!(
                !contract.inputs.is_empty(),
                "verification requires at least one input"
            );
            if let Some(runtime) = &contract.runtime {
                bounded(runtime, "runtime", 1024)?;
            }
            files.list(root, &contract.inputs, "inputs")?;
            files.check(root, &contract.checker)?;
            files.check(root, &contract.result)?;
            if completion {
                ensure!(
                    contract.outcome == Outcome::Passed,
                    "verification outcome must be passed for completion"
                );
            }
            json!({"inputs":contract.inputs.len(), "outcome":match contract.outcome {Outcome::Passed => "passed", Outcome::Failed => "failed", Outcome::Error => "error"}})
        }
        ContractKind::Contributions => {
            let contract: Contributions =
                serde_json::from_slice(bytes).context("invalid contributions contract")?;
            header(
                contract.schema_version,
                &contract.task_id,
                task,
                &contract.limitations,
            )?;
            ensure!(
                (1..=13).contains(&contract.assignments.len()),
                "contributions require 1..=13 assignments"
            );
            files.check(root, &contract.roster)?;
            ensure!(
                files.checked[&contract.roster.path].1 <= MAX_CONTRACT_BYTES as u64,
                "roster exceeds 128 KiB"
            );
            let roster_bytes =
                util::read_bounded(&root.join(&contract.roster.path), MAX_CONTRACT_BYTES as u64)?;
            ensure!(
                util::sha256(&roster_bytes) == contract.roster.sha256,
                "roster changed while reading"
            );
            let (after_hash, _) = State::file_evidence(root, &contract.roster.path)?;
            ensure!(
                after_hash == contract.roster.sha256,
                "roster changed while reading"
            );
            let roster: Value =
                serde_json::from_slice(&roster_bytes).context("invalid roster JSON")?;
            let objective = task["objective"]
                .as_str()
                .context("task objective missing")?;
            ensure!(
                roster["objective"].as_str() == Some(objective),
                "roster objective does not match task"
            );
            let selected = task["skills"]
                .as_array()
                .context("task selected skills missing")?;
            let roster_assignments = roster["assignments"]
                .as_array()
                .context("roster assignments missing")?;
            ensure!(
                (1..=13).contains(&roster_assignments.len()),
                "roster requires 1..=13 assignments"
            );
            let catalog = Catalog::embedded()?;
            let mut expected = BTreeSet::new();
            for assignment in roster_assignments {
                let role = assignment["role"].as_str().context("roster role missing")?;
                let skill = assignment["skill"]
                    .as_str()
                    .context("roster skill missing")?;
                let canonical = catalog
                    .capabilities()
                    .iter()
                    .find(|candidate| candidate.id == role)
                    .context("roster role must be a canonical capability ID")?;
                ensure!(
                    canonical.lead == skill,
                    "roster role {role} has incorrect lead skill"
                );
                ensure!(
                    selected.iter().any(|s| s.as_str() == Some(skill)),
                    "roster skill {skill} is not selected for task"
                );
                ensure!(expected.insert(role), "duplicate roster role: {role}");
            }
            let mut actual = BTreeMap::new();
            for assignment in &contract.assignments {
                bounded(&assignment.role, "contribution role", 128)?;
                ensure!(
                    expected.contains(assignment.role.as_str()),
                    "contribution role absent from roster: {}",
                    assignment.role
                );
                ensure!(
                    actual
                        .insert(assignment.role.as_str(), assignment)
                        .is_none(),
                    "duplicate contribution role: {}",
                    assignment.role
                );
                ensure!(
                    assignment.attempt > 0,
                    "contribution attempt must be positive"
                );
                if let Some(result_attempt) = assignment.result_attempt {
                    ensure!(
                        result_attempt == assignment.attempt,
                        "stale result_attempt for {}",
                        assignment.role
                    );
                }
                if let Some(id) = &assignment.host_agent_id {
                    bounded(id, "host_agent_id", 256)?;
                }
                if let Some(reason) = &assignment.reason {
                    bounded(reason, "contribution reason", 2048)?;
                }
                ensure!(
                    assignment.validation.len() <= 2048 && !assignment.validation.contains('\0'),
                    "validation exceeds 2048 bytes or contains NUL"
                );
                strings(&assignment.unverified_paths, "unverified_paths", 16, 512)?;
                strings(&assignment.depends_on, "depends_on", 12, 128)?;
                let dependencies: BTreeSet<_> = assignment.depends_on.iter().collect();
                ensure!(
                    dependencies.len() == assignment.depends_on.len(),
                    "duplicate dependency for {}",
                    assignment.role
                );
                for dependency in &assignment.depends_on {
                    ensure!(
                        dependency != &assignment.role && expected.contains(dependency.as_str()),
                        "invalid dependency {dependency} for {}",
                        assignment.role
                    );
                }
                files.list(root, &assignment.inputs, "contribution inputs")?;
                files.list(root, &assignment.evidence, "contribution evidence")?;
                if matches!(assignment.status, Status::Completed | Status::Reused) {
                    ensure!(
                        assignment.host_agent_id.is_some()
                            && assignment.result_attempt == Some(assignment.attempt),
                        "completed/reused contribution requires host_agent_id and matching result_attempt"
                    );
                    ensure!(
                        !assignment.inputs.is_empty() && !assignment.evidence.is_empty(),
                        "completed/reused contribution requires inputs and evidence"
                    );
                    bounded(&assignment.validation, "completed/reused validation", 2048)?;
                } else {
                    ensure!(
                        assignment.disposition != Disposition::Accepted,
                        "only completed/reused contributions can be accepted"
                    );
                }
                if assignment.status == Status::Waived {
                    ensure!(
                        assignment.reason.is_some()
                            && assignment.disposition == Disposition::Waived,
                        "waiver requires reason and waived disposition"
                    );
                } else {
                    ensure!(
                        assignment.disposition != Disposition::Waived,
                        "waived disposition requires waived status"
                    );
                }
                if completion {
                    ensure!(
                        assignment.acceptable(),
                        "contribution {} is not accepted or waived for completion",
                        assignment.role
                    );
                }
            }
            ensure!(
                actual.len() == expected.len(),
                "contributions do not cover every roster assignment"
            );
            // Kahn's algorithm rejects cycles without recursion or executing a dependency.
            let mut remaining: BTreeSet<_> = actual.keys().copied().collect();
            while !remaining.is_empty() {
                let ready: Vec<_> = remaining
                    .iter()
                    .copied()
                    .filter(|role| {
                        actual[role]
                            .depends_on
                            .iter()
                            .all(|dependency| !remaining.contains(dependency.as_str()))
                    })
                    .collect();
                ensure!(
                    !ready.is_empty(),
                    "contribution dependencies contain a cycle"
                );
                for role in ready {
                    remaining.remove(role);
                }
            }
            json!({"assignments":actual.len(), "acceptable":actual.values().filter(|a| a.acceptable()).count()})
        }
    };
    Ok(
        json!({"contract":kind,"schema_version":1,"valid":true,"completion_checked":completion,"file_bindings":files.bindings,"unique_files":files.checked.len(),"bound_bytes":files.bytes,"details":details,
        "authority":"Validated caller-declared coverage and file identities. This does not authenticate the roster or workers, establish liveness or approval, execute a checker, or prove semantic correctness. Undeclared dependencies are not checked."}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct Fixture {
        _temp: TempDir,
        root: std::path::PathBuf,
        task: Value,
    }

    impl Fixture {
        fn new() -> Self {
            let temp = TempDir::new().unwrap();
            let root = temp.path().canonicalize().unwrap();
            for (path, bytes) in [
                ("input.rs", "original input"),
                ("check.rs", "checker"),
                ("result.json", "result"),
            ] {
                fs::write(root.join(path), bytes).unwrap();
            }
            let task = json!({"id":"t_example","objective":"Implement a bounded evidence contract","skills":["backend-engineering","security-judgment"]});
            fs::write(
                root.join("roster.json"),
                serde_json::to_vec(&json!({"objective":task["objective"],"assignments":[
                    {"role":"backend-engineering","skill":"backend-engineering"},
                    {"role":"security-engineering","skill":"security-judgment"}
                ]}))
                .unwrap(),
            )
            .unwrap();
            Self {
                _temp: temp,
                root,
                task,
            }
        }

        fn binding(&self, path: &str) -> Value {
            json!({"path":path,"sha256":util::sha256(&fs::read(self.root.join(path)).unwrap())})
        }

        fn verification(&self) -> Value {
            json!({"schema_version":1,"task_id":"t_example","inputs":[self.binding("input.rs")],"checker":self.binding("check.rs"),"result":self.binding("result.json"),"outcome":"passed","runtime":"fixture","limitations":[]})
        }

        fn contribution(&self, role: &str) -> Value {
            json!({"role":role,"attempt":2,"result_attempt":2,"status":"completed","host_agent_id":format!("host/{role}"),"inputs":[self.binding("input.rs")],"evidence":[self.binding("result.json")],"depends_on":[],"validation":"Observed fixture output","unverified_paths":[],"disposition":"accepted"})
        }

        fn contributions(&self) -> Value {
            json!({"schema_version":1,"task_id":"t_example","roster":self.binding("roster.json"),"assignments":[self.contribution("backend-engineering"),self.contribution("security-engineering")],"limitations":[]})
        }

        fn validate(&self, value: &Value, kind: ContractKind, completion: bool) -> Result<Value> {
            validate(
                &self.root,
                &self.task,
                kind,
                &serde_json::to_vec(value)?,
                completion,
            )
        }

        fn rebind_roster(&self, report: &mut Value, roster: Value) {
            fs::write(
                self.root.join("roster.json"),
                serde_json::to_vec(&roster).unwrap(),
            )
            .unwrap();
            report["roster"] = self.binding("roster.json");
        }
    }

    #[test]
    fn verification_tracks_only_declared_inputs_checker_and_result() {
        let f = Fixture::new();
        let contract = f.verification();
        assert_eq!(
            f.validate(&contract, ContractKind::Verification, true)
                .unwrap()["unique_files"],
            3
        );
        fs::write(f.root.join("unrelated.txt"), "unrelated modification").unwrap();
        assert!(
            f.validate(&contract, ContractKind::Verification, true)
                .is_ok()
        );
        for file in ["input.rs", "check.rs", "result.json"] {
            let original = fs::read(f.root.join(file)).unwrap();
            fs::write(f.root.join(file), "changed").unwrap();
            assert!(
                f.validate(&contract, ContractKind::Verification, false)
                    .unwrap_err()
                    .to_string()
                    .contains("stale binding")
            );
            fs::write(f.root.join(file), original).unwrap();
        }
    }

    #[test]
    fn unsuccessful_checks_register_but_do_not_complete() {
        let f = Fixture::new();
        for outcome in ["failed", "error"] {
            let mut contract = f.verification();
            contract["outcome"] = json!(outcome);
            assert!(
                f.validate(&contract, ContractKind::Verification, false)
                    .is_ok()
            );
            assert!(
                f.validate(&contract, ContractKind::Verification, true)
                    .unwrap_err()
                    .to_string()
                    .contains("must be passed")
            );
        }
    }

    #[test]
    fn strict_versioned_contract_rejects_wrong_task_empty_inputs_and_unknown_fields() {
        let f = Fixture::new();
        for (field, value) in [
            ("schema_version", json!(2)),
            ("task_id", json!("another-task")),
            ("inputs", json!([])),
            ("approval", json!(true)),
            ("runtime", json!("")),
        ] {
            let mut contract = f.verification();
            contract[field] = value;
            assert!(
                f.validate(&contract, ContractKind::Verification, false)
                    .is_err(),
                "accepted {field}"
            );
        }
        let mut contract = f.verification();
        contract["checker"]["command"] = json!("must not execute");
        assert!(
            f.validate(&contract, ContractKind::Verification, false)
                .is_err()
        );
        assert!(
            validate(
                &f.root,
                &f.task,
                ContractKind::Verification,
                &vec![b' '; MAX_CONTRACT_BYTES + 1],
                false
            )
            .is_err()
        );
    }

    #[test]
    fn traversal_absolute_directory_and_symlink_bindings_are_rejected() {
        let f = Fixture::new();
        for path in ["../input.rs", "/input.rs", "."] {
            let mut contract = f.verification();
            contract["inputs"][0]["path"] = json!(path);
            assert!(
                f.validate(&contract, ContractKind::Verification, false)
                    .is_err(),
                "accepted {path}"
            );
        }
        fs::create_dir(f.root.join("directory")).unwrap();
        let mut contract = f.verification();
        contract["inputs"][0]["path"] = json!("directory");
        assert!(
            f.validate(&contract, ContractKind::Verification, false)
                .is_err()
        );
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(f.root.join("input.rs"), f.root.join("linked.rs")).unwrap();
            contract["inputs"][0]["path"] = json!("linked.rs");
            assert!(
                f.validate(&contract, ContractKind::Verification, false)
                    .is_err()
            );
        }
    }

    #[test]
    fn contribution_coverage_attempts_and_dependencies_are_checked() {
        let f = Fixture::new();
        let mut contract = f.contributions();
        contract["assignments"][0]["depends_on"] = json!(["security-engineering"]);
        let summary = f
            .validate(&contract, ContractKind::Contributions, true)
            .unwrap();
        assert_eq!(summary["details"]["acceptable"], 2);
        assert_eq!(summary["file_bindings"], 5);
        assert_eq!(summary["unique_files"], 3);
        let valid = contract.clone();
        contract["assignments"].as_array_mut().unwrap().pop();
        assert!(
            f.validate(&contract, ContractKind::Contributions, false)
                .unwrap_err()
                .to_string()
                .contains("cover every")
        );
        contract = valid.clone();
        contract["assignments"][1] = contract["assignments"][0].clone();
        assert!(
            f.validate(&contract, ContractKind::Contributions, false)
                .unwrap_err()
                .to_string()
                .contains("duplicate")
        );
        contract = valid.clone();
        contract["assignments"][1]["result_attempt"] = json!(1);
        assert!(
            f.validate(&contract, ContractKind::Contributions, false)
                .unwrap_err()
                .to_string()
                .contains("stale result_attempt")
        );
        contract = valid.clone();
        contract["assignments"][1]["depends_on"] = json!(["backend-engineering"]);
        assert!(
            f.validate(&contract, ContractKind::Contributions, false)
                .unwrap_err()
                .to_string()
                .contains("cycle")
        );
        contract = valid;
        contract["assignments"][1]["depends_on"] = json!(["unselected-role"]);
        assert!(
            f.validate(&contract, ContractKind::Contributions, false)
                .unwrap_err()
                .to_string()
                .contains("invalid dependency")
        );
    }

    #[test]
    fn partial_and_unknown_worker_reports_are_retained_without_claiming_completion() {
        let f = Fixture::new();
        for status in ["planned", "running", "unknown", "failed", "interrupted"] {
            let mut contract = f.contributions();
            let worker = &mut contract["assignments"][1];
            worker["status"] = json!(status);
            worker["disposition"] = json!("pending");
            worker["result_attempt"] = Value::Null;
            worker["host_agent_id"] = Value::Null;
            worker["inputs"] = json!([]);
            worker["evidence"] = json!([]);
            worker["validation"] = json!("");
            assert!(
                f.validate(&contract, ContractKind::Contributions, false)
                    .is_ok(),
                "rejected {status}"
            );
            assert!(
                f.validate(&contract, ContractKind::Contributions, true)
                    .is_err(),
                "completed {status}"
            );
        }
    }

    #[test]
    fn terminal_reports_require_host_identity_evidence_and_explicit_disposition() {
        let f = Fixture::new();
        for status in ["completed", "reused"] {
            let mut contract = f.contributions();
            contract["assignments"][1]["status"] = json!(status);
            assert!(
                f.validate(&contract, ContractKind::Contributions, true)
                    .is_ok()
            );
            for (field, value) in [
                ("host_agent_id", Value::Null),
                ("result_attempt", Value::Null),
                ("inputs", json!([])),
                ("evidence", json!([])),
                ("validation", json!(" ")),
            ] {
                let mut invalid = contract.clone();
                invalid["assignments"][1][field] = value;
                assert!(
                    f.validate(&invalid, ContractKind::Contributions, false)
                        .is_err(),
                    "accepted {status}/{field}"
                );
            }
            contract["assignments"][1]["disposition"] = json!("rejected");
            assert!(
                f.validate(&contract, ContractKind::Contributions, false)
                    .is_ok()
            );
            assert!(
                f.validate(&contract, ContractKind::Contributions, true)
                    .is_err()
            );
        }
        let mut contract = f.contributions();
        let worker = &mut contract["assignments"][1];
        worker["status"] = json!("waived");
        worker["disposition"] = json!("waived");
        assert!(
            f.validate(&contract, ContractKind::Contributions, false)
                .is_err()
        );
        contract["assignments"][1]["reason"] = json!("User explicitly narrowed this assignment");
        assert!(
            f.validate(&contract, ContractKind::Contributions, true)
                .is_ok()
        );
    }

    #[test]
    fn roster_binding_objective_skills_and_canonical_roles_prevent_forged_coverage() {
        let f = Fixture::new();
        let original: Value =
            serde_json::from_slice(&fs::read(f.root.join("roster.json")).unwrap()).unwrap();
        for mutation in 0..5 {
            let mut roster = original.clone();
            match mutation {
                0 => roster["objective"] = json!("A different objective"),
                1 => roster["assignments"][1]["role"] = json!("invented-role"),
                2 => roster["assignments"][1]["skill"] = json!("backend-engineering"),
                3 => roster["assignments"][1] = roster["assignments"][0].clone(),
                _ => {
                    roster["assignments"][1] =
                        json!({"role":"frontend-engineering","skill":"frontend-engineering"})
                }
            }
            let mut contract = f.contributions();
            f.rebind_roster(&mut contract, roster);
            assert!(
                f.validate(&contract, ContractKind::Contributions, false)
                    .is_err(),
                "accepted roster mutation {mutation}"
            );
        }
        let mut contract = f.contributions();
        f.rebind_roster(&mut contract, original);
        fs::write(f.root.join("roster.json"), "altered after binding").unwrap();
        assert!(
            f.validate(&contract, ContractKind::Contributions, false)
                .unwrap_err()
                .to_string()
                .contains("stale binding")
        );
    }

    #[test]
    fn binding_work_is_bounded_and_conflicting_shared_digests_are_rejected() {
        let f = Fixture::new();
        let binding: Binding = serde_json::from_value(f.binding("input.rs")).unwrap();
        let mut files = Files::default();
        for _ in 0..MAX_BINDINGS {
            files.check(&f.root, &binding).unwrap();
        }
        assert_eq!(files.checked.len(), 1);
        assert!(
            files
                .check(&f.root, &binding)
                .unwrap_err()
                .to_string()
                .contains("64 file bindings")
        );
        let mut files = Files {
            bytes: MAX_HASH_BYTES,
            ..Files::default()
        };
        assert!(
            files
                .check(&f.root, &binding)
                .unwrap_err()
                .to_string()
                .contains("64 MiB")
        );
        let mut contract = f.verification();
        contract["checker"] = contract["inputs"][0].clone();
        contract["checker"]["sha256"] = json!("0".repeat(64));
        assert!(
            f.validate(&contract, ContractKind::Verification, false)
                .unwrap_err()
                .to_string()
                .contains("conflicting digests")
        );
    }
}
