# Independent Astra reassessment — R2

**Verdict: APPROVE_WITH_LIMITS. Both original P3 findings are closed. No new blocking or concrete nonblocking defect was demonstrated in the revised paths inspected.** The evaluation-report suggestion is also addressed.

This approves the configuration-layer responsibilities in `REVIEW-CRITERIA.md`: discoverable domain routes, substantive available guidance, selective and preserving installation, explicit runtime boundaries and truthful verification claims. It does not certify universal task quality, actual desktop skill selection, every upstream specialist, or production/device behavior. Those evidence limits remain the reason for the qualified verdict; they are not outstanding repair requirements.

## Review identity and method

I reviewed only `/Users/nazmi/Documents/.ai/build/dual-independent-review/astra-r2`, beginning with `REVIEW-CRITERIA.md`, `SNAPSHOT.json`, `REVISION-NOTES.md` and `PRIOR-REVIEW.md`, which contains my own R1 report. The declared R2 source fingerprint is `b7da9b96692a824be2ebbeb930dcd5f8570b98c52df5b1a275ef5b1222aa8f93`.

This was a static reassessment. I did not execute project code, tests, installers, model invocations or browser sessions; use the network; delegate; inspect another review; or follow evidence paths outside the assigned snapshot. All file/line references below are relative to that snapshot. I independently hashed the five changed implementation files, both registries, both instruction files and the supplied current suite log. The code/configuration hashes match `SNAPSHOT.json` and the applicable entries in `review-evidence/current-verification-final.json`. I did not recompute the aggregate snapshot fingerprint.

The current suite log hashes to `b80fcdb6298e870041e6ab37f3875f8a97f6722a39b96a9a4e91925e24ac974e`. Reading and hashing that log verifies its identity, not its execution independently of the primary agent.

## Findings resolved

### Closed: byte-identical source updates had ambiguous provenance

`lib/harness.py:345–371` now explicitly defines the receipt as original-installation provenance and the lock as the current selection. For a validated unchanged copy, it compares repository, commit, path and source hash, and exposes both origins when they differ. It reports the matching installed hash and executable contract and expressly states that this does not claim the current source was fetched or reinstalled. The diagnostic reaches plan/sync/doctor output through `project_jobs` and `public_project` at lines 391–417.

This resolves the actual ambiguity without inventing a new download or needlessly rewriting installed bytes. The existing `project_state` validation remains ahead of that diagnostic. The changed code does not grant permission to replace a modified payload simply because provenance differs.

I inspected the new tests at `tests/test_harness.py:491–588`. They cover a pin-only change, plan/sync/doctor agreement, no new preparation, receipt/payload/mode/timestamp preservation, unchanged repeat sync, and different source/adaptation metadata producing the same installed content. Negative cases preserve user changes and reject altered execute bits. `docs/project-targets.md:86–90` gives the same semantics to users. These are meaningful checks of the original trigger, not merely assertions of the helper's return shape.

**Closure limit:** The tests were not executed by me. The closure rests on inspected control flow, relevant test cases and the primary's supplied current regression result. Provenance remains a diagnostic record, not independent proof of a remote source fetch.

### Closed: unpinned global exports could include interpreter residue

`lib/bundle.py:162–182` preserves normal local source validation and then excludes `__pycache__` members and `.pyc`/`.pyo` files only for a global entry without an explicit source or installed hash. Pinned globals, project payloads and upstream payloads retain their reviewed byte contracts. Selected-file requirements are checked again after filtering, and an executable declaration cannot silently point to a removed file.

The export continues to calculate its returned payload/provenance from the actual prepared files. This closes the machine-dependent payload problem without weakening hash enforcement globally.

`tests/test_bundle.py:219–281` exercises the important boundaries: full export equality before/after generated caches, retention of a real helper, unexpected bytecode rejected in reviewed project payloads, explicitly reviewed bytecode preserved, explicit global hashes still enforced, and excluded executable/extra-file declarations rejected. The code intentionally validates before ignoring residue; it does not use the filter to evade unsafe-file or resource-budget checks.

**Closure limit:** I did not run the wrapper or exports. I inspected the implementation and regression bodies and read the supplied export/test evidence.

### Addressed: aggregate evaluation reports hid reviewer kind

`lib/evaluation.py:389–415` now retains `review_kind` and `reviewer` only for a valid current review, adds `accepted_by`, and reports separate kind/acceptance counts. `accepted_by` is `none` for incomplete or unsuccessful acceptance, including cases where automated checks fail despite a recorded accepted review. Automated-only acceptance retains the prior boolean semantics while carrying no reviewer attribution.

The cases at `tests/test_evaluation.py:89–157` cover agent/human reviews, no required review, stale/invalid/pending records, declined reviews and failed checks. `docs/quality-harness.md:75–83` explains that these are recorded labels, not runtime attestation or an automatic claim of human approval. Changing five rubric headings to “Artifact review” is consistent with this distinction; historical case hashes must remain historical, as the documentation now states.

## Additional revisions and regression assessment

