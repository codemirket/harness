use mirket::{
    catalog::{Catalog, DesignContext, read_payload_directory},
    paths::Paths,
    project,
    util::sha256,
};
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParams, ReadResourceRequestParams},
    transport::TokioChildProcess,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

const OWNER: &str = "design-inspiration";
const MANIFEST: &str = "references/design-languages/library.json";
const IDS: [&str; 27] = [
    "govuk-procedural",
    "carbon-operations",
    "vanilla-publishing",
    "porsche-restraint",
    "guardian-editorial",
    "wise-expressive-utility",
    "geist-code",
    "material-responsive-shape",
    "fluent-spatial",
    "apple-liquid-glass",
    "duolingo-performance",
    "ibm-hybrid-illustration",
    "mit-generative-identity",
    "casa-subject-identity",
    "peeps-humanist",
    "fluent-dimensional-symbols",
    "pudding-subject-story",
    "ons-statistical",
    "ft-relationship-notation",
    "saleor-commerce",
    "atlassian-collaboration",
    "fiori-complex-workspace",
    "penpot-direct-manipulation",
    "vega-linked-analysis",
    "android-adaptive-continuity",
    "spectrum-bidirectional",
    "aria-alternative-input",
];

fn manifest() -> Value {
    let catalog = Catalog::embedded().unwrap();
    let mut index: Value =
        serde_json::from_str(&catalog.read(OWNER, MANIFEST, 48 * 1024).unwrap().content).unwrap();
    assert_eq!(index["schema_version"], 2);
    // The bounded index has no duplicate source bodies. Resolve canonical records
    // through the same delivery API used by clients.
    for instance in index["instances"].as_array_mut().unwrap() {
        let document = catalog
            .read(OWNER, instance["record"].as_str().unwrap(), 48 * 1024)
            .unwrap();
        let record: Value = serde_json::from_str(&document.content).unwrap();
        assert_eq!(instance["id"], record["id"]);
        let path = instance["record"].clone();
        *instance = record;
        instance["record"] = path;
    }
    index
}

