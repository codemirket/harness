//! Owned tool contracts. A request can select a tool, never lower its policy.
use super::policy::{Tier, ToolPolicy};
use crate::{paths::Paths, util};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    collections::BTreeSet,
    io::Read,
    path::{Path, PathBuf},
};

/// Static owned identities only; discovery never starts a package or model here.
pub(crate) fn known_names() -> Result<BTreeSet<String>> {
    let policies: BTreeMap<String, Value> =
        serde_json::from_str(include_str!("../../runtime/design/tools/tool-policy.json"))?;
    Ok([
        "workspace_read",
        "workspace_list",
        "workspace_write",
        "inspect_image",
    ]
    .into_iter()
    .map(str::to_owned)
    .chain(policies.keys().map(|name| format!("design/{name}")))
    .collect())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub path: PathBuf,
    pub sha256: String,
}

pub fn current_digest(path: &Path) -> Result<String> {
    util::reject_symlink_components(path)?;
    let before = std::fs::symlink_metadata(path)?;
    ensure!(
        before.is_file() && before.len() <= 64 * 1024 * 1024,
        "artifact is absent, irregular or exceeds design output limit"
    );
    let mut file = std::fs::File::open(path)?;
    let opened = file.metadata()?;
    ensure!(
        opened.is_file() && opened.len() <= 64 * 1024 * 1024,
        "artifact changed before readback"
    );
    let mut digest = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        bytes += count as u64;
        ensure!(
            bytes <= 64 * 1024 * 1024,
            "artifact exceeds design output limit"
        );
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn design_root(paths: &Paths) -> PathBuf {
    paths.root.join("execution/podman/staging/design/workspace")
}

pub(crate) fn design_scope(paths: &Paths, project: &Path) -> Result<PathBuf> {
    let root = design_root(paths);
    if project.starts_with(&root) && project != root {
        Ok(project.to_owned())
    } else {
        Ok(root
            .join("projects")
            .join(crate::agent_runtime::scope(paths, project)?))
    }
}

fn writable_file(paths: &Paths, project: &Path, value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    if !path.is_absolute() {
        return file(project, value);
    }
    let root = design_scope(paths, project)?;
    let relative = path
        .strip_prefix(&root)
        .context("absolute writes require the assigned design project workspace")?;
    file(&root, relative.to_str().context("non UTF-8 path")?)
}

fn design_file(root: &Path, relative: &str) -> Result<PathBuf> {
    util::safe_relative(Path::new(relative))?;
    let path = root.join(relative);
    util::reject_symlink_components(&path)?;
    Ok(path)
}

fn repository_internal(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|name| name.eq_ignore_ascii_case(".git"))
    })
}

fn readable_file(paths: &Paths, project: &Path, value: &str) -> Result<PathBuf> {
    if !Path::new(value).is_absolute() {
        return file(project, value);
    }
    let path = PathBuf::from(value);
    ensure!(
        !repository_internal(&path),
        "repository internals are outside worker file tools"
    );
    util::reject_symlink_components(&path)?;
    let canonical = path.canonicalize()?;
    ensure!(
        canonical.starts_with(project) || canonical.starts_with(design_scope(paths, project)?),
        "file is outside assigned workspaces"
    );
    Ok(canonical)
}

fn artifact_from_json(root: &Path, value: &Value) -> Result<Option<ArtifactRef>> {
    let Some(path) = value.get("path").and_then(Value::as_str) else {
        return Ok(None);
    };
    let Some(sha256) = value.get("sha256").and_then(Value::as_str) else {
        return Ok(None);
    };
    ensure!(
        sha256.len() == 64 && sha256.bytes().all(|b| b.is_ascii_hexdigit()),
        "design artifact digest is invalid"
    );
    Ok(Some(ArtifactRef {
        path: design_file(root, path)?,
        sha256: sha256.to_owned(),
    }))
}

