# Independent Astra final repair reassessment — R4

**Verdict: APPROVE_WITH_LIMITS. The R3 P3 diagnostic finding is closed. No remaining concrete defect or required revision was identified in this narrow reassessment.** The earlier provenance, export-residue and reviewer-attribution resolutions remain in effect.

Approval concerns the configuration layer under the original acceptance criteria. The limits concern unverified host/platform and task-quality outcomes, rather than an outstanding implementation repair.

## Scope and evidence identity

I read only `/Users/nazmi/Documents/.ai/build/dual-independent-review/astra-r4`, including its criteria, snapshot, revision notes, my own R3 report, the permission evidence and the affected implementation/tests/documentation. The declared source fingerprint is `feafa4b3cc23f94e2a3e99a9dd18b047c450606527fe60ed7d377cf9b1923253`. References below are relative to that snapshot.

I performed static inspection and read-only hashing. I did not execute project code/tests/installers, access the network, invoke models, delegate, inspect other snapshots or parent source, or change snapshot files. This report is the sole authorized write. I did not recompute the aggregate snapshot fingerprint.

The independently computed `lib/harness.py` hash is `9ddd4ea62864c71d957675d9dd20317137015491f96ff37ff0271aaca192296a`; the test-file hash is `3f4eecca919a78b22c16ddc561eacc299d74c80d9679c9ad240175615d113d89`. Both match the snapshot and current durable verification record. The permission-evidence record also names that implementation hash.

## Correction to the original trigger

My R3 mode-000 example was too broad for a selected target undergoing new-copy collision preflight. `lib/catalog.py:559–563` inspects existing sibling `SKILL.md` files before installing a new destination. When the blocked directory makes that inspection fail, reconciliation stops before writes. That is an existing safety boundary, not the later diagnostic failure, and it should remain intact.

The actual successful permission fixture therefore uses an active Codex target with no selected jobs, while Claude receives four selected skills. This reaches the preserved-copy diagnostic path without first requiring collision inspection of the blocked Codex copy. A separate unit fixture denies receipt metadata specifically, covering the diagnostic exception boundary within a target that also has selected jobs.

This narrows the concrete reproduction and the circumstances of the post-publication consequence in my earlier report. It does not invalidate the underlying finding that advisory reporting allowed filesystem exceptions to escape. The implementation now handles that boundary explicitly. `docs/project-targets.md:49–51` appropriately explains that diagnostic recovery does not bypass selected-target preflight.

## Repair assessment

At `lib/harness.py:431–447`, one local root-inspection boundary now covers guarded parent checks, link detection, existence/type checks and enumeration. An `OSError` yields `uninspected_skill_root` with an inaccessible state; unsafe paths remain distinguished. This closes the former gap where metadata lookup could fail before the enumeration handler.

At lines 454–479, the per-copy handler covers the visible directory, receipt metadata, opening, reading and closing. Filesystem failures yield an unreadable `uninspected_skill_copy` diagnostic and allow other reporting to continue. The previous failing receipt `lstat` is inside this boundary. Known selected destinations remain excluded from the advisory scan, and neither selected validation nor publication is wrapped by this recovery handler.

Receipt parsing remains bounded to 64 KiB. Lines 475–476 also catch `RecursionError`, so sufficiently nested JSON within that byte limit is treated as invalid instead of aborting reporting. Unknown/invalid metadata does not become ownership or cleanup authority. Link rejection, preservation and separation from internal installation jobs and lock entries remain intact.

The change is appropriately small. It recovers from failures in optional reporting without converting failures in selected reconciliation into success. I found no new exception-handling regression in these inspected paths.

## Regression evidence

I inspected all three new test methods in `tests/test_harness.py:433–496`:

- Root/parent metadata denial produces a root diagnostic, prevents enumeration and preserves files.
- Receipt metadata denial exercises plan, sync and doctor, expects an unreadable preserved diagnostic, avoids exposing the unread receipt's ID and verifies preservation plus completion of selected installation.
- A genuinely over-nested JSON string first demonstrates the parser failure, then exercises all three commands and expects invalid preserved diagnostics without blocking selected installation.

These complement the earlier enumeration-error, malformed-receipt and link-preservation tests. They test the actual failure boundaries rather than merely the helper's output shape.

The primary's `review-evidence/diagnostic-permissions-r4.json` reports an actual non-root Darwin mode-000 fixture: all three commands return zero, preserve the blocked directory mode and existing file states, report unreadable/overcomplex copies, and retain four selected lock entries. It records the selected-target negative control separately. These are supplied execution claims, not my own reproductions.

`review-evidence/suite-r4.log` reports **471 tests passed in 35.759 seconds**. Its independently computed hash, `63f44cb3cf8ffdb28eac797fcd8b282d1fe43bffcebd635c863c912313e9714e`, matches the durable record. This establishes the identity of the supplied log; I did not run the suite.

## Retained coverage and limits

The initial all-domain assessment remains unchanged: frontend/interface design, vector/raster routing and motion, QA, backend/API/MCP work, databases/migrations, operations/release/desktop integration, marketing/search, research, documentation/office artifacts, planning/product/debugging, analysis and coordination/security/evaluation. The final repair adds no new capability, runtime, dependency or service. Four complementary project skills plus the 14 globals retain the rich foundation, with the explicit portable profile available where corresponding globals are absent.

Actual desktop/Claude selection, native Windows, physical devices, other browsers, live providers and production recovery, bitmap craft outcomes, native Office rendering and human visual acceptance remain outside this review's evidence. Historical domain trials retain their original scope and fingerprints; they are not new R4 outcomes.

No further code change is requested by this assessment. The focused implementation inspection, relevant regression cases, attributed permission fixture and current suite evidence are sufficient to close the diagnostic issue while preserving those broader limits.
