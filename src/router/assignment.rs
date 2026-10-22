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
    pub execution_policy: super::design_gate::ExecutionPolicy,
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
        self.metadata["workspaces"] = json!({"project_root":project,"design_workspace":super::tools::design_scope(paths, project)?,"path_rule":"Relative file writes stay in the assigned project; absolute writes may target its assigned design workspace. Design tool paths are relative to that scoped workspace. For production, compile canonical design tokens and reference their outputs in authored source. Capture and export preserve the authored design. Sibling project assets are outside this worker scope."});
        Ok(())
    }

    pub fn resolve(&self, request: &SpawnRequest, policies: &[ToolPolicy]) -> Result<Resolution> {
        Ok(policy::resolve(request, policies, self.min_tier)?)
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
    request.limits.validate()?;
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
            "workflow {} requires host execution with explicitly selected tooling and is unavailable to project-scoped workers",
            workflow.id
        );
        allowed.extend(workflow.tools.iter().cloned());
        required.extend(workflow.required_tools.iter().cloned());
        verification.extend(workflow.verification_tools.iter().cloned());
        workflows.push(workflow);
    }
    if workflows.iter().any(|workflow| workflow.requires_operation) {
        let operation = request
            .deterministic_operation
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("selected workflow requires deterministic_operation"))?;
        operation.validate()?;
    }
    ensure!(
        !request.allow_already_satisfied
            || workflows.iter().all(|w| w.id == "source-implementation"),
        "already-satisfied outcomes are available only for an explicitly requested source-implementation assessment"
    );
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
        if request.primary_role.is_some() {
            continue;
        }
        let role = catalog.capability(&workflow.role)?;
        if !roles.iter().any(|selected| selected.id == role.id) {
            roles.push(role);
        }
    }
    unique(&request.supporting_skills, "supporting skills")?;
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
        ids.len() <= 32,
        "selected expertise exceeds 32 skill bodies; guidance remains bounded to 64 KiB"
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
    let mut execution_policy = super::design_gate::ExecutionPolicy::from_workflows(&workflows);
    execution_policy.host_verification |= request.host_review;
    let metadata = json!({"task_needs":request.task_needs,"execution_policy":execution_policy,"primary_role":primary.id,"min_tier":min_tier,"model_selection":request.model_selection,
        "roles":roles.iter().map(|role| json!({"id":role.id,"title":role.title,"lead_skill":role.lead,"professional":role.professional})).collect::<Vec<_>>(),
        "guidance":reads,"workflow_rules":workflows.iter().map(|workflow| json!({"id":workflow.id,"rule":workflow.rule})).collect::<Vec<_>>(),
        "tool_selection":if automatic {"automatic"} else {"explicit_scoped_subset"},"assigned_tools":request.assigned_tools,"required_tools":required,"verification_tools":request.verification_tools,
        "host_verification":if execution_policy.host_verification { Some(json!({"required_checks":request.host_verification_checks,"completion":"awaiting_host_verification","report_tool":"verification_handoff","boundary":"The worker reads back source; the host executes build/tests and reports hash-bound evidence. Worker output is not runtime verified."})) } else {None},
        "design_gate":request.design_input.as_ref().map(|input| super::design_gate::contract(input, &execution_policy)),
        "authority":"Task needs constrain Mirket-owned workers. Role guidance and task reasons never grant host permissions, authorize external effects or expand tools. Native host tools and spawners remain host-controlled."});
    ensure!(
        serde_json::to_vec(&metadata)?.len() <= 32 * 1024,
        "task contract exceeds bounded metadata"
    );
    Ok(Assignment {
        metadata,
        execution_policy,
        guidance,
        tools,
        min_tier,
        required_tools: required,
    })
}

#[cfg(test)]
include!("tests/assignment.rs");
