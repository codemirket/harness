# Task routing audit

Reviewed on 2026-10-06. The catalog supplies relevant UI, engineering,
documentation, animation and search workflows. Prior installation evidence did
not establish task selection or better design output. This audit found and
repaired concrete discovery and handoff defects; it does not certify UI quality.

## Findings and source corrections

- The registrar resolves requested profiles/IDs and companions. It is not a
  deterministic natural-language task router. The agent chooses workflows from
  skill descriptions, project instructions and the global selector.
- Global domain skills lacked specific catalog handoffs. The selector now links
  [task routing](../../skills/skill-catalog/references/task-routing.md), which
  distinguishes QA from redesign, repository documentation from document files,
  motion implementation from effect naming, and research from SEO/GEO. It records
  catalog IDs separately from installed invocation names and keeps project owners
  authoritative. Matching domain skills direct substantial tasks to this guide.
- `ui-design` discovery missed the main redesign and reference workflows. `docx`
  discovery missed portable office authoring. Search now normalizes separators,
  uses reviewed catalog tags alongside upstream discovery metadata, includes
  authored local skills, and ranks exact reviewed capability tags ahead of
  unreviewed leads. Raw inventory metadata remains unchanged and available as
  `source_description` on enriched search rows. Search is still metadata lookup;
  a whole task prompt is not a reliable query or an automatic selection.
- Installed `mobile-native`, `break-ui` and `prototype` referred to unavailable
  upstream animation/design siblings. Exact-count adaptations now route to the
  applicable interface/motion/catalog workflow and shorten discovery descriptions.
  Original source pins, source hashes, companion files and licenses are retained.
  The content-stress integration note also honors previously authorized fixes and
  existing fixture/review tools instead of requiring a new approval or toggle.
- Global `interface-design` now requires a rendered baseline, named visual
  problems, an intended direction, comparable after renders and refinement when
  those problems remain. Passing a build or accessibility scan is insufficient.

## Trixpo as the reference

The reference checkout at `/Users/nazmi/Projects/trixpo` was inspected read-only.
At the baseline, harness doctor passed for 41 Codex and 42 Claude registrations.
Its selected frontend capabilities include `frontend-engineering`,
`mobile-native`, `break-ui` and `motion-design`; technical documentation and
reference handoff capabilities are also registered. A dedicated redesign skill
is absent. Its own Open Precision, Visual Identity and Design Workflow already
require actual component rendering, screenshot inspection, both themes and
appropriate before/after comparisons. The project is not missing design rules.

This supports a hypothesis: an agent can complete engineering/QA work while
leaving composition and art direction insufficiently improved. It does not prove
the cause of the user's previous result; that task's prompt, trace, renders and
owner acceptance were not supplied. Automatically adding all style profiles would
introduce competing directions rather than resolve that uncertainty.

An independent fresh `gpt-6-sol` / `high` task-pickup exercise read the current
selector/route guide, global interface workflow, Trixpo's actual project owners
and installed frontend skill. For a property-editor hierarchy request, it kept
the real `draft | validated | retired` lifecycle, locale fields and persistent
Save action. It chose project brand guidance and an inspected-render iteration,
and identified optional catalog ID `taste-redesign-skill` (invocation
`redesign-existing-projects`) without installing it. Adjacent Nuxt caching
research and a mechanical variable rename received distinct, proportional routes.
No implementation or rendered UI review was performed in this exercise.

A separate eight-scenario selection probe covered UI polish, Vue async forms,
README/ADR writing, animation defects, installed-version framework research,
SEO/GEO auditing, a log typo and editable DOCX creation. It returned applicable
selections and boundaries. Some upstream choices were assessed from metadata and
adaptations rather than their full bodies. This is a selection smoke exercise,
not an activation benchmark, repeated A/B test or end-to-end execution result.

## Verification and rollout boundary

Discovery regression checks exposed missing UI-design aliases and DOCX authoring
before the metadata fixes. New behavior checks cover separator variants,
reviewed tags, source-filter preservation, local/global installability reporting,
unreviewed candidates and ranking. The full suite and generated-doc checks are
recorded in [verification](../verification.md).

The real pinned fetch/registration path installed a scratch both-target project:
12 foundation skills plus mobile-native, break-ui, prototype and motion-design,
32 copies total. Doctor returned 0 with every destination unchanged. All six
adapted Emil copies retained licenses, matched their reviewed installed hashes
and contained the corrected motion route without stale sibling invocations.
Repeat sync preserved bytes and timestamps. Registration executed no upstream
helpers. See [isolated evidence](../evidence/skill-routing-2026-10-06.json).

The generic skill-authoring validator could not start because PyYAML is absent.
No dependency was installed. Repository tests validate names, payload hashes,
adaptation and registration, but do not replace that optional validator or actual
workflow execution.

Global live links expose updated source bodies on this Mac. A fresh session is
needed to assess discovery from updated descriptions. Existing project copies
and generated plugin exports remain pinned until deliberately synchronized or
regenerated. Trixpo was not synchronized and its product files were not changed.

The next quality evaluation is a bounded real UI change using a reproducible
baseline and task-specific visual criteria. Inspect the resulting production
component in matching content, state, theme and viewport, preserve interaction
checks, and retain owner feedback. That evidence can establish improvement for
the tested surface; selection or installation alone cannot.