fn design_artifacts(root: &Path, name: &str, value: &Value) -> Result<Vec<ArtifactRef>> {
    let mut refs = Vec::new();
    if name == "design/odiff_compare"
        && let Some(item) = artifact_from_json(root, &value["diff"])?
    {
        refs.push(item);
    }
    // Typst and HyperFrames expose inspectable QA images for their PDF/video.
    // Their primary binary outputs are checked for stability separately.
    if !matches!(name, "design/typst_compile" | "design/hyperframes_render")
        && let Some(item) = artifact_from_json(root, &value["artifact"])?
    {
        refs.push(item);
    }
    if matches!(name, "design/typst_compile" | "design/hyperframes_render") {
        let key = if name == "design/typst_compile" {
            "pages"
        } else {
            "snapshots"
        };
        if let Some(items) = value[key].as_array() {
            for item in items {
                if let Some(found) = artifact_from_json(
                    root,
                    if key == "snapshots" {
                        &item["artifact"]
                    } else {
                        item
                    },
                )? {
                    refs.push(found);
                }
            }
        }
    }
    if name == "design/vector_render"
        && let Some(item) = artifact_from_json(root, &value["preview"])?
    {
        refs.push(item);
    }
    if name == "design/tokens_compile"
        && let Some(outputs) = value["outputs"].as_object()
    {
        for relative in outputs.values().filter_map(Value::as_str) {
            let path = design_file(root, relative)?;
            let bytes = util::read_bounded(&path, 1024 * 1024)?;
            refs.push(ArtifactRef {
                path,
                sha256: util::sha256(&bytes),
            });
        }
    }
    Ok(refs)
}

#[derive(Clone, Debug, Serialize)]
pub struct Definition {
    pub policy: ToolPolicy,
    pub description: String,
    pub input_schema: Value,
}

fn require_available(raw: &Value, assigned: &[String]) -> Result<()> {
    let name = raw["name"].as_str().context("design tool lacks name")?;
    if assigned
        .iter()
        .any(|tool| tool == &format!("design/{name}"))
        && let Some(availability) = raw.pointer("/_meta/mirket~1availability")
    {
        ensure!(
            availability["enabled"].as_bool() == Some(true),
            "assigned tool design/{name} is unavailable (group {}); no model was dispatched. Inspect mirket package status, then explicitly stop/start the required group: {}",
            availability["group"].as_str().unwrap_or("unknown"),
            availability["start"]
                .as_str()
                .unwrap_or("inspect package configuration")
        );
    }
    Ok(())
}

fn design_check_passed(name: &str, result: &rmcp::model::CallToolResult) -> bool {
    result.is_error != Some(true)
        && (name != "design/odiff_compare"
            || result
                .structured_content
                .as_ref()
                .and_then(|value| value.get("match"))
                .and_then(Value::as_bool)
                == Some(true))
}

fn definition(
    name: &str,
    tier: Tier,
    vision: bool,
    description: &str,
    properties: Value,
    required: Value,
) -> Definition {
    Definition {
        policy: ToolPolicy {
            name: name.into(),
            min_tier: tier,
            requires_vision: vision,
        },
        description: description.into(),
        input_schema: json!({"type":"object","additionalProperties":false,"properties":properties,"required":required}),
    }
}
pub async fn definitions(paths: &Paths, names: &[String]) -> Result<Vec<Definition>> {
    ensure!(
        !names.iter().any(|name| matches!(
            name.as_str(),
            "design/execute_code"
                | "design/export_shape"
                | "design/import_image"
                | "design/penpot_api_info"
                | "design/high_level_overview"
        )),
        "Penpot object operations require the host's official MCP connection; project-scoped workers cannot authorize shared editor objects. Use host editing and verification."
    );

    let mut all = vec![
        definition(
            "workspace_read",
            Tier::T1Deterministic,
            false,
            "Read UTF-8 source in the assigned project or fixed design workspace (at most 128 KiB).",
            json!({"path":{"type":"string"}}),
            json!(["path"]),
        ),
        definition(
            "workspace_list",
            Tier::T1Deterministic,
            false,
            "List one project directory, excluding .git, with at most 1000 entries.",
            json!({"path":{"type":"string"}}),
            json!(["path"]),
        ),
        definition(
            "workspace_write",
            Tier::T2Synthesis,
            false,
            "Write UTF-8 source using a project-relative path or an absolute path inside the assigned design workspace. Supply exactly one of content (whole file) or edits (1–64 sequential exact replacements, each old text must match once). Existing files require the exact expected SHA-256. Read back the changed file before completion.",
            json!({"path":{"type":"string"},"content":{"type":"string"},"edits":{"type":"array","minItems":1,"maxItems":64,"items":{"type":"object","additionalProperties":false,"properties":{"old":{"type":"string","minLength":1},"new":{"type":"string"}},"required":["old","new"]}},"expected_sha256":{"type":["string","null"]}}),
            json!(["path", "expected_sha256"]),
        ),
        definition(
            "inspect_image",
            Tier::T3Architect,
            true,
            "Inspect PNG/JPEG/WebP pixels from the project or its assigned design workspace. Use purpose reference for a stable brand/reference image, comparison for a saved alternative, or artifact for current output. At most two references receive retention priority within four images/8 MiB. Retained pixels are context, not verification after a later mutation.",
            json!({"path":{"type":"string"},"purpose":{"type":"string","enum":["artifact","reference","comparison"],"default":"artifact"}}),
            json!(["path"]),
        ),
    ];
    let policies: BTreeMap<String, Value> =
        serde_json::from_str(include_str!("../../runtime/design/tools/tool-policy.json"))?;
    if names.iter().any(|n| n.starts_with("design/")) {
        let schemas = crate::package_mcp::probe_with_schemas(
            crate::packages::launch(paths, "design").await?,
            true,
        )
        .await?;
        for raw in schemas["schemas"]
            .as_array()
            .context("design package lacks schemas")?
        {
            require_available(raw, names)?;
            let name = raw["name"].as_str().context("design tool lacks name")?;
            let policy = policies
                .get(name)
                .context("design tool has no owned capability policy")?;
            all.push(Definition {
                policy: ToolPolicy {
                    name: format!("design/{name}"),
                    min_tier: serde_json::from_value(policy["min_tier"].clone())?,
                    requires_vision: policy["requires_vision"]
                        .as_bool()
                        .context("tool vision gate missing")?,
                },
                description: raw["description"].as_str().unwrap_or("").into(),
                input_schema: raw["inputSchema"].clone(),
            });
        }
    }
    names
        .iter()
        .map(|name| {
            all.iter()
                .find(|t| &t.policy.name == name)
                .cloned()
                .with_context(|| format!("unknown assigned tool: {name}"))
        })
        .collect()
}

