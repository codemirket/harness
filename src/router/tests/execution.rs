#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_context_preserves_objective_and_identifies_exact_omissions() {
        let entries = vec![json!({"objective":"keep this constraint"}), json!({"tool":"workspace_read","result":"x".repeat(1000)}), json!({"assistant":{"action":"complete"}})];
        let context = active_context(&entries, 150).unwrap();
        assert_eq!(context["observations"][0]["content"]["objective"], "keep this constraint");
        assert_eq!(context["context_omissions"][0]["index"], 1);
        assert_eq!(context["context_omissions"][0]["sha256"], util::sha256(&serde_json::to_vec(&entries[1]).unwrap()));
        assert_eq!(entries.len(), 3);
    }
    #[test]
    fn no_change_requires_complete_successful_source_receipts() {
        let receipt = json!({"tool":"workspace_read","passed":true,"result":{"complete":true,"read_receipt":{"path":"/project/a","sha256":"abc"}}});
        assert_eq!(full_read_receipts(std::slice::from_ref(&receipt)).len(), 1);
        let mut partial = receipt; partial["result"]["complete"] = json!(false);
        assert!(full_read_receipts(&[partial]).is_empty());
    }
    #[test]
    fn comparison_inputs_remain_bound_without_clearing_pending_inspection() {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let path = dir.path().join("candidate.png");
        std::fs::write(&path, b"candidate pixels").unwrap();
        let candidate = ArtifactRef {
            path,
            sha256: util::sha256(b"candidate pixels"),
        };
        let mut pending = vec![candidate.clone()];
        let mut stable = vec![];
        let mut verified = BTreeSet::new();
        // Report-only comparison has no verifies receipt, whether or not a diff was saved.
        record_artifacts(&mut pending, &mut stable, &mut verified, &[], &[]);
        record_consumed(&mut stable, std::slice::from_ref(&candidate));
        assert_eq!(pending, vec![candidate.clone()]);
        assert!(matching_receipts(&pending, &[]).is_empty());
        assert!(verification_evidence(
            "design/odiff_compare",
            &pending,
            &[],
            &["design/odiff_compare".into()]
        ));
        assert!(!verification_evidence(
            "inspect_image",
            &pending,
            &[],
            &["inspect_image".into()]
        ));
        assert!(verified.is_empty());
        assert_eq!(stable, vec![candidate.clone()]);
        // An explicit successful equality gate may supply this exact receipt.
        assert_eq!(
            matching_receipts(&pending, std::slice::from_ref(&candidate)),
            pending
        );
        std::fs::write(&candidate.path, b"changed input").unwrap();
        assert_ne!(
            tools::current_digest(&stable[0].path).unwrap(),
            stable[0].sha256
        );
    }
    #[test]
    fn schema_discovery_does_not_satisfy_document_inspection() {
        let mut result = tools::ToolResult {
            value: json!({"tools":[{"name":"get_page_tree"}]}),
            image: None,
            mutates: false,
            passed: true,
            artifacts: vec![],
            stable_artifacts: vec![],
            consumed_artifacts: vec![],
            verifies: vec![],
            opaque_mutation: false,
        };
        assert!(!production_evidence("design/openpencil_inspect", &result));
        assert!(production_evidence("workspace_list", &result));
        result.verifies.push(ArtifactRef {
            path: PathBuf::from("saved.fig"),
            sha256: "a".repeat(64),
        });
        assert!(production_evidence("design/openpencil_inspect", &result));
        result.passed = false;
        assert!(!production_evidence("design/openpencil_inspect", &result));
    }
    #[tokio::test]
    async fn failed_native_vision_dispatch_retains_diagnostics_without_usage_ledger() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().to_path_buf()).unwrap();
        let mut registry = Registry {
            version: 2,
            refreshed_at: 0,
            fingerprint: String::new(),
            targets: vec![discovery::TargetStatus {
                target: Target {
                    name: "fixture".into(),
                    protocol: Protocol::Claude,
                    endpoint: None,
                    credential_env: None,
                    executable: Some(paths.home.join("absent-native-client")),
                },
                authenticated: true,
                execution_ready: true,
                diagnostic: None,
            }],
            models: vec![
                serde_json::from_value(json!({"id":"probe-model","target":"fixture",
                "display_name":"Fixture","description":"","tier":"T3_ARCHITECT","available":true,
                "capabilities":{"tools":true,"structured_outputs":true,"vision":null,
                "context_window":262144,"reasoning_efforts":["high"]}}))
                .unwrap(),
            ],
        };
        let resolution: Resolution = serde_json::from_value(json!({"min_tier":"T1_DETERMINISTIC",
            "requires_vision":true,"requires_structured_outputs":true,"requires_tools":false,
            "effort_profile":"high","requested_effort":"high","downgrade_rejected":false,"reasons":[]})).unwrap();
        for persist in [true, false] {
            let error = verify_vision(&paths, &mut registry, &resolution, persist)
                .await
                .unwrap_err();
            let diagnostic = error.to_string();
            assert!(diagnostic.contains("native vision conformance not established"));
            assert!(diagnostic.contains("first_failures"));
            assert!(!paths.root.join("runs").exists());
        }
    }
    #[cfg(unix)]
    #[test]
    fn native_vision_probe_accepts_system_temp_alias_but_not_image_aliases() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path().canonicalize().unwrap();
        let real = root.join("real");
        std::fs::create_dir(&real).unwrap();
        let alias = root.join("alias");
        std::os::unix::fs::symlink(&real, &alias).unwrap();
        // Reproduce the former macOS /var temporary path failure.
        let old = tempfile::tempdir_in(&alias).unwrap();
        let old_image = old.path().join("old.png");
        std::fs::write(&old_image, vision_fixture("abcd").unwrap().0).unwrap();
        assert!(util::read_bounded(&old_image, 4 * 1024 * 1024).is_err());
        let (_owner, image, expected) = vision_probe(&alias).unwrap();
        let bytes = util::read_bounded(&image, 4 * 1024 * 1024).unwrap();
        assert_eq!(
            super::super::native::image_media_type(&bytes),
            Some("image/png")
        );
        assert_eq!(expected.split(',').count(), 4);
        let image_alias = root.join("image.png");
        std::os::unix::fs::symlink(&image, &image_alias).unwrap();
        assert!(util::read_bounded(&image_alias, 4 * 1024 * 1024).is_err());
    }
    #[test]
    fn vision_probe_distinguishes_contract_failure_from_incorrect_pixels() {
        let action =
            |answer| json!({"action":"complete","tool":"","arguments_json":"{}","result":answer});
        let (passed, _) = vision_outcome(Ok(action("red,blue")), "red,blue");
        assert!(passed);
        let (passed, wrong) = vision_outcome(Ok(action("private incorrect answer")), "red,blue");
        assert!(!passed);
        assert_eq!(wrong["code"], "vision_answer_mismatch");
        assert_eq!(wrong["response_sha256"].as_str().unwrap().len(), 64);
        assert!(!wrong.to_string().contains("private"));
        let (_, invalid) = vision_outcome(Ok(json!({"private":"provider payload"})), "red,blue");
        assert_eq!(invalid["code"], "invalid_vision_action");
        let (_, transport) = vision_outcome(
            Err(anyhow::anyhow!("private provider detail")
                .context("native child closed its response stream")),
            "red,blue",
        );
        assert_eq!(transport["native_code"], "response_stream_closed");
        assert!(!transport.to_string().contains("private"));
        let (_, untrusted) = vision_outcome(
            Err(anyhow::anyhow!(
                "native child closed its response stream: secret"
            )),
            "red,blue",
        );
        assert!(untrusted["native_code"].is_null());
        assert!(!untrusted.to_string().contains("secret"));
    }

    #[test]
    fn paged_readback_needs_all_bytes_same_digest_after_last_mutation() {
        let artifact = ArtifactRef {
            path: PathBuf::from("/project/file"),
            sha256: "a".repeat(64),
        };
        let page = |offset, end| json!({"tool":"workspace_read","passed":true,"result":{"read_receipt":{"path":artifact.path,"sha256":artifact.sha256,"offset":offset,"end":end,"total":200000}}});
        let first = page(0, 131072);
        let last = page(131072, 200000);
        let pending = [artifact.clone()];
        assert!(paged_readbacks(std::slice::from_ref(&first), &pending).is_empty());
        assert!(paged_readbacks(std::slice::from_ref(&last), &pending).is_empty());
        assert_eq!(
            paged_readbacks(&[first.clone(), last.clone()], &pending),
            pending
        );
        assert!(
            paged_readbacks(
                &[
                    first.clone(),
                    json!({"tool":"workspace_write"}),
                    last.clone()
                ],
                &pending
            )
            .is_empty()
        );
        let mut wrong = last;
        wrong["result"]["read_receipt"]["sha256"] = json!("b".repeat(64));
        assert!(paged_readbacks(&[first, wrong], &pending).is_empty());
    }
    #[test]
    fn resume_contract_ignores_unused_candidates_and_observation_metadata() {
        let provider = Target {
            name: "fixture".into(),
            protocol: Protocol::OpenaiChat,
            endpoint: Some("http://127.0.0.1:1234/v1".into()),
            credential_env: None,
            executable: None,
        };
        let original: Model =
            serde_json::from_value(json!({"id":"a","target":"fixture","display_name":"A",
            "description":"old","tier":"T3_ARCHITECT","available":true,"capabilities":{
                "tools":true,"structured_outputs":true,"vision":true,"context_window":262144,
                "reasoning_efforts":["high"],"pricing":{"input":1}}}))
            .unwrap();
        let bindings = json!([provider_binding(&provider).unwrap()]);
        let mut registry = Registry {
            version: 2,
            refreshed_at: 0,
            fingerprint: String::new(),
            targets: vec![discovery::TargetStatus {
                target: provider,
                authenticated: true,
                execution_ready: true,
                diagnostic: None,
            }],
            models: vec![original.clone()],
        };
        let mut unused = original.clone();
        unused.id = "removed-fallback".into();
        registry.models[0].description = "new description".into();
        registry.models[0].display_name = "new label".into();
        registry.models[0]
            .classification_evidence
            .push("new explanation".into());
        registry.models[0].capabilities.pricing = Some(json!({"input":2}));
        compatible_model(&registry, &original, &bindings).unwrap();
        assert!(compatible_model(&registry, &unused, &bindings).is_err());
        let mut later = original.clone();
        later.id = "later".into();
        registry.models.push(later.clone());
        let saved = vec![original.clone(), unused.clone(), later];
        let exclusions = revalidate_candidates(&mut registry, &saved, 0, &bindings).unwrap();
        assert_eq!(exclusions.len(), 1);
        assert_eq!(exclusions[0]["resume_candidate_excluded"], 1);
        assert_eq!(next_candidate(&registry, &saved, 0), Some(2));
        assert!(revalidate_candidates(&mut registry, &saved, 1, &bindings).is_err());
        assert!(revalidate_candidates(&mut registry, &saved, 3, &bindings).is_err());

        for field in [
            "tools",
            "structured_outputs",
            "vision",
            "context_window",
            "max_completion_tokens",
            "thinking_budget_max",
            "reasoning_efforts",
            "thinking_modes",
        ] {
            let mut changed = serde_json::to_value(&original).unwrap();
            changed["capabilities"][field] = match field {
                "reasoning_efforts" | "thinking_modes" => json!(["different"]),
                "tools" | "structured_outputs" | "vision" => json!(false),
                _ => json!(123),
            };
            registry.models[0] = serde_json::from_value(changed).unwrap();
            assert!(
                compatible_model(&registry, &original, &bindings).is_err(),
                "{field}"
            );
        }
        registry.models[0] = original.clone();
        registry.models[0].capabilities.vision = None;
        assert!(
            compatible_model(&registry, &original, &bindings).is_err(),
            "unknown vision needs fresh conformance"
        );
        registry.models[0].capabilities.vision = Some(true);
        compatible_model(&registry, &original, &bindings).unwrap();
        registry.targets[0].target.endpoint = Some("http://127.0.0.1:5678/v1".into());
        assert!(compatible_model(&registry, &original, &bindings).is_err());
    }

    #[test]
    fn resume_provider_binding_detects_replaced_native_executable() {
        let home = tempfile::tempdir().unwrap();
        let executable = home.path().join("native");
        std::fs::write(&executable, b"first").unwrap();
        let provider = Target {
            name: "native".into(),
            protocol: Protocol::Claude,
            endpoint: None,
            credential_env: None,
            executable: Some(executable.clone()),
        };
        let before = provider_binding(&provider).unwrap();
        std::fs::write(executable, b"other").unwrap();
        assert_ne!(before, provider_binding(&provider).unwrap());
    }

    #[test]
    fn recovered_write_keeps_readback_and_budget_obligations() {
        let directory = tempfile::tempdir().unwrap();
        let project = directory.path().canonicalize().unwrap();
        let path = project.join("result.txt");
        std::fs::write(&path, "old").unwrap();
        let request: SpawnRequest = serde_json::from_value(json!({"objective":"Update result", "task_intent":"architecture", "complexity_class":"creative_architectural", "assigned_tools":[], "task_needs":[], "project":project})).unwrap();
        let mut state =
            ExecutionState::new(&request, &design_gate::ExecutionPolicy::from_workflows(&[]));
        state.turn = 7;
        let deadline = state.deadline;
        let before = ArtifactRef {
            path: path.clone(),
            sha256: util::sha256(b"old"),
        };
        state.pending.push(before.clone());
        state.verified.insert((path.clone(), before.sha256.clone()));
        state.completed.insert("workspace_read".into());
        let args = json!({"path":"result.txt","content":"new","expected_sha256":before.sha256});
        let intent = tools::write_intent(&project, &args).unwrap();
        std::fs::write(&path, "new").unwrap();
        state
            .recover_write(
                &Paths::new(project.clone()).unwrap(),
                &project,
                &json!({"arguments":args,"write_intent":intent}),
            )
            .unwrap();
        assert_eq!(
            state.pending,
            vec![ArtifactRef {
                path,
                sha256: util::sha256(b"new")
            }]
        );
        assert!(state.verified.is_empty());
        assert!(!state.completed.contains("workspace_read"));
        assert_eq!(state.turn, 7);
        assert_eq!(state.deadline, deadline);
        state.revalidate().unwrap();
        assert_eq!(state.transcript.last().unwrap()["replayed"], false);
    }

    #[test]
    fn explicit_model_selection_prevents_other_target_and_model_candidates() {
        let mut registry = Registry {
            version: 2,
            refreshed_at: 0,
            fingerprint: String::new(),
            targets: vec![],
            models: vec![],
        };
        for (target, id) in [
            ("chosen", "model-a"),
            ("other", "model-a"),
            ("chosen", "model-b"),
        ] {
            registry.models.push(serde_json::from_value(json!({"id":id,"target":target,"display_name":id,"description":"fixture","tier":"T3_ARCHITECT","available":true,"capabilities":{}})).unwrap());
        }
        let mut request: SpawnRequest = serde_json::from_value(json!({"objective":"Review artwork","task_intent":"creativity","complexity_class":"creative_architectural","assigned_tools":[],"task_needs":[],"project":"/project","model_selection":{"target":"chosen","id":"model-a","effort":"high"}})).unwrap();
        constrain_models(&mut registry, &request);
        assert_eq!(registry.models.len(), 1);
        assert_eq!(registry.models[0].target, "chosen");
        assert_eq!(registry.models[0].id, "model-a");
        request.model_selection.as_mut().unwrap().id = "missing".into();
        constrain_models(&mut registry, &request);
        assert!(registry.models.is_empty());
    }
    #[test]
    fn schema_has_no_model_override() {
        let s = action_schema();
        assert_eq!(s["additionalProperties"], false);
        assert!(s["properties"].get("model").is_none());
    }
    #[test]
    fn fixture_is_real_png_and_randomized() {
        let (a, ar) = vision_fixture("0123").unwrap();
        let (b, br) = vision_fixture("abcd").unwrap();
        assert!(a.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert_ne!(a, b);
        assert_ne!(ar, br);
    }
    #[test]
    fn failover_does_not_retry_policy_or_identity_failures() {
        assert!(retryable(&anyhow::anyhow!("HTTP status 429")));
        assert!(!retryable(&anyhow::anyhow!(
            "target changed selected model identity"
        )));
        assert!(!retryable(&anyhow::anyhow!("tool floor rejected")));
    }

    #[test]
    fn unrelated_image_receipt_cannot_satisfy_artifact_obligation() {
        let pending = vec![ArtifactRef {
            path: PathBuf::from("/workspace/render.png"),
            sha256: "a".repeat(64),
        }];
        let unrelated = ArtifactRef {
            path: PathBuf::from("/workspace/other.png"),
            sha256: "b".repeat(64),
        };
        assert!(matching_receipts(&pending, &[unrelated]).is_empty());
        assert_eq!(matching_receipts(&pending, &pending), pending);
    }

    #[test]
    fn rewriting_same_artifact_invalidates_prior_verification() {
        let path = PathBuf::from("/workspace/render.png");
        let first = ArtifactRef {
            path: path.clone(),
            sha256: "a".repeat(64),
        };
        let second = ArtifactRef {
            path: path.clone(),
            sha256: "b".repeat(64),
        };
        let mut pending = vec![first.clone()];
        let mut stable = Vec::new();
        let mut verified = BTreeSet::from([(path.clone(), first.sha256.clone())]);
        record_artifacts(
            &mut pending,
            &mut stable,
            &mut verified,
            std::slice::from_ref(&second),
            &[],
        );
        assert_eq!(pending, vec![second]);
        assert!(verified.is_empty());
    }

    #[test]
    fn visual_revision_keeps_references_but_requires_fresh_verification() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().canonicalize().unwrap();
        let reference = dir.join("brand.png");
        let output = dir.join("output.png");
        std::fs::write(&reference, b"reference pixels").unwrap();
        std::fs::write(&output, b"old pixels").unwrap();
        let mut images = Vec::new();
        let mut transcript = Vec::new();
        retain_image(
            &mut images,
            reference.clone(),
            tools::ImagePurpose::Reference,
            &mut transcript,
        )
        .unwrap();
        retain_image(
            &mut images,
            output.clone(),
            tools::ImagePurpose::Artifact,
            &mut transcript,
        )
        .unwrap();
        let mut completed = BTreeSet::from(["inspect_image".into()]);
        let mut inspected = true;

        // A source edit leaves both images available for comparison, but neither
        // establishes that the new source was rendered and inspected.
        invalidate_visual_context(&mut images, &mut completed, &mut inspected, &mut transcript);
        assert_eq!(images.len(), 2);
        assert!(completed.is_empty());
        assert!(!inspected);

        std::fs::write(&output, b"new pixels").unwrap();
        invalidate_visual_context(&mut images, &mut completed, &mut inspected, &mut transcript);
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].path, reference);
        assert!(
            transcript
                .iter()
                .any(|v| v["image_context"] == "changed_or_unavailable")
        );
        std::fs::remove_file(reference).unwrap();
        invalidate_visual_context(&mut images, &mut completed, &mut inspected, &mut transcript);
        assert!(images.is_empty());
    }

    #[test]
    fn visual_reference_priority_stays_within_count_and_byte_limits() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().canonicalize().unwrap();
        let mut images = Vec::new();
        let mut transcript = Vec::new();
        for n in 0..6 {
            let path = dir.join(format!("{n}.png"));
            std::fs::write(&path, [n]).unwrap();
            retain_image(
                &mut images,
                path,
                if n < 3 {
                    tools::ImagePurpose::Reference
                } else {
                    tools::ImagePurpose::Artifact
                },
                &mut transcript,
            )
            .unwrap();
        }
        assert_eq!(images.len(), 4);
        assert_eq!(
            images
                .iter()
                .filter(|i| i.purpose == tools::ImagePurpose::Reference)
                .count(),
            2
        );
        assert_eq!(images[0].path, dir.join("1.png"));
        assert_eq!(images.last().unwrap().path, dir.join("5.png"));
        let big = dir.join("large.png");
        std::fs::write(&big, vec![0; 4 * 1024 * 1024]).unwrap();
        retain_image(
            &mut images,
            big.clone(),
            tools::ImagePurpose::Comparison,
            &mut transcript,
        )
        .unwrap();
        let other = dir.join("other.png");
        std::fs::write(&other, vec![0; 4 * 1024 * 1024]).unwrap();
        retain_image(
            &mut images,
            other.clone(),
            tools::ImagePurpose::Artifact,
            &mut transcript,
        )
        .unwrap();
        assert!(images.iter().map(|i| i.bytes).sum::<u64>() <= MAX_RETAINED_IMAGE_BYTES);
        assert!(images.iter().any(|i| i.path == other));
        assert!(!images.iter().any(|i| i.path == big));
        // Two large references cannot crowd the newest inspected output out of
        // the very next inference step.
        images.clear();
        for name in ["first-reference.png", "second-reference.png"] {
            let path = dir.join(name);
            std::fs::write(&path, vec![0; 4 * 1024 * 1024]).unwrap();
            retain_image(
                &mut images,
                path,
                tools::ImagePurpose::Reference,
                &mut transcript,
            )
            .unwrap();
        }
        retain_image(
            &mut images,
            other.clone(),
            tools::ImagePurpose::Artifact,
            &mut transcript,
        )
        .unwrap();
        assert_eq!(images.len(), 2);
        assert_eq!(images.last().unwrap().path, other);
    }
}
