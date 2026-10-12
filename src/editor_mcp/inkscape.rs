use super::{
    EditorResult,
    files::{self, Destination},
    process::{self, Control},
    svg,
};
use crate::package_mcp::LaunchSpec;
use anyhow::{Context, Result, bail};
use rmcp::schemars;
use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Save {
    /// Existing project directory, relative SVG filename. Editable source is preserved as supplied.
    pub path: String,
    /// Self-contained UTF-8 vector markup, at most 96 KiB. Shapes, paths, gradients, clipping and text; no scripts, images, CSS sheets, filters or external resources.
    pub svg: String,
    #[serde(default)]
    pub overwrite: bool,
    /// Optional precondition for an existing file, obtained from an earlier result.
    pub expected_sha256: Option<String>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Inspect {
    pub path: String,
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    Png,
    Svg,
    Pdf,
}
impl Format {
    fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Svg => "svg",
            Self::Pdf => "pdf",
        }
    }
    fn media(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Svg => "image/svg+xml",
            Self::Pdf => "application/pdf",
        }
    }
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Export {
    /// Project-relative editable static SVG source. The output must be different.
    pub source: String,
    pub output: String,
    pub format: Format,
    /// PNG width in pixels, 1..2048. Height follows the page or selected object's ratio.
    pub width: Option<u32>,
    /// Export this existing SVG ID alone. Omit for the complete page.
    pub object_id: Option<String>,
    #[serde(default)]
    pub overwrite: bool,
    pub expected_sha256: Option<String>,
}
#[derive(Debug, Serialize, JsonSchema)]
pub struct Bounds {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

async fn bounds(
    spec: &LaunchSpec,
    bytes: &[u8],
    workspace: &Path,
    control: &Control<'_>,
) -> Result<Vec<Bounds>> {
    let input = workspace.join("input.svg");
    std::fs::write(&input, bytes)?;
    let output = process::run(
        spec,
        &["--query-all".into(), input.to_string_lossy().into_owned()],
        workspace,
        None,
        control,
    )
    .await?;
    let text = std::str::from_utf8(&output).context("Inkscape returned non-UTF8 bounds")?;
    let mut result = vec![];
    for line in text.lines().filter(|s| !s.is_empty()) {
        let fields: Vec<_> = line.split(',').collect();
        if fields.len() != 5
            || !svg::identifier(fields[0])
            || result.len() >= svg::MAX_NODES as usize
        {
            bail!("Inkscape returned invalid or excessive object bounds");
        }
        let values = fields[1..]
            .iter()
            .map(|s| s.parse::<f64>())
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if values
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 1_000_000.0)
            || values[2] < 0.0
            || values[3] < 0.0
        {
            bail!("Inkscape returned non-finite or out-of-range object bounds");
        }
        result.push(Bounds {
            id: fields[0].into(),
            x: values[0],
            y: values[1],
            width: values[2],
            height: values[3],
        });
    }
    if !result.iter().any(|b| b.width > 0.0 && b.height > 0.0) {
        bail!(
            "Inkscape found no visible vector bounds; inspect geometry and paint before saving/exporting"
        );
    }
    Ok(result)
}

pub async fn save(
    spec: &LaunchSpec,
    args: Save,
    workspace: &Path,
    control: &Control<'_>,
) -> Result<EditorResult> {
    if args.svg.len() > 96 * 1024 {
        bail!("svg_save markup exceeds 96 KiB; use smaller editable vector sources");
    }
    let inspection = svg::inspect(args.svg.as_bytes(), false)?;
    let destination = Destination::new(
        &spec.cwd,
        &args.path,
        &["svg"],
        args.overwrite,
        args.expected_sha256.as_deref(),
    )?;
    let objects = bounds(spec, args.svg.as_bytes(), workspace, control).await?;
    control.check()?;
    let artifact = destination.publish(args.svg.as_bytes(), "image/svg+xml")?;
    Ok(EditorResult {
        operation: "svg_save".into(),
        editable_source: Some(args.path),
        artifact: Some(artifact),
        inspection: Some(inspection),
        bounds: Some(objects),
        ..Default::default()
    })
}

pub async fn inspect(
    spec: &LaunchSpec,
    args: Inspect,
    workspace: &Path,
    control: &Control<'_>,
) -> Result<EditorResult> {
    let bytes = files::read(&spec.cwd, &args.path, &["svg"], files::MAX_SVG_BYTES)?;
    let inspection = svg::inspect(&bytes, false)?;
    let objects = bounds(spec, &bytes, workspace, control).await?;
    Ok(EditorResult {
        operation: "svg_inspect".into(),
        editable_source: Some(args.path),
        source_sha256: Some(crate::util::sha256(&bytes)),
        inspection: Some(inspection),
        bounds: Some(objects),
        ..Default::default()
    })
}

