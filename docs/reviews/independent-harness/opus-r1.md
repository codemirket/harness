# Independent harness acceptance review (read-only)

## 1. Verdict: **APPROVE_WITH_LIMITS**

I found no blocker. Two P2 defects should be fixed before relying on evaluation reports or growing the project foundation.

**Approved scope.** The current snapshot is approved as a configuration layer that does four things:

- **Routing.** It routes all requested roles to reviewed workflows. All 18 catalog IDs and invocation names in the routing table exist, have project scope, and match their install names. I checked each one by hand, for example `registry/catalog.json:5641/5657`, `:5918/5934`, `:7115/7131`, `:9795/9815`, `:10140/10159`, `:10537/10551`, `:13017/13024`.
- **Guidance depth.** It ships substantive authored skill bodies with worked reference material ("craft references") for:
  - interface design, vector illustration, the four motion specialists and frontend behaviour;
  - QA, backend/integration and databases;
  - research, documentation, product/work planning and debugging.
- **Safe installation.** Global and project copies are hash-pinned. Companions and conflicts are resolved, ownership receipts are kept, and unmanaged or modified content is preserved rather than overwritten (fail-closed). This comes from reading `lib/harness.py` and `lib/catalog.py`; the passing suite is primary-agent evidence, not mine.
- **Honest evidence.** It defines completion evidence per domain (`skills/skill-catalog/references/delivery-standards.md`). Its narrative docs label agent review, development fixtures and untested paths honestly.

**Not approved or not established:**
- any delivered-quality improvement;
- automatic discovery or selection of skills by the host apps;
- Vue/Nuxt/SvelteKit or DevOps depth comparable to the React, design and motion coverage;
- observed outcomes for QA, DevOps, marketing or planning work;
- native Windows behaviour;
- an evaluation-report `accepted` state that means human acceptance (D1).

---

## 2. Capability matrix

Legend: (a) routing/discovery · (b) installed guidance and specialist depth · (c) prerequisites and delivery gates · (d) observed evidence vs untested paths.

