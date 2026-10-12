//! Real browser accessibility checks; reports are evidence, not certification.
use super::{
    EditorResult, definition,
    files::Destination,
    process::{self, Control},
};
use crate::package_mcp::LaunchSpec;
use anyhow::{Context, Result, bail};
use rmcp::{model::Tool, schemars, schemars::JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::Path;

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Audit {
    /// Credential-free HTTP URL of a local project server on a loopback port above 1024.
    pub url: String,
    /// Distinct project-relative JSON report. Parent must exist.
    pub output: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// Optional CSS selector for the visible region to inspect.
    pub selector: Option<String>,
    /// Optional WCAG/best-practice tags; defaults to WCAG 2.2 AA and best-practice.
    #[serde(default)]
    pub tags: Vec<String>,
    /// At most 16 typed interactions to expose the state to audit. No script evaluation.
    #[serde(default)]
    pub actions: Vec<Action>,
    #[serde(default)]
    pub overwrite: bool,
    pub expected_sha256: Option<String>,
}
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Click { selector: String },
    Fill { selector: String, value: String },
    Press { selector: String, key: String },
}

fn validate(args: &Audit) -> Result<()> {
    let url = reqwest::Url::parse(&args.url).context("invalid local audit URL")?;
    if url.scheme() != "http"
        || !matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
        || url.port().is_none_or(|p| p <= 1024)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || args.url.len() > 2048
    {
        bail!("audit URL must be credential-free loopback HTTP with an explicit port above 1024");
    }
    if !(320..=1920).contains(&args.width.unwrap_or(1280))
        || !(240..=1200).contains(&args.height.unwrap_or(800))
    {
        bail!("audit viewport must be 320..1920 by 240..1200 pixels");
    }
    let selector_ok = |s: &str| !s.is_empty() && s.len() <= 512 && !s.chars().any(char::is_control);
    if args.selector.as_deref().is_some_and(|s| !selector_ok(s))
        || args.actions.len() > 16
        || args.tags.len() > 12
    {
        bail!("audit selection or action count exceeds the bounded profile");
    }
    for tag in &args.tags {
        if ![
            "wcag2a",
            "wcag2aa",
            "wcag21a",
            "wcag21aa",
            "wcag22aa",
            "best-practice",
        ]
        .contains(&tag.as_str())
        {
            bail!("unsupported axe tag: {tag}");
        }
    }
    for action in &args.actions {
        let selector = match action {
            Action::Click { selector } => selector,
            Action::Fill { selector, value } => {
                if value.len() > 2048 {
                    bail!("fill value exceeds 2048 bytes");
                }
                selector
            }
            Action::Press { selector, key } => {
                if ![
                    "Tab",
                    "Shift+Tab",
                    "Enter",
                    "Space",
                    "Escape",
                    "ArrowDown",
                    "ArrowUp",
                    "ArrowLeft",
                    "ArrowRight",
                ]
                .contains(&key.as_str())
                {
                    bail!("unsupported interaction key");
                }
                selector
            }
        };
        if !selector_ok(selector) {
            bail!("invalid bounded action selector");
        }
    }
    Ok(())
}

fn report(bytes: &[u8], url: &str) -> Result<Value> {
    if bytes.len() > 1024 * 1024 {
        bail!("axe report exceeds 1 MiB");
    }
    let value: Value = serde_json::from_slice(bytes)
        .context("axe runtime did not produce a usable JSON report")?;
    if value["testEngine"]["name"] != "axe-core"
        || value["testEngine"]["version"] != "4.14.0"
        || value["url"] != url
        || value["visible_elements"].as_u64().is_none_or(|n| n == 0)
    {
        bail!("axe report identity or rendered content does not match the requested audit");
    }
    for key in ["violations", "incomplete", "passes", "inapplicable"] {
        let entries = value[key]
            .as_array()
            .context("axe report is missing result arrays")?;
        if entries.len() > 200
            || entries.iter().any(|e| {
                e["id"].as_str().is_none_or(str::is_empty)
                    || e["nodes"].as_array().is_none_or(|n| n.len() > 32)
            })
        {
            bail!("axe report has invalid or excessive rule results");
        }
    }
    Ok(value)
}

