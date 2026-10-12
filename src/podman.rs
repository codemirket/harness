//! One explicitly registered Podman client, private control data and connection.
//! Nothing here selects another user's daemon, mounts a home, or starts a VM on a call.
use crate::{paths::Paths, util};
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct Engine {
    pub binary: PathBuf,
    pub env: BTreeMap<String, String>,
    pub staging: PathBuf,
    pub connection: String,
}

impl Engine {
    pub fn resolve(paths: &Paths) -> Result<Self> {
        let binary = crate::tools::verified(paths, "podman")
            .context("Podman is required; install the approved native client and register it with mirket tool register podman --executable /absolute/path/to/podman")?;
        let root = paths.root.join("execution/podman");
        for name in ["home", "config", "data", "cache", "run", "staging"] {
            let path = root.join(name);
            util::ensure_directory(&path)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))?;
            }
        }
        let root = root.canonicalize()?;
        // macOS Unix sockets have a short path limit. Podman places Ignition and
        // gvproxy sockets under HOME/TMPDIR, even with private XDG state.
        let short = if cfg!(target_os = "macos") {
            PathBuf::from("/private/tmp").join(format!(
                "mirket-{}",
                &util::sha256(paths.root.to_string_lossy().as_bytes())[..16]
            ))
        } else {
            root.join("run")
        };
        for path in [short.clone(), short.join("h"), short.join("t")] {
            util::reject_symlink_components(&path)?;
            if path.exists() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    ensure!(
                        std::fs::metadata(&path)?.uid() == std::fs::metadata(&paths.home)?.uid(),
                        "private Podman runtime directory belongs to another user"
                    );
                }
            }
            util::ensure_directory(&path)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))?;
            }
        }
        let helper = binary.parent().context("Podman helper directory")?;
        let conf = format!(
            "[engine]\nhelper_binaries_dir = [{}]\n",
            serde_json::to_string(&helper.to_string_lossy())?
        );
        let config = root.join("containers.conf");
        if config.exists() {
            ensure!(
                util::read_bounded(&config, 4096)? == conf.as_bytes(),
                "Mirket Podman helper configuration has drifted"
            );
        } else {
            util::atomic_write(&config, conf.as_bytes())?;
        }
        let mut env = BTreeMap::from([
            (
                "HOME".into(),
                short.join("h").to_string_lossy().into_owned(),
            ),
            (
                "XDG_CONFIG_HOME".into(),
                root.join("config").to_string_lossy().into_owned(),
            ),
            (
                "XDG_DATA_HOME".into(),
                root.join("data").to_string_lossy().into_owned(),
            ),
            (
                "XDG_CACHE_HOME".into(),
                root.join("cache").to_string_lossy().into_owned(),
            ),
            (
                "CONTAINERS_CONF".into(),
                config.to_string_lossy().into_owned(),
            ),
            (
                "PATH".into(),
                format!("{}:/usr/bin:/bin:/usr/sbin:/sbin", helper.display()),
            ),
            ("LANG".into(), "C.UTF-8".into()),
        ]);
        env.insert(
            "TMPDIR".into(),
            short.join("t").to_string_lossy().into_owned(),
        );
        if cfg!(target_os = "macos") {
            env.insert("CONTAINERS_MACHINE_PROVIDER".into(), "applehv".into());
        }
        let connection = format!(
            "mirket-{}",
            &util::sha256(paths.root.to_string_lossy().as_bytes())[..16]
        );
        Ok(Self {
            binary,
            env,
            staging: root.join("staging"),
            connection,
        })
    }
    pub fn owner(&self) -> String {
        self.connection.clone()
    }
    pub fn command(&self, args: &[String]) -> Command {
        let mut command = Command::new(&self.binary);
        command.env_clear().envs(&self.env);
        if cfg!(target_os = "macos") || cfg!(windows) {
            command.args(["--connection", &self.connection]);
        }
        command.args(args);
        command
    }
    pub fn raw(&self, args: &[String], timeout: Duration) -> Result<String> {
        let mut command = Command::new(&self.binary);
        command.env_clear().envs(&self.env).args(args);
        capture(command, timeout)
    }
    pub fn run(&self, args: &[String], timeout: Duration) -> Result<String> {
        capture(self.command(args), timeout)
    }
    pub async fn run_async(&self, args: &[String]) -> Result<String> {
        use tokio::io::AsyncReadExt;
        // A long-lived MCP session can outlive its private connection metadata.
        // Refuse drift before creating a replacement worker on any daemon.
        self.validate_machine()?;
        let mut command = tokio::process::Command::from(self.command(args));
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(unix)]
        command.process_group(0);
        let mut child = command.spawn().context("start registered Podman client")?;
        let mut guard = AsyncCaptureGuard(child.id());
        let stdout = child.stdout.take().context("Podman stdout")?;
        let stderr = child.stderr.take().context("Podman stderr")?;
        async fn bounded(pipe: impl tokio::io::AsyncRead + Unpin) -> Result<Vec<u8>> {
            let mut bytes = Vec::new();
            pipe.take(2 * 1024 * 1024 + 1)
                .read_to_end(&mut bytes)
                .await?;
            ensure!(
                bytes.len() <= 2 * 1024 * 1024,
                "Podman diagnostics exceed 2 MiB"
            );
            Ok(bytes)
        }
        let (status, stdout, stderr) = tokio::try_join!(
            async { Ok::<_, anyhow::Error>(child.wait().await?) },
            bounded(stdout),
            bounded(stderr)
        )?;
        ensure!(
            status.success(),
            "Podman failed ({status}): {}",
            String::from_utf8_lossy(&stderr)
                .chars()
                .take(2048)
                .collect::<String>()
        );
        guard.0 = None;
        String::from_utf8(stdout).context("Podman returned non UTF-8 metadata")
    }
    pub(crate) fn api_socket(&self) -> Result<PathBuf> {
        self.validate_machine()?;
        ensure!(
            cfg!(target_os = "macos"),
            "native Unix API controller is verified on macOS only"
        );
        Ok(PathBuf::from(&self.env["TMPDIR"])
            .join("podman")
            .join(format!("{}-api.sock", self.connection)))
    }
    pub fn ready(&self) -> Result<Value> {
        self.validate_machine()?;
        let value: Value = serde_json::from_str(&self.run(&["info".into(), "--format=json".into()], Duration::from_secs(8))
            .context("Mirket Podman environment is absent or stopped; run mirket environment init, then mirket environment start")?)?;
        ensure!(
            value["host"]["os"] == "linux" && value["host"]["security"]["rootless"] == true,
            "Mirket requires a rootless Linux Podman engine"
        );
        let arch = if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "amd64"
        };
        ensure!(
            value["host"]["arch"] == arch,
            "Podman architecture differs from host; emulation is refused"
        );
        Ok(value)
    }
    fn validate_machine(&self) -> Result<()> {
        if cfg!(target_os = "macos") {
            let config = PathBuf::from(
                self.env
                    .get("XDG_CONFIG_HOME")
                    .context("private Podman config")?,
            )
            .join("containers/podman/machine/applehv")
            .join(format!("{}.json", self.connection));
            let value: Value = serde_json::from_slice(
                &util::read_bounded(&config, 128 * 1024)
                    .context("Mirket VM is not initialized; run mirket environment init")?,
            )?;
            let mounts = value["Mounts"]
                .as_array()
                .context("machine mounts missing")?;
            let private_data =
                PathBuf::from(&self.env["XDG_DATA_HOME"]).join("containers/podman/machine");
            let identity = private_data.join("machine");
            let disk = private_data
                .join("applehv")
                .join(format!("{}-arm64.raw", self.connection));
            let efi = private_data
                .join("applehv")
                .join(format!("efi-bl-{}", self.connection));
            for path in [&identity, &disk, &efi] {
                util::reject_symlink_components(path)?;
            }
            let vm = &value["AppleHypervisor"]["Vfkit"];
            let endpoint = reqwest::Url::parse(
                vm["Endpoint"]
                    .as_str()
                    .context("VM control endpoint missing")?,
            )?;
            ensure!(
                endpoint.scheme() == "http"
                    && endpoint.host_str() == Some("localhost")
                    && endpoint.port().is_some_and(|p| p > 1024)
                    && endpoint.username().is_empty()
                    && endpoint.password().is_none()
                    && endpoint.path() == "/"
                    && endpoint.query().is_none()
                    && endpoint.fragment().is_none(),
                "Mirket VM control endpoint is not private loopback"
            );
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                ensure!(
                    value["HostUser"]["UID"] == std::fs::metadata(&self.staging)?.uid(),
                    "Mirket VM host user differs from its private staging owner"
                );
            }
            ensure!(
                value["SSH"]["IdentityPath"] == identity.to_string_lossy().as_ref()
                    && value["SSH"]["RemoteUsername"] == "core"
                    && value["ImagePath"]["Path"] == disk.to_string_lossy().as_ref()
                    && value["Resources"]["CPUs"] == 4
                    && value["Resources"]["Memory"] == 6144
                    && value["Resources"]["DiskSize"] == 40
                    && value["Resources"]["USBs"] == json!([])
                    && value["Rosetta"] == false
                    && vm["Rosetta"] == false
                    && vm["BinaryPath"].is_null()
                    && vm["VirtualMachine"]["vcpus"] == 4
                    && vm["VirtualMachine"]["memoryBytes"] == 6442450944u64
                    && vm["VirtualMachine"]["bootloader"]["kind"] == "efiBootloader"
                    && vm["VirtualMachine"]["bootloader"]["efiVariableStorePath"]
                        == efi.to_string_lossy().as_ref(),
                "Mirket machine runtime, identity, disk or emulation settings have drifted"
            );
            ensure!(
                value["Name"] == self.connection
                    && value["HostUser"]["Rootful"] == false
                    && mounts.len() == 1
                    && mounts[0]["Source"] == self.staging.to_string_lossy().as_ref()
                    && mounts[0]["Target"] == self.staging.to_string_lossy().as_ref()
                    && mounts[0]["ReadOnly"] == false
                    && mounts[0]["Type"] == "virtiofs",
                "Mirket machine sharing or ownership has drifted; refusing execution"
            );
            let connections: Value = serde_json::from_slice(&util::read_bounded(
                &PathBuf::from(&self.env["XDG_CONFIG_HOME"])
                    .join("containers/podman-connections.json"),
                128 * 1024,
            )?)?;
            let selected = &connections["Connection"]["Connections"][&self.connection];
            let expected = format!(
                "ssh://{}@127.0.0.1:{}/run/user/{}/podman/podman.sock",
                value["SSH"]["RemoteUsername"]
                    .as_str()
                    .context("VM SSH user")?,
                value["SSH"]["Port"].as_u64().context("VM SSH port")?,
                value["HostUser"]["UID"].as_u64().context("VM UID")?
            );
            ensure!(
                selected["URI"] == expected
                    && selected["Identity"] == identity.to_string_lossy().as_ref()
                    && selected["IsMachine"] == true,
                "Mirket Podman connection has drifted; refusing another engine"
            );
        }
        Ok(())
    }
    pub fn status(&self) -> Value {
        match self.ready() {
            Ok(info) => {
                json!({"ok":true,"connection":self.connection,"staging":self.staging,"engine":info["host"]})
            }
            Err(e) => json!({"ok":false,"connection":self.connection,"issue":format!("{e:#}")}),
        }
    }
    pub fn init(&self, image: &std::path::Path) -> Result<Value> {
        ensure!(
            cfg!(all(target_os = "macos", target_arch = "aarch64")),
            "Managed VM initialization is verified on macOS ARM64 only; other platforms remain held"
        );
        util::reject_symlink_components(image)?;
        // An approved, exact upstream boot artifact; no caller-selected ignition or helper.
        ensure!(
            crate::packages::verify_machine_image(image)?,
            "Podman machine image differs from its reviewed ARM64 pin"
        );
        self.raw(
            &[
                "machine".into(),
                "init".into(),
                "--provider=applehv".into(),
                "--cpus=4".into(),
                "--memory=6144".into(),
                "--disk-size=40".into(),
                "--update-connection=false".into(),
                "--image".into(),
                image.to_string_lossy().into_owned(),
                "--volume".into(),
                format!("{}:{}:rw", self.staging.display(), self.staging.display()),
                self.connection.clone(),
            ],
            Duration::from_secs(120),
        )?;
        Ok(json!({"initialized":self.connection,"shared":self.staging,"started":false}))
    }
    pub fn start(&self) -> Result<Value> {
        self.validate_machine()?;
        self.raw(
            &["machine".into(), "start".into(), self.connection.clone()],
            Duration::from_secs(120),
        )?;
        self.ready()?;
        Ok(self.status())
    }
    pub fn stop(&self) -> Result<Value> {
        self.validate_machine()?;
        // A valid loopback URL can still belong to a different VM. Read its
        // actual device identity before the native client sends a stop request.
        if cfg!(target_os = "macos") {
            let config = PathBuf::from(&self.env["XDG_CONFIG_HOME"])
                .join("containers/podman/machine/applehv")
                .join(format!("{}.json", self.connection));
            let configured: Value =
                serde_json::from_slice(&util::read_bounded(&config, 128 * 1024)?)?;
            let endpoint = reqwest::Url::parse(
                configured["AppleHypervisor"]["Vfkit"]["Endpoint"]
                    .as_str()
                    .context("VM control endpoint missing")?,
            )?;
            let client = reqwest::blocking::Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(2))
                .timeout(Duration::from_secs(3))
                .build()?;
            let response = client.get(endpoint.join("vm/inspect")?).send().context(
                "VM control unavailable; no stop request sent; inspect mirket environment status",
            )?;
            ensure!(
                response.status() == reqwest::StatusCode::OK,
                "VM control identity unavailable; no stop request sent"
            );
            let mut bytes = Vec::new();
            response.take(16 * 1024 + 1).read_to_end(&mut bytes)?;
            ensure!(
                bytes.len() <= 16 * 1024,
                "VM control identity exceeds 16 KiB; no stop request sent"
            );
            let actual: Value = serde_json::from_slice(&bytes)
                .context("invalid VM control identity; no stop request sent")?;
            self.validate_control_vm(&actual, &configured)?;
        }
        self.raw(
            &["machine".into(), "stop".into(), self.connection.clone()],
            Duration::from_secs(40),
        )?;
        Ok(json!({"stopped":self.connection,"data_preserved":true}))
    }
    fn validate_control_vm(&self, actual: &Value, configured: &Value) -> Result<()> {
        let private_data =
            PathBuf::from(&self.env["XDG_DATA_HOME"]).join("containers/podman/machine/applehv");
        let runtime = PathBuf::from(&self.env["TMPDIR"]).join("podman");
        let ignition = PathBuf::from(&self.env["HOME"])
            .join(".podman")
            .join(format!("{}-ignition.sock", self.connection));
        let devices = actual["devices"]
            .as_array()
            .context("VM control devices missing; no stop request sent")?;
        let kind_count = |kind: &str| devices.iter().filter(|d| d["kind"] == kind).count();
        // Podman 6.1.3 adds ignition only on the VM's first boot. Subsequent
        // boots keep the same owned disk, share and runtime socket, without
        // that device. No other optional or unknown device is admitted.
        let ignition_count = devices
            .iter()
            .filter(|d| d["kind"] == "virtiosock" && d["port"] == 1024)
            .count();
        ensure!(
            actual["vcpus"] == 4
                && actual["memoryBytes"] == 6442450944u64
                && actual["bootloader"]
                    == configured["AppleHypervisor"]["Vfkit"]["VirtualMachine"]["bootloader"]
                && ignition_count <= 1
                && devices.len() == 6 + ignition_count
                && kind_count("virtioblk") == 1
                && kind_count("virtiorng") == 1
                && kind_count("virtiosock") == 1 + ignition_count
                && kind_count("virtioserial") == 1
                && kind_count("virtionet") == 1
                && kind_count("virtiofs") == 1
                && actual["timesync"]["vsockPort"] == 1234,
            "VM control target is not the owned runtime; no stop request sent"
        );
        for device in devices {
            let owned = match device["kind"].as_str() {
                Some("virtioblk") => {
                    device["devName"] == "virtio-blk"
                        && device["imagePath"]
                            == private_data
                                .join(format!("{}-arm64.raw", self.connection))
                                .to_string_lossy()
                                .as_ref()
                }
                Some("virtiorng") => true,
                Some("virtiofs") => {
                    device["sharedDir"] == self.staging.to_string_lossy().as_ref()
                        && device["mountTag"] == configured["Mounts"][0]["Tag"]
                }
                Some("virtioserial") => {
                    device["logFile"]
                        == runtime
                            .join(format!("{}.log", self.connection))
                            .to_string_lossy()
                            .as_ref()
                }
                Some("virtionet") => {
                    device["nat"] == false
                        && device["socket"] == json!({})
                        && device["vfkitMagic"] == true
                }
                Some("virtiosock") => {
                    device["listen"] == true
                        && match device["port"].as_u64() {
                            Some(1025) => {
                                device["socketURL"]
                                    == runtime
                                        .join(format!("{}.sock", self.connection))
                                        .to_string_lossy()
                                        .as_ref()
                            }
                            Some(1024) => {
                                device["socketURL"] == ignition.to_string_lossy().as_ref()
                            }
                            _ => false,
                        }
                }
                _ => false,
            };
            ensure!(
                owned,
                "VM control device belongs to another runtime; no stop request sent"
            );
        }
        ensure!(
            devices
                .iter()
                .filter(|d| d["kind"] == "virtiosock" && d["port"] == 1025)
                .count()
                == 1,
            "VM control socket identity has drifted; no stop request sent"
        );
        Ok(())
    }
    /// Reconcile a remote create that completed after its native caller stopped.
    /// Active workers still own their input directory and are never selected.
    pub fn recover(&self) -> Result<Value> {
        self.ready()?;
        let deadline = Instant::now() + Duration::from_secs(20);
        let ids = self.run(
            &[
                "container".into(),
                "ls".into(),
                "--all".into(),
                "--filter".into(),
                format!("label=org.mirket.execution-owner={}", self.owner()),
                "--filter".into(),
                "label=org.mirket.execution-kind=offline-render".into(),
                "--format={{.ID}}".into(),
                "--no-trunc".into(),
            ],
            Duration::from_secs(5),
        )?;
        ensure!(
            ids.lines().count() <= 128,
            "too many owned workers; inspect environment before recovery"
        );
        let mut removed = Vec::new();
        for id in ids.lines() {
            ensure!(
                Instant::now() < deadline,
                "worker recovery deadline exceeded; retry mirket environment recover"
            );
            ensure!(
                id.len() == 64 && id.bytes().all(|b| b.is_ascii_hexdigit()),
                "invalid recovery container ID"
            );
            let inspected: Value = serde_json::from_str(&self.run(
                &["container".into(), "inspect".into(), id.into()],
                Duration::from_secs(3),
            )?)?;
            let value = &inspected[0];
            ensure!(
                value["Id"] == id,
                "recovery inspect returned another immutable container ID"
            );
            if orphaned_worker(value, &self.staging, &self.owner())? {
                self.run(
                    &["rm".into(), "--force".into(), "--ignore".into(), id.into()],
                    Duration::from_secs(3),
                )?;
                removed.push(id.to_owned());
            }
        }
        Ok(
            json!({"recovered":removed,"active_workers_preserved":true,"service_data_preserved":true}),
        )
    }
}

