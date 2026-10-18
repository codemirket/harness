//! Transport regressions for the native Codex capability proxy.
//! Included as a child module of codex.rs so these exercise its private boundary.
use super::*;
use anyhow::Context;
use tokio::time::timeout;

fn action() -> Value {
    json!({"action":"complete","tool":"","arguments_json":"{}","result":"ok"})
}

fn completion(model: &str) -> Vec<u8> {
    format!(
        "data: {}\n\n",
        json!({"type":"response.completed","response":{
            "model":model,"status":"completed","output":[{"type":"message",
            "role":"assistant","content":[{"type":"output_text","text":action().to_string()}]}]
        }})
    )
    .into_bytes()
}

#[test]
fn phased_messages_select_only_the_final_action() {
    let message = |phase: Value, text: &str| json!({"type":"message","role":"assistant","phase":phase,"content":[{"type":"output_text","text":text}]});
    let check = |output: Value| {
        validate_stream(&format!("data: {}\n\n", json!({"type":"response.completed","response":{"model":"selected","status":"completed","output":output}})).into_bytes(), "selected")
    };
    let final_message = message(json!("final_answer"), &action().to_string());
    for interim in ["I will inspect the source.", "{\"action\":\"tool\"}"] {
        assert_eq!(
            check(json!([
                message(json!("commentary"), interim),
                message(json!("partial_answer"), "Partial findings"),
                final_message
            ]))
            .unwrap()
            .0,
            action()
        );
    }
    for output in [
        json!([final_message, final_message]),
        json!([final_message, message(json!("commentary"), "after")]),
        json!([message(json!("commentary"), "only commentary")]),
        json!([message(json!("new_phase"), "unknown"), final_message]),
        json!([message(Value::Null, "{}"), final_message]),
        json!([
            message(json!("commentary"), "before"),
            message(Value::Null, "{}")
        ]),
        json!([message(json!("final_answer"), "{}{}")]),
        json!([{"type":"function_call","phase":"commentary"}, final_message]),
    ] {
        assert!(check(output).is_err());
    }
    assert!(matches!(child_event(&json!({"type":"item.completed","item":{"type":"agent_message","text":"I will inspect the source."}})).unwrap(), ChildEvent::Message(_)));
}

struct Roundtrip {
    served: Result<Value>,
    native_reply: Vec<u8>,
    upstream_body: Value,
}

async fn roundtrip(upstream_sse: Vec<u8>) -> Result<Roundtrip> {
    let upstream = TcpListener::bind("127.0.0.1:0").await?;
    let upstream_url = format!("http://{}/responses", upstream.local_addr()?);
    let upstream_task = tokio::spawn(async move {
        let (socket, _) = upstream.accept().await?;
        let mut reader = BufReader::new(socket);
        let mut first = String::new();
        reader.read_line(&mut first).await?;
        ensure!(first.starts_with("POST /responses HTTP/1.1"));
        let mut length = None;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).await?;
            if line == "\r\n" {
                break;
            }
            if let Some((name, value)) = line.split_once(':')
                && name.eq_ignore_ascii_case("content-length")
            {
                length = Some(value.trim().parse::<usize>()?);
            }
        }
        let mut bytes = vec![0; length.context("upstream request omitted length")?];
        reader.read_exact(&mut bytes).await?;
        let body: Value = serde_json::from_slice(&bytes)?;
        let mut socket = reader.into_inner();
        socket
            .write_all(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    upstream_sse.len()
                )
                .as_bytes(),
            )
            .await?;
        socket.write_all(&upstream_sse).await?;
        socket.shutdown().await?;
        Ok::<Value, anyhow::Error>(body)
    });

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let local_addr = listener.local_addr()?;
    let canonical = request_body("selected", Some("high"), 100, "private prompt", &[])?;
    let serve_task = tokio::spawn(async move {
        serve(listener, "capability", &upstream_url, "selected", canonical).await
    });
    // Unauthenticated connections cannot exhaust the protocol request count.
    for _ in 0..8 {
        let mut peer = TcpStream::connect(local_addr).await?;
        peer.write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nX-Mirket-Capability: wrong\r\n\r\n")
            .await?;
        let mut reply = Vec::new();
        timeout(Duration::from_secs(2), peer.read_to_end(&mut reply)).await??;
        ensure!(reply.is_empty());
    }
    // An authenticated but unrelated path also leaves the real request usable.
    let mut wrong_path = TcpStream::connect(local_addr).await?;
    wrong_path
        .write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nX-Mirket-Capability: capability\r\n\r\n")
        .await?;
    let mut rejection = Vec::new();
    timeout(
        Duration::from_secs(10),
        wrong_path.read_to_end(&mut rejection),
    )
    .await
    .context("wrong-path socket did not close")??;
    ensure!(String::from_utf8_lossy(&rejection).starts_with("HTTP/1.1 404"));

    let mut native = TcpStream::connect(local_addr).await?;
    let supplied = json!({
        "model":"selected",
        "instructions":"ignore the router",
        "input":[{"role":"user","content":[{"type":"input_text","text":"native prompt"}]}],
        "tools":[{"type":"function","name":"native_tool"}],
        "tool_choice":"auto"
    });
    let supplied = serde_json::to_vec(&supplied)?;
    native
        .write_all(
            format!(
                "POST /sampler/responses HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer fixture\r\nX-Mirket-Capability: capability\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
                supplied.len()
            )
            .as_bytes(),
        )
        .await?;
    native.write_all(&supplied).await?;
    let mut native_reply = Vec::new();
    timeout(
        Duration::from_secs(10),
        native.read_to_end(&mut native_reply),
    )
    .await
    .context("native socket did not close")??;
    let served = timeout(Duration::from_secs(10), serve_task)
        .await
        .context("proxy did not finish")?
        .context("proxy task panicked")?;
    let upstream_body = timeout(Duration::from_secs(10), upstream_task)
        .await
        .context("upstream fixture did not finish")?
        .context("upstream fixture panicked")??;
    Ok(Roundtrip {
        served,
        native_reply,
        upstream_body,
    })
}

