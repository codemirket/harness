//! Connection-owned warm embedding leaf. Only this process entry point loads ORT.
//! Inputs and vectors pass only through bounded anonymous pipes; they are never
//! staged in persistent files. The wire binds source and model identities. The shared supervisor retains admission until
//! the leaf is reaped. No model daemon, shell, PATH lookup or remote fetch occurs.

use crate::{
    embedding::{self, EmbeddingModel, PassageEmbedding, QueryEmbedding, VerifiedEmbeddingPayload},
    engine::revision,
    release::ReleaseId,
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::{
    io::{BufRead, Read, Write},
    path::PathBuf,
};
#[cfg(test)]
use std::{process::Stdio, sync::Arc, time::Duration};
#[cfg(test)]
use tokio::process::Command;

const MAX_FRAME: usize = 2 * 1024 * 1024;

#[cfg(test)]
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub state_root: PathBuf,
    pub release_id: String,
    pub job_id: String,
    pub package_root: PathBuf,
    pub job_dir: PathBuf,
    pub input: Input,
    pub input_revision: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub query: Option<String>,
    pub passages: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub schema: u8,
    pub input_revision: String,
    pub query: Option<QueryEmbedding>,
    pub passages: Vec<PassageEmbedding>,
}
fn validate_input(input: &Input) -> Result<(), String> {
    if input.query.is_none() && input.passages.is_empty() {
        return Err("embedding request is empty".into());
    }
    if input.passages.len() > embedding::MAX_PASSAGES {
        return Err("embedding passage count exceeds its limit".into());
    }
    let mut total = 0;
    for text in input.query.iter().chain(input.passages.iter()) {
        if text.trim().is_empty() || text.len() > embedding::MAX_INPUT_BYTES || text.contains('\0')
        {
            return Err("embedding input must be nonempty bounded UTF-8 text without NUL".into());
        }
        total += text.len();
    }
    if total > embedding::MAX_BATCH_BYTES {
        return Err("embedding batch exceeds its byte limit".into());
    }
    Ok(())
}

#[cfg(test)]
fn validate_request(request: &Request) -> Result<(), String> {
    ReleaseId::parse(&request.release_id).map_err(|_| "invalid embedding release")?;
    ReleaseId::parse(&request.job_id).map_err(|_| "invalid embedding operation")?;
    crate::process::validate_job_dir(&request.job_dir)?;
    if !request.package_root.is_absolute() {
        return Err("embedding package must be explicit and absolute".into());
    }
    validate_input(&request.input)?;
    let bytes =
        serde_json::to_vec(&request.input).map_err(|_| "embedding input encoding failed")?;
    if revision(&bytes) != request.input_revision {
        return Err("embedding input revision mismatch".into());
    }
    Ok(())
}

#[cfg(test)]
fn validate_outcome(request: &Request, outcome: &Output) -> Result<(), String> {
    validate_request(request)?;
    validate_output(&request.input_revision, &request.input, outcome)
}

