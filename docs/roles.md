# Select expertise by the result

Use `mirket capabilities list` for the current roles and exact aliases, then
`mirket capabilities plan ROLE` to inspect the lead, prerequisites, acceptance
evidence and a discriminating failure probe. Role names select expertise. They
do not authorize spending, account access, deployment, outreach or model changes.

## Professional perspectives and team assignments

Every role has a professional profile: its perspective, three priorities, concrete
method, decision rule, challenge to adjacent specialists and expected handoff.
These describe how the professional reaches and revises a judgment. The selected
skill supplies the detailed craft and runtime workflow. They express professional
practice without inventing credentials, human experience or stylistic mannerisms.

For substantive work spanning disciplines, the host interprets the request and
affected sources, then asks Mirket to assign the related professionals:

```sh
mirket capabilities facets
mirket capabilities team "Frontend Engineer" \
  --objective "Build a recoverable checkout flow" \
  --facet "web-interface=The checkout screen and browser state change" \
  --facet "service-contract=The order endpoint must reconcile retries" \
  --facet "security-boundary=Orders must be scoped to the authenticated customer"
```

This selects Frontend Engineering as integrator, with Web Design, Quality
Assurance, Backend Engineering and Application and Agent Security. Repeated roles
receive one assignment retaining each contribution. Each facet needs a concrete
task-specific reason; mentioning a database or a brand in passing is insufficient.
Use `--with ROLE` for additional explicitly chosen expertise. Up to eight facets
and thirteen distinct roles can form a plan; broader work needs bounded phases.
Facets select only their listed roles and never recursively expand support lists.

The host dispatches these assignments through its actual subagent tools. Give each
professional the relevant skill, raw evidence, owned paths, dependencies, permitted
effects and return requirements. Ask for its own initial diagnosis, alternatives
and evidence before sharing peer conclusions when practical. Queue within host
capacity; implementation dependencies determine when work can begin. The lead
integrates the contributions and resolves disagreements against the user's
constraints and observations, retaining unresolved dissent. Consensus alone is
insufficient to accept a product.

The plan is read-only and starts no agents. Save it before task creation, use its
`task_start` MCP fields or `task_start_cli.args`, bind the roster as task evidence,
and track real worker IDs and contribution paths in the existing task checkpoints.
The coordinator owns shared revision updates. Independent execution must be
observed through the host; skill delivery and artifact hashes cannot authenticate
a worker or judge its reasoning. If subagents are unavailable, disclose the limit
and label sequential professional perspectives accurately. Existing single-role
plans and direct handling of trivial work remain available.

## Choose one lead

| Work | Lead selection |
| --- | --- |
| Screen composition or visual refinement | Frontend Designer; Frontend Engineer supports actual framework and state behavior. |
| Backend, database, mobile or desktop behavior | The matching Engineer or Database Admin role; use the real platform runtime. |
| Editable graphics or illustrations | Graphic Designer or Illustrator for medium and composition; read `svg-creation` for vector production or use available native image tools for bitmap work. |
| Motion | Motion Designer starts with ordinary UI interactions; choose `advanced-motion` for art-directed website scenes, color/gradients and procedural Canvas, `svg-animation` for editable vector choreography, or `page-transitions` for navigation. The explicit `motion-studio` profile includes two contrasting browser examples. For a video deliverable, the optional HyperFrames and Motion Canvas packages supply frame inspection, seek verification and local video export through `advanced-motion` guidance. |
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

The ordinary project foundation relies on shared global skills. The explicit
`portable-foundation` profile installs shared guidance into the selected project
targets for hosts without a global setup. Choose it deliberately to avoid two
copies of the same invocation across project and global installations.

## Verify availability and outcome separately

The core guidance is available offline. Optional catalog entries can require
additional tools, SDKs, accounts or runtimes; their names do not supply those
prerequisites. `catalog show` identifies delivery, pins, dependencies and caveats.
Every role lead's dependency metadata names its required runtime or evidence route.
These requirements are descriptive: they neither install a tool nor guarantee it
can run. Native host tools and the project's existing runtime take precedence;
optional Mirket packages are selected only when they fit the requested deliverable.
For example, graphic work may use native image tools, SVG and a browser, Inkscape,
or a compatible design editor; no single package is required for every graphic.
Exact IDs remain directly searchable. Other searches favor positive role/name/tag
matches and available authored guidance over exclusion prose or manual references.
Within that tier, an exact role/name match precedes a tag match, which precedes
a partial name match.

`project doctor` checks installed copies. `doctor` checks device installation and
the configured protocol. Neither proves that an agent selected the skill or that
its output is correct. Verify representative work through the actual consumer:
durable data, saved artifacts, native applications, browsers or source evidence.
Report untested platforms and human or assistive-technology checks separately.
