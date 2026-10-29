//! Bounded, offline D2 diagram contract.
//!
//! This module deliberately does not spawn D2 or own a job lifecycle. The
//! worker that owns process supervision must call the validators here, build a
//! private job directory, and use the existing process owner for deadlines,
//! cancellation, output inspection and reaping.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fmt,
    fs::{self, File, Metadata, OpenOptions},
    io::{self, Read},
    path::{Component, Path, PathBuf},
};

pub const PROTOCOL_VERSION: u16 = 1;
pub const MAX_SOURCE_BYTES: usize = 1024 * 1024;
pub const MAX_OUTPUT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_TIMEOUT_MS: u64 = 60_000;
pub const MAX_EXECUTABLE_BYTES: u64 = 64 * 1024 * 1024;
pub const D2_VERSION: &str = "0.9.0";
pub const D2_MACOS_ARM64_SHA256: &str =
    "e8ba501a413bac33a4d93014c475b804de10fb2863a391583e23a8d88160e311";

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum D2Layout {
    Dagre,
    Elk,
    Tala,
}

impl D2Layout {
    pub const fn cli_value(self) -> &'static str {
        match self {
            Self::Dagre => "dagre",
            Self::Elk => "elk",
            Self::Tala => "tala",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum D2OutputFormat {
    #[default]
    Svg,
    Png,
}

impl D2OutputFormat {
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Svg => "svg",
            Self::Png => "png",
        }
    }

