use mirket::{
    catalog::Catalog,
    install::{self, SetupOptions, Target},
    paths::Paths,
    update::{self, ReleaseAsset, ReleaseManifest, UpdateOptions},
    util::sha256,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

struct Fixture {
    _temporary: tempfile::TempDir,
    paths: Paths,
    binary: PathBuf,
    catalog: Catalog,
}
impl Fixture {
    fn new() -> Self {
        let temporary = tempfile::tempdir().unwrap();
        let paths = Paths::new(temporary.path().canonicalize().unwrap()).unwrap();
        let binary = paths.home.join("source-mirket");
        fs::write(&binary, b"installation-test-executable").unwrap();
        Self {
            _temporary: temporary,
            paths,
            binary,
            catalog: Catalog::embedded().unwrap(),
        }
    }
    fn setup(&self, options: &SetupOptions) -> anyhow::Result<install::SetupReport> {
        install::setup_from_binary(&self.paths, &self.catalog, options, &self.binary)
    }
    fn write(&self, relative: &str, bytes: impl AsRef<[u8]>) {
        let path = self.paths.home.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
    fn read(&self, relative: &str) -> Vec<u8> {
        fs::read(self.paths.home.join(relative)).unwrap()
    }
}
fn codex() -> SetupOptions {
    SetupOptions {
        targets: vec![Target::Codex],
        ..SetupOptions::default()
    }
}
fn claude() -> SetupOptions {
    SetupOptions {
        targets: vec![Target::Claude],
        ..SetupOptions::default()
    }
}

// Removal fixtures model exact receipts/configuration from installed clients.
// They never contact the documentation endpoint or write to the real home.
fn documentation_server(target: Target) -> serde_json::Value {
    match target {
        Target::Codex => serde_json::json!({"url":"https://learn.microsoft.com/api/mcp"}),
        Target::Claude => {
            serde_json::json!({"type":"http","url":"https://learn.microsoft.com/api/mcp"})
        }
    }
}

fn write_documentation_entry(fixture: &Fixture, target: Target, customized: bool) {
    match target {
        Target::Codex => {
            let mut config = String::from_utf8(fixture.read(".codex/config.toml"))
                .unwrap()
                .parse::<toml_edit::DocumentMut>()
                .unwrap();
            let mut server = toml_edit::Table::new();
            server.insert(
                "url",
                toml_edit::value("https://learn.microsoft.com/api/mcp"),
            );
            if customized {
                server.insert("enabled", toml_edit::value(false));
            }
            config["mcp_servers"]["microsoft-learn"] = toml_edit::Item::Table(server);
            fixture.write(".codex/config.toml", config.to_string());
        }
        Target::Claude => {
            let mut config: serde_json::Value =
                serde_json::from_slice(&fixture.read(".claude.json")).unwrap();
            config["mcpServers"]["microsoft-learn"] = documentation_server(target);
            if customized {
                config["mcpServers"]["microsoft-learn"]["headers"] =
                    serde_json::json!({"x-personal":"keep-private"});
            }
            fixture.write(".claude.json", serde_json::to_vec_pretty(&config).unwrap());
        }
    }
}

fn record_documentation_ownership(fixture: &Fixture, target: Target, customized: bool) {
    let mut receipt: serde_json::Value =
        serde_json::from_slice(&fixture.read(".mirket/setup.json")).unwrap();
    receipt["options"]["microsoft_learn"] = serde_json::json!(true);
    let mut owned = documentation_server(target);
    if customized {
        owned["headers"] = serde_json::json!({"x-personal":"keep-private"});
    }
    receipt["servers"][target.to_string()]["microsoft-learn"] = owned;
    fixture.write(
        ".mirket/setup.json",
        serde_json::to_vec_pretty(&receipt).unwrap(),
    );
}

fn documentation_entry(fixture: &Fixture, target: Target) -> Option<serde_json::Value> {
    match target {
        Target::Codex => {
            let config = String::from_utf8(fixture.read(".codex/config.toml"))
                .unwrap()
                .parse::<toml_edit::DocumentMut>()
                .unwrap();
            config["mcp_servers"]
                .as_table_like()
                .unwrap()
                .get("microsoft-learn")
                .map(|entry| serde_json::json!({"toml":entry.to_string()}))
        }
        Target::Claude => {
            let config: serde_json::Value =
                serde_json::from_slice(&fixture.read(".claude.json")).unwrap();
            config["mcpServers"].get("microsoft-learn").cloned()
        }
    }
}

#[test]
fn removed_documentation_setup_removes_only_exact_owned_entries_and_receipt_choice() {
    let fixture = Fixture::new();
    fixture.setup(&SetupOptions::default()).unwrap();
    for target in [Target::Codex, Target::Claude] {
        write_documentation_entry(&fixture, target, false);
        record_documentation_ownership(&fixture, target, false);
    }
    fixture.write("documents/design.md", "private document stays");
    fixture.write(
        ".mirket/packages/data/personal/index.db",
        "private index stays",
    );
    assert_eq!(
        install::saved_options(&fixture.paths).unwrap(),
        Some(SetupOptions::default())
    );
    assert!(
        !install::doctor(&fixture.paths, &fixture.catalog)
            .unwrap()
            .ok
    );
    fixture.setup(&SetupOptions::default()).unwrap();
    for target in [Target::Codex, Target::Claude] {
        assert!(documentation_entry(&fixture, target).is_none());
    }
    let receipt: serde_json::Value =
        serde_json::from_slice(&fixture.read(".mirket/setup.json")).unwrap();
    assert!(receipt["options"].get("microsoft_learn").is_none());
    for target in ["codex", "claude"] {
        assert!(receipt["servers"][target].get("microsoft-learn").is_none());
    }
    assert_eq!(
        fixture.read("documents/design.md"),
        b"private document stays"
    );
    assert_eq!(
        fixture.read(".mirket/packages/data/personal/index.db"),
        b"private index stays"
    );
    assert!(
        install::doctor(&fixture.paths, &fixture.catalog)
            .unwrap()
            .ok
    );
    assert_eq!(fixture.setup(&SetupOptions::default()).unwrap().changed, 0);
}

#[test]
fn removed_documentation_preserves_customized_and_unmanaged_entries() {
    for owned in [false, true] {
        let fixture = Fixture::new();
        fixture.setup(&SetupOptions::default()).unwrap();
        for target in [Target::Codex, Target::Claude] {
            write_documentation_entry(&fixture, target, owned);
            if owned {
                record_documentation_ownership(&fixture, target, false);
            }
        }
        let configs = [
            fixture.read(".codex/config.toml"),
            fixture.read(".claude.json"),
        ];
        fixture.setup(&SetupOptions::default()).unwrap();
        assert_eq!(
            configs,
            [
                fixture.read(".codex/config.toml"),
                fixture.read(".claude.json")
            ]
        );
        let receipt: serde_json::Value =
            serde_json::from_slice(&fixture.read(".mirket/setup.json")).unwrap();
        for target in ["codex", "claude"] {
            assert!(receipt["servers"][target].get("microsoft-learn").is_none());
        }
        assert!(
            install::doctor(&fixture.paths, &fixture.catalog)
                .unwrap()
                .ok
        );
    }
}

#[test]
fn removed_documentation_does_not_delete_a_customized_receipt_contract() {
    let fixture = Fixture::new();
    fixture.setup(&claude()).unwrap();
    write_documentation_entry(&fixture, Target::Claude, true);
    record_documentation_ownership(&fixture, Target::Claude, true);
    let original = fixture.read(".claude.json");
    fixture.setup(&claude()).unwrap();
    assert_eq!(original, fixture.read(".claude.json"));
    assert!(
        install::doctor(&fixture.paths, &fixture.catalog)
            .unwrap()
            .ok
    );
}

#[test]
fn removed_documentation_cleanup_is_preflighted_before_any_target_changes() {
    let fixture = Fixture::new();
    fixture.setup(&SetupOptions::default()).unwrap();
    write_documentation_entry(&fixture, Target::Codex, false);
    record_documentation_ownership(&fixture, Target::Codex, false);
    fixture.write(".claude.json", "{malformed-settings");
    let before = fixture.read(".codex/config.toml");
    let receipt = fixture.read(".mirket/setup.json");
    assert!(fixture.setup(&SetupOptions::default()).is_err());
    assert_eq!(before, fixture.read(".codex/config.toml"));
    assert_eq!(receipt, fixture.read(".mirket/setup.json"));
    assert_eq!(fixture.read(".claude.json"), b"{malformed-settings");
}

#[test]
fn removed_documentation_receipt_migration_remains_strict() {
    for invalid in [
        serde_json::json!({"targets":["codex"],"microsoft_learn":"true"}),
        serde_json::json!({"targets":["codex"],"unexpected_option":true}),
    ] {
        assert!(serde_json::from_value::<SetupOptions>(invalid).is_err());
    }
    for removed in [false, true] {
        let options: SetupOptions = serde_json::from_value(serde_json::json!({
            "targets":["codex"],"microsoft_learn":removed
        }))
        .unwrap();
        assert_eq!(
            serde_json::to_value(options).unwrap(),
            serde_json::json!({"targets":["codex"]})
        );
    }
}

#[test]
fn removed_documentation_saved_choice_update_replays_without_removed_cli_flag() {
    let fixture = Fixture::new();
    fixture.setup(&SetupOptions::default()).unwrap();
    for target in [Target::Codex, Target::Claude] {
        write_documentation_entry(&fixture, target, false);
        record_documentation_ownership(&fixture, target, false);
    }
    let artifact = Path::new(env!("CARGO_BIN_EXE_mirket"))
        .canonicalize()
        .unwrap();
    let report = update::update(
        &fixture.paths,
        &UpdateOptions {
            sha256: Some(sha256(&fs::read(&artifact).unwrap())),
            from: Some(artifact),
            ..UpdateOptions::default()
        },
    )
    .unwrap();
    assert!(report.updated && report.reconfigured);
    for target in [Target::Codex, Target::Claude] {
        assert!(documentation_entry(&fixture, target).is_none());
    }
    assert!(
        install::doctor(&fixture.paths, &fixture.catalog)
            .unwrap()
            .ok
    );
}

const REMOVED_PACKAGE_IDS: [&str; 10] = [
    "semiotic",
    "agent-device",
    "safe-docx",
    "markitdown",
    "dbhub",
    "doc-scraper",
    "jscpd",
    "glaxnimate",
    "helios",
    "openpencil",
];

fn record_removed_packages(fixture: &Fixture, customized: bool) {
    let mut receipt: serde_json::Value =
        serde_json::from_slice(&fixture.read(".mirket/setup.json")).unwrap();
    receipt["options"]["packages"] = serde_json::json!(REMOVED_PACKAGE_IDS);
    let mut codex = String::from_utf8(fixture.read(".codex/config.toml"))
        .unwrap()
        .parse::<toml_edit::DocumentMut>()
        .unwrap();
    let mut claude: serde_json::Value =
        serde_json::from_slice(&fixture.read(".claude.json")).unwrap();
    for id in REMOVED_PACKAGE_IDS {
        let name = format!("mirket-package-{id}");
        receipt["options"]["package_settings"][id] = serde_json::json!({
            "database":fixture.paths.home.join("documents/private.sqlite"),
            "site":"private-fixture"
        });
        let args = serde_json::json!(["--home", fixture.paths.home, "package", "serve", id]);
        let mut owned = serde_json::json!({"command":fixture.paths.binary(),"args":args});
        let mut claude_owned = owned.clone();
        claude_owned["type"] = serde_json::json!("stdio");
        if matches!(id, "glaxnimate" | "helios" | "openpencil") {
            owned["startup_timeout_sec"] = serde_json::json!(90);
            owned["tool_timeout_sec"] = serde_json::json!(120);
            claude_owned["timeout"] = serde_json::json!(120_000);
        }
        receipt["servers"]["codex"][&name] = owned.clone();
        receipt["servers"]["claude"][&name] = claude_owned.clone();
        let mut table = toml_edit::Table::new();
        table.insert(
            "command",
            toml_edit::value(fixture.paths.binary().to_str().unwrap()),
        );
        let mut arguments = toml_edit::Array::new();
        for arg in args.as_array().unwrap() {
            arguments.push(arg.as_str().unwrap());
        }
        table.insert("args", toml_edit::value(arguments));
        for key in ["startup_timeout_sec", "tool_timeout_sec"] {
            if let Some(value) = owned.get(key) {
                table.insert(key, toml_edit::value(value.as_i64().unwrap()));
            }
        }
        if customized {
            table.insert("enabled", toml_edit::value(false));
            claude_owned["env"] = serde_json::json!({"PERSONAL_SETTING":"preserve"});
        }
        codex["mcp_servers"][&name] = toml_edit::Item::Table(table);
        claude["mcpServers"][&name] = claude_owned;
        for category in ["data", "cache", "store"] {
            fixture.write(
                &format!(".mirket/packages/{category}/{id}/user-data"),
                format!("{id}:{category}:preserve"),
            );
        }
    }
    fixture.write("documents/private.sqlite", "private database remains");
    fixture.write("documents/private.docx", "private document remains");
    fixture.write(
        "documents/illustration.svg",
        "editable illustration remains",
    );
    fixture.write("documents/animation.json", "editable animation remains");
    fixture.write(".codex/config.toml", codex.to_string());
    fixture.write(".claude.json", serde_json::to_vec_pretty(&claude).unwrap());
    fixture.write(
        ".mirket/setup.json",
        serde_json::to_vec_pretty(&receipt).unwrap(),
    );
}

fn assert_removed_package_user_data(fixture: &Fixture) {
    for id in REMOVED_PACKAGE_IDS {
        for category in ["data", "cache", "store"] {
            assert_eq!(
                fixture.read(&format!(".mirket/packages/{category}/{id}/user-data")),
                format!("{id}:{category}:preserve").as_bytes()
            );
        }
    }
    assert_eq!(
        fixture.read("documents/private.sqlite"),
        b"private database remains"
    );
    assert_eq!(
        fixture.read("documents/private.docx"),
        b"private document remains"
    );
    assert_eq!(
        fixture.read("documents/illustration.svg"),
        b"editable illustration remains"
    );
    assert_eq!(
        fixture.read("documents/animation.json"),
        b"editable animation remains"
    );
}

#[test]
fn removed_packages_saved_choices_remove_owned_launches_without_erasing_user_data() {
    let fixture = Fixture::new();
    fixture.setup(&SetupOptions::default()).unwrap();
    record_removed_packages(&fixture, false);
    let saved = install::saved_options(&fixture.paths).unwrap().unwrap();
    assert!(saved.packages.is_empty() && saved.package_settings.is_empty());
    fixture.setup(&saved).unwrap();
    let codex = String::from_utf8(fixture.read(".codex/config.toml")).unwrap();
    let claude = String::from_utf8(fixture.read(".claude.json")).unwrap();
    let receipt: serde_json::Value =
        serde_json::from_slice(&fixture.read(".mirket/setup.json")).unwrap();
    for id in REMOVED_PACKAGE_IDS {
        let name = format!("mirket-package-{id}");
        assert!(!codex.contains(&name) && !claude.contains(&name));
        assert!(receipt["servers"]["codex"].get(&name).is_none());
        assert!(receipt["servers"]["claude"].get(&name).is_none());
    }
    assert_removed_package_user_data(&fixture);
    assert!(
        install::doctor(&fixture.paths, &fixture.catalog)
            .unwrap()
            .ok
    );
}

#[test]
fn removed_packages_customized_launches_are_preserved_and_released() {
    let fixture = Fixture::new();
    fixture.setup(&SetupOptions::default()).unwrap();
    record_removed_packages(&fixture, true);
    let before = [
        fixture.read(".codex/config.toml"),
        fixture.read(".claude.json"),
    ];
    fixture
        .setup(&install::saved_options(&fixture.paths).unwrap().unwrap())
        .unwrap();
    assert_eq!(
        before,
        [
            fixture.read(".codex/config.toml"),
            fixture.read(".claude.json")
        ]
    );
    let receipt: serde_json::Value =
        serde_json::from_slice(&fixture.read(".mirket/setup.json")).unwrap();
    for id in REMOVED_PACKAGE_IDS {
        let name = format!("mirket-package-{id}");
        assert!(receipt["servers"]["codex"].get(&name).is_none());
        assert!(receipt["servers"]["claude"].get(&name).is_none());
    }
    assert_removed_package_user_data(&fixture);
    assert!(
        install::doctor(&fixture.paths, &fixture.catalog)
            .unwrap()
            .ok
    );
}

#[test]
fn removed_packages_native_candidate_update_replays_sanitized_saved_choices() {
    let fixture = Fixture::new();
    fixture.setup(&SetupOptions::default()).unwrap();
    record_removed_packages(&fixture, false);
    let artifact = Path::new(env!("CARGO_BIN_EXE_mirket"))
        .canonicalize()
        .unwrap();
    let report = update::update(
        &fixture.paths,
        &UpdateOptions {
            sha256: Some(sha256(&fs::read(&artifact).unwrap())),
            from: Some(artifact),
            ..UpdateOptions::default()
        },
    )
    .unwrap();
    assert!(report.updated && report.reconfigured);
    assert_eq!(
        install::saved_options(&fixture.paths).unwrap(),
        Some(SetupOptions::default())
    );
    let receipt: serde_json::Value =
        serde_json::from_slice(&fixture.read(".mirket/setup.json")).unwrap();
    for id in REMOVED_PACKAGE_IDS {
        for target in ["codex", "claude"] {
            assert!(
                receipt["servers"][target]
                    .get(format!("mirket-package-{id}"))
                    .is_none()
            );
        }
    }
    assert_removed_package_user_data(&fixture);
    assert!(
        install::doctor(&fixture.paths, &fixture.catalog)
            .unwrap()
            .ok
    );
}

#[test]
fn removed_packages_do_not_hide_invalid_receipt_shape_before_normalization() {
    for invalid_options in [
        serde_json::json!({"targets":["codex","claude"],"packages":["semiotic","semiotic"]}),
        serde_json::json!({"targets":["codex","claude"],"packages":vec!["jscpd";33]}),
        serde_json::json!({"targets":["codex","claude"],"package_settings":{"dbhub":{"private": "x".repeat(70*1024)}}}),
    ] {
        let fixture = Fixture::new();
        fixture.setup(&SetupOptions::default()).unwrap();
        let mut receipt: serde_json::Value =
            serde_json::from_slice(&fixture.read(".mirket/setup.json")).unwrap();
        receipt["options"] = invalid_options;
        fixture.write(
            ".mirket/setup.json",
            serde_json::to_vec_pretty(&receipt).unwrap(),
        );
        let before = fixture.read(".mirket/setup.json");
        assert!(install::saved_options(&fixture.paths).is_err());
        assert_eq!(before, fixture.read(".mirket/setup.json"));
    }
}

fn run_removed_replay(
    fixture: &Fixture,
    direct: &str,
    arguments: &[String],
) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_mirket"))
        .env("MIRKET_DIRECT", direct)
        .arg("--home")
        .arg(&fixture.paths.home)
        .args(["setup", "--yes", "--target", "all"])
        .args(arguments)
        .output()
        .unwrap()
}

#[test]
fn removed_arguments_are_reconciled_only_for_an_exact_authenticated_update_replay() {
    for explicit_packages in [false, true] {
        let fixture = Fixture::new();
        fixture.setup(&SetupOptions::default()).unwrap();
        record_removed_packages(&fixture, false);
        for target in [Target::Codex, Target::Claude] {
            write_documentation_entry(&fixture, target, false);
            record_documentation_ownership(&fixture, target, false);
        }
        let token = sha256(&fixture.read(".mirket/setup.json"));
        let mut arguments = vec!["--expected-state".into(), token, "--microsoft-learn".into()];
        if explicit_packages {
            for id in REMOVED_PACKAGE_IDS {
                arguments.extend(["--package".into(), id.into()]);
            }
        }
        let outcome = run_removed_replay(&fixture, "1", &arguments);
        assert!(
            outcome.status.success(),
            "{}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        assert_eq!(
            install::saved_options(&fixture.paths).unwrap(),
            Some(SetupOptions::default())
        );
        for target in [Target::Codex, Target::Claude] {
            assert!(documentation_entry(&fixture, target).is_none());
        }
        assert_removed_package_user_data(&fixture);
        assert!(
            install::doctor(&fixture.paths, &fixture.catalog)
                .unwrap()
                .ok
        );
    }
}

#[test]
fn removed_arguments_replay_refuses_unproven_or_inconsistent_choices_without_writes() {
    let fixture = Fixture::new();
    fixture.setup(&SetupOptions::default()).unwrap();
    record_removed_packages(&fixture, false);
    for target in [Target::Codex, Target::Claude] {
        write_documentation_entry(&fixture, target, false);
        record_documentation_ownership(&fixture, target, false);
    }
    let token = sha256(&fixture.read(".mirket/setup.json"));
    let original = [
        fixture.read(".mirket/setup.json"),
        fixture.read(".codex/config.toml"),
        fixture.read(".claude.json"),
    ];
    let valid = vec![
        "--expected-state".into(),
        token.clone(),
        "--microsoft-learn".into(),
    ];
    let mut cases = vec![
        vec!["--microsoft-learn".into()],
        vec![
            "--expected-state".into(),
            "0".repeat(64),
            "--microsoft-learn".into(),
        ],
        vec![
            "--expected-state".into(),
            token.clone(),
            "--microsoft-learn=false".into(),
        ],
    ];
    for extra in [
        vec!["--dry-run".into()],
        vec!["--no-packages".into()],
        vec!["--microsoft-learn".into()],
        vec!["--package".into(), "jscpd".into()],
    ] {
        let mut arguments = valid.clone();
        arguments.extend(extra);
        cases.push(arguments);
    }
    for arguments in cases {
        let outcome = run_removed_replay(&fixture, "1", &arguments);
        assert!(
            !outcome.status.success(),
            "unproven replay accepted: {arguments:?}"
        );
        assert_eq!(
            original,
            [
                fixture.read(".mirket/setup.json"),
                fixture.read(".codex/config.toml"),
                fixture.read(".claude.json")
            ]
        );
    }
    for direct in ["0", "false", ""] {
        let outcome = run_removed_replay(&fixture, direct, &valid);
        assert!(!outcome.status.success());
        assert_eq!(
            original,
            [
                fixture.read(".mirket/setup.json"),
                fixture.read(".codex/config.toml"),
                fixture.read(".claude.json")
            ]
        );
    }
    let mut receipt: serde_json::Value = serde_json::from_slice(&original[0]).unwrap();
    receipt["options"]["microsoft_learn"] = serde_json::json!(false);
    fixture.write(
        ".mirket/setup.json",
        serde_json::to_vec_pretty(&receipt).unwrap(),
    );
    let false_receipt = fixture.read(".mirket/setup.json");
    let outcome = run_removed_replay(
        &fixture,
        "1",
        &[
            "--expected-state".into(),
            sha256(&false_receipt),
            "--microsoft-learn".into(),
        ],
    );
    assert!(!outcome.status.success());
    assert_eq!(false_receipt, fixture.read(".mirket/setup.json"));
    receipt["options"]["microsoft_learn"] = serde_json::json!(true);
    receipt["options"]["targets"] = serde_json::json!(["claude"]);
    fixture.write(
        ".mirket/setup.json",
        serde_json::to_vec_pretty(&receipt).unwrap(),
    );
    let other_targets = fixture.read(".mirket/setup.json");
    let outcome = run_removed_replay(
        &fixture,
        "1",
        &[
            "--expected-state".into(),
            sha256(&other_targets),
            "--microsoft-learn".into(),
        ],
    );
    assert!(!outcome.status.success());
    assert_eq!(other_targets, fixture.read(".mirket/setup.json"));
    assert_removed_package_user_data(&fixture);
}

#[test]
fn package_setup_rejects_conflicts_before_downloading_or_changing_state() {
    let fixture = Fixture::new();
    fixture.setup(&SetupOptions::default()).unwrap();
    let receipt = fixture.read(".mirket/setup.json");
    fixture.write(
        ".codex/config.toml",
        "[mcp_servers.mirket-package-playwright]\ncommand = 'personal-server'\n",
    );
    let config = fixture.read(".codex/config.toml");
    let options = SetupOptions {
        packages: vec!["playwright".into()],
        ..SetupOptions::default()
    };
    assert!(
        fixture
            .setup(&options)
            .unwrap_err()
            .to_string()
            .contains("conflicts")
    );
    assert_eq!(receipt, fixture.read(".mirket/setup.json"));
    assert_eq!(config, fixture.read(".codex/config.toml"));
    assert!(!fixture.paths.root.join("packages").exists());
}

#[test]
fn runtime_and_unknown_packages_fail_before_installation() {
    for package in ["nodejs", "invented-package"] {
        let fixture = Fixture::new();
        let options = SetupOptions {
            packages: vec![package.into()],
            ..SetupOptions::default()
        };
        assert!(fixture.setup(&options).is_err());
        assert!(!fixture.paths.settings().exists());
        assert!(!fixture.paths.root.join("packages").exists());
    }
}

#[test]
fn setup_preserves_unrelated_settings_and_verifies_actual_payload() {
    let fixture = Fixture::new();
    fixture.write(".codex/config.toml", "# personal preference\nmodel = \"user-model\"\n[mcp_servers.personal]\ncommand = \"personal-tool\"\n");
    fixture.write(".claude.json", br#"{"oauthAccount":{"accountUuid":"fixture"},"mcpServers":{"personal":{"type":"stdio","command":"personal-tool"}}}"#);
    let options = SetupOptions::default();
    let report = fixture.setup(&options).unwrap();
    assert!(report.files > 20);
    let receipt: serde_json::Value =
        serde_json::from_slice(&fixture.read(".mirket/setup.json")).unwrap();
    assert_eq!(
        receipt["options"],
        serde_json::json!({"targets": ["codex", "claude"]})
    );
    assert_eq!(
        fixture.read(".codex/AGENTS.md"),
        fixture.catalog.instructions()
    );
    assert_eq!(
        fixture.read(".claude/CLAUDE.md"),
        fixture.catalog.instructions()
    );
    let codex = String::from_utf8(fixture.read(".codex/config.toml")).unwrap();
    assert!(codex.contains("# personal preference"));
    assert!(codex.contains("model = \"user-model\""));
    assert!(codex.contains("[mcp_servers.personal]"));
    let claude: serde_json::Value = serde_json::from_slice(&fixture.read(".claude.json")).unwrap();
    assert_eq!(claude["oauthAccount"]["accountUuid"], "fixture");
    assert_eq!(
        claude["mcpServers"]["mirket"]["command"],
        fixture.paths.binary().to_str().unwrap()
    );
    assert_eq!(
        claude["mcpServers"]["mirket"]["args"][1],
        fixture.paths.home.to_str().unwrap()
    );
    assert_eq!(
        install::saved_options(&fixture.paths).unwrap(),
        Some(options.clone())
    );
    assert!(
        install::doctor(&fixture.paths, &fixture.catalog)
            .unwrap()
            .ok
    );
    assert_eq!(fixture.setup(&options).unwrap().changed, 0);
}

#[test]
fn host_settings_reformat_and_unrelated_edits_remain_healthy_and_unchanged() {
    let fixture = Fixture::new();
    let options = SetupOptions::default();
    fixture.setup(&options).unwrap();

    let mut claude: serde_json::Value =
        serde_json::from_slice(&fixture.read(".claude.json")).unwrap();
    claude["mcpServers"]["personal"] = serde_json::json!({"command":"personal-tool"});
    let claude_bytes = format!(
        "{{\n\t\"theme\": \"dark\",\n\t\"mcpServers\": {},\n\t\"oauthAccount\": {{\"accountUuid\": \"updated-by-client\"}}\n}}",
        claude["mcpServers"]
    )
    .into_bytes();
    fixture.write(".claude.json", &claude_bytes);

    let codex_bytes = format!(
        "# Settings serialized by the client\nmodel = 'client-selected-model'\n\n[mcp_servers]\npersonal = {{ command = 'personal-tool' }}\nopenai-docs = {{ url = 'https://developers.openai.com/mcp' }}\nmirket = {{ args = ['--home', {}, 'mcp', 'serve'], command = {} }}\n",
        serde_json::to_string(&fixture.paths.home).unwrap(),
        serde_json::to_string(&fixture.paths.binary()).unwrap(),
    )
    .into_bytes();
    fixture.write(".codex/config.toml", &codex_bytes);

    let report = install::doctor(&fixture.paths, &fixture.catalog).unwrap();
    assert!(report.ok, "{report:?}");
    assert_eq!(fixture.setup(&options).unwrap().changed, 0);
    assert_eq!(fixture.read(".claude.json"), claude_bytes);
    assert_eq!(fixture.read(".codex/config.toml"), codex_bytes);
}

#[test]
fn modified_managed_file_blocks_all_planned_changes() {
    let fixture = Fixture::new();
    fixture.setup(&SetupOptions::default()).unwrap();
    fixture.write(".codex/AGENTS.md", "user-edited instructions");
    fs::remove_file(fixture.paths.home.join(".claude/CLAUDE.md")).unwrap();
    let receipt = fs::read(fixture.paths.settings()).unwrap();
    let settings = fixture.read(".claude.json");
    let error = fixture
        .setup(&SetupOptions::default())
        .unwrap_err()
        .to_string();
    assert!(error.contains("managed file was modified"), "{error}");
    assert!(!fixture.paths.home.join(".claude/CLAUDE.md").exists());
    assert_eq!(
        fixture.read(".codex/AGENTS.md"),
        b"user-edited instructions"
    );
    assert_eq!(fs::read(fixture.paths.settings()).unwrap(), receipt);
    assert_eq!(fixture.read(".claude.json"), settings);
    assert!(
        !install::doctor(&fixture.paths, &fixture.catalog)
            .unwrap()
            .ok
    );
}

#[test]
fn malformed_second_target_settings_produce_no_payload_writes() {
    let fixture = Fixture::new();
    fixture.write(".claude.json", "{invalid-json");
    let error = fixture
        .setup(&SetupOptions::default())
        .unwrap_err()
        .to_string();
    assert!(error.contains("invalid Claude JSON settings"));
    assert!(!fixture.paths.binary().exists());
    assert!(!fixture.paths.home.join(".codex/AGENTS.md").exists());
    assert!(!fixture.paths.home.join(".agents/skills").exists());
    assert_eq!(fixture.read(".claude.json"), b"{invalid-json");
}

#[test]
fn unmanaged_instruction_conflict_is_preserved() {
    let fixture = Fixture::new();
    fixture.write(".codex/AGENTS.md", "user instructions");
    assert!(
        fixture
            .setup(&codex())
            .unwrap_err()
            .to_string()
            .contains("unmanaged file conflicts")
    );
    assert_eq!(fixture.read(".codex/AGENTS.md"), b"user instructions");
    assert!(!fixture.paths.binary().exists());
}

#[test]
fn unexpected_skill_helper_is_a_conflict() {
    let fixture = Fixture::new();
    fixture.setup(&codex()).unwrap();
    let id = &fixture.catalog.global_skills()[0];
    fixture.write(&format!(".agents/skills/{id}/hook.sh"), "unmanaged hook");
    assert!(
        fixture
            .setup(&codex())
            .unwrap_err()
            .to_string()
            .contains("unmanaged file inside managed skill")
    );
    assert!(
        !install::doctor(&fixture.paths, &fixture.catalog)
            .unwrap()
            .ok
    );
}

#[test]
fn target_change_removes_only_owned_files_and_servers() {
    let fixture = Fixture::new();
    fixture.setup(&SetupOptions::default()).unwrap();
    fixture.write(".claude/skills/personal/SKILL.md", "personal skill");
    let mut settings: serde_json::Value =
        serde_json::from_slice(&fixture.read(".claude.json")).unwrap();
    settings["theme"] = "dark".into();
    settings["mcpServers"]["personal"] = serde_json::json!({"command":"custom"});
    fixture.write(".claude.json", serde_json::to_vec(&settings).unwrap());
    fixture.setup(&codex()).unwrap();
    assert!(!fixture.paths.home.join(".claude/CLAUDE.md").exists());
    assert_eq!(
        fixture.read(".claude/skills/personal/SKILL.md"),
        b"personal skill"
    );
    let settings: serde_json::Value =
        serde_json::from_slice(&fixture.read(".claude.json")).unwrap();
    assert_eq!(settings["theme"], "dark");
    assert_eq!(settings["mcpServers"]["personal"]["command"], "custom");
    assert!(settings["mcpServers"].get("mirket").is_none());
    assert!(
        install::doctor(&fixture.paths, &fixture.catalog)
            .unwrap()
            .ok
    );
}

#[test]
fn changing_an_owned_mcp_command_is_detected_without_overwrite() {
    let fixture = Fixture::new();
    fixture.setup(&claude()).unwrap();
    let mut settings: serde_json::Value =
        serde_json::from_slice(&fixture.read(".claude.json")).unwrap();
    settings["mcpServers"]["mirket"]["command"] = "/different/program".into();
    let edited = serde_json::to_vec(&settings).unwrap();
    fixture.write(".claude.json", &edited);
    assert!(
        fixture
            .setup(&claude())
            .unwrap_err()
            .to_string()
            .contains("managed MCP server mirket was modified")
    );
    assert_eq!(fixture.read(".claude.json"), edited);
    assert!(
        !install::doctor(&fixture.paths, &fixture.catalog)
            .unwrap()
            .ok
    );
}

#[test]
fn changing_a_codex_owned_mcp_server_is_detected_without_overwrite() {
    let fixture = Fixture::new();
    fixture.setup(&codex()).unwrap();
    let original = String::from_utf8(fixture.read(".codex/config.toml")).unwrap();
    let edited = original.replace(
        "https://developers.openai.com/mcp",
        "https://example.com/other-mcp",
    );
    assert_ne!(edited, original);
    fixture.write(".codex/config.toml", &edited);
    assert!(
        fixture
            .setup(&codex())
            .unwrap_err()
            .to_string()
            .contains("managed MCP server openai-docs was modified")
    );
    assert_eq!(fixture.read(".codex/config.toml"), edited.as_bytes());
    assert!(
        !install::doctor(&fixture.paths, &fixture.catalog)
            .unwrap()
            .ok
    );
}

#[test]
fn forged_receipt_cannot_claim_unrelated_files() {
    let fixture = Fixture::new();
    fixture.setup(&codex()).unwrap();
    let mut receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.paths.settings()).unwrap()).unwrap();
    receipt["files"][".ssh/config"] =
        serde_json::json!({"sha256":"0".repeat(64),"executable":false});
    fs::write(
        fixture.paths.settings(),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    assert!(
        fixture
            .setup(&codex())
            .unwrap_err()
            .to_string()
            .contains("unexpected managed path")
    );
}

#[test]
fn setup_lock_prevents_concurrent_installations() {
    let fixture = Fixture::new();
    fixture.paths.ensure().unwrap();
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(fixture.paths.root.join("setup.lock"))
        .unwrap();
    lock.try_lock().unwrap();
    assert!(
        fixture
            .setup(&codex())
            .unwrap_err()
            .to_string()
            .contains("another mirket setup operation")
    );
    assert!(!fixture.paths.binary().exists());
}

#[cfg(unix)]
#[test]
fn symlinked_destination_and_executable_mode_drift_are_rejected() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let fixture = Fixture::new();
    let outside = tempfile::tempdir().unwrap();
    fs::create_dir_all(fixture.paths.home.join(".agents")).unwrap();
    symlink(outside.path(), fixture.paths.home.join(".agents/skills")).unwrap();
    assert!(
        fixture
            .setup(&codex())
            .unwrap_err()
            .to_string()
            .contains("symlink")
    );
    assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
    fs::remove_file(fixture.paths.home.join(".agents/skills")).unwrap();
    fixture.setup(&codex()).unwrap();
    fs::set_permissions(fixture.paths.binary(), fs::Permissions::from_mode(0o644)).unwrap();
    assert!(
        fixture
            .setup(&codex())
            .unwrap_err()
            .to_string()
            .contains("managed file was modified")
    );
}

