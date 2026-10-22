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

#[cfg(unix)]
#[test]
fn newer_runtime_handoff_preserves_the_entry_pid_arguments_and_exit_status() {
    use std::{fs, os::unix::fs::PermissionsExt};

    let temporary = tempfile::tempdir().unwrap();
    let home = temporary.path().canonicalize().unwrap();
    let managed = home.join(".mirket/bin/mirket");
    fs::create_dir_all(managed.parent().unwrap()).unwrap();
    let script = b"#!/bin/sh\nprintf '%s\\n' \"$$\" \"$MIRKET_DIRECT\" \"$@\"\nexit 23\n";
    fs::write(&managed, script).unwrap();
    fs::set_permissions(&managed, fs::Permissions::from_mode(0o755)).unwrap();
    let next_major = env!("CARGO_PKG_VERSION_MAJOR").parse::<u64>().unwrap() + 1;
    let receipt = json!({
        "format":1,"version":format!("{next_major}.0.0"),
        "options":{"targets":["codex"]},
        "files":{".mirket/bin/mirket":{"sha256":mirket::util::sha256(script),"executable":true}},
        "servers":{}
    });
    fs::write(
        home.join(".mirket/setup.json"),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_mirket"))
        .env_remove("MIRKET_DIRECT")
        .arg("--home")
        .arg(&home)
        .arg("info")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let entry_pid = child.id();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(23));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(
        lines,
        [
            entry_pid.to_string(),
            "1".into(),
            "--home".into(),
            home.to_str().unwrap().into(),
            "info".into()
        ]
    );
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

#[test]
fn package_set_replaces_retired_selection_before_preparation_without_partial_setup() {
    let temporary = tempfile::tempdir().unwrap();
    let home = temporary.path();
    ok(home, &["setup", "--yes", "--target", "codex", "--json"]);
    let receipt_path = home.join(".mirket/setup.json");
    let mut receipt: Value =
        serde_json::from_slice(&std::fs::read(&receipt_path).unwrap()).unwrap();
    receipt["options"]["packages"] = json!(["retired-package"]);
    receipt["options"]["package_settings"] =
        json!({"retired-package":{"private":"preserve until success"}});
    let before_receipt = serde_json::to_vec(&receipt).unwrap();
    std::fs::write(&receipt_path, &before_receipt).unwrap();
    let config_path = home.join(".codex/config.toml");
    let before_config = std::fs::read(&config_path).unwrap();

    let result = invoke(home, &["package", "set", "design"]);
    assert!(!result.status.success());
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(
        error.contains("register the prerequisite") || error.contains("supported only"),
        "selection must reach runtime preparation instead of appending a retired ID: {error}"
    );
    assert!(!error.contains("select the design stack once"));
    assert_eq!(std::fs::read(receipt_path).unwrap(), before_receipt);
    assert_eq!(std::fs::read(config_path).unwrap(), before_config);
    assert!(
        !home
            .join(".mirket/packages/data/design/state.json")
            .exists()
    );
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn environment_init_checks_helpers_before_downloading_or_invoking_podman() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let temporary = tempfile::tempdir().unwrap();
    let home = temporary.path().canonicalize().unwrap();
    let bin = home.join("client/bin");
    fs::create_dir_all(&bin).unwrap();
    let client = bin.join("podman-remote");
    // Exit 99 identifies an accidental invocation; this fixture is never a VM.
    fs::write(
        &client,
        b"#!/bin/sh\necho client-was-invoked >&2\nexit 99\n",
    )
    .unwrap();
    fs::set_permissions(&client, fs::Permissions::from_mode(0o700)).unwrap();
    ok(
        &home,
        &[
            "tool",
            "register",
            "podman",
            "--executable",
            client.to_str().unwrap(),
        ],
    );
    for args in [
        vec!["environment", "init"],
        vec!["environment", "init", "--image", "/missing-fixture-image"],
        vec!["environment", "start"],
    ] {
        let result = invoke(&home, &args);
        assert!(!result.status.success());
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(error.contains("lacks executable gvproxy"), "{error}");
        assert!(!error.contains("client-was-invoked"));
        assert!(!home.join(".mirket/cache/podman").exists());
        assert!(
            !home
                .join(".mirket/execution/podman/data/containers/podman/machine")
                .exists()
        );
    }
    let paths = mirket::paths::Paths::new(home).unwrap();
    let engine = mirket::podman::Engine::resolve(&paths).unwrap();
    fs::remove_dir_all(Path::new(&engine.env["HOME"]).parent().unwrap()).unwrap();
}
