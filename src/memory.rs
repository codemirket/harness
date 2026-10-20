//! Typed host memory API. Project registration is enforced by the MCP caller.
use crate::{agent_runtime, paths::Paths};
use anyhow::{Result, ensure};
use rmcp::schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Read {
    pub project: PathBuf,
    pub query: Option<String>,
    pub id: Option<String>,
    pub cursor: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}
fn default_limit() -> usize {
    5
}
#[derive(Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Preference,
    Decision,
    Constraint,
    Fact,
}
#[derive(Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreviousFact {
    pub id: String,
    pub sha256: String,
}
#[derive(Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Put {
    pub project: PathBuf,
    /// One explicit, stable fact; never credentials or an automatic transcript dump.
    pub text: String,
    pub kind: Kind,
    /// Where the fact came from. This statement is not authenticated evidence.
    pub provenance: String,
    pub expires_at: Option<u64>,
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<PreviousFact>,
}
#[derive(Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Forget {
    pub project: PathBuf,
    pub id: String,
    pub expected_sha256: String,
}
fn identifier(value: &str) -> Result<()> {
    ensure!(
        value.len() == 36 && value.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-'),
        "invalid memory ID"
    );
    Ok(())
}
fn arguments(value: &impl Serialize) -> Result<Value> {
    let mut value = serde_json::to_value(value)?;
    value
        .as_object_mut()
        .expect("typed memory request")
        .remove("project");
    Ok(value)
}
pub async fn read(paths: &Paths, request: Read) -> Result<Value> {
    ensure!(
        (1..=20).contains(&request.limit),
        "memory limit must be 1..20"
    );
    ensure!(
        !(request.query.is_some() && request.id.is_some()),
        "choose memory query or ID"
    );
    ensure!(
        request.cursor.is_none() || (request.query.is_none() && request.id.is_none()),
        "cursor is only for memory listing"
    );
    if let Some(cursor) = &request.cursor {
        identifier(cursor)?;
    }
    if let Some(query) = &request.query {
        ensure!(
            !query.trim().is_empty() && query.len() <= 2048,
            "query requires 1..2048 bytes"
        );
    }
    if let Some(id) = &request.id {
        identifier(id)?;
    }
    agent_runtime::call(
        paths,
        Some(&request.project),
        "memory_read",
        arguments(&request)?,
    )
    .await
}
pub async fn put(paths: &Paths, request: Put) -> Result<Value> {
    if let Some(previous) = &request.supersedes {
        identifier(&previous.id)?;
        ensure!(
            previous.sha256.len() == 64 && previous.sha256.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid previous memory digest"
        );
    }
    for (text, max, label) in [
        (&request.text, 2048, "fact"),
        (&request.provenance, 1024, "provenance"),
        (&request.key, 128, "key"),
    ] {
        ensure!(
            !text.trim().is_empty() && text.len() <= max && !text.contains('\0'),
            "invalid memory {label}"
        );
    }
    agent_runtime::call(
        paths,
        Some(&request.project),
        "memory_put",
        arguments(&request)?,
    )
    .await
}
pub async fn forget(paths: &Paths, request: Forget) -> Result<Value> {
    identifier(&request.id)?;
    ensure!(
        request.expected_sha256.len() == 64
            && request
                .expected_sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit()),
        "invalid memory digest"
    );
    agent_runtime::call(
        paths,
        Some(&request.project),
        "memory_forget",
        arguments(&request)?,
    )
    .await
}
