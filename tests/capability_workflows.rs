use mirket::{catalog::Catalog, project};
use std::collections::BTreeSet;

#[test]
fn role_entrypoints_select_production_guidance_without_loading_optional_support() {
    let catalog = Catalog::embedded().unwrap();
    for (role, lead) in [
        ("Illustrator", "graphic-design"),
        ("Visual Asset Designer", "graphic-design"),
        ("Security Engineer", "security-judgment"),
        ("QA Engineer", "test-design"),
        ("AI Engineer", "ai-system-evaluation"),
        ("Data Engineer", "data-engineering"),
        ("Accessibility Engineer", "accessibility-engineering"),
        ("UX Researcher", "customer-research"),
        ("Desktop App Engineer", "desktop-integration"),
        ("MCP Engineer", "mcp-integration"),
        ("Experimentation Analyst", "experiment-design"),
        ("Site Reliability Engineer", "systems-engineering"),
        ("Release Engineer", "release-operations"),
        ("Data Migration Engineer", "data-migrations"),
        ("Presentation Designer", "office-authoring"),
    ] {
        let plan = catalog.capability_plan(role, &[]).unwrap();
        let reads = plan["read_sequence"].as_array().unwrap();
        assert_eq!(reads.len(), 1, "{role}");
        assert_eq!(reads[0]["id"], lead, "{role}");
        assert_eq!(plan["runtime_status"], "not_checked");
        assert_eq!(plan["activation_status"], "not_observed");
    }
    assert!(catalog.capability("a legal certification expert").is_err());
    assert!(catalog.capability("some made-up CFO title").is_err());
}

#[test]
fn consolidated_roles_preserve_legacy_names_and_compose_once() {
    let catalog = Catalog::embedded().unwrap();
    for (canonical, legacy) in [
        (
            "graphic-design",
            vec![
                "illustration",
                "Illustration",
                "Illustrator",
                "visual-assets",
                "Crafting Visual Assets",
                "Visual Asset Designer",
            ],
        ),
        (
            "web-design",
            vec!["design-revision", "Design Revisioning", "Design Revision"],
        ),
        (
            "quality-assurance",
            vec![
                "interface-qa",
                "Interface QA Engineering",
                "QA Engineer for Design",
                "Design QA",
                "Visual QA",
            ],
        ),
    ] {
        let expected = catalog.capability(canonical).unwrap();
        for name in &legacy {
            let plan = catalog.capability_plan(name, &[]).unwrap();
            assert_eq!(plan["primary"], canonical, "{name}");
            assert_eq!(plan["read_sequence"][0]["id"], expected.lead, "{name}");
            let search = catalog.search(name, 0, 1).unwrap();
            assert_eq!(search["items"][0]["id"], expected.lead, "{name}");
        }
        let supporting = legacy
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>();
        let plan = catalog.capability_plan(canonical, &supporting).unwrap();
        assert_eq!(plan["contracts"].as_array().unwrap().len(), 1);
        assert_eq!(plan["read_sequence"].as_array().unwrap().len(), 1);
    }
}

#[test]
fn legacy_and_new_role_selections_install_and_remove_the_canonical_lead() {
    let catalog = Catalog::embedded().unwrap();
    let project_dir = tempfile::tempdir().unwrap();
    let root = project_dir.path().canonicalize().unwrap();
    project::init(&catalog, &root, &["codex".into(), "claude".into()]).unwrap();
    project::add(
        &catalog,
        &root,
        &[],
        &[],
        &[
            "illustration".into(),
            "visual-assets".into(),
            "Data Migration Engineer".into(),
        ],
        None,
    )
    .unwrap();
    project::sync(&catalog, &root, &root.join("cache")).unwrap();
    for host in ["codex", "claude"] {
        let skills = root.join(project::target_skills_dir(host).unwrap());
        for lead in ["graphic-design", "data-migrations"] {
            assert_eq!(
                std::fs::read_to_string(skills.join(lead).join("SKILL.md")).unwrap(),
                catalog.read(lead, "SKILL.md", 64 * 1024).unwrap().content
            );
        }
        assert!(!skills.join("illustration").exists());
        assert!(!skills.join("visual-assets").exists());
    }
    project::remove(&catalog, &root, &[], &[], &["visual-assets".into()], None).unwrap();
    project::sync_with_options(&catalog, &root, &root.join("cache"), true).unwrap();
    assert_eq!(project::doctor(&catalog, &root).unwrap()["ok"], true);
    for host in ["codex", "claude"] {
        let skills = root.join(project::target_skills_dir(host).unwrap());
        assert!(!skills.join("graphic-design").exists());
        assert!(skills.join("data-migrations/SKILL.md").exists());
    }
}

