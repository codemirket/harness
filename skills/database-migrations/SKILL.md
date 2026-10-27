---
name: database-migrations
description: "Change persisted schema or storage when existing data must survive forward migration and recovery. Skip read-only queries and offline SQL navigation."
---

# Database migrations

Inspect current schema/data versions, migration history, consumers and deployment order. Define invariants that must hold before and after, including nullability, uniqueness, relationships, units and retained/forgotten data. Identify the original state to preserve and the recovery boundary.

Choose a compatible sequence for old/new readers and writers when they overlap. Separate reversible schema steps from irreversible data transformations. Validate assumptions against representative data before mutation; do not rely on an empty development database. Protect a verified recovery point within the project's authorization and privacy rules.

Test forward migration, interrupted/failed execution, repeated invocation and recovery with explicit disposable data. Reopen with real consumers and reconcile counts, constraints and transformed values. A successful migration runner does not establish data preservation.

Return the migration, deployment/recovery sequence and invariant evidence. Stop before an unapproved irreversible effect or when a required recovery prerequisite is absent. Do not weaken checks or delete problematic records merely to make the migration pass.