#[cfg(unix)]
#[test]
fn io_failure_rolls_back_completed_payload_writes() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    let directory = fixture.paths.home.join(".codex");
    fs::create_dir(&directory).unwrap();
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o500)).unwrap();
    let outcome = fixture.setup(&codex());
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
    // Elevated test users can write read-only directories; this case is then not a
    // usable OS failure probe and other preflight tests still cover preservation.
    if outcome.is_ok() {
        return;
    }
    assert!(
        outcome
            .unwrap_err()
            .to_string()
            .contains("completed writes were restored")
    );
    for id in fixture.catalog.global_skills() {
        assert!(
            !fixture
                .paths
                .home
                .join(format!(".agents/skills/{id}/SKILL.md"))
                .exists()
        );
    }
    assert!(!fixture.paths.binary().exists());
    assert!(!fixture.paths.settings().exists());
}

#[test]
fn update_rejects_bad_digest_before_executing_or_mutating() {
    let fixture = Fixture::new();
    fixture.setup(&codex()).unwrap();
    let receipt = fs::read(fixture.paths.settings()).unwrap();
    let before = fs::read(fixture.paths.binary()).unwrap();
    let error = update::update(
        &fixture.paths,
        &UpdateOptions {
            from: Some(fixture.binary.clone()),
            sha256: Some("0".repeat(64)),
            check: false,
            ..UpdateOptions::default()
        },
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("SHA-256 does not match"));
    assert_eq!(fs::read(fixture.paths.binary()).unwrap(), before);
    assert_eq!(fs::read(fixture.paths.settings()).unwrap(), receipt);
}

