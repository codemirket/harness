//! One finite video Node leaf. Explicit private payloads, bounded protocol and
//! owned group cleanup; publication belongs exclusively to ArtifactPlan.
#[cfg(test)]
use crate::engine::revision;
use crate::process::{OwnedChild, cancelled, drain_stderr, read_frame, validate_job_dir};
use crate::video::*;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::Path,
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    process::Command,
};

pub fn file_identity(path: &Path) -> Result<String, String> {
    crate::owned_scratch::explicit_path(path, false)?;
    let mut file = fs::File::open(path).map_err(|_| "video payload cannot be opened")?;
    let before = file
        .metadata()
        .map_err(|_| "video payload metadata unavailable")?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = file
            .read(&mut buffer)
            .map_err(|_| "video payload read failed")?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    let after = fs::symlink_metadata(path).map_err(|_| "video payload disappeared")?;
    if before.len() != after.len()
        || before.modified().ok() != after.modified().ok()
        || after.file_type().is_symlink()
    {
        return Err("video payload changed during inspection".into());
    }
    Ok(format!(
        "sha256:{}",
        hash.finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    ))
}
pub fn payload_identity(payload: &VideoPayload) -> Result<Vec<String>, String> {
    [
        &payload.node,
        &payload.adapter,
        &payload.chromium,
        &payload.ffmpeg,
        &payload.ffprobe,
    ]
    .into_iter()
    .map(|p| file_identity(p))
    .collect()
}

#[derive(Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum Startup {
    #[serde(rename = "ready")]
    Ready {
        protocol_version: u32,
        release_id: String,
        operation: String,
        policy_version: u32,
        max_frame_bytes: usize,
    },
    #[serde(rename = "error")]
    Error {
        protocol_version: u32,
        release_id: String,
        job_id: Option<String>,
        code: String,
        output_may_exist: bool,
    },
}

fn validate_startup(frame: &[u8], release_id: &str) -> Result<(), String> {
    let startup: Startup = serde_json::from_slice(frame).map_err(|_| "invalid video readiness")?;
    match startup {
        Startup::Ready {
            protocol_version,
            release_id: observed_release,
            operation,
            policy_version,
            max_frame_bytes,
        } if protocol_version == 1
            && observed_release == release_id
            && operation == "video.export"
            && policy_version == 1
            && max_frame_bytes == 65536 =>
        {
            Ok(())
        }
        Startup::Ready { .. } => Err("video readiness differs from private patched release".into()),
        Startup::Error {
            protocol_version,
            release_id: observed_release,
            job_id,
            code,
            output_may_exist,
        } if protocol_version == 1
            && observed_release == release_id
            && job_id.is_none()
            && !output_may_exist
            && !code.is_empty()
            && code.len() <= 64
            && code
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') =>
        {
            Err(format!("video adapter refused at startup ({code})"))
        }
        Startup::Error { .. } => Err("video startup refusal identity/shape differs".into()),
    }
}
#[derive(Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum Reply {
    #[serde(rename = "result")]
    Result {
        protocol_version: u32,
        release_id: String,
        job_id: String,
        status: String,
        output_path: std::path::PathBuf,
        output_revision: String,
        output_bytes: usize,
        evidence: Value,
    },
    #[serde(rename = "error")]
    Error {
        protocol_version: u32,
        release_id: String,
        job_id: Option<String>,
        code: String,
        output_may_exist: bool,
        #[serde(default)]
        diagnostics: Vec<VideoDiagnostic>,
    },
}

#[derive(Debug)]
pub struct VideoAdapterError {
    pub message: String,
    pub cleanup_confirmed: bool,
    pub diagnostics: Vec<VideoDiagnostic>,
}
impl std::fmt::Display for VideoAdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for VideoAdapterError {}
impl From<String> for VideoAdapterError {
    fn from(message: String) -> Self {
        Self {
            message,
            cleanup_confirmed: true,
            diagnostics: vec![],
        }
    }
}
impl From<&str> for VideoAdapterError {
    fn from(message: &str) -> Self {
        message.to_owned().into()
    }
}
fn unknown(message: impl Into<String>) -> VideoAdapterError {
    VideoAdapterError {
        message: message.into(),
        cleanup_confirmed: false,
        diagnostics: vec![],
    }
}

