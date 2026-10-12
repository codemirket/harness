//! Project-scoped compositions executed by the pinned HyperFrames producer.
use super::{EditorResult, files, process};
use crate::{package_mcp::LaunchSpec, util};
use anyhow::{Context, Result, bail, ensure};
use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

const MAX_MANIFEST: u64 = 16 * 1024;
const MAX_ASSET: u64 = 8 * 1024 * 1024;
const MAX_INPUT: u64 = 32 * 1024 * 1024;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Inspect {
    /// Project-relative composition JSON manifest. Its entry and assets are relative to its directory.
    pub source: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub source: String,
    /// Zero-based frame, strictly below the manifest frame count.
    pub frame: u32,
    /// Distinct project-relative PNG; parent must exist.
    pub output: String,
    #[serde(default)]
    pub overwrite: bool,
    pub expected_sha256: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Render {
    pub source: String,
    /// Distinct project-relative MP4 (H.264/AAC) or WebM (VP9/Opus with alpha); parent must exist.
    pub output: String,
    #[serde(default)]
    pub overwrite: bool,
    pub expected_sha256: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u32,
    entry: String,
    #[serde(default)]
    assets: Vec<String>,
    width: u32,
    height: u32,
    fps: u32,
    frames: u32,
}

fn media_type(path: &str) -> Result<&'static str> {
    Ok(match Path::new(path).extension().and_then(|s| s.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js" | "mjs") => "text/javascript",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("json") => "application/json",
        Some("wav") => "audio/wav",
        Some("mp3") => "audio/mpeg",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        _ => bail!(
            "composition assets must be HTML, CSS, JS, JSON, SVG, PNG, JPEG, WebP, WOFF/WOFF2, WAV, MP3, MP4 or WebM"
        ),
    })
}

fn static_svg_asset(bytes: &[u8]) -> Result<()> {
    let text = std::str::from_utf8(bytes).context("SVG assets must be UTF-8")?;
    let doc = roxmltree::Document::parse_with_options(
        text,
        roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: 16_384,
            ..Default::default()
        },
    )
    .context("malformed or oversized SVG asset")?;
    ensure!(
        doc.root_element().tag_name().name() == "svg"
            && doc.root_element().tag_name().namespace() == Some("http://www.w3.org/2000/svg"),
        "SVG asset must have an SVG-namespace root"
    );
    for node in doc.descendants().filter(|node| node.is_element()) {
        ensure!(
            ![
                "animate",
                "animateTransform",
                "animateMotion",
                "set",
                "script",
                "style",
                "foreignObject"
            ]
            .contains(&node.tag_name().name()),
            "mapped SVG images must be static: no SMIL, scripts, style sheets or foreignObject; inline editable SVG in the HTML for synchronized motion"
        );
        ensure!(
            node.attributes()
                .all(|attribute| !attribute.name().to_ascii_lowercase().starts_with("on")),
            "mapped SVG images cannot contain event handlers"
        );
    }
    Ok(())
}

struct Prepared {
    manifest: Manifest,
    job: Value,
    identity: String,
    inputs: BTreeSet<String>,
}

fn validate_frame(bytes: &[u8], metadata: &Value, dimensions: [u32; 2], frame: u32) -> Result<()> {
    let (width, height) = process::png(bytes, Some(dimensions[0]))?;
    ensure!(
        [width, height] == dimensions
            && metadata["frame"].as_u64() == Some(u64::from(frame))
            && metadata["output_bytes"].as_u64() == Some(bytes.len() as u64)
            && metadata["sha256"] == util::sha256(bytes),
        "saved frame identity does not match the requested composition frame"
    );
    Ok(())
}

fn validate_saved_frames(metadata: &Value, frames: u32) -> Result<()> {
    let source_unique = metadata["verification"]["unique_frames"]
        .as_u64()
        .filter(|count| (1..=u64::from(frames)).contains(count))
        .context("source frame verification lacks a bounded unique count")?;
    let decoded_unique = metadata["decoded_unique_count"]
        .as_u64()
        .filter(|count| (1..=u64::from(frames)).contains(count))
        .context("saved video verification lacks a bounded unique count")?;
    ensure!(
        metadata["decoded_frames"].as_u64() == Some(u64::from(frames))
            && (source_unique == 1 || decoded_unique > 1),
        "saved video is incomplete or frozen although composition samples change"
    );
    Ok(())
}

