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

fn unified_runtime(paths: &Paths) -> PathBuf {
    paths.root.join("p")
}

#[cfg(target_os = "macos")]
fn check_socket_budget(short: &std::path::Path, connection: &str) -> Result<()> {
    use std::os::unix::ffi::OsStrExt;
    for socket in [
        short
            .join("t/podman")
            .join(format!("{connection}-gvproxy.sock")),
        short
            .join("t/podman")
            .join(format!("{connection}-api.sock")),
        short
            .join("h/.podman")
            .join(format!("{connection}-ignition.sock")),
    ] {
        ensure!(
            socket.as_os_str().as_bytes().len() < 104,
            "home path is too long for private Podman sockets inside ~/.mirket: {}",
            socket.display()
        );
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Engine {
    pub root: PathBuf,
    pub binary: PathBuf,
    pub env: BTreeMap<String, String>,
    pub staging: PathBuf,
    pub connection: String,
    deadline: Option<Instant>,
}

fn validate_memory_allocation(memory_mib: u64, host_bytes: u64) -> Result<()> {
    ensure!(
        memory_mib >= 6144 && memory_mib <= host_bytes / (2 * 1024 * 1024),
        "VM memory must be at least 6144 MiB and leave at least half the host RAM available"
    );
    Ok(())
}

fn host_memory_bytes() -> Result<u64> {
    let mut command = Command::new("/usr/sbin/sysctl");
    command.args(["-n", "hw.memsize"]);
    let (stdout, _) = capture_output(command, Duration::from_secs(5))?;
    String::from_utf8(stdout)?
        .trim()
        .parse()
        .context("cannot determine host physical memory")
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
        // Podman host-side sockets and temporary files stay under ~/.mirket.
        let connection = format!(
            "mirket-{}",
            &util::sha256(paths.root.to_string_lossy().as_bytes())[..16]
        );
        let short = if cfg!(target_os = "macos") {
            unified_runtime(paths)
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
        let conf = helper_config(&binary)?;
        let config = root.join("containers.conf");
        if config.exists() {
            let saved = util::read_bounded(&config, 4096)?;
            ensure!(
                saved == conf.as_bytes(),
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
        if short == unified_runtime(paths) || cfg!(target_os = "linux") {
            let runtime = short.join("r");
            util::ensure_directory(&runtime)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&runtime, std::fs::Permissions::from_mode(0o700))?;
            }
            env.insert(
                "XDG_RUNTIME_DIR".into(),
                runtime.to_string_lossy().into_owned(),
            );
        }
        if cfg!(target_os = "macos") {
            env.insert("CONTAINERS_MACHINE_PROVIDER".into(), "applehv".into());
        }
        #[cfg(target_os = "linux")]
        {
            // Rootless cgroup delegation belongs to the logged-in user's bus.
            // Keep storage/config/runroot private; never inherit an arbitrary
            // DBUS address or a root session from the calling environment.
            use std::os::unix::fs::{FileTypeExt, MetadataExt};
            let uid = std::fs::metadata("/proc/self")?.uid();
            let bus = PathBuf::from(format!("/run/user/{uid}/bus"));
            if bus.exists() {
                util::reject_symlink_components(&bus)?;
                let metadata = std::fs::metadata(&bus)?;
                ensure!(
                    metadata.uid() == uid && metadata.file_type().is_socket(),
                    "invalid user session bus"
                );
                env.insert(
                    "DBUS_SESSION_BUS_ADDRESS".into(),
                    format!("unix:path={}", bus.display()),
                );
            }
        }
        Ok(Self {
            root: paths.root.clone(),
            binary,
            env,
            staging: root.join("staging"),
            connection,
            deadline: None,
        })
    }
    pub fn owner(&self) -> String {
        self.connection.clone()
    }
    pub(crate) fn with_deadline(mut self, deadline: Instant) -> Self {
        self.deadline = Some(deadline);
        self
    }
    pub(crate) fn remaining_budget(&self, timeout: Duration) -> Result<Duration> {
        let remaining = self.deadline.map_or(timeout, |deadline| {
            timeout.min(deadline.saturating_duration_since(Instant::now()))
        });
        ensure!(
            !remaining.is_zero(),
            "Podman operation deadline exceeded before dispatch"
        );
        Ok(remaining)
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
        capture(command, self.remaining_budget(timeout)?)
    }
    pub fn run(&self, args: &[String], timeout: Duration) -> Result<String> {
        capture(self.command(args), self.remaining_budget(timeout)?)
    }
    pub(crate) fn container_logs(&self, name: &str) -> Result<String> {
        let args = ["container", "logs", "--tail", "50", name].map(str::to_owned);
        let (stdout, stderr) = capture_output(self.command(&args), Duration::from_secs(10))?;
        Ok(format!(
            "{}{}",
            String::from_utf8_lossy(&stdout),
            String::from_utf8_lossy(&stderr)
        ))
    }
    pub fn ready(&self) -> Result<Value> {
        ensure_controller_supported()?;
        self.inspect_ready(true)
    }
    /// Development-only builds and tests may use native rootless Linux ARM64.
    /// Installation and runtime activation retain the managed-controller guard.
    pub(crate) fn candidate_ready(&self) -> Result<Value> {
        if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
            self.inspect_ready(false)
        } else {
            self.ready()
        }
    }
    fn inspect_ready(&self, require_localized: bool) -> Result<Value> {
        if require_localized {
            self.require_local_client()?;
        }
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
            let memory_mib = value["Resources"]["Memory"]
                .as_u64()
                .context("VM memory allocation missing")?;
            validate_memory_allocation(memory_mib, host_memory_bytes()?)?;
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
                    && value["Resources"]["DiskSize"] == 40
                    && value["Resources"]["USBs"] == json!([])
                    && value["Rosetta"] == false
                    && vm["Rosetta"] == false
                    && vm["BinaryPath"].is_null()
                    && vm["VirtualMachine"]["vcpus"] == 4
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
                json!({"ok":true,"connection":self.connection,"staging":self.staging,"runtime_home":self.env["HOME"],"runtime_tmp":self.env["TMPDIR"],"client_localized":self.binary.starts_with(&self.root),"engine":info["host"]})
            }
            Err(e) => {
                json!({"ok":false,"connection":self.connection,"runtime_home":self.env["HOME"],"runtime_tmp":self.env["TMPDIR"],"client_localized":self.binary.starts_with(&self.root),"issue":format!("{e:#}")})
            }
        }
    }
    fn require_local_client(&self) -> Result<()> {
        ensure!(
            self.binary.starts_with(&self.root),
            "Podman client is outside ~/.mirket; run mirket environment localize-client"
        );
        Ok(())
    }
    fn runtime_prefix(&self) -> Result<&std::path::Path> {
        std::path::Path::new(&self.env["HOME"])
            .parent()
            .context("private Podman runtime prefix missing")
    }
    fn lock_runtime_transition(&self) -> Result<util::FileLock> {
        let lock = self.root.join("execution/podman/runtime-consolidate.lock");
        let guard = util::FileLock::acquire(&lock)?;
        let home = self.root.parent().context("Mirket home path missing")?;
        let paths = Paths {
            home: home.to_owned(),
            root: self.root.clone(),
        };
        let selected = if cfg!(target_os = "macos") {
            unified_runtime(&paths)
        } else {
            self.root.join("execution/podman/run")
        };
        ensure!(
            self.runtime_prefix()? == selected,
            "Podman runtime location changed; retry with a fresh Mirket command"
        );
        Ok(guard)
    }
    pub fn init(&self, image: &std::path::Path) -> Result<Value> {
        ensure_controller_supported()?;
        let _runtime_lock = self.lock_runtime_transition()?;
        self.ensure_machine_helpers()?;
        self.require_local_client()?;
        #[cfg(target_os = "macos")]
        check_socket_budget(self.runtime_prefix()?, &self.connection)?;
        validate_memory_allocation(6144, host_memory_bytes()?)?;
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
        ensure_controller_supported()?;
        let _runtime_lock = self.lock_runtime_transition()?;
        self.ensure_machine_helpers()?;
        self.require_local_client()?;
        #[cfg(target_os = "macos")]
        check_socket_budget(self.runtime_prefix()?, &self.connection)?;
        self.validate_machine()?;
        self.raw(
            &["machine".into(), "start".into(), self.connection.clone()],
            Duration::from_secs(120),
        )?;
        self.ready()?;
        Ok(self.status())
    }
    pub fn resize(&self, memory_mib: u64) -> Result<Value> {
        ensure_controller_supported()?;
        let _runtime_lock = self.lock_runtime_transition()?;
        self.ensure_machine_helpers()?;
        validate_memory_allocation(memory_mib, host_memory_bytes()?)?;
        self.validate_machine()?;
        // Podman refuses to resize a running VM. Stopping remains an explicit
        // operation with the same disk, share and live-controller identity checks.
        self.raw(
            &[
                "machine".into(),
                "set".into(),
                format!("--memory={memory_mib}"),
                self.connection.clone(),
            ],
            Duration::from_secs(30),
        )?;
        self.validate_machine()?;
        let observed = self.raw(
            &[
                "machine".into(),
                "inspect".into(),
                "--format={{.Resources.Memory}}".into(),
                self.connection.clone(),
            ],
            Duration::from_secs(10),
        )?;
        ensure!(
            observed.trim().parse::<u64>()? == memory_mib,
            "Podman did not retain the requested memory allocation"
        );
        Ok(
            json!({"resized":self.connection,"memory_mib":memory_mib,"started":false,"data_preserved":true}),
        )
    }
    /// Require both AppleHV helpers before downloading an image or mutating a VM.
    pub(crate) fn ensure_machine_helpers(&self) -> Result<()> {
        ensure_machine_helpers(&self.binary)
    }
    pub fn stop(&self) -> Result<Value> {
        ensure_controller_supported()?;
        let _runtime_lock = self.lock_runtime_transition()?;
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
        // AppleHV derives the next VM from Resources. Its serialized Vfkit
        // template retains the creation-time memory after `machine set`.
        let memory_bytes = configured["Resources"]["Memory"]
            .as_u64()
            .and_then(|mib| mib.checked_mul(1024 * 1024))
            .context("configured VM memory missing or overflowing; no stop request sent")?;
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
                && actual["memoryBytes"] == memory_bytes
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
}