fn validate_vector(vector: &[f32]) -> Result<(), String> {
    if vector.len() != embedding::DIMENSIONS || vector.iter().any(|x| !x.is_finite()) {
        return Err("embedding vector dimensions or values are invalid".into());
    }
    let norm: f64 = vector.iter().map(|x| f64::from(*x).powi(2)).sum();
    if (norm - 1.0).abs() > 0.002 {
        return Err("embedding vector is not normalized".into());
    }
    Ok(())
}
fn validate_output(input_revision: &str, input: &Input, output: &Output) -> Result<(), String> {
    let identity = embedding::expected_model_identity().map_err(|e| e.code().to_owned())?;
    if output.schema != 1
        || output.input_revision != input_revision
        || output.passages.len() != input.passages.len()
    {
        return Err("embedding output is not bound to its input".into());
    }
    match (&input.query, &output.query) {
        (None, None) => (),
        (Some(text), Some(query))
            if query.model_identity == identity
                && query.input_bytes == text.len()
                && query.source_sha256 == embedding::source_digest(text)
                && query.input_tokens > 0
                && query.input_tokens <= embedding::MAX_INPUT_TOKENS =>
        {
            validate_vector(&query.vector)?
        }
        _ => return Err("embedding query result does not match its complete input".into()),
    }
    let mut chunk_count = 0;
    for (text, passage) in input.passages.iter().zip(&output.passages) {
        if passage.model_identity != identity
            || passage.input_bytes != text.len()
            || !passage.complete
            || passage.covered_bytes != text.len()
            || passage.chunks.is_empty()
        {
            return Err("embedding passage result has incomplete coverage".into());
        }
        let mut end = 0;
        let mut last_start = 0;
        let mut token_sum = 0usize;
        for chunk in &passage.chunks {
            chunk_count += 1;
            token_sum = token_sum
                .checked_add(chunk.input_tokens)
                .ok_or("embedding token count overflow")?;
            if chunk_count > embedding::MAX_CHUNKS
                || chunk.start_byte > end
                || chunk.end_byte <= end
                || chunk.start_byte < last_start
                || chunk.start_byte >= chunk.end_byte
                || chunk.end_byte > text.len()
                || !text.is_char_boundary(chunk.start_byte)
                || !text.is_char_boundary(chunk.end_byte)
                || chunk.input_tokens == 0
                || chunk.input_tokens > embedding::MAX_INPUT_TOKENS
            {
                return Err("embedding chunk bounds do not cover the complete source".into());
            }
            validate_vector(&chunk.vector)?;
            last_start = chunk.start_byte;
            end = chunk.end_byte;
        }
        if end != text.len() {
            return Err("embedding passage tail was omitted".into());
        }
        if passage.content_tokens == 0 || passage.content_tokens > token_sum {
            return Err("embedding content token count is inconsistent".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::{ChunkEmbedding, ModelIdentity};
    use std::{
        fs,
        path::Path,
        sync::atomic::{AtomicU64, Ordering},
    };

    fn vector() -> Vec<f32> {
        let mut v = vec![0.0; embedding::DIMENSIONS];
        v[0] = 1.0;
        v
    }
    fn identity() -> ModelIdentity {
        embedding::expected_model_identity().unwrap()
    }
    fn request(root: &Path) -> Request {
        let input = Input {
            query: None,
            passages: vec!["fixture".into()],
        };
        Request {
            state_root: root.join("state"),
            release_id: "test-release".into(),
            job_id: "test-embedding".into(),
            package_root: root.join("package"),
            job_dir: root.join("job"),
            input_revision: revision(&serde_json::to_vec(&input).unwrap()),
            input,
        }
    }
    fn output(request: &Request, text: &str) -> Output {
        Output {
            schema: 1,
            input_revision: request.input_revision.clone(),
            query: None,
            passages: vec![PassageEmbedding {
                model_identity: identity(),
                input_bytes: text.len(),
                content_tokens: 3,
                covered_bytes: text.len(),
                complete: true,
                chunks: vec![ChunkEmbedding {
                    start_byte: 0,
                    end_byte: text.len(),
                    input_tokens: 6,
                    vector: vector(),
                }],
            }],
        }
    }
    #[test]
    fn result_validation_refuses_wrong_model_gaps_tail_and_invalid_vectors() {
        let request = request(Path::new("/private/fixture"));
        let text = "İstanbul için kaynak";
        let input = Input {
            query: None,
            passages: vec![text.into()],
        };
        let valid = output(&request, text);
        validate_output(&request.input_revision, &input, &valid).unwrap();
        let mut altered = valid.clone();
        altered.passages[0].content_tokens = usize::MAX;
        assert!(validate_output(&request.input_revision, &input, &altered).is_err());
        let mut altered = valid.clone();
        altered.passages[0].model_identity.id = "other".into();
        assert!(validate_output(&request.input_revision, &input, &altered).is_err());
        let mut altered = valid.clone();
        altered.passages[0].chunks[0].start_byte = 2;
        assert!(validate_output(&request.input_revision, &input, &altered).is_err());
        let mut altered = valid.clone();
        altered.passages[0].chunks[0].end_byte -= 1;
        assert!(validate_output(&request.input_revision, &input, &altered).is_err());
        let mut altered = valid.clone();
        altered.passages[0].chunks[0].vector[0] = f32::NAN;
        assert!(validate_output(&request.input_revision, &input, &altered).is_err());
        let mut altered = valid.clone();
        altered.passages[0].chunks[0].vector.pop();
        assert!(validate_output(&request.input_revision, &input, &altered).is_err());
        let mut altered = valid.clone();
        altered.passages[0].chunks[0].vector[0] = 0.2;
        assert!(validate_output(&request.input_revision, &input, &altered).is_err());
        let mut altered = valid.clone();
        altered.query = Some(QueryEmbedding {
            model_identity: identity(),
            source_sha256: embedding::source_digest(""),
            input_bytes: 0,
            input_tokens: 4,
            vector: vector(),
        });
        assert!(validate_output(&request.input_revision, &input, &altered).is_err());
        let mut altered = valid;
        altered.input_revision = "other".into();
        assert!(validate_output(&request.input_revision, &input, &altered).is_err());
    }
    #[test]
    fn aggregate_input_and_utf8_overlap_are_bounded() {
        let input = Input {
            query: Some("a".repeat(embedding::MAX_INPUT_BYTES)),
            passages: vec!["b".repeat(embedding::MAX_INPUT_BYTES); 16],
        };
        assert!(validate_input(&input).is_err());
        assert!(
            validate_input(&Input {
                query: None,
                passages: vec![" ".into()]
            })
            .is_err()
        );
        assert!(
            validate_input(&Input {
                query: None,
                passages: Vec::new()
            })
            .is_err()
        );
        let request = request(Path::new("/private/fixture"));
        let input = Input {
            query: None,
            passages: vec!["İstanbul".into()],
        };
        let mut out = output(&request, "İstanbul");
        out.passages[0].chunks = vec![
            ChunkEmbedding {
                start_byte: 0,
                end_byte: 4,
                input_tokens: 4,
                vector: vector(),
            },
            ChunkEmbedding {
                start_byte: 2,
                end_byte: "İstanbul".len(),
                input_tokens: 5,
                vector: vector(),
            },
        ];
        validate_output(&request.input_revision, &input, &out).unwrap();
        out.passages[0].chunks[1].start_byte = 1;
        assert!(validate_output(&request.input_revision, &input, &out).is_err());
    }
    #[cfg(unix)]
    struct Fixture(PathBuf);
    #[cfg(unix)]
    impl Fixture {
        fn new() -> Self {
            use std::os::unix::fs::DirBuilderExt;
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "mirket-embedding-job-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
            let root = fs::canonicalize(root).unwrap();
            for child in ["job", "state"] {
                fs::DirBuilder::new()
                    .mode(0o700)
                    .create(root.join(child))
                    .unwrap();
            }
            Self(root)
        }
        fn input(&self, input: &Input) -> Request {
            let mut request = request(&self.0);
            request.input = input.clone();
            request.input_revision = revision(&serde_json::to_vec(input).unwrap());
            request
        }
    }
    #[cfg(unix)]
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[cfg(unix)]
    #[test]
    fn pipe_input_and_output_are_bound_without_persistent_content() {
        let fixture = Fixture::new();
        let mut request = fixture.input(&Input {
            query: None,
            passages: vec!["memory".into()],
        });
        let result = output(&request, "memory");
        validate_outcome(&request, &result).unwrap();
        request.input.passages[0] = "change".into();
        assert!(validate_outcome(&request, &result).is_err());
        assert_eq!(fs::read_dir(&request.job_dir).unwrap().count(), 0);
    }
    #[cfg(unix)]
    async fn native_ps(args: &[&str]) -> Result<String, String> {
        let output = Command::new("/bin/ps")
            .args(args)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .output()
            .await
            .map_err(|_| "native process observation failed")?;
        if output.stdout.len() > 4 * 1024 * 1024 {
            return Err("native process observation oversized".into());
        }
        if !output.status.success() {
            // A child (including the inventory ps itself) may exit between the
            // PID snapshot and its narrow argument query. It is no match.
            return if args.contains(&"-p") {
                Ok(String::new())
            } else {
                Err("native process inventory unavailable".into())
            };
        }
        String::from_utf8(output.stdout).map_err(|_| "native process observation invalid".into())
    }
    #[cfg(unix)]
    async fn native_processes() -> Result<Vec<(u32, u32)>, String> {
        // Inventory only process identities; inspect arguments solely for the
        // test's observed descendants, never unrelated host command lines.
        Ok(native_ps(&["-axo", "pid=,ppid="])
            .await?
            .lines()
            .filter_map(|line| {
                let mut fields = line.split_whitespace();
                Some((fields.next()?.parse().ok()?, fields.next()?.parse().ok()?))
            })
            .collect())
    }
    #[cfg(unix)]
    async fn native_command(pid: u32) -> Result<String, String> {
        Ok(native_ps(&["-ww", "-p", &pid.to_string(), "-o", "args="])
            .await?
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "))
    }
    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn native_observation_handles_disappeared_children_without_model_execution() {
        let inventory = native_processes().await.unwrap();
        assert!(inventory.iter().any(|(pid, _)| *pid == std::process::id()));
        assert!(!native_command(std::process::id()).await.unwrap().is_empty());
        assert!(native_command(u32::MAX).await.unwrap().is_empty());
    }

    #[cfg(unix)]
    async fn observe_live_embedding_leaf(binary: &Path) -> Result<(u32, u32), String> {
        let executable = binary.to_str().ok_or("test executable path is not UTF-8")?;
        let worker = format!("{executable} __embedding_supervisor")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let leaf = format!("{executable} __embedding_leaf")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let rows = native_processes().await?;
                for (worker_pid, parent) in &rows {
                    if *parent != std::process::id() || native_command(*worker_pid).await? != worker
                    {
                        continue;
                    }
                    for (leaf_pid, parent) in &rows {
                        if parent == worker_pid && native_command(*leaf_pid).await? == leaf {
                            return Ok((*worker_pid, *leaf_pid));
                        }
                    }
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .map_err(|_| "did not observe a live owned embedding leaf before cancellation".to_owned())?
    }

    #[cfg(unix)]
    fn session_config(
        fixture: &Fixture,
        binary: &Path,
        package: PathBuf,
        id: &str,
    ) -> crate::embedding_session::SessionConfig {
        use crate::owned_scratch::{DirectoryStamp, OwnedScratch};
        let state = fixture.0.join("state");
        crate::embedding_session::SessionConfig {
            state_root: state.clone(),
            release_id: "warm-native-test".into(),
            session_id: id.into(),
            package_root: package,
            scratch: OwnedScratch::create(&state, &DirectoryStamp::inspect(&state).unwrap(), id)
                .unwrap(),
            current_exe: binary.into(),
        }
    }
    #[cfg(target_os = "macos")]
    fn model_session_config(
        fixture: &Fixture,
        binary: &Path,
        package: PathBuf,
        id: &str,
    ) -> crate::embedding_session::SessionConfig {
        use std::os::unix::fs::PermissionsExt;
        assert!(binary.is_absolute());
        let mut config = session_config(fixture, binary, package, id);
        let launcher = fixture.0.join(format!("network-denied-{id}"));
        let quoted_binary = format!("'{}'", binary.to_str().unwrap().replace('\'', "'\"'\"'"));
        // Test driver stays outside Seatbelt so /bin/ps can inspect its owned
        // descendants. Only this private launcher execs sandbox-exec, which
        // execs the exact Mirket supervisor. Its same-binary native model leaf
        // inherits `(deny network*)`. No production override or global setup.
        fs::write(&launcher, format!(
            "#!/bin/sh\nexec /usr/bin/sandbox-exec -p '(version 1) (allow default) (deny network*)' {quoted_binary} \"$@\"\n"
        )).unwrap();
        fs::set_permissions(&launcher, fs::Permissions::from_mode(0o700)).unwrap();
        config.current_exe = launcher;
        config
    }
    #[cfg(target_os = "macos")]
    #[tokio::test(flavor = "current_thread")]
    async fn actual_warm_embedding_reuse_and_cancellation_restore_capacity() {
        use crate::embedding_session::{Inference, SessionHandle};
        use tokio::time::Instant;
        let (Some(bin), Some(package)) = (
            std::env::var_os("MIRKET_TEST_BIN"),
            std::env::var_os("MIRKET_TEST_EMBEDDING_PACKAGE"),
        ) else {
            eprintln!("SKIP actual warm embedding: explicit binary and private package required");
            return;
        };
        let fixture = Fixture::new();
        let input = Input {
            query: Some("Which tools run on the host?".into()),
            passages: vec!["Araçlar ana bilgisayarda özel paketlerden çalışır.".into()],
        };
        let deadline = Instant::now() + Duration::from_secs(60);
        let session = SessionHandle::start(
            model_session_config(
                &fixture,
                Path::new(&bin),
                PathBuf::from(&package),
                "warm-first",
            ),
            Arc::new(AtomicBool::new(false)),
            deadline,
        )
        .await
        .unwrap();
        let owned = observe_live_embedding_leaf(Path::new(&bin)).await.unwrap();
        let inference = Inference {
            job_id: "native-query".into(),
            input_revision: revision(&serde_json::to_vec(&input).unwrap()),
            input: input.clone(),
        };
        for sequence in 1..=2 {
            let output = session
                .infer(
                    inference.clone(),
                    Arc::new(AtomicBool::new(false)),
                    deadline,
                )
                .await
                .unwrap();
            validate_warm_output(
                &WarmInference {
                    sequence,
                    job_id: inference.job_id.clone(),
                    input_revision: inference.input_revision.clone(),
                    input: input.clone(),
                },
                &output,
            )
            .unwrap();
            assert_eq!(
                observe_live_embedding_leaf(Path::new(&bin)).await.unwrap(),
                owned
            );
        }
        assert!(session.shutdown().await.cleanup_confirmed);
        let admission = crate::leases::Admission::open(&fixture.0.join("state")).unwrap();
        {
            let _a = admission.try_acquire().unwrap();
            let _b = admission.try_acquire().unwrap();
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let trigger = cancel.clone();
        let binary = PathBuf::from(&bin);
        let stop = tokio::spawn(async move {
            let observed = observe_live_embedding_leaf(&binary).await;
            trigger.store(true, Ordering::Release);
            observed
        });
        let result = SessionHandle::start(
            model_session_config(
                &fixture,
                Path::new(&bin),
                PathBuf::from(package),
                "warm-cancel",
            ),
            cancel,
            Instant::now() + Duration::from_secs(60),
        )
        .await;
        let (supervisor, leaf) = stop.await.unwrap().unwrap();
        let error = match result {
            Err(e) => e,
            Ok(session) => {
                session.shutdown().await;
                panic!("expected cancellation")
            }
        };
        assert!(
            error.stopped.as_ref().is_some_and(|s| s.cleanup_confirmed),
            "{error}"
        );
        assert!(
            !native_processes()
                .await
                .unwrap()
                .iter()
                .any(|(pid, _)| *pid == supervisor || *pid == leaf)
        );
        let _a = admission.try_acquire().unwrap();
        let _b = admission.try_acquire().unwrap();
    }
    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn actual_unavailable_package_reply_is_reaped_before_semantic_error() {
        use crate::embedding_session::SessionHandle;
        let Some(bin) = std::env::var_os("MIRKET_TEST_BIN") else {
            eprintln!("SKIP native unavailable-payload: explicit binary required");
            return;
        };
        let fixture = Fixture::new();
        let package = fixture.0.join("empty-package");
        fs::create_dir(&package).unwrap();
        for n in 0..2 {
            let result = SessionHandle::start(
                session_config(
                    &fixture,
                    Path::new(&bin),
                    package.clone(),
                    &format!("unavailable-{n}"),
                ),
                Arc::new(AtomicBool::new(false)),
                tokio::time::Instant::now() + Duration::from_secs(10),
            )
            .await;
            let error = match result {
                Err(e) => e,
                Ok(session) => {
                    session.shutdown().await;
                    panic!("empty package accepted")
                }
            };
            assert!(error.message.starts_with("embedding failed ("), "{error}");
            assert!(
                error.stopped.as_ref().is_some_and(|s| s.cleanup_confirmed),
                "{error:?}"
            );
            assert_eq!(
                fs::read_dir(fixture.0.join("state/jobs")).unwrap().count(),
                0
            );
            let admission = crate::leases::Admission::open(&fixture.0.join("state")).unwrap();
            let _a = admission.try_acquire().unwrap();
            let _b = admission.try_acquire().unwrap();
        }
    }
}

/// Session protocol is distinct from the retired one-result/EOF transport.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WarmInit {
    pub protocol_version: u8,
    pub release_id: String,
    pub session_id: String,
    pub package_root: PathBuf,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WarmInference {
    pub sequence: u64,
    pub job_id: String,
    pub input_revision: String,
    pub input: Input,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WarmReply {
    pub protocol_version: u8,
    pub release_id: String,
    pub session_id: String,
    pub body: WarmBody,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WarmBody {
    Ready {
        model: embedding::ModelIdentity,
    },
    Result {
        sequence: u64,
        job_id: String,
        output: Output,
    },
    Failed {
        sequence: u64,
        job_id: String,
        code: String,
    },
}

fn warm_read<R: BufRead>(reader: &mut R) -> Result<Option<Vec<u8>>, String> {
    let mut frame = Vec::new();
    reader
        .take(MAX_FRAME as u64 + 1)
        .read_until(b'\n', &mut frame)
        .map_err(|_| "embedding request read failed")?;
    if frame.is_empty() {
        return Ok(None);
    }
    if frame.len() > MAX_FRAME || frame.last() != Some(&b'\n') {
        return Err("embedding request frame invalid".into());
    }
    Ok(Some(frame))
}
fn warm_write(init: &WarmInit, body: WarmBody) -> Result<(), String> {
    let reply = WarmReply {
        protocol_version: 2,
        release_id: init.release_id.clone(),
        session_id: init.session_id.clone(),
        body,
    };
    let mut bytes = serde_json::to_vec(&reply).map_err(|_| "embedding reply encoding failed")?;
    bytes.push(b'\n');
    if bytes.len() > MAX_FRAME {
        return Err("embedding reply exceeds its frame limit".into());
    }
    let mut stdout = std::io::stdout().lock();
    stdout
        .write_all(&bytes)
        .and_then(|_| stdout.flush())
        .map_err(|_| "embedding reply delivery failed".into())
}

pub(crate) fn validate_warm_input(request: &WarmInference) -> Result<(), String> {
    if request.sequence == 0 {
        return Err("invalid embedding sequence".into());
    }
    ReleaseId::parse(&request.job_id).map_err(|_| "invalid embedding operation")?;
    validate_input(&request.input)?;
    if revision(&serde_json::to_vec(&request.input).map_err(|_| "embedding input encoding failed")?)
        != request.input_revision
    {
        return Err("embedding input revision mismatch".into());
    }
    Ok(())
}
pub(crate) fn validate_warm_output(request: &WarmInference, output: &Output) -> Result<(), String> {
    validate_output(&request.input_revision, &request.input, output)
}

/// Blocking, single-model leaf. The separate supervisor owns cancellation.
/// EOF while idle unloads the whole native process; no ORT reset/reload occurs.
pub fn serve_warm_leaf() -> Result<(), String> {
    let mut stdin = std::io::stdin().lock();
    let bytes = warm_read(&mut stdin)?.ok_or("embedding initialization absent")?;
    let init: WarmInit =
        serde_json::from_slice(&bytes).map_err(|_| "embedding initialization invalid")?;
    if init.protocol_version != 2 || !init.package_root.is_absolute() {
        return Err("embedding initialization invalid".into());
    }
    ReleaseId::parse(&init.release_id).map_err(|_| "invalid embedding release")?;
    ReleaseId::parse(&init.session_id).map_err(|_| "invalid embedding session")?;
    let loaded = VerifiedEmbeddingPayload::open(&init.package_root).and_then(EmbeddingModel::load);
    let mut model = match loaded {
        Ok(model) => model,
        Err(error) => {
            return warm_write(
                &init,
                WarmBody::Failed {
                    sequence: 0,
                    job_id: String::new(),
                    code: error.code().into(),
                },
            );
        }
    };
    warm_write(
        &init,
        WarmBody::Ready {
            model: model.model_identity().clone(),
        },
    )?;
    let mut expected = 1u64;
    while let Some(bytes) = warm_read(&mut stdin)? {
        let request: WarmInference =
            serde_json::from_slice(&bytes).map_err(|_| "embedding request invalid")?;
        validate_warm_input(&request)?;
        if request.sequence != expected {
            return Err("embedding sequence mismatch".into());
        }
        expected = expected
            .checked_add(1)
            .ok_or("embedding sequence exhausted")?;
        let result = warm_compute(&mut model, &request);
        let body = match result {
            Ok(output) => WarmBody::Result {
                sequence: request.sequence,
                job_id: request.job_id,
                output,
            },
            Err(code) => {
                warm_write(
                    &init,
                    WarmBody::Failed {
                        sequence: request.sequence,
                        job_id: request.job_id,
                        code,
                    },
                )?;
                return Ok(());
            }
        };
        warm_write(&init, body)?;
    }
    Ok(())
}
fn warm_compute(model: &mut EmbeddingModel, request: &WarmInference) -> Result<Output, String> {
    let cancel = AtomicBool::new(false);
    let query = request
        .input
        .query
        .as_ref()
        .map(|text| model.embed_query(text, &cancel))
        .transpose()
        .map_err(|e| e.code().to_owned())?;
    let texts: Vec<&str> = request.input.passages.iter().map(String::as_str).collect();
    let passages = if texts.is_empty() {
        Vec::new()
    } else {
        model
            .embed_passages(&texts, &cancel)
            .map_err(|e| e.code().to_owned())?
    };
    let output = Output {
        schema: 1,
        input_revision: request.input_revision.clone(),
        query,
        passages,
    };
    validate_warm_output(request, &output).map_err(|_| "embedding_output_invalid")?;
    Ok(output)
}
