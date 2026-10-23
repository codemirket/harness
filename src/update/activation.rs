//! Optional activation journal. Installation and service effects retain their existing owners.
use super::*;
use anyhow::ensure;
use serde_json::{Value, json};

fn running_groups(status: &Value) -> Result<Vec<String>> {
    let services = status["services"]
        .as_array()
        .context("missing live service observations")?;
    let mut groups = Vec::new();
    for (service, group) in [
        ("design-tools", "core"),
        ("kroki", "diagrams"),
        ("kroki-excalidraw", "excalidraw"),
    ] {
        if services
            .iter()
            .any(|v| v["service"] == service && v["running"] == true)
        {
            groups.push(group.into());
        }
    }
    ensure!(
        groups.is_empty() || groups[0] == "core",
        "partial service topology: inspect before update activation"
    );
    ensure!(
        !groups.iter().any(|g| g == "excalidraw") || groups.iter().any(|g| g == "diagrams"),
        "partial diagram topology: inspect before update activation"
    );
    Ok(groups)
}

fn save(path: &Path, journal: &Value) -> Result<()> {
    crate::util::atomic_write_with_mode(path, &serde_json::to_vec_pretty(journal)?, Some(0o600))
}

pub(super) fn update(paths: &Paths, options: &UpdateOptions) -> Result<UpdateReport> {
    if !options.plan && !options.activate {
        return super::install_update(paths, options);
    }
    ensure!(
        !options.check && !(options.plan && options.activate),
        "select one update mode"
    );
    // Local hash-bound candidates make interruption recovery reproducible. Release
    // discovery remains available in installation-only mode.
    let source = options
        .from
        .as_ref()
        .context("activation requires --from and --sha256 for a reproducible candidate")?;
    let expected = options
        .sha256
        .as_ref()
        .context("activation requires --sha256")?;
    super::validate_digest(expected)?;
    let expected = expected.to_ascii_lowercase();
    ensure!(
        options.expected_entrypoint_sha256.is_none() || options.entrypoint.is_some(),
        "--expected-entrypoint-sha256 requires --entrypoint"
    );
    let entrypoint_request = json!({"path":options.entrypoint,
        "reset_version_from": options.reset_version_from,
        "expected_sha256":options.expected_entrypoint_sha256.as_ref().map(|v| v.to_ascii_lowercase())});
    ensure!(
        sha256(&read_bounded(source, MAX_BINARY)?) == expected,
        "activation candidate digest mismatch"
    );
    let _lock = crate::util::FileLock::acquire(&paths.root.join("update-activation.lock"))?;
    let journal_path = paths.root.join("update-activation.json");
    reject_symlink_components(&journal_path)?;
    let mut journal = if journal_path.exists() {
        let value: Value = serde_json::from_slice(&read_bounded(&journal_path, 64 * 1024)?)?;
        if value["phase"] != "services_restored" {
            ensure!(
                value["candidate_sha256"] == expected && value["schema_version"] == 1,
                "pending activation belongs to another candidate; inspect its journal before proceeding"
            );
            ensure!(
                value["entrypoint_request"] == entrypoint_request,
                "pending activation entrypoint options changed; resume with the original options"
            );
            Some(value)
        } else {
            None
        }
    } else {
        None
    };
    // Preflight the exact staged executable before stopping anything.
    let mut check = options.clone();
    check.plan = false;
    check.activate = false;
    check.check = true;
    check.entrypoint = None;
    check.expected_entrypoint_sha256 = None;
    let mut report = super::install_update(paths, &check)?;
    if let Some(entrypoint) = options.entrypoint.as_deref() {
        super::EntrypointPlan::prepare(
            paths,
            entrypoint,
            options.expected_entrypoint_sha256.as_deref(),
            &read_bounded(source, MAX_BINARY)?,
            &expected,
        )?
        .recheck()?;
    }
    if journal.is_none() {
        let state = read_state(paths)?.context("setup required")?;
        let groups = if state.options.packages.iter().any(|id| id == "design") {
            running_groups(&crate::packages::activation_status(paths)?)?
        } else {
            vec![]
        };
        journal = Some(json!({"schema_version":1,"candidate_sha256":expected,
            "entrypoint_request":entrypoint_request,
            "previous_sha256":sha256(&read_bounded(&paths.binary(),MAX_BINARY)?),
            "groups":groups,"phase":"prepared","clients":"pending_reconnection",
            "checklist":["validate candidate","stop previously running design groups","apply shared setup transaction","verify installed binary","restore recorded groups","observe each client after reconnection"]}));
    }
    let mut journal = journal.unwrap();
    if options.plan {
        report.activation = journal;
        return Ok(report);
    }
    let groups: Vec<String> = serde_json::from_value(journal["groups"].clone())?;
    ensure!(
        groups
            .iter()
            .all(|g| ["core", "diagrams", "excalidraw"].contains(&g.as_str())),
        "invalid activation groups"
    );
    save(&journal_path, &journal)?;
    let installed = sha256(&read_bounded(&paths.binary(), MAX_BINARY)?);
    ensure!(
        installed == expected || Some(installed.as_str()) == journal["previous_sha256"].as_str(),
        "installed binary changed outside this activation; inspect before resuming"
    );
    if journal["phase"] == "prepared" {
        if !groups.is_empty() {
            crate::packages::service_stop(paths, "design", false)?;
        }
        journal["phase"] = json!("services_stopped");
        save(&journal_path, &journal)?;
    }
    if journal["phase"] == "services_stopped" {
        report = super::install_update(paths, options)?;
        journal["phase"] = json!("installed");
        save(&journal_path, &journal)?;
    }
    ensure!(
        journal["phase"] == "installed",
        "unknown activation phase; inspect journal"
    );
    ensure!(
        sha256(&read_bounded(&paths.binary(), MAX_BINARY)?) == expected,
        "installed candidate drift before service restoration"
    );
    if let Some(entrypoint) = options.entrypoint.as_deref() {
        ensure!(
            sha256(&read_bounded(entrypoint, MAX_BINARY)?) == expected,
            "installed entrypoint drift before service restoration"
        );
    }
    if !groups.is_empty() {
        let mut command = Command::new(paths.binary());
        command
            .env("MIRKET_DIRECT", "1")
            .arg("--home")
            .arg(&paths.home)
            .args(["package", "start", "design"]);
        for group in &groups {
            command.args(["--group", group]);
        }
        // Existing start is idempotent and refuses to start a stopped VM.
        run_bounded(&mut command, Duration::from_secs(600))
            .context("service restoration pending; rerun the same activation to resume")?;
    }
    journal["phase"] = json!("services_restored");
    save(&journal_path, &journal)?;
    report.activation = journal;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn activation_preserves_only_observed_running_groups() {
        assert_eq!(running_groups(&json!({"services":[{"service":"design-tools","running":true},{"service":"kroki","running":false}]})).unwrap(), ["core"]);
        assert!(running_groups(&json!({"services":[{"service":"kroki","running":true}]})).is_err());
        assert!(running_groups(&json!({"error":"VM stopped"})).is_err());
        assert!(running_groups(&json!({"services":[]})).unwrap().is_empty());
    }
}
