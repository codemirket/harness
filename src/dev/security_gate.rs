//! Release risk gate over retained raw evidence; report-mode success is not promotion.
use crate::util;
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    io::Read,
    path::Path,
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
fn text<'a>(v: &'a Value, k: &str) -> Result<&'a str> {
    v[k].as_str()
        .filter(|s| !s.is_empty())
        .with_context(|| format!("missing security field: {k}"))
}
fn list<'a>(v: &'a Value, k: &str) -> Result<&'a Vec<Value>> {
    v[k].as_array()
        .with_context(|| format!("missing security list: {k}"))
}
fn evidence(root: &Path, reference: &Value) -> Result<Value> {
    let relative = Path::new(text(reference, "path")?);
    ensure!(
        !relative.is_absolute()
            && relative
                .components()
                .all(|c| matches!(c, std::path::Component::Normal(_))),
        "security evidence path must stay in bundle"
    );
    let path = root.join(relative);
    util::reject_symlink_components(&path)?;
    let bytes = util::read_bounded(&path, 64 * 1024 * 1024)?;
    ensure!(
        util::sha256(&bytes) == text(reference, "sha256")?,
        "security evidence hash mismatch"
    );
    Ok(serde_json::from_slice(&bytes)?)
}
fn disposition(f: &Value, artifact: &str, entries: &[Value], now: u64) -> Result<()> {
    let severity = f["Severity"].as_str().unwrap_or("UNKNOWN");
    if ["LOW", "MEDIUM", "NONE"].contains(&severity) {
        return Ok(());
    }
    ensure!(
        ["HIGH", "CRITICAL", "UNKNOWN"].contains(&severity),
        "invalid severity"
    );
    let matches: Vec<_> = entries
        .iter()
        .filter(|d| {
            d["artifact"] == artifact
                && d["advisory"] == f["VulnerabilityID"]
                && d["package"] == f["PkgName"]
                && d["version"] == f["InstalledVersion"]
                && d["path"] == f["PkgPath"]
        })
        .collect();
    ensure!(
        matches.len() == 1,
        "high/critical/unknown finding requires exactly one artifact/package/path disposition: {artifact} {}",
        f["VulnerabilityID"]
    );
    let d = matches[0];
    ensure!(
        d["expires_unix"].as_u64().is_some_and(|v| v > now),
        "expired security disposition"
    );
    for field in ["owner", "reason", "function_scope", "evidence_url"] {
        text(d, field)?;
    }
    let maintenance_only = f["VulnerabilityID"] == "RUSTSEC-2024-0436"
        && f["PkgName"] == "paste"
        && f["InstalledVersion"] == "1.0.15"
        && d["status"] == "maintenance_accepted"
        && d["expires_unix"]
            .as_u64()
            .is_some_and(|n| n <= now + 31 * 86400);
    ensure!(
        maintenance_only || ["not_affected", "vendor_disputed"].contains(&text(d, "status")?),
        "reachable or unknown high/critical risk blocks promotion"
    );
    Ok(())
}
fn supported_runtime(component: &Value, name: &str, floor: &[u64], now: u64) -> Result<()> {
    let parts: Vec<u64> = text(component, "version")?
        .split('.')
        .map(str::parse)
        .collect::<std::result::Result<_, _>>()?;
    ensure!(
        parts.len() == floor.len() && parts[0] == floor[0] && parts.as_slice() >= floor,
        "unsupported or stale runtime: {name}"
    );
    ensure!(
        component["support_until_unix"]
            .as_u64()
            .is_some_and(|n| n > now),
        "runtime support expired: {name}"
    );
    Ok(())
}
fn sbom_subject(sbom: &Value, id: &str) -> Result<()> {
    ensure!(sbom["bomFormat"] == "CycloneDX", "unsupported SBOM");
    let component = &sbom["metadata"]["component"];
    let hash = component["hashes"].as_array().is_some_and(|items| {
        items
            .iter()
            .any(|h| h["alg"] == "SHA-256" && h["content"] == id)
    });
    let image = component["properties"].as_array().is_some_and(|items| {
        items
            .iter()
            .any(|p| p["name"] == "aquasecurity:trivy:ImageID" && p["value"] == id)
    });
    ensure!(hash || image, "SBOM belongs to another artifact");
    ensure!(
        sbom["components"]
            .as_array()
            .is_some_and(|items| !items.is_empty()),
        "empty SBOM component inventory"
    );
    Ok(())
}
pub fn validate(path: &Path, source: &Value, builds: &[Value]) -> Result<()> {
    let root = path.parent().context("security bundle directory")?;
    let bundle: Value = serde_json::from_slice(&util::read_bounded(path, 8 * 1024 * 1024)?)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    ensure!(
        bundle["schema"] == 1 && bundle["source_sha256"] == *source,
        "stale security source binding"
    );
    let created = bundle["created_unix"]
        .as_u64()
        .context("security evidence date missing")?;
    ensure!(
        created <= now && now - created <= 7 * 86400,
        "security evidence is stale or future dated"
    );
    let dispositions = list(&bundle, "dispositions")?;
    let images = evidence(root, &bundle["image_summary"])?;
    ensure!(
        images["coverage_complete"] == true && images["candidate_source_sha256"] == *source,
        "image evidence incomplete or stale"
    );
    ensure!(
        images["scanner_unchanged"] == true && images["registries_unchanged"] == true,
        "scanner or registry changed during audit"
    );
    for database in ["VulnerabilityDB", "JavaDB"] {
        let updated = images["database_updated_unix"][database]
            .as_u64()
            .context("scanner database age absent")?;
        ensure!(
            updated <= now && now - updated <= 7 * 86400,
            "scanner database is stale or future dated"
        );
    }
    for (name, bytes) in [
        (
            "registry/design-stack.json",
            include_bytes!("../../registry/design-stack.json").as_slice(),
        ),
        (
            "registry/agent-runtime.json",
            include_bytes!("../../registry/agent-runtime.json").as_slice(),
        ),
    ] {
        ensure!(
            images["registry_sha256"][name] == util::sha256(bytes),
            "scanned registry differs: {name}"
        );
    }
    let inventory = images["images"]
        .as_object()
        .context("image inventory missing")?;
    let expected_aliases: BTreeSet<&str> = [
        "design-tools",
        "agent-runtime",
        "node-build",
        "go-build",
        "agent-build",
        "agent-base",
    ]
    .into_iter()
    .collect();
    ensure!(
        inventory
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            == expected_aliases,
        "image inventory has missing or retired runtime aliases"
    );
    let refs = bundle["image_reports"]
        .as_object()
        .context("raw image report references missing")?;
    ensure!(
        refs.keys().map(String::as_str).collect::<BTreeSet<_>>() == expected_aliases,
        "image reports have missing or retired runtime aliases"
    );
    for alias in [
        "design-tools",
        "agent-runtime",
        "node-build",
        "go-build",
        "agent-build",
        "agent-base",
    ] {
        let image = inventory
            .get(alias)
            .with_context(|| format!("unscanned image: {alias}"))?;
        ensure!(
            image["inventory_complete"] == true,
            "image inventory incomplete: {alias}"
        );
        let raw = evidence(root, refs.get(alias).context("raw image report missing")?)?;
        ensure!(
            refs[alias]["sha256"] == image["report_sha256"],
            "raw image report binding mismatch"
        );
        let identity = text(image, "target")?;
        ensure!(
            raw["Metadata"]["ImageID"] == image["image_id"]
                && (raw["ArtifactName"] == identity || raw["Metadata"]["ImageID"] == identity),
            "image identity mismatch"
        );
        ensure!(raw["Metadata"]["OS"]["EOSL"] != true, "unsupported OS");
        let results = list(&raw, "Results")?;
        ensure!(
            results.iter().any(|r| r["Class"] == "os-pkgs"
                && r["Packages"].as_array().is_some_and(|p| !p.is_empty())),
            "empty OS inventory"
        );
        for result in results {
            if let Some(findings) = result["Vulnerabilities"].as_array() {
                for finding in findings {
                    disposition(finding, identity, dispositions, now)?;
                }
            }
        }
        if ["design-tools", "agent-runtime"].contains(&alias) {
            ensure!(
                image["payloads"]["inventory_complete"] == true,
                "native/browser payload coverage missing"
            );
        }
    }
    let source_report = evidence(root, &bundle["source_report"])?;
    ensure!(
        source_report["ok"] == true && source_report["queried"].as_u64().is_some_and(|n| n > 0),
        "source audit incomplete"
    );
    for (name, bytes) in [
        ("Cargo.lock", include_bytes!("../../Cargo.lock").as_slice()),
        (
            "runtime/agent/native/Cargo.lock",
            include_bytes!("../../runtime/agent/native/Cargo.lock").as_slice(),
        ),
        (
            "runtime/design/tools/package-lock.json",
            include_bytes!("../../runtime/design/tools/package-lock.json").as_slice(),
        ),
    ] {
        ensure!(
            source_report["lockfiles"][name] == util::sha256(bytes),
            "source scan lockfile differs: {name}"
        );
    }
    for finding in list(&source_report, "findings")? {
        // Missing publisher severity remains unknown. Report-mode exceptions
        // cannot bypass the release disposition contract.
        let f = serde_json::json!({"VulnerabilityID":finding["advisory"],"PkgName":finding["name"],"InstalledVersion":finding["version"],"PkgPath":null,"Severity":finding["severity"].as_str().unwrap_or("UNKNOWN")});
        disposition(&f, text(&bundle, "source_sha256")?, dispositions, now)?;
    }
    let runtime = evidence(root, &bundle["runtime_inventory"])?;
    ensure!(
        runtime["source_sha256"] == *source,
        "runtime inventory stale"
    );
    ensure!(
        runtime["host"]["kernel"]
            .as_str()
            .is_some_and(|s| !s.is_empty())
            && runtime["host"]["podman"]
                .as_str()
                .is_some_and(|s| !s.is_empty()),
        "VM/kernel inventory absent; headers are not kernel evidence"
    );
    for (name, floor) in [
        ("node", vec![24, 22, 1]),
        ("chromium", vec![156, 0, 8078, 47]),
        ("go", vec![1, 27, 3]),
    ] {
        let component = &runtime["components"][name];
        supported_runtime(component, name, &floor, now)?;
        evidence(root, &component["observation"])?;
    }
    // Require artifact-specific SBOM/provenance references for every native build
    // and owned image. A signature verification receipt retains issuer/identity.
    let mut required: BTreeSet<String> = builds
        .iter()
        .map(|b| b["binary_sha256"].as_str().unwrap_or("").to_owned())
        .collect();
    for build in builds {
        let binaries = build["artifacts"]
            .as_object()
            .context("native installer inventory absent")?;
        for name in ["mirket", "install-codex", "install-claude", "install-all"] {
            required.insert(text(&Value::Object(binaries.clone()), name)?.to_owned());
        }
    }
    for alias in ["design-tools", "agent-runtime"] {
        required.insert(text(&inventory[alias], "image_id")?.to_owned());
    }
    let mut seen = BTreeSet::new();
    for item in list(&bundle, "artifacts")? {
        let id = text(item, "id")?;
        ensure!(seen.insert(id.to_owned()), "duplicate security artifact");
        let sbom = evidence(root, &item["sbom"])?;
        sbom_subject(&sbom, id)?;
        let payload = root.join(text(item, "payload")?);
        let signature = root.join(text(item, "sigstore_bundle")?);
        for file in [&payload, &signature] {
            ensure!(
                file.canonicalize()?.starts_with(root.canonicalize()?),
                "attestation path outside bundle"
            );
            util::reject_symlink_components(file)?;
        }
        let mut hash = sha2::Sha256::new();
        let mut file = std::fs::File::open(&payload)?;
        let mut buffer = [0u8; 65536];
        use sha2::Digest;
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hash.update(&buffer[..n]);
        }
        let payload_digest = format!("{:x}", hash.finalize());
        ensure!(
            payload_digest == text(item, "payload_sha256")?,
            "signed artifact bytes differ"
        );
        let verified = super::command_output(
            Command::new("gh")
                .args(["attestation", "verify"])
                .arg(&payload)
                .args(["--bundle"])
                .arg(&signature)
                .args([
                    "--repo",
                    "codemirket/harness",
                    "--signer-workflow",
                    "codemirket/harness/.github/workflows/check.yml",
                    "--predicate-type",
                    "https://github.com/codemirket/harness/provenance/v1",
                    "--format",
                    "json",
                ]),
            "verify release Sigstore identity and signature",
            Duration::from_secs(120),
            8 * 1024 * 1024,
        )?;
        let verified: Value = serde_json::from_slice(&verified)?;
        ensure!(
            verified.as_array().is_some_and(|v| !v.is_empty()),
            "empty signature verification result"
        );
        let attestation = evidence(root, &item["signature_verification"])?;
        ensure!(
            attestation["verified"] == true && attestation["subject"] == id,
            "unverified provenance subject"
        );
        text(&attestation, "identity")?;
        text(&attestation, "issuer")?;
        let provenance = evidence(root, &item["provenance"])?;
        ensure!(
            verified.as_array().is_some_and(|items| items
                .iter()
                .any(|v| v["verificationResult"]["statement"]["predicate"] == provenance)),
            "signed predicate differs from retained provenance"
        );
        ensure!(
            provenance["buildDefinition"]["resolvedDependencies"]
                .as_array()
                .is_some_and(|items| items
                    .iter()
                    .any(|v| v["uri"] == "mirket-source" && v["digest"]["sha256"] == *source)),
            "provenance source differs from candidate"
        );
        ensure!(
            id == payload_digest
                || provenance["runDetails"]["byproducts"]
                    .as_array()
                    .is_some_and(|items| items
                        .iter()
                        .any(|v| v["digest"]["sha256"] == payload_digest
                            && v["annotations"]["image_id"] == id)),
            "OCI subject does not bind image identity"
        );
        super::command_output(
            Command::new("gh")
                .args(["attestation", "verify"])
                .arg(root.join(text(&item["sbom"], "path")?))
                .args(["--bundle"])
                .arg(&signature)
                .args([
                    "--repo",
                    "codemirket/harness",
                    "--signer-workflow",
                    "codemirket/harness/.github/workflows/check.yml",
                    "--predicate-type",
                    "https://github.com/codemirket/harness/provenance/v1",
                ]),
            "verify artifact SBOM signature",
            Duration::from_secs(120),
            8 * 1024 * 1024,
        )?;
    }
    ensure!(
        required == seen,
        "artifact SBOM/provenance coverage differs from reviewed runtime"
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn store(root: &Path, name: &str, value: &Value) -> Value {
        let bytes = serde_json::to_vec(value).unwrap();
        std::fs::write(root.join(name), &bytes).unwrap();
        json!({"path":name,"sha256":util::sha256(&bytes)})
    }
    #[test]
    fn promotion_rejects_missing_stale_and_incomplete_evidence() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = root.join("security.json");
        let source = json!("a".repeat(64));
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut bundle =
            json!({"schema":1,"source_sha256":source,"created_unix":now,"dispositions":[]});
        let check = |bundle: &Value, expected: &str| {
            std::fs::write(&path, serde_json::to_vec(bundle).unwrap()).unwrap();
            let error = validate(&path, &source, &[]).unwrap_err().to_string();
            assert!(error.contains(expected), "{error}");
        };
        check(&bundle, "missing security field");
        bundle["created_unix"] = json!(now - 8 * 86400);
        check(&bundle, "stale or future");
        bundle["created_unix"] = json!(now);
        bundle["source_sha256"] = json!("old");
        check(&bundle, "stale security source");
        bundle["source_sha256"] = source.clone();
        bundle["image_summary"] = store(
            &root,
            "images.json",
            &json!({"coverage_complete":false,"candidate_source_sha256":source}),
        );
        check(&bundle, "image evidence incomplete");
        bundle["image_summary"] = store(
            &root,
            "images.json",
            &json!({"coverage_complete":true,"candidate_source_sha256":"old"}),
        );
        check(&bundle, "image evidence incomplete or stale");
        bundle["image_summary"] = store(
            &root,
            "images.json",
            &json!({"coverage_complete":true,"candidate_source_sha256":source,"images":{}}),
        );
        check(&bundle, "scanner or registry changed");
        std::fs::write(temp.path().join("images.json"), b"{}").unwrap();
        check(&bundle, "security evidence hash mismatch");
    }
    #[test]
    fn security_evidence_cannot_escape_bundle() {
        let temp = tempfile::tempdir().unwrap();
        for path in ["../other.json", "/tmp/other.json", "nested/../other.json"] {
            assert!(evidence(temp.path(), &json!({"path":path,"sha256":"ignored"})).is_err());
        }
    }
    #[test]
    fn unsupported_browser_and_expired_runtime_block_promotion() {
        let mut component = json!({"version":"156.0.8078.47","support_until_unix":200});
        assert!(supported_runtime(&component, "chromium", &[156, 0, 8078, 47], 100).is_ok());
        for version in ["156.0.8078.4", "155.0.9999.99", "unknown", "156.0"] {
            component["version"] = json!(version);
            assert!(supported_runtime(&component, "chromium", &[156, 0, 8078, 47], 100).is_err());
        }
        component["version"] = json!("24.22.1");
        assert!(supported_runtime(&component, "node", &[24, 22, 1], 200).is_err());
    }
    #[test]
    fn signed_sbom_must_describe_the_selected_artifact() {
        let bom = json!({"bomFormat":"CycloneDX","metadata":{"component":{"hashes":[{"alg":"SHA-256","content":"one"}]}},"components":[{"name":"library"}]});
        assert!(sbom_subject(&bom, "one").is_ok());
        assert!(sbom_subject(&bom, "other").is_err());
        let mut empty = bom;
        empty["components"] = json!([]);
        assert!(sbom_subject(&empty, "one").is_err());
    }
    #[test]
    fn risk_is_not_cleared_by_missing_fix_or_stale_waiver() {
        let f = json!({"Severity":"HIGH","VulnerabilityID":"CVE-test","PkgName":"zlib1g","InstalledVersion":"1","PkgPath":"/lib/z.so"});
        assert!(disposition(&f, "sha256:one", &[], 100).is_err());
        let d = json!({"artifact":"sha256:one","advisory":"CVE-test","package":"zlib1g","version":"1","path":"/lib/z.so","owner":"runtime","reason":"vendor confirms this binary omits affected minizip","function_scope":"contrib/minizip","evidence_url":"https://security-tracker.debian.org/","expires_unix":200,"status":"not_affected"});
        assert!(disposition(&f, "sha256:one", std::slice::from_ref(&d), 100).is_ok());
        assert!(disposition(&f, "sha256:other", std::slice::from_ref(&d), 100).is_err());
        assert!(disposition(&f, "sha256:one", std::slice::from_ref(&d), 200).is_err());
        let mut reachable = d;
        reachable["status"] = json!("reachable");
        assert!(disposition(&f, "sha256:one", &[reachable], 100).is_err());
        assert_eq!(f["Severity"], "HIGH");
    }
}
