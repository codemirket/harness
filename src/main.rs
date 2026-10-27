use clap::{Arg, Command};
use mirket::{
    codex_config::CodexConfig,
    distribution,
    engine::Engine,
    installation::Installation,
    release::{AdapterPayload, ReleaseId},
    release_store::ReleaseStore,
    server::Server,
    skill_sync::{ProjectSelection, SkillSync},
};
use rmcp::ServiceExt;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::Arc,
};
fn running_installed_launcher(home: &Path) -> bool {
    #[cfg(target_os = "macos")]
    {
        let launcher = home.join("bin/mirket");
        let Ok(metadata) = std::fs::symlink_metadata(&launcher) else {
            return false;
        };
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return false;
        }
        let (Ok(running), Ok(installed)) = (
            std::env::current_exe().and_then(|path| path.canonicalize()),
            launcher.canonicalize(),
        ) else {
            return false;
        };
        running == installed
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = home;
        false
    }
}
fn required(name: &'static str) -> Arg {
    Arg::new(name).long(name).required(true).num_args(1)
}
fn intent_retirement_command() -> Command {
    Command::new("intent-retire")
        .about(
            "Retire one expired, never-started artifact or memory preparation during idle maintenance",
        )
        .arg(required("state-dir"))
        .arg(required("release-id"))
        .arg(required("operation-id"))
        .arg(required("kind").value_parser(["artifact", "memory"]))
        .arg(Arg::new("project-root").long("project-root").num_args(1))
        .arg(
            Arg::new("shared-preferences")
                .long("shared-preferences")
                .action(clap::ArgAction::SetTrue),
        )
        .group(
            clap::ArgGroup::new("scope")
                .args(["project-root", "shared-preferences"])
                .required(true),
        )
}
fn memory_command() -> Command {
    let common = |command: Command| {
        command
            .arg(required("state-dir"))
            .arg(required("backup-dir"))
            .arg(required("release-id"))
            .arg(Arg::new("project-root").long("project-root").num_args(1))
            .arg(
                Arg::new("shared-preferences")
                    .long("shared-preferences")
                    .action(clap::ArgAction::SetTrue),
            )
            .group(
                clap::ArgGroup::new("scope")
                    .args(["project-root", "shared-preferences"])
                    .required(true),
            )
    };
    Command::new("memory")
        .about(
            "Inspect managed backups or explicitly prepare, commit and reconcile a scoped restore",
        )
        .subcommand_required(true)
        .subcommand(common(Command::new("backups").about(
            "List existing scoped recovery points without creating state",
        )))
        .subcommand(common(
            Command::new("export-prepare")
                .about("Prepare a same-scope canonical memory export to a new private file")
                .arg(required("path")),
        ))
        .subcommand(common(
            Command::new("import-prepare")
                .about("Bind a private JSON Lines archive for same-scope recovery-point admission")
                .arg(required("path")),
        ))
        .subcommand(common(
            Command::new("transfer-retire")
                .about("Retire one expired unused transfer preparation during maintenance")
                .arg(required("operation-id")),
        ))
        .subcommand(common(
            Command::new("transfer-status")
                .about("Inspect an issued memory transfer without replay")
                .arg(required("operation-id")),
        ))
        .subcommand(common(
            Command::new("transfer-commit")
                .about("Execute a prepared memory transfer during an idle maintenance window")
                .arg(required("operation-id")),
        ))
        .subcommand(common(
            Command::new("transfer-reconcile")
                .about("Reconcile only verified existing transfer authority")
                .arg(required("operation-id")),
        ))
        .subcommand(common(
            Command::new("restore-prepare")
                .about("Inspect a snapshot and issue a five-minute restore intent")
                .arg(required("snapshot-id")),
        ))
        .subcommand(common(
            Command::new("restore-status")
                .about("Inspect an issued restore without replay or state changes")
                .arg(required("operation-id")),
        ))
        .subcommand(common(
            Command::new("restore-commit")
                .about("Restore the prepared scope during an idle maintenance window")
                .arg(required("operation-id")),
        ))
        .subcommand(common(
            Command::new("restore-retire")
                .about("Retire one expired unused restore intent during an idle maintenance window")
                .arg(required("operation-id")),
        ))
        .subcommand(common(
            Command::new("restore-reconcile")
                .about("Resume only this operation's existing authoritative restore binding")
                .arg(required("operation-id")),
        ))
}
fn skills_command() -> Command {
    let global = |name: &'static str, about: &'static str| {
        Command::new(name)
            .about(about)
            .arg(required("home"))
            .arg(required("user-home"))
    };
    let project = |name: &'static str, about: &'static str| {
        Command::new(name)
            .about(about)
            .arg(required("home"))
            .arg(required("project-root"))
    };
    let selection = |command: Command| {
        command
            .arg(
                Arg::new("profile")
                    .long("profile")
                    .num_args(1)
                    .action(clap::ArgAction::Append),
            )
            .arg(
                Arg::new("include")
                    .long("include")
                    .num_args(1)
                    .action(clap::ArgAction::Append),
            )
            .arg(
                Arg::new("exclude")
                    .long("exclude")
                    .num_args(1)
                    .action(clap::ArgAction::Append),
            )
    };
    Command::new("skills")
        .about("Plan, install and inspect receipt-owned Codex native skills")
        .subcommand_required(true)
        .subcommand(global(
            "global-plan",
            "Preview the nine global skill copies",
        ))
        .subcommand(global(
            "global-sync",
            "Install or update the nine global skills",
        ))
        .subcommand(global(
            "global-doctor",
            "Inspect managed global skill copies",
        ))
        .subcommand(global(
            "global-reconcile",
            "Recover a known interrupted global sync",
        ))
        .subcommand(global(
            "global-remove",
            "Remove only unchanged managed global skills",
        ))
        .subcommand(selection(project(
            "project-plan",
            "Preview a project-local pinned skill selection without writing it",
        )))
        .subcommand(selection(project(
            "project-select",
            "Pin and automatically synchronize a new project skill selection",
        )))
        .subcommand(selection(project(
            "project-update",
            "Compare and replace an existing project skill selection, then synchronize",
        )))
        .subcommand(project(
            "project-sync",
            "Synchronize the saved project selection against the active release",
        ))
        .subcommand(project(
            "project-doctor",
            "Inspect managed project-local skill copies",
        ))
        .subcommand(project(
            "project-reconcile",
            "Recover a known interrupted project skill sync",
        ))
        .subcommand(project(
            "project-remove",
            "Remove only unchanged managed project-local skills",
        ))
}
fn command() -> Command {
    Command::new("mirket")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Local tools for host-owned agents")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(skills_command())
        .subcommand(
            Command::new("setup")
                .about("Install a verified local release and its managed Codex integration")
                .arg(required("home"))
                .arg(required("source"))
                .arg(required("user-home"))
                .arg(
                    Arg::new("project-root")
                        .long("project-root")
                        .num_args(1),
                ),
        )
        .subcommand(
            Command::new("update")
                .about("Update the release and managed Codex integration when sessions are idle")
                .arg(required("home"))
                .arg(required("source"))
                .arg(required("user-home"))
                .arg(
                    Arg::new("project-root")
                        .long("project-root")
                        .num_args(1),
                ),
        )
        .subcommand(
            Command::new("doctor")
                .about("Inspect the selected release and managed Codex integration without writing")
                .arg(required("home"))
                .arg(required("user-home"))
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("uninstall")
                .about("Remove verified payloads and managed Codex integration; preserve durable data")
                .arg(required("home"))
                .arg(required("user-home")),
        )
        .subcommand(
            Command::new("uninstall-status")
                .about("Inspect the selected uninstall plan and progress without writing")
                .arg(required("home"))
                .arg(required("user-home")),
        )
        .subcommand(
            Command::new("uninstall-reconcile")
                .about("Explicitly finish the original interrupted uninstall; preserve unknown changes")
                .arg(required("home"))
                .arg(required("user-home")),
        )
        .subcommand(
            Command::new("reinstall")
                .about("Install an exact release after a completed same-home uninstall; preserve durable data")
                .arg(required("home"))
                .arg(required("user-home"))
                .arg(required("source")),
        )
        .subcommand(
            Command::new("reinstall-status")
                .about("Inspect the original reinstall intent or completed receipt without writing")
                .arg(required("home"))
                .arg(required("user-home")),
        )
        .subcommand(
            Command::new("reinstall-reconcile")
                .about("Explicitly finish the original same-candidate reinstall")
                .arg(required("home"))
                .arg(required("user-home"))
                .arg(required("source")),
        )
        .subcommand(
            Command::new("codex-register")
                .about("Register the active installation in an explicitly selected Codex config")
                .arg(required("home"))
                .arg(required("config")),
        )
        .subcommand(
            Command::new("codex-remove")
                .about("Remove Mirket's managed entry from an explicitly selected Codex config")
                .arg(required("home"))
                .arg(required("config")),
        )
        .subcommand(
            Command::new("codex-reconcile")
                .about("Reconcile an interrupted Codex config transaction")
                .arg(required("home"))
                .arg(required("config")),
        )
        .subcommand(
            Command::new("codex-doctor")
                .about("Inspect the managed Codex entry without changing it")
                .arg(required("home"))
                .arg(required("config")),
        )
        .subcommand(
            Command::new("codex-guidance-install")
                .about("Install only the managed section in an explicitly selected Codex AGENTS.md")
                .arg(required("home"))
                .arg(required("guidance")),
        )
        .subcommand(
            Command::new("codex-guidance-remove")
                .about("Remove only the unchanged managed Codex guidance section")
                .arg(required("home"))
                .arg(required("guidance")),
        )
        .subcommand(
            Command::new("codex-guidance-doctor")
                .about("Inspect the managed Codex guidance section without changing it")
                .arg(required("home"))
                .arg(required("guidance")),
        )
        .subcommand(
            Command::new("codex-guidance-reconcile")
                .about("Reconcile only a known interrupted Codex guidance edit")
                .arg(required("home"))
                .arg(required("guidance")),
        )
        .subcommand(
            Command::new("serve")
                .about(
                    "Serve the verified active release in an explicitly selected installation home",
                )
                .arg(required("home"))
                .arg(
                    Arg::new("expected-release-id")
                        .long("expected-release-id")
                        .num_args(1)
                        .hide(true),
                ),
        )
        .subcommand(
            Command::new("dev")
                .about("Explicit disposable development commands; no host configuration changes")
                .subcommand_required(true)
                .subcommand(memory_command())
                .subcommand(intent_retirement_command())
                .subcommand(
                    Command::new("setup-core")
                        .about("Initialize a disposable core-only installation for development")
                        .arg(required("home"))
                        .arg(required("source")),
                )
                .subcommand(
                    Command::new("update-core")
                        .about("Exercise core-only staged activation in a disposable installation")
                        .arg(required("home"))
                        .arg(required("source")),
                )
                .subcommand(
                    Command::new("doctor-core")
                        .about("Inspect only the core release in a disposable installation")
                        .arg(required("home"))
                        .arg(Arg::new("json").long("json").action(clap::ArgAction::SetTrue)),
                )
                .subcommand(
                    Command::new("serve")
                        .about("Serve an explicit development payload over stdio MCP")
                        .arg(required("state-dir"))
                        .arg(required("release-id"))
                        .arg(required("node"))
                        .arg(Arg::new("backup-dir").long("backup-dir").num_args(1))
                        .arg(
                            Arg::new("embedding-package")
                                .long("embedding-package")
                                .num_args(1),
                        )
                        .arg(Arg::new("design-adapter").long("design-adapter").num_args(1).requires("design-font"))
                        .arg(Arg::new("design-font").long("design-font").num_args(1).requires("design-adapter"))
                        .arg(Arg::new("raster-module").long("raster-module").num_args(1).requires("design-adapter"))
                        .arg(Arg::new("openpencil-adapter").long("openpencil-adapter").num_args(1).requires_all(["openpencil-runtime", "openpencil-sha256"]))
                        .arg(Arg::new("openpencil-runtime").long("openpencil-runtime").num_args(1).requires_all(["openpencil-adapter", "openpencil-sha256"]))
                        .arg(Arg::new("openpencil-sha256").long("openpencil-sha256").num_args(1).requires_all(["openpencil-adapter", "openpencil-runtime"]))
                        .arg(Arg::new("video-adapter").long("video-adapter").num_args(1).requires_all(["video-chromium", "video-ffmpeg", "video-ffprobe"]))
                        .arg(Arg::new("video-chromium").long("video-chromium").num_args(1).requires_all(["video-adapter", "video-ffmpeg", "video-ffprobe"]))
                        .arg(Arg::new("video-ffmpeg").long("video-ffmpeg").num_args(1).requires_all(["video-adapter", "video-chromium", "video-ffprobe"]))
                        .arg(Arg::new("video-ffprobe").long("video-ffprobe").num_args(1).requires_all(["video-adapter", "video-chromium", "video-ffmpeg"])),
                )
                .subcommand(
                    Command::new("manifest")
                        .about("Create a manifest for an owned, complete release payload")
                        .arg(required("directory")),
                )
                .subcommand(
                    Command::new("home-init")
                        .about("Initialize an empty disposable installation home")
                        .arg(required("home")),
                )
                .subcommand(
                    Command::new("stage")
                        .about("Verify and stage a local release payload")
                        .arg(required("home"))
                        .arg(required("source")),
                )
                .subcommand(
                    Command::new("activate")
                        .about("Activate a staged development release without publishing the stable launcher")
                        .arg(required("home")),
                ),
        )
}
#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(message) = run().await {
        eprintln!("mirket: {message}");
        std::process::exit(1);
    }
}
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("__design_raster")) {
        return mirket::design_raster_leaf::serve_raster_leaf()
            .map_err(|e| std::io::Error::other(e).into());
    }
    if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("__worker")) {
        if std::env::args_os().count() != 2 {
            return Err("internal worker mode accepts no arguments".into());
        }
        return mirket::worker::serve_worker()
            .await
            .map_err(|e| std::io::Error::other(e).into());
    }
    if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("__embedding_leaf")) {
        if std::env::args_os().count() != 2 {
            return Err("internal embedding mode accepts no arguments".into());
        }
        return mirket::embedding_job::serve_warm_leaf()
            .map_err(|e| std::io::Error::other(e).into());
    }
    if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("__embedding_supervisor"))
    {
        if std::env::args_os().count() != 2 {
            return Err("internal embedding supervisor accepts no arguments".into());
        }
        return mirket::embedding_session::serve_supervisor()
            .await
            .map_err(|e| std::io::Error::other(e).into());
    }
    let args = command().get_matches();
    let (name, args) = args.subcommand().expect("required subcommand");
    if name == "skills" {
        run_skills(args).map_err(std::io::Error::other)?;
        return Ok(());
    }
    if name == "setup" {
        let report = Installation::setup_codex_with_project(
            Path::new(value(args, "home")),
            Path::new(value(args, "source")),
            Path::new(value(args, "user-home")),
            args.get_one::<String>("project-root").map(Path::new),
        )
        .map_err(std::io::Error::other)?;
        println!("{}", serde_json::to_string(&report)?);
        return Ok(());
    }
    if name == "update" {
        let installation =
            Installation::open(Path::new(value(args, "home"))).map_err(std::io::Error::other)?;
        let report = installation
            .update_codex_with_project(
                Path::new(value(args, "source")),
                Path::new(value(args, "user-home")),
                args.get_one::<String>("project-root").map(Path::new),
            )
            .map_err(std::io::Error::other)?;
        println!("{}", serde_json::to_string(&report)?);
        return Ok(());
    }
    if name == "doctor" {
        let report = Installation::doctor_codex(
            Path::new(value(args, "home")),
            Path::new(value(args, "user-home")),
        );
        println!("{}", serde_json::to_string(&report)?);
        if !matches!(report.status, "healthy" | "uninstalled") {
            return Err("Codex installation health check failed".into());
        }
        return Ok(());
    }
    if matches!(
        name,
        "uninstall" | "uninstall-status" | "uninstall-reconcile"
    ) {
        let installation =
            Installation::open(Path::new(value(args, "home"))).map_err(std::io::Error::other)?;
        let user_home = Path::new(value(args, "user-home"));
        let report = match name {
            "uninstall" => installation.uninstall_codex(user_home),
            "uninstall-status" => installation.uninstall_status(user_home),
            _ => installation.reconcile_uninstall(user_home),
        }
        .map_err(std::io::Error::other)?;
        println!("{}", serde_json::to_string(&report)?);
        return Ok(());
    }
    if matches!(
        name,
        "reinstall" | "reinstall-status" | "reinstall-reconcile"
    ) {
        let installation =
            Installation::open(Path::new(value(args, "home"))).map_err(std::io::Error::other)?;
        let user_home = Path::new(value(args, "user-home"));
        let report = match name {
            "reinstall" => {
                installation.reinstall_codex(Path::new(value(args, "source")), user_home)
            }
            "reinstall-status" => installation.reinstall_status(user_home),
            _ => installation.reconcile_reinstall(Path::new(value(args, "source")), user_home),
        }
        .map_err(std::io::Error::other)?;
        println!("{}", serde_json::to_string(&report)?);
        return Ok(());
    }
    if name == "codex-register" {
        let installation =
            Installation::open(Path::new(value(args, "home"))).map_err(std::io::Error::other)?;
        let report = installation
            .register_codex(Path::new(value(args, "config")))
            .map_err(std::io::Error::other)?;
        println!("{}", serde_json::to_string(&report)?);
        return Ok(());
    }
    if name == "codex-remove" {
        let installation =
            Installation::open(Path::new(value(args, "home"))).map_err(std::io::Error::other)?;
        let report = installation
            .remove_codex(Path::new(value(args, "config")))
            .map_err(std::io::Error::other)?;
        println!("{}", serde_json::to_string(&report)?);
        return Ok(());
    }
    if name == "codex-reconcile" {
        let installation =
            Installation::open(Path::new(value(args, "home"))).map_err(std::io::Error::other)?;
        let report = installation
            .reconcile_codex(Path::new(value(args, "config")))
            .map_err(std::io::Error::other)?;
        println!("{}", serde_json::to_string(&report)?);
        return Ok(());
    }
    if name == "codex-doctor" {
        let installation =
            Installation::open(Path::new(value(args, "home"))).map_err(std::io::Error::other)?;
        let health = Installation::doctor(installation.home());
        if health.status != "healthy" {
            return Err(std::io::Error::other(format!(
                "installation health check failed: {}",
                health.errors.join("; ")
            ))
            .into());
        }
        let report = CodexConfig::open(
            Path::new(value(args, "config")),
            &installation.codex_receipt_path(),
        )
        .and_then(|config| config.inspect())
        .map_err(std::io::Error::other)?;
        println!("{}", serde_json::to_string(&report)?);
        if report.status == "drifted" || report.status == "conflict" {
            return Err("managed Codex configuration requires explicit reconciliation".into());
        }
        return Ok(());
    }
    if matches!(
        name,
        "codex-guidance-install"
            | "codex-guidance-remove"
            | "codex-guidance-doctor"
            | "codex-guidance-reconcile"
    ) {
        let installation =
            Installation::open(Path::new(value(args, "home"))).map_err(std::io::Error::other)?;
        let path = Path::new(value(args, "guidance"));
        let report = match name {
            "codex-guidance-install" => installation.install_codex_guidance(path),
            "codex-guidance-remove" => installation.remove_codex_guidance(path),
            "codex-guidance-doctor" => installation.inspect_codex_guidance(path),
            "codex-guidance-reconcile" => installation.reconcile_codex_guidance(path),
            _ => unreachable!(),
        }
        .map_err(std::io::Error::other)?;
        println!("{}", serde_json::to_string(&report)?);
        if matches!(report.status, "conflict" | "drifted" | "pending") {
            return Err("managed Codex guidance needs inspection or reconciliation".into());
        }
        return Ok(());
    }
    if name == "serve" {
        let home = Path::new(value(args, "home")).canonicalize()?;
        let store = ReleaseStore::open(&home).map_err(std::io::Error::other)?;
        if running_installed_launcher(&home) {
            // Catalog discovery is allowed before the expensive payload hash,
            // but the bound session excludes activation and every tool call
            // waits for complete verification and engine opening.
            let bound = store.bind_active().map_err(std::io::Error::other)?;
            if args
                .get_one::<String>("expected-release-id")
                .is_some_and(|expected| expected != &bound.release_id)
            {
                return Err(
                    "active release changed during launcher handoff; reconnect explicitly".into(),
                );
            }
            mirket::deferred_server::serve_bound(bound, home).await?;
            return Ok(());
        }
        // Session admission protects the activation pointer while the saved
        // payload is verified. Reuse that one verified release for handoff and
        // engine startup instead of hashing the entire payload twice.
        let active = store.open_active().map_err(std::io::Error::other)?;
        let observed = &active.release;
        let expected = args.get_one::<String>("expected-release-id");
        if let Some(expected) = expected
            && expected != observed.id().as_str()
        {
            return Err(
                "active release changed during launcher handoff; reconnect explicitly".into(),
            );
        }
        if !observed.running_core_matches()? {
            if expected.is_some() {
                return Err(
                    "running core differs from the bound active release; refusing a second handoff"
                        .into(),
                );
            }
            #[cfg(target_os = "macos")]
            {
                use std::os::unix::process::CommandExt;
                // Keep maintenance exclusion through exec. CLOEXEC closes the
                // old session's locks; the new core acquires its own guard and
                // checks this exact release before opening an engine or stdin.
                // Activation in that small gap causes refusal, never a loop or
                // serving new resources with an old core. No wrapper remains.
                let error = std::process::Command::new(observed.executable())
                    .arg("serve")
                    .arg("--home")
                    .arg(&home)
                    .arg("--expected-release-id")
                    .arg(observed.id().as_str())
                    .exec();
                return Err(error.into());
            }
            #[cfg(windows)]
            {
                let status = std::process::Command::new(observed.executable())
                    .arg("serve")
                    .arg("--home")
                    .arg(&home)
                    .arg("--expected-release-id")
                    .arg(observed.id().as_str())
                    .status()?;
                if status.success() {
                    return Ok(());
                }
                return Err(format!("active release handoff failed: {status}").into());
            }
            #[cfg(not(any(target_os = "macos", windows)))]
            return Err("active-core launcher handoff is not qualified on this platform".into());
        }
        let installed_payload = mirket::installed_payload::resolve_installed_payload(
            &active.release,
            &active.state_root,
            &active.guard,
        )
        .map_err(std::io::Error::other)?;
        let engine = Engine::open(
            active.release.id().clone(),
            installed_payload,
            active.state_root.clone(),
        )
        .map_err(std::io::Error::other)?;
        let embedding_runtime = active
            .release
            .embedding_package()
            .map(|package| {
                mirket::memory_runtime::MemoryRuntime::new(
                    active.state_root.clone(),
                    active.release.id().clone(),
                    package,
                    active.release.executable(),
                )
                .map(Arc::new)
            })
            .transpose()
            .map_err(std::io::Error::other)?;
        let options = mirket::memory_service::MemoryOptions {
            backup_root: Some(home.join("backups")),
            embedding_runtime,
        };
        serve(engine, options).await?;
        drop(active);
        return Ok(());
    }
    let (name, args) = args.subcommand().expect("required dev subcommand");
    match name {
        "setup-core" => {
            let report = Installation::setup(
                Path::new(value(args, "home")),
                Path::new(value(args, "source")),
            )
            .map_err(std::io::Error::other)?;
            println!("{}", serde_json::to_string(&report)?);
        }
        "update-core" => {
            let installation = Installation::open(Path::new(value(args, "home")))
                .map_err(std::io::Error::other)?;
            let health = Installation::doctor(installation.home());
            if health.status != "healthy" {
                return Err(std::io::Error::other(format!(
                    "existing core installation is not healthy: {}",
                    health.errors.join("; ")
                ))
                .into());
            }
            let staged = installation
                .stage(Path::new(value(args, "source")))
                .map_err(std::io::Error::other)?;
            let activation = installation
                .activate_with_launcher()
                .map_err(std::io::Error::other)?;
            let launcher = (activation.status == "active").then(|| installation.launcher_path());
            println!(
                "{}",
                serde_json::json!({"staged": staged, "activation": activation, "launcher": launcher})
            );
        }
        "doctor-core" => {
            let report = Installation::doctor(Path::new(value(args, "home")));
            println!("{}", serde_json::to_string(&report)?);
            if report.status == "error" {
                return Err("core installation health check failed".into());
            }
        }
        "intent-retire" => {
            let state = Path::new(value(args, "state-dir"));
            let release = ReleaseId::parse(value(args, "release-id"))?;
            let operation_id = value(args, "operation-id");
            let project = args.get_one::<String>("project-root");
            let report = match value(args, "kind") {
                "artifact" => {
                    let project =
                        project.ok_or("artifact preparation retirement requires --project-root")?;
                    mirket::engine::retire_expired_preparation(
                        state,
                        &release,
                        Path::new(project),
                        operation_id,
                    )
                }
                "memory" => {
                    let scope = match project {
                        Some(path) => mirket::memory_service::Selection::Project {
                            project_root: PathBuf::from(path),
                        },
                        None => mirket::memory_service::Selection::SharedPreferences,
                    };
                    mirket::memory_service::retire_expired_preparation(
                        state,
                        &release,
                        &scope,
                        operation_id,
                    )
                }
                _ => unreachable!("Clap restricts the selected preparation kind"),
            }
            .map_err(std::io::Error::other)?;
            println!("{report}");
        }
        "memory" => {
            let (operation, selected) = args.subcommand().expect("required memory subcommand");
            let scope = match selected.get_one::<String>("project-root") {
                Some(root) => mirket::memory_service::Selection::Project {
                    project_root: PathBuf::from(root),
                },
                None => mirket::memory_service::Selection::SharedPreferences,
            };
            let admin = mirket::memory_admin::MemoryAdmin::new(
                PathBuf::from(value(selected, "state-dir")),
                PathBuf::from(value(selected, "backup-dir")),
                ReleaseId::parse(value(selected, "release-id"))?,
                mirket::memory_backup::BackupPolicy::default(),
            )
            .map_err(std::io::Error::other)?;
            let report = if operation == "backups" {
                serde_json::to_value(admin.backups(&scope).map_err(std::io::Error::other)?)?
            } else if matches!(operation, "export-prepare" | "import-prepare") {
                admin
                    .transfer_prepare(
                        &scope,
                        Path::new(value(selected, "path")),
                        operation == "import-prepare",
                    )
                    .map_err(std::io::Error::other)?
            } else if matches!(
                operation,
                "transfer-status" | "transfer-commit" | "transfer-reconcile" | "transfer-retire"
            ) {
                let id = value(selected, "operation-id");
                match operation {
                    "transfer-status" => admin.transfer_status(&scope, id),
                    "transfer-commit" => admin.transfer_commit(&scope, id),
                    "transfer-reconcile" => admin.transfer_reconcile(&scope, id),
                    "transfer-retire" => admin.transfer_retire(&scope, id),
                    _ => unreachable!(),
                }
                .map_err(std::io::Error::other)?
            } else {
                let report = match operation {
                    "restore-prepare" => admin.prepare(&scope, value(selected, "snapshot-id")),
                    "restore-status" => admin.status(&scope, value(selected, "operation-id")),
                    "restore-commit" => admin.commit(&scope, value(selected, "operation-id")),
                    "restore-reconcile" => admin.reconcile(&scope, value(selected, "operation-id")),
                    "restore-retire" => {
                        admin.retire_expired(&scope, value(selected, "operation-id"))
                    }
                    _ => unreachable!(),
                }
                .map_err(std::io::Error::other)?;
                serde_json::to_value(report)?
            };
            println!("{}", report);
        }
        "serve" => {
            let release = ReleaseId::parse(value(args, "release-id"))?;
            let mut payload = AdapterPayload::explicit(Path::new(value(args, "node")))?;
            if let Some(script) = args.get_one::<String>("design-adapter") {
                let select = |path: &str| -> Result<PathBuf, std::io::Error> {
                    let selected = Path::new(path);
                    if !selected.is_absolute() || !selected.is_file() {
                        return Err(std::io::Error::other(
                            "design development payload requires explicit existing absolute files",
                        ));
                    }
                    selected.canonicalize()
                };
                payload.design = Some(mirket::design::DesignPayload {
                    node: payload.node.clone(),
                    adapter: select(script)?,
                    font: select(value(args, "design-font"))?,
                    raster_module: args
                        .get_one::<String>("raster-module")
                        .map(|path| select(path))
                        .transpose()?,
                });
            }
            if let Some(script) = args.get_one::<String>("openpencil-adapter") {
                let adapter = Path::new(script);
                let runtime_root = Path::new(value(args, "openpencil-runtime"));
                let sha256 = value(args, "openpencil-sha256");
                if !adapter.is_absolute()
                    || !adapter.is_file()
                    || !runtime_root.is_absolute()
                    || !runtime_root.is_dir()
                    || sha256.len() != 64
                    || !sha256
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                {
                    return Err(std::io::Error::other(
                        "OpenPencil development payload requires explicit existing files, directory and lowercase SHA-256",
                    ).into());
                }
                payload.openpencil = Some(mirket::openpencil::OpenPencilPayload {
                    node: payload.node.clone(),
                    adapter: adapter.canonicalize()?,
                    runtime_root: runtime_root.canonicalize()?,
                    runtime_sha256: sha256.into(),
                });
            }
            if let Some(script) = args.get_one::<String>("video-adapter") {
                let select = |path: &str| -> Result<PathBuf, std::io::Error> {
                    let selected = Path::new(path);
                    if !selected.is_absolute() || !selected.is_file() {
                        return Err(std::io::Error::other(
                            "video development payload requires explicit existing absolute files",
                        ));
                    }
                    selected.canonicalize()
                };
                payload.video = Some(mirket::video::VideoPayload {
                    node: payload.node.clone(),
                    adapter: select(script)?,
                    chromium: select(value(args, "video-chromium"))?,
                    ffmpeg: select(value(args, "video-ffmpeg"))?,
                    ffprobe: select(value(args, "video-ffprobe"))?,
                });
            }
            let engine = Engine::open(release, payload, PathBuf::from(value(args, "state-dir")))
                .map_err(std::io::Error::other)?;
            let embedding_runtime = args
                .get_one::<String>("embedding-package")
                .map(|path| {
                    mirket::memory_runtime::MemoryRuntime::new(
                        engine.state_root().to_owned(),
                        engine.release.clone(),
                        PathBuf::from(path),
                        std::env::current_exe().map_err(|e| e.to_string())?,
                    )
                    .map(Arc::new)
                })
                .transpose()
                .map_err(std::io::Error::other)?;
            let options = mirket::memory_service::MemoryOptions {
                backup_root: args.get_one::<String>("backup-dir").map(PathBuf::from),
                embedding_runtime,
            };
            serve(engine, options).await?;
        }
        "manifest" => {
            let id = distribution::write_manifest(
                Path::new(value(args, "directory")),
                env!("CARGO_PKG_VERSION"),
                distribution::native_target()?,
            )?;
            println!(
                "{}",
                serde_json::json!({"release_id":id.as_str(),"status":"manifest_created"})
            );
        }
        "home-init" => {
            ReleaseStore::initialize(Path::new(value(args, "home")))
                .map_err(std::io::Error::other)?;
            println!(
                "{}",
                serde_json::json!({"status":"initialized","kind":"mirket-installation"})
            );
        }
        "stage" => {
            let store = ReleaseStore::open(Path::new(value(args, "home")))
                .map_err(std::io::Error::other)?;
            println!(
                "{}",
                serde_json::to_string(
                    &store
                        .stage(Path::new(value(args, "source")))
                        .map_err(std::io::Error::other)?
                )?
            );
        }
        "activate" => {
            let store = ReleaseStore::open(Path::new(value(args, "home")))
                .map_err(std::io::Error::other)?;
            println!(
                "{}",
                serde_json::to_string(&store.activate().map_err(std::io::Error::other)?)?
            );
        }
        _ => unreachable!(),
    }
    Ok(())
}
fn run_skills(args: &clap::ArgMatches) -> Result<(), String> {
    let (operation, args) = args.subcommand().ok_or("skill operation is required")?;
    let installation = Installation::open(Path::new(value(args, "home")))?;
    let _operation_guard = if matches!(
        operation,
        "global-plan" | "global-doctor" | "project-plan" | "project-doctor"
    ) {
        None
    } else {
        Some(installation.lock_codex_operation()?)
    };
    let (release_id, catalog, qualified) = installation.skill_catalog()?;
    let manager = if operation.starts_with("global-") {
        SkillSync::global(
            &catalog,
            Path::new(value(args, "user-home")),
            &installation.codex_skills_receipt_path(),
        )?
    } else {
        SkillSync::project(&catalog, Path::new(value(args, "project-root")))?
    };
    let empty = BTreeSet::new();
    match operation {
        "global-plan" => print_json(&manager.plan(&release_id, None, &empty)?)?,
        "global-sync" => print_json(&manager.sync(&release_id, None, &empty)?)?,
        "global-doctor" => print_json(&manager.inspect()?)?,
        "global-reconcile" => print_json(&manager.reconcile()?)?,
        "global-remove" => print_json(&manager.remove_managed()?)?,
        "project-plan" | "project-select" | "project-update" => {
            let selection = ProjectSelection::resolve(
                &catalog,
                &selected_values(args, "profile"),
                &selected_values(args, "include"),
                &selected_values(args, "exclude"),
            )?;
            let plan = manager.plan(&release_id, Some(&selection), &qualified)?;
            if operation == "project-plan" {
                print_json(&plan)?;
            } else {
                if !plan.unavailable.is_empty() {
                    return Err("selected skills require unavailable private capabilities".into());
                }
                print_json(&manager.select_and_sync(
                    &release_id,
                    &selection,
                    &qualified,
                    operation == "project-update",
                )?)?;
            }
        }
        "project-sync" => {
            let selection = ProjectSelection::load(Path::new(value(args, "project-root")))?
                .ok_or("project has no saved skill selection")?;
            print_json(&manager.sync(&release_id, Some(&selection), &qualified)?)?;
        }
        "project-doctor" => print_json(&manager.inspect()?)?,
        "project-reconcile" => print_json(&manager.reconcile()?)?,
        "project-remove" => print_json(&manager.remove_managed()?)?,
        _ => return Err(format!("unsupported skill operation: {operation}")),
    }
    Ok(())
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string(value).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn selected_values(args: &clap::ArgMatches, name: &str) -> Vec<String> {
    args.get_many::<String>(name)
        .map(|values| values.cloned().collect())
        .unwrap_or_default()
}

fn value<'a>(args: &'a clap::ArgMatches, name: &str) -> &'a str {
    args.get_one::<String>(name).expect("required CLI argument")
}
async fn serve(
    engine: Arc<Engine>,
    options: mirket::memory_service::MemoryOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let (input, output) = rmcp::transport::stdio();
    let transport = rmcp::transport::async_rw::AsyncRwTransport::new(input, output)
        .with_max_line_length(1024 * 1024);
    let server = Server::new(engine.clone(), options);
    let connection = server.clone().serve(transport).await?;
    let outcome = connection.waiting().await;
    engine.shutdown().await;
    let memory_shutdown = server.shutdown().await;
    outcome?;
    memory_shutdown.map_err(std::io::Error::other)?;
    Ok(())
}

