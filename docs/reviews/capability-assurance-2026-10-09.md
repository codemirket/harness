# Capability assurance — 2026-10-09

The harness now gives all 23 requested domains an explicit route, inspectable
guidance, a concrete delivery contract and a discriminating failure probe.
Eight missing specialist workflows were added. This is stronger workflow coverage
with bounded task evidence, not a certification of expertise or a measured general
improvement over another harness.

## What changed

- Added backend-engineering, brand-guidelines, mobile-design, systems-engineering,
  infrastructure-engineering, project-scaffolding, performance-engineering and
  executive-strategy, each with a substantive worked reference.
- Added [capability contracts](../capabilities.md) and `ai.py capabilities
  list/show/check`. Source checking rejects missing, unsupported, unreviewed or
  stale local guidance. It never labels runtime readiness or quality as passed.
- Updated catalog search/profile membership, task routing and delivery standards;
  strengthened interface QA with race, keyboard/focus and durable-effect probes.
  The 14 global skills and four-skill project foundation remain unchanged.
- Added three [reproducible assurance exercises](../../examples/assurance/README.md).
  They preserve raw observations and reject specified bad behavior. Plugin
  snapshots include these examples and the capability engine.
- Added a [feedback-control method](../../skills/ai-system-evaluation/references/feedback-controls.md)
  connecting observed defects to guidance, deterministic checks and artifact
  review. It applies the user-supplied harness-engineering article to this repo.

Existing skill hashes also needed refresh because their selected third-party
notice file had changed during publication. The payload includes that notice,
so unchanged main skill text does not imply an unchanged payload. Hashes were
recomputed from actual selected files, preserving notices.

## Evidence by requested capability

“Source reviewed” means actual instructions and relevant references were read.
“Exercised” names the limited result observed; it does not imply all paths in that
domain were tested. Full details and source links are in the focused reports.

| Capability | Available workflow and current evidence | Important remaining boundary |
| --- | --- | --- |
| Frontend Engineering | Existing frontend/async-state guidance; working Fieldwork source and current rendered views inspected | Framework-specific concurrency and real backend integration |
| Backend Engineering | New command-service workflow; 12 real SQLite callable-service checks and four detected defect mutations | HTTP authentication/serialization, distributed effects, process/power failure |
| Brand Guidelining | New brand workflow; token extraction, contrast calculation, editable masters and rendered specimen inspected | Owner approval, other channels, print and dark theme |
| Web Design | Existing interface-design composition workflow; current desktop/mobile Fieldwork renders inspected | Human visual preference and actual product content |
| Mobile Design | New platform-specific journey and keyboard/back/text guidance; responsive web rendered | Native build, simulator/device input, VoiceOver/TalkBack |
| SEO Expertise | Raw local HTTP evidence across eight URLs; 12 checks reject robots-only indexability and observe changed directives | Live crawling/indexing, Search Console, JS rendering and ranking |
| Deep Researches | Four-source conflicting-version brief checked and independently agent-reviewed against all inputs | Broad retrieval, inaccessible sources and held-out research accuracy |
| Interface QA Engineering | Expanded interface-QA reference; current specimen review found double focus styling and consistency concerns | Real app roles, async races, persistence and assistive-technology journeys |
| Illustration | SVG craft route; six original editable material studies rendered in context | Raster generation/provider and human artistic acceptance |
| Crafting Visual Assets | SVG asset family, extracted mark, dimensions and contrast inspected; explicit raster route | Additional export/editor/print and raster-provider checks |
| Animation | Existing control/navigation/advanced/SVG guidance; unchanged browser specimen has prior normal/reduced-motion evidence | New choreography, native/video engines, physical smoothness |
| Project Review | Architecture/security/test route; independent reviewers found and verified repairs in this integration | An unrelated project's architecture and consequential hidden defects |
| Planning | Existing dependency, continuity and delivery-review guidance applied to this multi-part change | Dedicated interruption/change-of-assumption trial |
| Product Management | Existing decision/slice/guardrail method; analysis brief supports a bounded rollout choice | Direct discovery and real customer/product decisions |
| Database Expert | SQLite transactions, rollback, simultaneous writes and indexed-query result parity exercised | Other database engines, actual restore and production workload |
| System Engineering | New topology/capacity workflow; bounded queue rejects overload and drains after modeled recovery | Real queue broker, outages, crash durability and production capacity |
| Devops and Infrastructure Engineering | New desired/observed-state workflow; synthetic plan-input changes detected | Real IaC provider state, deployment, backup/restore and recovery |
| Marketing Expert | Existing claim-preserving/channel-aware guidance source-reviewed | Channel-rendered campaign trial, audience response and conversion evidence |
| CEO perspective | New cash/capacity/allocation workflow; fictional monthly balances and decision-changing thresholds recalculated | Actual customer/economic inputs and owner decision |
| Data Analysis and Deep Understanding | Nine checks on account grain, deduplication, missing strata and mix reversal; independent agent review | Statistical inference, large/connected data and other grains |
| Project Scafolding | New staged-generator workflow; fixture preserves user files, executes valid/invalid entry and rejects conflicts | Actual selected native generator and target stack |
| Design Revisioning | Existing preservation workflow; brand extraction locates concrete consistency proposals without silently redesigning | A requested revision and comparable approved before/after result |
| Efficiency and Optimization Expert | New controlled-comparison workflow; equal query results, raw repeated read/write timings and storage tradeoff | Representative target workload, concurrent/cold behavior and production cost |

Focused evidence: [engineering](capabilities-engineering.md),
[design](capabilities-design.md), [business/research](capabilities-business.md),
[SEO](capabilities-seo.md). Generated `build/` evidence is local and intentionally
ignored; the shipped examples/evaluation fixtures provide the repeatable path.

## Integration and review

All eight new specialists were registered, synchronized and checked in an isolated
project for both `.agents/skills` and `.claude/skills`. Copied skill bodies and
references matched source; an existing sentinel file stayed unchanged. This proves
registration and payload portability, not that either desktop client activated a
skill in a fresh session. Specialists are selected by project need, not installed
globally merely because this audit covers many domains.

Independent agent review found a real validator mismatch: existing global-link
entries omit `license_files`, while the local payload reader requires it. The
capability checker now normalizes that optional field and has a regression using
the real schema. Review also corrected ambiguous backend leadership, raster asset
routing, native-device evidence wording and an unconditional marketing-experiment
requirement. Those fixes preserve one lead and task-appropriate scope.

The parent read the research brief against all four supplied sources, inspected
the analysis script/results/decision against the data contract, and recorded agent
acceptance in the existing evaluation mechanism. It also inspected the brand
specimen and mobile render. No human acceptance or blinded comparison was recorded.

Final verification: **563 repository tests passed**, including new capability
schema/integrity, two-target installation and retained assurance-exercise checks.
Registry documentation generation and Markdown checks passed. A fresh
`personal-workbench` export contained the capability CLI, all 23 valid source
contracts and retained assurance examples; exporting did not install the plugin.
The optional generic skill-creator validator could not run because PyYAML is absent;
repository frontmatter, payload, reference and installation checks did run.

## Recheck and use

```sh
python3 ai.py capabilities check
python3 -m unittest discover -s tests -q
python3 scripts/render_registry.py --check
python3 ai.py capabilities show backend-engineering
```

Follow `show` with project registration, sync and doctor for the needed skills.
Check the target runtime, read the selected skill and relevant reference, then
exercise its contract on the actual deliverable. Use source checks for source
integrity and task evidence for task quality. For a claim of broad improvement,
the next useful evidence is repeated comparable runs on authorized held-out
projects with independent review and human assessment of subjective work.
