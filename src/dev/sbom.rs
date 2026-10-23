//! Artifact-bound inventory from the actual default-feature Cargo resolution.
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    process::Command,
    time::Duration,
};
pub(super) fn write(
    project: &Path,
    stage: &Path,
    artifacts: &BTreeMap<String, String>,
    target: &str,
    source: &str,
) -> Result<()> {
    let bytes = super::command_output(
        Command::new(super::cargo_program())
            .current_dir(project)
            .args([
                "metadata",
                "--locked",
                "--format-version",
                "1",
                "--filter-platform",
                target,
            ]),
        "resolve release SBOM",
        Duration::from_secs(120),
        16 * 1024 * 1024,
    )?;
    let metadata: Value = serde_json::from_slice(&bytes)?;
    let root = metadata["resolve"]["root"]
        .as_str()
        .context("SBOM root missing")?;
    let nodes: BTreeMap<_, _> = metadata["resolve"]["nodes"]
        .as_array()
        .context("SBOM nodes missing")?
        .iter()
        .filter_map(|n| Some((n["id"].as_str()?, n)))
        .collect();
    let mut selected = BTreeSet::new();
    let mut pending = vec![root];
    while let Some(id) = pending.pop() {
        if !selected.insert(id) {
            continue;
        }
        for dep in nodes.get(id).context("unresolved SBOM node")?["deps"]
            .as_array()
            .context("SBOM dependency edges absent")?
        {
            if dep["dep_kinds"].as_array().is_some_and(|k| {
                k.iter()
                    .any(|v| v["kind"].is_null() || v["kind"] == "build")
            }) {
                pending.push(dep["pkg"].as_str().context("SBOM dependency ID missing")?);
            }
        }
    }
    let components:Vec<_>=metadata["packages"].as_array().context("SBOM packages missing")?.iter().filter(|p|p["id"].as_str().is_some_and(|id|selected.contains(id))).map(|p|json!({
        "type":"library","bom-ref":p["id"],"name":p["name"],"version":p["version"],
        "purl":format!("pkg:cargo/{}@{}",p["name"].as_str().unwrap_or(""),p["version"].as_str().unwrap_or("")),
        "properties":[{"name":"mirket:source","value":p["source"].as_str().unwrap_or("local")},{"name":"mirket:scope","value":"resolved runtime and build closure; excludes dev-only edges"}]
    })).collect();
    for (name, binary) in artifacts {
        let sbom = json!({"bomFormat":"CycloneDX","specVersion":"1.6","version":1,
        "metadata":{"component":{"type":"application","name":name,"version":env!("CARGO_PKG_VERSION"),"hashes":[{"alg":"SHA-256","content":binary}]},
            "properties":[{"name":"mirket:target","value":target},{"name":"mirket:source_sha256","value":source}]},"components":components});
        crate::util::atomic_write(
            &stage.join(format!("{name}-sbom.cdx.json")),
            &serde_json::to_vec_pretty(&sbom)?,
        )?;
    }
    let provenance = json!({"buildDefinition":{"buildType":"https://github.com/codemirket/harness/native-dist/v1",
        "externalParameters":{"target":target,"version":env!("CARGO_PKG_VERSION")},
        "resolvedDependencies":[{"uri":"mirket-source","digest":{"sha256":source}}]},
        "runDetails":{"builder":{"id":"https://github.com/codemirket/harness/.github/workflows/check.yml"}}});
    crate::util::atomic_write(
        &stage.join("mirket-provenance.json"),
        &serde_json::to_vec_pretty(&provenance)?,
    )?;
    Ok(())
}
