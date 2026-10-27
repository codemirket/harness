---
name: js-ts-navigation
description: "Find definitions or references in supplied JS, TS, JSX or TSX sources using bounded revision-linked navigation. Skip unsupported languages and full-project completeness claims."
---

# JavaScript and TypeScript navigation

Use the qualified Mirket TypeScript navigation capability on explicitly selected saved sources. Supply the imports/declarations required by the actual API and bind every source to its current revision. Do not scan the entire project, execute package code or fetch dependencies implicitly.

Distinguish lexical names, scoped symbols, imports/aliases and property references. Expand a result only against its original source identity and UTF-8 byte spans; do not confuse character indices or normalized line endings with saved bytes. A result covers the supplied graph, not every possible caller.

Check ambiguity, unsupported constructs, omitted imports and stale-source refusal. Inspect at least one same-name/different-scope or alias case when it can expose false resolution. Re-read changed sources before deciding whether a new query is still equivalent.

Return definitions/references with source locators, revisions, coverage and explicit unresolved edges. Stop when requested evidence is obtained or the missing graph/source prevents resolution. Use the host/project's own tools for other languages rather than claiming this path supports them.