| Role | (a) Routing / discovery | (b) Guidance depth | (c) Prereqs & gates | (d) Observed vs untested |
|---|---|---|---|---|
| **Frontend engineering** | `task-routing.md:23,27` → `frontend-engineering`. Project-only, not in the foundation; available via profiles (`catalog.json:14937-14960,15324-15361`). | `frontend-engineering/SKILL.md` covers render boundaries, state, mutations, sessions and performance. `references/async-state.md` has worked race, double-submit and optimistic-update examples. React/Next has upstream specialists. **Vue/Nuxt/SvelteKit get only `SKILL.md:29-35` plus doc links; no catalog ID mentions vue/nuxt/svelte.** | Runtime doctor checks Node, package manager, git and dependency presence, plus an optional loopback probe and one 1280×900 capture (`lib/runtime.py:478-507,509-566`). Gates: `delivery-standards.md:32-59`. | One fictional static case, agent-reviewed (`docs/quality-harness.md:113-116`). Real-project improvement is explicitly not claimed (`docs/reviews/skill-routing.md:47-50`). The user's reference task is a **Nuxt** editor (`docs/evidence/project-readiness-2026-10-07.json:39`). |
| **Visual / interface design** | Global `interface-design`; rows `task-routing.md:23-26`. Redesign and prototyping profiles exist; style profiles declare mutual conflicts (`catalog.json:5561-5636`). | `interface-design/SKILL.md` plus application-composition, public-page-composition and art-direction references. These are concrete, with worked corrections. | Comparable before/after renders across widths, themes and states (`delivery-standards.md:41-51`). The capture tool is a single desktop size only. **Default desktop Chrome capture timed out on the primary Mac** (`project-readiness-2026-10-07.json:86`). | Agent review only. No human visual acceptance and no real-project before/after. |
| **Illustration** | Vector: `task-routing.md:39`, `svg-studio`. Bitmap: native image tool (`:45`). | `svg-creation` plus `vector-craft.md` (Bézier, optical alignment, instance IDs, accessibility) and `scripts/audit_svg.py`, a structural check only (stated at `SKILL.md:81-84`). **Bitmap has a route only, no craft guidance.** | `delivery-standards.md:61-67`; the audit script needs only the Python standard library. | SVG exercise in Chromium at 320/800 px, accepted by the primary agent (`svg-studio-2026-10-08.json:241-263`). Bitmap generation was never exercised. |
| **Motion** | Four leads with an explicit split (`task-routing.md:10-15,36-40`); `motion-studio` profile. Search ranks the specialists first (`motion-studio-2026-10-08.json:123-158`). | `motion-design`, `page-transitions`, `advanced-motion` and `svg-animation`, each with a technically precise reference (e.g. view-transition flattening in `scene-mechanics.md:146-150`, `pathLength` drawing in `svg-motion-craft.md:79-102`). | Browser required; reduced motion at load and on live change; sampled frames are not performance evidence. | Extensive Chromium checks. Payload hashes still match the current catalog (`motion-studio…json:161-165` vs `catalog.json:484,515,547,14752,14790`). Safari, Firefox and physical devices untested. Its routing reads predate the current `task-routing.md` edit. |
| **QA / test engineering** | `task-routing.md:31`; `test-design` is in the foundation. Discovery test for "quality assurance" (`tests/test_catalog_discovery.py:57-73`). | `test-design` plus `quality-assurance.md`: oracles, checking effects after reload, a failure-probe table and device-evidence limits. | `delivery-standards.md:102-113`. `openai-playwright` needs Node and wraps an **unpinned npx package** (`catalog.json:10550-10561`). Readiness does not probe Playwright. | **No QA journey exercise.** Only partly covered by the frontend case's browser checks. Supported claim: guidance and routing only. |
| **Backend (services, APIs, databases, integrations)** | Rows `:27-29,32-33`. Backend profiles cover NestJS, FastAPI/Django, .NET, Spring, Go gRPC, Rails and Laravel (`catalog.json:14961-15017`); also databases, integrations and MCP. | `engineering-judgment` plus boundary-decisions and service-integration references (worked lost-response and webhook cases). `database-systems` plus four engine references (PostgreSQL facts checked, e.g. `postgresql.md:21`). `data-migrations`, `mcp-integration`. | **Readiness probes Node only** (`lib/runtime.py:563`). Gates: `delivery-standards.md:69-100,149-162`. | Ledger, paginated-client, SQLite-backfill and rollout cases pass with agent review. The baseline also passed the non-visual cases, so **no quality gain is shown** (`quality-harness.md:100-104`). No Postgres/Redis or live-provider exercise. |
| **DevOps** | `task-routing.md:30`. `release-operations` and `ci-maintenance` are in the foundation; operations, kubernetes and cloud-gcp profiles exist. The "DevOps" search query resolves (`test_catalog_discovery.py:59`). | `release-operations` (~50 lines) and `ci-maintenance` (~48 lines) are sound but have **no worked reference**. Terraform and observability skills are persona-style upstream entries with caveats (`catalog.json:10167-10174`). **Docker is manual only ("adapt-first")** (`catalog.json:8227-8243`). There is no CI-provider-specific workflow. | `delivery-standards.md:95-100`. Readiness never checks docker, terraform, kubectl or cloud CLIs. | **None for DevOps craft.** The scheduler and maintenance tests cover the harness itself, not user projects. |
| **Marketing** | Global `marketing-writing`; rows `:44,46`; marketing strategy/growth/content/community profiles. | `marketing-writing` plus `editorial-preservation.md` (worked claim-preserving edits). Authored conversion-review, experiment-design, customer-research and search skills; many upstream marketing skills. | `delivery-standards.md:129-134`: no invented proof; conversion lift is a hypothesis. | One editorial trial reviewed by an agent (`harness-upgrade.md:131`). No evaluation domain, rendered page or conversion evidence. |
| **Research** | Global plus foundation; row `:41`; separated from SEO and local code lookup. | `research-and-synthesis` plus `claim-verification.md`. | `delivery-standards.md:164-175`. | Cache-decision case: candidate agent-reviewed, baseline review still pending (`quality-harness.md:104-105`). |
| **Documentation** | Global `document-workflow`; rows `:34-35`; `addy-documentation-and-adrs`; documents profile. | `technical-documentation.md`: claim ledger, examples run exactly as published, ADR scope. Office authoring and parsing are in the foundation. | `delivery-standards.md:136-147`. | CLI-docs case; candidate README agent-reviewed. |
| **Planning (work and product)** | `work-planning` is global and in the foundation, triggered by its description and `AGENTS.md:38-39,77-79`; it has **no routing-table row**. Product work → row `:42` → `product-management` (project skill). | `work-planning` plus `delivery-review.md` (worked bulk-edit example); `product-management` plus `decision-to-delivery.md`. | `delivery-standards.md:26-30,117-120`. | Guided continuity trial (`agent-runtime-hardening-2026-10-08.json:280-314`) and the rollout-decision case. No planning evaluation domain. |
| **Debugging** | Rows `:27,30`; `debugging` is in the foundation. | `debugging` recipe (falsifiable hypotheses, bisection, condition-based waits) plus `engineering-judgment` "Diagnose a failure". | Engineering gates. | The ledger case is a repair, and both conditions passed. No intermittent-bug exercise. |