#[test]
fn all_twenty_seven_instances_are_bounded_readable_and_discoverable_individually() {
    let catalog = Catalog::embedded().unwrap();
    let library = manifest();
    let instances = library["instances"].as_array().unwrap();
    let ids: BTreeSet<_> = instances
        .iter()
        .map(|instance| instance["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, IDS.into_iter().collect());
    assert_eq!(
        catalog.check().unwrap()["design_reference_instances"],
        IDS.len()
    );
    for instance in instances {
        let id = instance["id"].as_str().unwrap();
        for key in ["record", "card", "example"] {
            let path = instance[key].as_str().unwrap();
            let document = catalog.read(OWNER, path, 48 * 1024).unwrap();
            assert_eq!(document.sha256, sha256(document.content.as_bytes()));
            assert_eq!(
                document.content.as_bytes(),
                fs::read(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("skills")
                        .join(OWNER)
                        .join(path)
                )
                .unwrap(),
                "saved source and embedded delivery differ: {id}/{key}"
            );
            assert!(
                key == "record" || document.content.contains("misleading"),
                "contrast missing: {id}/{key}"
            );
            assert!(
                catalog
                    .read(OWNER, path, document.content.len() - 1)
                    .is_err()
            );
        }
        let search = catalog.search(id, 0, 1).unwrap();
        assert_eq!(search["items"][0]["id"], OWNER, "{id}");
        assert_eq!(
            search["items"][0]["reference_matches"]["items"][0]["id"],
            id
        );
        assert_eq!(
            search["items"][0]["reference_matches"]["items"][0]["lesson"]["path"],
            instance["card"]
        );
    }
    for (query, expected) in [
        ("Porsche", "porsche-restraint"),
        ("statistical uncertainty", "ons-statistical"),
        ("Liquid Glass", "apple-liquid-glass"),
        ("Visual Vocabulary", "ft-relationship-notation"),
    ] {
        let results = catalog.search(query, 0, 10).unwrap();
        let owner = results["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["id"] == OWNER)
            .unwrap();
        assert_eq!(owner["reference_matches"]["items"][0]["id"], expected);
    }
    // Terms may not match by accidentally pooling two unrelated references.
    assert_eq!(catalog.search("Casa Wise", 0, 100).unwrap()["total"], 0);
    assert_eq!(
        catalog
            .search("definitely-unrelated-9f817", 0, 100)
            .unwrap()["total"],
        0
    );
    let generic = catalog.search("design", 0, 100).unwrap();
    let owner = generic["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == OWNER)
        .unwrap();
    assert!(owner["reference_matches"]["shown"].as_u64().unwrap() <= 5);
    assert!(serde_json::to_vec(owner).unwrap().len() < 12 * 1024);
    assert_eq!(
        catalog.describe(OWNER).unwrap()["reference_library"]["instances"],
        IDS.len()
    );
}

#[test]
fn structured_coverage_keeps_the_combination_and_limits_visible() {
    let catalog = Catalog::embedded().unwrap();
    let context = DesignContext {
        family: Some("commerce".into()),
        platform: Some("web".into()),
        interactions: vec!["cart-checkout".into()],
        constraints: vec!["keyboard".into()],
    };
    let search = catalog
        .search_with_design_context("definitely-unrelated-9f817", 0, 1, Some(&context))
        .unwrap();
    assert_eq!(search["total"], 0);
    assert_eq!(search["design_coverage"]["status"], "represented");
    assert_eq!(
        search["design_coverage"]["candidates"][0]["id"],
        "saleor-commerce"
    );
    assert_eq!(search["design_coverage"]["current_task_validated"], false);
    let context = DesignContext {
        family: Some("collaboration".into()),
        platform: Some("native-mobile".into()),
        interactions: vec!["offline-sync".into()],
        constraints: vec!["rtl".into()],
    };
    let search = catalog
        .search_with_design_context("", 0, 1, Some(&context))
        .unwrap();
    assert_eq!(search["design_coverage"]["status"], "partial");
    assert_eq!(search["design_coverage"]["combination_represented"], false);
    let native = DesignContext {
        family: Some("complex-workspaces".into()),
        platform: Some("native-mobile".into()),
        interactions: vec!["adaptive-navigation".into()],
        ..Default::default()
    };
    let search = catalog
        .search_with_design_context("", 0, 1, Some(&native))
        .unwrap();
    assert_eq!(search["design_coverage"]["status"], "partial");
    assert_eq!(
        search["design_coverage"]["candidates"][0]["level"],
        "source-only"
    );
    assert_eq!(
        search["design_coverage"]["candidates"][0]["missing"]["worked_example"],
        true
    );
}

#[test]
fn broad_reference_queries_offer_provisional_candidates_without_changing_search_or_coverage() {
    let catalog = Catalog::embedded().unwrap();
    let query =
        "An independent cultural programme needs subject-derived identity plus editorial hierarchy";
    // General catalog search keeps its existing all-terms contract. The design
    // route can offer a bounded way out of an otherwise empty reference lookup.
    assert_eq!(catalog.search(query, 0, 3).unwrap()["total"], 0);
    let result = catalog
        .search_with_design_context(query, 0, 3, None)
        .unwrap();
    assert_eq!(result["total"], 0);
    let candidates = &result["design_reference_candidates"];
    let items = candidates["items"].as_array().unwrap();
    assert!(items.len() <= 5);
    for id in ["casa-subject-identity", "guardian-editorial"] {
        assert!(items.iter().any(|item| item["id"] == id), "{id}");
    }
    assert_eq!(candidates["fit_validated"], false);
    assert_eq!(candidates["coverage_claimed"], false);
    assert!(items.iter().all(|item| {
        !item["matched_terms"].as_array().unwrap().is_empty()
            && !item["unmatched_terms"].as_array().unwrap().is_empty()
            && item["use_when"].is_string()
            && item["avoid_when"].is_string()
    }));
    assert!(serde_json::to_vec(candidates).unwrap().len() <= 24 * 1024);

    let absent = DesignContext {
        family: Some("device-specialist".into()),
        platform: Some("tv".into()),
        interactions: vec!["remote-navigation".into()],
        ..Default::default()
    };
    let result = catalog
        .search_with_design_context(query, 0, 3, Some(&absent))
        .unwrap();
    assert_eq!(result["design_coverage"]["status"], "unrepresented");
    assert_eq!(result["design_coverage"]["combination_represented"], false);
    assert_eq!(
        result["design_reference_candidates"]["coverage_claimed"],
        false
    );

    for query in [
        "opaque zephyr orchid torque",
        "avoid cultural programme subject identity",
        "No cultural programme subject identity please",
        "Don't use cultural programme subject identity",
        "Don’t use cultural programme subject identity",
        "excluding cultural programme subject identity",
    ] {
        assert!(
            catalog
                .search_with_design_context(query, 0, 3, None)
                .unwrap()
                .get("design_reference_candidates")
                .is_none(),
            "{query}"
        );
    }
    assert!(
        catalog
            .search_with_design_context("casa-subject-identity", 0, 3, None)
            .unwrap()
            .get("design_reference_candidates")
            .is_none()
    );
}

struct Installed {
    _temporary: tempfile::TempDir,
    paths: Paths,
    project: PathBuf,
}
impl Installed {
    fn new() -> Self {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().canonicalize().unwrap();
        let home = root.join("home");
        let project = root.join("project");
        fs::create_dir(&home).unwrap();
        fs::create_dir(&project).unwrap();
        let paths = Paths::new(home).unwrap();
        let this = Self {
            _temporary: temporary,
            paths,
            project,
        };
        this.run_at(
            Path::new(env!("CARGO_BIN_EXE_mirket")),
            &["setup", "--target", "all", "--yes"],
        );
        let project = this.project.to_str().unwrap();
        this.run(&["project", "init", "--project", project, "--target", "all"]);
        this.run(&["project", "add", "--project", project, "--skill", OWNER]);
        this.run(&["project", "sync", "--project", project]);
        this
    }
    fn run_at(&self, binary: &Path, args: &[&str]) -> Value {
        let output = Command::new(binary)
            .arg("--home")
            .arg(&self.paths.home)
            .arg("--json")
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
    fn run(&self, args: &[&str]) -> Value {
        self.run_at(&self.paths.binary(), args)
    }
}

#[test]
fn complete_library_reaches_both_hosts_through_existing_project_delivery() {
    let installed = Installed::new();
    let payload = Catalog::embedded().unwrap().payload(OWNER).unwrap();
    for target in ["codex", "claude"] {
        let copy = installed
            .project
            .join(project::target_skills_dir(target).unwrap())
            .join(OWNER);
        assert_eq!(
            read_payload_directory(&copy, &[".mirket-skill.json"]).unwrap(),
            payload.files,
            "{target}"
        );
    }
    let search = installed.run(&["catalog", "search", "Porsche", "--limit", "1"]);
    let show = installed.run(&["catalog", "show", OWNER]);
    assert_eq!(show["reference_library"]["instances"], IDS.len());
    assert_eq!(
        show["reference_library"]["recovery_practice"]["guide"]["path"],
        "references/design-languages/recovery-feedback.md"
    );
    assert_eq!(
        show["reference_library"]["comparison_practice"]["guide"]["path"],
        "references/design-languages/comparison-at-use-size.md"
    );
    assert_eq!(
        show["reference_library"]["comparison_practice"]["example"]["path"],
        "assets/design-languages/comparison-at-use-size/index.html"
    );
    let broad = installed.run(&[
        "catalog",
        "search",
        "A cultural programme needs subject-derived identity plus editorial hierarchy",
        "--limit",
        "3",
    ]);
    assert_eq!(broad["total"], 0);
    assert_eq!(broad["design_reference_candidates"]["fit_validated"], false);
    assert!(
        broad["design_reference_candidates"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == "casa-subject-identity")
    );
    let comparison = installed.run(&[
        "catalog",
        "search",
        "linked brushing analysis envelopes",
        "--limit",
        "3",
    ]);
    let vega = comparison["design_reference_candidates"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == "vega-linked-analysis")
        .unwrap();
    assert!(vega.get("comparison_practice").is_none());
    let practice = show["reference_library"]["comparison_practice"]["guide"]["path"]
        .as_str()
        .unwrap();
    assert_eq!(
        practice,
        "references/design-languages/comparison-at-use-size.md"
    );
    assert_eq!(
        installed.run(&["catalog", "read", OWNER, "--path", practice])["content"]
            .as_str()
            .unwrap()
            .as_bytes(),
        payload.files[practice]
    );
    let exact = installed.run(&["catalog", "search", "Vega", "--limit", "1"]);
    assert_eq!(
        exact["items"][0]["reference_matches"]["items"][0]["id"],
        "vega-linked-analysis"
    );
    assert!(
        exact["items"][0]["reference_matches"]["items"][0]
            .get("comparison_practice")
            .is_none()
    );
    assert_eq!(
        show["reference_library"]["coverage"]["path"],
        "references/design-languages/coverage.json"
    );
    let path = search["items"][0]["reference_matches"]["items"][0]["lesson"]["path"]
        .as_str()
        .unwrap();
    assert_eq!(
        installed.run(&["catalog", "read", OWNER, "--path", path])["content"]
            .as_str()
            .unwrap()
            .as_bytes(),
        payload.files[path]
    );
    let coverage = installed.run(&[
        "catalog",
        "search",
        "--family",
        "commerce",
        "--platform",
        "web",
        "--interaction",
        "cart-checkout",
        "--constraint",
        "keyboard",
        "--limit",
        "1",
    ]);
    assert_eq!(coverage["design_coverage"]["status"], "represented");
    assert_eq!(
        coverage["design_coverage"]["candidates"][0]["id"],
        "saleor-commerce"
    );
    let coverage = installed.run(&[
        "catalog",
        "search",
        "--family",
        "collaboration",
        "--platform",
        "native-mobile",
        "--interaction",
        "offline-sync",
        "--constraint",
        "rtl",
        "--limit",
        "1",
    ]);
    assert_eq!(
        coverage["design_coverage"]["combination_represented"],
        false
    );
    assert_eq!(
        installed.run(&[
            "project",
            "doctor",
            "--project",
            installed.project.to_str().unwrap()
        ])["ok"],
        true
    );
    assert_eq!(installed.run(&["doctor"])["ok"], true);
}

#[tokio::test]
async fn installed_mcp_discovers_specific_lesson_and_delivers_every_example() {
    let installed = Installed::new();
    let mut command = tokio::process::Command::new(installed.paths.binary());
    command
        .arg("--home")
        .arg(&installed.paths.home)
        .args(["mcp", "serve"])
        .kill_on_drop(true);
    let client = tokio::time::timeout(
        Duration::from_secs(15),
        ().serve(TokioChildProcess::new(command).unwrap()),
    )
    .await
    .unwrap()
    .unwrap();
    let operation = async {
        let search = client
            .call_tool(
                CallToolRequestParams::new("catalog_search").with_arguments(
                    json!({"query":"Porsche", "limit":1})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
            .await
            .unwrap();
        let search = search.structured_content.unwrap();
        let matched = &search["items"][0]["reference_matches"]["items"][0];
        assert_eq!(matched["id"], "porsche-restraint");
        let route = &matched["lesson"];
        let read = client
            .call_tool(
                CallToolRequestParams::new("skill_read").with_arguments(
                    json!({"skill_id":route["skill_id"],"path":route["path"]})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
            .await
            .unwrap();
        assert_ne!(read.is_error, Some(true));
        let manifest_read = client
            .call_tool(
                CallToolRequestParams::new("skill_read").with_arguments(
                    json!({"skill_id":OWNER,"path":MANIFEST})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
            .await
            .unwrap();
        assert_ne!(
            manifest_read.is_error,
            Some(true),
            "the complete provenance index must fit the real protocol result budget"
        );
        assert_eq!(
            serde_json::from_str::<Value>(
                manifest_read.structured_content.unwrap()["content"]
                    .as_str()
                    .unwrap()
            )
            .unwrap()["instances"]
                .as_array()
                .unwrap()
                .len(),
            IDS.len()
        );
        for path in [
            "references/design-languages/coverage.json",
            "references/design-languages/coverage.md",
            "references/design-languages/recovery-feedback.md",
            "assets/design-languages/recovery-feedback/index.html",
            "references/design-languages/comparison-at-use-size.md",
            "assets/design-languages/comparison-at-use-size/index.html",
        ] {
            let read = client
                .call_tool(
                    CallToolRequestParams::new("skill_read").with_arguments(
                        json!({"skill_id":OWNER,"path":path})
                            .as_object()
                            .unwrap()
                            .clone(),
                    ),
                )
                .await
                .unwrap();
            assert_ne!(read.is_error, Some(true), "{path}");
            let content = read.structured_content.unwrap();
            let expected = fs::read(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("skills")
                    .join(OWNER)
                    .join(path),
            )
            .unwrap();
            assert_eq!(content["content"].as_str().unwrap().as_bytes(), expected);
            let resource = client
                .read_resource(ReadResourceRequestParams::new(format!(
                    "mirket://skills/{OWNER}/{path}"
                )))
                .await
                .unwrap();
            let value = serde_json::to_value(resource).unwrap();
            assert_eq!(
                value["contents"][0]["text"].as_str().unwrap().as_bytes(),
                expected
            );
        }
        let broad = client.call_tool(CallToolRequestParams::new("catalog_search").with_arguments(json!({"query":"A cultural programme needs subject-derived identity plus editorial hierarchy","limit":3}).as_object().unwrap().clone())).await.unwrap();
        assert_ne!(broad.is_error, Some(true));
        let broad = broad.structured_content.unwrap();
        assert_eq!(broad["total"], 0);
        assert_eq!(
            broad["design_reference_candidates"]["coverage_claimed"],
            false
        );
        assert!(
            broad["design_reference_candidates"]["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["id"] == "casa-subject-identity")
        );
        let coverage = client.call_tool(CallToolRequestParams::new("catalog_search").with_arguments(json!({"query":"","limit":1,"design_context":{"family":"collaboration","platform":"native-mobile","interactions":["offline-sync"],"constraints":["rtl"]}}).as_object().unwrap().clone())).await.unwrap();
        assert_ne!(coverage.is_error, Some(true));
        let coverage = coverage.structured_content.unwrap();
        assert_eq!(coverage["design_coverage"]["status"], "partial");
        assert_eq!(
            coverage["design_coverage"]["combination_represented"],
            false
        );
        assert_eq!(coverage["design_coverage"]["current_task_validated"], false);
        let invalid = client
            .call_tool(
                CallToolRequestParams::new("catalog_search").with_arguments(
                    json!({"query":"","design_context":{"platform":"made-up-native"}})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
            .await
            .unwrap();
        assert_eq!(invalid.is_error, Some(true));
        let payload = Catalog::embedded().unwrap().payload(OWNER).unwrap();
        for instance in manifest()["instances"].as_array().unwrap() {
            for key in ["record", "card", "example"] {
                let path = instance[key].as_str().unwrap();
                let uri = format!("mirket://skills/{OWNER}/{path}");
                let response = client
                    .read_resource(ReadResourceRequestParams::new(uri))
                    .await
                    .unwrap();
                let value = serde_json::to_value(response).unwrap();
                assert_eq!(
                    value["contents"][0]["text"].as_str().unwrap().as_bytes(),
                    payload.files[path]
                );
                assert_eq!(
                    value["contents"][0]["mimeType"],
                    match key {
                        "card" => "text/markdown",
                        "record" => "application/json",
                        _ => "text/html",
                    }
                );
            }
        }
        assert!(client.read_resource(ReadResourceRequestParams::new("mirket://skills/design-inspiration/references/design-languages/../../../../Cargo.toml")).await.is_err());
    };
    let result = tokio::time::timeout(Duration::from_secs(45), operation).await;
    tokio::time::timeout(Duration::from_secs(5), client.cancel())
        .await
        .unwrap()
        .unwrap();
    result.unwrap();
}
