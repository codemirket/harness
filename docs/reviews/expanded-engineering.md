> Domain review evidence. Final IDs, status and counts are authoritative in [the catalog](../../registry/catalog.json); source-provider descriptions are not endorsements.

# Expanded engineering catalog review

Reviewed 2026-10-06. This extends the earlier pinned engineering review at `docs/reviews/engineering.md`; it does not invalidate that source inspection. The objective is now a broad **optional** registry across the lifecycle. An opinionated suite is a valid project choice when the user wants that workflow. It need not become global guidance, and registration alone does not authorize running installers, hooks, deployment or publishing.

## Deliverables and coverage

- `sources.json`: five pinned source records, checkout paths and license summaries.
- `inventory.json`: 128 actual skill entries, including test fixtures/internal maintenance marked as such. Descriptions are extracted from frontmatter; these are upstream discovery descriptions, not endorsements.
- `candidates.json`: 113 candidate records. **45 project-registerable skills** (30 Matt, 11 Superpowers, four Gstack methods) and 68 package/manual capabilities. Each project record includes actual frontmatter name, source path, companion IDs, dependencies, explicit integration text and coverage.
- Original revisions for Matt, Superpowers and Graphify were retained because all their skill bodies had already been read; selected supporting documents and scripts were reread/expanded here. No upstream executable, installer, hook, package script, plugin or browser was run.
- Gstack has a very large generated runtime suite. All skill frontmatter/path entries are inventoried. The router, host adapter, package/install contracts, lifecycle sections, licenses and four portable skills were inspected. **The entire runtime and every generated workflow body have not been audited.** Runtime entries are manual integrations, not cleared standalone payloads.
- All eight distributed context-mode skill bodies were read, plus plugin manifests, Codex integration docs, main routing references and selected start/postinstall/server security surfaces. Platform routing variants and upstream-maintainer ops skill are inventoried. This is **not** a full audit of its server/hooks/native dependencies.
- Selected standalone Gstack directories contain only their fully read `SKILL.md`; no missing scripts or shared runtime are implied.

## Lifecycle composition

| Need | Project-registerable choices | Optional full integration |
|---|---|---|
| Product discovery and deciding scope | Gstack office-hours/CEO review; Matt grilling, grill-with-docs, domain-modeling | Gstack office-hours + plan-ceo-review |
| Specifications, tickets and long-running plans | Matt to-spec, to-tickets, wayfinder, research; Superpowers writing-plans | Gstack spec/autoplan/plan-eng-review/plan-devex-review |
| Parallel execution and collaboration | Matt implement-spec, handoff, to-questionnaire; Superpowers dispatching-parallel-agents, subagent-driven-development | Gstack pair-agent/context-save/context-restore; context-mode session continuity |
| Implementation and regression prevention | Matt implement/tdd/diagnosing-bugs; Superpowers TDD/systematic-debugging/verification | Gstack investigate/qa/test-audit |
| Review and branch integration | Matt code-review; Superpowers requesting/receiving-code-review, finishing-a-development-branch | Gstack review/ship/landing-report |
| Architecture, onboarding and broad change impact | Matt codebase-design, domain-modeling, improve-codebase-architecture, setup-ts-deep-modules | Graphify graph/index/query workflows |
| Prototyping and human feedback | Matt prototype, grilling, wizard | Gstack design-shotgun/design-html/browse/QA |
| Deployment and operations | Matt wizard supports human-only provisioning; branch finishing supports integration choices | Gstack setup-deploy, land-and-deploy, canary, cso |
| Maintenance and learning | Matt triage/retro/teach/scaffold-exercises/writing-for-agents; Gstack portable retro | Gstack health/document-release/document-generate/learn; context-mode retrieval/stats |

Suggested profiles are additive capabilities rather than mandatory phases: `engineering-planning` (Matt setup, domain, grilling, research, spec, tickets); `engineering-orchestration` (Matt implement-spec and handoff); `superpowers-execution` (11 selected components with declared companions); `engineering-maintenance` (triage, session retro, review, documentation); `product-discovery` (portable Gstack office-hours and CEO review); `learning-workspace` (teach, questionnaire, exercises). Installing competing TDD/debugging methods is acceptable only if the project deliberately chooses which is authoritative for a given task. Do not auto-activate every related skill.

