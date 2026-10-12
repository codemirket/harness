//! Shared scoped publication boundary for Mirket-owned browser design adapters.
use super::{EditorResult, files, process};
use crate::{package_mcp::LaunchSpec, util};
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::{fs, path::Path};
pub(super) fn key(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && value.len() <= 64
            && value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'),
        "keys must contain 1..64 letters, digits, underscores or hyphens"
    );
    Ok(())
}
pub(super) fn remove_nulls(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|_, v| !v.is_null());
            for v in map.values_mut() {
                remove_nulls(v);
            }
        }
        Value::Array(a) => {
            for v in a {
                remove_nulls(v);
            }
        }
        _ => {}
    }
}
pub(super) async fn run(
    spec: &LaunchSpec,
    mut job: Value,
    source: Option<(&str, &str, u64)>,
    destination: Option<(&str, &str, bool, Option<&str>)>,
    workspace: &Path,
    control: &process::Control<'_>,
    engine: &str,
) -> Result<EditorResult> {
    let source_bytes = source
        .map(|(name, staged, max)| {
            let b = files::read(&spec.cwd, name, &["json"], max)?;
            fs::write(workspace.join(staged), &b)?;
            job["source"] = true.into();
            Ok::<_, anyhow::Error>(b)
        })
        .transpose()?;
    let target = destination
        .map(|(name, ext, overwrite, expected)| {
            if let Some((input, _, _)) = source {
                files::distinct_output(&spec.cwd, name, [input])?;
            }
            files::Destination::new(
                &spec.cwd,
                name,
                &[if ext == "penpot.json" { "json" } else { ext }],
                overwrite,
                expected,
            )
        })
        .transpose()?;
    remove_nulls(&mut job);
    util::atomic_write(&workspace.join("job.json"), &serde_json::to_vec(&job)?)?;
    let output = destination.map(|(_, ext, _, _)| workspace.join(format!("output.{ext}")));
    let report = process::run(
        spec,
        &["job.json".into()],
        workspace,
        output.as_deref(),
        control,
    )
    .await?;
    let metadata: Value =
        serde_json::from_slice(&report).context("design runner returned no valid result")?;
    ensure!(
        metadata["engine"] == engine && metadata["operation"] == job["operation"],
        "design result identity mismatch"
    );
    let operation = job["operation"].as_str().context("operation")?.to_owned();
    let artifact = if let (Some(target), Some(output)) = (target, output) {
        let bytes = util::read_bounded(&output, files::MAX_OUTPUT_BYTES)?;
        ensure!(
            !bytes.is_empty(),
            "design runner produced an empty artifact"
        );
        let ext = destination.context("destination")?.1;
        if ext == "png" {
            process::png(&bytes, None)?;
        } else if ext == "svg" {
            validate_svg(&bytes)?;
        } else if ext == "penpot" {
            validate_archive(&bytes, &metadata["inspection"])?;
        } else if ext == "penpot.json" {
            let _: Value = serde_json::from_slice(&bytes)?;
        }
        control.check()?;
        Some(target.publish(
            &bytes,
            match ext {
                "png" => "image/png",
                "svg" => "image/svg+xml",
                "penpot" => "application/zip",
                _ => "application/json",
            },
        )?)
    } else {
        None
    };
    Ok(EditorResult {
        operation,
        editable_source: destination
            .filter(|d| d.1 == "penpot.json")
            .map(|d| d.0.to_owned())
            .or_else(|| source.map(|s| s.0.to_owned())),
        source_sha256: source_bytes.as_ref().map(|b| util::sha256(b)),
        artifact,
        design: metadata.get("inspection").cloned(),
        notes: metadata
            .get("notes")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        ..Default::default()
    })
}
fn validate_svg(bytes: &[u8]) -> Result<()> {
    let text = std::str::from_utf8(bytes)?;
    let doc = roxmltree::Document::parse_with_options(
        text,
        roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: 16384,
            ..Default::default()
        },
    )?;
    ensure!(
        doc.root_element().tag_name().name() == "svg",
        "export is not SVG"
    );
    for node in doc.descendants() {
        ensure!(!node.is_pi(), "SVG processing instructions are refused");
        if !node.is_element() {
            continue;
        }
        ensure!(
            node.tag_name().namespace() == Some("http://www.w3.org/2000/svg"),
            "foreign SVG namespace is refused"
        );
        ensure!(
            matches!(
                node.tag_name().name(),
                "svg"
                    | "g"
                    | "defs"
                    | "clipPath"
                    | "pattern"
                    | "rect"
                    | "ellipse"
                    | "path"
                    | "text"
                    | "tspan"
                    | "style"
                    | "linearGradient"
                    | "radialGradient"
                    | "stop"
            ),
            "unsupported active or linked SVG element"
        );
        for attr in node.attributes() {
            ensure!(
                !attr.name().to_ascii_lowercase().starts_with("on"),
                "SVG event handlers are refused"
            );
            ensure!(
                attr.name() != "href" && !attr.value().contains(['\\', '<', '>']),
                "SVG linked resources or escapes are refused"
            );
            if attr.value().contains("url(") {
                ensure!(
                    attr.value().starts_with("url(#")
                        && attr.value().ends_with(')')
                        && !attr.value().contains(['\\', '"', '\'']),
                    "SVG paint may reference only internal objects"
                );
            }
        }
        if node.tag_name().name() == "style" {
            let css = node.text().unwrap_or("");
            ensure!(
                !css.contains(['\\', '<', '>']) && !css.contains("@import"),
                "active SVG style syntax is refused"
            );
            for block in css.split('}').filter(|b| !b.trim().is_empty()) {
                let (head, body) = block.split_once('{').context("malformed font style")?;
                ensure!(
                    head.trim() == "@font-face",
                    "only embedded font faces are supported in SVG styles"
                );
                for statement in body.split(';').filter(|b| !b.trim().is_empty()) {
                    // A data URI contains one semicolon; src is checked below as a complete declaration.
                    if statement.trim().starts_with("base64,") {
                        continue;
                    }
                    let (name, value) = statement
                        .split_once(':')
                        .context("malformed font declaration")?;
                    let value = value.trim();
                    match name.trim() {
                        "font-family" => ensure!(
                            value.len() <= 128
                                && value
                                    .bytes()
                                    .all(|b| b.is_ascii_alphanumeric() || b"'\" -_".contains(&b)),
                            "unsafe font family"
                        ),
                        "font-style" => ensure!(
                            matches!(value, "normal" | "italic"),
                            "unsupported font style"
                        ),
                        "font-weight" => ensure!(
                            value.parse::<u16>().is_ok_and(|v| (100..=900).contains(&v)),
                            "unsupported font weight"
                        ),
                        "font-display" => ensure!(value == "block", "unsupported font display"),
                        "src" => ensure!(
                            value == "url(data:font/woff" || value == "url(data:font/woff2",
                            "font must be embedded"
                        ),
                        _ => anyhow::bail!("unsupported SVG style declaration"),
                    }
                }
                let src = body
                    .split("src:")
                    .nth(1)
                    .context("font face has no source")?
                    .trim();
                let rest = src
                    .strip_prefix("url(data:font/woff;base64,")
                    .or_else(|| src.strip_prefix("url(data:font/woff2;base64,"))
                    .context("font must use embedded WOFF")?;
                let (encoded, suffix) = rest.split_once(')').context("malformed embedded font")?;
                ensure!(
                    !encoded.is_empty()
                        && encoded.len() <= 4 * 1024 * 1024
                        && encoded
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b"+/=".contains(&b)),
                    "malformed embedded font bytes"
                );
                ensure!(
                    matches!(suffix.trim(), "format('woff');" | "format('woff2');"),
                    "unsupported font source format"
                );
            }
        }
    }
    Ok(())
}
fn validate_archive(bytes: &[u8], inspection: &Value) -> Result<()> {
    use std::io::{Cursor, Read};
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).context("invalid native Penpot archive")?;
    ensure!(
        archive.len() <= 1024,
        "native Penpot archive has too many entries"
    );
    let mut names = std::collections::BTreeSet::new();
    let mut total = 0u64;
    let mut manifest = None;
    let mut file_record = None;
    let mut pages = 0;
    let mut shapes = std::collections::BTreeMap::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        ensure!(
            entry.enclosed_name().is_some()
                && !entry.name().contains('\\')
                && names.insert(entry.name().to_owned()),
            "unsafe native Penpot archive entry"
        );
        ensure!(
            entry.is_file() && entry.size() <= 2 * 1024 * 1024,
            "unsupported native Penpot archive entry"
        );
        total = total
            .checked_add(entry.size())
            .context("archive size overflow")?;
        ensure!(total <= 32 * 1024 * 1024, "oversized native Penpot archive");
        ensure!(
            entry.name().ends_with(".json")
                || (entry.name().starts_with("objects/") && entry.name().ends_with(".png")),
            "media is outside the typed Penpot profile"
        );
        let mut content = Vec::new();
        entry
            .by_ref()
            .take(2 * 1024 * 1024 + 1)
            .read_to_end(&mut content)?;
        ensure!(content.len() <= 2 * 1024 * 1024, "oversized Penpot entry");
        if entry.name().ends_with(".png") {
            process::png(&content, None)?;
        } else {
            let value: Value = serde_json::from_slice(&content)?;
            if entry.name() == "manifest.json" {
                manifest = Some(value);
            } else if entry.name().starts_with("files/") && entry.name().split('/').count() == 2 {
                file_record = Some(value);
            } else if entry.name().contains("/pages/") && entry.name().split('/').count() == 4 {
                pages += 1;
            } else if entry.name().contains("/pages/") && entry.name().split('/').count() == 5 {
                let id = value["id"]
                    .as_str()
                    .context("shape has no identity")?
                    .to_owned();
                shapes.insert(id, value);
            }
        }
    }
    let manifest = manifest.context("Penpot archive has no manifest")?;
    ensure!(
        manifest["version"] == 1 && manifest["type"] == "penpot/export-files",
        "unsupported Penpot native manifest"
    );
    ensure!(
        manifest["files"].as_array().is_some_and(|v| v.len() == 1),
        "expected one exported editable Penpot file"
    );
    let expected = inspection["nodes"]
        .as_array()
        .context("native export has no observed design")?;
    let file = manifest["files"][0]["id"]
        .as_str()
        .context("native file identity absent")?;
    ensure!(
        Some(file) == inspection["file_id"].as_str()
            && file_record.is_some_and(|f| f["id"] == file),
        "native export file identity differs"
    );
    ensure!(
        pages == 1 && !expected.is_empty() && shapes.len() == expected.len() + 1,
        "native archive has missing pages or editable shapes"
    );
    for node in expected {
        let id = node["id"]
            .as_str()
            .context("inspected shape has no identity")?;
        let saved = shapes
            .get(id)
            .context("native archive lost an editable shape")?;
        validate_native_shape(saved, node)?;
    }
    Ok(())
}

