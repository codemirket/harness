//! Shared immutable OCI image inspection for owned packages.
use crate::podman::Engine;
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::time::Duration;

pub(super) fn inspect(engine: &Engine, kind: &str, name: &str) -> Result<Value> {
    serde_json::from_str(&engine.run(
        &[
            kind.into(),
            "inspect".into(),
            name.into(),
            "--format".into(),
            "{{json .}}".into(),
        ],
        Duration::from_secs(30),
    )?)
    .context("invalid Podman inspection")
}
pub(super) fn normalize_image_id(value: &str) -> Result<String> {
    let digest = value.strip_prefix("sha256:").unwrap_or(value);
    ensure!(
        digest.len() == 64
            && digest
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
        "invalid OCI image ID"
    );
    Ok(format!("sha256:{digest}"))
}
pub(super) fn image_id(engine: &Engine, image: &str) -> Result<String> {
    let v = inspect(engine, "image", image)?;
    inspected_image_id(&v, image)
}
pub(super) fn inspected_image_id(v: &Value, image: &str) -> Result<String> {
    ensure!(
        v["Architecture"] == "arm64" && v["Os"] == "linux",
        "image is not native Linux ARM64: {image}"
    );
    let id = normalize_image_id(v["Id"].as_str().context("image ID missing")?)?;
    if let Some((_, digest)) = image.split_once("@sha256:") {
        let pinned = format!("sha256:{digest}");
        let listed = v["RepoDigests"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item.as_str() == Some(image)));
        ensure!(
            listed || v["Digest"] == pinned,
            "image manifest digest drift: {image}"
        );
    }
    Ok(id)
}
