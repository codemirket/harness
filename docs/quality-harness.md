# Delivery quality across projects

The harness now connects project context, task routing, substantive expertise,
available tools and inspected output. More installed skills alone cannot produce
consistently excellent work. The earlier routing repair at `604c54e` established
better discovery; this update adds concrete craft references and an output
feedback loop across domains.

## What was missing and what changed

Shared skills gave sound principles but little worked detail for many difficult
choices. Registration checks established available files, while agents could
finish without a comparable render, executed example or independently reviewed
conclusion. A general API task also needed a route distinct from MCP protocol
work. These are actionable gaps in the harness, not an explanation for every
model or project failure.

| Layer | Implemented change | Evidence it needs |
| --- | --- | --- |
| Project context | Preserve project instructions, accepted requirements, stack and available tools | Read actual owners/configuration; resolve material uncertainty |
| Routing | Explicit new-screen/public-page, API connector, MCP, technical documentation and research boundaries | Selected installed body and applicable reference, beyond a catalog match |
| Expertise | Application/public-page composition, interaction recipes, async ownership, engineering boundaries, service reliability, technical writing and claim verification | Decisions grounded in task and project rather than generic style rules |
| Execution | Domain delivery standards selected for substantial work | Actual renders, affected behaviors, examples, tool effects or supporting sources |
| Evaluation | Five prepared development tasks with bounded checks and separate artifact review | Current checks and reviewed output with scope and limitations |

The shared entry point stays small. Detailed procedures live beside each skill
and load only for a matching task. Profiles remain installation groups; they do
not prescribe loading every member. Existing brand, architecture, documents and
selected authoritative workflows govern implementation. The update adds no
production dependency, model runner, hook or account connection.

## Reachable expertise

- `interface-design`: [application composition](../skills/interface-design/references/application-composition.md)
  and [public-page composition](../skills/interface-design/references/public-page-composition.md).
- `motion-design`: [interaction recipes](../skills/motion-design/references/interaction-recipes.md),
  including interruption, ownership and reduced motion.
- `frontend-engineering`: [async state](../skills/frontend-engineering/references/async-state.md)
  with stale-result and cancellation examples.
- `engineering-judgment`: [boundary decisions](../skills/engineering-judgment/references/boundary-decisions.md)
  and [service integration](../skills/engineering-judgment/references/service-integration.md).
- `mcp-integration`: [protocol completion](../skills/mcp-integration/references/protocol-completion.md).
- `document-workflow`: [technical documentation](../skills/document-workflow/references/technical-documentation.md).
- `research-and-synthesis`: [claim verification](../skills/research-and-synthesis/references/claim-verification.md).
- `skill-catalog`: [task routing](../skills/skill-catalog/references/task-routing.md)
  and [delivery standards](../skills/skill-catalog/references/delivery-standards.md).
- `ai-system-evaluation`: [harness development](../skills/ai-system-evaluation/references/harness-development.md).

## Run a development exercise

The dependency-free `eval` CLI prepares a new scratch directory with task,
rubric, verifier, seed workspace and source fingerprints. It does not launch a
model or choose its settings. Assign the prepared task through an authorized agent
session, with ownership confined to its `workspace/`.

```sh
python3 ai.py eval list
mkdir -p build/evaluation-runs
python3 ai.py eval prepare --case documentation-filter-cli \
  --output build/evaluation-runs/docs-candidate \
  --model 'exact model/version or explicit unavailable label' \
  --condition candidate --settings 'reasoning, tools and budget'
# Complete the prepared task in workspace/, then run:
python3 ai.py eval check --run build/evaluation-runs/docs-candidate
# Inspect the actual README against task.md, rubric.md and sources before recording:
python3 ai.py eval review --run build/evaluation-runs/docs-candidate \
  --reviewer 'actual reviewer name' --kind agent --decision accepted \
  --evidence README.md --rationale 'Specific inspected evidence and material limits'
python3 ai.py eval report --run build/evaluation-runs/docs-candidate
```

The installed `skill-catalog/scripts/harness.py` wrapper exposes the same commands.
Foundation exports include the engine and evaluation fixtures, so they can move
without the original checkout. Use `--harness-source /absolute/snapshot` to record
retained baseline guidance. Model/settings, reviewer identity and source fingerprints
are recorded labels, not runtime attestation.

`check` executes copied local verifier and candidate code with normal host
permissions, a 1–60 second limit and bounded output. It validates control files,
immutable fixture inputs and consistent results. Candidate/run changes invalidate
recorded checks and reviews. Changed case definitions require fresh preparation.
An artifact replay after a verifier improvement is not a new independent trial.
These editable local records are not a security boundary or proof of human approval.

The frontend, documentation and research cases require separate qualitative review.
The frontend verifier can check image format/dimensions, labeled controls and a
review note; it cannot judge composition or prove interactions. Inspect every
required image and exercise the behaviors. Documentation needs source comparison
and executed examples; research needs claim-to-source review. Engineering and
integration checks establish only their exercised local contracts.

## Pilot observations on 2026-10-07

Four nonvisual tasks were completed using retained baseline guidance and four
using a frozen candidate skill snapshot. The candidate also completed one
fictional static frontend refinement, with actual browser interaction checks and
light/dark desktop/narrow captures. This was not an independent baseline UI model
trial. The visual comparison is against the seeded interface.

