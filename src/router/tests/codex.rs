#[cfg(test)]
mod tests {
    use super::*;
    fn sse(output: Value, status: &str, model: &str) -> Vec<u8> {
        format!("data: {}\n\n",json!({"type":"response.completed","response":{"model":model,"status":status,"output":output}})).into_bytes()
    }
    #[test]
    fn complete_stream_is_text_only_identity_bound_and_untruncated() {
        let output = json!([{"type":"message","role":"assistant","content":[{"type":"output_text","text":"{\"action\":\"complete\"}"}]}]);
        assert!(
            validate_stream(
                &sse(output.clone(), "completed", "selected"),
                "selected",
                &mut json!({"response_sha256":null})
            )
            .is_ok()
        );
        assert!(
            validate_stream(
                &sse(output.clone(), "incomplete", "selected"),
                "selected",
                &mut json!({"response_sha256":null})
            )
            .is_err()
        );
        assert!(
            validate_stream(
                &sse(output, "completed", "changed"),
                "selected",
                &mut json!({"response_sha256":null})
            )
            .is_err()
        );
        assert!(
            validate_stream(
                b"data: {\"type\":\"response.created\"}\n\n",
                "selected",
                &mut json!({"response_sha256":null})
            )
            .is_err()
        );
        assert!(validate_stream(&sse(json!([{"type":"function_call","name":"exec_command","arguments":"touch sentinel"}]),"completed","selected"),"selected", &mut json!({"response_sha256":null})).is_err());
        assert!(validate_stream(b"data: {\"type\":\"response.function_call_arguments.delta\",\"delta\":\"touch sentinel\"}\n\n","selected", &mut json!({"response_sha256":null})).is_err());
    }
    #[test]
    fn native_nonfatal_diagnostics_do_not_relax_execution_rejection() {
        assert!(matches!(child_event(&json!({"type":"item.completed","item":{"type":"error","message":"Code Mode is unavailable"}})).unwrap(),ChildEvent::Progress));
        for frame in [
            json!({"type":"error","message":"fatal"}),
            json!({"type":"turn.failed"}),
            json!({"type":"item.completed","item":{"type":"command_execution","command":"touch sentinel"}}),
        ] {
            assert!(child_event(&frame).is_err());
        }
        assert!(child_event(&json!({"type":"item.completed","item":{"type":"context_compaction"}})).is_err());
    }

    #[test]
    fn provider_context_or_image_refusal_is_not_treated_as_a_successful_turn() {
        for frame in [
            &br#"data: {"type":"response.failed","response":{"status":"failed","error":{"code":"context_length_exceeded"}}}

"#[..],
            &br#"data: {"type":"response.failed","response":{"status":"failed","error":{"code":"image_too_large"}}}

"#[..],
        ] {
            assert!(validate_stream(frame, "selected", &mut json!({"response_sha256":null})).is_err());
        }
    }

    #[test]
    fn canonical_request_has_no_native_context_or_tools() {
        let body = request_body("discovered", Some("high"), 64000, "owned prompt", &[]).unwrap();
        assert_eq!(body["tools"], json!([]));
        assert_eq!(body["tool_choice"], "none");
        assert_eq!(body["input"][0]["content"][0]["text"], "owned prompt");
        assert_eq!(body["reasoning"]["effort"], "high");
        assert_eq!(body["max_output_tokens"], 64000);
    }
}

#[cfg(test)]
#[path = "../codex_tests.rs"]
mod transport_tests;
