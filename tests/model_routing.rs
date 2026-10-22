use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

struct MockTarget {
    endpoint: String,
    models: Arc<Mutex<Vec<Value>>>,
    posts: Arc<Mutex<Vec<Value>>>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl MockTarget {
    fn new(models: Vec<Value>, responses: Vec<(u16, Value)>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let endpoint = format!("http://{}/v1", listener.local_addr().unwrap());
        let models = Arc::new(Mutex::new(models));
        let responses = Arc::new(Mutex::new(responses));
        let posts = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let (models_for_thread, responses_for_thread, posts_for_thread, stop_for_thread) = (
            Arc::clone(&models),
            Arc::clone(&responses),
            Arc::clone(&posts),
            Arc::clone(&stop),
        );
        let thread = thread::spawn(move || {
            while !stop_for_thread.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => serve(
                        stream,
                        &models_for_thread,
                        &responses_for_thread,
                        &posts_for_thread,
                    ),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("mock accept failed: {error}"),
                }
            }
        });
        Self {
            endpoint,
            models,
            posts,
            stop,
            thread: Some(thread),
        }
    }

    fn posts(&self) -> Vec<Value> {
        self.posts.lock().unwrap().clone()
    }
}

impl Drop for MockTarget {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let result = thread.join();
            if !std::thread::panicking() {
                result.unwrap();
            }
        }
    }
}