pub async fn export(
    spec: &LaunchSpec,
    args: Export,
    workspace: &Path,
    control: &Control<'_>,
) -> Result<EditorResult> {
    files::scoped(&spec.cwd, &args.source, &["svg"])?;
    files::scoped(&spec.cwd, &args.output, &[args.format.extension()])?;
    files::distinct_output(&spec.cwd, &args.output, [&args.source[..]])?;
    let bytes = files::read(&spec.cwd, &args.source, &["svg"], files::MAX_SVG_BYTES)?;
    let inspection = svg::inspect(&bytes, false)?;
    let destination = Destination::new(
        &spec.cwd,
        &args.output,
        &[args.format.extension()],
        args.overwrite,
        args.expected_sha256.as_deref(),
    )?;
    if let Some(id) = &args.object_id
        && (!svg::identifier(id)
            || !inspection.objects.iter().any(|o| {
                o.id == *id
                    && !matches!(
                        o.element.as_str(),
                        "defs"
                            | "linearGradient"
                            | "radialGradient"
                            | "stop"
                            | "clipPath"
                            | "title"
                            | "desc"
                    )
            }))
    {
        bail!("object_id must identify an existing drawable object or group");
    }
    let objects = bounds(spec, &bytes, workspace, control).await?;
    let staged = workspace.join(format!("export.{}", args.format.extension()));
    let mut argv = vec![
        format!("--export-type={}", args.format.extension()),
        format!("--export-filename={}", staged.display()),
    ];
    if let Some(id) = &args.object_id {
        argv.extend([format!("--export-id={id}"), "--export-id-only".into()]);
    } else {
        argv.push("--export-area-page".into());
    }
    let width = if matches!(args.format, Format::Png) {
        let width = args.width.unwrap_or(1024);
        if !(1..=2048).contains(&width) {
            bail!("PNG width must be 1..2048 pixels");
        }
        let ratio = if let Some(id) = &args.object_id {
            let b = objects
                .iter()
                .find(|b| b.id == *id)
                .context("selected object has no rendered bounds")?;
            b.height / b.width
        } else {
            inspection.height / inspection.width
        };
        let height = (f64::from(width) * ratio).ceil();
        if !height.is_finite()
            || !(1.0..=8192.0).contains(&height)
            || height * f64::from(width) > 16_777_216.0
        {
            bail!("requested PNG dimensions exceed the pixel budget");
        }
        argv.push(format!("--export-width={width}"));
        Some(width)
    } else {
        if args.width.is_some() {
            bail!("width applies only to PNG export");
        }
        None
    };
    match args.format {
        Format::Svg => argv.push("--export-plain-svg".into()),
        Format::Pdf => argv.push("--export-pdf-version=1.5".into()),
        Format::Png => {}
    }
    argv.push(workspace.join("input.svg").to_string_lossy().into_owned());
    process::run(spec, &argv, workspace, Some(&staged), control).await?;
    let output = process::output(&staged)?;
    let dimensions = match args.format {
        Format::Png => {
            let (w, h) = process::png(&output, width)?;
            Some([w, h])
        }
        Format::Svg => {
            svg::inspect(&output, false)?;
            None
        }
        Format::Pdf => {
            if output.len() < 128
                || !output.starts_with(b"%PDF-1.5")
                || !output[output.len().saturating_sub(32)..]
                    .windows(5)
                    .any(|w| w == b"%%EOF")
            {
                bail!("Inkscape output lacks a bounded PDF 1.5 envelope");
            }
            // Reopen the exported bytes with the bundled PDF importer. The
            // envelope alone cannot distinguish a usable PDF from truncated junk.
            let preview = workspace.join("pdf-check.png");
            process::run(
                spec,
                &[
                    "--pdf-poppler".into(),
                    "--export-type=png".into(),
                    "--export-width=128".into(),
                    format!("--export-filename={}", preview.display()),
                    staged.to_string_lossy().into_owned(),
                ],
                workspace,
                Some(&preview),
                control,
            )
            .await
            .context("reopen generated PDF for validation")?;
            process::png(&process::output(&preview)?, Some(128))
                .context("generated PDF did not produce a usable preview")?;
            None
        }
    };
    control.check()?;
    let artifact = destination.publish(&output, args.format.media())?;
    Ok(EditorResult {
        operation: "svg_export".into(),
        editable_source: Some(args.source),
        source_sha256: Some(crate::util::sha256(&bytes)),
        artifact: Some(artifact),
        pixel_dimensions: dimensions,
        ..Default::default()
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, fs, time::Duration};

    #[tokio::test]
    async fn export_refuses_case_and_file_aliases_before_launch_or_publication() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().canonicalize().unwrap();
        let workspace = tempfile::tempdir().unwrap();
        let original = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 80"><rect id="mark" width="100" height="80"/></svg>"#;
        fs::write(root.join("source.svg"), original).unwrap();
        fs::hard_link(root.join("source.svg"), root.join("alias.svg")).unwrap();
        let spec = LaunchSpec {
            container: None,
            command: root.join("must-not-launch"),
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: root.clone(),
            allowed_tools: None,
            denied_tools: Default::default(),
            input_policy: "none".into(),
            startup_timeout: Duration::from_secs(1),
        };
        let (_stop, shutdown) = tokio::sync::watch::channel(false);
        let control = Control {
            request: None,
            shutdown,
            timeout: Duration::from_secs(1),
        };
        for output in ["Source.svg", "alias.svg"] {
            let result = export(
                &spec,
                Export {
                    source: "source.svg".into(),
                    output: output.into(),
                    format: Format::Svg,
                    width: None,
                    object_id: None,
                    overwrite: true,
                    expected_sha256: None,
                },
                workspace.path(),
                &control,
            )
            .await;
            let error = result.err().expect("source alias must fail").to_string();
            assert!(error.contains("editable input"), "{error}");
            assert_eq!(fs::read(root.join("source.svg")).unwrap(), original);
            assert!(
                !workspace.path().join("input.svg").exists(),
                "editor must not be invoked for a source alias"
            );
        }
    }
}
