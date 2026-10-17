//! Stdio MCP coordination and the Mirket-owned capability-routed spawner.
use crate::{
    catalog::Catalog,
    paths::Paths,
    state::{State, contracts::ContractKind},
};
use anyhow::{Context, Result, bail};
use rmcp::{
    ErrorData, RoleServer, ServerHandler, ServiceExt,
    handler::server::wrapper::Parameters,
    model::{
        CallToolResult, Implementation, ListResourceTemplatesResult, ListResourcesResult,
        PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResponse,
        ReadResourceResult, Resource, ResourceContents, ResourceTemplate, ServerCapabilities,
        ServerConfig,
    },
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context as TaskContext, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, ReadBuf},
    sync::Semaphore,
};

pub const MAX_FRAME_BYTES: usize = 128 * 1024;
pub const MAX_RESULT_BYTES: usize = 64 * 1024;
const GUIDE: &str = "Mirket supplies professional guidance, optional team briefs, embedded references and durable task evidence. Use catalog_search/catalog_show to discover useful expertise; skill_read and resources return embedded material, never fetch upstream skills. The catalog is not exhaustive: the host/model can seek other sources, combine methods or use none. User/project requirements, source rights and host permissions govern. team_facets/team_plan help when independent assignments are useful; plans and read receipts do not establish execution. Keep sufficient existing workflows authoritative.

Use task tracking when coordination, resumption or user/project requirements warrant it. For a Mirket task, deliver selected guidance, checkpoint meaningful decisions and retain acceptance/failure-probe artifacts. Mutations require the expected revision and an idempotency key; after interruption inspect task_status/history and retry an uncertain operation with the same key and arguments. A cancelled request may still commit. Optional typed contracts validate declared bindings and coverage at completion; hashes establish identity, not correctness or authenticated worker testimony. Operation descriptions specify limits and receipt semantics. Use spawn_subagent for Mirket-owned workers: it enforces dynamically discovered model capabilities, task/tool tier floors, generous effort and upward-only failover. route_task previews eligibility; model_registry reports discovery evidence. Independent native host spawners remain outside this boundary. Host permissions still govern effects; Mirket exposes no arbitrary shell through MCP.";

#[derive(Clone)]
pub struct MirketServer {
    paths: Paths,
    state: State,
    catalog: Catalog,
    workers: Arc<Semaphore>,
}

impl MirketServer {
    pub fn new(paths: &Paths, catalog: Catalog) -> Result<Self> {
        Ok(Self {
            paths: paths.clone(),
            state: State::open(paths)?,
            catalog,
            workers: Arc::new(Semaphore::new(4)),
        })
    }

    fn registered_worker_project(&self, path: &std::path::Path) -> Result<std::path::PathBuf> {
        let canonical = crate::paths::canonical_project(path)?;
        self.state.project_id(&canonical)?;
        Ok(canonical)
    }

    async fn run(
        &self,
        operation: impl FnOnce(State, Catalog) -> Result<Value> + Send + 'static,
    ) -> CallToolResult {
        let Ok(permit) = self.workers.clone().try_acquire_owned() else {
            return failure("Mirket is busy; retry this operation with the same idempotency key");
        };
        let state = self.state.clone();
        let catalog = self.catalog.clone();
        let work = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            operation(state, catalog)
        });
        match tokio::time::timeout(Duration::from_secs(10), work).await {
            Ok(Ok(Ok(value))) => match serde_json::to_vec(&value) {
                Ok(bytes) if bytes.len() <= MAX_RESULT_BYTES => CallToolResult::structured(value),
                _ => failure("Result exceeds 64 KiB; request a narrower selection or page"),
            },
            Ok(Ok(Err(error))) => failure(&format!("{error:#}")),
            Ok(Err(_)) => failure("Mirket worker failed; inspect task_status before retrying"),
            Err(_) => failure(
                "Request deadline exceeded; the operation may finish. Read task_status or retry with the same idempotency key",
            ),
        }
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ModelQuery {
    #[serde(default)]
    refresh: bool,
    #[serde(default)]
    offset: usize,
    #[serde(default = "page_size")]
    limit: usize,
}

