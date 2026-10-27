//! Finite semantic navigation over explicitly selected saved source snapshots.
//! Rust binds paths/configuration. Node receives immutable selected snapshots
//! and explicit import bindings, without project discovery or execution.
use crate::{
    engine::revision,
    process::{AdapterError, OwnedChild, cancelled, drain_stderr, read_frame, validate_job_dir},
    release::ReleaseId,
    scope::ProjectScope,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashSet,
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

pub const ADAPTER_NAME: &str = "code-navigation-main.js";
pub const MAX_SOURCE_BYTES: usize = 262_144;
pub const MAX_TOTAL_BYTES: usize = 1_048_576;
pub const MAX_RESULT_BYTES: usize = 32_768;
pub(crate) const MAX_REQUEST_BYTES: usize = 2_097_152;
const MAX_TYPESCRIPT_SOURCES: usize = 16;
const MAX_TYPESCRIPT_PROJECT_SOURCES: usize = 8;
const MAX_TYPESCRIPT_DECLARATIONS: usize = 8;
const MAX_TYPESCRIPT_IMPORTS: usize = 128;
const COVERAGE: [&str; 4] = [
    "TypeScript 5.9.3 language service over only the supplied immutable source snapshots",
    "scope-aware definitions/references across explicitly selected project and declaration snapshots; complete describes only query truncation",
    "only explicit source-specific import bindings; no configuration/package discovery, plugins, default libraries, type acquisition or execution",
    "selected model is incomplete; missing/ambiguous/unused import bindings and unverified or unresolved definitions are refused",
];

#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Definitions,
    References,
}
/// Navigation admits a different, finite language set from syntax inspection.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NavigationLanguage {
    Js,
    Ts,
    Jsx,
    Tsx,
    /// An explicitly selected declaration snapshot, not another supported language.
    TsDeclaration,
}
#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceInput {
    pub path: PathBuf,
    pub language: NavigationLanguage,
    pub expected_revision: String,
}
#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImportBinding {
    pub source: usize,
    pub specifier: String,
    pub target: usize,
}
#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TypeScriptModel {
    pub imports: Vec<ImportBinding>,
}
#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NavigateRequest {
    pub sources: Vec<SourceInput>,
    pub source: usize,
    pub position_byte: usize,
    pub operation: Operation,
    pub limit: usize,
    /// Required for TypeScript-family requests, including an empty imports list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typescript_model: Option<TypeScriptModel>,
}
// No Debug: source contents must not enter diagnostics.
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LeafSource {
    pub language: NavigationLanguage,
    pub source_utf8: String,
    pub expected_revision: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub sources: Vec<LeafSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typescript_model: Option<TypeScriptModel>,
    pub source: usize,
    pub position_byte: usize,
    pub operation: Operation,
    pub limit: usize,
    pub max_response_bytes: usize,
}
impl Selection {
    fn result_budget(&self) -> usize {
        MAX_RESULT_BYTES
    }
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LeafRequest {
    pub protocol_version: u32,
    pub release_id: String,
    pub job_id: String,
    pub request: Selection,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Payload {
    pub node: PathBuf,
    pub adapter: PathBuf,
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
        .map(|bytes| bytes.len())
        .map_err(|_| "code navigation encoding failed".into())
}
fn bounded_path(path: &Path) -> bool {
    path.is_absolute()
        && path
            .to_str()
            .is_some_and(|s| s.len() <= 4096 && !s.contains('\0'))
}
fn validate_typescript_model(
    languages: &[NavigationLanguage],
    model: Option<&TypeScriptModel>,
) -> Result<(), String> {
    let model = model.ok_or("TypeScript navigation requires an explicit selected import model")?;
    let declarations = languages
        .iter()
        .filter(|language| **language == NavigationLanguage::TsDeclaration)
        .count();
    let projects = languages.len() - declarations;
    if !(1..=MAX_TYPESCRIPT_PROJECT_SOURCES).contains(&projects)
        || declarations > MAX_TYPESCRIPT_DECLARATIONS
        || model.imports.len() > MAX_TYPESCRIPT_IMPORTS
    {
        return Err("TypeScript navigation requires at most eight project sources, eight selected declarations and 128 explicit import bindings".into());
    }
    let mut unique = HashSet::new();
    for binding in &model.imports {
        if binding.source >= languages.len()
            || binding.target >= languages.len()
            || binding.specifier.is_empty()
            || binding.specifier.len() > 256
            || binding
                .specifier
                .chars()
                .any(|character| character <= '\u{1f}' || character == '\u{7f}')
            || !unique.insert((binding.source, &binding.specifier))
        {
            return Err("TypeScript import bindings must have unique source/specifier keys and bounded selected targets".into());
        }
    }
    Ok(())
}
fn typescript_model_revision(selection: &Selection) -> Result<String, String> {
    let model = selection
        .typescript_model
        .as_ref()
        .ok_or("missing TypeScript model")?;
    let sources: Vec<Value> = selection
        .sources
        .iter()
        .map(|source| json!([source.language, source.expected_revision]))
        .collect();
    let imports: Vec<Value> = model
        .imports
        .iter()
        .map(|binding| json!([binding.source, binding.specifier, binding.target]))
        .collect();
    serde_json::to_vec(&json!([sources, imports]))
        .map(|bytes| revision(&bytes))
        .map_err(|_| "TypeScript model revision encoding failed".into())
}
pub fn validate_input(request: &NavigateRequest) -> Result<(), String> {
    let max_sources = MAX_TYPESCRIPT_SOURCES;
    if !(1..=max_sources).contains(&request.sources.len())
        || request.source >= request.sources.len()
        || request.position_byte > MAX_SOURCE_BYTES
        || !(1..=100).contains(&request.limit)
        || request
            .sources
            .iter()
            .any(|s| !bounded_path(&s.path) || !crate::design::is_revision(&s.expected_revision))
    {
        return Err("code navigation requires explicit bounded sources, every observed revision and a bounded byte query".into());
    }
    validate_typescript_model(
        &request
            .sources
            .iter()
            .map(|source| source.language)
            .collect::<Vec<_>>(),
        request.typescript_model.as_ref(),
    )?;
    Ok(())
}
pub fn validate_request(request: &LeafRequest) -> Result<(), String> {
    let selection = &request.request;
    let max_sources = MAX_TYPESCRIPT_SOURCES;
    let max_total = MAX_TOTAL_BYTES;
    let max_request = MAX_REQUEST_BYTES;
    if request.protocol_version != 1
        || ReleaseId::parse(&request.release_id).is_err()
        || ReleaseId::parse(&request.job_id).is_err()
        || !(1..=max_sources).contains(&selection.sources.len())
        || selection.source >= selection.sources.len()
        || !(1..=100).contains(&selection.limit)
        || !(2048..=selection.result_budget()).contains(&selection.max_response_bytes)
        || selection.sources.iter().any(|s| {
            s.source_utf8.len() > MAX_SOURCE_BYTES
                || s.source_utf8.contains('\0')
                || revision(s.source_utf8.as_bytes()) != s.expected_revision
        })
        || selection
            .sources
            .iter()
            .map(|s| s.source_utf8.len())
            .sum::<usize>()
            > max_total
    {
        return Err("code navigation leaf identity, revisions, model or bounds are invalid".into());
    }
    validate_typescript_model(
        &selection
            .sources
            .iter()
            .map(|source| source.language)
            .collect::<Vec<_>>(),
        selection.typescript_model.as_ref(),
    )?;
    let text = &selection.sources[selection.source].source_utf8;
    if selection.position_byte >= text.len() || !text.is_char_boundary(selection.position_byte) {
        return Err("code navigation byte position must be inside a selected UTF-8 source".into());
    }
    if encoded(request)? >= max_request - 16_384 {
        return Err("code navigation sources exceed the escaped JSON transport budget".into());
    }
    Ok(())
}
pub struct Snapshot {
    scope: ProjectScope,
    paths: Vec<PathBuf>,
    pub request: LeafRequest,
}
pub enum SourceRead {
    Current(Box<Snapshot>),
    Changed(Value),
}
fn source_identities(sources: &[LeafSource]) -> Value {
    Value::Array(
        sources
            .iter()
            .enumerate()
            .map(|(source, saved)| {
                json!({"source":source,
        "revision":saved.expected_revision,"source_bytes":saved.source_utf8.len()})
            })
            .collect(),
    )
}
fn envelope(
    release: &str,
    scope: &ProjectScope,
    paths: &[PathBuf],
    selection: &Selection,
    outcome: Value,
) -> Value {
    let sources: Vec<Value> = paths.iter().zip(&selection.sources).map(|(path, source)| {
        json!({"path":path,"language":source.language,"revision":source.expected_revision,"source_bytes":source.source_utf8.len()})
    }).collect();
    json!({"release_id":release,"navigation":{"project_root":scope.root(),"sources":sources,
        "source":selection.source,"position_byte":selection.position_byte,"operation":selection.operation,"outcome":outcome}})
}
fn changed(current: Vec<Option<String>>) -> Value {
    json!({"status":"changed","current_revisions":current})
}
pub fn read_sources(
    project: &Path,
    input: NavigateRequest,
    release: String,
    job_id: String,
) -> Result<SourceRead, String> {
    validate_input(&input)?;
    if !bounded_path(project) {
        return Err("code navigation requires an explicit bounded project root".into());
    }
    let scope = ProjectScope::open(project).map_err(|e| e.to_string())?;
    let mut paths = Vec::new();
    let mut sources = Vec::new();
    let mut unique = HashSet::new();
    let mut total = 0;
    let mut stale = false;
    let max_total = MAX_TOTAL_BYTES;
    for selected in input.sources {
        let source = scope
            .read_text(&selected.path, MAX_SOURCE_BYTES)
            .map_err(|e| e.to_string())?;
        if !unique.insert(source.path.clone()) {
            return Err("code navigation sources must be distinct saved files".into());
        }
        total += source.text.len();
        if total > max_total || source.text.contains('\0') {
            return Err(
                "code navigation source model exceeds its byte bound or contains NUL".into(),
            );
        }
        let current = revision(source.text.as_bytes());
        stale |= current != selected.expected_revision;
        paths.push(source.path);
        sources.push(LeafSource {
            language: selected.language,
            source_utf8: source.text,
            expected_revision: current,
        });
    }
    let mut selection = Selection {
        sources,
        typescript_model: input.typescript_model,
        source: input.source,
        position_byte: input.position_byte,
        operation: input.operation,
        limit: input.limit,
        max_response_bytes: 2048,
    };
    let overhead = encoded(&envelope(&release, &scope, &paths, &selection, Value::Null))? - 4;
    selection.max_response_bytes = selection
        .result_budget()
        .checked_sub(overhead)
        .filter(|budget| *budget >= 2048)
        .ok_or("code navigation path metadata exceeds its complete result budget")?;
    if stale {
        let current = selection
            .sources
            .iter()
            .map(|source| Some(source.expected_revision.clone()))
            .collect();
        let response = envelope(&release, &scope, &paths, &selection, changed(current));
        if encoded(&response)? > selection.result_budget() {
            return Err("code navigation changed-source metadata exceeds its result budget".into());
        }
        return Ok(SourceRead::Changed(response));
    }
    let request = LeafRequest {
        protocol_version: 1,
        release_id: release,
        job_id,
        request: selection,
    };
    validate_request(&request)?;
    Ok(SourceRead::Current(Box::new(Snapshot {
        scope,
        paths,
        request,
    })))
}
impl Snapshot {
    /// Call only after owned worker cleanup. Every selected source is rechecked.
    pub fn finish(self, result: Value) -> Result<Value, String> {
        validate_result(&result, &self.request)?;
        let current: Vec<Option<String>> = self
            .paths
            .iter()
            .map(|path| {
                self.scope
                    .read_bytes(path, MAX_SOURCE_BYTES)
                    .ok()
                    .map(|saved| revision(&saved.bytes))
            })
            .collect();
        let same = current
            .iter()
            .zip(&self.request.request.sources)
            .all(|(current, saved)| current.as_deref() == Some(saved.expected_revision.as_str()));
        let result = if same { result } else { changed(current) };
        let response = envelope(
            &self.request.release_id,
            &self.scope,
            &self.paths,
            &self.request.request,
            result,
        );
        if encoded(&response)? > self.request.request.result_budget() {
            return Err("code navigation exceeds its complete result budget".into());
        }
        Ok(response)
    }
}
fn object<'a>(
    value: &'a Value,
    allowed: &[&str],
    required: &[&str],
) -> Result<&'a serde_json::Map<String, Value>, String> {
    let fields = value
        .as_object()
        .ok_or("code navigation result object is invalid")?;
    if fields.keys().any(|key| !allowed.contains(&key.as_str()))
        || required.iter().any(|key| !fields.contains_key(*key))
    {
        return Err("code navigation result fields violate the finite contract".into());
    }
    Ok(fields)
}
pub fn validate_result(result: &Value, request: &LeafRequest) -> Result<(), String> {
    if encoded(result)? > request.request.max_response_bytes {
        return Err("code navigation result exceeds its reserved budget".into());
    }
    let fields = object(
        result,
        &[
            "status",
            "backend",
            "sources",
            "model_complete",
            "coverage",
            "model_revision",
            "locations",
            "complete",
            "reason",
        ],
        &[
            "status",
            "backend",
            "sources",
            "model_revision",
            "model_complete",
            "coverage",
        ],
    )?;
    if fields["backend"] != json!({"name":"typescript-language-service","version":"5.9.3"})
        || fields["sources"] != source_identities(&request.request.sources)
        || fields["model_complete"] != false
        || fields["coverage"] != json!(COVERAGE)
        || fields["model_revision"] != typescript_model_revision(&request.request)?
    {
        return Err("code navigation backend, source model or incomplete coverage changed".into());
    }
    match fields["status"].as_str() {
        Some("locations") => {
            let locations = fields
                .get("locations")
                .and_then(Value::as_array)
                .filter(|a| a.len() <= request.request.limit)
                .ok_or("code navigation location count is invalid")?;
            if !fields.get("complete").is_some_and(Value::is_boolean)
                || fields.contains_key("reason")
            {
                return Err("code navigation query completion is invalid".into());
            }
            let mut previous = None;
            for item in locations {
                let item = object(
                    item,
                    &["source", "revision", "location", "is_write"],
                    &["source", "revision", "location"],
                )?;
                let index = item["source"]
                    .as_u64()
                    .and_then(|v| usize::try_from(v).ok())
                    .filter(|v| *v < request.request.sources.len())
                    .ok_or("code navigation location escaped the selected source model")?;
                let source = &request.request.sources[index];
                let range = object(
                    &item["location"],
                    &["start_byte", "end_byte"],
                    &["start_byte", "end_byte"],
                )?;
                let (Some(start), Some(end)) =
                    (range["start_byte"].as_u64(), range["end_byte"].as_u64())
                else {
                    return Err("code navigation location is not an integer range".into());
                };
                if start >= end
                    || end > source.source_utf8.len() as u64
                    || !source.source_utf8.is_char_boundary(start as usize)
                    || !source.source_utf8.is_char_boundary(end as usize)
                    || item["revision"] != source.expected_revision
                    || match request.request.operation {
                        Operation::Definitions => item.contains_key("is_write"),
                        Operation::References => {
                            !item.get("is_write").is_some_and(Value::is_boolean)
                        }
                    }
                {
                    return Err(
                        "code navigation original location or reference metadata is invalid".into(),
                    );
                }
                let key = (index, start, end);
                if previous.is_some_and(|previous| previous >= key) {
                    return Err("code navigation locations are duplicated or unordered".into());
                }
                previous = Some(key);
            }
        }
        Some("refused" | "budget_blocked") => {
            if fields.contains_key("locations")
                || fields.contains_key("complete")
                || !fields
                    .get("reason")
                    .and_then(Value::as_str)
                    .is_some_and(|r| {
                        !r.is_empty()
                            && r.len() <= 128
                            && r.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
                    })
            {
                return Err(
                    "code navigation refusal contains partial locations or an invalid reason"
                        .into(),
                );
            }
        }
        _ => return Err("code navigation status is invalid for the bound source model".into()),
    }
    Ok(())
}
pub fn validate_response(bytes: &[u8], request: &LeafRequest) -> Result<Value, String> {
    if bytes.len() > 65_536 {
        return Err("code navigation response frame exceeds its limit".into());
    }
    let response: Response =
        serde_json::from_slice(bytes).map_err(|_| "code navigation response schema is invalid")?;
    if response.protocol_version != 1
        || response.release_id != request.release_id
        || response.job_id != request.job_id
    {
        return Err("code navigation response identity differs from its request".into());
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
    if payload
        .adapter
        .file_name()
        .is_none_or(|name| name != ADAPTER_NAME)
    {
        return Err("code navigation requires its fixed private adapter".into());
    }
    [
        &payload.node,
        &payload.adapter,
        &payload.adapter.with_file_name("code-navigation.js"),
    ]
    .into_iter()
    .map(|path| {
        if !bounded_path(path) || path.canonicalize().ok().as_deref() != Some(path.as_path()) {
            return Err("code navigation requires canonical private payload paths".into());
        }
        let metadata = std::fs::symlink_metadata(path)
            .map_err(|_| "code navigation payload is unavailable")?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err("code navigation payload must be a real regular file".into());
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
        return Err("native code navigation ownership is not qualified on this platform".into());
    }
    validate_request(request)?;
    validate_job_dir(job_dir)?;
    let stamps = payload_stamps(payload)?;
    if cancel.load(Ordering::Acquire) || tokio::time::Instant::now() >= deadline {
        return Err("code navigation cancelled or expired before launch".into());
    }
    let deadline = deadline.min(tokio::time::Instant::now() + Duration::from_secs(30));
    let mut frame =
        serde_json::to_vec(request).map_err(|_| "code navigation request encoding failed")?;
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
        .map_err(|_| "private code navigation Node leaf could not launch")?;
    let mut owned = OwnedChild::new(child).map_err(unconfirmed)?;
    let (Some(mut stdin), Some(stdout), Some(stderr)) = (
        owned.child.stdin.take(),
        owned.child.stdout.take(),
        owned.child.stderr.take(),
    ) else {
        return match owned.terminate_and_reap().await {
            Ok(()) => Err("code navigation leaf pipes unavailable".into()),
            Err(error) => Err(unconfirmed(error)),
        };
    };
    let result: Result<Outcome, String> = {
        let execution = async {
            let protocol = async {
                stdin
                    .write_all(&frame)
                    .await
                    .map_err(|_| "code navigation request write failed")?;
                stdin
                    .shutdown()
                    .await
                    .map_err(|_| "code navigation request close failed")?;
                drop(stdin);
                let mut stdout = BufReader::new(stdout);
                let bytes = read_frame(&mut stdout).await?;
                let result = validate_response(&bytes, request)?;
                let mut extra = [0u8; 1];
                if stdout
                    .read(&mut extra)
                    .await
                    .map_err(|_| "code navigation stdout close failed")?
                    != 0
                {
                    return Err("code navigation leaf emitted trailing output".to_owned());
                }
                Ok(result)
            };
            let (result, stderr_bytes) = tokio::try_join!(protocol, drain_stderr(stderr))?;
            if !owned.wait().await?.success() {
                return Err("code navigation leaf exited unsuccessfully".into());
            }
            Ok(Outcome {
                result,
                stderr_bytes,
            })
        };
        tokio::select! { biased; _ = cancelled(cancel) => Err("code navigation cancelled".into()),
        result = tokio::time::timeout_at(deadline, execution) => result.map_err(|_| "code navigation deadline exceeded".to_owned()).and_then(|result| result) }
    };
    if result.is_err()
        && !owned.is_reaped()
        && let Err(error) = owned.terminate_and_reap().await
    {
        return Err(unconfirmed(format!(
            "code navigation leaf cleanup unconfirmed: {error}"
        )));
    }
    if payload_stamps(payload)? != stamps {
        return Err("code navigation payload changed during execution".into());
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
                "mirket-code-navigation-{}-{}",
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
    fn leaf(text: &str) -> LeafRequest {
        LeafRequest {
            protocol_version: 1,
            release_id: "0.0.1:code-navigation-test".into(),
            job_id: "op-navigation-test".into(),
            request: Selection {
                sources: vec![LeafSource {
                    language: NavigationLanguage::Ts,
                    source_utf8: text.into(),
                    expected_revision: revision(text.as_bytes()),
                }],
                typescript_model: Some(TypeScriptModel { imports: vec![] }),
                source: 0,
                position_byte: text.rfind("çay").unwrap(),
                operation: Operation::Definitions,
                limit: 100,
                max_response_bytes: 30_000,
            },
        }
    }
    fn result(request: &LeafRequest) -> Value {
        let source = &request.request.sources[0];
        let start = source.source_utf8.find("çay").unwrap();
        json!({"status":"locations","backend":{"name":"typescript-language-service","version":"5.9.3"},
            "sources":source_identities(&request.request.sources),"model_revision":typescript_model_revision(&request.request).unwrap(),"model_complete":false,"coverage":COVERAGE,
            "locations":[{"source":0,"revision":source.expected_revision,"location":{"start_byte":start,"end_byte":start+"çay".len()}}],"complete":true})
    }
    fn input(paths: &[(PathBuf, &str)]) -> NavigateRequest {
        NavigateRequest {
            sources: paths
                .iter()
                .map(|(path, text)| SourceInput {
                    path: path.clone(),
                    language: NavigationLanguage::Ts,
                    expected_revision: revision(text.as_bytes()),
                })
                .collect(),
            source: 0,
            position_byte: paths[0].1.rfind("çay").unwrap(),
            operation: Operation::Definitions,
            limit: 100,
            typescript_model: Some(TypeScriptModel { imports: vec![] }),
        }
    }
    #[test]
    fn navigation_response_rejects_changed_identity_fake_completeness_outside_model_and_utf8_interiors()
     {
        let request = leaf("\u{feff}// 😀\r\nconst çay = 1; çay;");
        let valid = result(&request);
        validate_request(&request).unwrap();
        validate_result(&valid, &request).unwrap();
        for mutation in 0..12 {
            let mut invalid = valid.clone();
            match mutation {
                0 => invalid["backend"]["version"] = json!("6.0.3"),
                1 => invalid["model_complete"] = json!(true),
                2 => invalid["sources"][0]["revision"] = json!(revision(b"changed")),
                3 => invalid["locations"][0]["source"] = json!(1),
                4 => invalid["locations"][0]["location"]["start_byte"] = json!(1),
                5 => invalid["locations"][0]["revision"] = json!(revision(b"changed")),
                6 => invalid["locations"]
                    .as_array_mut()
                    .unwrap()
                    .push(valid["locations"][0].clone()),
                7 => invalid["complete"] = json!("yes"),
                8 => invalid["locations"][0]["is_write"] = json!(true),
                9 => invalid["coverage"] = json!([]),
                10 => invalid["model_revision"] = json!(revision(b"wrong graph")),
                _ => invalid["project_root"] = json!("/never-read"),
            }
            assert!(
                validate_result(&invalid, &request).is_err(),
                "mutation {mutation}"
            );
        }
        let wrong_job = json!({"protocol_version":1,"release_id":request.release_id,"job_id":"wrong","result":valid});
        assert!(validate_response(&serde_json::to_vec(&wrong_job).unwrap(), &request).is_err());
    }
    #[test]
    fn navigation_refusals_cannot_leak_partial_locations() {
        let request = leaf("const çay = 1; çay;");
        let mut refused = result(&request);
        refused["status"] = json!("refused");
        refused["reason"] = json!("unresolved_import");
        assert!(validate_result(&refused, &request).is_err());
        refused.as_object_mut().unwrap().remove("locations");
        refused.as_object_mut().unwrap().remove("complete");
        validate_result(&refused, &request).unwrap();
        refused["status"] = json!("changed");
        assert!(validate_result(&refused, &request).is_err());
    }
    #[test]
    fn navigation_saved_unrelated_source_drift_discards_the_entire_result() {
        let fixture = Fixture::new();
        let first = fixture.0.join("çay source.ts");
        let second = fixture.0.join("unrelated.ts");
        let text = "\u{feff}// 😀\r\nconst çay = 1; çay;";
        let unrelated = "const unrelated = 1;";
        std::fs::write(&first, text).unwrap();
        std::fs::write(&second, unrelated).unwrap();
        let selected = input(&[(first.clone(), text), (second.clone(), unrelated)]);
        let SourceRead::Current(snapshot) = read_sources(
            &fixture.0,
            selected.clone(),
            "0.0.1:test".into(),
            "op-test".into(),
        )
        .unwrap() else {
            panic!("expected current model")
        };
        let result = result(&snapshot.request);
        std::fs::write(&second, "const unrelated = 2;").unwrap();
        let response = snapshot.finish(result).unwrap();
        assert_eq!(response["navigation"]["outcome"]["status"], "changed");
        assert!(response["navigation"]["outcome"].get("locations").is_none());
        assert!(matches!(
            read_sources(&fixture.0, selected, "0.0.1:test".into(), "op-test".into()).unwrap(),
            SourceRead::Changed(_)
        ));
    }
    #[test]
    fn navigation_validates_saved_scope_unique_paths_symlinks_and_required_revisions() {
        let fixture = Fixture::new();
        let path = fixture.0.join("source.ts");
        let text = "const çay = 1; çay;";
        std::fs::write(&path, text).unwrap();
        for paths in [
            vec![(path.clone(), text), (path.clone(), text)],
            vec![(fixture.0.join("../outside.ts"), text)],
        ] {
            assert!(
                read_sources(
                    &fixture.0,
                    input(&paths),
                    "0.0.1:test".into(),
                    "op-test".into()
                )
                .is_err()
            );
        }
        #[cfg(unix)]
        {
            let alias = fixture.0.join("alias.ts");
            std::os::unix::fs::symlink(&path, &alias).unwrap();
            assert!(
                read_sources(
                    &fixture.0,
                    input(&[(alias, text)]),
                    "0.0.1:test".into(),
                    "op-test".into()
                )
                .is_err()
            );
        }
        let mut invalid = input(&[(path, text)]);
        invalid.sources[0].expected_revision.clear();
        assert!(validate_input(&invalid).is_err());
        assert!(
            serde_json::from_value::<NavigateRequest>(
                json!({"sources":[{"path":"/project/source.ts","language":"ts"}],
            "source":0,"position_byte":0,"operation":"definitions","limit":100})
            )
            .is_err()
        );
    }
    #[test]
    fn navigation_saved_success_keeps_paths_out_of_the_leaf_and_exact_original_spans() {
        let fixture = Fixture::new();
        let path = fixture.0.join("çay source.ts");
        let text = "\u{feff}// 😀\r\nconst çay = 1; çay;";
        std::fs::write(&path, text).unwrap();
        let SourceRead::Current(snapshot) = read_sources(
            &fixture.0,
            input(&[(path.clone(), text)]),
            "0.0.1:test".into(),
            "op-test".into(),
        )
        .unwrap() else {
            panic!("expected current model")
        };
        let frame = serde_json::to_string(&snapshot.request).unwrap();
        assert!(!frame.contains(path.to_str().unwrap()));
        assert!(!frame.contains("project_root"));
        let response = snapshot.finish(result(&leaf(text))).unwrap();
        assert_eq!(response["navigation"]["sources"][0]["path"], json!(path));
        assert_eq!(response["navigation"]["outcome"]["status"], "locations");
    }
    #[test]
    fn navigation_leaf_bounds_reject_escaped_transport_overflow_and_byte_interiors() {
        let mut request = leaf("const çay = 1; çay;");
        request.request.position_byte += 1;
        assert!(validate_request(&request).is_err());
        let mut request = leaf("const çay = 1; çay;");
        request.request.sources = (0..8)
            .map(|_| LeafSource {
                language: NavigationLanguage::Ts,
                source_utf8: "\u{1}".repeat(MAX_SOURCE_BYTES / 2),
                expected_revision: revision("\u{1}".repeat(MAX_SOURCE_BYTES / 2).as_bytes()),
            })
            .collect();
        request.request.position_byte = 0;
        assert!(
            validate_request(&request)
                .unwrap_err()
                .contains("escaped JSON")
        );
    }
    #[test]
    fn typescript_model_requires_explicit_unique_bounded_selected_edges() {
        let mut request = leaf("const çay = 1; çay;");
        request.request.typescript_model = None;
        assert!(validate_request(&request).is_err());
        let mut request = leaf("const çay = 1; çay;");
        request.request.typescript_model = Some(TypeScriptModel {
            imports: vec![ImportBinding {
                source: 0,
                specifier: "prepared".into(),
                target: 1,
            }],
        });
        assert!(validate_request(&request).is_err());
        request.request.sources.push(LeafSource {
            language: NavigationLanguage::TsDeclaration,
            source_utf8: "export declare const çay: number;".into(),
            expected_revision: revision("export declare const çay: number;".as_bytes()),
        });
        validate_request(&request).unwrap();
        let model = request.request.typescript_model.as_mut().unwrap();
        model.imports.push(model.imports[0].clone());
        assert!(validate_request(&request).is_err());
        request
            .request
            .typescript_model
            .as_mut()
            .unwrap()
            .imports
            .pop();
        for specifier in [String::new(), "x".repeat(257), "invalid\0specifier".into()] {
            request.request.typescript_model.as_mut().unwrap().imports[0].specifier = specifier;
            assert!(validate_request(&request).is_err());
        }
        request
            .request
            .typescript_model
            .as_mut()
            .unwrap()
            .imports
            .clear();
        let mut invalid = serde_json::to_value(&request).unwrap();
        invalid["request"]["sources"][1]["language"] = json!("python_stub");
        assert!(serde_json::from_value::<LeafRequest>(invalid).is_err());
    }
    #[test]
    fn typescript_graph_revision_binds_declarations_and_edges_independently_of_locations() {
        let mut request = leaf("const çay = 1; çay;");
        request.request.sources.push(LeafSource {
            language: NavigationLanguage::TsDeclaration,
            source_utf8: "interface Global {}".into(),
            expected_revision: revision(b"interface Global {}"),
        });
        let original = typescript_model_revision(&request.request).unwrap();
        request
            .request
            .typescript_model
            .as_mut()
            .unwrap()
            .imports
            .push(ImportBinding {
                source: 0,
                specifier: "prepared".into(),
                target: 1,
            });
        assert_ne!(
            typescript_model_revision(&request.request).unwrap(),
            original
        );
        let with_edge = typescript_model_revision(&request.request).unwrap();
        request.request.sources[1].expected_revision = revision(b"interface Changed {}");
        assert_ne!(
            typescript_model_revision(&request.request).unwrap(),
            with_edge
        );
        let forged = result(&request);
        request
            .request
            .typescript_model
            .as_mut()
            .unwrap()
            .imports
            .clear();
        assert!(validate_result(&forged, &request).is_err());
    }
    #[test]
    fn typescript_saved_declaration_drift_suppresses_an_unrelated_success() {
        let fixture = Fixture::new();
        let path = fixture.0.join("source.ts");
        let declaration = fixture.0.join("selected.d.ts");
        let text = "const çay = 1; çay;";
        let types = "interface Selected {}";
        std::fs::write(&path, text).unwrap();
        std::fs::write(&declaration, types).unwrap();
        let mut selected = input(&[(path, text), (declaration.clone(), types)]);
        selected.sources[1].language = NavigationLanguage::TsDeclaration;
        let SourceRead::Current(snapshot) =
            read_sources(&fixture.0, selected, "0.0.1:test".into(), "op-test".into()).unwrap()
        else {
            panic!("current graph")
        };
        let valid = result(&snapshot.request);
        std::fs::write(declaration, "interface Changed {}").unwrap();
        let result = snapshot.finish(valid).unwrap();
        assert_eq!(result["navigation"]["outcome"]["status"], "changed");
        assert!(result["navigation"]["outcome"].get("locations").is_none());
    }
    fn selected_node() -> Option<PathBuf> {
        let Some(path) = std::env::var_os("MIRKET_TEST_NODE") else {
            eprintln!("SKIP native code navigation: explicit MIRKET_TEST_NODE required");
            return None;
        };
        let path = PathBuf::from(path);
        assert!(path.is_absolute());
        Some(path.canonicalize().unwrap())
    }
    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn navigation_real_private_leaf_preserves_scope_and_refuses_imports() {
        let Some(node) = selected_node() else {
            return;
        };
        let Some(adapter) = std::env::var_os("MIRKET_TEST_CODE_NAVIGATION_ADAPTER") else {
            eprintln!("SKIP explicit MIRKET_TEST_CODE_NAVIGATION_ADAPTER required");
            return;
        };
        let fixture = Fixture::new();
        let payload = Payload {
            node,
            adapter: PathBuf::from(adapter).canonicalize().unwrap(),
        };
        let request =
            leaf("\u{feff}// 😀\r\nconst çay = 1; function f(çay: number) { return çay; } çay;");
        let outcome = run_leaf(
            &payload,
            &request,
            &fixture.0,
            Arc::new(AtomicBool::new(false)),
            tokio::time::Instant::now() + Duration::from_secs(10),
        )
        .await
        .unwrap();
        assert_eq!(outcome.result["status"], "locations");
        assert_eq!(outcome.result["locations"].as_array().unwrap().len(), 1);
        let request = leaf("import { çay } from 'never-read'; çay;");
        let outcome = run_leaf(
            &payload,
            &request,
            &fixture.0,
            Arc::new(AtomicBool::new(false)),
            tokio::time::Instant::now() + Duration::from_secs(10),
        )
        .await
        .unwrap();
        assert_eq!(outcome.result["reason"], "missing_import_binding");
        let mut request = leaf("// 😀\r\nimport { çay } from 'prepared'; çay;");
        let declaration = "export declare const çay: number;\r\n";
        request.request.sources.push(LeafSource {
            language: NavigationLanguage::TsDeclaration,
            source_utf8: declaration.into(),
            expected_revision: revision(declaration.as_bytes()),
        });
        request.request.typescript_model = Some(TypeScriptModel {
            imports: vec![ImportBinding {
                source: 0,
                specifier: "prepared".into(),
                target: 1,
            }],
        });
        let outcome = run_leaf(
            &payload,
            &request,
            &fixture.0,
            Arc::new(AtomicBool::new(false)),
            tokio::time::Instant::now() + Duration::from_secs(10),
        )
        .await
        .unwrap();
        assert_eq!(outcome.result["status"], "locations");
        assert_eq!(outcome.result["locations"][0]["source"], 1);
        assert_eq!(
            outcome.result["model_revision"],
            typescript_model_revision(&request.request).unwrap()
        );
    }
    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn navigation_same_binary_worker_returns_locations_and_releases_admission() {
        let Some(node) = selected_node() else {
            return;
        };
        let (Some(binary), Some(adapter)) = (
            std::env::var_os("MIRKET_TEST_BIN"),
            std::env::var_os("MIRKET_TEST_CODE_NAVIGATION_ADAPTER"),
        ) else {
            eprintln!(
                "SKIP same-binary navigation worker: explicit MIRKET_TEST_BIN/MIRKET_TEST_CODE_NAVIGATION_ADAPTER required"
            );
            return;
        };
        let fixture = Fixture::new();
        let request = leaf("const çay = 1; function f(çay: number) { return çay; } çay;");
        let outcome = crate::worker::run_code_navigation_owned(
            &PathBuf::from(binary),
            crate::worker::CodeNavigationWork {
                state_root: fixture.0.clone(),
                payload: Payload {
                    node,
                    adapter: PathBuf::from(adapter),
                },
                request: request.clone(),
            },
            Arc::new(AtomicBool::new(false)),
        )
        .await
        .unwrap();
        validate_result(&outcome.result, &request).unwrap();
        assert_eq!(outcome.result["status"], "locations");
        let admission = crate::leases::Admission::open(&fixture.0).unwrap();
        let _first = admission.try_acquire().unwrap();
        let _second = admission.try_acquire().unwrap();
        assert!(!fixture.0.join("heavy-slot-0.unresolved").exists());
        assert!(!fixture.0.join("heavy-slot-1.unresolved").exists());
    }
    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn navigation_incomplete_oversized_and_cancelled_leaf_is_reaped() {
        let Some(node) = selected_node() else {
            return;
        };
        let fixture = Fixture::new();
        let adapter = fixture.0.join(ADAPTER_NAME);
        std::fs::write(fixture.0.join("code-navigation.js"), "// test sentinel").unwrap();
        for (index, output) in [
            "process.stdout.end('{incomplete')",
            "process.stdout.write('x'.repeat(65537))",
            "",
        ]
        .into_iter()
        .enumerate()
        {
            let pid_path = fixture.0.join(format!("pid-{index}"));
            std::fs::write(&adapter, format!("require('node:fs').writeFileSync({},String(process.pid));{output};setInterval(()=>{{}},1000);", serde_json::to_string(&pid_path).unwrap())).unwrap();
            let cancel = Arc::new(AtomicBool::new(false));
            let watcher_cancel = cancel.clone();
            let watcher = async {
                if index == 2 {
                    tokio::time::timeout(Duration::from_secs(3), async {
                        while !pid_path.exists() {
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }
                    })
                    .await
                    .unwrap();
                    watcher_cancel.store(true, Ordering::Release);
                }
            };
            let request = leaf("const çay = 1; çay;");
            let payload = Payload {
                node: node.clone(),
                adapter: adapter.clone(),
            };
            let (outcome, ()) = tokio::join!(
                run_leaf(
                    &payload,
                    &request,
                    &fixture.0,
                    cancel,
                    tokio::time::Instant::now() + Duration::from_secs(5)
                ),
                watcher
            );
            assert!(outcome.unwrap_err().cleanup_confirmed);
            let pid: i32 = std::fs::read_to_string(&pid_path).unwrap().parse().unwrap();
            assert!(pid > 0);
            // SAFETY: signal zero observes only this test's recorded child.
            assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::ESRCH)
            );
        }
    }
}
