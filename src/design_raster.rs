//! Bounded, private SVG-to-PNG rendering for design jobs.
//!
//! This module is deliberately a leaf. It parses an SVG snapshot in memory,
//! rejects anything that could resolve a resource or execute active content,
//! loads only a caller-supplied private font file, and renders into a bounded
//! pixmap.
//! Job ownership, deadlines and publication remain with the existing worker.

use resvg::{tiny_skia, usvg};
use std::{
    fmt, fs,
    path::{Component, Path},
    sync::Arc,
};

pub const RESVG_VERSION: &str = "0.49.0";
pub const MAX_SVG_BYTES: usize = 1024 * 1024;
pub const MAX_FONT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_WIDTH: u32 = 2048;
pub const MAX_HEIGHT: u32 = 2048;
pub const MAX_PIXELS: u64 = 4 * 1024 * 1024;
pub const MAX_PIXMAP_BYTES: u64 = MAX_PIXELS * 4;
pub const MAX_PNG_BYTES: usize = 16 * 1024 * 1024;
const MAX_ELEMENTS: u32 = 100_000;
const MAX_DEPTH: usize = 128;
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";
const XLINK_NAMESPACE: &str = "http://www.w3.org/1999/xlink";
const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";

/// Errors returned before or during bounded SVG rendering.
#[derive(Debug)]
pub enum DesignRasterError {
    InputTooLarge,
    InvalidUtf8,
    CompressedSvgRejected,
    DtdRejected,
    Xml(String),
    ElementLimit,
    NestingLimit,
    UnsupportedElement(String),
    UnsafeContent,
    InvalidFont,
    InvalidFontPath,
    FontRead(String),
    Svg(String),
    InvalidSize,
    DimensionLimit,
    PixelLimit,
    PixmapAllocation,
    PngEncoding(String),
    PngTooLarge,
}

impl fmt::Display for DesignRasterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputTooLarge => f.write_str("SVG exceeds the input budget"),
            Self::InvalidUtf8 => f.write_str("SVG must be UTF-8"),
            Self::CompressedSvgRejected => f.write_str("compressed SVG input is not admitted"),
            Self::DtdRejected => f.write_str("DTD and entity declarations are not admitted"),
            Self::Xml(error) => write!(f, "SVG XML parsing failed: {error}"),
            Self::ElementLimit => f.write_str("SVG element limit exceeded"),
            Self::NestingLimit => f.write_str("SVG nesting limit exceeded"),
            Self::UnsupportedElement(name) => write!(f, "SVG element is not admitted: {name}"),
            Self::UnsafeContent => {
                f.write_str("SVG contains an external resource or active content")
            }
            Self::InvalidFont => f.write_str("private font payload is not a supported font"),
            Self::InvalidFontPath => {
                f.write_str("private font path is not an explicit regular file")
            }
            Self::FontRead(error) => write!(f, "private font could not be loaded: {error}"),
            Self::Svg(error) => write!(f, "SVG conversion failed: {error}"),
            Self::InvalidSize => f.write_str("SVG has no finite positive size"),
            Self::DimensionLimit => f.write_str("SVG dimensions exceed the raster budget"),
            Self::PixelLimit => f.write_str("SVG pixel count exceeds the raster budget"),
            Self::PixmapAllocation => f.write_str("bounded pixmap allocation failed"),
            Self::PngEncoding(error) => write!(f, "PNG encoding failed: {error}"),
            Self::PngTooLarge => f.write_str("PNG exceeds the output budget"),
        }
    }
}

impl std::error::Error for DesignRasterError {}

