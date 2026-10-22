#[cfg(test)]
mod tests {
    use super::*;
    use crate::router::policy::{ComplexityClass, TaskIntent};
    use std::path::PathBuf;

    fn task(workflow: &str) -> SpawnRequest {
        SpawnRequest {
            limits: Default::default(),
            persistence: Default::default(),
            allow_already_satisfied: false,
            host_review: false,
            memory_recall: Default::default(),
            memory_queries: vec![],
            objective: "Handle the specified task".into(),
            task_intent: TaskIntent::Synthesis,
            complexity_class: ComplexityClass::Synthesis,
            deterministic_operation: (workflow == "structured-response")
                .then_some(crate::router::policy::DeterministicOperation::SortLines),
            deterministic_input: None,
            task_needs: vec![TaskNeed {
                workflow: workflow.into(),
                reason: "The requested deliverable needs this workflow.".into(),
            }],
            primary_role: None,
            supporting_skills: vec![],
            skill_references: vec![],
            assigned_tools: vec![],
            project: PathBuf::from("/project"),
            requires_vision: false,
            requested_tier: None,
            model_selection: None,
            design_input: Catalog::embedded()
                .unwrap()
                .workflow(workflow)
                .ok()
                .filter(|w| w.design_production)
                .map(|_| super::super::DesignInput::CodeFallback {
                    reason: "Test host has no image model".into(),
                }),
            verification_tools: vec![],
            host_verification_checks: super::super::handoff::default_checks(),
        }
    }

    #[test]
    fn editable_documents_are_scoped_worker_tools_with_export_and_pixel_verification() {
        let mut request = task("editable-interface");
        let assignment = prepare(&mut request).unwrap();
        for tool in [
            "design/openpencil_document",
            "design/openpencil_edit",
            "design/openpencil_export",
            "design/openpencil_inspect",
            "inspect_image",
        ] {
            assignment.authorize(tool).unwrap();
        }
        assert!(assignment.authorize("design/execute_code").is_err());
        let missing = assignment.missing_production_tools(&BTreeSet::new());
        assert!(missing.contains(&"design/openpencil_export".into()));
        assert!(missing.contains(&"inspect_image".into()));
    }

    #[test]
    fn design_contract_is_required_before_discovery_and_cannot_claim_an_empty_fallback() {
        let mut request = task("web-interface");
        request.design_input = None;
        assert!(
            prepare(&mut request)
                .err()
                .unwrap()
                .to_string()
                .contains("design_input")
        );
        request.design_input = Some(super::super::DesignInput::CodeFallback { reason: " ".into() });
        assert!(prepare(&mut request).is_err());
        request.design_input = Some(super::super::DesignInput::ImageToCode {
            image: "target.png".into(),
        });
        let assignment = prepare(&mut request).unwrap();
        assert_eq!(
            assignment.metadata["design_gate"]["input"]["mode"],
            "image_to_code"
        );
        assert_eq!(
            assignment.metadata["design_gate"]["execution_seconds"],
            1800
        );
        request.task_needs[0].workflow = "source-implementation".into();
        assert!(prepare(&mut request).is_err());
    }

    #[test]
    fn each_workflow_selects_only_its_tools_and_delivers_embedded_expertise() {
        let catalog = Catalog::embedded().unwrap();
        for workflow in catalog.workflows() {
            let mut request = task(&workflow.id);
            if workflow.execution_owner == crate::catalog::ExecutionOwner::Host {
                assert!(
                    prepare(&mut request)
                        .err()
                        .unwrap()
                        .to_string()
                        .contains("requires host execution")
                );
                continue;
            }
            let assignment = prepare(&mut request).unwrap();
            assert_eq!(
                request
                    .assigned_tools
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>(),
                workflow.tools.iter().cloned().collect()
            );
            assert_eq!(assignment.metadata["primary_role"], workflow.role);
            assert!(assignment.guidance.iter().any(|document|document.id==catalog.capability(&workflow.role).unwrap().lead));
            assert!(
                assignment
                    .guidance
                    .iter()
                    .all(|document| !document.content.is_empty() && document.sha256.len() == 64)
            );
        }
    }

    #[test]
    fn wrong_pipeline_review_mutation_and_missing_quality_tools_are_rejected() {
        for (workflow, tool) in [
            ("paged-media", "design/browser_screenshot"),
            ("diagram", "design/static_render"),
            ("static-graphic", "design/browser_screenshot"),
            ("web-interface", "design/static_render"),
            ("source-review", "workspace_write"),
            ("analysis", "design/execute_code"),
        ] {
            let mut request = task(workflow);
            request.assigned_tools = vec![tool.into()];
            assert!(prepare(&mut request).is_err());
        }
        let mut request = task("web-interface");
        request.assigned_tools = vec![
            "workspace_write".into(),
            "workspace_read".into(),
            "design/browser_screenshot".into(),
        ];
        assert!(prepare(&mut request).is_err());
    }

