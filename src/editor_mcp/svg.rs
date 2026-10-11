//! A deliberately limited vector profile, validated before either editor sees it.
use super::files::MAX_SVG_BYTES;
use anyhow::{Context, Result, bail};
use rmcp::schemars;
use roxmltree::{Document, ParsingOptions};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

const SVG_NS: &str = "http://www.w3.org/2000/svg";
const XLINK_NS: &str = "http://www.w3.org/1999/xlink";
const INKSCAPE_NS: &str = "http://www.inkscape.org/namespaces/inkscape";
const SODIPODI_NS: &str = "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd";
pub const MAX_NODES: u32 = 10_000;

#[derive(Debug, Serialize, rmcp::schemars::JsonSchema)]
pub struct Inspection {
    pub width: f64,
    pub height: f64,
    pub view_box: [f64; 4],
    pub elements: BTreeMap<String, usize>,
    pub objects: Vec<Object>,
    pub animations: usize,
    pub font_families: Vec<String>,
}
#[derive(Debug, Serialize, rmcp::schemars::JsonSchema)]
pub struct Object {
    pub id: String,
    pub element: String,
}

pub fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.as_bytes()[0].is_ascii_alphabetic()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.')
}

fn numbers(value: &str) -> Result<Vec<f64>> {
    value
        .split(|c: char| c.is_ascii_whitespace() || c == ',')
        .filter(|v| !v.is_empty())
        .map(|n| {
            let n: f64 = n.parse().context("expected finite SVG numbers")?;
            if !n.is_finite() || n.abs() > 1_000_000.0 {
                bail!("SVG number is non-finite or out of range");
            }
            Ok(n)
        })
        .collect()
}
fn dimension(value: Option<&str>, fallback: f64) -> Result<f64> {
    let size = match value {
        None => fallback,
        Some(s) => s
            .trim()
            .strip_suffix("px")
            .unwrap_or(s.trim())
            .parse::<f64>()
            .context("use explicit SVG dimensions in pixels (px or unitless)")?,
    };
    if !size.is_finite() || !(1.0..=8192.0).contains(&size) {
        bail!("SVG dimensions must be 1..8192 pixels");
    }
    Ok(size)
}

fn property(
    name: &str,
    value: &str,
    refs: &mut BTreeSet<String>,
    fonts: &mut BTreeSet<String>,
) -> Result<()> {
    if !matches!(
        name,
        "fill"
            | "fill-rule"
            | "fill-opacity"
            | "stroke"
            | "stroke-width"
            | "stroke-opacity"
            | "stroke-linecap"
            | "stroke-linejoin"
            | "stroke-miterlimit"
            | "stroke-dasharray"
            | "stroke-dashoffset"
            | "opacity"
            | "stop-color"
            | "stop-opacity"
            | "color"
            | "font-family"
            | "font-size"
            | "font-weight"
            | "font-style"
            | "text-anchor"
            | "dominant-baseline"
            | "letter-spacing"
            | "paint-order"
            | "clip-path"
            | "clip-rule"
            | "display"
            | "visibility"
            | "vector-effect"
    ) {
        bail!("unsupported SVG style property {name}; use the documented static vector profile");
    }
    if name == "font-family" {
        fonts.insert(value.into());
    }
    if value.contains(['\\', '@', '<', '>', '&', '\n', '\r']) {
        bail!("unsupported escape or active syntax in SVG style {name}");
    }
    if value.contains('(') || value.contains(')') {
        if let Some(id) = value
            .strip_prefix("url(#")
            .and_then(|v| v.strip_suffix(')'))
            .filter(|id| identifier(id))
        {
            if !matches!(name, "fill" | "stroke" | "clip-path") {
                bail!("internal references are unsupported for {name}");
            }
            refs.insert(id.into());
        } else if (value.starts_with("rgb(") || value.starts_with("rgba("))
            && value.ends_with(')')
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || b"rgba(.,% )".contains(&b))
        {
            // A bounded literal color, never a CSS function that can fetch content.
        } else {
            bail!("only literal colors or internal url(#id) paint references are supported");
        }
    }
    Ok(())
}

