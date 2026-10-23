//! Ordered visual evidence for owned design workers. Judgment remains model
//! testimony; these gates enforce evidence identity, sequence and a finite budget.
use super::tools::{self, ArtifactRef, ToolResult};
use anyhow::{Result, ensure};
use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// One shared budget for all declared deliverables, never a sum of workflow limits.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Budget {
    pub execution_seconds: u64,
    pub max_correction_rounds: usize,
}

/// Resolved once from the selected owned workflows, and retained in the contract.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionPolicy {
    pub budget: Budget,
    pub host_verification: bool,
    pub interactive: bool,
    pub motion: bool,
}
impl ExecutionPolicy {
    pub fn from_workflows(workflows: &[&crate::catalog::Workflow]) -> Self {
        use crate::catalog::BudgetClass;
        let class = workflows
            .iter()
            .map(|w| w.budget_class)
            .max()
            .unwrap_or(BudgetClass::Static);
        let (execution_seconds, max_correction_rounds) = match class {
            BudgetClass::Static => (900, 2),
            BudgetClass::Composition => (1800, 3),
            BudgetClass::Motion => (2700, 4),
        };
        Self {
            budget: Budget {
                execution_seconds,
                max_correction_rounds,
            },
            host_verification: workflows.iter().any(|w| w.host_verification),
            interactive: workflows.iter().any(|w| w.interactive),
            motion: workflows.iter().any(|w| w.motion),
        }
    }
}

pub async fn within_budget<T>(
    execution: impl std::future::Future<Output = Result<T>>,
    budget: std::time::Duration,
) -> Result<T> {
    tokio::time::timeout(budget, execution).await.unwrap_or_else(|_| {
        Err(anyhow::anyhow!("original execution budget exhausted; preserve partial output, do not automatically relaunch"))
    })
}

#[derive(Clone, Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum DesignInput {
    ImageToCode {
        image: String,
    },
    /// Compose a minimal proof in the deliverable's native medium, then freeze its render.
    NativeSpecimen {
        medium: Medium,
        reason: String,
    },
    /// Legacy input accepted during the 0.x migration; new clients use native_specimen.
    #[schemars(skip)]
    CodeFallback {
        reason: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Medium {
    Dom,
    Svg,
    Pixel,
    Diagram,
    Typeset,
    DesignDocument,
}

impl DesignInput {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::ImageToCode { image } => bounded(image, 1024),
            Self::CodeFallback { reason } | Self::NativeSpecimen { reason, .. } => {
                bounded(reason, 600)
            }
        }
    }
}

fn bounded(text: &str, max: usize) -> Result<()> {
    ensure!(
        !text.trim().is_empty() && text.len() <= max && !text.contains('\0'),
        "design gate text requires 1..={max} bytes without NUL"
    );
    Ok(())
}

