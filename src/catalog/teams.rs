//! Professional briefs and task-scoped rosters. The host dispatches actual agents.
use super::{Capability, Catalog, validate_id};
use anyhow::{Context, Result, ensure};
use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProfessionalProfile {
    pub perspective: String,
    pub priorities: Vec<String>,
    pub method: Vec<String>,
    pub decision_rule: String,
    pub challenge: String,
    pub handoff: String,
}

fn bounded(value: &str, label: &str, limit: usize) -> Result<()> {
    ensure!(
        !value.trim().is_empty() && value.len() <= limit && !value.contains('\0'),
        "{label} must contain 1..={limit} bytes and no NUL characters"
    );
    Ok(())
}

impl ProfessionalProfile {
    pub(super) fn validate(&self, role: &str) -> Result<()> {
        ensure!(
            self.priorities.len() == 3,
            "{role}: specify three priorities"
        );
        ensure!(
            (3..=5).contains(&self.method.len()),
            "{role}: method needs 3..=5 steps"
        );
        for value in [
            &self.perspective,
            &self.decision_rule,
            &self.challenge,
            &self.handoff,
        ]
        .into_iter()
        .chain(&self.priorities)
        .chain(&self.method)
        {
            bounded(value, &format!("{role} professional field"), 400)?;
        }
        ensure!(
            serde_json::to_vec(self)?.len() <= 1600,
            "{role}: professional brief exceeds 1600 bytes"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TeamFacet {
    pub id: String,
    pub when: String,
    pub assignments: Vec<FacetAssignment>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FacetAssignment {
    pub role: String,
    pub mode: String,
    pub contribution: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FacetSelection {
    /// Exact ID returned by team_facets or mirket capabilities facets.
    pub id: String,
    /// Concrete requirement or observed boundary that justifies this expertise.
    pub reason: String,
}

impl std::str::FromStr for FacetSelection {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        let (id, reason) = value
            .split_once('=')
            .ok_or("use --facet 'ID=task-specific reason'")?;
        Ok(Self {
            id: id.trim().into(),
            reason: reason.trim().into(),
        })
    }
}

pub(super) fn validate_facets(facets: &[TeamFacet], roles: &[Capability]) -> Result<()> {
    ensure!(
        !facets.is_empty() && facets.len() <= 48,
        "expected 1..=48 team facets"
    );
    let mut ids = BTreeSet::new();
    for facet in facets {
        validate_id(&facet.id)?;
        ensure!(ids.insert(&facet.id), "duplicate team facet: {}", facet.id);
        bounded(&facet.when, "facet selection boundary", 400)?;
        ensure!(
            (1..=4).contains(&facet.assignments.len()),
            "facet needs 1..=4 assignments"
        );
        let mut members = BTreeSet::new();
        for assignment in &facet.assignments {
            ensure!(
                roles.iter().any(|role| role.id == assignment.role),
                "unknown canonical team role: {}",
                assignment.role
            );
            ensure!(
                members.insert(&assignment.role),
                "duplicate role in facet: {}",
                assignment.role
            );
            ensure!(
                matches!(assignment.mode.as_str(), "produce" | "review" | "advise"),
                "unknown assignment mode: {}",
                assignment.mode
            );
            bounded(&assignment.contribution, "facet contribution", 400)?;
        }
    }
    Ok(())
}

impl Catalog {
    pub fn team_facets(&self) -> Value {
        json!({"facets":self.0.team_facets,
            "selection":"The host reads the task and affected sources, chooses the primary role and only applicable facets, and supplies a concrete reason for each. Do not classify from keywords alone or select the whole catalog.",
            "limits":{"facets":8,"roles":13},
            "execution":"Planning does not spawn agents or grant tool access."})
    }

    pub fn team_plan(
        &self,
        objective: &str,
        primary: &str,
        facets: &[FacetSelection],
        supporting: &[String],
    ) -> Result<Value> {
        bounded(objective, "objective", 4096)?;
        bounded(primary, "primary role", 128)?;
        ensure!(
            facets.len() <= 8,
            "select at most 8 task facets; split broader work into phases"
        );
        ensure!(
            supporting.len() <= 12,
            "select at most 12 explicit supporting roles"
        );
        let primary = self.capability(primary)?;
        let mut roles = vec![primary];
        let mut reasons: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        let mut modes: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        reasons.insert(primary.id.clone(), vec![json!({"source":"primary","contribution":"Own the user outcome, integration decisions and final verification."})]);
        modes
            .entry(primary.id.clone())
            .or_default()
            .insert("integrate".into());
        let mut selected_facets = BTreeSet::new();
        for selected in facets {
            bounded(&selected.id, "facet id", 128)?;
            bounded(&selected.reason, "facet reason", 400)?;
            ensure!(
                selected_facets.insert(&selected.id),
                "duplicate selected facet: {}",
                selected.id
            );
            let facet = self
                .0
                .team_facets
                .iter()
                .find(|facet| facet.id == selected.id)
                .with_context(|| {
                    format!(
                        "unknown team facet: {}; use team_facets or mirket capabilities facets",
                        selected.id
                    )
                })?;
            for assignment in &facet.assignments {
                let role = self.capability(&assignment.role)?;
                if !roles.iter().any(|existing| existing.id == role.id) {
                    roles.push(role);
                }
                reasons.entry(role.id.clone()).or_default().push(json!({"source":selected.id,"reason":selected.reason,"contribution":assignment.contribution}));
                modes
                    .entry(role.id.clone())
                    .or_default()
                    .insert(assignment.mode.clone());
            }
        }
        for name in supporting {
            bounded(name, "supporting role", 128)?;
            let role = self.capability(name)?;
            if !roles.iter().any(|existing| existing.id == role.id) {
                roles.push(role);
            }
            if !reasons.contains_key(&role.id) {
                reasons.insert(
                    role.id.clone(),
                    vec![json!({"source":"explicit","contribution":role.deliverable})],
                );
                modes
                    .entry(role.id.clone())
                    .or_default()
                    .insert("contribute".into());
            }
        }
        ensure!(
            roles.len() <= 13,
            "team exceeds 13 roles; split the objective into bounded phases instead of dropping specialists"
        );
        let selected: Vec<_> = roles.iter().skip(1).map(|role| role.id.clone()).collect();
        let plan = self.capability_plan(&primary.id, &selected)?;
        let assignments: Vec<_> = roles
            .iter()
            .map(|role| {
                json!({
                    "role":role.id,"title":role.title,"skill":role.lead,
                    "status":"planned","host_agent_id":Value::Null,
                    "modes":modes[&role.id],"selected_because":reasons[&role.id],
                    "professional":role.professional,"deliverable":role.deliverable,
                    "prerequisites":role.prerequisites,"acceptance":role.acceptance,
                    "failure_probe":role.failure_probe
                })
            })
            .collect();
        let mut skills: Vec<_> = roles.iter().map(|role| role.lead.clone()).collect();
        skills.push("agent-coordination".into());
        let acceptance = format!(
            "{} Each assigned professional returns scoped findings or an artifact with evidence and limits. The integrator resolves disagreements against requirements, checks affected consumer boundaries and accounts for every assignment; attach the roster and contribution evidence.",
            primary.acceptance.join("; ")
        );
        let failure_probe = format!(
            "{} Exercise the relevant specialist probes from the attached roster. An unexecuted assignment, self-review presented as independent review, or agreement without evidence must not establish completion.",
            primary.failure_probe
        );
        bounded(&acceptance, "team acceptance", 2048)?;
        bounded(&failure_probe, "team failure probe", 2048)?;
        let mut cli_args = vec![
            "task".to_owned(),
            "start".into(),
            "--capability".into(),
            primary.id.clone(),
            format!("--objective={objective}"),
        ];
        for role in roles.iter().skip(1) {
            cli_args.extend(["--with".into(), role.id.clone()]);
        }
        cli_args.extend([
            "--skill".into(),
            "agent-coordination".into(),
            "--acceptance".into(),
            acceptance.clone(),
            "--failure-probe".into(),
            failure_probe.clone(),
        ]);
        let result = json!({
            "objective":objective,"primary":primary.id,"facets":facets,
            "assignments":assignments,"read_sequence":plan["read_sequence"],
            "project_skills":plan["project_skills"],
            "coordination_read":{"id":"agent-coordination","path":"SKILL.md"},
            "task_start":{"objective":objective,"skills":skills,"acceptance":acceptance,"failure_probe":failure_probe},
            "task_start_cli":{"args":cli_args,"required":"Pass these as argv, adding --project PATH and --key KEY. Save the roster before starting, then bind it as task evidence and record assignments in checkpoints."},
            "execution":{"status":"not_dispatched","owner":"host","dispatch":"Use actual host subagents for independent specialist assignments. The coordinator may own the primary role. Inherit the chosen model/provider, respect available slots, queue later work and assign disjoint writes with explicit dependencies.",
                "fallback":"If independent agents are unavailable, disclose that limit. Separate sequential perspectives may aid work but do not count as independent professionals.",
                "brief":"Give each specialist the user objective, its professional profile, relevant skill body, raw sources, owned paths, allowed effects, dependencies, required output and failure probe. Ask for its own diagnosis and alternatives before showing other specialists' conclusions. No recursive delegation.",
                "return_contract":["Scoped artifact or recommendation and source/evidence paths","Alternatives, tradeoffs and uncertainty that could change the decision","Concrete challenge to an affected peer assumption, or no supported objection","Checks performed, unverified paths and handoff requirements"],
                "integration":"The lead reconciles disagreements by requirement and evidence, preserves unresolved dissent, and verifies the joined result. Do not manufacture disagreement or use a majority vote as acceptance.",
                "tracking":"Before task creation save this roster in the existing task workspace. Record a version 1 contributions receipt with task_evidence contract=contributions: retain its roster binding, cover each role, and include actual host agent IDs, attempts/result attempts, declared input/evidence bindings, dependency roles, validation, unverified paths and integrator disposition. Preview with task_validate; incomplete coverage blocks completion. Use contract=verification for explicit input/checker/result bindings. Record skill delivery for every selected role before completion. The host owns execution; the database does not authenticate workers or judge their findings."},
            "runtime_status":"not_checked","authority":"Role expertise does not grant execution permissions, professional credentials or authority over the user. Planning assigns responsibility; only observed host execution establishes work performed."
        });
        ensure!(
            serde_json::to_vec(&result)?.len() <= 60 * 1024,
            "team brief exceeds response budget; split work into phases"
        );
        Ok(result)
    }
}
