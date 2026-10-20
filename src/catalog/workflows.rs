//! Task deliverables bind expertise and owned tool choices, not professional authority.
use super::{Capability, Catalog, validate_id};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionOwner {
    #[default]
    Worker,
    Host,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Workflow {
    pub id: String,
    #[serde(default)]
    pub execution_owner: ExecutionOwner,
    pub when: String,
    pub role: String,
    /// Suitable explicit perspectives can replace this workflow's default expertise.
    pub alternative_roles: Vec<String>,
    /// Requires the bounded medium-aware visual contract at the owned worker boundary.
    #[serde(default)]
    pub design_production: bool,
    pub min_tier: crate::router::policy::Tier,
    pub tools: Vec<String>,
    pub required_tools: Vec<String>,
    pub verification_tools: Vec<String>,
    pub rule: String,
}

pub(super) fn validate(workflows: &[Workflow], roles: &[Capability]) -> Result<()> {
    ensure!(
        (1..=32).contains(&workflows.len()),
        "expected 1..=32 task workflows"
    );
    let known = crate::router::tools::known_names()?;
    let mut ids = BTreeSet::new();
    let mut covered = BTreeSet::new();
    for workflow in workflows {
        validate_id(&workflow.id)?;
        ensure!(
            ids.insert(&workflow.id),
            "duplicate task workflow: {}",
            workflow.id
        );
        ensure!(
            roles.iter().any(|role| role.id == workflow.role),
            "unknown workflow role: {}",
            workflow.role
        );
        let alternatives: BTreeSet<_> = workflow.alternative_roles.iter().collect();
        ensure!(
            alternatives.len() == workflow.alternative_roles.len(),
            "duplicate workflow alternative roles"
        );
        for role in alternatives {
            ensure!(
                roles.iter().any(|candidate| candidate.id == *role),
                "unknown workflow alternative role: {role}"
            );
        }
        for text in [&workflow.when, &workflow.rule] {
            ensure!(
                !text.trim().is_empty() && text.len() <= 600 && !text.contains('\0'),
                "workflow guidance must contain 1..=600 bytes and no NUL"
            );
        }
        let tools: BTreeSet<_> = workflow.tools.iter().collect();
        ensure!(
            tools.len() == workflow.tools.len() && tools.len() <= 19,
            "duplicate or excessive workflow tools"
        );
        for tool in &workflow.tools {
            ensure!(known.contains(tool), "unknown workflow tool: {tool}");
            covered.insert(tool.clone());
        }
        let required: BTreeSet<_> = workflow.required_tools.iter().collect();
        let verification: BTreeSet<_> = workflow.verification_tools.iter().collect();
        ensure!(
            required.len() == workflow.required_tools.len()
                && verification.len() == workflow.verification_tools.len(),
            "duplicate workflow requirements"
        );
        ensure!(
            required.is_subset(&tools) && verification.is_subset(&required),
            "workflow verification and required tools must be assigned: {}",
            workflow.id
        );
    }
    ensure!(
        covered == known,
        "every owned tool needs a task workflow before it can be routed"
    );
    Ok(())
}

impl Catalog {
    pub fn workflows(&self) -> &[Workflow] {
        &self.0.workflows
    }

    pub fn workflow(&self, id: &str) -> Result<&Workflow> {
        self.0.workflows.iter().find(|workflow| workflow.id == id)
            .with_context(|| format!("unknown task workflow: {id}; inspect capability_plan or mirket capabilities plan"))
    }

    pub(super) fn workflow_choices(&self, role: Option<&str>) -> Vec<Value> {
        self.workflows()
            .iter()
            .filter(|workflow| role.is_none_or(|role|workflow.role == role || workflow.alternative_roles.iter().any(|candidate|candidate == role)))
            .map(|workflow| serde_json::json!({"id":workflow.id,"when":workflow.when,"role":workflow.role,"min_tier":workflow.min_tier,"execution_owner":workflow.execution_owner,"worker_available":workflow.execution_owner == ExecutionOwner::Worker,"design_input_required":workflow.design_production,"explicit_role_can_replace_default":role.map(|role|workflow.alternative_roles.iter().any(|candidate|candidate==role))}))
            .collect()
    }
}
