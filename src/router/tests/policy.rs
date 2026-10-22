#[cfg(test)]
mod tests {
    use super::*;

    fn model(name: &str, description: &str) -> Model {
        Model {
            id: format!("dynamic-{name}"),
            target: "native".into(),
            display_name: name.into(),
            description: description.into(),
            line: None,
            capabilities: Capabilities {
                tools: Some(true),
                structured_outputs: Some(true),
                vision: Some(true),
                reasoning_efforts: vec!["low".into(), "high".into()],
                max_completion_tokens: Some(100_000),
                ..Capabilities::default()
            },
            tier: Tier::T1Deterministic,
            classification_evidence: vec![],
            available: true,
        }
    }

    fn request(intent: TaskIntent, class: ComplexityClass) -> SpawnRequest {
        SpawnRequest {
            limits: Default::default(),
            persistence: Default::default(),
            allow_already_satisfied: false,
            host_review: false,
            memory_recall: Default::default(),
            memory_queries: vec![],
            objective: "complete the task".into(),
            task_intent: intent,
            complexity_class: class,
            deterministic_operation: None,
            deterministic_input: None,
            assigned_tools: vec![],
            task_needs: vec![],
            primary_role: None,
            supporting_skills: vec![],
            skill_references: vec![],
            project: PathBuf::from("/project"),
            requires_vision: false,
            requested_tier: None,
            model_selection: None,
            design_input: None,
            verification_tools: vec![],
            host_verification_checks: super::super::handoff::default_checks(),
        }
    }

    // Test calls choose an explicit owned floor independently of objective prose.
    fn resolve(request: &SpawnRequest, tools: &[ToolPolicy]) -> Result<Resolution, PolicyError> {
        let floor = match request.task_intent {
            TaskIntent::FormatConversion
            | TaskIntent::CliParameters
            | TaskIntent::SchemaExtraction
            | TaskIntent::FileHygiene => Tier::T1Deterministic,
            TaskIntent::Synthesis | TaskIntent::MultiToolChaining => Tier::T2Synthesis,
            _ => Tier::T3Architect,
        };
        super::resolve(request, tools, floor)
    }

    #[test]
    fn native_operations_have_explicit_stable_semantics() {
        assert_eq!(DeterministicOperation::SortLines.execute("b\na\nb\n"), Some("a\nb\nb".into()));
        assert_eq!(DeterministicOperation::DeduplicateLines.execute("b\na\nb\n"), Some("b\na".into()));
        assert_eq!(DeterministicOperation::TrimWhitespace.execute("  Türkçe \n"), Some("Türkçe".into()));
        assert!(DeterministicOperation::RenameFile.validate().is_err());
        assert!(DeterministicOperation::FormatCliArguments.execute("a").is_none());
        assert!(RunLimits { max_calls: 0, ..Default::default() }.validate().is_err());
    }
    #[test]
    fn prose_and_legacy_labels_do_not_override_the_owned_workflow_floor() {
        let mut task = request(
            TaskIntent::Architecture,
            ComplexityClass::CreativeArchitectural,
        );
        for objective in [
            "Debugging a concurrent failure",
            "Format architecture notes",
            "Türkçe alanları sırala",
            "Convert CSV to JSON and summarize",
        ] {
            task.objective = objective.into();
            let resolved = super::resolve(&task, &[], Tier::T1Deterministic).unwrap();
            assert_eq!(resolved.min_tier, Tier::T1Deterministic);
            assert!(!resolved.requires_vision);
        }
        task.requested_tier = Some(Tier::T1Deterministic);
        assert!(
            super::resolve(&task, &[], Tier::T3Architect)
                .unwrap()
                .downgrade_rejected
        );
    }

    #[test]
    fn future_model_uses_capability_metadata_without_id_allowlist() {
        let mut future = model("Novel Future Model", "Frontier model for complex work");
        classify(&mut future);
        assert_eq!(future.tier, Tier::T3Architect);
        assert!(future.available);
        assert!(
            future
                .classification_evidence
                .iter()
                .any(|s| s.contains("quality unmeasured"))
        );
    }

