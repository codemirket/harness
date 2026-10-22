//! Workflow capability envelopes and explicit model selection.
//!
//! Classification uses current discovery metadata, never a published model ID.
//! Its tier describes supported controls, not measured quality or benchmark claims.
//! Adapters must set capability flags from a verified provider/host contract;
//! unknown flags quarantine a model rather than assuming support.

use rmcp::schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize,
)]
pub enum Tier {
    #[serde(rename = "T1_DETERMINISTIC")]
    T1Deterministic,
    #[serde(rename = "T2_SYNTHESIS")]
    T2Synthesis,
    #[serde(rename = "T3_ARCHITECT")]
    T3Architect,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Capabilities {
    pub tools: Option<bool>,
    pub structured_outputs: Option<bool>,
    pub vision: Option<bool>,
    #[serde(default)]
    pub reasoning_efforts: Vec<String>,
    #[serde(default)]
    pub thinking_modes: Vec<String>,
    pub context_window: Option<u64>,
    pub max_completion_tokens: Option<u64>,
    pub thinking_budget_max: Option<u64>,
    pub pricing: Option<Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Model {
    pub id: String,
    pub target: String,
    pub display_name: String,
    pub description: String,
    #[serde(default)]
    pub line: Option<String>,
    pub capabilities: Capabilities,
    pub tier: Tier,
    #[serde(default)]
    pub classification_evidence: Vec<String>,
    pub available: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ToolPolicy {
    pub name: String,
    pub min_tier: Tier,
    pub requires_vision: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskIntent {
    FormatConversion,
    CliParameters,
    SchemaExtraction,
    FileHygiene,
    Synthesis,
    MultiToolChaining,
    Creativity,
    Layout,
    Motion,
    Architecture,
    Debugging,
    MultiStepVisualQa,
    MultiConstraint,
    Ambiguity,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplexityClass {
    Deterministic,
    Synthesis,
    CreativeArchitectural,
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum DeterministicOperation {
    Convert {
        from: String,
        to: String,
    },
    FormatCliArguments,
    ExtractSchema {
        from: String,
    },
    TrimWhitespace,
    #[schemars(skip)]
    RenameFile,
    DeduplicateLines,
    SortLines,
}

impl DeterministicOperation {
    /// Only these transformations have deterministic native semantics. Others use
    /// schema-bound model inference and must not be described as deterministic.
    pub fn execute(&self, input: &str) -> Option<String> {
        match self {
            Self::TrimWhitespace => Some(input.trim().to_owned()),
            Self::SortLines => {
                let mut lines: Vec<_> = input.lines().collect();
                lines.sort();
                Some(lines.join("\n"))
            }
            Self::DeduplicateLines => {
                let mut seen = HashSet::new();
                Some(
                    input
                        .lines()
                        .filter(|line| seen.insert(*line))
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
            }
            _ => None,
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        let format = |name: &str| {
            [
                "json", "yaml", "yml", "csv", "tsv", "xml", "markdown", "md", "txt", "text",
                "html", "pdf",
            ]
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(name))
        };
        let valid = match self {
            Self::Convert { from, to } => format(from) && format(to),
            Self::ExtractSchema { from } => format(from),
            Self::RenameFile => false,
            _ => true,
        };
        anyhow::ensure!(valid, "unsupported deterministic operation format");
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MemoryRecall {
    Required,
    #[default]
    BestEffort,
    Off,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Persistence {
    #[default]
    Auto,
    Durable,
    Ephemeral,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct RunLimits {
    pub max_calls: usize,
    pub max_completion_tokens: u64,
    /// Conservative reservation: UTF-8 input bytes + image allowance + requested output tokens.
    /// This is not a provider billing estimate. Unknown usage is never treated as zero.
    pub token_reservation: u64,
    pub context_bytes: usize,
}
impl Default for RunLimits {
    fn default() -> Self {
        Self {
            max_calls: 256,
            max_completion_tokens: 8192,
            token_reservation: 8_000_000,
            context_bytes: 256 * 1024,
        }
    }
}
impl RunLimits {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            (1..=256).contains(&self.max_calls)
                && (4096..=64_000).contains(&self.max_completion_tokens)
                && (8192..=64_000_000).contains(&self.token_reservation)
                && (32 * 1024..=1024 * 1024).contains(&self.context_bytes),
            "run limits exceed supported bounds"
        );
        Ok(())
    }
}

/// Task needs constrain owned role, guidance and tool selection before execution.
/// An explicit model choice narrows discovered candidates; it cannot assert
/// capabilities, lower floors, add providers or authorize inference.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelSelection {
    pub target: String,
    pub id: String,
    pub effort: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnRequest {
    #[serde(default)]
    pub limits: RunLimits,
    #[serde(default)]
    pub persistence: Persistence,
    /// Explicitly permit a source-implementation assessment that requires no mutation.
    /// Exact source readback and host verification remain mandatory.
    #[serde(default)]
    pub allow_already_satisfied: bool,
    /// Require a separate host review with the named host_verification_checks, including for design.
    #[serde(default)]
    pub host_review: bool,
    #[serde(default)]
    pub memory_recall: MemoryRecall,
    /// Optional focused recall queries; at most eight, each at most 512 characters.
    /// The embedding tokenizer also rejects queries exceeding its token window.
    #[serde(default)]
    pub memory_queries: Vec<String>,
    pub objective: String,
    /// Legacy descriptive field, accepted for compatibility; workflows own routing floors.
    #[schemars(skip)]
    #[serde(default = "legacy_intent")]
    pub task_intent: TaskIntent,
    /// Legacy descriptive field, accepted for compatibility; use requested_tier to raise a floor.
    #[schemars(skip)]
    #[serde(default = "legacy_complexity")]
    pub complexity_class: ComplexityClass,
    #[serde(default)]
    pub deterministic_operation: Option<DeterministicOperation>,
    #[serde(default)]
    pub deterministic_input: Option<String>,
    pub assigned_tools: Vec<String>,
    pub task_needs: Vec<super::TaskNeed>,
    #[serde(default)]
    pub primary_role: Option<String>,
    #[serde(default)]
    pub supporting_skills: Vec<String>,
    #[serde(default)]
    pub skill_references: Vec<super::SkillReference>,
    pub project: PathBuf,
    #[serde(default)]
    pub requires_vision: bool,
    #[serde(default)]
    pub requested_tier: Option<Tier>,
    #[serde(default)]
    pub model_selection: Option<ModelSelection>,
    /// Required for design production: an inspected image, native specimen with
    /// suitability reason, or legacy capability fallback. Does not grant tool access.
    #[serde(default)]
    pub design_input: Option<super::DesignInput>,
    #[serde(default)]
    pub verification_tools: Vec<String>,
    /// Named checks the host must report after source implementation.
    #[serde(default = "super::handoff::default_checks")]
    pub host_verification_checks: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Resolution {
    pub min_tier: Tier,
    pub requires_vision: bool,
    pub requires_structured_outputs: bool,
    pub requires_tools: bool,
    pub effort_profile: String,
    pub requested_effort: Option<String>,
    pub downgrade_rejected: bool,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyError {
    EmptyObjective,
    EmptyProject,
    UnknownTool(String),
    DuplicateToolPolicy(String),
    UnassignedVerificationTool(String),
}

impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyObjective => write!(f, "objective must not be empty"),
            Self::EmptyProject => write!(f, "project must not be empty"),
            Self::UnknownTool(name) => write!(f, "unknown assigned tool: {name}"),
            Self::DuplicateToolPolicy(name) => write!(f, "duplicate tool policy: {name}"),
            Self::UnassignedVerificationTool(name) => {
                write!(f, "verification tool is not assigned: {name}")
            }
        }
    }
}

impl Error for PolicyError {}

/// Capability envelope, not model quality. Identity and marketing never affect eligibility.
/// Tier names remain wire-compatible; quality is unmeasured until backed by evaluations.
pub fn classify(model: &mut Model) {
    model
        .classification_evidence
        .retain(|e| !e.starts_with("policy: "));
    model.tier = Tier::T1Deterministic;
    if model.capabilities.tools.is_none() || model.capabilities.structured_outputs.is_none() {
        model.available = false;
        model
            .classification_evidence
            .push("policy: unknown tool or structured-output contract; unavailable".into());
        return;
    }
    if model.capabilities.tools == Some(true) && model.capabilities.structured_outputs == Some(true)
    {
        model.tier = if model.capabilities.reasoning_efforts.is_empty()
            || model
                .capabilities
                .reasoning_efforts
                .iter()
                .any(|e| effort_rank(e).is_some_and(|r| r >= 4))
        {
            Tier::T3Architect
        } else {
            Tier::T2Synthesis
        };
    }
    model.classification_evidence.push(
        "policy: verified capability envelope; quality unmeasured; no model-name classification"
            .into(),
    );
}

fn legacy_intent() -> TaskIntent {
    TaskIntent::Synthesis
}
fn legacy_complexity() -> ComplexityClass {
    ComplexityClass::Synthesis
}

/// Resolve a task floor using only the request and its owned tool policies.
pub fn resolve(
    request: &SpawnRequest,
    tools: &[ToolPolicy],
    workflow_floor: Tier,
) -> Result<Resolution, PolicyError> {
    if request.objective.trim().is_empty() {
        return Err(PolicyError::EmptyObjective);
    }
    if request.project.as_os_str().is_empty() {
        return Err(PolicyError::EmptyProject);
    }

    let mut policies = HashMap::new();
    for policy in tools {
        if policies.insert(policy.name.as_str(), policy).is_some() {
            return Err(PolicyError::DuplicateToolPolicy(policy.name.clone()));
        }
    }
    let assigned: HashSet<&str> = request.assigned_tools.iter().map(String::as_str).collect();
    for name in &request.verification_tools {
        if !assigned.contains(name.as_str()) {
            return Err(PolicyError::UnassignedVerificationTool(name.clone()));
        }
    }

    let mut min_tier = workflow_floor;
    let mut requires_vision = request.requires_vision;
    let mut reasons = vec![format!("owned workflow floor: {workflow_floor:?}")];

    for name in &request.assigned_tools {
        let policy = policies
            .get(name.as_str())
            .ok_or_else(|| PolicyError::UnknownTool(name.clone()))?;
        min_tier = min_tier.max(policy.min_tier);
        requires_vision |= policy.requires_vision;
        reasons.push(format!("tool {name} floor: {:?}", policy.min_tier));
    }

    let mut downgrade_rejected = false;
    if let Some(requested) = request.requested_tier {
        if requested < min_tier {
            downgrade_rejected = true;
            reasons.push(format!(
                "requested {requested:?} cannot lower the task/tool floor"
            ));
        } else {
            min_tier = requested;
            reasons.push(format!("requested tier floor: {requested:?}"));
        }
    }

    if requires_vision {
        reasons.push("vision required by task or assigned tool".into());
    }
    let requires_structured_outputs = true;
    let effort_profile = if min_tier >= Tier::T2Synthesis {
        "high"
    } else {
        "low"
    };
    Ok(Resolution {
        min_tier,
        requires_vision,
        requires_structured_outputs,
        effort_profile: effort_profile.into(),
        requested_effort: request
            .model_selection
            .as_ref()
            .map(|choice| choice.effort.clone()),
        downgrade_rejected,
        reasons,
        requires_tools: !request.assigned_tools.is_empty(),
    })
}

fn effort_rank(name: &str) -> Option<u8> {
    match name.to_ascii_lowercase().as_str() {
        "none" => Some(0),
        "minimal" => Some(1),
        "low" => Some(2),
        "medium" => Some(3),
        "high" => Some(4),
        "xhigh" => Some(5),
        "max" => Some(6),
        "ultra" => Some(7),
        _ => None,
    }
}

/// Choose a supported effort. `None` means either no reasoning-effort control
/// is advertised or a high effort is unavailable for a T2/T3 request. Callers
/// should use `rank_candidates` to distinguish those cases.
pub fn effort_for(model: &Model, resolution: &Resolution) -> Option<String> {
    let efforts = &model.capabilities.reasoning_efforts;
    let floor = if resolution.min_tier >= Tier::T2Synthesis {
        4
    } else {
        0
    };
    if let Some(requested) = &resolution.requested_effort {
        return efforts
            .iter()
            .find(|effort| {
                *effort == requested && effort_rank(effort).is_some_and(|rank| rank >= floor)
            })
            .cloned();
    }
    efforts
        .iter()
        .filter_map(|effort| effort_rank(effort).map(|rank| (rank, effort)))
        .filter(|(rank, _)| *rank >= floor)
        .min_by_key(|(rank, _)| *rank)
        .map(|(_, effort)| effort.clone())
}

/// Provider ceiling, capped at 64k; the executor applies the smaller per-run limit.
/// The executor must also subtract its actual input from the context window.
/// A missing completion limit cannot establish a safe explicit token request.
pub fn completion_tokens_for(model: &Model, _resolution: &Resolution) -> Option<u64> {
    model
        .capabilities
        .max_completion_tokens
        .filter(|limit| *limit > 0)
        .map(|limit| {
            let context_limit = model
                .capabilities
                .context_window
                .filter(|window| *window > 0);
            limit.min(64_000).min(context_limit.unwrap_or(u64::MAX))
        })
}

/// Return eligible models, preferring the highest capability tier that satisfies the floor.
/// IDs are used only as a deterministic final tie-breaker, never for policy.
pub fn rank_candidates(models: &[Model], resolution: &Resolution) -> Vec<Model> {
    let mut eligible: Vec<Model> = models
        .iter()
        .cloned()
        .map(|mut model| {
            classify(&mut model);
            model
        })
        .filter(|model| {
            model.available
                && model.capabilities.tools.is_some()
                && model.capabilities.structured_outputs.is_some()
                && model.tier >= resolution.min_tier
                && (!resolution.requires_vision || model.capabilities.vision == Some(true))
                && (!resolution.requires_tools || model.capabilities.tools == Some(true))
                && (!resolution.requires_structured_outputs
                    || model.capabilities.structured_outputs == Some(true))
                && (resolution.min_tier < Tier::T2Synthesis
                    || (model.capabilities.tools == Some(true)
                        && model.capabilities.structured_outputs == Some(true)))
                && (resolution.requested_effort.is_none()
                    || effort_for(model, resolution).is_some())
                && (model.capabilities.reasoning_efforts.is_empty()
                    || resolution.min_tier < Tier::T2Synthesis
                    || effort_for(model, resolution).is_some())
        })
        .collect();
    eligible.sort_by(|a, b| {
        b.tier
            .cmp(&a.tier)
            .then_with(|| a.target.cmp(&b.target))
            .then_with(|| a.id.cmp(&b.id))
    });
    eligible
}

#[cfg(test)]
include!("tests/policy.rs");