fn orphaned_worker(value: &Value, staging: &std::path::Path, owner: &str) -> Result<bool> {
    if value["Config"]["Labels"]["org.mirket.execution-owner"] != owner
        || value["Config"]["Labels"]["org.mirket.execution-kind"] != "offline-render"
        || value["Config"]["User"] != "1000:1000"
        || value["HostConfig"]["ReadonlyRootfs"] != true
        || value["HostConfig"]["NetworkMode"] != "none"
    {
        return Ok(false);
    }
    let mounts = value["Mounts"]
        .as_array()
        .context("worker mounts missing")?;
    let inputs = mounts
        .iter()
        .filter(|m| m["Destination"] == "/input")
        .collect::<Vec<_>>();
    if inputs.len() != 1 || inputs[0]["Type"] != "bind" || inputs[0]["RW"] != false {
        return Ok(false);
    }
    let source = PathBuf::from(
        inputs[0]["Source"]
            .as_str()
            .context("worker input source")?,
    );
    util::reject_symlink_components(&source)?;
    let Ok(relative) = source.strip_prefix(staging) else {
        return Ok(false);
    };
    let parts = relative.components().collect::<Vec<_>>();
    if parts.len() != 2
        || parts[1].as_os_str() != "input"
        || !parts[0]
            .as_os_str()
            .to_string_lossy()
            .starts_with("worker-")
    {
        return Ok(false);
    }
    let token = parts[0].as_os_str().to_string_lossy();
    let expected_name = format!("{owner}-{token}");
    if value["Name"]
        .as_str()
        .map(|name| name.trim_start_matches('/'))
        != Some(expected_name.as_str())
    {
        return Ok(false);
    }
    Ok(!source.exists() && !source.parent().context("worker directory")?.exists())
}

