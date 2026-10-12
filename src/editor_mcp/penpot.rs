//! Scoped handles and typed operations in a live local Penpot workspace.
use super::{EditorResult, design_job, process};
use crate::{package_mcp::LaunchSpec, util};
use anyhow::{Result, bail, ensure};
use rmcp::{
    model::Tool,
    schemars::{self, JsonSchema},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path};
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Board,
    Rectangle,
    Ellipse,
    Path,
    Text,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Fill {
    #[serde(rename = "fillColor")]
    pub color: String,
    #[serde(rename = "fillOpacity")]
    pub opacity: f64,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Row,
    Column,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub direction: Direction,
    pub gap: f64,
    pub padding: f64,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub key: String,
    pub kind: Kind,
    pub parent: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub text: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<u16>,
    pub path: Option<String>,
    pub fills: Option<Vec<Fill>>,
    pub opacity: Option<f64>,
    pub rotation: Option<f64>,
    pub corner_radius: Option<f64>,
    pub layout: Option<Layout>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub name: String,
    pub nodes: Vec<Node>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Create {
    pub document: Document,
    pub output: String,
    #[serde(default)]
    pub overwrite: bool,
    pub expected_sha256: Option<String>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Inspect {
    pub source: String,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Patch {
    pub key: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub opacity: Option<f64>,
    pub rotation: Option<f64>,
    pub text: Option<String>,
    pub path: Option<String>,
    pub fills: Option<Vec<Fill>>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Update {
    pub source: String,
    pub expected_revision: u64,
    pub patches: Vec<Patch>,
    pub output: String,
    #[serde(default)]
    pub overwrite: bool,
    pub expected_sha256: Option<String>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Png,
    Svg,
    Penpot,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Export {
    pub source: String,
    pub output: String,
    pub format: Format,
    pub object_key: Option<String>,
    pub scale: Option<f64>,
    #[serde(default)]
    pub overwrite: bool,
    pub expected_sha256: Option<String>,
}
pub fn definitions() -> Vec<Tool> {
    vec![
        super::definition::<Create>(
            "penpot_create",
            "Create a real editable Penpot design in Mirket’s local service through an active headless Chromium workspace and registered, permission-checked fixed plugin. Typed boards, shapes, paths, local text and flex layouts; 256 nodes. Project-relative output ends .penpot.json and identifies the native service document. Account and file ownership are isolated by canonical project. Reopens persisted geometry before publication. Podman service must be explicitly started through Mirket; no raw plugin scripts or API URLs.",
            false,
        ),
        super::definition::<Inspect>(
            "penpot_inspect",
            "Open a project-scoped .penpot.json handle and inspect actual persisted shapes, keys, geometry, text and current revision in the local Penpot workspace. The handle is not a standalone native file; service data must remain available. Use penpot_export format penpot for a portable editable archive.",
            true,
        ),
        super::definition::<Update>(
            "penpot_update",
            "Clone a project Penpot document, apply typed patches by stable inspected keys, and save a distinct scoped handle. Require expected_revision from current inspection; preserve original service design. Verify backend persistence and reopened geometry. Existing output requires explicit overwrite; cancellation can leave an unreferenced clone in the local service.",
            false,
        ),
        super::definition::<Export>(
            "penpot_export",
            "Export a distinct PNG/SVG for an inspected object_key, or a portable editable .penpot archive for the complete service document. Actual Plugin API PNG export, native Penpot SVG renderer, and current backend native archive export; no reconstructed shapes. SVG embeds bundled fonts and refuses other linked resources. PNG/SVG scale 0.25..2 and at most 2048 pixels per side. Scoped project outputs and overwrite intent; source remains editable. Requires running local Podman service and headless browser.",
            false,
        ),
    ]
}
fn number(v: f64, limit: f64) -> Result<()> {
    ensure!(
        v.is_finite() && v.abs() <= limit,
        "geometry must be finite and within {limit}"
    );
    Ok(())
}
fn fill(v: &Option<Vec<Fill>>) -> Result<()> {
    if let Some(fills) = v {
        ensure!(
            !fills.is_empty() && fills.len() <= 4,
            "use 1..4 solid fills"
        );
        for f in fills {
            ensure!(
                f.color.len() == 7
                    && f.color.starts_with('#')
                    && f.color.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit),
                "use six-digit hex colors"
            );
            ensure!(
                (0.0..=1.0).contains(&f.opacity),
                "fill opacity must be 0..1"
            );
        }
    }
    Ok(())
}
fn path(value: &Option<String>) -> Result<()> {
    if let Some(value) = value {
        ensure!(
            !value.is_empty() && value.len() <= 32768 && value.trim_end().ends_with(['Z', 'z']),
            "use a bounded closed SVG path"
        );
        let bytes = value.as_bytes();
        let mut cursor = 0;
        while cursor < bytes.len() {
            let byte = bytes[cursor];
            if byte.is_ascii_whitespace() || byte == b',' || b"mMlLhHvVcCsSqQtTaAzZ".contains(&byte)
            {
                cursor += 1;
                continue;
            }
            let start = cursor;
            if bytes.get(cursor).is_some_and(|b| b"+-".contains(b)) {
                cursor += 1;
            }
            let mut digits = 0;
            while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
                cursor += 1;
                digits += 1;
            }
            if bytes.get(cursor) == Some(&b'.') {
                cursor += 1;
                while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
                    cursor += 1;
                    digits += 1;
                }
            }
            ensure!(digits > 0, "invalid SVG path token");
            if bytes.get(cursor).is_some_and(|b| b"eE".contains(b)) {
                cursor += 1;
                if bytes.get(cursor).is_some_and(|b| b"+-".contains(b)) {
                    cursor += 1;
                }
                let exponent = cursor;
                while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
                    cursor += 1;
                }
                ensure!(cursor > exponent, "invalid SVG numeric exponent");
            }
            let number = value[start..cursor].parse::<f64>()?;
            ensure!(
                number.is_finite() && number.abs() <= 8192.0,
                "SVG path numbers must be finite and within 8192"
            );
        }
    }
    Ok(())
}
fn validate(d: &Document) -> Result<()> {
    ensure!(
        !d.name.is_empty() && d.name.len() <= 250,
        "design name must have 1..250 bytes"
    );
    ensure!(
        !d.nodes.is_empty() && d.nodes.len() <= 256,
        "use 1..256 nodes"
    );
    let mut keys = BTreeSet::new();
    let mut boards = BTreeSet::new();
    for n in &d.nodes {
        design_job::key(&n.key)?;
        ensure!(!keys.contains(&n.key), "duplicate node key");
        if let Some(parent) = &n.parent {
            ensure!(boards.contains(parent), "parent must be an earlier board");
        }
        number(n.x, 8192.0)?;
        number(n.y, 8192.0)?;
        for v in [n.width, n.height] {
            ensure!((1.0..=2048.0).contains(&v), "dimensions must be 1..2048");
        }
        fill(&n.fills)?;
        path(&n.path)?;
        if matches!(n.kind, Kind::Path) {
            ensure!(n.path.is_some(), "path shape requires SVG path data");
        } else {
            ensure!(n.path.is_none(), "path applies only to path shapes");
        }
        if matches!(n.kind, Kind::Text) {
            ensure!(
                n.text
                    .as_ref()
                    .is_some_and(|s| !s.is_empty() && s.len() <= 8192),
                "text requires 1..8192 bytes"
            );
        } else {
            ensure!(
                n.text.is_none() && n.font_size.is_none() && n.font_weight.is_none(),
                "text fields require text shape"
            );
        }
        ensure!(
            n.font_size.is_none_or(|v| (8.0..=256.0).contains(&v)),
            "font size must be 8..256"
        );
        ensure!(
            n.font_weight.is_none_or(|v| [400, 600, 700].contains(&v)),
            "font weight must be 400,600 or700"
        );
        ensure!(
            n.opacity.is_none_or(|v| (0.0..=1.0).contains(&v)),
            "opacity 0..1"
        );
        if let Some(v) = n.rotation {
            number(v, 360.0)?;
        }
        ensure!(
            n.corner_radius
                .is_none_or(|v| (0.0..=256.0).contains(&v) && matches!(n.kind, Kind::Rectangle)),
            "corner radius requires rectangle and 0..256"
        );
        if let Some(l) = &n.layout {
            ensure!(matches!(n.kind, Kind::Board), "layout requires board");
            ensure!(
                (0.0..=512.0).contains(&l.gap) && (0.0..=512.0).contains(&l.padding),
                "layout gap/padding 0..512"
            );
        }
        if matches!(n.kind, Kind::Board) {
            boards.insert(n.key.clone());
        }
        keys.insert(n.key.clone());
    }
    Ok(())
}
fn scope(spec: &LaunchSpec) -> Result<String> {
    Ok(util::sha256(
        spec.cwd.canonicalize()?.as_os_str().as_encoded_bytes(),
    ))
}
fn handle(name: &str) -> Result<()> {
    ensure!(
        name.ends_with(".penpot.json"),
        "Penpot handles must end .penpot.json"
    );
    Ok(())
}
pub async fn invoke(
    spec: &LaunchSpec,
    tool: &str,
    value: Value,
    workspace: &Path,
    control: &process::Control<'_>,
) -> Result<EditorResult> {
    let project_scope = scope(spec)?;
    match tool {
        "penpot_create" => {
            let v: Create = serde_json::from_value(value)?;
            validate(&v.document)?;
            handle(&v.output)?;
            design_job::run(
                spec,
                json!({"operation":"create","document":v.document,"project_scope":project_scope}),
                None,
                Some((
                    &v.output,
                    "penpot.json",
                    v.overwrite,
                    v.expected_sha256.as_deref(),
                )),
                workspace,
                control,
                "penpot",
            )
            .await
        }
        "penpot_inspect" => {
            let v: Inspect = serde_json::from_value(value)?;
            handle(&v.source)?;
            design_job::run(
                spec,
                json!({"operation":"inspect","project_scope":project_scope}),
                Some((&v.source, "input.penpot.json", 16384)),
                None,
                workspace,
                control,
                "penpot",
            )
            .await
        }
        "penpot_update" => {
            let v: Update = serde_json::from_value(value)?;
            handle(&v.source)?;
            handle(&v.output)?;
            ensure!(
                !v.patches.is_empty() && v.patches.len() <= 256,
                "use 1..256 patches"
            );
            let mut keys = BTreeSet::new();
            for p in &v.patches {
                ensure!(keys.insert(p.key.clone()), "duplicate patch key");
                ensure!(
                    p.x.is_some()
                        || p.y.is_some()
                        || p.width.is_some()
                        || p.height.is_some()
                        || p.opacity.is_some()
                        || p.rotation.is_some()
                        || p.text.is_some()
                        || p.path.is_some()
                        || p.fills.is_some(),
                    "patch has no change"
                );
                design_job::key(&p.key)?;
                fill(&p.fills)?;
                path(&p.path)?;
                for n in [p.x, p.y].into_iter().flatten() {
                    number(n, 8192.0)?;
                }
                if let Some(rotation) = p.rotation {
                    number(rotation, 360.0)?;
                }
                for n in [p.width, p.height].into_iter().flatten() {
                    ensure!((1.0..=2048.0).contains(&n), "dimensions must be 1..2048");
                }
                ensure!(
                    p.opacity.is_none_or(|v| (0.0..=1.0).contains(&v)),
                    "opacity 0..1"
                );
                ensure!(
                    p.text
                        .as_ref()
                        .is_none_or(|v| !v.is_empty() && v.len() <= 8192),
                    "text must be 1..8192 bytes"
                );
            }
            design_job::run(spec,json!({"operation":"update","patches":v.patches,"expected_revision":v.expected_revision,"project_scope":project_scope}),Some((&v.source,"input.penpot.json",16384)),Some((&v.output,"penpot.json",v.overwrite,v.expected_sha256.as_deref())),workspace,control,"penpot").await
        }
        "penpot_export" => {
            let v: Export = serde_json::from_value(value)?;
            handle(&v.source)?;
            let ext = match v.format {
                Format::Png => "png",
                Format::Svg => "svg",
                Format::Penpot => "penpot",
            };
            if !matches!(v.format, Format::Penpot) {
                design_job::key(v.object_key.as_deref().ok_or_else(|| {
                    anyhow::anyhow!("PNG/SVG export requires inspected object_key")
                })?)?;
            }
            ensure!(
                v.scale.is_none_or(|s| (0.25..=2.0).contains(&s)),
                "scale must be 0.25..2"
            );
            design_job::run(spec,json!({"operation":"export","format":v.format,"object_key":v.object_key,"scale":v.scale,"project_scope":project_scope}),Some((&v.source,"input.penpot.json",16384)),Some((&v.output,ext,v.overwrite,v.expected_sha256.as_deref())),workspace,control,"penpot").await
        }
        _ => bail!("unknown Penpot tool"),
    }
}
pub async fn probe(
    spec: &LaunchSpec,
    workspace: &Path,
    control: &process::Control<'_>,
) -> Result<()> {
    let _ = design_job::run(
        spec,
        json!({"operation":"probe"}),
        None,
        None,
        workspace,
        control,
        "penpot",
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn penpot_profile_refuses_scripts_invalid_paths_and_unbounded_geometry() {
        assert!(serde_json::from_value::<Create>(json!({"document":{"name":"a","nodes":[]},"output":"x.penpot.json","script":"penpot.createBoard()"})).is_err());
        assert!(handle("x.json").is_err());
        assert!(path(&Some("<script/>".into())).is_err());
        for invalid in [
            "M1e999 0L0 0L1 1Z",
            "M9000 0L0 0L1 1Z",
            "MNaN 0L0 0L1 1Z",
            "M1e 0L0 0L1 1Z",
        ] {
            assert!(path(&Some(invalid.into())).is_err());
        }
        assert!(path(&Some("M1e2 0L0 0L1 1Z".into())).is_ok());
        let document:Document=serde_json::from_value(json!({"name":"poster","nodes":[{"key":"poster","kind":"board","x":0,"y":0,"width":9000,"height":200}]})).unwrap();
        assert!(validate(&document).is_err());
    }
}
