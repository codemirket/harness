# R2 independent reassessment (read-only)

## 1. Verdict: **APPROVE_WITH_LIMITS**

Nothing blocks approval. My two R1 P2 findings are addressed: D1 fully, and D2 for the default setup path. The R2 code changes I read introduced no safety regression. Four P3 items remain, all leftovers or inconsistencies of the same kind.

**Approved scope.** R1's scope still stands: routing, authored guidance, pinned and fail-closed installs, per-domain delivery evidence, and honest labelling. R2 adds four approved behaviours:
- **Foundation:** new projects get a four-skill default with no global-name overlap, plus an explicit 12-skill `portable-foundation` for machines without the globals.
- **Evaluation reports:** each run row now records who reviewed it (`review_kind`, `reviewer`) and who accepted it (`accepted_by`).
- **Readiness:** in project mode, the exit status depends on the project's own checks.
- **Claude delegate:** an optional, validated `--effort` flag with no fallback.

**Still not established:**
- any improvement in delivered quality;
- which copy Codex desktop picks when two skills share a name, and Claude's actual precedence (the docs cite sources; nobody has observed either);
- Vue/Nuxt and DevOps depth comparable to the React, design and motion coverage;
- observed QA, DevOps, marketing or planning outcomes;
- native Windows behaviour;
- role-name search for marketer, illustrator, animator and researcher (N3).

---

## 2. Disposition of my R1 findings

| ID | Status | Evidence (my static inspection) |
|---|---|---|
| **D1** report turned agent review into `accepted` | **Resolved** | See below. |
| **D2** default foundation duplicated 8 global names | **Resolved for the default path; leftovers in N1 and N2** | See below. |
| **D3** readiness exit depended on every client | **Resolved** | See below. |
| **D4** evaluation domain allowlist | **Unchanged**, as the revision notes state (`lib/evaluation.py:24-25`) | Not a configuration bug. Claims for design, motion, QA, operations, marketing and planning stay narrow. |
| **D5** evidence labels and fingerprints | **Mostly resolved** | Current diagnostics carry `reader_sha256` and `registry_sha256` (`review-evidence/current-verification-final.json:3-4`). The historical AGENTS line count is labelled (`docs/verification.md:77`). Leftovers are in N4. |
| **D6** catalog vs frontmatter descriptions | **Resolved** | New test at `tests/test_catalog_data.py:73-81`. Spot checks: `registry/catalog.json:376` equals `skills/debugging/SKILL.md:3`, and `:328` equals `skills/marketing-writing/SKILL.md:3`. |
| **D7** routing table untested | **Mostly resolved** | See below. |

**D1 detail.**
- Code (`lib/evaluation.py`):
  - Reviewer kind and identity are kept only when the review is current and valid (`:401-402`).
  - `accepted_by` is set only when checks passed and the review accepted (`:403-404`).
  - Per-kind and per-acceptor counts are added (`:413-415`).
- Tests: `tests/test_evaluation.py:89-157` cover agent, human, declined, invalid, stale, pending and failed-check runs.
- The five rubrics are now titled "Artifact review" (`evaluations/cases/*/rubric.md:1`).
- The semantics are documented in `docs/quality-harness.md:76-84` and `skills/ai-system-evaluation/references/harness-development.md:70-72`.

**D2 detail.**
- New defaults: `registry/catalog.json:14861-14869` (four skills) and `:15410-15426` (portable, 12 skills).
- Tests: `tests/test_catalog_data.py:16-23,25-47`.
- Docs: `docs/project-targets.md:25-33`, `setup/README.md:198-204`, `skills/skill-catalog/SKILL.md:20-23,40-46`, `instructions/AGENTS.md:17-19`, `instructions/CLAUDE.md:14-18`.
- Primary-supplied: a Codex CLI app-server listing showed 16 entries for the eight names before and 8 after (user scope only).

**D3 detail.**
- `lib/runtime.py:589-595,614`: project mode sets `ready` from the requested project checks and reports `clients_ready` separately; explicit auth failures still exit 1.
- Tests: `tests/test_project_readiness.py:228-277`.
- The readiness paragraph now mentions `--browser` (`skills/skill-catalog/SKILL.md:79-80`).
- Still unproven: the default Chrome capture on the user's Mac.

