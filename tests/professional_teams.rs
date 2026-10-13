use mirket::catalog::{Catalog, FacetSelection};
use serde_json::Value;
use std::{collections::BTreeSet, path::Path, process::Command};

fn facet(id: &str, reason: &str) -> FacetSelection {
    FacetSelection {
        id: id.into(),
        reason: reason.into(),
    }
}

#[test]
fn checkout_assigns_complementary_professionals_and_preserves_shared_contributions() {
    let catalog = Catalog::embedded().unwrap();
    let plan = catalog
        .team_plan(
            "Build a recoverable checkout flow",
            "Frontend Engineer",
            &[
                facet("web-interface", "Checkout screen and browser state change"),
                facet("service-contract", "Order endpoint reconciles retries"),
                facet(
                    "security-boundary",
                    "Orders are scoped to authenticated customers",
                ),
            ],
            &["Design QA".into(), "Frontend Engineer".into()],
        )
        .unwrap();
    let assignments = plan["assignments"].as_array().unwrap();
    let roles: BTreeSet<_> = assignments
        .iter()
        .map(|a| a["role"].as_str().unwrap())
        .collect();
    assert_eq!(
        roles,
        BTreeSet::from([
            "frontend-engineering",
            "web-design",
            "backend-engineering",
            "quality-assurance",
            "security-engineering"
        ])
    );
    assert_eq!(assignments.len(), roles.len());
    let qa = assignments
        .iter()
        .find(|a| a["role"] == "quality-assurance")
        .unwrap();
    assert_eq!(qa["selected_because"].as_array().unwrap().len(), 2);
    assert_eq!(plan["execution"]["status"], "not_dispatched");
    for assignment in assignments {
        let role = catalog
            .capability(assignment["role"].as_str().unwrap())
            .unwrap();
        assert_eq!(
            assignment["professional"],
            serde_json::to_value(&role.professional).unwrap()
        );
        assert_eq!(assignment["skill"], role.lead);
        assert_eq!(assignment["status"], "planned");
        assert!(assignment["host_agent_id"].is_null());
        assert!(
            plan["read_sequence"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["id"] == role.lead)
        );
    }
    assert!(
        plan["task_start"]["skills"]
            .as_array()
            .unwrap()
            .iter()
            .any(|id| id == "agent-coordination")
    );
    assert!(serde_json::to_vec(&plan).unwrap().len() < 60 * 1024);
}

#[test]
fn team_selection_is_bounded_explicit_and_does_not_expand_support_recursively() {
    let catalog = Catalog::embedded().unwrap();
    let solo = catalog
        .team_plan("Review the supplied cash assumption", "CFO", &[], &[])
        .unwrap();
    assert_eq!(solo["assignments"].as_array().unwrap().len(), 1);
    for facets in [
        vec![facet("made-up-boundary", "Requested")],
        vec![facet("web-interface", " ")],
        vec![facet("web-interface", "x"), facet("web-interface", "y")],
        vec![facet("web-interface", "x"); 9],
        [
            "web-interface",
            "mobile-journey",
            "desktop-application",
            "service-contract",
            "data-pipeline",
            "brand-identity",
            "office-artifact",
            "venture-validation",
        ]
        .iter()
        .map(|id| facet(id, "Part of the requested outcome"))
        .collect(),
    ] {
        assert!(
            catalog
                .team_plan("Broad work", "CTO", &facets, &[])
                .is_err()
        );
    }
    assert!(catalog.team_plan("", "CFO", &[], &[]).is_err());
    assert!(
        catalog
            .team_plan("x", "CFO", &[], &["test-design".into()])
            .is_err()
    );
    let facets = catalog.team_facets();
    let mut covered = BTreeSet::new();
    for selected in facets["facets"].as_array().unwrap() {
        let id = selected["id"].as_str().unwrap();
        let assignments = selected["assignments"].as_array().unwrap();
        let primary = assignments[0]["role"].as_str().unwrap();
        let plan = catalog
            .team_plan(
                "Exercise one declared boundary",
                primary,
                &[facet(id, "Observed task boundary")],
                &[],
            )
            .unwrap();
        assert_eq!(
            plan["assignments"].as_array().unwrap().len(),
            assignments.len()
        );
        for row in assignments {
            covered.insert(row["role"].as_str().unwrap());
        }
    }
    assert_eq!(
        covered,
        catalog
            .capabilities()
            .iter()
            .map(|r| r.id.as_str())
            .collect()
    );
}

