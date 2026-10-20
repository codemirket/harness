//! Numeric-only observations. Native SDK prices are estimates, never invoices.
use super::discovery::Protocol;
use crate::{paths::Paths, util};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

/// Receipts are published exclusively and synced before dispatch. A crash leaves
/// an explicit unknown attempt; a retry always gets its own identity.
fn publish(path: &Path, value: &Value) -> Result<()> {
    util::reject_symlink_components(path)?;
    let parent = path.parent().context("accounting parent")?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(&serde_json::to_vec_pretty(value)?)?;
    file.as_file().sync_all()?;
    file.persist_noclobber(path)?;
    Ok(())
}

pub(super) fn initialize(run: &Path) -> Result<()> {
    publish(
        &run.join("accounting-version.json"),
        &json!({"schema":1,
        "source_sha256":env!("MIRKET_SOURCE_SHA256"),
        "scope":"Vision conformance probes recorded from run creation; discovery sends no inference request."}),
    )
}

pub(super) struct Probe {
    directory: PathBuf,
    started: std::time::Instant,
}

impl Probe {
    pub(super) fn start(run: &Path, target: &str, model: &str) -> Result<Self> {
        let root = run.join("probes");
        util::ensure_directory(&root)?;
        let directory = tempfile::Builder::new()
            .prefix("probe-")
            .tempdir_in(root)?
            .keep();
        publish(
            &directory.join("intent.json"),
            &json!({"schema":1,
            "probe_id":directory.file_name().context("probe ID")?.to_string_lossy(),
            "run_id":run.file_name().context("run ID")?.to_string_lossy(),
            "target":target,"model":model,"kind":"vision_conformance",
            "started_at":super::durable::now(),"source_sha256":env!("MIRKET_SOURCE_SHA256")}),
        )?;
        Ok(Self {
            directory,
            started: std::time::Instant::now(),
        })
    }
    pub(super) fn finish(self, observation: Value, passed: bool, diagnostic: Value) -> Result<()> {
        publish(
            &self.directory.join("result.json"),
            &json!({"schema":1,
            "status":if passed {"passed"} else {"failed"},
            "elapsed_ms":self.started.elapsed().as_millis(),
            "inference":observation,"diagnostic":diagnostic}),
        )
    }
}

fn probes(run: &Path) -> Result<Vec<Value>> {
    let root = run.join("probes");
    util::reject_symlink_components(&root)?;
    if !root.exists() {
        return Ok(vec![]);
    }
    let mut entries = fs::read_dir(&root)?
        .take(1025)
        .collect::<std::io::Result<Vec<_>>>()?;
    ensure!(
        entries.len() <= 1024,
        "probe receipt limit exceeded; accounting is incomplete"
    );
    entries.sort_by_key(|entry| entry.file_name());
    let mut output = Vec::new();
    for entry in entries {
        let path = entry.path();
        util::reject_symlink_components(&path)?;
        ensure!(entry.file_type()?.is_dir(), "invalid probe receipt entry");
        let intent: Value =
            serde_json::from_slice(&util::read_bounded(&path.join("intent.json"), 16384)?)?;
        ensure!(
            intent["schema"] == 1
                && intent["kind"] == "vision_conformance"
                && intent["probe_id"].as_str() == entry.file_name().to_str()
                && intent["run_id"].as_str() == run.file_name().and_then(|s| s.to_str()),
            "probe receipt identity mismatch"
        );
        let result_path = path.join("result.json");
        util::reject_symlink_components(&result_path)?;
        let result = if result_path.exists() {
            let value: Value = serde_json::from_slice(&util::read_bounded(&result_path, 16384)?)?;
            ensure!(
                value["schema"] == 1
                    && matches!(value["status"].as_str(), Some("passed" | "failed"))
                    && value["inference"].is_object(),
                "invalid completed probe receipt"
            );
            value
        } else {
            json!({"status":"pending_or_interrupted","inference":empty()})
        };
        output.push(json!({"intent":intent,"result":result,"inference":result["inference"]}));
    }
    Ok(output)
}

