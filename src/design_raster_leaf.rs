//! Hidden same-binary raster leaf. Its parent worker owns the process group.

use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

const FONT_SHA256: &str = "bfb7bb691513f12e734dc346c03a03f784912432d7e3fa8e56efcf906fe86b3d";
const FONT_BYTES: u64 = 2_049_096;

pub fn verify_font(path: &Path) -> Result<(), String> {
    if !path.is_absolute() || path.file_name().and_then(|v| v.to_str()) != Some("NotoSans.ttf") {
        return Err("private design font path is invalid".into());
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| "private design font is unavailable")?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() != FONT_BYTES {
        return Err("private design font is not the admitted regular file".into());
    }
    let bytes = fs::read(path).map_err(|_| "private design font could not be read")?;
    let digest: String = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if bytes.len() as u64 != FONT_BYTES || digest != FONT_SHA256 {
        return Err("private design font hash differs from admission".into());
    }
    Ok(())
}

pub fn serve_raster_leaf() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 7 || args[1] != "__design_raster" {
        return Err("internal design raster mode requires exact arguments".into());
    }
    let input = PathBuf::from(&args[2]);
    let output = PathBuf::from(&args[3]);
    let font = PathBuf::from(&args[4]);
    let width: u32 = args[5]
        .to_string_lossy()
        .parse()
        .map_err(|_| "invalid render width")?;
    let height: u32 = args[6]
        .to_string_lossy()
        .parse()
        .map_err(|_| "invalid render height")?;
    if width == 0
        || height == 0
        || width > crate::design::MAX_DIMENSION
        || height > crate::design::MAX_DIMENSION
    {
        return Err("render dimensions exceed their bound".into());
    }
    let parent = input.parent().ok_or("render source parent is missing")?;
    if parent != output.parent().ok_or("render stage parent is missing")?
        || input.extension().and_then(|v| v.to_str()) != Some("svg")
        || output.extension().and_then(|v| v.to_str()) != Some("png")
        || input == output
    {
        return Err("render source and stage must be private .svg/.png siblings".into());
    }
    crate::process::validate_job_dir(parent)?;
    verify_font(&font)?;
    let metadata = fs::symlink_metadata(&input).map_err(|_| "render source is unavailable")?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > crate::design::MAX_INPUT_BYTES as u64
    {
        return Err("render source must be a bounded regular file".into());
    }
    let source = fs::read(&input).map_err(|_| "render source could not be read")?;
    if source.len() > crate::design::MAX_INPUT_BYTES {
        return Err("render source exceeds its byte limit".into());
    }
    let png = crate::design_raster::render_svg(&source, width, height, &font)
        .map_err(|error| error.to_string())?;
    if png.is_empty() || png.len() > crate::design::MAX_OUTPUT_BYTES {
        return Err("rendered PNG exceeds its output bound".into());
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(&output)
        .map_err(|_| "render stage must be absent and writable")?;
    file.write_all(&png)
        .and_then(|_| file.sync_all())
        .map_err(|_| "render stage write failed")?;
    Ok(())
}
