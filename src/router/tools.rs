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
        "workspace_search",
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
    // Typst exposes inspectable QA images for its PDF.
    // Their primary binary outputs are checked for stability separately.
    if !matches!(name, "design/typst_compile")
        && let Some(item) = artifact_from_json(root, &value["artifact"])?
    {
        refs.push(item);
    }
    if name == "design/typst_compile"
        && let Some(items) = value["pages"].as_array()
    {
        for item in items {
            if let Some(found) = artifact_from_json(root, item)? {
                refs.push(found);
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

fn inspection_artifacts(
    root: &Path,
    value: &Value,
    artifacts: &mut Vec<ArtifactRef>,
) -> Result<Vec<ArtifactRef>> {
    let mut originals = Vec::new();
    let Some(plans) = value.get("inspection") else {
        return Ok(originals);
    };
    let plans = plans.as_array().context("invalid inspection plans")?;
    ensure!(plans.len() <= 128, "too many inspection plans");
    for plan in plans {
        let source =
            artifact_from_json(root, &plan["source"])?.context("inspection source missing")?;
        let index = artifacts
            .iter()
            .position(|a| a == &source)
            .context("inspection source is not a produced artifact")?;
        ensure!(
            current_digest(&source.path)? == source.sha256,
            "inspection source changed"
        );
        let width = plan["width"].as_u64().context("inspection width missing")?;
        let height = plan["height"]
            .as_u64()
            .context("inspection height missing")?;
        ensure!(
            width > 0 && height > 0 && width.checked_mul(height).is_some_and(|n| n <= 17_000_000),
            "inspection pixel budget exceeded"
        );
        let tiles = plan["tiles"]
            .as_array()
            .context("inspection tiles missing")?;
        ensure!(
            !tiles.is_empty() && tiles.len() <= 64,
            "invalid inspection tile count"
        );
        let mut rectangles = Vec::new();
        let mut area = 0;
        for tile in tiles {
            let x = tile["left"].as_u64().context("tile left missing")?;
            let y = tile["top"].as_u64().context("tile top missing")?;
            let w = tile["width"].as_u64().context("tile width missing")?;
            let h = tile["height"].as_u64().context("tile height missing")?;
            ensure!(
                w > 0
                    && h > 0
                    && x.checked_add(w).is_some_and(|r| r <= width)
                    && y.checked_add(h).is_some_and(|b| b <= height),
                "tile outside source"
            );
            for &(rx, ry, rw, rh) in &rectangles {
                ensure!(
                    x >= rx + rw || rx >= x + w || y >= ry + rh || ry >= y + h,
                    "overlapping inspection tiles"
                );
            }
            rectangles.push((x, y, w, h));
            area += w * h;
        }
        ensure!(
            area == width * height,
            "inspection tiles omit source pixels"
        );
        artifacts.remove(index);
        for item in std::iter::once(&plan["overview"]).chain(tiles) {
            let receipt =
                artifact_from_json(root, item)?.context("inspection image receipt missing")?;
            let bytes = util::read_bounded(&receipt.path, 4 * 1024 * 1024)?;
            ensure!(
                bytes.starts_with(b"\x89PNG\r\n\x1a\n") && util::sha256(&bytes) == receipt.sha256,
                "inspection image changed or invalid"
            );
            artifacts.push(receipt);
        }
        originals.push(source);
    }
    Ok(originals)
}

// An editable binary is verified through a consuming edit/export, then its
// exported pixels still require inspection. Unrelated exports cannot clear it.
fn openpencil_source_receipt(root: &Path, value: &Value) -> Result<Option<ArtifactRef>> {
    let source = value["source"]
        .as_str()
        .context("OpenPencil omitted source identity")?;
    if value["artifact"]["path"] == source {
        return Ok(None);
    } // in-place save replaced the old revision
    let receipt = artifact_from_json(
        root,
        &json!({"path":source,"sha256":value["source_sha256"]}),
    )?
    .context("OpenPencil omitted source digest")?;
    ensure!(
        current_digest(&receipt.path)? == receipt.sha256,
        "OpenPencil source changed after consumption"
    );
    Ok(Some(receipt))
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
    if result.is_error == Some(true) {
        return false;
    }
    if name != "design/odiff_compare" {
        return true;
    }
    let Some(value) = result.structured_content.as_ref() else {
        return false;
    };
    match (value["match"].as_bool(), value["expect_match"].as_bool()) {
        (Some(equal), Some(expected)) => !expected || equal,
        _ => false,
    }
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
    let mut all = vec![
        definition(
            "workspace_read",
            Tier::T1Deterministic,
            false,
            "Read UTF-8 source up to 16 MiB in pages of at most 128 KiB. Continue at next_offset with expected_sha256. Partial pages never verify a whole artifact.",
            json!({"path":{"type":"string"},"offset":{"type":"integer","minimum":0},"limit":{"type":"integer","minimum":4,"maximum":131072},"expected_sha256":{"type":"string"}}),
            json!(["path"]),
        ),
        definition(
            "workspace_list",
            Tier::T1Deterministic,
            false,
            "List one project directory excluding .git in pages of at most 1000 entries. Continue at next_offset with expected_sha256; changed listings reject continuation.",
            json!({"path":{"type":"string"},"offset":{"type":"integer","minimum":0},"limit":{"type":"integer","minimum":1},"expected_sha256":{"type":"string"}}),
            json!(["path"]),
        ),
        definition(
            "workspace_search",
            Tier::T1Deterministic,
            false,
            "Bounded literal (default) or regex UTF-8 search with source hashes and line numbers. Applies project-local .gitignore rules; excludes common generated directories and symlinks. Does not read global Git excludes. Discovery does not verify whole artifacts.",
            json!({"query":{"type":"string","minLength":1,"maxLength":4096},"mode":{"type":"string","enum":["literal","regex"],"default":"literal"},"path":{"type":"string"},"limit":{"type":"integer","minimum":1,"maximum":200}}),
            json!(["query"]),
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
struct PageArgs {
    path: String,
    #[serde(default)]
    offset: usize,
    limit: Option<usize>,
    expected_sha256: Option<String>,
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
    /// Consumed inputs remain stable without replacing pending output verification.
    pub consumed_artifacts: Vec<ArtifactRef>,
    pub verifies: Vec<ArtifactRef>,
    pub opaque_mutation: bool,
}

pub fn may_mutate(name: &str) -> bool {
    name == "workspace_write"
        || (name.starts_with("design/") && !matches!(name, "design/openpencil_inspect"))
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

fn design_completed(name: &str, result: &rmcp::model::CallToolResult) -> bool {
    let Some(native) = name.strip_prefix("design/") else {
        return false;
    };
    known_names().is_ok_and(|names| names.contains(name))
        && result.structured_content.as_ref().is_some_and(|value| {
            value["execution"]["tool"] == native && value["execution"]["state"] == "completed"
        })
}

fn design_rejected_before_dispatch(name: &str, result: &rmcp::model::CallToolResult) -> bool {
    // Only owned direct tools issue this contract. Upstream content or
    // an error message alone cannot attest that no mutation was dispatched.
    let Some(native) = name.strip_prefix("design/") else {
        return false;
    };
    if !known_names().is_ok_and(|names| names.contains(name)) || result.is_error != Some(true) {
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
    ensure!(known_names()?.contains(name), "unknown owned tool: {name}");
    let mut image = None;
    let mut mutates = false;
    let mut passed = true;
    let mut artifacts = Vec::new();
    let mut stable_artifacts = Vec::new();
    let mut consumed_artifacts = Vec::new();
    let mut verifies = Vec::new();
    let mut opaque_mutation = false;
    let value = match name {
        "workspace_search" => super::search::search(project, args)?,
        "workspace_read" => {
            let a: PageArgs = serde_json::from_value(args)?;
            let path = readable_file(paths, project, &a.path)?;
            let bytes = util::read_bounded(&path, 16 * 1024 * 1024)?;
            let sha = util::sha256(&bytes);
            let text = String::from_utf8(bytes).context("file is not UTF-8")?;
            let limit = a.limit.unwrap_or(128 * 1024);
            ensure!(
                (4..=128 * 1024).contains(&limit),
                "read limit must be 4..131072 bytes"
            );
            ensure!(
                a.offset <= text.len() && text.is_char_boundary(a.offset),
                "offset must be a UTF-8 boundary within the file"
            );
            ensure!(
                a.offset == 0 || a.expected_sha256.is_some(),
                "continuation requires expected_sha256"
            );
            ensure!(
                a.expected_sha256
                    .as_ref()
                    .is_none_or(|expected| expected == &sha),
                "file changed since previous page"
            );
            let mut end = a.offset.saturating_add(limit).min(text.len());
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            let complete = a.offset == 0 && end == text.len();
            if complete {
                verifies.push(ArtifactRef {
                    path: path.clone(),
                    sha256: sha.clone(),
                });
            }
            json!({"path":a.path,"content":&text[a.offset..end],"sha256":sha,"offset":a.offset,"total_bytes":text.len(),"next_offset":(end < text.len()).then_some(end),"complete":complete,"read_receipt":{"path":path,"sha256":sha,"offset":a.offset,"end":end,"total":text.len()}})
        }
        "workspace_list" => {
            let a: PageArgs = serde_json::from_value(args)?;
            let dir = if a.path == "." {
                project.to_owned()
            } else {
                file(project, &a.path)?
            };
            let limit = a.limit.unwrap_or(1000);
            ensure!(
                (1..=1000).contains(&limit),
                "list limit must be 1..1000 entries"
            );
            let mut entries = Vec::new();
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                if repository_internal(Path::new(&entry.file_name())) {
                    continue;
                }
                ensure!(
                    entries.len() < 100_000,
                    "directory exceeds bounded 100000-entry scan"
                );
                entries.push(
                    entry
                        .file_name()
                        .into_string()
                        .map_err(|_| anyhow::anyhow!("directory entry is not UTF-8"))?,
                );
            }
            entries.sort();
            let sha = util::sha256(&serde_json::to_vec(&entries)?);
            ensure!(a.offset <= entries.len(), "offset exceeds listing");
            ensure!(
                a.offset == 0 || a.expected_sha256.is_some(),
                "continuation requires expected_sha256"
            );
            ensure!(
                a.expected_sha256
                    .as_ref()
                    .is_none_or(|expected| expected == &sha),
                "directory listing changed since previous page"
            );
            let end = a.offset.saturating_add(limit).min(entries.len());
            json!({"entries":&entries[a.offset..end],"sha256":sha,"offset":a.offset,"total_entries":entries.len(),"next_offset":(end < entries.len()).then_some(end)})
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
            // A completed render with negative QA is repairable, not an
            // uncertain mutation. It stays failed and requires fresh proof.
            if mutates
                && ((result.is_error != Some(true) && (passed || name == "design/odiff_compare"))
                    || design_completed(name, &result))
            {
                if let Some(structured) = result.structured_content.as_ref() {
                    artifacts = design_artifacts(&design_scope(paths, project)?, name, structured)?;
                    stable_artifacts.extend(inspection_artifacts(
                        &design_scope(paths, project)?,
                        structured,
                        &mut artifacts,
                    )?);
                    if name == "design/odiff_compare" {
                        // Diagnostics need identity/stability, not a visual acceptance gate.
                        stable_artifacts.append(&mut artifacts);
                    }
                    if matches!(name, "design/typst_compile")
                        && let Some(primary) = artifact_from_json(
                            &design_scope(paths, project)?,
                            &structured["artifact"],
                        )?
                    {
                        stable_artifacts.push(primary);
                    }
                }
                opaque_mutation = artifacts.is_empty() && stable_artifacts.is_empty();
            } else if mutates {
                opaque_mutation = true;
            }
            if matches!(
                name,
                "design/openpencil_edit" | "design/openpencil_export" | "design/openpencil_inspect"
            ) && passed
                && let Some(structured) = result.structured_content.as_ref()
                && (name != "design/openpencil_inspect" || structured.get("source").is_some())
                && let Some(receipt) =
                    openpencil_source_receipt(&design_scope(paths, project)?, structured)?
            {
                verifies.push(receipt);
            }
            if name == "design/odiff_compare" && passed {
                let structured = result
                    .structured_content
                    .as_ref()
                    .context("comparison receipt missing")?;
                ensure!(
                    structured["expect_match"].as_bool()
                        == Some(args["expect_match"].as_bool().unwrap_or(false)),
                    "comparison expectation changed"
                );
                for key in ["baseline", "candidate"] {
                    let receipt = artifact_from_json(&design_scope(paths, project)?, &json!({"path":structured[key],"sha256":structured[format!("{key}_sha256")]}))?.context("comparison omitted input identity")?;
                    ensure!(
                        current_digest(&receipt.path)? == receipt.sha256,
                        "comparison input changed after consumption"
                    );
                    if key == "candidate" && structured["expect_match"] == true {
                        verifies.push(receipt.clone());
                    }
                    consumed_artifacts.push(receipt);
                }
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
        consumed_artifacts,
        verifies,
        opaque_mutation,
    })
}

#[cfg(test)]
include!("tests/tools.rs");