fn prepare(root: &Path, source: &str, workspace: &Path) -> Result<Prepared> {
    let bytes = files::read(root, source, &["json"], MAX_MANIFEST)?;
    let manifest: Manifest = serde_json::from_slice(&bytes).context(
        "invalid composition manifest; use version, entry, assets, width, height, fps and frames",
    )?;
    ensure!(manifest.version == 1, "composition version must be 1");
    ensure!(
        (64..=1920).contains(&manifest.width)
            && (64..=1920).contains(&manifest.height)
            && manifest.width.is_multiple_of(2)
            && manifest.height.is_multiple_of(2)
            && u64::from(manifest.width) * u64::from(manifest.height) <= 2_073_600,
        "use even dimensions from 64 to 1920, with at most 2,073,600 pixels"
    );
    ensure!(
        [24, 30, 60].contains(&manifest.fps)
            && manifest.frames > 0
            && manifest.frames <= 900
            && manifest.frames <= manifest.fps * 30,
        "use 24, 30 or 60 fps, 1..900 frames and at most 30 seconds"
    );
    ensure!(
        manifest.assets.len() <= 64,
        "at most 64 explicit composition assets are supported"
    );
    ensure!(
        manifest.entry.ends_with(".html") && !manifest.entry.contains('/'),
        "composition entry must be an HTML filename in the manifest directory"
    );
    let parent = Path::new(source).parent().context("manifest parent")?;

    let mut inputs = BTreeSet::from([source.to_owned()]);
    let mut identities = vec![json!({"path":source,"sha256":util::sha256(&bytes)})];
    let mut total = bytes.len() as u64;
    let mut entry = String::new();
    let mut media = Vec::new();
    for name in std::iter::once(&manifest.entry).chain(&manifest.assets) {
        // Validate relative to the manifest as well as to the selected project.
        // An asset must not reach a sibling composition through a '..' segment.
        ensure!(
            !name.contains(['%', '?', '#']),
            "composition resource paths cannot contain %, ? or #"
        );
        util::safe_relative(Path::new(name))?;
        ensure!(
            !name.split('/').any(|s| s.eq_ignore_ascii_case("__mirket")),
            "__mirket is reserved for the pinned GSAP runtime"
        );
        ensure!(
            name == &manifest.entry || !name.eq_ignore_ascii_case("index.html"),
            "index.html is reserved for the private capture alias"
        );
        let relative = parent
            .join(name)
            .to_str()
            .context("UTF-8 resource path")?
            .to_owned();
        ensure!(
            inputs.insert(relative.clone()),
            "duplicate composition input: {relative}"
        );
        let mime = media_type(name)?;
        if mime.starts_with("video/") || mime.starts_with("audio/") {
            media.push(name.clone());
        }
        let asset = files::read(
            root,
            &relative,
            &[
                "html", "css", "js", "mjs", "svg", "png", "jpg", "jpeg", "webp", "woff", "woff2",
                "json", "wav", "mp3", "mp4", "webm",
            ],
            MAX_ASSET,
        )?;
        total += asset.len() as u64;
        ensure!(total <= MAX_INPUT, "composition inputs exceed 32 MiB");
        ensure!(!asset.is_empty(), "composition asset is empty: {relative}");
        if mime == "image/svg+xml" {
            static_svg_asset(&asset)?;
        }
        if mime.starts_with("text/") || ["image/svg+xml", "application/json"].contains(&mime) {
            std::str::from_utf8(&asset).context("composition text assets must be UTF-8")?;
        }
        let staged = format!("input/{name}");
        let destination = workspace.join(&staged);
        fs::create_dir_all(destination.parent().context("staged asset parent")?)?;
        fs::write(destination, &asset)?;
        if entry.is_empty() {
            entry = name.clone();
        }
        identities.push(json!({"path":relative,"sha256":util::sha256(&asset)}));
    }
    let job = json!({"entry":entry,"media":media,"width":manifest.width,"height":manifest.height,"fps":manifest.fps,"frames":manifest.frames});
    Ok(Prepared {
        manifest,
        job,
        identity: util::sha256(&serde_json::to_vec(&identities)?),
        inputs,
    })
}

