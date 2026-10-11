//! A shapes-only native animation profile. Every document is checked before opening it.
use super::{
    EditorResult,
    files::{self, Destination},
    process::{self, Control},
    svg,
};
use crate::package_mcp::LaunchSpec;
use anyhow::{Context, Result, bail};
use rmcp::schemars::{self, JsonSchema};
use roxmltree::{Document, Node, ParsingOptions};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

const SVG_NS: &str = "http://www.w3.org/2000/svg";
const INKSCAPE_NS: &str = "http://www.inkscape.org/namespaces/inkscape";
const SODIPODI_NS: &str = "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd";
const MAX_OBJECTS: usize = 2_000;
const MAX_POINTS: usize = 16_384;
const MAX_RAWR: u64 = 2 * 1024 * 1024;
const FPS: f64 = 60.0;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TranslateKeyframe {
    /// Fraction of the duration, from 0 through 1. Every track starts at 0 and ends at 1.
    pub at: f64,
    pub x: f64,
    pub y: f64,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpacityKeyframe {
    pub at: f64,
    /// Opacity from 0 (transparent) through 1 (opaque).
    pub value: f64,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Track {
    /// Existing SVG group ID. Translation replaces that group's local translation.
    pub target_id: String,
    pub translate: Option<Vec<TranslateKeyframe>>,
    pub opacity: Option<Vec<OpacityKeyframe>>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Create {
    /// Project-relative static SVG. Solid-color shapes and paths; no text, gradients, clips or dashed strokes.
    pub source: String,
    /// Distinct project-relative .rawr output. The SVG source is preserved.
    pub output: String,
    /// 0.05..30 seconds, at 60 frames per second.
    pub duration_seconds: f64,
    /// At most 64 named groups, each with 2..64 linear translation or opacity keyframes.
    pub tracks: Vec<Track>,
    #[serde(default)]
    pub overwrite: bool,
    pub expected_sha256: Option<String>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Inspect {
    pub path: String,
}
#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FrameFormat {
    Png,
    Svg,
}
impl FrameFormat {
    fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Svg => "svg",
        }
    }
    fn media(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Svg => "image/svg+xml",
        }
    }
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Render {
    pub source: String,
    pub output: String,
    pub format: FrameFormat,
    /// Zero-based frame before the document's last_frame. One selected frame per request.
    pub frame: u32,
    #[serde(default)]
    pub overwrite: bool,
    pub expected_sha256: Option<String>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Export {
    pub source: String,
    pub output: String,
    /// False plays once and freezes the final state. True explicitly requests an indefinite loop.
    #[serde(default)]
    pub loop_animation: bool,
    #[serde(default)]
    pub overwrite: bool,
    pub expected_sha256: Option<String>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Retime {
    pub source: String,
    /// A distinct .rawr filename; the source is preserved.
    pub output: String,
    pub duration_seconds: f64,
    #[serde(default)]
    pub overwrite: bool,
    pub expected_sha256: Option<String>,
}

#[derive(Default, Serialize)]
struct Animation {
    width: u32,
    height: u32,
    fps: u32,
    first_frame: u32,
    last_frame: f64,
    duration_seconds: f64,
    objects: usize,
    drawable_shapes: usize,
    keyframes: usize,
    groups: Vec<Value>,
}

fn duration(seconds: f64) -> Result<f64> {
    if !seconds.is_finite() || !(0.05..=30.0).contains(&seconds) {
        bail!("animation duration must be 0.05..30 seconds");
    }
    Ok(seconds * FPS)
}
fn object(value: &Value) -> Result<&Map<String, Value>> {
    value
        .as_object()
        .context("native document expected an object")
}
fn keys<'a>(value: &'a Value, allowed: &[&str]) -> Result<&'a Map<String, Value>> {
    let map = object(value)?;
    if let Some(key) = map.keys().find(|key| !allowed.contains(&key.as_str())) {
        bail!("unsupported native document field {key}");
    }
    Ok(map)
}
fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value> {
    value
        .get(key)
        .with_context(|| format!("native document lacks {key}"))
}
fn text(value: &Value, limit: usize) -> Result<&str> {
    let text = value
        .as_str()
        .context("native document expected a string")?;
    if text.len() > limit || text.chars().any(char::is_control) {
        bail!("native document string is too long or contains controls");
    }
    Ok(text)
}
fn number(value: &Value, min: f64, max: f64) -> Result<f64> {
    let n = value
        .as_f64()
        .context("native document expected a number")?;
    if !n.is_finite() || !(min..=max).contains(&n) {
        bail!("native document number is outside its permitted range");
    }
    Ok(n)
}
fn boolean(value: &Value) -> Result<bool> {
    value
        .as_bool()
        .context("native document expected a boolean")
}
fn point(value: &Value, min: f64, max: f64) -> Result<[f64; 2]> {
    keys(value, &["x", "y"])?;
    Ok([
        number(field(value, "x")?, min, max)?,
        number(field(value, "y")?, min, max)?,
    ])
}
fn color(value: &Value) -> Result<()> {
    let c = text(value, 9)?;
    if !matches!(c.len(), 7 | 9)
        || !c.starts_with('#')
        || !c[1..].bytes().all(|b| b.is_ascii_hexdigit())
    {
        bail!("native paint must be a literal RGB/RGBA color");
    }
    Ok(())
}
fn plain(value: &Value) -> Result<&Value> {
    keys(value, &["value"])?;
    field(value, "value")
}
fn scalar(value: &Value, min: f64, max: f64) -> Result<f64> {
    number(plain(value)?, min, max)
}
fn fixed_point(value: &Value, min: f64, max: f64) -> Result<[f64; 2]> {
    point(plain(value)?, min, max)
}
fn enum_value(value: &Value, allowed: &[&str]) -> Result<()> {
    if !allowed.contains(&text(value, 64)?) {
        bail!("unsupported native enum value");
    }
    Ok(())
}
fn array(value: &Value, limit: usize) -> Result<&Vec<Value>> {
    let values = value
        .as_array()
        .context("native document expected an array")?;
    if values.len() > limit {
        bail!("native document array exceeds its item budget");
    }
    Ok(values)
}
fn type_is(value: &Value, expected: &str) -> Result<()> {
    if field(value, "__type__")?.as_str() != Some(expected) {
        bail!("native document expected {expected}");
    }
    Ok(())
}

struct Validator {
    report: Animation,
    uuids: BTreeSet<String>,
    points: usize,
    tracks: usize,
}
impl Validator {
    fn identity(&mut self, value: &Value) -> Result<()> {
        let uuid = text(field(value, "uuid")?, 36)?;
        if uuid.len() != 36
            || uuid == "00000000-0000-0000-0000-000000000000"
            || !uuid.bytes().enumerate().all(|(i, b)| {
                if [8, 13, 18, 23].contains(&i) {
                    b == b'-'
                } else {
                    b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
                }
            })
            || !self.uuids.insert(uuid.into())
        {
            bail!("native UUIDs must be valid and unique");
        }
        text(field(value, "name")?, 256)?;
        Ok(())
    }
    fn animated(&mut self, value: &Value, is_point: bool, min: f64, max: f64) -> Result<usize> {
        keys(value, &["value", "keyframes"])?;
        if let Some(v) = value.get("value") {
            if value.get("keyframes").is_some() {
                bail!("native property cannot contain both a value and keyframes");
            }
            if is_point {
                point(v, min, max)?;
            } else {
                number(v, min, max)?;
            }
            return Ok(0);
        }
        let frames = array(field(value, "keyframes")?, 64)?;
        if frames.len() < 2 {
            bail!("animated native properties need 2..64 keyframes");
        }
        self.tracks += 1;
        if self.tracks > 128 {
            bail!("native document exceeds 128 animation tracks");
        }
        let mut previous = -1.0;
        for (i, frame) in frames.iter().enumerate() {
            keys(
                frame,
                if is_point {
                    &[
                        "time",
                        "value",
                        "before",
                        "after",
                        "tan_in",
                        "tan_out",
                        "point_type",
                    ]
                } else {
                    &["time", "value", "before", "after"]
                },
            )?;
            let time = number(field(frame, "time")?, 0.0, self.report.last_frame)?;
            if time <= previous
                || (i == 0 && time != 0.0)
                || (i == frames.len() - 1 && (time - self.report.last_frame).abs() > 0.000001)
            {
                bail!("native keyframe times must increase from zero to the document duration");
            }
            previous = time;
            if point(field(frame, "before")?, 0.0, 1.0)? != [0.0, 0.0]
                || point(field(frame, "after")?, 0.0, 1.0)? != [1.0, 1.0]
            {
                bail!("only linear native keyframe interpolation is supported");
            }
            if is_point {
                let p = point(field(frame, "value")?, min, max)?;
                if point(field(frame, "tan_in")?, min, max)? != p
                    || point(field(frame, "tan_out")?, min, max)? != p
                    || field(frame, "point_type")?.as_i64() != Some(0)
                {
                    bail!("native motion tangents must match the linear keyframe position");
                }
            } else {
                number(field(frame, "value")?, min, max)?;
            }
        }
        self.report.keyframes += frames.len();
        Ok(frames.len())
    }
    fn animation(&self, value: &Value) -> Result<()> {
        keys(value, &["__type__", "first_frame", "last_frame"])?;
        type_is(value, "AnimationContainer")?;
        if number(field(value, "first_frame")?, 0.0, 0.0)? != 0.0
            || (number(field(value, "last_frame")?, 3.0, 1800.0)? - self.report.last_frame).abs()
                > 0.000001
        {
            bail!("all native layers must share the composition's bounded duration");
        }
        Ok(())
    }
    fn transform(&mut self, value: &Value) -> Result<usize> {
        keys(
            value,
            &["__type__", "anchor_point", "position", "rotation", "scale"],
        )?;
        type_is(value, "Transform")?;
        fixed_point(field(value, "anchor_point")?, -1_000_000.0, 1_000_000.0)?;
        fixed_point(field(value, "scale")?, -1000.0, 1000.0)?;
        scalar(field(value, "rotation")?, -36_000.0, 36_000.0)?;
        self.animated(field(value, "position")?, true, -1_000_000.0, 1_000_000.0)
    }
    fn shape(&mut self, value: &Value, depth: usize) -> Result<()> {
        self.report.objects += 1;
        if depth > 48 || self.report.objects > MAX_OBJECTS {
            bail!("native shapes exceed the depth or 2000-object budget");
        }
        let kind = text(field(value, "__type__")?, 32)?;
        let extra: &[&str] = match kind {
            "Group" => &["auto_orient", "opacity", "shapes", "transform"],
            "Layer" => &[
                "auto_orient",
                "opacity",
                "shapes",
                "transform",
                "animation",
                "mask",
                "parent",
                "render",
            ],
            "Fill" => &["color", "fill_rule", "opacity", "use"],
            "Stroke" => &[
                "cap",
                "color",
                "join",
                "miter_limit",
                "opacity",
                "use",
                "width",
            ],
            "Rect" => &["position", "reversed", "rounded", "size"],
            "Ellipse" => &["position", "reversed", "size"],
            "Path" => &["closed", "reversed", "shape"],
            _ => bail!(
                "native {kind} is outside the shapes-only profile; text, images, compositions, effects and scripts are refused"
            ),
        };
        let mut allowed = vec![
            "__type__",
            "uuid",
            "name",
            "group_color",
            "locked",
            "visible",
        ];
        allowed.extend_from_slice(extra);
        keys(value, &allowed)?;
        self.identity(value)?;
        color(field(value, "group_color")?)?;
        boolean(field(value, "locked")?)?;
        boolean(field(value, "visible")?)?;
        match kind {
            "Group" | "Layer" => {
                if boolean(field(value, "auto_orient")?)? {
                    bail!("native auto-orient is unsupported");
                }
                if kind == "Layer" {
                    self.animation(field(value, "animation")?)?;
                    if !field(value, "parent")?.is_null() {
                        bail!("linked native layer parents are unsupported");
                    }
                    boolean(field(value, "render")?)?;
                    let mask = field(value, "mask")?;
                    keys(mask, &["__type__", "inverted", "mask"])?;
                    type_is(mask, "MaskSettings")?;
                    if boolean(field(mask, "inverted")?)?
                        || field(mask, "mask")?.as_str() != Some("NoMask")
                    {
                        bail!("native masks are outside the shapes-only profile");
                    }
                }
                let opacity = self.animated(field(value, "opacity")?, false, 0.0, 1.0)?;
                let translation = self.transform(field(value, "transform")?)?;
                self.report.groups.push(json!({"name":field(value,"name")?,"uuid":field(value,"uuid")?,"translation_keyframes":translation,"opacity_keyframes":opacity}));
                for child in array(field(value, "shapes")?, MAX_OBJECTS)? {
                    self.shape(child, depth + 1)?;
                }
            }
            "Fill" | "Stroke" => {
                color(plain(field(value, "color")?)?)?;
                scalar(field(value, "opacity")?, 0.0, 1.0)?;
                if !field(value, "use")?.is_null() {
                    bail!("native paint links and gradients are unsupported");
                }
                if kind == "Fill" {
                    enum_value(field(value, "fill_rule")?, &["NonZero", "EvenOdd"])?;
                } else {
                    enum_value(field(value, "cap")?, &["ButtCap", "RoundCap", "SquareCap"])?;
                    enum_value(
                        field(value, "join")?,
                        &["MiterJoin", "RoundJoin", "BevelJoin"],
                    )?;
                    number(field(value, "miter_limit")?, 0.0, 10_000.0)?;
                    scalar(field(value, "width")?, 0.0, 10_000.0)?;
                }
            }
            "Rect" | "Ellipse" => {
                fixed_point(field(value, "position")?, -1_000_000.0, 1_000_000.0)?;
                let size = plain(field(value, "size")?)?;
                keys(size, &["width", "height"])?;
                number(field(size, "width")?, 0.0, 1_000_000.0)?;
                number(field(size, "height")?, 0.0, 1_000_000.0)?;
                boolean(field(value, "reversed")?)?;
                if kind == "Rect" {
                    scalar(field(value, "rounded")?, 0.0, 1_000_000.0)?;
                }
                self.report.drawable_shapes += 1;
            }
            "Path" => {
                boolean(field(value, "closed")?)?;
                boolean(field(value, "reversed")?)?;
                let shape = plain(field(value, "shape")?)?;
                keys(shape, &["closed", "points"])?;
                boolean(field(shape, "closed")?)?;
                let points = array(field(shape, "points")?, MAX_POINTS)?;
                if points.is_empty() {
                    bail!("native path has no points");
                }
                self.points += points.len();
                if self.points > MAX_POINTS {
                    bail!("native document exceeds 16384 path points");
                }
                for p in points {
                    keys(p, &["pos", "tan_in", "tan_out", "type"])?;
                    for key in ["pos", "tan_in", "tan_out"] {
                        point(field(p, key)?, -1_000_000.0, 1_000_000.0)?;
                    }
                    if !matches!(field(p, "type")?.as_i64(), Some(0..=2)) {
                        bail!("unsupported native path point type");
                    }
                }
                self.report.drawable_shapes += 1;
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}

fn validate(value: &Value) -> Result<Animation> {
    keys(value, &["format", "assets", "info", "metadata"])?;
    let format = field(value, "format")?;
    keys(
        format,
        &["format_version", "generator", "generator_version"],
    )?;
    if field(format, "format_version")?.as_u64() != Some(8) {
        bail!("only native Glaxnimate schema 8 is supported");
    }
    text(field(format, "generator")?, 256)?;
    text(field(format, "generator_version")?, 256)?;
    let info = field(value, "info")?;
    keys(info, &["author", "description", "keywords"])?;
    text(field(info, "author")?, 1024)?;
    text(field(info, "description")?, 4096)?;
    for keyword in array(field(info, "keywords")?, 64)? {
        text(keyword, 256)?;
    }
    if !object(field(value, "metadata")?)?.is_empty() {
        bail!("arbitrary native document metadata is unsupported");
    }
    let assets = field(value, "assets")?;
    keys(
        assets,
        &[
            "__type__",
            "uuid",
            "name",
            "colors",
            "compositions",
            "fonts",
            "gradient_colors",
            "gradients",
            "images",
        ],
    )?;
    type_is(assets, "Assets")?;
    let mut v = Validator {
        report: Animation::default(),
        uuids: BTreeSet::new(),
        points: 0,
        tracks: 0,
    };
    v.identity(assets)?;
    for (key, kind) in [
        ("colors", "NamedColorList"),
        ("compositions", "CompositionList"),
        ("fonts", "FontList"),
        ("gradient_colors", "GradientColorsList"),
        ("gradients", "GradientList"),
        ("images", "BitmapList"),
    ] {
        let container = field(assets, key)?;
        keys(container, &["__type__", "uuid", "name", "values"])?;
        type_is(container, kind)?;
        v.identity(container)?;
        let items = array(field(container, "values")?, 1)?;
        if (key == "compositions" && items.len() != 1)
            || (key != "compositions" && !items.is_empty())
        {
            bail!(
                "native profile requires one composition and no fonts, images, gradient or linked color assets"
            );
        }
    }
    let comp = &assets["compositions"]["values"][0];
    keys(
        comp,
        &[
            "__type__",
            "uuid",
            "name",
            "animation",
            "fps",
            "group_color",
            "height",
            "width",
            "locked",
            "shapes",
            "visible",
        ],
    )?;
    type_is(comp, "Composition")?;
    v.identity(comp)?;
    if number(field(comp, "fps")?, FPS, FPS)? != FPS {
        bail!("native profile uses 60 fps");
    }
    let width = field(comp, "width")?
        .as_u64()
        .filter(|n| (1..=8192).contains(n))
        .context("native width must be 1..8192 pixels")?;
    let height = field(comp, "height")?
        .as_u64()
        .filter(|n| (1..=8192).contains(n))
        .context("native height must be 1..8192 pixels")?;
    if width * height > 16_777_216 {
        bail!("native composition exceeds the 16 megapixel budget");
    }
    v.report.width = width as u32;
    v.report.height = height as u32;
    v.report.fps = 60;
    v.report.last_frame = number(field(field(comp, "animation")?, "last_frame")?, 3.0, 1800.0)?;
    v.report.duration_seconds = v.report.last_frame / FPS;
    v.animation(field(comp, "animation")?)?;
    color(field(comp, "group_color")?)?;
    boolean(field(comp, "locked")?)?;
    boolean(field(comp, "visible")?)?;
    for shape in array(field(comp, "shapes")?, MAX_OBJECTS)? {
        v.shape(shape, 0)?;
    }
    if v.report.drawable_shapes == 0 {
        bail!("native composition has no drawable shapes");
    }
    Ok(v.report)
}

fn rawr(bytes: &[u8]) -> Result<(Value, Animation)> {
    if bytes.is_empty() || bytes.len() as u64 > MAX_RAWR {
        bail!("native animation must be 1 byte..2 MiB");
    }
    let value: Value = serde_json::from_slice(bytes).context("malformed native animation JSON")?;
    let report = validate(&value)?;
    Ok((value, report))
}
fn xml(text: &str) -> Result<Document<'_>> {
    Document::parse_with_options(
        text,
        ParsingOptions {
            allow_dtd: false,
            nodes_limit: svg::MAX_NODES,
            ..Default::default()
        },
    )
    .context("malformed vector XML")
}
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn attribute_name(attr: roxmltree::Attribute<'_, '_>) -> Result<String> {
    let prefix = match attr.namespace() {
        None => "",
        Some(INKSCAPE_NS) => "inkscape:",
        Some(SODIPODI_NS) => "sodipodi:",
        Some("http://www.w3.org/1999/xlink") => "xlink:",
        Some("http://www.w3.org/XML/1998/namespace") => "xml:",
        _ => bail!("unsupported vector namespace"),
    };
    Ok(format!("{prefix}{}", attr.name()))
}
fn start_tag(node: Node<'_, '_>, out: &mut String) -> Result<()> {
    out.push('<');
    out.push_str(node.tag_name().name());
    if node.has_tag_name((SVG_NS, "svg")) {
        out.push_str(" xmlns=\"http://www.w3.org/2000/svg\" xmlns:inkscape=\"http://www.inkscape.org/namespaces/inkscape\" xmlns:sodipodi=\"http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd\" xmlns:xlink=\"http://www.w3.org/1999/xlink\"");
    }
    Ok(())
}
fn attribute(out: &mut String, name: &str, value: &str) {
    out.push_str(&format!(" {name}=\"{}\"", escape(value)));
}
fn key_times(times: impl Iterator<Item = f64>) -> String {
    times.map(|t| t.to_string()).collect::<Vec<_>>().join(";")
}
fn verify_times(times: &[f64]) -> Result<()> {
    if !(2..=64).contains(&times.len())
        || times.first() != Some(&0.0)
        || times.last() != Some(&1.0)
        || times
            .iter()
            .any(|t| !t.is_finite() || !(0.0..=1.0).contains(t))
        || times.windows(2).any(|w| w[1] <= w[0])
    {
        bail!("each track needs 2..64 increasing keyframes, starting at at=0 and ending at at=1");
    }
    Ok(())
}
fn injected_svg(source: &[u8], tracks: &[Track], seconds: f64) -> Result<Vec<u8>> {
    let inspection = svg::inspect(source, false)?;
    if inspection.width * inspection.height > 16_777_216.0
        || inspection.elements.keys().any(|s| {
            matches!(
                s.as_str(),
                "text" | "tspan" | "linearGradient" | "radialGradient" | "stop" | "clipPath"
            )
        })
    {
        bail!(
            "animation input supports solid-color shapes and paths only; text, gradients and clipping require conversion to supported vector geometry"
        );
    }
    if tracks.is_empty() || tracks.len() > 64 {
        bail!("animation needs 1..64 named group tracks");
    }
    let doc = xml(std::str::from_utf8(source)?)?;
    // The native stroke model has no dash pattern, and its rectangle model has one corner radius.
    // Refuse those imports rather than returning a valid but visibly different document.
    for node in doc.descendants().filter(Node::is_element) {
        let property = |name: &str, value: &str| -> Result<()> {
            if name == "stroke-dasharray" && value.trim() != "none" {
                bail!(
                    "Glaxnimate does not preserve dashed strokes; convert the dash pattern to explicit vector shapes first"
                );
            }
            if name == "stroke-dashoffset"
                && value
                    .trim()
                    .strip_suffix("px")
                    .unwrap_or(value.trim())
                    .parse::<f64>()?
                    != 0.0
            {
                bail!("Glaxnimate does not preserve dash offsets");
            }
            Ok(())
        };
        for attr in node.attributes() {
            property(attr.name(), attr.value())?;
            if attr.name() == "style" {
                for declaration in attr.value().split(';').filter(|d| !d.trim().is_empty()) {
                    let (name, value) = declaration
                        .split_once(':')
                        .context("malformed inline style")?;
                    property(name.trim(), value.trim())?;
                }
            }
        }
        if node.has_tag_name((SVG_NS, "rect")) {
            let radius = |value: &str| -> Result<f64> {
                let r = value
                    .trim()
                    .strip_suffix("px")
                    .unwrap_or(value.trim())
                    .parse::<f64>()
                    .context("animation rectangle corner radii must use pixels")?;
                if !r.is_finite() || !(0.0..=1_000_000.0).contains(&r) {
                    bail!("invalid animation rectangle corner radius");
                }
                Ok(r)
            };
            let rx = node.attribute("rx").map(radius).transpose()?;
            let ry = node.attribute("ry").map(radius).transpose()?;
            if let (Some(x), Some(y)) = (rx, ry)
                && x != y
            {
                bail!(
                    "Glaxnimate supports one rounded-rectangle radius; convert unequal rx/ry corners to paths first"
                );
            }
        }
    }
    let mut additions = BTreeMap::new();
    for track in tracks {
        if !svg::identifier(&track.target_id) || additions.contains_key(track.target_id.as_str()) {
            bail!("animation group targets must be unique SVG IDs");
        }
        let group = doc
            .descendants()
            .find(|node| node.attribute("id") == Some(track.target_id.as_str()))
            .context("animation target ID is absent")?;
        if !group.has_tag_name((SVG_NS, "g")) {
            bail!("animation targets must identify SVG groups");
        }
        let mut markup = String::new();
        if let Some(keys) = &track.translate {
            if group.attribute("transform").is_some() {
                bail!(
                    "a translation target cannot already have a transform; put the static transform in an inner group"
                );
            }
            verify_times(&keys.iter().map(|k| k.at).collect::<Vec<_>>())?;
            if keys.iter().any(|k| {
                !k.x.is_finite()
                    || !k.y.is_finite()
                    || k.x.abs() > 1_000_000.0
                    || k.y.abs() > 1_000_000.0
            }) {
                bail!("translation coordinates must be finite and bounded by 1000000");
            }
            let values = keys
                .iter()
                .map(|k| format!("{} {}", k.x, k.y))
                .collect::<Vec<_>>()
                .join(";");
            markup.push_str(&format!("<animateTransform attributeName=\"transform\" type=\"translate\" values=\"{values}\" keyTimes=\"{}\" dur=\"{seconds}s\" repeatCount=\"1\" fill=\"freeze\"/>",key_times(keys.iter().map(|k|k.at))));
        }
        if let Some(keys) = &track.opacity {
            verify_times(&keys.iter().map(|k| k.at).collect::<Vec<_>>())?;
            if keys
                .iter()
                .any(|k| !k.value.is_finite() || !(0.0..=1.0).contains(&k.value))
            {
                bail!("opacity keyframes must be 0..1");
            }
            markup.push_str(&format!("<animate attributeName=\"opacity\" values=\"{}\" keyTimes=\"{}\" dur=\"{seconds}s\" repeatCount=\"1\" fill=\"freeze\"/>",key_times(keys.iter().map(|k|k.value)),key_times(keys.iter().map(|k|k.at))));
        }
        if markup.is_empty() {
            bail!("each group track needs translation or opacity keyframes");
        }
        additions.insert(track.target_id.as_str(), markup);
    }
    fn write(
        node: Node<'_, '_>,
        additions: &BTreeMap<&str, String>,
        out: &mut String,
    ) -> Result<()> {
        if node.is_text() {
            out.push_str(&escape(node.text().unwrap_or_default()));
            return Ok(());
        }
        if !node.is_element() {
            return Ok(());
        }
        start_tag(node, out)?;
        for attr in node.attributes() {
            attribute(out, &attribute_name(attr)?, attr.value());
        }
        out.push('>');
        for child in node.children() {
            write(child, additions, out)?;
        }
        if let Some(addition) = node.attribute("id").and_then(|id| additions.get(id)) {
            out.push_str(addition);
        }
        out.push_str(&format!("</{}>", node.tag_name().name()));
        Ok(())
    }
    let mut output = String::new();
    write(doc.root_element(), &additions, &mut output)?;
    svg::inspect(output.as_bytes(), true)?;
    Ok(output.into_bytes())
}

/// Native linear positions export as cubic motion paths with coincident endpoint tangents.
/// Explicit translate values preserve keyframe timing without SVG motion-path distance mapping.
fn motion_values(path: &str) -> Result<String> {
    let tokens = path
        .split(|c: char| c.is_ascii_whitespace() || c == ',')
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>();
    let num = |s: &str| -> Result<f64> {
        let n = s.parse::<f64>()?;
        if !n.is_finite() || n.abs() > 1_000_000.0 {
            bail!("invalid motion path coordinate");
        }
        Ok(n)
    };
    if tokens.len() < 10 || tokens[0] != "M" || tokens[3] != "C" {
        bail!("unsupported native linear motion path");
    }
    let mut previous = [num(tokens[1])?, num(tokens[2])?];
    let mut values = vec![format!("{} {}", previous[0], previous[1])];
    let mut i = 3;
    while i < tokens.len() {
        if tokens[i] == "C" {
            i += 1;
        }
        let part = tokens
            .get(i..i + 6)
            .context("incomplete native cubic motion segment")?;
        let first = [num(part[0])?, num(part[1])?];
        let last = [num(part[2])?, num(part[3])?];
        let end = [num(part[4])?, num(part[5])?];
        if first != previous || last != end {
            bail!("curved native motion paths are outside the linear track profile");
        }
        values.push(format!("{} {}", end[0], end[1]));
        previous = end;
        i += 6;
    }
    if values.len() > 64 {
        bail!("native motion path exceeds the keyframe budget");
    }
    Ok(values.join(";"))
}
fn normalized_svg(bytes: &[u8], animated: bool, looping: bool) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(bytes)?;
    if text.contains("<!DOCTYPE") || text.contains("<!ENTITY") {
        bail!("native SVG declarations are unsupported");
    }
    let doc = xml(text)?;
    if !doc.root_element().has_tag_name((SVG_NS, "svg")) {
        bail!("native export is not SVG");
    }
    fn write(node: Node<'_, '_>, out: &mut String, animated: bool, looping: bool) -> Result<()> {
        if node.is_text() {
            out.push_str(&escape(node.text().unwrap_or_default()));
            return Ok(());
        }
        if node.is_pi() {
            bail!("native SVG processing instructions are refused");
        }
        if !node.is_element() {
            return Ok(());
        }
        if node.has_tag_name((SVG_NS, "metadata")) || node.has_tag_name((SODIPODI_NS, "namedview"))
        {
            return Ok(());
        }
        if node.tag_name().namespace() != Some(SVG_NS) {
            bail!("native SVG contains unsupported foreign content");
        }
        let animation = matches!(
            node.tag_name().name(),
            "animate" | "animateTransform" | "animateMotion"
        );
        if animation && !animated {
            bail!("selected-frame export unexpectedly retained animation");
        }
        if node.tag_name().name() == "animateMotion" {
            let values =
                motion_values(node.attribute("path").context("native motion lacks path")?)?;
            out.push_str("<animateTransform attributeName=\"transform\" type=\"translate\"");
            attribute(out, "values", &values);
        } else {
            start_tag(node, out)?;
        }
        for attr in node.attributes() {
            if attr.namespace() == Some(INKSCAPE_NS)
                && matches!(attr.name(), "export-xdpi" | "export-ydpi")
            {
                continue;
            }
            if animation && matches!(attr.name(), "begin" | "dur" | "repeatCount" | "fill") {
                continue;
            }
            if node.tag_name().name() == "animateMotion"
                && matches!(attr.name(), "path" | "values" | "attributeName")
            {
                continue;
            }
            if animation && attr.name() == "keyTimes" {
                attribute(
                    out,
                    "keyTimes",
                    &attr
                        .value()
                        .split(';')
                        .map(str::trim)
                        .collect::<Vec<_>>()
                        .join(";"),
                );
            } else {
                attribute(out, &attribute_name(attr)?, attr.value());
            }
        }
        if animation {
            let begin = node
                .attribute("begin")
                .unwrap_or("0")
                .trim_end_matches('s')
                .parse::<f64>()?;
            if begin != 0.0 {
                bail!("native animation has an unsupported begin time");
            }
            let seconds = node
                .attribute("dur")
                .context("native animation lacks duration")?
                .trim_end_matches('s')
                .parse::<f64>()?;
            duration(seconds)?;
            attribute(out, "begin", "0s");
            attribute(out, "dur", &format!("{seconds}s"));
            attribute(out, "repeatCount", if looping { "indefinite" } else { "1" });
            attribute(out, "fill", "freeze");
        }
        out.push('>');
        for child in node.children() {
            write(child, out, animated, looping)?;
        }
        out.push_str(&format!(
            "</{}>",
            if node.tag_name().name() == "animateMotion" {
                "animateTransform"
            } else {
                node.tag_name().name()
            }
        ));
        Ok(())
    }
    let mut output = String::new();
    write(doc.root_element(), &mut output, animated, looping)?;
    svg::inspect(output.as_bytes(), animated)?;
    Ok(output.into_bytes())
}

async fn native_export(
    spec: &LaunchSpec,
    input: &Path,
    output: &Path,
    format: &str,
    workspace: &Path,
    control: &Control<'_>,
) -> Result<Vec<u8>> {
    process::run(
        spec,
        &[
            input.to_string_lossy().into_owned(),
            "--export".into(),
            output.to_string_lossy().into_owned(),
            "--export-format".into(),
            format.into(),
        ],
        workspace,
        Some(output),
        control,
    )
    .await?;
    process::output(output)
}
fn verify_imported_tracks(value: &Value, tracks: &[Track], last_frame: f64) -> Result<()> {
    fn groups<'a>(value: &'a Value, name: &str, result: &mut Vec<&'a Value>) {
        if let Some(map) = value.as_object() {
            if map.get("__type__").and_then(Value::as_str) == Some("Group")
                && map.get("name").and_then(Value::as_str) == Some(name)
            {
                result.push(value);
            }
            for child in map.values() {
                groups(child, name, result);
            }
        } else if let Some(items) = value.as_array() {
            for child in items {
                groups(child, name, result);
            }
        }
    }
    let near = |actual: &Value, expected: f64| {
        actual
            .as_f64()
            .is_some_and(|n| (n - expected).abs() <= 0.000001)
    };
    for track in tracks {
        let mut matches = vec![];
        groups(value, &track.target_id, &mut matches);
        if matches.len() != 1 {
            bail!(
                "native import did not preserve the unique named group {}",
                track.target_id
            );
        }
        let group = matches[0];
        if let Some(expected) = &track.translate {
            let frames = group["transform"]["position"]["keyframes"]
                .as_array()
                .context("native import lost translation keyframes")?;
            if frames.len() != expected.len()
                || frames.iter().zip(expected).any(|(actual, key)| {
                    !near(&actual["time"], key.at * last_frame)
                        || !near(&actual["value"]["x"], key.x)
                        || !near(&actual["value"]["y"], key.y)
                })
            {
                bail!("native import changed requested translation values or timing");
            }
        }
        if let Some(expected) = &track.opacity {
            let frames = group["opacity"]["keyframes"]
                .as_array()
                .context("native import lost opacity keyframes")?;
            if frames.len() != expected.len()
                || frames.iter().zip(expected).any(|(actual, key)| {
                    !near(&actual["time"], key.at * last_frame)
                        || !near(&actual["value"], key.value)
                })
            {
                bail!("native import changed requested opacity values or timing");
            }
        }
    }
    Ok(())
}
pub async fn create(
    spec: &LaunchSpec,
    args: Create,
    workspace: &Path,
    control: &Control<'_>,
) -> Result<EditorResult> {
    duration(args.duration_seconds)?;
    let source = files::read(&spec.cwd, &args.source, &["svg"], files::MAX_SVG_BYTES)?;
    let bytes = injected_svg(&source, &args.tracks, args.duration_seconds)?;
    let destination = Destination::new(
        &spec.cwd,
        &args.output,
        &["rawr"],
        args.overwrite,
        args.expected_sha256.as_deref(),
    )?;
    let input = workspace.join("input.svg");
    std::fs::write(&input, bytes)?;
    let native = native_export(
        spec,
        &input,
        &workspace.join("created.rawr"),
        "glaxnimate",
        workspace,
        control,
    )
    .await?;
    let (value, report) = rawr(&native)?;
    if (report.duration_seconds - args.duration_seconds).abs() > 0.000001 {
        bail!("native import changed the requested animation duration");
    }
    for track in &args.tracks {
        let groups = report
            .groups
            .iter()
            .filter(|g| g["name"].as_str() == Some(&track.target_id))
            .collect::<Vec<_>>();
        if groups.len() != 1
            || groups[0]["translation_keyframes"].as_u64()
                != Some(track.translate.as_ref().map_or(0, Vec::len) as u64)
            || groups[0]["opacity_keyframes"].as_u64()
                != Some(track.opacity.as_ref().map_or(0, Vec::len) as u64)
        {
            bail!("native import did not preserve the requested named group tracks");
        }
    }
    verify_imported_tracks(&value, &args.tracks, report.last_frame)?;
    control.check()?;
    let artifact = destination.publish(&native, "application/json")?;
    Ok(EditorResult {
        operation: "animation_create".into(),
        editable_source: Some(args.output),
        source_sha256: Some(crate::util::sha256(&source)),
        artifact: Some(artifact),
        animation: Some(serde_json::to_value(report)?),
        ..Default::default()
    })
}
pub async fn inspect(
    spec: &LaunchSpec,
    args: Inspect,
    _workspace: &Path,
    control: &Control<'_>,
) -> Result<EditorResult> {
    control.check()?;
    let bytes = files::read(&spec.cwd, &args.path, &["rawr"], MAX_RAWR)?;
    let (_, report) = rawr(&bytes)?;
    Ok(EditorResult {
        operation: "animation_inspect".into(),
        editable_source: Some(args.path),
        source_sha256: Some(crate::util::sha256(&bytes)),
        animation: Some(serde_json::to_value(report)?),
        ..Default::default()
    })
}
pub async fn render(
    spec: &LaunchSpec,
    args: Render,
    workspace: &Path,
    control: &Control<'_>,
) -> Result<EditorResult> {
    let bytes = files::read(&spec.cwd, &args.source, &["rawr"], MAX_RAWR)?;
    let (value, report) = rawr(&bytes)?;
    if f64::from(args.frame) >= report.last_frame {
        bail!("selected frame must be before last_frame");
    }
    let destination = Destination::new(
        &spec.cwd,
        &args.output,
        &[args.format.extension()],
        args.overwrite,
        args.expected_sha256.as_deref(),
    )?;
    // Open the validated canonical object, avoiding parser differences for duplicate JSON keys.
    let input = workspace.join("input.rawr");
    std::fs::write(&input, serde_json::to_vec(&value)?)?;
    let output = workspace.join(format!("frame.{}", args.format.extension()));
    process::run(
        spec,
        &[
            input.to_string_lossy().into_owned(),
            "--render".into(),
            output.to_string_lossy().into_owned(),
            "--render-format".into(),
            args.format.extension().into(),
            "--frame".into(),
            args.frame.to_string(),
        ],
        workspace,
        Some(&output),
        control,
    )
    .await?;
    let native = process::output(&output)?;
    let (saved, pixels) = match args.format {
        FrameFormat::Svg => (normalized_svg(&native, false, false)?, None),
        FrameFormat::Png => {
            let (w, h) = process::png(&native, Some(report.width))?;
            if h != report.height {
                bail!("native PNG height differs from the composition");
            }
            (native, Some([w, h]))
        }
    };
    control.check()?;
    let artifact = destination.publish(&saved, args.format.media())?;
    Ok(EditorResult {
        operation: "animation_frame".into(),
        editable_source: Some(args.source),
        source_sha256: Some(crate::util::sha256(&bytes)),
        artifact: Some(artifact),
        pixel_dimensions: pixels,
        animation: Some(json!({"frame":args.frame,"fps":60})),
        ..Default::default()
    })
}
pub async fn export(
    spec: &LaunchSpec,
    args: Export,
    workspace: &Path,
    control: &Control<'_>,
) -> Result<EditorResult> {
    let bytes = files::read(&spec.cwd, &args.source, &["rawr"], MAX_RAWR)?;
    let (value, report) = rawr(&bytes)?;
    if report.keyframes == 0 {
        bail!("animated SVG export requires at least one animation track");
    }
    let destination = Destination::new(
        &spec.cwd,
        &args.output,
        &["svg"],
        args.overwrite,
        args.expected_sha256.as_deref(),
    )?;
    let input = workspace.join("input.rawr");
    std::fs::write(&input, serde_json::to_vec(&value)?)?;
    let native = native_export(
        spec,
        &input,
        &workspace.join("animation.svg"),
        "svg",
        workspace,
        control,
    )
    .await?;
    let saved = normalized_svg(&native, true, args.loop_animation)?;
    let inspection = svg::inspect(&saved, true)?;
    let expected_tracks = report
        .groups
        .iter()
        .map(|g| {
            usize::from(g["translation_keyframes"].as_u64().unwrap_or(0) > 0)
                + usize::from(g["opacity_keyframes"].as_u64().unwrap_or(0) > 0)
        })
        .sum::<usize>();
    if inspection.animations != expected_tracks {
        bail!("native export changed the number of animation tracks");
    }
    control.check()?;
    let artifact = destination.publish(&saved, "image/svg+xml")?;
    Ok(EditorResult {
        operation: "animation_export".into(),
        editable_source: Some(args.source),
        source_sha256: Some(crate::util::sha256(&bytes)),
        artifact: Some(artifact),
        inspection: Some(inspection),
        animation: Some(
            json!({"duration_seconds":report.duration_seconds,"loop_animation":args.loop_animation}),
        ),
        ..Default::default()
    })
}
fn retime_value(value: &mut Value, ratio: f64) {
    if let Some(map) = value.as_object_mut() {
        if map.get("__type__").and_then(Value::as_str) == Some("AnimationContainer")
            && let Some(last) = map.get_mut("last_frame")
        {
            *last = json!(last.as_f64().unwrap() * ratio);
        }
        if let Some(frames) = map.get_mut("keyframes").and_then(Value::as_array_mut) {
            for frame in frames {
                frame["time"] = json!(frame["time"].as_f64().unwrap() * ratio);
            }
        }
        for child in map.values_mut() {
            retime_value(child, ratio);
        }
    } else if let Some(values) = value.as_array_mut() {
        for child in values {
            retime_value(child, ratio);
        }
    }
}
pub async fn retime(
    spec: &LaunchSpec,
    args: Retime,
    workspace: &Path,
    control: &Control<'_>,
) -> Result<EditorResult> {
    if args.source == args.output {
        bail!("retime output must be distinct from its editable source");
    }
    let source_path = files::scoped(&spec.cwd, &args.source, &["rawr"])?;
    let output_path = files::scoped(&spec.cwd, &args.output, &["rawr"])?;
    if output_path.exists() && output_path.canonicalize()? == source_path.canonicalize()? {
        bail!("retime output resolves to the editable source");
    }
    let frames = duration(args.duration_seconds)?;
    let bytes = files::read(&spec.cwd, &args.source, &["rawr"], MAX_RAWR)?;
    let (mut value, report) = rawr(&bytes)?;
    let destination = Destination::new(
        &spec.cwd,
        &args.output,
        &["rawr"],
        args.overwrite,
        args.expected_sha256.as_deref(),
    )?;
    retime_value(&mut value, frames / report.last_frame);
    validate(&value)?;
    let input = workspace.join("retimed-input.rawr");
    std::fs::write(&input, serde_json::to_vec(&value)?)?;
    let saved = native_export(
        spec,
        &input,
        &workspace.join("retimed.rawr"),
        "glaxnimate",
        workspace,
        control,
    )
    .await?;
    let (_, updated) = rawr(&saved)?;
    if (updated.duration_seconds - args.duration_seconds).abs() > 0.000001
        || updated.keyframes != report.keyframes
    {
        bail!("native retiming did not preserve the requested duration and keyframes");
    }
    control.check()?;
    let artifact = destination.publish(&saved, "application/json")?;
    Ok(EditorResult {
        operation: "animation_retime".into(),
        editable_source: Some(args.output),
        source_sha256: Some(crate::util::sha256(&bytes)),
        artifact: Some(artifact),
        animation: Some(serde_json::to_value(updated)?),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    const STATIC: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="640" height="240" viewBox="0 0 640 240"><g id="box"><rect x="20" y="60" width="80" height="80" fill="#e04040"/></g></svg>"##;
    fn track() -> Track {
        Track {
            target_id: "box".into(),
            translate: Some(vec![
                TranslateKeyframe {
                    at: 0.0,
                    x: 0.0,
                    y: 0.0,
                },
                TranslateKeyframe {
                    at: 0.25,
                    x: 120.0,
                    y: 0.0,
                },
                TranslateKeyframe {
                    at: 1.0,
                    x: 240.0,
                    y: 0.0,
                },
            ]),
            opacity: None,
        }
    }
    #[test]
    fn typed_tracks_preserve_source_and_reject_ambiguous_targets() {
        let bytes = injected_svg(STATIC.as_bytes(), &[track()], 2.0).unwrap();
        let inspect = svg::inspect(&bytes, true).unwrap();
        assert_eq!(inspect.animations, 1);
        assert!(
            std::str::from_utf8(&bytes)
                .unwrap()
                .contains("keyTimes=\"0;0.25;1\"")
        );
        let mut invalid = track();
        invalid.target_id = "missing".into();
        assert!(injected_svg(STATIC.as_bytes(), &[invalid], 2.0).is_err());
        assert!(
            injected_svg(
                STATIC
                    .replace("id=\"box\"", "id=\"box\" transform=\"translate(1 2)\"")
                    .as_bytes(),
                &[track()],
                2.0
            )
            .is_err()
        );
        let mut invalid = track();
        invalid.translate.as_mut().unwrap()[1].at = 1.0;
        assert!(injected_svg(STATIC.as_bytes(), &[invalid], 2.0).is_err());
    }
    #[test]
    fn native_motion_normalization_preserves_nonuniform_keyframe_timing() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd" viewBox="0 0 100 100"><metadata><foreign/></metadata><sodipodi:namedview/><g><rect width="10" height="10"/><animateMotion attributeName="transform" path="M 0,0 C 0,0 120,0 120,0 C 120,0 240,0 240,0" values="; ; " keyTimes="0;0.25;1" keySplines="0 0 1 1;0 0 1 1" calcMode="spline" dur="2.000000" begin="0.000000" repeatCount="indefinite"/></g></svg>"##;
        let output = normalized_svg(svg.as_bytes(), true, false).unwrap();
        let text = std::str::from_utf8(&output).unwrap();
        assert!(text.contains("values=\"0 0;120 0;240 0\""));
        assert!(text.contains("keyTimes=\"0;0.25;1\""));
        assert!(text.contains("repeatCount=\"1\""));
        assert!(text.contains("fill=\"freeze\""));
        assert!(
            !text.contains("animateMotion")
                && !text.contains("metadata")
                && !text.contains("namedview")
        );
        assert!(normalized_svg(svg.as_bytes(), false, false).is_err());
        assert!(motion_values("M 0,0 C 0,10 120,0 120,0").is_err());
        assert_eq!(
            motion_values("M -38,18 C -38,18 12,-18 12,-18 12,-18 28,-24 28,-24").unwrap(),
            "-38 18;12 -18;28 -24"
        );
        assert!(motion_values("M 0,0 C 0,0 20,0").is_err());
    }
    #[test]
    fn known_lossy_stroke_and_corner_features_are_refused() {
        for extra in [
            "stroke-dasharray=\"2 10\"",
            "style=\"stroke-dasharray:2 10\"",
            "stroke-dashoffset=\"2\"",
            "rx=\"8\" ry=\"4\"",
            "rx=\"10%\"",
        ] {
            let source = STATIC.replace("<rect ", &format!("<rect {extra} "));
            assert!(
                injected_svg(source.as_bytes(), &[track()], 2.0).is_err(),
                "{extra}"
            );
        }
        for extra in [
            "stroke-dasharray=\"none\"",
            "rx=\"28\"",
            "rx=\"8\" ry=\"8px\"",
        ] {
            let source = STATIC.replace("<rect ", &format!("<rect {extra} "));
            injected_svg(source.as_bytes(), &[track()], 2.0).unwrap();
        }
    }
    #[test]
    fn external_resources_and_duration_overflows_are_rejected() {
        for (name, body) in [
            ("image", "<image href='file:///tmp/a.png'/>"),
            ("text", "<text>Example</text>"),
            ("gradient", "<defs><linearGradient id='paint'/></defs>"),
        ] {
            let input = STATIC.replace("</svg>", &format!("{body}</svg>"));
            assert!(
                injected_svg(input.as_bytes(), &[track()], 2.0).is_err(),
                "{name}"
            );
        }
        for seconds in [0.0, 0.049, 30.001, f64::NAN, f64::INFINITY] {
            assert!(duration(seconds).is_err());
        }
        assert!(rawr(br#"{"format":{"format_version":8},"script":"anything"}"#).is_err());
    }
    fn native_fixture() -> Value {
        serde_json::from_str(r###"{"assets":{"__type__":"Assets","colors":{"__type__":"NamedColorList","name":"","uuid":"00000000-0000-4000-8000-000000000002","values":[]},"compositions":{"__type__":"CompositionList","name":"","uuid":"00000000-0000-4000-8000-000000000003","values":[{"__type__":"Composition","animation":{"__type__":"AnimationContainer","first_frame":0,"last_frame":120},"fps":60,"group_color":"#00000000","height":240,"locked":false,"name":"","shapes":[{"__type__":"Layer","animation":{"__type__":"AnimationContainer","first_frame":0,"last_frame":120},"auto_orient":false,"group_color":"#00000000","locked":false,"mask":{"__type__":"MaskSettings","inverted":false,"mask":"NoMask"},"name":"Layer","opacity":{"value":1},"parent":null,"render":true,"shapes":[{"__type__":"Group","auto_orient":false,"group_color":"#00000000","locked":false,"name":"moving-box","opacity":{"keyframes":[{"after":{"x":1,"y":1},"before":{"x":0,"y":0},"time":0,"value":1},{"after":{"x":1,"y":1},"before":{"x":0,"y":0},"time":120,"value":0.5}]},"shapes":[{"__type__":"Group","auto_orient":false,"group_color":"#00000000","locked":false,"name":"box","opacity":{"value":1},"shapes":[{"__type__":"Fill","color":{"value":"#e04040"},"fill_rule":"NonZero","group_color":"#00000000","locked":false,"name":"","opacity":{"value":1},"use":null,"uuid":"00000000-0000-4000-8000-000000000008","visible":true},{"__type__":"Stroke","cap":"ButtCap","color":{"value":"#00000000"},"group_color":"#00000000","join":"MiterJoin","locked":false,"miter_limit":4,"name":"","opacity":{"value":1},"use":null,"uuid":"00000000-0000-4000-8000-000000000009","visible":true,"width":{"value":1}},{"__type__":"Rect","group_color":"#00000000","locked":false,"name":"","position":{"value":{"x":60,"y":100}},"reversed":false,"rounded":{"value":0},"size":{"value":{"height":80,"width":80}},"uuid":"00000000-0000-4000-8000-000000000010","visible":true}],"transform":{"__type__":"Transform","anchor_point":{"value":{"x":0,"y":0}},"position":{"value":{"x":0,"y":0}},"rotation":{"value":0},"scale":{"value":{"x":1,"y":1}}},"uuid":"00000000-0000-4000-8000-000000000007","visible":true}],"transform":{"__type__":"Transform","anchor_point":{"value":{"x":0,"y":0}},"position":{"keyframes":[{"after":{"x":1,"y":1},"before":{"x":0,"y":0},"point_type":0,"tan_in":{"x":0,"y":0},"tan_out":{"x":0,"y":0},"time":0,"value":{"x":0,"y":0}},{"after":{"x":1,"y":1},"before":{"x":0,"y":0},"point_type":0,"tan_in":{"x":240,"y":0},"tan_out":{"x":240,"y":0},"time":120,"value":{"x":240,"y":0}}]},"rotation":{"value":0},"scale":{"value":{"x":1,"y":1}}},"uuid":"00000000-0000-4000-8000-000000000006","visible":true}],"transform":{"__type__":"Transform","anchor_point":{"value":{"x":0,"y":0}},"position":{"value":{"x":0,"y":0}},"rotation":{"value":0},"scale":{"value":{"x":1,"y":1}}},"uuid":"00000000-0000-4000-8000-000000000005","visible":true}],"uuid":"00000000-0000-4000-8000-000000000004","visible":true,"width":640}]},"fonts":{"__type__":"FontList","name":"","uuid":"00000000-0000-4000-8000-000000000011","values":[]},"gradient_colors":{"__type__":"GradientColorsList","name":"","uuid":"00000000-0000-4000-8000-000000000012","values":[]},"gradients":{"__type__":"GradientList","name":"","uuid":"00000000-0000-4000-8000-000000000013","values":[]},"images":{"__type__":"BitmapList","name":"","uuid":"00000000-0000-4000-8000-000000000014","values":[]},"name":"","uuid":"00000000-0000-4000-8000-000000000001"},"format":{"format_version":8,"generator":"Glaxnimate","generator_version":"0.6.0-git-d2da67a"},"info":{"author":"","description":"","keywords":[]},"metadata":{}}"###).unwrap()
    }
    #[test]
    fn native_profile_preserves_real_geometry_and_retimes_all_keys() {
        let mut fixture = native_fixture();
        let report = validate(&fixture).unwrap();
        assert_eq!((report.width, report.height, report.fps), (640, 240, 60));
        assert_eq!(
            (report.last_frame, report.keyframes, report.drawable_shapes),
            (120.0, 4, 1)
        );
        retime_value(&mut fixture, 2.0);
        let updated = validate(&fixture).unwrap();
        assert_eq!(
            (
                updated.last_frame,
                updated.duration_seconds,
                updated.keyframes
            ),
            (240.0, 4.0, 4)
        );
        assert_eq!(
            fixture["assets"]["compositions"]["values"][0]["shapes"][0]["shapes"][0]["transform"]["position"]
                ["keyframes"][1]["time"],
            240.0
        );
    }
    #[test]
    fn native_profile_rejects_links_unknown_fields_and_unbounded_animation() {
        let mut malicious = native_fixture();
        malicious["assets"]["images"]["values"] =
            json!([{"__type__":"Bitmap","filename":"file:///private/secret"}]);
        assert!(validate(&malicious).is_err());
        let mut malicious = native_fixture();
        malicious["assets"]["compositions"]["values"][0]["script"] = json!("execute");
        assert!(validate(&malicious).is_err());
        let mut malicious = native_fixture();
        malicious["assets"]["compositions"]["values"][0]["animation"]["last_frame"] = json!(1801);
        assert!(validate(&malicious).is_err());
        let mut malicious = native_fixture();
        malicious["assets"]["compositions"]["values"][0]["shapes"][0]["parent"] =
            json!("00000000-0000-4000-8000-000000000001");
        assert!(validate(&malicious).is_err());
        let mut malicious = native_fixture();
        malicious["assets"]["compositions"]["values"][0]["shapes"][0]["shapes"][0]["transform"]["position"]
            ["keyframes"][1]["tan_in"]["y"] = json!(40);
        assert!(validate(&malicious).is_err());
        let mut malicious = native_fixture();
        malicious["assets"]["compositions"]["values"][0]["shapes"][0]["shapes"][0]["opacity"]["keyframes"]
            [0]["before"]["y"] = json!(0.2);
        assert!(validate(&malicious).is_err());
        let mut malicious = native_fixture();
        malicious["metadata"] = json!({"url":"https://example.invalid/asset"});
        assert!(validate(&malicious).is_err());
    }
    #[test]
    fn importer_success_must_preserve_values_not_only_keyframe_counts() {
        let mut fixture = native_fixture();
        let tracks = vec![Track {
            target_id: "moving-box".into(),
            translate: Some(vec![
                TranslateKeyframe {
                    at: 0.0,
                    x: 0.0,
                    y: 0.0,
                },
                TranslateKeyframe {
                    at: 1.0,
                    x: 240.0,
                    y: 0.0,
                },
            ]),
            opacity: Some(vec![
                OpacityKeyframe {
                    at: 0.0,
                    value: 1.0,
                },
                OpacityKeyframe {
                    at: 1.0,
                    value: 0.5,
                },
            ]),
        }];
        verify_imported_tracks(&fixture, &tracks, 120.0).unwrap();
        fixture["assets"]["compositions"]["values"][0]["shapes"][0]["shapes"][0]["opacity"]["keyframes"]
            [1]["value"] = json!(0.8);
        assert!(verify_imported_tracks(&fixture, &tracks, 120.0).is_err());
    }
}