    pub const fn stage_name(self) -> &'static str {
        match self {
            Self::Svg => "result.svg",
            Self::Png => "result.png",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct D2RenderRequest {
    pub protocol_version: u16,
    pub release_id: String,
    pub job_id: String,
    pub job_root: PathBuf,
    pub input_path: PathBuf,
    pub source_revision: String,
    pub output_path: PathBuf,
    pub layout: D2Layout,
    pub output_format: D2OutputFormat,
    pub timeout_ms: u64,
    pub max_output_bytes: usize,
}

impl D2RenderRequest {
    pub fn validate(&self) -> Result<(), D2RequestError> {
        if self.protocol_version != PROTOCOL_VERSION {
            return Err(D2RequestError::UnsupportedProtocol);
        }
        if !exact_revision(&self.release_id) || !bounded_token(&self.job_id, 128) {
            return Err(D2RequestError::InvalidIdentity);
        }
        if !safe_absolute_path(&self.job_root)
            || !safe_absolute_path(&self.input_path)
            || !safe_absolute_path(&self.output_path)
            || !self.input_path.starts_with(&self.job_root)
            || !self.output_path.starts_with(&self.job_root)
            || self.input_path == self.output_path
        {
            return Err(D2RequestError::InvalidPath);
        }
        if self.input_path.extension().and_then(|v| v.to_str()) != Some("d2")
            || self.output_path.extension().and_then(|v| v.to_str())
                != Some(self.output_format.extension())
        {
            return Err(D2RequestError::InvalidPath);
        }
        if !exact_revision(&self.source_revision) {
            return Err(D2RequestError::InvalidRevision);
        }
        if self.timeout_ms == 0 || self.timeout_ms > MAX_TIMEOUT_MS {
            return Err(D2RequestError::InvalidTimeout);
        }
        if self.max_output_bytes == 0 || self.max_output_bytes > MAX_OUTPUT_BYTES {
            return Err(D2RequestError::InvalidOutputBudget);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum D2RequestError {
    UnsupportedProtocol,
    InvalidIdentity,
    InvalidPath,
    InvalidRevision,
    InvalidTimeout,
    InvalidOutputBudget,
    UnsafeSource,
}

impl fmt::Display for D2RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnsupportedProtocol => "unsupported D2 protocol version",
            Self::InvalidIdentity => "release and job identifiers must be bounded safe tokens",
            Self::InvalidPath => {
                "D2 paths must be absolute, canonical, job-scoped .d2 and matching .svg/.png paths"
            }
            Self::InvalidRevision => "source revision must be sha256:<lowercase 64-hex>",
            Self::InvalidTimeout => "D2 timeout must be between 1ms and 60000ms",
            Self::InvalidOutputBudget => "D2 output budget must be between 1 and 16MiB",
            Self::UnsafeSource => "D2 source contains an import or external asset reference",
        })
    }
}

impl std::error::Error for D2RequestError {}

/// Reject source constructs that could cause D2 to read an unadmitted file,
/// fetch an asset or emit a navigation link. This is intentionally conservative,
/// rather than claiming to parse the complete D2 language.
pub fn validate_source(source: &str) -> Result<(), D2RequestError> {
    if source.len() > MAX_SOURCE_BYTES || source.as_bytes().contains(&0) {
        return Err(D2RequestError::UnsafeSource);
    }
    for line in source.lines() {
        let trimmed = line.trim_start();
        let lower = trimmed.to_ascii_lowercase();
        if trimmed.starts_with("import ")
            || trimmed.starts_with("import\t")
            || trimmed.starts_with("import(")
            || lower.contains("http://")
            || lower.contains("https://")
            || lower.contains("file://")
            || lower.contains("data:")
            || lower.contains('@')
            || asset_directive(&lower)
        {
            return Err(D2RequestError::UnsafeSource);
        }
        if line
            .chars()
            .any(|ch| ch.is_control() && ch != '\t' && ch != '\r')
        {
            return Err(D2RequestError::UnsafeSource);
        }
    }
    Ok(())
}

fn asset_directive(line: &str) -> bool {
    let lower_line = line.to_ascii_lowercase();
    if [
        "icon:",
        "image:",
        "font:",
        "font-family:",
        "font_family:",
        "link:",
    ]
    .iter()
    .any(|directive| lower_line.contains(directive))
    {
        return true;
    }
    if lower_line.contains("shape:") && lower_line.contains("image") {
        return true;
    }
    let Some((key, value)) = line.split_once(':') else {
        return false;
    };
    let key = key.trim();
    let value = value.trim();
    matches!(
        key,
        "icon" | "image" | "font" | "font-family" | "font_family"
    ) || (key == "shape" && value.eq_ignore_ascii_case("image"))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct D2Executable {
    pub path: PathBuf,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Debug)]
pub enum D2ExecutableError {
    UnsupportedPlatform,
    InvalidPath,
    Missing,
    Symlink,
    NotRegularFile,
    NotExecutable,
    TooLarge,
    Io(io::Error),
    DigestMismatch {
        expected: &'static str,
        actual: String,
    },
}

impl fmt::Display for D2ExecutableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => f.write_str("D2 v0.9.0 is qualified only on macOS ARM64"),
            Self::InvalidPath => {
                f.write_str("D2 executable path must be an absolute canonical private d2 path")
            }
            Self::Missing => f.write_str("D2 executable is missing"),
            Self::Symlink => f.write_str("D2 executable or an ancestor is a symlink"),
            Self::NotRegularFile => f.write_str("D2 executable is not a regular file"),
            Self::NotExecutable => f.write_str("D2 executable has no execute permission"),
            Self::TooLarge => f.write_str("D2 executable exceeds the admitted size bound"),
            Self::Io(error) => write!(f, "D2 executable read failed: {error}"),
            Self::DigestMismatch { .. } => {
                f.write_str("D2 executable digest differs from the admitted payload")
            }
        }
    }
}

impl std::error::Error for D2ExecutableError {}

pub fn verify_private_executable(path: &Path) -> Result<D2Executable, D2ExecutableError> {
    if !cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        return Err(D2ExecutableError::UnsupportedPlatform);
    }
    verify_executable_with_pin(path, D2_MACOS_ARM64_SHA256)
}

fn verify_executable_with_pin(
    path: &Path,
    expected: &'static str,
) -> Result<D2Executable, D2ExecutableError> {
    if !safe_absolute_path(path) || path.file_name().and_then(|v| v.to_str()) != Some("d2") {
        return Err(D2ExecutableError::InvalidPath);
    }
    refuse_symlink_ancestors(path)?;
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            D2ExecutableError::Missing
        } else {
            D2ExecutableError::Io(error)
        }
    })?;
    if metadata.file_type().is_symlink() {
        return Err(D2ExecutableError::Symlink);
    }
    if !metadata.is_file() {
        return Err(D2ExecutableError::NotRegularFile);
    }
    if metadata.len() == 0 || metadata.len() > MAX_EXECUTABLE_BYTES {
        return Err(D2ExecutableError::TooLarge);
    }
    if !is_executable(&metadata) {
        return Err(D2ExecutableError::NotExecutable);
    }
    let mut file = open_nofollow(path).map_err(D2ExecutableError::Io)?;
    let mut digest = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(D2ExecutableError::Io)?;
        if read == 0 {
            break;
        }
        bytes += read as u64;
        if bytes > MAX_EXECUTABLE_BYTES {
            return Err(D2ExecutableError::TooLarge);
        }
        digest.update(&buffer[..read]);
    }
    let actual: String = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if actual != expected {
        return Err(D2ExecutableError::DigestMismatch { expected, actual });
    }
    Ok(D2Executable {
        path: path.to_path_buf(),
        sha256: actual,
        bytes,
    })
}