fn failure(message: &str) -> CallToolResult {
    let message: String = message.chars().take(2048).collect();
    CallToolResult::structured_error(json!({"error":message}))
}

fn coordination_policy(mut tool: rmcp::model::Tool) -> rmcp::model::Tool {
    let min_tier = if tool.name == "spawn_subagent" {
        "T2_SYNTHESIS"
    } else {
        "T1_DETERMINISTIC"
    };
    tool.meta.get_or_insert_with(Default::default).insert(
        "mirket/policy".into(),
        json!({"min_tier":min_tier,"requires_vision":false}),
    );
    tool
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Search {
    /// Short capability or skill keywords, at most 512 bytes.
    query: String,
    #[serde(default)]
    offset: usize,
    /// Number of results, from 1 to 20.
    #[serde(default = "page_size")]
    limit: usize,
    /// Optional exact family/platform/interaction/constraint IDs from the design
    /// coverage registry. Reports scoped gaps; never validates the current task.
    design_context: Option<crate::catalog::DesignContext>,
}
fn page_size() -> usize {
    10
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ProjectPage {
    #[serde(default)]
    offset: usize,
    /// Number of projects, from 1 to 10.
    #[serde(default = "page_size")]
    limit: usize,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Plan {
    /// Capability name or role alias, such as Backend Engineer or CFO.
    capability: String,
    /// Up to 12 supporting capability names or role aliases, such as Security Engineer.
    /// Optional skill IDs belong in skill_read, not here. Omit unrelated expertise.
    #[serde(default)]
    #[schemars(length(max = 12))]
    supporting: Vec<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct TeamPlan {
    /// The actual user outcome, at most 4096 bytes. The host interprets its scope.
    objective: String,
    /// Lead capability name or role alias owning the integrated outcome.
    primary: String,
    /// Applicable task boundaries from team_facets, each with a concrete reason.
    #[serde(default)]
    #[schemars(length(max = 8))]
    facets: Vec<crate::catalog::FacetSelection>,
    /// Additional explicitly chosen capability names; aliases are deduplicated.
    #[serde(default)]
    #[schemars(length(max = 12))]
    supporting: Vec<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct SkillSelection {
    /// One exact catalog skill ID, obtained from discovery.
    #[schemars(length(min = 1, max = 128))]
    skill_id: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct SkillRead {
    skill_id: String,
    /// Embedded path, usually SKILL.md or a file under references/ or assets/.
    #[serde(default = "skill_path")]
    path: String,
    /// Set task_id, revision and key together to record delivery of the selected SKILL.md.
    task_id: Option<String>,
    revision: Option<i64>,
    key: Option<String>,
}
fn skill_path() -> String {
    "SKILL.md".into()
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Start {
    /// A project ID returned by project_list. Roots can only be registered by the CLI.
    project_id: String,
    objective: String,
    /// One lead skill and only the support required by actual boundaries (1..16 IDs).
    skills: Vec<String>,
    /// Observable result that the host must verify on the saved artifact or application.
    acceptance: String,
    /// Discriminating negative case that can detect a convincing but incorrect result.
    failure_probe: String,
    /// Stable idempotency key for this operation, at most 128 bytes.
    key: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Task {
    /// Read one task by ID, or omit and supply project_id to discover interrupted tasks.
    task_id: Option<String>,
    project_id: Option<String>,
    /// Opaque next_cursor from a previous page. Task updates do not move pages; newly inserted tasks are not a frozen snapshot.
    cursor: Option<String>,
    #[serde(default = "page_size")]
    limit: usize,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct History {
    task_id: String,
    cursor: Option<String>,
    #[serde(default = "history_limit")]
    limit: usize,
}
fn history_limit() -> usize {
    10
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Validate {
    task_id: String,
    path: String,
    contract: ContractKind,
    #[serde(default)]
    completion: bool,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    task_id: String,
    revision: i64,
    note: String,
    key: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Evidence {
    task_id: String,
    revision: i64,
    /// Existing regular file under the task's registered root. Absolute, parent and symlink paths are rejected.
    path: String,
    kind: EvidenceKind,
    /// Opt-in structured receipt. Completion rechecks bindings and successful coverage.
    contract: Option<ContractKind>,
    /// What the host observed, the command or artifact checked, and its result. This is caller testimony.
    summary: String,
    key: String,
}
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum EvidenceKind {
    Acceptance,
    FailureProbe,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Finish {
    task_id: String,
    revision: i64,
    action: FinishAction,
    reason: Option<String>,
    key: String,
}
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum FinishAction {
    Complete,
    Reopen,
}

#[tool_router]
impl MirketServer {
    #[tool(
        description = "List live dynamically discovered models and their verified/unknown capabilities. Refresh bypasses the TTL cache; this does not execute inference.",
        annotations(read_only_hint = true, open_world_hint = true)
    )]
    async fn model_registry(&self, Parameters(args): Parameters<ModelQuery>) -> CallToolResult {
        if args.limit == 0 || args.limit > 20 {
            return failure("limit must be 1..=20");
        }
        match crate::router::discovery::discover(&self.paths, args.refresh).await {
            Ok(registry) => CallToolResult::structured(
                json!({"models":registry.models.iter().skip(args.offset).take(args.limit).collect::<Vec<_>>(),"total":registry.models.len(),"targets":registry.targets,"refreshed_at":registry.refreshed_at}),
            ),
            Err(error) => failure(&format!("{error:#}")),
        }
    }

    #[tool(
        description = "Resolve task complexity, owned tool floors, modalities and generous inference effort. Returns eligible dynamic models without spawning a worker. Raw model IDs and caller-provided tool policies are rejected.",
        annotations(read_only_hint = true, open_world_hint = true)
    )]
    async fn route_task(
        &self,
        Parameters(args): Parameters<crate::router::SpawnRequest>,
    ) -> CallToolResult {
        match crate::router::execution::plan(&self.paths, &args).await {
            Ok(value) => CallToolResult::structured(value),
            Err(error) => failure(&format!("{error:#}")),
        }
    }

    #[tool(
        description = "Spawn a Mirket-owned worker through dynamic capability routing. Declare task intent, complexity, assigned tool names and project. Enforces tier/vision floors, high reasoning, upward-only failover and post-mutation verification. May write assigned project/design artifacts; no arbitrary shell or native nested agents. Independent host spawners remain outside this boundary.",
        annotations(read_only_hint = false, open_world_hint = true)
    )]
    async fn spawn_subagent(
        &self,
        Parameters(mut args): Parameters<crate::router::SpawnRequest>,
    ) -> CallToolResult {
        let Ok(project) = self.registered_worker_project(&args.project) else {
            return failure(
                "MCP workers require a CLI-registered project root; register it with mirket project register before dispatch",
            );
        };
        args.project = project;
        let Ok(_permit) = self.workers.clone().try_acquire_owned() else {
            return failure(
                "Mirket worker capacity is occupied; inspect existing runs before retrying",
            );
        };
        match crate::router::execution::spawn(&self.paths, args).await {
            Ok(mut value) => {
                if value["result"]
                    .as_str()
                    .is_some_and(|text| text.len() > 32 * 1024)
                {
                    let preview = value["result"]
                        .as_str()
                        .unwrap()
                        .chars()
                        .take(4096)
                        .collect::<String>();
                    value["result"] = json!({"preview":preview,"full_result":"Read the saved trace file for the complete result"});
                }
                CallToolResult::structured(value)
            }
            Err(error) => failure(&format!("{error:#}")),
        }
    }

    #[tool(
        description = "Find reviewed skills by keyword with bounded pagination and guidance routes. Optional design_context checks the complete requested design combination against scoped reference coverage, exposing gaps without claiming task validation. Read the design-inspiration coverage registry for exact axis IDs. Use catalog_show for pins and caveats.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn catalog_search(&self, Parameters(args): Parameters<Search>) -> CallToolResult {
        self.run(move |_, catalog| {
            if args.query.len() > 512 || args.offset > 1_000_000 || !(1..=20).contains(&args.limit)
            {
                bail!("query, offset or page size exceeds its limit");
            }
            catalog.search_with_design_context(
                &args.query,
                args.offset,
                args.limit,
                args.design_context.as_ref(),
            )
        })
        .await
    }

    #[tool(
        description = "Inspect one selected skill's source pin, dependencies, adaptation, caveats and read route without fetching a payload or executing tools. Upstream and reference entries are not readable via skill_read.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn catalog_show(&self, Parameters(args): Parameters<SkillSelection>) -> CallToolResult {
        self.run(move |_, catalog| {
            if args.skill_id.is_empty() || args.skill_id.len() > 128 {
                bail!("selected skill ID must contain 1..=128 bytes");
            }
            catalog.describe(&args.skill_id)
        })
        .await
    }

    #[tool(
        description = "Resolve a role or capability into concrete lead skills, prerequisites, acceptance evidence and failure probes. A role grants expertise, not authority.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn capability_plan(&self, Parameters(args): Parameters<Plan>) -> CallToolResult {
        self.run(move |_, catalog| {
            if args.capability.len() > 128 || args.supporting.iter().any(|item| item.len() > 128) {
                bail!("capability selection exceeds its limit");
            }
            catalog.capability_plan(&args.capability, &args.supporting)
        })
        .await
    }

    #[tool(
        description = "Inspect task boundaries and their complementary professional assignments. Interpret the actual request and sources, then select only applicable facets with concrete reasons in team_plan.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn team_facets(&self) -> CallToolResult {
        self.run(move |_, catalog| Ok(catalog.team_facets())).await
    }

    #[tool(
        description = "Build a task-scoped professional team from a lead role and justified task facets. Returns distinct perspectives, methods, assignment briefs, checks and task_start inputs. Dispatch Mirket workers with spawn_subagent; this planning tool does not run models or grant permissions.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn team_plan(&self, Parameters(args): Parameters<TeamPlan>) -> CallToolResult {
        self.run(move |_, catalog| {
            catalog.team_plan(
                &args.objective,
                &args.primary,
                &args.facets,
                &args.supporting,
            )
        })
        .await
    }

    #[tool(
        description = "Read an embedded skill body, reference or source asset. Supply task_id, revision and key together to record delivery of a selected SKILL.md. Delivery means content was returned, not that the agent applied it.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn skill_read(&self, Parameters(args): Parameters<SkillRead>) -> CallToolResult {
        self.run(move |state,catalog| {
            if args.skill_id.len()>128 || args.path.len()>512 { bail!("skill id or path exceeds its limit"); }
            let doc = catalog.read(&args.skill_id,&args.path,48*1024)?;
            let mut output = json!({"skill_id":doc.id,"path":doc.path,"sha256":doc.sha256,"content":doc.content});
            match (args.task_id,args.revision,args.key) {
                (None,None,None) => {},
                (Some(task),Some(revision),Some(key)) if args.path == "SKILL.md" => {
                    let receipt = state.record_skill(&task,revision,&args.skill_id,output["sha256"].as_str().context("invalid skill hash")?,&key)?;
                    output["delivery"] = json!({"task_id":receipt["id"],"revision":receipt["revision"],"meaning":"Selected skill content delivered; application remains the host agent's responsibility"});
                },
                _ => bail!("provide task_id, revision and key together for SKILL.md delivery"),
            }
            Ok(output)
        }).await
    }

    #[tool(
        description = "List roots explicitly registered through mirket CLI. MCP cannot register or expand project roots.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn project_list(&self, Parameters(args): Parameters<ProjectPage>) -> CallToolResult {
        self.run(move |state, _| state.projects_page(args.offset, args.limit))
            .await
    }

    #[tool(
        description = "Start a durable task in a registered project with selected skills and explicit acceptance and failure-probe criteria. Writes only the Mirket state database.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn task_start(&self, Parameters(args): Parameters<Start>) -> CallToolResult {
        self.run(move |state, _| {
            state.begin(
                &args.project_id,
                &args.objective,
                &args.skills,
                &args.acceptance,
                &args.failure_probe,
                &args.key,
            )
        })
        .await
    }

    #[tool(
        description = "Read a durable task by task_id, or list task IDs newest-created first with project_id and an optional returned cursor. Follow next_cursor; updates do not move tasks between pages. Resume here after interruption.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn task_status(&self, Parameters(args): Parameters<Task>) -> CallToolResult {
        self.run(move |state, _| match (args.task_id, args.project_id) {
            (Some(task), None) => state.status(&task),
            (None, Some(project)) => state.tasks(&project, args.cursor.as_deref(), args.limit),
            _ => bail!("provide exactly one of task_id or project_id"),
        })
        .await
    }

    #[tool(
        description = "Retrieve all retained task checkpoints in revision-descending pages of 1..20 items. Follow next_cursor to recover older constraints; task_status contains only the recent eight.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn task_history(&self, Parameters(args): Parameters<History>) -> CallToolResult {
        self.run(move |state, _| {
            state.task_history(&args.task_id, args.cursor.as_deref(), args.limit)
        })
        .await
    }

    #[tool(
        description = "Validate an explicit verification or contributions JSON contract without recording it. Checks current declared file bindings; completion mode also requires passed results and accepted or reasoned waived coverage. Never executes checks or authenticates worker testimony.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn task_validate(&self, Parameters(args): Parameters<Validate>) -> CallToolResult {
        self.run(move |state, _| {
            state.validate_contract(&args.task_id, &args.path, args.contract, args.completion)
        })
        .await
    }

    #[tool(
        description = "Record decisions, constraints, evidence and the next action for an active task. Use the current revision and a stable idempotency key. Returns a compact versioned receipt; task_status and task_history recover retained notes.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn task_checkpoint(&self, Parameters(args): Parameters<Checkpoint>) -> CallToolResult {
        self.run(move |state, _| {
            state.checkpoint(&args.task_id, args.revision, &args.note, &args.key)
        })
        .await
    }

    #[tool(
        description = "Record SHA-256 identity of an existing acceptance or failure-probe artifact (maximum 16 MiB) under the registered project. The host must run and inspect the actual checks; this tool never executes commands.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn task_evidence(&self, Parameters(args): Parameters<Evidence>) -> CallToolResult {
        self.run(move |state, _| {
            state.evidence_with_contract(
                &args.task_id,
                args.revision,
                &args.path,
                match args.kind {
                    EvidenceKind::Acceptance => "acceptance",
                    EvidenceKind::FailureProbe => "failure_probe",
                },
                &args.summary,
                &args.key,
                args.contract,
            )
        })
        .await
    }

    #[tool(
        description = "Complete a task only when all selected guidance was delivered and acceptance/failure-probe artifacts still match current bytes. This checks evidence identity, not semantic correctness. Reopen a completed task with a reason when further work is needed.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn task_finish(&self, Parameters(args): Parameters<Finish>) -> CallToolResult {
        self.run(move |state, _| match args.action {
            FinishAction::Complete => {
                if args.reason.is_some() {
                    bail!("reason is only accepted when reopening a task");
                }
                state.complete(&args.task_id, args.revision, &args.key)
            }
            FinishAction::Reopen => state.reopen(
                &args.task_id,
                args.revision,
                &args.reason.context("reopen requires a reason")?,
                &args.key,
            ),
        })
        .await
    }
}

