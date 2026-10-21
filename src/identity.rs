//! Identity belongs to the running process, not the current bytes at its old path.
use crate::{catalog::Catalog, paths::Paths, util};
use serde_json::{Value, json};
use std::sync::OnceLock;

static LOADED: OnceLock<Option<String>> = OnceLock::new();

pub fn capture() -> &'static Option<String> {
    LOADED.get_or_init(|| {
        let path = std::env::current_exe().ok()?;
        util::read_bounded(&path, 256 * 1024 * 1024)
            .ok()
            .map(|bytes| util::sha256(&bytes))
    })
}

pub fn version() -> String {
    format!(
        "{}+{}",
        env!("CARGO_PKG_VERSION"),
        &env!("MIRKET_SOURCE_SHA256")[..12]
    )
}

pub(crate) fn info(paths: &Paths, catalog: &Catalog) -> Value {
    let mut info = status(paths, catalog);
    info["name"] = json!("mirket");
    info["version"] = json!(env!("CARGO_PKG_VERSION"));
    info["target"] = json!(env!("MIRKET_TARGET"));
    info
}

pub fn status(paths: &Paths, catalog: &Catalog) -> Value {
    let loaded = capture();
    let installed = util::read_bounded(&paths.binary(), 256 * 1024 * 1024)
        .ok()
        .map(|bytes| util::sha256(&bytes));
    let matches_process = loaded.as_ref().zip(installed.as_ref()).map(|(a, b)| a == b);
    let selected = crate::agent_runtime::enabled(paths).ok();
    let mut actions = Vec::new();
    if matches_process == Some(false) {
        actions.push("Running and installed binaries differ. Reconnect this client after an authorized update, then compare runtime_status again.");
    } else if matches_process.is_none() {
        actions.push("Executable identity is incomplete. Use mirket doctor to diagnose the installation before relying on its state.");
    }
    if selected.is_none() {
        actions.push("Saved package selection could not be read. Use mirket doctor; do not assume memory or durable execution is available.");
    }
    json!({
        "process": {"version":env!("CARGO_PKG_VERSION"), "source_sha256":env!("MIRKET_SOURCE_SHA256"),
            "target":env!("MIRKET_TARGET"), "pid":std::process::id(), "executable_sha256":loaded},
        "catalog": {"sha256":catalog.fingerprint(), "skills":catalog.skills().len(), "profiles":catalog.profiles().len()},
        "installed_binary": {"path":paths.binary(), "sha256":installed,
            "matches_process": matches_process},
        "agent_runtime": {"selected":selected,
            "expected_recipe":crate::packages::agent::recipe(),
            "expected_versions":crate::packages::agent::definition().ok().map(|v| v["versions"].clone()),
            "live_identity":"Call mirket package probe agent-runtime for live identity; it launches a contained process. memory_read additionally opens project storage. Selection alone is not readiness.",
            "adoption":"When selected, use focused memory_read for relevant prior decisions and memory_put for verified stable facts. Mirket workers use LangGraph automatically; inspect workflow_status before resuming an interrupted run."},
        "recommended_actions":actions,
        "inspection_scope":"Passive process, installed-binary and saved-selection inspection. Does not start a VM, probe providers, launch containers or reconcile managed files.",
        "activation": "This response observes this MCP/CLI process only; other desktop connections and design services require their own calls.",
        "verification": "Identity and connection are not evidence that a user task passed."
    })
}