/// Render an SVG snapshot into a bounded PNG at the requested dimensions.
///
/// The font path must point to the verified private font shipped in the active
/// release. System font discovery and resource resolution are disabled.
pub fn render_svg(
    input: &[u8],
    width: u32,
    height: u32,
    font_path: &Path,
) -> Result<Vec<u8>, DesignRasterError> {
    if input.len() > MAX_SVG_BYTES {
        return Err(DesignRasterError::InputTooLarge);
    }
    if width == 0 || height == 0 {
        return Err(DesignRasterError::DimensionLimit);
    }
    if width > MAX_WIDTH || height > MAX_HEIGHT {
        return Err(DesignRasterError::DimensionLimit);
    }
    let pixels = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or(DesignRasterError::PixelLimit)?;
    if pixels > MAX_PIXELS {
        return Err(DesignRasterError::PixelLimit);
    }
    let font_path = verify_font_path(font_path)?;
    if input.starts_with(&[0x1f, 0x8b]) {
        return Err(DesignRasterError::CompressedSvgRejected);
    }
    let text = std::str::from_utf8(input).map_err(|_| DesignRasterError::InvalidUtf8)?;
    let document = validate_svg(text)?;

    let mut options = usvg::Options {
        resources_dir: None,
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: Box::new(|_, _, _| None),
            resolve_string: Box::new(|_, _| None),
        },
        fontdb: Arc::new(usvg::fontdb::Database::new()),
        ..usvg::Options::default()
    };
    let private_family = {
        let database = options.fontdb_mut();
        let before = database.faces().count();
        database
            .load_font_file(&font_path)
            .map_err(|error| DesignRasterError::FontRead(error.to_string()))?;
        if database.faces().count() == before {
            return Err(DesignRasterError::InvalidFont);
        }
        database
            .faces()
            .next()
            .and_then(|face| face.families.first())
            .map(|(family, _)| family.clone())
            .ok_or(DesignRasterError::InvalidFont)?
    };
    options.font_family = private_family;

    let tree = usvg::Tree::from_xmltree(&document, &options)
        .map_err(|error| DesignRasterError::Svg(error.to_string()))?;
    let size = tree.size();
    let source_width = finite_dimension(size.width()).ok_or(DesignRasterError::InvalidSize)?;
    let source_height = finite_dimension(size.height()).ok_or(DesignRasterError::InvalidSize)?;
    if source_width == 0 || source_height == 0 {
        return Err(DesignRasterError::InvalidSize);
    }
    let scale_x = width as f32 / source_width as f32;
    let scale_y = height as f32 / source_height as f32;
    let scale = scale_x.min(scale_y);
    if !scale.is_finite() || scale <= 0.0 {
        return Err(DesignRasterError::InvalidSize);
    }
    let offset_x = (width as f32 - source_width as f32 * scale) / 2.0;
    let offset_y = (height as f32 - source_height as f32 * scale) / 2.0;
    let mut pixmap =
        tiny_skia::Pixmap::new(width, height).ok_or(DesignRasterError::PixmapAllocation)?;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale).post_translate(offset_x, offset_y),
        &mut pixmap.as_mut(),
    );
    let png = pixmap
        .encode_png()
        .map_err(|error| DesignRasterError::PngEncoding(error.to_string()))?;
    if png.len() > MAX_PNG_BYTES {
        return Err(DesignRasterError::PngTooLarge);
    }
    Ok(png)
}

fn verify_font_path(path: &Path) -> Result<std::path::PathBuf, DesignRasterError> {
    if !path.is_absolute()
        || path.to_str().is_none_or(|value| value.len() > 4096)
        || path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(DesignRasterError::InvalidFontPath);
    }
    let mut ancestor = path;
    while let Some(parent) = ancestor.parent() {
        let metadata =
            fs::symlink_metadata(ancestor).map_err(|_| DesignRasterError::InvalidFontPath)?;
        if metadata.file_type().is_symlink() {
            return Err(DesignRasterError::InvalidFontPath);
        }
        if parent == ancestor {
            break;
        }
        ancestor = parent;
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| DesignRasterError::InvalidFontPath)?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_FONT_BYTES as u64 {
        return Err(DesignRasterError::InvalidFontPath);
    }
    Ok(path.to_path_buf())
}

fn finite_dimension(value: f32) -> Option<u32> {
    if !value.is_finite() || value <= 0.0 {
        return None;
    }
    let rounded = value.ceil();
    if rounded > u32::MAX as f32 {
        None
    } else {
        Some(rounded as u32)
    }
}