pub fn inspect(bytes: &[u8], animated: bool) -> Result<Inspection> {
    if bytes.is_empty() || bytes.len() as u64 > MAX_SVG_BYTES {
        bail!("SVG must contain 1..2 MiB of UTF-8 XML");
    }
    let text = std::str::from_utf8(bytes).context("SVG must be UTF-8")?;
    if text.contains("<!DOCTYPE") || text.contains("<!ENTITY") {
        bail!("SVG DTDs and entities are not supported");
    }
    let doc = Document::parse_with_options(
        text,
        ParsingOptions {
            allow_dtd: false,
            nodes_limit: MAX_NODES,
            ..Default::default()
        },
    )
    .context("malformed SVG XML")?;
    let root = doc.root_element();
    if root.tag_name().namespace() != Some(SVG_NS) || root.tag_name().name() != "svg" {
        bail!("root must be svg in the SVG namespace");
    }
    let v = numbers(
        root.attribute("viewBox")
            .context("SVG needs an explicit viewBox")?,
    )?;
    if v.len() != 4 || v[2] <= 0.0 || v[3] <= 0.0 {
        bail!("viewBox needs four finite numbers and positive dimensions");
    }
    let mut result = Inspection {
        width: dimension(root.attribute("width"), v[2])?,
        height: dimension(root.attribute("height"), v[3])?,
        view_box: [v[0], v[1], v[2], v[3]],
        elements: BTreeMap::new(),
        objects: vec![],
        animations: 0,
        font_families: vec![],
    };
    let mut ids = BTreeMap::new();
    let mut refs = BTreeSet::new();
    let mut fonts = BTreeSet::new();
    let mut drawable = 0;
    for node in doc.descendants() {
        if node.is_pi() {
            bail!("SVG processing instructions are refused");
        }
        if !node.is_element() {
            continue;
        }
        if node.ancestors().count() > 64 {
            bail!("SVG nesting exceeds 64 levels");
        }
        let name = node.tag_name().name();
        if node.tag_name().namespace() != Some(SVG_NS)
            || !matches!(
                name,
                "svg"
                    | "g"
                    | "defs"
                    | "title"
                    | "desc"
                    | "path"
                    | "rect"
                    | "circle"
                    | "ellipse"
                    | "line"
                    | "polyline"
                    | "polygon"
                    | "text"
                    | "tspan"
                    | "linearGradient"
                    | "radialGradient"
                    | "stop"
                    | "clipPath"
                    | "animate"
                    | "animateTransform"
            )
        {
            bail!(
                "unsupported SVG element {name}; scripts, CSS sheets, images, filters, use, foreign content and linked resources are outside this vector profile"
            );
        }
        if name == "svg" && node != root {
            bail!("nested SVG viewports are unsupported");
        }
        *result.elements.entry(name.into()).or_default() += 1;
        if matches!(
            name,
            "path" | "rect" | "circle" | "ellipse" | "line" | "polyline" | "polygon" | "text"
        ) {
            drawable += 1;
        }
        if let Some(id) = node.attribute("id") {
            if !identifier(id) || ids.insert(id.to_owned(), node).is_some() {
                bail!("SVG IDs must be unique simple identifiers up to 128 bytes");
            }
            result.objects.push(Object {
                id: id.into(),
                element: name.into(),
            });
        }
        let animation = matches!(name, "animate" | "animateTransform");
        if animation {
            if !animated {
                bail!(
                    "animated SVG requires the animation workflow; static export would lose motion"
                );
            }
            validate_animation(node)?;
            result.animations += 1;
            if result.animations > 128 {
                bail!("SVG exceeds 128 animation tracks");
            }
        }
        if node.attribute("href").is_some() && node.attribute((XLINK_NS, "href")).is_some() {
            bail!("use only one href spelling for gradient inheritance");
        }
        for attr in node.attributes() {
            let key = attr.name();
            let value = attr.value().trim();
            if value.len() > 65_536 {
                bail!("SVG attribute exceeds 64 KiB");
            }
            if let Some(ns) = attr.namespace() {
                if ns == INKSCAPE_NS && matches!(key, "label" | "groupmode") {
                    continue;
                }
                if ns == SODIPODI_NS && matches!(key, "role" | "nodetypes") {
                    continue;
                }
                if ns == "http://www.w3.org/XML/1998/namespace" && key == "space" {
                    continue;
                }
                if !(ns == XLINK_NS && key == "href") {
                    bail!("unsupported SVG namespaced attribute {key}");
                }
            }
            match key {
                "id"
                | "version"
                | "viewBox"
                | "width"
                | "height"
                | "x"
                | "y"
                | "x1"
                | "y1"
                | "x2"
                | "y2"
                | "cx"
                | "cy"
                | "r"
                | "rx"
                | "ry"
                | "dx"
                | "dy"
                | "fx"
                | "fy"
                | "fr"
                | "d"
                | "points"
                | "transform"
                | "gradientTransform"
                | "gradientUnits"
                | "spreadMethod"
                | "offset"
                | "clipPathUnits"
                | "preserveAspectRatio"
                | "pathLength"
                | "role"
                | "aria-label"
                | "aria-labelledby"
                | "aria-describedby"
                | "aria-hidden"
                | "focusable" => {}
                "href" => {
                    if !matches!(name, "linearGradient" | "radialGradient") {
                        bail!("href is supported only for internal gradient inheritance");
                    }
                    let id = value
                        .strip_prefix('#')
                        .filter(|id| identifier(id))
                        .context("external SVG references are refused")?;
                    refs.insert(id.into());
                }
                "style" => {
                    for declaration in value.split(';').filter(|d| !d.trim().is_empty()) {
                        let (key, value) = declaration
                            .split_once(':')
                            .context("malformed inline SVG style")?;
                        property(key.trim(), value.trim(), &mut refs, &mut fonts)?;
                    }
                }
                "attributeName" | "attributeType" | "values" | "keyTimes" | "keySplines"
                | "dur" | "begin" | "repeatCount" | "calcMode" | "additive" | "type"
                    if animation => {}
                "fill" if animation => {}
                _ => property(key, value, &mut refs, &mut fonts)?,
            }
        }
    }
    if drawable == 0 {
        bail!("SVG has no drawable vector objects");
    }
    for reference in refs {
        let target = ids
            .get(&reference)
            .context("SVG contains a missing internal paint/clip reference")?;
        if !matches!(
            target.tag_name().name(),
            "linearGradient" | "radialGradient" | "clipPath"
        ) {
            bail!("SVG reference does not identify a paint or clip definition");
        }
    }
    // Inheritance cycles can make a tiny input expensive. Clip paths may contain
    // geometry only, without recursively referencing clips or paint servers.
    for node in doc.descendants().filter(|n| n.is_element()) {
        if node
            .ancestors()
            .any(|p| p.has_tag_name((SVG_NS, "clipPath")))
            && node.attributes().any(|a| a.value().contains("url("))
        {
            bail!("clipPath content cannot reference another definition");
        }
        let mut seen = BTreeSet::new();
        let mut current = node;
        while let Some(reference) = current
            .attribute("href")
            .or_else(|| current.attribute((XLINK_NS, "href")))
        {
            let id = reference
                .strip_prefix('#')
                .context("external SVG reference")?;
            if !seen.insert(id) || seen.len() > 64 {
                bail!("cyclic SVG gradient reference or inheritance exceeds 64 levels");
            }
            current = *ids.get(id).context("missing gradient reference")?;
            if !matches!(
                current.tag_name().name(),
                "linearGradient" | "radialGradient"
            ) {
                bail!("gradient inheritance must target another gradient");
            }
        }
    }
    result.font_families = fonts.into_iter().collect();
    Ok(result)
}