    #[test]
    fn explicit_guidance_replaces_defaults_without_expanding_tools() {
        let mut request = task("source-review");
        request.primary_role = Some("CFO".into());
        let assignment = prepare(&mut request).unwrap();
        assert_eq!(assignment.metadata["primary_role"], "financial-analysis");
        assert!(
            assignment
                .guidance
                .iter()
                .any(|d| d.id == "financial-analysis")
        );
        assert!(
            assignment
                .guidance
                .iter()
                .all(|d| d.id != "architecture-review")
        );
        assert!(assignment.authorize("workspace_read").is_ok());
        assert!(assignment.authorize("workspace_write").is_err());
        request.supporting_skills = vec!["marketing-writing".into()];
        let assignment = prepare(&mut request).unwrap();
        assert!(
            assignment
                .guidance
                .iter()
                .any(|d| d.id == "marketing-writing")
        );
        assert!(assignment.authorize("workspace_write").is_err());
    }

    #[test]
    fn suitable_explicit_specialist_replaces_default_without_extra_guidance() {
        let mut request = task("source-implementation");
        request.primary_role = Some("Frontend Engineer".into());
        let assignment = prepare(&mut request).unwrap();
        assert_eq!(assignment.metadata["primary_role"], "frontend-engineering");
        assert_eq!(
            assignment
                .guidance
                .iter()
                .map(|doc| doc.id.as_str())
                .collect::<Vec<_>>(),
            ["frontend-engineering"]
        );
        assert!(assignment.authorize("design/execute_code").is_err());
    }

    #[test]
    fn explicit_cross_domain_roles_deliver_guidance_without_changing_workflow_scope() {
        let catalog = Catalog::embedded().unwrap();
        for workflow in catalog
            .workflows()
            .iter()
            .filter(|w| w.execution_owner == crate::catalog::ExecutionOwner::Worker)
        {
            for alternative in ["financial-analysis", "frontend-engineering", "project-review"] {
                let mut request = task(&workflow.id);
                request.primary_role = Some(alternative.into());
                let assignment = prepare(&mut request).unwrap();
                let specialist = catalog.capability(alternative).unwrap();
                assert_eq!(assignment.metadata["primary_role"], specialist.id);
                assert!(assignment.guidance.iter().any(|d| d.id == specialist.lead));
                assert_eq!(
                    request.assigned_tools.iter().collect::<BTreeSet<_>>(),
                    workflow.tools.iter().collect()
                );
                assert_eq!(
                    assignment.required_tools,
                    workflow.required_tools.iter().cloned().collect()
                );
                for verification in &workflow.verification_tools {
                    assert!(request.verification_tools.contains(verification));
                }
            }
        }
    }

    #[test]
    fn output_arguments_must_have_a_usable_verification_route_before_effects() {
        let mut request = task("static-graphic");
        request.assigned_tools = vec!["design/static_render".into(), "inspect_image".into()];
        let assignment = prepare(&mut request).unwrap();
        assert!(
            assignment
                .validate_arguments("design/static_render", &json!({"format":"png"}))
                .is_ok()
        );
        assert!(
            assignment
                .validate_arguments("design/static_render", &json!({"format":"svg"}))
                .is_err()
        );
        request.assigned_tools.push("workspace_write".into());
        assert!(prepare(&mut request).is_err());
    }

    #[test]
    fn composed_needs_union_only_selected_workflows_and_reject_malformed_scope() {
        let mut request = task("web-interface");
        request.task_needs.push(TaskNeed {
            workflow: "vector-art".into(),
            reason: "The UI needs a bespoke illustration.".into(),
        });
        let assignment = prepare(&mut request).unwrap();
        assert!(assignment.authorize("design/vector_render").is_ok());
        assert!(assignment.authorize("design/typst_compile").is_err());
        assert!(
            assignment
                .guidance
                .iter()
                .any(|d| d.id == "interface-design")
        );
        assert!(assignment.guidance.iter().any(|d| d.id == "graphic-design"));
        request.task_needs.push(request.task_needs[0].clone());
        assert!(prepare(&mut request).is_err());
        let mut request = task("unknown");
        assert!(prepare(&mut request).is_err());
        request.task_needs.clear();
        assert!(prepare(&mut request).is_err());
        let mut request = task("analysis");
        request.task_needs[0].reason = " ".into();
        assert!(prepare(&mut request).is_err());
    }

