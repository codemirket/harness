//! Pinned, offline CPU embedding computation for a fresh dedicated worker.
//!
//! No model/runtime is initialized until explicit payload verification and load.
//! The caller must give this module a private, immutable release directory and a
//! fresh process with no earlier `ort` calls. File checks detect drift; they are
//! not a sandbox against a writer racing replacement of native libraries.
//! Cancellation is cooperative between chunks; the supervisor owns hard stops.

use fastembed::{
    InitOptionsUserDefined, OutputKey, Pooling, QuantizationMode, TextEmbedding, TokenizerFiles,
    UserDefinedEmbeddingModel,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    fs::{self, File},
    io::Read,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

const PINS: &str = include_str!("../packaging/admission/m2-semantic-minilm.runtime.json");
pub const DIMENSIONS: usize = 384;
pub const MAX_INPUT_BYTES: usize = 16 * 1024;
pub const MAX_PASSAGES: usize = 32;
pub const MAX_BATCH_BYTES: usize = 256 * 1024;
pub const MAX_CHUNKS: usize = 128;
pub const MAX_INPUT_TOKENS: usize = 128;
pub const CONTENT_TOKENS: usize = 126;
pub const OVERLAP_TOKENS: usize = 16;
const QUERY_PREFIX: &str = "";
const PASSAGE_PREFIX: &str = "";
const MODEL_FILE: &str = "model/onnx/model_qint8_arm64.onnx";
const MAX_PACKAGE_ENTRIES: usize = 32;
const MAX_RECEIPT_BYTES: u64 = 64 * 1024;
static RUNTIME_CLAIMED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub enum EmbeddingError {
    Payload(String),
    UnsupportedTarget(String),
    Input(String),
    Tokenization(String),
    Runtime(String),
    InvalidVector,
    Cancelled,
    AlreadyLoaded,
}
impl EmbeddingError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Payload(_) => "embedding_payload_invalid",
            Self::UnsupportedTarget(_) => "embedding_target_unqualified",
            Self::Input(_) => "embedding_input_invalid",
            Self::Tokenization(_) => "embedding_tokenization_failed",
            Self::Runtime(_) => "embedding_runtime_failed",
            Self::InvalidVector => "embedding_vector_invalid",
            Self::Cancelled => "embedding_cancelled",
            Self::AlreadyLoaded => "embedding_worker_requires_restart",
        }
    }
}
impl fmt::Display for EmbeddingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Payload(s)
            | Self::UnsupportedTarget(s)
            | Self::Input(s)
            | Self::Tokenization(s)
            | Self::Runtime(s) => write!(f, "{}: {s}", self.code()),
            _ => f.write_str(self.code()),
        }
    }
}
impl std::error::Error for EmbeddingError {}
type Result<T> = std::result::Result<T, EmbeddingError>;
fn payload_error(e: impl fmt::Display) -> EmbeddingError {
    EmbeddingError::Payload(e.to_string())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelIdentity {
    pub id: String,
    pub model_revision: String,
    pub dimensions: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryEmbedding {
    pub model_identity: ModelIdentity,
    /// SHA-256 of raw UTF-8 query bytes before model tokenization.
    pub source_sha256: String,
    pub input_bytes: usize,
    /// Includes special tokens; no truncated input is accepted.
    pub input_tokens: usize,
    pub vector: Vec<f32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChunkEmbedding {
    pub start_byte: usize,
    pub end_byte: usize,
    /// Includes special tokens for this chunk.
    pub input_tokens: usize,
    pub vector: Vec<f32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PassageEmbedding {
    pub model_identity: ModelIdentity,
    pub input_bytes: usize,
    pub content_tokens: usize,
    pub covered_bytes: usize,
    pub complete: bool,
    pub chunks: Vec<ChunkEmbedding>,
}

#[derive(Clone, Deserialize)]
struct FilePin {
    path: String,
    bytes: u64,
    sha256: String,
}
#[derive(Deserialize)]
struct TargetPins {
    assembly_complete: bool,
    files: Vec<FilePin>,
    incomplete_reason: Option<String>,
}
#[derive(Deserialize)]
struct ModelPins {
    revision: String,
    selected: Vec<FilePin>,
    contract: serde_json::Value,
}
#[derive(Deserialize)]
struct PayloadPins {
    schema: u32,
    runtime_version: String,
    provider: String,
    telemetry: bool,
    model: ModelPins,
    targets: BTreeMap<String, TargetPins>,
}
fn compiled_pins() -> Result<PayloadPins> {
    let p: PayloadPins = serde_json::from_str(PINS).map_err(payload_error)?;
    if p.schema != 1
        || p.runtime_version != "1.31.0"
        || p.provider != "CPUExecutionProvider"
        || p.telemetry
        || p.model.contract["dimensions"] != DIMENSIONS
        || p.model.contract["max_tokens"] != MAX_INPUT_TOKENS
        || p.model.contract["pooling"] != "attention-mask mean"
        || p.model.contract["normalization"] != "L2"
        || p.model.contract["query_prefix"] != QUERY_PREFIX
        || p.model.contract["passage_prefix"] != PASSAGE_PREFIX
        || p.model.contract["precision"] != "dynamic int8"
    {
        return Err(payload_error(
            "compiled embedding contract differs from implementation",
        ));
    }
    Ok(p)
}
fn target_name() -> &'static str {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "aarch64-apple-darwin"
    } else if cfg!(all(
        target_os = "windows",
        target_arch = "aarch64",
        target_env = "msvc"
    )) {
        "aarch64-pc-windows-msvc"
    } else if cfg!(all(
        target_os = "windows",
        target_arch = "x86_64",
        target_env = "msvc"
    )) {
        "x86_64-pc-windows-msvc"
    } else {
        "unsupported"
    }
}
fn selected_files(pins: &PayloadPins, target: &str) -> Result<Vec<FilePin>> {
    let target = pins
        .targets
        .get(target)
        .ok_or_else(|| EmbeddingError::UnsupportedTarget(target.into()))?;
    if !target.assembly_complete {
        return Err(EmbeddingError::UnsupportedTarget(
            target.incomplete_reason.clone().unwrap_or_default(),
        ));
    }
    let mut files = target.files.clone();
    files.extend(pins.model.selected.iter().cloned().map(|mut f| {
        f.path = format!("model/{}", f.path);
        f
    }));
    Ok(files)
}
fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

/// Bind a derived vector to the exact raw UTF-8 source, not its length.
pub fn source_digest(text: &str) -> String {
    hex(Sha256::digest(text.as_bytes()))
}
fn identity(pins: &PayloadPins) -> Result<ModelIdentity> {
    // Notices are verified but do not change numeric vector identity.
    let artifacts: Vec<_> = pins
        .model
        .selected
        .iter()
        .filter(|file| file.path.starts_with("onnx/") || file.path.starts_with("1_Pooling/"))
        .map(|file| serde_json::json!({"path":file.path,"sha256":file.sha256}))
        .collect();
    let onnx_sha256 = pins
        .model
        .selected
        .iter()
        .find(|file| file.path == "onnx/model_qint8_arm64.onnx")
        .map(|file| file.sha256.clone())
        .ok_or_else(|| payload_error("MiniLM ONNX graph pin is missing"))?;
    let material = serde_json::json!({
        "repository": "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2",
        "model_revision": pins.model.revision,
        "onnx_sha256": onnx_sha256,
        "artifacts": artifacts,
        "contract": pins.model.contract,
        "runtime_version": pins.runtime_version,
        "provider": "CPUExecutionProvider",
        "fastembed": "7.5.0",
        "ort": "2.0.0-rc.15",
        "content_tokens": CONTENT_TOKENS,
        "overlap_tokens": OVERLAP_TOKENS,
        "single_text": true
    });
    let material_sha256 = hex(Sha256::digest(
        serde_json::to_vec(&material).map_err(payload_error)?,
    ));
    Ok(ModelIdentity {
        id: format!(
            "minilm-l12-v2:{}:{}:{}",
            pins.model.revision, onnx_sha256, material_sha256
        ),
        model_revision: pins.model.revision.clone(),
        dimensions: DIMENSIONS,
    })
}
/// Compute the expected numeric-model identity without filesystem or ORT access.
pub fn expected_model_identity() -> Result<ModelIdentity> {
    identity(&compiled_pins()?)
}

fn no_links(path: &Path) -> Result<()> {
    for p in path.ancestors() {
        let m = fs::symlink_metadata(p).map_err(payload_error)?;
        if m.file_type().is_symlink() {
            return Err(payload_error("symlinked embedding payload path"));
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if m.file_attributes() & 0x400 != 0 {
                return Err(payload_error("reparse point in embedding payload"));
            }
        }
    }
    Ok(())
}
fn regular_open(path: &Path, bytes: u64) -> Result<File> {
    no_links(path)?;
    let before = fs::symlink_metadata(path).map_err(payload_error)?;
    if !before.is_file() || before.len() != bytes {
        return Err(payload_error(
            "embedding file type/size differs from pinned source",
        ));
    }
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x00200000);
    }
    let file = options.open(path).map_err(payload_error)?;
    let after = file.metadata().map_err(payload_error)?;
    if !after.is_file() || after.len() != bytes {
        return Err(payload_error("embedding file changed while opening"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if before.dev() != after.dev() || before.ino() != after.ino() {
            return Err(payload_error("embedding file replaced while opening"));
        }
    }
    Ok(file)
}
fn verify_file(root: &Path, pin: &FilePin) -> Result<()> {
    let mut file = regular_open(&root.join(&pin.path), pin.bytes)?;
    let mut h = Sha256::new();
    let mut count = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buffer).map_err(payload_error)?;
        if n == 0 {
            break;
        }
        count += n as u64;
        if count > pin.bytes {
            return Err(payload_error("embedding file grew during verification"));
        }
        h.update(&buffer[..n]);
    }
    if count != pin.bytes || hex(h.finalize()) != pin.sha256 {
        return Err(payload_error(format!(
            "pinned source hash mismatch: {}",
            pin.path
        )));
    }
    Ok(())
}
fn verified_bytes(root: &Path, pin: &FilePin) -> Result<Vec<u8>> {
    let file = regular_open(&root.join(&pin.path), pin.bytes)?;
    let mut bytes = Vec::with_capacity(pin.bytes as usize);
    file.take(pin.bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(payload_error)?;
    if bytes.len() as u64 != pin.bytes || hex(Sha256::digest(&bytes)) != pin.sha256 {
        return Err(payload_error("embedding input changed after verification"));
    }
    Ok(bytes)
}
fn verify_tree(root: &Path, pins: &[FilePin]) -> Result<()> {
    if !root.is_absolute()
        || root
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
    {
        return Err(payload_error(
            "explicit absolute normalized payload root required",
        ));
    }
    no_links(root)?;
    if !root.is_dir() {
        return Err(payload_error("embedding root is not a directory"));
    }
    let mut expected = BTreeSet::from([PathBuf::from("package-receipt.json")]);
    let mut dirs = BTreeSet::new();
    for p in pins {
        let path = Path::new(&p.path);
        if path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
            || p.path.contains('\\')
            || p.path.contains(':')
            || p.bytes > 128 * 1024 * 1024
            || !expected.insert(path.to_owned())
        {
            return Err(payload_error("invalid compiled embedding file pin"));
        }
        for parent in path
            .ancestors()
            .skip(1)
            .filter(|p| !p.as_os_str().is_empty())
        {
            dirs.insert(parent.to_owned());
        }
    }
    let mut todo = vec![root.to_owned()];
    let mut actual = BTreeSet::new();
    let mut actual_dirs = BTreeSet::new();
    let mut count = 0;
    while let Some(dir) = todo.pop() {
        for entry in fs::read_dir(dir).map_err(payload_error)? {
            let entry = entry.map_err(payload_error)?;
            count += 1;
            if count > MAX_PACKAGE_ENTRIES {
                return Err(payload_error("embedding package entry limit exceeded"));
            }
            let path = entry.path();
            no_links(&path)?;
            let kind = entry.file_type().map_err(payload_error)?;
            let relative = path.strip_prefix(root).map_err(payload_error)?.to_owned();
            if kind.is_dir() {
                actual_dirs.insert(relative);
                todo.push(path);
            } else if kind.is_file() {
                actual.insert(relative);
            } else {
                return Err(payload_error("nonregular embedding payload member"));
            }
        }
    }
    if actual != expected || actual_dirs != dirs {
        return Err(payload_error(
            "missing, extra or competing embedding payload files",
        ));
    }
    for pin in pins {
        verify_file(root, pin)?;
    }
    Ok(())
}

/// Verified source identities; fields cannot be supplied by an untrusted receipt.
pub struct VerifiedEmbeddingPayload {
    root: PathBuf,
    files: Vec<FilePin>,
    identity: ModelIdentity,
}
impl VerifiedEmbeddingPayload {
    pub fn open(root: &Path) -> Result<Self> {
        let pins = compiled_pins()?;
        let target = target_name();
        let files = selected_files(&pins, target)?;
        verify_tree(root, &files)?;
        verify_receipt(root, target, &pins, &files)?;
        Ok(Self {
            root: root.to_owned(),
            files,
            identity: identity(&pins)?,
        })
    }
    pub fn model_identity(&self) -> &ModelIdentity {
        &self.identity
    }
    fn read(&self, name: &str) -> Result<Vec<u8>> {
        let pin = self
            .files
            .iter()
            .find(|f| f.path == name)
            .ok_or_else(|| payload_error("required model file not admitted"))?;
        verified_bytes(&self.root, pin)
    }
}

/// Check the small source-pin receipt before a release manifest can be signed
/// or accepted. The release verifier separately hashes every large model file.
pub(crate) fn verify_package_receipt(root: &Path, target: &str) -> Result<()> {
    let pins = compiled_pins()?;
    let files = selected_files(&pins, target)?;
    verify_receipt(root, target, &pins, &files)
}

fn verify_receipt(root: &Path, target: &str, pins: &PayloadPins, files: &[FilePin]) -> Result<()> {
    let receipt = root.join("package-receipt.json");
    let size = fs::symlink_metadata(&receipt).map_err(payload_error)?.len();
    if size > MAX_RECEIPT_BYTES {
        return Err(payload_error("oversized embedding package receipt"));
    }
    let mut bytes = Vec::new();
    regular_open(&receipt, size)?
        .take(MAX_RECEIPT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(payload_error)?;
    let actual: serde_json::Value = serde_json::from_slice(&bytes).map_err(payload_error)?;
    let expected = serde_json::json!({"schema":1,"status":"private-runtime","target":target,
        "manifest_sha256":hex(Sha256::digest(PINS.as_bytes())),"runtime_version":pins.runtime_version,
        "model_revision":pins.model.revision,"provider":"CPUExecutionProvider","telemetry":false,
        "files":files.iter().map(|f|serde_json::json!({"path":f.path,"bytes":f.bytes,"sha256":f.sha256})).collect::<Vec<_>>()});
    if actual != expected {
        return Err(payload_error(
            "package receipt differs from compiled source pins",
        ));
    }
    Ok(())
}

pub struct EmbeddingModel {
    inner: TextEmbedding,
    identity: ModelIdentity,
}
fn check_cancel(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(EmbeddingError::Cancelled)
    } else {
        Ok(())
    }
}
fn validate_text(text: &str) -> Result<()> {
    if text.len() > MAX_INPUT_BYTES || text.trim().is_empty() {
        return Err(EmbeddingError::Input(
            "text must be nonblank and at most 16KiB UTF-8".into(),
        ));
    }
    Ok(())
}
fn validate_batch(texts: &[&str]) -> Result<()> {
    if texts.is_empty()
        || texts.len() > MAX_PASSAGES
        || texts
            .iter()
            .try_fold(0usize, |n, s| n.checked_add(s.len()))
            .is_none_or(|n| n > MAX_BATCH_BYTES)
    {
        return Err(EmbeddingError::Input(
            "passage batch count/byte limit exceeded".into(),
        ));
    }
    for text in texts {
        validate_text(text)?;
    }
    Ok(())
}
fn checked_token_count(count: usize) -> Result<usize> {
    if count > MAX_INPUT_TOKENS {
        return Err(EmbeddingError::Input(
            "input exceeds 128 tokens; truncation refused".into(),
        ));
    }
    Ok(count)
}
fn validate_vector(v: &[f32]) -> Result<()> {
    if v.len() != DIMENSIONS || v.iter().any(|x| !x.is_finite()) {
        return Err(EmbeddingError::InvalidVector);
    }
    let norm = v.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>().sqrt();
    if (norm - 1.0).abs() > 1e-3 {
        return Err(EmbeddingError::InvalidVector);
    }
    Ok(())
}
fn floor_boundary(text: &str, mut p: usize) -> usize {
    while !text.is_char_boundary(p) {
        p -= 1;
    }
    p
}
fn ceil_boundary(text: &str, mut p: usize) -> usize {
    while !text.is_char_boundary(p) {
        p += 1;
    }
    p
}
fn source_spans(text: &str, offsets: &[(usize, usize)]) -> Result<Vec<(usize, usize)>> {
    if offsets.is_empty()
        || offsets.iter().any(|&(a, b)| a > b || b > text.len())
        || offsets.windows(2).any(|p| p[0].0 > p[1].0)
    {
        return Err(EmbeddingError::Tokenization(
            "invalid or empty source token offsets".into(),
        ));
    }
    let mut spans = Vec::new();
    let mut i = 0;
    while i < offsets.len() {
        let j = (i + CONTENT_TOKENS).min(offsets.len());
        let start = if i == 0 {
            0
        } else {
            floor_boundary(text, offsets[i].0)
        };
        let end = if j == offsets.len() {
            text.len()
        } else {
            ceil_boundary(
                text,
                offsets[i..j]
                    .iter()
                    .map(|x| x.1)
                    .max()
                    .unwrap()
                    .max(offsets[j].0),
            )
        };
        if end <= start {
            return Err(EmbeddingError::Tokenization(
                "nonprogressing source token span".into(),
            ));
        }
        spans.push((start, end));
        if spans.len() > MAX_CHUNKS {
            return Err(EmbeddingError::Input("passage chunk limit exceeded".into()));
        }
        if j == offsets.len() {
            break;
        }
        i += CONTENT_TOKENS - OVERLAP_TOKENS;
    }
    validate_coverage(text, &spans)?;
    Ok(spans)
}
fn validate_coverage(text: &str, spans: &[(usize, usize)]) -> Result<()> {
    let mut covered = 0;
    for &(start, end) in spans {
        if start > covered
            || end <= start
            || end > text.len()
            || !text.is_char_boundary(start)
            || !text.is_char_boundary(end)
        {
            return Err(EmbeddingError::Tokenization(
                "incomplete or invalid UTF-8 source coverage".into(),
            ));
        }
        covered = covered.max(end);
    }
    if covered != text.len() {
        return Err(EmbeddingError::Tokenization(
            "passage tail is not covered".into(),
        ));
    }
    Ok(())
}
impl EmbeddingModel {
    /// Only call in a fresh dedicated worker; native initialization is not cancellable.
    pub fn load(payload: VerifiedEmbeddingPayload) -> Result<Self> {
        validate_telemetry_setting(std::env::var_os("ORT_DISABLE_TELEMETRY").as_deref())?;
        verify_tree(&payload.root, &payload.files)?;
        let tokenizer_files = TokenizerFiles {
            tokenizer_file: payload.read("model/onnx/tokenizer.json")?,
            config_file: payload.read("model/onnx/config.json")?,
            special_tokens_map_file: payload.read("model/onnx/special_tokens_map.json")?,
            tokenizer_config_file: payload.read("model/onnx/tokenizer_config.json")?,
        };
        let model = UserDefinedEmbeddingModel::new(payload.read(MODEL_FILE)?, tokenizer_files)
            .with_pooling(Pooling::Mean)
            .with_quantization(QuantizationMode::Dynamic);
        let runtime = payload
            .files
            .iter()
            .find(|f| f.path.ends_with(".dylib") || f.path.ends_with(".dll"))
            .ok_or_else(|| payload_error("private runtime library is missing"))?;
        verify_file(&payload.root, runtime)?;
        if RUNTIME_CLAIMED.swap(true, Ordering::AcqRel) {
            return Err(EmbeddingError::AlreadyLoaded);
        }
        let env = ort::init_from(payload.root.join(&runtime.path))
            .map_err(|e| EmbeddingError::Runtime(e.to_string()))?
            .with_name("mirket-private-embedding")
            .with_telemetry(false)
            .with_execution_providers([ort::ep::CPU::default().build().error_on_failure()])
            .build()
            .map_err(|e| EmbeddingError::Runtime(e.to_string()))?;
        if env
            .execution_providers()
            .iter()
            .map(|p| p.name())
            .collect::<Vec<_>>()
            != ["CPUExecutionProvider"]
        {
            return Err(EmbeddingError::Runtime(
                "unexpected execution provider".into(),
            ));
        }
        let mut model = model;
        model.output_key = Some(OutputKey::ByName("last_hidden_state"));
        let inner = TextEmbedding::try_new_from_user_defined(
            model,
            InitOptionsUserDefined::new()
                .with_environment(env)
                .with_execution_providers(vec![ort::ep::CPU::default().build().error_on_failure()])
                .with_intra_threads(2)
                .with_max_length(MAX_INPUT_TOKENS),
        )
        .map_err(|e| EmbeddingError::Runtime(e.to_string()))?;
        Ok(Self {
            inner,
            identity: payload.identity,
        })
    }
    pub fn model_identity(&self) -> &ModelIdentity {
        &self.identity
    }
    fn checked_tokens(&self, text: &str) -> Result<usize> {
        let mut tokenizer = self.inner.tokenizer.clone();
        tokenizer
            .with_truncation(None)
            .map_err(|_| EmbeddingError::Tokenization("tokenizer operation failed".into()))?;
        tokenizer.with_padding(None);
        let encoding = tokenizer
            .encode(text, true)
            .map_err(|_| EmbeddingError::Tokenization("tokenizer operation failed".into()))?;
        let count = encoding.get_ids().len();
        checked_token_count(count)
    }
    fn embed_one(&mut self, text: &str, cancel: &AtomicBool) -> Result<Vec<f32>> {
        check_cancel(cancel)?;
        let mut values = self
            .inner
            .embed([text], Some(1))
            .map_err(|e| EmbeddingError::Runtime(e.to_string()))?;
        check_cancel(cancel)?;
        if values.len() != 1 {
            return Err(EmbeddingError::InvalidVector);
        }
        let v = values.remove(0);
        validate_vector(&v)?;
        Ok(v)
    }
    pub fn embed_query(&mut self, text: &str, cancel: &AtomicBool) -> Result<QueryEmbedding> {
        check_cancel(cancel)?;
        validate_text(text)?;
        let input = format!("{QUERY_PREFIX}{text}");
        let input_tokens = self.checked_tokens(&input)?;
        let vector = self.embed_one(&input, cancel)?;
        Ok(QueryEmbedding {
            model_identity: self.identity.clone(),
            source_sha256: source_digest(text),
            input_bytes: text.len(),
            input_tokens,
            vector,
        })
    }
    pub fn embed_passages(
        &mut self,
        texts: &[&str],
        cancel: &AtomicBool,
    ) -> Result<Vec<PassageEmbedding>> {
        check_cancel(cancel)?;
        validate_batch(texts)?;
        // Prepare every span/count first: invalid late inputs produce no native inference.
        let mut plans = Vec::new();
        let mut total_chunks = 0;
        for text in texts {
            check_cancel(cancel)?;
            validate_text(text)?;
            let mut tokenizer = self.inner.tokenizer.clone();
            tokenizer
                .with_truncation(None)
                .map_err(|_| EmbeddingError::Tokenization("tokenizer operation failed".into()))?;
            tokenizer.with_padding(None);
            let encoding = tokenizer
                .encode(*text, false)
                .map_err(|_| EmbeddingError::Tokenization("tokenizer operation failed".into()))?;
            let spans = source_spans(text, encoding.get_offsets())?;
            total_chunks += spans.len();
            if total_chunks > MAX_CHUNKS {
                return Err(EmbeddingError::Input(
                    "request exceeds 128 passage chunks".into(),
                ));
            }
            let mut chunks = Vec::new();
            for (start, end) in spans {
                check_cancel(cancel)?;
                let n = self.checked_tokens(&format!("{PASSAGE_PREFIX}{}", &text[start..end]))?;
                chunks.push((start, end, n));
            }
            plans.push((encoding.get_ids().len(), chunks));
        }
        let mut result = Vec::new();
        for (text, (content_tokens, plan)) in texts.iter().zip(plans) {
            let mut chunks = Vec::new();
            for (start_byte, end_byte, input_tokens) in plan {
                let vector = self.embed_one(
                    &format!("{PASSAGE_PREFIX}{}", &text[start_byte..end_byte]),
                    cancel,
                )?;
                chunks.push(ChunkEmbedding {
                    start_byte,
                    end_byte,
                    input_tokens,
                    vector,
                });
            }
            result.push(PassageEmbedding {
                model_identity: self.identity.clone(),
                input_bytes: text.len(),
                content_tokens,
                covered_bytes: text.len(),
                complete: true,
                chunks,
            });
        }
        Ok(result)
    }
}

fn validate_telemetry_setting(value: Option<&std::ffi::OsStr>) -> Result<()> {
    if value != Some(std::ffi::OsStr::new("1")) {
        return Err(EmbeddingError::Runtime(
            "fresh embedding process requires ORT_DISABLE_TELEMETRY=1 before native initialization"
                .into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_must_be_disabled_before_native_initialization() {
        use std::ffi::OsStr;
        assert!(validate_telemetry_setting(None).is_err());
        for value in ["", "0", "true", "01", " 1", "1 "] {
            assert!(validate_telemetry_setting(Some(OsStr::new(value))).is_err());
        }
        validate_telemetry_setting(Some(OsStr::new("1"))).unwrap();
    }
    use std::sync::atomic::AtomicU64;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "mirket-embedding-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&p).unwrap();
            Self(p.canonicalize().unwrap())
        }
        fn file(&self, path: &str, bytes: &[u8]) -> FilePin {
            let p = self.0.join(path);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, bytes).unwrap();
            FilePin {
                path: path.into(),
                bytes: bytes.len() as u64,
                sha256: hex(Sha256::digest(bytes)),
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn offsets(text: &str) -> Vec<(usize, usize)> {
        text.char_indices()
            .filter(|(_, c)| !c.is_whitespace())
            .map(|(i, c)| (i, i + c.len_utf8()))
            .collect()
    }

    #[test]
    fn embedded_pins_and_identity_bind_numeric_contract() {
        let pins = compiled_pins().unwrap();
        let a = identity(&pins).unwrap();
        assert_eq!(a.dimensions, 384);
        assert_eq!(a, expected_model_identity().unwrap());
        assert_eq!(a.id, identity(&compiled_pins().unwrap()).unwrap().id);
        let graph = pins
            .model
            .selected
            .iter()
            .find(|file| file.path == "onnx/model_qint8_arm64.onnx")
            .unwrap();
        assert!(a.id.starts_with("minilm-l12-v2:e8f8c211226b894fcb81acc59f3b34ba3efd5f42:"));
        assert!(a.id.contains(&graph.sha256));
        assert!(selected_files(&pins, "x86_64-pc-windows-msvc").is_err());
        assert!(selected_files(&pins, "aarch64-pc-windows-msvc").is_err());
        assert!(selected_files(&pins, "unsupported").is_err());
        let mut changed = compiled_pins().unwrap();
        changed.model.selected[0].sha256 = "f".repeat(64);
        assert_ne!(a.id, identity(&changed).unwrap().id);
        changed = compiled_pins().unwrap();
        changed.model.contract["query_prefix"] = serde_json::json!("different: ");
        assert_ne!(a.id, identity(&changed).unwrap().id);
    }
    #[test]
    fn release_receipt_rejects_a_stale_semantic_manifest() {
        let f = Fixture::new();
        let pins = compiled_pins().unwrap();
        let target = "aarch64-apple-darwin";
        let files = selected_files(&pins, target).unwrap();
        let mut receipt = serde_json::json!({"schema":1,"status":"private-runtime","target":target,
            "manifest_sha256":hex(Sha256::digest(PINS.as_bytes())),"runtime_version":pins.runtime_version,
            "model_revision":pins.model.revision,"provider":"CPUExecutionProvider","telemetry":false,
            "files":files.iter().map(|file|serde_json::json!({"path":file.path,"bytes":file.bytes,"sha256":file.sha256})).collect::<Vec<_>>()});
        fs::write(
            f.0.join("package-receipt.json"),
            serde_json::to_vec(&receipt).unwrap(),
        )
        .unwrap();
        verify_package_receipt(&f.0, target).unwrap();
        receipt["manifest_sha256"] =
            serde_json::json!("13e196052d838b226fe3b25069cc4cac8176849ac7d70af341edf33170a8d989");
        fs::write(
            f.0.join("package-receipt.json"),
            serde_json::to_vec(&receipt).unwrap(),
        )
        .unwrap();
        assert!(verify_package_receipt(&f.0, target).is_err());
    }
    #[test]
    fn serialized_identity_rejects_unknown_metadata() {
        let identity = expected_model_identity().unwrap();
        let mut encoded = serde_json::to_value(&identity).unwrap();
        encoded["document_text"] = serde_json::json!("unlisted");
        assert!(serde_json::from_value::<ModelIdentity>(encoded).is_err());
    }

    #[test]
    fn payload_hashes_are_independent_of_self_authored_receipt() {
        let f = Fixture::new();
        let pin = f.file("model/file", b"expected");
        f.file("package-receipt.json", b"{}");
        verify_tree(&f.0, std::slice::from_ref(&pin)).unwrap();
        fs::write(f.0.join("model/file"), b"tampered").unwrap();
        fs::write(f.0.join("package-receipt.json"), b"{\"sha256\":\"forged\"}").unwrap();
        assert!(
            verify_tree(&f.0, &[pin])
                .unwrap_err()
                .to_string()
                .contains("hash mismatch")
        );
    }
    #[test]
    fn payload_rejects_extra_missing_paths_and_tree_explosion() {
        let f = Fixture::new();
        let pin = f.file("model/file", b"expected");
        f.file("package-receipt.json", b"{}");
        let extra = f.file("runtime/extra.dll", b"unlisted");
        assert!(verify_tree(&f.0, std::slice::from_ref(&pin)).is_err());
        fs::remove_file(f.0.join(extra.path)).unwrap();
        fs::remove_dir(f.0.join("runtime")).unwrap();
        fs::remove_file(f.0.join("model/file")).unwrap();
        assert!(verify_tree(&f.0, std::slice::from_ref(&pin)).is_err());
        for i in 0..MAX_PACKAGE_ENTRIES {
            f.file(&format!("extra{i}"), b"x");
        }
        assert!(
            verify_tree(&f.0, &[pin])
                .unwrap_err()
                .to_string()
                .contains("entry limit")
        );
    }
    #[cfg(unix)]
    #[test]
    fn payload_rejects_symlinked_member_and_root() {
        use std::os::unix::fs::symlink;
        let f = Fixture::new();
        let pin = f.file("model/file", b"expected");
        f.file("package-receipt.json", b"{}");
        fs::rename(f.0.join("model/file"), f.0.join("original")).unwrap();
        symlink(f.0.join("original"), f.0.join("model/file")).unwrap();
        assert!(verify_tree(&f.0, std::slice::from_ref(&pin)).is_err());
        let second = Fixture::new();
        symlink(&f.0, second.0.join("link")).unwrap();
        assert!(verify_tree(&second.0.join("link"), &[pin]).is_err());
    }
    #[test]
    fn bounds_cancel_and_vector_validation_are_explicit() {
        assert!(validate_text("").is_err());
        assert!(validate_text(" \n").is_err());
        assert!(validate_text(&"x".repeat(MAX_INPUT_BYTES + 1)).is_err());
        validate_text(&"x".repeat(MAX_INPUT_BYTES)).unwrap();
        assert!(validate_batch(&[]).is_err());
        assert!(validate_batch(&["x"; MAX_PASSAGES + 1]).is_err());
        let full = "x".repeat(MAX_INPUT_BYTES);
        assert!(validate_batch(&vec![full.as_str(); 17]).is_err());
        assert_eq!(
            checked_token_count(MAX_INPUT_TOKENS).unwrap(),
            MAX_INPUT_TOKENS
        );
        assert!(checked_token_count(MAX_INPUT_TOKENS + 1).is_err());
        assert!(matches!(
            check_cancel(&AtomicBool::new(true)),
            Err(EmbeddingError::Cancelled)
        ));
        let mut v = vec![0.; DIMENSIONS];
        v[0] = 1.;
        validate_vector(&v).unwrap();
        v[0] = f32::NAN;
        assert!(validate_vector(&v).is_err());
        v[0] = 0.;
        assert!(validate_vector(&v).is_err());
        assert!(validate_vector(&[1.]).is_err());
    }
    #[test]
    fn source_chunks_cover_turkish_cjk_combining_marks_and_tail() {
        let text = format!(
            "   {}   TAIL_末尾_İstanbul",
            "İstanbul 你好 e\u{301}ışık ".repeat(120)
        );
        let spans = source_spans(&text, &offsets(&text)).unwrap();
        assert!(spans.len() > 1);
        validate_coverage(&text, &spans).unwrap();
        assert_eq!(spans[0].0, 0);
        assert_eq!(spans.last().unwrap().1, text.len());
        assert!(spans.windows(2).all(|p| p[1].0 < p[0].1));
        let tail = spans.last().unwrap();
        assert!(text[tail.0..tail.1].contains("TAIL_末尾_İstanbul"));
        assert!(
            spans
                .iter()
                .all(|&(a, b)| text.is_char_boundary(a) && text.is_char_boundary(b))
        );
    }
    #[test]
    fn source_spans_preserve_whitespace_and_expand_partial_utf8_offsets() {
        let text = format!("{}{}{}", "a".repeat(400), " ".repeat(600), "猫".repeat(400));
        validate_coverage(&text, &source_spans(&text, &offsets(&text)).unwrap()).unwrap();
        let repeated = "猫".repeat(500);
        let partial = (0..500).map(|i| (i * 3 + 1, i * 3 + 2)).collect::<Vec<_>>();
        let expanded = source_spans(&repeated, &partial).unwrap();
        assert!(expanded.len() > 1);
        validate_coverage(&repeated, &expanded).unwrap();
        let text = "Aİ猫e\u{301}";
        let spans = source_spans(text, &[(0, 1), (2, 3), (4, 6), (6, 7), (8, 9)]).unwrap();
        assert_eq!(spans, vec![(0, text.len())]);
        assert!(source_spans("abc", &[(0, 4)]).is_err());
        assert!(source_spans("abc", &[]).is_err());
        assert!(validate_coverage("abcdef", &[(0, 2), (3, 6)]).is_err());
    }
    #[test]
    fn query_source_digest_is_raw_utf8_and_survives_serialization() {
        assert_eq!(
            source_digest("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_ne!(source_digest("abc"), source_digest("abd"));
        assert_ne!(source_digest("abc"), source_digest("query: abc"));
        let query = QueryEmbedding {
            model_identity: expected_model_identity().unwrap(),
            source_sha256: source_digest("İ猫"),
            input_bytes: "İ猫".len(),
            input_tokens: 5,
            vector: vec![0.0; DIMENSIONS],
        };
        let value = serde_json::to_value(&query).unwrap();
        let decoded: QueryEmbedding = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(decoded.source_sha256, source_digest("İ猫"));
        let mut missing = value;
        missing.as_object_mut().unwrap().remove("source_sha256");
        assert!(serde_json::from_value::<QueryEmbedding>(missing).is_err());
    }
    #[test]
    #[ignore = "explicit verified payload and heavy-memory admission required"]
    fn actual_private_package() {
        let root = PathBuf::from(
            std::env::var_os("MIRKET_TEST_EMBEDDING_PACKAGE")
                .expect("explicit MIRKET_TEST_EMBEDDING_PACKAGE required"),
        );
        let payload = VerifiedEmbeddingPayload::open(&root).unwrap();
        let identity = payload.model_identity().clone();
        let mut model = EmbeddingModel::load(payload).unwrap();
        let cancel = AtomicBool::new(false);
        let first = model
            .embed_query("Türkçe ve English: güncelleme nasıl geri alınır?", &cancel)
            .unwrap();
        let again = model
            .embed_query("Türkçe ve English: güncelleme nasıl geri alınır?", &cancel)
            .unwrap();
        assert_eq!(first.model_identity, identity);
        assert_eq!(
            first.source_sha256,
            source_digest("Türkçe ve English: güncelleme nasıl geri alınır?")
        );
        assert!(
            first
                .vector
                .iter()
                .zip(again.vector)
                .all(|(a, b)| (a - b).abs() < 1e-6)
        );
        validate_vector(&first.vector).unwrap();
        let text = "İstanbul'da güvenli güncelleme ve rollback açıklaması.";
        let passage = model.embed_passages(&[text], &cancel).unwrap().remove(0);
        let direct = model
            .embed_one(&format!("{PASSAGE_PREFIX}{text}"), &cancel)
            .unwrap();
        assert_eq!(passage.chunks.len(), 1);
        assert_eq!(passage.chunks[0].vector, direct);
        let long = format!(
            "{}TAIL_末尾_SCOPE_REVISION_İstanbul",
            "Türkçe memory 你好 güvenli güncelleme. ".repeat(120)
        );
        assert!(long.len() <= MAX_INPUT_BYTES);
        assert!(model.embed_query(&long, &cancel).is_err());
        let embedded = model.embed_passages(&[&long], &cancel).unwrap().remove(0);
        assert!(embedded.chunks.len() > 1);
        assert!(embedded.complete);
        assert_eq!(embedded.covered_bytes, long.len());
        let spans = embedded
            .chunks
            .iter()
            .map(|c| (c.start_byte, c.end_byte))
            .collect::<Vec<_>>();
        validate_coverage(&long, &spans).unwrap();
        assert!(
            embedded
                .chunks
                .iter()
                .all(|c| c.input_tokens <= MAX_INPUT_TOKENS)
        );
        let last = embedded.chunks.last().unwrap();
        assert!(long[last.start_byte..last.end_byte].contains("TAIL_末尾_SCOPE_REVISION_İstanbul"));
        for c in embedded.chunks {
            validate_vector(&c.vector).unwrap();
        }
        assert!(matches!(
            model.embed_query("cancelled", &AtomicBool::new(true)),
            Err(EmbeddingError::Cancelled)
        ));
    }
}
