use mirket::role_eval::{self, RunReport, Suite};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn fixture() -> (Suite, String, RunReport) {
    let bytes = include_bytes!("../evals/role-workflow/suite.json");
    let suite: Suite = serde_json::from_slice(bytes).unwrap();
    let trials: Vec<Value> = suite.cases.iter().flat_map(|case| suite.configurations.iter().map(move |config| json!({
        "case_id":case.id,"configuration":config.id,"trial":1,
        "input_sha256":case.input.sha256,"rubric_sha256":case.rubric.sha256,"configuration_sha256":config.file.sha256,
        "outcome":"passed","score":1.0,"verdicts":case.criteria.iter().map(|c| (c.clone(),true)).collect::<std::collections::BTreeMap<_,_>>(),
        "evidence":[{"path":"synthetic-output.txt","sha256":digest(b"SYNTHETIC CHECKER TEST; NO MODEL EXECUTED\n")}],
        "rationale":"Synthetic consistency fixture, not a role-quality observation.","cost_usd":null,"latency_ms":null
    }))).collect();
    let hash = digest(bytes);
    let report: RunReport = serde_json::from_value(json!({"schema_version":1,"provenance":"synthetic","suite_sha256":hash,"checker_sha256":suite.checker.sha256,"trials":trials,"limitations":["Synthetic checker regression only; no model run or role-quality evidence."]})).unwrap();
    (suite, hash, report)
}
#[test]
fn deterministic_aggregates_retain_failures_and_unknown_measurements() {
    let (suite, hash, mut report) = fixture();
    report.trials[0].outcome = "infrastructure_failed".into();
    report.trials[0].score = None;
    report.trials[0].verdicts.clear();
    report.trials[0].evidence.clear();
    report.trials[1].outcome = "task_failed".into();
    report.trials[1].score = Some(2.0 / 3.0);
    *report.trials[1].verdicts.values_mut().next().unwrap() = false;
    report.trials[1].cost_usd = Some(0.25);
    let result = role_eval::validate(&suite, &hash, &report).unwrap();
    assert_eq!(result, role_eval::validate(&suite, &hash, &report).unwrap());
    let group = &result["aggregates"][0];
    assert_eq!(group["trials"], 6);
    assert_eq!(group["infrastructure_failed"], 1);
    assert_eq!(group["task_failed"], 1);
    assert_eq!(group["passed"], 4);
    assert_eq!(group["unknown_cost_trials"], 5);
    assert_eq!(group["known_cost_usd"], 0.25);
    assert!(group["total_cost_usd"].is_null());
    assert!((group["mean_score"].as_f64().unwrap() - 14.0 / 15.0).abs() < 1e-12);
}
#[test]
fn rejects_unknown_duplicate_missing_and_unplanned_cells() {
    let (suite, hash, report) = fixture();
    let mut bad = report.clone();
    bad.trials[0].case_id = "unknown".into();
    assert!(
        role_eval::validate(&suite, &hash, &bad)
            .unwrap_err()
            .to_string()
            .contains("unknown case")
    );
    let mut bad = report.clone();
    bad.trials[1] = bad.trials[0].clone();
    assert!(
        role_eval::validate(&suite, &hash, &bad)
            .unwrap_err()
            .to_string()
            .contains("duplicate")
    );
    let mut bad = report.clone();
    bad.trials.pop();
    assert!(
        role_eval::validate(&suite, &hash, &bad)
            .unwrap_err()
            .to_string()
            .contains("missing trial")
    );
    let mut bad = report;
    bad.trials[0].trial = 2;
    assert!(role_eval::validate(&suite, &hash, &bad).is_err());
}
#[test]
fn rejects_split_overlap_even_under_renamed_case_ids() {
    let (mut suite, hash, report) = fixture();
    suite.held_out.push(suite.development[0].clone());
    assert!(
        role_eval::validate(&suite, &hash, &report)
            .unwrap_err()
            .to_string()
            .contains("split overlap")
    );
    let (mut suite, hash, report) = fixture();
    suite.cases[6].input.sha256 = suite.cases[0].input.sha256.clone();
    assert!(
        role_eval::validate(&suite, &hash, &report)
            .unwrap_err()
            .to_string()
            .contains("input overlap")
    );
}
#[test]
fn rejects_identity_drift_nonfinite_scores_and_fabricated_aggregates() {
    let (suite, hash, report) = fixture();
    for field in ["input_sha256", "rubric_sha256", "configuration_sha256"] {
        let mut value = serde_json::to_value(&report).unwrap();
        value["trials"][0][field] = json!("stale");
        let bad = serde_json::from_value(value).unwrap();
        assert!(
            role_eval::validate(&suite, &hash, &bad)
                .unwrap_err()
                .to_string()
                .contains("identity mismatch")
        );
    }
    for score in [f64::NAN, f64::INFINITY, -0.1, 1.1, 0.5] {
        let mut bad = report.clone();
        bad.trials[0].score = Some(score);
        assert!(role_eval::validate(&suite, &hash, &bad).is_err());
    }
    let mut bad = report.clone();
    bad.checker_sha256 = "stale".into();
    assert!(role_eval::validate(&suite, &hash, &bad).is_err());
    let mut value = serde_json::to_value(report).unwrap();
    value["mean_score"] = json!(1.0);
    assert!(serde_json::from_value::<RunReport>(value).is_err());
}
#[test]
fn infrastructure_failures_cannot_be_scored_and_verdicts_must_be_complete() {
    let (suite, hash, report) = fixture();
    let mut bad = report.clone();
    bad.trials[0].outcome = "infrastructure_failed".into();
    assert!(role_eval::validate(&suite, &hash, &bad).is_err());
    let mut bad = report.clone();
    bad.trials[0].verdicts.clear();
    assert!(role_eval::validate(&suite, &hash, &bad).is_err());
    let mut bad = report;
    bad.trials[0].cost_usd = Some(-1.0);
    assert!(role_eval::validate(&suite, &hash, &bad).is_err());
}
fn copy_fixture(root: &Path, suite: &Suite) {
    let project = Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = std::iter::once("evals/role-workflow/suite.json")
        .chain(std::iter::once(suite.checker.path.as_str()))
        .chain(suite.configurations.iter().map(|c| c.file.path.as_str()))
        .chain(
            suite
                .cases
                .iter()
                .flat_map(|c| [c.input.path.as_str(), c.rubric.path.as_str()]),
        );
    for path in files {
        let destination = root.join(path);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(project.join(path), destination).unwrap();
    }
}
#[test]
fn file_checker_detects_stale_input_rubric_configuration_and_saved_output() {
    let (suite, _, report) = fixture();
    let temp = tempfile::tempdir().unwrap();
    copy_fixture(temp.path(), &suite);
    fs::write(
        temp.path().join("synthetic-output.txt"),
        b"SYNTHETIC CHECKER TEST; NO MODEL EXECUTED\n",
    )
    .unwrap();
    fs::write(
        temp.path().join("report.json"),
        serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    let check = || {
        role_eval::check(
            temp.path(),
            Path::new("evals/role-workflow/suite.json"),
            Path::new("report.json"),
        )
    };
    check().unwrap();
    for path in [
        &suite.cases[0].input.path,
        &suite.cases[0].rubric.path,
        &suite.configurations[0].file.path,
        &"synthetic-output.txt".to_string(),
    ] {
        let original = fs::read(temp.path().join(path)).unwrap();
        fs::write(temp.path().join(path), b"changed").unwrap();
        assert!(
            check()
                .unwrap_err()
                .to_string()
                .contains("stale evaluation binding")
        );
        fs::write(temp.path().join(path), original).unwrap();
    }
    fs::write(temp.path().join("unrelated.txt"), b"irrelevant").unwrap();
    check().unwrap();
    assert!(role_eval::suite(temp.path(), Path::new("../suite.json")).is_err());
}
#[cfg(unix)]
#[test]
fn rejects_symlink_evidence() {
    let (suite, _, report) = fixture();
    let temp = tempfile::tempdir().unwrap();
    copy_fixture(temp.path(), &suite);
    fs::write(
        temp.path().join("target.txt"),
        b"SYNTHETIC CHECKER TEST; NO MODEL EXECUTED\n",
    )
    .unwrap();
    std::os::unix::fs::symlink("target.txt", temp.path().join("synthetic-output.txt")).unwrap();
    fs::write(
        temp.path().join("report.json"),
        serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    assert!(
        role_eval::check(
            temp.path(),
            Path::new("evals/role-workflow/suite.json"),
            Path::new("report.json")
        )
        .unwrap_err()
        .to_string()
        .contains("symlink")
    );
}

#[test]
fn planned_repeats_and_comparison_arms_require_every_cell() {
    let (mut suite, hash, report) = fixture();
    let mut second = suite.configurations[0].clone();
    second.id = "candidate".into();
    suite.configurations.push(second);
    suite.trials_per_case = 2;
    let mut comparison = report.clone();
    comparison.trials.clear();
    for trial in report.trials {
        for config in &suite.configurations {
            for index in 1..=2 {
                let mut cell = trial.clone();
                cell.configuration = config.id.clone();
                cell.trial = index;
                comparison.trials.push(cell);
            }
        }
    }
    let checked = role_eval::validate(&suite, &hash, &comparison).unwrap();
    assert_eq!(checked["aggregates"].as_array().unwrap().len(), 4);
    comparison.trials.pop();
    assert!(role_eval::validate(&suite, &hash, &comparison).is_err());
}

#[test]
fn held_out_only_confirmation_retains_every_assignment_and_split_check() {
    let (mut suite, hash, report) = fixture();
    suite.cases.truncate(4);
    suite.development.clear();
    suite.held_out = suite.cases.iter().map(|case| case.id.clone()).collect();
    for case in &mut suite.cases {
        case.split = "held_out".into();
    }
    let mut candidate = suite.configurations[0].clone();
    candidate.id = "candidate".into();
    suite.configurations.push(candidate);
    suite.trials_per_case = 2;
    let mut confirmation = report.clone();
    confirmation.trials.clear();
    for trial in report.trials.into_iter().take(4) {
        for config in &suite.configurations {
            for repeat in 1..=2 {
                let mut cell = trial.clone();
                cell.configuration = config.id.clone();
                cell.trial = repeat;
                confirmation.trials.push(cell);
            }
        }
    }
    let checked = role_eval::validate(&suite, &hash, &confirmation).unwrap();
    let groups = checked["aggregates"].as_array().unwrap();
    assert_eq!(groups.len(), 2);
    for group in groups {
        assert_eq!(group["split"], "held_out");
        assert_eq!(group["trials"], 8);
    }
    let mut missing = confirmation.clone();
    missing.trials.pop();
    assert!(role_eval::validate(&suite, &hash, &missing).is_err());
    let mut duplicate = confirmation.clone();
    duplicate.trials[1] = duplicate.trials[0].clone();
    assert!(role_eval::validate(&suite, &hash, &duplicate).is_err());
    let mut bad_split = suite.clone();
    bad_split.cases[0].split = "development".into();
    assert!(role_eval::validate(&bad_split, &hash, &confirmation).is_err());
    let mut unassigned = suite.clone();
    unassigned.held_out.pop();
    assert!(role_eval::validate(&unassigned, &hash, &confirmation).is_err());
    suite.held_out.clear();
    assert!(role_eval::validate(&suite, &hash, &confirmation).is_err());
}

#[test]
fn oversized_infrastructure_evidence_and_unfilled_host_configuration_reject() {
    let (suite, hash, mut report) = fixture();
    let temp = tempfile::tempdir().unwrap();
    copy_fixture(temp.path(), &suite);
    fs::write(
        temp.path().join("synthetic-output.txt"),
        b"SYNTHETIC CHECKER TEST; NO MODEL EXECUTED\n",
    )
    .unwrap();
    report.provenance = "host_observed".into();
    fs::write(
        temp.path().join("report.json"),
        serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    assert!(
        role_eval::check(
            temp.path(),
            Path::new("evals/role-workflow/suite.json"),
            Path::new("report.json")
        )
        .unwrap_err()
        .to_string()
        .contains("unfilled template")
    );
    report.trials[0].outcome = "infrastructure_failed".into();
    report.trials[0].score = None;
    report.trials[0].verdicts.clear();
    report.trials[0].evidence = vec![report.trials[0].evidence[0].clone(); 17];
    assert!(
        role_eval::validate(&suite, &hash, &report)
            .unwrap_err()
            .to_string()
            .contains("too many evidence")
    );
}
