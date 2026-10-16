//! A bounded recovery path for broad reference queries, separate from coverage.
use super::{Instance, MATCH_LIMIT, record_path, route};
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::collections::BTreeSet;

fn words(text: &str) -> BTreeSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .map(str::to_lowercase)
        .filter(|word| word.len() >= 3)
        .collect()
}

fn informative(word: &str) -> bool {
    !matches!(
        word,
        "the"
            | "and"
            | "for"
            | "with"
            | "from"
            | "through"
            | "while"
            | "when"
            | "that"
            | "this"
            | "into"
            | "has"
            | "are"
            | "can"
            | "each"
            | "same"
            | "then"
            | "its"
            | "only"
            | "needs"
            | "need"
            | "must"
            | "should"
            | "rather"
            | "design"
            | "system"
            | "visual"
            | "original"
            | "source"
            | "example"
            | "local"
            | "web"
            | "interface"
            | "language"
            | "reference"
            | "prototype"
            | "task"
            | "use"
            | "work"
            | "data"
            | "state"
            | "accessible"
            | "responsive"
            | "purposeful"
            | "modern"
            | "plus"
    )
}

pub(super) fn partial_candidates(instances: &[Instance], query: &str) -> Result<Option<Value>> {
    // This lexical aid cannot interpret exclusion or negation. Prefer no hint
    // to silently reversing a negative request. Ordinary search is unchanged.
    // Check before removing short words, retaining apostrophes in contractions.
    if query
        .split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '’')
        .map(str::to_lowercase)
        .any(|word| {
            matches!(
                word.as_str(),
                "no" | "not"
                    | "avoid"
                    | "avoids"
                    | "avoiding"
                    | "without"
                    | "never"
                    | "exclude"
                    | "excludes"
                    | "excluded"
                    | "excluding"
                    | "don't"
                    | "don’t"
                    | "dont"
                    | "cannot"
                    | "can't"
                    | "can’t"
            )
        })
    {
        return Ok(None);
    }
    let terms: BTreeSet<_> = words(query)
        .into_iter()
        .filter(|word| informative(word))
        .collect();
    if terms.len() < 3 {
        return Ok(None);
    }
    let mut ranked: Vec<_> = instances
        .iter()
        .filter_map(|instance| {
            let anchors = words(&format!(
                "{} {}",
                instance.aliases.join(" "),
                instance.tags.join(" ")
            ));
            let positive = words(&format!(
                "{} {} {} {} {} {}",
                instance.id,
                instance.title,
                instance.language,
                instance.summary,
                instance.use_when,
                anchors.iter().cloned().collect::<Vec<_>>().join(" ")
            ));
            let matched: Vec<_> = terms.intersection(&positive).cloned().collect();
            let anchor_count = terms.intersection(&anchors).count();
            // An isolated shared adjective is weak evidence. No poor-fit wording,
            // external source body, famous-source rank or family union is scored.
            if matched.len() < 2 || anchor_count == 0 {
                return None;
            }
            let missing: Vec<_> = terms.difference(&positive).cloned().collect();
            Some((anchor_count * 3 + matched.len(), instance, matched, missing))
        })
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.id.cmp(&b.1.id)));
    if ranked.is_empty() {
        return Ok(None);
    }
    let result = json!({
        "match_mode":"partial-keyword", "fit_validated":false, "coverage_claimed":false,
        "meaning":"Optional, non-exhaustive discovery hints after no complete reference keyword match. Lexical overlap is not relevance, a design direction, a quality score or platform coverage. Compare their stated fit with the task; other sources, combined perspectives or no reference may be more useful.",
        "query_terms":terms, "total":ranked.len(), "shown":ranked.len().min(MATCH_LIMIT),
        "selection_guide":route("references/design-languages/index.md"),
        "items":ranked.iter().take(MATCH_LIMIT).map(|(_, instance, matched, missing)| json!({
            "id":instance.id,"title":instance.title,"summary":instance.summary,
            "matched_terms":matched,"unmatched_terms":missing,
            "use_when":instance.use_when,"avoid_when":instance.avoid_when,
            "record":route(&record_path(&instance.id)),"lesson":route(&instance.card),"example":route(&instance.example)
        })).collect::<Vec<_>>()
    });
    ensure!(
        serde_json::to_vec(&result)?.len() <= 24 * 1024,
        "design reference candidates exceed response budget"
    );
    Ok(Some(result))
}
