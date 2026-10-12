//! Private, fixed Podman 6.1.3 compatibility API operations over an owned Unix socket.
//!
//! The host validates the managed engine and owns container creation and cancellation.
//! Dropping an attached request closes its stream; the caller must separately remove
//! its immutable container ID. No socket or API operation is exposed to MCP callers.
//! Sources: <https://github.com/containers/podman/blob/v6.1.3/pkg/api/server/register_exec.go>,
//! <https://github.com/containers/podman/blob/v6.1.3/pkg/api/handlers/compat/exec.go>,
//! <https://github.com/containers/podman/blob/v6.1.3/pkg/api/server/register_containers.go>,
//! <https://github.com/containers/podman/blob/v6.1.3/pkg/api/server/register_archive.go>.
#[cfg(unix)]
use crate::util;
use anyhow::{Context, Result, bail, ensure};
use reqwest::{Client, Response, StatusCode};
use serde_json::{Value, json};
#[cfg(unix)]
use std::time::Duration;
use std::{
    io::Read,
    path::{Path, PathBuf},
};
use tokio::io::{AsyncRead, AsyncReadExt};

const STREAM_LIMIT: usize = 128 * 1024;
const FRAME_LIMIT: usize = 8192;
const OUTPUT_LIMIT: usize = 32 * 1024 * 1024;
const ARCHIVE_LIMIT: usize = 33 * 1024 * 1024;
const PREFIX: &str = "http://localhost/v1.41";

#[derive(Clone, Debug)]
pub struct Api {
    client: Client,
    socket: PathBuf,
    #[cfg(unix)]
    identity: SocketIdentity,
}

#[cfg(unix)]
#[derive(Clone, Debug, PartialEq)]
struct SocketIdentity {
    uid: u32,
    device: u64,
    inode: u64,
}

