# Codex skill surface audit

Static review on 2026-10-26 found two related project-selection defects. The
Codex-only boundary, nine-global/58-project shape, unbranded IDs and receipt-owned
native copies match the approved plan. The independent review made no native
calls; the integration repair and disposable Mac verification below followed it.

## Findings

### P2 — Failed selection activation leaves the new selection committed

`src/main.rs:1228–1235` saves or replaces `.mirket/project.json` before calling
`SkillSync::sync`. That sync can fail while acquiring a destination lock or
creating a directory (`src/skill_sync.rs:581–604`), before it records a skill
transaction. It can also roll back an application failure
(`src/skill_sync.rs:609–617`). Its transaction contains only the before/after
skill receipts, and rollback restores packages and the receipt
(`src/skill_sync.rs:830–871`); neither path restores the selection.

Thus a failed `skills project-update --include technical-docs` can leave
`technical-docs` pinned while the previous `architecture` copy remains active.
The CLI reports an error, but a later `project-sync`, `setup --project-root` or
`update --project-root` consumes the changed pin. The approved catalog contract
promises immediate activation and rollback of partial failure
(`planning/skill-catalog-design.md:173–175`). The selection and activated copies
need one recoverable transition, or an explicit pending state that cannot be
mistaken for a completed selection update. Unknown external edits must still be
preserved.

Bounded disposable reproduction for the integration owner:

1. Use an already-qualified disposable installed home and a new disposable
   project. Select `architecture` successfully and record selection, receipt and
   native-copy bytes.
2. As a normal macOS user, keep `.mirket` writable but temporarily remove write
   permission from the existing `.agents/skills` directory. Use a cleanup guard
   to restore its exact original mode. A held destination lock is an alternative
   deterministic failure injection.
3. Run `skills project-update --home HOME --project-root PROJECT --include
   technical-docs`. The read-only preflight should pass; activation should fail
   at the later native write/lock boundary.
4. Check all three saved surfaces after failure. The source predicts a new
   `technical-docs` selection with the original `architecture` package/receipt.
   Restore permissions before any follow-up command. No real user home or
   installation is needed.

The ordering and missing selection rollback are proven by source inspection.
The permission/lock reproduction was designed but not run in this review.

### P2 — Project doctor can report managed for an invalid or unapplied selection

`skills project-doctor` prints `manager.inspect()` directly
(`src/main.rs:1243`). `inspect` loads `.mirket/project.json` only when no receipt
exists (`src/skill_sync.rs:441–465`). With a committed receipt it checks the
recorded packages against that receipt and returns `managed` if their bytes
match (`src/skill_sync.rs:472–503`). It never reads or validates the current
selection on that path.

After one successful selection, a valid change to the pinned IDs, deletion of
the selection, or even malformed selection JSON can therefore still produce
`status: managed`. This also conceals the split state from the first finding.
Copy integrity and agreement with the saved selection are distinct facts; the
project doctor needs to report both. Internal receipt/rollback inspection can
remain focused on ownership so a new diagnostic does not accidentally block
recovery.

Bounded disposable reproduction:

1. Successfully select `architecture` in a disposable project.
2. Save the exact `.mirket/project.json` bytes, then replace `include` and
   `selected` with `["technical-docs"]`, leaving the schema and other arrays
   valid.
3. Run `skills project-doctor --home HOME --project-root PROJECT`. The source
   predicts `managed` with the `architecture` receipt despite the unapplied pin.
4. Separately try malformed JSON and a missing selection; each should be
   diagnosed without changing native copies. Restore the original file after
   each probe. A repaired doctor should retain ordinary managed success and
   existing drift detection as controls.

These outcomes were predicted from the branch structure and then exercised in
the disposable CLI fixture below.

## Repair and disposable Mac verification

`ProjectSelection::saved_bytes` captures the exact bounded prior pin, and
`restore_exact` restores those bytes (or removes a newly created pin) only while
the current bytes still equal the selection just written. It uses the existing
path and selection lock checks, preserves unknown edits and leaves directories
in place. The CLI now compares the receipt release, managed-copy statuses and
pending state after a failed sync. It restores the prior pin only when those
observations are unchanged; if a skill effect is uncertain, it preserves the
selection for reconciliation rather than claiming rollback. Project doctor
reports a structured `conflict` for missing, malformed, unknown or unapplied
pins with a committed receipt. Pending receipt status remains available for
reconciliation.

