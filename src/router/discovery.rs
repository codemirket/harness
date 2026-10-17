use super::policy::{Capabilities, Model, Tier, classify};
use crate::{paths::Paths, util};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const REGISTRY_VERSION: u32 = 2;
const MAX_METADATA: usize = 4 * 1024 * 1024;
static DISCOVERY_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Codex,
    Claude,
    OpenaiChat,
    OpenaiResponses,
    Anthropic,
    Ollama,
}

/// Targets, never model identities or caller-provided capability assertions.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub name: String,
    pub protocol: Protocol,
    pub endpoint: Option<String>,
    pub credential_env: Option<String>,
    pub executable: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "ttl")]
    pub ttl_seconds: u64,
    #[serde(default)]
    pub targets: Vec<Target>,
}
fn ttl() -> u64 {
    300
}
impl Default for Config {
    fn default() -> Self {
        Self {
            ttl_seconds: ttl(),
            targets: vec![],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TargetStatus {
    pub target: Target,
    pub authenticated: bool,
    pub execution_ready: bool,
    pub diagnostic: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Registry {
    pub version: u32,
    pub refreshed_at: u64,
    pub fingerprint: String,
    pub targets: Vec<TargetStatus>,
    pub models: Vec<Model>,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn executable_identity(path: &Path) -> Result<Value> {
    let mut file = File::open(path).context("read inference executable identity")?;
    let metadata = file.metadata().context("inspect inference executable")?;
    ensure!(
        metadata.is_file(),
        "inference executable is not a regular file"
    );
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos().min(u64::MAX as u128) as u64);
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .context("hash inference executable")?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode()
    };
    #[cfg(not(unix))]
    let mode = 0u32;
    Ok(
        json!({"bytes":metadata.len(),"modified_ns":modified,"mode":mode,"sha256":format!("{:x}",digest.finalize())}),
    )
}

fn cache_fingerprint_with_env<F>(
    paths: &Paths,
    targets: &[Target],
    mut credential: F,
) -> Result<String>
where
    F: FnMut(&str) -> Option<OsString>,
{
    let mut credentials = Vec::with_capacity(targets.len());
    let mut executables = Vec::with_capacity(targets.len());
    for target in targets {
        let credential_digest = target.credential_env.as_ref().map(|name| {
            credential(name).map(|value| {
                let mut input = b"mirket-routing-credential-v1\0".to_vec();
                input.extend_from_slice(name.as_bytes());
                input.push(0);
                input.extend_from_slice(value.as_encoded_bytes());
                util::sha256(&input)
            })
        });
        credentials.push(json!({"target":target.name,"credential_digest":credential_digest}));
        if let Some(path) = &target.executable {
            let identity = executable_identity(path).unwrap_or_else(|_| json!({"unreadable":true}));
            executables.push(json!({"target":target.name,"identity":identity}));
        }
    }
    Ok(util::sha256(&serde_json::to_vec(&json!({
        "targets":targets,
        "home":paths.home,
        "path":std::env::var_os("PATH"),
        "version":env!("CARGO_PKG_VERSION"),
        "credentials":credentials,
        "executables":executables,
    }))?))
}

fn cache_fingerprint(paths: &Paths, targets: &[Target]) -> Result<String> {
    cache_fingerprint_with_env(paths, targets, |name| std::env::var_os(name))
}
pub fn config(paths: &Paths) -> Result<Config> {
    let path = paths.root.join("routing.json");
    let config: Config = if path.exists() {
        serde_json::from_slice(&util::read_bounded(&path, 128 * 1024)?)
            .context("invalid routing.json: configure targets, not model IDs")?
    } else {
        Config::default()
    };
    ensure!(
        (1..=86400).contains(&config.ttl_seconds),
        "routing TTL must be 1..=86400 seconds"
    );
    ensure!(config.targets.len() <= 32, "at most 32 inference targets");
    Ok(config)
}

fn on_path(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|p| p.join(name))
        .find(|p| p.is_file())
        .and_then(|p| p.canonicalize().ok())
}
fn endpoint_target(name: &str, protocol: Protocol, endpoint: String, key: Option<&str>) -> Target {
    Target {
        name: name.into(),
        protocol,
        endpoint: Some(endpoint),
        credential_env: key.map(str::to_owned),
        executable: None,
    }
}
fn targets(paths: &Paths, config: &Config) -> Result<Vec<Target>> {
    let mut targets = config.targets.clone();
    for (name, protocol) in [("codex", Protocol::Codex), ("claude", Protocol::Claude)] {
        if !targets.iter().any(|t| t.protocol == protocol)
            && let Some(executable) = on_path(name)
        {
            targets.push(Target {
                name: name.into(),
                protocol,
                executable: Some(executable),
                endpoint: None,
                credential_env: None,
            });
        }
    }
    // Desktop and PATH CLIs can have different protocol versions and menus.
    // Probe each installed bundle, rather than inheriting the PATH menu as a
    // substitute. An explicit native target remains the user's override.
    if cfg!(target_os = "macos") && !config.targets.iter().any(|t| t.protocol == Protocol::Codex) {
        let roots = [
            PathBuf::from("/Applications"),
            paths.home.join("Applications"),
        ];
        for root in roots {
            for bundle in ["Codex.app", "ChatGPT.app"] {
                for relative in [
                    "Contents/Resources/codex-cli/bin/codex",
                    "Contents/Resources/codex",
                ] {
                    let candidate = root.join(bundle).join(relative);
                    if !candidate.is_file() {
                        continue;
                    }
                    let executable = candidate.canonicalize()?;
                    if targets
                        .iter()
                        .any(|t| t.executable.as_ref() == Some(&executable))
                    {
                        continue;
                    }
                    let name = format!(
                        "codex-desktop-{}",
                        targets
                            .iter()
                            .filter(|t| t.protocol == Protocol::Codex)
                            .count()
                    );
                    targets.push(Target {
                        name,
                        protocol: Protocol::Codex,
                        executable: Some(executable),
                        endpoint: None,
                        credential_env: None,
                    });
                }
            }
        }
    }
    for (name, protocol, base, key, default) in [
        (
            "openai",
            Protocol::OpenaiResponses,
            "OPENAI_BASE_URL",
            "OPENAI_API_KEY",
            "https://api.openai.com/v1",
        ),
        (
            "anthropic",
            Protocol::Anthropic,
            "ANTHROPIC_BASE_URL",
            "ANTHROPIC_API_KEY",
            "https://api.anthropic.com/v1",
        ),
        (
            "openrouter",
            Protocol::OpenaiChat,
            "OPENROUTER_BASE_URL",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
        ),
    ] {
        if std::env::var_os(key).is_some() && !targets.iter().any(|t| t.name == name) {
            targets.push(endpoint_target(
                name,
                protocol,
                std::env::var(base).unwrap_or(default.into()),
                Some(key),
            ));
        }
    }
    // Only configured endpoints are contacted; no network-wide target scanning.
    let host_config = paths.home.join(".codex/config.toml");
    if host_config.exists() {
        let text = String::from_utf8(util::read_bounded(&host_config, 1024 * 1024)?)?;
        if let Ok(doc) = text.parse::<toml_edit::DocumentMut>()
            && let Some(providers) = doc.get("model_providers").and_then(|v| v.as_table_like())
        {
            for (name, item) in providers.iter() {
                if let Some(url) = item.get("base_url").and_then(|v| v.as_str()) {
                    let key = item.get("env_key").and_then(|v| v.as_str());
                    if key.is_some_and(|k| std::env::var_os(k).is_none()) {
                        continue;
                    }
                    let protocol =
                        if item.get("wire_api").and_then(|v| v.as_str()) == Some("responses") {
                            Protocol::OpenaiResponses
                        } else {
                            Protocol::OpenaiChat
                        };
                    let id = format!("codex-provider-{name}");
                    if !targets.iter().any(|t| t.name == id) {
                        targets.push(endpoint_target(&id, protocol, url.to_owned(), key));
                    }
                }
            }
        }
    }
    if on_path("ollama").is_some() && !targets.iter().any(|t| t.protocol == Protocol::Ollama) {
        targets.push(endpoint_target(
            "ollama",
            Protocol::Ollama,
            std::env::var("OLLAMA_HOST").unwrap_or("http://127.0.0.1:11434".into()),
            None,
        ));
    }
    let mut names = BTreeSet::new();
    for target in &targets {
        ensure!(
            !target.name.is_empty()
                && target.name.len() <= 128
                && names.insert(target.name.clone()),
            "invalid or duplicate inference target name"
        );
        if matches!(target.protocol, Protocol::Codex | Protocol::Claude) {
            ensure!(
                target
                    .executable
                    .as_ref()
                    .is_some_and(|p| p.is_absolute() && p.is_file()),
                "native target executable must be an existing absolute file"
            );
        } else {
            validated_url(
                target
                    .endpoint
                    .as_deref()
                    .context("HTTP target requires endpoint")?,
            )?;
        }
        if let Some(env) = &target.credential_env {
            ensure!(
                !env.is_empty() && env.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'),
                "invalid credential environment name"
            );
        }
    }
    Ok(targets)
}

pub fn validated_url(endpoint: &str) -> Result<reqwest::Url> {
    let url = reqwest::Url::parse(endpoint).context("invalid target endpoint")?;
    ensure!(
        url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none(),
        "target endpoint must not contain credentials, query or fragment"
    );
    let local = matches!(
        url.host_str(),
        Some("localhost" | "127.0.0.1" | "::1" | "[::1]")
    );
    ensure!(
        url.scheme() == "https" || (url.scheme() == "http" && local),
        "inference endpoint requires HTTPS, except loopback daemons"
    );
    Ok(url)
}
pub fn client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(600))
        .build()?)
}
pub fn request(
    client: &reqwest::Client,
    target: &Target,
    suffix: &str,
) -> Result<reqwest::RequestBuilder> {
    let base = target
        .endpoint
        .as_deref()
        .context("target lacks endpoint")?
        .trim_end_matches('/');
    validated_url(base)?;
    let mut request = client.get(format!("{base}/{suffix}"));
    if let Some(key) = &target.credential_env {
        let value = std::env::var(key).context("target credential environment is not set")?;
        ensure!(!value.is_empty(), "target credential environment is empty");
        request = if target.protocol == Protocol::Anthropic {
            request.header("x-api-key", value)
        } else {
            request.bearer_auth(value)
        };
    }
    if target.protocol == Protocol::Anthropic {
        request = request.header("anthropic-version", "2023-06-01");
    }
    Ok(request)
}
pub async fn response_json(mut response: reqwest::Response) -> Result<Value> {
    let status = response.status();
    // Never expose remote error bodies: they can echo prompts or credentials.
    ensure!(
        status.is_success(),
        "inference target HTTP status {}",
        status.as_u16()
    );
    ensure!(
        response
            .content_length()
            .is_none_or(|n| n <= MAX_METADATA as u64),
        "target response exceeds limit"
    );
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        ensure!(
            bytes.len() + chunk.len() <= MAX_METADATA,
            "target response exceeds limit"
        );
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).context("target did not return JSON")
}
fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}
fn number(raw: &Value, keys: &[&str]) -> Option<u64> {
    keys.iter()
        .find_map(|k| raw.pointer(k).and_then(Value::as_u64))
}
fn boolean(raw: &Value, keys: &[&str]) -> Option<bool> {
    keys.iter()
        .find_map(|k| raw.pointer(k).and_then(Value::as_bool))
}

