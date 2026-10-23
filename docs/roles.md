# Select expertise by the result

Use `mirket capabilities list` for current roles and aliases, then
`mirket capabilities plan ROLE` to inspect a lead's prerequisites and evidence.
Routine work needs no role plan. A role supplies expertise; it does not
authorize spending, account access, deployment, outreach or model changes.

## Professional perspectives and team assignments

Every role has a professional profile: its perspective, three priorities, concrete
method, decision rule, challenge to adjacent specialists and expected handoff.
These describe how the professional reaches and revises a judgment. The selected
skill supplies the detailed craft and runtime workflow. They express professional
practice without inventing credentials, human experience or stylistic mannerisms.

For substantive work spanning disciplines, use the [skill-catalog task
execution reference](../skills/skill-catalog/references/task-execution.md) and
ask Mirket to assign professionals only when separate independent work would
materially help or the user/project explicitly requests collaboration. Cross-
discipline scope alone does not require a team; when that gate is met, assign
only professionals whose boundaries are affected:

```sh
mirket capabilities facets
mirket capabilities team "Frontend Engineer" \
  --objective "Build a recoverable checkout flow" \
  --facet "web-interface=The checkout screen and browser state change" \
  --facet "service-contract=The order endpoint must reconcile retries" \
  --facet "security-boundary=Orders must be scoped to the authenticated customer"
```

This plan selects Frontend Engineering as integrator, with other roles only where
the facets justify them. Each facet needs a concrete reason; `--with ROLE` is
explicit support. Up to eight facets and thirteen distinct roles form a plan;
broader work needs bounded phases. The plan is read-only and starts no agents.
The host dispatches useful independent assignments with bounded ownership, and
the lead integrates and verifies them. If no independent host agents exist,
label sequential professional perspectives accurately. Use one existing task
record when durable continuity or coordination requires it.

## Discover starter bundles

In [1.0.11](release-status.md), `mirket catalog profiles` shows
eight starter bundles. Use `mirket catalog profiles --all` for specialist and
alternative bundles. Existing IDs and saved selections keep their membership.
Whole-workflow alternatives remain opt-in through a source keyword or exact skill
ID; avoid stacking competing execution methods for the same assignment.

## Choose one lead

| Work | Lead selection |
| --- | --- |
| Screen composition or visual refinement | Frontend Designer; Frontend Engineer supports actual framework and state behavior. |
| Backend, database, mobile or desktop behavior | The matching Engineer or Database Admin role; use the real platform runtime. |
| Editable graphics or illustrations | Graphic Designer or Illustrator for medium and composition; use `svg-creation` with SVGO for vectors, Sharp for raster preparation. |
| Motion | Motion Designer starts with ordinary UI interactions; choose `advanced-motion` for art-directed website scenes, color/gradients and procedural Canvas, `svg-animation` for editable vector choreography, or `page-transitions` for navigation. The explicit `motion-studio` profile includes two contrasting browser examples. For video, explicitly select a host encoder; Mirket bundles none. Inspect frames and playback. |
| Regressions and release assessment | QA Engineer for system and user-journey checks; the same role's interface mode adds visual baselines, interaction and accessibility evidence. |
| Trust boundaries and agent tools | Security Engineer; MCP Engineer owns protocol, transport and client/tool behavior when that boundary is affected. |
| Ingestion, replay and published datasets | Data Engineer; Data Analyst owns interpretation and Database Admin owns engine mechanics. |
| Inclusive user journeys | Accessibility Engineer; verify keyboard, semantics and relevant assistive technology, separating source and automated checks from actual runtime evidence. |
| LLM, retrieval and tool-using features | AI Engineer; use fixed evaluation inputs and verify the promised effect independently of fluent output. |
| Reliability, queues and failure containment | System Engineer or Site Reliability Engineer; Devops Engineer owns infrastructure changes and release guidance supports authorized rollout. |
| Artifact promotion and rollout recovery | Release Engineer owns immutable artifact identity, mixed-version compatibility, progression gates and observed recovery. |
| Schema changes, backfills and data cutover | Data Migration Engineer owns ordering, concurrent-write coverage, resumability and semantic reconciliation; Database Admin supports engine mechanics. |
| Customer behavior and product choices | UX Researcher supplies traceable observations; Product Manager owns the resulting product decision. |
| Causal tests | Experimentation Analyst; define units, assignment, stopping and integrity checks before interpreting an uplift. |
| Business decisions | CEO, CFO, CTO, Entrepreneur or Market Analyst for the actual decision, with clearly labeled assumptions and decision authority. |
| Marketing | Marketing Expert for positioning and copy; choose customer, search, conversion, experiment or channel guidance for the broader outcome. The marketing profiles expose optional launch, pricing, email and community methods. |
| Technical and editorial documentation | Documentation Expert owns source fidelity, runnable examples and reader tasks; Native Translator owns locale and semantic fidelity. |
| Editable office artifacts | Document Designer or Presentation Designer selects Office Document Production and the available native format workflow; Excel Expert owns workbook logic and recalculation. |

