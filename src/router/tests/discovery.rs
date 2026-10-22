#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn selected_vision_observation_preserves_shared_menu_and_rejects_stale_metadata() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
        util::ensure_directory(&paths.cache()).unwrap();
        let target = Target {
            name: "fixture".into(),
            protocol: Protocol::Claude,
            endpoint: None,
            credential_env: None,
            executable: None,
        };
        let model = |id| normalize(&target, &json!({"resolvedModel":id}), true).unwrap();
        let first = model("first");
        let other = model("other");
        let mut observed = first.clone();
        observed.capabilities.vision = Some(true);
        observed.classification_evidence.push("probe passed".into());
        let original = Registry {
            version: REGISTRY_VERSION,
            refreshed_at: 123,
            fingerprint: "generation".into(),
            targets: vec![],
            models: vec![first.clone(), other.clone()],
        };
        let path = paths.cache().join("inference-models.json");
        util::atomic_write_with_mode(&path, &serde_json::to_vec(&original).unwrap(), Some(0o600))
            .unwrap();
        record_vision(&paths, "generation", &first, &observed)
            .await
            .unwrap();
        let bytes = util::read_bounded(&path, MAX_METADATA as u64).unwrap();
        let cached: Registry = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(cached.models.len(), 2);
        assert_eq!(cached.models[0].capabilities.vision, Some(true));
        assert_eq!(
            serde_json::to_value(&cached.models[1]).unwrap(),
            serde_json::to_value(other).unwrap()
        );
        assert_eq!(cached.refreshed_at, 123);
        // Neither a stale generation nor stale pre-probe metadata may overwrite.
        record_vision(&paths, "old-generation", &first, &observed)
            .await
            .unwrap();
        record_vision(&paths, "generation", &first, &observed)
            .await
            .unwrap();
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
    #[test]
    fn model_budget_bounds_retained_metadata_without_rejecting_unused_provider_fields() {
        let target = endpoint_target(
            "fixture",
            Protocol::OpenaiChat,
            "http://127.0.0.1:1/v1".into(),
            None,
        );
        assert!(
            normalize(
                &target,
                &json!({"id":"published","unused_provider_data":"x".repeat(64 * 1024)}),
                true
            )
            .is_ok()
        );
        assert!(
            normalize(
                &target,
                &json!({"id":"published","description":"x".repeat(32 * 1024)}),
                true
            )
            .is_err()
        );
    }
    #[test]
    fn implicit_providers_cannot_bypass_target_budget() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().to_owned()).unwrap();
        std::fs::create_dir(home.path().join(".codex")).unwrap();
        let providers = (0..33)
            .map(|i| {
                format!("[model_providers.provider{i}]\nbase_url = \"http://127.0.0.1:1/v1\"\n")
            })
            .collect::<String>();
        std::fs::write(home.path().join(".codex/config.toml"), providers).unwrap();
        assert!(
            targets(&paths, &Config::default())
                .unwrap_err()
                .to_string()
                .contains("after automatic discovery")
        );
    }

    #[test]
    fn discovery_coordination_is_scoped_to_the_home() {
        let first = discovery_gate(Path::new("/one"));
        let same = discovery_gate(Path::new("/one"));
        let other = discovery_gate(Path::new("/two"));
        assert!(Arc::ptr_eq(&first, &same));
        assert!(!Arc::ptr_eq(&first, &other));
    }

    async fn catalog_server(
        body: String,
        barrier: Option<Arc<tokio::sync::Barrier>>,
    ) -> (Target, tokio::task::JoinHandle<usize>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let target = endpoint_target(
            &listener.local_addr().unwrap().to_string(),
            Protocol::OpenaiChat,
            format!("http://{}/v1", listener.local_addr().unwrap()),
            None,
        );
        let task = tokio::spawn(async move {
            let mut requests = 0;
            while let Ok((mut stream, _)) = listener.accept().await {
                let mut buffer = [0u8; 8192];
                assert!(stream.read(&mut buffer).await.unwrap() > 0);
                requests += 1;
                if let Some(barrier) = &barrier {
                    barrier.wait().await;
                }
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                if stream.write_all(header.as_bytes()).await.is_ok() {
                    let _ = stream.write_all(body.as_bytes()).await;
                }
                if requests == 2 {
                    break;
                }
            }
            requests
        });
        (target, task)
    }

    #[tokio::test]
    async fn paginated_catalog_has_one_streaming_byte_budget() {
        let body = json!({"data":[{"id":"one","description":"x".repeat(MAX_METADATA / 2)}],"has_more":true,"last_id":"next"}).to_string();
        let (target, server) = catalog_server(body, None).await;
        let home = tempfile::tempdir().unwrap();
        let error = probe(&target, home.path()).await.unwrap_err();
        assert!(error.to_string().contains("exceeds limit"));
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(5), server)
                .await
                .unwrap()
                .unwrap(),
            2
        );
    }

    #[tokio::test]
    async fn independent_targets_are_probed_concurrently() {
        let barrier = Arc::new(tokio::sync::Barrier::new(2));
        let body = json!({"data":[{"id":"discovered-model"}]}).to_string();
        let (one, task_one) = catalog_server(body.clone(), Some(barrier.clone())).await;
        let (two, task_two) = catalog_server(body, Some(barrier)).await;
        let home = tempfile::tempdir().unwrap();
        let (statuses, models) = probe_targets(vec![one, two], home.path(), Duration::from_secs(3))
            .await
            .unwrap();
        task_one.abort();
        task_two.abort();
        assert!(statuses.iter().all(|status| status.execution_ready));
        assert_eq!(models.len(), 2);
    }
    #[test]
    fn unknown_capabilities_stay_unknown() {
        let target = endpoint_target(
            "fixture",
            Protocol::OpenaiChat,
            "http://127.0.0.1:1/v1".into(),
            None,
        );
        let m = normalize(&target, &json!({"id":"future-generation"}), true).unwrap();
        assert_eq!(m.tier, Tier::T1Deterministic);
        assert_eq!(m.capabilities.vision, None);
        assert_eq!(m.capabilities.tools, None);
    }
    #[test]
    fn endpoint_cannot_embed_auth_or_leak_to_redirect() {
        for url in [
            "http://public.example/v1",
            "http://localhost.evil.example/v1",
            "http://user@localhost:1234/v1",
            "https://user:pass@example.test/v1",
            "https://example.test/v1?key=private",
            "https://example.test/v1#token",
        ] {
            assert!(validated_url(url).is_err());
        }
        assert!(validated_url("http://127.0.0.1:23456/v1").is_ok());
        assert!(validated_url("http://[::1]:23456/v1").is_ok());
    }
    #[test]
    fn config_has_no_model_registry_surface() {
        assert!(serde_json::from_value::<Config>(json!({"models":["fixture"]})).is_err());
    }

    #[test]
    fn public_remote_metadata_does_not_establish_inference_auth() {
        let remote = endpoint_target(
            "remote-fixture",
            Protocol::Anthropic,
            "https://models.example.test/v1".into(),
            None,
        );
        assert!(!http_inference_ready_with_env(&remote, |_| None));
        let loopback = endpoint_target(
            "loopback-fixture",
            Protocol::Ollama,
            "http://127.0.0.1:11434".into(),
            None,
        );
        assert!(http_inference_ready_with_env(&loopback, |_| None));
        let secure_loopback = endpoint_target(
            "secure-loopback-fixture",
            Protocol::OpenaiChat,
            "https://localhost:8443/v1".into(),
            None,
        );
        assert!(http_inference_ready_with_env(&secure_loopback, |_| None));

        let keyed = endpoint_target(
            "keyed-fixture",
            Protocol::OpenaiResponses,
            "https://models.example.test/v1".into(),
            Some("SYNTHETIC_API_KEY"),
        );
        assert!(!http_inference_ready_with_env(&keyed, |_| None));
        assert!(!http_inference_ready_with_env(&keyed, |_| Some(
            OsString::new()
        )));
        assert!(http_inference_ready_with_env(&keyed, |_| Some(
            OsString::from("synthetic")
        )));
    }

    #[test]
    fn cached_public_catalog_cannot_retain_forged_inference_readiness() {
        let target = endpoint_target(
            "remote-fixture",
            Protocol::OpenaiChat,
            "https://models.example.test/v1".into(),
            None,
        );
        let mut model = normalize(
            &target,
            &json!({"id":"synthetic-future-workhorse","supports_tools":true,"supports_structured_outputs":true,"reasoning_efforts":["high"]}),
            true,
        )
        .unwrap();
        assert!(model.available);
        model.available = true;
        let mut registry = Registry {
            version: REGISTRY_VERSION,
            refreshed_at: now(),
            fingerprint: "synthetic".into(),
            targets: vec![TargetStatus {
                target,
                authenticated: true,
                execution_ready: true,
                diagnostic: None,
            }],
            models: vec![model],
        };
        rebind_cached_http_auth_with_env(&mut registry, |_| None);
        reclassify_cached_models(&mut registry);
        assert!(!registry.targets[0].authenticated);
        assert!(!registry.targets[0].execution_ready);
        assert!(
            registry.targets[0]
                .diagnostic
                .as_deref()
                .unwrap()
                .contains("Discovery only")
        );
        assert!(!registry.models[0].available);
    }

    #[tokio::test]
    async fn native_readiness_never_comes_from_endpoint_shape() {
        let target = Target {
            name: "native-fixture".into(),
            protocol: Protocol::Claude,
            endpoint: Some("http://127.0.0.1:1234".into()),
            credential_env: None,
            executable: None,
        };
        assert!(!http_inference_ready_with_env(&target, |_| None));
        assert!(probe_one(target, PathBuf::from("/unused")).await.is_err());
    }

    #[test]
    fn claude_requires_resolved_identity_and_supported_effort_contract() {
        let target = Target {
            name: "claude-fixture".into(),
            protocol: Protocol::Claude,
            endpoint: None,
            credential_env: None,
            executable: Some(PathBuf::from("/unused/claude")),
        };
        assert!(
            normalize(
                &target,
                &json!({"value":"opus","supportsEffort":true,"supportedEffortLevels":["high"]}),
                true
            )
            .is_err()
        );
        let raw = json!({"value":"opus","resolvedModel":"future-full-model","supportsEffort":false,"supportedEffortLevels":["high"]});
        let model = normalize(&target, &raw, true).unwrap();
        assert_eq!(model.id, "future-full-model");
        assert!(model.capabilities.reasoning_efforts.is_empty());
        let high = normalize(&target, &json!({"value":"opus","resolvedModel":"future-full-model","supportsEffort":true,"supportedEffortLevels":["high"]}), true).unwrap();
        assert_eq!(high.capabilities.reasoning_efforts, vec!["high"]);
    }

    #[test]
    fn anthropic_nested_capabilities_and_limits_are_normalized_without_tool_inference() {
        let target = endpoint_target(
            "anthropic-fixture",
            Protocol::Anthropic,
            "https://example.test/v1".into(),
            Some("SYNTHETIC_KEY"),
        );
        let raw = json!({
            "id":"synthetic-future-core",
            "display_name":"Synthetic Future Core",
            "line":"opus",
            "lifecycle":"active",
            "max_input_tokens":180000,
            "max_tokens":96000,
            "capabilities":{
                "structured_outputs":{"supported":true},
                "image_input":{"supported":true},
                "effort":{
                    "supported":true,
                    "low":{"supported":true},
                    "medium":{"supported":false},
                    "high":{"supported":true},
                    "xhigh":{"supported":true},
                    "max":{"supported":false}
                },
                "thinking":{
                    "supported":true,
                    "types":{
                        "adaptive":{"supported":true},
                        "enabled":{"supported":false},
                        "disabled":{"supported":true}
                    }
                },
                "server_tools":{"supported":true}
            }
        });
        let model = normalize(&target, &raw, true).unwrap();
        assert_eq!(model.display_name, "Synthetic Future Core");
        assert_eq!(model.line.as_deref(), Some("opus"));
        assert_eq!(model.capabilities.structured_outputs, Some(true));
        assert_eq!(model.capabilities.vision, Some(true));
        assert_eq!(model.capabilities.tools, None);
        assert_eq!(
            model.capabilities.reasoning_efforts,
            vec!["low".to_string(), "high".to_string(), "xhigh".to_string()]
        );
        assert_eq!(
            model.capabilities.thinking_modes,
            vec!["adaptive".to_string(), "disabled".to_string()]
        );
        assert_eq!(model.capabilities.context_window, Some(180000));
        assert_eq!(model.capabilities.max_completion_tokens, Some(96000));
        assert_eq!(model.tier, Tier::T1Deterministic);
        assert!(!model.available);

        let mut with_client_tools = raw.clone();
        with_client_tools["supports_tools"] = json!(true);
        let capable = normalize(&target, &with_client_tools, true).unwrap();
        assert_eq!(capable.tier, Tier::T3Architect);
        assert!(capable.available);

        let mut unsupported = with_client_tools;
        unsupported["capabilities"]["structured_outputs"]["supported"] = json!(false);
        unsupported["capabilities"]["image_input"]["supported"] = json!(false);
        unsupported["capabilities"]["effort"]["supported"] = json!(false);
        unsupported["capabilities"]["thinking"]["supported"] = json!(false);
        let unsupported = normalize(&target, &unsupported, true).unwrap();
        assert_eq!(unsupported.capabilities.structured_outputs, Some(false));
        assert_eq!(unsupported.capabilities.vision, Some(false));
        assert!(unsupported.capabilities.reasoning_efforts.is_empty());
        assert!(unsupported.capabilities.thinking_modes.is_empty());
        assert_eq!(unsupported.tier, Tier::T1Deterministic);
    }

    #[test]
    fn anthropic_lifecycle_is_enforced_and_family_labels_do_not_rank_models() {
        let target = endpoint_target(
            "anthropic-fixture",
            Protocol::Anthropic,
            "https://example.test/v1".into(),
            None,
        );
        let raw = json!({
            "id":"synthetic-future-opus-frontier",
            "display_name":"Synthetic Opus Frontier",
            "description":"Full-size flagship with deep reasoning",
            "line":"haiku",
            "supports_tools":true,
            "max_tokens":65536,
            "capabilities":{
                "structured_outputs":{"supported":true},
                "effort":{"supported":true,"high":{"supported":true}},
                "thinking":{"supported":false,"types":{"enabled":{"supported":true}}}
            }
        });
        let utility = normalize(&target, &raw, true).unwrap();
        assert_eq!(utility.tier, Tier::T3Architect);
        assert!(utility.available);
        assert!(utility.capabilities.thinking_modes.is_empty());

        let mut retired = raw.clone();
        retired["line"] = json!("opus");
        retired["lifecycle"] = json!("retired");
        let retired = normalize(&target, &retired, true).unwrap();
        assert_eq!(retired.tier, Tier::T3Architect);
        assert!(!retired.available);
    }

    #[test]
    fn ollama_capability_strings_expose_thinking_without_assuming_structured_output() {
        let target = endpoint_target(
            "ollama-fixture",
            Protocol::Ollama,
            "http://127.0.0.1:11434".into(),
            None,
        );
        let model = normalize(
            &target,
            &json!({"name":"synthetic-future-core","capabilities":["tools","vision","thinking"]}),
            true,
        )
        .unwrap();
        assert_eq!(model.capabilities.tools, Some(true));
        assert_eq!(model.capabilities.vision, Some(true));
        assert_eq!(
            model.capabilities.thinking_modes,
            vec!["thinking".to_string()]
        );
        assert_eq!(model.capabilities.structured_outputs, None);
        assert!(!model.available);
    }

    #[test]
    fn cache_fingerprint_changes_with_credentials_and_executable_bytes() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().into()).unwrap();
        let executable = home.path().join("native-fixture");
        std::fs::write(&executable, b"first native binary").unwrap();
        let target = Target {
            name: "native-fixture".into(),
            protocol: Protocol::Claude,
            endpoint: None,
            credential_env: Some("MIRKET_ROUTER_TEST_CREDENTIAL".into()),
            executable: Some(executable.clone()),
        };
        let fingerprint = |value: Option<&str>| {
            cache_fingerprint_with_env(&paths, std::slice::from_ref(&target), |_| {
                value.map(OsString::from)
            })
            .unwrap()
        };
        let first = fingerprint(Some("first-secret"));
        assert_ne!(first, fingerprint(None));
        assert_ne!(first, fingerprint(Some("second-secret")));
        std::fs::write(&executable, b"second native binary").unwrap();
        assert_ne!(first, fingerprint(Some("first-secret")));
    }

    #[test]
    fn cache_reclassification_rejects_forged_tier_and_unready_target() {
        let target = endpoint_target(
            "fixture",
            Protocol::OpenaiChat,
            "http://127.0.0.1:1/v1".into(),
            None,
        );
        let mut model = normalize(&target, &json!({"id":"unknown-future"}), true).unwrap();
        model.tier = Tier::T3Architect;
        model.available = true;
        let mut registry = Registry {
            version: REGISTRY_VERSION,
            refreshed_at: now(),
            fingerprint: "fixture".into(),
            targets: vec![TargetStatus {
                target,
                authenticated: true,
                execution_ready: false,
                diagnostic: None,
            }],
            models: vec![model],
        };
        reclassify_cached_models(&mut registry);
        assert_eq!(registry.models[0].tier, Tier::T1Deterministic);
        assert!(!registry.models[0].available);
    }

    #[test]
    fn pagination_rejects_incomplete_catalog_at_page_limit() {
        let mut cursors = BTreeSet::new();
        let page = json!({"has_more":true,"last_id":"future/model?x=1"});
        assert_eq!(
            next_model_page(&page, 0, &mut cursors).unwrap().as_deref(),
            Some("models?after_id=future%2Fmodel%3Fx%3D1&limit=100")
        );
        assert!(next_model_page(&page, 1, &mut cursors).is_err());
        let mut cursors = BTreeSet::new();
        assert!(next_model_page(&page, 63, &mut cursors).is_err());
    }
}
