//! Narrow, offline embedding process. Facts, tools and credentials stay with Rust's host owner.
use anyhow::{Context, Result, ensure};
use fastembed::{
    InitOptionsUserDefined, Pooling, QuantizationMode, TextEmbedding, TokenizerFiles,
    UserDefinedEmbeddingModel,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, BufRead, Read, Write},
    path::Path,
    time::Instant,
};

const MAX_REQUEST: u64 = 64 * 1024;
const MODEL: &str = include_str!("model.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    operation: String,
    #[serde(default)]
    texts: Vec<String>,
    #[serde(default)]
    fit_auto_queries: bool,
}

fn identity() -> Value {
    let model: Value = serde_json::from_str(MODEL).expect("embedded model manifest");
    json!({"protocol":1,"version":env!("CARGO_PKG_VERSION"),"engine":"fastembed-rs",
        "fastembed":"7.2.0","model":model["model"],"revision":model["revision"],
        "model_manifest_sha256":format!("{:x}", Sha256::digest(MODEL.as_bytes())),
        "dimensions":384,"max_tokens":128})
}

fn asset(root: &Path, name: &str) -> Result<Vec<u8>> {
    let manifest: Value = serde_json::from_str(MODEL)?;
    let pin = &manifest["files"][name];
    let path = root.join(name);
    let metadata = fs::symlink_metadata(&path)?;
    ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "model asset is not a regular file"
    );
    ensure!(
        Some(metadata.len()) == pin["bytes"].as_u64(),
        "model asset size drift"
    );
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    fs::File::open(path)?
        .take(metadata.len() + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 == metadata.len(),
        "model asset changed while reading"
    );
    ensure!(
        json!(format!("{:x}", Sha256::digest(&bytes))) == pin["sha256"],
        "model asset digest drift"
    );
    Ok(bytes)
}

fn load(root: &Path) -> Result<TextEmbedding> {
    let environment = ort::init_from("/opt/native/libonnxruntime.so")?
        .with_telemetry(false)
        .build()?;
    let tokenizer = TokenizerFiles {
        tokenizer_file: asset(root, "tokenizer.json")?,
        config_file: asset(root, "config.json")?,
        special_tokens_map_file: asset(root, "special_tokens_map.json")?,
        tokenizer_config_file: asset(root, "tokenizer_config.json")?,
    };
    let model = UserDefinedEmbeddingModel::new(asset(root, "model_optimized.onnx")?, tokenizer)
        .with_pooling(Pooling::Mean)
        .with_quantization(QuantizationMode::Dynamic);
    TextEmbedding::try_new_from_user_defined(
        model,
        InitOptionsUserDefined::new()
            .with_environment(environment)
            .with_max_length(128)
            .with_intra_threads(2),
    )
    .context("initialize pinned local embedding model")
}

fn validate(request: &Request) -> Result<()> {
    ensure!(
        request.operation == "embed" || request.operation == "identity",
        "unsupported operation"
    );
    if request.operation == "identity" {
        ensure!(
            request.texts.is_empty() && !request.fit_auto_queries,
            "identity does not accept text"
        );
    } else {
        ensure!(
            (1..=8).contains(&request.texts.len()),
            "embedding batch must contain 1..8 texts"
        );
        ensure!(
            request
                .texts
                .iter()
                .all(|s| !s.trim().is_empty() && s.len() <= 8192),
            "invalid embedding text"
        );
    }
    Ok(())
}

fn resident_bytes() -> Option<u64> {
    // The shipped helper runs on Linux. No subprocess or model/fact data is read.
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("VmRSS:"))?;
    line.split_whitespace()
        .nth(1)?
        .parse::<u64>()
        .ok()?
        .checked_mul(1024)
}