#[test]
fn every_role_delivers_its_exact_lead_offline_to_both_project_targets() {
    let catalog = Catalog::embedded().unwrap();
    let project_dir = tempfile::tempdir().unwrap();
    let root = project_dir.path().canonicalize().unwrap();
    project::init(&catalog, &root, &["codex".into(), "claude".into()]).unwrap();
    let capabilities: Vec<_> = catalog
        .capabilities()
        .iter()
        .map(|role| role.id.clone())
        .collect();
    project::add(&catalog, &root, &[], &[], &capabilities, None).unwrap();
    project::sync(&catalog, &root, &root.join("cache")).unwrap();
    assert_eq!(project::doctor(&catalog, &root).unwrap()["ok"], true);

    let mut leads = BTreeSet::new();
    for role in catalog.capabilities() {
        assert_ne!(
            role.lead, "skill-catalog",
            "a catalog reader is not a production lead"
        );
        for name in std::iter::once(&role.title).chain(&role.aliases) {
            assert_eq!(catalog.capability(name).unwrap().id, role.id);
        }
        if !leads.insert(&role.lead) {
            continue;
        }
        let document = catalog.read(&role.lead, "SKILL.md", 64 * 1024).unwrap();
        assert!(!document.content.is_empty());
        let skill = catalog.skill(&role.lead).unwrap();
        for host in ["codex", "claude"] {
            let path = root
                .join(project::target_skills_dir(host).unwrap())
                .join(&skill.name)
                .join("SKILL.md");
            if catalog.global_skills().contains(&role.lead) {
                assert!(
                    !path.exists(),
                    "normal project selection must not duplicate shared {role:?}"
                );
            } else {
                assert_eq!(std::fs::read_to_string(&path).unwrap(), document.content);
            }
        }
    }
    // Visual production references must reach actual clients, not only the
    // catalog. These files define medium selection and multi-reference critique.
    for reference in ["art-direction-and-critique.md", "image-production.md"] {
        let relative = format!("references/{reference}");
        let document = catalog
            .read("graphic-design", &relative, 16 * 1024)
            .unwrap();
        for host in ["codex", "claude"] {
            let path = root
                .join(project::target_skills_dir(host).unwrap())
                .join("graphic-design")
                .join(&relative);
            assert_eq!(std::fs::read_to_string(path).unwrap(), document.content);
        }
    }
}

#[test]
fn role_plans_keep_workflow_discovery_compact_and_bind_the_selected_role() {
    let catalog = Catalog::embedded().unwrap();
    let plan = catalog.capability_plan("Frontend Engineer", &[]).unwrap();
    assert_eq!(
        plan["worker_request_defaults"]["primary_role"],
        "frontend-engineering"
    );
    assert_eq!(
        plan["worker_request_defaults"]["assigned_tools"],
        serde_json::json!([])
    );
    let index = plan["worker_workflows"].as_array().unwrap();
    assert_eq!(index.len(), catalog.workflows().len());
    assert!(
        index.iter().all(|row| row["id"].is_string()
            && row["when"].is_string()
            && row.get("tools").is_none()
            && row.get("alternative_roles").is_none()),
        "discovery should not deliver every unrelated tool/role policy"
    );
    assert!(
        plan["role_workflows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["id"] == "source-implementation"
                && row["explicit_role_can_replace_default"] == true)
    );
}

#[test]
fn outcome_search_reaches_owned_guidance_and_compact_plan_preserves_role_decisions() {
    let catalog = Catalog::embedded().unwrap();
    for (query, owner, workflow) in [
        ("static graphic", "graphic-design", "static-graphic"),
        ("editable design", "interface-design", "editable-interface"),
        ("diagram", "graphic-design", "diagram"),
        ("video", "motion-design", "motion"),
    ] {
        let result = catalog.search(query, 0, 5).unwrap();
        let item = result["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["id"] == owner)
            .unwrap_or_else(|| panic!("{query}: {result}"));
        assert!(
            item["workflows"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["id"] == workflow),
            "{query}: {item}"
        );
    }
    assert!(
        catalog.search("memory recall", 0, 5).unwrap()["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["id"] == "context-management")
    );
    let plan = catalog.capability_plan("Graphic Designer", &[]).unwrap();
    let rows = plan["role_workflows"].as_array().unwrap();
    for row in rows {
        let id = row["id"].as_str().unwrap();
        let workflow = catalog.workflow(id).unwrap();
        assert_eq!(row["explicit_role_can_replace_default"], true);
        assert_eq!(row.as_object().unwrap().len(), 2);
        assert!(
            plan["worker_workflows"]
                .as_array()
                .unwrap()
                .iter()
                .any(|w| w["id"] == id && w["when"] == workflow.when)
        );
    }
}

#[test]
fn saved_motion_identity_is_retained_without_promising_a_worker_encoder() {
    let catalog = Catalog::embedded().unwrap();
    let workflow = catalog.workflow("motion").unwrap();
    assert_eq!(
        workflow.execution_owner,
        mirket::catalog::ExecutionOwner::Host
    );
    assert!(workflow.tools.is_empty());
    assert!(workflow.required_tools.is_empty());
    assert!(workflow.rule.contains("no video encoder"));
    assert_eq!(
        catalog
            .workflow("interactive-motion")
            .unwrap()
            .execution_owner,
        mirket::catalog::ExecutionOwner::Worker
    );
}