**D7 detail.**
- `tests/test_catalog_discovery.py:36-56` parses ``` `id` (invocation `name`) ``` pairs and ``` `x` profile ``` mentions from the routing table.
- Not parsed:
  - bare lead IDs;
  - `openai-playwright` "invokes as `playwright`" (`task-routing.md:31`);
  - "`marketing-*` profiles" in the plural (`:45`);
  - `motion-studio`, `operations` and `testing-web` when not followed by the word "profile".
- I checked all of these by hand and every one exists, e.g. `catalog.json:10541/10555` (name `playwright`), `:11287`, `:11313`, `:14727`, `:15016`, `:15137` and `:15400`.

**Other R2 changes I read: no regressions found.**
- **Effort flag** (`lib/claude_delegate.py`):
  - values are limited to a fixed list (`:24`) and validated before any CLI call (`:343-344`);
  - `--effort` support is probed only when the flag is requested (`:230-233,355-356`);
  - the exact value is passed through (`:368-369`), with no fallback;
  - tests cover this against a fixture CLI (`tests/test_claude_delegate.py:163-194,364-378`), and the guidance documents it (`skills/agent-coordination/references/claude-code.md:32-38`).
- **Bundle residue** (`lib/bundle.py:172-181`): `__pycache__`/`.pyc`/`.pyo` files are removed only for unpinned global entries, and only after validation. Pinned entries still fail on unexpected bytes (`tests/test_bundle.py:241-269`). Excluded files that are declared executable or extra are rejected (`:271-281`). The test suite also stops bytecode being written into pinned helpers (`tests/test_svg_audit.py:14-20`).
- **Provenance reporting** (`lib/harness.py:345-371,393-398`) only reports. It does not rewrite receipts, and the tests check that timestamps are preserved (`tests/test_harness.py:491-563`).
- **Other regression checks:**
  - Skipping an ID that is no longer selected stays valid; only unknown IDs are rejected (`lib/catalog.py:687-689`).
  - No catalog entry `requires` any of the eight overlapping IDs (multiline search found none).
  - Nightly maintenance reconciles global items only (`lib/maintenance.py:358`), so existing projects are not changed automatically.

---

## 3. Remaining and new defects (all P3, none blocking)

### N1: Routed specialist profiles bring back the global-name duplicates
- **Trigger.** On a machine with the globals installed, follow the routing table, e.g. `project init --profile operations` (the DevOps row, `task-routing.md:30`), `--profile testing-web` (QA, `:31`) or `--profile documents` (`:35`).
- **Location.** These profiles still list globally installed IDs (`registry/catalog.json`):

  | Global ID | Profiles that still include it |
  |---|---|
  | `ci-maintenance` | `engineering-core` (`:14876`), `maintenance` (`:14927`), `full-stack` (`:14954`), `testing-web` (`:15023`), `operations` (`:15144`) |
  | `agent-coordination`, `context-management` | `collaboration` (`:14918-14919`) |
  | `research-and-synthesis` | `ai-systems` (`:15170`), `research` (`:15176`), `idea-research` (`:15383`) |
  | `document-parsing` | `data-pipelines` (`:15160`), `research` (`:15177`), `documents` (`:15186`) |
  | `office-authoring` | `documents` (`:15185`) |
  | `ai-system-evaluation` | `ai-systems` (`:15168`); it is global per `registry/harness.json:17` |

  The duplicate-name test covers only the default profile (`tests/test_catalog_data.py:16-23`).
- **Consequence.** This produces D2's effect on a smaller scale, through the very rows the router recommends. Each such profile adds one to three same-name copies:
  - Codex lists both copies (primary observation for the default case);
  - the pinned project copy can drift from the live global copy;
  - the repository's own rules are contradicted: "Avoid duplicate global/project names" (`skills/skill-catalog/SKILL.md:20-21`) and "Do not install both versions by default" (`docs/project-targets.md:27`).
- **Smallest remedy.** Remove the nine project-scoped global IDs from every profile except `portable-foundation`. An alternative is to emit a `global_name_overlap` warning in plan, add and sync. Then extend the test to assert that no other profile resolves to a global name.
- **Check.** The extended test fails before the change and passes after. Then run an app-server `skills/list` after `project init --profile operations --profile testing-web --profile documents`.