    #[test]
    fn workflow_floor_survives_mechanical_labels_and_explicit_tool_subset() {
        let mut request = task("source-review");
        request.objective = "Convert CSV to JSON".into();
        request.task_intent = TaskIntent::FormatConversion;
        request.complexity_class = ComplexityClass::Deterministic;
        request.deterministic_operation =
            Some(crate::router::policy::DeterministicOperation::Convert {
                from: "CSV".into(),
                to: "JSON".into(),
            });
        request.requested_tier = Some(Tier::T1Deterministic);
        request.assigned_tools = vec!["workspace_read".into()];
        let assignment = prepare(&mut request).unwrap();
        let resolved = assignment
            .resolve(
                &request,
                &[ToolPolicy {
                    name: "workspace_read".into(),
                    min_tier: Tier::T1Deterministic,
                    requires_vision: false,
                }],
            )
            .unwrap();
        assert_eq!(resolved.min_tier, Tier::T3Architect);
        assert_eq!(resolved.effort_profile, "high");
        assert!(resolved.downgrade_rejected);
    }

    #[test]
    fn export_only_and_mechanical_workflows_resolve_without_redundant_request_labels() {
        let mut response = task("structured-response");
        response.deterministic_operation = None;
        assert!(prepare(&mut response).is_err());
        response.deterministic_operation =
            Some(crate::router::policy::DeterministicOperation::SortLines);
        response.objective = "Satırları sırala".into();
        assert!(prepare(&mut response).is_ok());
        let mut export: SpawnRequest = serde_json::from_value(json!({
            "objective":"Export the saved design unchanged", "project":"/project",
            "task_needs":[{"workflow":"editable-export","reason":"Export an existing file"}], "assigned_tools":[]
        })).unwrap();
        let assignment = prepare(&mut export).unwrap();
        assert!(!assignment.execution_policy.host_verification);
        assert!(export.design_input.is_none());
        assert!(
            !export
                .assigned_tools
                .contains(&"design/openpencil_edit".into())
        );
        assert!(!export.assigned_tools.contains(&"workspace_write".into()));
        let completed = export.assigned_tools.iter().cloned().collect();
        assert!(assignment.missing_production_tools(&completed).is_empty());
        let mut edit = task("editable-interface");
        let assignment = prepare(&mut edit).unwrap();
        assert!(
            assignment
                .missing_production_tools(&completed)
                .contains(&"design/openpencil_edit".into())
        );
        let mut tokens = task("design-tokens");
        tokens.objective = "Compile responsive architecture tokens".into();
        let assignment = prepare(&mut tokens).unwrap();
        let policies = tokens
            .assigned_tools
            .iter()
            .map(|name| ToolPolicy {
                name: name.clone(),
                min_tier: Tier::T1Deterministic,
                requires_vision: false,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            assignment.resolve(&tokens, &policies).unwrap().min_tier,
            Tier::T1Deterministic
        );
    }

    #[test]
    fn expressive_scene_references_fit_the_actual_worker_delivery_limit() {
        let mut request = task("interactive-motion");
        request.supporting_skills = vec!["advanced-motion".into()];
        request.skill_references = [
            "references/art-direction-and-medium.md",
            "references/connected-layers.md",
            "references/scene-mechanics.md",
            "references/gsap-integration.md",
        ]
        .into_iter()
        .map(|path| SkillReference {
            skill_id: "advanced-motion".into(),
            path: path.into(),
        })
        .collect();
        let assignment = prepare(&mut request).unwrap();
        for reference in &request.skill_references {
            assert!(
                assignment
                    .guidance
                    .iter()
                    .any(|document| document.path == reference.path)
            );
        }
    }

    #[test]
    fn references_are_bounded_to_selected_embedded_expertise() {
        let mut request = task("source-implementation");
        request.skill_references = vec![SkillReference {
            skill_id: "backend-engineering".into(),
            path: "references/command-service.md".into(),
        }];
        let assignment = prepare(&mut request).unwrap();
        assert!(
            assignment
                .guidance
                .iter()
                .any(|d| d.path == "references/command-service.md")
        );
        request.skill_references[0].skill_id = "marketing-writing".into();
        assert!(prepare(&mut request).is_err());
        request.skill_references[0] = SkillReference {
            skill_id: "backend-engineering".into(),
            path: "../../Cargo.toml".into(),
        };
        assert!(prepare(&mut request).is_err());
    }
}
