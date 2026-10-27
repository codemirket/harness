---
name: sql-navigation
description: "Trace SQL definitions and references through project queries, migrations and schema sources. Skip database execution unless the task explicitly calls for it."
---

# SQL navigation

Use Codex source inspection and the project's existing SQL parser, language server or database tooling when available. Identify the dialect and version from project configuration or explicit user context. Select the actual query, migration and schema sources before resolving names; do not infer a complete deployed schema from a partial repository snapshot.

Distinguish aliases, CTEs, tables, columns and scopes using constructs supported by the selected tool. Cite saved file locations and revisions; preserve exact byte spans only when a tool or direct check establishes them. Report missing schema, ambiguous names, dynamic SQL and unsupported syntax rather than guessing a resolution or treating a parse as validation of semantics.

Verify a scope or alias counterexample when it can expose a false match. Recheck source evidence after edits. A source analysis describes the supplied repository state, not the production catalog or every dynamically generated query.

Return the resolved source locations, dialect/schema basis, coverage and unresolved references. Stop at the supported evidence boundary or a concrete missing schema/tool. Query execution is a separate effect under host permissions; do not let navigation silently become a database mutation or credential search.
