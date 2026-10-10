use serde_json::{Value, json};
use std::{
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
fn ok(home: &Path, args: &[&str]) -> Value {
    let output = invoke(home, args);
    assert!(
        output.status.success(),
        "{:?}: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&output.stdout)))
}

#[test]
fn cli_tracks_selected_skills_and_rejects_changed_evidence() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let root = project.path().to_str().unwrap();
    assert!(!invoke(home.path(), &["setup"]).status.success());
    assert!(!home.path().join(".mirket").exists());
    ok(home.path(), &["project", "register", "--project", root]);
    let started = ok(
        home.path(),
        &[
            "task",
            "start",
            "--project",
            root,
            "--capability",
            "CFO",
            "--with",
            "Excel Expert",
            "--objective",
            "Reconcile cash timing",
            "--key",
            "cash-start",
        ],
    );
    let id = started["id"]
        .as_str()
        .or_else(|| started["task_id"].as_str())
        .expect("task id");
    let mut revision = started["revision"].as_i64().unwrap();
    for skill in ["financial-analysis", "spreadsheet-analysis"] {
        let read = ok(
            home.path(),
            &[
                "task",
                "read",
                id,
                skill,
                "--revision",
                &revision.to_string(),
                "--key",
                skill,
            ],
        );
        assert!(!read["document"]["content"].as_str().unwrap().is_empty());
        revision = read["receipt"]["revision"].as_i64().unwrap();
    }
    std::fs::write(
        project.path().join("cash.csv"),
        "opening,receipts,payments,closing\n100,200,250,50\n",
    )
    .unwrap();
    std::fs::write(
        project.path().join("probe.txt"),
        "Delayed receipts: 100 + 0 - 250 = -150\n",
    )
    .unwrap();
    for (path, kind, key) in [
        ("cash.csv", "acceptance", "cash-evidence"),
        ("probe.txt", "failure-probe", "probe-evidence"),
    ] {
        let evidence = ok(
            home.path(),
            &[
                "task",
                "evidence",
                id,
                path,
                "--kind",
                kind,
                "--revision",
                &revision.to_string(),
                "--key",
                key,
                "--summary",
                "Checked the cash arithmetic",
            ],
        );
        revision = evidence["revision"].as_i64().unwrap();
    }
    std::fs::write(project.path().join("cash.csv"), "changed").unwrap();
    let rejected = invoke(
        home.path(),
        &[
            "task",
            "complete",
            id,
            "--revision",
            &revision.to_string(),
            "--key",
            "cash-complete",
        ],
    );
    assert!(!rejected.status.success());
    std::fs::write(
        project.path().join("cash.csv"),
        "opening,receipts,payments,closing\n100,200,250,50\n",
    )
    .unwrap();
    let complete = ok(
        home.path(),
        &[
            "task",
            "complete",
            id,
            "--revision",
            &revision.to_string(),
            "--key",
            "cash-complete",
        ],
    );
    assert_eq!(complete["status"], "complete");
    let list = ok(home.path(), &["task", "list", "--project", root]);
    assert_eq!(list["tasks"].as_array().unwrap().len(), 1);
}

#[test]
fn setup_doctor_and_registered_executable_use_only_the_selected_home() {
    let home = tempfile::tempdir().unwrap();
    ok(
        home.path(),
        &["setup", "--target", "all", "--yes", "--json"],
    );
    let doctor = ok(home.path(), &["doctor", "--json"]);
    assert_eq!(doctor["ok"], true);
    assert_eq!(doctor["mcp"]["ok"], true);
    ok(
        home.path(),
        &[
            "tool",
            "register",
            "mirket-test",
            "--executable",
            env!("CARGO_BIN_EXE_mirket"),
        ],
    );
    let child = invoke(
        home.path(),
        &["tool", "run", "mirket-test", "--", "--version"],
    );
    assert!(child.status.success());
    assert!(child.stdout.starts_with(b"mirket "));
    ok(home.path(), &["tool", "remove", "mirket-test"]);
    assert_eq!(ok(home.path(), &["tool", "list"]), json!({}));
}

#[test]
fn standalone_installer_installs_the_complete_cli() {
    let home = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_install-codex"))
        .arg("--home")
        .arg(home.path())
        .args(["--yes", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let binary = home.path().join(".mirket/bin").join(if cfg!(windows) {
        "mirket.exe"
    } else {
        "mirket"
    });
    let output = Command::new(binary).args(["--version"]).output().unwrap();
    assert!(output.status.success());
    assert!(output.stdout.starts_with(b"mirket "));
    assert!(home.path().join(".codex/AGENTS.md").is_file());
    assert!(!home.path().join(".claude/CLAUDE.md").exists());
}
