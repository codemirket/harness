use mirket::package_mcp::{LaunchSpec, call, probe, probe_with_schemas};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

fn fixture_spec(root: &Path) -> LaunchSpec {
    LaunchSpec {
        startup_timeout: std::time::Duration::from_secs(30),
        command: env!("CARGO_BIN_EXE_mirket").into(),
        args: vec![
            "--home".into(),
            root.to_string_lossy().into_owned(),
            "mcp".into(),
            "serve".into(),
        ],
        env: BTreeMap::new(),
        cwd: root.to_path_buf(),
        allowed_tools: Some(BTreeSet::from(["catalog_search".into()])),
        denied_tools: BTreeSet::new(),
        input_policy: "none".into(),
        read_only_sqlite: None,
    }
}

#[tokio::test]
async fn package_mcp_real_child_initializes_filters_and_executes_a_useful_call() {
    let home = tempfile::tempdir().unwrap();
    let spec = fixture_spec(home.path());
    let observed = probe(spec.clone()).await.unwrap();
    assert_eq!(observed["tools"], json!(["catalog_search"]));
    assert!(observed.get("schemas").is_none());
    let contract = probe_with_schemas(spec.clone(), true).await.unwrap();
    assert_eq!(contract["schemas"][0]["name"], "catalog_search");
    assert!(contract["schemas"][0]["inputSchema"]["properties"]["query"].is_object());
    let result = call(
        spec.clone(),
        "catalog_search".into(),
        json!({"query":"engineering", "limit":3})
            .as_object()
            .unwrap()
            .clone(),
    )
    .await
    .unwrap();
    assert_ne!(result.is_error, Some(true));
    let value = result.structured_content.unwrap();
    assert!(value.to_string().contains("engineering"));
    let blocked = call(spec, "task_start".into(), Default::default())
        .await
        .unwrap_err();
    assert!(
        blocked
            .to_string()
            .contains("unavailable under this profile")
    );
}

