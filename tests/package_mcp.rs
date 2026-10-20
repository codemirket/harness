use mirket::package_mcp::{DEFAULT_REQUEST_TIMEOUT, LaunchSpec, call, probe, probe_with_schemas};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

fn fixture_spec(root: &Path) -> LaunchSpec {
    LaunchSpec {
        startup_timeout: std::time::Duration::from_secs(30),
        request_timeout: DEFAULT_REQUEST_TIMEOUT,
        tool_timeouts: BTreeMap::new(),
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
async fn package_mcp_relative_command_fails_before_spawn() {
    let home = tempfile::tempdir().unwrap();
    let mut spec = fixture_spec(home.path());
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
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };

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

    fn unresponsive_spec(root: &Path) -> LaunchSpec {
        let mut spec = fixture_spec(root);
        spec.command = "/bin/sh".into();
        spec.args = vec![
            "-c".into(),
            "trap '' TERM\necho $$ > child.pid\nwhile :; do /bin/sleep 1; done".into(),
        ];
        spec
    }

    async fn graceful_native_fixture(root: &Path) -> (LaunchSpec, tokio::process::Child) {
        // The native job deliberately has a different group, as a renderer
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
    async fn package_mcp_real_stdio_startup_deadline_covers_tools_list() {
        let home = tempfile::tempdir().unwrap();
        let command = home.path().join("hanging-package");
        let script = concat!(
            "#!/bin/sh\n",
            "set -eu\n",
            "while IFS= read -r line; do\n",
            // This fixture consumes the pinned client's compact JSON and
            // numeric request IDs. Shell builtins keep helper process startup
            // out of the short deadline intended to exercise tools/list.
            "    case \"$line\" in\n",
            "    *'\"method\":\"initialize\"'*)\n",
            "        id=${line#*'\"id\":'}\n",
            "        id=${id%%[!0-9]*}\n",
            "        [ -n \"$id\" ]\n",
            "        printf '{\"jsonrpc\":\"2.0\",\"id\":%s,\"result\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{\"tools\":{}},\"serverInfo\":{\"name\":\"fixture\",\"version\":\"1\"}}}\\n' \"$id\"\n",
            "        ;;\n",
            "    *'\"method\":\"tools/list\"'*)\n",
            "        : > \"$MARKER\"\n",
            "        while IFS= read -r ignored; do :; done\n",
            "        exit 0\n",
            "        ;;\n",
            "    esac\n",
            "done\n",
        );
        fs::write(&command, script).unwrap();
        fs::set_permissions(&command, fs::Permissions::from_mode(0o700)).unwrap();
        let marker = home.path().join("tools-list-seen");
        let mut spec = fixture_spec(home.path());
        spec.command = command;
        spec.args.clear();
        spec.startup_timeout = Duration::from_millis(500);
        spec.env
            .insert("MARKER".into(), marker.to_string_lossy().into_owned());
        let started = Instant::now();
        let error = tokio::time::timeout(Duration::from_secs(2), probe(spec))
            .await
            .expect("hanging discovery must stay within the outer test bound")
            .unwrap_err();
        assert!(marker.exists(), "the real child must receive tools/list");
        assert!(format!("{error:#}").contains("startup deadline exceeded"));
        assert!(
            started.elapsed() < Duration::from_millis(1500),
            "startup discovery exceeded its configured deadline: {:?}",
            started.elapsed()
        );
    }

    #[tokio::test]
    async fn package_mcp_aborted_startup_allows_parent_to_clean_its_native_job() {
        let home = tempfile::tempdir().unwrap();
        let (spec, mut native) = graceful_native_fixture(home.path()).await;
        let task = tokio::spawn(probe(spec));
        let parent = await_parent_pid(home.path()).await;
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                if fs::read(home.path().join("native-cleaned"))
                    .map(|bytes| bytes == b"cleaned")
                    .unwrap_or(false)
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("the blocking reaper must eventually signal the package parent");
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

    #[test]
    fn package_mcp_aborted_reaper_does_not_block_executor_progress() {
        let home = tempfile::tempdir().unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let (parent, abort_elapsed) = runtime.block_on(async {
            let task = tokio::spawn(probe(unresponsive_spec(home.path())));
            let parent = await_parent_pid(home.path()).await;
            task.abort();
            let started = Instant::now();
            assert!(task.await.unwrap_err().is_cancelled());
            (parent, started.elapsed())
        });

        // ManagedChild::Drop waits for TERM/KILL cleanup on a blocking worker.
        // A synchronous Drop would prevent this timer from being polled until
        // the entire five-second grace interval had elapsed.
        assert!(
            abort_elapsed < Duration::from_secs(1),
            "aborting package startup blocked the executor for {:?}",
            abort_elapsed
        );
        runtime.block_on(async {
            tokio::time::timeout(
                Duration::from_millis(250),
                tokio::time::sleep(Duration::from_millis(20)),
            )
            .await
            .expect("aborting package startup must leave the executor responsive");
        });
        drop(runtime);
        let status = std::process::Command::new("/bin/kill")
            .args(["-0", parent.trim()])
            .output()
            .unwrap();
        assert!(!status.status.success());
    }

    #[test]
    fn package_mcp_runtime_shutdown_waits_for_aborted_reaper() {
        let home = tempfile::tempdir().unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let parent = runtime.block_on(async {
            let task = tokio::spawn(probe(unresponsive_spec(home.path())));
            let parent = await_parent_pid(home.path()).await;
            task.abort();
            assert!(task.await.unwrap_err().is_cancelled());
            parent
        });

        // Runtime::drop must retain ownership of the blocking reaper until it
        // has finished. If the join handle were detached from shutdown, the
        // child would still be alive at this point.
        drop(runtime);
        let status = std::process::Command::new("/bin/kill")
            .args(["-0", parent.trim()])
            .output()
            .unwrap();
        assert!(
            !status.status.success(),
            "runtime shutdown returned before the package child was reaped"
        );
    }
}
