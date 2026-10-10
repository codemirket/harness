use crate::{
    catalog::Catalog,
    install::{self, SetupOptions, Target},
    paths::Paths,
    project,
    state::State,
};
use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::{Value, json};
use std::{
    io::{self, IsTerminal, Write},
    path::PathBuf,
};

#[derive(Parser)]
#[command(
    name = "mirket",
    version,
    about = "Local expertise and workflow coordination for Codex and Claude",
    long_about = "Mirket installs reviewed skills, coordinates evidence-backed work and checks your local harness. Start with mirket setup."
)]
struct Cli {
    /// Use this existing user-home directory for all harness and client state.
    #[arg(long, global = true)]
    home: Option<PathBuf>,
    /// Emit structured JSON for command results.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Report this executable's release version and compilation target.
    Info,
    /// Interactively set up Codex Desktop, Claude Desktop Code, or both.
    Setup(SetupArgs),
    /// Check installed files, configuration, tools, and optionally a project.
    Doctor {
        #[arg(long)]
        project: Option<PathBuf>,
    },
    /// Update Mirket and reapply your saved environment setup.
    Update {
        #[arg(long)]
        check: bool,
        #[arg(long)]
        from: Option<PathBuf>,
        #[arg(long, requires = "from")]
        sha256: Option<String>,
    },
    /// Find, inspect, and prepare reviewed expertise.
    Catalog {
        #[command(subcommand)]
        command: CatalogCommand,
    },
    /// Compose focused expertise from an outcome or professional role.
    Capabilities {
        #[command(subcommand)]
        command: CapabilityCommand,
    },
    /// Register a project and install only its selected expertise.
    Project {
        #[command(subcommand)]
        command: ProjectCommand,
    },
    /// Coordinate durable work, skill delivery, and artifact evidence.
    Task {
        #[command(subcommand)]
        command: TaskCommand,
    },
    /// Serve the local Model Context Protocol over standard input/output.
    Mcp {
        #[command(subcommand)]
        command: McpCommand,
    },
    /// Register and invoke additional local executables under host permissions.
    Tool {
        #[command(subcommand)]
        command: ToolCommand,
    },
    /// Deterministic checks for authored artifacts.
    Craft {
        #[command(subcommand)]
        command: CraftCommand,
    },
    /// Build, check, and package a Mirket source checkout.
    Dev {
        #[command(subcommand)]
        command: DevCommand,
    },
    /// Measure local startup and catalog latency with a reproducible workload.
    Benchmark {
        #[arg(long, default_value_t = 200)]
        iterations: usize,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum TargetChoice {
    Codex,
    Claude,
    All,
}
impl TargetChoice {
    fn targets(self) -> Vec<Target> {
        match self {
            Self::Codex => vec![Target::Codex],
            Self::Claude => vec![Target::Claude],
            Self::All => vec![Target::Codex, Target::Claude],
        }
    }
}
#[derive(Args)]
struct SetupArgs {
    #[arg(long, value_enum)]
    target: Option<TargetChoice>,
    /// Apply explicit or saved choices without an interactive prompt.
    #[arg(long)]
    yes: bool,
    /// Validate setup and show the proposed installation without writing it.
    #[arg(long)]
    dry_run: bool,
    #[arg(long, hide = true)]
    expected_state: Option<String>,
    /// Enable the Microsoft Learn documentation server.
    #[arg(long,num_args=0..=1,default_missing_value="true",action=clap::ArgAction::Set)]
    microsoft_learn: Option<bool>,
}
#[derive(Subcommand)]
enum CatalogCommand {
    Search {
        #[arg(default_value = "")]
        query: String,
        #[arg(long, default_value_t = 0)]
        offset: usize,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    Show {
        id: String,
    },
    Read {
        id: String,
        #[arg(long, default_value = "SKILL.md")]
        path: String,
        #[arg(long, default_value_t = 65536)]
        max_bytes: usize,
    },
    Profiles,
    /// Download and verify a selected pinned payload without executing it.
    Fetch {
        id: String,
    },
    Check,
}
#[derive(Subcommand)]
enum CapabilityCommand {
    List,
    Show {
        name: String,
    },
    Plan {
        name: String,
        #[arg(long = "with")]
        supporting: Vec<String>,
    },
}
#[derive(Args)]
struct ProjectPath {
    #[arg(long, default_value = ".")]
    project: PathBuf,
}
#[derive(Subcommand)]
enum ProjectCommand {
    Init {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long, value_enum, default_value = "all")]
        target: TargetChoice,
    },
    Add {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        profile: Vec<String>,
        #[arg(long)]
        skill: Vec<String>,
        #[arg(long)]
        capability: Vec<String>,
        #[arg(long, value_enum)]
        target: Option<TargetChoice>,
    },
    Remove {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        profile: Vec<String>,
        #[arg(long)]
        skill: Vec<String>,
        #[arg(long)]
        capability: Vec<String>,
        #[arg(long, value_enum)]
        target: Option<TargetChoice>,
    },
    Configure {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long, value_enum)]
        target: TargetChoice,
    },
    Plan(ProjectPath),
    Sync {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        prune: bool,
    },
    Doctor(ProjectPath),
    Register(ProjectPath),
    /// Delete a project's coordination records, keeping its files and selections.
    Unregister {
        /// Registered project ID from `mirket project list`.
        id: String,
        /// Confirm deletion of the project's task history and receipts.
        #[arg(long, required = true)]
        yes: bool,
    },
    List {
        #[arg(long, default_value_t = 0)]
        offset: usize,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
}
#[derive(Args)]
struct Mutation {
    task: String,
    #[arg(long)]
    revision: i64,
    #[arg(long)]
    key: String,
}
#[derive(Subcommand)]
enum TaskCommand {
    List {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long, default_value_t = 0)]
        offset: usize,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    Start {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        objective: String,
        #[arg(long)]
        capability: String,
        #[arg(long = "with")]
        supporting: Vec<String>,
        #[arg(long)]
        key: String,
    },
    Status {
        task: String,
    },
    Checkpoint {
        #[command(flatten)]
        mutation: Mutation,
        #[arg(long)]
        note: String,
    },
    Read {
        #[command(flatten)]
        mutation: Mutation,
        skill: String,
    },
    Evidence {
        #[command(flatten)]
        mutation: Mutation,
        path: String,
        #[arg(long, value_enum)]
        kind: EvidenceKind,
        #[arg(long)]
        summary: String,
    },
    Complete {
        #[command(flatten)]
        mutation: Mutation,
    },
    Reopen {
        #[command(flatten)]
        mutation: Mutation,
        #[arg(long)]
        reason: String,
    },
}
#[derive(Clone, Copy, ValueEnum)]
enum EvidenceKind {
    Acceptance,
    FailureProbe,
}
#[derive(Subcommand)]
enum McpCommand {
    Serve,
    Doctor {
        #[arg(long, default_value_t = 50)]
        iterations: usize,
    },
}
#[derive(Subcommand)]
enum ToolCommand {
    List,
    Doctor,
    Register {
        name: String,
        #[arg(long)]
        executable: PathBuf,
    },
    Remove {
        name: String,
    },
    Run {
        name: String,
        #[arg(long, default_value = ".")]
        cwd: PathBuf,
        #[arg(long, default_value_t = 300)]
        timeout: u64,
        #[arg(last = true)]
        args: Vec<String>,
    },
}
#[derive(Subcommand)]
enum CraftCommand {
    Contrast {
        foreground: String,
        background: String,
    },
}
#[derive(Subcommand)]
enum DevCommand {
    Fmt {
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
    Check {
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
    /// Run focused tests; --ignored explicitly enables opt-in tests such as downloads.
    Test {
        filter: Option<String>,
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        ignored: bool,
    },
    Build {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        release: bool,
    },
    Dist {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Verify native release artifacts and combine their update manifests.
    Manifest {
        #[arg(long = "input", required = true)]
        inputs: Vec<PathBuf>,
        #[arg(long)]
        output: PathBuf,
    },
}

fn runtime() -> Result<tokio::runtime::Runtime> {
    Ok(tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?)
}

pub fn entry(installer_target: Option<&str>) {
    let mut args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let basename = args
        .first()
        .and_then(|s| std::path::Path::new(s).file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or("mirket");
    // Every standalone installer carries the complete CLI. Its installed name is mirket.
    if basename.starts_with("install-")
        && !args.iter().any(|arg| arg == "--version")
        && let Some(target) = installer_target
    {
        args.splice(1..1, ["setup".into(), "--target".into(), target.into()]);
    }
    if std::env::var_os("MIRKET_DIRECT").is_none() {
        let mut home = None;
        let mut arguments = args.iter().skip(1);
        while let Some(argument) = arguments.next() {
            if argument == "--" {
                break;
            }
            if argument == "--home" {
                home = arguments.next().map(PathBuf::from);
            } else if let Some(value) = argument.to_str().and_then(|s| s.strip_prefix("--home=")) {
                home = Some(PathBuf::from(value));
            }
        }
        // A bootstrap installed on PATH continues to the verified managed release.
        // Invalid installation state is left to doctor/setup for actionable diagnosis.
        if let Ok(paths) = Paths::discover(home)
            && let Ok(Some(binary)) = install::newer_runtime(&paths)
        {
            match std::process::Command::new(binary)
                .args(args.iter().skip(1))
                .env("MIRKET_DIRECT", "1")
                .status()
            {
                Ok(status) => std::process::exit(status.code().unwrap_or(1)),
                Err(error) => {
                    eprintln!("mirket: could not start the installed runtime: {error}");
                    std::process::exit(1);
                }
            }
        }
    }
    let cli = Cli::parse_from(args);
    let as_json = cli.json;
    match dispatch(cli) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            if as_json {
                eprintln!("{}", json!({"ok":false,"error":format!("{error:#}")}));
            } else {
                eprintln!("mirket: {error:#}");
            }
            std::process::exit(1);
        }
    }
}

fn emit(value: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn ask(prompt: &str, default: &str) -> Result<String> {
    print!("{prompt} [{default}]: ");
    io::stdout().flush()?;
    let mut answer = String::new();
    if io::stdin().read_line(&mut answer)? == 0 {
        bail!("setup cancelled: input closed");
    }
    let answer = answer.trim();
    Ok(if answer.is_empty() {
        default.to_string()
    } else {
        answer.to_string()
    })
}

fn setup(paths: &Paths, catalog: &Catalog, args: SetupArgs, as_json: bool) -> Result<i32> {
    let saved = install::saved_options(paths)?;
    let mut options = SetupOptions {
        targets: args
            .target
            .map(TargetChoice::targets)
            .or_else(|| saved.as_ref().map(|s| s.targets.clone()))
            .unwrap_or_else(|| TargetChoice::All.targets()),
        microsoft_learn: args
            .microsoft_learn
            .or_else(|| saved.as_ref().map(|s| s.microsoft_learn))
            .unwrap_or(false),
    };
    if args.dry_run {
        emit(&install::plan(paths, catalog, &options)?)?;
        return Ok(0);
    }
    if !args.yes {
        if !io::stdin().is_terminal() || as_json {
            bail!("interactive setup needs a terminal; use --yes with --target codex|claude|all");
        }
        println!(
            "\nMirket setup\nInstall reviewed skills and a local coordination server.\nHome: {}\nRuntime: {}\n",
            paths.home.display(),
            paths.binary().display()
        );
        if args.target.is_none() {
            let default = if options.targets.len() == 2 {
                "1"
            } else if options.targets.first() == Some(&Target::Codex) {
                "2"
            } else {
                "3"
            };
            loop {
                options.targets = match ask(
                    "Target: 1) Codex + Claude  2) Codex  3) Claude",
                    default,
                )?
                .as_str()
                {
                    "1" => TargetChoice::All.targets(),
                    "2" => TargetChoice::Codex.targets(),
                    "3" => TargetChoice::Claude.targets(),
                    _ => {
                        println!("Enter 1, 2, or 3.");
                        continue;
                    }
                };
                break;
            }
        }
        if args.microsoft_learn.is_none() {
            loop {
                options.microsoft_learn = match ask(
                    "Enable Microsoft Learn documentation?",
                    if options.microsoft_learn { "yes" } else { "no" },
                )?
                .to_lowercase()
                .as_str()
                {
                    "yes" | "y" => true,
                    "no" | "n" => false,
                    _ => {
                        println!("Enter yes or no.");
                        continue;
                    }
                };
                break;
            }
        }
        println!(
            "\nTargets: {}\nOpenAI Docs: enabled\nMicrosoft Learn: {}\nSetup writes managed skill copies, shared instructions and MCP entries. Existing file conflicts stop setup for inspection.",
            options
                .targets
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", "),
            if options.microsoft_learn {
                "enabled"
            } else {
                "disabled"
            }
        );
        if !matches!(
            ask("Apply this setup?", "yes")?.to_lowercase().as_str(),
            "yes" | "y"
        ) {
            println!("Setup cancelled.");
            return Ok(0);
        }
    }
    let report = install::setup_checked(paths, catalog, &options, args.expected_state.as_deref())?;
    if as_json {
        emit(&report)?;
    } else {
        println!(
            "Mirket {} ready: {} managed files, {} changed.\nRuntime: {}\nRun mirket doctor, then open a new agent session.",
            report.version,
            report.files,
            report.changed,
            report.binary.display()
        );
        println!(
            "Add {} to your PATH if mirket is not already on it.",
            report.binary.parent().context("binary parent")?.display()
        );
    }
    Ok(0)
}

fn dispatch(cli: Cli) -> Result<i32> {
    let paths = Paths::discover(cli.home)?;
    if let Command::Mcp {
        command: McpCommand::Serve,
    } = cli.command
    {
        runtime()?.block_on(crate::mcp::serve(paths, Catalog::embedded()?))?;
        return Ok(0);
    }
    if let Command::Mcp {
        command: McpCommand::Doctor { iterations },
    } = cli.command
    {
        let report = runtime()?.block_on(crate::mcp::protocol_probe(
            &std::env::current_exe()?,
            &paths.home,
            iterations,
        ))?;
        emit(&report)?;
        return Ok(0);
    }
    if let Command::Tool {
        command:
            ToolCommand::Run {
                name,
                cwd,
                timeout,
                args,
            },
    } = cli.command
    {
        return runtime()?.block_on(crate::tools::run(&paths, &name, &cwd, &args, timeout));
    }
    let catalog = Catalog::embedded()?;
    let value = match cli.command {
        Command::Info => {
            json!({"name":"mirket","version":env!("CARGO_PKG_VERSION"),"target":env!("MIRKET_TARGET")})
        }
        Command::Setup(args) => return setup(&paths, &catalog, args, cli.json),
        Command::Doctor { project } => {
            let report = install::doctor(&paths, &catalog)?;
            let tools = crate::tools::doctor(&paths)?;
            let project_report = project
                .as_ref()
                .map(|p| project::doctor(&catalog, p))
                .transpose()?;
            let protocol = if report.ok {
                match runtime()?.block_on(crate::mcp::protocol_probe(
                    &paths.binary(),
                    &paths.home,
                    3,
                )) {
                    Ok(result) => json!({"ok":true,"result":result}),
                    Err(error) => json!({"ok":false,"error":format!("{error:#}")}),
                }
            } else {
                json!({"ok":false,"status":"not_run","reason":"fix installation integrity checks first"})
            };
            let ok = report.ok
                && protocol["ok"] == true
                && tools["ok"] == true
                && project_report.as_ref().is_none_or(|p| p["ok"] == true);
            if cli.json {
                emit(
                    &json!({"ok":ok,"installation":report,"mcp":protocol,"tools":tools,"project":project_report}),
                )?;
            } else {
                println!(
                    "Mirket doctor: {}",
                    if ok { "healthy" } else { "issues found" }
                );
                for check in report.checks {
                    println!("  {}  {} — {}", check.status, check.name, check.message);
                }
                if protocol["ok"] == true {
                    println!(
                        "  pass  local MCP — handshake, discovery, tool calls and error probes passed."
                    );
                } else {
                    emit(&protocol)?;
                }
                if tools["ok"] != true {
                    emit(&tools)?;
                }
                if let Some(project) = project_report {
                    emit(&project)?;
                }
            }
            return Ok(if ok { 0 } else { 1 });
        }
        Command::Update {
            check,
            from,
            sha256,
        } => serde_json::to_value(crate::update::update(
            &paths,
            &crate::update::UpdateOptions {
                from,
                sha256,
                check,
            },
        )?)?,
        Command::Catalog { command } => match command {
            CatalogCommand::Search {
                query,
                offset,
                limit,
            } => catalog.search(&query, offset, limit)?,
            CatalogCommand::Show { id } => serde_json::to_value(catalog.skill(&id)?)?,
            CatalogCommand::Read {
                id,
                path,
                max_bytes,
            } => {
                let document = catalog.read(&id, &path, max_bytes)?;
                if !cli.json {
                    print!("{}", document.content);
                    return Ok(0);
                }
                serde_json::to_value(document)?
            }
            CatalogCommand::Profiles => serde_json::to_value(catalog.profiles())?,
            CatalogCommand::Fetch { id } => {
                let payload = catalog.prepare(&id, &paths.cache())?;
                json!({"id":id,"sha256":payload.sha256,"files":payload.files.len(),"executed":false})
            }
            CatalogCommand::Check => catalog.check()?,
        },
        Command::Capabilities { command } => match command {
            CapabilityCommand::List => serde_json::to_value(catalog.capabilities())?,
            CapabilityCommand::Show { name } => serde_json::to_value(catalog.capability(&name)?)?,
            CapabilityCommand::Plan { name, supporting } => {
                catalog.capability_plan(&name, &supporting)?
            }
        },
        Command::Project { command } => match command {
            ProjectCommand::Init { project, target } => {
                let result = project::init(
                    &catalog,
                    &project,
                    &target
                        .targets()
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>(),
                )?;
                let registration = State::open(&paths)?.register_project(&project)?;
                json!({"project":result,"registration":registration})
            }
            ProjectCommand::Add {
                project,
                profile,
                skill,
                capability,
                target,
            } => {
                let target = target.and_then(|t| match t {
                    TargetChoice::Codex => Some("codex"),
                    TargetChoice::Claude => Some("claude"),
                    TargetChoice::All => None,
                });
                project::add(&catalog, &project, &profile, &skill, &capability, target)?
            }
            ProjectCommand::Plan(args) => project::plan(&catalog, &args.project)?,
            ProjectCommand::Remove {
                project,
                profile,
                skill,
                capability,
                target,
            } => project::remove(
                &catalog,
                &project,
                &profile,
                &skill,
                &capability,
                target.and_then(|t| match t {
                    TargetChoice::Codex => Some("codex"),
                    TargetChoice::Claude => Some("claude"),
                    TargetChoice::All => None,
                }),
            )?,
            ProjectCommand::Configure { project, target } => project::set_targets(
                &catalog,
                &project,
                &target
                    .targets()
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            )?,
            ProjectCommand::Sync { project, prune } => {
                project::sync_with_options(&catalog, &project, &paths.cache(), prune)?
            }
            ProjectCommand::Doctor(args) => project::doctor(&catalog, &args.project)?,
            ProjectCommand::Register(args) => {
                State::open(&paths)?.register_project(&args.project)?
            }
            ProjectCommand::Unregister { id, .. } => {
                State::open(&paths)?.unregister_project(&id)?
            }
            ProjectCommand::List { offset, limit } => {
                State::open(&paths)?.projects_page(offset, limit)?
            }
        },
        Command::Task { command } => {
            let state = State::open(&paths)?;
            match command {
                TaskCommand::List {
                    project,
                    offset,
                    limit,
                } => {
                    let project = crate::paths::canonical_project(&project)?;
                    let project_id = state.project_id(&project)?;
                    state.tasks(&project_id, offset, limit)?
                }
                TaskCommand::Start {
                    project,
                    objective,
                    capability,
                    supporting,
                    key,
                } => {
                    let project = crate::paths::canonical_project(&project)?;
                    let project_id = state.project_id(&project)?;
                    let primary = catalog.capability(&capability)?;
                    let mut skills = vec![primary.lead.clone()];
                    for role in &supporting {
                        let lead = catalog.capability(role)?.lead.clone();
                        if !skills.contains(&lead) {
                            skills.push(lead);
                        }
                    }
                    state.begin(
                        &project_id,
                        &objective,
                        &skills,
                        &primary.acceptance.join("; "),
                        &primary.failure_probe,
                        &key,
                    )?
                }
                TaskCommand::Status { task } => state.status(&task)?,
                TaskCommand::Checkpoint { mutation, note } => {
                    state.checkpoint(&mutation.task, mutation.revision, &note, &mutation.key)?
                }
                TaskCommand::Read { mutation, skill } => {
                    let document = catalog.read(&skill, "SKILL.md", 65536)?;
                    let receipt = state.record_skill(
                        &mutation.task,
                        mutation.revision,
                        &skill,
                        &document.sha256,
                        &mutation.key,
                    )?;
                    json!({"receipt":receipt,"document":document})
                }
                TaskCommand::Evidence {
                    mutation,
                    path,
                    kind,
                    summary,
                } => state.evidence(
                    &mutation.task,
                    mutation.revision,
                    &path,
                    match kind {
                        EvidenceKind::Acceptance => "acceptance",
                        EvidenceKind::FailureProbe => "failure_probe",
                    },
                    &summary,
                    &mutation.key,
                )?,
                TaskCommand::Complete { mutation } => {
                    state.complete(&mutation.task, mutation.revision, &mutation.key)?
                }
                TaskCommand::Reopen { mutation, reason } => {
                    state.reopen(&mutation.task, mutation.revision, &reason, &mutation.key)?
                }
            }
        }
        Command::Tool { command } => match command {
            ToolCommand::List => crate::tools::list(&paths)?,
            ToolCommand::Doctor => crate::tools::doctor(&paths)?,
            ToolCommand::Register { name, executable } => {
                crate::tools::register(&paths, &name, &executable)?
            }
            ToolCommand::Remove { name } => crate::tools::remove(&paths, &name)?,
            ToolCommand::Run { .. } => unreachable!(),
        },
        Command::Craft {
            command:
                CraftCommand::Contrast {
                    foreground,
                    background,
                },
        } => crate::craft::contrast(&foreground, &background)?,
        Command::Dev { command } => match command {
            DevCommand::Fmt { project } => crate::dev::format(&project)?,
            DevCommand::Check { project } => crate::dev::check(&project)?,
            DevCommand::Test {
                project,
                filter,
                ignored,
            } => crate::dev::test(&project, filter.as_deref(), ignored)?,
            DevCommand::Build { project, release } => crate::dev::build(&project, release)?,
            DevCommand::Dist { project, output } => crate::dev::dist(&project, &output)?,
            DevCommand::Manifest { inputs, output } => crate::release::manifest(&inputs, &output)?,
        },
        Command::Benchmark { iterations } => crate::dev::benchmark(&catalog, iterations)?,
        Command::Mcp { .. } => unreachable!(),
    };
    let failed = value.get("ok") == Some(&Value::Bool(false));
    emit(&value)?;
    Ok(if failed { 1 } else { 0 })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cli_definitions_are_consistent() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
    #[test]
    fn setup_and_update_contracts() {
        assert!(Cli::try_parse_from(["mirket", "setup", "--yes", "--target", "all"]).is_ok());
        assert!(Cli::try_parse_from(["mirket", "setup", "--target", "other"]).is_err());
        assert!(Cli::try_parse_from(["mirket", "update", "--sha256", "bad"]).is_err());
    }
}