fn supported_names(raw: &Value, base: &str, names: &[&str]) -> Vec<String> {
    if raw
        .pointer(&format!("{base}/supported"))
        .and_then(Value::as_bool)
        != Some(true)
    {
        return Vec::new();
    }
    names
        .iter()
        .filter(|name| {
            raw.pointer(&format!("{base}/{name}/supported"))
                .and_then(Value::as_bool)
                == Some(true)
        })
        .map(|name| (*name).to_owned())
        .collect()
}

fn anthropic_thinking_modes(raw: &Value) -> Vec<String> {
    if boolean(raw, &["/capabilities/thinking/supported"]) != Some(true) {
        return Vec::new();
    }
    ["adaptive", "enabled", "disabled"]
        .iter()
        .filter(|name| {
            raw.pointer(&format!("/capabilities/thinking/types/{name}/supported"))
                .and_then(Value::as_bool)
                == Some(true)
        })
        .map(|name| (*name).to_owned())
        .collect()
}

pub fn normalize(target: &Target, raw: &Value, execution_ready: bool) -> Result<Model> {
    let id = if target.protocol == Protocol::Claude {
        raw.get("resolvedModel")
            .and_then(Value::as_str)
            .context("Claude model entry has no resolved identity")?
    } else {
        raw.get("resolvedModel")
            .or_else(|| raw.get("id"))
            .or_else(|| raw.get("model"))
            .or_else(|| raw.get("name"))
            .and_then(Value::as_str)
            .context("target model has no native identity")?
    };
    ensure!(
        !id.is_empty() && id.len() <= 256 && !id.contains(['\0', '\n', '\r']),
        "invalid model identity"
    );
    let parameters = strings(raw.get("supported_parameters"));
    let modalities = strings(
        raw.get("inputModalities")
            .or_else(|| raw.pointer("/architecture/input_modalities"))
            .or_else(|| raw.pointer("/capabilities/input_modalities")),
    );
    let caps = strings(raw.get("capabilities"));
    let mut efforts = strings(
        raw.get("reasoning_efforts")
            .or_else(|| raw.pointer("/capabilities/reasoning_efforts")),
    );
    if let Some(list) = raw
        .get("supportedReasoningEfforts")
        .and_then(Value::as_array)
    {
        efforts = list
            .iter()
            .filter_map(|v| v.get("reasoningEffort").and_then(Value::as_str))
            .map(str::to_owned)
            .collect();
    }
    if target.protocol == Protocol::Claude {
        efforts = if raw.get("supportsEffort").and_then(Value::as_bool) == Some(true) {
            strings(raw.get("supportedEffortLevels"))
        } else {
            Vec::new()
        };
    } else if target.protocol == Protocol::Anthropic {
        efforts = supported_names(
            raw,
            "/capabilities/effort",
            &["low", "medium", "high", "xhigh", "max"],
        );
    } else if let Some(list) = raw.get("supportedEffortLevels").and_then(Value::as_array) {
        efforts = list
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect();
    }
    let native = matches!(target.protocol, Protocol::Codex | Protocol::Claude);
    let discovered_tools = boolean(raw, &["/capabilities/tools", "/supports_tools"])
        .or_else(|| parameters.contains(&"tools".into()).then_some(true))
        .or_else(|| caps.contains(&"tools".into()).then_some(true));
    let discovered_structured_outputs = boolean(
        raw,
        &[
            "/capabilities/structured_outputs",
            "/capabilities/structured_outputs/supported",
            "/supports_structured_outputs",
        ],
    )
    .or_else(|| {
        parameters
            .iter()
            .any(|p| p == "structured_outputs" || p == "response_format")
            .then_some(true)
    });
    let capabilities = Capabilities {
        tools: if native { Some(true) } else { discovered_tools },
        structured_outputs: if native {
            Some(true)
        } else {
            discovered_structured_outputs
        },
        vision: boolean(
            raw,
            &[
                "/capabilities/vision",
                "/capabilities/image_input/supported",
                "/supports_vision",
            ],
        )
        .or_else(|| {
            (!modalities.is_empty()).then(|| {
                modalities
                    .iter()
                    .any(|m| m == "image" || m == "vision" || m == "image_input")
            })
        })
        .or_else(|| caps.contains(&"vision".into()).then_some(true)),
        reasoning_efforts: efforts,
        thinking_modes: if target.protocol == Protocol::Anthropic {
            anthropic_thinking_modes(raw)
        } else if target.protocol == Protocol::Ollama && caps.iter().any(|cap| cap == "thinking") {
            vec!["thinking".into()]
        } else {
            Vec::new()
        },
        context_window: number(
            raw,
            &[
                "/max_input_tokens",
                "/context_length",
                "/context_window",
                "/top_provider/context_length",
            ],
        ),
        max_completion_tokens: number(
            raw,
            &[
                "/max_tokens",
                "/max_completion_tokens",
                "/max_output_tokens",
                "/top_provider/max_completion_tokens",
            ],
        ),
        thinking_budget_max: number(
            raw,
            &["/thinking_budget_max", "/capabilities/thinking_budget_max"],
        ),
        pricing: raw.get("pricing").cloned(),
    };
    let mut evidence = vec![format!("discovered metadata from {}", target.name)];
    if native {
        // These are properties of Mirket's schema-bound action adapter, not a
        // claim about an undocumented provider-native function-call feature.
        evidence.push(
            "native CLI strict JSON action protocol; tool dispatch is owned by Mirket".into(),
        );
        if target.protocol == Protocol::Claude && !capabilities.reasoning_efforts.is_empty() {
            evidence.push("Claude initialize supportsEffort/supportedEffortLevels".into());
        }
    }
    let mut model = Model {
        id: id.into(),
        target: target.name.clone(),
        display_name: raw
            .get("displayName")
            .or_else(|| raw.get("display_name"))
            .or_else(|| raw.get("name"))
            .and_then(Value::as_str)
            .unwrap_or(id)
            .into(),
        description: raw
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("")
            .into(),
        line: raw.get("line").and_then(Value::as_str).map(str::to_owned),
        capabilities,
        tier: Tier::T1Deterministic,
        classification_evidence: evidence,
        available: execution_ready
            && raw.get("lifecycle").and_then(Value::as_str) != Some("retired"),
    };
    classify(&mut model);
    Ok(model)
}