fn validate_svg(text: &str) -> Result<usvg::roxmltree::Document<'_>, DesignRasterError> {
    let lower = text.to_ascii_lowercase();
    if lower.contains("<!doctype")
        || lower.contains("<!entity")
        || lower.contains("<?xml-stylesheet")
    {
        return Err(DesignRasterError::DtdRejected);
    }
    if text.as_bytes().contains(&0) {
        return Err(DesignRasterError::UnsafeContent);
    }

    let document = usvg::roxmltree::Document::parse_with_options(
        text,
        usvg::roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: MAX_ELEMENTS,
            entity_resolver: None,
        },
    )
    .map_err(|error| DesignRasterError::Xml(error.to_string()))?;

    let mut elements = 0_u32;
    for node in document.descendants() {
        if node.is_element() {
            elements = elements.saturating_add(1);
            if elements > MAX_ELEMENTS {
                return Err(DesignRasterError::ElementLimit);
            }
            if node.ancestors().count() > MAX_DEPTH {
                return Err(DesignRasterError::NestingLimit);
            }
            let tag = node.tag_name();
            if let Some(namespace) = tag.namespace()
                && namespace != SVG_NAMESPACE
            {
                return Err(DesignRasterError::UnsafeContent);
            }
            let name = tag.name();
            if !admitted_element(name) {
                return Err(DesignRasterError::UnsupportedElement(name.to_owned()));
            }
            for attribute in node.attributes() {
                validate_attribute(attribute.name(), attribute.namespace(), attribute.value())?;
            }
            if name == "style" && node.text().is_some_and(unsafe_text) {
                return Err(DesignRasterError::UnsafeContent);
            }
        } else if node.is_text() && node.text().is_some_and(unsafe_text) {
            return Err(DesignRasterError::UnsafeContent);
        }
    }
    let root = document.root_element();
    if root.tag_name().name() != "svg" {
        return Err(DesignRasterError::Xml(
            "SVG document root must be <svg>".into(),
        ));
    }
    Ok(document)
}

fn admitted_element(name: &str) -> bool {
    matches!(
        name,
        "svg"
            | "g"
            | "defs"
            | "path"
            | "rect"
            | "circle"
            | "ellipse"
            | "line"
            | "polyline"
            | "polygon"
            | "text"
            | "tspan"
            | "style"
            | "title"
            | "desc"
            | "metadata"
            | "use"
            | "linearGradient"
            | "radialGradient"
            | "stop"
            | "clipPath"
            | "mask"
            | "marker"
            | "pattern"
            | "filter"
            | "feBlend"
            | "feColorMatrix"
            | "feComponentTransfer"
            | "feComposite"
            | "feConvolveMatrix"
            | "feDiffuseLighting"
            | "feDisplacementMap"
            | "feFlood"
            | "feGaussianBlur"
            | "feMerge"
            | "feMergeNode"
            | "feMorphology"
            | "feOffset"
            | "feSpecularLighting"
            | "feTile"
            | "feTurbulence"
            | "feDistantLight"
            | "fePointLight"
            | "feSpotLight"
    ) && name != "feImage"
}

fn validate_attribute(
    name: &str,
    namespace: Option<&str>,
    value: &str,
) -> Result<(), DesignRasterError> {
    if namespace.is_some_and(|value| value != XLINK_NAMESPACE && value != XML_NAMESPACE) {
        return Err(DesignRasterError::UnsafeContent);
    }
    let lower_name = name.to_ascii_lowercase();
    if (lower_name.starts_with("on") || lower_name == "href" || lower_name == "xlink:href")
        && !value.trim_start().starts_with('#')
    {
        return Err(DesignRasterError::UnsafeContent);
    }
    if unsafe_text(value) {
        return Err(DesignRasterError::UnsafeContent);
    }
    Ok(())
}

