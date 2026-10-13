use mirket::{
    catalog::Catalog,
    paths::Paths,
    state::{State, contracts::ContractKind},
    util,
};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

struct Fixture {
    _home: tempfile::TempDir,
    paths: Paths,
    project: PathBuf,
    state: State,
    project_id: String,
    task_id: String,
}

impl Fixture {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
        let project = paths.home.join("project");
        fs::create_dir(&project).unwrap();
        let state = State::open(&paths).unwrap();
        let project_id = state.register_project(&project).unwrap()["id"]
            .as_str()
            .unwrap()
            .to_owned();
        let task_id = state
            .begin(
                &project_id,
                "Verify source-bound work",
                &["engineering-judgment".into()],
                "Current source passes its checker",
                "Changed inputs invalidate the receipt",
                "start",
            )
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_owned();
        let f = Self {
            _home: home,
            paths,
            project,
            state,
            project_id,
            task_id,
        };
        f.write("input.txt", "price=10");
        f.write("checker.py", "assert price == 10");
        f.write("result.txt", "passed: expected price 10");
        f.write("failure.txt", "changing price to 11 fails the checker");
        f
    }
    fn write(&self, path: &str, content: &str) {
        fs::write(self.project.join(path), content).unwrap();
    }
    fn binding(&self, path: &str) -> Value {
        json!({"path":path,"sha256":util::sha256(&fs::read(self.project.join(path)).unwrap())})
    }
    fn receipt(&self, outcome: &str) {
        self.write("verification.json", &serde_json::to_string_pretty(&json!({
            "schema_version":1,"task_id":self.task_id,
            "inputs":[self.binding("input.txt")],"checker":self.binding("checker.py"),
            "result":self.binding("result.txt"),"outcome":outcome,
            "runtime":"fixture checker; not executed by Mirket", "limitations":["Checks identity only"]
        })).unwrap());
    }
    fn revision(&self) -> i64 {
        self.state.status(&self.task_id).unwrap()["revision"]
            .as_i64()
            .unwrap()
    }
    fn ready(&self, outcome: &str) {
        let skill = Catalog::embedded()
            .unwrap()
            .read("engineering-judgment", "SKILL.md", 65536)
            .unwrap();
        self.state
            .record_skill(
                &self.task_id,
                self.revision(),
                "engineering-judgment",
                &skill.sha256,
                "read",
            )
            .unwrap();
        self.receipt(outcome);
        self.state
            .evidence_with_contract(
                &self.task_id,
                self.revision(),
                "verification.json",
                "acceptance",
                "Input-bound result",
                "typed",
                Some(ContractKind::Verification),
            )
            .unwrap();
        self.state
            .evidence(
                &self.task_id,
                self.revision(),
                "failure.txt",
                "failure_probe",
                "Observed mismatched-price failure",
                "failure",
            )
            .unwrap();
    }
}

#[test]
fn completion_rechecks_each_bound_file_but_ignores_unrelated_changes() {
    for path in ["input.txt", "checker.py", "result.txt"] {
        let f = Fixture::new();
        f.ready("passed");
        let revision = f.revision();
        let before = fs::read(f.project.join(path)).unwrap();
        f.write(path, "changed after verification");
        // The receipt bytes are unchanged; checking only the attached report's
        // hash would falsely accept this stale result.
        let error = f
            .state
            .complete(&f.task_id, revision, "finish")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("stale") && error.contains(path),
            "{path}: {error}"
        );
        assert_eq!(f.revision(), revision);
        assert_eq!(f.state.status(&f.task_id).unwrap()["status"], "active");
        fs::write(f.project.join(path), before).unwrap();
        f.write("unrelated.txt", "Does not affect the declared result");
        assert_eq!(
            f.state.complete(&f.task_id, revision, "finish").unwrap()["status"],
            "complete"
        );
    }
}