## garrytan/gstack

Pinned [c285d88b90d39116ccfa2b901f80ea0fce0b26eb](https://github.com/garrytan/gstack/tree/c285d88b90d39116ccfa2b901f80ea0fce0b26eb).

### Four portable project entries

- `openclaw/skills/gstack-openclaw-office-hours`: substantive demand/status-quo/user/wedge questions, premise challenge and alternatives, startup versus builder mode. **Integration required:** remove the effect of founder scoring, personal endorsement and YC promotional closing; keep respectful evidence-based uncertainty. Adapt `memory/` to agreed project docs.
- `openclaw/skills/gstack-openclaw-ceo-review`: scope choice plus architecture, errors, security, data, tests, operations, state, API, performance and UI review. Useful when a user explicitly wants a thorough plan challenge. Scale repeated questions/section ceremony to the actual task and prior answers.
- `openclaw/skills/gstack-openclaw-investigate`: root-cause/hypothesis loop with regression evidence. Three failed hypotheses do not prove architecture is wrong; scale tests and preserve autonomy.
- `openclaw/skills/gstack-openclaw-retro`: repository activity/history comparison. Its raw LOC, commit timestamps, inferred session hours, per-author ranking and highest-LOC “ship” are **not valid productivity or quality measures**. Integration text explicitly limits them to incomplete activity observations and selects highlights by actual impact. No mandatory Telegram format or personnel judgments.

These four have unique frontmatter names and MIT licensing from root `LICENSE`. Their exact compatibility notes are in `candidates.json`, ready to prepend as declared modifications without changing the reviewed upstream body.

### Full lifecycle suite

The [router](https://github.com/garrytan/gstack/blob/c285d88b90d39116ccfa2b901f80ea0fce0b26eb/SKILL.md) exposes discovery, architecture, DX/design reviews, autoplanning, specs, implementation investigation, QA, security, release, deployment, canary checks, documentation, retrospectives, context checkpoints and browser collaboration. It is a legitimate comprehensive alternative, not rejected for being opinionated.

It is **not a folder-copy package**. Generated skills refer to shared `bin`, `lib`, `sections`, browser/dist, runtime status, analytics and optional outside review. [Codex host metadata](https://github.com/garrytan/gstack/blob/c285d88b90d39116ccfa2b901f80ea0fce0b26eb/hosts/codex.ts) marks Codex experimental, uses advisory safety rather than enforced Claude hooks, rewrites paths/tools and suppresses unsupported delegations. Its outside-review boundary prompt forbids reading `.agents` skills; that is a subreview boundary, not a replacement for our global guidance.

[Setup](https://github.com/garrytan/gstack/blob/c285d88b90d39116ccfa2b901f80ea0fce0b26eb/setup) needs Git and Bun; Windows additionally needs Node and Bash-compatible setup, uses copies when links cannot work, and must refresh generated skills after updates. Browser integration prefers Aside on supported macOS with existing authenticated sessions, otherwise built Chromium. CSO has additional native build/runtime prerequisites. Cross-model review needs the other authenticated agent CLI. Do not promise Codex parity or tested Windows operation from source inspection alone.

The router runs start/end helpers, local analytics, operational learning and optional artifact sync, with runtime consent gates. Team mode includes updates and repository settings. Browser cookies, authenticated sessions, outside-model review and optional remote sync materially change access/egress; select them explicitly. A user choosing shipping must still distinguish PR creation from merging/deploying. Choose pinned installation and review updates; do not silently substitute the upstream auto-upgrade workflow for catalog pinning.

Root [LICENSE](https://github.com/garrytan/gstack/blob/c285d88b90d39116ccfa2b901f80ea0fce0b26eb/LICENSE) is MIT. [NOTICE.md](https://github.com/garrytan/gstack/blob/c285d88b90d39116ccfa2b901f80ea0fce0b26eb/NOTICE.md) lists derived Apache-2.0 design materials (impeccable and Google DESIGN.md related implementation). Full runtime distribution must retain notices plus `licenses/Apache-2.0.txt`; the four portable methods are not listed derived design files.

## mksglu/context-mode

Pinned [e5fcca6802d484db3f4ec04f6d2021938852b2a0](https://github.com/mksglu/context-mode/tree/e5fcca6802d484db3f4ec04f6d2021938852b2a0), package 1.0.169.

Strong optional package for repeated large tool output, local FTS5 indexing/search and session continuity. `ctx-index` and `ctx-search` express useful path-based capture and scoped batch retrieval; `ctx-stats`/`ctx-doctor` expose diagnostics. `ctx-purge` is destructive and `ctx-upgrade` installs and reconfigures, so they must not be incidental routing actions. `ctx-insight` is a hosted dashboard launcher, distinct from local retrieval.

[package.json](https://github.com/mksglu/context-mode/blob/e5fcca6802d484db3f4ec04f6d2021938852b2a0/package.json) requires **Node >=22.5.0**, ships `better-sqlite3` and a postinstall script, and specifies **Elastic-2.0**. [LICENSE](https://github.com/mksglu/context-mode/blob/e5fcca6802d484db3f4ec04f6d2021938852b2a0/LICENSE) restricts hosted/managed offerings, protects licensing functionality/notices and requires modification notices. Personal local use and a public marketplace service are different questions; retain the actual license with any redistributed material rather than labelling it MIT/open-source by assumption.

[Codex plugin manifest](https://github.com/mksglu/context-mode/blob/e5fcca6802d484db3f4ec04f6d2021938852b2a0/.codex-plugin/plugin.json) bundles MCP, skills and hooks. Upstream docs require host-version-dependent hook flags/trust. A working stats tool proves MCP reachability, **not** active/trusted hooks. Validate current host behavior during any future installation instead of blindly changing settings.

Important integration issues:

- The README fallback copies routing files over project/global `AGENTS.md`. **Do not run that copy command** against this shared guidance. Merge scoped project instructions or rely on supported hooks after review.
- [MCP config](https://github.com/mksglu/context-mode/blob/e5fcca6802d484db3f4ec04f6d2021938852b2a0/.codex-plugin/mcp.json) sets `default_tools_approval_mode: approve`. The [server](https://github.com/mksglu/context-mode/blob/e5fcca6802d484db3f4ec04f6d2021938852b2a0/src/server.ts) labels execution destructive/open-world and documents arbitrary code with network access. “Sandbox” is not evidence of an OS security boundary; keep host permissions in force.
- [start.mjs](https://github.com/mksglu/context-mode/blob/e5fcca6802d484db3f4ec04f6d2021938852b2a0/start.mjs) and [postinstall](https://github.com/mksglu/context-mode/blob/e5fcca6802d484db3f4ec04f6d2021938852b2a0/scripts/postinstall.mjs) perform cache/registry self-healing, dependency repair and Windows junction handling. Review target paths and backups before installing; this is more than passive Markdown.
- The main routing skill says essentially all read/query commands must use context-mode, while its anti-pattern reference recommends direct tools for small output. Resolve this explicitly to task-based routing; don't impose two contradictory global rules.
- Indexes and session history persist content; exclude secrets and sensitive files, choose project storage scope, and distinguish purge scope. A source claiming 98% context savings has not demonstrated that result on this user's workload.

Do not automatically install its individual skills. Keep one package integration entry with the eight skills as discoverable capabilities, plus the platform variants in inventory.

## Matt Pocock expansion

Pinned [4588b32ecab9ecc9fc8cc6b6c5e7d675b6004b0d](https://github.com/mattpocock/skills/tree/4588b32ecab9ecc9fc8cc6b6c5e7d675b6004b0d), MIT root license. Thirty project candidates are fully enumerated in JSON. This now includes the previously omitted lifecycle: tracker setup, spec synthesis, vertical slices, decision-map orchestration, concurrent integration, issue triage, prototypes, session retro, handoff/questionnaire, interactive learning, precommit checks and TypeScript boundaries.

Use exact upstream names to keep slash references and companion discovery working. `requires` records linked companion skills; setup config is still a separate **invocation**, not a side effect of registration. Preserve `agents/openai.yaml` and its explicit-invocation policies. Existing docs/glossary/tracker conventions win. Tracker writes and automatic commits in `implement`, `implement-spec`, `research`, prototypes and setup tools need task authorization; integration text makes that clear.

Notable caveats: architecture report templates load CDN Tailwind and Mermaid with loose security; escape untrusted labels or use safer local rendering. Wizard's Bash template writes bare `.env` values, so generated stages require quoting/format review and secret hygiene. Git guardrail regexes are incomplete defense in depth, not a security sandbox. `setup-ts-deep-modules` is explicitly in progress and assumes its root-file/public, subfolder/private convention; do not retrofit it blindly.

`ask-matt` remains inventory-only because our catalog router replaces its bundle-wide recommendations. `pr` remains inventory-only pending clarification of copied almost-verbatim Humanlayer material declared in `CREDITS.md`; the general root MIT license does not independently prove that third-party text's license chain. Remaining in-progress writing/loop/harness-specific handoff skills are inventoried and can be evaluated as separate workflows; no arbitrary numerical cap was imposed.

## Superpowers expansion

Pinned [8ca22dba9a94f28898bbce59f2537ff4d87c747d](https://github.com/obra/superpowers/tree/8ca22dba9a94f28898bbce59f2537ff4d87c747d), MIT. Eleven fully reviewed components now form an explicit optional suite: planning, worktree isolation, inline or subagent execution, parallel dispatch, TDD, systematic debugging, request/receive review, verification and branch finishing.

The choice of a full workflow is legitimate. The integration note is about host portability and authorization, not silently deleting the chosen workflow's rigor. Preserve model preferences and task scope. Treat upstream success anecdotes as anecdotes; test validity comes from observed behavior. Debugging examples that log environment values need redaction; npm-specific polluter helper must be adapted to the actual test runner and paths (it is not a general bisection or proof that all tests are clean).

`executing-plans/scripts/task-start` and `task-done` directly invoke sibling `subagent-driven-development/scripts/{task-brief,sdd-workspace}`; directory layout and executable modes must survive registration. Its review also uses sibling `requesting-code-review/code-reviewer.md`. Those dependencies are declared. The writing-plans entry remains useful standalone; execution references become a separately selected execution profile rather than inducing a cycle.

Four non-selected skills remain inventoried: `using-superpowers` is suite/bootstrap integration; `brainstorming` includes a Node/browser companion requiring a wider integration review; `diagnosing-superpowers` is session/log diagnosis with its own sensitive-data handling; `writing-skills` has a large evaluation/reference package. They are not rejected for verbosity, but are not newly marked fully cleared payloads without complete supporting review.

## Graphify expansion

Pinned [5c7b84792f453582676548185aaec3824d51dfe2](https://github.com/Graphify-Labs/graphify/tree/5c7b84792f453582676548185aaec3824d51dfe2). Prior canonical skill/platform/dependency review remains applicable. Promote it from “architecture curiosity” to a useful optional **codebase intelligence tool**: onboarding, map retrieval, graph queries, impact analysis, multi-file planning and context handoff across sessions. Capture indexed revision and configuration; stale or inferred edges are not proof of runtime behavior.

This stays a package/manual entry because Python installation, graph dependencies, provider configuration and global-guidance mutations cannot be supplied by copying `graphify/skill.md`. Preserve `LICENSE`, `LICENSE-MIT`, `NOTICE`. Avoid unattended `pip --upgrade`, `--break-system-packages`, provider substitution and automatic edits to shared global CLAUDE.md. Choose local-only or authorized provider processing according to data sensitivity.
