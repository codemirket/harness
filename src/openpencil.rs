//! Bounded editable OpenPencil file contract. Publication and process ownership stay with design.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const ENGINE_VERSION: &str = "0.16.0";
pub const MODE: &str = "editable_file";
pub const MAX_INPUT_BYTES: usize = 1024 * 1024;
pub const MAX_OUTPUT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_NODES: usize = 256;
pub const MAX_DEPTH: usize = 16;
pub const MAX_EDITS: usize = 64;
pub const MAX_DIMENSION: f64 = 2048.0;
pub const MIN_TIMEOUT_MS: u64 = 100;
pub const MAX_TIMEOUT_MS: u64 = 60_000;

/// Exact private paths supplied by the distribution owner; no PATH or host package fallback.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpenPencilPayload {
    pub node: PathBuf,
    pub adapter: PathBuf,
    pub runtime_root: PathBuf,
    pub runtime_sha256: String,
}

/// Fixed geometry, single-line text and fixed no-wrap native layout; no resources or code.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpenPencilProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<OpenPencilLayout>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_top: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_right: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_bottom: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_left: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_align: Option<OpenPencilPrimaryAlign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counter_align: Option<OpenPencilCounterAlign>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenPencilLayout {
    None,
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenPencilPrimaryAlign {
    Start,
    Center,
    End,
    SpaceBetween,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenPencilCounterAlign {
    Start,
    Center,
    End,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpenPencilEdit {
    pub path: String,
    pub properties: OpenPencilProperties,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum OpenPencilOperation {
    Create,
    Patch { edits: Vec<OpenPencilEdit> },
    ExportSvg { target_path: String },
}

pub fn valid_target_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 2048
        && path.split('/').count() <= MAX_DEPTH
        && path.split('/').all(|part| {
            !part.is_empty() && part.chars().count() <= 128 && !part.chars().any(char::is_control)
        })
}

fn bounded(value: Option<f64>, minimum: f64, maximum: f64) -> bool {
    value.is_none_or(|number| number.is_finite() && (minimum..=maximum).contains(&number))
}

impl OpenPencilProperties {
    pub fn validate(&self) -> Result<(), String> {
        if self == &Self::default() {
            return Err("an OpenPencil edit needs editable design properties".into());
        }
        if !bounded(self.x, -MAX_DIMENSION, MAX_DIMENSION)
            || !bounded(self.y, -MAX_DIMENSION, MAX_DIMENSION)
            || !bounded(self.width, 1.0, MAX_DIMENSION)
            || !bounded(self.height, 1.0, MAX_DIMENSION)
            || !bounded(self.opacity, 0.0, 1.0)
            || !bounded(self.corner_radius, 0.0, 512.0)
            || !bounded(self.font_size, 1.0, 256.0)
            || !bounded(self.line_height, 1.0, 512.0)
            || [
                self.gap,
                self.padding_top,
                self.padding_right,
                self.padding_bottom,
                self.padding_left,
            ]
            .into_iter()
            .any(|value| !bounded(value, 0.0, 512.0))
        {
            return Err("OpenPencil properties exceed finite bounds".into());
        }
        if self.text.as_ref().is_some_and(|text| {
            text.is_empty()
                || text.encode_utf16().count() > 4096
                || text.chars().any(|c| {
                    c.is_control()
                        || ('\u{202a}'..='\u{202e}').contains(&c)
                        || ('\u{2066}'..='\u{2069}').contains(&c)
                })
        }) {
            return Err(
                "OpenPencil text must be one bounded nonempty line without controls".into(),
            );
        }
        if self.fill.as_ref().is_some_and(|fill| {
            !matches!(fill.len(), 7 | 9)
                || !fill.starts_with('#')
                || !fill.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
        }) {
            return Err("OpenPencil fill must be a hexadecimal RGB or RGBA color".into());
        }
        Ok(())
    }
}

impl OpenPencilOperation {
    pub const fn input_extension(&self) -> &'static str {
        match self {
            Self::Create => "json",
            Self::Patch { .. } | Self::ExportSvg { .. } => "fig",
        }
    }
    pub const fn output_extension(&self) -> &'static str {
        match self {
            Self::Create | Self::Patch { .. } => "fig",
            Self::ExportSvg { .. } => "svg",
        }
    }
    pub const fn wire_name(&self) -> &'static str {
        match self {
            Self::Create => "openpencil.create",
            Self::Patch { .. } => "openpencil.patch",
            Self::ExportSvg { .. } => "openpencil.export_svg",
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Create => Ok(()),
            Self::Patch { edits } => {
                if edits.is_empty() || edits.len() > MAX_EDITS {
                    return Err("OpenPencil patch requires 1–64 bounded edits".into());
                }
                for edit in edits {
                    if !valid_target_path(&edit.path) {
                        return Err("OpenPencil edit target path is invalid".into());
                    }
                    edit.properties.validate()?;
                }
                Ok(())
            }
            Self::ExportSvg { target_path } => {
                if valid_target_path(target_path) {
                    Ok(())
                } else {
                    Err("OpenPencil SVG target path is invalid".into())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_editable_properties_and_only_three_operations_are_representable() {
        let good = OpenPencilOperation::Patch {
            edits: vec![OpenPencilEdit {
                path: "Card/Box".into(),
                properties: OpenPencilProperties {
                    width: Some(144.0),
                    ..Default::default()
                },
            }],
        };
        assert!(good.validate().is_ok());
        assert!(
            serde_json::from_str::<OpenPencilOperation>(
                r#"{"kind":"patch","edits":[{"path":"Text","properties":{"font_family":"remote font"}}]}"#
            )
            .is_err()
        );
        assert!(serde_json::from_str::<OpenPencilOperation>(r#"{"kind":"text_edit"}"#).is_err());
        assert!(
            OpenPencilProperties {
                width: Some(f64::NAN),
                ..Default::default()
            }
            .validate()
            .is_err()
        );
        assert!(!valid_target_path("Card//Box"));
        let editable: OpenPencilOperation = serde_json::from_str(r#"{"kind":"patch","edits":[{"path":"Button/Label","properties":{"text":"Sent İ","font_size":18}},{"path":"Button","properties":{"layout":"vertical","gap":12,"padding_left":8,"counter_align":"center"}}]}"#).unwrap();
        assert!(editable.validate().is_ok());
        for text in ["", "two\nlines", "\u{202e}direction"] {
            assert!(
                OpenPencilProperties {
                    text: Some(text.into()),
                    ..Default::default()
                }
                .validate()
                .is_err()
            );
        }
        assert!(
            OpenPencilProperties {
                text: Some("😀".repeat(2049)),
                ..Default::default()
            }
            .validate()
            .is_err()
        );
        assert!(serde_json::from_str::<OpenPencilProperties>(r#"{"layout":"grid"}"#).is_err());
    }
}