| Revised boundary | Assessment and limits |
| --- | --- |
| Default foundation | The four project entries are architecture-review, debugging, test-design and release-operations. Combined with the unchanged set of 14 globals, they retain all 12 original foundation capabilities without duplicating the eight shared names. `portable-foundation` explicitly retains the full 12 for environments without those globals. This is a leaner placement of guidance, not a removal of its substance on a configured host. |
| Availability and migration | `skills/skill-catalog/SKILL.md:40–46` requires checking global availability and selecting the portable option when absent. Instructions and `docs/project-targets.md` agree. Old unselected copies are deliberately preserved, with an explicit warning that changing the selection alone does not remove duplicates. No real-project cleanup or successful migration is claimed. |
| Discovery | Catalog tags expose designer, documenter, tester, planner and debugger leads. `tests/test_catalog_discovery.py` verifies expected first results and distinguishes globally available leads from project-installable entries. The task-routing reference now explicitly separates delivery planning from product prioritization, and checks link documented invocation names/profiles to catalog entries. No new domain-quality outcome follows merely from an alias. |
| Claude effort | `lib/claude_delegate.py:218–233` probes the exact flag when an effort is requested; lines 327–369 validate and forward the exact supported token. Omission preserves the default. The new tests cover every accepted value, invalid values rejected before CLI access, missing/prefix-only flag names and no implicit fallback. The reference correctly leaves model/provider support to the CLI and does not promise a live invocation. |
| Project runtime readiness | `lib/runtime.py:589–595` makes project-mode `ready` reflect requested project checks and preserves aggregate client readiness as `clients_ready`. Non-project behavior remains in `diagnose`; explicit authentication attention still affects exit status at line 614. `tests/test_project_readiness.py:228–277` covers both directions of project failure/success, missing clients, explicit auth and non-project semantics. This avoids making unrelated optional clients prerequisites for a valid project check. It still does not establish engine compatibility, application health or visual acceptance. |
| Global instruction weight | The actual files are 93 lines for AGENTS and 41 for Claude guidance. The former remains below the stated 100-line bound. Current documentation labels old counts as historical. No new default runtime, service, MCP or production dependency is introduced. |

I found no source-level regression in these changes. In particular, the fixes preserve explicit authorization boundaries, selected payload contracts, no-fetch idempotence, and separation between registration/readiness and acceptance. Complete-installation readiness has not been weakened by the project-mode CLI adjustment.

## All-domain assessment retained

The detailed R1 capability matrix in the supplied `PRIOR-REVIEW.md` remains applicable to the substantive guidance not changed by these repairs: frontend behavior and interface composition; vector/raster routing; motion mechanics; QA; backend/API/MCP boundaries; databases and migrations; DevOps/release/desktop work; marketing/conversion/search; research; documentation/office artifacts; planning/product/debugging; analysis; and coordination/security/evaluation.

The material updates to that matrix are placement and discovery: the rich foundation is now the union of globals and four complementary project skills, with an explicit portable alternative; role aliases and delivery-planning routing are more direct; and evidence summaries expose their reviewer kind. The revision does not establish new craft outcomes in any domain. Earlier frontend, motion, SVG, analysis and SQLite exercises retain their original fingerprints, environments and narrow acceptance scope. I did not repeat the R1 substantive skill-body review or claim that those historical trials ran against the R2 aggregate source.

## Current supplied evidence

I inspected the current evidence records, rather than treating revision prose alone as proof:

- `current-suite-after-repairs.log` reports **462 tests passed in 35.313 seconds**. This is primary-supplied execution evidence.
- `current-registration-final-check.json` reports **30 exact default-plus-specialist copies** across scratch targets and **110 file states** unchanged by repeat sync, including bytes, modes and timestamps.
- `current-portable-final-check.json` reports **24 exact portable copies**, explicitly limiting the claim to scratch file registration rather than a live host without globals.
- `current-export-final-check.json` reports **31 original skill files** retained and effort/alias wrapper checks passing. It expressly excludes client plugin activation.
- The before/after Codex discovery records list **16 then 8 entries** for the eight formerly duplicated names: two scopes before and only global copies after. Their method is a read-only CLI app-server listing, with no model turn. I did not access the paths listed in those records. They do not establish desktop selection or Claude invocation.

None of these records is a controlled all-domain quality benchmark. That is an appropriate limit for the configuration acceptance scope, not grounds to require an additional runtime.

## Open findings, suggestions and review coverage

**Open concrete defects: none identified in this focused reassessment. Required revisions for this verdict: none.** The R1 suggestions to preserve fingerprinted, narrowly attributed domain evidence and avoid adding a heavyweight runtime remain sound operating advice, not blockers.

Material unverified capabilities remain: actual desktop skill activation/selection, full remote upstream bodies, native Windows behavior, physical devices and OS preference changes, other browsers, live provider/service/cloud behavior, production deployments and recovery, raster-generation output, native Office rendering and human visual acceptance. Global availability must be checked on each target; the reduced project foundation alone cannot supply absent globals.

R2 inspection covered the changed provenance/export/evaluation paths and their surrounding validation/publication behavior; effort parsing, capability probe and invocation flow; runtime project-readiness result/exit flow; the new targeted regression bodies in harness, bundle, evaluation, Claude-delegate, project-readiness, catalog-data and catalog-discovery tests; foundation/global registry selections and alias metadata; changed instruction/selector/routing/Claude-reference guidance; relevant project-target, quality and verification documentation; and the current evidence records enumerated above. I independently checked the selected hashes and instruction line counts using read-only utilities.

I did not run any project tests or claim their supplied results as my execution. The R1 coverage inventory defines the retained baseline; this report defines the narrower R2 reassessment and closes its two concrete findings.
