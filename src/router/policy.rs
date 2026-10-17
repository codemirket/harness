//! Deterministic task floors and provisional model capability classification.
//!
//! Classification uses current discovery metadata, never a published model ID.
//! Its tier is a routing heuristic, not a measured quality or benchmark claim.
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
    Convert { from: String, to: String },
    FormatCliArguments,
    ExtractSchema { from: String },
    TrimWhitespace,
    RenameFile,
    DeduplicateLines,
    SortLines,
}

/// Task needs constrain owned role, guidance and tool selection before execution.
/// No model identifier is accepted at this boundary.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnRequest {
    pub objective: String,
    pub task_intent: TaskIntent,
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
    pub verification_tools: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Resolution {
    pub min_tier: Tier,
    pub requires_vision: bool,
    pub requires_structured_outputs: bool,
    pub requires_tools: bool,
    pub effort_profile: String,
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

fn model_words(model: &Model) -> Vec<String> {
    format!("{} {} {}", model.id, model.display_name, model.description)
        .to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect()
}

fn contains_word(words: &[String], needle: &str) -> bool {
    words.iter().any(|word| word == needle)
}

fn lightweight_signal(model: &Model, words: &[String]) -> Option<&'static str> {
    let identity: Vec<_> = format!("{} {}", model.id, model.display_name)
        .to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect();
    for marker in ["luna", "flash", "haiku", "mini", "nano", "small"] {
        if contains_word(&identity, marker) {
            return Some(marker);
        }
    }
    let description = model.description.to_ascii_lowercase();
    if description.contains("fast utility") || description.contains("fast and affordable") {
        return Some("fast utility");
    }
    if let Some(line) = model.line.as_deref() {
        return (line == "haiku").then_some("line:haiku");
    }
    [
        "luna",
        "flash",
        "haiku",
        "mini",
        "nano",
        "small",
        "speed",
        "affordable",
    ]
    .into_iter()
    .find(|marker| contains_word(words, marker))
}

fn frontier_signal(model: &Model, words: &[String]) -> Option<&'static str> {
    if let Some(line) = model.line.as_deref() {
        return (line == "opus").then_some("line:opus");
    }
    for marker in ["opus", "ultra", "max", "frontier"] {
        if contains_word(words, marker) {
            return Some(marker);
        }
    }
    if words
        .windows(2)
        .any(|pair| pair[0] == "deep" && pair[1] == "think")
    {
        return Some("deep-think");
    }
    None
}

fn full_size_signal(model: &Model, words: &[String]) -> Option<&'static str> {
    if let Some(line) = model.line.as_deref() {
        return match line {
            "sonnet" => Some("line:sonnet"),
            "opus" => Some("line:opus"),
            _ => None,
        };
    }
    for marker in ["flagship", "workhorse", "sonnet", "pro"] {
        if contains_word(words, marker) {
            return Some(marker);
        }
    }
    if words
        .windows(2)
        .any(|pair| pair[0] == "full" && pair[1] == "size")
    {
        return Some("full-size");
    }
    None
}

fn supports_high_reasoning(model: &Model) -> bool {
    model
        .capabilities
        .reasoning_efforts
        .iter()
        .any(|effort| effort_rank(effort).is_some_and(|rank| rank >= 4))
}

/// Recompute a model's provisional tier from metadata and verified flags.
/// Preserve `available = false` when an adapter has marked the model unavailable.
pub fn classify(model: &mut Model) {
    model
        .classification_evidence
        .retain(|evidence| !evidence.starts_with("policy: "));
    model.tier = Tier::T1Deterministic;

    let words = model_words(model);
    if let Some(signal) = lightweight_signal(model, &words) {
        model.classification_evidence.push(format!(
            "policy: lightweight morphology ({signal}); fixed mechanical tier"
        ));
        if model.capabilities.tools.is_none() || model.capabilities.structured_outputs.is_none() {
            model.available = false;
            model
                .classification_evidence
                .push("policy: unknown tools or structured-output contract; unavailable".into());
        }
        return;
    }

    if model.capabilities.tools.is_none() || model.capabilities.structured_outputs.is_none() {
        model.available = false;
        model
            .classification_evidence
            .push("policy: unknown tools or structured-output contract; unavailable".into());
        return;
    }

    if model.capabilities.tools == Some(true) && model.capabilities.structured_outputs == Some(true)
    {
        let frontier = frontier_signal(model, &words);
        let full_size = full_size_signal(model, &words);
        if frontier.is_some() || full_size.is_some() {
            model.tier = Tier::T2Synthesis;
            model.classification_evidence.push(format!(
                "policy: verified tools and structured outputs; full-size signal {}",
                frontier.or(full_size).unwrap_or_default()
            ));
        } else {
            model.classification_evidence.push(
                "policy: no positive full-size or frontier evidence; quarantined at T1".into(),
            );
        }
        if model.tier == Tier::T2Synthesis
            && supports_high_reasoning(model)
            && (frontier.is_some() || full_size.is_some())
        {
            model.tier = Tier::T3Architect;
            model
                .classification_evidence
                .push("policy: high reasoning plus frontier or full-size workhorse signal".into());
        }
    } else {
        model.classification_evidence.push(
            "policy: missing verified tools or structured outputs; mechanical tier only".into(),
        );
    }
}