fn open_nofollow(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    options.open(path)
}

fn refuse_symlink_ancestors(path: &Path) -> Result<(), D2ExecutableError> {
    for ancestor in path.ancestors() {
        let metadata = fs::symlink_metadata(ancestor).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                D2ExecutableError::Missing
            } else {
                D2ExecutableError::Io(error)
            }
        })?;
        if metadata.file_type().is_symlink() {
            return Err(D2ExecutableError::Symlink);
        }
    }
    Ok(())
}

fn is_executable(metadata: &Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        true
    }
}

fn bounded_token(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
}

fn exact_revision(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn safe_absolute_path(path: &Path) -> bool {
    path.is_absolute()
        && path
            .to_str()
            .is_some_and(|value| !value.is_empty() && value.len() <= 4096 && !value.contains('\0'))
        && path
            .components()
            .all(|component| matches!(component, Component::RootDir | Component::Normal(_)))
}

/// The direct D2 PNG route uses the existing raster allocation budget. Check
/// the complete chunk envelope before the existing decoder allocates pixels.
/// Only native D2's RGB/RGBA, non-interlaced PNG and bounded EXIF are admitted;
/// compressed metadata and animated/unknown chunks cannot reach the decoder.
pub fn validate_png(bytes: &[u8]) -> Result<(), &'static str> {
    if bytes.len() > MAX_OUTPUT_BYTES || bytes.len() < 45 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("D2 PNG signature/byte budget is invalid");
    }
    let mut cursor = 8usize;
    let mut dimensions = None;
    let mut saw_data = false;
    let mut saw_exif = false;
    while cursor < bytes.len() {
        if bytes.len() - cursor < 12 {
            return Err("D2 PNG chunk is truncated");
        }
        let length = u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().unwrap()) as usize;
        let end = cursor
            .checked_add(12)
            .and_then(|v| v.checked_add(length))
            .filter(|v| *v <= bytes.len())
            .ok_or("D2 PNG chunk extent is invalid")?;
        let kind = &bytes[cursor + 4..cursor + 8];
        let data = &bytes[cursor + 8..end - 4];
        let crc = u32::from_be_bytes(bytes[end - 4..end].try_into().unwrap());
        if png_crc(&bytes[cursor + 4..end - 4]) != crc {
            return Err("D2 PNG chunk CRC is invalid");
        }
        if dimensions.is_none() && kind != b"IHDR" {
            return Err("D2 PNG must start with IHDR");
        }
        match kind {
            b"IHDR" if cursor == 8 && length == 13 => {
                let width = u32::from_be_bytes(data[..4].try_into().unwrap());
                let height = u32::from_be_bytes(data[4..8].try_into().unwrap());
                if width == 0
                    || height == 0
                    || width > crate::design_raster::MAX_WIDTH
                    || height > crate::design_raster::MAX_HEIGHT
                    || u64::from(width) * u64::from(height) > crate::design_raster::MAX_PIXELS
                    || data[8] != 8
                    || !matches!(data[9], 2 | 6)
                    || data[10..13] != [0, 0, 0]
                {
                    return Err("D2 PNG dimensions or pixel format exceed the raster contract");
                }
                dimensions = Some((width, height));
            }
            b"eXIf" if !saw_data && !saw_exif && length <= 1024 => saw_exif = true,
            b"IDAT" => saw_data = true,
            b"IEND" if length == 0 && end == bytes.len() && saw_data => {
                let decoded = resvg::tiny_skia::Pixmap::decode_png(bytes)
                    .map_err(|_| "D2 PNG pixel decoding failed")?;
                if Some((decoded.width(), decoded.height())) != dimensions {
                    return Err("D2 PNG decoded dimensions changed");
                }
                return Ok(());
            }
            _ => return Err("D2 PNG chunk is unsupported or out of order"),
        }
        cursor = end;
    }
    Err("D2 PNG end/data is missing")
}

