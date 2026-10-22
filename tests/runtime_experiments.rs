//! Opt-in comparisons in an existing prepared runtime. Synthetic data only.
use mirket::{packages::agent, paths::Paths};
use std::{path::PathBuf, time::Duration};

#[test]
#[ignore = "requires explicitly prepared agent-runtime and MIRKET_TEST_ENGINE_HOME"]
fn multilingual_retrieval_ablation() {
    let home =
        PathBuf::from(std::env::var("MIRKET_TEST_ENGINE_HOME").expect("explicit engine home"));
    let paths = Paths::new(home).unwrap();
    let (engine, mut args, _) = agent::command(&paths, None).unwrap();
    let image = args.pop().unwrap();
    args.extend([
        "--entrypoint=python".into(),
        image,
        "-B".into(),
        "-c".into(),
        include_str!("../evals/role-workflow/experiments/retrieval-ablation.py").into(),
    ]);
    let output = engine.run(&args, Duration::from_secs(600)).unwrap();
    let value: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
    assert_eq!(value["private_data_mounted"], false);
    assert_eq!(value["arms"].as_array().unwrap().len(), 4);
    println!("{output}");
    if let Ok(path) = std::env::var("MIRKET_EXPERIMENT_OUTPUT") {
        use std::io::Write;
        let path = PathBuf::from(path);
        assert!(path.is_absolute());
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .unwrap()
            .write_all(output.as_bytes())
            .unwrap();
    }
}

mod journal {
    include!("../evals/role-workflow/experiments/journal-prototype.rs");
}