#[tokio::test]
async fn package_mcp_unknown_policy_and_relative_command_fail_before_spawn() {
    let home = tempfile::tempdir().unwrap();
    let mut spec = fixture_spec(home.path());
    spec.input_policy = "typo".into();
    assert!(
        probe(spec.clone())
            .await
            .unwrap_err()
            .to_string()
            .contains("unknown package input policy")
    );
    spec.input_policy = "none".into();
    spec.command = "mirket".into();
    assert!(
        probe(spec)
            .await
            .unwrap_err()
            .to_string()
            .contains("absolute executable")
    );
    for duration in [
        std::time::Duration::ZERO,
        std::time::Duration::from_secs(121),
    ] {
        let mut spec = fixture_spec(home.path());
        spec.startup_timeout = duration;
        assert!(
            probe(spec)
                .await
                .unwrap_err()
                .to_string()
                .contains("startup timeout")
        );
    }
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::{fs, time::Duration};

    async fn assert_reaped(pid: &str) {
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                let status = tokio::process::Command::new("/bin/kill")
                    .args(["-0", pid])
                    .output()
                    .await
                    .unwrap();
                if !status.status.success() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("the managed fixture child must be terminated and reaped");
    }

    async fn await_parent_pid(root: &Path) -> String {
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                if let Ok(pid) = fs::read_to_string(root.join("child.pid"))
                    && !pid.trim().is_empty()
                {
                    break pid;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap()
    }

    async fn graceful_native_fixture(root: &Path) -> (LaunchSpec, tokio::process::Child) {
        // The native job deliberately has a different group, just as editor_mcp
        // jobs do. Killing only the immediate MCP group cannot stop this job.
        let native = tokio::process::Command::new("/bin/sleep")
            .arg("60")
            .process_group(0)
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let mut spec = fixture_spec(root);
        spec.command = "/bin/sh".into();
        spec.env.insert(
            "MIRKET_FIXTURE_NATIVE_PID".into(),
            native.id().unwrap().to_string(),
        );
        spec.args = vec!["-c".into(),
            "trap '/bin/kill -TERM \"$MIRKET_FIXTURE_NATIVE_PID\"; printf cleaned > native-cleaned; exit 0' TERM\necho $$ > child.pid\nwhile :; do /bin/sleep 1; done".into()];
        (spec, native)
    }

    #[tokio::test]
    async fn package_mcp_startup_deadline_allows_parent_to_clean_its_native_job() {
        let home = tempfile::tempdir().unwrap();
        let (mut spec, mut native) = graceful_native_fixture(home.path()).await;
        spec.startup_timeout = Duration::from_millis(250);
        let error = probe(spec).await.unwrap_err();
        assert!(format!("{error:#}").contains("startup deadline exceeded"));
        assert_eq!(
            fs::read(home.path().join("native-cleaned")).unwrap(),
            b"cleaned"
        );
        let parent = fs::read_to_string(home.path().join("child.pid")).unwrap();
        assert_reaped(parent.trim()).await;
        tokio::time::timeout(Duration::from_secs(3), native.wait())
            .await
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn package_mcp_aborted_startup_allows_parent_to_clean_its_native_job() {
        let home = tempfile::tempdir().unwrap();
        let (spec, mut native) = graceful_native_fixture(home.path()).await;
        let task = tokio::spawn(probe(spec));
        let parent = await_parent_pid(home.path()).await;
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        assert_eq!(
            fs::read(home.path().join("native-cleaned")).unwrap(),
            b"cleaned"
        );
        assert_reaped(parent.trim()).await;
        tokio::time::timeout(Duration::from_secs(3), native.wait())
            .await
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn package_mcp_unresponsive_startup_group_is_forcibly_reaped() {
        let home = tempfile::tempdir().unwrap();
        let mut spec = fixture_spec(home.path());
        spec.command = "/bin/sh".into();
        spec.args = vec![
            "-c".into(),
            "trap '' TERM\necho $$ > child.pid\n/bin/sleep 60 &\necho $! > descendant.pid\nwait"
                .into(),
        ];
        spec.startup_timeout = Duration::from_millis(250);
        let error = probe(spec).await.unwrap_err();
        assert!(format!("{error:#}").contains("startup deadline exceeded"));
        for name in ["child.pid", "descendant.pid"] {
            assert_reaped(fs::read_to_string(home.path().join(name)).unwrap().trim()).await;
        }
    }

    #[tokio::test]
    #[ignore = "requires MIRKET_FIXTURE_HOME and MIRKET_FIXTURE_PROJECT with jscpd installed through mirket"]
    async fn package_mcp_installed_jscpd_host_lifecycle() {
        use rmcp::{
            ClientLifecycleMode, ClientServiceExt,
            model::{CallToolRequestParams, ProtocolVersion},
            transport::TokioChildProcess,
        };
        let home =
            std::env::var("MIRKET_FIXTURE_HOME").expect("set an explicit disposable fixture home");
        let project = std::env::var("MIRKET_FIXTURE_PROJECT")
            .expect("set an explicit disposable fixture project");
        assert!(Path::new(&home).is_absolute() && Path::new(&project).is_absolute());
        for current_protocol in [false, true] {
            let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_mirket"));
            command
                .args([
                    "--home",
                    &home,
                    "package",
                    "serve",
                    "jscpd",
                    "--project",
                    &project,
                ])
                .kill_on_drop(true);
            let transport = TokioChildProcess::new(command).unwrap();
            let bridge_pid = transport.id().unwrap();
            let lifecycle = if current_protocol {
                ClientLifecycleMode::Discover {
                    preferred_versions: vec![ProtocolVersion::V_2026_07_28],
                }
            } else {
                ClientLifecycleMode::Initialize
            };
            let client = tokio::time::timeout(
                Duration::from_secs(20),
                ().serve_with_lifecycle(transport, lifecycle),
            )
            .await
            .unwrap()
            .unwrap();
            let processes = tokio::process::Command::new("/bin/ps")
                .args(["-axo", "pid=,ppid="])
                .output()
                .await
                .unwrap();
            assert!(processes.status.success());
            let child_pids: Vec<String> = String::from_utf8(processes.stdout)
                .unwrap()
                .lines()
                .filter_map(|line| {
                    let mut fields = line.split_whitespace();
                    let pid = fields.next()?;
                    let parent: u32 = fields.next()?.parse().ok()?;
                    (parent == bridge_pid).then(|| pid.to_owned())
                })
                .collect();
            assert_eq!(
                child_pids.len(),
                1,
                "the fixture must have one managed jscpd child"
            );
            if current_protocol {
                // A discovered host may wait longer than the child startup deadline
                // before its first regular request. This is still a live session.
                tokio::time::sleep(Duration::from_secs(31)).await;
            }
            let tools = tokio::time::timeout(Duration::from_secs(5), client.list_all_tools())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(tools.len(), 4);
            assert!(!tools.iter().any(|tool| tool.name == "compare_folders"));
            let result = tokio::time::timeout(
                Duration::from_secs(5),
                client.call_tool(CallToolRequestParams::new("get_statistics")),
            )
            .await
            .unwrap()
            .unwrap();
            assert_ne!(result.is_error, Some(true));
            assert!(result.structured_content.is_some());
            let denied = client
                .call_tool(CallToolRequestParams::new("compare_folders"))
                .await
                .unwrap();
            assert_eq!(denied.is_error, Some(true));
            if current_protocol {
                let status = tokio::process::Command::new("/bin/kill")
                    .args(["-TERM", &bridge_pid.to_string()])
                    .status()
                    .await
                    .unwrap();
                assert!(status.success());
            }
            client.cancel().await.unwrap();
            assert_reaped(&bridge_pid.to_string()).await;
            for pid in child_pids {
                assert_reaped(&pid).await;
            }
        }
    }

    #[tokio::test]
    async fn package_mcp_child_receives_only_explicit_environment() {
        let home = tempfile::tempdir().unwrap();
        let mut spec = fixture_spec(home.path());
        spec.command = "/bin/sh".into();
        spec.args = vec!["-c".into(), "/usr/bin/env > child-environment.txt".into()];
        spec.env.insert("MIRKET_FIXTURE".into(), "explicit".into());
        assert!(probe(spec).await.is_err());
        let environment = fs::read_to_string(home.path().join("child-environment.txt")).unwrap();
        assert!(
            environment
                .lines()
                .any(|line| line == "MIRKET_FIXTURE=explicit")
        );
        assert!(!environment.lines().any(|line| line.starts_with("HOME=")
            || line.starts_with("CODEX_HOME=")
            || line.starts_with("OPENAI_API_KEY=")));
    }

    #[tokio::test]
    async fn package_mcp_real_sdk_child_is_reaped_after_probe() {
        let home = tempfile::tempdir().unwrap();
        let mut spec = fixture_spec(home.path());
        let binary = spec.command.to_string_lossy().into_owned();
        let args = spec.args.clone();
        spec.command = "/bin/sh".into();
        spec.args = vec![
            "-c".into(),
            "echo $$ > child.pid; exec \"$@\"".into(),
            "fixture".into(),
            binary,
        ];
        spec.args.extend(args);
        probe(spec).await.unwrap();
        let pid = fs::read_to_string(home.path().join("child.pid")).unwrap();
        assert_reaped(pid.trim()).await;
    }

    #[tokio::test]
    async fn package_mcp_cancelled_startup_terminates_the_managed_child() {
        let home = tempfile::tempdir().unwrap();
        let mut spec = fixture_spec(home.path());
        spec.command = "/bin/sh".into();
        spec.args = vec![
            "-c".into(),
            "echo $$ > child.pid; exec /bin/sleep 60".into(),
        ];
        let task = tokio::spawn(probe(spec));
        let pid = tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                if let Ok(pid) = fs::read_to_string(home.path().join("child.pid"))
                    && !pid.trim().is_empty()
                {
                    break pid;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        assert_reaped(pid.trim()).await;
    }
}
