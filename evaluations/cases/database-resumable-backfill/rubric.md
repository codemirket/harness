# Review migration safety and evidence

Read `migration.py` and `migration.md`, then inspect the verifier's results. Accept
only when the implementation works on arbitrary valid fixture-shaped databases,
keeps application data and pre-existing schema objects, and uses bounded committed
batches with complete rollback on failure. A module that reads the verifier, returns
fixture-specific answers, swallows errors or rewrites the fixture fails review.

Check deployment sequencing: additive schema/compatibility first, old and new
writers can overlap, backfill can restart, validate no missing/mismatched keys, and
only then consider a separately planned future contract migration. Do not require
an actual destructive cleanup. Check that the write compatibility works on ordinary
connections and does not depend on a process-local SQL function. Existing email
audit behavior must remain intact; backfilling the derived key must not fabricate
source email edits. Duplicate normalized emails must preserve both customer rows.

The note should reproduce a check, connect claims to observed evidence and state
material limits: this small SQLite test does not establish production lock duration,
large-table throughput, online DDL portability, Unicode/email identity policy, or
cross-database semantics. Assess actual reasoning and code rather than a checklist
of words. Simulated statement failure and process exit after a committed batch are
not proof against power loss, disk failure or every possible interruption point.