    #[test]
    fn unknown_contract_is_quarantined_even_with_frontier_label() {
        let mut unknown = model("Future Frontier", "deep reasoning");
        unknown.capabilities.tools = None;
        classify(&mut unknown);
        assert!(!unknown.available);
        assert_eq!(unknown.tier, Tier::T1Deterministic);
        let mut task = request(TaskIntent::FormatConversion, ComplexityClass::Deterministic);
        task.objective = "Convert JSON to YAML".into();
        task.deterministic_operation = Some(DeterministicOperation::Convert {
            from: "JSON".into(),
            to: "YAML".into(),
        });
        let resolution = resolve(&task, &[]).unwrap();
        assert!(rank_candidates(&[unknown], &resolution).is_empty());
    }

    #[test]
    fn creative_intent_and_owned_tool_raise_floor() {
        let creative = resolve(
            &request(TaskIntent::Layout, ComplexityClass::Deterministic),
            &[],
        )
        .unwrap();
        assert_eq!(creative.min_tier, Tier::T3Architect);
        let mut synthesis = request(TaskIntent::FormatConversion, ComplexityClass::Deterministic);
        synthesis.objective = "Convert JSON to YAML".into();
        synthesis.deterministic_operation = Some(DeterministicOperation::Convert {
            from: "JSON".into(),
            to: "YAML".into(),
        });
        synthesis.assigned_tools.push("browser".into());
        let tools = [ToolPolicy {
            name: "browser".into(),
            min_tier: Tier::T2Synthesis,
            requires_vision: true,
        }];
        let resolved = resolve(&synthesis, &tools).unwrap();
        assert_eq!(resolved.min_tier, Tier::T2Synthesis);
        assert!(resolved.requires_vision);
        assert!(resolved.requires_structured_outputs);
    }

    #[test]
    fn unknown_tools_and_unassigned_verification_fail_closed() {
        let mut req = request(TaskIntent::Synthesis, ComplexityClass::Synthesis);
        req.assigned_tools.push("unknown".into());
        assert_eq!(
            resolve(&req, &[]).unwrap_err(),
            PolicyError::UnknownTool("unknown".into())
        );
        req.verification_tools.push("other".into());
        assert_eq!(
            resolve(&req, &[]).unwrap_err(),
            PolicyError::UnassignedVerificationTool("other".into())
        );
    }

    #[test]
    fn requested_tier_is_upward_only_and_caller_complexity_cannot_lower_intent() {
        let mut req = request(TaskIntent::Debugging, ComplexityClass::Deterministic);
        req.requested_tier = Some(Tier::T1Deterministic);
        let result = resolve(&req, &[]).unwrap();
        assert_eq!(result.min_tier, Tier::T3Architect);
        assert!(result.downgrade_rejected);

        req.task_intent = TaskIntent::FormatConversion;
        req.objective = "Convert JSON to YAML".into();
        req.deterministic_operation = Some(DeterministicOperation::Convert {
            from: "JSON".into(),
            to: "YAML".into(),
        });
        req.complexity_class = ComplexityClass::Deterministic;
        req.requested_tier = Some(Tier::T3Architect);
        let raised = resolve(&req, &[]).unwrap();
        assert_eq!(raised.min_tier, Tier::T3Architect);
        assert!(!raised.downgrade_rejected);
    }

    #[test]
    fn vision_and_high_effort_are_checked_at_candidate_boundary() {
        let mut visual = request(
            TaskIntent::MultiStepVisualQa,
            ComplexityClass::Deterministic,
        );
        visual.requires_vision = true;
        let resolution = resolve(&visual, &[]).unwrap();
        let mut candidate = model("Novel Workhorse", "full-size workhorse model");
        classify(&mut candidate);
        candidate.capabilities.vision = None;
        assert!(rank_candidates(&[candidate.clone()], &resolution).is_empty());
        candidate.capabilities.vision = Some(true);
        candidate.capabilities.reasoning_efforts = vec!["medium".into()];
        assert!(rank_candidates(&[candidate.clone()], &resolution).is_empty());
        candidate
            .capabilities
            .reasoning_efforts
            .push("xhigh".into());
        assert_eq!(
            effort_for(&candidate, &resolution).as_deref(),
            Some("xhigh")
        );
        assert_eq!(rank_candidates(&[candidate], &resolution).len(), 1);
    }

