//! Explicit offline decision trials in the selected runtime; never mounts user facts.
use mirket::{packages, paths::Paths};
use std::{fs, path::PathBuf, time::Duration};

#[test]
#[ignore = "requires an explicitly selected prepared agent runtime"]
fn retrieval_ablation_in_disposable_container() {
    let home =
        PathBuf::from(std::env::var("MIRKET_RUNTIME_TEST_HOME").expect("explicit runtime home"));
    let paths = Paths::new(home).unwrap();
    let (engine, mut args, _) = packages::agent::command(&paths, None).unwrap();
    let image = args.pop().unwrap();
    let stage = tempfile::Builder::new()
        .prefix("retrieval-ablation-")
        .tempdir_in(&engine.staging)
        .unwrap();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("evals/harness-remediation/retrieval-ablation.py");
    fs::copy(&source, stage.path().join("trial.py")).unwrap();
    args.extend([
        format!("--volume={}:/trial:ro", stage.path().display()),
        "--entrypoint=python".into(),
        image,
        "-B".into(),
        "/trial/trial.py".into(),
    ]);
    let output = engine.run(&args, Duration::from_secs(300)).unwrap();
    println!("{output}");
    let report: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(report["arms"].as_array().unwrap().len(), 4);
    assert_eq!(report["private_data_mounted"], false);
    if let Ok(path) = std::env::var("MIRKET_TRIAL_REPORT") {
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .unwrap();
        file.write_all(output.as_bytes()).unwrap();
    }
}

#[test]
#[ignore = "requires an explicit candidate design image and prepared engine"]
fn design_specialist_decision_trials() {
    let paths = Paths::new(PathBuf::from(
        std::env::var("MIRKET_RUNTIME_TEST_HOME").expect("explicit runtime home"),
    ))
    .unwrap();
    let engine = mirket::podman::Engine::resolve(&paths).unwrap();
    engine.ready().unwrap();
    let image =
        std::env::var("MIRKET_DESIGN_TRIAL_IMAGE").expect("explicit checked candidate image");
    assert!(image.starts_with("sha256:") && image.len() == 71);
    let stage = tempfile::Builder::new()
        .prefix("design-decision-")
        .tempdir_in(&engine.staging)
        .unwrap();
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("evals/harness-remediation/design-decision.mjs"),
        stage.path().join("trial.mjs"),
    )
    .unwrap();
    fs::create_dir(stage.path().join("workspace")).unwrap();
    let args = vec![
        "run".into(),
        "--rm".into(),
        "--network=none".into(),
        "--pull=never".into(),
        "--read-only".into(),
        "--cap-drop=ALL".into(),
        "--security-opt=no-new-privileges".into(),
        "--memory=2g".into(),
        "--memory-swap=2g".into(),
        "--pids-limit=512".into(),
        "--tmpfs=/tmp:rw,size=768m,mode=1777".into(),
        format!("--volume={}:/workspace:rw", stage.path().join("workspace").display()),
        format!("--volume={}:/opt/design/trial:ro", stage.path().display()),
        "--entrypoint=node".into(),
        image,
        "-e".into(),
        "const{spawn}=require('node:child_process');const c=spawn(process.execPath,['/opt/design/trial/trial.mjs'],{stdio:'inherit'});c.on('exit',code=>process.exitCode=code??1);".into(),
    ];
    let output = engine.run(&args, Duration::from_secs(300)).unwrap();
    let report: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    if let Ok(path) = std::env::var("MIRKET_DESIGN_TRIAL_REPORT") {
        use std::io::Write;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .unwrap()
            .write_all(output.as_bytes())
            .unwrap();
    }
    assert_eq!(
        report["ok"], true,
        "video decision trial failed: {}",
        report["error"]
    );
}