fn entrypoint_options(fixture: &Fixture) -> UpdateOptions {
    let artifact = Path::new(env!("CARGO_BIN_EXE_mirket"))
        .canonicalize()
        .unwrap();
    let entrypoint = fixture.paths.home.join("path").join(if cfg!(windows) {
        "mirket.exe"
    } else {
        "mirket"
    });
    fs::create_dir_all(entrypoint.parent().unwrap()).unwrap();
    fs::write(&entrypoint, b"previous entrypoint").unwrap();
    UpdateOptions {
        sha256: Some(sha256(&fs::read(&artifact).unwrap())),
        from: Some(artifact),
        entrypoint: Some(entrypoint),
        expected_entrypoint_sha256: Some(sha256(b"previous entrypoint")),
        ..UpdateOptions::default()
    }
}

#[test]
fn entrypoint_update_rejects_invalid_destinations_before_changing_installation() {
    let fixture = Fixture::new();
    fixture.setup(&codex()).unwrap();
    let options = entrypoint_options(&fixture);
    let receipt = fs::read(fixture.paths.settings()).unwrap();
    let binary = fs::read(fixture.paths.binary()).unwrap();
    let path = options.entrypoint.as_ref().unwrap();
    let mut probes = Vec::new();
    let mut probe = options.clone();
    probe.expected_entrypoint_sha256 = Some("0".repeat(64));
    probes.push(probe);
    let mut probe = options.clone();
    probe.expected_entrypoint_sha256 = None;
    probes.push(probe);
    for destination in [
        PathBuf::from("mirket"),
        fixture.paths.home.join("path/../path/mirket"),
        fixture.paths.home.join("personal-tool"),
        fixture.paths.home.join("missing/mirket"),
        fixture.paths.binary(),
    ] {
        let mut probe = options.clone();
        probe.entrypoint = Some(destination);
        probes.push(probe);
    }
    let mut probe = options.clone();
    probe.check = true;
    probes.push(probe);
    let mut probe = options.clone();
    probe.entrypoint = None;
    probes.push(probe);
    let directory = fixture.paths.home.join("directory/mirket");
    fs::create_dir_all(&directory).unwrap();
    let mut probe = options.clone();
    probe.entrypoint = Some(directory);
    probes.push(probe);
    for probe in probes {
        assert!(update::update(&fixture.paths, &probe).is_err(), "{probe:?}");
        assert_eq!(fs::read(path).unwrap(), b"previous entrypoint");
        assert_eq!(fs::read(fixture.paths.settings()).unwrap(), receipt);
        assert_eq!(fs::read(fixture.paths.binary()).unwrap(), binary);
    }
}

