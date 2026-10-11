//! Explicit installed-package lifecycle probes; never use a personal home.
use anyhow::{Context, Result, ensure};
use mirket::{packages, paths::Paths, util::sha256};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn invoke(home: &Path, args: &[&str]) -> Result<Output> {
    Ok(Command::new(env!("CARGO_BIN_EXE_mirket"))
        .arg("--home")
        .arg(home)
        .arg("--json")
        .args(args)
        .output()?)
}
fn success(home: &Path, args: &[&str]) -> Result<Value> {
    let output = invoke(home, args)?;
    ensure!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}
fn selected(home: &Path) -> Result<Value> {
    Ok(
        serde_json::from_slice::<Value>(&fs::read(home.join(".mirket/setup.json"))?)?["options"]
            .clone(),
    )
}

#[test]
#[ignore = "mutates an explicitly marked disposable installed-package home"]
fn packages_lifecycle_live_removal_update_and_drift_preserve_ownership() -> Result<()> {
    let home = std::path::PathBuf::from(
        std::env::var_os("MIRKET_PACKAGE_TEST_HOME")
            .context("explicit disposable package home is required")?,
    )
    .canonicalize()?;
    ensure!(
        fs::read(home.join(".mirket-package-test-home"))? == b"disposable package test home\n",
        "test-home marker is required"
    );
    let options = selected(&home)?;
    ensure!(
        options["packages"]
            .as_array()
            .context("packages")?
            .contains(&json!("jscpd")),
        "install jscpd in the disposable home first"
    );
    ensure!(
        options["targets"] == json!(["codex", "claude"]),
        "set up both clients first"
    );
    let project = tempfile::tempdir_in(std::env::temp_dir().canonicalize()?)?;
    let paths = Paths::new(home.clone())?;
    // Resolve the actual generation; operations and execution below use the CLI.
    let launch = packages::launch(&paths, "jscpd", project.path())?;
    let store_root = launch
        .command
        .parent()
        .context("native executable parent")?;
    let codex_path = home.join(".codex/config.toml");
    let claude_path = home.join(".claude.json");
    let mut codex = fs::read_to_string(&codex_path)?.parse::<toml_edit::DocumentMut>()?;
    let mut claude: Value = serde_json::from_slice(&fs::read(&claude_path)?)?;
    let fixture_name = format!("package-lifecycle-user-owned-{}", std::process::id());
    let name = fixture_name.as_str();
    ensure!(
        codex["mcp_servers"].get(name).is_none() && claude["mcpServers"].get(name).is_none(),
        "fixture marker already exists"
    );
    codex["mcp_servers"][name] = toml_edit::Item::Table(toml_edit::Table::new());
    codex["mcp_servers"][name]["command"] = toml_edit::value("user-owned-fixture");
    claude["mcpServers"][name] = json!({"command":"user-owned-fixture"});
    fs::write(&codex_path, codex.to_string())?;
    fs::write(&claude_path, serde_json::to_vec_pretty(&claude)?)?;

    success(&home, &["package", "remove", "jscpd"])?;
    let removed = selected(&home)?;
    ensure!(
        !removed["packages"]
            .as_array()
            .unwrap()
            .contains(&json!("jscpd")),
        "selection was retained"
    );
    ensure!(
        removed["package_settings"] == options["package_settings"],
        "unrelated settings changed"
    );
    let not_selected = invoke(&home, &["package", "probe", "jscpd"])?;
    ensure!(
        !not_selected.status.success()
            && String::from_utf8_lossy(&not_selected.stderr).contains("not selected"),
        "removed package was launchable"
    );
    ensure!(
        store_root.is_dir(),
        "removal purged reusable package content"
    );
    success(&home, &["package", "install", "jscpd"])?;
    let reinstalled = selected(&home)?;
    let membership = |value: &Value| {
        value["packages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| id.as_str().unwrap().to_owned())
            .collect::<std::collections::BTreeSet<_>>()
    };
    ensure!(
        membership(&reinstalled) == membership(&options),
        "reinstall changed selected packages"
    );
    let mut options = options;
    options["packages"] = reinstalled["packages"].clone();
    ensure!(reinstalled == options, "reinstall changed saved choices");

    let skill = home.join(".agents/skills/engineering-judgment/SKILL.md");
    let expected_skill = fs::read(&skill)?;
    fs::remove_file(&skill)?;
    let binary = env!("CARGO_BIN_EXE_mirket");
    let digest = sha256(&fs::read(binary)?);
    let report = success(&home, &["update", "--from", binary, "--sha256", &digest])?;
    ensure!(
        report["reconfigured"] == true && fs::read(&skill)? == expected_skill,
        "update did not restore managed expertise"
    );
    ensure!(
        selected(&home)? == options,
        "update changed package choices or settings"
    );

    let before = [paths.settings(), codex_path.clone(), claude_path.clone()]
        .into_iter()
        .map(|path| fs::read(&path).map(|bytes| (path, bytes)))
        .collect::<std::io::Result<Vec<_>>>()?;
    let marker = store_root.join("unexpected-package-fixture");
    ensure!(!marker.exists(), "drift marker already exists");
    fs::write(&marker, "extra content must block launch and setup\n")?;
    let probe = invoke(&home, &["package", "probe", "jscpd"])?;
    let setup = invoke(&home, &["setup", "--yes"])?;
    fs::remove_file(&marker)?;
    for output in [probe, setup] {
        ensure!(
            !output.status.success() && String::from_utf8_lossy(&output.stderr).contains("drifted"),
            "package drift was not refused"
        );
    }
    for (path, bytes) in before {
        ensure!(
            fs::read(path)? == bytes,
            "failed setup changed active state"
        );
    }
    let mut codex = fs::read_to_string(&codex_path)?.parse::<toml_edit::DocumentMut>()?;
    let mut claude: Value = serde_json::from_slice(&fs::read(&claude_path)?)?;
    ensure!(
        codex["mcp_servers"][name]["command"].as_str() == Some("user-owned-fixture")
            && claude["mcpServers"][name]["command"] == "user-owned-fixture",
        "unrelated client server changed"
    );
    codex["mcp_servers"]
        .as_table_like_mut()
        .unwrap()
        .remove(name);
    claude["mcpServers"].as_object_mut().unwrap().remove(name);
    fs::write(&codex_path, codex.to_string())?;
    fs::write(&claude_path, serde_json::to_vec_pretty(&claude)?)?;
    success(
        &home,
        &[
            "package",
            "probe",
            "jscpd",
            "--project",
            project.path().to_str().unwrap(),
        ],
    )?;
    Ok(())
}