fn intent_floor(intent: TaskIntent) -> Tier {
    match intent {
        TaskIntent::FormatConversion
        | TaskIntent::CliParameters
        | TaskIntent::SchemaExtraction
        | TaskIntent::FileHygiene => Tier::T1Deterministic,
        TaskIntent::Synthesis | TaskIntent::MultiToolChaining => Tier::T2Synthesis,
        TaskIntent::Creativity
        | TaskIntent::Layout
        | TaskIntent::Motion
        | TaskIntent::Architecture
        | TaskIntent::Debugging
        | TaskIntent::MultiStepVisualQa
        | TaskIntent::MultiConstraint
        | TaskIntent::Ambiguity => Tier::T3Architect,
    }
}

fn complexity_floor(class: ComplexityClass) -> Tier {
    match class {
        ComplexityClass::Deterministic => Tier::T1Deterministic,
        ComplexityClass::Synthesis => Tier::T2Synthesis,
        ComplexityClass::CreativeArchitectural => Tier::T3Architect,
    }
}

fn mechanical_contract(request: &SpawnRequest) -> bool {
    let objective = request.objective.trim();
    let format = |name: &str| {
        [
            "json", "yaml", "yml", "csv", "tsv", "xml", "markdown", "md", "txt", "text", "html",
            "pdf",
        ]
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(name))
    };
    match (&request.task_intent, &request.deterministic_operation) {
        (TaskIntent::FormatConversion, Some(DeterministicOperation::Convert { from, to })) => {
            format(from)
                && format(to)
                && objective.eq_ignore_ascii_case(&format!("Convert {from} to {to}"))
        }
        (TaskIntent::CliParameters, Some(DeterministicOperation::FormatCliArguments)) => {
            objective.eq_ignore_ascii_case("Format CLI arguments")
        }
        (TaskIntent::SchemaExtraction, Some(DeterministicOperation::ExtractSchema { from })) => {
            format(from) && objective.eq_ignore_ascii_case(&format!("Extract fields from {from}"))
        }
        (TaskIntent::FileHygiene, Some(DeterministicOperation::TrimWhitespace)) => {
            objective.eq_ignore_ascii_case("Trim trailing whitespace")
        }
        (TaskIntent::FileHygiene, Some(DeterministicOperation::RenameFile)) => {
            objective.eq_ignore_ascii_case("Rename a file")
        }
        (TaskIntent::FileHygiene, Some(DeterministicOperation::DeduplicateLines)) => {
            objective.eq_ignore_ascii_case("Deduplicate lines")
        }
        (TaskIntent::FileHygiene, Some(DeterministicOperation::SortLines)) => {
            objective.eq_ignore_ascii_case("Sort lines")
        }
        _ => false,
    }
}

