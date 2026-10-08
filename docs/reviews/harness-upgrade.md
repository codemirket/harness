# Harness review and focused upgrade

Reviewed 2026-10-08, starting from `ddfe784`. Codex desktop remains the primary
client; Claude Code receives portable project skills and bounded assignments.
The request was to challenge the existing harness across design, illustration,
backend, databases, operations, QA, marketing, product, analysis and research.

The main finding is a gap between available expertise and delivered outcomes.
The registry already has substantial specialist coverage. However, several ordinary
role queries surfaced only unreviewed entries, the task map omitted entire domains,
and product decisions and quantitative analysis lacked concise general workflows.
More runtime packages would not resolve those failures by themselves.

## Resource decisions

All twelve requested resources were reviewed through their accessible pages and
relevant underlying material. Access limits and untested claims are explicit below.
Article examples and vendor claims are evidence about those sources, not measured
gains for this repository. No upstream installer, hook or service was executed.

| Source and evidence | Applied decision and limit |
| --- | --- |
| [Anthropic: long-running applications](https://www.anthropic.com/engineering/harness-design-long-running-apps), 2026-03-24; frontend, full-stack and iteration sections | Adopt selective independent inspection of actual results and observable acceptance. The article later removes compulsory sprints/context resets; it does not support a permanent three-agent pipeline. Selected, costly demonstrations do not establish general reliability or this harness's quality. |
| [Microsoft Agent Harness](https://learn.microsoft.com/en-us/agent-framework/concepts/harness?pivots=programming-language-csharp), updated 2026-09-21; architecture/capability matrix plus [looping](https://learn.microsoft.com/en-us/agent-framework/agents/looping) and [background agents](https://learn.microsoft.com/en-us/agent-framework/agents/background-agents) | Keep model execution, approvals and child lifecycle owned by the actual host. These are SDK runtime mechanisms, some experimental, rather than portable skill semantics. Adopt bounded repair and distinct completion/blocked states; do not install a second runtime. |
| [Claude: harnessing intelligence](https://claude.com/resources/articles/harnessing-claudes-intelligence), 2026-04-02; tools, progressive skills and context persistence | Retain small entrypoints, conditional references and useful tool boundaries. Remove machinery when evidence shows it no longer helps. Provider-specific cache/context behavior is not configurable by this repository; no model or billing switch follows from the article. |
| [Addy Osmani: harness engineering](https://addyosmani.com/blog/agent-harness-engineering/), 2026-04-19; failure-derived scaffolding, feedback and hooks | Tie a repeated failure to a specific regression, fixture or corrected workflow. Reject unconditional stop hooks, lexical shell filters as security, and permanent rule accumulation. The article is expert synthesis, not a controlled comparison; its linked primary sources contain narrower conclusions. |
| [OpenAI: harness engineering](https://openai.com/index/harness-engineering/), 2026-02-11; application legibility, repository knowledge, constraints and limits | Keep a short map, expose actual UI/data/diagnostics, and enforce suitable project invariants through existing tests. Add outcome-focused review. Do not copy its six-layer architecture, permissive merge approach or background cleanup into unrelated projects. The authors explicitly warn that their end-to-end autonomy depends on their environment. |
| [ECC](https://ecc.tools/) and [pinned repository](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775); product-capability, product-lens, verification-loop, evaluation/learning/memory sections, Codex config and MIT license | Current HEAD already matches this harness's pin. Retain explicit product contracts; reject PMF inference from repository/pricing clues, forced scores and universal coverage thresholds. Some example pipelines mask producer exit status. Keep existing verified gates. Hook observation, extra memory runtime and default MCP commands add execution/data scope without demonstrated benefit here. |
| [Composio Awesome Codex Skills](https://github.com/composio-community/awesome-codex-skills/tree/0930e1373789d2eda449039f7ac154b33031de89); five relevant bodies, server helper and per-skill licenses | Planning, formula help and browser inspection overlap existing workflows. Notion and connect require real tools/OAuth/CLI/external effects. No root license was found; individual notices vary. The browser helper checks a port, not app readiness, and does not drain its piped output. No helper was executed. Defer integrations to a concrete missing service; no bulk import. |
| [Paperclip](https://paperclip.ing/) and [pinned repository](https://github.com/paperclipai/paperclip/tree/5717523b9ea7a2d76efbd6eb73414de9c06c6f96); README, heartbeat/budget docs and code, Codex/Claude local adapter docs, task lifecycle and MIT license | It is a Node/React control plane with database and CLI-agent lifecycle, not a Codex desktop configuration package. Adopt clear ownership, bounded work and durable next actions. Defer deployment until sustained organization-level operations need it. Static budget/cancellation code is not verified billing protection for this session. |
| [TypeUI design skills](https://www.typeui.sh/design-skills), [docs](https://www.typeui.sh/docs/creative/design-skills), [Codex guide](https://www.typeui.sh/docs/guides/codex), [MCP](https://www.typeui.sh/docs/overview/mcp-server), Atlas/Charm previews and [EULA](https://www.typeui.sh/license) | A coherent visual direction and component references can help. Keep project-specific use optional. Public previews do not expose complete account-gated payloads or prove production quality; no MCP runtime audit occurred. Do not redistribute its resource collection into this public registry under the observed EULA. |
| [Peter Yang: no-ai-slop](https://github.com/petergyang/no-ai-slop/tree/000650b156983f5159695b441477f4e63b25dc85); full skill, eval, metadata and MIT license | This is a prose editor, not a UI engine. Preserve meaning and voice while replacing empty claims with known specifics. Do not install its absolute word bans globally. Its self-check is not independent outcome evaluation; new local examples and wording are original. |
| [Tony Lee: twelve design skills](https://tonylee.im/en/blog/12-free-skills-escape-ai-slop-design/), 2026-03-19; complete article and selected linked primary bodies | Use focused complementary guidance with one visual owner. Do not install the whole list or adopt font/style bans. Checked Anthropic frontend-design, Design and Refine, Designer Skills Pack and Make Interfaces Feel Better: counts/descriptions drift, and not every advertised mechanism was substantiated. Companion/license review is incomplete for direct import. |
| [Hardik Pandya: stop-slop](https://github.com/hardikpandya/stop-slop/tree/8da1f030185bdfe8471220585162991eaeb970e9); full skill, phrase/structure/example references and MIT license | Reject the universal prose prohibitions and unvalidated score threshold. Some sample revisions change certainty or substance. Strengthen the existing writing lead with a factual-preservation audit instead of another competing editor. No upstream prose is copied. |

The design article's linked primary checks used Anthropic frontend-design at
`71cdddec623889d38af14b7a489670a03186f659`, Design and Refine at
`913a7ab2ac7e0f48eab77c93f2f5376150864b02`, and Make Interfaces Feel Better at
`35545ea1512ad59fa463e6b1f95ca9c052981fe6`. Those source inspections do not promote
their entire packages to installable status. The Composio browser recommendation
was checked against [Playwright's current readiness documentation](https://playwright.dev/docs/api/class-page#page-wait-for-load-state),
which favors application assertions over `networkidle` for tests.

## Changes at the responsible layer

| Observed gap | Focused change |
| --- | --- |
| `database administrator`, `DevOps`, `quality assurance`, `deep research` missed relevant reviewed guidance | Add task vocabulary to reviewed metadata and regression-test combined catalog search against the full indexed inventory. Product/analysis get equivalent query coverage. Search remains discovery, not a natural-language agent router. |
| Requested domains missing from the outcome map | Add explicit database, operations, backend, QA, product, quantitative analysis, marketing and raster-illustration routes, with corresponding completion standards and invocation boundaries. |
| Product discovery methods did not provide a general decision-to-delivery lead | Add `product-management` to the existing `product-discovery` profile. Research, experiments and architecture remain supports selected for actual needs. |
| Analytics offerings favored notebooks, queries and presentation | Add `data-analysis` to the existing `data-analytics` profile: establish population/grain/units, reconcile data, compare cohorts, reproduce calculations and bound conclusions. |
| Review could rely on a convincing implementation summary | Add a conditional `work-planning` delivery-review reference: inspect the actual journey/artifact, separate defects/preferences/unverified paths, repair from fresh evidence and stop repeated unproductive iteration. |
| Visual and editorial judgment lacked concrete correction examples | Extend existing interface and marketing leads with reference interpretation, matched rendered critique and preservation of claims/qualifiers/commitments. Keep project identity and factual accuracy ahead of novelty. |
| Test writing did not cover the whole QA acceptance path | Extend `test-design` with risk-based journey QA, durable effects, meaningful failure probes and device-evidence limits. |
| Five development fixtures left data/product and database recovery unexercised | Add two executable development cases with independent numeric/state expectations and separate qualitative review. They are public checks, not a held-out benchmark. |

The global skill set and foundation stay unchanged. Existing commands, manifest
schema and provider targets remain in use. No new production dependency, daemon,
hook, account connection, default MCP or orchestration framework is introduced.

## Configuration and host responsibilities

| Owner | Responsibility |
| --- | --- |
| This repository | Reviewed skills, task map, portable registration/provenance, installation and readiness diagnostics, development fixtures and delivery guidance. |
| Codex desktop / Claude Code | Actual model/tool loop, discovery, permissions, browser/delegation/session capabilities exposed in that runtime. A skill cannot implement missing host behavior. |
| Each project | Accepted business rules, design identity, architecture, start/test/build commands, realistic fixtures, observability and environment-specific deployment/recovery gates. |
| Task owner | Intended outcome, consequential unresolved decisions, production/external authority and final preference where subjective acceptance matters. |

[Current Codex skill documentation](https://learn.chatgpt.com/docs/build-skills)
describes progressive discovery and a bounded initial skill list; large installations
can have shortened descriptions or omitted entries. Duplicate names do not merge.
The selector now prefers the project's managed version when both it and a global
copy exist, and uses catalog search if initial discovery omits a needed capability.
Project copies preserve portability and pinning; broad installation still does not
prove that a runtime selected or applied the intended skill.

## Tools that justify their cost

Use a tool to close a named execution or evidence gap. More servers do not provide
more expertise automatically. First use the host's native capabilities or the
project's existing CLI/test infrastructure.

| Need | Useful capability and readiness proof |
| --- | --- |
| Interface, illustration or motion | A browser that can render and exercise the actual app; editable vector tools/code for SVG, image generation for bitmap assets. Inspect final sizes, states and embedding. Screenshot capture alone is insufficient. |
| Reference fidelity | Supplied source assets/design documents; optionally a project-selected Figma or TypeUI connection when access and the design workflow justify it. Verify a scoped read and rendering; do not connect accounts globally by default. |
| Database or DevOps | Existing engine/client, migrations, isolated data, logs/metrics/traces and release toolchain. Observe actual queries/effects and recovery within authorized scope. A generic database MCP is unnecessary when these already suffice. |
| Backend and QA | Existing test runner, realistic service fixtures and a suitable browser/API client. Verify the consumer result and relevant failure boundaries. |
| Research and product/marketing analysis | Search/source retrieval and reproducible SQL, scripts or spreadsheets. Connect private knowledge or analytics only for a concrete requested source; retain grain, attribution and access boundaries. |

Run `runtime doctor` on a new/changed environment, then the project's own readiness
and affected checks. Physical device behavior, engine semantics, authenticated
services and qualitative acceptance require their own observations. Reuse evidence
only while its relevant inputs remain unchanged.

## Apply and maintain

Global live links follow this checkout. Managed copies and existing projects need
deliberate synchronization. Retain project selections; add `product-management`
or `data-analysis` when relevant, then run project sync and doctor. Existing users
of their containing profiles receive the additions on deliberate sync. These new
skills are not forced into every project or into the foundation.

Use the project-agnostic handoff in [quality harness](../quality-harness.md).
When an actual task fails, identify whether the cause was discovery, missing craft,
runtime access, project context or verification. Add the smallest reproducing check
or correction in that owner. Recheck after material host/model changes; neither a
growing rulebook nor a shrinking one proves improvement without task evidence.

## Verification and limits

[Recorded evidence](../evidence/harness-upgrade-2026-10-08.json) retains relevant
source/artifact hashes, checks, reviewer basis and limits. The full suite passes
426 tests in 32.315 seconds. Generated tables, shell syntax and affected local
links pass. Eight changed skill entrypoints pass the official quick-validator
rules using existing Ruby/Psych in place of unavailable PyYAML; no package was
installed for that check.

Two isolated projects each install 14 selections for both providers: 28 copies
per project, all byte-exact. Doctor passes; repeat sync leaves all 56 copies
unchanged. Global doctor reports 30 unchanged links. An isolated native export
retains foundation plus the two new skills; portable-CLI tests exercise the new
evaluation cases after the original checkout is removed.

| Development exercise | Observed result |
| --- | --- |
| Existing frontend hierarchy | Four structural checks pass. A separate reviewer inspected matched before/after renders and independently reproduced all four final screenshots byte-for-byte. Browser journeys cover locale, edits, keyboard Save, themes, Draft preservation, labels/focus and narrow overflow. |
| Product/rollout analysis | Nine checks pass, including alternate datasets, conflicting deliveries and missing strata. An independent SQL calculation confirms pooled +17.5 percentage points versus common-device-mix -12.5 points. The reviewed recommendation preserves observational limits. |
| SQLite backfill | Thirteen checks pass for bounded commits, rollback, old/new writers, schema/data preservation and process restart; nine additional focused checks pass. The reviewed migration note distinguishes local behavior from production recovery/load claims. |
| Editorial preservation | A separate agent retains the exact opening/CTA, pilot qualifiers, approximate range and evaluation limits while removing unsupported claims. Primary source-to-copy review passes; no rendered marketing page or conversion result is claimed. |

Six intentionally broken or fixture-specific implementations are rejected by the
two new verifiers. These are author-side test instruments, separate from the fresh
agent trials. Final Git checks caught CRLF fixture line endings; the analysis
artifacts were replayed unchanged against normalized LF inputs with identical
parsed rows, then checked and reviewed again. This is not another model trial.
Independent harness review caught one misnamed editorial reference;
the corrected route was rechecked.

These are guided public development exercises, with agent reviews and an unavailable
exact model-version label. They do not measure a causal improvement over the old
harness, prove spontaneous routing for arbitrary prompts, or replace a real-project
visual acceptance test. The frontend guidance-only snapshot lacked a complete
catalog CLI checkout; the worker read the frozen files directly. Actual installed
and exported CLI behavior was checked separately.

No new Claude model task, native Windows run, physical device, Safari/Firefox,
screen-reader, real customer experiment or production migration was exercised.
The new cases and stronger review paths make those remaining claims testable when
their actual projects and environments are in scope. Existing Trixpo sources were
not changed during this harness upgrade.
