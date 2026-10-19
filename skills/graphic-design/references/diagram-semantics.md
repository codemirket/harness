# Design the relationship before the diagram

Use for explanatory flows, architecture, provenance and state diagrams. The
existing production-and-proof reference owns rendering and export. Kroki remains
the route for supported declarative graph/sequence layouts; this method does not
replace layout engines with manually positioned graph nodes.

## Decide what the reader must infer

Recover the facts from the actual system or supplied source. Name entities,
relationships, direction, conditions and the question this view answers. Keep
unknowns visible as unknowns. Record a small correspondence between source facts
and diagram elements when importing or simplifying: what was retained, merged,
omitted and why. Attractive geometry cannot establish semantic fidelity.

Choose a representation by the relationship, not by an available template:

| Relationship | Information that must survive | Convincing but false shortcut |
| --- | --- | --- |
| Queue or capacity | Producer, buffer/capacity, consumer, overflow or backpressure when relevant | A single forward arrow hiding the full-buffer behavior |
| Policy or routing | Evaluated condition, pass/fail, skipped versus not reached, outcome | Green arrows that imply every check ran successfully |
| Provenance or transformation | Source identity, transformation, resulting artifact and missing/uncertain fields | A polished output with the source association silently erased |
| Lifecycle | Actual states, triggering events, waiting, retry/cancel and terminal outcomes | A linear happy path presented as the whole lifecycle |
| Trust boundary | Where authority/data crosses a boundary and which operation crosses it | Grouping by visual proximity and implying equal trust |
| Architecture change | Stable entity identity, changed responsibility and changed edges | Two independently laid-out graphs making movement look like change |

Avoid invented exact capacities, timings, probabilities or authorization rules.
Use a visibly illustrative value only when the task permits examples. For a large
system, choose a useful view and link its continuation rather than dropping
required states to satisfy a fixed node limit.

## Construct readable geometry

Use layout direction and grouping to express hierarchy. Distinguish containers,
nodes and annotations; an annotation should not look like another runtime actor.
Give connectors explicit endpoints, direction and condition labels where needed.
Check label collisions, crossings, ambiguous junctions and the implied order of
branches at the actual embedding width. Decorative proximity must not manufacture
a relationship absent from the source.

Use color redundantly with labels, line treatment or symbols. Preserve the
project's visual system and the requested density. Fixed palettes, mandatory
fonts, bans on diagonal connectors and universal node/arrow counts are not
semantic rules. Inspect the complete picture and readable labels at use size;
a geometry checker alone cannot verify meaning or hierarchy.

## Compare architecture without manufacturing a difference

Assign stable source identities to unchanged entities and relationships. Keep
before/after positions and scale comparable where the layout permits. Label
added, removed, changed, moved and rewired separately; a moved box is not
necessarily a changed service. If layout must change, explain that separately
from the system delta. Retain removed edges long enough to make their removal
understandable, without implying that they still operate in the resulting state.

Example: replacing a worker's direct database write with an event queue adds a
queue and consumer and rewires the write path. The unchanged API should remain
recognizable. Show the actual failure/retry responsibility if the question is
reliability; do not infer exactly-once delivery from the addition of a queue.

## Animate actual events

Give movement a semantic job: enqueue, evaluate, reject, retry, transform or
resolve. One traveling marker should have a consistent identity. Arrival at a
node does not imply successful completion unless that is the documented event.
Do not visually run mutually exclusive paths simultaneously as though they were
one execution. A separate comparison or labeled scenario can show alternatives.

Keep the static/reduced-motion view complete enough to answer the diagram's
question. For long sequences, show the relevant state and a readable explanation;
controls and essential labels must not wait for an entrance animation. Use the
existing motion owner for playback, interruption and export: `motion-design` for
ordinary event feedback, `advanced-motion` for coordinated explanatory scenes,
and `svg-animation` for editable vector choreography. A blinking connector
is not evidence that the modeled system behaves that way.

## Verify a misleading alternative

Alongside ordinary source/output review, try a failure the picture could hide:
full buffer, denied policy, missing field, cancellation or rewired edge. Trace it
through the diagram and the actual contract. Repair the semantic model before
polishing the affected shape. Deliver editable source, inspected export, legend
where needed and any deliberately omitted scope.

## Provenance

Selective methods adapted in original wording from cathrynlavery/diagram-design
at `550fd66edff058bc73056c3416b76d69cfe865ba`: `SKILL.md`, semantic-patterns,
type-architecture-delta and animation references. MIT © 2025 Cathryn Lavery;
the full notice accompanies the skill. No upstream icons, fonts, scripts,
templates, controller conventions or mandatory style questionnaire are bundled.
