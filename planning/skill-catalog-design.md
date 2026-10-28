# Mirket skill catalog — proposed Codex-native design

**Planning revision, 2026-10-25.** This replaces the earlier one-global/27-project draft. It records the authoring and selection specification behind the [consolidated plan](implementation-plan.md). The resulting nine global and 52 project `SKILL.md` files are authored in the current source tree; disposable macOS ARM64 Codex-native installation and selected skill discovery passed. Matched-baseline model outcome evaluation remains open. All thirteen previously accepted subject outcomes remain covered, with clearer boundaries.

**Accepted follow-up, 2026-10-27:** eight project skills now record reviewed
method adaptations, with three bounded on-demand references included in the
catalog, release manifest and native skill receipts. The nine/global and
58/project split was unchanged in that pass. See the [source mapping](reviews/2026-10-27/source-design-adoption.md)
and [implementation/evidence](../docs/design-adoption-20261027.md). General
quality uplift and catalog-wide acceptance remain unqualified.

## First-release retention decision — 2026-10-28

The user authorized dropping skills that do not contribute quality or
performance. v0.0.1 retains nine global and 52 opt-in project skills across seven
profiles. The six tested product/marketing skills and `content-growth` profile
are retired: `customer-research`, `product-strategy`, `positioning`,
`marketing-copy`, `conversion`, and `experiments-launch`. Two valid enabled
bundle trials timed out while both no-skill baselines produced independently
correct artifacts. This is a bundle-level release decision, not attribution of
harm to each member. Codex still owns those ordinary product workflows.
`jobs-to-be-done`, `prototype-test` and `content-search` remain direct selections.

Retain a skill for a concrete routing contract, non-obvious quality safeguard,
or measured useful-output contribution that justifies context/maintenance cost.
Do not protect a fixed catalog size or require a skill for work Codex already
handles well. Preserve negative evidence; add or restore guidance only for a
demonstrated gap. A small tied comparison is not proof of universal uselessness.
The [evaluation](../docs/supplemental-model-evaluation-20261028.md) remains
unchanged historical evidence; 305 retained public development cases are not
relabeled as executed acceptance.

## Review findings

I reviewed the 58 unique retained [reference entries](references.md), the original harness's 54 specialist skill names as a gap inventory, pinned skill-file trees and decision-relevant bodies from representative A–D sources, the [independent capability review](reviews/2026-10-24/capability.md), and official [Codex skills](https://learn.chatgpt.com/docs/build-skills) and [AGENTS.md](https://learn.chatgpt.com/docs/agent-configuration/agents-md) documentation. The original inventory checks coverage; it is not a template to copy. Upstream tree counts include host copies, fixtures and orchestration variants, so they are not expertise counts.

