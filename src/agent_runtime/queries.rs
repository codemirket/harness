//! Deterministic memory-query planning for the native agent runtime.
//!
//! Planning is deliberately separate from embedding and retrieval.  It owns
//! request validation, bounded objective chunking, and the incomplete signal;
//! the runtime pool owns tokenizer fitting and returns the effective queries.

use anyhow::{Context, Result, ensure};
use serde_json::Value;

const MAX_QUERIES: usize = 8;
const OBJECTIVE_CHUNK_CHARS: usize = 192;
const MAX_EXPLICIT_QUERY_CHARS: usize = 512;

/// Plan bounded retrieval queries from a native runtime request.
///
/// The returned tuple is `(queries, incomplete, automatic)`.  `automatic` is
/// true when the planner derived queries from the objective and task needs;
/// callers pass it to the embedding pool so fitting can be reported separately
/// from planning.  Explicit queries remain in request order and do not gain
/// objective-derived queries.
pub fn plan(request: &Value) -> Result<(Vec<String>, bool, bool)> {
    let explicit = request["memory_queries"].as_array();
    let automatic = explicit.is_none_or(|queries| queries.is_empty());
    let mut queries = Vec::new();
    let mut incomplete = false;

    if let Some(list) = explicit.filter(|queries| !queries.is_empty()) {
        ensure!(list.len() <= MAX_QUERIES, "too many memory queries");
        for query in list {
            let text = query.as_str().context("query text required")?;
            ensure!(
                !text.trim().is_empty() && text.chars().count() <= MAX_EXPLICIT_QUERY_CHARS,
                "invalid explicit query"
            );
            if !queries.iter().any(|existing| existing == text) {
                queries.push(text.to_owned());
            }
        }
    } else {
        let objective = request["objective"]
            .as_str()
            .context("objective required")?;
        let chars: Vec<char> = objective.chars().collect();
        let chunks: Vec<String> = chars
            .chunks(OBJECTIVE_CHUNK_CHARS)
            .map(|chunk| chunk.iter().collect())
            .collect();

        if let Some(first) = chunks.first() {
            queries.push(first.clone());
        }
        if let Some(last) = chunks.last().filter(|chunk| !queries.contains(*chunk)) {
            queries.push(last.clone());
        }

        for need in request["task_needs"].as_array().into_iter().flatten() {
            if let Some(text) = need["reason"].as_str() {
                if queries.len() < MAX_QUERIES {
                    queries.push(text.chars().take(OBJECTIVE_CHUNK_CHARS).collect());
                    incomplete |= text.chars().count() > OBJECTIVE_CHUNK_CHARS;
                } else {
                    incomplete = true;
                }
            }
        }

        for chunk in &chunks {
            if queries.len() < MAX_QUERIES && !queries.contains(chunk) {
                queries.push(chunk.clone());
            }
        }
        incomplete |= chunks.iter().any(|chunk| !queries.contains(chunk));
    }

    ensure!(!queries.is_empty(), "no recall query");
    Ok((queries, incomplete, automatic))
}

#[cfg(test)]
mod tests {
    use super::plan;
    use serde_json::json;

    #[test]
    fn preserves_explicit_queries_without_objective_expansion() {
        let request = json!({
            "objective": "ignored objective",
            "memory_queries": ["late requirement", "German financial reporting"]
        });
        let (queries, incomplete, automatic) = plan(&request).unwrap();
        assert_eq!(queries, ["late requirement", "German financial reporting"]);
        assert!(!incomplete);
        assert!(!automatic);
    }

    #[test]
    fn keeps_late_objective_content_in_a_bounded_plan() {
        let request = json!({
            "objective": format!("{} Turkish offline behavior is required", "context ".repeat(40)),
            "task_needs": []
        });
        let (queries, incomplete, automatic) = plan(&request).unwrap();
        assert!(automatic);
        assert!(
            queries
                .iter()
                .any(|query| query.contains("Turkish offline behavior"))
        );
        assert!(!incomplete);
    }

    #[test]
    fn reports_omitted_objective_and_need_content() {
        let request = json!({
            "objective": "x".repeat(192 * 12),
            "task_needs": (0..8)
                .map(|index| json!({"reason": format!("need-{index} {}", "z".repeat(220))}))
                .collect::<Vec<_>>()
        });
        let (queries, incomplete, automatic) = plan(&request).unwrap();
        assert!(automatic);
        assert!(queries.len() <= 8);
        assert!(incomplete);
    }
}