/// Explicit recovery after replacing the registered Podman installation. Never
/// rewrite arbitrary container settings or start/change the existing machine.
pub fn repair_helpers(paths: &Paths, expected_sha256: &str) -> Result<Value> {
    let binary = crate::tools::verified(paths, "podman")?;
    ensure_machine_helpers(&binary)?;
    let root = paths.root.join("execution/podman");
    util::reject_symlink_components(&root)?;
    let _lock = util::FileLock::acquire(&root.join("helpers.lock"))?;
    let config = root.join("containers.conf");
    util::reject_symlink_components(&config)?;
    let saved = util::read_bounded(&config, 4096)?;
    ensure!(
        util::sha256(&saved) == expected_sha256,
        "helper configuration changed; inspect it before repair"
    );
    let document: toml_edit::DocumentMut = std::str::from_utf8(&saved)?.parse()?;
    let engine = document
        .get("engine")
        .and_then(toml_edit::Item::as_table)
        .context("expected owned helper configuration")?;
    let dirs = engine
        .get("helper_binaries_dir")
        .and_then(toml_edit::Item::as_array)
        .context("expected helper directories")?;
    let previous: Vec<&str> = dirs
        .iter()
        .map(|v| v.as_str().context("invalid helper path"))
        .collect::<Result<_>>()?;
    ensure!(
        document.len() == 1 && engine.len() == 1 && !previous.is_empty() && previous.len() <= 3,
        "refusing unrelated container configuration"
    );
    let canonical = format!(
        "[engine]\nhelper_binaries_dir = {}\n",
        serde_json::to_string(&previous)?
    );
    ensure!(
        saved == canonical.as_bytes(),
        "refusing non-owned container configuration"
    );
    let replacement = format!(
        "[engine]\nhelper_binaries_dir = {}\n",
        serde_json::to_string(&helper_directories(&binary)?)?
    );
    ensure!(
        util::read_bounded(&config, 4096)? == saved,
        "helper configuration changed during repair"
    );
    util::atomic_write(&config, replacement.as_bytes())?;
    Ok(
        json!({"repaired":config,"before_sha256":expected_sha256,"sha256":util::sha256(replacement.as_bytes()),"machine_changed":false}),
    )
}

