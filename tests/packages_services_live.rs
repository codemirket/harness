//! Opt-in real private Podman recovery and isolation probes in a marked disposable Mirket home.
#![cfg(all(target_os = "macos", target_arch = "aarch64"))]
use anyhow::{Context, Result, ensure};
use mirket::{paths::Paths, podman::Engine};
use serde_json::{Value, json};
use std::time::Duration;
use std::{
    fs,
    net::TcpListener,
    path::Path,
    process::{Command, Output},
};

fn invoke(home: &Path, args: &[&str]) -> Result<Output> {
    Ok(Command::new(env!("CARGO_BIN_EXE_mirket"))
        .arg("--home")
        .arg(home)
        .arg("--json")
        .args(args)
        .output()?)
}
fn good(home: &Path, args: &[&str]) -> Result<Value> {
    let r = invoke(home, args)?;
    ensure!(
        r.status.success(),
        "{args:?}: stdout={} stderr={}",
        String::from_utf8_lossy(&r.stdout),
        String::from_utf8_lossy(&r.stderr)
    );
    Ok(serde_json::from_slice(&r.stdout)?)
}
fn penpot_status(home: &Path) -> Result<Value> {
    // The shared disposable home may select unrelated packages whose ongoing
    // integration is not this fixture's ownership. Inspect only Penpot's result;
    // a package-status exit failure must still contain valid structured results.
    let r = invoke(home, &["package", "status"])?;
    let value: Value = serde_json::from_slice(&r.stdout).context("package status JSON")?;
    value["checks"]
        .as_array()
        .and_then(|checks| checks.iter().find(|check| check["id"] == "penpot"))
        .cloned()
        .context("package status omitted Penpot")
}
fn podman(home: &Path, args: &[&str]) -> Result<String> {
    let paths = Paths::new(home.to_owned())?;
    let engine = Engine::resolve(&paths)?;
    engine.run(
        &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        Duration::from_secs(30),
    )
}