The six new focused cases and all 21 `skill_sync` tests passed. A fresh
disposable macOS ARM64 `design`+`skills` release was assembled from the current
core and catalog, installed to a private home, and passed full `doctor` with
the nine global skills, guidance and registration managed. Selecting
`architecture` returned `synced` and project doctor returned `managed`.
With project skill-directory writes temporarily disabled, updating to
`technical-docs` failed at the intended native-write boundary; exact selection,
receipt and native `architecture` hashes were unchanged after rollback, and
doctor remained `managed`. The original directory mode was restored by a
finally guard. A valid unapplied pin, malformed JSON and missing pin each
returned `conflict`; restoring original bytes returned `managed` without
changing the receipt. A subsequent normal update to `technical-docs` returned
`synced` and doctor returned `managed` for that exact skill.

The isolated fixture is
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/skill-selection-repro-20261026.DtfJgW`.
Its release ID is
`sha256:27529e2c633c3cc06e5aa98b1330a5e346619bab0b420c5f8b735c92c94c33cf`;
installed core SHA-256 is
`4d0b2c2519fc28b09506fbb361dce5073d8c0af4fe83ba75d5134bda09daf7a3`.
The failed-update and doctor-probe reports have SHA-256
`19e0170508a4623a571cb6262884a76e51edd1149a5bd736aef9e4f78c888fff`
and `621f30c1336790e406f50b252ceb8c0b80b574b2c27d04449ffe551533eb3a06`
respectively. This verifies the actual disposable CLI/filesystem path, not
Codex Desktop GUI activation or Windows behavior.

## Cross-installation transaction follow-up

An independent second pass found that the first rollback could still race a
different Mirket installation home targeting the same project. It also found a
no-op acknowledgement case: another home's `project-sync` could accept a new
pin with unchanged selected IDs while the first operation was deciding whether
to restore its old pin. Receipt comparison alone cannot identify that
acknowledgement.

`SkillSync::select_and_sync` now holds a project-owned operation lock across the
exact prior-byte snapshot, typed pin replacement, native sync and conditional
recovery. Public project sync, removal, reconciliation and removal completion
join that same lock before native mutation or a no-op sync result. The existing
native locks still use destination, receipt, then selection order. Recovery
checks native state and restores exact prior bytes while holding those native
locks; it leaves a newer native commit intact. A pre-write comparison failure
or an idempotent selection has no rollback token. Manual or hostile same-user
filesystem changes remain subject to the exact-byte/path checks, not a claim
of process containment.

The 25 focused skill-sync tests passed, including a competing native commit
between failed activation and recovery, and a same-ID/profile-only sync that
must refuse while the project operation lock is held. The full current-source
gate passed on macOS ARM64: 546 library tests, 11 CLI tests, one active
integration test, seven deliberately ignored cases, and strict Clippy with
warnings denied. A second source-only independent review found no further
concrete transaction or lock-order defect. The installed-CLI retest below is
against the final disposable binary, not the earlier first-repair fixture.

The disposable installed release
`sha256:06084a9203ebd60c572b4bc6e6af393b46ce35d88ab0dc9210d478ee71ed0895`
contains core SHA-256
`d338f769e0ce95d45741dfa20dcb57d6512a9af232d11a3a003962ea668cf1de`.
The actual installed `project-sync` refused while an independent process held
the project operation lock, then returned `unchanged` after its release. A
forced native-skill write failure during `project-update` returned an error and
left the exact selection, receipt and installed skill hashes unchanged; project
doctor remained `managed`. A normal update then returned `synced`, project
doctor remained `managed`, and full installation doctor reported `healthy`.
The fixture's `desktop_activation_verified` field is false. The final report
at `skill-selection-repro-20261026.DtfJgW/final-selection-repro.json` under
the private task-state root has SHA-256
`2f1a169e491acef17013dfc57e728f825f22dcecdfae774fb18df667ffec92fd`.
Final `src/main.rs` and `src/skill_sync.rs` SHA-256 values are
`fd2e03e0db87b3e344548cae6a650476ac35bc34e3f0c5b293608c354266e453`
and `7a6ef6980745f0a5fd747e0d5eaa6645f5008e7f4897eaf0673d2f1587980acb`.

## Confirmed alignment

- The saved catalog contains nine global skills, 58 opt-in project skills and
  eight profiles. All 67 body byte counts, SHA-256 values, native names and
  descriptions match their entries. The global bodies total 14,916 bytes; all
  bodies total 112,937 bytes. No missing relative body link was found.
- Setup installs only the global set and optionally synchronizes one explicitly
  supplied project's existing pin (`src/installation.rs:376–452`). Global and
  project destinations are `.agents/skills`; project ownership is recorded in
  `.mirket/skills-receipt.json` (`src/skill_sync.rs:294–325`). Profiles resolve
  once and sync consumes `selected`, so recipe changes do not silently enlarge
  an existing pin (`src/skill_sync.rs:149–181, 673–677`).
- The global paragraph selects Mirket as the primary route for its qualified
  operations, discloses an unavailable route, and retains Codex permission and
  agent authority (`src/codex_guidance.rs:18`). The native global runtime skill
  distinguishes installed bytes, running readiness and actual useful output;
  it permits reuse of unchanged readiness evidence.
- Skill ID validation explicitly rejects `mirket-` prefixes
  (`src/native_skills.rs:462–474`). Ownership comes from receipts and file hashes,
  with unmanaged collision refusal (`src/skill_sync.rs:711–717`), rather than
  claiming every similarly named personal skill.
- A targeted case-insensitive search found no Claude client adapter, Claude
  config path, old guide-read/skill-read API, model router or worker spawner in
  `src`, `adapters`, `scripts`, `tests` or installed skill bodies. The retained
  Anthropic references are catalog/provenance comparison or link-only sources,
  not executable Claude machinery.
- Browser/video-required project entries remain unavailable when their release
  component is unqualified. Current qualification maps only the admitted SQL,
  Design, Raster, OpenPencil and Diagram components
  (`src/installation.rs:959–985`). This is an honest open release gate, not a
  reason to advertise a catalog entry as a ready runtime.

## Optional simplification, not a current defect

`QualifiedCatalog::open` requires an exact human status sentence and the literal
`planned-not-run` per-skill evaluation status (`src/native_skills.rs:83–89,
158–170`). The evaluation-contract validator also requires `planned-not-run`
(`src/native_skills.rs:422–425`). This couples installation validity to one
authoring report state: recording completed evaluation would require a Rust
change even when shape, rights and body hashes are valid. A future schema could
use a bounded status vocabulary and separate evaluation evidence from the native
copy contract. The current saved catalog is valid, so this is a maintainability
proposal, not a release blocker established by this review.

## Scope and evidence

The review traced CLI selection, registration, setup/update integration, native
copy planning/inspection/rollback, the release assembler and catalog validation.
It read all nine global bodies and the operation-dependent browser/design/video
specialists, and checked metadata/frontmatter/hashes for every project entry. It
did not independently re-evaluate all 58 specialist methods or any upstream
reference body. A small read-only Python check established the inventory and
byte consistency above; it did not invoke a Mirket runtime, upstream helper,
installer or test build.

Pre-fix reviewed source identity:

| File | SHA-256 |
| --- | --- |
| `src/main.rs` | `39d79a7a27531a49ea561c3f1bbc3eed232b0deb09698033b4bf7b1e7333276d` |
| `src/skill_sync.rs` | `25c4561c024171847401f001cfc8fb8642e82a155682f0ea1d0917577456093d` |
| `src/native_skills.rs` | `06b570288ef6a8af61f9646dc3c56475083c786b2b233da574086f8b9e794a46` |
| `src/codex_guidance.rs` | `95c796310ffd5364080f01d1296cdc57a85592405140e6ef4cb30f190dea5d32` |
| `skills/catalog.json` | `990913ec44496bb9a22683f357fac9c0ae77596a3ec87d8a98023d2526b9394c` |

First-repair `src/main.rs` and `src/skill_sync.rs` SHA-256 were
`b708201cf4d17eed9966b17faaa2cbb9d690ec6d67adf85d56a6c710f390d815`
and `dd916aa6c0c6ac1c5c80ab5e041ae651d0f1a7ef87b9c75fc715fc420e641d5d`.

No actual Codex Desktop GUI activation, model adherence, matched-baseline skill
evaluation or native Windows behavior is established by this static audit. The
existing qualification documents retain their own platform and body-version
limits.
