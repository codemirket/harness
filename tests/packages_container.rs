//! Offline boundary probes. Fixtures are fixed programs in disposable homes,
//! not a daemon, image installer, or evidence of native-container readiness.
use std::{fs, path::Path, process::Command};

fn cli(home: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_mirket"))
        .arg("--home")
        .arg(home)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn packages_container_absent_podman_is_actionable_without_selecting_a_host_daemon() {
    let temporary = tempfile::tempdir().unwrap();
    let home = temporary.path().canonicalize().unwrap();
    let canary = home.join("unrelated-application-data");
    fs::write(&canary, b"user data stays unchanged").unwrap();
    let result = cli(&home, &["environment", "status"]);
    assert!(!result.status.success());
    let diagnostic = String::from_utf8_lossy(&result.stderr);
    assert!(
        diagnostic.contains("mirket tool register podman"),
        "{diagnostic}"
    );
    assert_eq!(fs::read(&canary).unwrap(), b"user data stays unchanged");
    assert!(!home.join(".mirket/execution/podman").exists());
}

#[cfg(unix)]
mod unix {
    use super::*;
    #[cfg(target_os = "macos")]
    use mirket::util;
    use mirket::{paths::Paths, podman::Engine};
    #[cfg(target_os = "macos")]
    use serde_json::{Value, json};
    #[cfg(target_os = "macos")]
    use std::os::unix::fs::MetadataExt;
    use std::{
        os::unix::fs::PermissionsExt,
        path::PathBuf,
        time::{Duration, Instant},
    };

    struct Fixture {
        _temporary: tempfile::TempDir,
        paths: Paths,
        binary: PathBuf,
        canary: PathBuf,
    }
    impl Fixture {
        fn new(body: &str) -> Self {
            let temporary = tempfile::tempdir().unwrap();
            let paths = Paths::new(temporary.path().canonicalize().unwrap()).unwrap();
            let binary = paths.home.join("fixture-podman");
            fs::write(&binary, format!("#!/bin/sh\n{body}\n")).unwrap();
            fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
            let canary = paths.home.join("unrelated-application-data");
            fs::write(&canary, b"private unrelated host data").unwrap();
            fs::set_permissions(&canary, fs::Permissions::from_mode(0o400)).unwrap();
            let registration = cli(
                &paths.home,
                &[
                    "tool",
                    "register",
                    "podman",
                    "--executable",
                    binary.to_str().unwrap(),
                ],
            );
            assert!(
                registration.status.success(),
                "{}",
                String::from_utf8_lossy(&registration.stderr)
            );
            Self {
                _temporary: temporary,
                paths,
                binary,
                canary,
            }
        }
        fn engine(&self) -> Engine {
            let mut engine = Engine::resolve(&self.paths).unwrap();
            // Fixture-only data; production Engine::resolve never inherits it.
            engine.env.insert(
                "MIRKET_TEST_FIXTURE".into(),
                self.paths.home.to_string_lossy().into_owned(),
            );
            engine
        }
        fn assert_canary(&self) {
            assert_eq!(
                fs::read(&self.canary).unwrap(),
                b"private unrelated host data"
            );
            assert_eq!(
                fs::metadata(&self.canary).unwrap().permissions().mode() & 0o777,
                0o400
            );
        }
        #[cfg(target_os = "macos")]
        fn machine(&self, engine: &Engine) -> (PathBuf, Value, PathBuf, Value) {
            let config_root = PathBuf::from(&engine.env["XDG_CONFIG_HOME"]);
            let config = config_root
                .join("containers/podman/machine/applehv")
                .join(format!("{}.json", engine.connection));
            let identity = PathBuf::from(&engine.env["XDG_DATA_HOME"])
                .join("containers/podman/machine/machine");
            fs::create_dir_all(identity.parent().unwrap()).unwrap();
            fs::write(&identity, b"private fixture identity, not a credential").unwrap();
            let uid = fs::metadata(&self.paths.home).unwrap().uid();
            let machine = json!({
                "Name":engine.connection,
                "HostUser":{"Rootful":false,"UID":uid},
                "Mounts":[{"Source":engine.staging,"Target":engine.staging,"ReadOnly":false,"Type":"virtiofs","Tag":"0123456789abcdef0123456789abcdef"}],
                "SSH":{"IdentityPath":identity,"Port":42222,"RemoteUsername":"core"},
                "Resources":{"CPUs":4,"Memory":6144,"DiskSize":40,"USBs":[]},
                "Rosetta":false,
                "ImagePath":{"Path":PathBuf::from(&engine.env["XDG_DATA_HOME"]).join(format!("containers/podman/machine/applehv/{}-arm64.raw",engine.connection))},
                // Shape observed from the managed Podman6.1.3 AppleHV config.
                // This is an offline refusal fixture, not an executed VM.
                "AppleHypervisor":{"Vfkit":{
                    "LogLevel":"panic","Endpoint":"http://localhost:54979","BinaryPath":null,"Rosetta":false,
                    "VirtualMachine":{"vcpus":4,"memoryBytes":6442450944u64,
                        "bootloader":{"kind":"efiBootloader","createVariableStore":true,
                            "efiVariableStorePath":PathBuf::from(&engine.env["XDG_DATA_HOME"]).join(format!("containers/podman/machine/applehv/efi-bl-{}",engine.connection))}}
                }}
            });
            let connection_path = config_root.join("containers/podman-connections.json");
            let connections = json!({"Connection":{"Default":engine.connection,"Connections":{
                engine.connection.clone():{
                    "URI":format!("ssh://core@127.0.0.1:42222/run/user/{uid}/podman/podman.sock"),
                    "Identity":identity,"IsMachine":true
                }
            }},"Farm":{}});
            write_json(&config, &machine);
            write_json(&connection_path, &connections);
            (config, machine, connection_path, connections)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            // Engine's short Unix-socket path is also owned by this unique home.
            // A TempDir alone cannot clean that macOS-specific sibling location.
            #[cfg(target_os = "macos")]
            {
                let short = PathBuf::from("/private/tmp").join(format!(
                    "mirket-{}",
                    &util::sha256(self.paths.root.to_string_lossy().as_bytes())[..16]
                ));
                if short.exists() {
                    fs::remove_dir_all(short).unwrap();
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn write_json(path: &Path, value: &Value) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
    }

    #[test]
    fn packages_container_changed_registered_client_is_refused_before_execution() {
        let fixture = Fixture::new("printf executed > \"$(dirname \"$0\")/executed\"");
        fs::write(&fixture.binary, b"#!/bin/sh\nprintf changed\n").unwrap();
        let result = cli(&fixture.paths.home, &["environment", "status"]);
        assert!(!result.status.success());
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(error.contains("executable changed"), "{error}");
        assert!(!fixture.paths.home.join("executed").exists());
        assert!(!fixture.paths.root.join("execution/podman").exists());
        fixture.assert_canary();
    }

    #[test]
    fn packages_container_control_paths_are_private_and_diagnostics_stay_separate() {
        let fixture = Fixture::new(
            "printf diagnostic >&2\nprintf '%s\\n' \"$HOME\" \"$XDG_CONFIG_HOME\" \"$XDG_DATA_HOME\" \"$XDG_CACHE_HOME\" \"$CONTAINERS_CONF\" \"$TMPDIR\" \"${MIRKET_TEST_UNRELATED_SECRET-unset}\"",
        );
        let engine = fixture.engine();
        assert_eq!(
            engine.staging,
            fixture.paths.root.join("execution/podman/staging")
        );
        assert!(!engine.env.contains_key("CONTAINER_HOST"));
        assert!(!engine.env.contains_key("DOCKER_HOST"));
        assert!(!engine.env.contains_key("SSH_AUTH_SOCK"));
        for name in [
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
            "CONTAINERS_CONF",
        ] {
            assert!(
                Path::new(&engine.env[name])
                    .starts_with(fixture.paths.root.join("execution/podman"))
            );
        }
        assert_ne!(Path::new(&engine.env["HOME"]), fixture.paths.home);
        let output = engine.raw(&[], Duration::from_secs(3)).unwrap();
        let actual = output.lines().collect::<Vec<_>>();
        assert_eq!(actual.len(), 7);
        for (index, name) in [
            "HOME",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
            "CONTAINERS_CONF",
            "TMPDIR",
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(actual[index], engine.env[*name]);
        }
        assert_eq!(actual[6], "unset");
        assert!(!output.contains("diagnostic"));
        assert_eq!(
            fs::metadata(&engine.staging).unwrap().permissions().mode() & 0o777,
            0o700
        );
        fixture.assert_canary();
    }

    #[test]
    fn packages_container_readiness_refuses_rootful_foreign_os_and_emulated_architecture() {
        let architecture = if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "amd64"
        };
        for (os, arch, rootless, expected) in [
            ("linux", architecture, false, "rootless Linux"),
            ("darwin", architecture, true, "rootless Linux"),
            (
                "linux",
                "unreviewed-architecture",
                true,
                "emulation is refused",
            ),
        ] {
            let info =
                serde_json::json!({"host":{"os":os,"arch":arch,"security":{"rootless":rootless}}});
            let fixture = Fixture::new(&format!("printf '%s\\n' '{info}'"));
            let engine = fixture.engine();
            #[cfg(target_os = "macos")]
            fixture.machine(&engine);
            let error = engine.ready().unwrap_err();
            assert!(error.to_string().contains(expected), "{error:#}");
            fixture.assert_canary();
        }
    }

    #[test]
    fn packages_container_symlinked_staging_cannot_select_unrelated_host_files() {
        let fixture = Fixture::new("exit 64");
        let staging = fixture.paths.root.join("execution/podman/staging");
        fs::create_dir_all(staging.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&fixture.canary, &staging).unwrap();
        let error = Engine::resolve(&fixture.paths).unwrap_err();
        assert!(error.to_string().contains("symlink"), "{error:#}");
        assert!(fs::symlink_metadata(&staging).unwrap().is_symlink());
        fixture.assert_canary();
    }

    #[cfg(target_os = "macos")]
    fn ready_fixture() -> Fixture {
        let architecture = if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "amd64"
        };
        let info = json!({"host":{"os":"linux","arch":architecture,"security":{"rootless":true}}});
        Fixture::new(&format!(
            "printf '%s\\n' \"$*\" >> \"$MIRKET_TEST_FIXTURE/calls\"\nprintf '%s\\n' '{}'",
            info
        ))
    }

    #[cfg(target_os = "macos")]
    fn control_vm(engine: &Engine, machine: &Value) -> Value {
        let runtime = PathBuf::from(&engine.env["TMPDIR"]).join("podman");
        json!({
            "vcpus":4,"memoryBytes":6442450944u64,
            "bootloader":machine["AppleHypervisor"]["Vfkit"]["VirtualMachine"]["bootloader"],
            "devices":[
                {"kind":"virtioblk","devName":"virtio-blk","imagePath":machine["ImagePath"]["Path"]},
                {"kind":"virtiorng"},
                {"kind":"virtiosock","port":1025,"socketURL":runtime.join(format!("{}.sock",engine.connection)),"listen":true},
                {"kind":"virtioserial","logFile":runtime.join(format!("{}.log",engine.connection))},
                {"kind":"virtionet","nat":false,"socket":{},"vfkitMagic":true,"macAddress":"5a:94:ef:e4:0c:ee"},
                {"kind":"virtiofs","mountTag":machine["Mounts"][0]["Tag"],"sharedDir":engine.staging},
                {"kind":"virtiosock","port":1024,"socketURL":PathBuf::from(&engine.env["HOME"]).join(".podman").join(format!("{}-ignition.sock",engine.connection)),"listen":true}
            ],"timesync":{"vsockPort":1234}
        })
    }

    #[cfg(target_os = "macos")]
    struct ControlListener {
        endpoint: String,
        thread: Option<std::thread::JoinHandle<String>>,
        stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }
    #[cfg(target_os = "macos")]
    impl ControlListener {
        fn new(status: &str, body: Vec<u8>, redirect: bool) -> Self {
            use std::io::{Read, Write};
            use std::sync::atomic::{AtomicBool, Ordering};
            let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
            let port = listener.local_addr().unwrap().port();
            listener.set_nonblocking(true).unwrap();
            let endpoint = format!("http://localhost:{port}");
            let header = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n",
                body.len(),
                if redirect {
                    format!("Location: {endpoint}/foreign-control\r\n")
                } else {
                    String::new()
                }
            );
            let stop = std::sync::Arc::new(AtomicBool::new(false));
            let server_stop = stop.clone();
            let thread = std::thread::spawn(move || {
                let deadline = Instant::now() + Duration::from_secs(10);
                let mut stream = loop {
                    match listener.accept() {
                        Ok((stream, _)) => break stream,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            if server_stop.load(Ordering::Relaxed) || Instant::now() >= deadline {
                                return "no request received".into();
                            }
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        Err(error) => panic!("control listener accept: {error}"),
                    }
                };
                // BSD accepted sockets may inherit the listener's nonblocking
                // mode. Give this fixed response fixture a bounded blocking read.
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                stream
                    .set_write_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut request = Vec::new();
                let mut buffer = [0u8; 1024];
                while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                    let count = stream.read(&mut buffer).unwrap();
                    assert_ne!(count, 0, "control request ended before its headers");
                    request.extend_from_slice(&buffer[..count]);
                    assert!(
                        request.len() <= 8192,
                        "control request headers exceed fixture limit"
                    );
                }
                stream.write_all(header.as_bytes()).unwrap();
                stream.write_all(&body).unwrap();
                String::from_utf8(request).unwrap()
            });
            Self {
                endpoint,
                thread: Some(thread),
                stop,
            }
        }
        fn assert_inspected(mut self) {
            let request = self.thread.take().unwrap().join().unwrap();
            assert!(
                request.starts_with("GET /vm/inspect HTTP/1.1\r\n"),
                "{request}"
            );
        }
    }
    #[cfg(target_os = "macos")]
    impl Drop for ControlListener {
        fn drop(&mut self) {
            self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn packages_container_stop_verifies_actual_owned_vm_before_native_mutation() {
        let fixture = ready_fixture();
        let engine = fixture.engine();
        let (config, mut machine, _, _) = fixture.machine(&engine);
        let response = control_vm(&engine, &machine);
        let listener =
            ControlListener::new("200 OK", serde_json::to_vec(&response).unwrap(), false);
        machine["AppleHypervisor"]["Vfkit"]["Endpoint"] = json!(listener.endpoint);
        write_json(&config, &machine);
        let result = engine.stop().unwrap();
        assert_eq!(result["stopped"], engine.connection);
        assert_eq!(result["data_preserved"], true);
        assert_eq!(
            fs::read_to_string(fixture.paths.home.join("calls")).unwrap(),
            format!("machine stop {}\n", engine.connection)
        );
        listener.assert_inspected();
        fixture.assert_canary();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn packages_container_stop_handles_subsequent_boot_without_ignition() {
        let fixture = ready_fixture();
        let engine = fixture.engine();
        let (config, mut machine, _, _) = fixture.machine(&engine);
        let mut response = control_vm(&engine, &machine);
        response["devices"].as_array_mut().unwrap().pop();
        let listener =
            ControlListener::new("200 OK", serde_json::to_vec(&response).unwrap(), false);
        machine["AppleHypervisor"]["Vfkit"]["Endpoint"] = json!(listener.endpoint);
        write_json(&config, &machine);
        assert_eq!(engine.stop().unwrap()["stopped"], engine.connection);
        assert_eq!(
            fs::read_to_string(fixture.paths.home.join("calls")).unwrap(),
            format!("machine stop {}\n", engine.connection)
        );
        listener.assert_inspected();
        fixture.assert_canary();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn packages_container_stop_refuses_foreign_vm_redirect_and_oversized_identity() {
        let fixture = ready_fixture();
        let engine = fixture.engine();
        let (config, original, _, _) = fixture.machine(&engine);
        let actual = control_vm(&engine, &original);
        let mut responses = Vec::new();
        for pointer in [
            "/devices/0/imagePath",
            "/bootloader/efiVariableStorePath",
            "/devices/5/sharedDir",
            "/devices/2/socketURL",
            "/devices/6/socketURL",
            "/devices/3/logFile",
        ] {
            let mut foreign = actual.clone();
            *foreign.pointer_mut(pointer).unwrap() = json!(fixture.canary);
            responses.push((
                pointer,
                "200 OK",
                serde_json::to_vec(&foreign).unwrap(),
                false,
            ));
        }
        let mut foreign_tag = actual.clone();
        foreign_tag["devices"][5]["mountTag"] = json!("another-machine-tag");
        responses.push((
            "mount tag",
            "200 OK",
            serde_json::to_vec(&foreign_tag).unwrap(),
            false,
        ));
        let mut resumed = actual.clone();
        resumed["devices"].as_array_mut().unwrap().pop();
        for pointer in [
            "/devices/0/imagePath",
            "/devices/5/sharedDir",
            "/devices/2/socketURL",
        ] {
            let mut foreign = resumed.clone();
            *foreign.pointer_mut(pointer).unwrap() = json!(fixture.canary);
            responses.push((
                pointer,
                "200 OK",
                serde_json::to_vec(&foreign).unwrap(),
                false,
            ));
        }
        for (case, device) in [
            ("duplicate runtime socket", resumed["devices"][2].clone()),
            ("duplicate ignition socket", actual["devices"][6].clone()),
            (
                "unexpected device",
                json!({"kind":"virtiofs","sharedDir":fixture.canary}),
            ),
        ] {
            let mut foreign = actual.clone();
            foreign["devices"].as_array_mut().unwrap().push(device);
            responses.push((case, "200 OK", serde_json::to_vec(&foreign).unwrap(), false));
        }
        responses.push(("redirect", "302 Found", b"{}".to_vec(), true));
        responses.push(("oversized", "200 OK", vec![b' '; 16 * 1024 + 1], false));
        for (case, status, body, redirect) in responses {
            let listener = ControlListener::new(status, body, redirect);
            let mut machine = original.clone();
            machine["AppleHypervisor"]["Vfkit"]["Endpoint"] = json!(listener.endpoint);
            write_json(&config, &machine);
            let error = engine.stop().unwrap_err();
            assert!(
                error.to_string().contains("no stop request sent"),
                "{case}: {error:#}"
            );
            assert!(
                !fixture.paths.home.join("calls").exists(),
                "{case}: native stop must never be called"
            );
            listener.assert_inspected();
            fixture.assert_canary();
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn packages_container_stop_refuses_unavailable_vm_without_claiming_success() {
        let fixture = ready_fixture();
        let engine = fixture.engine();
        let (config, mut machine, _, _) = fixture.machine(&engine);
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        machine["AppleHypervisor"]["Vfkit"]["Endpoint"] = json!(format!(
            "http://localhost:{}",
            listener.local_addr().unwrap().port()
        ));
        write_json(&config, &machine);
        drop(listener);
        let error = engine.stop().unwrap_err();
        assert!(
            error
                .to_string()
                .contains("VM control unavailable; no stop request sent"),
            "{error:#}"
        );
        assert!(!fixture.paths.home.join("calls").exists());
        fixture.assert_canary();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn packages_container_machine_shares_refuse_home_mounts_and_contract_drift() {
        let fixture = ready_fixture();
        let engine = fixture.engine();
        let (config, original, _, _) = fixture.machine(&engine);
        assert!(engine.ready().is_ok());
        let calls = fixture.paths.home.join("calls");
        let mut expanded = original.clone();
        expanded["Mounts"].as_array_mut().unwrap().push(
            json!({"Source":fixture.paths.home,"Target":"/host","ReadOnly":true,"Type":"virtiofs"}),
        );
        let mut wrong_source = original.clone();
        wrong_source["Mounts"][0]["Source"] = json!(fixture.paths.home);
        let mut wrong_target = original.clone();
        wrong_target["Mounts"][0]["Target"] = json!("/host");
        let mut wrong_type = original.clone();
        wrong_type["Mounts"][0]["Type"] = json!("9p");
        let mut wrong_mode = original.clone();
        wrong_mode["Mounts"][0]["ReadOnly"] = json!(true);
        let mut wrong_name = original.clone();
        wrong_name["Name"] = json!("unrelated-machine");
        let mut rootful = original.clone();
        rootful["HostUser"]["Rootful"] = json!(true);
        for value in [
            expanded,
            wrong_source,
            wrong_target,
            wrong_type,
            wrong_mode,
            wrong_name,
            rootful,
        ] {
            fs::remove_file(&calls).unwrap();
            write_json(&config, &value);
            let error = engine.ready().unwrap_err();
            assert!(error.to_string().contains("drifted"), "{error:#}");
            assert!(!calls.exists(), "refusal must precede any engine API call");
            write_json(&config, &original);
            engine.ready().unwrap();
            fixture.assert_canary();
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn packages_container_connection_refuses_another_daemon_or_identity() {
        let fixture = ready_fixture();
        let engine = fixture.engine();
        let (_, _, connection_path, original) = fixture.machine(&engine);
        engine.ready().unwrap();
        let calls = fixture.paths.home.join("calls");
        let mut other_uri = original.clone();
        other_uri["Connection"]["Connections"][&engine.connection]["URI"] =
            json!("ssh://core@127.0.0.1:42223/run/user/501/podman/podman.sock");
        let mut other_identity = original.clone();
        other_identity["Connection"]["Connections"][&engine.connection]["Identity"] =
            json!(fixture.canary);
        let mut user_connection = original.clone();
        user_connection["Connection"]["Connections"][&engine.connection]["IsMachine"] =
            json!(false);
        let mut missing = original.clone();
        missing["Connection"]["Connections"]
            .as_object_mut()
            .unwrap()
            .remove(&engine.connection);
        for value in [other_uri, other_identity, user_connection, missing] {
            fs::remove_file(&calls).unwrap();
            write_json(&connection_path, &value);
            let error = engine.ready().unwrap_err();
            assert!(
                error.to_string().contains("connection has drifted"),
                "{error:#}"
            );
            assert!(
                !calls.exists(),
                "connection refusal must precede the engine API"
            );
            write_json(&connection_path, &original);
            engine.ready().unwrap();
            fixture.assert_canary();
        }
    }

    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn packages_container_cached_engine_refuses_drift_before_worker_creation() {
        let fixture = ready_fixture();
        let engine = fixture.engine();
        let (config, original_machine, connections, original_connections) =
            fixture.machine(&engine);
        engine.ready().unwrap();
        let calls = fixture.paths.home.join("calls");
        for connection_drift in [false, true] {
            fs::remove_file(&calls).unwrap();
            if connection_drift {
                let mut changed = original_connections.clone();
                changed["Connection"]["Connections"][&engine.connection]["URI"] =
                    json!("ssh://core@127.0.0.1:42223/run/user/501/podman/podman.sock");
                write_json(&connections, &changed);
            } else {
                let mut changed = original_machine.clone();
                changed["Mounts"][0]["Source"] = json!(fixture.paths.home);
                write_json(&config, &changed);
            }
            let error = engine.run_async(&["run".into()]).await.unwrap_err();
            assert!(error.to_string().contains("drifted"), "{error:#}");
            assert!(
                !calls.exists(),
                "drift refusal must precede worker creation"
            );
            write_json(&config, &original_machine);
            write_json(&connections, &original_connections);
            engine.run_async(&["run".into()]).await.unwrap();
            assert!(fs::read_to_string(&calls).unwrap().contains("run"));
            fixture.assert_canary();
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn packages_container_identity_equality_cannot_select_shared_foreign_credentials() {
        let fixture = ready_fixture();
        let engine = fixture.engine();
        let (config, machine, connection_path, connections) = fixture.machine(&engine);
        engine.ready().unwrap();
        let calls = fixture.paths.home.join("calls");
        for replacement in [json!(fixture.canary), Value::Null] {
            fs::remove_file(&calls).unwrap();
            let mut changed_machine = machine.clone();
            let mut changed_connections = connections.clone();
            changed_machine["SSH"]["IdentityPath"] = replacement.clone();
            changed_connections["Connection"]["Connections"][&engine.connection]["Identity"] =
                replacement;
            write_json(&config, &changed_machine);
            write_json(&connection_path, &changed_connections);
            let error = engine.ready().unwrap_err();
            assert!(error.to_string().contains("identity"), "{error:#}");
            assert!(
                !calls.exists(),
                "matching foreign fields cannot bypass private identity ownership"
            );
            write_json(&config, &machine);
            write_json(&connection_path, &connections);
            engine.ready().unwrap();
            fixture.assert_canary();
        }
        fs::remove_file(&calls).unwrap();
        let identity = PathBuf::from(machine["SSH"]["IdentityPath"].as_str().unwrap());
        fs::remove_file(&identity).unwrap();
        std::os::unix::fs::symlink(&fixture.canary, &identity).unwrap();
        assert!(engine.ready().unwrap_err().to_string().contains("symlink"));
        assert!(!calls.exists());
        fixture.assert_canary();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn packages_container_coordinated_uid_and_connection_drift_is_refused() {
        let fixture = ready_fixture();
        let engine = fixture.engine();
        let (config, machine, connection_path, connections) = fixture.machine(&engine);
        engine.ready().unwrap();
        let calls = fixture.paths.home.join("calls");
        fs::remove_file(&calls).unwrap();
        let mut changed_machine = machine.clone();
        let mut changed_connections = connections;
        changed_machine["HostUser"]["UID"] = json!(0);
        changed_connections["Connection"]["Connections"][&engine.connection]["URI"] =
            json!(format!(
                "ssh://core@127.0.0.1:{}/run/user/0/podman/podman.sock",
                machine["SSH"]["Port"].as_u64().unwrap()
            ));
        write_json(&config, &changed_machine);
        write_json(&connection_path, &changed_connections);
        let error = engine.ready().unwrap_err();
        assert!(error.to_string().contains("staging owner"), "{error:#}");
        assert!(
            !calls.exists(),
            "matching guest UID and URI fields cannot select another user service"
        );
        fixture.assert_canary();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn packages_container_vm_disk_helpers_resources_and_emulation_are_pinned() {
        let fixture = ready_fixture();
        let engine = fixture.engine();
        let (config, original, _, _) = fixture.machine(&engine);
        engine.ready().unwrap();
        let calls = fixture.paths.home.join("calls");
        let cases = [
            ("/ImagePath/Path", json!(fixture.canary)),
            ("/AppleHypervisor/Vfkit/BinaryPath", json!(fixture.binary)),
            (
                "/AppleHypervisor/Vfkit/VirtualMachine/bootloader/efiVariableStorePath",
                json!(fixture.canary),
            ),
            (
                "/AppleHypervisor/Vfkit/VirtualMachine/bootloader/kind",
                json!("linuxBootloader"),
            ),
            ("/AppleHypervisor/Vfkit/VirtualMachine/vcpus", json!(8)),
            (
                "/AppleHypervisor/Vfkit/VirtualMachine/memoryBytes",
                json!(8589934592u64),
            ),
            ("/AppleHypervisor/Vfkit/Rosetta", json!(true)),
            ("/Resources/CPUs", json!(8)),
            ("/Resources/Memory", json!(8192)),
            ("/Resources/DiskSize", json!(400)),
            ("/Resources/USBs", json!([{"Vendor":1,"Product":2}])),
            ("/SSH/RemoteUsername", json!("root")),
            ("/Rosetta", json!(true)),
            (
                "/AppleHypervisor/Vfkit/Endpoint",
                json!("http://0.0.0.0:54979"),
            ),
            (
                "/AppleHypervisor/Vfkit/Endpoint",
                json!("http://127.0.0.1:54979"),
            ),
            (
                "/AppleHypervisor/Vfkit/Endpoint",
                json!("https://localhost:54979"),
            ),
            (
                "/AppleHypervisor/Vfkit/Endpoint",
                json!("http://user:password@localhost:54979"),
            ),
            (
                "/AppleHypervisor/Vfkit/Endpoint",
                json!("http://localhost:80"),
            ),
            ("/AppleHypervisor/Vfkit/Endpoint", json!("http://localhost")),
            (
                "/AppleHypervisor/Vfkit/Endpoint",
                json!("http://localhost:54979/control"),
            ),
            (
                "/AppleHypervisor/Vfkit/Endpoint",
                json!("http://localhost:54979/?other=service"),
            ),
            (
                "/AppleHypervisor/Vfkit/Endpoint",
                json!("http://localhost:54979/#other-service"),
            ),
        ];
        for (pointer, value) in cases {
            fs::remove_file(&calls).unwrap();
            let mut changed = original.clone();
            *changed.pointer_mut(pointer).unwrap() = value;
            write_json(&config, &changed);
            let error = engine.ready().unwrap_err();
            let expected = if pointer.ends_with("/Endpoint") {
                "private loopback"
            } else {
                "drifted"
            };
            assert!(error.to_string().contains(expected), "{pointer}: {error:#}");
            assert!(
                !calls.exists(),
                "{pointer}: refusal must precede engine execution"
            );
            write_json(&config, &original);
            engine.ready().unwrap();
            fixture.assert_canary();
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn packages_container_recovery_removes_only_a_late_owned_create() {
        let architecture = if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "amd64"
        };
        let info = json!({"host":{"os":"linux","arch":architecture,"security":{"rootless":true}}});
        let fixture = Fixture::new(&format!(
            "if [ \"$1\" = --connection ]; then shift 2; fi\nprintf '%s\\n' \"$*\" >> \"$MIRKET_TEST_FIXTURE/calls\"\ncase \"$1 $2\" in\n 'info --format=json') printf '%s\\n' '{info}';;\n 'container ls') /bin/cat \"$MIRKET_TEST_FIXTURE/list\";;\n 'container inspect') /bin/cat \"$MIRKET_TEST_FIXTURE/$3.json\";;\n 'rm --force') [ \"$3\" = --ignore ] && [ \"$#\" = 4 ] || exit 64; printf '%s\\n' \"$4\" >> \"$MIRKET_TEST_FIXTURE/removed\";;\n *) exit 64;;\nesac"
        ));
        let engine = fixture.engine();
        fixture.machine(&engine);
        let list = fixture.paths.home.join("list");
        fs::write(&list, b"").unwrap();
        // The remote create is absent when cancellation checks it. It appears
        // later after the owned host stage has been removed.
        assert_eq!(engine.recover().unwrap()["recovered"], json!([]));
        let make = |id: &str, token: &str| json!({"Id":id,"Name":format!("{}-{token}",engine.owner()),"Config":{"User":"1000:1000","Labels":{"org.mirket.execution-owner":engine.owner(),"org.mirket.execution-kind":"offline-render"}},"HostConfig":{"ReadonlyRootfs":true,"NetworkMode":"none"},"Mounts":[{"Destination":"/input","Source":engine.staging.join(token).join("input"),"Type":"bind","RW":false}]});
        let ids = [
            "a".repeat(64),
            "b".repeat(64),
            "c".repeat(64),
            "d".repeat(64),
            "e".repeat(64),
            "f".repeat(64),
        ];
        let late = make(&ids[0], "worker-late123");
        let active = make(&ids[1], "worker-active123");
        fs::create_dir_all(engine.staging.join("worker-active123/input")).unwrap();
        let mut customized = make(&ids[2], "worker-custom123");
        customized["Name"] = json!("unrelated-application");
        let mut foreign_owner = make(&ids[3], "worker-foreign123");
        foreign_owner["Config"]["Labels"]["org.mirket.execution-owner"] = json!("another-owner");
        let unrelated_inputs = fixture
            .paths
            .home
            .join("unrelated-application-inputs/input");
        fs::create_dir_all(&unrelated_inputs).unwrap();
        fs::write(
            unrelated_inputs.join("document"),
            b"preserve unrelated application input",
        )
        .unwrap();
        let mut foreign_mount = make(&ids[4], "worker-mount123");
        foreign_mount["Mounts"][0]["Source"] = json!(unrelated_inputs);
        let incomplete_stage = make(&ids[5], "worker-preparing123");
        fs::create_dir(engine.staging.join("worker-preparing123")).unwrap();
        for (id, value) in ids.iter().zip([
            late,
            active,
            customized,
            foreign_owner,
            foreign_mount,
            incomplete_stage,
        ]) {
            write_json(
                &fixture.paths.home.join(format!("{id}.json")),
                &json!([value]),
            );
        }
        fs::write(&list, format!("{}\n", ids.join("\n"))).unwrap();
        let result = engine.recover().unwrap();
        assert_eq!(result["recovered"], json!([ids[0]]));
        assert_eq!(
            fs::read_to_string(fixture.paths.home.join("removed")).unwrap(),
            format!("{}\n", ids[0])
        );
        assert!(engine.staging.join("worker-active123/input").exists());
        assert!(engine.staging.join("worker-preparing123").exists());
        assert_eq!(
            fs::read(unrelated_inputs.join("document")).unwrap(),
            b"preserve unrelated application input"
        );
        fixture.assert_canary();
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    #[test]
    fn packages_container_init_refuses_unpinned_machine_image_without_running_podman() {
        let fixture = ready_fixture();
        let engine = fixture.engine();
        let image = fixture.paths.home.join("unreviewed-boot-image");
        fs::write(&image, b"not the approved boot image").unwrap();
        let error = engine.init(&image).unwrap_err();
        assert!(
            error.to_string().contains("reviewed ARM64 pin"),
            "{error:#}"
        );
        assert!(!fixture.paths.home.join("calls").exists());
        fixture.assert_canary();
    }

    struct Descendant {
        pid: Option<u32>,
    }
    impl Descendant {
        fn read(path: &Path) -> Self {
            Self {
                pid: Some(fs::read_to_string(path).unwrap().trim().parse().unwrap()),
            }
        }
        fn exists(&self) -> bool {
            Command::new("/bin/kill")
                .args(["-0", &self.pid.unwrap().to_string()])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .unwrap()
                .success()
        }
        fn assert_gone(mut self) {
            let deadline = Instant::now() + Duration::from_secs(2);
            while self.exists() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
            assert!(
                !self.exists(),
                "metadata operation left its descendant running"
            );
            self.pid = None;
        }
    }
    impl Drop for Descendant {
        fn drop(&mut self) {
            if let Some(pid) = self.pid {
                // Failure cleanup targets only the PID written by our fixed fixture.
                let _ = Command::new("/bin/kill")
                    .args(["-KILL", &pid.to_string()])
                    .status();
            }
        }
    }

    #[test]
    fn packages_container_metadata_failures_terminate_diagnostic_descendants() {
        for mode in ["pipe-held", "deadline", "overflow"] {
            let timeout = if mode == "deadline" {
                Duration::from_secs(1)
            } else {
                Duration::from_secs(4)
            };
            let mut attempt = 0;
            let (fixture, result) = loop {
                attempt += 1;
                let fixture = Fixture::new(
                    "/bin/sleep 20 &\nprintf '%s' \"$!\" > \"$MIRKET_TEST_FIXTURE/descendant.pid\"\ncase \"$1\" in\n pipe-held) printf complete; exit 0;;\n deadline) wait;;\n overflow) /usr/bin/head -c 2097153 /dev/zero;;\n *) exit 64;;\nesac",
                );
                let result = fixture.engine().raw(&[mode.into()], timeout);
                if fixture.paths.home.join("descendant.pid").exists() {
                    break (fixture, result);
                }
                // The call's one-second deadline includes process startup. A
                // busy host may expire it before the fixture writes its PID.
                // Retry only that unstarted probe, in a fresh disposable home;
                // any confirmed descendant still has to pass cleanup below.
                assert!(
                    mode == "deadline"
                        && result
                            .as_ref()
                            .is_err_and(|error| error.to_string().contains("exceeded")),
                    "{mode}: unexpected failure before fixture startup; attempt={attempt}, result={result:?}"
                );
                assert!(
                    attempt < 3,
                    "{mode}: fixture never confirmed startup in three bounded attempts; result={result:?}"
                );
                fixture.assert_canary();
            };
            // Read/own the child before assertions, so a failing regression still
            // terminates the exact fixture process and leaves no task scratch.
            let child = Descendant::read(&fixture.paths.home.join("descendant.pid"));
            let error = result.unwrap_err();
            let expected = match mode {
                "pipe-held" => "pipes did not close",
                "deadline" => "exceeded",
                "overflow" => "exceed 2 MiB",
                _ => unreachable!(),
            };
            assert!(error.to_string().contains(expected), "{mode}: {error:#}");
            child.assert_gone();
            fixture.assert_canary();
        }
    }
}