After integrity and verifier refinements, all nine completed outputs were copied
into freshly prepared runs and checked against the final cases. Baseline and
candidate nonvisual outputs both passed; this pilot demonstrates a usable local
workflow, **not a measured quality gain**. Candidate documentation, research and
frontend artifacts received independent agent review. Baseline documentation and
research outputs have automated checks only and retain pending review.

Review found actual failures before acceptance: an inadequate dark-hover contrast
token, misleading PNG filenames containing JPEG bytes, and two clipped PNGs that
passed dimension checks. The hover token was corrected; the browser captures were
recreated and visually reviewed. Source hashes and screenshots alone would have
missed part of this failure. Agent review is not human acceptance.

| Exercise | Final automated evidence | Review scope |
| --- | --- | --- |
| Frontend hierarchy | 4 checks; four actual browser PNGs at 1280×900 and 375×900 | Independent image/source review; primary exercised locale, edit, Save, theme and Tab focus |
| Ledger repair | 5 checks in each condition | Refund sign, empty/mixed totals, integer precision and invalid inputs |
| CLI documentation | 4 checks in each condition | Candidate README reviewed; examples, no-match and invalid limit executed |
| HTTP integration | 6 checks in each condition | Actual loopback pagination, auth, confirmed/uncertain/refused writes and failed later page |
| Cache research | 3 checks in each condition | Candidate source interpretation reviewed; discriminating live check remains proposed |

The runtime exposed inherited agents but no exact model/version identifier.
Selection/read notes are self-reported; there were no normalized timing/cost
measurements or repeated trials. Public synthetic cases are development fixtures,
not held-out evaluation. No product repository was edited, no real provider was
contacted, and no product's motion or production behavior was certified.
[Recorded checks and limits](evidence/quality-harness-2026-10-07.json).

## Apply it to an existing project

Use the current harness checkout and inspect its version before registration.
Global links follow the source; managed global and project copies need deliberate
sync. Persist missing specialists using `project add`, then `project sync` and
`project doctor`. Keep existing selections and project owners. Registration does
not authorize dependencies, deployments or accounts.

A handoff prompt for an authorized project task:

```text
You are already inside the target project. Work on <specific requested outcome>
using the current personal harness at <harness path>. For a new or changed
environment, run runtime doctor --project <current project directory> --json;
check relevant device, tools, client/auth and browser/renderer readiness through
the project's workflow. Start local services only through that workflow; use
native browser tools or an explicit local URL/capture to inspect the result.
Reuse readiness evidence while relevant inputs remain unchanged.
Inspect project instructions, .ai/project.json, stack,
existing implementation and accepted requirements. Use skill-catalog task routing
and affected delivery standards; read the selected installed skills and applicable
references. Persist justified missing capabilities through project add, sync and
doctor; do not load the whole catalog. Preserve unrelated changes and project owners.

Implement the requested result and refine actual observed defects. For interface
work, preserve brand and behavior, inspect comparable before/after renders at the
affected widths/themes/states, and exercise controls, focus and async behavior.
For engineering/integration, test affected contracts and failure/replay boundaries.
For documentation, execute examples and verify claims. For research, check source
applicability and counterevidence. Complete project gates and report delivered
changes, actual evidence and remaining limits. Do not treat registration or a
passing build as qualitative acceptance. Keep production actions within explicit
authorization.
```

## Focused readiness and routing update

The focused readiness/routing update adds optional project prerequisites and
local response/capture to the existing `runtime doctor`, sharpens six authored
skill descriptions and makes result verification explicit in the existing
delivery standards. It adds no dependency, service registry or MCP configuration
layer. [Usage and limits](../setup/README.md#project-readiness) distinguish tool
availability, response, capture, build and visual acceptance.

Five read-only selection probes chose distinct relevant workflows for visual
polish, asynchronous behavior, technical writing, UI motion and SDK research.
Trixpo's prerequisite probe and a capture of its pre-existing static Storybook
were exercised without changing its sources. The inspected capture showed the
Storybook shell with its content area still loading; it is renderer evidence,
not a completed component review. Desktop Chrome timed out on this device;
the already-installed Playwright Chromium headless shell completed the capture
through `--browser`. This does not establish better Trixpo design or general
output improvement. [Recorded evidence](evidence/project-readiness-2026-10-07.json).

## Next evidence needed for broad confidence

Select authorized real tasks before tuning the guidance: a representative project
screen with realistic content/states, an integration failure, a public contract
change, a source-driven guide and a contested research decision. Retain a baseline,
fix model/settings/tool access and budgets, repeat both conditions and inspect
outputs with domain rubrics and owner review. Keep some tasks held out. Measure
failure categories and effort as well as task success; resolve disagreements
against evidence. Preserve useful failures as regressions.

Trixpo is a suitable future project evaluation after deliberate skill sync, but
this update does not claim improved Trixpo output. Pixel fidelity needs an accepted
reference, real content/assets and rendered comparison for that particular task.
Production readiness also depends on the affected project's runtime, data,
authorization, accessibility, deployment and release evidence.
