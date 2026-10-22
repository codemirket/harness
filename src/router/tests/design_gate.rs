#[cfg(test)]
mod tests {
    use super::*;

    fn test_policy(needs: &[super::super::TaskNeed]) -> ExecutionPolicy {
        let catalog = crate::catalog::Catalog::embedded().unwrap();
        ExecutionPolicy::from_workflows(
            &needs
                .iter()
                .map(|n| catalog.workflow(&n.workflow).unwrap())
                .collect::<Vec<_>>(),
        )
    }
    fn gate(input: DesignInput, needs: &[super::super::TaskNeed]) -> Gate {
        Gate::new(input, &test_policy(needs))
    }

    #[test]
    fn static_review_can_omit_irrelevant_axes_but_not_current_pixels() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let artifact = image(&root, "target.png");
        let mut gate = fallback();
        let mut review = report("target", &artifact, true);
        review["checks"].as_object_mut().unwrap().retain(|key, _| key == "composition");
        assert!(gate.review(&root, review.clone()).is_err());
        observe(&mut gate, &artifact, true);
        assert!(gate.review(&root, review).is_ok());
    }

    #[tokio::test]
    async fn deadline_stops_pending_execution_and_keeps_finished_results() {
        use std::time::Duration;
        let pending = std::future::pending::<Result<()>>();
        assert!(
            within_budget(pending, Duration::from_millis(1))
                .await
                .is_err()
        );
        assert_eq!(
            within_budget(async { Ok(7) }, Duration::from_secs(1))
                .await
                .unwrap(),
            7
        );
    }

    fn image(root: &Path, name: &str) -> ArtifactRef {
        let path = root.join(name);
        std::fs::write(&path, name).unwrap();
        ArtifactRef {
            sha256: tools::current_digest(&path).unwrap(),
            path,
        }
    }
    fn receipt(artifact: &ArtifactRef, render: bool) -> ToolResult {
        ToolResult {
            value: json!({}),
            image: (!render).then(|| artifact.path.clone()),
            mutates: render,
            passed: true,
            artifacts: if render {
                vec![artifact.clone()]
            } else {
                vec![]
            },
            stable_artifacts: vec![],
            consumed_artifacts: vec![],
            verifies: if render {
                vec![]
            } else {
                vec![artifact.clone()]
            },
            opaque_mutation: false,
        }
    }
    fn observe(gate: &mut Gate, artifact: &ArtifactRef, render: bool) {
        if render {
            gate.observe("design/browser_screenshot", &receipt(artifact, true));
        }
        gate.observe("inspect_image", &receipt(artifact, false));
    }
    fn report(stage: &str, artifact: &ArtifactRef, pass: bool) -> Value {
        let mut checks = serde_json::Map::new();
        for axis in [
            "composition",
            "typography",
            "assets",
            "responsive",
            "behavior",
            "motion",
        ] {
            checks.insert(
                axis.into(),
                json!({"status":"pass","observation":format!("Observed {axis} in this fixture")}),
            );
        }
        if !pass {
            checks["composition"]["status"] = json!("fail");
        }
        json!({"stage":stage,"images":[artifact.path],"comparison":"Matched hierarchy; narrow crop preserves the focal subject.","checks":checks})
    }
    fn fallback() -> Gate {
        gate(
            DesignInput::CodeFallback {
                reason: "No host image model".into(),
            },
            &[],
        )
    }
    fn mutation() -> ToolResult {
        ToolResult {
            value: json!({}),
            image: None,
            mutates: true,
            passed: true,
            artifacts: vec![],
            stable_artifacts: vec![],
            consumed_artifacts: vec![],
            verifies: vec![],
            opaque_mutation: false,
        }
    }

    #[test]
    fn malformed_reviews_return_feedback_without_consuming_or_resetting_review_budget() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let target = image(&root, "target.png");
        let mut gate = fallback();
        observe(&mut gate, &target, true);
        for malformed in ["not-json".to_owned(), "{}".to_owned(), "x".repeat(16385)] {
            assert_eq!(gate.review_feedback(&root, &malformed)["accepted"], false);
            assert_eq!(gate.target_reviews, 0);
            assert!(gate.complete().is_err());
        }
        let target_review = report("target", &target, true).to_string();
        assert_eq!(
            gate.review_feedback(&root, &target_review)["target_frozen"],
            true
        );
        assert_eq!(gate.target_reviews, 1);
        let failed = report("final", &target, false).to_string();
        for _ in 0..=gate.budget.max_correction_rounds {
            assert_eq!(gate.review_feedback(&root, &failed)["passed"], false);
        }
        assert_eq!(
            gate.review_feedback(&root, &report("final", &target, true).to_string())["accepted"],
            false
        );
        assert!(gate.complete().is_err());
        assert!(gate.before_tool("workspace_write").is_err());
    }

    #[test]
    fn image_to_code_blocks_effects_until_declared_target_is_inspected_and_passed() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let target = image(&root, "target.png");
        let unrelated = image(&root, "other.png");
        let mut gate = gate(
            DesignInput::ImageToCode {
                image: "target.png".into(),
            },
            &[],
        );
        assert!(gate.before_tool("workspace_write").is_err());
        assert!(gate.before_tool("workspace_read").is_ok());
        assert!(gate.review(&root, report("target", &target, true)).is_err());
        observe(&mut gate, &unrelated, false);
        assert!(
            gate.review(&root, report("target", &unrelated, true))
                .is_err()
        );
        observe(&mut gate, &target, false);
        gate.review(&root, report("target", &target, true)).unwrap();
        assert!(gate.before_tool("workspace_write").is_ok());
        assert!(gate.complete().is_err());
        assert!(gate.review(&root, report("final", &target, true)).is_err());
        std::fs::write(&target.path, "replaced target").unwrap();
        assert!(gate.before_tool("workspace_write").is_err());
    }

    #[test]
    fn fallback_requires_rendered_proof_then_final_comparison_and_rejects_stale_output() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let target = image(&root, "target.png");
        let output = image(&root, "output.png");
        let mut gate = fallback();
        assert!(gate.before_tool("workspace_write").is_ok());
        observe(&mut gate, &target, false);
        assert!(gate.review(&root, report("target", &target, true)).is_err());
        observe(&mut gate, &target, true);
        gate.review(&root, report("target", &target, true)).unwrap();
        gate.observe("workspace_write", &mutation());
        assert!(gate.review(&root, report("final", &target, true)).is_err());
        observe(&mut gate, &output, true);
        gate.review(&root, report("final", &output, true)).unwrap();
        gate.complete().unwrap();
        gate.observe("workspace_write", &mutation());
        assert!(gate.complete().is_err());
        assert!(gate.review(&root, report("final", &output, true)).is_err());
    }

    #[test]
    fn native_specimen_needs_a_current_render_and_cannot_change_its_target() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let target = image(&root, "native.png");
        let output = image(&root, "final.png");
        let input: DesignInput = serde_json::from_value(json!({
            "mode":"native_specimen", "medium":"svg", "reason":"Editable attached vector geometry"
        }))
        .unwrap();
        input.validate().unwrap();
        let mut gate = gate(input, &[]);
        assert!(gate.before_tool("workspace_write").is_ok());
        observe(&mut gate, &target, false);
        assert!(gate.review(&root, report("target", &target, true)).is_err());
        observe(&mut gate, &target, true);
        gate.review(&root, report("target", &target, true)).unwrap();
        gate.observe("workspace_write", &mutation());
        assert!(gate.review(&root, report("final", &target, true)).is_err());
        observe(&mut gate, &output, true);
        gate.review(&root, report("final", &output, true)).unwrap();
        gate.complete().unwrap();
        std::fs::write(&target.path, "replacement").unwrap();
        assert!(gate.complete().is_err());
        assert!(gate.before_tool("workspace_write").is_err());
    }

    #[test]
    fn native_input_requires_suitability_and_a_supported_medium() {
        for medium in [
            "dom",
            "svg",
            "pixel",
            "diagram",
            "typeset",
            "design_document",
        ] {
            let mut value = json!({"mode":"native_specimen", "medium":medium, "reason":"Appropriate native proof"});
            let input: DesignInput = serde_json::from_value(value.clone()).unwrap();
            input.validate().unwrap();
            value["reason"] = json!(" ");
            assert!(
                serde_json::from_value::<DesignInput>(value.clone())
                    .unwrap()
                    .validate()
                    .is_err()
            );
            value["reason"] = json!("Ready");
            value["execution_seconds"] = json!(999999);
            assert!(serde_json::from_value::<DesignInput>(value).is_err());
        }
        assert!(
            serde_json::from_value::<DesignInput>(
                json!({"mode":"native_specimen", "medium":"unknown", "reason":"Ready"})
            )
            .is_err()
        );
    }

    #[test]
    fn deliverable_budgets_are_shared_and_enforce_the_resolved_review_limit() {
        for (names, seconds, corrections) in [
            (vec!["vector-art", "diagram", "static-graphic"], 900, 2),
            (vec!["web-interface"], 1800, 3),
            (vec!["editable-interface", "paged-media"], 1800, 3),
            (vec!["motion"], 2700, 4),
            (
                vec!["interactive-motion", "web-interface", "vector-art"],
                2700,
                4,
            ),
        ] {
            let needs: Vec<_> = names
                .iter()
                .map(|name| super::super::TaskNeed {
                    workflow: (*name).into(),
                    reason: "Actual deliverable".into(),
                })
                .collect();
            let input = DesignInput::NativeSpecimen {
                medium: Medium::Dom,
                reason: "Native composition".into(),
            };
            let contract = contract(&input, &test_policy(&needs));
            assert_eq!(contract["execution_seconds"], seconds);
            assert_eq!(contract["max_correction_rounds"], corrections);
            let mut gate = gate(input, &needs);
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().canonicalize().unwrap();
            let target = image(&root, "target.png");
            observe(&mut gate, &target, true);
            gate.review(&root, report("target", &target, true)).unwrap();
            for remaining in (0..=corrections).rev() {
                let review = gate.review(&root, report("final", &target, false)).unwrap();
                assert_eq!(review["final_reviews_remaining"], remaining);
                assert_eq!(gate.before_tool("workspace_write").is_ok(), remaining > 0);
            }
            assert!(gate.review(&root, report("final", &target, true)).is_err());
            assert!(gate.complete().is_err());
        }
    }

    #[test]
    fn required_checks_cannot_be_waived_or_replaced_with_a_generic_score() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let target = image(&root, "target.png");
        let mut gate = gate(
            DesignInput::CodeFallback {
                reason: "Unavailable image model".into(),
            },
            &[super::super::TaskNeed {
                workflow: "interactive-motion".into(),
                reason: "Animated UI".into(),
            }],
        );
        observe(&mut gate, &target, true);
        gate.review(&root, report("target", &target, true)).unwrap();
        for axis in ["composition", "responsive", "behavior", "motion"] {
            let mut r = report("final", &target, true);
            r["checks"][axis]["status"] = json!("not_applicable");
            assert!(gate.review(&root, r).is_err());
        }
        let mut r = report("final", &target, true);
        r["checks"] = json!({"score":10});
        assert!(gate.review(&root, r).is_err());
    }

    #[test]
    fn failed_checks_cannot_complete_or_create_an_unbounded_repair_loop() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let target = image(&root, "target.png");
        let mut gate = fallback();
        observe(&mut gate, &target, true);
        gate.review(&root, report("target", &target, true)).unwrap();
        for _ in 0..3 {
            assert_eq!(
                gate.review(&root, report("final", &target, false)).unwrap()["passed"],
                false
            );
            assert!(gate.complete().is_err());
        }
        assert!(gate.before_tool("workspace_write").is_err());
        assert!(gate.review(&root, report("final", &target, true)).is_err());
    }

    #[test]
    fn target_repairs_are_bounded_and_target_cannot_be_reselected() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let target = image(&root, "target.png");
        let mut gate = fallback();
        observe(&mut gate, &target, true);
        for _ in 0..2 {
            gate.review(&root, report("target", &target, false))
                .unwrap();
        }
        assert!(gate.before_tool("workspace_write").is_err());
        assert!(gate.review(&root, report("target", &target, true)).is_err());
        let mut gate = fallback();
        observe(&mut gate, &target, true);
        gate.review(&root, report("target", &target, true)).unwrap();
        assert!(gate.review(&root, report("target", &target, true)).is_err());
    }
}
