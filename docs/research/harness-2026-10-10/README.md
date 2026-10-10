# Harness redesign and source review — 10 October 2026

The harness now composes focused expertise around an observable outcome. Its core
remains portable, inspectable Python with no added runtime dependencies. The host
executes models and tools; projects own their architecture and release rules.
Nine new authored specialists fill financial, market, venture, translation,
spreadsheet, graphic, mobile, IT and technical-leadership gaps. Thirty-three
outcome contracts cover all 24 requested roles, with explicit aliases,
prerequisites, acceptance evidence and failure probes.

## Review scope and method

The request contains **40 unique repositories and two documentation services**;
ECC appeared in both lists and was reviewed once. Each repository was inventoried
at a fresh immutable commit. Reviewers read the relevant instruction bodies,
companions, manifests, implementation paths and licenses, following consequential
claims into source. Large collections were sampled by the affected capability and
runtime boundary; this is not a claim to have read every file or audited every
dependency. Retrieval did not run upstream applications, hooks or installers.

Four retained chapters contain findings, risks and exact file links. Their JSON
companions record every requested URL, observed revision, inspected paths,
decision and harness application:

- [Harnesses, SDKs and application runtimes](harness.md) — 12 repositories;
  [machine-readable evidence](harness.json).
- [Design, illustration and motion](design.md) — 9 repositories;
  [machine-readable evidence](design.json).
- [Engineering and agent workflows](workflows.md) — 10 repositories, including ECC;
  [machine-readable evidence](workflows.json).
- [Business, planning, providers and documentation services](business-providers.md)
  — 9 repositories and 2 services; [machine-readable evidence](business-providers.json).

A fresh research revision is not an automatically approved installer update.
`registry/catalog.json` retains the explicitly reviewed payload pins and
adaptations used for installation. These reports distinguish the new observations
from those older installable bytes. Third-party material retains its license;
new workflows are original synthesis, not copied SDKs or restricted skill bodies.

## Decisions that changed the implementation

| Finding | Resulting design |
| --- | --- |
| SDK sessions, task notes, durable execution and idempotent effects solve different problems. | Keep one host runtime. Shared guidance separates retrievable context, resumable work and effect reconciliation; no background memory or second agent loop. |
| Broad catalogs and persona prompts obscure the actual workflow and its costs. | `capabilities plan ROLE --with ROLE` selects explicit, deduplicated leads, validates their payloads and exposes paths, entrypoint bytes and a source fingerprint. Optional support stays unloaded. |
| Useful expertise is missing in financial, market, language and platform boundaries. | Nine original specialists provide conditional worked examples and acceptance criteria. `project init/add --capability` persists concrete skill IDs through the existing verified installer. |
| Several packages bundle executable hooks, output capture, binary downloads, model calls or separate services. | Keep those integrations manual. Prefer available host tools; review effects, dependencies, licenses and client compatibility before adopting a runtime. |
| Taste's previously pinned redesign body prescribed fabricated metrics and randomized dates despite its adaptation note. | Replace those exact source clauses with truth-preserving instructions, require occurrence counts and update the adapted payload hash. Preserve the original source hash and provenance. |
| UI guidance often imposes universal aesthetics, stack choices or state checklists. | Add brief-to-proof craft guidance and a medium-aware graphic-design lead; derive composition and state coverage from the actual brief and consumer. |
| Startup finance and market examples contain sign, unit and denominator defects. | Require cash/profit reconciliation, coherent scenarios, compatible market units and channel/capacity constraints. Add an executable cash-decision evaluation with duplicates, delayed receipts and year boundaries. |
| Translation structure and language quality require different checks. | Add localization guidance and a protected-token evaluation with independent semantic review. A parser pass cannot certify fluency or human approval. |
| Prompt labels, success scores and registration are frequently mistaken for evidence. | Keep source integrity, installation, runtime readiness, selection, task behavior and owner acceptance separate. Include role contracts in evaluation provenance. |

The standard-library installation engine, source hashes, conflict handling and
portable target adapters already serve these boundaries. They remain because of
their function and verification coverage. Replacing them with an SDK or a new
agent application would add lifecycle and compatibility costs without satisfying
an unmet task in this repository.

## Use the composed workflow

From the harness checkout:

```sh
python3 ai.py capabilities plan "CFO" --with "Excel Expert"
python3 ai.py capabilities plan "Mobile App Engineer" --with "Mobile App Designer"
python3 ai.py capabilities show "Senior Team Lead Engineer"
python3 ai.py capabilities check
```

The plan is read-only and accepts declared IDs, titles and aliases. It is not a
natural-language classifier. Read its selected skill bodies and relevant
references, inspect actual tools/data, and register missing project leads:

```sh
python3 ai.py project add --project /absolute/project --capability "CFO" --capability "Excel Expert" --dry-run
python3 ai.py project add --project /absolute/project --capability "CFO" --capability "Excel Expert"
python3 ai.py project sync --project /absolute/project
python3 ai.py project doctor --project /absolute/project
```

The global set stays at 14 skills. Shared leads use that installation; project
roles add only their concrete leads. Supporting skills and stack profiles remain
conditional. A role never grants a tool, account, certification or authority.

## Requested role coverage