fn next_model_page(
    raw: &Value,
    page: usize,
    cursors: &mut BTreeSet<String>,
) -> Result<Option<String>> {
    if raw.get("has_more").and_then(Value::as_bool) != Some(true) {
        return Ok(None);
    }
    ensure!(page < 63, "model pagination exceeded 64 pages");
    let cursor = raw
        .get("last_id")
        .and_then(Value::as_str)
        .context("model pagination lacks cursor")?;
    ensure!(cursors.insert(cursor.to_owned()), "repeated model cursor");
    let mut url = reqwest::Url::parse("https://pagination.invalid/models")?;
    url.query_pairs_mut()
        .append_pair("after_id", cursor)
        .append_pair("limit", "100");
    Ok(Some(format!("models?{}", url.query().unwrap())))
}

fn reclassify_cached_models(registry: &mut Registry) {
    for model in &mut registry.models {
        classify(model);
        model.available &= registry.targets.iter().any(|status| {
            status.target.name == model.target && status.authenticated && status.execution_ready
        });
    }
}

fn http_inference_ready_with_env<F>(target: &Target, mut credential: F) -> bool
where
    F: FnMut(&str) -> Option<OsString>,
{
    if matches!(target.protocol, Protocol::Codex | Protocol::Claude) {
        return false;
    }
    if let Some(name) = target.credential_env.as_deref() {
        return credential(name)
            .and_then(|value| value.into_string().ok())
            .is_some_and(|value| !value.is_empty());
    }
    target
        .endpoint
        .as_deref()
        .and_then(|endpoint| validated_url(endpoint).ok())
        .and_then(|url| url.host_str().map(str::to_owned))
        .is_some_and(|host| matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1"))
}

async fn authenticated_inference_ready(target: &Target, home: &Path) -> bool {
    match target.protocol {
        Protocol::Claude => {
            let Some(executable) = target.executable.as_deref() else {
                return false;
            };
            super::native::claude_authenticated(executable, home)
                .await
                .is_ok()
        }
        Protocol::Codex => {
            let Some(executable) = target.executable.as_deref() else {
                return false;
            };
            super::native::codex_authenticated(executable, home)
                .await
                .is_ok()
        }
        _ => http_inference_ready_with_env(target, |name| std::env::var_os(name)),
    }
}

fn rebind_cached_http_auth_with_env<F>(registry: &mut Registry, mut credential: F)
where
    F: FnMut(&str) -> Option<OsString>,
{
    for status in &mut registry.targets {
        if matches!(status.target.protocol, Protocol::Codex | Protocol::Claude) {
            continue;
        }
        let current = http_inference_ready_with_env(&status.target, &mut credential);
        status.authenticated &= current;
        status.execution_ready &= status.authenticated;
        if !current {
            status.diagnostic = Some(
                "Discovery only: remote inference credentials are absent; public model metadata does not establish inference access"
                    .into(),
            );
        }
    }
}

async fn cached_native_auth_current(registry: &Registry, home: &Path) -> bool {
    for status in &registry.targets {
        match status.target.protocol {
            Protocol::Claude => {
                let Some(executable) = status.target.executable.as_deref() else {
                    return false;
                };
                let current = super::native::claude_authenticated(executable, home)
                    .await
                    .is_ok();
                if current != status.authenticated {
                    return false;
                }
            }
            Protocol::Codex => {
                let Some(executable) = status.target.executable.as_deref() else {
                    return false;
                };
                let current = super::native::codex_authenticated(executable, home)
                    .await
                    .is_ok();
                if current != status.authenticated {
                    return false;
                }
            }
            _ => {}
        }
    }
    true
}

async fn probe(target: &Target, home: &Path) -> Result<Vec<Value>> {
    match target.protocol {
        Protocol::Codex => {
            super::native::discover_codex(
                target
                    .executable
                    .as_deref()
                    .context("native executable missing")?,
                home,
            )
            .await
        }
        Protocol::Claude => {
            super::native::discover_claude(
                target
                    .executable
                    .as_deref()
                    .context("native executable missing")?,
                home,
            )
            .await
        }
        _ => {
            let client = client()?;
            let suffix = if target.protocol == Protocol::Ollama {
                "api/tags"
            } else {
                "models"
            };
            let mut result = Vec::new();
            let mut next = suffix.to_owned();
            let mut cursors = BTreeSet::new();
            for page in 0..64 {
                let raw = response_json(
                    request(&client, target, &next)?
                        .timeout(Duration::from_secs(15))
                        .send()
                        .await?,
                )
                .await?;
                let list = raw
                    .get("data")
                    .or_else(|| raw.get("models"))
                    .and_then(Value::as_array)
                    .context("target model list lacks data/models array")?;
                result.extend(list.iter().cloned());
                ensure!(result.len() <= 4096, "model catalog exceeds limit");
                if target.protocol == Protocol::Ollama {
                    break;
                }
                if let Some(next_page) = next_model_page(&raw, page, &mut cursors)? {
                    next = next_page;
                } else {
                    break;
                }
            }
            if target.protocol == Protocol::Ollama {
                for raw in &mut result {
                    let Some(name) = raw.get("name").and_then(Value::as_str) else {
                        continue;
                    };
                    let get = request(&client, target, "api/show")?.build()?;
                    let mut post = client
                        .post(get.url().clone())
                        .json(&json!({"model":name}))
                        .timeout(Duration::from_secs(15));
                    for (header, value) in get.headers() {
                        post = post.header(header, value);
                    }
                    let response = post.send().await?;
                    let details = response_json(response).await?;
                    raw["capabilities"] =
                        details.get("capabilities").cloned().unwrap_or(Value::Null);
                    if let Some(info) = details.get("model_info").and_then(Value::as_object) {
                        raw["context_window"] = info
                            .iter()
                            .find(|(k, _)| k.ends_with(".context_length"))
                            .map(|(_, v)| v.clone())
                            .unwrap_or(Value::Null);
                    }
                }
            }
            Ok(result)
        }
    }
}

pub async fn discover(paths: &Paths, refresh: bool) -> Result<Registry> {
    let _guard = DISCOVERY_LOCK.lock().await;
    let config = config(paths)?;
    let targets = targets(paths, &config)?;
    let fingerprint = cache_fingerprint(paths, &targets)?;
    let cache = paths.cache().join("inference-models.json");
    if !refresh
        && cache.exists()
        && let Ok(bytes) = util::read_bounded(&cache, MAX_METADATA as u64)
        && let Ok(mut registry) = serde_json::from_slice::<Registry>(&bytes)
        && registry.version == REGISTRY_VERSION
        && registry.fingerprint == fingerprint
        && registry.refreshed_at <= now()
        && now().saturating_sub(registry.refreshed_at) < config.ttl_seconds
        && registry.targets.len() == targets.len()
        && targets.iter().all(|target| {
            registry
                .targets
                .iter()
                .filter(|status| status.target == *target)
                .count()
                == 1
        })
        && cached_native_auth_current(&registry, &paths.home).await
    {
        // The cache is an optimization, not a source of policy tiers.
        // Recompute classification and bind availability to current auth.
        rebind_cached_http_auth_with_env(&mut registry, |name| std::env::var_os(name));
        reclassify_cached_models(&mut registry);
        return Ok(registry);
    }
    let mut registry = Registry {
        version: REGISTRY_VERSION,
        refreshed_at: now(),
        fingerprint,
        targets: vec![],
        models: vec![],
    };
    for target in targets {
        match tokio::time::timeout(Duration::from_secs(30),probe(&target,&paths.home)).await {
            Ok(Ok(raw))=>{
                let authenticated = authenticated_inference_ready(&target, &paths.home).await;
                let execution_ready = authenticated;
                let mut seen=BTreeSet::new();
                for item in raw {if let Ok(model)=normalize(&target,&item,execution_ready) && seen.insert(model.id.clone()) {registry.models.push(model);}}
                let diagnostic = if !authenticated {
                    Some("Discovery only: remote inference credentials are absent or native account access is unavailable; public model metadata does not establish inference access".into())
                } else {
                    None
                };
                registry.targets.push(TargetStatus{target,authenticated,execution_ready,diagnostic});
            }
            _=>registry.targets.push(TargetStatus{target,authenticated:false,execution_ready:false,diagnostic:Some("Target probe unavailable, unauthenticated, malformed, or exceeded its deadline; no stale models are eligible".into())}),
        }
    }
    util::ensure_directory(&paths.cache())?;
    let _lock = util::FileLock::acquire(&paths.cache().join("inference-models.lock"))?;
    util::atomic_write_with_mode(&cache, &serde_json::to_vec_pretty(&registry)?, Some(0o600))?;
    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_capabilities_stay_unknown() {
        let target = endpoint_target(
            "fixture",
            Protocol::OpenaiChat,
            "http://127.0.0.1:1/v1".into(),
            None,
        );
        let m = normalize(&target, &json!({"id":"future-generation"}), true).unwrap();
        assert_eq!(m.tier, Tier::T1Deterministic);
        assert_eq!(m.capabilities.vision, None);
        assert_eq!(m.capabilities.tools, None);
    }
    #[test]
    fn endpoint_cannot_embed_auth_or_leak_to_redirect() {
        for url in [
            "http://public.example/v1",
            "http://localhost.evil.example/v1",
            "http://user@localhost:1234/v1",
            "https://user:pass@example.test/v1",
            "https://example.test/v1?key=private",
            "https://example.test/v1#token",
        ] {
            assert!(validated_url(url).is_err());
        }
        assert!(validated_url("http://127.0.0.1:23456/v1").is_ok());
        assert!(validated_url("http://[::1]:23456/v1").is_ok());
    }
    #[test]
    fn config_has_no_model_registry_surface() {
        assert!(serde_json::from_value::<Config>(json!({"models":["fixture"]})).is_err());
    }

    #[test]
    fn public_remote_metadata_does_not_establish_inference_auth() {
        let remote = endpoint_target(
            "remote-fixture",
            Protocol::Anthropic,
            "https://models.example.test/v1".into(),
            None,
        );
        assert!(!http_inference_ready_with_env(&remote, |_| None));
        let loopback = endpoint_target(
            "loopback-fixture",
            Protocol::Ollama,
            "http://127.0.0.1:11434".into(),
            None,
        );
        assert!(http_inference_ready_with_env(&loopback, |_| None));
        let secure_loopback = endpoint_target(
            "secure-loopback-fixture",
            Protocol::OpenaiChat,
            "https://localhost:8443/v1".into(),
            None,
        );
        assert!(http_inference_ready_with_env(&secure_loopback, |_| None));

        let keyed = endpoint_target(
            "keyed-fixture",
            Protocol::OpenaiResponses,
            "https://models.example.test/v1".into(),
            Some("SYNTHETIC_API_KEY"),
        );
        assert!(!http_inference_ready_with_env(&keyed, |_| None));
        assert!(!http_inference_ready_with_env(&keyed, |_| Some(
            OsString::new()
        )));
        assert!(http_inference_ready_with_env(&keyed, |_| Some(
            OsString::from("synthetic")
        )));
    }

    #[test]
    fn cached_public_catalog_cannot_retain_forged_inference_readiness() {
        let target = endpoint_target(
            "remote-fixture",
            Protocol::OpenaiChat,
            "https://models.example.test/v1".into(),
            None,
        );
        let mut model = normalize(
            &target,
            &json!({"id":"synthetic-future-workhorse","supports_tools":true,"supports_structured_outputs":true,"reasoning_efforts":["high"]}),
            true,
        )
        .unwrap();
        assert!(model.available);
        model.available = true;
        let mut registry = Registry {
            version: REGISTRY_VERSION,
            refreshed_at: now(),
            fingerprint: "synthetic".into(),
            targets: vec![TargetStatus {
                target,
                authenticated: true,
                execution_ready: true,
                diagnostic: None,
            }],
            models: vec![model],
        };
        rebind_cached_http_auth_with_env(&mut registry, |_| None);
        reclassify_cached_models(&mut registry);
        assert!(!registry.targets[0].authenticated);
        assert!(!registry.targets[0].execution_ready);
        assert!(
            registry.targets[0]
                .diagnostic
                .as_deref()
                .unwrap()
                .contains("Discovery only")
        );
        assert!(!registry.models[0].available);
    }

    #[tokio::test]
    async fn native_readiness_never_comes_from_endpoint_shape() {
        let target = Target {
            name: "native-fixture".into(),
            protocol: Protocol::Claude,
            endpoint: Some("http://127.0.0.1:1234".into()),
            credential_env: None,
            executable: None,
        };
        assert!(!http_inference_ready_with_env(&target, |_| None));
        assert!(!authenticated_inference_ready(&target, Path::new("/unused")).await);
    }

    #[test]
    fn claude_requires_resolved_identity_and_supported_effort_contract() {
        let target = Target {
            name: "claude-fixture".into(),
            protocol: Protocol::Claude,
            endpoint: None,
            credential_env: None,
            executable: Some(PathBuf::from("/unused/claude")),
        };
        assert!(
            normalize(
                &target,
                &json!({"value":"opus","supportsEffort":true,"supportedEffortLevels":["high"]}),
                true
            )
            .is_err()
        );
        let raw = json!({"value":"opus","resolvedModel":"future-full-model","supportsEffort":false,"supportedEffortLevels":["high"]});
        let model = normalize(&target, &raw, true).unwrap();
        assert_eq!(model.id, "future-full-model");
        assert!(model.capabilities.reasoning_efforts.is_empty());
        let high = normalize(&target, &json!({"value":"opus","resolvedModel":"future-full-model","supportsEffort":true,"supportedEffortLevels":["high"]}), true).unwrap();
        assert_eq!(high.capabilities.reasoning_efforts, vec!["high"]);
    }

    #[test]
    fn anthropic_nested_capabilities_and_limits_are_normalized_without_tool_inference() {
        let target = endpoint_target(
            "anthropic-fixture",
            Protocol::Anthropic,
            "https://example.test/v1".into(),
            Some("SYNTHETIC_KEY"),
        );
        let raw = json!({
            "id":"synthetic-future-core",
            "display_name":"Synthetic Future Core",
            "line":"opus",
            "lifecycle":"active",
            "max_input_tokens":180000,
            "max_tokens":96000,
            "capabilities":{
                "structured_outputs":{"supported":true},
                "image_input":{"supported":true},
                "effort":{
                    "supported":true,
                    "low":{"supported":true},
                    "medium":{"supported":false},
                    "high":{"supported":true},
                    "xhigh":{"supported":true},
                    "max":{"supported":false}
                },
                "thinking":{
                    "supported":true,
                    "types":{
                        "adaptive":{"supported":true},
                        "enabled":{"supported":false},
                        "disabled":{"supported":true}
                    }
                },
                "server_tools":{"supported":true}
            }
        });
        let model = normalize(&target, &raw, true).unwrap();
        assert_eq!(model.display_name, "Synthetic Future Core");
        assert_eq!(model.line.as_deref(), Some("opus"));
        assert_eq!(model.capabilities.structured_outputs, Some(true));
        assert_eq!(model.capabilities.vision, Some(true));
        assert_eq!(model.capabilities.tools, None);
        assert_eq!(
            model.capabilities.reasoning_efforts,
            vec!["low".to_string(), "high".to_string(), "xhigh".to_string()]
        );
        assert_eq!(
            model.capabilities.thinking_modes,
            vec!["adaptive".to_string(), "disabled".to_string()]
        );
        assert_eq!(model.capabilities.context_window, Some(180000));
        assert_eq!(model.capabilities.max_completion_tokens, Some(96000));
        assert_eq!(model.tier, Tier::T1Deterministic);
        assert!(!model.available);

        let mut with_client_tools = raw.clone();
        with_client_tools["supports_tools"] = json!(true);
        let capable = normalize(&target, &with_client_tools, true).unwrap();
        assert_eq!(capable.tier, Tier::T3Architect);
        assert!(capable.available);

        let mut unsupported = with_client_tools;
        unsupported["capabilities"]["structured_outputs"]["supported"] = json!(false);
        unsupported["capabilities"]["image_input"]["supported"] = json!(false);
        unsupported["capabilities"]["effort"]["supported"] = json!(false);
        unsupported["capabilities"]["thinking"]["supported"] = json!(false);
        let unsupported = normalize(&target, &unsupported, true).unwrap();
        assert_eq!(unsupported.capabilities.structured_outputs, Some(false));
        assert_eq!(unsupported.capabilities.vision, Some(false));
        assert!(unsupported.capabilities.reasoning_efforts.is_empty());
        assert!(unsupported.capabilities.thinking_modes.is_empty());
        assert_eq!(unsupported.tier, Tier::T1Deterministic);
    }

    #[test]
    fn anthropic_line_and_lifecycle_override_identity_hints() {
        let target = endpoint_target(
            "anthropic-fixture",
            Protocol::Anthropic,
            "https://example.test/v1".into(),
            None,
        );
        let raw = json!({
            "id":"synthetic-future-opus-frontier",
            "display_name":"Synthetic Opus Frontier",
            "description":"Full-size flagship with deep reasoning",
            "line":"haiku",
            "supports_tools":true,
            "max_tokens":65536,
            "capabilities":{
                "structured_outputs":{"supported":true},
                "effort":{"supported":true,"high":{"supported":true}},
                "thinking":{"supported":false,"types":{"enabled":{"supported":true}}}
            }
        });
        let utility = normalize(&target, &raw, true).unwrap();
        assert_eq!(utility.tier, Tier::T1Deterministic);
        assert!(utility.available);
        assert!(utility.capabilities.thinking_modes.is_empty());

        let mut retired = raw.clone();
        retired["line"] = json!("opus");
        retired["lifecycle"] = json!("retired");
        let retired = normalize(&target, &retired, true).unwrap();
        assert_eq!(retired.tier, Tier::T3Architect);
        assert!(!retired.available);
    }

    #[test]
    fn ollama_capability_strings_expose_thinking_without_assuming_structured_output() {
        let target = endpoint_target(
            "ollama-fixture",
            Protocol::Ollama,
            "http://127.0.0.1:11434".into(),
            None,
        );
        let model = normalize(
            &target,
            &json!({"name":"synthetic-future-core","capabilities":["tools","vision","thinking"]}),
            true,
        )
        .unwrap();
        assert_eq!(model.capabilities.tools, Some(true));
        assert_eq!(model.capabilities.vision, Some(true));
        assert_eq!(
            model.capabilities.thinking_modes,
            vec!["thinking".to_string()]
        );
        assert_eq!(model.capabilities.structured_outputs, None);
        assert!(!model.available);
    }

    #[test]
    fn cache_fingerprint_changes_with_credentials_and_executable_bytes() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().into()).unwrap();
        let executable = home.path().join("native-fixture");
        std::fs::write(&executable, b"first native binary").unwrap();
        let target = Target {
            name: "native-fixture".into(),
            protocol: Protocol::Claude,
            endpoint: None,
            credential_env: Some("MIRKET_ROUTER_TEST_CREDENTIAL".into()),
            executable: Some(executable.clone()),
        };
        let fingerprint = |value: Option<&str>| {
            cache_fingerprint_with_env(&paths, std::slice::from_ref(&target), |_| {
                value.map(OsString::from)
            })
            .unwrap()
        };
        let first = fingerprint(Some("first-secret"));
        assert_ne!(first, fingerprint(None));
        assert_ne!(first, fingerprint(Some("second-secret")));
        std::fs::write(&executable, b"second native binary").unwrap();
        assert_ne!(first, fingerprint(Some("first-secret")));
    }

    #[test]
    fn cache_reclassification_rejects_forged_tier_and_unready_target() {
        let target = endpoint_target(
            "fixture",
            Protocol::OpenaiChat,
            "http://127.0.0.1:1/v1".into(),
            None,
        );
        let mut model = normalize(&target, &json!({"id":"unknown-future"}), true).unwrap();
        model.tier = Tier::T3Architect;
        model.available = true;
        let mut registry = Registry {
            version: REGISTRY_VERSION,
            refreshed_at: now(),
            fingerprint: "fixture".into(),
            targets: vec![TargetStatus {
                target,
                authenticated: true,
                execution_ready: false,
                diagnostic: None,
            }],
            models: vec![model],
        };
        reclassify_cached_models(&mut registry);
        assert_eq!(registry.models[0].tier, Tier::T1Deterministic);
        assert!(!registry.models[0].available);
    }

    #[test]
    fn pagination_rejects_incomplete_catalog_at_page_limit() {
        let mut cursors = BTreeSet::new();
        let page = json!({"has_more":true,"last_id":"future/model?x=1"});
        assert_eq!(
            next_model_page(&page, 0, &mut cursors).unwrap().as_deref(),
            Some("models?after_id=future%2Fmodel%3Fx%3D1&limit=100")
        );
        assert!(next_model_page(&page, 1, &mut cursors).is_err());
        let mut cursors = BTreeSet::new();
        assert!(next_model_page(&page, 63, &mut cursors).is_err());
    }
}
