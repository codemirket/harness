//! Advertise the compiled MCP catalog promptly for an installed launcher.
//! The bound session guard prevents activation while a single background task
//! verifies the complete installed release. Every tool call waits for that
//! result before opening or using project state; catalog discovery is not a
//! health claim.

use crate::{
    distribution::VerifiedRelease, engine::Engine, installed_payload::resolve_installed_payload,
    maintenance::SessionGuard, memory_runtime::MemoryRuntime, memory_service::MemoryOptions,
    release_store::BoundActiveRelease, server::Server,
};
use rmcp::{
    RoleServer, ServerHandler, ServiceExt,
    model::{
        CacheScope, CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock,
        ListToolsResult, PaginatedRequestParams, ProtocolVersion, ResultType, ServerCapabilities,
        ServerConfig, Tool,
    },
    service::RequestContext,
};
use std::{borrow::Cow, path::PathBuf, sync::Arc};
use tokio::sync::{OnceCell, watch};

struct ReadyRuntime {
    engine: Arc<Engine>,
    server: Arc<Server>,
}

#[derive(Clone)]
struct DeferredServer {
    verified: watch::Receiver<Option<Result<Arc<VerifiedRelease>, String>>>,
    runtime: Arc<OnceCell<Result<Arc<ReadyRuntime>, String>>>,
    state_root: PathBuf,
    home: PathBuf,
}

impl DeferredServer {
    async fn verified_release(&self) -> Result<Arc<VerifiedRelease>, String> {
        let mut receiver = self.verified.clone();
        loop {
            if let Some(result) = receiver.borrow_and_update().clone() {
                return result;
            }
            receiver.changed().await.map_err(|_| {
                "installed release verification stopped before readiness".to_owned()
            })?;
        }
    }

    async fn ready(&self) -> Result<Arc<ReadyRuntime>, String> {
        let release = self.verified_release().await?;
        self.runtime
            .get_or_init(|| async {
                open_verified(&release, self.state_root.clone(), &self.home).map(Arc::new)
            })
            .await
            .clone()
    }
}

impl ServerHandler for DeferredServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
    }

    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(ProtocolVersion::known_up_to(&ProtocolVersion::V_2025_11_25))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        Server::catalog_tool(name)
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        let supports_cache_hints = context
            .protocol_version()
            .is_some_and(|version| version >= ProtocolVersion::V_2026_07_28);
        Ok(ListToolsResult {
            result_type: Some(ResultType::COMPLETE),
            tools: Server::catalog_tools(),
            meta: None,
            next_cursor: None,
            ttl_ms: supports_cache_hints.then_some(0),
            cache_scope: supports_cache_hints.then_some(CacheScope::Public),
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, rmcp::ErrorData> {
        match self.ready().await {
            Ok(runtime) => runtime.server.call_tool(request, context).await,
            Err(message) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "installed release verification failed: {message}"
            ))])
            .into()),
        }
    }
}

fn verify_release(
    release_path: PathBuf,
    expected_release_id: String,
) -> Result<Arc<VerifiedRelease>, String> {
    let release = VerifiedRelease::open_installed(&release_path).map_err(|e| e.to_string())?;
    if release.id().as_str() != expected_release_id {
        return Err("active identity mismatch".into());
    }
    if !release.running_core_matches().map_err(|e| e.to_string())? {
        return Err("running core differs from the bound active release".into());
    }
    Ok(Arc::new(release))
}

fn open_verified(
    release: &VerifiedRelease,
    state_root: PathBuf,
    home: &std::path::Path,
) -> Result<ReadyRuntime, String> {
    // The outer connection-bound guard remains live while this second guard
    // binds the recipient choice to the verified release and engine startup.
    let session = SessionGuard::acquire(&state_root).map_err(|error| error.to_string())?;
    let payload = resolve_installed_payload(release, &state_root, &session)?;
    let engine = Engine::open(release.id().clone(), payload, state_root.clone())?;
    drop(session);
    let embedding_runtime = release
        .embedding_package()
        .map(|package| {
            MemoryRuntime::new(
                state_root.clone(),
                release.id().clone(),
                package,
                release.executable(),
            )
            .map(Arc::new)
        })
        .transpose()?;
    let server = Arc::new(Server::new(
        engine.clone(),
        MemoryOptions {
            backup_root: Some(home.join("backups")),
            embedding_runtime,
        },
    ));
    Ok(ReadyRuntime { engine, server })
}

pub async fn serve_bound(
    bound: BoundActiveRelease,
    home: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = watch::channel(None);
    let verification = tokio::task::spawn_blocking({
        let release_path = bound.release_path.clone();
        let release_id = bound.release_id.clone();
        move || {
            let outcome = verify_release(release_path, release_id);
            sender.send_replace(Some(outcome));
        }
    });
    let (input, output) = rmcp::transport::stdio();
    let transport = rmcp::transport::async_rw::AsyncRwTransport::new(input, output)
        .with_max_line_length(1024 * 1024);
    let catalog = DeferredServer {
        verified: receiver,
        runtime: Arc::new(OnceCell::new()),
        state_root: bound.state_root.clone(),
        home,
    };
    let connection = catalog.clone().serve(transport).await;
    let outcome: Result<(), String> = match connection {
        Ok(connection) => connection
            .waiting()
            .await
            .map(|_| ())
            .map_err(|e| e.to_string()),
        Err(error) => Err(error.to_string()),
    };
    verification.await?;
    if let Some(Ok(runtime)) = catalog.runtime.get() {
        runtime.engine.shutdown().await;
        runtime
            .server
            .shutdown()
            .await
            .map_err(std::io::Error::other)?;
    }
    drop(bound);
    outcome.map_err(std::io::Error::other)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn first_tool_waits_for_verification_result() {
        let (sender, receiver) = watch::channel(None);
        let catalog = DeferredServer {
            verified: receiver,
            runtime: Arc::new(OnceCell::new()),
            state_root: PathBuf::new(),
            home: PathBuf::new(),
        };
        assert!(
            tokio::time::timeout(Duration::from_millis(20), catalog.verified_release())
                .await
                .is_err()
        );
        sender.send_replace(Some(Err("tampered runtime".into())));
        assert_eq!(
            catalog.verified_release().await.err().as_deref(),
            Some("tampered runtime")
        );
        assert!(catalog.runtime.get().is_none());
    }

    #[test]
    fn deferred_catalog_is_the_live_server_catalog() {
        let tools = Server::catalog_tools();
        assert!(tools.iter().any(|tool| tool.name == "runtime_status"));
        assert!(tools.iter().any(|tool| tool.name == "design_prepare"));
        for tool in &tools {
            assert_eq!(Server::catalog_tool(&tool.name), Some(tool.clone()));
        }
    }
}