pub fn definitions() -> Vec<Tool> {
    vec![definition::<Audit>(
        "accessibility_audit",
        "Audit a running local project page with real axe-core 4.14.0 in pinned headless Chromium. Optional typed click/fill/press actions expose a menu, dialog or other state before inspection. Saves a bounded project JSON report with violations, incomplete/manual-review results and truncation flags. Local HTTP only; off-origin requests, downloads and popups are refused. Report output needs explicit overwrite intent. A zero-violation result covers the inspected rendered state and automated rules only; manually inspect keyboard order, focus, motion and content.",
        false,
    )]
}
pub async fn invoke(
    spec: &LaunchSpec,
    tool: &str,
    value: Value,
    workspace: &Path,
    control: &Control<'_>,
) -> Result<EditorResult> {
    if tool != "accessibility_audit" {
        bail!("unknown accessibility tool");
    }
    let args: Audit = serde_json::from_value(value)?;
    validate(&args)?;
    let destination = Destination::new(
        &spec.cwd,
        &args.output,
        &["json"],
        args.overwrite,
        args.expected_sha256.as_deref(),
    )?;
    let input = workspace.join("audit.json");
    let output = workspace.join("report.json");
    std::fs::write(&input, serde_json::to_vec(&args)?)?;
    process::run(
        spec,
        &[
            "audit".into(),
            input.to_string_lossy().into_owned(),
            output.to_string_lossy().into_owned(),
        ],
        workspace,
        Some(&output),
        control,
    )
    .await?;
    let bytes = process::output(&output)?;
    let result = report(&bytes, &args.url)?;
    control.check()?;
    let artifact = destination.publish(&bytes, "application/json")?;
    Ok(EditorResult { operation: tool.into(), artifact: Some(artifact), design: Some(json!({
        "engine":"axe-core", "version":"4.14.0", "url":result["url"],"viewport":result["viewport"],
        "violations":result["violations"].as_array().map(Vec::len),"incomplete":result["incomplete"].as_array().map(Vec::len),
        "truncated":result["truncated"],"manual_review_required":true
    })), notes: vec!["Automated accessibility evidence for the inspected visible state; not certification or human approval.".into()], ..Default::default() })
}
pub async fn probe(spec: &LaunchSpec, workspace: &Path, control: &Control<'_>) -> Result<()> {
    let bytes = process::run(spec, &["--probe".into()], workspace, None, control).await?;
    let value: Value = serde_json::from_slice(&bytes)?;
    if value["engine"] != "axe-core"
        || value["version"] != "4.14.0"
        || value["failure_probe"] != "button-name"
        || value["browser_version"].as_str().is_none_or(str::is_empty)
    {
        bail!("axe runtime did not detect the deliberate inaccessible button");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(url: &str) -> Audit {
        serde_json::from_value(json!({"url":url,"output":"report.json"})).unwrap()
    }
    #[test]
    fn refuses_non_local_targets_and_unbounded_actions() {
        for u in [
            "https://example.com",
            "http://127.0.0.1:80",
            "http://user@localhost:3000",
            "file:///tmp/page.html",
            "http://localhost:3000/#a",
        ] {
            assert!(validate(&args(u)).is_err());
        }
        assert!(validate(&args("http://127.0.0.1:3000/test")).is_ok());
        let mut a = args("http://localhost:3000");
        a.actions.push(Action::Press {
            selector: "button".into(),
            key: "F12".into(),
        });
        assert!(validate(&a).is_err());
    }
    #[test]
    fn rejects_success_without_a_usable_report() {
        assert!(report(b"{}", "http://localhost:3000").is_err());
        assert!(report(br#"{"testEngine":{"name":"axe-core","version":"4.14.0"},"url":"http://localhost:3000","visible_elements":0}"#, "http://localhost:3000").is_err());
    }
}
