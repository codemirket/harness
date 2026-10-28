# Opportunity and decision map

Use the requested artifact/layout; the path below is a default when none is specified. Embed the relevant records and share evidence links in a combined deliverable instead of creating extra sidecars.

For a non-trivial product decision, keep a project-local
`product/opportunity-map.json` with this chain:

`outcome -> opportunity -> assumption -> test -> decision`.

Each observed opportunity and decision has a stable id and links to the
evidence register. An assumption may be explicitly `unknown` with no evidence
reference while it is untested; do not invent evidence to fill that field.
Record rejected hypotheses and why they were rejected. A test gets a decision
rule before it runs; an inconclusive result remains inconclusive instead of
being promoted to a roadmap commitment. Keep the smallest test that can
distinguish the live options and name the owner, time boundary, review date,
and stop condition in the surrounding decision record.

This is a decision aid, not a promise that a roadmap outcome will occur. It
combines opportunity-solution-tree decomposition with a DATE-style sequence:
diagnose the actual constraint, analyse alternatives, take a differentiated
path, then experiment. Weekly or biweekly discovery is a useful cadence, not a
license to manufacture evidence when the domain cannot support it. See
[Teresa Torres, 2025-05-21](https://www.producttalk.org/ship-value-weekly/)
and [Lenny Rachitsky / Krithika Muthukumar, 2025-05-25](https://www.lennysnewsletter.com/p/growth-tactics-from-openai-and-stripes-first-marketer).

Repository qualification tooling enforces referential integrity and explicit
rejection reasons. It does not choose a strategy or require conversion proof
from a planning file.