    #[test]
    fn completion_headroom_respects_known_limit_and_schema_rejects_model_id() {
        let mut candidate = model("Novel Workhorse", "full-size workhorse model");
        let resolution = resolve(
            &request(
                TaskIntent::Architecture,
                ComplexityClass::CreativeArchitectural,
            ),
            &[],
        )
        .unwrap();
        assert_eq!(completion_tokens_for(&candidate, &resolution), Some(64_000));
        candidate.capabilities.max_completion_tokens = Some(24_000);
        assert_eq!(completion_tokens_for(&candidate, &resolution), Some(24_000));
        candidate.capabilities.context_window = Some(16_000);
        assert_eq!(completion_tokens_for(&candidate, &resolution), Some(16_000));
        let mut json = serde_json::to_value(request(
            TaskIntent::FormatConversion,
            ComplexityClass::Deterministic,
        ))
        .unwrap();
        json["model_id"] = Value::String("provider-fixed-name".into());
        assert!(serde_json::from_value::<SpawnRequest>(json).is_err());
    }

    #[test]
    fn explicit_effort_is_exact_supported_and_cannot_lower_the_floor() {
        let mut task = request(
            TaskIntent::Creativity,
            ComplexityClass::CreativeArchitectural,
        );
        let mut candidate = model("Novel Frontier", "frontier model");
        classify(&mut candidate);
        task.model_selection = Some(ModelSelection {
            target: candidate.target.clone(),
            id: candidate.id.clone(),
            effort: "high".into(),
        });
        let mut resolution = resolve(&task, &[]).unwrap();
        assert_eq!(effort_for(&candidate, &resolution).as_deref(), Some("high"));
        assert_eq!(rank_candidates(&[candidate.clone()], &resolution).len(), 1);
        for invalid in ["low", "unknown", "ultra"] {
            resolution.requested_effort = Some(invalid.into());
            assert!(rank_candidates(&[candidate.clone()], &resolution).is_empty());
        }
        resolution.requested_effort = Some("high".into());
        candidate.capabilities.reasoning_efforts.clear();
        assert!(rank_candidates(&[candidate], &resolution).is_empty());
    }

    #[test]
    fn marketing_names_and_family_metadata_cannot_change_eligibility() {
        let mut reference = model("Neutral", "unclassified");
        classify(&mut reference);
        for name in [
            "Flash-Lite Frontier",
            "Mini",
            "Haiku",
            "Opus",
            "Pro",
            "Unknown",
        ] {
            let mut candidate = model(name, "fast affordable frontier full-size workhorse");
            candidate.line = Some(name.to_lowercase());
            classify(&mut candidate);
            assert_eq!(candidate.tier, reference.tier);
            assert_eq!(candidate.available, reference.available);
            classify(&mut candidate);
            assert_eq!(
                candidate
                    .classification_evidence
                    .iter()
                    .filter(|e| e.starts_with("policy: "))
                    .count(),
                1
            );
        }
    }

    #[test]
    fn full_size_without_extended_reasoning_stays_t2() {
        let mut full = model("Novel Pro", "full-size synthesis model");
        full.capabilities.reasoning_efforts = vec!["low".into(), "medium".into()];
        classify(&mut full);
        assert_eq!(full.tier, Tier::T2Synthesis);
        let task = resolve(
            &request(TaskIntent::Synthesis, ComplexityClass::Deterministic),
            &[],
        )
        .unwrap();
        assert!(rank_candidates(&[full], &task).is_empty()); // high effort required when supported
    }

    #[test]
    fn assigned_tools_require_verified_support_without_a_tool_count_penalty() {
        let mut req = request(TaskIntent::FormatConversion, ComplexityClass::Deterministic);
        req.objective = "Convert CSV to JSON".into();
        req.deterministic_operation = Some(DeterministicOperation::Convert {
            from: "CSV".into(),
            to: "JSON".into(),
        });
        req.assigned_tools.push("reader".into());
        let policies = [
            ToolPolicy {
                name: "reader".into(),
                min_tier: Tier::T1Deterministic,
                requires_vision: false,
            },
            ToolPolicy {
                name: "writer".into(),
                min_tier: Tier::T1Deterministic,
                requires_vision: false,
            },
        ];
        let one = resolve(&req, &policies).unwrap();
        assert_eq!(one.min_tier, Tier::T1Deterministic);
        let mut text_only = model("Novel Flash", "fast utility");
        text_only.capabilities.tools = Some(false);
        classify(&mut text_only);
        assert!(rank_candidates(&[text_only], &one).is_empty());
        req.assigned_tools.push("writer".into());
        assert_eq!(
            resolve(&req, &policies).unwrap().min_tier,
            Tier::T1Deterministic
        );
    }
}