#[test]
fn failed_and_infrastructure_error_receipts_are_retained_but_cannot_complete() {
    for outcome in ["failed", "error"] {
        let f = Fixture::new();
        f.ready(outcome);
        assert!(
            f.state
                .validate_contract(
                    &f.task_id,
                    "verification.json",
                    ContractKind::Verification,
                    false
                )
                .is_ok()
        );
        let error = f
            .state
            .validate_contract(
                &f.task_id,
                "verification.json",
                ContractKind::Verification,
                true,
            )
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("outcome") && error.contains("passed"),
            "{error}"
        );
        assert!(
            f.state
                .complete(&f.task_id, f.revision(), "finish")
                .is_err()
        );
        let status = f.state.status(&f.task_id).unwrap();
        assert_eq!(status["evidence"].as_array().unwrap().len(), 2);
        assert_eq!(status["status"], "active");
        f.receipt("passed");
        f.state
            .evidence_with_contract(
                &f.task_id,
                f.revision(),
                "verification.json",
                "acceptance",
                "Corrected result after rerun",
                "rerun",
                Some(ContractKind::Verification),
            )
            .unwrap();
        assert_eq!(
            f.state
                .complete(&f.task_id, f.revision(), "finish")
                .unwrap()["status"],
            "complete"
        );
    }
}

#[test]
fn ordinary_evidence_cannot_remove_an_existing_typed_contract() {
    let f = Fixture::new();
    f.ready("passed");
    let revision = f.revision();
    f.write("input.txt", "stale source");
    // Re-recording the receipt without a type cannot remove validation. Both
    // rejecting this request and preserving/revalidating the type are valid.
    let _ = f.state.evidence(
        &f.task_id,
        revision,
        "verification.json",
        "acceptance",
        "Attempt ordinary replacement",
        "downgrade",
    );
    let error = f
        .state
        .complete(&f.task_id, f.revision(), "finish")
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("stale") && error.contains("input.txt"),
        "{error}"
    );
    // Ordinary JSON remains ordinary unless a contract was explicitly chosen.
    let ordinary = Fixture::new();
    ordinary.write(
        "arbitrary.json",
        "{\"schema_version\":999,\"outcome\":\"failed\"}",
    );
    ordinary
        .state
        .evidence(
            &ordinary.task_id,
            ordinary.revision(),
            "arbitrary.json",
            "acceptance",
            "Caller-supplied JSON",
            "ordinary",
        )
        .unwrap();
}

#[test]
fn read_only_validation_changes_neither_revision_nor_evidence() {
    let f = Fixture::new();
    f.receipt("passed");
    let before = f.state.status(&f.task_id).unwrap();
    f.state
        .validate_contract(
            &f.task_id,
            "verification.json",
            ContractKind::Verification,
            true,
        )
        .unwrap();
    assert_eq!(f.state.status(&f.task_id).unwrap(), before);
    f.write("input.txt", "changed");
    assert!(
        f.state
            .validate_contract(
                &f.task_id,
                "verification.json",
                ContractKind::Verification,
                false
            )
            .is_err()
    );
    assert_eq!(f.state.status(&f.task_id).unwrap(), before);
}

fn restore_v1_schema(f: &Fixture) {
    // v2 adds only this separate table. Dropping it from a fixture with no typed
    // evidence reproduces the exact v1 layout while retaining genuine records.
    let database = Connection::open(f.paths.database()).unwrap();
    database
        .execute_batch("DROP TABLE contracts; PRAGMA user_version=1;")
        .unwrap();
}

