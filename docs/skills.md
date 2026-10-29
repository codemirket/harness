# Native skill catalog contract

Current inventory: **9 global skills, 52 project skills, 7 profiles**.
The [catalog](../skills/catalog.json) is the machine-readable source of IDs,
descriptions, capability pairings, profiles, body/resource hashes and development
cases. The [skill route audit](skill-routes.md) provides
individual review rows and source/evidence links. Avoid parallel inventories
whose counts or selected skills can drift.

## Installation and authority

Pi loads skill resources from the assembled local package. The `global` and
`project` groups are catalog selection groups, not copied host directories.
Profiles union only their baseline `skills`; conditional entries require explicit
selection. Exact IDs remain selectable, with no `mirket-` prefix. Pi settings are
the sole active selection source. A project package object replaces the same
global package object; `autoload:false` filtering uses pi's native delta semantics.
An unfiltered package can load every manifest skill and must be reported honestly.

Validate body/resource hashes before assembly. Do not copy into `.agents/skills`,
write a second profile database or alter unrelated package settings. Profile edits
must bind the original settings bytes and refuse drift or duplicate ownership.
Pi owns models/authentication, sessions and built-in read/edit/write/bash tools.
Mirket owns scoped memory, selected JS/TS navigation and finite creative jobs.
Browser/search and image generation require concrete admitted integrations;
documents, recalculation, rendering and SQL use installed project commands.
No skill can provision a dependency/account merely by mentioning a reference.

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
skill route audit rather than a second full list here.

The seven optional capability IDs are `navigation.js-ts`, `design.tokens`,
`design.openpencil`, `design.vector`, `design.raster`, `design.diagrams`, and
`video`. Memory is a core route, not an optional pairing ID. Required unavailable
capabilities make skill activation refuse. Conditional capabilities must be
checked before the particular operation. Host-only guidance still requires
actual available pi/project tools.

A stable MCP tool list is not readiness. Use `runtime_status` to inspect configured
payloads and revalidate operation identity/scope at use. Standard design readiness
does not imply OpenPencil or Sharp. No receipt establishes useful output.

## Authoring, resources and rights

Each body needs a distinct trigger/nontrigger boundary, decision method, concrete
output, failure/stop rule and evidence requirement. Prefer independently worded
methods over imported instructions. Only include resources that resolve a real
domain question; resources are opt-in reading, not additional global prompts.

The [reference selection](references.md), [provenance ledger](../skills/PROVENANCE.md)
and [focused adoption mapping](../packaging/provenance/skills/source-design-adoption.md)
separate comparison/link-only sources from the eight reviewed method adaptations.
Reference rights never grant rights to unrelated artwork, fonts or executable
packages. Hallmark/Taste and HyperFrames remain selected; Remotion is excluded
from adoption.

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
contract, non-obvious safeguard or measured usefulness. A small tie does not
prove a skill universally useless, and a failed bundle does not identify an
individual causal culprit. [Qualification](qualification.md) states the bounded
comparison's limits.

Creative direction stays with a capable lead and comparable reviewer when useful.
Respect explicit model/reasoning choices, disjoint ownership and lead integration.
One task-level review/repair budget spans specialists and specimens. Managed delegation limits apply only to the optional Mirket entry point; they
do not sandbox arbitrary shell processes or unrelated extensions.