pub(crate) fn helper_config(binary: &std::path::Path) -> Result<String> {
    Ok(format!(
        "[engine]\nhelper_binaries_dir = {}\n",
        serde_json::to_string(&helper_directories(binary)?)?
    ))
}

fn helper_directories(binary: &std::path::Path) -> Result<Vec<PathBuf>> {
    let bin = binary.parent().context("Podman helper directory")?;
    let mut directories = vec![bin.to_owned()];
    // Homebrew splits the machine helpers from its canonical bin/podman-remote.
    // Stay within that selected installation; never consult a global PATH/config.
    if bin.file_name().is_some_and(|name| name == "bin") {
        let libexec = bin
            .parent()
            .context("Podman installation prefix")?
            .join("libexec/podman");
        if libexec.is_dir() {
            util::reject_symlink_components(&libexec)?;
            let mut available = false;
            for name in ["gvproxy", "vfkit"] {
                let path = libexec.join(name);
                util::reject_symlink_components(&path)?;
                let valid = path.is_file();
                #[cfg(unix)]
                let valid = valid && {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::metadata(&path)?.permissions().mode() & 0o111 != 0
                };
                available |= valid;
            }
            if available {
                directories.push(libexec);
            }
        }
    }
    Ok(directories)
}

pub(crate) fn machine_helper_files(binary: &std::path::Path) -> Result<[PathBuf; 2]> {
    util::reject_symlink_components(binary)?;
    ensure!(
        binary.is_file(),
        "selected Podman client is not a regular file"
    );
    #[cfg(unix)]
    let selected_uid = {
        use std::os::unix::fs::MetadataExt;
        std::fs::metadata(binary)?.uid()
    };
    let directories = helper_directories(binary)?;
    let mut selected = Vec::new();
    for name in ["gvproxy", "vfkit"] {
        let mut found = None;
        for directory in &directories {
            let path = directory.join(name);
            util::reject_symlink_components(&path)?;
            let metadata = match std::fs::metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => {
                    return Err(error).with_context(|| format!("inspect Podman helper {name}"));
                }
            };
            ensure!(
                metadata.is_file(),
                "Podman helper {name} is not a regular file"
            );
            #[cfg(unix)]
            {
                use std::os::unix::fs::{MetadataExt, PermissionsExt};
                ensure!(
                    metadata.uid() == selected_uid,
                    "Podman helper {name} has different ownership from the selected client"
                );
                ensure!(
                    metadata.permissions().mode() & 0o111 != 0,
                    "Podman helper {name} is not executable"
                );
            }
            found = Some(path);
            break;
        }
        selected.push(found.with_context(|| format!("selected Podman installation lacks executable {name}; install its native machine helpers before environment init or start"))?);
    }
    Ok([selected.remove(0), selected.remove(0)])
}