pub(super) fn run_summary(run: &Path, attempts: &[Value]) -> Result<Value> {
    let records = probes(run)?;
    let probe_summary = summary(&records);
    let mut all = attempts.to_vec();
    all.extend(records.iter().cloned());
    let mut total = summary(&all);
    total["scope"] = json!(
        "Recorded worker attempts plus distinct durable vision probes for this run, including resumed probes. Discovery uses control/model-list requests, not inference."
    );
    total["worker"] = summary(attempts);
    total["vision_probes"] = probe_summary;
    total["vision_probes"]["scope"] = json!(
        "Distinct durable vision conformance probe attempts; pending and interrupted receipts retain unknown usage and cost."
    );
    total["probe_receipts"] = json!(records);
    let marker = run.join("accounting-version.json");
    util::reject_symlink_components(&marker)?;
    let complete = if marker.exists() {
        let value: Value = serde_json::from_slice(&util::read_bounded(&marker, 4096)?)?;
        ensure!(value["schema"] == 1, "unknown accounting schema");
        true
    } else {
        false
    };
    total["probe_history_complete"] = json!(complete);
    if !complete {
        total["total_cost_estimate_usd"] = Value::Null;
    }
    total["billing_usd"] = Value::Null;
    total["billing_source"] =
        json!("unavailable; provider usage and SDK estimates are not an account invoice");
    Ok(total)
}

pub fn inspect_run(paths: &Paths, id: &str) -> Result<Value> {
    let run = super::durable::run_path(paths, id)?;
    let _lock = util::FileLock::acquire(&run.join("execution.lock"))?;
    let trace = run.join("trace.json");
    util::reject_symlink_components(&trace)?;
    let attempts = if trace.exists() {
        let value: Value = serde_json::from_slice(&util::read_bounded(&trace, 32 * 1024 * 1024)?)?;
        value["attempts"]
            .as_array()
            .context("trace attempts missing")?
            .clone()
    } else {
        vec![]
    };
    let mut result = run_summary(&run, &attempts)?;
    result["run_id"] = json!(id);
    result["worker_trace_available"] = json!(trace.exists());
    if !trace.exists() {
        result["total_cost_estimate_usd"] = Value::Null;
    }
    Ok(result)
}

pub(super) fn empty() -> Value {
    json!({"usage":null,"usage_source":"unavailable","response_sha256":null,
        "cost_estimate_usd":null,"cost_source":"unavailable"})
}

pub(super) fn observe(raw: &Value, protocol: &Protocol, observation: &mut Value) {
    let usage = &raw["usage"];
    let (input, output, total, cached, written, reasoning) = match protocol {
        Protocol::OpenaiChat => (
            &usage["prompt_tokens"],
            &usage["completion_tokens"],
            &usage["total_tokens"],
            &usage["prompt_tokens_details"]["cached_tokens"],
            &usage["prompt_tokens_details"]["cache_write_tokens"],
            &usage["completion_tokens_details"]["reasoning_tokens"],
        ),
        Protocol::OpenaiResponses | Protocol::Codex => (
            &usage["input_tokens"],
            &usage["output_tokens"],
            &usage["total_tokens"],
            &usage["input_tokens_details"]["cached_tokens"],
            &usage["input_tokens_details"]["cache_write_tokens"],
            &usage["output_tokens_details"]["reasoning_tokens"],
        ),
        Protocol::Anthropic | Protocol::Claude => (
            &usage["input_tokens"],
            &usage["output_tokens"],
            &Value::Null,
            &usage["cache_read_input_tokens"],
            &usage["cache_creation_input_tokens"],
            &Value::Null,
        ),
        Protocol::Ollama => (
            &raw["prompt_eval_count"],
            &raw["eval_count"],
            &Value::Null,
            &Value::Null,
            &Value::Null,
            &Value::Null,
        ),
    };
    let fields = [
        ("input_tokens", input),
        ("output_tokens", output),
        ("total_tokens", total),
        ("cached_input_tokens", cached),
        ("cache_write_input_tokens", written),
        ("reasoning_output_tokens", reasoning),
    ];
    if fields.iter().any(|(_, value)| value.as_u64().is_some()) {
        observation["usage"] = fields
            .iter()
            .map(|(key, value)| ((*key).to_owned(), json!(value.as_u64())))
            .collect::<serde_json::Map<_, _>>()
            .into();
        observation["usage_source"] = json!(if *protocol == Protocol::Claude {
            "claude_result_main_loop"
        } else {
            "provider_reported"
        });
        observation["input_token_semantics"] = json!(if matches!(
            protocol,
            Protocol::Claude | Protocol::Anthropic
        ) {
            "uncached_input; cache_read and cache_write are separate"
        } else {
            "input includes cache counters; reasoning is part of output"
        });
    }
    // Each adapter call starts a new native session and sends exactly one user
    // turn. Use only its terminal result, never sum cumulative assistant frames.
    if *protocol == Protocol::Claude
        && let Some(cost) = raw["total_cost_usd"]
            .as_f64()
            .filter(|v| v.is_finite() && *v >= 0.0)
    {
        observation["cost_estimate_usd"] = json!(cost);
        observation["cost_source"] = json!("native_sdk_estimate");
    }
}

