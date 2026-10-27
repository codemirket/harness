//! Read-only selected-source semantics use the connection's finite task owner.
use super::{Control, Engine, err};
use crate::code_navigation::{NavigateRequest, Payload, SourceRead};
use serde::Deserialize;
use serde_json::Value;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

#[derive(Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CodeNavigateArgs {
    pub expected_release_id: String,
    pub project_root: PathBuf,
    pub request: NavigateRequest,
}
struct CancelOnDrop(Arc<AtomicBool>);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}
impl Engine {
    pub async fn navigate_code(
        self: &Arc<Self>,
        args: CodeNavigateArgs,
        request_cancelled: impl std::future::Future<Output = ()> + Send,
    ) -> Result<Value, String> {
        self.guard(&args.expected_release_id)?;
        crate::code_navigation::validate_input(&args.request)?;
        let configured = self.payload.design.as_ref().ok_or(
            "private code navigation runtime is not configured; no host Node fallback is used",
        )?;
        let payload = Payload {
            node: configured.node.clone(),
            adapter: configured
                .adapter
                .with_file_name(crate::code_navigation::ADAPTER_NAME),
        };
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let sequence = NEXT
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| "code navigation request identity capacity exhausted")?;
        let job_id = format!("op-code-navigation-{}-{sequence:x}", self.owner_id);
        let cancel = Arc::new(AtomicBool::new(false));
        let _cancel_on_drop = CancelOnDrop(cancel.clone());
        let (sender, mut receiver) = tokio::sync::oneshot::channel();
        {
            let mut registry = self.running.lock().map_err(err)?;
            if registry.closed {
                return Err("connection is shutting down; no new navigation may start".into());
            }
            registry
                .active
                .retain(|_, control| !control.handle.is_finished());
            if registry.active.len() >= 8 {
                return Err("connection job capacity is busy".into());
            }
            let this = self.clone();
            let task_id = job_id.clone();
            let task_cancel = cancel.clone();
            let handle = tokio::spawn(async move {
                let result = this
                    .execute_code_navigation(args, task_id, payload, task_cancel)
                    .await;
                let _ = sender.send(result);
            });
            registry.active.insert(
                job_id,
                Control {
                    cancel: cancel.clone(),
                    handle,
                },
            );
        }
        tokio::select! { biased;
            _ = request_cancelled => { cancel.store(true, Ordering::Release);
                receiver.await.map_err(|_| "code navigation ownership task ended without a result".to_owned())??;
                Err("code navigation cancelled after cleanup".into()) },
            result = &mut receiver => result.map_err(|_| "code navigation ownership task ended without a result".to_owned())?,
        }
    }
    async fn execute_code_navigation(
        &self,
        args: CodeNavigateArgs,
        job_id: String,
        payload: Payload,
        cancel: Arc<AtomicBool>,
    ) -> Result<Value, String> {
        let release = self.release.as_str().to_owned();
        let snapshot = tokio::task::spawn_blocking(move || {
            crate::code_navigation::read_sources(&args.project_root, args.request, release, job_id)
        })
        .await
        .map_err(|_| "code navigation source snapshot failed")??;
        if cancel.load(Ordering::Acquire) {
            return Err("code navigation cancelled before dispatch".into());
        }
        let snapshot = match snapshot {
            SourceRead::Current(snapshot) => *snapshot,
            SourceRead::Changed(response) => return Ok(response),
        };
        let result = crate::worker::run_code_navigation_owned(
            &std::env::current_exe().map_err(err)?,
            crate::worker::CodeNavigationWork {
                state_root: self.state_root.clone(),
                payload,
                request: snapshot.request.clone(),
            },
            cancel.clone(),
        )
        .await
        .map_err(err)?
        .result;
        if cancel.load(Ordering::Acquire) {
            return Err("code navigation cancelled after cleanup".into());
        }
        tokio::task::spawn_blocking(move || snapshot.finish(result))
            .await
            .map_err(|_| "code navigation source verification failed")?
    }
}