#[cfg(test)]
mod development_cli_tests {
    use super::*;

    fn development_args() -> Vec<&'static str> {
        vec![
            "mirket",
            "dev",
            "serve",
            "--state-dir",
            "/private/state",
            "--release-id",
            "test",
            "--node",
            "/private/node",
        ]
    }

    #[test]
    fn raster_development_payload_requires_selected_design_files() {
        let mut args = development_args();
        args.extend(["--raster-module", "/private/sharp.mjs"]);
        assert!(command().try_get_matches_from(args).is_err());
        let mut args = development_args();
        args.extend([
            "--design-adapter",
            "/private/design-main.js",
            "--raster-module",
            "/private/sharp.mjs",
        ]);
        assert!(command().try_get_matches_from(args).is_err());
        let mut args = development_args();
        args.extend([
            "--design-adapter",
            "/private/design-main.js",
            "--design-font",
            "/private/NotoSans.ttf",
            "--raster-module",
            "/private/sharp.mjs",
        ]);
        assert!(command().try_get_matches_from(args).is_ok());
    }

    #[test]
    fn development_command_rejects_removed_browser_and_sql_arguments() {
        for removed in [
            "--browser-adapter",
            "--browser-chromium",
            "--sql-python",
            "--sql-wheel",
        ] {
            let mut args = development_args();
            args.extend([removed, "/private/removed"]);
            assert_eq!(
                command().try_get_matches_from(args).unwrap_err().kind(),
                clap::error::ErrorKind::UnknownArgument
            );
        }
        let mut args = development_args();
        args.extend(["--unknown-option", "value"]);
        let error = command().try_get_matches_from(args).unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn setup_and_update_accept_an_optional_project_root() {
        for command_name in ["setup", "update"] {
            let args = [
                "mirket",
                command_name,
                "--home",
                "/private/mirket",
                "--source",
                "/private/release",
                "--user-home",
                "/private/user",
                "--project-root",
                "/private/project",
            ];
            assert!(command().try_get_matches_from(args).is_ok());
        }
    }

    #[test]
    fn uninstall_commands_require_both_explicit_roots_and_offer_no_data_deletion_flag() {
        for name in ["uninstall", "uninstall-status", "uninstall-reconcile"] {
            assert!(
                command()
                    .try_get_matches_from(["mirket", name, "--home", "/private/mirket"])
                    .is_err()
            );
            assert!(
                command()
                    .try_get_matches_from([
                        "mirket",
                        name,
                        "--home",
                        "/private/mirket",
                        "--user-home",
                        "/private/user"
                    ])
                    .is_ok()
            );
            assert!(
                command()
                    .try_get_matches_from([
                        "mirket",
                        name,
                        "--home",
                        "/private/mirket",
                        "--user-home",
                        "/private/user",
                        "--remove-data"
                    ])
                    .is_err()
            );
        }
    }

    #[test]
    fn reinstall_mutations_require_exact_source_and_status_is_read_only() {
        for name in ["reinstall", "reinstall-reconcile"] {
            let base = [
                "mirket",
                name,
                "--home",
                "/private/mirket",
                "--user-home",
                "/private/user",
            ];
            assert!(command().try_get_matches_from(base).is_err());
            let mut complete = base.to_vec();
            complete.extend(["--source", "/private/release"]);
            assert!(command().try_get_matches_from(&complete).is_ok());
            complete.push("--remove-data");
            assert!(command().try_get_matches_from(complete).is_err());
        }
        assert!(
            command()
                .try_get_matches_from([
                    "mirket",
                    "reinstall-status",
                    "--home",
                    "/private/mirket",
                    "--user-home",
                    "/private/user"
                ])
                .is_ok()
        );
        assert!(
            command()
                .try_get_matches_from([
                    "mirket",
                    "reinstall-status",
                    "--home",
                    "/private/mirket",
                    "--user-home",
                    "/private/user",
                    "--source",
                    "/private/release"
                ])
                .is_err()
        );
    }
}
