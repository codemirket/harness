# Usage accounting removed by product decision — 20 October 2026

Users track usage and billing in their provider's own panel. Mirket no longer
owns a usage-accounting utility, invoice reconciliation or a cost dashboard.
This supersedes the accounting feature and remaining billing work described in
the earlier harness reports; their recorded measurements remain historical evidence.

The change removes `mirket agent accounting`, token/cost observation and
aggregation, accounting-version markers and durable probe ledgers. Execution
no longer creates a run just to account for preflight inference, or reads an
accounting ledger before saving its final trace. Existing saved records are
not deleted or migrated.

Response hashes, elapsed time, bounded error diagnostics, model identity checks,
vision conformance, recovery state and execution budgets remain. Provider token
limits still constrain inference requests; they are execution controls, not a
usage tracker. Windows process accounting remains an operating-system mechanism
for child-process cleanup and is unrelated to usage or billing.

Regression coverage checks that HTTP success and malformed-action failures keep
response identity, timing and specific errors without usage fields, accounting
summaries or ledger files. Native Codex and Claude observations retain only
response identity, including verified Claude failures. Failed initial/resumed
vision checks still return diagnostics without creating accounting state. The
CLI parser rejects the removed accounting command.

Verification: macOS ARM64 formatting, Clippy and all Rust targets passed: **412
passed, zero failed, seven opt-in tests ignored**. Catalog and disposable setup/doctor
passed. Direct CLI checks confirm the old command was available before removal
and is now absent from help and rejected with exit 2. Four tests for the retired
ledger/aggregation implementation were removed; adapter and execution regressions
were updated to enforce the new product boundary. The initial documentation gate
found a historical link to the deleted module; it now points to the last review
commit, and the report seal records that link-only amendment. The complete gate
passed afterward.

[Verification record](harness-accounting-removal/checks.json).
The earlier hosted CI validates its named historical snapshot, not this removal.
No deployment is part of this change. Image security work remains open; billing
integration is no longer a harness requirement.