| Requested role | Lead workflow | Required result |
| --- | --- | --- |
| Frontend Designer | [`interface-design`](../../../skills/interface-design/SKILL.md) | Rendered pages with purposeful composition, real content, responsive states and working interactions. |
| Backend Engineer | [`backend-engineering`](../../../skills/backend-engineering/SKILL.md) | A versioned service contract exercised through authorization, persistence and failure recovery. |
| Database Admin | [`database-systems`](../../../skills/database-systems/SKILL.md) | Correct constraints, representative query plans and migration/recovery evidence on the named engine. |
| Devops Engineer | [`infrastructure-engineering`](../../../skills/infrastructure-engineering/SKILL.md) | Reviewed infrastructure plan and reproducible build/release/recovery evidence for an exact target. |
| QA Engineer for Design | [`test-design`](../../../skills/test-design/SKILL.md) | A defect report from exercised roles/states with reproducible triggers, expected results and exact visual evidence. |
| Graphic Designer | [`graphic-design`](../../../skills/graphic-design/SKILL.md) | A coherent graphic composition or asset family matched to the brief, medium, brand and delivery specifications. |
| CEO | [`executive-strategy`](../../../skills/executive-strategy/SKILL.md) | A strategic choice connecting customer value, economics, capacity, cash timing and reversibility. |
| CFO | [`financial-analysis`](../../../skills/financial-analysis/SKILL.md) | A reconciled financial decision model with cash timing, obligations, base/downside scenarios and decision thresholds. |
| CTO | [`technical-leadership`](../../../skills/technical-leadership/SKILL.md) | An executable technical decision or delivery plan connecting business constraints, interfaces, ownership, sequencing and integration evidence. |
| Deep Researcher | [`research-and-synthesis`](../../../skills/research-and-synthesis/SKILL.md) | A source-linked synthesis with claim-level support, counterevidence, applicability and unresolved uncertainty. |
| Product Manager | [`product-management`](../../../skills/product-management/SKILL.md) | A defensible product decision, bounded delivery slice, guardrails and measurement contract. |
| Marketing Expert | [`marketing-writing`](../../../skills/marketing-writing/SKILL.md) | Audience-specific positioning/copy with traceable claims and channel fit; a measurable experiment when the task requires one. |
| Native Translator | [`localization`](../../../skills/localization/SKILL.md) | A fluent target-locale translation preserving meaning, register, terminology, formatting and application variables. |
| Project Manager | [`work-planning`](../../../skills/work-planning/SKILL.md) | An executable sequence with dependencies, decisions, ownership, acceptance evidence and resume state. |
| IT Expert | [`it-operations`](../../../skills/it-operations/SKILL.md) | A diagnosed and verified workstation, network or service repair with exact target, cause evidence and recovery path. |
| Optimization Engineer | [`performance-engineering`](../../../skills/performance-engineering/SKILL.md) | A measured bottleneck fix with correctness parity, representative workload and repeatable before/after measurements. |
| Senior Team Lead Engineer | [`technical-leadership`](../../../skills/technical-leadership/SKILL.md) | An executable technical decision or delivery plan connecting business constraints, interfaces, ownership, sequencing and integration evidence. |
| Market Analyst Expert | [`market-analysis`](../../../skills/market-analysis/SKILL.md) | A sourced market decision with customer segments, competitors, reachable demand, explicit assumptions and sensitivity. |
| Entreprenuer | [`venture-validation`](../../../skills/venture-validation/SKILL.md) | A bounded venture thesis, falsifiable demand test, unit economics and a decision to proceed, change or stop. |
| Excel Expert | [`spreadsheet-analysis`](../../../skills/spreadsheet-analysis/SKILL.md) | An editable workbook or live-workbook result with correct formulas, reconciled totals, preserved features and inspected calculated output. |
| Documentation Expert | [`document-workflow`](../../../skills/document-workflow/SKILL.md) | Reader-appropriate documentation or an editable document whose claims, examples, links and final format match authoritative sources. |
| Mobile App Designer | [`mobile-design`](../../../skills/mobile-design/SKILL.md) | Platform-specific navigation, touch/input, accessibility and state recovery verified in context. |
| Mobile App Engineer | [`mobile-engineering`](../../../skills/mobile-engineering/SKILL.md) | A runnable mobile feature on the chosen stack with lifecycle recovery, secure storage, deep links and recoverable network effects. |
| System Engineer | [`systems-engineering`](../../../skills/systems-engineering/SKILL.md) | A coherent system model, capacity/failure analysis, contracts and a verifiable operational design. |

The full [capability contracts](../../capabilities.md) contain failure probes,
prerequisites and acceptance conditions, including additional focused boundaries
such as frontend implementation, diagrams, motion and MCP integration.

## Verification and limits

The [verification report](verification.md) records the exact checks, representative
exercises and independent review. The [task record](../../work/harness-redesign.md)
tracks integration and cleanup; older [verification records](../../verification.md) remain dated
historical evidence.

Source inspection supports the decisions above. It does not establish comparative
model quality, lower latency, lower token cost, a complete security audit or
production readiness across all 24 professions. The new evaluations are public
development cases, not a held-out benchmark. Their structural/arithmetic checks
and agent reviews remain distinct from native-human language review, live client
activation and owner approval.

Optional browser, document, mobile, database, SDK and MCP runtimes must be exercised
in their actual task environment. Microsoft Learn remains an available disabled
service definition; source review did not enable it or connect accounts. OpenAI's
documentation service supplies documentation retrieval, not API execution. No
new hooks, daemons, provider billing routes or background memory capture were
installed as part of the redesign.
