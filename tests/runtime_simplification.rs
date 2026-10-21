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
