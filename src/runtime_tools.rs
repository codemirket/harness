//! Explicit setup choices overlay a verified release without modifying its bytes.
//! A binding is never discovered through PATH and is rechecked before every job.
use crate::release::AdapterPayload;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Choices {
    schema: u32,
    release_id: String,
    tools: BTreeMap<String, Binding>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    choice: String,
    path: Option<PathBuf>,
    sha256: Option<String>,
    version: Option<String>,
}
const IDS: [&str; 1] = ["chromium"];
fn choices(state: &Path, release: &str) -> Result<Option<Choices>, String> {
    let file = state
        .parent()
        .ok_or("runtime state has no installation home")?
        .join("runtime-tools.json");
    let bytes = match fs::symlink_metadata(&file) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("runtime choices unavailable".into()),
        Ok(m) if !m.is_file() || m.file_type().is_symlink() || m.len() > 65536 => {
            return Err("invalid runtime choices file".into());
        }
        Ok(_) => fs::read(file).map_err(|_| "runtime choices unreadable")?,
    };
    let value: Choices = serde_json::from_slice(&bytes).map_err(|_| "invalid runtime choices")?;
    if value.schema != 1
        || value.release_id != release
        || value.tools.keys().any(|k| !IDS.contains(&k.as_str()))
    {
        return Err("runtime choices differ from this release; run /mirket setup".into());
    }
    for binding in value.tools.values() {
        if !["skip", "installed", "pinned"].contains(&binding.choice.as_str())
            || (binding.choice == "skip" && (binding.path.is_some() || binding.sha256.is_some()))
            || (binding.choice != "skip"
                && (binding.path.as_ref().is_none_or(|p| !p.is_absolute())
                    || binding.sha256.as_ref().is_none_or(|h| {
                        h.len() != 64
                            || !h
                                .bytes()
                                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                    })
                    || binding
                        .version
                        .as_ref()
                        .is_none_or(|v| v.is_empty() || v.len() > 256)))
        {
            return Err("invalid runtime tool binding".into());
        }
    }
    Ok(Some(value))
}
fn selected<'a>(payload: &'a AdapterPayload, id: &str) -> Option<&'a Path> {
    match id {
        "chromium" => payload.video.as_ref().map(|p| p.chromium.as_path()),
        _ => None,
    }
}
fn verify(binding: &Binding) -> Result<&Path, String> {
    let path = binding.path.as_deref().ok_or("runtime tool was skipped")?;
    let identity = crate::process_video::file_identity(path)?;
    if identity.strip_prefix("sha256:") != binding.sha256.as_deref() {
        return Err("runtime executable changed; run /mirket setup to select it again".into());
    }
    Ok(path)
}
pub fn apply(state: &Path, release: &str, payload: &mut AdapterPayload) -> Result<(), String> {
    let choices = match choices(state, release) {
        Ok(choices) => choices,
        Err(_) => {
            payload.video = None;
            return Ok(());
        }
    };
    if let Some(choices) = &choices {
        for (id, binding) in &choices.tools {
            if id == "chromium"
                && let Some(video) = payload.video.as_mut()
            {
                video.chromium = verify(binding)
                    .ok()
                    .map(Path::to_path_buf)
                    .unwrap_or_default();
            }
        }
    }
    if payload.video.as_ref().is_some_and(|video| {
        !video.tools_configured()
            || [&video.chromium, &video.ffmpeg, &video.ffprobe]
                .iter()
                .any(|p| !p.is_file())
    }) {
        payload.video = None;
    }
    Ok(())
}
pub fn verify_for(
    state: &Path,
    release: &str,
    payload: &AdapterPayload,
    ids: &[&str],
) -> Result<(), String> {
    let selected_choices = choices(state, release)?;
    let installed = state
        .parent()
        .is_some_and(|home| home.join("installation.json").is_file());
    for id in ids {
        if let Some(binding) = selected_choices.as_ref().and_then(|c| c.tools.get(*id)) {
            let path = verify(binding)?;
            if selected(payload, id) != Some(path) {
                return Err("runtime selection changed; reload pi before starting jobs".into());
            }
        } else if installed {
            return Err("runtime binding missing; run /mirket setup before starting video".into());
        }
    }
    Ok(())
}
pub fn status(state: &Path, release: &str, payload: &AdapterPayload) -> Value {
    let choices = match choices(state, release) {
        Ok(c) => c,
        Err(error) => return json!({"status":"changed","error":error}),
    };
    let tools: Vec<Value> = ["chromium"].iter().map(|id| {
        if let Some(binding) = choices.as_ref().and_then(|c| c.tools.get(*id)) {
            let status = if binding.choice == "skip" { "missing" } else if verify(binding).is_ok() { "ready" } else { "changed" };
            json!({"id":id,"status":status,"choice":binding.choice,"path":binding.path,"version":binding.version,"active":selected(payload,id)==binding.path.as_deref()})
        } else { json!({"id":id,"status":if selected(payload,id).is_some(){"ready"}else{"missing"},"choice":"package","path":selected(payload,id)}) }
    }).collect();
    json!({"tools":tools})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_identity_is_rechecked_and_skip_disables() {
        let root = std::env::temp_dir().join(format!(
            "mirket-tool-binding-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let home = root.as_path();
        let state = home.join("state");
        fs::create_dir(&state).unwrap();
        let exe = home.join("chrome");
        fs::write(&exe, b"first").unwrap();
        let digest = crate::process_video::file_identity(&exe)
            .unwrap()
            .trim_start_matches("sha256:")
            .to_owned();
        fs::write(home.join("runtime-tools.json"),json!({"schema":1,"releaseId":"r1","tools":{"chromium":{"choice":"installed","path":exe,"sha256":digest,"version":"fixture"}}}).to_string()).unwrap();
        let mut payload = AdapterPayload::explicit(&exe).unwrap();
        payload.video = Some(crate::video::VideoPayload {
            node: exe.clone(),
            adapter: exe.clone(),
            chromium: PathBuf::new(),
            ffmpeg: exe.clone(),
            ffprobe: exe.clone(),
        });
        apply(&state, "r1", &mut payload).unwrap();
        verify_for(&state, "r1", &payload, &["chromium"]).unwrap();
        assert_eq!(
            status(&state, "r1", &payload)["tools"][0]["status"],
            "ready"
        );
        fs::write(home.join("installation.json"), b"fixture installed marker").unwrap();
        let binding_bytes = fs::read(home.join("runtime-tools.json")).unwrap();
        fs::remove_file(home.join("runtime-tools.json")).unwrap();
        assert!(
            verify_for(&state, "r1", &payload, &["chromium"])
                .unwrap_err()
                .contains("binding missing")
        );
        fs::write(home.join("runtime-tools.json"), binding_bytes).unwrap();
        fs::write(&exe, b"changed").unwrap();
        assert!(
            verify_for(&state, "r1", &payload, &["chromium"])
                .unwrap_err()
                .contains("changed")
        );
        assert_eq!(
            status(&state, "r1", &payload)["tools"][0]["status"],
            "changed"
        );
        apply(&state, "r1", &mut payload).unwrap();
        assert!(payload.video.is_none());
        fs::write(
            home.join("runtime-tools.json"),
            json!({"schema":1,"releaseId":"r1","tools":{"chromium":{"choice":"skip"}}}).to_string(),
        )
        .unwrap();
        apply(&state, "r1", &mut payload).unwrap();
        assert!(payload.video.is_none());
        assert_eq!(
            status(&state, "r1", &payload)["tools"][0]["status"],
            "missing"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