pub fn file(project: &Path, relative: &str) -> Result<PathBuf> {
    util::safe_relative(Path::new(relative))?;
    ensure!(
        !repository_internal(Path::new(relative)),
        "repository internals are outside worker file tools"
    );
    let path = project.join(relative);
    util::reject_symlink_components(&path)?;
    Ok(path)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileArgs {
    path: String,
}
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ImagePurpose {
    #[default]
    Artifact,
    Reference,
    Comparison,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ImageArgs {
    path: String,
    #[serde(default)]
    purpose: ImagePurpose,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WriteArgs {
    path: String,
    content: Option<String>,
    edits: Option<Vec<TextEdit>>,
    expected_sha256: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TextEdit {
    old: String,
    new: String,
}

fn edited_content(original: &[u8], edits: &[TextEdit]) -> Result<String> {
    ensure!(
        (1..=64).contains(&edits.len()),
        "expected 1..=64 exact edits"
    );
    let mut result = std::str::from_utf8(original)
        .context("edited file is not UTF-8")?
        .to_owned();
    for edit in edits {
        ensure!(!edit.old.is_empty(), "edit old text must not be empty");
        let index = result
            .find(&edit.old)
            .context("edit old text did not match; no file was written")?;
        let next = index + edit.old.chars().next().expect("nonempty edit").len_utf8();
        ensure!(
            !result[next..].contains(&edit.old),
            "edit old text must match exactly once; no file was written"
        );
        ensure!(
            result.len() - edit.old.len() + edit.new.len() <= 1024 * 1024,
            "edited worker file exceeds 1 MiB"
        );
        result = result.replacen(&edit.old, &edit.new, 1);
    }
    Ok(result)
}

/// Bind exact before/after bytes before dispatch so interruption never needs a
/// blind write replay. This is evidence of the requested state, not authorship.
pub fn write_intent_scoped(paths: &Paths, project: &Path, args: &Value) -> Result<Value> {
    let value = args["path"].as_str().context("missing write path")?;
    let absolute = writable_file(paths, project, value)?;
    let root = if Path::new(value).is_absolute() {
        design_scope(paths, project)?
    } else {
        project.to_owned()
    };
    let mut relative = args.clone();
    relative["path"] = json!(absolute.strip_prefix(&root)?);
    let mut intent = write_intent(&root, &relative)?;
    intent["path"] = json!(value);
    Ok(intent)
}

pub fn reconcile_write_scoped(
    paths: &Paths,
    project: &Path,
    intent: &Value,
) -> Result<(bool, ArtifactRef)> {
    let value = intent["path"].as_str().context("missing intent path")?;
    let absolute = writable_file(paths, project, value)?;
    let root = if Path::new(value).is_absolute() {
        design_scope(paths, project)?
    } else {
        project.to_owned()
    };
    let mut relative = intent.clone();
    relative["path"] = json!(absolute.strip_prefix(&root)?);
    reconcile_write(&root, &relative)
}

pub fn write_intent(project: &Path, args: &Value) -> Result<Value> {
    let a: WriteArgs = serde_json::from_value(args.clone())?;
    ensure!(
        a.content.is_some() != a.edits.is_some(),
        "provide content or edits"
    );
    let path = file(project, &a.path)?;
    let old = if path.exists() {
        let bytes = util::read_bounded(&path, 1024 * 1024)?;
        ensure!(
            a.expected_sha256.as_deref() == Some(util::sha256(&bytes).as_str()),
            "existing file requires matching expected_sha256"
        );
        Some(bytes)
    } else {
        ensure!(
            a.expected_sha256.is_none() && a.edits.is_none(),
            "expected file does not exist"
        );
        None
    };
    let content = match (a.content, a.edits) {
        (Some(content), None) => content,
        (None, Some(edits)) => edited_content(old.as_deref().unwrap_or_default(), &edits)?,
        _ => unreachable!(),
    };
    ensure!(content.len() <= 1024 * 1024, "worker file exceeds 1 MiB");
    Ok(
        json!({"path":a.path,"before_sha256":old.as_deref().map(util::sha256),"after_sha256":util::sha256(content.as_bytes())}),
    )
}

pub fn reconcile_write(project: &Path, intent: &Value) -> Result<(bool, ArtifactRef)> {
    let path = file(
        project,
        intent["path"]
            .as_str()
            .context("missing write intent path")?,
    )?;
    let after = intent["after_sha256"]
        .as_str()
        .context("missing write intent digest")?;
    ensure!(
        after.len() == 64 && after.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid write intent digest"
    );
    let current = if path.exists() {
        Some(current_digest(&path)?)
    } else {
        None
    };
    let matches = current.as_deref() == Some(after);
    ensure!(
        matches || current.as_deref() == intent["before_sha256"].as_str(),
        "pending write differs from both intended and original bytes; external inspection required"
    );
    Ok((
        matches,
        ArtifactRef {
            path,
            sha256: after.into(),
        },
    ))
}

pub struct ToolResult {
    pub value: Value,
    pub image: Option<PathBuf>,
    pub mutates: bool,
    pub passed: bool,
    pub artifacts: Vec<ArtifactRef>,
    pub stable_artifacts: Vec<ArtifactRef>,
    pub verifies: Vec<ArtifactRef>,
    pub opaque_mutation: bool,
}

pub fn may_mutate(name: &str) -> bool {
    name == "workspace_write"
        || (name.starts_with("design/")
            && !matches!(
                name,
                "design/penpot_api_info" | "design/high_level_overview"
            ))
}
fn design_mutates(name: &str, result: &rmcp::model::CallToolResult) -> bool {
    if design_rejected_before_dispatch(name, result) {
        return false;
    }
    if name == "design/odiff_compare" {
        return result.is_error == Some(true)
            || result
                .structured_content
                .as_ref()
                .and_then(|value| value.get("diff"))
                .is_none_or(|diff| !diff.is_null());
    }
    may_mutate(name)
}

fn design_rejected_before_dispatch(name: &str, result: &rmcp::model::CallToolResult) -> bool {
    // Only owned direct tools issue this contract. Forwarded Penpot content or
    // an error message alone cannot attest that no mutation was dispatched.
    let Some(native) = name.strip_prefix("design/") else {
        return false;
    };
    if !matches!(
        native,
        "tokens_compile"
            | "static_render"
            | "browser_screenshot"
            | "kroki_render"
            | "hyperframes_render"
            | "typst_compile"
            | "raster_transform"
            | "vector_render"
            | "svg_optimize"
            | "odiff_compare"
    ) || result.is_error != Some(true)
    {
        return false;
    }
    result.structured_content.as_ref().is_some_and(|value| {
        let execution = &value["execution"];
        execution["tool"] == native
            && execution["state"] == "not_started"
            && matches!(
                execution["code"].as_str(),
                Some(
                    "invalid_arguments"
                        | "disabled_group"
                        | "render_admission"
                        | "output_preflight"
                )
            )
    })
}
pub async fn invoke(paths: &Paths, project: &Path, name: &str, args: Value) -> Result<ToolResult> {
    let mut image = None;
    let mut mutates = false;
    let mut passed = true;
    let mut artifacts = Vec::new();
    let mut stable_artifacts = Vec::new();
    let mut verifies = Vec::new();
    let mut opaque_mutation = false;
    let value = match name {
        "workspace_read" => {
            let a: FileArgs = serde_json::from_value(args)?;
            let path = readable_file(paths, project, &a.path)?;
            let bytes = util::read_bounded(&path, 128 * 1024)?;
            verifies.push(ArtifactRef {
                path,
                sha256: util::sha256(&bytes),
            });
            json!({"path":a.path,"content":String::from_utf8(bytes.clone()).context("file is not UTF-8")?,"sha256":util::sha256(&bytes)})
        }
        "workspace_list" => {
            let a: FileArgs = serde_json::from_value(args)?;
            let dir = if a.path == "." {
                project.to_owned()
            } else {
                file(project, &a.path)?
            };
            let mut entries = Vec::new();
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                if repository_internal(Path::new(&entry.file_name())) {
                    continue;
                }
                ensure!(entries.len() < 1000, "directory exceeds 1000 entries");
                entries.push(entry.file_name().to_string_lossy().into_owned());
            }
            entries.sort();
            json!({"entries":entries})
        }
        "workspace_write" => {
            let a: WriteArgs = serde_json::from_value(args)?;
            ensure!(
                a.content.is_some() != a.edits.is_some(),
                "provide exactly one of content or edits"
            );
            ensure!(
                a.content
                    .as_ref()
                    .is_none_or(|value| value.len() <= 1024 * 1024),
                "worker file exceeds 1 MiB"
            );
            let path = writable_file(paths, project, &a.path)?;
            let locks = paths.cache().join("worker-writes");
            util::ensure_directory(&locks)?;
            let _lock = util::FileLock::acquire(
                &locks.join(util::sha256(path.to_string_lossy().as_bytes())),
            )?;
            let old = if path.exists() {
                let old = util::read_bounded(&path, 1024 * 1024)?;
                ensure!(
                    a.expected_sha256.as_deref() == Some(util::sha256(&old).as_str()),
                    "existing file requires matching expected_sha256"
                );
                old
            } else {
                ensure!(a.expected_sha256.is_none(), "expected file does not exist");
                ensure!(a.edits.is_none(), "exact edits require an existing file");
                Vec::new()
            };
            let content = match (a.content, a.edits) {
                (Some(content), None) => content,
                (None, Some(edits)) => edited_content(&old, &edits)?,
                _ => unreachable!("validated exclusive write mode"),
            };
            util::atomic_write(&path, content.as_bytes())?;
            mutates = true;
            artifacts.push(ArtifactRef {
                path,
                sha256: util::sha256(content.as_bytes()),
            });
            json!({"path":a.path,"sha256":util::sha256(content.as_bytes())})
        }
        "inspect_image" => {
            let a: ImageArgs = serde_json::from_value(args)?;
            let path = readable_file(paths, project, &a.path)?;
            let bytes = util::read_bounded(&path, 4 * 1024 * 1024)?;
            let png = bytes.starts_with(b"\x89PNG\r\n\x1a\n");
            let jpeg = bytes.starts_with(&[255, 216, 255]);
            let webp = bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP");
            ensure!(
                png || jpeg || webp,
                "inspection supports PNG/JPEG/WebP pixels"
            );
            image = Some(path);
            verifies.push(ArtifactRef {
                path: image.as_ref().unwrap().clone(),
                sha256: util::sha256(&bytes),
            });
            json!({"image_attached":true,"path":a.path,"sha256":util::sha256(&bytes),"purpose":a.purpose})
        }
        _ if name.starts_with("design/") => {
            let result = crate::package_mcp::call(
                crate::packages::launch_scoped(paths, &design_scope(paths, project)?).await?,
                name[7..].into(),
                args.as_object()
                    .cloned()
                    .context("tool arguments must be an object")?,
            )
            .await?;
            passed = design_check_passed(name, &result);
            mutates = design_mutates(name, &result);
            if mutates
                && result.is_error != Some(true)
                && (passed || name == "design/odiff_compare")
            {
                if name == "design/export_shape" {
                    let requested = args
                        .get("filePath")
                        .and_then(Value::as_str)
                        .context("Penpot export omitted filePath")?;
                    let relative = Path::new(requested)
                        .strip_prefix("/workspace")
                        .context("Penpot export path is outside design workspace")?;
                    let path = design_file(
                        &design_scope(paths, project)?,
                        relative
                            .to_str()
                            .context("Penpot export path is not UTF-8")?,
                    )?;
                    let bytes = util::read_bounded(&path, 32 * 1024 * 1024)?;
                    artifacts.push(ArtifactRef {
                        path,
                        sha256: util::sha256(&bytes),
                    });
                } else if let Some(structured) = result.structured_content.as_ref() {
                    artifacts = design_artifacts(&design_scope(paths, project)?, name, structured)?;
                    if matches!(name, "design/typst_compile" | "design/hyperframes_render")
                        && let Some(primary) = artifact_from_json(
                            &design_scope(paths, project)?,
                            &structured["artifact"],
                        )?
                    {
                        stable_artifacts.push(primary);
                    }
                }
                opaque_mutation = artifacts.is_empty();
            } else if mutates {
                opaque_mutation = true;
            }
            if name == "design/odiff_compare"
                && passed
                && let Some(candidate) = result
                    .structured_content
                    .as_ref()
                    .and_then(|v| v.get("candidate"))
                    .and_then(Value::as_str)
            {
                let path = design_file(&design_scope(paths, project)?, candidate)?;
                let bytes = util::read_bounded(&path, 32 * 1024 * 1024)?;
                verifies.push(ArtifactRef {
                    path,
                    sha256: util::sha256(&bytes),
                });
            }
            if name == "design/browser_screenshot"
                && passed
                && let Some(source) = args.get("source").and_then(Value::as_str)
            {
                let path = design_file(&design_scope(paths, project)?, source)?;
                let bytes = util::read_bounded(&path, 32 * 1024 * 1024)?;
                verifies.push(ArtifactRef {
                    path,
                    sha256: util::sha256(&bytes),
                });
            }
            serde_json::to_value(result)?
        }
        _ => bail!("tool is not registered"),
    };
    Ok(ToolResult {
        value,
        image,
        mutates,
        passed,
        artifacts,
        stable_artifacts,
        verifies,
        opaque_mutation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupted_write_reconciles_bytes_without_replay() {
        let project = tempfile::tempdir().unwrap();
        let root = project.path().canonicalize().unwrap();
        let args = json!({"path":"result.txt","content":"new bytes"});
        let intent = write_intent(&root, &args).unwrap();
        assert!(!reconcile_write(&root, &intent).unwrap().0);
        assert!(!root.join("result.txt").exists());
        std::fs::write(root.join("result.txt"), "new bytes").unwrap();
        assert!(reconcile_write(&root, &intent).unwrap().0);
        let edits = json!({"path":"result.txt","edits":[{"old":"new","new":"changed"}],"expected_sha256":util::sha256(b"new bytes")});
        let edit_intent = write_intent(&root, &edits).unwrap();
        assert!(!reconcile_write(&root, &edit_intent).unwrap().0);
        std::fs::write(root.join("result.txt"), "changed bytes").unwrap();
        assert!(reconcile_write(&root, &edit_intent).unwrap().0);
        std::fs::write(root.join("result.txt"), "unrelated user edit").unwrap();
        assert!(reconcile_write(&root, &edit_intent).is_err());
        let mut escaped = intent.clone();
        escaped["path"] = json!("../escape");
        assert!(reconcile_write(&root, &escaped).is_err());
        assert!(write_intent(&root, &edits).is_err());
    }

    #[test]
    fn disabled_assigned_design_tools_fail_preflight_without_blocking_other_work() {
        let raw = json!({"name":"kroki_render","_meta":{"mirket/availability":{"enabled":false,"group":"diagrams","start":"mirket package start design --group diagrams"}}});
        assert!(require_available(&raw, &["design/vector_render".into()]).is_ok());
        let error = require_available(&raw, &["design/kroki_render".into()])
            .unwrap_err()
            .to_string();
        assert!(error.contains("no model was dispatched") && error.contains("--group diagrams"));
        let mut ready = raw.clone();
        ready["_meta"]["mirket/availability"]["enabled"] = json!(true);
        assert!(require_available(&ready, &["design/kroki_render".into()]).is_ok());
    }

    #[test]
    fn only_bound_owned_predispatch_rejections_can_clear_mutation_uncertainty() {
        use rmcp::model::CallToolResult;
        let receipt = json!({"execution":{"tool":"vector_render","state":"not_started","code":"invalid_arguments"}});
        assert!(!design_mutates(
            "design/vector_render",
            &CallToolResult::structured_error(receipt.clone())
        ));
        let mut conflict = receipt.clone();
        conflict["execution"]["code"] = json!("output_preflight");
        assert!(!design_mutates(
            "design/vector_render",
            &CallToolResult::structured_error(conflict)
        ));
        assert!(design_mutates(
            "design/vector_render",
            &CallToolResult::structured(receipt.clone())
        ));
        assert!(design_mutates(
            "design/browser_screenshot",
            &CallToolResult::structured_error(receipt.clone())
        ));
        let mut remote = receipt.clone();
        remote["execution"]["tool"] = json!("execute_code");
        assert!(design_mutates(
            "design/execute_code",
            &CallToolResult::structured_error(remote)
        ));
        for (key, value) in [("state", "started"), ("code", "renderer_failure")] {
            let mut altered = receipt.clone();
            altered["execution"][key] = json!(value);
            assert!(design_mutates(
                "design/vector_render",
                &CallToolResult::structured_error(altered)
            ));
        }
        assert!(design_mutates(
            "design/vector_render",
            &CallToolResult::structured_error(json!({"message":"operation did not start"}))
        ));
    }
    #[tokio::test]
    async fn exact_edits_are_digest_bound_and_transactional() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().into()).unwrap();
        let project = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let original = "first α line\nsecond line\n";
        let file = project.path().join("source");
        std::fs::write(&file, original).unwrap();
        for args in [
            json!({"path":"source","edits":[{"old":"first","new":"changed"}],"expected_sha256":null}),
            json!({"path":"source","edits":[{"old":"first","new":"changed"},{"old":"absent","new":"bad"}],"expected_sha256":util::sha256(original.as_bytes())}),
            json!({"path":"source","edits":[{"old":"line","new":"ambiguous"}],"expected_sha256":util::sha256(original.as_bytes())}),
            json!({"path":"source","content":"wrong","edits":[{"old":"first","new":"changed"}],"expected_sha256":util::sha256(original.as_bytes())}),
        ] {
            assert!(
                invoke(&paths, project.path(), "workspace_write", args)
                    .await
                    .is_err()
            );
            assert_eq!(std::fs::read_to_string(&file).unwrap(), original);
        }
        let result = invoke(&paths, project.path(), "workspace_write", json!({"path":"source","edits":[{"old":"α","new":"β"},{"old":"second","new":"last"}],"expected_sha256":util::sha256(original.as_bytes())})).await.unwrap();
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            "first β line\nlast line\n"
        );
        assert!(result.mutates && result.artifacts.len() == 1);
        assert!(
            edited_content(
                b"aaa",
                &[TextEdit {
                    old: "aa".into(),
                    new: "b".into()
                }]
            )
            .is_err()
        );
        assert!(
            edited_content(
                b"small",
                &[TextEdit {
                    old: "small".into(),
                    new: "x".repeat(1024 * 1024 + 1)
                }]
            )
            .is_err()
        );
    }
    #[test]
    fn odiff_mismatch_is_a_failed_gate_even_when_transport_succeeded() {
        use rmcp::model::CallToolResult;
        assert!(!design_check_passed(
            "design/odiff_compare",
            &CallToolResult::structured(json!({"match": false, "exit_code": 21}))
        ));
        assert!(!design_check_passed(
            "design/odiff_compare",
            &CallToolResult::structured(json!({"exit_code": 0}))
        ));
        assert!(design_check_passed(
            "design/odiff_compare",
            &CallToolResult::structured(json!({"match": true, "exit_code": 0}))
        ));
        assert!(!design_check_passed(
            "design/odiff_compare",
            &CallToolResult::structured_error(json!({"match": true}))
        ));
    }

    #[test]
    fn odiff_published_diagnostics_are_mutations_with_artifact_receipts() {
        use rmcp::model::CallToolResult;
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().into()).unwrap();
        std::fs::create_dir_all(design_root(&paths)).unwrap();
        let value = json!({"match":false,"diff":{"path":"diff.png","sha256":"a".repeat(64)}});
        assert!(may_mutate("design/odiff_compare"));
        assert!(design_mutates(
            "design/odiff_compare",
            &CallToolResult::structured(value.clone())
        ));
        let artifacts =
            design_artifacts(&design_root(&paths), "design/odiff_compare", &value).unwrap();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].path, design_root(&paths).join("diff.png"));
        assert!(!design_mutates(
            "design/odiff_compare",
            &CallToolResult::structured(json!({"match":true,"diff":null}))
        ));
        assert!(design_mutates(
            "design/odiff_compare",
            &CallToolResult::structured_error(json!({"match":true,"diff":null}))
        ));
    }

    #[test]
    fn repository_internals_reject_case_aliases_on_every_path_surface() {
        let root = tempfile::tempdir().unwrap();
        let project = root.path().canonicalize().unwrap();
        let paths = crate::paths::Paths::new(project.clone()).unwrap();
        for relative in [".git/config", ".GIT/config", ".GiT/config"] {
            assert!(
                file(&project, relative)
                    .unwrap_err()
                    .to_string()
                    .contains("repository internals")
            );
            assert!(
                readable_file(&paths, &project, project.join(relative).to_str().unwrap())
                    .unwrap_err()
                    .to_string()
                    .contains("repository internals")
            );
        }
    }

    #[test]
    fn paths_cannot_escape_project() {
        let root = tempfile::tempdir().unwrap();
        let paths = Paths::new(root.path().into()).unwrap();
        for p in ["../private", "/tmp/file", ".git/config"] {
            assert!(file(root.path(), p).is_err());
        }
        std::fs::create_dir_all(root.path().join(".git")).unwrap();
        std::fs::write(root.path().join(".git/config"), "private").unwrap();
        assert!(
            readable_file(
                &paths,
                root.path(),
                root.path().join(".git/config").to_str().unwrap()
            )
            .is_err()
        );
    }
    #[tokio::test]
    async fn writes_require_observed_identity() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().into()).unwrap();
        let project = tempfile::tempdir().unwrap();
        std::fs::write(project.path().join("a"), "original").unwrap();
        assert!(
            invoke(
                &paths,
                project.path(),
                "workspace_write",
                json!({"path":"a","content":"changed","expected_sha256":null})
            )
            .await
            .is_err()
        );
        assert_eq!(
            std::fs::read_to_string(project.path().join("a")).unwrap(),
            "original"
        );
    }

    #[test]
    fn design_artifact_refs_stay_in_assigned_workspace() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().into()).unwrap();
        let project = home.path().canonicalize().unwrap().join("project");
        std::fs::create_dir(&project).unwrap();
        let root = design_scope(&paths, &project).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("render.png"), b"image").unwrap();
        let digest = util::sha256(b"image");
        let refs = design_artifacts(
            &root,
            "design/static_render",
            &json!({"artifact":{"path":"render.png","sha256":digest}}),
        )
        .unwrap();
        assert_eq!(
            refs,
            vec![ArtifactRef {
                path: root.join("render.png"),
                sha256: util::sha256(b"image")
            }]
        );
        assert!(
            design_artifacts(
                &root,
                "design/static_render",
                &json!({"artifact":{"path":"../outside.png","sha256":util::sha256(b"image")}})
            )
            .is_err()
        );
        assert_eq!(
            readable_file(&paths, &project, root.join("render.png").to_str().unwrap()).unwrap(),
            root.join("render.png")
        );
    }

    #[test]
    fn primary_artifact_digest_rejects_changed_or_deleted_output() {
        let root = tempfile::tempdir().unwrap();
        let pdf = root.path().canonicalize().unwrap().join("document.pdf");
        std::fs::write(&pdf, b"%PDF-original").unwrap();
        let expected = current_digest(&pdf).unwrap();
        assert_eq!(expected, util::sha256(b"%PDF-original"));
        std::fs::write(&pdf, b"%PDF-changed").unwrap();
        assert_ne!(current_digest(&pdf).unwrap(), expected);
        std::fs::remove_file(&pdf).unwrap();
        assert!(current_digest(&pdf).is_err());
    }
}

