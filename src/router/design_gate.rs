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
    /// Compatibility path for hosts without an image producer.
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
    /// Exactly composition, typography, assets, responsive, behavior and motion.
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
                | "design/kroki_render"
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
            review.checks.len() == axes.len()
                && axes.iter().all(|key| review.checks.contains_key(*key)),
            "design review requires exactly composition, typography, assets, responsive, behavior and motion"
        );
        let final_stage = matches!(review.stage, Stage::Final);
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