fn unsafe_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    if lower.contains("javascript:")
        || lower.contains("http://")
        || lower.contains("https://")
        || lower.contains("file://")
        || lower.contains("data:")
        || lower.contains("@import")
        || lower.contains("@font-face")
    {
        return true;
    }
    let mut rest = lower.as_str();
    while let Some(index) = rest.find("url(") {
        rest = &rest[index + 4..];
        let Some(end) = rest.find(')') else {
            return true;
        };
        let target = rest[..end].trim().trim_matches(['\'', '"']);
        if !target.starts_with('#') {
            return true;
        }
        rest = &rest[end + 1..];
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="24" viewBox="0 0 32 24"><rect width="32" height="24" fill="#102030"/><path d="M2 2h28v20H2z" fill="none" stroke="#fff"/></svg>"##;

    fn fixture_font() -> PathBuf {
        if let Some(path) = std::env::var_os("MIRKET_PRIVATE_FONT") {
            let path = PathBuf::from(path);
            if path.is_file() {
                return path;
            }
        }
        let cargo_home = std::env::var_os("CARGO_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .or_else(|| std::env::var_os("USERPROFILE"))
                    .map(|home| PathBuf::from(home).join(".cargo"))
            })
            .expect("set MIRKET_PRIVATE_FONT or provide a Cargo home");
        let source_root = cargo_home.join("registry/src");
        for index in std::fs::read_dir(source_root).expect("cargo registry source") {
            let index = index.expect("cargo registry index").path();
            let path = index.join("fontdb-0.24.0/tests/fonts/Tuffy.ttf");
            if path.is_file() {
                return path;
            }
        }
        panic!("set MIRKET_PRIVATE_FONT to a valid test font");
    }

    #[test]
    fn renders_bounded_static_vector_to_png() {
        let png = render_svg(SVG, 32, 24, &fixture_font()).expect("static vector should render");
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(png.len() <= MAX_PNG_BYTES);
    }

    #[test]
    fn preserves_source_aspect_ratio_inside_requested_canvas() {
        let source = br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50"><rect width="100" height="50" fill="#1463ff"/></svg>"##;
        let png = render_svg(source, 100, 100, &fixture_font()).unwrap();
        let image = tiny_skia::Pixmap::decode_png(&png).unwrap();
        assert_eq!(image.pixel(50, 5).unwrap().alpha(), 0);
        assert!(image.pixel(50, 50).unwrap().alpha() > 0);
        assert_eq!(image.pixel(50, 95).unwrap().alpha(), 0);
    }

    #[test]
    fn accepts_d2_style_markers_and_internal_references() {
        let d2_like = br##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32" viewBox="0 0 64 32"><style><![CDATA[.edge{stroke:#334455;fill:none}]]></style><defs><marker id="arrow" markerWidth="6" markerHeight="6" refX="5" refY="3" orient="auto"><path d="M0 0L6 3L0 6Z"/></marker></defs><path class="edge" marker-end="url(#arrow)" d="M4 16H58"/></svg>"##;
        assert!(render_svg(d2_like, 64, 32, &fixture_font()).is_ok());
    }

    #[test]
    fn rejects_external_images_and_urls_before_resvg() {
        let image = br##"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"><image href="https://example.invalid/a.png"/></svg>"##;
        assert!(matches!(
            render_svg(image, 1, 1, &fixture_font()),
            Err(DesignRasterError::UnsafeContent | DesignRasterError::UnsupportedElement(_))
        ));
        let css = br##"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"><style>.x{fill:url(http://example.invalid/x)}</style><rect class="x" width="1" height="1"/></svg>"##;
        assert!(matches!(
            render_svg(css, 1, 1, &fixture_font()),
            Err(DesignRasterError::UnsafeContent)
        ));
    }

    #[test]
    fn rejects_dtd_svgz_and_invalid_private_font() {
        let dtd =
            br##"<!DOCTYPE svg><svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"/>"##;
        assert!(matches!(
            render_svg(dtd, 1, 1, &fixture_font()),
            Err(DesignRasterError::DtdRejected)
        ));
        assert!(matches!(
            render_svg(b"\x1f\x8bnot-svg", 1, 1, &fixture_font()),
            Err(DesignRasterError::CompressedSvgRejected)
        ));
        assert!(matches!(
            render_svg(SVG, 1, 1, Path::new("relative.ttf")),
            Err(DesignRasterError::InvalidFontPath)
        ));
    }

    #[test]
    fn enforces_dimensions_before_opening_font() {
        assert!(matches!(
            render_svg(SVG, MAX_WIDTH + 1, 1, Path::new("missing.ttf")),
            Err(DesignRasterError::DimensionLimit)
        ));
        assert!(matches!(
            render_svg(SVG, 0, 1, Path::new("missing.ttf")),
            Err(DesignRasterError::DimensionLimit)
        ));
    }

    #[test]
    fn renders_text_with_explicit_private_font() {
        let text = br##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="24"><text x="1" y="18" font-size="18">x</text></svg>"##;
        let png =
            render_svg(text, 32, 24, &fixture_font()).expect("private font should render text");
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        let pixmap = tiny_skia::Pixmap::decode_png(&png).expect("rendered PNG should decode");
        assert!(
            pixmap.pixels().iter().any(|pixel| pixel.alpha() > 0),
            "private font must draw visible glyphs"
        );
    }
}