#[cfg(unix)]
fn socket_identity(socket: &Path, owner: Option<u32>) -> Result<SocketIdentity> {
    use std::os::unix::fs::{FileTypeExt, MetadataExt};
    util::reject_symlink_components(socket)?;
    let metadata =
        std::fs::symlink_metadata(socket).context("inspect private Podman API socket")?;
    ensure!(
        metadata.file_type().is_socket(),
        "Podman API path is not a Unix socket"
    );
    // A kernel-created anonymous file identifies the effective filesystem owner
    // without trusting HOME, a username environment variable, or an external command.
    let uid = match owner {
        Some(uid) => uid,
        None => tempfile::tempfile()?.metadata()?.uid(),
    };
    ensure!(
        metadata.uid() == uid,
        "Podman API socket belongs to another user"
    );
    Ok(SocketIdentity {
        uid,
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

impl Api {
    pub fn new(socket: &Path) -> Result<Self> {
        #[cfg(not(unix))]
        {
            let _ = socket;
            bail!("The native Podman Unix API is not verified on this platform")
        }
        #[cfg(unix)]
        {
            ensure!(socket.is_absolute(), "Podman API socket must be absolute");
            let identity = socket_identity(socket, None)?;
            let client = Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .http1_only()
                .connect_timeout(Duration::from_secs(3))
                .timeout(Duration::from_secs(120))
                .pool_max_idle_per_host(4)
                .unix_socket(socket.to_owned())
                .build()?;
            Ok(Self {
                client,
                socket: socket.to_owned(),
                identity,
            })
        }
    }

    fn check_socket(&self) -> Result<()> {
        #[cfg(unix)]
        {
            ensure!(
                socket_identity(&self.socket, Some(self.identity.uid))? == self.identity,
                "Podman API socket changed; recreate the owned runtime after checking environment status"
            );
            Ok(())
        }
        #[cfg(not(unix))]
        {
            let _ = &self.socket;
            bail!("The native Podman Unix API is not verified on this platform")
        }
    }

    /// Run only the fixed, packaged job launcher, never a shell or project command.
    pub async fn exec(&self, cid: &str, cmd: &[String]) -> Result<Vec<u8>> {
        identifier(cid)?;
        command(cmd)?;
        self.check_socket()?;
        let request = serde_json::to_vec(&json!({
            "AttachStdin":false,"AttachStdout":true,"AttachStderr":true,
            "Tty":false,"Cmd":cmd,"User":"1000:1000","WorkingDir":"/work","Privileged":false
        }))?;
        ensure!(
            request.len() <= 4096,
            "Podman exec request exceeds its fixed contract"
        );
        let response = self
            .client
            .post(format!("{PREFIX}/containers/{cid}/exec"))
            .header("Content-Type", "application/json")
            .body(request)
            .send()
            .await
            .context("create owned Podman execution")?;
        let created = metadata(response, StatusCode::CREATED, "create execution").await?;
        let exec_id = created["Id"]
            .as_str()
            .context("Podman returned no execution ID")?;
        identifier(exec_id)?;
        let response = self
            .client
            .post(format!("{PREFIX}/exec/{exec_id}/start"))
            .header("Connection", "Upgrade")
            .header("Upgrade", "tcp")
            .json(&json!({"Detach":false,"Tty":false}))
            .send()
            .await
            .context("attach owned Podman execution")?;
        if response.status() != StatusCode::SWITCHING_PROTOCOLS {
            http_failure(response, "attach execution").await?;
            unreachable!();
        }
        let upgraded = response
            .upgrade()
            .await
            .context("Podman did not provide the attached stream")?;
        let (stdout, stderr) = multiplexed(upgraded).await?;
        let response = self
            .client
            .get(format!("{PREFIX}/exec/{exec_id}/json"))
            .send()
            .await
            .context("inspect completed Podman execution")?;
        let inspected = metadata(response, StatusCode::OK, "inspect execution").await?;
        completed(&inspected, cid, exec_id, &stderr)?;
        Ok(stdout)
    }

    /// Read one validated artifact from the guest's bounded output filesystem.
    pub async fn output(&self, cid: &str, key: &str, name: &str) -> Result<Vec<u8>> {
        identifier(cid)?;
        job_key(key)?;
        output_name(name)?;
        self.check_socket()?;
        let response = self
            .client
            .get(format!("{PREFIX}/containers/{cid}/archive"))
            .query(&[("path", format!("/output/{key}/{name}"))])
            .send()
            .await
            .context("read owned Podman artifact archive")?;
        if response.status() != StatusCode::OK {
            http_failure(response, "read output").await?;
            unreachable!();
        }
        let bytes = bounded(response, ARCHIVE_LIMIT).await?;
        artifact(&bytes, name)
    }

    /// Remove the captured immutable ID. A missing container is already cleaned up.
    pub async fn remove(&self, cid: &str) -> Result<()> {
        identifier(cid)?;
        self.check_socket()?;
        let response = self
            .client
            .delete(format!("{PREFIX}/containers/{cid}"))
            .query(&[("force", "true"), ("v", "false")])
            .send()
            .await
            .context("remove owned Podman container")?;
        if matches!(
            response.status(),
            StatusCode::NO_CONTENT | StatusCode::NOT_FOUND
        ) {
            return Ok(());
        }
        http_failure(response, "remove container").await
    }
}

fn identifier(id: &str) -> Result<()> {
    ensure!(
        id.len() == 64
            && id
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "Podman requires an immutable lowercase 64-digit container or execution ID"
    );
    Ok(())
}
fn job_key(key: &str) -> Result<()> {
    ensure!(
        !key.is_empty()
            && key.len() <= 64
            && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'),
        "Invalid private Podman job key"
    );
    Ok(())
}
fn output_name(name: &str) -> Result<()> {
    ensure!(
        matches!(
            name,
            "output.png" | "output.mp4" | "output.webm" | "output.svg" | "report.json"
        ),
        "Podman output basename is not reviewed"
    );
    Ok(())
}
fn command(cmd: &[String]) -> Result<()> {
    ensure!(
        cmd.len() == 3 && cmd[0] == "/opt/nodejs/bin/node" && cmd[1] == "/opt/mirket/job.mjs",
        "Only the fixed packaged Podman job launcher may execute"
    );
    job_key(&cmd[2])
}

async fn bounded(mut response: Response, limit: usize) -> Result<Vec<u8>> {
    if let Some(size) = response.content_length() {
        ensure!(
            size <= limit as u64,
            "Podman HTTP response exceeds {limit} bytes"
        );
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .context("read bounded Podman response")?
    {
        let needed = bytes
            .len()
            .checked_add(chunk.len())
            .context("Podman response size overflow")?;
        ensure!(
            needed <= limit,
            "Podman HTTP response exceeds {limit} bytes"
        );
        if bytes.capacity() < needed {
            let capacity = needed
                .max(bytes.capacity().saturating_mul(2))
                .max(16384)
                .min(limit);
            bytes.try_reserve_exact(capacity - bytes.len())?;
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
async fn http_failure(response: Response, operation: &str) -> Result<()> {
    let status = response.status();
    let bytes = bounded(response, STREAM_LIMIT).await?;
    bail!(
        "Podman API {operation} failed ({status}): {}",
        diagnostic(&bytes)
    );
}
async fn metadata(response: Response, expected: StatusCode, operation: &str) -> Result<Value> {
    if response.status() != expected {
        http_failure(response, operation).await?;
        unreachable!();
    }
    serde_json::from_slice(&bounded(response, STREAM_LIMIT).await?)
        .with_context(|| format!("Podman API {operation} returned invalid bounded JSON"))
}
fn diagnostic(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .chars()
        .take(2000)
        .map(|c| {
            if c.is_control() && !matches!(c, '\n' | '\t') {
                '?'
            } else {
                c
            }
        })
        .collect()
}

/// Docker's non-TTY stream uses an 8-byte header, not HTTP chunks or text lines.
async fn multiplexed(mut stream: impl AsyncRead + Unpin) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut frames = 0;
    loop {
        let mut header = [0u8; 8];
        if stream.read(&mut header[..1]).await? == 0 {
            return Ok((stdout, stderr));
        }
        stream
            .read_exact(&mut header[1..])
            .await
            .context("truncated Podman stream header")?;
        frames += 1;
        ensure!(
            frames <= FRAME_LIMIT,
            "Podman stream exceeds its frame count bound"
        );
        ensure!(
            matches!(header[0], 1 | 2) && header[1..4] == [0, 0, 0],
            "Invalid Podman stdout/stderr framing"
        );
        let length = u32::from_be_bytes(header[4..8].try_into()?) as usize;
        let target = if header[0] == 1 {
            &mut stdout
        } else {
            &mut stderr
        };
        let size = target
            .len()
            .checked_add(length)
            .context("Podman stream size overflow")?;
        ensure!(
            size <= STREAM_LIMIT,
            "Podman diagnostic stream exceeds 128 KiB"
        );
        let offset = target.len();
        target.resize(size, 0);
        stream
            .read_exact(&mut target[offset..])
            .await
            .context("truncated Podman stream payload")?;
    }
}
fn completed(value: &Value, cid: &str, exec_id: &str, stderr: &[u8]) -> Result<()> {
    ensure!(
        value["ContainerID"] == cid && value["ID"] == exec_id,
        "Podman execution belongs to another container or session"
    );
    ensure!(
        value["Running"] == false,
        "Podman execution has not finished"
    );
    let code = value["ExitCode"]
        .as_i64()
        .context("Podman execution has no verified exit status")?;
    ensure!(
        code == 0,
        "Podman execution failed with exit code {code}: {}",
        diagnostic(stderr)
    );
    Ok(())
}

fn artifact(bytes: &[u8], name: &str) -> Result<Vec<u8>> {
    output_name(name)?;
    ensure!(
        bytes.len() <= ARCHIVE_LIMIT && bytes.len().is_multiple_of(512),
        "Podman output archive is oversized or truncated"
    );
    let mut archive = tar::Archive::new(bytes);
    let mut entries = archive
        .entries()
        .context("invalid Podman output archive")?
        .raw(true);
    let mut entry = entries
        .next()
        .context("Podman returned an empty output archive")??;
    ensure!(
        entry.header().entry_type().is_file() && entry.path_bytes().as_ref() == name.as_bytes(),
        "Podman output archive must contain the requested regular file only"
    );
    ensure!(
        entry.raw_header_position() == 0 && entry.raw_file_position() == 512,
        "Unexpected Podman archive metadata"
    );
    let size = entry.header().size()?;
    ensure!(
        size > 0 && size <= OUTPUT_LIMIT as u64,
        "Podman output is empty or exceeds 32 MiB"
    );
    let mut output = vec![0; size as usize];
    entry
        .read_exact(&mut output)
        .context("Podman output member is truncated")?;
    drop(entry);
    ensure!(
        entries.next().is_none(),
        "Podman output archive contains multiple members"
    );
    let body_end = 512 + size as usize;
    let padded_end = body_end.div_ceil(512) * 512;
    ensure!(
        bytes.len() >= padded_end + 1024 && bytes[body_end..].iter().all(|byte| *byte == 0),
        "Podman output archive has a missing terminator or unexpected trailing data"
    );
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Cursor,
        pin::Pin,
        task::{Context as TaskContext, Poll},
    };
    use tokio::io::ReadBuf;

    const CID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const EXEC: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn frame(kind: u8, bytes: &[u8]) -> Vec<u8> {
        let mut frame = vec![kind, 0, 0, 0];
        frame.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        frame.extend_from_slice(bytes);
        frame
    }
    struct Fragmented {
        cursor: Cursor<Vec<u8>>,
        chunk: usize,
    }
    impl AsyncRead for Fragmented {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _: &mut TaskContext<'_>,
            buffer: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            let count = buffer.remaining().min(self.chunk);
            let mut bytes = vec![0; count];
            match Read::read(&mut self.cursor, &mut bytes) {
                Ok(count) => {
                    buffer.put_slice(&bytes[..count]);
                    Poll::Ready(Ok(()))
                }
                Err(error) => Poll::Ready(Err(error)),
            }
        }
    }
    #[tokio::test]
    async fn podman_api_fragmented_stream_keeps_stdout_stderr_separate() {
        let data = [
            frame(1, b"first"),
            frame(2, b"diagnostic"),
            frame(1, b"second"),
        ]
        .concat();
        for chunk in [1, 3, 9, 4096] {
            let (stdout, stderr) = multiplexed(Fragmented {
                cursor: Cursor::new(data.clone()),
                chunk,
            })
            .await
            .unwrap();
            assert_eq!(stdout, b"firstsecond");
            assert_eq!(stderr, b"diagnostic");
        }
    }
    #[tokio::test]
    async fn podman_api_stream_refuses_framing_truncation_and_overflow() {
        let mut reserved = frame(1, b"value");
        reserved[2] = 1;
        let mut truncated_header = frame(1, b"value");
        truncated_header.truncate(4);
        let mut truncated_body = frame(1, b"value");
        truncated_body.pop();
        for data in [
            reserved,
            frame(0, b"stdin"),
            truncated_header,
            truncated_body,
            frame(1, &vec![1; STREAM_LIMIT + 1]),
            [frame(2, &vec![2; STREAM_LIMIT]), frame(2, b"overflow")].concat(),
            vec![frame(1, b""); FRAME_LIMIT + 1].concat(),
        ] {
            assert!(multiplexed(Cursor::new(data)).await.is_err());
        }
    }
    #[test]
    fn podman_api_identifiers_keys_commands_and_outputs_are_fixed() {
        identifier(CID).unwrap();
        job_key("job-Ab12").unwrap();
        output_name("output.png").unwrap();
        command(&[
            "/opt/nodejs/bin/node".into(),
            "/opt/mirket/job.mjs".into(),
            "job-Ab12".into(),
        ])
        .unwrap();
        for id in [
            "",
            "container-name",
            "../container",
            &"A".repeat(64),
            &format!("sha256:{CID}"),
        ] {
            assert!(identifier(id).is_err());
        }
        for key in [
            "",
            "../escape",
            "/absolute",
            "job%2fescape",
            "job?escape",
            "job_unsupported",
            &"a".repeat(65),
        ] {
            assert!(job_key(key).is_err());
        }
        for name in [
            "../output.png",
            "/output.png",
            "output.gif",
            "output.png?escape",
        ] {
            assert!(output_name(name).is_err());
        }
        for cmd in [
            vec!["/bin/sh".into(), "-c".into(), "true".into()],
            vec![
                "/opt/nodejs/bin/node".into(),
                "project.js".into(),
                "job-Ab12".into(),
            ],
            vec![
                "/opt/nodejs/bin/node".into(),
                "/opt/mirket/job.mjs".into(),
                "../escape".into(),
            ],
        ] {
            assert!(command(&cmd).is_err());
        }
    }
    #[test]
    fn podman_api_exit_inspection_rejects_convincing_false_success() {
        let valid = json!({"ContainerID":CID,"ID":EXEC,"Running":false,"ExitCode":0});
        completed(&valid, CID, EXEC, b"").unwrap();
        for (field, value) in [
            ("ContainerID", json!(EXEC)),
            ("ID", json!(CID)),
            ("Running", json!(true)),
            ("ExitCode", json!(7)),
            ("ExitCode", Value::Null),
        ] {
            let mut invalid = valid.clone();
            invalid[field] = value;
            assert!(completed(&invalid, CID, EXEC, b"intentional failure").is_err());
        }
        let mut nonzero = valid;
        nonzero["ExitCode"] = 7.into();
        let error = completed(&nonzero, CID, EXEC, b"intentional failure").unwrap_err();
        assert!(
            error
                .to_string()
                .contains("exit code 7: intentional failure")
        );
    }

    fn member(name: &str, data: &[u8], kind: tar::EntryType) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_path(name).unwrap();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_entry_type(kind);
        if kind.is_symlink() || kind.is_hard_link() {
            header.set_link_name("outside").unwrap();
        }
        header.set_cksum();
        builder.append(&header, data).unwrap();
        builder.into_inner().unwrap()
    }
    #[test]
    fn podman_api_output_accepts_only_complete_single_regular_artifact() {
        let valid = member("output.png", b"saved artifact", tar::EntryType::Regular);
        assert_eq!(artifact(&valid, "output.png").unwrap(), b"saved artifact");
        for kind in [
            tar::EntryType::Symlink,
            tar::EntryType::Link,
            tar::EntryType::Directory,
            tar::EntryType::GNULongName,
        ] {
            assert!(artifact(&member("output.png", b"", kind), "output.png").is_err());
        }
        assert!(
            artifact(
                &member("other.png", b"saved artifact", tar::EntryType::Regular),
                "output.png"
            )
            .is_err()
        );
        assert!(
            artifact(
                &member("output.png", b"", tar::EntryType::Regular),
                "output.png"
            )
            .is_err()
        );
        let mut truncated = valid.clone();
        truncated.truncate(1024);
        assert!(artifact(&truncated, "output.png").is_err());
        let mut corrupt = valid.clone();
        corrupt[0] ^= 1;
        assert!(artifact(&corrupt, "output.png").is_err());
        let trailing = [
            valid.clone(),
            member("other.png", b"hidden", tar::EntryType::Regular),
        ]
        .concat();
        assert!(artifact(&trailing, "output.png").is_err());
        let mut two = valid[..1024].to_vec();
        two.extend(member("output.png", b"second", tar::EntryType::Regular));
        assert!(artifact(&two, "output.png").is_err());
        let mut escaped = valid;
        escaped[..100].fill(0);
        escaped[..13].copy_from_slice(b"../output.png");
        let mut header = tar::Header::from_byte_slice(&escaped[..512]).clone();
        header.set_cksum();
        escaped[..512].copy_from_slice(header.as_bytes());
        assert!(artifact(&escaped, "output.png").is_err());
    }
    #[test]
    fn podman_api_output_refuses_oversized_member_before_reading() {
        let mut header = tar::Header::new_gnu();
        header.set_path("output.png").unwrap();
        header.set_size(OUTPUT_LIMIT as u64 + 1);
        header.set_mode(0o644);
        header.set_cksum();
        let mut data = header.as_bytes().to_vec();
        data.resize(2048, 0);
        assert!(
            artifact(&data, "output.png")
                .unwrap_err()
                .to_string()
                .contains("32 MiB")
        );
    }
    #[cfg(unix)]
    #[test]
    fn podman_api_socket_refuses_regular_files_links_and_replaced_endpoint() {
        use std::os::unix::{fs::symlink, net::UnixListener};
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let regular = root.join("regular");
        std::fs::write(&regular, b"not a socket").unwrap();
        assert!(Api::new(&regular).is_err());
        let socket = root.join("private.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        let api = Api::new(&socket).unwrap();
        let alias = root.join("alias.sock");
        symlink(&socket, &alias).unwrap();
        assert!(Api::new(&alias).is_err());
        drop(listener);
        std::fs::remove_file(&socket).unwrap();
        let _replacement = UnixListener::bind(&socket).unwrap();
        assert!(api.check_socket().is_err());
    }

    #[cfg(unix)]
    fn fake_request(
        listener: &std::os::unix::net::UnixListener,
    ) -> (std::os::unix::net::UnixStream, String, Value) {
        use std::{io::ErrorKind, time::Instant};
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error)
                    if error.kind() == ErrorKind::WouldBlock && Instant::now() < deadline =>
                {
                    std::thread::sleep(Duration::from_millis(1))
                }
                Err(error) => panic!("bounded fake API accept: {error}"),
            }
        };
        // BSD/macOS can inherit the listener's nonblocking flag at accept.
        // Reading a newly connected request must wait for its headers within
        // the existing timeout, rather than race the client's first write.
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut bytes = Vec::new();
        let headers_end = loop {
            let mut buffer = [0; 1024];
            let count = stream.read(&mut buffer).unwrap();
            assert!(
                count > 0 && bytes.len() + count <= 12 * 1024,
                "bounded request header"
            );
            bytes.extend_from_slice(&buffer[..count]);
            if let Some(offset) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                break offset + 4;
            }
        };
        let headers = std::str::from_utf8(&bytes[..headers_end])
            .unwrap()
            .to_owned();
        let length = headers
            .lines()
            .find_map(|line| {
                line.split_once(':')
                    .filter(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                    .map(|(_, value)| value.trim().parse::<usize>().unwrap())
            })
            .unwrap_or(0);
        assert!(length <= 4096);
        while bytes.len() < headers_end + length {
            let mut buffer = [0; 1024];
            let count = stream.read(&mut buffer).unwrap();
            assert!(count > 0);
            bytes.extend_from_slice(&buffer[..count]);
            assert!(bytes.len() <= 12 * 1024);
        }
        let body = if length == 0 {
            Value::Null
        } else {
            serde_json::from_slice(&bytes[headers_end..headers_end + length]).unwrap()
        };
        (stream, headers, body)
    }

    #[cfg(unix)]
    fn fake_response(mut stream: std::os::unix::net::UnixStream, status: &str, body: &[u8]) {
        use std::io::Write;
        write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .unwrap();
        stream.write_all(body).unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn podman_api_http_upgrade_preserves_buffered_frames_and_checks_exit() {
        use std::{io::Write, os::unix::net::UnixListener};
        for exit_code in [0, 7] {
            let directory = tempfile::tempdir().unwrap();
            let socket = directory.path().canonicalize().unwrap().join("fake.sock");
            let listener = UnixListener::bind(&socket).unwrap();
            listener.set_nonblocking(true).unwrap();
            let api = Api::new(&socket).unwrap();
            let server = std::thread::spawn(move || {
                let (stream, headers, body) = fake_request(&listener);
                assert_eq!(
                    headers.lines().next().unwrap(),
                    format!("POST /v1.41/containers/{CID}/exec HTTP/1.1")
                );
                assert_eq!(
                    body,
                    json!({"AttachStdin":false,"AttachStdout":true,"AttachStderr":true,"Tty":false,"Cmd":["/opt/nodejs/bin/node","/opt/mirket/job.mjs","job-test"],"User":"1000:1000","WorkingDir":"/work","Privileged":false})
                );
                fake_response(
                    stream,
                    "201 Created",
                    format!("{{\"Id\":\"{EXEC}\"}}").as_bytes(),
                );
                let (mut stream, headers, body) = fake_request(&listener);
                assert_eq!(
                    headers.lines().next().unwrap(),
                    format!("POST /v1.41/exec/{EXEC}/start HTTP/1.1")
                );
                assert!(headers.to_ascii_lowercase().contains("upgrade: tcp\r\n"));
                assert_eq!(body, json!({"Detach":false,"Tty":false}));
                // Header and framed data share a write: the HTTP upgrade must keep
                // bytes already read beyond the end of the headers.
                let data = [b"HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: tcp\r\n\r\n".to_vec(),frame(1,b"useful saved result"),frame(2,b"intentional diagnostic")].concat();
                stream.write_all(&data).unwrap();
                drop(stream);
                let (stream, headers, body) = fake_request(&listener);
                assert_eq!(
                    headers.lines().next().unwrap(),
                    format!("GET /v1.41/exec/{EXEC}/json HTTP/1.1")
                );
                assert_eq!(body, Value::Null);
                fake_response(
                    stream,
                    "200 OK",
                    serde_json::to_vec(
                        &json!({"ContainerID":CID,"ID":EXEC,"Running":false,"ExitCode":exit_code}),
                    )
                    .unwrap()
                    .as_slice(),
                );
            });
            let result = api
                .exec(
                    CID,
                    &[
                        "/opt/nodejs/bin/node".into(),
                        "/opt/mirket/job.mjs".into(),
                        "job-test".into(),
                    ],
                )
                .await;
            server.join().unwrap();
            if exit_code == 0 {
                assert_eq!(result.unwrap(), b"useful saved result");
            } else {
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains("exit code 7: intentional diagnostic")
                );
            }
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn podman_api_http_output_and_cleanup_use_only_fixed_paths() {
        use std::os::unix::net::UnixListener;
        let directory = tempfile::tempdir().unwrap();
        let socket = directory.path().canonicalize().unwrap().join("fake.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        let api = Api::new(&socket).unwrap();
        let server = std::thread::spawn(move || {
            let (stream, headers, body) = fake_request(&listener);
            assert_eq!(
                headers.lines().next().unwrap(),
                format!(
                    "GET /v1.41/containers/{CID}/archive?path=%2Foutput%2Fjob-test%2Foutput.png HTTP/1.1"
                )
            );
            assert_eq!(body, Value::Null);
            fake_response(
                stream,
                "200 OK",
                &member("output.png", b"usable artifact", tar::EntryType::Regular),
            );
            for status in [
                "204 No Content",
                "404 Not Found",
                "500 Internal Server Error",
            ] {
                let (stream, headers, body) = fake_request(&listener);
                assert_eq!(
                    headers.lines().next().unwrap(),
                    format!("DELETE /v1.41/containers/{CID}?force=true&v=false HTTP/1.1")
                );
                assert_eq!(body, Value::Null);
                fake_response(stream, status, b"");
            }
        });
        assert_eq!(
            api.output(CID, "job-test", "output.png").await.unwrap(),
            b"usable artifact"
        );
        assert!(api.remove(CID).await.is_ok());
        assert!(api.remove(CID).await.is_ok());
        assert!(api.remove(CID).await.is_err());
        server.join().unwrap();
    }
}