#[cfg(unix)]
#[test]
fn entrypoint_update_rejects_file_and_parent_symlinks() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    fixture.setup(&codex()).unwrap();
    let options = entrypoint_options(&fixture);
    let receipt = fs::read(fixture.paths.settings()).unwrap();
    let links = fixture.paths.home.join("links");
    fs::create_dir(&links).unwrap();
    symlink(options.entrypoint.as_ref().unwrap(), links.join("mirket")).unwrap();
    symlink(fixture.paths.home.join("path"), links.join("directory")).unwrap();
    for destination in [links.join("mirket"), links.join("directory/mirket")] {
        let mut probe = options.clone();
        probe.entrypoint = Some(destination);
        assert!(
            format!("{:#}", update::update(&fixture.paths, &probe).unwrap_err())
                .contains("symlink")
        );
    }
    assert_eq!(
        fs::read(options.entrypoint.unwrap()).unwrap(),
        b"previous entrypoint"
    );
    assert_eq!(fs::read(fixture.paths.settings()).unwrap(), receipt);
}

#[test]
fn entrypoint_update_copies_verified_binary_and_retries_idempotently() {
    let fixture = Fixture::new();
    fixture.setup(&codex()).unwrap();
    let options = entrypoint_options(&fixture);
    let path = options.entrypoint.as_ref().unwrap();
    let personal = path.parent().unwrap().join("personal-tool");
    fs::write(&personal, b"unrelated executable").unwrap();
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_mirket"));
    command
        .env("MIRKET_DIRECT", "1")
        .arg("--home")
        .arg(&fixture.paths.home)
        .args(["--json", "update", "--from"])
        .arg(options.from.as_ref().unwrap())
        .arg("--sha256")
        .arg(options.sha256.as_ref().unwrap())
        .arg("--entrypoint")
        .arg(path)
        .arg("--expected-entrypoint-sha256")
        .arg(options.expected_entrypoint_sha256.as_ref().unwrap());
    let first = command.output().unwrap();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let first: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(first["updated"], true);
    assert_eq!(first["entrypoint"]["updated"], true);
    assert_eq!(first["entrypoint"]["path"], path.to_str().unwrap());
    assert_eq!(
        first["entrypoint"]["sha256"],
        *options.sha256.as_ref().unwrap()
    );
    assert_eq!(
        fs::read(path).unwrap(),
        fs::read(fixture.paths.binary()).unwrap()
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o755
        );
    }
    let actual = std::process::Command::new(path)
        .env("MIRKET_DIRECT", "1")
        .arg("--version")
        .output()
        .unwrap();
    assert!(actual.status.success());
    assert_eq!(
        String::from_utf8(actual.stdout).unwrap().trim(),
        format!("mirket {}", env!("CARGO_PKG_VERSION"))
    );
    let second = command.output().unwrap();
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    let second: serde_json::Value = serde_json::from_slice(&second.stdout).unwrap();
    assert_eq!(second["updated"], false);
    assert_eq!(second["entrypoint"]["updated"], false);
    assert_eq!(fs::read(&personal).unwrap(), b"unrelated executable");
    assert!(!fs::read_dir(path.parent().unwrap()).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".mirket-entrypoint-")
    }));
}

