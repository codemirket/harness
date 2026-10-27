# M2 canonical-memory integration review

Reviewed `src/memory_service.rs` and its canonical store, identity and operation-receipt boundaries on 2026-10-24. This is a source review of the finite canonical/lexical slice, followed by the actual MCP journey recorded in `docs/m2-memory-verification.md`. It does not establish full M2 acceptance or semantic recall quality.

## Findings resolved during review

1. **P2 — claimed commit could remain misleadingly prepared.** The initial `commit` removed the in-memory intent before source validation and the durable `unknown` write. A stale revision or a busy operation-store lock could therefore consume the intent without a canonical effect while leaving its JSON receipt `prepared`. A duplicate concurrent call could also observe that inconsistent state. The current implementation keeps the pending mutex and intent through validation and the durable unknown write, then removes it (`MemoryService::commit`, `src/memory_service.rs:289`). This closes the race without enabling effect replay. A failed pre-write receipt update retains the intent; a successful update leaves subsequent inspection at unknown or committed.

2. **P2 — missing canonical confirmation could still report committed.** Initially, `inspect` trusted a JSON `committed` status when the SQLite database or mutation receipt was absent. It now reports `confirmation_unavailable` (`MemoryService::inspect`, `src/memory_service.rs:267`). SQLite mutation receipts remain the canonical commit evidence. A receipt with `available:false` still establishes that the operation committed, while explicitly reporting that its target no longer exists.

These fixes were re-read in source. The later MCP fixture exercised concurrent same-ID commits, stale prepared corrections and response loss. Missing canonical confirmation and failed convenience-receipt writes need targeted service failure injection; the ordinary journey alone does not establish those branches.

## Boundaries checked

- Release matching precedes identity registration and all effectful memory entry points. `inspect` deliberately remains available across release changes and does not replay mutations.
- Receipts bind the explicit scope, native checkout identity and connection owner. Another owner cannot commit a prepared intent. Missing in-memory intents lead to inspection, not reconstruction of a write from durable content.
- Prepare validates input and exact source revision. Commit repeats the same source-revision/checkout check; SQLite revision CAS supplies the final atomic stale-write protection (`verify_source_checkout` and `MemoryService::commit`).
- Search binds checkout context from the explicitly selected root. Exact reads verify the returned revision's checkout. History checks every returned revision rather than trusting only the current head (`MemoryService::search`, `read`, `history`, and `verify_checkout`). Repository-wide records remain distinguishable by absent checkout applicability.
- The canonical transaction commits a content-free operation receipt with the record revision. Same-ID/same-intent retries resolve the existing revision; different intent fails. An unavailable target cannot be recreated by replay. The JSON convenience receipt may lag without losing the SQLite confirmation.
- Canonical inputs bound content to 16 KiB, provenance to eight individually bounded sources, applicability fields to 256 bytes and timestamps/revisions to SQLite's signed integer range. Search/history enforce at most 20 records and a 32 KiB serialized result budget. Exact reads return one bounded record; they do not promise the search-page budget. Up to 64 unexpired intent bodies are held per connection. No source URI is dereferenced, transcript ingested or stored content treated as authority.

No additional concrete high-severity replay, cross-project or checkout-access defect was found in the revised finite service boundary.

## Remaining qualification and intended scope

The actual MCP fixture subsequently passed stale-release rejection, read-only absent stores, two concurrent clients, same-ID concurrent commit, stale revision, response loss/reconnection, checkout-specific reads/history and bounded critical-constraint recall. The operation store's existing finite receipt capacity also applies to memory preparations; no receipt reclamation or unbounded lifetime write capacity is claimed.

Branch/source revision values are provenance supplied by the caller, not automatic evidence of current checkout-file validity. Checkout identity alone does not establish that a fact still applies after a branch change. Full M2 still requires the agreed source/freshness workflow, semantic retrieval, forgetting exclusions and backup/restore qualification; their absence is an incomplete planned gate rather than proof that the current canonical transaction is defective.

The canonical store's 16 actual SQLite tests passed before this integration review. The only code adjustment authorized during review removed the unused production database-path field/getter; the test-only helper now uses `Connection::path()`. Backup work should use SQLite's consistent backup API rather than treating a raw file path as a snapshot contract.