struct CaptureGuard {
    child: std::process::Child,
    armed: bool,
}
struct AsyncCaptureGuard(Option<u32>);
impl Drop for AsyncCaptureGuard {
    fn drop(&mut self) {
        #[cfg(unix)]
        if let Some(pid) = self.0.take() {
            let _ = Command::new("/bin/kill")
                .env_clear()
                .args(["-KILL", "--", &format!("-{pid}")])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
}
impl Drop for CaptureGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        #[cfg(unix)]
        {
            let _ = Command::new("/bin/kill")
                .env_clear()
                .args(["-KILL", "--", &format!("-{}", self.child.id())])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Metadata is bounded and diagnostics never enter an MCP stdout stream.
fn capture(mut command: Command, timeout: Duration) -> Result<String> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut guard = CaptureGuard {
        child: command.spawn().context("start registered Podman client")?,
        armed: true,
    };
    let child = &mut guard.child;
    let (tx, rx) = std::sync::mpsc::channel();
    for (kind, pipe) in [
        (
            true,
            Box::new(child.stdout.take().unwrap()) as Box<dyn Read + Send>,
        ),
        (
            false,
            Box::new(child.stderr.take().unwrap()) as Box<dyn Read + Send>,
        ),
    ] {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let mut data = Vec::new();
            let result = pipe.take(2 * 1024 * 1024 + 1).read_to_end(&mut data);
            let _ = tx.send((kind, result, data));
        });
    }
    let begin = Instant::now();
    let mut stdout = None;
    let mut stderr = None;
    let status = loop {
        while let Ok((kind, result, bytes)) = rx.try_recv() {
            result?;
            if bytes.len() > 2 * 1024 * 1024 {
                let _ = child.kill();
                let _ = child.wait();
                bail!("Podman diagnostics exceed 2 MiB");
            }
            if kind {
                stdout = Some(bytes);
            } else {
                stderr = Some(bytes);
            }
        }
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if begin.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            bail!(
                "Podman operation exceeded {} seconds; reconcile with mirket environment status or package status",
                timeout.as_secs()
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    for _ in 0..usize::from(stdout.is_none()) + usize::from(stderr.is_none()) {
        let (kind, result, bytes) = rx
            .recv_timeout(Duration::from_secs(2))
            .context("Podman diagnostic pipes did not close")?;
        result?;
        ensure!(
            bytes.len() <= 2 * 1024 * 1024,
            "Podman diagnostics exceed 2 MiB"
        );
        if kind {
            stdout = Some(bytes);
        } else {
            stderr = Some(bytes);
        }
    }
    ensure!(
        status.success(),
        "Podman failed ({status}): {}",
        String::from_utf8_lossy(&stderr.unwrap_or_default())
            .chars()
            .take(2048)
            .collect::<String>()
    );
    let text = String::from_utf8(stdout.unwrap_or_default())
        .context("Podman returned non UTF-8 metadata")?;
    guard.armed = false;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn orphan_recovery_preserves_active_jobs_and_unrelated_data() {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let stage = dir.path().join("staging");
        std::fs::create_dir(&stage).unwrap();
        let input = stage.join("worker-ABC123/input");
        std::fs::create_dir_all(&input).unwrap();
        let mut value = json!({"Name":"mirket-fixture-worker-ABC123","Config":{"User":"1000:1000","Labels":{"org.mirket.execution-owner":"mirket-fixture","org.mirket.execution-kind":"offline-render"}},"HostConfig":{"ReadonlyRootfs":true,"NetworkMode":"none"},"Mounts":[{"Destination":"/input","Source":input,"Type":"bind","RW":false}]});
        assert!(!orphaned_worker(&value, &stage, "mirket-fixture").unwrap());
        std::fs::remove_dir_all(input.parent().unwrap()).unwrap();
        assert!(orphaned_worker(&value, &stage, "mirket-fixture").unwrap());
        value["Name"] = json!("unrelated-app");
        assert!(!orphaned_worker(&value, &stage, "mirket-fixture").unwrap());
        value["Name"] = json!("mirket-fixture-worker-ABC123");
        value["Mounts"][0]["Source"] = json!(dir.path().join("outside/input"));
        assert!(!orphaned_worker(&value, &stage, "mirket-fixture").unwrap());
        assert!(dir.path().exists());
    }
}