#[tool_handler]
impl ServerHandler for MirketServer {
    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, ErrorData> {
        let tools = Self::tool_router()
            .list_all()
            .into_iter()
            .map(coordination_policy)
            .collect();
        Ok(rmcp::model::ListToolsResult::with_all_items(tools))
    }

    fn get_tool(&self, name: &str) -> Option<rmcp::model::Tool> {
        Self::tool_router()
            .get(name)
            .cloned()
            .map(coordination_policy)
    }

    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new("mirket", env!("CARGO_PKG_VERSION")))
        .with_instructions(GUIDE)
    }

    async fn list_resources(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        if request.and_then(|p| p.cursor).is_some() {
            return Err(ErrorData::invalid_params(
                "no resource cursor is available",
                None,
            ));
        }
        Ok(ListResourcesResult::with_all_items(vec![
            Resource::new("mirket://guide", "Mirket coordination guide")
                .with_mime_type("text/plain"),
        ]))
    }

    async fn list_resource_templates(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        if request.and_then(|p| p.cursor).is_some() {
            return Err(ErrorData::invalid_params(
                "no resource cursor is available",
                None,
            ));
        }
        Ok(ListResourceTemplatesResult::with_all_items(vec![
            ResourceTemplate::new(
                "mirket://skills/{skill_id}/{+path}",
                "Embedded skill body, reference or source asset",
            ),
        ]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        let uri = request.uri;
        if uri == "mirket://guide" {
            return Ok(ReadResourceResult::new(vec![ResourceContents::text(GUIDE, uri)]).into());
        }
        let (skill_id, path) = uri
            .strip_prefix("mirket://skills/")
            .and_then(|suffix| suffix.split_once('/'))
            .filter(|(id, path)| id.len() <= 128 && path.len() <= 512)
            .ok_or_else(|| ErrorData::resource_not_found(uri.clone(), None))?;
        let doc = self
            .catalog
            .read(skill_id, path, 48 * 1024)
            .map_err(|error| {
                ErrorData::resource_not_found(
                    uri.clone(),
                    Some(json!({"reason":error.to_string()})),
                )
            })?;
        let mime_type = skill_resource_mime(path);
        Ok(ReadResourceResult::new(vec![
            ResourceContents::text(doc.content, uri).with_mime_type(mime_type),
        ])
        .into())
    }
}

fn skill_resource_mime(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "md" | "markdown" => "text/markdown",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" | "mjs" => "text/javascript",
        "svg" => "image/svg+xml",
        "json" => "application/json",
        _ => "text/plain",
    }
}