fn ensure_machine_helpers(binary: &std::path::Path) -> Result<()> {
    machine_helper_files(binary)?;
    Ok(())
}

fn controller_supported(os: &str, arch: &str) -> bool {
    os == "macos" && arch == "aarch64"
}

pub(crate) fn ensure_controller_supported() -> Result<()> {
    ensure_controller_supported_for(std::env::consts::OS, std::env::consts::ARCH)
}

fn ensure_controller_supported_for(os: &str, arch: &str) -> Result<()> {
    ensure!(
        controller_supported(os, arch),
        "managed Podman execution is verified on macOS ARM64 only; other platforms remain held"
    );
    Ok(())
}

struct CaptureGuard {
    child: std::process::Child,
    armed: bool,
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
fn failure_diagnostics(stdout: &[u8], stderr: &[u8]) -> String {
    let tail = |bytes: &[u8]| {
        String::from_utf8_lossy(&bytes[bytes.len().saturating_sub(8192)..]).into_owned()
    };
    format!("stderr:\n{}\nstdout:\n{}", tail(stderr), tail(stdout))
}

fn capture(command: Command, timeout: Duration) -> Result<String> {
    let (stdout, _) = capture_output(command, timeout)?;
    String::from_utf8(stdout).context("Podman returned non UTF-8 metadata")
}

fn capture_output(mut command: Command, timeout: Duration) -> Result<(Vec<u8>, Vec<u8>)> {
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
    let stdout = stdout.unwrap_or_default();
    let stderr = stderr.unwrap_or_default();
    ensure!(
        status.success(),
        "Podman failed ({status}): {}",
        failure_diagnostics(&stdout, &stderr)
    );
    guard.armed = false;
    Ok((stdout, stderr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expired_operation_budget_rejects_before_starting_any_command() {
        let engine = Engine {
            root: PathBuf::new(),
            binary: PathBuf::from("/must-not-execute"),
            env: BTreeMap::new(),
            staging: PathBuf::new(),
            connection: "owned".into(),
            deadline: Some(Instant::now() - Duration::from_secs(1)),
        };
        assert!(
            engine
                .raw(&[], Duration::from_secs(30))
                .unwrap_err()
                .to_string()
                .contains("deadline exceeded before dispatch")
        );
        let engine = engine.with_deadline(Instant::now() + Duration::from_secs(1));
        assert!(
            engine.remaining_budget(Duration::from_secs(30)).unwrap() <= Duration::from_secs(1)
        );
        assert_eq!(
            engine.remaining_budget(Duration::from_millis(1)).unwrap(),
            Duration::from_millis(1)
        );
    }

    #[test]
    fn memory_allocation_preserves_host_headroom_and_rejects_overflow() {
        let host = 16 * 1024 * 1024 * 1024;
        assert!(validate_memory_allocation(6144, host).is_ok());
        assert!(validate_memory_allocation(8192, host).is_ok());
        for value in [0, 4096, 8193, u64::MAX] {
            assert!(validate_memory_allocation(value, host).is_err());
        }
        assert!(validate_memory_allocation(6144, 8 * 1024 * 1024 * 1024).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn failed_build_retains_underlying_stdout_diagnostics() {
        let mut command = Command::new("/bin/sh");
        command.args([
            "-c",
            "printf 'Certificate issuer is unknown\\n'; printf 'Build step failed\\n' >&2; exit 100",
        ]);
        let error = capture(command, Duration::from_secs(5))
            .unwrap_err()
            .to_string();
        assert!(error.contains("Certificate issuer is unknown"));
        assert!(error.contains("Build step failed"));
    }

    #[cfg(unix)]
    #[test]
    fn successful_log_command_preserves_both_container_streams() {
        let mut command = Command::new("/bin/sh");
        command.args([
            "-c",
            "printf 'service output'; printf 'startup failure' >&2",
        ]);
        let (stdout, stderr) = capture_output(command, Duration::from_secs(5)).unwrap();
        assert_eq!(stdout, b"service output");
        assert_eq!(stderr, b"startup failure");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn private_socket_budget_rejects_overlong_home_paths() {
        assert!(
            check_socket_budget(
                std::path::Path::new("/Users/example/.mirket/p"),
                "mirket-64f61db634acccc9"
            )
            .is_ok()
        );
        let long = format!("/Users/{}/.mirket/p", "x".repeat(90));
        assert!(
            check_socket_budget(std::path::Path::new(&long), "mirket-64f61db634acccc9").is_err()
        );
    }

    #[cfg(unix)]
    #[test]
    fn selected_installation_helpers_require_explicit_repair_for_config_changes() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let temporary = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let paths = Paths::new(temporary.path().to_owned()).unwrap();
        let bin = temporary.path().join("installation/bin");
        std::fs::create_dir_all(&bin).unwrap();
        let binary = bin.join("podman-remote");
        std::fs::write(&binary, b"fixture, never executed").unwrap();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
        crate::tools::register(&paths, "podman", &binary).unwrap();
        let before = Engine::resolve(&paths).unwrap();
        let config = PathBuf::from(&before.env["CONTAINERS_CONF"]);
        assert_eq!(
            helper_directories(&binary).unwrap(),
            std::slice::from_ref(&bin)
        );
        assert!(
            before
                .ensure_machine_helpers()
                .unwrap_err()
                .to_string()
                .contains("gvproxy")
        );
        if cfg!(target_os = "macos") {
            // A missing helper wins over image validation or a machine command.
            assert!(
                before
                    .init(temporary.path().join("absent-image").as_path())
                    .unwrap_err()
                    .to_string()
                    .contains("gvproxy")
            );
            assert!(before.start().unwrap_err().to_string().contains("gvproxy"));
        }

        let libexec = temporary.path().join("installation/libexec/podman");
        std::fs::create_dir_all(&libexec).unwrap();
        for name in ["gvproxy", "vfkit"] {
            let helper = libexec.join(name);
            std::fs::write(&helper, b"fixture, never executed").unwrap();
            std::fs::set_permissions(helper, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let previous = std::fs::read(&config).unwrap();
        assert!(
            Engine::resolve(&paths)
                .unwrap_err()
                .to_string()
                .contains("has drifted")
        );
        repair_helpers(&paths, &util::sha256(&previous)).unwrap();
        assert_eq!(
            helper_directories(&binary).unwrap(),
            [bin.clone(), libexec.clone()]
        );
        let expected = format!(
            "[engine]\nhelper_binaries_dir = {}\n",
            serde_json::to_string(&[bin, libexec.clone()]).unwrap()
        );
        assert_eq!(std::fs::read_to_string(&config).unwrap(), expected);
        let relocated = "[engine]\nhelper_binaries_dir = [\"/previous/installation/bin\"]\n";
        std::fs::write(&config, relocated).unwrap();
        assert!(
            Engine::resolve(&paths)
                .unwrap_err()
                .to_string()
                .contains("has drifted")
        );
        assert!(repair_helpers(&paths, &"0".repeat(64)).is_err());
        assert_eq!(std::fs::read_to_string(&config).unwrap(), relocated);
        let repaired = repair_helpers(&paths, &util::sha256(relocated.as_bytes())).unwrap();
        assert_eq!(repaired["machine_changed"], false);
        assert_eq!(std::fs::read_to_string(&config).unwrap(), expected);
        std::fs::write(&config, format!("{expected}# unexpected change\n")).unwrap();
        let changed = std::fs::read(&config).unwrap();
        assert!(repair_helpers(&paths, &util::sha256(&changed)).is_err());
        assert_eq!(std::fs::read(&config).unwrap(), changed);
        assert!(
            Engine::resolve(&paths)
                .unwrap_err()
                .to_string()
                .contains("has drifted")
        );

        std::fs::remove_file(libexec.join("gvproxy")).unwrap();
        symlink(&binary, libexec.join("gvproxy")).unwrap();
        assert!(helper_directories(&binary).is_err());
        if cfg!(target_os = "macos") {
            std::fs::remove_dir_all(PathBuf::from(&before.env["HOME"]).parent().unwrap()).unwrap();
        }
    }

    #[cfg(unix)]
    #[test]
    fn machine_helpers_accept_colocated_and_split_installations_but_not_partial_pairs() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let temporary = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let bin = temporary.path().join("installation/bin");
        let libexec = temporary.path().join("installation/libexec/podman");
        std::fs::create_dir_all(&bin).unwrap();
        let binary = bin.join("podman-remote");
        std::fs::write(&binary, b"fixture, never executed").unwrap();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();

        assert!(
            ensure_machine_helpers(&binary)
                .unwrap_err()
                .to_string()
                .contains("gvproxy")
        );
        let gvproxy = bin.join("gvproxy");
        std::fs::write(&gvproxy, b"fixture, never executed").unwrap();
        std::fs::set_permissions(&gvproxy, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert!(
            ensure_machine_helpers(&binary)
                .unwrap_err()
                .to_string()
                .contains("vfkit")
        );
        let vfkit = bin.join("vfkit");
        std::fs::write(&vfkit, b"fixture, never executed").unwrap();
        std::fs::set_permissions(&vfkit, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(
            ensure_machine_helpers(&binary)
                .unwrap_err()
                .to_string()
                .contains("not executable")
        );
        std::fs::set_permissions(&vfkit, std::fs::Permissions::from_mode(0o700)).unwrap();
        ensure_machine_helpers(&binary).unwrap();

        std::fs::create_dir_all(&libexec).unwrap();
        std::fs::remove_file(&gvproxy).unwrap();
        let libexec_gvproxy = libexec.join("gvproxy");
        std::fs::write(&libexec_gvproxy, b"fixture, never executed").unwrap();
        std::fs::set_permissions(&libexec_gvproxy, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(
            helper_directories(&binary).unwrap(),
            [bin.clone(), libexec.clone()]
        );
        ensure_machine_helpers(&binary).unwrap(); // one helper in each selected directory

        std::fs::remove_file(&vfkit).unwrap();
        let libexec_vfkit = libexec.join("vfkit");
        std::fs::write(&libexec_vfkit, b"fixture, never executed").unwrap();
        std::fs::set_permissions(&libexec_vfkit, std::fs::Permissions::from_mode(0o700)).unwrap();
        ensure_machine_helpers(&binary).unwrap(); // both helpers in the exact libexec sibling

        std::fs::remove_file(&libexec_gvproxy).unwrap();
        symlink(&binary, &libexec_gvproxy).unwrap();
        assert!(ensure_machine_helpers(&binary).is_err());
    }

    #[test]
    fn managed_controller_support_is_platform_explicit() {
        assert!(controller_supported("macos", "aarch64"));
        for (os, arch) in [
            ("macos", "x86_64"),
            ("linux", "aarch64"),
            ("linux", "x86_64"),
            ("windows", "aarch64"),
        ] {
            assert!(!controller_supported(os, arch), "{os} {arch}");
            assert!(ensure_controller_supported_for(os, arch).is_err());
        }
    }
}