fn validate_diagnostics(items: &[VideoDiagnostic], request: &VideoRequest) -> Result<(), String> {
    let identifier = |value: &str, max: usize| {
        !value.is_empty()
            && value.len() <= max
            && value
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'.'))
    };
    if items.len() > 16
        || serde_json::to_vec(items)
            .map_err(|_| "invalid video diagnostics")?
            .len()
            > 8192
    {
        return Err("video diagnostics exceed their bounded contract".into());
    }
    for item in items {
        if !matches!(
            item.phase.as_str(),
            "source" | "reference" | "root" | "font" | "caption"
        ) || !relative(&item.path)
            || !request
                .sources
                .iter()
                .any(|source| source.path == item.path)
            || !identifier(&item.field, 64)
            || !identifier(&item.reason, 100)
            || item.expected.is_empty()
            || item.expected.len() > 256
            || item.expected.chars().any(char::is_control)
            || item
                .cue_index
                .is_some_and(|index| index >= 1000 || item.phase != "caption")
        {
            return Err("video diagnostic location/shape differs from selected sources".into());
        }
    }
    Ok(())
}

pub async fn run_video_adapter(
    payload: &VideoPayload,
    request: &VideoRequest,
    cancel: Arc<AtomicBool>,
    deadline: tokio::time::Instant,
) -> Result<VideoOutcome, VideoAdapterError> {
    if !cfg!(target_os = "macos") {
        return Err("video descendant ownership is qualified only for the macOS development route; Windows admission remains blocked".into());
    }
    request.profile.validate()?;
    crate::release::ReleaseId::parse(&request.release_id).map_err(|e| e.to_string())?;
    crate::release::ReleaseId::parse(&request.job_id).map_err(|e| e.to_string())?;
    validate_job_dir(&request.job_root)?;
    payload_identity(payload)?;
    if request.protocol_version != 1
        || request.sources.is_empty()
        || request.sources.len() > MAX_FILES
        || !relative(&request.entry_file)
        || request.input_root != request.job_root.join("source")
        || request.output_path.parent() != Some(request.job_root.as_path())
        || request.timeout_ms == 0
        || request.timeout_ms > MAX_TIMEOUT_MS
        || request.max_output_bytes == 0
        || request.max_output_bytes > MAX_OUTPUT_BYTES
        || request.output_path.extension().and_then(|v| v.to_str())
            != Some(request.profile.format.extension())
    {
        return Err("video request does not match the finite stage contract".into());
    }
    match fs::symlink_metadata(&request.output_path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        _ => return Err("video output stage must be fresh".into()),
    }
    if cancel.load(Ordering::Acquire) || tokio::time::Instant::now() >= deadline {
        return Err("video cancelled/expired before launch".into());
    }
    let mut frame = serde_json::to_vec(request).map_err(|_| "video request encoding failed")?;
    frame.push(b'\n');
    if frame.len() > 65536 {
        return Err("video request metadata exceeds protocol budget".into());
    }
    let mut command = Command::new(&payload.node);
    command
        .arg(&payload.adapter)
        .arg("--release-id")
        .arg(&request.release_id)
        .arg("--chromium")
        .arg(&payload.chromium)
        .arg("--ffmpeg")
        .arg(&payload.ffmpeg)
        .arg("--ffprobe")
        .arg(&payload.ffprobe)
        .env_clear()
        .env("HOME", &request.job_root)
        .env("TMPDIR", &request.job_root)
        .env("TMP", &request.job_root)
        .env("TEMP", &request.job_root)
        .env("XDG_CACHE_HOME", request.job_root.join("cache"))
        .env("XDG_CONFIG_HOME", request.job_root.join("config"))
        .current_dir(&request.job_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.as_std_mut().process_group(0);
    }
    let mut child = OwnedChild::new(
        command
            .spawn()
            .map_err(|e| format!("video adapter launch failed: {e}"))?,
    )
    .map_err(unknown)?;
    let (Some(mut stdin), Some(stdout), Some(stderr)) = (
        child.child.stdin.take(),
        child.child.stdout.take(),
        child.child.stderr.take(),
    ) else {
        return match child.terminate_and_reap().await {
            Ok(()) => Err("video pipes unavailable".into()),
            Err(e) => Err(unknown(e)),
        };
    };
    let exchange = async {
        let protocol = async {
            let mut stdout = BufReader::new(stdout);
            let ready =
                tokio::time::timeout(std::time::Duration::from_secs(15), read_frame(&mut stdout))
                    .await
                    .map_err(|_| "video readiness deadline exceeded")??;
            validate_startup(&ready, &request.release_id)?;
            stdin
                .write_all(&frame)
                .await
                .map_err(|_| "video request write failed")?;
            let response = read_frame(&mut stdout).await?;
            let reply: Reply =
                serde_json::from_slice(&response).map_err(|_| "invalid video completion")?;
            let result = match reply {
                Reply::Result {
                    protocol_version,
                    release_id,
                    job_id,
                    status,
                    output_path,
                    output_revision,
                    output_bytes,
                    evidence,
                } => {
                    if protocol_version != 1
                        || release_id != request.release_id
                        || job_id != request.job_id
                        || status != "staged"
                        || output_path != request.output_path
                        || !crate::design::is_revision(&output_revision)
                        || output_bytes == 0
                        || output_bytes > request.max_output_bytes
                        || !evidence.is_object()
                    {
                        return Err("video completion identity/stage/bounds differ".into());
                    }
                    VideoOutcome {
                        output_revision,
                        output_bytes,
                        evidence,
                        cleanup_confirmed: true,
                    }
                }
                Reply::Error {
                    protocol_version,
                    release_id,
                    job_id,
                    code,
                    output_may_exist,
                    diagnostics,
                } => {
                    if protocol_version != 1
                        || release_id != request.release_id
                        || job_id.as_deref() != Some(&request.job_id)
                        || code.is_empty()
                        || code.len() > 100
                        || !code.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
                    {
                        return Err("video error identity differs".into());
                    }
                    validate_diagnostics(&diagnostics, request)?;
                    return Err(VideoAdapterError {
                        message: format!(
                            "video adapter refused ({code}); private stage may exist: {output_may_exist}"
                        ),
                        cleanup_confirmed: true,
                        diagnostics,
                    });
                }
            };
            let mut trailing = [0u8; 1];
            if stdout
                .read(&mut trailing)
                .await
                .map_err(|_| "video stdout close failed")?
                != 0
            {
                return Err("unexpected trailing video stdout".into());
            }
            Ok(result)
        };
        let (outcome, stderr) = tokio::try_join!(protocol, async {
            drain_stderr(stderr).await.map_err(VideoAdapterError::from)
        })?;
        if stderr > 16384 {
            return Err("video diagnostics exceed their private byte budget".into());
        }
        // Keep the unreaped leader while cleaning its group: Chromium/media
        // must remain non-detached under the exact checked policy patch.
        Ok(outcome)
    };
    let result: Result<VideoOutcome, VideoAdapterError> = tokio::select! {biased;_ = cancelled(cancel.clone())=>Err("video cancelled".into()),value=tokio::time::timeout_at(deadline,exchange)=>value.unwrap_or_else(|_|Err("video deadline exceeded".into()))};
    if let Err(e) = child.terminate_and_reap().await {
        return Err(unknown(format!(
            "video owned-group cleanup unconfirmed: {e}"
        )));
    }
    let outcome = result?;
    let root = request.job_root.clone();
    let path = request.output_path.clone();
    let max_bytes = request.max_output_bytes;
    let profile = request.profile.clone();
    let expected_revision = outcome.output_revision.clone();
    let expected_bytes = outcome.output_bytes;
    tokio::task::spawn_blocking(move || {
        let keep_running =
            || !cancel.load(Ordering::Acquire) && tokio::time::Instant::now() < deadline;
        let output = crate::artifacts::VideoFileRevision::inspect_checked(
            &root,
            &path,
            max_bytes,
            &keep_running,
        )
        .map_err(|e| e.to_string())?;
        if output.byte_len != expected_bytes || output.revision != expected_revision {
            return Err("video artifact bytes differ from completed result".to_owned());
        }
        validate_artifact_file(&output, &profile, &keep_running)
    })
    .await
    .map_err(|_| "video file verification worker failed")??;
    Ok(outcome)
}

