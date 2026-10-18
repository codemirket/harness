//! Shared final-action validation for streamed Codex and HTTP Responses output.
use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;

pub(super) fn validate_item(item: &Value) -> Result<()> {
    match item["type"].as_str() {
        Some("reasoning") => Ok(()),
        Some("message") => {
            ensure!(
                item["role"] == "assistant",
                "unexpected Responses output role"
            );
            ensure!(
                item["phase"].is_null()
                    || matches!(
                        item["phase"].as_str(),
                        Some("commentary" | "partial_answer" | "final_answer")
                    ),
                "unknown Responses message phase"
            );
            for part in item["content"]
                .as_array()
                .context("Responses output content missing")?
            {
                ensure!(
                    part["type"] == "output_text",
                    "non-text Responses output rejected"
                );
            }
            Ok(())
        }
        _ => bail!("Responses native tool or unknown output rejected before dispatch"),
    }
}

/// Commentary and partial answers are never executable actions. Validate every
/// item, then require one final message (or a single phase-absent message).
pub(super) fn action(output: &Value) -> Result<Value> {
    let items = output.as_array().context("Responses output missing")?;
    let mut answer = None;
    let mut messages = 0;
    for item in items {
        validate_item(item)?;
        if item["type"] == "message" {
            messages += 1;
            ensure!(
                answer.is_none(),
                "Responses output continued after final answer"
            );
            if item["phase"].is_null() || item["phase"] == "final_answer" {
                answer = Some(item);
            }
        }
    }
    let answer = answer.context("Responses final answer missing")?;
    ensure!(
        messages == 1 || answer["phase"] == "final_answer",
        "Responses multiple messages require an explicit final answer"
    );
    let texts = answer["content"]
        .as_array()
        .context("Responses text missing")?
        .iter()
        .map(|part| part["text"].as_str().context("Responses text missing"))
        .collect::<Result<Vec<_>>>()?;
    let action: Value =
        serde_json::from_str(&texts.join("")).context("Responses action is not JSON")?;
    ensure!(action.is_object(), "Responses action is not an object");
    Ok(action)
}