fn serve(
    mut stream: TcpStream,
    models: &Mutex<Vec<Value>>,
    responses: &Mutex<Vec<(u16, Value)>>,
    posts: &Mutex<Vec<Value>>,
) {
    // Accepted sockets can inherit the listener's nonblocking mode on macOS.
    stream.set_nonblocking(false).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut input = Vec::new();
    let mut chunk = [0u8; 8192];
    let headers_end = loop {
        let count = stream.read(&mut chunk).unwrap();
        assert!(count > 0, "mock request ended before headers");
        input.extend_from_slice(&chunk[..count]);
        assert!(input.len() <= 2 * 1024 * 1024, "mock request too large");
        if let Some(end) = input.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
            break end + 4;
        }
    };
    let headers = String::from_utf8(input[..headers_end].to_vec()).unwrap();
    let first = headers.lines().next().unwrap();
    let content_length = headers
        .lines()
        .find_map(|line| {
            line.split_once(':')
                .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                .map(|(_, value)| value.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    assert!(content_length <= 2 * 1024 * 1024);
    while input.len() - headers_end < content_length {
        let count = stream.read(&mut chunk).unwrap();
        assert!(count > 0, "mock request ended before body");
        input.extend_from_slice(&chunk[..count]);
        assert!(input.len() <= 2 * 1024 * 1024);
    }
    let (status, response) = if first.starts_with("GET /v1/models") {
        (200, json!({"data":models.lock().unwrap().clone()}))
    } else if first.starts_with("POST /v1/chat/completions")
        || first.starts_with("POST /v1/responses")
    {
        let request: Value =
            serde_json::from_slice(&input[headers_end..headers_end + content_length]).unwrap();
        posts.lock().unwrap().push(request);
        let mut queued = responses.lock().unwrap();
        assert!(!queued.is_empty(), "unexpected completion request");
        queued.remove(0)
    } else {
        panic!("unexpected mock request: {first}");
    };
    if status == 0 {
        // A body truncated by transport, not a completed model response with
        // invalid JSON. No action may be dispatched from these partial bytes.
        stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 1000\r\nConnection: close\r\n\r\n{\"partial\":").unwrap();
        stream.flush().unwrap();
        return;
    }
    let bytes = serde_json::to_vec(&response).unwrap();
    let reason = if status == 200 {
        "OK"
    } else {
        "Too Many Requests"
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        bytes.len()
    )
    .unwrap();
    stream.write_all(&bytes).unwrap();
    stream.flush().unwrap();
}

fn model(id: &str, description: &str) -> Value {
    json!({
        "id": id,
        "displayName": id,
        "description": description,
        "supports_tools": true,
        "supports_structured_outputs": true,
        "reasoning_efforts": ["high"],
        "context_length": 262144,
        "max_completion_tokens": 64000
    })
}

fn target(name: &str, server: &MockTarget) -> Value {
    json!({
        "name": name,
        "protocol": "openai_chat",
        "endpoint": server.endpoint,
        "credential_env": null,
        "executable": null
    })
}

fn configure(home: &Path, targets: Vec<Value>) {
    let root = home.join(".mirket");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("routing.json"),
        serde_json::to_vec(&json!({"ttl_seconds":300,"targets":targets})).unwrap(),
    )
    .unwrap();
}

fn request(home: &Path, project: &Path, assigned: &[&str], verification: &[&str]) -> PathBuf {
    let path = home.join("request.json");
    fs::write(
        &path,
        serde_json::to_vec(&json!({
            "objective": "Inspect the assigned workspace and produce a checked artifact with evidence.",
            "task_intent": "architecture",
            "complexity_class": "creative_architectural",
            "requested_tier": "T3_ARCHITECT",
            "deterministic_operation": {"action":"sort_lines"},
            "assigned_tools": assigned,
            "task_needs": [{"workflow":if assigned.contains(&"workspace_write") {"source-implementation"} else if assigned.contains(&"inspect_image") {"visual-review"} else if assigned.is_empty() {"structured-response"} else {"analysis"},"reason":"The fixture exercises the corresponding declared deliverable."}],
            "project": project,
            "requires_vision": false,
            "verification_tools": verification
        }))
        .unwrap(),
    )
    .unwrap();
    path
}

fn invoke(home: &Path, empty_path: &Path, args: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mirket"))
        .arg("--home")
        .arg(home)
        .args(args)
        .env("HOME", home)
        .env("PATH", empty_path)
        .env("MIRKET_DIRECT", "1")
        .env_remove("OPENAI_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("OPENROUTER_API_KEY")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            let output = child.wait_with_output().unwrap();
            panic!(
                "mirket CLI timed out: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn ok(home: &Path, empty_path: &Path, args: &[&str]) -> Value {
    let output = invoke(home, empty_path, args);
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn action(model: &str, action: Value) -> (u16, Value) {
    (
        200,
        json!({
            "model":model,
            "choices":[{
                "finish_reason":"stop",
                "message":{"content":action.to_string()}
            }]
        }),
    )
}

#[test]
fn responses_dispatches_only_final_answers_after_commentary() {
    let sandbox = tempfile::tempdir().unwrap();
    let home = sandbox.path().join("home");
    let project = sandbox.path().join("project");
    let empty_path = sandbox.path().join("empty-path");
    for dir in [&home, &project, &empty_path] {
        fs::create_dir_all(dir).unwrap();
    }
    fs::write(project.join("input.txt"), "checked").unwrap();
    let selected = "frontier-phased";
    let response = |action: Value| {
        (
            200,
            json!({"model":selected,"status":"completed","output":[
                {"type":"message","role":"assistant","phase":"commentary","content":[{"type":"output_text","text":json!({"action":"blocked","tool":"","arguments_json":"{}","result":"interim, never dispatch"}).to_string()}]},
                {"type":"message","role":"assistant","phase":"final_answer","content":[{"type":"output_text","text":action.to_string()}]}
            ]}),
        )
    };
    let server = MockTarget::new(
        vec![model(selected, "frontier model")],
        vec![
            response(
                json!({"action":"tool","tool":"workspace_read","arguments_json":"{\"path\":\"input.txt\"}","result":""}),
            ),
            response(
                json!({"action":"complete","tool":"","arguments_json":"{}","result":"read final action only"}),
            ),
        ],
    );
    let mut configured = target("mock", &server);
    configured["protocol"] = json!("openai_responses");
    configure(&home, vec![configured]);
    let request = request(&home, &project, &["workspace_read"], &["workspace_read"]);
    let result = ok(
        &home,
        &empty_path,
        &["agent", "spawn", "--request", request.to_str().unwrap()],
    );
    assert_eq!(result["result"], "read final action only");
    assert_eq!(server.posts().len(), 2);
}

#[test]
fn refresh_observes_new_models_without_published_ids_or_inference() {
    let sandbox = tempfile::tempdir().unwrap();
    let home = sandbox.path().join("home");
    let project = sandbox.path().join("project");
    let empty_path = sandbox.path().join("empty-path");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&empty_path).unwrap();
    let server = MockTarget::new(vec![model("frontier-alpha", "frontier model")], vec![]);
    configure(&home, vec![target("mock", &server)]);
    let first = ok(&home, &empty_path, &["models", "refresh"]);
    assert_eq!(first["models"].as_array().unwrap().len(), 1);
    server
        .models
        .lock()
        .unwrap()
        .push(model("frontier-beta", "frontier model"));
    let cached = ok(&home, &empty_path, &["models", "list"]);
    assert_eq!(cached["models"].as_array().unwrap().len(), 1);
    let refreshed = ok(&home, &empty_path, &["models", "refresh"]);
    let ids: Vec<_> = refreshed["models"]
        .as_array()
        .unwrap()
        .iter()
        .map(|model| model["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["frontier-alpha", "frontier-beta"]);
    let request = request(&home, &project, &[], &[]);
    let route = ok(
        &home,
        &empty_path,
        &["models", "route", "--request", request.to_str().unwrap()],
    );
    assert_eq!(route["execution"], "not_dispatched");
    assert_eq!(route["resolution"]["min_tier"], "T3_ARCHITECT");
    assert_eq!(route["candidates"].as_array().unwrap().len(), 2);
    assert!(server.posts().is_empty(), "route must not run inference");
}

#[test]
fn spawn_requires_post_write_verification_and_sends_high_effort() {
    let sandbox = tempfile::tempdir().unwrap();
    let home = sandbox.path().join("home");
    let project = sandbox.path().join("project");
    let empty_path = sandbox.path().join("empty-path");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&empty_path).unwrap();
    let selected = "frontier-checked";
    let server = MockTarget::new(
        vec![model(selected, "frontier model")],
        vec![
            action(
                selected,
                json!({"action":"tool","tool":"workspace_write","arguments_json":r#"{"path":"result.txt","content":"checked","expected_sha256":null}"#,"result":""}),
            ),
            action(
                selected,
                json!({"action":"complete","tool":"","arguments_json":"{}","result":"premature"}),
            ),
            action(
                selected,
                json!({"action":"tool","tool":"workspace_read","arguments_json":r#"{"path":"result.txt"}"#,"result":""}),
            ),
            action(
                selected,
                json!({"action":"complete","tool":"","arguments_json":"{}","result":"verified"}),
            ),
        ],
    );
    configure(&home, vec![target("mock", &server)]);
    let request = request(
        &home,
        &project,
        &["workspace_write", "workspace_read"],
        &["workspace_read"],
    );
    let result = ok(
        &home,
        &empty_path,
        &["agent", "spawn", "--request", request.to_str().unwrap()],
    );
    assert_eq!(result["status"], "awaiting_host_verification");
    assert_eq!(result["result"], "verified");
    assert_eq!(
        result["host_verification"]["artifacts"][0]["path"],
        "result.txt"
    );
    // Bind inputs before executing the actual check, then report its fresh log.
    let report = home.join("host-report.json");
    fs::write(&report, serde_json::to_vec(&json!({
        "project":project,"run_id":result["run_id"],"expected_trace_sha256":result["trace_sha256"],
        "begin":{"key":"first-host-check","inputs":["result.txt"],"commands":{"project_checks":"Rust assert result.txt equals checked"}}
    })).unwrap()).unwrap();
    let prepared = ok(
        &home,
        &empty_path,
        &["agent", "verify", "--request", report.to_str().unwrap()],
    );
    assert_eq!(fs::read(project.join("result.txt")).unwrap(), b"checked");
    let log = b"Host assertion: result.txt equals checked; passed";
    fs::write(project.join("check.log"), log).unwrap();
    fs::write(&report, serde_json::to_vec(&json!({
        "project":project,"run_id":result["run_id"],"expected_trace_sha256":prepared["trace_sha256"],
        "attempt_id":prepared["host_verification"]["active_attempt"],
        "checks":[{"name":"project_checks","status":"passed","summary":"Compared actual output with expected bytes", "evidence":{"path":"check.log","sha256":mirket::util::sha256(log)}}]
    })).unwrap()).unwrap();
    let verified = ok(
        &home,
        &empty_path,
        &["agent", "verify", "--request", report.to_str().unwrap()],
    );
    assert_eq!(verified["status"], "host_verified");
    let replay = ok(
        &home,
        &empty_path,
        &["agent", "verify", "--request", report.to_str().unwrap()],
    );
    assert_eq!(replay, verified);

    assert_eq!(
        result["task_contract"]["primary_role"],
        "backend-engineering"
    );
    assert_eq!(
        fs::read_to_string(project.join("result.txt")).unwrap(),
        "checked"
    );
    let posts = server.posts();
    let prompt: Value = serde_json::from_str(
        posts[0]["messages"][0]["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let guidance = &prompt["expertise_guidance"][0];
    let expected = mirket::catalog::Catalog::embedded()
        .unwrap()
        .read("backend-engineering", "SKILL.md", 16 * 1024)
        .unwrap();
    assert_eq!(guidance["id"], expected.id);
    assert_eq!(guidance["sha256"], expected.sha256);
    assert_eq!(
        guidance["content"], expected.content,
        "actual inference must receive the selected skill body, not only its receipt"
    );
    assert_eq!(
        posts.len(),
        4,
        "premature completion must not pass verification"
    );
    assert!(posts.iter().all(|body| body["model"] == selected));
    assert!(posts.iter().all(|body| body["reasoning_effort"] == "high"));
    assert!(
        posts
            .iter()
            .all(|body| body["max_completion_tokens"] == 8192)
    );
    assert!(
        posts[2]["messages"][0]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("verification_gate")
    );
    assert!(
        result["trace"]
            .as_str()
            .is_some_and(|path| Path::new(path).is_file())
    );
}

#[test]
fn unrelated_readback_cannot_verify_a_written_artifact() {
    let sandbox = tempfile::tempdir().unwrap();
    let home = sandbox.path().join("home");
    let project = sandbox.path().join("project");
    let empty_path = sandbox.path().join("empty-path");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&empty_path).unwrap();
    fs::write(project.join("unrelated.txt"), "unrelated").unwrap();
    let selected = "frontier-checked";
    let server = MockTarget::new(
        vec![model(selected, "frontier model")],
        vec![
            action(
                selected,
                json!({"action":"tool","tool":"workspace_write","arguments_json":r#"{"path":"result.txt","content":"checked","expected_sha256":null}"#,"result":""}),
            ),
            action(
                selected,
                json!({"action":"tool","tool":"workspace_read","arguments_json":r#"{"path":"unrelated.txt"}"#,"result":""}),
            ),
            action(
                selected,
                json!({"action":"complete","tool":"","arguments_json":"{}","result":"premature"}),
            ),
            action(
                selected,
                json!({"action":"tool","tool":"workspace_read","arguments_json":r#"{"path":"result.txt"}"#,"result":""}),
            ),
            action(
                selected,
                json!({"action":"complete","tool":"","arguments_json":"{}","result":"verified"}),
            ),
        ],
    );
    configure(&home, vec![target("mock", &server)]);
    let request = request(
        &home,
        &project,
        &["workspace_write", "workspace_read"],
        &["workspace_read"],
    );
    let result = ok(
        &home,
        &empty_path,
        &["agent", "spawn", "--request", request.to_str().unwrap()],
    );
    assert_eq!(result["result"], "verified");
    assert_eq!(server.posts().len(), 5);
    assert!(
        server.posts()[3]["messages"][0]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("unverified_artifacts")
    );
}

#[test]
fn inspected_pixels_survive_intervening_read_only_turns() {
    let sandbox = tempfile::tempdir().unwrap();
    let home = sandbox.path().join("home");
    let project = sandbox.path().join("project");
    let empty_path = sandbox.path().join("empty-path");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&empty_path).unwrap();
    // A complete one-pixel PNG; the model transport must retain its bytes.
    let png: &[u8] = &[
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6,
        0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 11, 73, 68, 65, 84, 120, 156, 99, 96, 0, 2, 0, 0, 5, 0,
        1, 122, 94, 171, 63, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
    ];
    fs::write(project.join("pixel.png"), png).unwrap();
    fs::write(
        project.join("notes.txt"),
        "Inspect the image and record its color.",
    )
    .unwrap();
    let selected = "frontier-vision";
    let mut visual_model = model(selected, "frontier model");
    visual_model["supports_vision"] = json!(true);
    let server = MockTarget::new(
        vec![visual_model],
        vec![
            action(
                selected,
                json!({"action":"tool","tool":"inspect_image","arguments_json":r#"{"path":"pixel.png"}"#,"result":""}),
            ),
            action(
                selected,
                json!({"action":"tool","tool":"workspace_read","arguments_json":r#"{"path":"notes.txt"}"#,"result":"The inspected pixel is visible."}),
            ),
            action(
                selected,
                json!({"action":"complete","tool":"","arguments_json":"{}","result":"Pixel inspected after reading notes"}),
            ),
        ],
    );
    configure(&home, vec![target("mock", &server)]);
    let request_path = request(
        &home,
        &project,
        &["inspect_image", "workspace_read"],
        &["inspect_image"],
    );
    let mut request_json: Value =
        serde_json::from_slice(&fs::read(&request_path).unwrap()).unwrap();
    request_json["requires_vision"] = json!(true);
    fs::write(&request_path, serde_json::to_vec(&request_json).unwrap()).unwrap();
    let result = ok(
        &home,
        &empty_path,
        &[
            "agent",
            "spawn",
            "--request",
            request_path.to_str().unwrap(),
        ],
    );
    assert_eq!(result["status"], "complete");
    let posts = server.posts();
    assert_eq!(posts.len(), 3);
    fn content(post: &Value) -> &Vec<Value> {
        post["messages"][0]["content"].as_array().unwrap()
    }
    assert_eq!(content(&posts[0]).len(), 1);
    assert_eq!(content(&posts[1]).len(), 2);
    assert_eq!(content(&posts[2]).len(), 2);
    let after_inspection = content(&posts[1])[1]["image_url"]["url"].as_str().unwrap();
    let after_readback = content(&posts[2])[1]["image_url"]["url"].as_str().unwrap();
    assert_eq!(after_inspection, after_readback);
    assert!(after_inspection.starts_with("data:image/png;base64,"));
}

#[test]
fn pinned_transport_retry_keeps_the_model_and_never_replays_a_prior_write() {
    let sandbox = tempfile::tempdir().unwrap();
    let home = sandbox.path().join("home");
    let project = sandbox.path().join("project");
    let empty_path = sandbox.path().join("empty-path");
    for path in [&home, &project, &empty_path] {
        fs::create_dir_all(path).unwrap();
    }
    let server = MockTarget::new(
        vec![model("frontier-pinned", "frontier model")],
        vec![
            action(
                "frontier-pinned",
                json!({"action":"tool","tool":"workspace_write","arguments_json":r#"{"path":"once.txt","content":"written once","expected_sha256":null}"#,"result":""}),
            ),
            (0, Value::Null),
            action(
                "frontier-pinned",
                json!({"action":"tool","tool":"workspace_read","arguments_json":r#"{"path":"once.txt"}"#,"result":""}),
            ),
            action(
                "frontier-pinned",
                json!({"action":"complete","tool":"","arguments_json":"{}","result":"verified after transport recovery"}),
            ),
        ],
    );
    configure(&home, vec![target("pinned", &server)]);
    let file = request(
        &home,
        &project,
        &["workspace_write", "workspace_read"],
        &["workspace_read"],
    );
    let mut value: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    value["model_selection"] = json!({"target":"pinned","id":"frontier-pinned","effort":"high"});
    fs::write(&file, serde_json::to_vec(&value).unwrap()).unwrap();
    let result = ok(
        &home,
        &empty_path,
        &["agent", "spawn", "--request", file.to_str().unwrap()],
    );
    assert_eq!(result["model"]["id"], "frontier-pinned");
    assert_eq!(
        fs::read_to_string(project.join("once.txt")).unwrap(),
        "written once"
    );
    let posts = server.posts();
    assert_eq!(posts.len(), 4);
    assert_eq!(
        posts[1], posts[2],
        "retry must use unchanged inference input, not replay the prior action"
    );
}

#[test]
fn pinned_transport_retries_stop_at_the_run_budget() {
    let sandbox = tempfile::tempdir().unwrap();
    let home = sandbox.path().join("home");
    let project = sandbox.path().join("project");
    let empty_path = sandbox.path().join("empty-path");
    for path in [&home, &project, &empty_path] {
        fs::create_dir_all(path).unwrap();
    }
    let server = MockTarget::new(
        vec![model("frontier-pinned", "frontier model")],
        vec![(0, Value::Null); 3],
    );
    configure(&home, vec![target("pinned", &server)]);
    let file = request(&home, &project, &[], &[]);
    let result = invoke(
        &home,
        &empty_path,
        &["agent", "spawn", "--request", file.to_str().unwrap()],
    );
    assert!(!result.status.success());
    assert_eq!(server.posts().len(), 3);
    assert!(fs::read_dir(&project).unwrap().next().is_none());
}

#[test]
fn rate_limit_failover_preserves_tier_and_skips_lower_model() {
    let sandbox = tempfile::tempdir().unwrap();
    let home = sandbox.path().join("home");
    let project = sandbox.path().join("project");
    let empty_path = sandbox.path().join("empty-path");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&empty_path).unwrap();
    let primary = MockTarget::new(
        vec![model("frontier-alpha", "frontier model")],
        vec![(429, json!({"error":"simulated throttle"}))],
    );
    let backup = MockTarget::new(
        vec![model("frontier-beta", "frontier model")],
        vec![action(
            "frontier-beta",
            json!({"action":"complete","tool":"","arguments_json":"{}","result":"from backup"}),
        )],
    );
    let lower = MockTarget::new(vec![model("utility-mini", "small utility model")], vec![]);
    configure(
        &home,
        vec![
            target("a-primary", &primary),
            target("b-backup", &backup),
            target("z-lower", &lower),
        ],
    );
    let request = request(&home, &project, &[], &[]);
    let result = ok(
        &home,
        &empty_path,
        &["agent", "spawn", "--request", request.to_str().unwrap()],
    );
    assert_eq!(result["result"], "from backup");
    assert_eq!(result["model"]["id"], "frontier-beta");
    assert_eq!(primary.posts().len(), 1);
    assert_eq!(backup.posts().len(), 1);
    assert!(lower.posts().is_empty(), "failover must not lower the tier");
}

#[test]
fn changed_missing_or_truncated_model_response_cannot_write() {
    let write = json!({"action":"tool","tool":"workspace_write","arguments_json":r#"{"path":"unsafe.txt","content":"wrong","expected_sha256":null}"#,"result":""});
    for case in ["changed", "missing", "truncated"] {
        let sandbox = tempfile::tempdir().unwrap();
        let home = sandbox.path().join("home");
        let project = sandbox.path().join("project");
        let empty_path = sandbox.path().join("empty-path");
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&empty_path).unwrap();
        let mut response = action("frontier-selected", write.clone()).1;
        match case {
            "changed" => response["model"] = json!("different-model"),
            "missing" => {
                response.as_object_mut().unwrap().remove("model");
            }
            "truncated" => response["choices"][0]["finish_reason"] = json!("length"),
            _ => unreachable!(),
        }
        let server = MockTarget::new(
            vec![model("frontier-selected", "frontier model")],
            vec![(200, response)],
        );
        configure(&home, vec![target("mock", &server)]);
        let request = request(
            &home,
            &project,
            &["workspace_write", "workspace_read"],
            &["workspace_read"],
        );
        let output = invoke(
            &home,
            &empty_path,
            &["agent", "spawn", "--request", request.to_str().unwrap()],
        );
        assert!(!output.status.success(), "{case} response was accepted");
        assert!(
            !project.join("unsafe.txt").exists(),
            "{case} response wrote a file"
        );
        assert_eq!(server.posts().len(), 1);
    }
}

#[test]
fn mechanical_utility_can_run_but_architecture_and_model_overrides_cannot() {
    let sandbox = tempfile::tempdir().unwrap();
    let home = sandbox.path().join("home");
    let project = sandbox.path().join("project");
    let empty_path = sandbox.path().join("empty-path");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&empty_path).unwrap();
    let mut utility = model("utility-mini", "small utility model");
    utility["reasoning_efforts"] = json!(["low"]);
    let server = MockTarget::new(
        vec![utility],
        vec![action(
            "utility-mini",
            json!({"action":"complete","tool":"","arguments_json":"{}","result":"{\"name\":\"Ada\"}"}),
        )],
    );
    configure(&home, vec![target("mock", &server)]);

    let mechanical = json!({
        "objective": "Convert CSV to JSON",
        "deterministic_operation": {"action":"convert","from":"CSV","to":"JSON"},
        "deterministic_input": "name\nAda",
        "task_intent": "format_conversion",
        "complexity_class": "deterministic",
        "assigned_tools": [],
        "task_needs": [{"workflow":"structured-response","reason":"Convert the supplied CSV into one JSON response without effects."}],
        "project": project.to_str().unwrap(),
        "requires_vision": false,
        "verification_tools": []
    });
    let mechanical_path = home.join("mechanical.json");
    fs::write(&mechanical_path, serde_json::to_vec(&mechanical).unwrap()).unwrap();
    let route = ok(
        &home,
        &empty_path,
        &[
            "models",
            "route",
            "--request",
            mechanical_path.to_str().unwrap(),
        ],
    );
    assert_eq!(route["resolution"]["min_tier"], "T1_DETERMINISTIC");
    assert_eq!(route["candidates"][0]["id"], "utility-mini");
    let result = ok(
        &home,
        &empty_path,
        &[
            "agent",
            "spawn",
            "--request",
            mechanical_path.to_str().unwrap(),
        ],
    );
    assert_eq!(result["model"]["id"], "utility-mini");
    assert_eq!(result["result"], "{\"name\":\"Ada\"}");
    assert_eq!(server.posts().len(), 1);

    let architecture_path = request(&home, &project, &[], &[]);
    let architecture = ok(
        &home,
        &empty_path,
        &[
            "models",
            "route",
            "--request",
            architecture_path.to_str().unwrap(),
        ],
    );
    assert_eq!(architecture["resolution"]["min_tier"], "T3_ARCHITECT");
    assert!(architecture["candidates"].as_array().unwrap().is_empty());
    let output = invoke(
        &home,
        &empty_path,
        &[
            "agent",
            "spawn",
            "--request",
            architecture_path.to_str().unwrap(),
        ],
    );
    assert!(!output.status.success());
    assert_eq!(server.posts().len(), 1, "architecture must not use T1");

    for field in ["model_id", "model"] {
        let mut overridden = mechanical.clone();
        overridden[field] = json!("utility-mini");
        let path = home.join(format!("override-{field}.json"));
        fs::write(&path, serde_json::to_vec(&overridden).unwrap()).unwrap();
        let output = invoke(
            &home,
            &empty_path,
            &["agent", "spawn", "--request", path.to_str().unwrap()],
        );
        assert!(!output.status.success(), "{field} override was accepted");
        assert_eq!(
            server.posts().len(),
            1,
            "{field} override reached inference"
        );
    }
}

#[test]
fn inappropriate_task_tools_are_rejected_before_discovery_or_inference() {
    let sandbox = tempfile::tempdir().unwrap();
    let home = sandbox.path().join("home");
    let project = sandbox.path().join("project");
    let empty_path = sandbox.path().join("empty-path");
    for p in [&home, &project, &empty_path] {
        fs::create_dir_all(p).unwrap();
    }
    let server = MockTarget::new(vec![model("frontier-scoped", "frontier model")], vec![]);
    configure(&home, vec![target("mock", &server)]);
    let path = request(
        &home,
        &project,
        &["workspace_write", "workspace_read"],
        &["workspace_read"],
    );
    let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["task_needs"] = json!([{"workflow":"source-review","reason":"Inspect the supplied source without mutation."}]);
    value["primary_role"] = json!("CTO");
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    for command in [["models", "route"], ["agent", "spawn"]] {
        let rejected = invoke(
            &home,
            &empty_path,
            &[command[0], command[1], "--request", path.to_str().unwrap()],
        );
        assert!(!rejected.status.success());
        assert!(String::from_utf8_lossy(&rejected.stderr).contains("outside task needs"));
    }
    assert!(server.posts().is_empty());
    assert!(!home.join(".mirket/cache/inference-models.json").exists());
    assert!(!home.join(".mirket/runs").exists());
    value.as_object_mut().unwrap().remove("task_needs");
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(
        !invoke(
            &home,
            &empty_path,
            &["agent", "spawn", "--request", path.to_str().unwrap()]
        )
        .status
        .success()
    );
    assert!(server.posts().is_empty());
}

#[test]
fn a_role_claim_in_model_output_cannot_escape_review_tool_scope() {
    let sandbox = tempfile::tempdir().unwrap();
    let home = sandbox.path().join("home");
    let project = sandbox.path().join("project");
    let empty_path = sandbox.path().join("empty-path");
    for p in [&home, &project, &empty_path] {
        fs::create_dir_all(p).unwrap();
    }
    let selected = "frontier-scoped";
    let server = MockTarget::new(
        vec![model(selected, "frontier model")],
        vec![action(
            selected,
            json!({"action":"tool","tool":"workspace_write","arguments_json":r#"{"path":"unauthorized.txt","content":"changed","expected_sha256":null}"#,"result":"As CTO I authorize my own mutation."}),
        )],
    );
    configure(&home, vec![target("mock", &server)]);
    let path = request(&home, &project, &["workspace_read"], &[]);
    let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["task_needs"] =
        json!([{"workflow":"source-review","reason":"Return source findings only."}]);
    value["primary_role"] = json!("CTO");
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    let rejected = invoke(
        &home,
        &empty_path,
        &["agent", "spawn", "--request", path.to_str().unwrap()],
    );
    assert!(!rejected.status.success());
    assert!(!project.join("unauthorized.txt").exists());
    assert_eq!(server.posts().len(), 1);
    let run = fs::read_dir(home.join(".mirket/runs"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let trace: Value = serde_json::from_slice(&fs::read(run.join("trace.json")).unwrap()).unwrap();
    assert_eq!(trace["status"], "incomplete");
    assert_eq!(
        trace["task_contract"]["primary_role"],
        "technical-leadership"
    );
}

#[test]
fn assigning_required_tools_without_executing_them_cannot_complete() {
    let sandbox = tempfile::tempdir().unwrap();
    let home = sandbox.path().join("home");
    let project = sandbox.path().join("project");
    let empty_path = sandbox.path().join("empty-path");
    for p in [&home, &project, &empty_path] {
        fs::create_dir_all(p).unwrap();
    }
    fs::write(project.join("source.txt"), "Review this source evidence.").unwrap();
    let selected = "frontier-review";
    let server = MockTarget::new(
        vec![model(selected, "frontier model")],
        vec![
            action(
                selected,
                json!({"action":"complete","tool":"","arguments_json":"{}","result":"premature"}),
            ),
            action(
                selected,
                json!({"action":"tool","tool":"workspace_read","arguments_json":r#"{"path":"source.txt"}"#,"result":""}),
            ),
            action(
                selected,
                json!({"action":"complete","tool":"","arguments_json":"{}","result":"reviewed source"}),
            ),
        ],
    );
    configure(&home, vec![target("mock", &server)]);
    let path = request(&home, &project, &["workspace_read"], &[]);
    let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["task_needs"] = json!([{"workflow":"source-review","reason":"Inspect the supplied source before reporting findings."}]);
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    let result = ok(
        &home,
        &empty_path,
        &["agent", "spawn", "--request", path.to_str().unwrap()],
    );
    assert_eq!(result["result"], "reviewed source");
    let posts = server.posts();
    assert_eq!(posts.len(), 3);
    assert!(
        posts[1]["messages"][0]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("missing_required_tools")
    );
}

#[test]
fn required_recall_without_runtime_rejects_before_any_inference() {
    let sandbox = tempfile::tempdir().unwrap();
    let home = sandbox.path().join("home");
    let project = sandbox.path().join("project");
    let empty_path = sandbox.path().join("empty-path");
    for p in [&home, &project, &empty_path] {
        fs::create_dir_all(p).unwrap();
    }
    let server = MockTarget::new(vec![model("frontier-recall", "frontier model")], vec![]);
    configure(&home, vec![target("mock", &server)]);
    let path = request(&home, &project, &[], &[]);
    let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["memory_recall"] = json!("required");
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    let output = invoke(
        &home,
        &empty_path,
        &["agent", "spawn", "--request", path.to_str().unwrap()],
    );
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("required memory recall is unavailable")
    );
    assert!(server.posts().is_empty());
    assert!(!home.join(".mirket/runs").exists());
}

#[test]
fn inference_diagnostics_keep_numeric_usage_without_private_fields() {
    for invalid in [false, true] {
        let sandbox = tempfile::tempdir().unwrap();
        let home = sandbox.path().join("home");
        let project = sandbox.path().join("project");
        let empty = sandbox.path().join("empty");
        for p in [&home, &project, &empty] {
            fs::create_dir_all(p).unwrap();
        }
        let selected = "frontier-receipt";
        let content = if invalid {
            "not-json PRIVATE_PROVIDER_BODY".into()
        } else {
            json!({"action":"complete","tool":"","arguments_json":"{}","result":"Checked synthetic evidence."}).to_string()
        };
        let raw = json!({"model":selected,"choices":[{"finish_reason":"stop","message":{"content":content}}],"usage":{"prompt_tokens":1234,"completion_tokens":56,"total_tokens":1290,"private":"DO_NOT_SAVE_USAGE_FIELD"}});
        let digest = mirket::util::sha256(&serde_json::to_vec(&raw).unwrap());
        let server = MockTarget::new(vec![model(selected, "frontier model")], vec![(200, raw)]);
        configure(&home, vec![target("mock", &server)]);
        let path = request(&home, &project, &[], &[]);
        let output = invoke(
            &home,
            &empty,
            &["agent", "spawn", "--request", path.to_str().unwrap()],
        );
        assert_eq!(
            output.status.success(),
            !invalid,
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let run = fs::read_dir(home.join(".mirket/runs"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let text = fs::read_to_string(run.join("trace.json")).unwrap();
        let trace: Value = serde_json::from_str(&text).unwrap();
        let inference = &trace["attempts"][0]["inference"];
        assert_eq!(inference.as_object().unwrap().len(), 3);
        assert_eq!(inference["usage"]["input_tokens"], 1234);
        assert_eq!(inference["usage"]["output_tokens"], 56);
        assert!(inference["usage"]["cost_usd"].is_null());
        assert!(trace.get("accounting").is_none());
        assert!(!run.join("accounting-version.json").exists());
        assert!(!run.join("probes").exists());
        if output.status.success() {
            let result: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert!(result.get("accounting").is_none());
        }
        assert_eq!(inference["response_sha256"], digest);
        assert!(inference["elapsed_ms"].is_u64());
        assert!(!text.contains("PRIVATE_PROVIDER_BODY"));
        assert!(!text.contains("DO_NOT_SAVE_USAGE_FIELD"));
        if invalid {
            assert_eq!(trace["error"]["code"], "invalid_json");
            assert_eq!(trace["error"]["json_line"], 1);
        }
    }
}

#[test]
fn a_large_written_artifact_requires_all_digest_bound_read_pages() {
    let sandbox = tempfile::tempdir().unwrap();
    let home = sandbox.path().join("home");
    let project = sandbox.path().join("project");
    let empty = sandbox.path().join("empty");
    for p in [&home, &project, &empty] {
        fs::create_dir_all(p).unwrap();
    }
    let selected = "frontier-pages";
    let content = "x".repeat(200_000);
    let sha = mirket::util::sha256(content.as_bytes());
    let tool = |name: &str, args: Value| {
        action(
            selected,
            json!({"action":"tool","tool":name,"arguments_json":args.to_string(),"result":""}),
        )
    };
    let mut live = model(selected, "frontier model");
    live["context_length"] = json!(1048576);
    let server = MockTarget::new(
        vec![live],
        vec![
            tool(
                "workspace_write",
                json!({"path":"large.txt","content":content,"expected_sha256":null}),
            ),
            tool("workspace_read", json!({"path":"large.txt"})),
            action(
                selected,
                json!({"action":"complete","tool":"","arguments_json":"{}","result":"Premature partial read cannot pass."}),
            ),
            tool(
                "workspace_read",
                json!({"path":"large.txt","offset":131072,"expected_sha256":sha}),
            ),
            action(
                selected,
                json!({"action":"complete","tool":"","arguments_json":"{}","result":"All bytes read after the write."}),
            ),
        ],
    );
    configure(&home, vec![target("mock", &server)]);
    let path = request(
        &home,
        &project,
        &["workspace_write", "workspace_read"],
        &["workspace_read"],
    );
    let result = ok(
        &home,
        &empty,
        &["agent", "spawn", "--request", path.to_str().unwrap()],
    );
    assert_eq!(result["result"], "All bytes read after the write.");
    assert_eq!(server.posts().len(), 5);
    assert_eq!(
        fs::read_to_string(project.join("large.txt")).unwrap(),
        content
    );
    let trace: Value =
        serde_json::from_slice(&fs::read(result["trace"].as_str().unwrap()).unwrap()).unwrap();
    assert!(
        trace["transcript"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["verification_gate"] == "pending")
    );
}

#[test]
fn already_satisfied_keeps_source_unchanged_and_requires_host_checks() {
    let sandbox = tempfile::tempdir().unwrap();
    let home = sandbox.path().join("home");
    let project = sandbox.path().join("project");
    let empty = sandbox.path().join("empty");
    for dir in [&home, &project, &empty] {
        fs::create_dir_all(dir).unwrap();
    }
    fs::write(project.join("ready.txt"), "already correct").unwrap();
    let selected = "neutral-mini";
    let server = MockTarget::new(
        vec![model(selected, "neutral")],
        vec![
            action(
                selected,
                json!({"action":"tool","tool":"workspace_read","arguments_json":"{\"path\":\"ready.txt\"}","result":""}),
            ),
            action(
                selected,
                json!({"action":"already_satisfied","tool":"","arguments_json":"{}","result":"The requested source is already present."}),
            ),
        ],
    );
    configure(&home, vec![target("mock", &server)]);
    let path = request(
        &home,
        &project,
        &["workspace_write", "workspace_read"],
        &["workspace_read"],
    );
    let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["allow_already_satisfied"] = json!(true);
    value["limits"] = json!({"max_calls":2,"max_completion_tokens":4096});
    fs::write(&path, value.to_string()).unwrap();
    let result = ok(
        &home,
        &empty,
        &["agent", "spawn", "--request", path.to_str().unwrap()],
    );
    assert_eq!(result["status"], "awaiting_host_verification");
    assert_eq!(
        result["host_verification"]["artifacts"][0]["path"],
        "ready.txt"
    );
    assert_eq!(
        fs::read(project.join("ready.txt")).unwrap(),
        b"already correct"
    );
    assert_eq!(server.posts().len(), 2);
    assert!(
        server
            .posts()
            .iter()
            .all(|p| p["max_completion_tokens"] == 4096)
    );
}

#[test]
fn native_line_operations_do_not_discover_or_invoke_models() {
    let sandbox = tempfile::tempdir().unwrap();
    let home = sandbox.path().join("home");
    let project = sandbox.path().join("project");
    let empty = sandbox.path().join("empty");
    for dir in [&home, &project, &empty] {
        fs::create_dir_all(dir).unwrap();
    }
    let path = request(&home, &project, &[], &[]);
    let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["deterministic_input"] = json!("b\na\nb");
    fs::write(&path, value.to_string()).unwrap();
    let result = ok(
        &home,
        &empty,
        &["agent", "spawn", "--request", path.to_str().unwrap()],
    );
    assert_eq!(result["execution"], "pure_function");
    assert_eq!(result["result"], "a\nb\nb");
    assert!(result["model"].is_null());
}