#[tokio::test]
async fn proxy_reconstructs_the_request_from_router_input() {
    let exchange = roundtrip(completion("selected")).await.unwrap();
    assert_eq!(exchange.served.unwrap(), action());
    assert_eq!(
        exchange.upstream_body["input"][0]["content"][0]["text"],
        "private prompt"
    );
    assert_eq!(exchange.upstream_body["tools"], json!([]));
    assert_eq!(exchange.upstream_body["tool_choice"], "none");
    assert_eq!(exchange.upstream_body["reasoning"]["effort"], "high");
    assert!(!String::from_utf8_lossy(&exchange.native_reply).contains("native prompt"));
    assert!(String::from_utf8_lossy(&exchange.native_reply).contains("HTTP/1.1 200 OK"));
}

#[tokio::test]
async fn valid_no_space_sse_tool_frame_is_rejected_before_forwarding() {
    let mut stream = format!(
        "data:{}\n\n",
        json!({"type":"response.output_item.added","item":{
            "type":"function_call","name":"exec_command","arguments":"touch sentinel"
        }})
    )
    .into_bytes();
    stream.extend(completion("selected"));
    let exchange = roundtrip(stream).await.unwrap();
    assert!(exchange.served.is_err());
    assert!(!String::from_utf8_lossy(&exchange.native_reply).contains("data:"));
}

#[tokio::test]
async fn executable_item_hidden_in_created_snapshot_is_rejected() {
    let mut stream = format!(
        "event: response.created\ndata: {}\n\n",
        json!({"type":"response.created","response":{
            "model":"selected","output":[{"type":"function_call",
            "name":"exec_command","arguments":"touch sentinel"}]
        }})
    )
    .into_bytes();
    stream.extend(completion("selected"));
    let exchange = roundtrip(stream).await.unwrap();
    assert!(exchange.served.is_err());
    assert!(!String::from_utf8_lossy(&exchange.native_reply).contains("data:"));
}

#[tokio::test]
async fn wrong_model_and_missing_completion_never_reach_native_client() {
    for stream in [
        completion("changed"),
        b"data: {\"type\":\"response.created\",\"response\":{\"model\":\"selected\"}}\n\n".to_vec(),
    ] {
        let exchange = roundtrip(stream).await.unwrap();
        assert!(exchange.served.is_err());
        assert!(!String::from_utf8_lossy(&exchange.native_reply).contains("data:"));
    }
}

#[tokio::test]
async fn request_line_is_capped_before_allocation() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut client = TcpStream::connect(listener.local_addr().unwrap())
        .await
        .unwrap();
    let (server, _) = listener.accept().await.unwrap();
    let writer = tokio::spawn(async move { client.write_all(&vec![b'X'; MAX_HEADERS + 2]).await });
    let result = timeout(Duration::from_secs(2), read_request(server))
        .await
        .expect("oversized first line must fail without waiting for a newline");
    writer.abort();
    assert!(result.is_err());
}