**Installation and targets (cross-cutting):**
- Destinations:
  - Codex: `~/.codex/AGENTS.md` and `~/.agents/skills`
  - Claude Code: `~/.claude/CLAUDE.md` and `~/.claude/skills`
  - Projects: `.agents/skills` and `.claude/skills` (`registry/harness.json:26-46`; `lib/catalog.py:538`)
- Global sync (`lib/harness.py:100-232`):
  - stages each item and restores it if the swap fails;
  - keeps per-item receipts;
  - refuses unmanaged or modified destinations (`:154-161`).
- Project sync (`lib/harness.py:311-342,411-442`) updates only intact catalog-owned copies whose receipt hash matches, and verifies source and adapted hashes before any write (`lib/catalog.py:603-644`).
- `project add` writes only the manifest, with identity and byte checks (`lib/harness.py:482-574`).

---

## 3. Prioritized defects (none blocking)

### D1 (P2): the evaluation report turns an agent review into `accepted: true` and drops the reviewer kind
- **Trigger.** `eval review --kind agent --decision accepted`, then `eval report`.
- **Location.**
  - `lib/evaluation.py:400` sets `accepted`; the rows at `:401-405` carry no review kind or reviewer.
  - `stored_review_status` returns only the decision (`:365-378`).
  - The test asserts this behaviour (`tests/test_evaluation.py:76-77`).
  - Five of seven rubrics are titled **"# Human review"** (`evaluations/cases/{frontend…,engineering…,documentation…,integration…,research…}/rubric.md:1`).
- **Consequence.** The harness's only machine-readable outcome summary cannot tell agent review from human review. That contradicts:
  - `delivery-standards.md:182-184` ("Retain human and agent review as distinct kinds of evidence");
  - `ai-system-evaluation/references/harness-development.md:65-66` ("Report … the reviewer kind").

  The `limits` text at `:408-411` softens this, but downstream summaries will read `accepted: true` as acceptance. Every recorded review in the evidence is an agent review.
- **Smallest remedy.**
  1. Add `review_kind` and `reviewer` to each report row.
  2. Either reserve `accepted` for human review (or `not_required`) and add `agent_accepted`, or emit `accepted_by: human|agent|none`.
  3. Retitle the rubrics as "Review".
- **Check.** A unit test where an agent-only review yields `review_kind: "agent"` and no human-acceptance state, plus `review_counts` keyed by kind.

