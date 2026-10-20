use mirket::{
    paths::Paths,
    router::handoff::{self, Request},
    util,
};
use serde_json::json;
use std::fs;

struct Fixture {
    _home: tempfile::TempDir,
    paths: Paths,
    request: Request,
}
impl Fixture {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().to_path_buf()).unwrap();
        let project = paths.home.join("project");
        fs::create_dir(&project).unwrap();
        fs::write(project.join("source.rs"), "source").unwrap();
        fs::write(project.join("check.log"), "tests passed").unwrap();
        let run = paths.root.join("runs/routed-fixture");
        fs::create_dir_all(&run).unwrap();
        let trace = serde_json::to_vec(
            &json!({"status":"awaiting_host_verification","host_verification":{
            "project":project, "status":"awaiting_host_verification", "required_checks":["tests"],
            "artifacts":[{"path":"source.rs","sha256":util::sha256(b"source")}]}}),
        )
        .unwrap();
        fs::write(run.join("trace.json"), &trace).unwrap();
        let mut request: Request = serde_json::from_value(json!({"project":project,"run_id":"routed-fixture","expected_trace_sha256":util::sha256(&trace),
            "checks":[{"name":"tests","status":"passed","summary":"Executed test fixture","evidence":{"path":"check.log","sha256":util::sha256(b"tests passed")}}]})).unwrap();
        let mut begin = request.clone();
        begin.checks = None;
        begin.begin = Some(serde_json::from_value(json!({"key":"initial", "inputs":["source.rs"], "commands":{"tests":"fixture test runner v1"}})).unwrap());
        let started = handoff::record(&paths, &begin).unwrap();
        request.expected_trace_sha256 = started["trace_sha256"].as_str().map(str::to_owned);
        request.attempt_id = started["host_verification"]["active_attempt"]
            .as_str()
            .map(str::to_owned);
        Self {
            _home: home,
            paths,
            request,
        }
    }
}

#[test]
fn stale_sources_logs_and_trace_cannot_verify() {
    for (file, content) in [
        ("source.rs", "changed source"),
        ("check.log", "changed log"),
    ] {
        let f = Fixture::new();
        fs::write(f.request.project.join(file), content).unwrap();
        let result = handoff::record(&f.paths, &f.request);
        if file == "source.rs" {
            assert_eq!(result.unwrap()["status"], "verification_stale");
        } else {
            assert!(result.is_err());
        }
    }
    let mut f = Fixture::new();
    f.request.expected_trace_sha256 = Some("0".repeat(64));
    assert!(handoff::record(&f.paths, &f.request).is_err());
}

#[test]
fn complete_coverage_and_evidence_are_required_and_failures_stay_pending() {
    let mut f = Fixture::new();
    f.request.checks.as_mut().unwrap()[0].evidence = None;
    assert!(handoff::record(&f.paths, &f.request).is_err());
    f.request.checks.as_mut().unwrap()[0].status = handoff::CheckStatus::NotRun;
    assert_eq!(
        handoff::record(&f.paths, &f.request).unwrap()["status"],
        "awaiting_host_verification"
    );
    let mut f = Fixture::new();
    f.request.checks = Some(vec![]);
    assert!(handoff::record(&f.paths, &f.request).is_err());
    let mut f = Fixture::new();
    let check = f.request.checks.as_ref().unwrap()[0].clone();
    f.request.checks.as_mut().unwrap().push(check);
    assert!(handoff::record(&f.paths, &f.request).is_err());
    let mut f = Fixture::new();
    f.request.checks.as_mut().unwrap()[0].status = handoff::CheckStatus::Failed;
    assert_eq!(
        handoff::record(&f.paths, &f.request).unwrap()["status"],
        "awaiting_host_verification"
    );
}

#[test]
fn verification_replays_are_idempotent_but_stale_success_is_rejected() {
    let f = Fixture::new();
    let verified = handoff::record(&f.paths, &f.request).unwrap();
    assert_eq!(verified["status"], "host_verified");
    assert_eq!(handoff::record(&f.paths, &f.request).unwrap(), verified);
    let mut query = f.request.clone();
    query.checks = None;
    fs::write(f.request.project.join("check.log"), "changed").unwrap();
    assert_eq!(
        handoff::record(&f.paths, &query).unwrap()["status"],
        "verification_stale"
    );
    assert!(handoff::record(&f.paths, &f.request).is_err());
}

#[test]
fn paths_and_project_scope_cannot_escape_the_handoff() {
    let mut f = Fixture::new();
    f.request.run_id = "routed-fixture/../other".into();
    assert!(handoff::record(&f.paths, &f.request).is_err());
    let mut f = Fixture::new();
    f.request.project = f.paths.home.clone();
    assert!(handoff::record(&f.paths, &f.request).is_err());
    let mut f = Fixture::new();
    f.request.checks.as_mut().unwrap()[0]
        .evidence
        .as_mut()
        .unwrap()
        .path = "../check.log".into();
    assert!(handoff::record(&f.paths, &f.request).is_err());
}

#[cfg(unix)]
#[test]
fn symlink_evidence_is_rejected() {
    let f = Fixture::new();
    let log = f.request.project.join("check.log");
    fs::remove_file(&log).unwrap();
    let outside = f.paths.home.join("outside.log");
    fs::write(&outside, "tests passed").unwrap();
    std::os::unix::fs::symlink(outside, log).unwrap();
    assert!(handoff::record(&f.paths, &f.request).is_err());
}

