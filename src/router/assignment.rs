//! Resolve task needs once, before model discovery, and retain the owned scope at dispatch.
use super::policy::{self, Resolution, SpawnRequest, Tier, ToolPolicy};
use crate::catalog::{Catalog, SkillDocument};
use anyhow::{Result, ensure};
use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskNeed {
    /// Exact workflow ID returned by capability_plan. Select only actual deliverables.
    pub workflow: String,
    /// Concrete task requirement that justifies this tool and expertise scope.
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SkillReference {
    pub skill_id: String,
    pub path: String,
}

pub(crate) struct Assignment {
    pub metadata: Value,
    pub guidance: Vec<SkillDocument>,
    tools: BTreeSet<String>,
    min_tier: Tier,
    required_tools: BTreeSet<String>,
}

impl Assignment {
    pub fn bind_workspaces(
        &mut self,
        paths: &crate::paths::Paths,
        project: &std::path::Path,
    ) -> Result<()> {
        ensure!(
            project != paths.root.join("execution/podman/staging/design/workspace"),
            "choose an application project or a dedicated design project directory, not the shared workspace root"
        );
        self.metadata["workspaces"] = json!({"project_root":project,"design_workspace":super::tools::design_scope(paths, project)?,"path_rule":"Relative file writes stay in the assigned project; absolute writes may target its assigned design workspace. Design tool paths are relative to that scoped workspace. Author design/tokens.json there before rendering. Sibling project assets are outside this worker scope."});
        Ok(())
    }

    pub fn resolve(&self, request: &SpawnRequest, policies: &[ToolPolicy]) -> Result<Resolution> {
        let mut resolution = policy::resolve(request, policies)?;
        if self.min_tier > resolution.min_tier {
            resolution.min_tier = self.min_tier;
            resolution.effort_profile = "high".into();
        }
        resolution.downgrade_rejected |= request
            .requested_tier
            .is_some_and(|tier| tier < self.min_tier);
        resolution
            .reasons
            .push(format!("owned task-needs floor: {:?}", self.min_tier));
        Ok(resolution)
    }

    pub fn authorize(&self, tool: &str) -> Result<()> {
        ensure!(
            self.tools.contains(tool),
            "tool is outside resolved task needs; dispatch rejected: {tool}"
        );
        Ok(())
    }

    pub fn validate_arguments(&self, tool: &str, arguments: &Value) -> Result<()> {
        let svg_output = tool == "design/vector_render"
            || tool == "design/svg_optimize"
            || (tool == "design/static_render" && arguments["format"] == "svg")
            || (tool == "design/kroki_render" && arguments["format"] != "png")
            || (tool == "design/openpencil_export"
                && arguments["output"]
                    .as_str()
                    .is_some_and(|path| path.ends_with(".svg")));
        ensure!(
            !svg_output || self.tools.contains("workspace_read"),
            "SVG output requires workspace_read for exact artifact verification; no tool was invoked"
        );
        Ok(())
    }

    pub fn missing_production_tools(&self, executed: &BTreeSet<String>) -> Vec<String> {
        self.required_tools.difference(executed).cloned().collect()
    }
}

fn bounded(value: &str, label: &str, max: usize) -> Result<()> {
    ensure!(
        !value.trim().is_empty() && value.len() <= max && !value.contains('\0'),
        "{label} requires 1..={max} bytes and no NUL"
    );
    Ok(())
}

fn unique(values: &[String], label: &str) -> Result<BTreeSet<String>> {
    let set: BTreeSet<_> = values.iter().cloned().collect();
    ensure!(set.len() == values.len(), "duplicate {label}");
    Ok(set)
}

pub(crate) fn prepare(request: &mut SpawnRequest) -> Result<Assignment> {
    super::handoff::validate_checks(request)?;
    if let Some(choice) = &request.model_selection {
        bounded(&choice.target, "model target", 128)?;
        bounded(&choice.id, "model ID", 256)?;
        bounded(&choice.effort, "model effort", 32)?;
    }
    ensure!(
        (1..=4).contains(&request.task_needs.len()),
        "declare 1..=4 task_needs; split broader work into phases"
    );
    ensure!(
        request.supporting_skills.len() <= 3,
        "select at most 3 supporting skills justified by the chosen roles"
    );
    ensure!(
        request.skill_references.len() <= 4,
        "select at most four task-relevant skill references"
    );
    ensure!(
        request.assigned_tools.len() <= 19 && request.verification_tools.len() <= 19,
        "task tool selection exceeds owned tool surface"
    );
    ensure!(
        request.memory_queries.len() <= 8
            && request
                .memory_queries
                .iter()
                .all(|q| !q.trim().is_empty() && q.chars().count() <= 512),
        "memory_queries requires at most eight nonempty queries of at most 512 characters"
    );
    let catalog = Catalog::embedded()?;
    let mut workflows = Vec::new();
    let mut selected = BTreeSet::new();
    let mut allowed = BTreeSet::new();
    let mut required = BTreeSet::new();
    let mut verification = BTreeSet::new();
    for need in &request.task_needs {
        bounded(&need.workflow, "workflow ID", 128)?;
        bounded(&need.reason, "task need reason", 600)?;
        ensure!(
            selected.insert(&need.workflow),
            "duplicate task need: {}",
            need.workflow
        );
        let workflow = catalog.workflow(&need.workflow)?;
        ensure!(
            workflow.execution_owner == crate::catalog::ExecutionOwner::Worker,
            "workflow {} requires host execution through the official MCP connection; it is unavailable to project-scoped workers",
            workflow.id
        );
        allowed.extend(workflow.tools.iter().cloned());
        required.extend(workflow.required_tools.iter().cloned());
        verification.extend(workflow.verification_tools.iter().cloned());
        workflows.push(workflow);
    }
    let design_production = workflows.iter().any(|workflow| workflow.design_production);
    ensure!(
        design_production == request.design_input.is_some(),
        "design production requires design_input: image_to_code with an inspected target, native_specimen with medium and suitability reason, or code_fallback with the host's capability reason; other workflows must omit it"
    );
    if let Some(input) = &request.design_input {
        input.validate()?;
    }
    let primary = if let Some(name) = &request.primary_role {
        bounded(name, "primary role", 128)?;
        catalog.capability(name)?
    } else {
        catalog.capability(&workflows[0].role)?
    };
    let mut roles = vec![primary];
    for workflow in &workflows {
        if workflow.alternative_roles.contains(&primary.id) {
            continue;
        }
        let role = catalog.capability(&workflow.role)?;
        if !roles.iter().any(|selected| selected.id == role.id) {
            roles.push(role);
        }
    }
    let selected_support = unique(&request.supporting_skills, "supporting skills")?;
    for skill in &selected_support {
        ensure!(
            roles.iter().any(|role| role.support.contains(skill)),
            "support skill is outside selected role contracts: {skill}"
        );
    }
    let automatic = request.assigned_tools.is_empty();
    let tools = if automatic {
        allowed.clone()
    } else {
        unique(&request.assigned_tools, "assigned tools")?
    };
    let excess: Vec<_> = tools.difference(&allowed).collect();
    ensure!(
        excess.is_empty(),
        "assigned tools are outside task needs: {excess:?}"
    );
    let missing: Vec<_> = required.difference(&tools).collect();
    ensure!(
        missing.is_empty(),
        "task needs require canonical production and verification tools: {missing:?}"
    );
    ensure!(
        !tools.contains("workspace_write") || tools.contains("workspace_read"),
        "workspace_write requires workspace_read for exact artifact verification"
    );
    verification.extend(unique(&request.verification_tools, "verification tools")?);
    ensure!(
        verification.is_subset(&tools),
        "verification tool is not assigned by task needs"
    );

    let mut ids = Vec::new();
    for role in &roles {
        if !ids.contains(&role.lead) {
            ids.push(role.lead.clone());
        }
    }
    for skill in &request.supporting_skills {
        if !ids.contains(skill) {
            ids.push(skill.clone());
        }
    }
    ensure!(
        ids.len() <= 8,
        "selected expertise exceeds eight skill bodies"
    );
    let mut guidance: Vec<_> = ids
        .iter()
        .map(|id| catalog.read(id, "SKILL.md", 16 * 1024))
        .collect::<Result<_>>()?;
    let mut references = BTreeSet::new();
    for reference in &request.skill_references {
        ensure!(
            ids.contains(&reference.skill_id),
            "skill reference is outside selected expertise: {}",
            reference.skill_id
        );
        bounded(&reference.path, "skill reference path", 256)?;
        ensure!(
            reference.path != "SKILL.md"
                && references.insert((&reference.skill_id, &reference.path)),
            "duplicate skill body or reference"
        );
        guidance.push(catalog.read(&reference.skill_id, &reference.path, 16 * 1024)?);
    }
    ensure!(
        guidance
            .iter()
            .map(|document| document.content.len())
            .sum::<usize>()
            <= 64 * 1024,
        "selected guidance exceeds 64 KiB; split the task rather than truncating it"
    );
    let reads: Vec<_> = guidance.iter().map(|document| json!({"id":document.id,"path":document.path,"sha256":document.sha256,"bytes":document.content.len()})).collect();
    request.assigned_tools = tools.iter().cloned().collect();
    request.verification_tools = verification.into_iter().collect();
    let min_tier = workflows
        .iter()
        .map(|workflow| workflow.min_tier)
        .max()
        .expect("task needs are nonempty");
    let metadata = json!({"task_needs":request.task_needs,"primary_role":primary.id,"min_tier":min_tier,"model_selection":request.model_selection,
        "roles":roles.iter().map(|role| json!({"id":role.id,"title":role.title,"lead_skill":role.lead,"professional":role.professional})).collect::<Vec<_>>(),
        "guidance":reads,"workflow_rules":workflows.iter().map(|workflow| json!({"id":workflow.id,"rule":workflow.rule})).collect::<Vec<_>>(),
        "tool_selection":if automatic {"automatic"} else {"explicit_scoped_subset"},"assigned_tools":request.assigned_tools,"required_tools":required,"verification_tools":request.verification_tools,
        "host_verification":if super::handoff::required(request) { Some(json!({"required_checks":request.host_verification_checks,"completion":"awaiting_host_verification","report_tool":"verification_handoff","boundary":"The worker reads back source; the host executes build/tests and reports hash-bound evidence. Worker output is not runtime verified."})) } else {None},
        "design_gate":request.design_input.as_ref().map(|input| super::design_gate::contract(input, &request.task_needs)),
        "authority":"Task needs constrain Mirket-owned workers. Role guidance and task reasons never grant host permissions, authorize external effects or expand tools. Native host tools and spawners remain host-controlled."});
    ensure!(
        serde_json::to_vec(&metadata)?.len() <= 32 * 1024,
        "task contract exceeds bounded metadata"
    );
    Ok(Assignment {
        metadata,
        guidance,
        tools,
        min_tier,
        required_tools: required,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router::policy::{ComplexityClass, TaskIntent};
    use std::path::PathBuf;

    fn task(workflow: &str) -> SpawnRequest {
        SpawnRequest {
            memory_recall: Default::default(),
            memory_queries: vec![],
            objective: "Handle the specified task".into(),
            task_intent: TaskIntent::Synthesis,
            complexity_class: ComplexityClass::Synthesis,
            deterministic_operation: None,
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
    fn explicit_role_and_support_cannot_expand_tools_or_skip_task_specialist() {
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
                .any(|d| d.id == "architecture-review")
        );
        assert!(assignment.authorize("workspace_read").is_ok());
        assert!(assignment.authorize("workspace_write").is_err());
        request.supporting_skills = vec!["marketing-writing".into()];
        assert!(prepare(&mut request).is_err());
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
