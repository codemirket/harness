//! Package-specific recipe and private data; setup/update still own installation.
use super::oci::{image_id, inspect};
use crate::{paths::Paths, podman::Engine, util};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

const REGISTRY: &str = include_str!("../../registry/agent-runtime.json");
const BUILD: &str = "org.mirket.agent-build";
const OWNER: &str = "org.mirket.agent-owner";

pub fn definition() -> Result<Value> {
    Ok(serde_json::from_str(REGISTRY)?)
}
pub fn files() -> Vec<(&'static str, &'static [u8])> {
    vec![
        (
            "Dockerfile",
            include_bytes!("../../runtime/agent/Dockerfile"),
        ),
        ("embed.rs", include_bytes!("../../runtime/agent/embed.rs")),
        (
            "native/Cargo.toml",
            include_bytes!("../../runtime/agent/native/Cargo.toml"),
        ),
        (
            "native/Cargo.lock",
            include_bytes!("../../runtime/agent/native/Cargo.lock"),
        ),
        (
            "model.json",
            include_bytes!("../../runtime/agent/model.json"),
        ),
        (
            "native.json",
            include_bytes!("../../runtime/agent/native.json"),
        ),
        (
            "recovery-contract.json",
            include_bytes!("../../runtime/agent/recovery-contract.json"),
        ),
        ("NOTICE.md", include_bytes!("../../runtime/agent/NOTICE.md")),
    ]
}

pub fn recipe() -> String {
    let mut hash = Sha256::new();
    hash.update(REGISTRY.as_bytes());
    for (name, bytes) in files() {
        hash.update((name.len() as u64).to_le_bytes());
        hash.update(name.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    format!("{:x}", hash.finalize())
}
fn root(paths: &Paths) -> PathBuf {
    paths.root.join("packages/data/agent-runtime")
}
fn receipt_path(paths: &Paths) -> PathBuf {
    root(paths)
        .join("receipts")
        .join(format!("{}.json", recipe()))
}
pub fn data(paths: &Paths) -> PathBuf {
    paths.root.join("runtime/projects")
}
fn owner(paths: &Paths) -> String {
    format!(
        "mirket-agent-{}",
        &util::sha256(paths.root.to_string_lossy().as_bytes())[..16]
    )
}
fn engine(paths: &Paths) -> Result<Engine> {
    let engine = Engine::resolve(paths)?;
    ensure!(
        engine.ready()?["host"]["arch"] == "arm64",
        "agent runtime requires native Linux ARM64"
    );
    Ok(engine)
}
fn active(engine: &Engine, paths: &Paths) -> Result<Vec<String>> {
    Ok(engine
        .run(
            &[
                "ps".into(),
                "-a".into(),
                "--filter".into(),
                format!("label={OWNER}={}", owner(paths)),
                "--format".into(),
                "{{.Names}}".into(),
            ],
            Duration::from_secs(30),
        )?
        .lines()
        .map(str::to_owned)
        .collect())
}
fn file_matches(path: &Path, expected: &Value) -> Result<bool> {
    util::reject_symlink_components(path)?;
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() || Some(metadata.len()) != expected["bytes"].as_u64() {
        return Ok(false);
    }
    let mut file = fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 131072];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()) == expected["sha256"])
}
fn model_files(paths: &Paths, stage: &Path, name: &str, contents: &[u8]) -> Result<()> {
    let model: Value = serde_json::from_slice(contents)?;
    let cache = paths.cache().join("agent-models");
    util::ensure_directory(&cache)?;
    let destination = if name == "model.json" {
        stage.join("model")
    } else {
        let base = stage.join("model-cache").join(format!(
            "models--{}",
            model["repository"]
                .as_str()
                .context("model repository")?
                .replace('/', "--")
        ));
        util::ensure_directory(&base.join("refs"))?;
        util::atomic_write(
            &base.join("refs/main"),
            model["revision"]
                .as_str()
                .context("model revision")?
                .as_bytes(),
        )?;
        base.join("snapshots")
            .join(model["revision"].as_str().context("model revision")?)
    };
    util::ensure_directory(&destination)?;
    let client = reqwest::blocking::Client::builder()
        .https_only(true)
        .timeout(Duration::from_secs(600))
        .build()?;
    for (file, expected) in model["files"].as_object().context("model files")? {
        util::safe_relative(Path::new(file))?;
        let digest = expected["sha256"].as_str().context("model digest")?;
        ensure!(
            digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid model digest"
        );
        let limit = expected["bytes"].as_u64().context("model bytes")?;
        ensure!(limit <= 256 * 1024 * 1024, "model file exceeds cap");
        let cached = cache.join(digest);
        if cached.exists() {
            ensure!(
                file_matches(&cached, expected)?,
                "cached model differs from pin"
            );
        } else {
            let mut response = client
                .get(expected["url"].as_str().context("model URL")?)
                .send()?
                .error_for_status()?;
            let mut temporary = tempfile::NamedTempFile::new_in(&cache)?;
            let mut size = 0;
            let mut buffer = [0; 131072];
            loop {
                let n = response.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                size += n as u64;
                ensure!(size <= limit, "model download exceeds pin");
                temporary.write_all(&buffer[..n])?;
            }
            temporary.flush()?;
            ensure!(
                file_matches(temporary.path(), expected)?,
                "downloaded model differs from pin"
            );
            temporary
                .persist_noclobber(&cached)
                .context("publish model cache")?;
        }
        let target = destination.join(file);
        fs::copy(cached, &target)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target, fs::Permissions::from_mode(0o644))?;
        }
    }
    Ok(())
}
fn native_files(paths: &Paths, stage: &Path) -> Result<()> {
    let pin: Value = serde_json::from_str(include_str!("../../runtime/agent/native.json"))?;
    let cache = paths.cache().join("agent-native");
    util::ensure_directory(&cache)?;
    let file = cache.join(pin["sha256"].as_str().context("native digest")?);
    if !file.exists() {
        let client = reqwest::blocking::Client::builder()
            .https_only(true)
            .timeout(Duration::from_secs(300))
            .build()?;
        let mut response = client
            .get(pin["url"].as_str().context("native URL")?)
            .send()?
            .error_for_status()?;
        let mut temp = tempfile::NamedTempFile::new_in(&cache)?;
        let cap = pin["bytes"].as_u64().context("native size")?;
        let copied = std::io::copy(&mut (&mut response).take(cap + 1), &mut temp)?;
        ensure!(
            copied == cap && file_matches(temp.path(), &pin)?,
            "native artifact differs from pin"
        );
        temp.persist_noclobber(&file)
            .context("publish native cache")?;
    }
    ensure!(file_matches(&file, &pin)?, "native cache drift");
    let mut archive = zip::ZipArchive::new(fs::File::open(file)?)?;
    for (target, pin) in pin["files"].as_object().context("native members")? {
        util::safe_relative(Path::new(target))?;
        let member = pin["member"].as_str().context("native member")?;
        let entry = archive.by_name(member)?;
        let cap = pin["bytes"].as_u64().context("native member size")?;
        ensure!(
            entry.is_file() && entry.size() == cap,
            "native member differs from pin"
        );
        let mut bytes = Vec::new();
        entry.take(cap + 1).read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() as u64 == cap && util::sha256(&bytes) == pin["sha256"],
            "native member digest drift"
        );
        util::atomic_write_with_mode(&stage.join("native-libs").join(target), &bytes, Some(0o644))?;
    }
    Ok(())
}

