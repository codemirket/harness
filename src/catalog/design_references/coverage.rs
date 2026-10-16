//! Inventory claims are scoped to one lesson and platform, never a union of tags.
use super::{Instance, nonempty, record_path, resource, route};
use anyhow::{Context, Result, ensure};
use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const PATH: &str = "references/design-languages/coverage.json";
const GUIDE: &str = "references/design-languages/coverage.md";

/// Exact taxonomy IDs from design-inspiration's coverage registry. Omitted axes
/// are unspecified, not implicitly satisfied for the eventual user task.
#[derive(Debug, Default, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DesignContext {
    pub family: Option<String>,
    pub platform: Option<String>,
    #[serde(default)]
    #[schemars(length(max = 8))]
    pub interactions: Vec<String>,
    #[serde(default)]
    #[schemars(length(max = 8))]
    pub constraints: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Coverage {
    schema_version: u32,
    axes: Axes,
    cells: Vec<Cell>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Axes {
    families: Vec<Axis>,
    platforms: Vec<Axis>,
    interactions: Vec<Axis>,
    constraints: Vec<Axis>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Axis {
    id: String,
    meaning: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Level {
    WorkedExample,
    SourceOnly,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Cell {
    instance: String,
    family: String,
    platform: String,
    interactions: Vec<String>,
    constraints: Vec<String>,
    level: Level,
    scope: String,
    limits: String,
}

fn member(axes: &[Axis], value: &str, axis: &str) -> Result<()> {
    ensure!(
        axes.iter().any(|entry| entry.id == value),
        "unknown design {axis} ID: {value}; read {PATH}"
    );
    Ok(())
}

fn members(axes: &[Axis], values: &[String], axis: &str) -> Result<()> {
    let mut seen = BTreeSet::new();
    ensure!(
        values.len() <= 8,
        "at most 8 design {axis} IDs are supported"
    );
    for value in values {
        member(axes, value, axis)?;
        ensure!(seen.insert(value), "duplicate design {axis} ID: {value}");
    }
    Ok(())
}

impl Coverage {
    pub(super) fn load(files: &BTreeMap<String, Vec<u8>>, instances: &[Instance]) -> Result<Self> {
        resource(files, PATH)?;
        resource(files, GUIDE)?;
        let coverage: Self =
            serde_json::from_slice(&files[PATH]).context("invalid design coverage registry")?;
        coverage.validate(instances)?;
        Ok(coverage)
    }

    fn validate(&self, instances: &[Instance]) -> Result<()> {
        ensure!(
            self.schema_version == 1,
            "unsupported design coverage schema"
        );
        for axis in [
            &self.axes.families,
            &self.axes.platforms,
            &self.axes.interactions,
            &self.axes.constraints,
        ] {
            ensure!(
                (1..=64).contains(&axis.len()),
                "design coverage axis must contain 1..=64 entries"
            );
            let mut seen = BTreeSet::new();
            for entry in axis {
                super::validate_id(&entry.id)?;
                ensure!(
                    seen.insert(&entry.id)
                        && nonempty(&entry.meaning)
                        && entry.meaning.len() <= 512,
                    "invalid or duplicate coverage axis: {}",
                    entry.id
                );
            }
        }
        ensure!(
            (1..=192).contains(&self.cells.len()),
            "design coverage must contain 1..=192 scoped cells"
        );
        let mut cells = BTreeSet::new();
        let mut represented = BTreeSet::new();
        for cell in &self.cells {
            ensure!(
                instances
                    .iter()
                    .any(|instance| instance.id == cell.instance),
                "unknown design coverage instance: {}",
                cell.instance
            );
            member(&self.axes.families, &cell.family, "family")?;
            member(&self.axes.platforms, &cell.platform, "platform")?;
            members(&self.axes.interactions, &cell.interactions, "interaction")?;
            members(&self.axes.constraints, &cell.constraints, "constraint")?;
            ensure!(
                [&cell.scope, &cell.limits]
                    .iter()
                    .all(|value| nonempty(value) && value.len() <= 2048),
                "missing or oversized design coverage scope: {}",
                cell.instance
            );
            let mut interactions = cell.interactions.clone();
            let mut constraints = cell.constraints.clone();
            interactions.sort();
            constraints.sort();
            ensure!(
                cells.insert((
                    cell.instance.clone(),
                    cell.family.clone(),
                    cell.platform.clone(),
                    interactions,
                    constraints
                )),
                "duplicate scoped design coverage cell: {}",
                cell.instance
            );
            represented.insert(cell.instance.as_str());
        }
        ensure!(
            instances
                .iter()
                .all(|instance| represented.contains(instance.id.as_str())),
            "each design instance needs explicit coverage scope"
        );
        Ok(())
    }

    fn validate_context(&self, context: &DesignContext) -> Result<()> {
        ensure!(
            context.family.is_some()
                || context.platform.is_some()
                || !context.interactions.is_empty()
                || !context.constraints.is_empty(),
            "design context needs at least one explicit axis"
        );
        if let Some(family) = &context.family {
            member(&self.axes.families, family, "family")?;
        }
        if let Some(platform) = &context.platform {
            member(&self.axes.platforms, platform, "platform")?;
        }
        members(
            &self.axes.interactions,
            &context.interactions,
            "interaction",
        )?;
        members(&self.axes.constraints, &context.constraints, "constraint")?;
        Ok(())
    }

    pub(super) fn lookup(&self, context: &DesignContext, instances: &[Instance]) -> Result<Value> {
        self.validate_context(context)?;
        let mut candidates: Vec<_> = self.cells.iter().filter(|cell| {
            // A requested family is a hard relevance boundary. Generic keyboard
            // support elsewhere cannot make an absent device family look covered.
            if let Some(family) = &context.family { return &cell.family == family; }
            if let Some(platform) = &context.platform { return &cell.platform == platform; }
            context.interactions.iter().any(|value| cell.interactions.contains(value))
                || context.constraints.iter().any(|value| cell.constraints.contains(value))
        }).map(|cell| {
            let family = context.family.as_ref().filter(|value| *value != &cell.family);
            let platform = context.platform.as_ref().filter(|value| *value != &cell.platform);
            let interactions: Vec<_> = context.interactions.iter().filter(|value| !cell.interactions.contains(value)).collect();
            let constraints: Vec<_> = context.constraints.iter().filter(|value| !cell.constraints.contains(value)).collect();
            let missing_count = usize::from(family.is_some()) + usize::from(platform.is_some()) + interactions.len() + constraints.len();
            (missing_count, cell, json!({"family":family,"platform":platform,"interactions":interactions,"constraints":constraints,"worked_example":cell.level != Level::WorkedExample}))
        }).collect();
        candidates.sort_by(|(a_missing, a, _), (b_missing, b, _)| {
            a_missing
                .cmp(b_missing)
                .then_with(|| {
                    (a.level != Level::WorkedExample).cmp(&(b.level != Level::WorkedExample))
                })
                .then_with(|| a.instance.cmp(&b.instance))
                .then_with(|| a.platform.cmp(&b.platform))
        });
        let full = candidates
            .iter()
            .filter(|(missing, cell, _)| *missing == 0 && cell.level == Level::WorkedExample)
            .count();
        let status = if full > 0 {
            "represented"
        } else if candidates.is_empty() {
            "unrepresented"
        } else {
            "partial"
        };
        let result = json!({
            "requested":context,"status":status,"combination_represented":full > 0,
            "current_task_validated":false,
            "meaning":"Inventory coverage of only the requested axes. A single scoped worked example must contain the complete combination; unrelated cells are never pooled. Verify the actual task separately.",
            "taxonomy":route(PATH),"guide":route(GUIDE),
            "total_candidates":candidates.len(),"shown":candidates.len().min(5),
            "candidates":candidates.iter().take(5).map(|(missing, cell, gaps)| {
                let instance = instances.iter().find(|instance| instance.id == cell.instance).expect("validated coverage identity");
                json!({"id":cell.instance,"title":instance.title,"family":cell.family,"platform":cell.platform,
                    "interactions":cell.interactions,"constraints":cell.constraints,"level":cell.level,
                    "complete_requested_combination":*missing == 0 && cell.level == Level::WorkedExample,
                    "missing":gaps,"scope":cell.scope,"limits":cell.limits,
                    "record":route(&record_path(&instance.id)),"lesson":route(&instance.card),"example":route(&instance.example)})
            }).collect::<Vec<_>>(),
            "next_action":if full > 0 { "These scoped examples are available for study or comparison. Representation does not require using them; project, native and other sources remain available. Verify the actual result on its target platform." } else { "This inventory gap does not restrict possible designs or sources. Use relevant existing knowledge, project or primary guidance; investigate actual uncertainty and verify platform behavior before claiming task coverage." }
        });
        ensure!(
            serde_json::to_vec(&result)?.len() <= 24 * 1024,
            "design coverage result exceeds bounded response; narrow the request"
        );
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn library() -> super::super::Library {
        let payload = crate::catalog::Catalog::embedded()
            .unwrap()
            .payload(super::super::OWNER)
            .unwrap();
        super::super::Library::load(&payload).unwrap()
    }

    #[test]
    fn lookup_preserves_gaps_and_never_certifies_the_current_task() {
        let library = library();
        let context = DesignContext {
            family: Some("procedural-services".into()),
            platform: Some("web".into()),
            interactions: vec!["forms-validation".into()],
            constraints: vec!["keyboard".into()],
        };
        let result = library.coverage_for(&context).unwrap();
        assert_eq!(result["status"], "represented");
        assert_eq!(result["current_task_validated"], false);
        assert_eq!(result["candidates"][0]["id"], "govuk-procedural");
        let absent = DesignContext {
            family: Some("device-specialist".into()),
            platform: Some("tv".into()),
            interactions: vec!["remote-navigation".into()],
            ..Default::default()
        };
        assert_eq!(
            library.coverage_for(&absent).unwrap()["status"],
            "unrepresented"
        );
        let complex = DesignContext {
            family: Some("complex-workspaces".into()),
            platform: Some("native-mobile".into()),
            interactions: vec!["offline-sync".into()],
            constraints: vec!["rtl".into(), "screen-reader".into()],
        };
        let result = library.coverage_for(&complex).unwrap();
        assert_eq!(result["status"], "partial");
        assert_eq!(result["combination_represented"], false);
        for candidate in result["candidates"].as_array().unwrap() {
            assert_eq!(candidate["complete_requested_combination"], false);
            assert!(
                candidate["missing"]["interactions"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("offline-sync"))
            );
            assert!(
                candidate["missing"]["constraints"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("screen-reader"))
            );
        }
    }

    #[test]
    fn unrelated_cells_cannot_be_pooled_into_a_complete_combination() {
        let mut library = library();
        let cell = &mut library.coverage.cells[0];
        cell.platform = "native-mobile".into();
        cell.constraints.clear();
        let cell = &mut library.coverage.cells[1];
        cell.family = "procedural-services".into();
        cell.constraints.push("rtl".into());
        let context = DesignContext {
            family: Some("procedural-services".into()),
            platform: Some("native-mobile".into()),
            constraints: vec!["rtl".into()],
            ..Default::default()
        };
        let result = library.coverage_for(&context).unwrap();
        assert_eq!(result["status"], "partial");
        assert_eq!(result["combination_represented"], false);
        // Even exact source guidance still lacks a worked native implementation.
        library.coverage.cells[0].constraints.push("rtl".into());
        library.coverage.cells[0].level = Level::SourceOnly;
        let result = library.coverage_for(&context).unwrap();
        assert_eq!(result["status"], "partial");
        assert_eq!(result["candidates"][0]["missing"]["worked_example"], true);
    }

    #[test]
    fn invalid_requests_and_unaccounted_coverage_are_rejected() {
        let subject = library();
        for context in [
            DesignContext::default(),
            DesignContext {
                family: Some("unrecognized".into()),
                ..Default::default()
            },
            DesignContext {
                constraints: vec!["keyboard".into(), "keyboard".into()],
                ..Default::default()
            },
        ] {
            assert!(subject.coverage_for(&context).is_err());
        }
        for mutation in ["instance", "axis", "duplicate", "scope", "version"] {
            let mut library = library();
            match mutation {
                "instance" => library.coverage.cells[0].instance = "absent".into(),
                "axis" => library.coverage.cells[0].platform = "imaginary".into(),
                "duplicate" => library.coverage.axes.platforms.push(Axis {
                    id: "web".into(),
                    meaning: "duplicate".into(),
                }),
                "scope" => library.coverage.cells[0].scope.clear(),
                "version" => library.coverage.schema_version = 2,
                _ => unreachable!(),
            }
            assert!(
                library.coverage.validate(&library.instances).is_err(),
                "{mutation}"
            );
        }
    }
}