#[cfg(unix)]
#[test]
fn entrypoint_update_refuses_destination_changed_after_preflight() {
    let fixture = Fixture::new();
    fixture.setup(&codex()).unwrap();
    let mut options = entrypoint_options(&fixture);
    let path = options.entrypoint.as_ref().unwrap();
    let quoted = format!("'{}'", path.to_str().unwrap().replace('\'', "'\\''"));
    let bytes = format!(
        "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then printf 'concurrent edit' > {quoted}; printf 'mirket {}\\n'; exit 0; fi\nexit 88\n",
        env!("CARGO_PKG_VERSION")
    );
    let artifact = fixture.paths.home.join("changing-candidate");
    fs::write(&artifact, &bytes).unwrap();
    options.from = Some(artifact);
    options.sha256 = Some(sha256(bytes.as_bytes()));
    let receipt = fs::read(fixture.paths.settings()).unwrap();
    let binary = fs::read(fixture.paths.binary()).unwrap();
    let error = format!(
        "{:#}",
        update::update(&fixture.paths, &options).unwrap_err()
    );
    assert!(
        error.contains("entrypoint changed since preflight"),
        "{error}"
    );
    assert_eq!(fs::read(path).unwrap(), b"concurrent edit");
    assert_eq!(fs::read(fixture.paths.settings()).unwrap(), receipt);
    assert_eq!(fs::read(fixture.paths.binary()).unwrap(), binary);
}