/// Limits each incoming newline-delimited frame before the SDK buffers it.
pub struct BoundedLines<R> {
    inner: R,
    bytes: usize,
    maximum: usize,
    failed: bool,
}

fn frame_limit_error(maximum: usize) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("MCP frame exceeds {maximum} bytes"),
    )
}

impl<R> BoundedLines<R> {
    pub fn new(inner: R) -> Self {
        Self::with_limit(inner, MAX_FRAME_BYTES)
    }

    /// Construct a frame limiter for a boundary with a caller-specific limit.
    /// The default constructor preserves the coordination MCP contract.
    pub(crate) fn with_limit(inner: R, maximum: usize) -> Self {
        Self {
            inner,
            bytes: 0,
            maximum,
            failed: false,
        }
    }
}
impl<R: AsyncRead + Unpin> AsyncRead for BoundedLines<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if this.failed {
            return Poll::Ready(Err(frame_limit_error(this.maximum)));
        }
        let start = buffer.filled().len();
        match Pin::new(&mut this.inner).poll_read(cx, buffer) {
            Poll::Ready(Ok(())) => {
                for byte in &buffer.filled()[start..] {
                    if *byte == b'\n' {
                        this.bytes = 0;
                    } else {
                        this.bytes += 1;
                    }
                    if this.bytes > this.maximum {
                        this.failed = true;
                        buffer.set_filled(start);
                        return Poll::Ready(Err(frame_limit_error(this.maximum)));
                    }
                }
                Poll::Ready(Ok(()))
            }
            other => other,
        }
    }
}