/// Independent streaming container/archive check after the adapter's decode.
pub(crate) fn validate_artifact_file(
    output: &crate::artifacts::VideoFileRevision,
    profile: &VideoProfile,
    keep_running: &impl Fn() -> bool,
) -> Result<(), String> {
    let mut file = output.open_reader().map_err(|e| e.to_string())?;
    let mut reader = CheckedReader {
        reader: &mut file,
        keep_running,
    };
    validate_reader(&mut reader, output.byte_len, profile)?;
    output.verify_reader(&file).map_err(|e| e.to_string())
}
struct CheckedReader<'a, R, F> {
    reader: R,
    keep_running: &'a F,
}
impl<R: Read, F: Fn() -> bool> Read for CheckedReader<'_, R, F> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if !(self.keep_running)() {
            return Err(std::io::Error::other(
                "video cancelled or deadline exceeded",
            ));
        }
        self.reader.read(buffer)
    }
}

#[cfg(test)]
fn validate_artifact(bytes: &[u8], profile: &VideoProfile) -> Result<(), String> {
    validate_reader(&mut std::io::Cursor::new(bytes), bytes.len(), profile)
}
fn validate_reader<R: Read>(
    reader: &mut R,
    len: usize,
    profile: &VideoProfile,
) -> Result<(), String> {
    if profile.format == VideoFormat::PngSequenceTar {
        return validate_tar(reader, len, profile);
    }
    let mut head = [0u8; 12];
    reader
        .read_exact(&mut head)
        .map_err(|_| "video container is truncated")?;
    let valid = match profile.format {
        VideoFormat::Mp4 => len > 12 && &head[4..8] == b"ftyp",
        VideoFormat::Webm => head.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]),
        VideoFormat::Gif => head.starts_with(b"GIF89a") || head.starts_with(b"GIF87a"),
        VideoFormat::PngSequenceTar => unreachable!(),
    };
    if !valid {
        return Err("video output container signature is invalid".into());
    }
    Ok(())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TarManifest {
    schema: u8,
    format: String,
    width: u32,
    height: u32,
    fps: FrameRate,
    frame_count: u32,
    entries: Vec<TarEntry>,
}
#[derive(Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TarEntry {
    path: String,
    bytes: usize,
    revision: String,
}
fn octal(field: &[u8]) -> Result<usize, String> {
    let s = std::str::from_utf8(field)
        .map_err(|_| "invalid tar numeric field")?
        .trim_matches(['\0', ' ']);
    usize::from_str_radix(s, 8).map_err(|_| "invalid tar number".into())
}
struct HashReader<R> {
    reader: R,
    hash: Sha256,
}
impl<R: Read> Read for HashReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let n = self.reader.read(buffer)?;
        self.hash.update(&buffer[..n]);
        Ok(n)
    }
}
fn discard<R: Read>(reader: &mut R, len: usize, zero: bool) -> Result<(), String> {
    let mut remaining = len;
    let mut buffer = [0u8; 65536];
    while remaining != 0 {
        let n = remaining.min(buffer.len());
        reader
            .read_exact(&mut buffer[..n])
            .map_err(|_| "video archive/chunk is truncated")?;
        if zero && buffer[..n].iter().any(|v| *v != 0) {
            return Err("PNG tar padding/termination is invalid".into());
        }
        remaining -= n;
    }
    Ok(())
}
fn validate_png<R: Read>(reader: &mut R, len: usize, profile: &VideoProfile) -> Result<(), String> {
    if len < 45 {
        return Err("PNG frame is truncated".into());
    }
    let mut header = [0u8; 33];
    reader
        .read_exact(&mut header)
        .map_err(|_| "PNG frame header is truncated")?;
    if &header[..8] != b"\x89PNG\r\n\x1a\n"
        || header[8..12] != 13_u32.to_be_bytes()
        || &header[12..16] != b"IHDR"
        || header[16..20] != profile.width.to_be_bytes()
        || header[20..24] != profile.height.to_be_bytes()
    {
        return Err("PNG frame dimensions/header differ from profile".into());
    }
    let mut cursor = 33usize;
    let mut saw_data = false;
    while cursor < len {
        let mut chunk = [0u8; 8];
        reader
            .read_exact(&mut chunk)
            .map_err(|_| "PNG chunk is truncated")?;
        let size = u32::from_be_bytes(chunk[..4].try_into().unwrap()) as usize;
        let end = cursor
            .checked_add(12)
            .and_then(|v| v.checked_add(size))
            .filter(|v| *v <= len)
            .ok_or("PNG chunk extent is invalid")?;
        match &chunk[4..8] {
            b"IHDR" | b"acTL" => {
                return Err("repeated or animated PNG header is unsupported".into());
            }
            b"IDAT" => saw_data = true,
            b"IEND" => {
                if size != 0 || end != len || !saw_data {
                    return Err("PNG end/data contract is invalid".into());
                }
                discard(reader, 4, false)?;
                return Ok(());
            }
            _ => (),
        }
        discard(reader, size + 4, false)?;
        cursor = end;
    }
    Err("PNG frame end is missing".into())
}
fn validate_tar<R: Read>(reader: &mut R, len: usize, profile: &VideoProfile) -> Result<(), String> {
    if !len.is_multiple_of(512) {
        return Err("PNG tar is not block aligned".into());
    }
    let mut cursor = 0usize;
    let mut entries = vec![];
    let mut names = std::collections::HashSet::new();
    let mut manifest = None;
    let mut terminated = false;
    while cursor + 512 <= len {
        let mut h = [0u8; 512];
        reader
            .read_exact(&mut h)
            .map_err(|_| "PNG tar header truncated")?;
        cursor += 512;
        if h.iter().all(|b| *b == 0) {
            if len - cursor < 512 {
                return Err("PNG tar termination is invalid".into());
            }
            discard(reader, len - cursor, true)?;
            terminated = true;
            break;
        }
        if manifest.is_some()
            || h[257..263] != *b"ustar\0"
            || h[156] != b'0'
            || h[345..500].iter().any(|b| *b != 0)
        {
            return Err("PNG tar requires ordered unprefixed regular ustar entries".into());
        }
        let checksum: usize = h
            .iter()
            .enumerate()
            .map(|(i, b)| {
                if (148..156).contains(&i) {
                    32
                } else {
                    usize::from(*b)
                }
            })
            .sum();
        if octal(&h[148..156])? != checksum {
            return Err("PNG tar checksum differs".into());
        }
        let name_end = h[..100].iter().position(|b| *b == 0).unwrap_or(100);
        let name = std::str::from_utf8(&h[..name_end])
            .map_err(|_| "PNG tar name is not UTF-8")?
            .to_owned();
        if !relative(Path::new(&name)) || name.contains('/') || !names.insert(name.clone()) {
            return Err("PNG tar entry traverses, duplicates or nests paths".into());
        }
        if names.len() > MAX_FRAMES as usize + 4 {
            return Err("PNG tar has too many entries".into());
        }
        let size = octal(&h[124..136])?;
        let padded = size
            .checked_add(511)
            .map(|v| v / 512 * 512)
            .ok_or("tar extent overflow")?;
        let end = cursor
            .checked_add(padded)
            .filter(|v| *v <= len)
            .ok_or("PNG tar entry exceeds archive")?;
        let mut data = HashReader {
            reader: reader.by_ref().take(size as u64),
            hash: Sha256::new(),
        };
        if name == "manifest.json" {
            if size > 4 * 1024 * 1024 {
                return Err("PNG tar manifest exceeds metadata budget".into());
            }
            let mut bytes = vec![0u8; size];
            data.read_exact(&mut bytes)
                .map_err(|_| "PNG tar manifest truncated")?;
            manifest = Some(
                serde_json::from_slice::<TarManifest>(&bytes)
                    .map_err(|_| "PNG tar manifest invalid")?,
            );
        } else {
            let index = entries.len();
            if index < profile.frame_count as usize {
                if name != format!("frame_{:06}.png", index + 1) {
                    return Err("PNG frame order differs from profile".into());
                }
                validate_png(&mut data, size, profile)?;
            } else {
                if !matches!(name.as_str(), "audio.m4a" | "captions.srt" | "captions.vtt") {
                    return Err("unexpected PNG sequence sidecar".into());
                }
                discard(&mut data, size, false)?;
            }
            entries.push(TarEntry {
                path: name,
                bytes: size,
                revision: format!(
                    "sha256:{}",
                    data.hash
                        .finalize()
                        .iter()
                        .map(|v| format!("{v:02x}"))
                        .collect::<String>()
                ),
            });
        }
        discard(reader, padded - size, true)?;
        cursor = end;
    }
    let manifest = manifest.ok_or("PNG tar manifest missing")?;
    if !terminated
        || manifest.schema != 1
        || manifest.format != "png_sequence_tar"
        || manifest.width != profile.width
        || manifest.height != profile.height
        || manifest.fps != profile.fps
        || manifest.frame_count != profile.frame_count
        || manifest.entries != entries
        || entries.len() < profile.frame_count as usize
    {
        return Err("PNG tar manifest differs from saved entries/prepared profile".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_refusal_reports_bounded_code_and_rejects_forged_identity() {
        let ready = serde_json::json!({"type":"ready","protocol_version":1,
            "release_id":"test:video","operation":"video.export",
            "policy_version":1,"max_frame_bytes":65536});
        assert!(validate_startup(&serde_json::to_vec(&ready).unwrap(), "test:video").is_ok());

        let error = serde_json::json!({"type":"error","protocol_version":1,
            "release_id":"test:video","job_id":null,"code":"video_export_failed",
            "output_may_exist":false});
        let encoded = serde_json::to_vec(&error).unwrap();
        assert_eq!(
            validate_startup(&encoded, "test:video").unwrap_err(),
            "video adapter refused at startup (video_export_failed)"
        );
        assert_eq!(
            validate_startup(&encoded, "other:release").unwrap_err(),
            "video startup refusal identity/shape differs"
        );
        let mut forged = error;
        forged["output_may_exist"] = serde_json::json!(true);
        assert_eq!(
            validate_startup(&serde_json::to_vec(&forged).unwrap(), "test:video").unwrap_err(),
            "video startup refusal identity/shape differs"
        );
        forged["output_may_exist"] = serde_json::json!(false);
        forged["code"] = serde_json::json!("video_export_failed\nunsafe");
        assert_eq!(
            validate_startup(&serde_json::to_vec(&forged).unwrap(), "test:video").unwrap_err(),
            "video startup refusal identity/shape differs"
        );
    }

    #[test]
    fn wrong_profile_container_is_refused() {
        let p = VideoProfile {
            width: 640,
            height: 360,
            fps: FrameRate {
                numerator: 12,
                denominator: 1,
            },
            frame_count: 24,
            format: VideoFormat::Mp4,
            captions_path: None,
        };
        assert!(validate_artifact(b"GIF89a fake", &p).is_err());
        let mut p = p;
        p.format = VideoFormat::PngSequenceTar;
        assert!(validate_artifact(&[0; 1024], &p).is_err());
    }

    fn test_tar_header(name: &str, size: usize) -> [u8; 512] {
        let mut header = [0u8; 512];
        header[..name.len()].copy_from_slice(name.as_bytes());
        header[124..136].copy_from_slice(format!("{size:011o}\0").as_bytes());
        header[148..156].fill(32);
        header[156] = b'0';
        header[257..263].copy_from_slice(b"ustar\0");
        let checksum: usize = header.iter().map(|v| usize::from(*v)).sum();
        header[148..156].copy_from_slice(format!("{checksum:06o}\0 ").as_bytes());
        header
    }
    // Container fixture only; the adapter's independent complete PNG decode is
    // a separate native test and is not claimed by these synthetic chunk bytes.
    fn test_png(data_bytes: usize) -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        bytes.extend(13_u32.to_be_bytes());
        bytes.extend(b"IHDR");
        bytes.extend(640_u32.to_be_bytes());
        bytes.extend(360_u32.to_be_bytes());
        bytes.extend([8, 6, 0, 0, 0]);
        bytes.extend([0; 4]);
        bytes.extend((data_bytes as u32).to_be_bytes());
        bytes.extend(b"IDAT");
        bytes.resize(bytes.len() + data_bytes + 4, 0);
        bytes.extend(0_u32.to_be_bytes());
        bytes.extend(b"IEND");
        bytes.extend([0; 4]);
        bytes
    }
    fn test_tar(frame_name: &str, frame: &[u8], frame_revision: &str) -> Vec<u8> {
        let manifest = serde_json::to_vec(&serde_json::json!({"schema":1,"format":"png_sequence_tar","width":640,"height":360,"fps":{"numerator":12,"denominator":1},"frame_count":1,
            "entries":[{"path":frame_name,"bytes":frame.len(),"revision":frame_revision}]})).unwrap();
        let mut archive = vec![];
        for (name, data) in [(frame_name, frame), ("manifest.json", manifest.as_slice())] {
            archive.extend(test_tar_header(name, data.len()));
            archive.extend(data);
            archive.resize(archive.len().div_ceil(512) * 512, 0);
        }
        archive.extend([0; 1024]);
        archive
    }
    #[test]
    fn streaming_tar_above_buffered_cap_checks_each_chunk_hash_and_manifest() {
        let profile = VideoProfile {
            width: 640,
            height: 360,
            fps: FrameRate {
                numerator: 12,
                denominator: 1,
            },
            frame_count: 1,
            format: VideoFormat::PngSequenceTar,
            captions_path: None,
        };
        let png = test_png(crate::artifacts::HARD_MAX_BYTES + 1);
        let mut archive = test_tar("frame_000001.png", &png, &revision(&png));
        struct Bounded<'a>(std::io::Cursor<&'a [u8]>);
        impl Read for Bounded<'_> {
            fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
                assert!(b.len() <= 65536, "body read exceeded streaming buffer");
                std::io::Read::read(&mut self.0, b)
            }
        }
        assert!(archive.len() > crate::artifacts::HARD_MAX_BYTES);
        validate_reader(
            &mut Bounded(std::io::Cursor::new(&archive)),
            archive.len(),
            &profile,
        )
        .unwrap();
        archive[512 + 50] ^= 1;
        assert!(
            validate_artifact(&archive, &profile)
                .unwrap_err()
                .contains("manifest differs")
        );
        let png = test_png(1);
        for name in ["../escape.png", "frame_000002.png"] {
            assert!(validate_artifact(&test_tar(name, &png, &revision(&png)), &profile).is_err());
        }
        let mut archive = test_tar("frame_000001.png", &png, &revision(&png));
        archive[512 + png.len()] = 1;
        assert!(
            validate_artifact(&archive, &profile)
                .unwrap_err()
                .contains("padding")
        );
        let mut archive = test_tar("frame_000001.png", &png, &revision(&png));
        archive.truncate(archive.len() - 512);
        assert!(validate_artifact(&archive, &profile).is_err());
    }
    #[tokio::test(flavor = "current_thread")]
    async fn blocking_video_hash_can_be_cancelled_while_runtime_remains_responsive() {
        let root = std::env::temp_dir().join(format!(
            "mirket-video-io-cancel-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        fs::write(root.join("render"), b"rendered").unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let worker_root = root.clone();
        let (entered, ready) = tokio::sync::oneshot::channel();
        let entered = std::sync::Mutex::new(Some(entered));
        let task = tokio::task::spawn_blocking(move || {
            crate::artifacts::VideoFileRevision::inspect_checked(
                &worker_root,
                &worker_root.join("render"),
                100,
                &|| {
                    if let Some(entered) = entered.lock().unwrap().take() {
                        let _ = entered.send(());
                        while !worker_cancel.load(Ordering::Acquire) {
                            std::thread::sleep(std::time::Duration::from_millis(1));
                        }
                    }
                    !worker_cancel.load(Ordering::Acquire)
                },
            )
        });
        tokio::time::timeout(std::time::Duration::from_secs(2), ready)
            .await
            .unwrap()
            .unwrap();
        cancel.store(true, Ordering::Release);
        let result = tokio::time::timeout(std::time::Duration::from_secs(2), task)
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(
            result,
            Err(crate::artifacts::ArtifactError::VideoStopped)
        ));
        fs::remove_dir_all(root).unwrap();
    }
    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn malformed_final_frame_and_corrupt_stage_are_refused_after_group_cleanup() {
        let Some(node) = std::env::var_os("MIRKET_TEST_NODE") else {
            return;
        };
        for mode in [
            "malformed",
            "corrupt",
            "diagnostic",
            "diagnostic-forged-path",
            "diagnostic-over-budget",
        ] {
            let root = std::env::temp_dir().join(format!(
                "mirket-video-negative-{}-{}-{}",
                std::process::id(),
                mode,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&root).unwrap();
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
            let root = fs::canonicalize(root).unwrap();
            let adapter = root.join("adapter.mjs");
            let script = r#"import fs from 'node:fs';import crypto from 'node:crypto';
const release=process.argv[3];
console.log(JSON.stringify({type:'ready',protocol_version:1,release_id:release,operation:'video.export',policy_version:1,max_frame_bytes:65536}));
process.stdin.once('data',chunk=>{const r=JSON.parse(chunk);if(MODE==='malformed'){console.log('{not a valid result}');}else if(MODE.startsWith('diagnostic')){const d={phase:'root',path:MODE==='diagnostic-forged-path'?'unselected.html':'index.html',field:'data-no-timeline',reason:'required',expected:'literal marker on the composition root'};console.log(JSON.stringify({type:'error',protocol_version:1,release_id:release,job_id:r.job_id,code:'authored_root_profile_mismatch',output_may_exist:false,diagnostics:MODE==='diagnostic-over-budget'?Array(17).fill(d):[d]}));}else{const b=Buffer.from('corrupt media');fs.writeFileSync(r.output_path,b);console.log(JSON.stringify({type:'result',protocol_version:1,release_id:release,job_id:r.job_id,status:'staged',output_path:r.output_path,output_revision:'sha256:'+crypto.createHash('sha256').update(b).digest('hex'),output_bytes:b.length,evidence:{complete_decode:true}}));}process.stdin.destroy();process.stdout.end();});"#;
            fs::write(&adapter, script.replace("MODE", &format!("'{mode}'"))).unwrap();
            let payload = VideoPayload {
                node: fs::canonicalize(&node).unwrap(),
                adapter: adapter.clone(),
                chromium: adapter.clone(),
                ffmpeg: adapter.clone(),
                ffprobe: adapter,
            };
            fs::create_dir(root.join("source")).unwrap();
            let request = VideoRequest {
                protocol_version: 1,
                release_id: "test:video".into(),
                job_id: "negative-1".into(),
                job_root: root.clone(),
                input_root: root.join("source"),
                entry_file: "index.html".into(),
                sources: vec![SourceFile {
                    path: "index.html".into(),
                    revision: revision(b"source"),
                    bytes: 6,
                }],
                output_path: root.join("result.mp4"),
                profile: VideoProfile {
                    width: 640,
                    height: 360,
                    fps: FrameRate {
                        numerator: 12,
                        denominator: 1,
                    },
                    frame_count: 24,
                    format: VideoFormat::Mp4,
                    captions_path: None,
                },
                timeout_ms: 30000,
                max_output_bytes: MAX_OUTPUT_BYTES,
            };
            let failure = run_video_adapter(
                &payload,
                &request,
                Arc::new(AtomicBool::new(false)),
                tokio::time::Instant::now() + std::time::Duration::from_secs(5),
            )
            .await
            .unwrap_err();
            assert!(failure.cleanup_confirmed, "{mode}: {failure}");
            let expected = match mode {
                "malformed" => "invalid video completion",
                "corrupt" => "container signature",
                "diagnostic" => "authored_root_profile_mismatch",
                "diagnostic-forged-path" => "diagnostic location/shape",
                _ => "diagnostics exceed",
            };
            assert!(failure.message.contains(expected), "{mode}: {failure}");
            if mode == "diagnostic" {
                assert_eq!(failure.diagnostics.len(), 1);
                assert_eq!(failure.diagnostics[0].field, "data-no-timeline");
                assert!(!request.output_path.exists());
            } else {
                assert!(failure.diagnostics.is_empty());
            }
            fs::remove_dir_all(root).unwrap();
        }
    }

    /// Explicit private development payloads only. No native candidate is
    /// discovered, downloaded or selected from PATH by this qualification.
    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn native_candidate_formats_decode_and_preserve_ordered_png_motion() {
        let names = [
            "MIRKET_TEST_NODE",
            "MIRKET_TEST_VIDEO_CHROMIUM",
            "MIRKET_TEST_VIDEO_FFMPEG",
            "MIRKET_TEST_VIDEO_FFPROBE",
            "MIRKET_TEST_VIDEO_FIXTURE",
        ];
        let Some(paths) = names
            .iter()
            .map(|name| std::env::var_os(name).map(std::path::PathBuf::from))
            .collect::<Option<Vec<_>>>()
        else {
            return;
        };
        let payload = VideoPayload {
            node: fs::canonicalize(&paths[0]).unwrap(),
            adapter: fs::canonicalize(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("adapters/dist/video-main.js"),
            )
            .unwrap(),
            chromium: fs::canonicalize(&paths[1]).unwrap(),
            ffmpeg: fs::canonicalize(&paths[2]).unwrap(),
            ffprobe: fs::canonicalize(&paths[3]).unwrap(),
        };
        let formats = match std::env::var("MIRKET_TEST_VIDEO_FORMAT").ok().as_deref() {
            Some("mp4") => vec![VideoFormat::Mp4],
            None => vec![
                VideoFormat::Mp4,
                VideoFormat::Webm,
                VideoFormat::Gif,
                VideoFormat::PngSequenceTar,
            ],
            Some(_) => {
                panic!("explicit native format selector supports only the bounded MP4 follow-up")
            }
        };
        for (index, format) in formats.into_iter().enumerate() {
            let root = std::env::temp_dir().join(format!(
                "mirket-native-video-{}-{index}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&root).unwrap();
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
            let root = fs::canonicalize(root).unwrap();
            let input = root.join("source");
            fs::create_dir(&input).unwrap();
            fs::set_permissions(&input, fs::Permissions::from_mode(0o700)).unwrap();
            let mut sources = vec![];
            for name in [
                "index.html",
                "NotoSans.ttf",
                "anime.umd.min.js",
                "voice.wav",
                "music.wav",
            ] {
                let bytes = fs::read(paths[4].join(name)).unwrap();
                fs::write(input.join(name), &bytes).unwrap();
                sources.push(SourceFile {
                    path: name.into(),
                    revision: revision(&bytes),
                    bytes: bytes.len(),
                });
            }
            sources.sort_by(|a, b| a.path.cmp(&b.path));
            let request = VideoRequest {
                protocol_version: 1,
                release_id: "test:video-native".into(),
                job_id: format!("native-{index}"),
                job_root: root.clone(),
                input_root: input,
                entry_file: "index.html".into(),
                sources,
                output_path: root.join(format!("result.{}", format.extension())),
                profile: VideoProfile {
                    width: 640,
                    height: 360,
                    fps: FrameRate {
                        numerator: 12,
                        denominator: 1,
                    },
                    frame_count: 24,
                    format,
                    captions_path: None,
                },
                timeout_ms: 60000,
                max_output_bytes: MAX_OUTPUT_BYTES,
            };
            let outcome = run_video_adapter(
                &payload,
                &request,
                Arc::new(AtomicBool::new(false)),
                tokio::time::Instant::now() + std::time::Duration::from_secs(60),
            )
            .await;
            assert!(
                outcome.is_ok(),
                "{format:?}: {outcome:?}; retained {}",
                root.display()
            );
            let outcome = outcome.unwrap();
            assert!(outcome.cleanup_confirmed);
            if format == VideoFormat::PngSequenceTar {
                let archive = fs::read(&request.output_path).unwrap();
                let mut at = 0;
                let mut left = vec![];
                while at + 512 < archive.len() {
                    let header = &archive[at..at + 512];
                    if header.iter().all(|b| *b == 0) {
                        break;
                    }
                    let size = octal(&header[124..136]).unwrap();
                    let n = header[..100].iter().position(|b| *b == 0).unwrap_or(100);
                    let name = std::str::from_utf8(&header[..n]).unwrap();
                    if matches!(name, "frame_000001.png" | "frame_000024.png") {
                        let image = resvg::tiny_skia::Pixmap::decode_png(
                            &archive[at + 512..at + 512 + size],
                        )
                        .unwrap();
                        let first = (0..640)
                            .find(|x| {
                                let p = image.pixel(*x, 118).unwrap();
                                p.red() > 180
                                    && p.green() < 140
                                    && p.blue() < 120
                                    && p.alpha() == 255
                            })
                            .unwrap();
                        left.push(first);
                    }
                    at += 512 + size.div_ceil(512) * 512;
                }
                assert_eq!(
                    left,
                    vec![32, 492],
                    "intended Anime element must move; a changing CSS neighbor is insufficient"
                );
            }
            fs::remove_dir_all(&root).unwrap();
        }
    }
}
