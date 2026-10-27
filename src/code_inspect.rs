//! Bounded original-source JavaScript/TypeScript transport. Source scope stays in the parent;
//! the finite private Node leaf receives only UTF-8 bytes and source identity.
use crate::{
    engine::revision,
    process::{AdapterError, OwnedChild, cancelled, drain_stderr, read_frame, validate_job_dir},
    release::ReleaseId,
    scope::ProjectScope,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    process::Command,
};

pub const MAX_SOURCE_BYTES: usize = 1024 * 1024;
pub const MAX_REQUEST_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_RESULT_BYTES: usize = 32 * 1024;
pub const ADAPTER_NAME: &str = "code-inspect-main.js";

#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Js,
    Ts,
    Jsx,
    Tsx,
}

pub type CodeLanguage = Language;

#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InspectRequest<L = Language> {
    pub path: PathBuf,
    pub language: L,
    pub pointer: String,
    pub expected_revision: Option<String>,
    pub start_child: usize,
    pub limit: usize,
}
impl<L: Into<Language>> InspectRequest<L> {
    pub fn into_shared(self) -> InspectRequest<Language> {
        InspectRequest {
            path: self.path,
            language: self.language.into(),
            pointer: self.pointer,
            expected_revision: self.expected_revision,
            start_child: self.start_child,
            limit: self.limit,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Payload {
    pub node: PathBuf,
    pub adapter: PathBuf,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub language: Language,
    pub pointer: String,
    pub expected_revision: String,
    pub start_child: usize,
    pub limit: usize,
    pub max_response_bytes: usize,
}

// Deliberately no Debug: original source must not enter diagnostic output.
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LeafRequest {
    pub protocol_version: u32,
    pub release_id: String,
    pub job_id: String,
    pub source_revision: String,
    pub source_utf8: String,
    pub request: Selection,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub result: Value,
    pub stderr_bytes: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Response {
    protocol_version: u32,
    release_id: String,
    job_id: String,
    result: Value,
}

fn encoded(value: &impl Serialize) -> Result<usize, String> {
    serde_json::to_vec(value)
        .map(|value| value.len())
        .map_err(|_| "code inspection cannot be encoded".into())
}
fn bounded_path(path: &Path) -> bool {
    path.is_absolute()
        && path
            .to_str()
            .is_some_and(|value| value.len() <= 4096 && !value.contains('\0'))
}
fn pointer(value: &str) -> bool {
    value.len() <= 4096
        && (value.is_empty()
            || value.starts_with('/')
                && value[1..].split('/').all(|part| {
                    !part.is_empty()
                        && (part == "0" || !part.starts_with('0'))
                        && part.bytes().all(|byte| byte.is_ascii_digit())
                }))
}
pub fn validate_input<L>(request: &InspectRequest<L>) -> Result<(), String> {
    if !bounded_path(&request.path)
        || !pointer(&request.pointer)
        || request.start_child > 32768
        || !(1..=100).contains(&request.limit)
        || request
            .expected_revision
            .as_deref()
            .is_some_and(|value| !crate::design::is_revision(value))
        || (!request.pointer.is_empty() || request.start_child > 0)
            && request.expected_revision.is_none()
    {
        return Err("code inspection requires an absolute bounded path, canonical ordinal pointer, exact revision for selection/continuation and bounded pagination".into());
    }
    Ok(())
}
pub fn validate_request(request: &LeafRequest) -> Result<(), String> {
    let selection = &request.request;
    if request.protocol_version != 1
        || ReleaseId::parse(&request.release_id).is_err()
        || ReleaseId::parse(&request.job_id).is_err()
        || request.source_utf8.len() > MAX_SOURCE_BYTES
        || revision(request.source_utf8.as_bytes()) != request.source_revision
        || selection.expected_revision != request.source_revision
        || !pointer(&selection.pointer)
        || selection.start_child > 32768
        || !(1..=100).contains(&selection.limit)
        || !(1024..=MAX_RESULT_BYTES).contains(&selection.max_response_bytes)
    {
        return Err(
            "code inspection leaf request identity, revision, source or selection is invalid"
                .into(),
        );
    }
    // Preserve the existing worker cap. Highly escaped source can refuse even
    // below the raw byte cap; no truncation or alternate source file is used.
    if encoded(request)? >= MAX_REQUEST_BYTES - 16_384 {
        return Err("code inspection source exceeds its escaped JSON transport budget".into());
    }
    Ok(())
}

pub struct Snapshot {
    scope: ProjectScope,
    path: PathBuf,
    pub request: LeafRequest,
}
pub enum SourceRead {
    Current(Box<Snapshot>),
    Changed(Value),
}
fn envelope(
    release: &str,
    scope: &ProjectScope,
    path: &Path,
    selection: &Selection,
    outcome: Value,
) -> Value {
    json!({"release_id":release,"inspection":{"project_root":scope.root(),"path":path,
        "language":selection.language,"pointer":selection.pointer,"outcome":outcome}})
}
fn changed(current: Option<String>) -> Value {
    json!({"status":"changed","current_revision":current})
}

pub fn read_source(
    project: &Path,
    input: InspectRequest<Language>,
    release: String,
    job_id: String,
) -> Result<SourceRead, String> {
    validate_input(&input)?;
    if !bounded_path(project) {
        return Err("code inspection requires an explicit bounded project root".into());
    }
    let scope = ProjectScope::open(project).map_err(|error| error.to_string())?;
    let source = scope
        .read_text(&input.path, MAX_SOURCE_BYTES)
        .map_err(|error| error.to_string())?;
    let current = revision(source.text.as_bytes());
    let mut selection = Selection {
        language: input.language,
        pointer: input.pointer,
        expected_revision: current.clone(),
        start_child: input.start_child,
        limit: input.limit,
        max_response_bytes: 1024,
    };
    let overhead = encoded(&envelope(
        &release,
        &scope,
        &source.path,
        &selection,
        Value::Null,
    ))? - 4;
    selection.max_response_bytes = MAX_RESULT_BYTES
        .checked_sub(overhead)
        .filter(|budget| *budget >= 1024)
        .ok_or("code inspection path/release metadata exceeds its result budget")?;
    if input
        .expected_revision
        .is_some_and(|expected| expected != current)
    {
        return Ok(SourceRead::Changed(envelope(
            &release,
            &scope,
            &source.path,
            &selection,
            changed(Some(current)),
        )));
    }
    let request = LeafRequest {
        protocol_version: 1,
        release_id: release,
        job_id,
        source_revision: current,
        source_utf8: source.text,
        request: selection,
    };
    validate_request(&request)?;
    Ok(SourceRead::Current(Box::new(Snapshot {
        scope,
        path: source.path,
        request,
    })))
}
impl Snapshot {
    pub fn finish(self, result: Value) -> Result<Value, String> {
        validate_result(&result, &self.request)?;
        let current = self
            .scope
            .read_bytes(&self.path, MAX_SOURCE_BYTES)
            .ok()
            .map(|source| revision(&source.bytes));
        let outcome = if current.as_deref() == Some(self.request.source_revision.as_str()) {
            result
        } else {
            changed(current)
        };
        let response = envelope(
            &self.request.release_id,
            &self.scope,
            &self.path,
            &self.request.request,
            outcome,
        );
        if encoded(&response)? > MAX_RESULT_BYTES {
            return Err("code inspection exceeds its complete 32 KiB result budget".into());
        }
        Ok(response)
    }
}

fn object<'a>(
    value: &'a Value,
    allowed: &[&str],
    required: &[&str],
) -> Result<&'a serde_json::Map<String, Value>, String> {
    let object = value
        .as_object()
        .ok_or("code inspection result object is invalid")?;
    if object.keys().any(|key| !allowed.contains(&key.as_str()))
        || required.iter().any(|key| !object.contains_key(*key))
    {
        return Err("code inspection result fields do not match the finite contract".into());
    }
    Ok(object)
}
fn small_token(value: &Value) -> bool {
    value.as_str().is_some_and(|value| {
        !value.is_empty()
            && value.len() <= 128
            && value
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_')
    })
}
fn syntax_kind(value: &Value) -> bool {
    value.as_str().is_some_and(|value| {
        !value.is_empty() && value.len() <= 128 && value.bytes().all(|byte| byte.is_ascii_graphic())
    })
}
fn range(value: &Value, source: &str) -> bool {
    let Ok(fields) = object(
        value,
        &["start_byte", "end_byte"],
        &["start_byte", "end_byte"],
    ) else {
        return false;
    };
    let (Some(start), Some(end)) = (fields["start_byte"].as_u64(), fields["end_byte"].as_u64())
    else {
        return false;
    };
    start <= end
        && end <= source.len() as u64
        && source.is_char_boundary(start as usize)
        && source.is_char_boundary(end as usize)
}
fn node(value: &Value, request: &LeafRequest) -> Result<(), String> {
    let fields = object(
        value,
        &[
            "pointer",
            "kind",
            "name",
            "location",
            "location_reason",
            "name_location",
            "value_location",
            "child_count",
        ],
        &["pointer", "kind", "location", "child_count"],
    )?;
    if !fields["pointer"].as_str().is_some_and(pointer)
        || !syntax_kind(&fields["kind"])
        || !fields["child_count"]
            .as_u64()
            .is_some_and(|count| count <= 32768)
        || fields
            .get("name")
            .is_some_and(|value| value.as_str().is_none_or(|value| value.len() > 4096))
        || (fields["location"].is_null() && !fields.get("location_reason").is_some_and(small_token))
        || (!fields["location"].is_null()
            && (fields.contains_key("location_reason")
                || !range(&fields["location"], &request.source_utf8)))
        || ["name_location", "value_location"].iter().any(|key| {
            fields
                .get(*key)
                .is_some_and(|value| !range(value, &request.source_utf8))
        })
    {
        return Err("code inspection node metadata or original UTF-8 location is invalid".into());
    }
    Ok(())
}
pub fn validate_result(result: &Value, request: &LeafRequest) -> Result<(), String> {
    if encoded(result)? > request.request.max_response_bytes {
        return Err("code inspection result exceeds its reserved response budget".into());
    }
    let fields = object(
        result,
        &[
            "status",
            "backend",
            "revision",
            "source_bytes",
            "structure_complete",
            "coverage",
            "issues",
            "issues_truncated",
            "reason",
            "selected",
            "children",
            "child_count",
            "start_child",
            "complete",
            "continuation",
            "blocked_child",
        ],
        &[
            "status",
            "backend",
            "revision",
            "source_bytes",
            "structure_complete",
            "coverage",
            "issues",
        ],
    )?;
    let backend = json!({"name":"typescript","version":"5.9.3"});
    if fields["backend"] != backend
        || fields["revision"] != request.source_revision
        || fields["source_bytes"] != request.source_utf8.len()
        || !fields["structure_complete"].is_boolean()
        || fields
            .get("issues_truncated")
            .is_some_and(|value| !value.is_boolean())
        || !fields["coverage"].as_array().is_some_and(|values| {
            values.len() <= 8
                && values
                    .iter()
                    .all(|v| v.as_str().is_some_and(|s| s.len() <= 512))
        })
    {
        return Err(
            "code inspection result backend, source identity or coverage is invalid".into(),
        );
    }
    let issues = fields["issues"]
        .as_array()
        .filter(|issues| issues.len() <= 16)
        .ok_or("code inspection issue sample exceeds its limit")?;
    for issue in issues {
        let fields = object(issue, &["reason", "location"], &["reason", "location"])?;
        if !small_token(&fields["reason"])
            || !fields["location"].is_null() && !range(&fields["location"], &request.source_utf8)
        {
            return Err("code inspection issue sample is invalid".into());
        }
    }
    if let Some(selected) = fields.get("selected") {
        node(selected, request)?;
        if selected["pointer"] != request.request.pointer {
            return Err("code inspection selection pointer changed".into());
        }
    }
    match fields["status"].as_str() {
        Some("page") => {
            let selected = fields
                .get("selected")
                .ok_or("code inspection page has no selected node")?;
            let children = fields
                .get("children")
                .and_then(Value::as_array)
                .filter(|children| children.len() <= request.request.limit)
                .ok_or("code inspection page children exceed their limit")?;
            let count = fields
                .get("child_count")
                .and_then(Value::as_u64)
                .filter(|count| *count <= 32768)
                .ok_or("code inspection child count is invalid")? as usize;
            if selected["child_count"] != count
                || fields.get("start_child") != Some(&json!(request.request.start_child))
                || request.request.start_child > count
                || request.request.start_child + children.len() > count
            {
                return Err("code inspection page child accounting is invalid".into());
            }
            let mut last = 0;
            for (index, child) in children.iter().enumerate() {
                node(child, request)?;
                if child["pointer"]
                    != format!(
                        "{}/{}",
                        request.request.pointer,
                        request.request.start_child + index
                    )
                {
                    return Err("code inspection child occurrence pointer changed".into());
                }
                if let Some(start) = child["location"]["start_byte"].as_u64() {
                    if start < last {
                        return Err("code inspection children are not source ordered".into());
                    }
                    last = start;
                }
            }
            let end = request.request.start_child + children.len();
            let complete = fields
                .get("complete")
                .and_then(Value::as_bool)
                .ok_or("code inspection page completion is missing")?;
            if complete != (end == count) || !complete && children.is_empty() {
                return Err("code inspection pagination does not make progress".into());
            }
            let continuation = if complete {
                Value::Null
            } else {
                json!({"pointer":request.request.pointer,"expected_revision":request.source_revision,"start_child":end,"limit":request.request.limit})
            };
            if fields.get("continuation") != Some(&continuation)
                || fields.contains_key("reason")
                || fields.contains_key("blocked_child")
            {
                return Err("code inspection continuation does not match its page".into());
            }
        }
        Some("budget_blocked") => {
            if !fields.get("reason").is_some_and(small_token) {
                return Err("code inspection metadata block has no bounded reason".into());
            }
            let blocked = fields
                .get("blocked_child")
                .ok_or("code inspection metadata block has no source identity")?;
            let blocked = object(
                blocked,
                &["pointer", "kind", "location"],
                &["pointer", "kind", "location"],
            )?;
            if !blocked["pointer"].as_str().is_some_and(pointer)
                || !syntax_kind(&blocked["kind"])
                || !blocked["location"].is_null()
                    && !range(&blocked["location"], &request.source_utf8)
                || fields
                    .get("children")
                    .is_some_and(|value| value.as_array().is_none_or(|a| !a.is_empty()))
                || fields
                    .get("continuation")
                    .is_some_and(|value| !value.is_null())
            {
                return Err("code inspection metadata block is invalid".into());
            }
        }
        Some("missing_pointer" | "invalid_start" | "refused") => {
            if fields.contains_key("children")
                || fields.contains_key("continuation")
                || fields.contains_key("blocked_child")
                || fields["status"] == "refused"
                    && (!fields.get("reason").is_some_and(small_token)
                        || fields.contains_key("selected")
                        || fields["structure_complete"] != false)
            {
                return Err(
                    "code inspection refusal/missing result contains a partial page".into(),
                );
            }
        }
        _ => return Err("code inspection result status is not valid for the bound source".into()),
    }
    Ok(())
}
pub fn validate_response(bytes: &[u8], request: &LeafRequest) -> Result<Value, String> {
    if bytes.len() > 65_536 {
        return Err("code inspection response frame exceeds its limit".into());
    }
    let response: Response = serde_json::from_slice(bytes)
        .map_err(|_| "code inspection response does not match its strict schema")?;
    if response.protocol_version != 1
        || response.release_id != request.release_id
        || response.job_id != request.job_id
    {
        return Err(
            "code inspection response protocol/release/job does not match its request".into(),
        );
    }
    validate_result(&response.result, request)?;
    Ok(response.result)
}

#[derive(PartialEq, Eq)]
struct Stamp {
    bytes: u64,
    modified: Option<SystemTime>,
    #[cfg(unix)]
    identity: (u64, u64),
}
fn payload_stamps(payload: &Payload) -> Result<Vec<Stamp>, String> {
    [
        &payload.node,
        &payload.adapter,
        &payload.adapter.with_file_name("code-source-inspect.js"),
    ]
    .into_iter()
    .map(|path| {
        if !bounded_path(path) || path.canonicalize().ok().as_deref() != Some(path.as_path()) {
            return Err(
                "code inspection runtime requires explicit canonical private payload paths".into(),
            );
        }
        let metadata = std::fs::symlink_metadata(path)
            .map_err(|_| "code inspection runtime payload is unavailable")?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err("code inspection runtime payload must be a real regular file".into());
        }
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        Ok(Stamp {
            bytes: metadata.len(),
            modified: metadata.modified().ok(),
            #[cfg(unix)]
            identity: (metadata.dev(), metadata.ino()),
        })
    })
    .collect()
}
fn unconfirmed(message: String) -> AdapterError {
    AdapterError {
        message,
        cleanup_confirmed: false,
    }
}
pub async fn run_leaf(
    payload: &Payload,
    request: &LeafRequest,
    job_dir: &Path,
    cancel: Arc<AtomicBool>,
    deadline: tokio::time::Instant,
) -> Result<Outcome, AdapterError> {
    if !cfg!(unix) {
        return Err("native code inspection ownership is not qualified on this platform".into());
    }
    validate_request(request)?;
    validate_job_dir(job_dir)?;
    let stamps = payload_stamps(payload)?;
    if cancel.load(Ordering::Acquire) || tokio::time::Instant::now() >= deadline {
        return Err("code inspection cancelled or expired before launch".into());
    }
    let deadline = deadline.min(tokio::time::Instant::now() + Duration::from_secs(30));
    let mut frame =
        serde_json::to_vec(request).map_err(|_| "code inspection request cannot be encoded")?;
    frame.push(b'\n');
    let mut command = Command::new(&payload.node);
    command
        .arg(&payload.adapter)
        .arg("--release-id")
        .arg(&request.release_id)
        .env_clear()
        .env("TMPDIR", job_dir)
        .env("TMP", job_dir)
        .env("TEMP", job_dir)
        .current_dir(job_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.as_std_mut().process_group(0);
    }
    let child = command
        .spawn()
        .map_err(|_| "private code inspection Node leaf could not launch")?;
    let mut owned = OwnedChild::new(child).map_err(unconfirmed)?;
    let (Some(mut stdin), Some(stdout), Some(stderr)) = (
        owned.child.stdin.take(),
        owned.child.stdout.take(),
        owned.child.stderr.take(),
    ) else {
        return match owned.terminate_and_reap().await {
            Ok(()) => Err("code inspection leaf pipes are unavailable".into()),
            Err(error) => Err(unconfirmed(error)),
        };
    };
    let result: Result<Outcome, String> = {
        let execution = async {
            let protocol = async {
                stdin
                    .write_all(&frame)
                    .await
                    .map_err(|_| "code inspection request write failed")?;
                stdin
                    .shutdown()
                    .await
                    .map_err(|_| "code inspection request close failed")?;
                drop(stdin);
                let mut stdout = BufReader::new(stdout);
                let bytes = read_frame(&mut stdout).await?;
                let result = validate_response(&bytes, request)?;
                let mut extra = [0u8; 1];
                if stdout
                    .read(&mut extra)
                    .await
                    .map_err(|_| "code inspection stdout close failed")?
                    != 0
                {
                    return Err("code inspection leaf emitted trailing output".to_owned());
                }
                Ok(result)
            };
            let (result, stderr_bytes) = tokio::try_join!(protocol, drain_stderr(stderr))?;
            if !owned.wait().await?.success() {
                return Err("code inspection leaf exited unsuccessfully".into());
            }
            Ok(Outcome {
                result,
                stderr_bytes,
            })
        };
        tokio::select! { biased; _ = cancelled(cancel) => Err("code inspection cancelled".into()),
        result = tokio::time::timeout_at(deadline,execution) => result.map_err(|_| "code inspection deadline exceeded".to_owned()).and_then(|result| result) }
    };
    if result.is_err()
        && !owned.is_reaped()
        && let Err(error) = owned.terminate_and_reap().await
    {
        return Err(unconfirmed(format!(
            "code inspection leaf cleanup unconfirmed: {error}"
        )));
    }
    if payload_stamps(payload)? != stamps {
        return Err("code inspection payload changed during execution".into());
    }
    result.map_err(AdapterError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "mirket-code-inspect-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
            }
            Self(path.canonicalize().unwrap())
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn request(text: &str) -> LeafRequest {
        LeafRequest {
            protocol_version: 1,
            release_id: "0.0.1:code-inspect-test".into(),
            job_id: "op-code-inspect-test".into(),
            source_revision: revision(text.as_bytes()),
            source_utf8: text.into(),
            request: Selection {
                language: Language::Ts,
                pointer: String::new(),
                expected_revision: revision(text.as_bytes()),
                start_child: 0,
                limit: 100,
                max_response_bytes: 30000,
            },
        }
    }
    fn page(request: &LeafRequest) -> Value {
        json!({"status":"page","backend":{"name":"typescript","version":"5.9.3"},"revision":request.source_revision,"source_bytes":request.source_utf8.len(),
            "structure_complete":true,"coverage":[],"issues":[],"selected":{"pointer":"","kind":"document","location":{"start_byte":0,"end_byte":request.source_utf8.len()},"child_count":1},
            "children":[{"pointer":"/0","kind":"text","location":{"start_byte":0,"end_byte":request.source_utf8.len()},"child_count":0}],"child_count":1,"start_child":0,"complete":true,"continuation":null})
    }
    fn input(path: PathBuf) -> InspectRequest<Language> {
        InspectRequest {
            path,
            language: Language::Ts,
            pointer: String::new(),
            expected_revision: None,
            start_child: 0,
            limit: 100,
        }
    }
    #[test]
    fn code_inspection_public_languages_are_finite() {
        for language in [
            "html",
            "css",
            "js",
            "ts",
            "jsx",
            "tsx",
            "python",
            "bash",
            "powershell",
            "json",
            "javascript",
        ] {
            let args = json!({"expected_release_id":"0.0.1:test","project_root":"/project",
                "request":{"path":"/project/source","language":language,"pointer":"",
                    "expected_revision":null,"start_child":0,"limit":100}});
            assert_eq!(
                serde_json::from_value::<crate::engine::CodeInspectArgs>(args).is_ok(),
                ["js", "ts", "jsx", "tsx"].contains(&language)
            );
        }
    }
    #[test]
    fn code_inspect_validates_fixed_backend_and_original_spans_for_all_script_kinds() {
        for language in [Language::Js, Language::Ts, Language::Jsx, Language::Tsx] {
            let mut request = request("\u{feff}// 😀\r\nconst çay = 1;");
            request.request.language = language;
            let mut result = page(&request);
            result["backend"] = json!({"name":"typescript","version":"5.9.3"});
            validate_result(&result, &request).unwrap();
            let mut wrong_backend = result.clone();
            wrong_backend["backend"]["version"] = json!("6.0.3");
            assert!(validate_result(&wrong_backend, &request).is_err());
            result["children"][0]["location"]["start_byte"] = json!(1);
            assert!(validate_result(&result, &request).is_err());
        }
    }
    #[test]
    fn code_inspect_response_rejects_wrong_identity_fake_completion_and_surrogate_interior() {
        let request = request("😀çay\r\n");
        let valid = page(&request);
        validate_result(&valid, &request).unwrap();
        for mutate in 0..8 {
            let mut result = valid.clone();
            match mutate {
                0 => result["revision"] = json!(revision(b"changed")),
                1 => result["source_bytes"] = json!(1),
                2 => result["backend"]["version"] = json!("unadmitted"),
                3 => result["children"][0]["location"]["start_byte"] = json!(1),
                4 => result["children"][0]["pointer"] = json!("/7"),
                5 => result["children"] = json!([]),
                6 => result["continuation"] = json!({"source_path":"/never-read"}),
                _ => result["unexpected"] = json!(true),
            }
            assert!(
                validate_result(&result, &request).is_err(),
                "mutation {mutate}"
            );
        }
        let response = json!({"protocol_version":1,"release_id":request.release_id,"job_id":"wrong","result":valid});
        assert!(validate_response(&serde_json::to_vec(&response).unwrap(), &request).is_err());
    }
    #[test]
    fn code_inspect_snapshot_discards_spans_after_saved_source_drift() {
        let fixture = Fixture::new();
        let path = fixture.0.join("source.html");
        std::fs::write(&path, "<p>çay</p>\r\n").unwrap();
        let SourceRead::Current(snapshot) = read_source(
            &fixture.0,
            input(path.clone()),
            "0.0.1:code-inspect-test".into(),
            "op-code-inspect-test".into(),
        )
        .unwrap() else {
            panic!("expected current snapshot");
        };
        let result = page(&snapshot.request);
        std::fs::write(&path, "<p>changed</p>\r\n").unwrap();
        let response = snapshot.finish(result).unwrap();
        assert_eq!(response["inspection"]["outcome"]["status"], "changed");
        assert!(response["inspection"]["outcome"].get("children").is_none());
        let mut stale = input(path.clone());
        stale.expected_revision = Some(revision(b"stale"));
        assert!(matches!(
            read_source(
                &fixture.0,
                stale,
                "0.0.1:code-inspect-test".into(),
                "op-code-inspect-test".into()
            )
            .unwrap(),
            SourceRead::Changed(_)
        ));
    }
    #[test]
    fn code_inspect_input_rejects_scope_escapes_symlinks_and_escaped_frame_overflow() {
        let fixture = Fixture::new();
        assert!(
            read_source(
                &fixture.0,
                input(fixture.0.join("../outside")),
                "0.0.1:code-inspect-test".into(),
                "op-code-inspect-test".into()
            )
            .is_err()
        );
        #[cfg(unix)]
        {
            let target = fixture.0.join("source.html");
            std::fs::write(&target, "<p/>").unwrap();
            let link = fixture.0.join("link.html");
            std::os::unix::fs::symlink(&target, &link).unwrap();
            assert!(
                read_source(
                    &fixture.0,
                    input(link),
                    "0.0.1:code-inspect-test".into(),
                    "op-code-inspect-test".into()
                )
                .is_err()
            );
        }
        let request = request(&"\u{1}".repeat(MAX_SOURCE_BYTES));
        assert!(
            validate_request(&request)
                .unwrap_err()
                .contains("escaped JSON")
        );
    }
    fn selected_node() -> Option<PathBuf> {
        let Some(path) = std::env::var_os("MIRKET_TEST_NODE") else {
            eprintln!("SKIP native code inspection transport: explicit MIRKET_TEST_NODE required");
            return None;
        };
        let path = PathBuf::from(path);
        assert!(path.is_absolute());
        Some(path.canonicalize().unwrap())
    }
    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn code_inspect_hostile_incomplete_and_oversized_leaf_frames_are_reaped() {
        let Some(node) = selected_node() else {
            return;
        };
        let fixture = Fixture::new();
        let adapter = fixture.0.join(ADAPTER_NAME);
        std::fs::write(fixture.0.join("code-source-inspect.js"), "// test sentinel").unwrap();
        for (index, output) in [
            "process.stdout.end('{incomplete')",
            "process.stdout.write('x'.repeat(65537))",
        ]
        .into_iter()
        .enumerate()
        {
            let pid_path = fixture.0.join(format!("pid-{index}"));
            let code = format!(
                "require('node:fs').writeFileSync({},String(process.pid));{output};setInterval(()=>{{}},1000);",
                serde_json::to_string(&pid_path).unwrap()
            );
            std::fs::write(&adapter, code).unwrap();
            let error = run_leaf(
                &Payload {
                    node: node.clone(),
                    adapter: adapter.clone(),
                },
                &request("<p/>"),
                &fixture.0,
                Arc::new(AtomicBool::new(false)),
                tokio::time::Instant::now() + Duration::from_secs(5),
            )
            .await
            .unwrap_err();
            assert!(error.cleanup_confirmed, "{error}");
            let pid: i32 = std::fs::read_to_string(pid_path).unwrap().parse().unwrap();
            assert!(pid > 0);
            // SAFETY: signal zero observes only this test's recorded child.
            assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::ESRCH)
            );
        }
    }
    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn code_inspect_cancelled_leaf_is_reaped_before_completion() {
        let Some(node) = selected_node() else {
            return;
        };
        let fixture = Fixture::new();
        let adapter = fixture.0.join(ADAPTER_NAME);
        let pid_path = fixture.0.join("pid");
        std::fs::write(fixture.0.join("code-source-inspect.js"), "// test sentinel").unwrap();
        std::fs::write(&adapter,format!("require('node:fs').writeFileSync({},String(process.pid));setInterval(()=>{{}},1000);",serde_json::to_string(&pid_path).unwrap())).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let watcher_cancel = cancel.clone();
        let watcher = async {
            tokio::time::timeout(Duration::from_secs(3), async {
                while !pid_path.exists() {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await
            .unwrap();
            watcher_cancel.store(true, Ordering::Release);
        };
        let leaf = request("<p/>");
        let payload = Payload { node, adapter };
        let (result, ()) = tokio::join!(
            run_leaf(
                &payload,
                &leaf,
                &fixture.0,
                cancel,
                tokio::time::Instant::now() + Duration::from_secs(5)
            ),
            watcher
        );
        let error = result.unwrap_err();
        assert!(error.cleanup_confirmed);
        assert!(error.message.contains("cancelled"));
        let pid: i32 = std::fs::read_to_string(pid_path).unwrap().parse().unwrap();
        // SAFETY: signal zero observes only this test's recorded child.
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
    }
    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn code_inspect_real_worker_returns_page_and_releases_admission() {
        let Some(node) = selected_node() else {
            return;
        };
        let (Some(bin), Some(adapter)) = (
            std::env::var_os("MIRKET_TEST_BIN"),
            std::env::var_os("MIRKET_TEST_CODE_INSPECT_ADAPTER"),
        ) else {
            eprintln!(
                "SKIP owned code inspection worker: explicit MIRKET_TEST_BIN/MIRKET_TEST_CODE_INSPECT_ADAPTER required"
            );
            return;
        };
        let fixture = Fixture::new();
        for (language, source, backend) in [
            (
                Language::Js,
                "\u{feff}// 😀\r\nfunction çay() { return 1; }",
                "typescript",
            ),
            (
                Language::Ts,
                "\u{feff}// 😀\r\nconst çay: number = 1;",
                "typescript",
            ),
            (
                Language::Jsx,
                "\u{feff}// 😀\r\nconst çay = <Box/>;",
                "typescript",
            ),
            (
                Language::Tsx,
                "\u{feff}// 😀\r\nconst çay = <Box/>;",
                "typescript",
            ),
        ] {
            let mut request = request(source);
            request.request.language = language;
            let result = crate::worker::run_code_inspect_owned(
                &PathBuf::from(&bin),
                crate::worker::CodeInspectWork {
                    state_root: fixture.0.clone(),
                    payload: Payload {
                        node: node.clone(),
                        adapter: PathBuf::from(&adapter),
                    },
                    request: request.clone(),
                },
                Arc::new(AtomicBool::new(false)),
            )
            .await
            .unwrap();
            assert_eq!(result.result["status"], "page");
            assert_eq!(result.result["revision"], request.source_revision);
            assert_eq!(result.result["backend"]["name"], backend);
        }
        let admission = crate::leases::Admission::open(&fixture.0).unwrap();
        let _first = admission.try_acquire().unwrap();
        let _second = admission.try_acquire().unwrap();
        assert!(!fixture.0.join("heavy-slot-0.unresolved").exists());
        assert!(!fixture.0.join("heavy-slot-1.unresolved").exists());
    }
}