### D2 (P2): the default foundation duplicates 8 global skill names in every project
- **Trigger.** Run `project init` (or `project add`, which re-adds defaults) on a machine with the global install.
- **Location.**
  - Globals: `registry/harness.json:5-20`.
  - Foundation: `registry/catalog.json:14856-14871`.
  - Overlap: `work-planning`, `context-management`, `agent-coordination`, `research-and-synthesis`, `security-judgment`, `ci-maintenance`, `document-parsing`, `office-authoring`.
  - This contradicts the repository's own rule at `docs/runtime-integrations.md:18` ("avoid competing global copies with the same frontmatter name").
  - The only mitigation is guidance (`skills/skill-catalog/SKILL.md:20-22`: prefer the project copy). `docs/reviews/harness-upgrade.md:70-75` concedes duplicate names do not merge in Codex and that the initial skill list is bounded.
- **Consequence.**
  - Eight redundant discovery entries per project and target, which strains the stated efficiency requirement.
  - Version skew between live global links and pinned project copies. This is happening now: the modified `context-management`, `security-judgment` and `agent-coordination` are live globally, while existing projects hold the old copies.
  - The "prefer project copy" rule may not hold if a host resolves same-named skills by its own precedence. Claude Code's personal-vs-project precedence was not verified here and may favour the personal copy.
- **Smallest remedy.** Verify host behaviour first. If duplicates are listed or shadowed, keep the 14 globals, reduce the default foundation to the non-global skills (`architecture-review`, `debugging`, `test-design`, `release-operations`), and offer the other 8 as an explicit portability profile for devices or collaborators without the global install.
- **Check.** In a scratch foundation project, put a marker line in each copy of `work-planning`. Start fresh Codex desktop and Claude Code sessions, then record the number of listed entries and which file is actually loaded.

### D3 (P3): project readiness exit status depends on every client app; the default capture path is unproven on the primary device
- **Location.**
  - `lib/runtime.py:366` (`ready` = all of desktop, Codex CLI and Claude Code), `:590`, `:609-610`.
  - Linux desktop identity can never verify (`:317-322,354-355`).
  - Evidence: `project-readiness-2026-10-07.json:86`.
  - The readiness instructions at `skills/skill-catalog/SKILL.md:69-77` do not mention `--browser`.
- **Consequence.** `runtime doctor --project` exits 1 on Codex-only or Linux machines even when the project's tools are fine, so it is a weak gate. The documented default Chrome capture timed out on the user's Mac; only an explicit `--browser` to a Playwright headless shell worked.
- **Remedy.** When `--project` is given, base the exit code on `project.requested_checks_passed` and report client readiness separately. Mention `--browser` in the readiness paragraph.
- **Check.** A unit test with Claude Code absent and project tools available, expecting exit 0 in project mode; plus a default-discovery capture on the primary Mac.

### D4 (P3): the evaluation domain allowlist excludes several requested roles
- **Location.** `lib/evaluation.py:24-25`, enforced at `:71`.
- **Consequence.** Design and illustration, motion, QA, operations, marketing and planning cannot get fingerprinted development cases without a code change. Their evidence stays ad hoc (`docs/evidence`) or absent. Per the review criteria this is not a configuration bug in itself, but it is why those roles' claims must stay narrow.
- **Remedy.** Extend the domain list and add one discriminating case each. Examples: a QA journey with a persistence-after-reload defect, a release/rollback plan review, claim-preserving marketing copy, an SVG asset, an interrupted transition.
- **Check.** `eval list --domain <new>`, plus seeded broken-implementation rejections like the existing ones.

### D5 (P3): evidence labels and fingerprints
- **Location and issues.**
  - The field `catalog_sha256` in `review-evidence/live-archives.json:2` holds the hash recorded for `lib/catalog.py` (`agent-runtime-hardening-2026-10-08.json:38`), not for `registry/catalog.json` (`:39`).
  - `review-evidence/registration.json` has no input fingerprint, timestamp or catalog hash.
  - `docs/verification.md:77` ("AGENTS.md is 83 lines") sits in a historical table but conflicts with the current 91 lines.