pub fn prepare(paths: &Paths) -> Result<()> {
    let engine = engine(paths)?;
    util::ensure_directory(&root(paths))?;
    let _lock = util::FileLock::acquire(&root(paths).join("prepare.lock"))?;
    for name in active(&engine, paths)? {
        ensure!(
            inspect(&engine, "container", &name)?["Config"]["Labels"][BUILD] == recipe(),
            "stop active agent-runtime sessions before changing their generation"
        );
    }
    if receipt_path(paths).exists() {
        verify(paths)?;
        return Ok(());
    }
    let definition = definition()?;
    for key in ["build_image", "runtime_image"] {
        let base = definition[key].as_str().context("base image")?;
        if image_id(&engine, base).is_err() {
            engine.run(
                &["pull".into(), "--platform=linux/arm64".into(), base.into()],
                Duration::from_secs(900),
            )?;
        }
        image_id(&engine, base)?;
    }
    let tag = format!("localhost/mirket-agent-runtime:{}", &recipe()[..16]);
    if image_id(&engine, &tag).is_err() {
        let parent = engine.staging.join("agent-runtime/build");
        util::ensure_directory(&parent)?;
        let stage = tempfile::tempdir_in(parent)?;
        for (name, bytes) in files() {
            util::atomic_write(&stage.path().join(name), bytes)?;
            if name == "model.json" {
                model_files(paths, stage.path(), name, bytes)?;
            }
        }
        native_files(paths, stage.path())?;
        engine.run(
            &[
                "build".into(),
                "--pull=never".into(),
                "--platform=linux/arm64".into(),
                "--memory=4g".into(),
                "--memory-swap=4g".into(),
                "--label".into(),
                format!("{BUILD}={}", recipe()),
                "--tag".into(),
                tag.clone(),
                stage.path().to_string_lossy().into_owned(),
            ],
            Duration::from_secs(1800),
        )?;
    }
    ensure!(
        inspect(&engine, "image", &tag)?["Labels"][BUILD] == recipe(),
        "agent image recipe drift"
    );
    let image = image_id(&engine, &tag)?;
    util::ensure_directory(&data(paths))?;
    util::ensure_directory(receipt_path(paths).parent().context("receipt parent")?)?;
    util::atomic_write_with_mode(
        &receipt_path(paths),
        &serde_json::to_vec_pretty(
            &json!({"schema_version":1,"owner":owner(paths),"recipe":recipe(),"image":image}),
        )?,
        Some(0o600),
    )
}
pub fn verify(paths: &Paths) -> Result<Value> {
    let receipt: Value = serde_json::from_slice(&util::read_bounded(&receipt_path(paths), 4096).context("agent-runtime is not installed for this release; run mirket package install design agent-runtime")?)?;
    ensure!(
        receipt["schema_version"] == 1
            && receipt["owner"] == owner(paths)
            && receipt["recipe"] == recipe(),
        "agent runtime receipt drift"
    );
    let engine = engine(paths)?;
    let image = receipt["image"].as_str().context("agent image missing")?;
    ensure!(
        image_id(&engine, image)? == image
            && inspect(&engine, "image", image)?["Labels"][BUILD] == recipe(),
        "agent runtime image drift"
    );
    util::reject_symlink_components(&data(paths))?;
    ensure!(data(paths).is_dir(), "agent runtime data directory missing");
    Ok(
        json!({"ok":true,"ready":true,"package":"agent-runtime","receipt":receipt,"lifecycle":"on_demand","data":data(paths),"network":"none"}),
    )
}
pub fn command(paths: &Paths, scope: Option<&str>) -> Result<(Engine, Vec<String>, String)> {
    let verified = verify(paths)?;
    let engine = engine(paths)?;
    let name = format!(
        "{}-{}",
        owner(paths),
        &util::sha256(
            format!("{}-{:?}", std::process::id(), std::time::SystemTime::now()).as_bytes()
        )[..16]
    );
    let mut args = vec![
        "run".into(),
        "--rm".into(),
        "-i".into(),
        "--pull=never".into(),
        "--name".into(),
        name.clone(),
        "--network=none".into(),
        "--userns=keep-id:uid=1000,gid=1000".into(),
        "--user=1000:1000".into(),
        "--read-only".into(),
        "--cap-drop=ALL".into(),
        "--security-opt=no-new-privileges".into(),
        "--memory=1536m".into(),
        "--memory-swap=1536m".into(),
        "--cpus=2".into(),
        "--pids-limit=128".into(),
        "--http-proxy=false".into(),
        "--log-driver=none".into(),
        "--tmpfs=/tmp:rw,nosuid,nodev,noexec,size=256m,mode=1777".into(),
        "--label".into(),
        format!("{OWNER}={}", owner(paths)),
        "--label".into(),
        format!("{BUILD}={}", recipe()),
    ];
    if let Some(scope) = scope {
        ensure!(
            scope.len() == 64 && scope.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid runtime scope"
        );
        // Scope binds the host lease only. The embedding process never mounts facts or workflows.
        args.extend(["--label".into(), format!("org.mirket.project={scope}")]);
    }
    args.push(
        verified["receipt"]["image"]
            .as_str()
            .context("agent image")?
            .into(),
    );
    Ok((engine, args, name))
}
pub fn stop(paths: &Paths, purge: bool) -> Result<Value> {
    ensure!(
        !purge,
        "agent runtime data purge requires separately scoped memory/workflow deletion; no data removed"
    );
    let engine = engine(paths)?;
    for name in active(&engine, paths)? {
        engine.run(
            &["rm".into(), "--force".into(), name],
            Duration::from_secs(30),
        )?;
    }
    Ok(json!({"package":"agent-runtime","stopped":true,"data_retained":true}))
}
pub fn owned_session(engine: &Engine, name: &str) -> Result<bool> {
    let value = inspect(engine, "container", name)?;
    let owner = value["Config"]["Labels"][OWNER].as_str().unwrap_or("");
    Ok(owner.starts_with("mirket-agent-")
        && name.starts_with(&format!("{owner}-"))
        && value["Config"]["Labels"][BUILD] == recipe())
}