#[cfg(test)]
mod scoped_design_tests {
    use super::*;
    #[tokio::test]
    async fn project_design_reads_writes_and_recovery_reject_siblings() {
        let home = tempfile::tempdir().unwrap();
        let root = home.path().canonicalize().unwrap();
        let paths = Paths::new(root.clone()).unwrap();
        let a = root.join("a");
        let b = root.join("b");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        let own = design_scope(&paths, &a).unwrap();
        let other = design_scope(&paths, &b).unwrap();
        std::fs::create_dir_all(&own).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        std::fs::write(other.join("private.txt"), "other project").unwrap();
        assert!(readable_file(&paths, &a, other.join("private.txt").to_str().unwrap()).is_err());
        let args = json!({"path":own.join("source.txt"),"content":"owned"});
        let intent = write_intent_scoped(&paths, &a, &args).unwrap();
        invoke(&paths, &a, "workspace_write", args).await.unwrap();
        assert!(reconcile_write_scoped(&paths, &a, &intent).unwrap().0);
        assert!(reconcile_write_scoped(&paths, &b, &intent).is_err());
        assert!(invoke(&paths,&b,"workspace_write",json!({"path":own.join("source.txt"),"content":"bad","expected_sha256":util::sha256(b"owned")})).await.is_err());
    }
}
