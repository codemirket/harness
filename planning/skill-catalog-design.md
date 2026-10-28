# Native skill catalog contract

Current inventory: **9 global skills, 52 project skills, 7 profiles**.
The [catalog](../skills/catalog.json) is the machine-readable source of IDs,
descriptions, capability pairings, profiles, body/resource hashes and development
cases. The [capability matrix](../docs/deep-review-capability-matrix.md) provides
individual review rows and source/evidence links. Avoid parallel inventories
whose counts or selected skills can drift.

## Installation and authority

Setup installs the nine concise global methods and managed Codex guidance.
Project skills are opt-in; `.mirket/project.json` pins selected IDs. Profiles add
only their baseline `skills`; `conditional` entries need explicit selection.
Sync does not reinterpret profile membership. Exact IDs are also selectable.
No skill name gains a `mirket-` prefix.

Only Mirket-owned files are replaced. Validate source bodies and resource hashes,
refuse unrelated collisions and modified managed copies, preserve rollback and
show the selected changes. No passive tool call writes project skill files.
More than 12 selected project skills produces a context-cost warning.

Codex owns models, native subagents, permissions, shell, editing, Git, general
browsing, documents and SQL. Mirket owns scoped memory, selected JS/TS navigation,
design/video and skill distribution. No skill may imply an unavailable operation
or provision a dependency/account merely by mentioning a reference.

## Nine global skills

`runtime-operations`, `project-memory`, `evidence-research`, `task-planning`,
`agent-coordination-strategy`, `code-change`, `root-cause-debugging`,
`critical-review`, and `outcome-verification` supply narrow general methods.
Keep their triggers selective and their guidance lean. Domain-specific examples
and detailed procedures belong to opted-in project skills/resources.

## Project skills and pairing

Project bodies cover engineering/delivery, browser research/operations, design,
motion/video, selected product discovery/content, data/AI, documents and trust.
Current individual triggers, outputs and references live in the catalog and
capability matrix rather than a second full list here.

The seven optional capability IDs are `navigation.js-ts`, `design.tokens`,
`design.openpencil`, `design.vector`, `design.raster`, `design.diagrams`, and
`video`. Memory is a core route, not an optional pairing ID. Required unavailable
capabilities make skill activation refuse. Conditional capabilities must be
checked before the particular operation. Host-only guidance still requires
actual available Codex/project tools.

A stable MCP tool list is not readiness. Use `runtime_status` to inspect configured
payloads and revalidate operation identity/scope at use. Standard design readiness
does not imply OpenPencil or Sharp. No receipt establishes useful output.

## Authoring, resources and rights

Each body needs a distinct trigger/nontrigger boundary, decision method, concrete
output, failure/stop rule and evidence requirement. Prefer independently worded
methods over imported instructions. Only include resources that resolve a real
domain question; resources are opt-in reading, not additional global prompts.

The [reference selection](references.md), [provenance ledger](../skills/PROVENANCE.md)
and [focused adoption mapping](reviews/2026-10-27/source-design-adoption.md)
separate comparison/link-only sources from the eight reviewed method adaptations.
Reference rights never grant rights to unrelated artwork, fonts or executable
packages. Hallmark/Taste and HyperFrames remain selected; Remotion is historical
comparison evidence only, excluded from adoption.

Run `python3 scripts/refresh-skill-catalog.py --write` after changing a body,
resource or authoring basis, then run the same command without `--write` to verify
consistency. The refresh derives hashes and provenance from repository-owned
inputs; it cannot infer source fidelity, grant rights or complete acceptance.
Retain the pinned retrieval/license receipts and focused reviews because they
are current provenance inputs, even though their observation date is earlier.

## Evaluation and retention

The catalog contains **305 planned development cases**, five per skill. These
are not completed acceptance trials. Static trigger/selection/pairing checks,
installed tool outcomes, sampled model comparisons and human acceptance are
separate evidence. Keep model inputs, settings, baseline, budgets and oracles
matched; inspect saved artifacts and actual effects independently.

Skills must earn their context and maintenance cost through a concrete routing
contract, non-obvious safeguard or measured usefulness. The six tested product/
marketing bodies and content-growth profile were retired after repeated bundle
failure; ordinary product/marketing work remains with Codex. A small tie does
not prove a skill universally useless, and a failed bundle does not identify an
individual causal culprit. The [release qualification](../docs/first-release-qualification-20261028.md)
retains those outcomes and limits.

Creative direction stays with a capable lead and comparable reviewer when useful.
Respect explicit model/reasoning choices, disjoint ownership and lead integration.
One task-level review/repair budget spans specialists and specimens. Guidance is
advisory; Mirket cannot hard-enforce Codex spawning or bypass host approval.