fn png_crc(bytes: &[u8]) -> u32 {
    const TABLE: [u32; 256] = {
        let mut table = [0; 256];
        let mut index = 0;
        while index < 256 {
            let mut crc = index as u32;
            let mut bit = 0;
            while bit < 8 {
                crc = (crc >> 1) ^ (0xedb88320 & (0u32.wrapping_sub(crc & 1)));
                bit += 1;
            }
            table[index] = crc;
            index += 1;
        }
        table
    };
    !bytes.iter().fold(u32::MAX, |crc, byte| {
        TABLE[((crc ^ u32::from(*byte)) & 255) as usize] ^ (crc >> 8)
    })
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum D2RenderResult {
    Staged {
        output_path: PathBuf,
        output_revision: String,
        output_bytes: usize,
        layout: D2Layout,
        output_format: D2OutputFormat,
    },
    Refused {
        reason: D2Refusal,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum D2Refusal {
    SourceChanged,
    OutputMissing,
    OutputTooLarge,
    InvalidSvg,
    UnsupportedSource,
    Deadline,
    ProcessFailed,
}

/// Process-owner evidence returned alongside the typed result. The worker
/// fills this after its existing supervisor has reaped D2; this module never
/// creates or controls that process.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct D2RenderOutcome {
    pub result: D2RenderResult,
    pub stderr_bytes: usize,
    pub cleanup_confirmed: bool,
}

pub fn validate_result(
    request: &D2RenderRequest,
    result: &D2RenderResult,
) -> Result<(), D2RequestError> {
    request.validate()?;
    if let D2RenderResult::Staged {
        output_path,
        output_revision,
        output_bytes,
        layout,
        output_format,
    } = result
        && (output_path != &request.output_path
            || layout != &request.layout
            || output_format != &request.output_format
            || *output_bytes == 0
            || *output_bytes > request.max_output_bytes
            || !exact_revision(output_revision))
    {
        return Err(D2RequestError::InvalidPath);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn request() -> D2RenderRequest {
        D2RenderRequest {
            protocol_version: PROTOCOL_VERSION,
            release_id: format!("sha256:{}", "b".repeat(64)),
            job_id: "job-1".into(),
            job_root: PathBuf::from("/private/job"),
            input_path: PathBuf::from("/private/job/source.d2"),
            source_revision: format!("sha256:{}", "a".repeat(64)),
            output_path: PathBuf::from("/private/job/result.svg"),
            layout: D2Layout::Dagre,
            output_format: D2OutputFormat::Svg,
            timeout_ms: 10_000,
            max_output_bytes: 1024,
        }
    }

    #[test]
    fn request_is_finite_and_job_scoped() {
        assert!(request().validate().is_ok());
        let mut invalid = request();
        invalid.output_path = PathBuf::from("/tmp/result.svg");
        assert_eq!(invalid.validate(), Err(D2RequestError::InvalidPath));
    }

    #[test]
    fn output_format_requires_matching_extension_and_rejects_unknown_values() {
        let mut png = request();
        png.output_format = D2OutputFormat::Png;
        assert_eq!(png.validate(), Err(D2RequestError::InvalidPath));
        png.output_path.set_extension("png");
        assert!(png.validate().is_ok());
        let mut value = serde_json::to_value(&png).unwrap();
        value["output_format"] = serde_json::json!("pdf");
        assert!(serde_json::from_value::<D2RenderRequest>(value).is_err());
        let staged = D2RenderResult::Staged {
            output_path: png.output_path.clone(),
            output_revision: png.source_revision.clone(),
            output_bytes: 1,
            layout: png.layout,
            output_format: D2OutputFormat::Svg,
        };
        assert!(validate_result(&png, &staged).is_err());
    }

    fn png_fixture() -> Vec<u8> {
        resvg::tiny_skia::Pixmap::new(2, 3)
            .unwrap()
            .encode_png()
            .unwrap()
    }

    fn repair_chunk_crc(bytes: &mut [u8], cursor: usize) {
        let length = u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().unwrap()) as usize;
        let end = cursor + length + 12;
        let crc = png_crc(&bytes[cursor + 4..end - 4]);
        bytes[end - 4..end].copy_from_slice(&crc.to_be_bytes());
    }

    #[test]
    fn png_checks_crc_dimensions_envelope_and_actual_pixels() {
        let valid = png_fixture();
        assert_eq!(validate_png(&valid), Ok(()));
        let mut bad_crc = valid.clone();
        bad_crc[29] ^= 1;
        assert!(validate_png(&bad_crc).unwrap_err().contains("CRC"));
        let mut oversized = valid.clone();
        oversized[16..20].copy_from_slice(&2049u32.to_be_bytes());
        repair_chunk_crc(&mut oversized, 8);
        assert!(validate_png(&oversized).unwrap_err().contains("dimensions"));
        let mut trailing = valid.clone();
        trailing.push(0);
        assert!(validate_png(&trailing).is_err());
        assert!(validate_png(&valid[..valid.len() - 1]).is_err());
        let mut corrupt_data = valid.clone();
        let mut cursor = 33;
        while &corrupt_data[cursor + 4..cursor + 8] != b"IDAT" {
            cursor += u32::from_be_bytes(corrupt_data[cursor..cursor + 4].try_into().unwrap())
                as usize
                + 12;
        }
        corrupt_data[cursor + 8] = 0; // Invalid zlib header, but a valid chunk CRC.
        repair_chunk_crc(&mut corrupt_data, cursor);
        assert_eq!(
            validate_png(&corrupt_data),
            Err("D2 PNG pixel decoding failed")
        );
        let mut unknown_chunk = valid.clone();
        unknown_chunk[12..16].copy_from_slice(b"acTL");
        repair_chunk_crc(&mut unknown_chunk, 8);
        assert!(validate_png(&unknown_chunk).is_err());
    }

    #[test]
    fn source_rejects_imports_and_external_assets_but_allows_labels() {
        assert!(validate_source("client: { label: \"https://example.invalid\" }").is_err());
        assert_eq!(validate_source("client: { label: \"plain text\" }"), Ok(()));
        assert_eq!(
            validate_source("import \"other.d2\""),
            Err(D2RequestError::UnsafeSource)
        );
        assert_eq!(
            validate_source("client: { image: \"x.png\" }"),
            Err(D2RequestError::UnsafeSource)
        );
        for source in [
            "client: @./other.d2",
            "client: { link: ./other.svg }",
            "client: { shape: image }",
        ] {
            assert_eq!(validate_source(source), Err(D2RequestError::UnsafeSource));
        }
        assert_eq!(
            validate_source("client: @file x.png"),
            Err(D2RequestError::UnsafeSource)
        );
    }

    #[cfg(unix)]
    #[test]
    fn executable_verifier_rejects_wrong_bytes_and_symlink() {
        use std::{
            os::unix::fs::PermissionsExt,
            time::{SystemTime, UNIX_EPOCH},
        };
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_root = if Path::new("/private/tmp").is_dir() {
            PathBuf::from("/private/tmp")
        } else {
            std::env::temp_dir()
        };
        let root = temp_root.join(format!("mirket-d2-test-{stamp}"));
        fs::create_dir(&root).unwrap();
        let path = root.join("d2");
        fs::write(&path, b"wrong").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        assert!(matches!(
            verify_executable_with_pin(
                &path,
                "0000000000000000000000000000000000000000000000000000000000000000"
            ),
            Err(D2ExecutableError::DigestMismatch { .. })
        ));
        let link = root.join("link");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        let alias = root.join("alias");
        std::os::unix::fs::symlink(&root, &alias).unwrap();
        assert!(matches!(
            verify_executable_with_pin(&alias.join("d2"), D2_MACOS_ARM64_SHA256),
            Err(D2ExecutableError::Symlink)
        ));
        let named_link = root.join("d2-link");
        std::os::unix::fs::symlink(&path, &named_link).unwrap();
        assert!(matches!(
            verify_executable_with_pin(&named_link, D2_MACOS_ARM64_SHA256),
            Err(D2ExecutableError::InvalidPath)
        ));
        fs::remove_dir_all(root).unwrap();
    }
}
