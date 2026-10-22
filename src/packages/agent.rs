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
            "licenses/mem0.txt",
            include_bytes!("../../runtime/agent/licenses/mem0.txt"),
        ),
        (
            "licenses/langgraph.txt",
            include_bytes!("../../runtime/agent/licenses/langgraph.txt"),
        ),
        (
            "Dockerfile",
            include_bytes!("../../runtime/agent/Dockerfile"),
        ),
        (
            "requirements.lock",
            include_bytes!("../../runtime/agent/requirements.lock"),
        ),
        ("bridge.py", include_bytes!("../../runtime/agent/bridge.py")),
        ("memory.py", include_bytes!("../../runtime/agent/memory.py")),
        (
            "lifecycle.py",
            include_bytes!("../../runtime/agent/lifecycle.py"),
        ),
        (
            "workflow.py",
            include_bytes!("../../runtime/agent/workflow.py"),
        ),
        (
            "model.json",
            include_bytes!("../../runtime/agent/model.json"),
        ),
        (
            "recovery-contract.json",
            include_bytes!("../../runtime/agent/recovery-contract.json"),
        ),
        ("bm25.json", include_bytes!("../../runtime/agent/bm25.json")),
        (
            "tests/test_runtime.py",
            include_bytes!("../../runtime/agent/test_runtime.py"),
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
    paths
        .root
        .join("execution/podman/staging/agent-runtime/data")
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
        fs::copy(cached, destination.join(file))?;
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
    let base = definition()?["python_image"]
        .as_str()
        .context("base image")?
        .to_owned();
    if image_id(&engine, &base).is_err() {
        engine.run(
            &["pull".into(), "--platform=linux/arm64".into(), base.clone()],
            Duration::from_secs(900),
        )?;
    }
    image_id(&engine, &base)?;
    let tag = format!("localhost/mirket-agent-runtime:{}", &recipe()[..16]);
    if image_id(&engine, &tag).is_err() {
        let parent = engine.staging.join("agent-runtime/build");
        util::ensure_directory(&parent)?;
        let stage = tempfile::tempdir_in(parent)?;
        for (name, bytes) in files() {
            util::atomic_write(&stage.path().join(name), bytes)?;
            if ["model.json", "bm25.json"].contains(&name) {
                model_files(paths, stage.path(), name, bytes)?;
            }
        }
        engine.run(
            &[
                "build".into(),
                "--pull=never".into(),
                "--platform=linux/arm64".into(),
                "--memory=2g".into(),
                "--memory-swap=2g".into(),
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
        let directory = data(paths).join(scope);
        util::ensure_directory(&directory)?;
        args.extend([
            "--volume".into(),
            format!("{}:/data/{scope}:rw", directory.display()),
        ]);
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

/// Real dependencies and restart/failure fixtures in the production containment.
pub fn test(paths: &Paths) -> Result<Value> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let identity = runtime.block_on(crate::agent_runtime::call(
        paths,
        None,
        "identity",
        json!({}),
    ))?;
    let (engine, mut args, _) = command(paths, None)?;
    let image = args.pop().context("test image")?;
    // Fixture facts live only in the container tmpfs; no installed memory mount.
    args.extend([
        "--env=MIRKET_BRIDGE=/opt/agent/bridge.py".into(),
        "--env=MIRKET_CONTAINED=1".into(),
        "--entrypoint=python".into(),
        image,
        "-B".into(),
        "/opt/agent/tests/test_runtime.py".into(),
    ]);
    let output = engine.run(&args, Duration::from_secs(300))?;
    Ok(
        json!({"ok":true,"package":"agent-runtime","recipe":recipe(),"runtime":identity["runtime"],"tests":"Rust IPC identity/EOF; real Mem0 retrieval, scope, retries, expiry, forget; LangGraph restart, stale checkpoint, effect and host handoffs; containment","output":output}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipe_covers_runtime_models_lock_and_notices() {
        let files = files();
        for name in [
            "bridge.py",
            "memory.py",
            "workflow.py",
            "requirements.lock",
            "model.json",
            "bm25.json",
            "licenses/mem0.txt",
            "licenses/langgraph.txt",
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

    #[test]
    #[ignore = "explicit registered engine; builds image and runs real offline fixtures"]
    fn contained_agent_runtime_on_registered_engine() {
        let home = std::env::var_os("MIRKET_TEST_ENGINE_HOME").expect("explicit test engine home");
        let paths = Paths::new(PathBuf::from(home).canonicalize().unwrap()).unwrap();
        prepare(&paths).unwrap();
        assert_eq!(test(&paths).unwrap()["ok"], true);
    }
}