pub fn contract(input: &DesignInput, policy: &ExecutionPolicy) -> Value {
    let budget = policy.budget;
    json!({
        "input":input, "execution_seconds":budget.execution_seconds,
        "max_correction_rounds":budget.max_correction_rounds,
        "action":"design_review", "tool":"",
        "arguments_schema":schemars::schema_for!(Review),
        "sequence":"Inspect the target then submit stage target before any mutation in image_to_code. In native_specimen or code_fallback compose one minimal proof, render and inspect it, then submit stage target before expansion. The target is frozen for this run. Implement real editable structure and behavior, render current output, inspect it and submit stage final with comparison and located observations for every check. The initial final review plus max_correction_rounds repairs share execution_seconds of wall time, including renders and verification; combined workflows use the largest limit, never their sum. Failed or stale evidence prevents completion. Do not relaunch workers to reset the budget. No image provider or shell access is added.",
        "judgment_boundary":"Checks are worker testimony, not an automated aesthetic score or human approval. Source-to-render semantic correspondence and the suitability of a native medium or truth of a fallback reason still need host judgment."
    })
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Stage {
    Target,
    Final,
}

#[derive(Debug, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Status {
    Pass,
    Fail,
    NotApplicable,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Check {
    status: Status,
    observation: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Review {
    stage: Stage,
    /// Already inspected target, or current rendered output(s), in assigned workspaces.
    images: Vec<String>,
    /// Target: intended hierarchy. Final: observed differences from the frozen target.
    comparison: String,
    /// Composition plus applicable axes; final interactive work requires responsive/behavior, motion requires motion.
    checks: BTreeMap<String, Check>,
}

#[derive(Serialize, Deserialize)]
pub struct Gate {
    input: DesignInput,
    target: Vec<ArtifactRef>,
    observed: BTreeMap<PathBuf, ArtifactRef>,
    rendered: BTreeMap<PathBuf, ArtifactRef>,
    final_pass: bool,
    reviews: usize,
    target_reviews: usize,
    budget: Budget,
    interactive: bool,
    motion: bool,
}

impl Gate {
    pub fn new(input: DesignInput, policy: &ExecutionPolicy) -> Self {
        Self {
            input,
            target: vec![],
            observed: BTreeMap::new(),
            rendered: BTreeMap::new(),
            final_pass: false,
            reviews: 0,
            target_reviews: 0,
            budget: policy.budget,
            interactive: policy.interactive,
            motion: policy.motion,
        }
    }

    pub fn before_tool(&self, name: &str) -> Result<()> {
        if tools::may_mutate(name) {
            ensure!(
                !self.target.is_empty() || self.target_reviews < 2,
                "target correction budget exhausted; report incomplete"
            );
            ensure!(
                self.reviews < (self.budget.max_correction_rounds + 1),
                "design correction budget exhausted; preserve output and report incomplete"
            );
            ensure!(
                !matches!(self.input, DesignInput::ImageToCode { .. }) || !self.target.is_empty(),
                "image-to-code requires target inspection and a passing target review before mutation"
            );
            self.check_target()?;
        }
        Ok(())
    }

    fn check_target(&self) -> Result<()> {
        for image in &self.target {
            ensure!(
                tools::current_digest(&image.path)? == image.sha256,
                "frozen design target changed; run remains incomplete"
            );
        }
        Ok(())
    }

    pub fn observe(&mut self, name: &str, result: &ToolResult) {
        if result.mutates {
            self.final_pass = false;
            // Captures/exports/diffs do not change authored source. Authoring,
            // including renderers that accept source, invalidates older output.
            if !matches!(
                name,
                "design/browser_screenshot" | "design/openpencil_export" | "design/odiff_compare"
            ) {
                self.rendered.clear();
            }
        }
        if !result.passed {
            return;
        }
        if matches!(
            name,
            "design/browser_screenshot"
                | "design/openpencil_export"
                | "design/static_render"
                | "design/typst_compile"
                | "design/vector_render"
        ) {
            for image in &result.artifacts {
                self.rendered.insert(image.path.clone(), image.clone());
            }
        }
        if let Some(image) = &result.image {
            for receipt in &result.verifies {
                if &receipt.path == image {
                    self.observed.insert(image.clone(), receipt.clone());
                }
            }
        }
    }

    /// Rejected testimony has no effect. The caller's existing turn/deadline
    /// budget still bounds retries; accepted reviews consume the review budget.
    pub fn review_feedback(&mut self, project: &Path, arguments: &str) -> Value {
        let result = (|| {
            ensure!(arguments.len() <= 16 * 1024, "design review exceeds 16 KiB");
            self.review(project, serde_json::from_str(arguments)?)
        })();
        match result {
            Ok(receipt) => receipt,
            Err(error) => json!({"accepted":false,"passed":false,
                "reason":error.to_string().chars().take(1200).collect::<String>()}),
        }
    }

    pub fn review(&mut self, project: &Path, args: Value) -> Result<Value> {
        let review: Review = serde_json::from_value(args)?;
        ensure!(
            (1..=8).contains(&review.images.len()),
            "design review needs 1..=8 inspected images"
        );
        bounded(&review.comparison, 2048)?;
        let axes = [
            "composition",
            "typography",
            "assets",
            "responsive",
            "behavior",
            "motion",
        ];
        ensure!(
            review.checks.contains_key("composition")
                && review.checks.keys().all(|key| axes.contains(&key.as_str())),
            "design review requires composition and only recognized applicable axes"
        );
        let final_stage = matches!(review.stage, Stage::Final);
        if final_stage {
            for axis in ["responsive", "behavior"]
                .into_iter()
                .filter(|_| self.interactive)
                .chain(std::iter::once("motion").filter(|_| self.motion))
            {
                ensure!(
                    review.checks.contains_key(axis),
                    "missing required design check: {axis}"
                );
            }
        }
        for (axis, check) in &review.checks {
            bounded(&check.observation, 1200)?;
            let required = axis == "composition"
                || (final_stage
                    && ((self.interactive && matches!(axis.as_str(), "responsive" | "behavior"))
                        || (self.motion && axis == "motion")));
            ensure!(
                !required || check.status != Status::NotApplicable,
                "required design check cannot be waived: {axis}"
            );
        }
        let passed = review.checks.values().all(|c| c.status != Status::Fail);
        let mut images = Vec::new();
        for path in &review.images {
            let path = project.join(path).canonicalize()?;
            let receipt = self.observed.get(&path).ok_or_else(|| {
                anyhow::anyhow!("review image was not inspected: {}", path.display())
            })?;
            ensure!(
                tools::current_digest(&path)? == receipt.sha256,
                "review image changed after inspection"
            );
            if final_stage || !matches!(self.input, DesignInput::ImageToCode { .. }) {
                ensure!(
                    self.rendered.get(&path) == Some(receipt),
                    "review must bind current rendered output, not unrelated or stale pixels"
                );
            }
            images.push(receipt.clone());
        }
        match review.stage {
            Stage::Target => {
                ensure!(self.target.is_empty(), "design target is already frozen");
                ensure!(
                    self.target_reviews < 2,
                    "target correction budget exhausted; report incomplete"
                );
                if let DesignInput::ImageToCode { image } = &self.input {
                    let expected = project.join(image).canonicalize()?;
                    ensure!(
                        images.iter().any(|item| item.path == expected),
                        "review does not include the declared target image"
                    );
                }
                self.target_reviews += 1;
                if passed {
                    self.target = images;
                }
            }
            Stage::Final => {
                ensure!(
                    !self.target.is_empty(),
                    "inspect and pass the target gate before final review"
                );
                self.check_target()?;
                ensure!(
                    self.reviews < (self.budget.max_correction_rounds + 1),
                    "design review budget exhausted; report incomplete"
                );
                self.reviews += 1;
                self.final_pass = passed;
            }
        }
        Ok(
            json!({"passed":passed,"target_frozen":!self.target.is_empty(),"final_pass":self.final_pass,
            "final_reviews_remaining":(self.budget.max_correction_rounds + 1)-self.reviews,"judgment":"worker testimony, not human approval"}),
        )
    }

    pub fn complete(&self) -> Result<()> {
        ensure!(
            self.final_pass,
            "design quality gate has no passing final review of current rendered output"
        );
        self.check_target()
    }
}

#[cfg(test)]
include!("tests/design_gate.rs");