### N2: Existing projects keep eight untracked, frozen copies that nothing reports
- **Trigger.** Run `project sync` in any project that previously synced the 12-skill foundation.
- **Location.**
  - Plan and sync build jobs and the lock from the current selection only (`lib/harness.py:374-405,448-479`).
  - Doctor checks only the selected rows and the lock (`:674-681`).
  - The only mitigation is documentation (`docs/project-targets.md:31-33`, `setup/README.md:201-204`).
- **Consequence.**
  - The lock drops the eight entries.
  - Their copies stay on disk and remain listed by Codex (compare the 16 entries in `review-evidence/current-codex-discovery-before.json`).
  - Sync no longer updates them, so they freeze at their old pins.
  - No command identifies them.
  - Compared with R1, where sync kept them current, this is mildly worse for existing projects. It is documented, and safe because nothing is deleted.
- **Smallest remedy.** In plan and doctor, report (never delete) receipt-bearing copies under `.agents/skills` and `.claude/skills` whose IDs are no longer selected, and flag any that share a name with a global.
- **Check.** In a scratch project, sync `portable-foundation`, then switch the manifest to `project-foundation`. Plan should list eight unselected copies with the overlap flag, and file states should stay unchanged.

### N3: Role-name aliases cover five of the requested roles
- **Location.**
  - Tags added: `designer` (`catalog.json:320`), `documenter` (`:368`), `debugger` (`:396`), `tester` (`:429`), `planner` (`:11384`).
  - Test: `tests/test_catalog_discovery.py:98-116`.
  - Matching is a substring test over entry fields and tags (`lib/catalog.py:751-755`).
- **Consequence (my static inference, not executed).**
  - `marketer` matches only indexed-only entries (e.g. `registry/source-index.json:9466,22409`).
  - `animator` matches only `magic-animator` (`:21932`) and similar.
  - `researcher` matches only entries like `wiki-researcher` (`:36830`).
  - `illustrator` matches nothing anywhere in `registry/`.

  This is the same class of failure as `review-evidence/role-aliases-before.log`, but for marketing, motion, research and illustration.
- **Remedy.** Add tags: `marketer` → `marketing-writing`, `illustrator` → `svg-creation`, `animator` → `motion-design`, `researcher` → `research-and-synthesis`. Extend the test's route table to match.
- **Check.** The extended test fails before the tags are added and passes after.

### N4: R2 verification is not recorded in the repository; some historical claims read as current
- **Location.**
  - `docs/verification.md:3-10`: the newest entry is still the 441-test run.
  - `docs/verification.md:60`: "project-foundation profile contains 12…" in the present tense.
  - `docs/verification.md:76`: "12 foundation skills".
  - `docs/reviews/agent-runtime-hardening.md:136`: "24 byte-exact copies".
  - `docs/evidence/agent-runtime-hardening-2026-10-08.json:80`: `catalog_sha256` still holds the `lib/catalog.py` hash (compare `:38`).
  - No `docs/evidence` file contains the 462-test result, the 30/24-copy registrations or the app-server listings. These exist only in `review-evidence/`, which `SNAPSHOT.json:208-211` treats as an excluded build artifact.
- **Consequence.** After merge, the repository's verification record understates the current state and contradicts the new profile.
- **Remedy.** Add a dated `docs/evidence/*.json` with reader, registry and source fingerprints, plus a `verification.md` entry. Mark the 12-skill rows as historical and rename the mislabelled field.
- **Check.** The existing link and generated-table checks, plus a grep for "contains 12".

---

## 4. Capability matrix: changed cells only

All other R1 cells are unchanged, including the limits on Vue/Nuxt depth, DevOps references, Docker, Node-only readiness, bitmap craft and Windows.

| Role | Change in R2 | Limit |
|---|---|---|
| **Planning** | **(a)** New "Plan or resume substantial delivery work" row (`task-routing.md:42`) and the `planner` alias. | **(d)** Unchanged: guided trials only. |
| **Design / Documentation / QA / Debugging** | **(a)** The `designer`, `documenter`, `tester` and `debugger` aliases rank the authored lead first. Global leads are not claimed as project-installable (`test_catalog_discovery.py:103-116`). Red/green logs are primary-supplied. | — |
| **Marketing / Illustration / Motion / Research** | **(a)** Outcome-based routing rows unchanged. | Role-name search is unrouted (N3). |
| **Cross-cutting install** | Four-skill default plus explicit 12-skill portable profile. | Leftover duplicates via profiles (N1) and in existing projects (N2). Host desktop selection unverified. |
| **Evaluation (all roles)** | **(d)** Reports separate agent from human acceptance (D1). | No new outcome runs. The rubric hash changes make old runs historical (`quality-harness.md:81-84`). |
| **Frontend / readiness** | **(c)** Project-mode exit status fixed. `--browser` fallback documented. | Default capture on the user's Mac still unproven. |
| **Delegation** | Optional validated `--effort`. | Whether the real Claude CLI accepts each level for each model is unverified; the wrapper fails closed. |