pub async fn serve(paths: Paths, catalog: Catalog) -> Result<()> {
    let discovery_paths = paths.clone();
    let discovery = tokio::spawn(async move {
        let _ = crate::router::discovery::discover(&discovery_paths, false).await;
    });
    let service = MirketServer::new(&paths, catalog)?
        .serve((BoundedLines::new(tokio::io::stdin()), tokio::io::stdout()))
        .await?;
    service.waiting().await?;
    discovery.abort();
    Ok(())
}

/// Exercise the actual CLI's stdio boundary with the official MCP client.
pub async fn protocol_probe(
    binary: &std::path::Path,
    home: &std::path::Path,
    iterations: usize,
) -> Result<Value> {
    use rmcp::{model::CallToolRequestParams, transport::TokioChildProcess};
    use std::time::Instant;
    if !(1..=10_000).contains(&iterations) {
        bail!("iterations must be from 1 to 10000");
    }
    let started = Instant::now();
    let mut command = tokio::process::Command::new(binary);
    command
        .arg("--home")
        .arg(home)
        .arg("mcp")
        .arg("serve")
        .kill_on_drop(true);
    let transport = TokioChildProcess::new(command)?;
    let client = tokio::time::timeout(Duration::from_secs(15), ().serve(transport))
        .await
        .context("MCP startup deadline exceeded")??;
    let startup_ms = started.elapsed().as_secs_f64() * 1000.0;
    let operation = async {
        let tools = client.list_all_tools().await?;
        let names: Vec<String> = tools.iter().map(|tool| tool.name.to_string()).collect();
        let expected = [
            "model_registry",
            "route_task",
            "spawn_subagent",
            "catalog_search",
            "catalog_show",
            "capability_plan",
            "team_facets",
            "team_plan",
            "skill_read",
            "project_list",
            "task_start",
            "task_status",
            "task_history",
            "task_validate",
            "task_checkpoint",
            "task_evidence",
            "task_finish",
        ];
        if names.len() != expected.len()
            || expected
                .iter()
                .any(|name| !names.iter().any(|found| found == name))
        {
            bail!("MCP tool discovery differs from the Mirket contract");
        }
        let request = || {
            CallToolRequestParams::new("catalog_search").with_arguments(
                json!({"query":"engineering","limit":3})
                    .as_object()
                    .unwrap()
                    .clone(),
            )
        };
        let mut latencies = Vec::with_capacity(iterations);
        let request_start = Instant::now();
        for _ in 0..iterations {
            let start = Instant::now();
            let result = tokio::time::timeout(Duration::from_secs(12), client.call_tool(request()))
                .await
                .context("MCP request deadline exceeded")??;
            if result.is_error == Some(true) || result.structured_content.is_none() {
                bail!("catalog request did not produce useful structured output");
            }
            latencies.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        let elapsed_seconds = request_start.elapsed().as_secs_f64();
        let mut concurrent = tokio::task::JoinSet::new();
        for _ in 0..4 {
            let peer = client.peer().clone();
            let args = request();
            concurrent.spawn(async move { peer.call_tool(args).await });
        }
        while let Some(result) = concurrent.join_next().await {
            if result??.is_error == Some(true) {
                bail!("concurrent catalog request failed");
            }
        }
        let selected = client
            .call_tool(
                CallToolRequestParams::new("catalog_show").with_arguments(
                    json!({"skill_id":"ecc-rust-testing"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
            .await?;
        if selected.is_error == Some(true)
            || selected.structured_content.as_ref().is_none_or(|value| {
                value["guidance_access"]["body_readable"] != false
                    || value["source_pin"]["commit"]
                        .as_str()
                        .is_none_or(|pin| pin.len() != 40)
                    || !value["caveats"].is_array()
            })
        {
            bail!("selected upstream metadata did not establish its pinned read route");
        }
        let upstream = client
            .call_tool(
                CallToolRequestParams::new("skill_read").with_arguments(
                    json!({"skill_id":"ecc-rust-testing"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
            .await?;
        if upstream.is_error != Some(true) {
            bail!("embedded-only skill_read accepted an upstream payload");
        }
        let read = client
            .call_tool(
                CallToolRequestParams::new("skill_read").with_arguments(
                    json!({"skill_id":"engineering-judgment"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
            .await?;
        if read.is_error == Some(true)
            || read
                .structured_content
                .as_ref()
                .and_then(|value| value["content"].as_str())
                .is_none_or(str::is_empty)
        {
            bail!("skill delivery produced no content");
        }
        let malformed = client
            .call_tool(
                CallToolRequestParams::new("catalog_search")
                    .with_arguments(json!({"query":42}).as_object().unwrap().clone()),
            )
            .await;
        if malformed.is_ok()
            && !malformed
                .as_ref()
                .is_ok_and(|value| value.is_error == Some(true))
        {
            bail!("malformed input was accepted");
        }
        let inaccessible = client
            .read_resource(ReadResourceRequestParams::new("file:///etc/passwd"))
            .await;
        if inaccessible.is_ok() {
            bail!("arbitrary file resource was accepted");
        }
        let resource = client
            .read_resource(ReadResourceRequestParams::new(
                "mirket://skills/engineering-judgment/SKILL.md",
            ))
            .await?;
        let resource_bytes = serde_json::to_vec(&resource)?.len();
        if !(100..=MAX_FRAME_BYTES).contains(&resource_bytes) {
            bail!("resource content has an unexpected size");
        }
        latencies.sort_by(f64::total_cmp);
        let percentile = |p: f64| {
            latencies[((latencies.len() as f64 * p).ceil() as usize)
                .saturating_sub(1)
                .min(latencies.len() - 1)]
        };
        Ok::<Value, anyhow::Error>(
            json!({"ok":true,"transport":"stdio","sdk":"rmcp 3.5.1","iterations":iterations,
        "startup_ms":startup_ms,"request_median_ms":percentile(0.5),"request_p95_ms":percentile(0.95),
        "sequential_requests_per_second":iterations as f64/elapsed_seconds,"concurrent_requests":4,
        "tools":names,"skill_delivery":true,"resource_read":true,"malformed_input_rejected":true,"arbitrary_resource_rejected":true,
        "selected_metadata_read_route":true,"upstream_body_rejected":true,
        "measurement":"One spawned process; startup includes initialization. Request samples use catalog_search after discovery. Values describe this run, not a performance guarantee."}),
        )
    };
    let outcome = tokio::time::timeout(Duration::from_secs(60), operation).await;
    let cleanup = tokio::time::timeout(Duration::from_secs(5), client.cancel()).await;
    let value = outcome.context("MCP probe deadline exceeded")??;
    cleanup.context("MCP probe cleanup deadline exceeded")??;
    Ok(value)
}

#[cfg(all(test, unix))]
mod router_boundary_tests {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn registered_worker_keeps_the_resolved_root_after_original_symlink_changes() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
        let registered = paths.home.join("registered");
        let unrelated = paths.home.join("unregistered");
        let alias = paths.home.join("alias");
        std::fs::create_dir(&registered).unwrap();
        std::fs::create_dir(&unrelated).unwrap();
        let server = MirketServer::new(&paths, Catalog::embedded().unwrap()).unwrap();
        server.state.register_project(&registered).unwrap();
        symlink(&registered, &alias).unwrap();
        let resolved = server.registered_worker_project(&alias).unwrap();
        std::fs::remove_file(&alias).unwrap();
        symlink(&unrelated, &alias).unwrap();
        assert_eq!(resolved, registered);
        assert!(server.registered_worker_project(&alias).is_err());
        assert!(server.state.project_id(&resolved).is_ok());
    }
}