- **Remedy.** Rename the field to `reader_sha256`, add `registry_sha256`, fingerprint the registration evidence, and annotate the historical line count.

### D6 (P3): authored skill descriptions differ between catalog metadata and installed frontmatter
- **Location.** `debugging` (`registry/catalog.json:374` vs `skills/debugging/SKILL.md:3`) and `marketing-writing` (`catalog.json:327` vs `SKILL.md:3`).
- **Consequence.** Catalog search and host discovery match on different text. For example, "intermittent" matches the host-loaded frontmatter but not catalog search.
- **Remedy and check.** A test asserting catalog description equals frontmatter for local and global-link entries, or generate one from the other.

### D7 (P3): the routing table is hand-maintained and untested
- **Location.** No test references `task-routing.md` or `delivery-standards.md` (searched `tests/`).
- **Current state.** All 18 routed IDs and invocation names are correct today (checked by hand).
- **Remedy and check.** Parse the backticked IDs and invocation names from `task-routing.md` and assert each exists, has project scope, matches its install name, and belongs to the profiles the table names.

---

## 4. Non-blocking suggestions and unverified behaviours

**Unverified behaviours (none established by me or by the repository evidence):**
- **Host discovery and selection.**
  - No evidence lists Codex desktop discovering `~/.agents/skills` in a fresh session. Codex system skills on this Mac live under `~/.codex/skills/.system` (`agent-runtime-hardening…json:255`).
  - The forward exercises were guided, with directed reads of repository and scratch paths (`motion-studio-2026-10-08.json:296-309`).
  - Automatic routing for arbitrary prompts is explicitly unclaimed (`harness-upgrade.md:141-146`).
- **Archive limits.** The new limits on downloaded source archives (`lib/catalog.py:26-31,204-294`) were re-checked live for **2 of about 26 archive-mode sources** (`review-evidence/live-archives.json`; the hashes do match `catalog.json:735,756,5670-5671`). Other sources fail closed if over budget; nothing is written, but their selections become uninstallable.
  - Check: run the payload-preparation step once for one installable entry per archive-mode source and record expanded bytes and header counts.
- **Native Windows.** Copy mode, Task Scheduler, process-tree cleanup and capture are all unverified (`docs/verification.md:87`).
- **The 441-test pass is primary-agent evidence.** I only confirmed by search that 441 `def test_` definitions exist across 24 files, which is consistent with `review-evidence/suite.log`.
- **Hashes.** The 12 foundation hashes in `review-evidence/registration.json` match the catalog strings (e.g. `catalog.json:395,427,455,11026,11074,11101,11358,11381`). Current-file-to-hash equality is enforced by `tests/test_catalog_data.py:49-58`, but I did not compute any hash.

**Suggestions:**
- **Vue/Nuxt depth.** Add a Vue/Nuxt reference to `frontend-engineering` (SSR payload, keyed `useFetch`/`useAsyncData`, hydration, composable reactivity), modelled on `async-state.md`. The user's reference project appears to be Nuxt.
- **DevOps depth.**
  - Add one worked release/rollback and infrastructure-plan review reference.
  - Correct and promote `ecc-docker-patterns`, or author a container workflow.
  - Optionally report docker, terraform and kubectl presence in readiness.
- **Optional Claude Code at install.** The complete installer hard-requires Claude Code CLI (`lib/install.py:26-27`; `lib/runtime.py:366`) even though Claude is a secondary client. Add an optional-client mode for Codex-only devices.
- **Live links skip review.** macOS link mode exposes uncommitted working-tree edits globally right away. The current diff is already live if the "30 unchanged live links" install is in place. When review gating matters, review in a separate worktree or use copy mode.
- **Foundation re-added on `add`.** `project add` re-inserts `project-foundation` even when a project deliberately removed it (`lib/harness.py:516-518`). This is documented, but it erodes "selective" setups.
- **Claude guidance not loaded on delegation.** Delegate guidance in `~/.claude/CLAUDE.md` is never loaded on the harness's own delegation path (`lib/claude_delegate.py:355-360`: `--safe-mode`, `--setting-sources ''`). This is documented at `setup/README.md:112`; just keep it in mind.
- **Nightly code execution.** Maintenance runs updated code from `origin/main` every night after checking the remote (`lib/maintenance.py:342`). That is residual trust placed in the user's own remote.
- **Incidental observation, not a test.** This review session exposes only Read/Glob/Grep and carries the instruction text from `lib/claude_delegate.py:41-51`. That is consistent with the runner's tool restriction; it does not test its deny rules.

