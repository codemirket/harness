# Scoped canonical memory enumeration

`memory_list` enumerates eligible current record heads without search terms,
lexical ranking or model inference. Use it to review a scope's decisions or task
state and to page through critical constraints that exceed one recall response.
It does not list historical revisions; use `memory_history` for those and
`memory_read` for one exact revision.

## Request and page contract

The MCP call takes `expected_release_id`, an explicit `scope`, and `request`.
Project scope is `{ "kind": "project", "project_root": "/absolute/project" }`;
shared preferences require their own `{ "kind": "shared_preferences" }` call.
A missing project identity or memory store is refused without registering a scope
or creating storage. Stale releases are refused before those effects as well.

```json
{
  "expected_release_id": "running-release-id",
  "scope": { "kind": "project", "project_root": "/absolute/project" },
  "request": {
    "context": { "task_key": null, "checkout_id": null, "operation": null, "at_time": null },
    "kind": "decision",
    "status": "current",
    "critical_only": true,
    "applicability": "matching",
    "limit": 20,
    "max_bytes": 32768,
    "cursor": null
  }
}
```

`kind` is optional (`fact`, `decision`, `preference`, `task_state`, or `null`).
`status` selects `current`, `unresolved` or `both`. `critical_only:false` permits
both critical and ordinary records. Limits are 1–20 entries and 1024–32768 bytes
for the serialized page value; MCP transport can additionally carry a text copy.

The server binds the selected checkout; a caller does not acquire sibling-task
access by supplying another checkout ID. `matching` returns applicable records.
`unclassified` returns records that may apply but need missing context, such as a
task key or operation. A known context mismatch remains excluded. Validity windows,
current-head status, supersession and known forgetting apply on every page.
Repository-wide records may be shared between verified worktrees; checkout-bound
records remain local to their selected checkout.

A page contains `scope`, `entries`, `valid_at`, `observed_at`,
`stable_snapshot:false`, `has_more`, and `next_cursor`. Entries use one of two
forms:

- `detail:"summary"` contains `reference` (`record_id`, `revision`), `kind`,
  `status`, `critical`, `applicability`, `recorded_at`, `valid_from`, `valid_until`,
  `excerpt`, `content_bytes`, `content_truncated` and `source_count`.
- `detail:"reference_only"` contains `reference` and `reason:"byte_budget"` when
  the summary cannot fit. The full record remains available through exact read.

An excerpt is at most 256 UTF-8 bytes and ends at a character boundary. It may
omit a decisive exception, provenance or the remainder of a plan. Neither an
excerpt nor the reference-only fallback is enough to treat the record as fully
inspected. Read its exact returned revision and examine source evidence before
acting.

## Continuation and concurrent changes

Follow `next_cursor` until `has_more` is false. The cursor contains `schema:1`,
`after_record_id`, `valid_at` and `query_sha256`. Reuse it unchanged with the same
scope, context and filters. Page size and byte budget may change; changing the
bound selection is rejected. The checksum binds selection, not permissions or
content authenticity.

The first page chooses validity time from `context.at_time` or the current time;
continuation retains it through the cursor. Entries advance by record ID, not
lexical score or recency. Each page observes current eligible heads, but the
whole pass is not a stable canonical snapshot. A correction ahead of the cursor
can appear at its new revision; a forgotten record must disappear. An earlier
page can become stale, and a record added behind the cursor can be missed.
Restart a pass when concurrent changes matter to the decision. Do not call a
terminal page proof that the scope is permanently complete or unchanged.

The cursor boundary need not still exist: forgetting or correction between calls
does not authorize replay, restore an old record, or invalidate ordinary keyset
progress. An exact read can likewise find that a listed record was superseded or
forgotten before inspection; preserve that uncertainty and recheck current state.

## Verification scope

The disposable [memory MCP fixture](../tests/memory-smoke.mjs) now includes a
23-critical-decision enumeration with pagination, exact-read expansion of a
constraint beyond the excerpt, status/kind/critical/applicability filters,
unclassified context, checkout isolation and a minimal-reference byte-budget
case. It also checks stale-release and unregistered read-only behavior, refuses
a cursor with changed filters, and corrects/forgets unseen records between pages.

The actual MCP enumeration journey passed on macOS 26.5.2 ARM64 with private
Node 24.22.1 and core SHA-256
`d115785773dde1c23eac0d00b7a6bc0bd7880746c507ad6811b9fa1e9ef03b49`.
The combined native suite passed 331 library tests and one deterministic retrieval
fixture, with three manual tests ignored and strict all-target Clippy passing.
All nine focused enumeration cases passed. The configured memory, transfer and
restore journey also passed with network denied and telemetry disabled before
native model initialization. No new retrieval scoring or model adoption occurred.

An initial MCP fixture compared a reference with a full saved record; the test
was corrected to compare the exact ID/revision fields. Its failure also exposed
concurrent shutdown receipt-lock contention, now covered by three simultaneous
clients retiring real unused intents without committing or replaying them.
The retained final evidence is `enumeration-json-shutdown-native.log` and
`enumeration-json-memory-final.log` under the implementation task-state evidence
directory. No multilingual retrieval-quality, workload latency, archive or native
client qualification is implied by these checks. See the
[memory skill](../skills/memory/SKILL.md) for the surrounding save, inspection,
forgetting and continuation workflows.
