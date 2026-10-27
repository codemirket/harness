---
name: retrieval-evaluation
description: "Evaluate memory or RAG retrieval claims using relevance, coverage, stale/conflicting facts and downstream answer tests. Skip generic model evaluation without retrieval."
---

# Retrieval evaluation

Define the information need, corpus snapshot, scopes, source validity and expected evidence. Include exact, lexical, semantic, multilingual and no-answer cases where relevant, plus stale/conflicting facts and restricted-scope distractors. Preserve a reviewed relevance/source set and held-out tasks.

Measure retrieval relevance and coverage separately from answer correctness. Inspect what was indexed, filtered, truncated or unavailable; a no-hit result must not be conflated with an incomplete index. Compare bounded source expansion and downstream answers under matched context/result budgets.

Use the qualified Mirket memory operation when evaluating its actual route and the host for analysis. Check that current critical constraints survive ranking, stale facts are rejected/superseded and another scope cannot leak into answers. Evaluate canonical correction/forgetting separately from derived-index refresh.

Return corpus/query identity, retrieved evidence, downstream outcomes and failure analysis. Stop when the retrieval decision is supported or missing corpus/coverage prevents it. A similarity metric, embedding package receipt or fluent answer does not establish useful recall.
