//! Pinned Linux images built from the same verified package generations.
use super::Registry;
use crate::{paths::Paths, podman::Engine, util};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

const JOB_RUNNER: &str = include_str!("../../runtime/packages/container/job.mjs");
// Chrome 155/Node 24.21 observed loads plus reviewed NSS/font dependencies.
// The pinned donor fixes every alias/version matched here; no system tools,
// Mesa, donor LLVM, GStreamer, Perl or desktop GUI library tree is copied.
const LIBRARY_FAMILIES: &str = "libX11.so* libXau.so* libXcomposite.so* libXdamage.so* libXdmcp.so* libXext.so* libXfixes.so* libXi.so* libXrandr.so* libXrender.so* libasound.so* libatk-1.0.so* libatspi.so* libblkid.so* libbrotlicommon.so* libbrotlidec.so* libbsd.so* libbz2.so* libc.so* libcap.so* libdbus-1.so* libdl.so* libdrm.so* libexpat.so* libffi.so* libfontconfig.so* libfreebl3.so* libfreeblpriv3.so* libfreetype.so* libgbm.so* libgcc_s.so* libgcrypt.so* libgio-2.0.so* libglib-2.0.so* libgmodule-2.0.so* libgobject-2.0.so* libgpg-error.so* libgraphite2.so* libharfbuzz.so* liblz4.so* liblzma.so* libm.so* libmd.so* libmount.so* libnspr4.so* libnss3.so* libnss_dns.so* libnss_files.so* libnssckbi.so* libnssutil3.so* libpcre2-8.so* libplc4.so* libplds4.so* libpng16.so* libpthread.so* libresolv.so* libselinux.so* libsmime3.so* libsoftokn3.so* libssl3.so* libstdc++.so* libsystemd.so* libudev.so* libuuid.so* libva-drm.so* libva.so* libwayland-client.so* libxcb-shm.so* libxcb.so* libxkbcommon.so* libz.so* libzstd.so*";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub verified_hosts: Vec<String>,
    pub image_base: BTreeMap<String, String>,
    pub cpu: u32,
    pub memory_mib: u32,
    pub pids: u32,
    pub network: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ImageIdentity {
    pub id: String,
    pub recipe: String,
}
#[derive(Clone, Debug)]
pub struct Runtime {
    pub engine: Engine,
    pub id: String,
    pub image: String,
    pub profile: Profile,
    pub gate: Arc<tokio::sync::Semaphore>,
    worker: Arc<Mutex<Option<Arc<Worker>>>>,
}
pub(super) fn append_identity(bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(util::sha256(include_bytes!("container.rs")).as_bytes());
    bytes.extend_from_slice(util::sha256(JOB_RUNNER.as_bytes()).as_bytes());
}
pub(super) fn target() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "aarch64-unknown-linux-gnu"
    } else {
        "x86_64-unknown-linux-gnu"
    }
}
pub(super) fn validate(profile: &Profile, id: &str) -> Result<()> {
    ensure!(
        profile.verified_hosts == ["aarch64-apple-darwin"],
        "only the verified macOS ARM64 controller may activate this OCI profile"
    );
    ensure!(
        matches!(id, "motion-canvas" | "hyperframes"),
        "container profile is not reviewed for this package"
    );
    ensure!(
        (1..=4).contains(&profile.cpu)
            && (512..=4096).contains(&profile.memory_mib)
            && (64..=512).contains(&profile.pids)
            && !profile.network,
        "invalid offline design container limits"
    );
    for (target, image) in &profile.image_base {
        let expected = match target.as_str() {
            "aarch64-unknown-linux-gnu" => {
                "mcr.microsoft.com/playwright@sha256:824f1a789072e648c62541c2cfa4479c4061a290d5c27766d67dc1dcbc19b321"
            }
            "x86_64-unknown-linux-gnu" => {
                "mcr.microsoft.com/playwright@sha256:cf0daee9b994042e011bc29f20cdff1a9f682a039b43fcd738f7d8a9d3bcd9d6"
            }
            _ => anyhow::bail!("unreviewed OCI platform"),
        };
        ensure!(
            image == expected,
            "OCI base differs from reviewed manifest pin"
        );
    }
    ensure!(
        !profile.image_base.is_empty(),
        "container profile has no pinned base"
    );
    Ok(())
}
fn image_id(id: &str) -> Result<()> {
    let digest = id
        .strip_prefix("sha256:")
        .context("OCI image requires immutable SHA-256 ID")?;
    ensure!(
        digest.len() == 64
            && digest
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
        "invalid OCI image ID"
    );
    Ok(())
}
pub(super) fn verify(paths: &Paths, identity: &ImageIdentity) -> Result<()> {
    image_id(&identity.id)?;
    let engine = Engine::resolve(paths)?;
    engine.ready()?;
    let inspected: Value = serde_json::from_str(&engine.run(
        &[
            "image".into(),
            "inspect".into(),
            identity.id.clone(),
            "--format={{json .}}".into(),
        ],
        Duration::from_secs(8),
    )?)?;
    verified_image(&inspected, identity)
}
fn verified_image(inspected: &Value, identity: &ImageIdentity) -> Result<()> {
    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "amd64"
    };
    ensure!(
        inspected["Architecture"] == arch && inspected["Os"] == "linux",
        "OCI image architecture or OS differs from native execution target"
    );
    ensure!(
        inspected["Id"].as_str().is_some_and(
            |id| id.trim_start_matches("sha256:") == identity.id.trim_start_matches("sha256:")
        ) && inspected["Config"]["Labels"]["org.mirket.recipe"] == identity.recipe,
        "OCI image/config has drifted"
    );
    Ok(())
}
fn copy_tree(
    source: &Path,
    target: &Path,
    total: &mut u64,
    limit: u64,
    count: &mut usize,
    depth: usize,
    check: &impl Fn() -> Result<()>,
) -> Result<()> {
    use std::io::{Read, Write};
    check()?;
    ensure!(depth <= 32, "container staging is too deeply nested");
    util::reject_symlink_components(source)?;
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        check()?;
        let entry = entry?;
        *count += 1;
        ensure!(*count <= 65536, "container staging has too many entries");
        let meta = entry.file_type()?;
        let dest = target.join(entry.file_name());
        ensure!(!meta.is_symlink(), "container staging refuses links");
        if meta.is_dir() {
            copy_tree(&entry.path(), &dest, total, limit, count, depth + 1, check)?;
        } else {
            ensure!(meta.is_file(), "container staging refuses special files");
            let mut input = fs::File::open(entry.path())?;
            ensure!(input.metadata()?.is_file(), "container input changed type");
            let mut output = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&dest)?;
            let mut buffer = [0u8; 65536];
            loop {
                check()?;
                let n = input.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                *total = total
                    .checked_add(n as u64)
                    .context("staging size overflow")?;
                ensure!(
                    *total <= limit,
                    "container staging exceeds its {} byte input bound",
                    limit
                );
                output.write_all(&buffer[..n])?;
            }
            fs::set_permissions(dest, input.metadata()?.permissions())?;
        }
    }
    Ok(())
}
pub(super) fn build(
    paths: &Paths,
    registry: &Registry,
    id: &str,
    target: &str,
    root: &Path,
) -> Result<ImageIdentity> {
    let package = registry.get(id)?;
    let profile = package
        .container
        .as_ref()
        .context("container profile missing")?;
    let base = profile
        .image_base
        .get(target)
        .context("OCI target is held")?;
    let engine = Engine::resolve(paths)?;
    engine.ready()?;
    let context = tempfile::Builder::new()
        .prefix("build-")
        .tempdir_in(&engine.staging)?;
    let mut total = 0;
    let ids = registry.order(&[id.into()])?;
    for dependency in &ids {
        let source = if dependency == id {
            root.to_owned()
        } else {
            registry.root_for(paths, dependency, target)?
        };
        copy_tree(
            &source,
            &context.path().join(dependency),
            &mut total,
            2 * 1024 * 1024 * 1024,
            &mut 0,
            0,
            &|| Ok(()),
        )?;
    }
    fs::write(context.path().join("job.mjs"), JOB_RUNNER)?;
    fs::write(
        context.path().join("passwd"),
        "root:x:0:0:root:/nonexistent:/sbin/nologin\nmirket:x:1000:1000:Mirket:/tmp/home:/sbin/nologin\n",
    )?;
    fs::write(context.path().join("group"), "root:x:0:\nmirket:x:1000:\n")?;
    let arch = if target.starts_with("aarch64") {
        "aarch64"
    } else {
        "x86_64"
    };
    let loader = if arch == "aarch64" {
        "ld-linux-aarch64.so.1"
    } else {
        "ld-linux-x86-64.so.2"
    };
    let loader_dest = if arch == "aarch64" { "lib" } else { "lib64" };
    let mut libraries = LIBRARY_FAMILIES
        .split_whitespace()
        .map(|family| format!("/usr/lib/{arch}-linux-gnu/{family}"))
        .collect::<Vec<_>>();
    if arch == "x86_64" {
        libraries.push(format!("/usr/lib/{arch}-linux-gnu/libmvec.so*"));
    }
    let libraries = libraries.join(" ");
    let mut recipe = format!(
        "FROM {base} AS libraries\nFROM scratch\nCOPY --from=libraries {libraries} /lib/{arch}-linux-gnu/\nCOPY --from=libraries /lib/{loader} /{loader_dest}/{loader}\nCOPY --from=libraries /etc/fonts /etc/fonts\nCOPY --from=libraries /usr/share/fonts /usr/share/fonts\nCOPY --from=libraries /usr/share/fontconfig /usr/share/fontconfig\nCOPY --from=libraries /usr/share/common-licenses /usr/share/common-licenses\nCOPY --from=libraries /usr/share/doc /usr/share/doc\nCOPY --from=libraries /var/lib/dpkg/status /opt/mirket/base-dpkg-status\nCOPY passwd /etc/passwd\nCOPY group /etc/group\nCOPY job.mjs /opt/mirket/job.mjs\n"
    );
    for dep in &ids {
        recipe.push_str(&format!("COPY {dep} /opt/{dep}\n"));
    }
    recipe.push_str(&format!("ENV LD_LIBRARY_PATH=/lib/{arch}-linux-gnu HOME=/tmp/home XDG_CACHE_HOME=/tmp/cache MIRKET_CONTAINER=1 MIRKET_MOTION_CANVAS_ROOT=/opt/motion-canvas MIRKET_HYPERFRAMES_ROOT=/opt/hyperframes MIRKET_CHROMIUM=/opt/chromium/chrome-headless-shell MIRKET_FFMPEG=/opt/ffmpeg/bin/ffmpeg MIRKET_FFPROBE=/opt/ffmpeg/bin/ffprobe PATH=/opt/nodejs/bin\nUSER 1000:1000\nWORKDIR /work\nENTRYPOINT []\n"));
    let recipe_id = util::sha256(
        format!(
            "{}\n{}\n{}",
            recipe,
            registry.identity(id, target)?,
            JOB_RUNNER
        )
        .as_bytes(),
    );
    recipe.push_str(&format!("LABEL org.mirket.recipe={recipe_id}\n"));
    fs::write(context.path().join("Containerfile"), recipe)?;
    engine.run(&["pull".into(), base.clone()], Duration::from_secs(900))?;
    let image = engine.run(
        &[
            "build".into(),
            "--network=none".into(),
            "--timestamp=1".into(),
            "--layers".into(),
            "--pull=never".into(),
            "--format=oci".into(),
            "--iidfile".into(),
            context
                .path()
                .join("image-id")
                .to_string_lossy()
                .into_owned(),
            context.path().to_string_lossy().into_owned(),
        ],
        Duration::from_secs(900),
    )?;
    eprintln!(
        "mirket package: OCI prepared for {id}: {}",
        image.lines().last().unwrap_or("built")
    );
    let id = String::from_utf8(util::read_bounded(&context.path().join("image-id"), 128)?)?
        .trim()
        .to_owned();
    let identity = ImageIdentity {
        id: if id.starts_with("sha256:") {
            id
        } else {
            format!("sha256:{id}")
        },
        recipe: recipe_id,
    };
    verify(paths, &identity)?;
    Ok(identity)
}
pub(super) fn runtime(
    paths: &Paths,
    registry: &Registry,
    id: &str,
    receipt: super::store::Receipt,
) -> Result<Runtime> {
    let image = receipt
        .container
        .context("container generation is missing; run mirket setup")?;
    Ok(Runtime {
        engine: Engine::resolve(paths)?,
        id: id.into(),
        image: image.id,
        gate: Arc::new(tokio::sync::Semaphore::new(1)),
        worker: Arc::new(Mutex::new(None)),
        profile: registry
            .get(id)?
            .container
            .clone()
            .context("container profile")?,
    })
}

