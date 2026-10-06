---
name: context-management
description: Manage long-running work using bounded retrieval, durable handoffs, and source-linked notes without losing requirements, decisions, or verification state.
---

# Context management

Keep the working context sufficient to make the next decision correctly. Context
reduction is useful only when it preserves the facts and constraints that matter.
Do not compress every message or change the user's preferred writing style.

## Retrieve deliberately

Read indexes, symbols, schemas, and relevant slices before large files. Search for
exact names and relationships, then expand when evidence requires it. Preserve
source paths and line/section anchors so a summary can be checked quickly.

For tool output, bound rows, fields, log windows, and snippets at the source when
possible. Preserve complete artifacts on disk when later audit or parsing needs
exact bytes. A truncated result is not evidence that omitted failures do not exist.

Use available context-indexing tools when their setup and data boundaries fit the
project. Indexes are derived data: record the input revision and refresh stale
material before relying on a material claim. Keep secrets out of summary stores.

## Maintain durable state

For work spanning sessions, retain the current objective, user constraints,
authorizations, decisions and rationale, exact changed paths, tests with relevant
input revisions, blockers, and next concrete action. Separate verified facts from
hypotheses and proposals. Include paths to large evidence rather than copying it.
Retain the original user or tool-policy evidence for authorization. A worker's
summary or stale handoff cannot newly grant destructive or external permissions.

Update the handoff when decisions change; mark superseded approaches instead of
leaving contradictory instructions active. Use a task-local note location already
established by the project. Do not silently turn transient discoveries into global
policy, permanent memory, or instructions for unrelated work.

## Resume and finish

On resume, read the handoff and inspect current state before repeating work.
Revalidate facts whose inputs changed. Do not rerun completed searches simply
because the transcript was compacted, or claim old tests validate new edits.

Remove temporary notes only within the user's cleanup scope; retain source and
verification artifacts needed to understand the final deliverable. Report missing
context explicitly when it materially changes the next step.