fn execute(mut request: Request, model: &mut Option<TextEmbedding>, root: &Path) -> Result<Value> {
    validate(&request)?;
    if request.operation == "identity" {
        return Ok(json!({"identity":identity()}));
    }
    let load_started = Instant::now();
    let cold = model.is_none();
    if cold {
        *model = Some(load(root)?);
    }
    let load_ms = if cold {
        load_started.elapsed().as_secs_f64() * 1000.0
    } else {
        0.0
    };
    let model = model.as_mut().context("embedding model unavailable")?;
    let mut shortened = false;
    for text in &mut request.texts {
        loop {
            let encoded = model
                .tokenizer
                .encode(text.as_str(), true)
                .map_err(|_| anyhow::anyhow!("tokenization failed"))?;
            if encoded.get_overflowing().is_empty() && encoded.len() <= 128 {
                break;
            }
            ensure!(
                request.fit_auto_queries,
                "text exceeds 128-token embedding window"
            );
            let count = text.chars().count();
            ensure!(count > 1, "automatic query cannot fit embedding window");
            *text = text
                .chars()
                .take(count.saturating_sub((count / 8).max(1)))
                .collect();
            shortened = true;
        }
    }
    let effective_texts = request.texts.clone();
    let inference_started = Instant::now();
    // Quantized kernels can depend on the batch. Keep each sentence independent
    // so warm reuse and multi-query admission cannot alter retrieval vectors.
    let mut vectors = Vec::with_capacity(request.texts.len());
    for text in request.texts {
        let mut output = model.embed(vec![text], None)?;
        ensure!(output.len() == 1, "embedding output count mismatch");
        let vector = output.remove(0);
        ensure!(
            vector.len() == 384
                && vector.iter().all(|v| v.is_finite())
                && vector.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>() > 0.,
            "invalid embedding output"
        );
        vectors.push(vector);
    }
    Ok(
        json!({"vectors":vectors,"effective_texts":effective_texts,"queries_shortened":shortened,"identity":identity(),"resources":{"resident_bytes":resident_bytes(),"scope":"embedding helper process, sampled after inference"},"timing_ms":{
        "model_load":load_ms,"inference":inference_started.elapsed().as_secs_f64()*1000.0}}),
    )
}

fn run() -> Result<()> {
    let root = Path::new("/opt/models");
    let mut model = None;
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let mut bytes = Vec::new();
        let length = (&mut input)
            .take(MAX_REQUEST + 1)
            .read_until(b'\n', &mut bytes)?;
        if length == 0 {
            break;
        }
        // Close on framing failure; never parse a truncated prefix as another request.
        ensure!(
            length <= MAX_REQUEST as usize && bytes.last() == Some(&b'\n'),
            "invalid request frame"
        );
        let result = serde_json::from_slice::<Request>(&bytes)
            .map_err(anyhow::Error::from)
            .and_then(|r| execute(r, &mut model, root));
        let response = match result {
            Ok(value) => json!({"result":value}),
            // Upstream errors may quote private text or filesystem paths.
            Err(_) => {
                json!({"error":"embedding request rejected; inspect model identity and input bounds"})
            }
        };
        serde_json::to_writer(&mut output, &response)?;
        output.write_all(b"\n")?;
        output.flush()?;
    }
    Ok(())
}

fn main() {
    if run().is_err() {
        eprintln!("embedding helper stopped after protocol or runtime failure");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_does_not_load_model_or_read_assets() {
        let mut model = None;
        let result = execute(
            Request {
                fit_auto_queries: false,
                operation: "identity".into(),
                texts: vec![],
            },
            &mut model,
            Path::new("/missing"),
        )
        .unwrap();
        assert_eq!(result["identity"]["dimensions"], 384);
        assert!(model.is_none());
    }

    #[test]
    fn reject_unknown_fields_operations_and_oversized_batches_before_model_load() {
        assert!(
            serde_json::from_value::<Request>(json!({"operation":"embed","tool":"shell"})).is_err()
        );
        for request in [
            Request {
                fit_auto_queries: false,
                operation: "shell".into(),
                texts: vec![],
            },
            Request {
                fit_auto_queries: false,
                operation: "embed".into(),
                texts: vec!["fact".into(); 9],
            },
            Request {
                fit_auto_queries: false,
                operation: "embed".into(),
                texts: vec![" ".into()],
            },
        ] {
            let mut model = None;
            assert!(execute(request, &mut model, Path::new("/missing")).is_err());
            assert!(model.is_none());
        }
    }
}