impl Runtime {
    pub async fn job(
        &self,
        workspace: &Path,
        args: &[String],
        check: &impl Fn() -> Result<()>,
    ) -> Result<Job> {
        check()?;
        let cached = self
            .worker
            .lock()
            .map_err(|_| anyhow::anyhow!("container worker lock poisoned"))?
            .clone();
        let cached = match cached {
            Some(worker) if worker.created.elapsed() >= Duration::from_secs(890) => {
                *self
                    .worker
                    .lock()
                    .map_err(|_| anyhow::anyhow!("worker lock poisoned"))? = None;
                worker.stop()?;
                None
            }
            other => other,
        };
        let worker = match cached {
            Some(worker) => worker,
            None => {
                let worker = Arc::new(Worker::start(self).await?);
                *self
                    .worker
                    .lock()
                    .map_err(|_| anyhow::anyhow!("container worker lock poisoned"))? =
                    Some(worker.clone());
                worker
            }
        };
        let mut preparation = WorkerPreparation {
            worker: worker.clone(),
            shared: self.worker.clone(),
            armed: true,
        };
        let stage = tempfile::Builder::new()
            .prefix("job-")
            .tempdir_in(worker.stage.path().join("input"))?;
        let mut total = 0;
        copy_tree(
            workspace,
            stage.path(),
            &mut total,
            64 * 1024 * 1024,
            &mut 0,
            0,
            check,
        )?;
        let args = args
            .iter()
            .map(|arg| {
                arg.strip_prefix(&format!("{}/", workspace.display()))
                    .map(str::to_owned)
                    .unwrap_or_else(|| arg.clone())
            })
            .collect::<Vec<_>>();
        fs::write(
            stage.path().join("launch.json"),
            serde_json::to_vec(&json!({"package":self.id,"args":args}))?,
        )?;
        let key = stage
            .path()
            .file_name()
            .context("job key")?
            .to_string_lossy()
            .into_owned();
        let api = crate::podman_api::Api::new(&self.engine.api_socket()?)?;
        preparation.armed = false;
        Ok(Job {
            stage,
            key,
            api,
            worker,
            finished: false,
            shared: self.worker.clone(),
        })
    }
}
struct Worker {
    stage: tempfile::TempDir,
    cid: String,
    engine: Engine,
    _lease: util::FileLock,
    stopped: std::sync::atomic::AtomicBool,
    created: std::time::Instant,
}
impl std::fmt::Debug for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Worker").field("cid", &self.cid).finish()
    }
}
impl Worker {
    async fn start(runtime: &Runtime) -> Result<Self> {
        let mut lease = None;
        for n in 0..2 {
            if let Ok(lock) =
                util::FileLock::acquire(&runtime.engine.staging.join(format!("worker-{n}.lock")))
            {
                lease = Some(lock);
                break;
            }
        }
        let lease=lease.context("Mirket execution capacity is busy (two workers); close an idle package MCP session and retry")?;
        let stage = tempfile::Builder::new()
            .prefix("worker-")
            .tempdir_in(&runtime.engine.staging)?;
        fs::create_dir(stage.path().join("input"))?;
        let name = format!(
            "{}-{}",
            runtime.engine.owner(),
            stage
                .path()
                .file_name()
                .context("worker name")?
                .to_string_lossy()
        );
        let mut pending = PendingWorker {
            engine: runtime.engine.clone(),
            name: name.clone(),
            armed: true,
        };
        let launch = vec![
            "run".into(),
            "--cidfile".into(),
            stage
                .path()
                .join("container-id")
                .to_string_lossy()
                .into_owned(),
            "--detach".into(),
            "--init".into(),
            "--pull=never".into(),
            "--timeout=900".into(),
            "--name".into(),
            name,
            "--label".into(),
            format!("org.mirket.execution-owner={}", runtime.engine.owner()),
            "--label".into(),
            "org.mirket.execution-kind=offline-render".into(),
            "--user=1000:1000".into(),
            "--read-only".into(),
            "--read-only-tmpfs=false".into(),
            "--cap-drop=ALL".into(),
            "--security-opt=no-new-privileges".into(),
            "--network=none".into(),
            "--ipc=private".into(),
            "--pid=private".into(),
            format!("--cpus={}", runtime.profile.cpu),
            format!("--memory={}m", runtime.profile.memory_mib),
            format!("--memory-swap={}m", runtime.profile.memory_mib),
            format!("--pids-limit={}", runtime.profile.pids),
            "--ulimit=nofile=1024:1024".into(),
            "--tmpfs=/tmp:rw,nosuid,nodev,noexec,size=128m,mode=1777".into(),
            "--tmpfs=/work:rw,nosuid,nodev,noexec,size=256m,mode=1777".into(),
            "--tmpfs=/output:rw,nosuid,nodev,noexec,size=40m,mode=1777".into(),
            "--shm-size=128m".into(),
            "--mount".into(),
            format!(
                "type=bind,src={},dst=/input,ro",
                stage.path().join("input").display()
            ),
            runtime.image.clone(),
            "/opt/nodejs/bin/node".into(),
            "-e".into(),
            "setInterval(()=>{},30000)".into(),
        ];
        let cid = runtime.engine.run_async(&launch).await?.trim().to_owned();
        ensure!(
            cid.len() == 64 && cid.bytes().all(|b| b.is_ascii_hexdigit()),
            "Podman returned invalid immutable container ID"
        );
        pending.armed = false;
        Ok(Self {
            stage,
            cid,
            engine: runtime.engine.clone(),
            _lease: lease,
            stopped: std::sync::atomic::AtomicBool::new(false),
            created: std::time::Instant::now(),
        })
    }
    fn stop(&self) -> Result<()> {
        use std::sync::atomic::Ordering;
        if self.stopped.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        // IDs are immutable: a replacement with the same name cannot be targeted.
        let result = self.engine.run(
            &[
                "rm".into(),
                "--force".into(),
                "--ignore".into(),
                "--time=1".into(),
                self.cid.clone(),
            ],
            Duration::from_secs(8),
        );
        if result.is_err() {
            self.stopped.store(false, Ordering::SeqCst);
        }
        result.map(|_| ())
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            eprintln!(
                "mirket: owned container {} cleanup failed: {error:#}; inspect mirket environment status",
                self.cid
            );
        }
    }
}
// Preparation errors or cancellation must not leave an unreachable worker.
struct WorkerPreparation {
    worker: Arc<Worker>,
    shared: Arc<Mutex<Option<Arc<Worker>>>>,
    armed: bool,
}
impl Drop for WorkerPreparation {
    fn drop(&mut self) {
        if self.armed {
            if let Ok(mut slot) = self.shared.lock() {
                *slot = None;
            }
            if let Err(error) = self.worker.stop() {
                eprintln!("mirket: worker preparation cleanup failed: {error:#}");
            }
        }
    }
}
struct PendingWorker {
    engine: Engine,
    name: String,
    armed: bool,
}
impl Drop for PendingWorker {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        // A cancelled native launcher may have already created the remote object.
        // Reconcile only this random name with our exact ownership label; delete ID.
        let mut reconciled = false;
        for _ in 0..3 {
            if let Ok(raw) = self.engine.run(
                &["container".into(), "inspect".into(), self.name.clone()],
                Duration::from_secs(2),
            ) && let Ok(value) = serde_json::from_str::<Value>(&raw)
                && value[0]["Config"]["Labels"]["org.mirket.execution-owner"] == self.engine.owner()
                && let Some(cid) = value[0]["Id"].as_str()
                && cid.len() == 64
                && cid.bytes().all(|b| b.is_ascii_hexdigit())
            {
                let result = self.engine.run(
                    &["rm".into(), "--force".into(), "--ignore".into(), cid.into()],
                    Duration::from_secs(3),
                );
                if result.is_ok() {
                    reconciled = true;
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        if !reconciled {
            eprintln!(
                "mirket: remote creation may complete later; use mirket environment recover to reconcile proven orphaned workers"
            );
        }
    }
}
pub struct Job {
    pub stage: tempfile::TempDir,
    key: String,
    api: crate::podman_api::Api,
    worker: Arc<Worker>,
    shared: Arc<Mutex<Option<Arc<Worker>>>>,
    finished: bool,
}
impl Job {
    pub async fn execute(&self) -> Result<Vec<u8>> {
        self.api
            .exec(
                &self.worker.cid,
                &[
                    "/opt/nodejs/bin/node".into(),
                    "/opt/mirket/job.mjs".into(),
                    self.key.clone(),
                ],
            )
            .await
    }
    pub async fn collect(&self, workspace: &Path, expected: Option<&Path>) -> Result<()> {
        if let Some(output) = expected {
            let name = output
                .file_name()
                .and_then(|n| n.to_str())
                .context("output name")?;
            let bytes = self.api.output(&self.worker.cid, &self.key, name).await?;
            let destination = workspace.join(name);
            ensure!(!destination.exists(), "private output already exists");
            util::atomic_write(&destination, &bytes)?;
        }
        Ok(())
    }
    pub fn complete(&mut self) {
        self.finished = true;
    }
    pub async fn failed(&mut self) -> Result<()> {
        *self
            .shared
            .lock()
            .map_err(|_| anyhow::anyhow!("worker lock poisoned"))? = None;
        match tokio::time::timeout(Duration::from_secs(3), self.api.remove(&self.worker.cid)).await
        {
            Ok(Ok(())) => {
                self.worker
                    .stopped
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }
            _ => self.worker.stop()?,
        }
        self.finished = true;
        Ok(())
    }
    fn failed_sync(&mut self) -> Result<()> {
        self.finished = true;
        *self
            .shared
            .lock()
            .map_err(|_| anyhow::anyhow!("worker lock poisoned"))? = None;
        self.worker.stop()
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        if !self.finished
            && let Err(error) = self.failed_sync()
        {
            eprintln!("mirket: failed worker cleanup: {error:#}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn image_id_label_and_architecture_must_all_match() {
        let identity = ImageIdentity {
            id: format!("sha256:{}", "a".repeat(64)),
            recipe: "reviewed".into(),
        };
        let mut value = json!({"Id":"a".repeat(64),"Os":"linux","Architecture":if cfg!(target_arch="aarch64"){"arm64"}else{"amd64"},"Config":{"Labels":{"org.mirket.recipe":"reviewed"}}});
        verified_image(&value, &identity).unwrap();
        value["Config"]["Labels"]["org.mirket.recipe"] = json!("changed");
        assert!(verified_image(&value, &identity).is_err());
        value["Config"]["Labels"]["org.mirket.recipe"] = json!("reviewed");
        value["Id"] = json!("b".repeat(64));
        assert!(verified_image(&value, &identity).is_err());
        value["Id"] = json!("a".repeat(64));
        value["Architecture"] = json!("emulated");
        assert!(verified_image(&value, &identity).is_err());
        assert!(image_id("latest").is_err());
    }
    #[test]
    fn staging_bounds_apply_before_copy_and_check_cancellation() {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let input = dir.path().join("input");
        fs::create_dir(&input).unwrap();
        fs::write(input.join("a"), [0u8; 1024]).unwrap();
        assert!(
            copy_tree(
                &input,
                &dir.path().join("oversized"),
                &mut 0,
                100,
                &mut 0,
                0,
                &|| Ok(())
            )
            .is_err()
        );
        assert_eq!(
            fs::metadata(dir.path().join("oversized/a")).unwrap().len(),
            0
        );
        assert!(
            copy_tree(
                &input,
                &dir.path().join("cancelled"),
                &mut 0,
                2048,
                &mut 0,
                0,
                &|| anyhow::bail!("cancelled")
            )
            .is_err()
        );
        assert!(!dir.path().join("cancelled").exists());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(input.join("a"), input.join("linked")).unwrap();
            assert!(
                copy_tree(
                    &input,
                    &dir.path().join("linked-copy"),
                    &mut 0,
                    2048,
                    &mut 0,
                    0,
                    &|| Ok(())
                )
                .is_err()
            );
        }
    }
}