/// Actual immutable helper under production containment, with no user state mount.
pub fn test(paths: &Paths) -> Result<Value> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let mut helper=crate::agent_runtime::EmbeddingSession::open(paths,None).await?;
        let response=helper.call(None,"embed",json!({"texts":["Türkçe yanıtları tercih ederim.","Türkçe yanıtları tercih ederim.","The database uses SQLite."]})).await?;
        let vectors:Vec<Vec<f32>>=serde_json::from_value(response["result"]["vectors"].clone())?;
        ensure!(vectors.len()==3 && vectors.iter().all(|v|v.len()==384 && v.iter().all(|x|x.is_finite())),"invalid contained vectors");
        ensure!(vectors[0]==vectors[1],"same text produced different vectors");
        let cosine:f64=vectors[0].iter().zip(&vectors[2]).map(|(a,b)|f64::from(*a)*f64::from(*b)).sum();
        ensure!(cosine<0.95,"embedding discriminating fixture collapsed");
        ensure!(response["result"]["resources"]["resident_bytes"].as_u64().is_some_and(|v|v>0),"contained helper RSS unavailable");
        ensure!(response["transport"]["serialization_ms"].is_number(),"serialization timing missing");
        helper.close().await?;
        Ok(json!({"ok":true,"package":"agent-runtime","recipe":recipe(),"runtime":response["result"]["identity"],"tests":"real contained offline embedding: multilingual input, repeat determinism, distinct-text discrimination, framing and EOF","timing_ms":response["result"]["timing_ms"],"transport":response["transport"],"resources":response["result"]["resources"]}))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipe_covers_runtime_models_lock_and_notices() {
        let files = files();
        for name in [
            "embed.rs",
            "native/Cargo.toml",
            "native/Cargo.lock",
            "model.json",
            "native.json",
            "recovery-contract.json",
            "NOTICE.md",
        ] {
            assert!(
                files
                    .iter()
                    .any(|(path, bytes)| *path == name && !bytes.is_empty())
            );
        }
        assert_eq!(recipe().len(), 64);
        assert_eq!(definition().unwrap()["host_connection"], false);
    }

    #[tokio::test]
    #[ignore = "explicit prepared engine; real multilingual retrieval corpus"]
    async fn contained_native_retrieval_corpus() {
        let home = std::env::var_os("MIRKET_TEST_ENGINE_HOME").expect("explicit engine home");
        let paths = Paths::new(PathBuf::from(home).canonicalize().unwrap()).unwrap();
        let prepared = paths.clone();
        tokio::task::spawn_blocking(move || prepare(&prepared))
            .await
            .unwrap()
            .unwrap();
        let corpus: Value =
            serde_json::from_str(include_str!("../../tests/fixtures/stability/corpus.json"))
                .unwrap();
        let facts = corpus["facts"].as_array().unwrap();
        let queries = corpus["queries"].as_array().unwrap();
        let texts: Vec<_> = facts
            .iter()
            .map(|f| f["text"].clone())
            .chain(queries.iter().map(|q| q["query"].clone()))
            .collect();
        let mut helper = crate::agent_runtime::EmbeddingSession::open(&paths, None)
            .await
            .unwrap();
        let response = helper
            .call(None, "embed", json!({"texts":texts}))
            .await
            .unwrap();
        let vectors: Vec<Vec<f32>> =
            serde_json::from_value(response["result"]["vectors"].clone()).unwrap();
        if let Ok(baseline_image) = std::env::var("MIRKET_BASELINE_AGENT_IMAGE") {
            assert!(
                baseline_image.starts_with("sha256:")
                    && baseline_image.len() == 71
                    && baseline_image[7..].bytes().all(|b| b.is_ascii_hexdigit())
            );
            let baseline_paths = paths.clone();
            let fixture = serde_json::to_string(&texts).unwrap();
            let script = format!(
                "import json,hashlib; from memory import embedding_model; texts=json.loads({}); model=embedding_model(); print(json.dumps({{'vectors':[next(model.embed([text])).tolist() for text in texts], 'manifest':hashlib.sha256(open('/opt/agent/model.json','rb').read()).hexdigest()}}))",
                serde_json::to_string(&fixture).unwrap()
            );
            let baseline = tokio::task::spawn_blocking(move || {
                let engine = engine(&baseline_paths).unwrap();
                let args = vec![
                    "run".into(),
                    "--rm".into(),
                    "--network=none".into(),
                    "--read-only".into(),
                    "--cap-drop=ALL".into(),
                    "--security-opt=no-new-privileges".into(),
                    "--memory=1536m".into(),
                    "--cpus=2".into(),
                    "--pids-limit=128".into(),
                    "--tmpfs=/tmp:rw,nosuid,nodev,noexec,size=256m,mode=1777".into(),
                    "--entrypoint=python".into(),
                    baseline_image,
                    "-B".into(),
                    "-c".into(),
                    script,
                ];
                let output = engine.run(&args, Duration::from_secs(120)).unwrap();
                serde_json::from_str::<Value>(&output).unwrap()
            })
            .await
            .unwrap();
            assert_eq!(
                baseline["manifest"],
                util::sha256(include_bytes!("../../runtime/agent/model.json"))
            );
            let expected: Vec<Vec<f32>> =
                serde_json::from_value(baseline["vectors"].clone()).unwrap();
            assert_eq!(expected.len(), vectors.len());
            for (old, new) in expected.iter().zip(&vectors) {
                assert_eq!(old.len(), new.len());
                // The prior Qdrant cosine collection normalized Python FastEmbed
                // vectors at insertion/query. Compare the effective unit vectors,
                // since fastembed-rs performs that normalization itself.
                let old_norm = old
                    .iter()
                    .map(|x| f64::from(*x).powi(2))
                    .sum::<f64>()
                    .sqrt();
                let new_norm = new
                    .iter()
                    .map(|x| f64::from(*x).powi(2))
                    .sum::<f64>()
                    .sqrt();
                assert!(
                    old_norm.is_finite() && old_norm > 0. && new_norm.is_finite() && new_norm > 0.
                );
                let max_error = old
                    .iter()
                    .zip(new)
                    .map(|(a, b)| {
                        (f64::from(*a) / old_norm - f64::from(*b) / new_norm).abs() as f32
                    })
                    .fold(0f32, f32::max);
                let cosine: f64 = old
                    .iter()
                    .zip(new)
                    .map(|(a, b)| f64::from(*a) * f64::from(*b) / (old_norm * new_norm))
                    .sum();
                assert!(
                    max_error <= 0.001 && cosine >= 0.9999,
                    "embedding parity failed: max_error={max_error}, cosine={cosine}"
                );
            }
        }

        let overflowing = "bounded token fixture ".repeat(100);
        assert!(
            helper
                .call(None, "embed", json!({"texts":[overflowing]}))
                .await
                .is_err()
        );
        let fitted = helper
            .call(
                None,
                "embed",
                json!({"texts":[overflowing],"fit_auto_queries":true}),
            )
            .await
            .unwrap();
        assert_eq!(fitted["result"]["queries_shortened"], true);
        assert!(
            fitted["result"]["effective_texts"][0]
                .as_str()
                .unwrap()
                .len()
                < overflowing.len()
        );
        helper.close().await.unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let mut store = crate::memory::store::FactStore::open(
            &tmp.path().canonicalize().unwrap().join("facts.sqlite"),
            &"a".repeat(64),
            "fixture",
        )
        .unwrap();
        let mut ids = std::collections::BTreeMap::new();
        for (i, fact) in facts.iter().enumerate() {
            ids.insert(
                fact["key"].as_str().unwrap(),
                store.put(fact, &vectors[i], 100).unwrap()["memory"]["id"].clone(),
            );
        }
        for (i, query) in queries.iter().enumerate() {
            let found = store
                .search(
                    query["query"].as_str().unwrap(),
                    Some(&vectors[facts.len() + i]),
                    5,
                    101,
                )
                .unwrap();
            let cosine: Vec<f64> = vectors[..facts.len()]
                .iter()
                .map(|v| {
                    v.iter()
                        .zip(&vectors[facts.len() + i])
                        .map(|(a, b)| f64::from(*a) * f64::from(*b))
                        .sum()
                })
                .collect();
            eprintln!(
                "corpus {} cosine={cosine:?} results={}",
                query["id"], found["results"]
            );
            let expected = query["expected_keys"].as_array().unwrap();
            if expected.is_empty() {
                assert!(
                    found["results"].as_array().unwrap().is_empty(),
                    "unanswerable retrieval must abstain"
                );
            } else {
                assert_eq!(
                    found["results"][0]["id"],
                    ids[expected[0].as_str().unwrap()],
                    "{}",
                    query["id"]
                );
            }
        }
    }

    #[test]
    #[ignore = "explicit registered engine; builds image and runs real offline fixtures"]
    fn contained_agent_runtime_on_registered_engine() {
        let home = std::env::var_os("MIRKET_TEST_ENGINE_HOME").expect("explicit test engine home");
        let paths = Paths::new(PathBuf::from(home).canonicalize().unwrap()).unwrap();
        prepare(&paths).unwrap();
        assert_eq!(test(&paths).unwrap()["ok"], true);
    }
}
