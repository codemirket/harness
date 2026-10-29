//! The finite authored-HTML video contract. HTML/CSS/JS remains the editable
//! source; these records select saved files and output profiles, not a scene DSL.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

pub const MAX_FILES: usize = 128;
pub const MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;
// Video bodies are streamed through fixed-size buffers. At ten minutes this
// permits about 28 Mbit/s, while remaining finite for two authorized heavy jobs.
pub const MAX_OUTPUT_BYTES: usize = 2 * 1024 * 1024 * 1024;
// PNG exports retain frames while building the tar: two output extents plus
// 1 GiB for producer intermediates, profile/cache, sources and diagnostics.
pub const MAX_WORK_BYTES: u64 = 5 * 1024 * 1024 * 1024;
pub const MAX_TIMEOUT_MS: u64 = 30 * 60 * 1000;
pub const MAX_FRAMES: u32 = 18_000;

#[derive(Clone, Debug)]
pub struct VideoPayload {
    pub node: PathBuf,
    pub adapter: PathBuf,
    /// Empty in a release template until explicit setup binds rendering Chrome.
    /// Installed runtime resolution never exposes an incomplete template for jobs.
    pub chromium: PathBuf,
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
}

impl VideoPayload {
    /// A template can ship the adapter before external tools are selected.
    /// Identity and capabilities are checked separately by setup and at use.
    pub fn tools_configured(&self) -> bool {
        [&self.chromium, &self.ffmpeg, &self.ffprobe]
            .into_iter()
            .all(|path| path.is_absolute())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VideoFormat {
    Mp4,
    Webm,
    Gif,
    PngSequenceTar,
}
impl VideoFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Webm => "webm",
            Self::Gif => "gif",
            Self::PngSequenceTar => "tar",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FrameRate {
    pub numerator: u32,
    pub denominator: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VideoAsset {
    /// Relative to the selected HTML file's directory. No implicit tree scan.
    pub path: PathBuf,
    pub expected_revision: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VideoProfile {
    pub width: u32,
    pub height: u32,
    pub fps: FrameRate,
    /// Finite capture count, also subject to the 2 GiB streamed output and
    /// 5 GiB private work-tree budgets. Content must fit both limits.
    pub frame_count: u32,
    pub format: VideoFormat,
    /// Selected UTF-8 JSON asset containing ordered, non-overlapping cues with
    /// integer `start_frame`, `end_frame`, and Unicode `text`. Burned into the
    /// render; the PNG tar also retains SRT and WebVTT sidecars.
    pub captions_path: Option<PathBuf>,
}
impl VideoProfile {
    pub fn validate(&self) -> Result<(), String> {
        let fps = self.fps;
        if self.width == 0
            || self.height == 0
            || self.width > 1920
            || self.height > 1920
            || u64::from(self.width) * u64::from(self.height) > 1920 * 1080
            || !self.width.is_multiple_of(2)
            || !self.height.is_multiple_of(2)
            || fps.numerator == 0
            || fps.denominator == 0
            || fps.numerator > 60_000
            || fps.denominator > 1001
            || fps.numerator > 60 * fps.denominator
            || fps.numerator < fps.denominator
            || self.frame_count == 0
            || self.frame_count > MAX_FRAMES
            || u64::from(self.frame_count) * u64::from(fps.denominator)
                > 600 * u64::from(fps.numerator)
        {
            return Err(
                "video profile exceeds finite dimension, frame, fps or duration limits".into(),
            );
        }
        if self.captions_path.as_deref().is_some_and(|p| !relative(p)) {
            return Err("caption source must be a selected relative asset".into());
        }
        Ok(())
    }
}

#[derive(Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VideoPrepareArgs {
    pub expected_release_id: String,
    pub project_root: PathBuf,
    pub source_path: PathBuf,
    pub expected_revision: String,
    pub assets: Vec<VideoAsset>,
    pub destination_path: PathBuf,
    /// Development export profile with finite file-backed output/scratch budgets.
    /// Source integration does not establish measured long-video support.
    pub profile: VideoProfile,
}

#[derive(Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VideoJobArgs {
    pub expected_release_id: String,
    pub project_root: PathBuf,
    pub operation_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceFile {
    pub path: PathBuf,
    pub revision: String,
    pub bytes: usize,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VideoRequest {
    pub protocol_version: u32,
    pub release_id: String,
    pub job_id: String,
    pub job_root: PathBuf,
    pub input_root: PathBuf,
    pub entry_file: PathBuf,
    pub sources: Vec<SourceFile>,
    pub output_path: PathBuf,
    pub profile: VideoProfile,
    pub timeout_ms: u64,
    pub max_output_bytes: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VideoOutcome {
    pub output_revision: String,
    pub output_bytes: usize,
    pub evidence: serde_json::Value,
    pub cleanup_confirmed: bool,
}

/// Bounded authoring feedback, never source bodies or caption text. The process
/// boundary checks paths against the revision-bound selected source manifest.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VideoDiagnostic {
    pub phase: String,
    pub path: PathBuf,
    pub field: String,
    pub reason: String,
    pub expected: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cue_index: Option<u32>,
}

pub fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .to_str()
            .is_some_and(|v| v.len() <= 1024 && !v.contains(['\0', '\\', ':']))
        && path.components().all(|v| matches!(v, Component::Normal(_)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn video_template_requires_explicit_absolute_tool_paths() {
        let root = std::env::temp_dir();
        let mut payload = VideoPayload {
            node: root.join("node"),
            adapter: root.join("video-main.js"),
            chromium: PathBuf::new(),
            ffmpeg: root.join("ffmpeg"),
            ffprobe: root.join("ffprobe"),
        };
        assert!(!payload.tools_configured());
        payload.chromium = PathBuf::from("chrome");
        assert!(!payload.tools_configured());
        payload.chromium = root.join("selected-chrome");
        assert!(payload.tools_configured());
        payload.ffprobe = PathBuf::new();
        assert!(!payload.tools_configured());
    }

    #[test]
    fn rational_profile_and_asset_bounds_refuse_unsafe_work() {
        let mut p = VideoProfile {
            width: 640,
            height: 360,
            fps: FrameRate {
                numerator: 24_000,
                denominator: 1001,
            },
            frame_count: 48,
            format: VideoFormat::PngSequenceTar,
            captions_path: None,
        };
        assert!(p.validate().is_ok());
        p.fps.denominator = 0;
        assert!(p.validate().is_err());
        assert!(!relative(Path::new("../font.ttf")));
        assert!(!relative(Path::new("C:\\font.ttf")));
        assert!(relative(Path::new("fonts/yerel İ.ttf")));
    }
}