The useful split is by **decision and deliverable**, not by upstream repository. [Superpowers](https://github.com/obra/superpowers/tree/8ca22dba9a94f28898bbce59f2537ff4d87c747d/skills) and [Matt's engineering skills](https://github.com/mattpocock/skills/tree/4588b32ecab9ecc9fc8cc6b6c5e7d675b6004b0d/skills/engineering) contribute planning, diagnosis, module design and evidence methods; their full agent loops and commit rituals do not transfer. [Hallmark](https://github.com/Nutlope/hallmark/blob/13ac0ec7e148655948100b6396439e481361d690/skills/hallmark/SKILL.md) and [Taste](https://github.com/Leonxlnx/taste-skill/blob/f624f37a8c650b9cc550253104925efe4c8de246/skills/taste-skill/SKILL.md) address art direction, while [LottieFiles motion design](https://github.com/LottieFiles/motion-design-skill/blob/f9a8a041b85185ee4881b3471d3415e939aac772/skills/motion-design/SKILL.md) addresses choreography and timing. These should not collapse into one vague design skill. The selected [marketing](https://github.com/coreyhaines31/marketingskills/tree/dda3841f0b294e01e93b1541486beefbfab0915e/skills) and [Wondel](https://github.com/wondelai/skills/tree/c172996495bed0fcd26896a9416b2093fd7073f0) corpora cover different research, strategy, messaging and conversion decisions, but their every upstream skill need not become a Mirket package.

The B sources are mainly tool-design comparisons: [Crawl4AI](https://github.com/unclecode/crawl4ai/blob/30165f1ee1fb61f6cce1477feed2acc5e2181e25/docs/md_v2/core/quickstart.md) distinguishes crawling and extraction; [Serena](https://github.com/oraios/serena/tree/abb0a0f01b94bc8050086f0fb860077d17730324) motivates symbol-aware navigation; [Stagehand](https://github.com/browserbase/stagehand/tree/0f93c295deabce3cd7b14cdbdeacb1771f93378d) and [Skyvern](https://github.com/Skyvern-AI/skyvern/tree/eccc1c60b225105c3281cf2cc5e76b4170099fbb) illustrate browser observation and action. Mirket keeps bounded JS/TS navigation; Codex and project tools own general browsing and SQL navigation. Private Chrome is reserved for video rendering. LangChain/LangGraph and promptfoo inform AI design/evaluation, not a second agent runtime. The G design systems are examples of design reasoning, not brands, fonts, assets or code to redistribute.

This reviews catalog shape and selected decision-relevant portions; it is **not** an audit of every upstream file or license. Exact source section, revision, terms and adaptation rights must be checked during authoring. The restricted [Anthropic DOCX material](https://github.com/anthropics/skills/blob/683bc88e56f3e09ba94f7055977f3d3aa499f202/skills/docx/LICENSE.txt) stays link-only; [Trail of Bits' license](https://github.com/trailofbits/skills/blob/196a9073ab10d35bacf111f945e02c00b2c6883a/LICENSE) needs separate treatment for adapted prose. No upstream text, executable, dependency, asset or account is adopted by this proposal.

## Installation and authority

Mirket installs **nine** concise, general-purpose native skills for the Codex user and a short managed global AGENTS.md paragraph. The paragraph states the durable rule: for qualified Mirket-owned memory, JS/TS navigation, design and video operations, use Mirket first; use Codex for general browsing and Codex/project tools for SQL; diagnose or report an unavailable operation. Explicit user/project instructions and Codex permissions remain authoritative. Codex owns models, native subagents, shell, editing, Git and host document tools. Mirket does not spawn or route agents and cannot intercept every native Codex spawn.

Coordination guidance is strong and testable: useful independent deliverables, disjoint ownership, task-appropriate expertise and lead-owned integration. Creative direction, design judgment and motion are not assigned to a lower-capability or lower-reasoning child merely for speed. Respect an explicit user model/effort/provider choice. A small global skill description should not trigger on every trivial task.

Global skills live in the Codex user's native skill location; project skills become ordinary project source under `.agents/skills/<id>/` **only after opt-in**. The release manifest records ownership and file hashes; names alone do not establish ownership. The global skill supplies the general method; a project specialist owns the distinct domain decision. An explicit user skill choice beats inference. A file extension alone does not trigger a specialist. Native [Codex skill discovery](https://learn.chatgpt.com/docs/build-skills) makes short descriptions and a small installed subset important.

## Nine global skills

| ID | Trigger and required result | Boundary / selected basis |
| --- | --- | --- |
| runtime-operations | Select, invoke or recover a qualified Mirket operation; establish project scope, readiness and effect identity. | Mirket-first route, not host permission authority; local contracts, agent-skills, Codex docs. |
| project-memory | Retained facts, corrections, forgetting or handoff affect the task; return scoped source-linked context and stale-state checks. | No passive transcript harvesting; qmd-methods, matt, local memory contract. |
| evidence-research | A consequential claim/comparison needs evidence; preserve source identity, date, disagreement and answerability. | Retrieved text is data; openai, anthropic, google, qmd-methods. |
| task-planning | Dependent work or uncertain scope needs sequence; define owner, gates, evidence and stop condition. | No plan ritual for trivial edits; superpowers, matt, ponytail, codex-practices, adhd. |
| agent-coordination-strategy | Independent Codex agents materially help; assign disjoint briefs and suitable capability, then integrate evidence. | Codex alone spawns/owns agents; superpowers, gstack, archify. |
| code-change | Substantive code behavior changes; find owner/consumers and smallest complete change. | Host/project toolchains edit and build; matt, superpowers, ecc, ponytail, karpathy. |
| root-cause-debugging | An unexplained failure needs reproduction, root-cause trace and discriminating hypothesis. | No symptom patch loop; superpowers, matt, gstack. |
| critical-review | A change/plan/artifact needs independent defect critique with current evidence and severity. | Advisory evidence, not self-certified approval or auto-commit; gstack, archify, sentry-methods, trail-of-bits. |
| outcome-verification | Completion is claimed; inspect actual saved behavior/artifact/effect and a convincing failure case. | Receipt/hash/verdict alone is insufficient; superpowers, agent-skills, trail-of-bits, wcag. |

## Fifty-eight opt-in project skills

These are **catalog options**, not 52 entries loaded into every project. Every entry has a distinct trigger and observable output. A Mirket route requires a qualified private capability; a host route uses Codex/project tools. A method skill cannot advertise an unavailable operation as ready. Framework- and dialect-specific detail belongs in on-demand references rather than one top-level skill per upstream library.

### Engineering and delivery — 13

| ID | Use when; expected output | Route; selected basis |
| --- | --- | --- |
| architecture | Topology or module boundaries change; ownership map, alternatives and failure tradeoffs. | Host; matt, archify, gstack, ponytail. |
| api-integration | HTTP/MCP/tool contracts change; typed input/result/error, authority and consumer probes. | Host; ecc, openai, google, ponytail. |
| frontend-engineering | Component/state behavior changes; accessible states and inspected browser result. | Host + Codex browser; matt, ecc, selected C/G. |
| backend-engineering | Service/job/concurrency changes; state ownership and failure-path evidence. | Host; ecc, matt, superpowers. |
| database-migrations | Schema/storage migration affects data; invariants, forward/recovery tests and protected original. | Host; matt, ecc, trail-of-bits. |
| test-design | Test adequacy is uncertain; failure case, invariant/property or differential comparator. | Host; superpowers, trail-of-bits, sentry-methods. |
| performance | A measured bottleneck or budget regression exists; controlled baseline/profile/result. | Host; gstack, ecc, codex-practices. |
| ci-release | Build, CI, packaging or update fails; exact revision, reproduction and recovery evidence. | Host; gstack, ecc, archify. |
| infrastructure | Deployment or environment configuration changes; ownership, drift, rollout and rollback evidence. | Host; ecc, gstack, archify. |
| native-desktop | A macOS/Windows desktop integration changes; permissions, lifecycle, native build and installed GUI proof. | Host platform/project toolchains; ecc, fluent, apple-liquid-glass, official SDK docs. |
| dependency-admission | A new executable/library/model is proposed; closure, rights, build effects and gate. | Host; ecc, sentry-methods, local admission contract. |
| js-ts-navigation | JS/TS/JSX/TSX definitions/references need revision-bound navigation; spans or explicit limits. | Qualified Mirket navigation; serena comparison. |
| sql-navigation | SQL references need dialect/schema context; verified source locations and explicit missing-schema limits. | Codex/project SQL source tools; serena and local SQL contract. |

### Browser and web evidence — 3

| ID | Use when; expected output | Route; selected basis |
| --- | --- | --- |
| browser-research | Public/local pages must be inspected and cited; bounded page evidence and extraction limits. | Codex browser; crawl4ai, stagehand. |
| browser-automation | Authorized multi-step site interaction; scoped observation/action/verification and uncertain-effect reconciliation. | Codex browser; stagehand, skyvern, cua, ponytail. No imported autonomous loop. |
| browser-qa | Live web journey needs testing; stateful interaction, responsive capture and failure evidence. | Codex browser; gstack, impeccable, stagehand, wcag. |

### Design and visual artifacts — 12

| ID | Use when; expected output | Route; selected basis |
| --- | --- | --- |
| art-direction | Visual concept needs a distinct direction; brief, references, composition and inspected specimen. | Host/design tools; hallmark, taste, owl-concepts, mengto-inspiration, better-design. |
| interface-composition | Screens/components are produced; hierarchy, real content/states and responsive proof. | Host + Codex browser/design; hallmark, taste, emil, react-bits, canvas-ui when relevant, G systems. |
| design-critique | Existing UI needs diagnosis/polish; ranked observed defects and before/after evidence. | Codex browser when live; impeccable, design-anatomy, gstack, owl-concepts. |
| design-systems | Repeated UI language needs tokens/components; compiled tokens consumed by source. | Qualified Mirket design; hallmark, impeccable, reshaped, geist-code, atlassian, fluent. |
| typography | Type hierarchy or reading surface matters; specimen and legibility check. | Host/design; geist-code, guardian, govuk, vanilla-publishing, reshaped. No font redistribution by reference. |
| accessibility | Interaction/visual access needs review; criterion-linked keyboard/focus/label/reflow/motion evidence. | Host + Codex browser; wcag, govuk, spectrum, fluent. |
| openpencil | Editable design source requested; created/reopened/changed source and checked export. | Qualified Mirket OpenPencil file workflow; C visual references. |
| vector-art | SVG/icons/sprites delivered; editable source, optimized preview and export fidelity. | Qualified Mirket SVG/SVGO/resvg; diagram-design, react-bits, G systems. |
| raster-art | Crop/composite/web-image export needed; source-linked size/color/alpha checks. | Qualified Mirket Sharp; taste, hallmark, better-design. |
| diagrams | Architecture/process/data relationships need a diagram; source and semantic/layout review. | Qualified Mirket D2; archify, diagram-design, G systems. |
| data-visualization | Data plotted visually; chart encoding, units, uncertainty and source-data audit. | Host/design; statistical-analysis, guardian, spectrum, diagram-design. |
| localization | UI is multilingual/bidirectional; message integrity, expansion and layout proof. | Host + Codex browser; spectrum, govuk, fluent, wcag. |

### Motion and video — 5

| ID | Use when; expected output | Route; selected basis |
| --- | --- | --- |
| motion-direction | Motion language/choreography needed; purpose, timing, interruption and reduced-motion spec. | Host/design; emil, kyle-motion, lottiefiles-motion, kingy-motion. |
| ui-motion | Interactive animation implemented; lifecycle-safe CSS/WAAPI/Anime/Motion/GSAP choice and playback proof. | Host project code + Codex browser; anime, motion, gsap-official, animate.css, motion-primitives, canvas-ui when relevant. |
| scroll-motion | Scroll scene/page transition requested; trigger/state map and responsive/interruption checks. | Host project code + Codex browser; scroll-craft, gsap-official, emil, kingy-motion. |
| svg-lottie | Vector/Lottie animation asset; editable source, tested feature subset and preview parity. | Host + qualified vector/video route; lottiefiles-motion, diagram-design, motion-primitives. |
| video-production | Exported video requested; editable HyperFrames source, private Chromium/FFmpeg render and decoded-frame/audio/caption QA. | Qualified Mirket video; pinned HyperFrames authoring and kingy-motion as comparisons. |

### Product, discovery and growth — 3

| ID | Use when; expected output | Route; selected basis |
| --- | --- | --- |
| jobs-to-be-done | Product problem needs customer-progress framing; job/forces and evidence-linked alternatives. | Host; wondel, marketing. |
| prototype-test | Idea/flow needs low-cost validation; prototype, task, observation and decision. | Host/design tools and Codex browser as available; wondel, owl-concepts, hallmark. |
| content-search | Content strategy/organic discovery; intent, information architecture and verifiable page change. | Host + Codex browser when inspecting; marketing, wondel, govuk, guardian. |

### Data and AI systems — 8

| ID | Use when; expected output | Route; selected basis |
| --- | --- | --- |
| data-analysis | Dataset cleaned/joined/summarized; grain, missingness, units and reproducible result. | Host; statistical-analysis and local data contracts. |
| statistical-inference | Effect/comparison interpreted; design, dependence, uncertainty and multiplicity check. | Host; statistical-analysis, trail-of-bits. |
| sql-analysis | Query answers a data question; dialect/schema, row grain, null/join and result verification. | Host DB tools; statistical-analysis. SQL source navigation uses Codex/project tools. |
| data-pipelines | Import/ETL/replay affects data; lineage, idempotency and duplicate/late-event tests. | Host; ecc, matt, trail-of-bits. |
| financial-models | Revenue, cost or pricing model informs a decision; assumptions, scenarios, units and reconciled saved workbook. | Codex host spreadsheet tools; wondel, marketing, statistical-analysis. |
| ai-tool-design | LLM feature exposes prompts/tools/actions; bounded schema, error context, authority and failure probes. | Host; openai, anthropic, google, langchain/langgraph comparisons. |
| ai-evaluation | Prompt/agent/model behavior compared; fixed tasks, matched conditions, independent outcomes and variability. | Host; promptfoo, agent-skills, openai, anthropic, google. |
| retrieval-evaluation | Memory/RAG quality claimed; relevance, coverage, stale facts and downstream answer test. | Host + qualified Mirket memory when relevant; promptfoo, qmd-methods, langchain/langgraph. |

### Documents and communication — 4

| ID | Use when; expected output | Route; selected basis |
| --- | --- | --- |
| technical-docs | API/architecture/user documentation changes; verified commands, links and reader task. | Host; openai, google, wondel technical-documentation. |
| document-artifacts | DOCX/PDF or mixed paged work; source preservation, render/inspect and saved-file reopen. | Codex host tools and format documentation; Anthropic DOCX link-only. |
| workbook-authoring | Workbook formulas/tables/charts; recalculation and saved-file evidence. | Codex host tools and format documentation; no Office runtime. |
| slide-deck-authoring | Editable deck authored; slide hierarchy, source consistency and rendered-page review. | Codex host tools and format documentation; no Office runtime. |

### Trust and operations — 4

| ID | Use when; expected output | Route; selected basis |
| --- | --- | --- |
| threat-model | New trust boundary/sensitive workflow; assets, attacker capability, misuse and mitigations. | Host; openai threat-model, ecc, trail-of-bits. |
| security-review | Changed code/dependency/skill may cross authority/data boundary; contextual finding and negative test. | Host; sentry-methods, trail-of-bits, ecc, promptfoo if AI-specific. |
| privacy-data | Retention, consent, credentials or deletion affected; data flow and verified lifecycle. | Host; openai, google, local memory/forgetting contract. |
| reliability-incident | Operational failure/recovery/resilience; state transition, replay/rollback and observed recovery. | Host; gstack, archify, ecc, matt. |

## Reference coverage and reuse boundary

The matrix records influence, **not text adoption**. It covers all retained families and keeps runtime decisions separate from source selection. Detailed section-level records belong in skills/PROVENANCE.md before any adapted body ships.

| Group | Retained IDs | Catalog use and exclusions |
| --- | --- | --- |
| A — engineering/agents | superpowers, matt, ecc, openai, anthropic, google, ponytail, adhd, gstack, archify | Global methods, engineering, architecture, review and research. No parallel agent framework, universal activation, forced commit or provider switch. ADHD is communication material, not diagnosis. |
| B — tools/AI | crawl4ai, serena, cua, stagehand, skyvern, ponytail, langchain, langgraph, promptfoo | Browser/navigation/AI concepts. No extra crawler, desktop CUA backend, framework loop, hosted browser or evaluation service. Ponytail counts once overall. |
| C — visual/motion | hallmark, taste, emil, impeccable, gsap-official, diagram-design, canvas-ui, kyle-motion, lottiefiles-motion, owl-concepts, mengto-inspiration, scroll-craft, motion-primitives, better-design, design-anatomy, anime, motion | Art direction, critique, systems, interaction, motion and vector/diagram methods. Some are libraries/examples, not skills; none installs by reference selection. |
| D — product/growth | marketing, wondel | Research, jobs, strategy, positioning, conversion, experiments and launch. No copied book text, paid campaign, outreach or account. |
| E — design examples | animate.css, react-bits, remotion | Conditional CSS/component/video examples. HyperFrames is the selected video runtime; Remotion is not another renderer. |
| F — additional | karpathy, codex-practices, kingy-motion | Coding preference, Codex workflow and video composition. “Karpathy” names a third-party repository, not an authorship claim. |
| G — visual systems | geist-code, atlassian, fluent, reshaped, apple-liquid-glass, govuk, vanilla-publishing, guardian, spectrum | Typography, layout, state, platform material, accessibility and bidirectionality examples. No brand imitation or font redistribution. |
| H — methods | agent-skills, trail-of-bits, qmd-methods, sentry-methods, statistical-analysis, wcag | Skill evaluation, test oracles, source expansion, security review, inference and access checks. Rights vary; no upstream helper/scientific runtime imported. |

## Project selection and synchronization

The catalog is versioned release data. Each entry requires an ID, short description, positive/negative trigger, deliverable/evidence contract, qualified-capability requirement, provenance/rights record, source hash and evaluation IDs. Native skill bodies contain no upstream installer, hidden bootstrap or provider credential flow.

Mirket proposes a **small selection** from explicit project purpose, declared frameworks/languages, package manifests and observed work. The initial profile recipes below are editable starting points; conditional entries require matching work, not merely a file extension. A profile is a reviewable set of IDs, not a dependency installer or claim that every tool is ready. Prefer roughly six to twelve specialists in an ordinary project; warn when a larger explicit selection crowds Codex discovery.

| Profile | Initial skills to propose | Add only when the work calls for it |
| --- | --- | --- |
| web-product | frontend-engineering, interface-composition, accessibility, browser-qa, test-design, design-systems | art-direction, ui-motion, localization, content-search |
| design-studio | art-direction, design-critique, design-systems, openpencil, vector-art, raster-art | diagrams, video-production, svg-lottie, typography |
| browser-workflows | browser-research, browser-automation, browser-qa | security-review |
| backend-service | backend-engineering, api-integration, test-design, database-migrations, threat-model, reliability-incident | performance, sql-navigation, infrastructure, ci-release |
| library-cli | architecture, api-integration, test-design, ci-release, dependency-admission | performance, js-ts-navigation, native-desktop |
| data-ai | data-analysis, statistical-inference, ai-tool-design, ai-evaluation, retrieval-evaluation | sql-analysis, sql-navigation, data-pipelines |
| documents | technical-docs, document-artifacts, workbook-authoring, slide-deck-authoring | data-visualization, localization, financial-models |

Profile values are exact, unbranded catalog IDs. Web extraction guidance remains an on-demand reference within browser-research. If several installed skills match one task, Codex selects one deliverable lead and only the specialists needed for genuinely separate boundaries; installation is not an instruction to stack every skill. Explicit inclusion/exclusion wins over a profile recipe. Mirket warns about a selected Mirket tool skill whose private capability has not passed the current platform gate; host browser and SQL skills check their actual tool availability at use.

The project owner first sees an exact skill plan/diff and opts in through .mirket/project.json with profiles/IDs and exclusions. Initial resolution becomes a pinned selection. Project selection and update synchronize **that approved selection** immediately; an explicit `mirket skills project-sync --project-root PATH` synchronizes it later. `mirket setup` and `mirket update` also synchronize one selected project when passed `--project-root PATH`, after preflighting its pinned selection and capability requirements. Without that option they manage only the global skill surface; Mirket does not scan or silently mutate other projects. New stack evidence suggests a future refresh; it does not silently churn committed skills. Exact IDs can be selected without a profile.

Sync validates all destinations, stages atomically, refuses edited managed-copy drift and rolls back partial failure. Disabling removes only an unchanged managed copy. Passive status, memory, source inspection or a new task cannot mutate a project. The project may commit selected skill packages under `.agents/skills/`. Setup and sync refuse collisions with unmanaged skills in either native scope and never silently suffix or rename an ID. No unrelated skill, AGENTS.md content, client setting or current user installation is overwritten. A skill requiring an unqualified Mirket browser/design/video/navigation route is marked unavailable in the plan rather than installed as if it worked. Host-owned method skills can be selected without a private runtime.

## Authoring and acceptance

Author one Mirket-native body per ID, using local contracts as authority and exact external portions as evidence. Before adaptation record pinned URL/path/heading, retained technique, terms, classification (adaptable, link-only, comparison-only, rejected), removed host/runtime assumptions and an observable acceptance case in skills/PROVENANCE.md. Do not copy restricted Anthropic DOCX text, Trail of Bits prose without compatible treatment, branded assets/fonts, first-use launchers, npm/pip bootstrap, provider keys or upstream agent orchestration. Framework examples are conditional on actual project dependencies.

Every entry has discriminating frontmatter, a concise decision method, output/evidence rule and stop condition. The 69 authored files replace the thirteen stale guide drafts and their old guide-read references. Engineering, Verification, Research, Interface, Motion, Product, Documents, Memory, Security, Reliability, Data, AI evaluation and Planning/coordination remain represented across the new boundaries without repeating global text in specialists. Native activation and model-outcome gates are tracked in the [evaluation contract](../docs/m4-guide-evaluation.md).

First validate frontmatter, IDs, links, hashes, rights, capability claims and profile resolution for all 61 skills. Keep two positive, two negative and one mixed authored development case per entry; run representative activation and nontrigger checks across the full catalog. Prioritize matched held-out, no-skill comparisons in clean Codex Desktop contexts for skills governing external effects, security, memory, coordination, and design/video or analytical quality. Independently inspect their saved outputs, retain failures and repeat variable cases. Expand deeper comparison when broad screening finds a defect. Measure unnecessary calls, description collisions and loaded context, not merely skill appearance. Mac activation is the first-release gate; Windows activation is a separate gate for its planned target.
