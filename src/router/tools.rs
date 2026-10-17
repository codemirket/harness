//! Owned tool contracts. A request can select a tool, never lower its policy.
use super::policy::{Tier, ToolPolicy};
use crate::{paths::Paths, util};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Eq, PartialEq)]
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

fn design_file(paths: &Paths, relative: &str) -> Result<PathBuf> {
    util::safe_relative(Path::new(relative))?;
    let path = design_root(paths).join(relative);
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
        canonical.starts_with(project) || canonical.starts_with(design_root(paths)),
        "file is outside assigned workspaces"
    );
    Ok(canonical)
}

fn artifact_from_json(paths: &Paths, value: &Value) -> Result<Option<ArtifactRef>> {
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
        path: design_file(paths, path)?,
        sha256: sha256.to_owned(),
    }))
}

fn design_artifacts(paths: &Paths, name: &str, value: &Value) -> Result<Vec<ArtifactRef>> {
    let mut refs = Vec::new();
    // Typst and HyperFrames expose inspectable QA images for their PDF/video.
    // Their primary binary outputs are checked for stability separately.
    if !matches!(name, "design/typst_compile" | "design/hyperframes_render")
        && let Some(item) = artifact_from_json(paths, &value["artifact"])?
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
                    paths,
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
    if name == "design/tokens_compile"
        && let Some(outputs) = value["outputs"].as_object()
    {
        for relative in outputs.values().filter_map(Value::as_str) {
            let path = design_file(paths, relative)?;
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
            "Write UTF-8 source in the assigned project. Existing files require the exact expected SHA-256.",
            json!({"path":{"type":"string"},"content":{"type":"string"},"expected_sha256":{"type":["string","null"]}}),
            json!(["path", "content", "expected_sha256"]),
        ),
        definition(
            "inspect_image",
            Tier::T3Architect,
            true,
            "Inspect an existing PNG/JPEG/WebP image from the project or fixed design workspace. Image is attached to the next inference step; inspect it before completion.",
            json!({"path":{"type":"string"}}),
            json!(["path"]),
        ),
    ];
    let policies: BTreeMap<String, Value> =
        serde_json::from_str(include_str!("../../runtime/design/tools/tool-policy.json"))?;
    if names.iter().any(|n| n.starts_with("design/")) {
        let schemas =
            crate::package_mcp::probe_with_schemas(crate::packages::launch(paths, "design")?, true)
                .await?;
        for raw in schemas["schemas"]
            .as_array()
            .context("design package lacks schemas")?
        {
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
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WriteArgs {
    path: String,
    content: String,
    expected_sha256: Option<String>,
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
                "design/penpot_api_info" | "design/high_level_overview" | "design/odiff_compare"
            ))
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
            ensure!(a.content.len() <= 1024 * 1024, "worker file exceeds 1 MiB");
            let path = file(project, &a.path)?;
            let locks = paths.cache().join("worker-writes");
            util::ensure_directory(&locks)?;
            let _lock = util::FileLock::acquire(
                &locks.join(util::sha256(path.to_string_lossy().as_bytes())),
            )?;
            if path.exists() {
                let old = util::read_bounded(&path, 1024 * 1024)?;
                ensure!(
                    a.expected_sha256.as_deref() == Some(util::sha256(&old).as_str()),
                    "existing file requires matching expected_sha256"
                );
            } else {
                ensure!(a.expected_sha256.is_none(), "expected file does not exist");
            }
            util::atomic_write(&path, a.content.as_bytes())?;
            mutates = true;
            artifacts.push(ArtifactRef {
                path,
                sha256: util::sha256(a.content.as_bytes()),
            });
            json!({"path":a.path,"sha256":util::sha256(a.content.as_bytes())})
        }
        "inspect_image" => {
            let a: FileArgs = serde_json::from_value(args)?;
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
            json!({"image_attached":true,"path":a.path,"sha256":util::sha256(&bytes)})
        }
        _ if name.starts_with("design/") => {
            let result = crate::package_mcp::call(
                crate::packages::launch(paths, "design")?,
                name[7..].into(),
                args.as_object()
                    .cloned()
                    .context("tool arguments must be an object")?,
            )
            .await?;
            passed = design_check_passed(name, &result);
            mutates = may_mutate(name);
            if mutates && passed {
                if name == "design/export_shape" {
                    let requested = args
                        .get("filePath")
                        .and_then(Value::as_str)
                        .context("Penpot export omitted filePath")?;
                    let relative = Path::new(requested)
                        .strip_prefix("/workspace")
                        .context("Penpot export path is outside design workspace")?;
                    let path = design_file(
                        paths,
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
                    artifacts = design_artifacts(paths, name, structured)?;
                    if matches!(name, "design/typst_compile" | "design/hyperframes_render")
                        && let Some(primary) = artifact_from_json(paths, &structured["artifact"])?
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
                let path = design_file(paths, candidate)?;
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
                let path = design_file(paths, source)?;
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
    fn design_artifact_refs_stay_in_fixed_workspace() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().into()).unwrap();
        let root = design_root(&paths);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("render.png"), b"image").unwrap();
        let digest = util::sha256(b"image");
        let refs = design_artifacts(
            &paths,
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
                &paths,
                "design/static_render",
                &json!({"artifact":{"path":"../outside.png","sha256":util::sha256(b"image")}})
            )
            .is_err()
        );
        assert_eq!(
            readable_file(
                &paths,
                home.path(),
                root.join("render.png").to_str().unwrap()
            )
            .unwrap(),
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
