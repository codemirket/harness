#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_result_retains_numeric_usage_without_private_provider_text() {
        for failed in [false, true] {
            let raw = json!({"type":"result","subtype":if failed {"error_during_execution"} else {"success"},
                "is_error":failed,"usage":{"input_tokens":30,"output_tokens":12,"cache_read_input_tokens":200},
                "total_cost_usd":0.01,"structured_output":{"action":"complete"},"result":"PRIVATE_PROVIDER_TEXT"});
            let mut observation = json!({"response_sha256":null});
            assert_eq!(
                claude_result(&raw, true, true, &mut observation).is_err(),
                failed
            );
            assert_eq!(
                observation,
                json!({"response_sha256":crate::util::sha256(&serde_json::to_vec(&raw).unwrap()),"usage":crate::router::execution::usage_receipt(&raw)})
            );
            assert!(!observation.to_string().contains("PRIVATE_PROVIDER_TEXT"));
            let mut unknown = json!({"response_sha256":null});
            assert!(claude_result(&raw, false, true, &mut unknown).is_err());
            assert_eq!(unknown, json!({"response_sha256":null}));
        }
    }

    #[test]
    fn image_inputs_are_regular_bounded_files() {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let path = dir.path().join("image.png");
        std::fs::write(&path, b"\x89PNG\r\n\x1a\n").unwrap();
        assert!(user_frame("inspect", std::slice::from_ref(&path)).is_ok());
        std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(MAX_IMAGE_BYTES as u64 + 1)
            .unwrap();
        assert!(user_frame("inspect", std::slice::from_ref(&path)).is_err());
        assert!(user_frame("inspect", &[dir.path().to_path_buf()]).is_err());
        #[cfg(unix)]
        {
            std::fs::write(&path, b"\x89PNG\r\n\x1a\n").unwrap();
            let link = dir.path().join("link.png");
            std::os::unix::fs::symlink(&path, &link).unwrap();
            assert!(user_frame("inspect", &[link]).is_err());
        }
    }

    #[test]
    fn action_requires_current_exact_identity_and_effort_before_inference() {
        let current = json!({"models":[{
            "value":"native-alias", "resolvedModel":"dynamic-resolved-model",
            "supportsEffort":true, "supportedEffortLevels":["medium","high"]
        }]});
        assert!(
            validate_claude_selection(&current, "dynamic-resolved-model", Some("high")).is_ok()
        );
        assert!(validate_claude_selection(&current, "native-alias", Some("high")).is_err());
        let mut changed = current.clone();
        changed["models"][0]["supportsEffort"] = json!(false);
        assert!(
            validate_claude_selection(&changed, "dynamic-resolved-model", Some("high")).is_err()
        );
        changed = current.clone();
        changed["models"][0]["supportedEffortLevels"] = json!(["medium"]);
        assert!(
            validate_claude_selection(&changed, "dynamic-resolved-model", Some("high")).is_err()
        );
        changed = current;
        changed["models"][0]["resolvedModel"] = json!("new-dynamic-model");
        assert!(
            validate_claude_selection(&changed, "dynamic-resolved-model", Some("high")).is_err()
        );
    }
}