Task probes are examples of meaningful failures, not instructions to manufacture
unrelated work. A presentation-only change does not require a database migration;
a query repair does not require designing a new ingestion platform. Apply the
contract to the requested outcome and affected consumers.

## Shared roles and distinct modes

Several names select the same complete role. Planning through any alias returns
one canonical contract and lead, including when aliases are composed together.
Choose the mode from the deliverable rather than loading every method.

| Canonical role | Other accepted names | Mode-specific evidence |
| --- | --- | --- |
| Graphic Design | Illustrator, Visual Asset Designer, Illustration, Crafting Visual Assets | Illustration construction and focal detail; asset-family consistency and consumer integration; composition and type at delivery size. |
| Web Design | Frontend Designer, UI Designer, Web Designer, Design Revision, Design Revisioning | New-page composition or revision with approved constraints and comparable before/after states. |
| Quality Assurance | QA Engineer, Quality Engineer, Software Test Engineer, QA Engineer for Design, Design QA, Visual QA, Interface QA Engineering | Independent behavior expectations and actual effects; interface mode adds stable visual baselines and relevant accessibility checks. |

The IDs `illustration` and `visual-assets` resolve to `graphic-design`,
`design-revision` to `web-design`, and `interface-qa` to `quality-assurance`.
These aliases preserve existing requests and project selection commands.

Adjacent roles remain distinct when their result or evidence differs. Brand rules
need application specimens; graphic production needs actual masters and exports.
Infrastructure owns desired and observed configuration; systems engineering owns
capacity and failure behavior; release engineering owns artifact promotion and
recovery. Data pipelines own repeatable ingestion; migrations own a bounded
transition between data contracts. Product choices, venture validation and executive
allocation have different decision criteria. Office production owns saved-file
editability and layout while documentation owns claims and usable explanations.

## Compose only necessary support

```sh
mirket capabilities plan "Data Engineer" --with "Database Admin"
mirket capabilities plan "Accessibility Engineer" --with "Frontend Engineer"
mirket capabilities plan "AI Engineer" --with "MCP Engineer"
mirket catalog read data-engineering
```

`--with` accepts capability names. A contract's `support` list instead contains
optional skill IDs. Use `mirket catalog read ID` for the relevant body and
`mirket project add --project /absolute/project --skill ID` when installation is
needed. Profiles group project installations; they are not a request to load
every member or activate competing provider workflows.

`project init` records target hosts without selecting project skills. Add the
`project-foundation` profile explicitly for an engineering or release boundary.
The explicit `portable-foundation` profile installs shared guidance into the
selected project targets for hosts without a global setup. Choose it deliberately
to avoid two copies of the same invocation across project and global
installations.

## Verify availability and outcome separately

The core guidance is available offline. Optional catalog entries can require
additional tools, SDKs, accounts or runtimes; their names do not supply those
prerequisites. `catalog show` identifies delivery, pins, dependencies and caveats.
Every role lead's dependency metadata names its required runtime or evidence route.
These requirements are descriptive: they neither install a tool nor guarantee it
can run. Native host tools and the project's existing runtime take precedence;
optional Mirket packages are selected only when they fit the requested deliverable.
For example, graphic work routes through the single `design` package: Satori plus
resvg for static composition, local D2/TALA for system diagrams, OpenPencil file tools for editable
files and Playwright plus Odiff for inspection. Select the tool from the output.
Exact IDs remain directly searchable. Search ranks query relevance first:
positive role/name/tag matches precede description-only matches;
within that relevance tier, exact role/name matches precede exact tags and
partial names. Delivery is a final tie-breaker only; catalog search does not
inspect whether an entry is installed in a home, project or host.

`project doctor` checks installed copies. `doctor` checks device installation and
the configured protocol. Neither proves that an agent selected the skill or that
its output is correct. Verify representative work through the actual consumer:
durable data, saved artifacts, native applications, browsers or source evidence.
Report untested platforms and human or assistive-technology checks separately.
