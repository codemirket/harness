use mirket::{
    catalog::{Catalog, MAX_FILE, read_payload_directory},
    paths::Paths,
    project,
    util::sha256,
};
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParams, ReadResourceRequestParams, ResourceContents},
    transport::TokioChildProcess,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::Duration,
};

const SKILL: &str = "advanced-motion";
const RECEIPT: &str = ".mirket-skill.json";

fn example_sources() -> BTreeMap<String, Vec<u8>> {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("skills/advanced-motion/assets");
    let mut files = BTreeMap::new();
    for example in ["solstice", "signal", "hyperframes"] {
        let local = read_payload_directory(&source.join(example), &[]).unwrap();
        assert!(
            local.contains_key(if example == "hyperframes" {
                "scene.html"
            } else {
                "index.html"
            }),
            "{example} has no entry page"
        );
        files.extend(
            local
                .into_iter()
                .map(|(name, bytes)| (format!("assets/{example}/{name}"), bytes)),
        );
    }
    files
}

fn invoke(binary: &Path, home: &Path, args: &[&str]) -> Output {
    Command::new(binary)
        .arg("--home")
        .arg(home)
        .arg("--json")
        .args(args)
        .output()
        .unwrap()
}

fn successful(binary: &Path, home: &Path, args: &[&str]) -> Value {
    let output = invoke(binary, home, args);
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stdout)))
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
        successful(
            Path::new(env!("CARGO_BIN_EXE_mirket")),
            &paths.home,
            &["setup", "--target", "all", "--yes"],
        );
        Self {
            _temporary: temporary,
            paths,
            project,
        }
    }

    fn run(&self, args: &[&str]) -> Value {
        successful(&self.paths.binary(), &self.paths.home, args)
    }

    fn select_examples(&self) {
        let root = self.project.to_str().unwrap();
        self.run(&["project", "init", "--project", root, "--target", "all"]);
        self.run(&["project", "add", "--project", root, "--skill", SKILL]);
        self.run(&["project", "sync", "--project", root]);
    }
}

#[test]
fn motion_payload_embeds_complete_sources_with_bounded_text_reads() {
    let catalog = Catalog::embedded().unwrap();
    let payload = catalog.payload(SKILL).unwrap();
    let sources = example_sources();
    let embedded: BTreeMap<_, _> = payload
        .files
        .iter()
        .filter(|(path, _)| {
            path.starts_with("assets/solstice/")
                || path.starts_with("assets/signal/")
                || path.starts_with("assets/hyperframes/")
        })
        .map(|(path, bytes)| (path.clone(), bytes.clone()))
        .collect();
    assert_eq!(
        embedded, sources,
        "each standalone page needs its complete local assets"
    );

    for (path, bytes) in &sources {
        assert!(!bytes.is_empty(), "empty example resource: {path}");
        // The actual MCP read boundary is 48 KiB, below the general catalog limit.
        let document = catalog.read(SKILL, path, 48 * 1024).unwrap();
        assert_eq!(document.content.as_bytes(), bytes, "{path}");
        assert_eq!(document.sha256, sha256(bytes), "{path}");
        assert!(catalog.read(SKILL, path, bytes.len() - 1).is_err());
    }
    let svg = std::str::from_utf8(&sources["assets/solstice/solar-instrument.svg"]).unwrap();
    let vector = roxmltree::Document::parse(svg).unwrap();
    assert_eq!(vector.root_element().tag_name().name(), "svg");
    assert_eq!(
        vector.root_element().tag_name().namespace(),
        Some("http://www.w3.org/2000/svg")
    );
    assert!(vector.root_element().attribute("viewBox").is_some());

    for path in [
        "../SKILL.md",
        "assets/solstice/../../../../Cargo.toml",
        "/assets/solstice/index.html",
        "assets\\solstice\\index.html",
        "assets/solstice/missing.html",
    ] {
        assert!(catalog.read(SKILL, path, 48 * 1024).is_err(), "{path}");
    }
    for limit in [0, MAX_FILE + 1] {
        assert!(
            catalog
                .read(SKILL, "assets/solstice/index.html", limit)
                .is_err()
        );
    }
}