#[test]
fn v1_migration_preserves_task_history_evidence_and_idempotent_replies() {
    let f = Fixture::new();
    let evidence_reply = f
        .state
        .evidence(
            &f.task_id,
            1,
            "result.txt",
            "acceptance",
            "Legacy evidence",
            "legacy-evidence",
        )
        .unwrap();
    f.state
        .checkpoint(
            &f.task_id,
            2,
            "Original requirement retained",
            "legacy-note",
        )
        .unwrap();
    let expected = f.state.status(&f.task_id).unwrap();
    restore_v1_schema(&f);
    let migrated = State::open(&f.paths).unwrap();
    assert_eq!(migrated.status(&f.task_id).unwrap(), expected);
    assert_eq!(
        migrated
            .evidence(
                &f.task_id,
                1,
                "result.txt",
                "acceptance",
                "Legacy evidence",
                "legacy-evidence"
            )
            .unwrap(),
        evidence_reply
    );
    assert_eq!(
        migrated.task_history(&f.task_id, None, 10).unwrap()["checkpoints"][0]["note"],
        "Original requirement retained"
    );
    assert_eq!(
        migrated.tasks(&f.project_id, None, 10).unwrap()["tasks"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let database = Connection::open(f.paths.database()).unwrap();
    assert_eq!(
        database
            .pragma_query_value::<i64, _>(None, "user_version", |row| row.get(0))
            .unwrap(),
        2
    );
    assert_eq!(
        database
            .query_row::<i64, _, _>("SELECT count(*) FROM contracts", [], |row| row.get(0))
            .unwrap(),
        0
    );
}

#[test]
fn malformed_v1_is_rejected_without_partial_migration() {
    let f = Fixture::new();
    restore_v1_schema(&f);
    let database = Connection::open(f.paths.database()).unwrap();
    database
        .execute_batch("CREATE TABLE unrecognized(payload TEXT);")
        .unwrap();
    assert!(State::open(&f.paths).is_err());
    assert_eq!(
        database
            .pragma_query_value::<i64, _>(None, "user_version", |row| row.get(0))
            .unwrap(),
        1
    );
    assert_eq!(
        database
            .query_row::<i64, _, _>(
                "SELECT count(*) FROM sqlite_schema WHERE name='contracts'",
                [],
                |row| row.get(0)
            )
            .unwrap(),
        0
    );
    assert_eq!(
        database
            .query_row::<String, _, _>(
                "SELECT objective FROM tasks WHERE id=?1",
                [&f.task_id],
                |row| row.get(0)
            )
            .unwrap(),
        "Verify source-bound work"
    );
}

#[test]
fn contribution_registration_keeps_original_roster_and_completion_requires_every_role() {
    let f = Fixture::new();
    let task = f
        .state
        .begin(
            &f.project_id,
            "Integrate two specialists",
            &["backend-engineering".into(), "security-judgment".into()],
            "Both assignments are accounted",
            "Dropping an assignment cannot bypass its result",
            "team-start",
        )
        .unwrap();
    let task_id = task["id"].as_str().unwrap();
    let revision = || {
        f.state.status(task_id).unwrap()["revision"]
            .as_i64()
            .unwrap()
    };
    for skill_id in ["backend-engineering", "security-judgment"] {
        let skill = Catalog::embedded()
            .unwrap()
            .read(skill_id, "SKILL.md", 65536)
            .unwrap();
        f.state
            .record_skill(task_id, revision(), skill_id, &skill.sha256, skill_id)
            .unwrap();
    }
    let roster = json!({"objective":"Integrate two specialists", "assignments":[
        {"role":"backend-engineering","skill":"backend-engineering"},
        {"role":"security-engineering","skill":"security-judgment"}
    ]});
    f.write("roster.json", &serde_json::to_string(&roster).unwrap());
    let assignment = |role: &str| {
        json!({
            "role":role,"attempt":1,"status":"planned","inputs":[],"evidence":[],
            "depends_on":[],"validation":"","unverified_paths":[],"disposition":"pending"
        })
    };
    let mut receipt = json!({"schema_version":1,"task_id":task_id,"roster":f.binding("roster.json"),
        "assignments":[assignment("backend-engineering"),assignment("security-engineering")],"limitations":[]});
    f.write(
        "contributions.json",
        &serde_json::to_string(&receipt).unwrap(),
    );
    f.state
        .evidence_with_contract(
            task_id,
            revision(),
            "contributions.json",
            "acceptance",
            "Team progress",
            "team-progress",
            Some(ContractKind::Contributions),
        )
        .unwrap();
    f.state
        .evidence(
            task_id,
            revision(),
            "failure.txt",
            "failure_probe",
            "Failure observation",
            "team-failure",
        )
        .unwrap();
    let before = revision();
    let error = f
        .state
        .complete(task_id, before, "team-finish")
        .unwrap_err()
        .to_string();
    assert!(error.contains("not accepted"), "{error}");

    // A newly coherent but smaller roster would pass the format validator.
    // The persisted contract must prevent silently replacing the original team.
    let mut smaller = roster.clone();
    smaller["assignments"].as_array_mut().unwrap().pop();
    f.write("roster.json", &serde_json::to_string(&smaller).unwrap());
    let mut rebound = receipt.clone();
    rebound["roster"] = f.binding("roster.json");
    rebound["assignments"].as_array_mut().unwrap().pop();
    f.write(
        "contributions.json",
        &serde_json::to_string(&rebound).unwrap(),
    );
    let error = f
        .state
        .evidence_with_contract(
            task_id,
            before,
            "contributions.json",
            "acceptance",
            "Drop the unexecuted review",
            "team-rebind",
            Some(ContractKind::Contributions),
        )
        .unwrap_err()
        .to_string();
    assert!(error.contains("originally recorded roster"), "{error}");
    assert_eq!(revision(), before);

    f.write("roster.json", &serde_json::to_string(&roster).unwrap());
    for assignment in receipt["assignments"].as_array_mut().unwrap() {
        assignment["status"] = json!("completed");
        assignment["disposition"] = json!("accepted");
        assignment["result_attempt"] = json!(1);
        assignment["host_agent_id"] =
            json!(format!("fixture-{}", assignment["role"].as_str().unwrap()));
        assignment["inputs"] = json!([f.binding("input.txt")]);
        assignment["evidence"] = json!([f.binding("result.txt")]);
        assignment["validation"] =
            json!("The fixture records completed outputs; no model execution is claimed");
    }
    f.write(
        "contributions.json",
        &serde_json::to_string(&receipt).unwrap(),
    );
    f.state
        .evidence_with_contract(
            task_id,
            before,
            "contributions.json",
            "acceptance",
            "Both results accepted",
            "team-results",
            Some(ContractKind::Contributions),
        )
        .unwrap();
    assert_eq!(
        f.state
            .complete(task_id, revision(), "team-finish")
            .unwrap()["status"],
        "complete"
    );
}

#[test]
fn contribution_attempts_cannot_regress_or_change_worker_without_a_new_attempt() {
    let f = Fixture::new();
    let task = f
        .state
        .begin(
            &f.project_id,
            "Retain newest professional result",
            &["backend-engineering".into()],
            "Newest attempt owns the contribution",
            "A late older result cannot replace newer work",
            "attempt-start",
        )
        .unwrap();
    let task_id = task["id"].as_str().unwrap();
    let revision = || {
        f.state.status(task_id).unwrap()["revision"]
            .as_i64()
            .unwrap()
    };
    f.write(
        "roster.json",
        &serde_json::to_string(&json!({"objective":"Retain newest professional result",
        "assignments":[{"role":"backend-engineering","skill":"backend-engineering"}]}))
        .unwrap(),
    );
    let receipt = |attempt: u32, worker: &str| {
        json!({
            "schema_version":1,"task_id":task_id,"roster":f.binding("roster.json"),"limitations":[],
            "assignments":[{
                "role":"backend-engineering","attempt":attempt,"result_attempt":attempt,
                "status":"completed","host_agent_id":worker,"inputs":[f.binding("input.txt")],
                "evidence":[f.binding("result.txt")],"depends_on":[],"validation":"Observed fixture result",
                "unverified_paths":[],"disposition":"accepted"
            }]
        })
    };
    let save = |attempt, worker, key| {
        f.write(
            "contributions.json",
            &serde_json::to_string(&receipt(attempt, worker)).unwrap(),
        );
        f.state.evidence_with_contract(
            task_id,
            revision(),
            "contributions.json",
            "acceptance",
            "Worker result",
            key,
            Some(ContractKind::Contributions),
        )
    };
    save(2, "worker-current", "attempt-two").unwrap();
    let current_revision = revision();
    let current_status = f.state.status(task_id).unwrap();

    // This report is internally consistent: attempt and result_attempt both1.
    // Only comparison against the persisted newer contribution rejects it.
    assert!(save(1, "worker-older", "late-attempt-one").is_err());
    assert_eq!(revision(), current_revision);
    assert_eq!(f.state.status(task_id).unwrap(), current_status);
    assert!(save(2, "worker-replacement", "same-attempt-other-worker").is_err());
    assert_eq!(revision(), current_revision);
    assert_eq!(f.state.status(task_id).unwrap(), current_status);
    // A new attempt legitimately permits replacing the worker.
    save(3, "worker-replacement", "attempt-three").unwrap();
    assert_eq!(revision(), current_revision + 1);
    let expected_hash = util::sha256(&fs::read(f.project.join("contributions.json")).unwrap());
    assert_eq!(
        f.state.status(task_id).unwrap()["evidence"][0]["sha256"],
        expected_hash
    );
}