/// Resolve a task floor using only the request and its owned tool policies.
pub fn resolve(request: &SpawnRequest, tools: &[ToolPolicy]) -> Result<Resolution, PolicyError> {
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

    let mut task_floor =
        intent_floor(request.task_intent).max(complexity_floor(request.complexity_class));
    let objective_words: Vec<_> = request
        .objective
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    let complex_signal = [
        "architecture",
        "architectural",
        "architect",
        "debug",
        "debugging",
        "creative",
        "creativity",
        "layout",
        "choreography",
        "animation",
        "motion",
        "ambiguous",
        "ambiguity",
        "constraints",
        "multiconstraint",
        "visual",
        "spatial",
        "dashboard",
        "interface",
        "poster",
        "illustration",
        "storyboard",
        "composition",
        "typography",
        "responsive",
    ]
    .into_iter()
    .find(|word| objective_words.iter().any(|part| part == word));
    if complex_signal.is_some() {
        task_floor = Tier::T3Architect;
    }
    let mechanical_intent = matches!(
        request.task_intent,
        TaskIntent::FormatConversion
            | TaskIntent::CliParameters
            | TaskIntent::SchemaExtraction
            | TaskIntent::FileHygiene
    );
    let known_mechanical_output = !mechanical_intent || mechanical_contract(request);
    if !known_mechanical_output {
        task_floor = Tier::T3Architect;
    }
    let mut min_tier = task_floor;
    let mut requires_vision = request.requires_vision
        || matches!(
            request.task_intent,
            TaskIntent::Layout | TaskIntent::Motion | TaskIntent::MultiStepVisualQa
        );
    let mut reasons = vec![format!("task floor: {task_floor:?}")];
    if let Some(signal) = complex_signal {
        reasons.push(format!(
            "objective requires architectural/creative reasoning: {signal}"
        ));
    }
    if !known_mechanical_output {
        reasons.push("unrecognized mechanical output; elevated to T3".into());
    }
    if mechanical_intent && known_mechanical_output {
        reasons.push("explicit single-operation mechanical contract".into());
    }

    for name in &request.assigned_tools {
        let policy = policies
            .get(name.as_str())
            .ok_or_else(|| PolicyError::UnknownTool(name.clone()))?;
        min_tier = min_tier.max(policy.min_tier);
        requires_vision |= policy.requires_vision;
        reasons.push(format!("tool {name} floor: {:?}", policy.min_tier));
    }

    if request.assigned_tools.len() > 1 {
        min_tier = min_tier.max(Tier::T2Synthesis);
        reasons.push("multiple assigned tools require at least T2".into());
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
    if efforts.is_empty() {
        return None;
    }
    let floor = if resolution.min_tier >= Tier::T2Synthesis {
        4
    } else {
        0
    };
    efforts
        .iter()
        .filter_map(|effort| effort_rank(effort).map(|rank| (rank, effort)))
        .filter(|(rank, _)| *rank >= floor)
        .min_by_key(|(rank, _)| *rank)
        .map(|(_, effort)| effort.clone())
}

/// Aim for 64k completion tokens, capped by known positive provider limits.
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
                && (model.capabilities.reasoning_efforts.is_empty()
                    || resolution.min_tier < Tier::T2Synthesis
                    || effort_for(model, resolution).is_some())
        })
        .collect();
    eligible.sort_by(|a, b| {
        b.tier
            .cmp(&a.tier)
            .then_with(|| {
                frontier_signal(b, &model_words(b))
                    .is_some()
                    .cmp(&frontier_signal(a, &model_words(a)).is_some())
            })
            .then_with(|| {
                let strongest = |m: &Model| {
                    m.capabilities
                        .reasoning_efforts
                        .iter()
                        .filter_map(|e| effort_rank(e))
                        .max()
                };
                strongest(b).cmp(&strongest(a))
            })
            .then_with(|| {
                b.capabilities
                    .max_completion_tokens
                    .cmp(&a.capabilities.max_completion_tokens)
            })
            .then_with(|| {
                b.capabilities
                    .context_window
                    .cmp(&a.capabilities.context_window)
            })
            .then_with(|| a.target.cmp(&b.target))
            .then_with(|| a.id.cmp(&b.id))
    });
    eligible
}

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
            verification_tools: vec![],
        }
    }

    #[test]
    fn synthesis_label_cannot_lower_architectural_or_debugging_work() {
        for objective in [
            "Debugging a concurrent failure",
            "Resolve architecture constraints",
            "Invent a creative layout",
        ] {
            let mut task = request(TaskIntent::Synthesis, ComplexityClass::Synthesis);
            task.objective = objective.into();
            task.requested_tier = Some(Tier::T1Deterministic);
            let resolution = resolve(&task, &[]).unwrap();
            assert_eq!(resolution.min_tier, Tier::T3Architect);
            assert_eq!(resolution.effort_profile, "high");
            assert!(resolution.downgrade_rejected);
        }
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
                .any(|s| s.contains("frontier"))
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
    fn high_effort_and_large_limits_do_not_promote_utility_model() {
        let mut utility = model("Utility", "Fast utility for routine work");
        utility.capabilities.reasoning_efforts = vec!["ultra".into()];
        utility.capabilities.context_window = Some(1_000_000);
        classify(&mut utility);
        assert_eq!(utility.tier, Tier::T1Deterministic);
        let architect = resolve(
            &request(TaskIntent::Architecture, ComplexityClass::Synthesis),
            &[],
        )
        .unwrap();
        assert!(rank_candidates(&[utility], &architect).is_empty());
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
    fn lightweight_model_morphology_overrides_frontier_and_reasoning_signals() {
        let mut lite = model("Novel Flash-Lite Frontier", "full-size frontier workhorse");
        lite.capabilities.reasoning_efforts = vec!["ultra".into()];
        lite.classification_evidence
            .push("adapter: strict JSON action conformance".into());
        classify(&mut lite);
        assert_eq!(lite.tier, Tier::T1Deterministic);
        assert!(
            lite.classification_evidence
                .iter()
                .any(|s| s.starts_with("adapter: "))
        );
        classify(&mut lite);
        assert_eq!(
            lite.classification_evidence
                .iter()
                .filter(|s| s.starts_with("policy: "))
                .count(),
            1
        );
    }

    #[test]
    fn lightweight_identity_cannot_be_promoted_by_provider_family_metadata() {
        let mut haiku = model("Novel Opus", "frontier workhorse");
        haiku.line = Some("haiku".into());
        classify(&mut haiku);
        assert_eq!(haiku.tier, Tier::T1Deterministic);

        let mut sonnet = model("Novel Mini", "fast utility");
        sonnet.line = Some("sonnet".into());
        classify(&mut sonnet);
        assert_eq!(sonnet.tier, Tier::T1Deterministic);

        let mut standard = model("Novel Standard", "balanced speed and intelligence");
        standard.line = Some("sonnet".into());
        classify(&mut standard);
        assert_eq!(standard.tier, Tier::T3Architect);
    }

    #[test]
    fn unknown_model_is_t1_even_with_verified_tools_and_high_reasoning() {
        let mut unknown = model("Future Unknown", "general model for tasks");
        classify(&mut unknown);
        assert_eq!(unknown.tier, Tier::T1Deterministic);
        assert!(unknown.available);
        let architect = resolve(
            &request(TaskIntent::Architecture, ComplexityClass::Deterministic),
            &[],
        )
        .unwrap();
        assert!(rank_candidates(&[unknown], &architect).is_empty());
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
    fn mechanical_intent_cannot_hide_creative_or_unknown_output() {
        let mut req = request(TaskIntent::FormatConversion, ComplexityClass::Deterministic);
        req.objective = "Convert this design into a responsive layout".into();
        assert_eq!(resolve(&req, &[]).unwrap().min_tier, Tier::T3Architect);
        req.objective = "Please handle this".into();
        assert_eq!(resolve(&req, &[]).unwrap().min_tier, Tier::T3Architect);
        req.objective = "Convert this document".into();
        assert_eq!(resolve(&req, &[]).unwrap().min_tier, Tier::T3Architect);
        req.objective = "Convert CSV to JSON".into();
        req.deterministic_operation = Some(DeterministicOperation::Convert {
            from: "CSV".into(),
            to: "JSON".into(),
        });
        assert_eq!(resolve(&req, &[]).unwrap().min_tier, Tier::T1Deterministic);
    }

    #[test]
    fn mechanical_contract_rejects_compound_objectives_and_missing_operation() {
        let mut req = request(TaskIntent::FormatConversion, ComplexityClass::Deterministic);
        req.objective = "Convert CSV to JSON".into();
        assert_eq!(resolve(&req, &[]).unwrap().min_tier, Tier::T3Architect);
        req.deterministic_operation = Some(DeterministicOperation::Convert {
            from: "CSV".into(),
            to: "JSON".into(),
        });
        assert_eq!(resolve(&req, &[]).unwrap().min_tier, Tier::T1Deterministic);
        req.objective = "Convert CSV to JSON and summarize the findings".into();
        assert_eq!(resolve(&req, &[]).unwrap().min_tier, Tier::T3Architect);
    }

    #[test]
    fn generic_file_or_parameter_labels_do_not_establish_mechanical_work() {
        for (intent, objective) in [
            (TaskIntent::FileHygiene, "Build dashboard file"),
            (TaskIntent::FileHygiene, "Handle this file"),
            (TaskIntent::CliParameters, "Select a command"),
            (TaskIntent::SchemaExtraction, "Consider the schema"),
        ] {
            let mut req = request(intent, ComplexityClass::Deterministic);
            req.objective = objective.into();
            assert_eq!(resolve(&req, &[]).unwrap().min_tier, Tier::T3Architect);
        }
        for (intent, objective) in [
            (TaskIntent::FileHygiene, "Trim trailing whitespace"),
            (TaskIntent::CliParameters, "Format CLI arguments"),
            (TaskIntent::SchemaExtraction, "Extract fields from JSON"),
        ] {
            let mut req = request(intent, ComplexityClass::Deterministic);
            req.objective = objective.into();
            req.deterministic_operation = Some(match intent {
                TaskIntent::FileHygiene => DeterministicOperation::TrimWhitespace,
                TaskIntent::CliParameters => DeterministicOperation::FormatCliArguments,
                TaskIntent::SchemaExtraction => DeterministicOperation::ExtractSchema {
                    from: "JSON".into(),
                },
                _ => unreachable!(),
            });
            assert_eq!(resolve(&req, &[]).unwrap().min_tier, Tier::T1Deterministic);
        }
    }

    #[test]
    fn assigned_tools_require_verified_support_and_multiple_tools_raise_tier() {
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
            Tier::T2Synthesis
        );
    }
}