fn invoke(home: &Path, args: &[String]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_mirket"))
        .arg("--home")
        .arg(home)
        .args(args)
        .output()
        .unwrap()
}

fn ok(home: &Path, args: &[String]) -> Value {
    let result = invoke(home, args);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&result.stdout).unwrap()
}

#[test]
fn generated_cli_arguments_create_replayable_tasks_and_bind_the_saved_roster() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let root = project.path().to_string_lossy().into_owned();
    ok(
        home.path(),
        &[
            "project".into(),
            "register".into(),
            "--project".into(),
            root.clone(),
        ],
    );
    // Leading options and shell-looking data must remain a literal objective.
    let objective = "--help must document a model labeled $(unchanged) with a quoted 'name'";
    let plan = ok(
        home.path(),
        &[
            "capabilities".into(),
            "team".into(),
            "CFO".into(),
            format!("--objective={objective}"),
            "--facet".into(),
            "workbook=The deliverable is an editable forecast workbook".into(),
        ],
    );
    let bytes = serde_json::to_vec_pretty(&plan).unwrap();
    std::fs::write(project.path().join("roster.json"), &bytes).unwrap();
    let mut args: Vec<String> =
        serde_json::from_value(plan["task_start_cli"]["args"].clone()).unwrap();
    args.extend([
        "--project".into(),
        root,
        "--key".into(),
        "team-start".into(),
    ]);
    let task = ok(home.path(), &args);
    assert_eq!(
        ok(
            home.path(),
            &[
                "task".into(),
                "status".into(),
                task["id"].as_str().unwrap().into()
            ]
        )["objective"],
        objective
    );
    assert_eq!(task["result"]["skills"], plan["task_start"]["skills"]);
    assert_eq!(ok(home.path(), &args), task);
    assert!(
        task.get("team_plan").is_none(),
        "task replay must not append a newly generated roster"
    );
    let id = task["id"].as_str().unwrap().to_owned();
    let mut revision = task["revision"].as_i64().unwrap();
    let bound = ok(
        home.path(),
        &[
            "task".into(),
            "evidence".into(),
            id.clone(),
            "roster.json".into(),
            "--revision".into(),
            revision.to_string(),
            "--key".into(),
            "roster".into(),
            "--kind".into(),
            "acceptance".into(),
            "--summary".into(),
            "Recorded planned roles; no worker execution claimed".into(),
        ],
    );
    revision = bound["revision"].as_i64().unwrap();
    let recovered = ok(home.path(), &["task".into(), "status".into(), id.clone()]);
    assert_eq!(recovered["evidence"][0]["path"], "roster.json");
    // Every selected body, including coordination, must be delivered.
    for skill in task["result"]["skills"].as_array().unwrap() {
        let skill = skill.as_str().unwrap();
        let result = ok(
            home.path(),
            &[
                "task".into(),
                "read".into(),
                id.clone(),
                skill.into(),
                "--revision".into(),
                revision.to_string(),
                "--key".into(),
                format!("read-{skill}"),
            ],
        );
        revision = result["receipt"]["revision"].as_i64().unwrap();
    }
    std::fs::write(
        project.path().join("probe.txt"),
        "Fixture verified roster membership, task replay and artifact mutation refusal.",
    )
    .unwrap();
    let result = ok(
        home.path(),
        &[
            "task".into(),
            "evidence".into(),
            id.clone(),
            "probe.txt".into(),
            "--revision".into(),
            revision.to_string(),
            "--key".into(),
            "probe".into(),
            "--kind".into(),
            "failure-probe".into(),
            "--summary".into(),
            "Task lifecycle fixture only; no financial outcome attested".into(),
        ],
    );
    revision = result["revision"].as_i64().unwrap();
    let finish = vec![
        "task".into(),
        "complete".into(),
        id,
        "--revision".into(),
        revision.to_string(),
        "--key".into(),
        "complete".into(),
    ];
    std::fs::write(project.path().join("roster.json"), b"changed assignment").unwrap();
    let refused = invoke(home.path(), &finish);
    assert!(!refused.status.success());
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains("evidence is stale: roster.json"),
        "{}",
        String::from_utf8_lossy(&refused.stderr)
    );
    std::fs::write(project.path().join("roster.json"), bytes).unwrap();
    assert_eq!(ok(home.path(), &finish)["status"], "complete");
}