#[test]
fn offline_update_runs_new_cli_and_reapplies_saved_setup() {
    let fixture = Fixture::new();
    fixture.setup(&claude()).unwrap();
    fixture.write(".claude/CLAUDE.md", fixture.catalog.instructions());
    let artifact = Path::new(env!("CARGO_BIN_EXE_mirket"))
        .canonicalize()
        .unwrap();
    let digest = sha256(&fs::read(&artifact).unwrap());
    let report = update::update(
        &fixture.paths,
        &UpdateOptions {
            from: Some(artifact),
            sha256: Some(digest.clone()),
            check: false,
            ..UpdateOptions::default()
        },
    )
    .unwrap();
    assert!(report.updated);
    assert!(report.reconfigured);
    assert_eq!(sha256(&fs::read(fixture.paths.binary()).unwrap()), digest);
    assert_eq!(
        install::saved_options(&fixture.paths).unwrap(),
        Some(claude())
    );
    assert!(
        install::doctor(&fixture.paths, &fixture.catalog)
            .unwrap()
            .ok
    );
    assert!(!fixture.paths.home.join(".codex/AGENTS.md").exists());
    let settings: serde_json::Value =
        serde_json::from_slice(&fixture.read(".claude.json")).unwrap();
    assert_eq!(
        settings["mcpServers"]["openai-docs"]["url"],
        "https://developers.openai.com/mcp"
    );
    assert!(!fs::read_dir(&fixture.paths.root).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".update-")
    }));
}

