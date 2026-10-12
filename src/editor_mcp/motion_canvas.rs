//! Rich, editable vector choreography rendered by actual Motion Canvas 3.17.2.
use super::{EditorResult, files, process};
use crate::{package_mcp::LaunchSpec, util};
use anyhow::{Context, Result, bail, ensure};
use rmcp::{
    model::Tool,
    schemars::{self, JsonSchema},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;

const MAX_SOURCE: u64 = 2 * 1024 * 1024;
const VERSION: &str = "3.17.2";
const SOURCE_PIN: &str = "e735a995aa738ca682feb4204a39ee7db7b5d279";

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Inspect {
    /// Project-relative editable version-1 vector scene JSON. See docs/motion-canvas.md.
    pub source: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub source: String,
    /// Zero-based frame below the source's frame count.
    pub frame: u32,
    /// Distinct project-relative PNG; parent must already exist.
    pub output: String,
    #[serde(default)]
    pub overwrite: bool,
    pub expected_sha256: Option<String>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Render {
    pub source: String,
    /// Distinct project-relative silent H.264 MP4; parent must already exist.
    pub output: String,
    #[serde(default)]
    pub overwrite: bool,
    pub expected_sha256: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Scene {
    version: u32,
    width: u32,
    height: u32,
    fps: u32,
    frames: u32,
    background: Option<String>,
    #[serde(default)]
    gradients: Vec<Value>,
    nodes: Vec<Node>,
    #[serde(default)]
    tracks: Vec<Value>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Node {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    parent: Option<String>,
    props: serde_json::Map<String, Value>,
}

pub fn definitions() -> Vec<Tool> {
    vec![
        super::definition::<Inspect>(
            "motion_canvas_inspect",
            "Inspect an editable version-1 vector choreography JSON through actual Motion Canvas 3.17.2 Scene2D, PlaybackManager and Stage. Supports SVG path geometry and morphs, Inter Latin/Turkish text, rects, ellipses, nested groups, shape clipping, linear/radial/conic gradients and timed easing tracks. Returns actual runtime node/gradient state, source identity and font readiness; reports up to 64 nodes. No arbitrary project code, configs, scripts, cloud accounts or package installation. Even dimensions 64..1920 up to 1080p pixels; 24/30/60 fps, at most 900 frames/30 seconds, 256 nodes and 512 tracks. This is video/canvas choreography, not editable animated-SVG export or browser-native UI behavior.",
            true,
        ),
        super::definition::<Frame>(
            "motion_canvas_frame",
            "Seek a rich editable Motion Canvas vector scene to a zero-based frame and publish a validated PNG. Source stays unchanged; output must be distinct with an existing project-relative parent. Existing outputs require explicit overwrite; expected_sha256 protects a revision. Inspect intermediate frames and focal detail to judge visual quality.",
            false,
        ),
        super::definition::<Inspect>(
            "motion_canvas_verify",
            "Render sample frames of an editable Motion Canvas scene in forward and reverse order using its actual playback engine. Fail on changing pixels at the same frame, invalid path/gradient/choreography, missing fonts, runtime errors or blocked resources. Reports bounded frame hashes and unique count; this checks local frame repeatability, not aesthetics or universal determinism.",
            true,
        ),
        super::definition::<Render>(
            "motion_canvas_render",
            "Render a rich editable vector choreography through actual pinned Motion Canvas playback and shared local FFmpeg to silent H.264 MP4. Verify sampled frame repeatability, decode all saved frames and reconcile dimensions, timestamps, frame count and codec before atomic publication. Source is preserved. At most 1080p, 900 frames/30 seconds and 32 MiB output; 90-second job deadline. This profile does not use the official FFmpeg exporter or its installer.",
            false,
        ),
    ]
}

fn prepare(root: &Path, source: &str) -> Result<(Scene, Value, String)> {
    let bytes = files::read(root, source, &["json"], MAX_SOURCE)?;
    let scene: Scene =
        serde_json::from_slice(&bytes).context("invalid editable Motion Canvas scene JSON")?;
    ensure!(scene.version == 1, "Motion Canvas scene version must be 1");
    ensure!(
        scene.width >= 64
            && scene.width <= 1920
            && scene.height >= 64
            && scene.height <= 1920
            && scene.width.is_multiple_of(2)
            && scene.height.is_multiple_of(2)
            && u64::from(scene.width) * u64::from(scene.height) <= 2_073_600,
        "scene requires even dimensions 64..1920 with at most 1080p pixels"
    );
    ensure!(
        [24, 30, 60].contains(&scene.fps)
            && scene.frames > 0
            && scene.frames <= 900
            && scene.frames <= scene.fps * 30,
        "scene requires 24/30/60 fps and at most 900 frames/30 seconds"
    );
    ensure!(
        !scene.nodes.is_empty()
            && scene.nodes.len() <= 256
            && scene.tracks.len() <= 512
            && scene.gradients.len() <= 32,
        "scene exceeds node, gradient or choreography bounds"
    );
    if let Some(background) = &scene.background {
        ensure!(background.len() <= 9, "use a hex background color");
    }
    let mut ids = std::collections::BTreeSet::new();
    for node in &scene.nodes {
        ensure!(
            !node.id.is_empty() && node.id.len() <= 64 && ids.insert(node.id.clone()),
            "node IDs must be unique and bounded"
        );
        ensure!(
            ["group", "rect", "ellipse", "path", "text"].contains(&node.kind.as_str()),
            "unsupported vector node type"
        );
        ensure!(
            node.parent
                .as_ref()
                .is_none_or(|parent| parent != &node.id && ids.contains(parent)),
            "parents must precede their children"
        );
        ensure!(
            node.props.len() <= 24,
            "node property count exceeds the typed profile"
        );
    }
    let value = serde_json::from_slice(&bytes)?;
    Ok((scene, value, util::sha256(&bytes)))
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
    control.check()?;
    let (scene, value, identity) = prepare(&spec.cwd, source)?;
    if let Some(frame) = frame {
        ensure!(
            frame < scene.frames,
            "frame must be below the scene frame count"
        );
    }
    let extension = if operation == "render" { "mp4" } else { "png" };
    let output = workspace.join(format!("output.{extension}"));
    let target = destination
        .map(|(name, overwrite, expected)| {
            files::scoped(&spec.cwd, name, &[extension])?;
            files::distinct_output(&spec.cwd, name, [source])?;
            files::Destination::new(&spec.cwd, name, &[extension], overwrite, expected)
        })
        .transpose()?;
    let mut job = json!({"operation":operation,"scene":value});
    if let Some(frame) = frame {
        job["frame"] = frame.into();
    }
    if target.is_some() {
        job["output"] = format!("output.{extension}").into();
    }
    util::atomic_write(&workspace.join("job.json"), &serde_json::to_vec(&job)?)?;
    let bytes = process::run(
        spec,
        &["job.json".into()],
        workspace,
        target.as_ref().map(|_| output.as_path()),
        control,
    )
    .await?;
    let report: Value =
        serde_json::from_slice(&bytes).context("Motion Canvas runner returned no valid result")?;
    ensure!(
        report["engine"] == "motion-canvas"
            && report["version"] == VERSION
            && report["source_commit"] == SOURCE_PIN
            && report["operation"] == operation
            && report["inspection"]["playback_api"] == "Scene2D+PlaybackManager+Stage",
        "runtime did not establish the requested Motion Canvas engine and operation"
    );
    for (key, expected) in [
        ("width", scene.width),
        ("height", scene.height),
        ("fps", scene.fps),
        ("frames", scene.frames),
    ] {
        ensure!(
            report[key].as_u64() == Some(u64::from(expected)),
            "Motion Canvas {key} differs from the editable source"
        );
    }
    if operation == "verify" || operation == "render" {
        ensure!(
            report["verification"]["deterministic"] == true,
            "frame repeatability was not verified"
        );
    }
    let mut result = EditorResult {
        operation: format!("motion_canvas_{operation}"),
        editable_source: Some(source.into()),
        source_sha256: Some(identity),
        pixel_dimensions: Some([scene.width, scene.height]),
        animation: Some(report.clone()),
        ..Default::default()
    };
    if let Some(target) = target {
        let bytes = process::output(&output)?;
        if operation == "frame" {
            let (width, height) = process::png(&bytes, Some(scene.width))?;
            ensure!(
                width == scene.width
                    && height == scene.height
                    && report["frame"].as_u64() == frame.map(u64::from)
                    && report["sha256"] == util::sha256(&bytes),
                "saved frame does not match the requested scene frame"
            );
        } else {
            ensure!(
                bytes.len() >= 32
                    && &bytes[4..8] == b"ftyp"
                    && report["decoded_frames"].as_u64() == Some(u64::from(scene.frames))
                    && report["codec"] == "h264",
                "saved output was not decoded as the expected H.264 video"
            );
        }
        control.check()?;
        result.artifact = Some(target.publish(
            &bytes,
            if operation == "frame" {
                "image/png"
            } else {
                "video/mp4"
            },
        )?);
    }
    Ok(result)
}

pub async fn invoke(
    spec: &LaunchSpec,
    tool: &str,
    value: Value,
    workspace: &Path,
    control: &process::Control<'_>,
) -> Result<EditorResult> {
    match tool {
        "motion_canvas_inspect" | "motion_canvas_verify" => {
            let args: Inspect = serde_json::from_value(value)?;
            execute(
                spec,
                &args.source,
                if tool.ends_with("verify") {
                    "verify"
                } else {
                    "inspect"
                },
                None,
                None,
                workspace,
                control,
            )
            .await
        }
        "motion_canvas_frame" => {
            let args: Frame = serde_json::from_value(value)?;
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
        "motion_canvas_render" => {
            let args: Render = serde_json::from_value(value)?;
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
        _ => bail!("unsupported Motion Canvas operation"),
    }
}
pub async fn probe(
    spec: &LaunchSpec,
    workspace: &Path,
    control: &process::Control<'_>,
) -> Result<()> {
    let bytes = process::run(spec, &["--probe".into()], workspace, None, control).await?;
    let report: Value =
        serde_json::from_slice(&bytes).context("Motion Canvas probe returned no JSON")?;
    ensure!(
        report["engine"] == "motion-canvas"
            && report["version"] == VERSION
            && report["source_commit"] == SOURCE_PIN
            && report["inspection"]["playback_api"] == "Scene2D+PlaybackManager+Stage"
            && report["verification"]["deterministic"] == true
            && report["verification"]["unique_count"]
                .as_u64()
                .is_some_and(|n| n > 1),
        "Motion Canvas probe did not verify actual changing rendered geometry"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    fn fixture() -> Value {
        json!({"version":1,"width":640,"height":360,"fps":30,"frames":60,"nodes":[{"id":"path","type":"path","props":{"data":"M0 0L20 20Z","fill":"#fff"}}]})
    }
    #[test]
    fn source_identity_changes_and_invalid_structural_input_is_refused() {
        let root = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let write = |value: &Value| {
            fs::write(
                root.path().join("scene.json"),
                serde_json::to_vec(value).unwrap(),
            )
            .unwrap()
        };
        let original = fixture();
        write(&original);
        let before = prepare(root.path(), "scene.json").unwrap().2;
        let mut revised = original.clone();
        revised["nodes"][0]["props"]["fill"] = json!("#123");
        write(&revised);
        assert_ne!(before, prepare(root.path(), "scene.json").unwrap().2);
        for (key, value) in [
            ("version", json!(2)),
            ("frames", json!(901)),
            ("fps", json!(300)),
            ("width", json!(641)),
            ("config", json!("do not execute")),
            ("nodes", json!([])),
        ] {
            let mut invalid = original.clone();
            invalid[key] = value;
            write(&invalid);
            assert!(prepare(root.path(), "scene.json").is_err(), "{key}");
        }
        assert!(prepare(root.path(), "../scene.json").is_err());
    }
    #[cfg(unix)]
    #[test]
    fn exports_cannot_alias_or_symlink_the_editable_source() {
        let root = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        fs::write(root.path().join("scene.json"), b"{}").unwrap();
        fs::hard_link(
            root.path().join("scene.json"),
            root.path().join("alias.mp4"),
        )
        .unwrap();
        assert!(files::distinct_output(root.path(), "alias.mp4", ["scene.json"]).is_err());
        std::os::unix::fs::symlink(
            root.path().join("scene.json"),
            root.path().join("symlink.mp4"),
        )
        .unwrap();
        assert!(files::Destination::new(root.path(), "symlink.mp4", &["mp4"], true, None).is_err());
    }
}
