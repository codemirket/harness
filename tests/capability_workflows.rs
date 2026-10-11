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
}
