---
name: critical-review
description: "Critique a concrete change, plan or artifact for consequential defects using current evidence, severity and a reproducible consequence. Skip cosmetic rewriting unless requested."
---

# Critical review

Bind the review to the actual revision, diff or saved artifact and its intended contract. Inspect affected consumers and enough unchanged context to understand the boundary. Prioritize defects that change correctness, user outcomes, security, recovery or maintainability at realistic scale.

Check completeness across callers, fixtures, configuration and exports, including validation or recovery lost during consolidation. Question added machinery only when a simpler existing path meets the same contract; tie any removal recommendation to a concrete maintenance or failure cost.

For each candidate finding, trace the triggering conditions and concrete consequence. Check existing protections and tests before reporting it. Prefer a minimal example, source path/line or observed artifact defect that another person can verify. Separate demonstrated defects from a material unresolved question; discard speculative warnings with no plausible affected path.

Rank findings by impact and likelihood under stated conditions, not by dramatic language. Give a concise correction direction without forcing a preferred architecture. A review is advisory evidence; it does not certify human approval or authorize commits, publication or deployment.

Return actionable findings first, with precise locations and verification limits. If no actionable defect is found, say so and name the meaningful untested boundaries. Stop when the requested surface and high-risk consumers are inspected to the agreed depth; do not manufacture findings or broaden into an unsolicited audit.