---

## 5. Non-blocking suggestions and unverified behaviours

**Suggestions:**
- For passing cases that need no review, use `accepted_by: "automated"` instead of `"none"` (`lib/evaluation.py:404`; test at `tests/test_evaluation.py:115-117`). As written, `accepted_by_counts` lumps them together with unaccepted runs.
- The bundle's global residue filter does not exclude `.DS_Store` (`lib/bundle.py:176-177`). The global-sync file walk and the snapshot both do (`lib/harness.py:59`, `lib/bundle.py:96`). A Finder-created file could ship in an exported plugin.
- Widen the D7 routing parser to cover bare IDs and the "invokes as" phrasing.
- R1 suggestions still apply: Vue/Nuxt and DevOps worked references, optional Claude at install time, review gating for live links, and `project add` re-inserting the foundation (`lib/harness.py:553-555`; now cheaper with four entries).

**Unverified behaviours (by me and by the supplied evidence):**
- Which same-name copy Codex desktop selects.
- Claude's personal-over-project precedence. It is cited (`docs/project-targets.md:28-29`), but I did not fetch the source and the evidence does not observe it.
- Search results for role nouns other than the five tested ones (N3 is an inference).
- Real-CLI acceptance of `--effort` values.
- Native Windows behaviour and physical devices.
- Any QA, DevOps or marketing outcome.

**Incidental observation, not a test:** this session's system text matches `lib/claude_delegate.py:42-52` word for word, and only Read/Glob/Grep are exposed. That is consistent with the runner, but I cannot observe the effort value.

---

## 6. Evidence scope

**Primary-supplied (I did not run any of it):**
- `current-suite-after-repairs.log`: 462 tests in 35.313 s.
- `current-registration-final-check.json`: 30 exact copies, and 110 file states preserved on a repeat sync.
- `current-portable-final-check.json`: 24 portable copies.
- `current-codex-discovery-before/after.json`: the app-server listings.
- `current-export-final-check.json`: the export run from `/tmp`.
- The role-alias red/green logs. They report 15 discovery tests; the file now has 16, so the logs predate the final version.
- `global-doctor.log`, which has no fingerprint or timestamp.

**My static inspection:**
- Read the code paths and tests cited above.
- 462 `def test_` definitions across 24 files, consistent with the log.
- String comparison only: 15 registration and 7 portable hashes all match catalog strings.
- Checked the routed IDs and profiles by hand.

**Files inspected:**
- Review inputs: `REVIEW-CRITERIA.md`, `SNAPSHOT.json`, `REVISION-NOTES.md`, `PRIOR-REVIEW.md`, and all of `review-evidence/`.
- Instructions and routing: `instructions/{AGENTS,CLAUDE}.md`, `registry/harness.json`, `skills/skill-catalog/{SKILL.md, references/*, scripts/harness.py}`.
- Registry: `registry/catalog.json` (profiles, authored entries, Playwright) and the relevant parts of `source-index.json`.
- Code: `lib/{evaluation,runtime,claude_delegate,bundle,harness}.py` (changed paths), parts of `lib/catalog.py`, and a search of `lib/maintenance.py`.
- Tests: `tests/test_{evaluation,catalog_data,catalog_discovery,claude_delegate,project_readiness,svg_audit}.py` and parts of `test_bundle.py` and `test_harness.py`.
- Docs: `docs/{quality-harness,project-targets,verification}.md` (relevant parts), `docs/reviews/agent-runtime-hardening.md` (lines 120-156), the matching evidence JSON (lines 1-110), `setup/README.md` (lines 138-207), `README.md:5`, the plugins and runtime-integrations notes.
- Skills and evaluations: the frontend-engineering Vue/Nuxt lines, the agent-coordination and ai-system-evaluation references, and one rubric.

**Not executed:** the test suite, any CLI command, hashing, rendering, network access (including the cited official docs), and anything on Windows or Linux.
