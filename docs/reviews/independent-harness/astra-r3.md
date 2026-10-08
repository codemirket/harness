# Independent Astra final reassessment — R3

**Verdict: APPROVE_WITH_LIMITS.** The original two P3 findings remain resolved, and the final changes improve discovery and make preserved duplicate registrations visible without deleting them. I found one new P3 diagnostic regression: an inaccessible obsolete skill directory can abort reporting. I found no blocking payload-integrity defect or demonstrated loss of the requested domain capabilities.

This approves the configuration layer within the original criteria, with the concrete minor issue below and the retained execution/quality limits. It does not certify universal model performance, actual desktop selection, production behavior or every upstream specialist.

## Scope and method

I read only `/Users/nazmi/Documents/.ai/build/dual-independent-review/astra-r3`, starting with its criteria, snapshot, revision notes and my own prior report. The declared source fingerprint is `e295ca4a11ad1eaa2427a9e93de1aeaf2be18818de440a2c3b623a3ebbccfdde`. Paths and line numbers below refer to this immutable snapshot.

I used read-only inspection and hashing. I did not execute project code/tests/installers, use the network, invoke a model, delegate, mutate files, inspect parent source or another snapshot/reviewer, or follow external paths named in evidence. I independently checked the hashes of the changed implementation/test files, current catalog and supplied suite log; the source hashes agree with the snapshot. I did not recompute the aggregate snapshot hash. The R3 suite log hashes to `7ad62fad8059a8734a8015ed549bf616a7a7f046f420bb99cdcf54ebb76ff8ba`.

## Remaining concrete finding

### P3 — An inaccessible preserved copy can abort plan/doctor or post-sync reporting

**Locations:** `lib/harness.py:452–464`, especially line 458; underlying helper `lib/catalog.py:35–43`; post-publication reporting at `lib/harness.py:559–561`.

**Trigger:** A valid project selects ordinary accessible skills, while its active target also contains an obsolete/unselected directory under `.agents/skills` or `.claude/skills` whose search permission is denied. Its directory entry remains visible from the readable parent, but accessing `.skill-catalog.json` inside it fails. A concrete POSIX example is an obsolete directory with mode `000`, inspected as a non-root user.

**Control-flow evidence:** The scan can enumerate the parent, `lstat` the obsolete directory itself and identify it as a directory. It then calls `catalog.is_link(receipt)` before entering the receipt-read `try` block. `is_link` catches only `FileNotFoundError`; the `PermissionError` from the receipt's `lstat` therefore escapes. Other metadata probes in this diagnostic path can similarly raise an `OSError` outside the current handlers. The existing inaccessible-root test at `tests/test_harness.py:423–431` mocks `Path.iterdir`, so it does not exercise this per-copy failure.

**Consequence:** An unrelated preserved path can make plan/doctor fail instead of yielding an uninspected diagnostic. Sync can already have published selected copies and its lock before the reporting exception; the top-level CLI then exits unsuccessfully without the normal JSON result. The path is not deleted or modified, and this does not demonstrate payload corruption or unauthorized replacement. I classify it as a minor diagnostic/reliability regression, not a blocker for the configuration architecture.

**Smallest remedy:** Catch filesystem inspection failures at the root/per-copy diagnostic boundary, emit an explicit `uninspected_skill_copy` or root diagnostic and continue. Keep this handling confined to reporting: selected-copy validation and mutation failures must continue to fail closed. Do not catch such errors globally or follow a denied/symlinked path to discover more detail.

**Meaningful check:** Add a regression that raises `PermissionError` on the obsolete receipt's `lstat`, plus a non-root POSIX permission fixture where practical. Check plan/sync/doctor output and preservation, including sync with a real selected update; ensure the update completes and is reported while the inaccessible copy remains untouched. Cover inaccessible root metadata as well as enumeration. I did not execute this reproduction; this is a static finding from the inspected exception boundaries.

## Resolution and regression assessment

The original provenance issue remains closed. `project_provenance` at `lib/harness.py:345–371` still separates original installation from current selection, and the normal selected-copy integrity checks precede it. The additional diagnostics do not relabel receipts, invent a source fetch or enter the lock as selected work.