#[cfg(unix)]
#[test]
fn failing_new_runtime_preserves_installation() {
    let fixture = Fixture::new();
    fixture.setup(&codex()).unwrap();
    let candidate = fixture.paths.home.join("failing-runtime");
    let bytes = format!(
        "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 'mirket {}'; exit 0; fi\necho 'fixture setup failure' >&2\nexit 4\n",
        env!("CARGO_PKG_VERSION")
    );
    fs::write(&candidate, &bytes).unwrap();
    let receipt = fs::read(fixture.paths.settings()).unwrap();
    let binary = fs::read(fixture.paths.binary()).unwrap();
    let outcome = update::update(
        &fixture.paths,
        &UpdateOptions {
            from: Some(candidate),
            sha256: Some(sha256(bytes.as_bytes())),
            check: false,
            ..UpdateOptions::default()
        },
    );
    assert!(
        outcome
            .unwrap_err()
            .to_string()
            .contains("new runtime setup failed")
    );
    assert_eq!(fs::read(fixture.paths.settings()).unwrap(), receipt);
    assert_eq!(fs::read(fixture.paths.binary()).unwrap(), binary);
}

#[test]
fn release_manifest_requires_matching_official_assets_and_unique_targets() {
    let target = update::target_triple().unwrap();
    let asset = ReleaseAsset {
        target: target.into(),
        url: format!(
            "https://github.com/codemirket/harness/releases/download/mirket-v1.0.0/mirket-{target}"
        ),
        sha256: "a".repeat(64),
    };
    let mut manifest = ReleaseManifest {
        version: "1.0.0".into(),
        assets: vec![asset.clone()],
    };
    update::validate_manifest(&manifest).unwrap();
    manifest.assets[0].url = "https://example.com/mirket".into();
    assert!(update::validate_manifest(&manifest).is_err());
    manifest.assets = vec![asset.clone(), asset];
    assert!(update::validate_manifest(&manifest).is_err());
    manifest.assets.truncate(1);
    manifest.version = "01.0.0".into();
    assert!(update::validate_manifest(&manifest).is_err());
}

