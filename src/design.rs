//! Finite, revision-bound design operations shared by the engine and worker.
//!
//! The host owns project permissions and publication. A private Node leaf
//! compiles tokens or optimizes SVG; the same-binary worker rasterizes SVG.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

pub const PROTOCOL_VERSION: u32 = 1;
pub const MAX_INPUT_BYTES: usize = 1024 * 1024;
pub const MAX_RASTER_INPUT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_OUTPUT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_DIMENSION: u32 = 2048;
pub const MAX_TIMEOUT_MS: u64 = 60_000;

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenFormat {
    Css,
    Json,
}

impl TokenFormat {
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Css => "css",
            Self::Json => "json",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RasterFormat {
    Png,
    Jpeg,
    Webp,
    Avif,
}

impl RasterFormat {
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
            Self::Avif => "avif",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RasterFit {
    Fill,
    Inside,
}

#[derive(Clone, Debug, Deserialize, PartialEq, JsonSchema, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DesignOperation {
    TokensCompile {
        format: TokenFormat,
    },
    SvgOptimize,
    SvgRenderPng {
        width: u32,
        height: u32,
    },
    RasterTransform {
        source_format: RasterFormat,
        output_format: RasterFormat,
        width: u32,
        height: u32,
        fit: RasterFit,
    },
    OpenPencil {
        operation: crate::openpencil::OpenPencilOperation,
    },
}

impl DesignOperation {
    pub const fn input_extension(&self) -> &'static str {
        match self {
            Self::TokensCompile { .. } => "json",
            Self::SvgOptimize | Self::SvgRenderPng { .. } => "svg",
            Self::RasterTransform { source_format, .. } => source_format.extension(),
            Self::OpenPencil { operation } => operation.input_extension(),
        }
    }
    pub const fn output_extension(&self) -> &'static str {
        match self {
            Self::TokensCompile { format } => format.extension(),
            Self::SvgOptimize => "svg",
            Self::SvgRenderPng { .. } => "png",
            Self::RasterTransform { output_format, .. } => output_format.extension(),
            Self::OpenPencil { operation } => operation.output_extension(),
        }
    }

    pub const fn max_input_bytes(&self) -> usize {
        match self {
            Self::RasterTransform { .. } => MAX_RASTER_INPUT_BYTES,
            _ => MAX_INPUT_BYTES,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DesignPayload {
    pub node: PathBuf,
    pub adapter: PathBuf,
    pub font: PathBuf,
    /// Present only after an exact private raster module is selected.
    pub raster_module: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    content = "payload",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum DesignLeafPayload {
    Standard(DesignPayload),
    OpenPencil(crate::openpencil::OpenPencilPayload),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DesignRequest {
    pub protocol_version: u32,
    pub release_id: String,
    pub job_id: String,
    pub job_root: PathBuf,
    pub input_path: PathBuf,
    pub source_revision: String,
    pub output_path: PathBuf,
    pub operation: DesignOperation,
    pub timeout_ms: u64,
    pub max_output_bytes: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DesignOutcome {
    pub output_revision: String,
    pub output_bytes: usize,
    pub input_bytes: usize,
    pub stderr_bytes: usize,
    pub cleanup_confirmed: bool,
}

fn safe_absolute(path: &Path) -> bool {
    path.is_absolute()
        && path
            .to_str()
            .is_some_and(|value| value.len() <= 4096 && !value.contains('\0'))
        && !path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
}

pub fn is_revision(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

impl DesignRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.protocol_version != PROTOCOL_VERSION
            || crate::release::ReleaseId::parse(&self.release_id).is_err()
            || crate::release::ReleaseId::parse(&self.job_id).is_err()
        {
            return Err("design request has invalid protocol or identity".into());
        }
        if !safe_absolute(&self.job_root)
            || !safe_absolute(&self.input_path)
            || !safe_absolute(&self.output_path)
            || !self.input_path.starts_with(&self.job_root)
            || !self.output_path.starts_with(&self.job_root)
            || self.input_path == self.output_path
            || self.input_path.extension().and_then(|v| v.to_str())
                != Some(self.operation.input_extension())
            || self.output_path.extension().and_then(|v| v.to_str())
                != Some(self.operation.output_extension())
        {
            return Err(
                "design paths must be bounded, absolute, job-scoped and format-matched".into(),
            );
        }
        if !is_revision(&self.source_revision) {
            return Err("design source revision must be a lowercase SHA-256 digest".into());
        }
        if self.timeout_ms == 0
            || self.timeout_ms > MAX_TIMEOUT_MS
            || self.max_output_bytes == 0
            || self.max_output_bytes > MAX_OUTPUT_BYTES
        {
            return Err("design work deadline or output budget is invalid".into());
        }
        if let DesignOperation::SvgRenderPng { width, height } = &self.operation
            && (*width == 0
                || *height == 0
                || *width > MAX_DIMENSION
                || *height > MAX_DIMENSION
                || width
                    .checked_mul(*height)
                    .is_none_or(|pixels| pixels > MAX_DIMENSION * MAX_DIMENSION))
        {
            return Err("design render dimensions exceed the pixel budget".into());
        }
        if let DesignOperation::RasterTransform { width, height, .. } = &self.operation
            && (*width == 0
                || *height == 0
                || *width > MAX_DIMENSION
                || *height > MAX_DIMENSION
                || width
                    .checked_mul(*height)
                    .is_none_or(|pixels| pixels > MAX_DIMENSION * MAX_DIMENSION))
        {
            return Err("raster dimensions exceed the pixel budget".into());
        }
        if let DesignOperation::OpenPencil { operation } = &self.operation {
            operation.validate()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(operation: DesignOperation) -> DesignRequest {
        DesignRequest {
            protocol_version: 1,
            release_id: "0.0.1:test".into(),
            job_id: "job-1".into(),
            job_root: PathBuf::from("/private/job"),
            input_path: PathBuf::from(format!(
                "/private/job/source.{}",
                operation.input_extension()
            )),
            source_revision: format!("sha256:{}", "a".repeat(64)),
            output_path: PathBuf::from(format!(
                "/private/job/result.{}",
                operation.output_extension()
            )),
            operation,
            timeout_ms: 30_000,
            max_output_bytes: MAX_OUTPUT_BYTES,
        }
    }

    #[test]
    fn only_scoped_format_matched_requests_validate() {
        let mut value = request(DesignOperation::TokensCompile {
            format: TokenFormat::Css,
        });
        assert!(value.validate().is_ok());
        value.output_path = PathBuf::from("/private/elsewhere/result.css");
        assert!(value.validate().is_err());
        value = request(DesignOperation::SvgRenderPng {
            width: 2048,
            height: 2048,
        });
        assert!(value.validate().is_ok());
        value.operation = DesignOperation::SvgRenderPng {
            width: 2049,
            height: 1,
        };
        assert!(value.validate().is_err());

        let raster = DesignOperation::RasterTransform {
            source_format: RasterFormat::Png,
            output_format: RasterFormat::Webp,
            width: 512,
            height: 256,
            fit: RasterFit::Inside,
        };
        assert_eq!(raster.input_extension(), "png");
        assert_eq!(raster.output_extension(), "webp");
        assert_eq!(raster.max_input_bytes(), MAX_RASTER_INPUT_BYTES);
        let mut value = request(raster);
        assert!(value.validate().is_ok());
        value.operation = DesignOperation::RasterTransform {
            source_format: RasterFormat::Png,
            output_format: RasterFormat::Webp,
            width: 2049,
            height: 256,
            fit: RasterFit::Fill,
        };
        assert!(value.validate().is_err());
    }
}
