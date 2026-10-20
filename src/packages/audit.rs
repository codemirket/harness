//! Read-only Docker archive export of an installed image for external scanners.
use crate::{paths::Paths, podman::Engine, util};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path, time::Duration};

pub(crate) fn export(paths: &Paths, alias: &str, output: &Path, candidate: bool) -> Result<Value> {
    util::reject_symlink_components(output)?;
    ensure!(
        output.is_absolute() && !output.exists(),
        "export needs a new absolute output path"
    );
    let image = if candidate && alias != "agent-runtime" {
        super::services::candidate_image(paths, alias)?
    } else if alias == "agent-runtime" {
        if candidate {
            super::agent::prepare(paths)?;
        }
        super::agent::verify(paths)?["receipt"]["image"]
            .as_str()
            .context("agent image missing")?
            .to_owned()
    } else {
        super::services::verify(paths)?["images"][alias]
            .as_str()
            .context("unknown installed design image alias")?
            .to_owned()
    };
    let engine = Engine::resolve(paths)?;
    engine.ready()?;
    ensure!(
        super::oci::image_id(&engine, &image)? == image,
        "image identity changed"
    );
    let parent = output.parent().context("output parent")?.canonicalize()?;
    // Save into an owned temporary directory. Publish with an exclusive hard
    // link so even a destination created during export cannot be overwritten.
    let stage = tempfile::tempdir_in(parent)?;
    let archive = stage.path().join("image.tar");
    (|| {
        engine.run(
            &[
                "save".into(),
                "--format".into(),
                "docker-archive".into(),
                "--output".into(),
                archive.to_string_lossy().into_owned(),
                image.clone(),
            ],
            Duration::from_secs(600),
        )?;
        let mut source = fs::File::open(&archive)?;
        let size = source.metadata()?.len();
        ensure!(size > 0, "empty image export");
        let mut hash = Sha256::new();
        let mut buffer = [0; 64 * 1024];
        loop {
            let count = source.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hash.update(&buffer[..count]);
        }
        fs::hard_link(&archive, output)?;
        Ok(
            json!({"ok":true,"alias":alias,"image_id":image,"platform":"linux/arm64","candidate":candidate,
            "source_sha256":env!("MIRKET_SOURCE_SHA256"),
            "archive":output,"archive_sha256":format!("{:x}",hash.finalize()),"bytes":size,
            "limits":"Image identity/export only; advisory coverage requires a scanner and its database identity."}),
        )
    })()
}