async fn execute(
    spec: &LaunchSpec,
    source: &str,
    operation: &str,
    frame: Option<u32>,
    destination: Option<(&str, bool, Option<&str>)>,
    workspace: &Path,
    control: &process::Control<'_>,
) -> Result<EditorResult> {
    let mut prepared = prepare(&spec.cwd, source, workspace)?;
    if let Some(frame) = frame {
        ensure!(
            frame < prepared.manifest.frames,
            "frame must be below the manifest frame count"
        );
        prepared.job["frame"] = frame.into();
    }
    let extension = if operation == "render" {
        let name = destination.context("render output required")?.0;
        match Path::new(name).extension().and_then(|s| s.to_str()) {
            Some("mp4") => "mp4",
            Some("webm") => "webm",
            _ => bail!("render output must be .mp4 or .webm"),
        }
    } else {
        "png"
    };
    prepared.job["format"] = extension.into();
    let output_name = format!("output.{extension}");
    let output = workspace.join(&output_name);
    let target = destination
        .map(|(name, overwrite, expected)| {
            files::scoped(&spec.cwd, name, &[extension])?;
            files::distinct_output(&spec.cwd, name, prepared.inputs.iter().map(String::as_str))?;
            files::Destination::new(&spec.cwd, name, &[extension], overwrite, expected)
        })
        .transpose()?;
    prepared.job["operation"] = operation.into();
    if target.is_some() {
        prepared.job["output"] = output_name.into();
    }
    util::atomic_write(
        &workspace.join("job.json"),
        &serde_json::to_vec(&prepared.job)?,
    )?;
    let report = process::run(
        spec,
        &["job.json".into()],
        workspace,
        target.as_ref().map(|_| output.as_path()),
        control,
    )
    .await?;
    let metadata: Value = serde_json::from_slice(&report)
        .context("HyperFrames runner returned no valid result; output was not published")?;
    ensure!(
        metadata["engine"] == "hyperframes"
            && metadata["version"] == "0.8.149"
            && metadata["operation"] == operation
            && valid_isolation(&metadata, spec.container.is_some()),
        "HyperFrames result identity does not match requested operation"
    );
    for (key, value) in [
        ("width", prepared.manifest.width),
        ("height", prepared.manifest.height),
        ("fps", prepared.manifest.fps),
        ("frames", prepared.manifest.frames),
    ] {
        ensure!(
            metadata[key].as_u64() == Some(u64::from(value)),
            "HyperFrames result {key} differs from the composition"
        );
    }
    let mut result = EditorResult {
        operation: format!("composition_{operation}"),
        editable_source: Some(source.into()),
        source_sha256: Some(prepared.identity),
        pixel_dimensions: Some([prepared.manifest.width, prepared.manifest.height]),
        animation: Some(metadata.clone()),
        ..Default::default()
    };
    if operation == "verify" || operation == "render" {
        ensure!(
            metadata["verification"]["deterministic"] == true,
            "HyperFrames did not verify forward/reverse frame consistency"
        );
    }
    if let Some(target) = target {
        let bytes = process::output(&output)?;
        if operation == "frame" {
            validate_frame(
                &bytes,
                &metadata,
                [prepared.manifest.width, prepared.manifest.height],
                frame.context("requested frame required")?,
            )?;
        } else {
            ensure!(
                bytes.len() >= 32
                    && if extension == "mp4" {
                        &bytes[4..8] == b"ftyp"
                    } else {
                        &bytes[..4] == b"\x1a\x45\xdf\xa3"
                    },
                "encoder did not produce the requested media container"
            );
            ensure!(
                metadata["codec"] == if extension == "mp4" { "h264" } else { "vp9" },
                "export did not decode to the requested frames"
            );
            validate_saved_frames(&metadata, prepared.manifest.frames)?;
        }
        control.check()?;
        result.artifact = Some(target.publish(
            &bytes,
            if operation == "frame" {
                "image/png"
            } else {
                if extension == "mp4" {
                    "video/mp4"
                } else {
                    "video/webm"
                }
            },
        )?);
    }
    Ok(result)
}

pub async fn inspect(
    spec: &LaunchSpec,
    args: Inspect,
    workspace: &Path,
    control: &process::Control<'_>,
    verify: bool,
) -> Result<EditorResult> {
    execute(
        spec,
        &args.source,
        if verify { "verify" } else { "inspect" },
        None,
        None,
        workspace,
        control,
    )
    .await
}
pub async fn frame(
    spec: &LaunchSpec,
    args: Frame,
    workspace: &Path,
    control: &process::Control<'_>,
) -> Result<EditorResult> {
    execute(
        spec,
        &args.source,
        "frame",
        Some(args.frame),
        Some((
            &args.output,
            args.overwrite,
            args.expected_sha256.as_deref(),
        )),
        workspace,
        control,
    )
    .await
}
pub async fn render(
    spec: &LaunchSpec,
    args: Render,
    workspace: &Path,
    control: &process::Control<'_>,
) -> Result<EditorResult> {
    execute(
        spec,
        &args.source,
        "render",
        None,
        Some((
            &args.output,
            args.overwrite,
            args.expected_sha256.as_deref(),
        )),
        workspace,
        control,
    )
    .await
}

