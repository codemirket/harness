//! Candidate-bound contained acceptance and release assembly. Receipts are evidence, not authority.
use crate::{source_identity, util};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    io::Write,
    path::{Path, PathBuf},
};
pub const CHECKS: &[&str] = &[
    "contained-runtime",
    "desktop-contracts",
    "parallel-runtime",
    "design-output",
    "backup-recovery",
];
pub fn require_new(path: &Path) -> Result<()> {
    util::reject_symlink_components(path)?;
    ensure!(!path.try_exists()?, "receipt output already exists");
    Ok(())
}
pub fn save_new(path: &Path, value: &Value) -> Result<()> {
    require_new(path)?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer_pretty(&mut temp, value)?;
    temp.write_all(b"\n")?;
    temp.as_file().sync_all()?;
    require_new(path)?;
    temp.persist_noclobber(path)?;
    Ok(())
}
pub fn candidate_identity(project: &Path) -> Result<Value> {
    let source = source_identity::fingerprint(project)?;
    ensure!(
        source == env!("MIRKET_SOURCE_SHA256"),
        "acceptance candidate differs from checkout; build and use the matching candidate executable"
    );
    let binary = util::sha256(&util::read_bounded(
        &std::env::current_exe()?,
        256 * 1024 * 1024,
    )?);
    ensure!(
        crate::identity::capture().as_ref() == Some(&binary),
        "candidate executable changed since startup"
    );
    Ok(
        json!({"source_sha256":source,"binary_sha256":binary,"target":env!("MIRKET_TARGET"),"version":env!("CARGO_PKG_VERSION")}),
    )
}
fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&util::read_bounded(
        path,
        1024 * 1024,
    )?)?)
}
fn validate(receipt: &Value, builds: &[Value]) -> Result<()> {
    ensure!(
        receipt["schema"] == 1 && receipt["ok"] == true && receipt["checks"] == json!(CHECKS),
        "incomplete contained acceptance receipt"
    );
    let identity = &receipt["identity"];
    for key in ["source_sha256", "binary_sha256"] {
        ensure!(
            identity[key]
                .as_str()
                .is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())),
            "invalid acceptance identity"
        );
    }
    ensure!(!builds.is_empty(), "no candidate builds");
    for build in builds {
        ensure!(
            build["schema"] == 1
                && build["source_sha256"] == identity["source_sha256"]
                && build["version"] == identity["version"],
            "release inputs differ from accepted source/version (including locks, recipes, tests and toolchain)"
        );
    }
    ensure!(
        builds.iter().any(|b| b["target"] == identity["target"]
            && b["binary_sha256"] == identity["binary_sha256"]),
        "accepted executable is not one of the release candidates"
    );
    Ok(())
}
pub fn promote(inputs: &[PathBuf], output: &Path, acceptance: &Path) -> Result<Value> {
    let receipt = read(acceptance)?;
    let mut builds = Vec::new();
    for input in inputs {
        let build = read(
            &input
                .parent()
                .context("manifest directory")?
                .join("mirket-build.json"),
        )?;
        let manifest: crate::update::ReleaseManifest = serde_json::from_value(read(input)?)?;
        ensure!(
            manifest.assets.len() == 1,
            "promotion inputs must be individual native distributions"
        );
        let asset = &manifest.assets[0];
        ensure!(
            build["version"] == manifest.version
                && build["target"] == asset.target
                && build["binary_sha256"] == asset.sha256,
            "build receipt differs from native manifest"
        );
        builds.push(build);
    }
    validate(&receipt, &builds)?;
    // Existing verifier checks every local asset byte before atomic publication.
    crate::release::manifest(inputs, output)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn promotion_rejects_stale_source_binary_or_missing_checks() {
        let identity = json!({"source_sha256":"a".repeat(64),"binary_sha256":"b".repeat(64),"target":"aarch64-apple-darwin","version":"1.0.11"});
        let receipt = json!({"schema":1,"ok":true,"checks":CHECKS,"identity":identity});
        let mut build = identity.clone();
        build["schema"] = json!(1);
        assert!(validate(&receipt, &[build.clone()]).is_ok());
        for key in ["source_sha256", "binary_sha256", "version", "target"] {
            let mut changed = build.clone();
            changed[key] = json!("changed");
            assert!(validate(&receipt, &[changed]).is_err(), "{key}");
        }
        let mut missing = receipt.clone();
        missing["checks"] = json!([]);
        assert!(validate(&missing, &[build.clone()]).is_err());
        let mut other = build.clone();
        other["target"] = json!("x86_64-pc-windows-msvc");
        other["binary_sha256"] = json!("c".repeat(64));
        assert!(validate(&receipt, &[build, other]).is_ok());
    }
    #[test]
    fn receipt_never_overwrites_prior_evidence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().canonicalize().unwrap().join("acceptance.json");
        save_new(&path, &json!({"first":true})).unwrap();
        assert!(save_new(&path, &json!({"second":true})).is_err());
        assert_eq!(read(&path).unwrap(), json!({"first":true}));
    }
}
