use clap::{Arg, Command};
use mirket::{
    distribution,
    engine::Engine,
    installation::Installation,
    release::{AdapterPayload, ReleaseId},
    release_store::ReleaseStore,
    server::Server,
};
use rmcp::ServiceExt;
use std::{
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
fn command() -> Command {
    Command::new("mirket")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Verified native capabilities for pi")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("setup").about("Set up a verified native payload in a new pi-owned home").arg(required("home")).arg(required("source")))
        .subcommand(Command::new("update").about("Stage and activate a verified native payload when idle").arg(required("home")).arg(required("source")))
        .subcommand(Command::new("doctor").about("Inspect native payload integrity; pi integration requires its loaded-session doctor").arg(required("home")))
        .subcommand(Command::new("verify-pi-host").about("Verify an explicit external pi closure without installing or executing it").arg(required("root")).arg(required("inventory")).arg(required("sha256")))
        .subcommand(Command::new("verify").about("Verify an explicit native payload without installing or executing it").arg(required("source")))
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
    if name == "verify-pi-host" {
        let report = mirket::pi_host::verify(
            Path::new(value(args, "root")),
            Path::new(value(args, "inventory")),
            value(args, "sha256"),
        )?;
        println!("{}", serde_json::to_string(&report)?);
        return Ok(());
    }
    if name == "verify" {
        let release = distribution::VerifiedRelease::open(Path::new(value(args, "source")))?;
        println!(
            "{}",
            serde_json::json!({"release_id": release.id().as_str(), "manifest": release.manifest()})
        );
        return Ok(());
    }
    if name == "setup" {
        let report = Installation::setup(
            Path::new(value(args, "home")),
            Path::new(value(args, "source")),
        )
        .map_err(std::io::Error::other)?;
        println!("{}", serde_json::to_string(&report)?);
        return Ok(());
    }
    if name == "update" {
        let installation =
            Installation::open(Path::new(value(args, "home"))).map_err(std::io::Error::other)?;
        let health = Installation::doctor(installation.home());
        if health.status != "healthy" {
            return Err("native installation is not healthy; preserve and inspect it".into());
        }
        let staged = installation
            .stage(Path::new(value(args, "source")))
            .map_err(std::io::Error::other)?;
        let activation = installation
            .activate_with_launcher()
            .map_err(std::io::Error::other)?;
        println!(
            "{}",
            serde_json::json!({"staged": staged, "activation": activation})
        );
        return Ok(());
    }
    if name == "doctor" {
        let report = Installation::doctor(Path::new(value(args, "home")));
        println!("{}", serde_json::to_string(&report)?);
        if report.status == "error" {
            return Err("native payload integrity check failed".into());
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
    fn native_setup_does_not_accept_host_configuration_or_legacy_commands() {
        assert!(
            command()
                .try_get_matches_from([
                    "mirket",
                    "setup",
                    "--home",
                    "/private/pi/mirket",
                    "--source",
                    "/private/payload"
                ])
                .is_ok()
        );
        assert!(
            command()
                .try_get_matches_from([
                    "mirket",
                    "setup",
                    "--home",
                    "/private/pi/mirket",
                    "--source",
                    "/private/payload",
                    "--user-home",
                    "/private/user"
                ])
                .is_err()
        );
        for removed in [
            "codex-register",
            "codex-remove",
            "codex-doctor",
            "codex-guidance-install",
            "skills",
            "uninstall",
            "reinstall",
        ] {
            assert!(command().try_get_matches_from(["mirket", removed]).is_err());
        }
    }
}