pub fn definitions() -> Vec<rmcp::model::Tool> {
    vec![
        super::definition::<Inspect>(
            "composition_inspect",
            "Inspect an editable local HyperFrames composition through its actual producer runtime. Version-1 JSON manifest: entry, explicit assets, width, height, fps, frames. Root data-composition-id/data-width/data-height/data-duration and the upstream GSAP timeline must agree. Mapped local HTML/CSS/JS/JSON/SVG/images/fonts, WAV/MP3/MP4/WebM only. Browser JavaScript runs in an offline Podman worker; Chromium's inner sandbox is disabled. Requests are bounded to private staged assets. Even dimensions 64..1920, <=1080p pixels, 24/30/60 fps, <=900 frames/30 seconds. No package manager, remote fonts or hosted service.",
            true,
        ),
        super::definition::<Frame>(
            "composition_frame",
            "Seek a HyperFrames composition to a zero-based frame using the actual upstream capture session and save a validated PNG. Project-relative source/output, distinct output, existing parent, explicit overwrite and optional expected_sha256. Sample intermediate frames to inspect motion.",
            false,
        ),
        super::definition::<Inspect>(
            "composition_verify",
            "Compare sampled forward and reverse HyperFrames seeks for identical frame bytes. Reject missing assets, remote requests, page failures and manifest/root mismatch. This reports runtime repeatability, not visual approval.",
            true,
        ),
        super::definition::<Render>(
            "composition_render",
            "Render through HyperFrames compilation, capture, media mixing and encoding to MP4 (H.264/AAC) or WebM (VP9/Opus, alpha). Explicit local asset manifest, no external resources. Verify forward/reverse samples, decode video and check dimensions/fps/frame count and expected audio before atomic publication. <=32 MiB output and 90 seconds per job; existing files require overwrite=true. Watch the saved output to judge quality.",
            false,
        ),
    ]
}

pub async fn invoke(
    spec: &LaunchSpec,
    tool: &str,
    value: Value,
    workspace: &Path,
    control: &process::Control<'_>,
) -> Result<EditorResult> {
    match tool {
        "composition_inspect" | "composition_verify" => {
            inspect(
                spec,
                serde_json::from_value(value)?,
                workspace,
                control,
                tool == "composition_verify",
            )
            .await
        }
        "composition_frame" => {
            frame(spec, serde_json::from_value(value)?, workspace, control).await
        }
        "composition_render" => {
            render(spec, serde_json::from_value(value)?, workspace, control).await
        }
        _ => bail!("unknown HyperFrames tool"),
    }
}

fn valid_isolation(value: &Value, container: bool) -> bool {
    container && value["isolation"] == "podman" && value["browser_sandbox"] == false
}

