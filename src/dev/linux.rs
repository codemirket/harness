//! Native ARM64 Linux checks in a disposable compiler container.
use crate::{paths::Paths, podman::Engine, source_identity, util};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path, time::Duration};

const IMAGE: &str = "docker.io/library/rust@sha256:8a2a102d089fbd151137b9473c01b1ed4519a64fb9c032a03609b815701615c8";

fn copy(source: &Path, destination: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(source)?;
    ensure!(
        !metadata.file_type().is_symlink(),
        "source snapshot refuses links"
    );
    if metadata.is_dir() {
        fs::create_dir_all(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            if entry.file_name() != ".DS_Store" {
                copy(&entry.path(), &destination.join(entry.file_name()))?;
            }
        }
    } else {
        ensure!(metadata.is_file(), "source snapshot requires regular files");
        fs::create_dir_all(destination.parent().context("snapshot parent")?)?;
        fs::copy(source, destination)?;
    }
    Ok(())
}

fn capture(project: &Path, snapshot: &Path) -> Result<()> {
    fs::create_dir(snapshot)?;
    let mut inputs: BTreeSet<_> = source_identity::INPUTS.iter().copied().collect();
    inputs.extend([
        "docs",
        "evals",
        "website",
        ".github",
        "AGENTS.md",
        "CONTRIBUTING.md",
        "README.md",
        "LICENSE",
        ".gitattributes",
    ]);
    for name in inputs {
        copy(&project.join(name), &snapshot.join(name))
            .with_context(|| format!("snapshot public source {name}"))?;
    }
    Ok(())
}

pub(crate) fn check(project: &Path, engine_home: &Path, output: &Path) -> Result<Value> {
    let project = super::checkout(project)?;
    util::reject_symlink_components(output)?;
    ensure!(
        output.is_absolute() && !output.exists(),
        "Linux evidence directory must be new and absolute"
    );
    let before = source_identity::fingerprint(&project)?;
    let engine = Engine::resolve(&Paths::new(engine_home.to_owned())?)?;
    engine.ready()?;
    fs::create_dir(output)?;
    let stage = tempfile::Builder::new()
        .prefix("linux-check-")
        .tempdir_in(&engine.staging)?;
    let snapshot = stage.path().join("source");
    capture(&project, &snapshot)?;
    let evidence = stage.path().join("evidence");
    fs::create_dir(&evidence)?;
    ensure!(
        source_identity::fingerprint(&snapshot)? == before,
        "snapshot source drift"
    );
    let name = format!(
        "mirket-{}",
        stage
            .path()
            .file_name()
            .context("stage name")?
            .to_string_lossy()
    );
    let script = r#"set -eu
df -h /tmp
cp -R /source /tmp/project
cd /tmp/project
cargo install --locked --debug --path . --bin mirket --root /tmp/bootstrap
/tmp/bootstrap/bin/mirket info --json > /evidence/identity.json
/tmp/bootstrap/bin/mirket dev check --project .
/tmp/bootstrap/bin/mirket catalog check
mkdir /tmp/disposable-home
/tmp/bootstrap/bin/mirket --home /tmp/disposable-home setup --target all --yes
/tmp/bootstrap/bin/mirket --home /tmp/disposable-home doctor --json
/tmp/bootstrap/bin/mirket --home /tmp/disposable-home mcp doctor --iterations 2
df -h /tmp
"#;
    // Source-only mount: no home, credentials, engine socket or private task state.
    let args = vec![
        "run".into(),
        "--rm".into(),
        "--name".into(),
        name.clone(),
        "--platform".into(),
        "linux/arm64".into(),
        "--cap-drop=ALL".into(),
        "--security-opt=no-new-privileges".into(),
        "--pids-limit=512".into(),
        "--memory=6g".into(),
        "--cpus=4".into(),
        "--env=CARGO_BUILD_JOBS=2".into(),
        // Integration tests install several copies of the CLI. Keep assertions
        // and all gates, but avoid exhausting the shared VM with debug symbols
        // and incremental caches that cannot be reused after this run.
        "--env=CARGO_PROFILE_DEV_DEBUG=0".into(),
        "--env=CARGO_PROFILE_TEST_DEBUG=0".into(),
        "--env=CARGO_INCREMENTAL=0".into(),
        "--volume".into(),
        format!("{}:/source:ro", snapshot.display()),
        "--volume".into(),
        format!("{}:/evidence:rw", evidence.display()),
        IMAGE.into(),
        "sh".into(),
        "-c".into(),
        format!(
            "({script}) > /evidence/complete.log 2>&1; outcome=$?; df -h /tmp >> /evidence/complete.log; tail -80 /evidence/complete.log; exit $outcome"
        ),
    ];
    let result = engine.run(&args, Duration::from_secs(2400));
    let _ = engine.run(
        &["rm".into(), "--force".into(), name],
        Duration::from_secs(30),
    );
    let unchanged = source_identity::fingerprint(&project)? == before;
    let log = fs::read_to_string(evidence.join("complete.log")).unwrap_or_else(|_| match &result {
        Ok(log) => log.clone(),
        Err(error) => format!("{error:#}"),
    });
    let identity: Value = fs::read(evidence.join("identity.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or(Value::Null);
    let matched = identity["process"]["source_sha256"] == before
        && identity["process"]["target"] == "aarch64-unknown-linux-gnu";
    util::atomic_write(&output.join("linux-check.log"), log.as_bytes())?;
    let report = json!({"ok":result.is_ok() && unchanged && matched,"source_sha256":before,
        "native_identity":identity,"native_identity_matches":matched,
        "source_unchanged":unchanged,"compiler_image":IMAGE,"platform":"aarch64-unknown-linux-gnu",
        "build_settings":{"jobs":2,"debug_symbols":false,"incremental":false,"debug_assertions":true},
        "log_sha256":util::sha256(log.as_bytes()),
        "limits":"Native Linux ARM64 process execution inside the explicitly selected macOS VM; no Windows, x86 Linux, desktop GUI or package-controller support claim."});
    util::atomic_write(
        &output.join("receipt.json"),
        &serde_json::to_vec_pretty(&report)?,
    )?;
    result?;
    ensure!(unchanged, "source changed during Linux checks");
    ensure!(matched, "Linux candidate identity differs from source");
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn captured_checkout_contains_contract_test_inputs_without_private_state() {
        let project = Path::new(env!("CARGO_MANIFEST_DIR"));
        let temporary = tempfile::tempdir().unwrap();
        let snapshot = temporary.path().join("snapshot");
        capture(project, &snapshot).unwrap();
        assert_eq!(
            source_identity::fingerprint(project).unwrap(),
            source_identity::fingerprint(&snapshot).unwrap()
        );
        // The link gate traverses public guidance beyond the compiled payload.
        // Omitting website previously passed compilation, then failed that gate.
        for path in [
            "website/README.md",
            "docs/README.md",
            "evals/harness-remediation/README.md",
            ".github/workflows/check.yml",
        ] {
            assert_eq!(
                fs::read(project.join(path)).unwrap(),
                fs::read(snapshot.join(path)).unwrap()
            );
        }
        assert!(!snapshot.join(".git").exists());
        assert!(!snapshot.join(".mirket").exists());
    }

    #[test]
    fn snapshot_preserves_bytes_and_refuses_linked_private_input() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("file"), "public source").unwrap();
        let destination = root.path().join("copy");
        copy(&source, &destination).unwrap();
        assert_eq!(
            fs::read(destination.join("file")).unwrap(),
            b"public source"
        );
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("/private", source.join("alias")).unwrap();
            assert!(copy(&source, &root.path().join("rejected")).is_err());
        }
    }
}