#[test]
#[ignore = "requires explicitly installed/running Penpot in marked disposable home"]
fn penpot_service_drift_and_failed_start_preserve_prior_state() -> Result<()> {
    let home = std::path::PathBuf::from(
        std::env::var_os("MIRKET_PACKAGE_TEST_HOME")
            .context("absolute disposable home required")?,
    )
    .canonicalize()?;
    ensure!(
        fs::read(home.join(".mirket-package-test-home"))? == b"disposable package test home\n",
        "disposable marker required"
    );
    let state: Value = serde_json::from_slice(&fs::read(
        home.join(".mirket/packages/data/penpot/service/state.json"),
    )?)?;
    let owner = state["owner"].as_str().context("owner")?;
    let port = state["port"].as_u64().context("port")? as u16;
    let frontend = format!("{owner}-penpot-frontend");
    let mut trace = vec![json!({"start":good(&home,&["package","start","penpot"])?})];
    let engine = Engine::resolve(&Paths::new(home.clone())?)?;
    ensure!(
        state["engine_owner"] == engine.owner(),
        "service is not bound to the private connection"
    );
    let network: Value = serde_json::from_str(&podman(
        &home,
        &["network", "inspect", owner, "--format", "{{json .}}"],
    )?)?;
    ensure!(
        network["internal"] == true,
        "service network permits egress"
    );
    let mut isolation = vec![];
    for alias in [
        "penpot-postgres",
        "penpot-valkey",
        "penpot-backend",
        "penpot-exporter",
        "penpot-frontend",
    ] {
        let name = format!("{owner}-{alias}");
        let value: Value = serde_json::from_str(&podman(
            &home,
            &["container", "inspect", &name, "--format", "{{json .}}"],
        )?)?;
        let h = &value["HostConfig"];
        ensure!(
            h["LogConfig"]["Type"] == "k8s-file"
                && h["LogConfig"]["Size"] == "4MB"
                && h["LogConfig"]["Path"]
                    == format!(
                        "{}/ctr.log",
                        value["StaticDir"].as_str().context("private storage")?
                    ),
            "unbounded or redirected service logging: {alias}"
        );
        let logs = podman(&home, &["logs", "--tail", "200", &name])?;
        ensure!(logs.len() <= 2 * 1024 * 1024, "oversized bounded logs");
        let backend_ready_line = logs.lines().any(|line| {
            line.contains("Undertow")
                || line.contains("undertow")
                || line.contains("started server")
                || line.contains("starting server")
        });
        if alias == "penpot-backend" {
            ensure!(
                !logs.is_empty() && backend_ready_line,
                "backend logs do not prove the reviewed server startup"
            );
            let options = podman(
                &home,
                &[
                    "exec",
                    &name,
                    "/bin/sh",
                    "-c",
                    "awk '$2 == \"/run/mirket\" || $2 == \"/run/mirket-native\" || $2 == \"/tmp/undertow\" {print $2, $4}' /proc/mounts",
                ],
            )?;
            let native = options
                .lines()
                .find(|line| line.starts_with("/run/mirket-native "))
                .context("native library mount")?;
            let general = options
                .lines()
                .find(|line| line.starts_with("/run/mirket "))
                .context("general temporary mount")?;
            let multipart = options
                .lines()
                .find(|line| line.starts_with("/tmp/undertow "))
                .context("multipart temporary mount")?;
            ensure!(
                native.contains("size=8192k")
                    && native.contains("nosuid")
                    && native.contains("nodev")
                    && !native.contains("noexec")
                    && general.contains("noexec")
                    && multipart.contains("noexec")
                    && multipart.contains("nosuid")
                    && multipart.contains("nodev")
                    && multipart.contains("size=65536k"),
                "native-library exception escaped its dedicated 8MiB mount"
            );
            trace.push(json!({"native_library_storage":{"pinned_component":"zstd-jni1.5.7-12","private_mount":"/run/mirket-native","bytes_ceiling":8*1024*1024,"executable_mapping":true,"general_temp_noexec":true,"kernel_mount_options":options,"actual_loading_requires_design_call":true}}));
            trace.push(json!({"multipart_storage":{"pinned_component":"Yetti e810f87","private_mount":"/tmp/undertow","bytes_ceiling":64*1024*1024,"noexec":true,"host_mounts":false,"actual_upload_requires_export_call":true}}));
        }
        trace.push(json!({"bounded_service_logs":{"service":alias,"retrieval_succeeded":true,"driver":"k8s-file","limit":"4MB","tail_lines":200,"observed_bytes":logs.len(),"backend_startup_observed":backend_ready_line,"raw_logs_retained":false}}));
        ensure!(
            h["ReadonlyRootfs"] == true && h["SecurityOpt"] == json!(["no-new-privileges"]),
            "root or privilege boundary: {alias}"
        );
        ensure!(
            value["Config"]["User"]
                .as_str()
                .is_some_and(|s| s != "0:0" && !s.starts_with("0:")),
            "root user: {alias}"
        );
        ensure!(
            value["Mounts"]
                .as_array()
                .is_some_and(|m| m.iter().all(|v| v["Type"] == "volume")),
            "host mount: {alias}"
        );
        ensure!(
            h["Memory"].as_u64().is_some_and(|v| v > 0)
                && h["PidsLimit"].as_u64().is_some_and(|v| v > 0),
            "missing limit: {alias}"
        );
        ensure!(
            value["NetworkSettings"]["Networks"]
                .as_object()
                .is_some_and(|v| v.len() == 1 && v.contains_key(owner)),
            "extra network: {alias}"
        );
        let actual = podman(
            &home,
            &[
                "exec",
                &name,
                "/bin/sh",
                "-c",
                "id -u; cat /sys/fs/cgroup/memory.max /sys/fs/cgroup/memory.swap.max /sys/fs/cgroup/pids.max /sys/fs/cgroup/cpu.max; sed -n '/^CapEff:/p; /^NoNewPrivs:/p' /proc/self/status",
            ],
        )?;
        let observed: Vec<_> = actual.lines().collect();
        ensure!(
            observed.len() == 7,
            "incomplete kernel limit probe: {alias}"
        );
        ensure!(
            observed[0]
                == value["Config"]["User"]
                    .as_str()
                    .unwrap()
                    .split(':')
                    .next()
                    .unwrap(),
            "wrong actual user: {alias}"
        );
        ensure!(
            observed[1].parse::<u64>()? == h["Memory"].as_u64().unwrap() && observed[2] == "0",
            "memory limits not enforced: {alias}"
        );
        ensure!(
            observed[3].parse::<u64>()? == h["PidsLimit"].as_u64().unwrap(),
            "PID limit not enforced: {alias}"
        );
        let cpu: Vec<u64> = observed[4]
            .split_whitespace()
            .map(str::parse)
            .collect::<std::result::Result<_, _>>()?;
        ensure!(
            cpu.len() == 2 && cpu[0] * 1_000_000_000 / cpu[1] == h["NanoCpus"].as_u64().unwrap(),
            "CPU limit not enforced: {alias}"
        );
        ensure!(
            observed[5].split_whitespace().nth(1) == Some("0000000000000000")
                && observed[6].split_whitespace().nth(1) == Some("1"),
            "capabilities or no-new-privileges not enforced: {alias}"
        );
        isolation.push(json!({"service":alias,"user":value["Config"]["User"],"readonly_root":true,"no_new_privileges":true,"memory_bytes":h["Memory"],"pids":h["PidsLimit"],"native_architecture":"arm64","network":"internal","kernel_limits":observed}));
    }
    let unrelated = tempfile::tempdir()?;
    let sentinel = unrelated.path().join("private-sentinel.txt");
    fs::write(&sentinel, b"Unrelated host application data")?;
    let denied=engine.run(&["exec".into(),frontend.clone(),"/bin/sh".into(),"-c".into(),"test ! -e \"$1\" && test ! -e /run/podman/podman.sock && test ! -e /var/run/docker.sock && ! touch /var/www/app/mirket-forbidden".into(),"mirket-isolation-probe".into(),sentinel.to_str().context("UTF8 sentinel")?.into()],Duration::from_secs(10));
    ensure!(
        denied.is_ok(),
        "host data or immutable application root accessible"
    );
    ensure!(
        fs::read(&sentinel)? == b"Unrelated host application data",
        "host sentinel changed"
    );
    trace.push(json!({"isolation":isolation,"host_data_and_socket_absent":true,"immutable_root_write_denied":true,"private_connection":engine.owner()}));
    // Repeated start must adopt the exact managed configuration, including Valkey's
    // explicit /data volume rather than Podman's anonymous image-volume default.
    ensure!(penpot_status(&home)?["ok"] == true, "initial service drift");
    podman(&home, &["container", "stop", "--time", "5", &frontend])?;
    let occupied = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))?;
    let failed = invoke(&home, &["package", "start", "penpot"])?;
    ensure!(
        !failed.status.success(),
        "occupied frontend port was accepted"
    );
    ensure!(
        String::from_utf8_lossy(&failed.stderr).contains("prior running state"),
        "failed start did not report recovery"
    );
    let mut preserved = vec![];
    for alias in [
        "penpot-postgres",
        "penpot-valkey",
        "penpot-backend",
        "penpot-exporter",
        "penpot-frontend",
    ] {
        let name = format!("{owner}-{alias}");
        let r = podman(
            &home,
            &[
                "container",
                "inspect",
                &name,
                "--format",
                "{{.State.Running}}",
            ],
        )?;
        let running = r.trim() == "true";
        ensure!(
            running == (alias != "penpot-frontend"),
            "rollback changed prior service state: {alias}"
        );
        preserved.push(json!({"service":alias,"running":running}));
    }
    trace.push(json!({"occupied_port_failure":true,"prior_states_preserved":preserved}));
    drop(occupied);
    good(&home, &["package", "start", "penpot"])?;
    podman(
        &home,
        &["container", "update", "--restart=always", &frontend],
    )?;
    let drift = penpot_status(&home);
    let probe = invoke(
        &home,
        &[
            "package",
            "probe",
            "penpot",
            "--project",
            home.to_str().context("UTF8 home")?,
        ],
    )?;
    // Repair the fixture before assessing assertions; later suites need the service.
    podman(&home, &["container", "update", "--restart=no", &frontend])?;
    let drift = drift?;
    ensure!(
        drift["ok"] == false
            && drift["error"].as_str().is_some_and(
                |e| e.contains("isolation/resource configuration has drifted: penpot-frontend")
            ),
        "doctor did not diagnose the changed restart policy: {drift}"
    );
    ensure!(
        !probe.status.success()
            && String::from_utf8_lossy(&probe.stderr)
                .contains("isolation/resource configuration has drifted: penpot-frontend"),
        "launch did not diagnose the changed restart policy"
    );
    ensure!(
        penpot_status(&home)?["ok"] == true,
        "repair did not restore service integrity"
    );
    trace.push(json!({"same_owner_same_image_restart_drift_refused":true,"probe_refused":true,"repair_verified":true}));
    if let Some(path) = std::env::var_os("MIRKET_SERVICES_EVIDENCE_DIR") {
        let path = std::path::PathBuf::from(path);
        ensure!(
            path.is_absolute() && path.is_dir(),
            "absolute existing evidence directory required"
        );
        fs::write(
            path.join("service-live.json"),
            serde_json::to_vec_pretty(&trace)?,
        )?;
    }
    Ok(())
}