#[test]
fn motion_payload_installed_cli_preserves_both_target_assets_and_refuses_drift() {
    let installed = Installed::new();
    let instructions = installed.project.join("AGENTS.md");
    fs::write(&instructions, "Keep this project's own design decisions.\n").unwrap();
    let personal = installed.project.join(".agents/skills/personal/notes.txt");
    fs::create_dir_all(personal.parent().unwrap()).unwrap();
    fs::write(&personal, "Unmanaged notes stay unchanged.\n").unwrap();
    installed.select_examples();
    let root = installed.project.to_str().unwrap();
    let catalog = Catalog::embedded().unwrap();
    let payload = catalog.payload(SKILL).unwrap();
    let mut copies = BTreeMap::new();
    for target in ["codex", "claude"] {
        let copy = installed
            .project
            .join(project::target_skills_dir(target).unwrap())
            .join(SKILL);
        assert_eq!(
            read_payload_directory(&copy, &[RECEIPT]).unwrap(),
            payload.files,
            "{target} must receive every resource, not only Markdown"
        );
        let receipt: Value =
            serde_json::from_slice(&fs::read(copy.join(RECEIPT)).unwrap()).unwrap();
        assert_eq!(receipt["skill"], SKILL);
        assert_eq!(receipt["sha256"], payload.sha256);
        assert_eq!(receipt["executable_files"], json!(payload.executables));
        copies.insert(target, copy);
    }
    for (path, bytes) in example_sources() {
        let read = installed.run(&["catalog", "read", SKILL, "--path", &path]);
        assert_eq!(read["content"].as_str().unwrap().as_bytes(), bytes);
        assert_eq!(read["sha256"], sha256(&bytes));
    }
    let current = installed.run(&["project", "sync", "--project", root]);
    assert!(
        current["skills"]
            .as_array()
            .unwrap()
            .iter()
            .all(|skill| skill["status"] == "current")
    );
    assert_eq!(
        installed.run(&["project", "doctor", "--project", root])["ok"],
        true
    );
    assert_eq!(installed.run(&["doctor"])["ok"], true);
    assert_eq!(
        fs::read_to_string(&instructions).unwrap(),
        "Keep this project's own design decisions.\n"
    );
    assert_eq!(
        fs::read_to_string(&personal).unwrap(),
        "Unmanaged notes stay unchanged.\n"
    );

    // Codex needs a write, while the later Claude target has a modified asset.
    // Reconciliation must detect all conflicts before repairing the first target.
    fs::remove_dir_all(&copies["codex"]).unwrap();
    let changed = copies["claude"].join("assets/solstice/styles.css");
    let mut bytes = fs::read(&changed).unwrap();
    bytes.extend_from_slice(b"\n/* local author refinement */\n");
    fs::write(&changed, bytes).unwrap();
    let before = read_payload_directory(&installed.project, &[]).unwrap();
    let failure = invoke(
        &installed.paths.binary(),
        &installed.paths.home,
        &["project", "sync", "--project", root],
    );
    assert!(!failure.status.success());
    assert!(String::from_utf8_lossy(&failure.stderr).contains("managed skill was modified"));
    assert_eq!(
        read_payload_directory(&installed.project, &[]).unwrap(),
        before
    );
    assert!(!copies["codex"].exists());
}

#[tokio::test]
async fn motion_payload_installed_mcp_delivers_typed_sources_and_rejects_escaped_resources() {
    let installed = Installed::new();
    let mut command = tokio::process::Command::new(installed.paths.binary());
    command
        .arg("--home")
        .arg(&installed.paths.home)
        .args(["mcp", "serve"])
        .kill_on_drop(true);
    let transport = TokioChildProcess::new(command).unwrap();
    let client = tokio::time::timeout(Duration::from_secs(15), ().serve(transport))
        .await
        .unwrap()
        .unwrap();
    let operation = async {
        let tools = client.list_all_tools().await.unwrap();
        assert!(tools.iter().any(|tool| tool.name == "skill_read"));
        let templates = client.list_all_resource_templates().await.unwrap();
        let template = templates
            .iter()
            .find(|template| template.uri_template == "mirket://skills/{skill_id}/{+path}")
            .expect("embedded resource template is discoverable");
        assert!(
            template.mime_type.is_none(),
            "mixed source formats need per-resource MIME types"
        );

        let mut files = example_sources();
        files.insert(
            "SKILL.md".into(),
            Catalog::embedded().unwrap().payload(SKILL).unwrap().files["SKILL.md"].clone(),
        );
        for (path, bytes) in files {
            let tool_read = client
                .call_tool(
                    CallToolRequestParams::new("skill_read").with_arguments(
                        json!({"skill_id": SKILL, "path": path})
                            .as_object()
                            .unwrap()
                            .clone(),
                    ),
                )
                .await
                .unwrap();
            assert_ne!(tool_read.is_error, Some(true), "{path}");
            let read = tool_read.structured_content.unwrap();
            assert_eq!(
                read["content"].as_str().unwrap().as_bytes(),
                bytes,
                "{path}"
            );
            assert_eq!(read["sha256"], sha256(&bytes));
            let uri = format!("mirket://skills/{SKILL}/{path}");
            let resource = client
                .read_resource(ReadResourceRequestParams::new(uri.clone()))
                .await
                .unwrap();
            let value = serde_json::to_value(resource).unwrap();
            let contents = value["contents"].as_array().unwrap();
            assert_eq!(contents.len(), 1);
            let text: ResourceContents = serde_json::from_value(contents[0].clone()).unwrap();
            let ResourceContents::TextResourceContents {
                uri: returned,
                mime_type,
                text,
                ..
            } = text
            else {
                panic!("text source returned as a blob: {path}");
            };
            assert_eq!(returned, uri);
            assert_eq!(text.as_bytes(), bytes);
            let expected = match Path::new(&path).extension().unwrap().to_str().unwrap() {
                "html" => "text/html",
                "css" => "text/css",
                "js" => "text/javascript",
                "svg" => "image/svg+xml",
                "json" => "application/json",
                "md" => "text/markdown",
                "txt" => "text/plain",
                extension => panic!("add a source-format expectation for {extension}"),
            };
            assert_eq!(mime_type.as_deref(), Some(expected), "{path}");
        }
        for uri in [
            "file:///etc/passwd",
            "mirket://skills/advanced-motion/assets/solstice/../../../../Cargo.toml",
            "mirket://skills/advanced-motion/assets\\solstice\\index.html",
            "mirket://skills/advanced-motion/assets/solstice/missing.html",
            "mirket://skills/advanced-motion/assets/solstice/%2e%2e/SKILL.md",
        ] {
            assert!(
                client
                    .read_resource(ReadResourceRequestParams::new(uri))
                    .await
                    .is_err(),
                "{uri}"
            );
        }
        // Refusal does not disrupt the initialized server's next legitimate read.
        client
            .read_resource(ReadResourceRequestParams::new(
                "mirket://skills/advanced-motion/assets/solstice/index.html",
            ))
            .await
            .unwrap();
    };
    let result = tokio::time::timeout(Duration::from_secs(45), operation).await;
    tokio::time::timeout(Duration::from_secs(5), client.cancel())
        .await
        .unwrap()
        .unwrap();
    result.unwrap();
}