---

## 5. Files inspected and checks not executed

**Read in full or in the relevant sections:**
- **Instructions and criteria:** `REVIEW-CRITERIA.md`, `README.md`, `instructions/AGENTS.md`, `instructions/CLAUDE.md`.
- **Routing:** `skills/skill-catalog/{SKILL.md, references/task-routing.md, references/delivery-standards.md, scripts/catalog.py, scripts/harness.py}`.
- **Registry:**
  - `registry/harness.json`;
  - `registry/catalog.json`: sources, profiles, authored entries, plus the routed, Docker, Terraform, Playwright, taste and emil entries.
- **Skills:**
  - interface-design (SKILL and 3 references), frontend-engineering (with async-state);
  - motion-design (with recipes), page-transitions (with navigation-craft), advanced-motion (with scene-mechanics);
  - svg-creation (with vector-craft and `audit_svg.py`), svg-animation (with svg-motion-craft);
  - test-design (with quality-assurance);
  - engineering-judgment (with both references), debugging, database-systems (with postgresql), data-migrations, release-operations, ci-maintenance;
  - marketing-writing (with editorial-preservation), conversion-review, experiment-design, customer-research;
  - research-and-synthesis, document-workflow (with technical-documentation), work-planning (with delivery-review), product-management, data-analysis;
  - agent-coordination (with claude-code), context-management (with repository-retrieval), security-judgment (with agent-tool-boundaries), ai-system-evaluation (with harness-development).
- **Code:** `ai.py`, `lib/{harness,catalog,install,runtime,evaluation,claude_delegate,bundle}.py`, `lib/maintenance.py` (searched only), `setup/{macos.sh, windows.ps1, README.md}`.
- **Evaluations:** `evaluations/suite.json`; the frontend case's `task.md`, `rubric.md` and `verify.py`; all rubric titles.
- **Tests:** `tests/test_catalog_discovery.py`, `tests/test_catalog_data.py`, part of `tests/test_evaluation.py`; test-definition counts across all test files.
- **Docs and evidence:**
  - `docs/{verification.md, quality-harness.md, runtime-integrations.md}` (start);
  - `docs/reviews/{agent-runtime-hardening, harness-upgrade, skill-routing, motion-studio}.md`;
  - `docs/evidence/{agent-runtime-hardening, motion-studio, project-readiness}*.json`, `docs/evidence/svg-studio-2026-10-08.json` (lines 130-277);
  - all four `review-evidence/` files.

**Not inspected:**
- the remaining skills and references: claim-verification, decision-to-delivery, grain-and-comparisons, mcp-integration and protocol-completion, the redis/sqlite/document-database references, search, desktop, document-parsing, office-authoring, architecture-review;
- `lib/settings.py`, `lib/schedule.py`, most tests, the other six evaluation cases;
- `docs/{source-review,catalog,project-targets,plugins,settings,scheduling}.md`, other review and evidence files, and `docs/examples`;
- `registry/source-index.json` and `registry/codex-settings.json`.

**Checks not executed (no Bash, network or rendering available):**
- the test suite and the registry renderer's `--check`;
- any hashing of files against catalog or evidence values (string comparisons only);
- any install, sync, doctor, export, evaluation or delegation command;
- browser rendering or viewing of the example artwork and motion studies;
- current external documentation, including Codex and Claude Code skill-precedence behaviour;
- any Windows or Linux behaviour.
