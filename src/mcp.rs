//! Stdio MCP access to Mirket's coordination API. Execution belongs to the host.
use crate::{catalog::Catalog, paths::Paths, state::State};
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
const GUIDE: &str = "Mirket coordinates expertise, task state and verification evidence. Call capability_plan to choose a lead and conditional support, then skill_read for the selected guidance. Use project_list to find a root registered by the CLI. For substantial work call task_start, read each selected skill with its task/revision/key, checkpoint progress, record actual acceptance and failure-probe artifact paths, and task_finish. Run commands with the host's tools and permissions. Mirket never launches a shell or a model. Evidence hashes establish identity at verification time; caller summaries and skill delivery receipts do not prove semantic correctness or agent cognition. Register a missing root through `mirket project register --project PATH` under host permissions. All mutations require an idempotency key and expected revision. After interruption read task_status; retry an uncertain operation with the same key and same arguments. A cancelled or timed-out request can finish a short database transaction; do not assume it was rolled back. Resources expose embedded skill files only. Request bodies are limited to 128 KiB, structured results to 64 KiB, task snapshots to 60 KiB before commit, and four blocking operations may run concurrently.";

#[derive(Clone)]
pub struct MirketServer {
    state: State,
    catalog: Catalog,
    workers: Arc<Semaphore>,
}

impl MirketServer {
    pub fn new(paths: &Paths, catalog: Catalog) -> Result<Self> {
        Ok(Self {
            state: State::open(paths)?,
            catalog,
            workers: Arc::new(Semaphore::new(4)),
        })
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

fn failure(message: &str) -> CallToolResult {
    let message: String = message.chars().take(2048).collect();
    CallToolResult::structured_error(json!({"error":message}))
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
    /// Conditional support selected for actual task boundaries; omit unrelated expertise.
    #[serde(default)]
    supporting: Vec<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct SkillRead {
    skill_id: String,
    /// Embedded path, usually SKILL.md or a referenced file under references/.
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
    #[serde(default)]
    offset: usize,
    #[serde(default = "page_size")]
    limit: usize,
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
        description = "Find reviewed skills and capabilities by keyword. Results are paginated; retrieve selected bodies with skill_read.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn catalog_search(&self, Parameters(args): Parameters<Search>) -> CallToolResult {
        self.run(move |_, catalog| {
            if args.query.len() > 512 || args.offset > 1_000_000 || !(1..=20).contains(&args.limit)
            {
                bail!("query, offset or page size exceeds its limit");
            }
            catalog.search(&args.query, args.offset, args.limit)
        })
        .await
    }

    #[tool(
        description = "Resolve a role or capability into concrete lead skills, prerequisites, acceptance evidence and failure probes. A role grants expertise, not authority.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn capability_plan(&self, Parameters(args): Parameters<Plan>) -> CallToolResult {
        self.run(move |_, catalog| {
            if args.capability.len() > 128
                || args.supporting.len() > 16
                || args.supporting.iter().any(|item| item.len() > 128)
            {
                bail!("capability selection exceeds its limit");
            }
            catalog.capability_plan(&args.capability, &args.supporting)
        })
        .await
    }

    #[tool(
        description = "Read an embedded skill body or reference. Supply task_id, revision and key together to record delivery of a selected SKILL.md. Delivery means content was returned, not that the agent applied it.",
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
        description = "Read a durable task by task_id, or list task IDs in a registered project with project_id and pagination. Resume here after interruption.",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn task_status(&self, Parameters(args): Parameters<Task>) -> CallToolResult {
        self.run(move |state, _| match (args.task_id, args.project_id) {
            (Some(task), None) => state.status(&task),
            (None, Some(project)) => state.tasks(&project, args.offset, args.limit),
            _ => bail!("provide exactly one of task_id or project_id"),
        })
        .await
    }

    #[tool(
        description = "Record decisions, constraints, evidence and the next action for an active task. Use the current revision and a stable idempotency key.",
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
            state.evidence(
                &args.task_id,
                args.revision,
                &args.path,
                match args.kind {
                    EvidenceKind::Acceptance => "acceptance",
                    EvidenceKind::FailureProbe => "failure_probe",
                },
                &args.summary,
                &args.key,
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
                "Embedded skill body or reference",
            )
            .with_mime_type("text/markdown"),
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
        Ok(ReadResourceResult::new(vec![
            ResourceContents::text(doc.content, uri).with_mime_type("text/markdown"),
        ])
        .into())
    }
}

/// Limits each incoming newline-delimited frame before the SDK buffers it.
pub struct BoundedLines<R> {
    inner: R,
    bytes: usize,
    failed: bool,
}
impl<R> BoundedLines<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            bytes: 0,
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
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "MCP frame exceeds 128 KiB",
            )));
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
                    if this.bytes > MAX_FRAME_BYTES {
                        this.failed = true;
                        buffer.set_filled(start);
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "MCP frame exceeds 128 KiB",
                        )));
                    }
                }
                Poll::Ready(Ok(()))
            }
            other => other,
        }
    }
}

pub async fn serve(paths: Paths, catalog: Catalog) -> Result<()> {
    let service = MirketServer::new(&paths, catalog)?
        .serve((BoundedLines::new(tokio::io::stdin()), tokio::io::stdout()))
        .await?;
    service.waiting().await?;
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
            "catalog_search",
            "capability_plan",
            "skill_read",
            "project_list",
            "task_start",
            "task_status",
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
        "measurement":"One spawned process; startup includes initialization. Request samples use catalog_search after discovery. Values describe this run, not a performance guarantee."}),
        )
    };
    let outcome = tokio::time::timeout(Duration::from_secs(60), operation).await;
    let cleanup = tokio::time::timeout(Duration::from_secs(5), client.cancel()).await;
    let value = outcome.context("MCP probe deadline exceeded")??;
    cleanup.context("MCP probe cleanup deadline exceeded")??;
    Ok(value)
}