fn validate_animation(node: roxmltree::Node<'_, '_>) -> Result<()> {
    let name = node
        .attribute("attributeName")
        .context("animation needs attributeName")?;
    if node.tag_name().name() == "animateTransform" {
        if name != "transform"
            || !matches!(
                node.attribute("type"),
                Some("translate" | "scale" | "rotate")
            )
        {
            bail!("only translate, scale and rotate animation are supported");
        }
    } else if name != "opacity" {
        bail!(
            "only opacity and transform animation are supported; path morphing, CSS and motion paths are unsupported"
        );
    }
    let duration = node
        .attribute("dur")
        .and_then(|v| v.strip_suffix('s'))
        .context("animation duration must use seconds")?
        .parse::<f64>()?;
    if !duration.is_finite() || !(0.05..=30.0).contains(&duration) {
        bail!("animation duration must be 0.05..30 seconds");
    }
    if !matches!(node.attribute("begin"), None | Some("0" | "0s"))
        || !matches!(
            node.attribute("repeatCount"),
            None | Some("1" | "indefinite")
        )
        || !matches!(node.attribute("fill"), None | Some("freeze" | "remove"))
        || !matches!(
            node.attribute("calcMode"),
            None | Some("linear" | "spline" | "discrete")
        )
    {
        bail!(
            "unsupported animation timing; event triggers and unbounded repeat counts are refused"
        );
    }
    let values: Vec<_> = node
        .attribute("values")
        .context("animation needs explicit values")?
        .split(';')
        .collect();
    if !(2..=64).contains(&values.len()) {
        bail!("animation must have 2..64 keyframes");
    }
    for value in &values {
        let tuple = numbers(value)?;
        let valid = match node.attribute("type") {
            Some("translate" | "scale") => (1..=2).contains(&tuple.len()),
            Some("rotate") => matches!(tuple.len(), 1 | 3),
            _ => tuple.len() == 1 && (0.0..=1.0).contains(&tuple[0]),
        };
        if !valid {
            bail!("animation keyframe has invalid value or arity");
        }
    }
    if let Some(times) = node.attribute("keyTimes") {
        let times = times
            .split(';')
            .map(str::parse::<f64>)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if times.len() != values.len()
            || times.first() != Some(&0.0)
            || times.last() != Some(&1.0)
            || times
                .iter()
                .any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
            || times.windows(2).any(|x| x[1] <= x[0])
        {
            bail!("animation keyTimes must increase from 0 to 1 and match values");
        }
    }
    if let Some(splines) = node.attribute("keySplines") {
        let splines = splines
            .split(';')
            .map(numbers)
            .collect::<Result<Vec<_>>>()?;
        if splines.len() + 1 != values.len()
            || splines
                .iter()
                .any(|s| s.len() != 4 || s.iter().any(|x| !(0.0..=1.0).contains(x)))
        {
            bail!("invalid bounded animation keySplines");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const GOOD: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 80"><defs><linearGradient id="gold"><stop stop-color="#fd8"/></linearGradient></defs><path id="wing" d="M10 10L90 40L10 70Z" fill="url(#gold)"/></svg>"##;
    #[test]
    fn dimensions_objects_and_paint_are_inspected() {
        let r = inspect(GOOD.as_bytes(), false).unwrap();
        assert_eq!((r.width, r.height), (100.0, 80.0));
        assert_eq!(r.objects.len(), 2);
    }
    #[test]
    fn references_have_bounded_depth_and_no_ambiguous_edges() {
        let chain = (0..80)
            .map(|i| format!("<linearGradient id='g{i}' href='#g{}'/>", i + 1))
            .collect::<String>();
        let source = format!(
            "<svg xmlns='{SVG_NS}' viewBox='0 0 100 100'><defs>{chain}<linearGradient id='g80'/></defs><rect width='100' height='100' fill='url(#g0)'/></svg>"
        );
        assert!(
            inspect(source.as_bytes(), false)
                .unwrap_err()
                .to_string()
                .contains("64 levels")
        );
        for definitions in [
            "<linearGradient id='a' href='#b' xlink:href='#a'/><linearGradient id='b'/>",
            "<clipPath id='a' clip-path='url(#a)'><rect width='1' height='1'/></clipPath>",
            "<linearGradient id='a' href='#b'/><clipPath id='b'/>",
        ] {
            let source = format!(
                "<svg xmlns='{SVG_NS}' xmlns:xlink='{XLINK_NS}' viewBox='0 0 100 100'><defs>{definitions}</defs><rect width='10' height='10'/></svg>"
            );
            assert!(inspect(source.as_bytes(), false).is_err());
        }
    }
    #[test]
    fn rejects_active_resources_malformed_and_unsupported_content() {
        for body in [
            "<script>alert(1)</script>",
            "<image href='file:///etc/passwd'/>",
            "<use href='#self' id='self'/>",
            "<style>@import 'https://example.org'</style>",
            "<rect onclick='run()'/>",
            "<rect fill='url(https://example.org/a)'/>",
            "<rect style='fill:u\\72l(https://example.org)'/>",
            "<animate attributeName='d' dur='1s' values='M0 0;M1 1'/>",
            "<path>",
        ] {
            let svg = format!(
                "<svg xmlns='{SVG_NS}' viewBox='0 0 100 100'><rect width='40' height='40'/>{body}</svg>"
            );
            assert!(inspect(svg.as_bytes(), true).is_err(), "{body}");
        }
        assert!(inspect(format!("<!DOCTYPE svg>{GOOD}").as_bytes(), false).is_err());
        assert!(
            inspect(
                GOOD.replace("viewBox=\"0 0 100 80\"", "viewBox=\"0 0 NaN 80\"")
                    .as_bytes(),
                false
            )
            .is_err()
        );
    }
}