pub(super) fn summary(attempts: &[Value]) -> Value {
    let observations: Vec<_> = attempts
        .iter()
        .filter(|v| v.get("inference").is_some() || v.get("turn").is_some())
        .map(|v| &v["inference"])
        .collect();
    let mut known_cost = 0.0;
    let mut unknown_cost = 0;
    for value in &observations {
        if let Some(cost) = value["cost_estimate_usd"]
            .as_f64()
            .filter(|v| v.is_finite() && *v >= 0.0)
        {
            known_cost += cost;
        } else {
            unknown_cost += 1;
        }
    }
    // Do not mix Anthropic's uncached input with inclusive input counters.
    // Per-attempt counters retain their own scope and semantics.
    json!({"inference_attempts":observations.len(),
        "attempts_with_usage":observations.iter().filter(|v| !v["usage"].is_null()).count(),
        "unknown_cost_attempts":unknown_cost,
        "known_cost_estimate_usd":if known_cost.is_finite() {Some(known_cost)} else {None},
        "total_cost_estimate_usd":if !observations.is_empty() && unknown_cost == 0 && known_cost.is_finite() {Some(known_cost)} else {None},
        "scope":"Recorded worker inference attempts, including retries and restored attempts; excludes discovery and vision conformance probes.",
        "limits":"SDK estimates are not billing. Missing, interrupted or older observations remain unknown; input counters have provider-specific cache semantics."})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn probe_failures_crashes_and_resume_retries_are_durable_and_not_double_counted() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().to_path_buf()).unwrap();
        let run = paths.root.join("runs/routed-fixture");
        util::ensure_directory(&run).unwrap();
        initialize(&run).unwrap();
        let known = || {
            json!({"usage":{"input_tokens":10},"cost_estimate_usd":0.25,
            "cost_source":"native_sdk_estimate"})
        };
        let failed = Probe::start(&run, "fixture", "model").unwrap();
        failed
            .finish(known(), false, json!({"code":"contract_mismatch"}))
            .unwrap();
        let first = inspect_run(&paths, "routed-fixture").unwrap();
        assert_eq!(first["vision_probes"]["inference_attempts"], 1);
        assert_eq!(first["known_cost_estimate_usd"], 0.25);
        assert_eq!(first["probe_receipts"][0]["result"]["status"], "failed");
        // Simulate process loss before a result. Keep the intent and account as unknown.
        drop(Probe::start(&run, "fixture", "model").unwrap());
        Probe::start(&run, "fixture", "model")
            .unwrap()
            .finish(known(), true, json!({}))
            .unwrap();
        let worker = json!({"inference":known()});
        let total = run_summary(&run, std::slice::from_ref(&worker)).unwrap();
        assert_eq!(total["inference_attempts"], 4);
        assert_eq!(total["vision_probes"]["inference_attempts"], 3);
        assert_eq!(total["known_cost_estimate_usd"], 0.75);
        assert_eq!(total["unknown_cost_attempts"], 1);
        assert!(total["total_cost_estimate_usd"].is_null());
        assert!(total["billing_usd"].is_null());
        assert_eq!(total, run_summary(&run, &[worker]).unwrap());
        assert!(initialize(&run).is_err()); // no replay can reset the ledger's origin
    }
    #[test]
    fn legacy_history_and_corrupt_or_misattributed_probes_cannot_claim_complete_cost() {
        let dir = tempfile::tempdir().unwrap();
        let run = dir.path().canonicalize().unwrap();
        let legacy = run_summary(&run, &[json!({"inference":{"cost_estimate_usd":0.1}})]).unwrap();
        assert_eq!(legacy["probe_history_complete"], false);
        assert!(legacy["total_cost_estimate_usd"].is_null());
        let probe = Probe::start(&run, "fixture", "model").unwrap();
        let intent = probe.directory.join("intent.json");
        let mut value: Value = serde_json::from_slice(&fs::read(&intent).unwrap()).unwrap();
        value["run_id"] = json!("routed-other");
        fs::write(intent, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(run_summary(&run, &[]).is_err());
    }
    #[test]
    fn native_counters_preserve_unknowns_cache_semantics_and_estimate_provenance() {
        let mut observed = empty();
        observe(
            &json!({"usage":{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":90,
            "cache_creation_input_tokens":20,"private":"SECRET"},"total_cost_usd":0.125,
            "modelUsage":{"auxiliary":{"costUSD":900}}}),
            &Protocol::Claude,
            &mut observed,
        );
        assert_eq!(observed["usage"]["input_tokens"], 10);
        assert_eq!(observed["usage"]["cached_input_tokens"], 90);
        assert!(observed["usage"]["total_tokens"].is_null());
        assert_eq!(observed["cost_estimate_usd"], 0.125);
        assert_eq!(observed["cost_source"], "native_sdk_estimate");
        assert!(!observed.to_string().contains("SECRET"));
        assert!(!observed.to_string().contains("auxiliary"));
        for raw in [
            json!({}),
            json!({"usage":{"input_tokens":-1,"output_tokens":"5"},"total_cost_usd":-2}),
        ] {
            let mut observed = empty();
            observe(&raw, &Protocol::Claude, &mut observed);
            assert!(observed["usage"].is_null());
            assert!(observed["cost_estimate_usd"].is_null());
        }
        let mut codex = empty();
        observe(
            &json!({"usage":{"input_tokens":100,"output_tokens":20,"total_tokens":120,
            "input_tokens_details":{"cached_tokens":90},"output_tokens_details":{"reasoning_tokens":12}}}),
            &Protocol::Codex,
            &mut codex,
        );
        assert_eq!(codex["usage"]["total_tokens"], 120);
        assert_eq!(codex["usage"]["reasoning_output_tokens"], 12);
        assert!(codex["cost_estimate_usd"].is_null());
    }
    #[test]
    fn retries_failures_and_old_receipts_cannot_become_zero_total_cost() {
        let known = json!({"inference":{"usage":{"input_tokens":10},"cost_estimate_usd":0.25}});
        let total = summary(&[
            known.clone(),
            json!({"retry":"inference_transport"}),
            known.clone(),
        ]);
        assert_eq!(total["total_cost_estimate_usd"], 0.5);
        assert_eq!(total["inference_attempts"], 2);
        let partial = summary(&[known, json!({"inference":{"usage":null}})]);
        assert_eq!(partial["known_cost_estimate_usd"], 0.25);
        assert_eq!(partial["unknown_cost_attempts"], 1);
        assert!(partial["total_cost_estimate_usd"].is_null());
        let interrupted = summary(&[
            json!({"inference":{"cost_estimate_usd":0.25}}),
            json!({"turn":2,"model":"timed-out-before-receipt"}),
        ]);
        assert_eq!(interrupted["unknown_cost_attempts"], 1);
        assert!(interrupted["total_cost_estimate_usd"].is_null());
        assert!(summary(&[])["total_cost_estimate_usd"].is_null());
        assert!(
            summary(&[
                json!({"inference":{"cost_estimate_usd":1e308}}),
                json!({"inference":{"cost_estimate_usd":1e308}})
            ])["total_cost_estimate_usd"]
                .is_null()
        );
    }
}