#[test]
fn update_generation_guard_preserves_newer_saved_choices() {
    let fixture = Fixture::new();
    fixture.setup(&codex()).unwrap();
    let inspected_generation = sha256(&fs::read(fixture.paths.settings()).unwrap());
    fixture.setup(&claude()).unwrap();
    let newer = fs::read(fixture.paths.settings()).unwrap();
    let error = install::setup_checked(
        &fixture.paths,
        &fixture.catalog,
        &codex(),
        Some(&inspected_generation),
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("setup selection changed during update"));
    assert_eq!(fs::read(fixture.paths.settings()).unwrap(), newer);
    assert_eq!(
        install::saved_options(&fixture.paths).unwrap(),
        Some(claude())
    );
    assert!(!fixture.paths.home.join(".codex/AGENTS.md").exists());
}

#[cfg(unix)]
#[test]
fn ordinary_permission_changes_are_preserved() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    fixture.setup(&codex()).unwrap();
    let instructions = fixture.paths.home.join(".codex/AGENTS.md");
    fs::set_permissions(&instructions, fs::Permissions::from_mode(0o600)).unwrap();
    fs::set_permissions(fixture.paths.binary(), fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(fixture.setup(&codex()).unwrap().changed, 0);
    assert!(
        install::doctor(&fixture.paths, &fixture.catalog)
            .unwrap()
            .ok
    );
    assert_eq!(
        fs::metadata(instructions).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn newer_managed_runtime_requires_matching_owned_bytes_and_prevents_downgrade() {
    let fixture = Fixture::new();
    fixture.setup(&codex()).unwrap();
    assert!(install::newer_runtime(&fixture.paths).unwrap().is_none());
    let mut receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.paths.settings()).unwrap()).unwrap();
    let next_major = env!("CARGO_PKG_VERSION_MAJOR").parse::<u64>().unwrap() + 1;
    receipt["version"] = format!("{next_major}.0.0").into();
    fs::write(
        fixture.paths.settings(),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    assert_eq!(
        install::newer_runtime(&fixture.paths).unwrap(),
        Some(fixture.paths.binary())
    );
    assert!(
        !install::doctor(&fixture.paths, &fixture.catalog)
            .unwrap()
            .ok
    );
    assert!(
        fixture
            .setup(&codex())
            .unwrap_err()
            .to_string()
            .contains("newer than this runtime")
    );
    fs::write(fixture.paths.binary(), b"changed executable").unwrap();
    assert!(
        install::newer_runtime(&fixture.paths)
            .unwrap_err()
            .to_string()
            .contains("differs from its setup receipt")
    );
}
