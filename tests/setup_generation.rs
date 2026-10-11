use mirket::{install, paths::Paths, util::sha256};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn invoke(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mirket"))
        .arg("--home")
        .arg(home)
        .args(args)
        .output()
        .unwrap()
}

fn setup(home: &Path, target: &str) {
    let output = invoke(home, &["setup", "--yes", "--target", target]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn setup_generation_snapshot_preserves_a_later_saved_selection() {
    let home = tempfile::tempdir().unwrap();
    let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
    setup(home.path(), "codex");
    let snapshot = install::setup_snapshot(&paths).unwrap();
    assert_eq!(
        snapshot.options.unwrap().targets,
        vec![install::Target::Codex]
    );
    assert_eq!(snapshot.token, sha256(&fs::read(paths.settings()).unwrap()));

    setup(home.path(), "claude");
    let later_receipt = fs::read(paths.settings()).unwrap();
    let later_config = fs::read(home.path().join(".claude.json")).unwrap();
    let output = invoke(
        home.path(),
        &[
            "setup",
            "--yes",
            "--target",
            "codex",
            "--expected-state",
            &snapshot.token,
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("setup selection changed"));
    assert_eq!(fs::read(paths.settings()).unwrap(), later_receipt);
    assert_eq!(
        fs::read(home.path().join(".claude.json")).unwrap(),
        later_config
    );
    assert!(!home.path().join(".codex/AGENTS.md").exists());
    assert!(home.path().join(".claude/CLAUDE.md").exists());
}

#[test]
fn setup_generation_absent_snapshot_does_not_replace_a_first_commit() {
    let home = tempfile::tempdir().unwrap();
    let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
    let snapshot = install::setup_snapshot(&paths).unwrap();
    assert!(snapshot.options.is_none());
    setup(home.path(), "codex");
    let later_receipt = fs::read(paths.settings()).unwrap();
    let output = invoke(
        home.path(),
        &[
            "setup",
            "--yes",
            "--target",
            "claude",
            "--expected-state",
            &snapshot.token,
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("setup selection changed"));
    assert_eq!(fs::read(paths.settings()).unwrap(), later_receipt);
    assert!(home.path().join(".codex/AGENTS.md").exists());
    assert!(!home.path().join(".claude/CLAUDE.md").exists());
}

#[test]
fn setup_generation_matching_absent_and_saved_tokens_apply() {
    let home = tempfile::tempdir().unwrap();
    let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
    for target in ["codex", "claude"] {
        let snapshot = install::setup_snapshot(&paths).unwrap();
        let output = invoke(
            home.path(),
            &[
                "setup",
                "--yes",
                "--target",
                target,
                "--expected-state",
                &snapshot.token,
            ],
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert_eq!(
        install::saved_options(&paths).unwrap().unwrap().targets,
        vec![install::Target::Claude]
    );
}

#[test]
fn setup_generation_invalid_tokens_do_not_create_installation_state() {
    let home = tempfile::tempdir().unwrap();
    for token in [
        String::new(),
        "missing".into(),
        "a".repeat(63),
        "a".repeat(65),
        "A".repeat(64),
    ] {
        let output = invoke(
            home.path(),
            &[
                "setup",
                "--yes",
                "--target",
                "codex",
                "--expected-state",
                &token,
            ],
        );
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("invalid setup generation token"));
        assert!(!home.path().join(".mirket").exists());
        assert!(!home.path().join(".codex").exists());
    }
}