#[test]
fn fresh_attempt_replaces_stale_logs_without_rewriting_history() {
    let mut f = Fixture::new();
    let old = handoff::record(&f.paths, &f.request).unwrap();
    let previous = old["host_verification"]["attempts"][0].clone();
    fs::write(f.request.project.join("check.log"), "new test result").unwrap();
    let mut query = f.request.clone();
    query.checks = None;
    let stale = handoff::record(&f.paths, &query).unwrap();
    assert_eq!(stale["status"], "verification_stale");
    let mut begin = query.clone();
    begin.expected_trace_sha256 = stale["trace_sha256"].as_str().map(str::to_owned);
    begin.begin = Some(serde_json::from_value(json!({"key":"rerun", "inputs":["source.rs"], "commands":{"tests":"fixture test runner v1"}})).unwrap());
    let prepared = handoff::record(&f.paths, &begin).unwrap();
    assert_eq!(handoff::record(&f.paths, &begin).unwrap(), prepared);
    f.request.attempt_id = prepared["host_verification"]["active_attempt"]
        .as_str()
        .map(str::to_owned);
    f.request.expected_trace_sha256 = prepared["trace_sha256"].as_str().map(str::to_owned);
    f.request.checks.as_mut().unwrap()[0]
        .evidence
        .as_mut()
        .unwrap()
        .sha256 = util::sha256(b"new test result");
    let result = handoff::record(&f.paths, &f.request).unwrap();
    assert_eq!(result["status"], "host_verified");
    assert_eq!(result["host_verification"]["attempts"][0], previous);
    assert_eq!(
        result["host_verification"]["attempts"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn verification_tracks_config_directory_additions_and_repairs() {
    let mut f = Fixture::new();
    fs::create_dir(f.request.project.join("config")).unwrap();
    fs::write(f.request.project.join("config/settings.json"), "{}").unwrap();
    let mut query = f.request.clone();
    query.checks = None;
    let current = handoff::record(&f.paths, &query).unwrap();
    let mut begin = query.clone();
    begin.expected_trace_sha256 = current["trace_sha256"].as_str().map(str::to_owned);
    begin.begin=Some(serde_json::from_value(json!({"key":"config", "inputs":["source.rs","config"],"commands":{"tests":"fixture runner"}})).unwrap());
    let prepared = handoff::record(&f.paths, &begin).unwrap();
    f.request.expected_trace_sha256 = prepared["trace_sha256"].as_str().map(str::to_owned);
    f.request.attempt_id = prepared["host_verification"]["active_attempt"]
        .as_str()
        .map(str::to_owned);
    assert_eq!(
        handoff::record(&f.paths, &f.request).unwrap()["status"],
        "host_verified"
    );
    fs::write(f.request.project.join("config/added.json"), "changed").unwrap();
    assert_eq!(
        handoff::record(&f.paths, &query).unwrap()["status"],
        "verification_stale"
    );
    // A host repair is admitted only by starting another attempt before its checks.
    fs::write(f.request.project.join("source.rs"), "repaired").unwrap();
    let current = handoff::record(&f.paths, &query).unwrap();
    begin.expected_trace_sha256 = current["trace_sha256"].as_str().map(str::to_owned);
    begin.begin.as_mut().unwrap().key = "repair".into();
    let prepared = handoff::record(&f.paths, &begin).unwrap();
    f.request.expected_trace_sha256 = prepared["trace_sha256"].as_str().map(str::to_owned);
    f.request.attempt_id = prepared["host_verification"]["active_attempt"]
        .as_str()
        .map(str::to_owned);
    assert_eq!(
        handoff::record(&f.paths, &f.request).unwrap()["status"],
        "host_verified"
    );
}

#[test]
fn large_manifests_stay_in_trace_and_design_inputs_invalidate_success() {
    let mut f = Fixture::new();
    let scope = f
        .paths
        .root
        .join("execution/podman/staging/design/workspace/projects")
        .join(util::sha256(f.request.project.to_string_lossy().as_bytes()));
    fs::create_dir_all(scope.join("source")).unwrap();
    for i in 0..1000 {
        fs::write(scope.join(format!("source/{i}.txt")), "specimen").unwrap();
    }
    let mut begin = f.request.clone();
    begin.checks = None;
    begin.begin = Some(serde_json::from_value(json!({"key":"design", "inputs":["source.rs"], "design_inputs":["source"], "commands":{"tests":"real app runner"}})).unwrap());
    let prepared = handoff::record(&f.paths, &begin).unwrap();
    assert!(serde_json::to_vec(&prepared).unwrap().len() < 64 * 1024);
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(prepared["trace_path"].as_str().unwrap()).unwrap())
            .unwrap();
    assert_eq!(
        saved["host_verification"]["attempts"][1]["design_files"]
            .as_object()
            .unwrap()
            .len(),
        1001
    );
    f.request.expected_trace_sha256 = prepared["trace_sha256"].as_str().map(str::to_owned);
    f.request.attempt_id = prepared["host_verification"]["active_attempt"]
        .as_str()
        .map(str::to_owned);
    assert_eq!(
        handoff::record(&f.paths, &f.request).unwrap()["status"],
        "host_verified"
    );
    fs::write(scope.join("source/100.txt"), "changed").unwrap();
    f.request.checks = None;
    assert_eq!(
        handoff::record(&f.paths, &f.request).unwrap()["status"],
        "verification_stale"
    );
}