pub async fn probe(
    spec: &LaunchSpec,
    workspace: &Path,
    control: &process::Control<'_>,
) -> Result<()> {
    let result = process::run(spec, &["--probe".into()], workspace, None, control).await?;
    let value: Value =
        serde_json::from_slice(&result).context("invalid HyperFrames runtime probe")?;
    ensure!(
        value["engine"] == "hyperframes"
            && value["version"] == "0.8.149"
            && valid_isolation(&value, spec.container.is_some())
            && value["verification"]["deterministic"] == true
            && value["verification"]["unique_frames"]
                .as_u64()
                .is_some_and(|n| n > 1),
        "HyperFrames probe did not demonstrate upstream seeking and the selected isolation boundary"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png() -> Vec<u8> {
        use std::io::Write;
        fn chunk(bytes: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
            bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
            bytes.extend_from_slice(kind);
            bytes.extend_from_slice(data);
            let mut crc = flate2::Crc::new();
            crc.update(kind);
            crc.update(data);
            bytes.extend_from_slice(&crc.sum().to_be_bytes());
        }
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut header = Vec::new();
        header.extend_from_slice(&2u32.to_be_bytes());
        header.extend_from_slice(&2u32.to_be_bytes());
        header.extend_from_slice(&[8, 6, 0, 0, 0]);
        chunk(&mut bytes, b"IHDR", &header);
        let mut compressed =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
        compressed
            .write_all(&[
                0, 255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 255,
            ])
            .unwrap();
        chunk(&mut bytes, b"IDAT", &compressed.finish().unwrap());
        chunk(&mut bytes, b"IEND", &[]);
        bytes
    }

    #[test]
    fn saved_frame_rejects_stale_index_missing_identity_and_wrong_bytes() {
        let bytes = png();
        let report = json!({"frame":7,"output_bytes":bytes.len(),"sha256":util::sha256(&bytes)});
        validate_frame(&bytes, &report, [2, 2], 7).unwrap();
        assert!(validate_frame(&bytes, &report, [2, 2], 8).is_err());
        assert!(validate_frame(&bytes, &report, [2, 3], 7).is_err());
        assert!(validate_frame(&bytes, &json!({"frame":7}), [2, 2], 7).is_err());
        for (key, value) in [
            ("frame", json!(6)),
            ("sha256", json!("wrong")),
            ("output_bytes", json!(bytes.len() + 1)),
        ] {
            let mut wrong = report.clone();
            wrong[key] = value;
            assert!(validate_frame(&bytes, &wrong, [2, 2], 7).is_err(), "{key}");
        }
        assert!(validate_frame(b"successful but no PNG", &report, [2, 2], 7).is_err());
    }

    #[test]
    fn frozen_saved_video_is_refused_without_banning_static_compositions() {
        let changing = json!({"verification":{"unique_frames":3},"decoded_frames":30,"decoded_unique_count":20});
        validate_saved_frames(&changing, 30).unwrap();
        let mut frozen = changing.clone();
        frozen["decoded_unique_count"] = 1.into();
        assert!(validate_saved_frames(&frozen, 30).is_err());
        frozen["verification"]["unique_frames"] = 1.into();
        validate_saved_frames(&frozen, 30).unwrap();
        for (key, value) in [
            ("decoded_frames", json!(29)),
            ("decoded_unique_count", json!(0)),
            ("decoded_unique_count", json!(31)),
            ("decoded_unique_count", Value::Null),
        ] {
            let mut wrong = changing.clone();
            wrong[key] = value;
            assert!(validate_saved_frames(&wrong, 30).is_err(), "{key}");
        }
    }

    #[test]
    fn capture_must_match_the_enforced_runtime_boundary() {
        let container = json!({"isolation":"podman", "browser_sandbox":false});
        let native = json!({"isolation":"native-browser", "browser_sandbox":true});
        assert!(valid_isolation(&container, true));
        assert!(!valid_isolation(&native, false));
        assert!(!valid_isolation(&container, false));
        assert!(!valid_isolation(&native, true));
        assert!(!valid_isolation(
            &json!({"isolation":"podman", "browser_sandbox":true}),
            true
        ));
        assert!(!valid_isolation(&json!({}), true));
    }
    fn fixture(root: &Path) {
        fs::write(root.join("composition.json"), serde_json::to_vec(&json!({"version":1,"entry":"scene.html","assets":["tone.wav"],"width":320,"height":180,"fps":30,"frames":30})).unwrap()).unwrap();
        fs::write(
            root.join("scene.html"),
            "<!doctype html><h1>staged input</h1>",
        )
        .unwrap();
        fs::write(root.join("tone.wav"), "local bytes").unwrap();
    }
    #[test]
    fn mapped_bytes_identity_media_and_reserved_paths() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        fixture(&root);
        let scratch = tempfile::tempdir().unwrap();
        let before = prepare(&root, "composition.json", scratch.path()).unwrap();
        assert_eq!(before.job["entry"], "scene.html");
        assert_eq!(before.job["media"], json!(["tone.wav"]));
        fs::write(root.join("tone.wav"), "new local bytes").unwrap();
        assert_ne!(
            before.identity,
            prepare(&root, "composition.json", scratch.path())
                .unwrap()
                .identity
        );
        for path in ["../outside.wav", "__mirket/gsap.min.js", "index.html"] {
            fs::write(root.join("composition.json"),serde_json::to_vec(&json!({"version":1,"entry":"scene.html","assets":[path],"width":320,"height":180,"fps":30,"frames":30})).unwrap()).unwrap();
            assert!(prepare(&root, "composition.json", scratch.path()).is_err());
        }
    }
    #[test]
    fn output_cannot_alias_editable_sources() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        fixture(&root);
        let inputs = BTreeSet::from(["scene.html".to_owned()]);
        assert!(
            files::distinct_output(&root, "SCENE.HTML", inputs.iter().map(String::as_str)).is_err()
        );
        fs::hard_link(root.join("scene.html"), root.join("output.webm")).unwrap();
        assert!(
            files::distinct_output(&root, "output.webm", inputs.iter().map(String::as_str))
                .is_err()
        );
    }
}