The original global export residue issue also remains closed. `lib/bundle.py:162–184` now excludes root/nested `.DS_Store` files as well as interpreter residue, only for unpinned global payloads and after ordinary safety validation. It then rechecks required files and executable declarations. `tests/test_bundle.py:219–282` preserves full-export equality with residue present, real helper retention, pinned/project byte contracts and rejection of removed declared files. I found no broadening that silently filters reviewed upstream content.

Selected global-name overlap warnings are appropriately limited. `lib/harness.py:478–499` compares configured names and leaves the requested selection intact; its note asks the user to check actual discovery. It does not pretend to know whether globals are installed or which copy a client chooses. This also covers overlaps introduced by specialist profiles, rather than only the default foundation.

Preserved-copy reporting is correctly separate from internal jobs and lock entries. The scanner bounds receipt reads, recognizes unknown or malformed IDs without trusting them, reports linked paths without intentionally traversing them, and skips ordinary unmanaged directories without receipts. `tests/test_harness.py:315–431` covers selected overlaps, portable-to-normal migration, unknown/malformed/link cases and preservation. `tests/test_target_reporting.py:104–129` preserves rejection of a completely empty selection while allowing an active target with no selected jobs to report leftovers. Doctor's success decision still uses selected jobs and the current lock, not the appended diagnostic rows. The exception-handling gap above is the remaining edge.

The additional marketer, illustrator, animator and researcher aliases point to substantive authored leads. `tests/test_catalog_discovery.py:98–120` preserves the distinction between catalog discovery and project installability, while its invocation/profile check now recognizes singular and plural profile wording. The illustrator alias routes to editable vector guidance; the existing bitmap route remains tool-dependent. An alias does not establish a new artistic or research outcome.

Documentation at `docs/project-targets.md:35–48` accurately distinguishes warnings, preserved diagnostics, configured names, actual discovery and deliberate cleanup. It should remain accurate after the inaccessible-copy edge is repaired. The current verification record retains historical results rather than silently treating them as R3 executions.

## Evidence assessed

The primary's `suite-r3.log` reports **468 tests passed in 36.399 seconds**. This is supplied execution evidence, not my test run.

Current registration evidence reports **30 exact normal-plus-specialist copies** and **110 file states** preserved by repeat sync. The portable/migration record reports **24 portable copies**, then **8 selected copies and 16 preserved diagnostics** across both targets, with **84 skill files unchanged** and a four-entry current lock. The migration discovery record and documentation explicitly acknowledge that preserved copies remain discoverable. This is preservation plus visibility, not completed cleanup.

The current export record reports **40 original global payload files**, including selected license material, and passing wrapper checks. It does not claim plugin activation.

Archive compatibility evidence reports successful full archive parsing and one original/adapted payload for each of **14 eligible archive-mode sources**, with matching expected/actual hashes. It excludes manual-only and file-fetch delivery and does not execute upstream skills. Its recorded registry hash is the earlier R2 value; the durable verification record explicitly attributes applicability to unchanged reader, source and selected-entry definitions despite unrelated later aliases. I independently confirmed the current reader hash matches the recorded reader. I did not rerun network compatibility or independently certify every remote payload.

## Retained domain coverage and limits

The initial all-domain assessment and R2 qualification remain applicable: frontend and interface composition; vector/raster routing and motion; QA; backend/API/MCP work; databases/migrations; operations/release/desktop work; marketing/conversion/search; research; documentation/office artifacts; planning/product/debugging; analysis; and coordination/security/evaluation. R3 primarily improves discovery and diagnostics, without new skill payloads or craft trials. The 14 globals plus four complementary project skills retain the rich foundation; the full 12 remain available through the explicit portable profile.

Material unverified paths remain actual desktop/Claude skill selection, native Windows, physical devices, other browsers, live provider/infrastructure and production recovery, bitmap craft outcomes, native Office rendering and human visual acceptance. Historical guided domain outcomes retain their own fingerprints and limits. No additional runtime, dependency, service or mandatory global workflow is needed to address this review.

My R3 coverage comprises the changed harness reporting paths and callers, catalog link helper and CLI error handling, bundle filter and regression boundaries, alias metadata/routing checks, affected test bodies, changed target/verification documentation, selected historical fingerprint correction and supplied current registration/migration/export/archive evidence. No checks were executed by me. **One concrete P3 issue remains; no blocking revision is requested by this verdict.**