fn validate_native_shape(saved: &Value, observed: &Value) -> Result<()> {
    let expected_type = match observed["type"].as_str() {
        Some("board") => "frame",
        Some("rectangle") => "rect",
        Some("ellipse") => "circle",
        Some("path") => "path",
        Some("text") => "text",
        _ => anyhow::bail!("native archive inspection contains an unsupported shape"),
    };
    ensure!(
        saved["type"] == expected_type && saved["name"] == observed["name"],
        "native archive lost object type or name"
    );
    let points = saved["points"]
        .as_array()
        .context("native shape has no bounds points")?;
    ensure!(
        points.len() == 4,
        "native shape requires four bounds points"
    );
    // Penpot's current Plugin API derives x/y from points and dimensions from
    // selrect. Paths have no top-level geometry, and rotated bounds differ from x/y.
    for field in ["x", "y", "width", "height"] {
        let actual = if matches!(field, "x" | "y") {
            let mut minimum = f64::INFINITY;
            for point in points {
                let coordinate = point[field]
                    .as_f64()
                    .context("native bounds coordinate absent")?;
                ensure!(coordinate.is_finite(), "nonfinite native bounds coordinate");
                minimum = minimum.min(coordinate);
            }
            minimum
        } else {
            saved["selrect"][field]
                .as_f64()
                .context("native selection dimensions absent")?
        };
        let expected = observed[field]
            .as_f64()
            .context("inspection shape geometry absent")?;
        ensure!(
            actual.is_finite() && expected.is_finite() && (actual - expected).abs() <= 0.1,
            "native archive geometry differs from inspection"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn svg_export_rejects_active_content_and_font_links() {
        let valid=br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><defs><style>@font-face {font-family:'sourcesanspro';font-style:normal;font-weight:400;font-display:block;src: url(data:font/woff;base64,d09GRg==) format('woff');}</style></defs><rect width="10" height="10" fill="#102b32"/></svg>"##;
        assert!(validate_svg(valid).is_ok());
        for invalid in [
            r#"<svg><rect/></svg>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><script/></svg>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><rect onclick="alert(1)"/></svg>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><image href="file:///etc/passwd"/></svg>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><style>@font-face {src:url(http://127.0.0.1/font.woff)}</style></svg>"#,
            r#"<!DOCTYPE svg [<!ENTITY x SYSTEM "file:///etc/passwd">]><svg xmlns="http://www.w3.org/2000/svg">&x;</svg>"#,
        ] {
            assert!(validate_svg(invalid.as_bytes()).is_err(), "{invalid}");
        }
    }
    #[test]
    fn opaque_or_empty_native_export_cannot_succeed() {
        assert!(validate_archive(b"success", &serde_json::json!({})).is_err());
        let mut bytes = std::io::Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(&mut bytes);
        zip.start_file("manifest.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        use std::io::Write;
        zip.write_all(br#"{"version":1,"type":"penpot/export-files","files":[{"id":"fake"}]}"#)
            .unwrap();
        zip.finish().unwrap();
        assert!(
            validate_archive(
                bytes.get_ref(),
                &serde_json::json!({"file_id":"fake","nodes":[]})
            )
            .is_err()
        );
    }

    #[test]
    fn native_path_and_rotated_bounds_use_current_selection_geometry() {
        let mut saved = serde_json::json!({
            "type":"path","name":"mountain",
            "selrect":{"x":512,"y":260,"width":340,"height":180},
            "points":[{"x":512,"y":260},{"x":852,"y":260},{"x":852,"y":440},{"x":512,"y":440}]
        });
        let observed = serde_json::json!({"type":"path","name":"mountain","x":512,"y":260,"width":340,"height":180});
        assert!(validate_native_shape(&saved, &observed).is_ok());
        saved["selrect"]["width"] = serde_json::json!(341);
        assert!(validate_native_shape(&saved, &observed).is_err());
        saved["points"] = serde_json::json!([]);
        assert!(validate_native_shape(&saved, &observed).is_err());

        let rotated = serde_json::json!({"type":"rect","name":"label","x":30,"y":20,"rotation":30,
            "selrect":{"width":100,"height":60},
            "points":[{"x":51.69873,"y":-0.98076},{"x":138.30127,"y":49.01924},{"x":108.30127,"y":100.98076},{"x":21.69873,"y":50.98076}]});
        let observed = serde_json::json!({"type":"rectangle","name":"label","x":21.69873,"y":-0.98076,"width":100,"height":60});
        assert!(validate_native_shape(&rotated, &observed).is_ok());
    }
}
