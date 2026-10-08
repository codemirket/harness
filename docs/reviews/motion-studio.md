# Page transitions and advanced motion

Two authored project specialists extend the existing motion and vector coverage:

- [`page-transitions`](../../skills/page-transitions/SKILL.md) leads route changes,
  shared-element continuity, SPA/multi-page integration, real DOM commitment,
  history, focus, scroll, request ownership and graceful visual failure.
- [`advanced-motion`](../../skills/advanced-motion/SKILL.md) leads coordinated DOM
  timelines, interruption-aware FLIP/layout changes, transform ownership, CSS 3D,
  scroll sequences, responsive rebuilding and scene disposal.

The `motion-studio` profile includes these plus `motion-design`, `svg-creation`
and `svg-animation`. Ordinary control feedback, navigation, complex DOM scenes
and vector choreography have different leads. The global selector routes by the
requested outcome and relevant craft references; the profile makes the workflows
available without requiring agents to read every member on every task.

Global defaults remain at 14. Existing `svg-studio` selections retain their two
skills. The existing `design-studio` export bundle includes the broader motion
profile, with duplicate selections deduplicated. No library, framework, renderer,
MCP server, hook, account or configuration sidecar is added. Agents use native APIs
or the project's existing runtime and follow its dependency rules when a concrete
need justifies an addition.

## Apply to an existing project

From the harness checkout:

```sh
python3 ai.py project add --project /absolute/project --profile motion-studio
python3 ai.py project sync --project /absolute/project
python3 ai.py project doctor --project /absolute/project
```

For an unregistered project, use `project init --project /absolute/project
--profile motion-studio` first; select `--target both` only when Claude Code also
needs the project copies. Preserve existing targets and selections. A fresh
session/client discovery refresh may be needed to expose newly installed skills.
Updating the harness alone does not select specialists in existing projects.

For a new or changed device/environment, use `runtime doctor --project
/absolute/project --json` and the project's own build/browser checks. A registered
workflow does not supply browser support, a compatible router or physical device
coverage. An optional capture must be inspected with actual interactions.

## Craft and integration boundaries

Navigation work starts with usable settled screens and a truthful data flow.
Shared identity, hierarchy and reading position determine the transition. A
skipped native capture is separate from a failed application update and must not
replay a committed mutation. Async work needs the router's ownership guard;
skipping visual animation does not cancel a request or callback. Focus, titles,
announcements and scroll restoration are navigation behavior.

Complex scenes start with a focal action, supporting beats and a clear settled
pose. Independent transform owners, compatible coordinate spaces and a common
clock prevent effects from fighting. The reference's FLIP example explicitly
limits itself to stable shells, translation and synchronous layout. CSS 3D
topology, flattening and scroll ranges require actual target-browser checks;
these are not a full WebGL, 3D asset or video production pipeline.

Reduced motion, interruptions and lifecycle belong in the design from the start.
Agents must render, inspect normal playback and important intermediate beats,
exercise controls and refine visible defects. Source validity, property names,
registration and screenshots cannot certify aesthetics or performance.

## Development example

The retained [Paper room study](../examples/motion-studio/index.html) is one
editable editorial example: a collection/detail transition carries the closed
folio into its new placement, then an optional five-track sequence opens it.
Travel, tilt, hinge, inner paper and annotation have independent owners on a
common clock. Replay, pause/resume, retargeting and immediate reduced-motion
controls exercise the lifecycle. It is original study material, not a required
brand, design system, router or animation controller for other projects.

Serve it with an existing Node runtime from the harness checkout:

```sh
PAPER_ROOM_PORT=0 node docs/examples/motion-studio/server.cjs
```

Open the printed loopback URL. The small developer server supplies the two JSON
entries and direct-route fallback; no dependencies are installed. Opening the
HTML as a file does not exercise its actual HTTP/data/navigation boundary. The
server is a local preview, not a production hosting implementation.

Visual iteration corrected a separated backing during shared capture by naming
an outer flat presentation wrapper around the closed object's self-contained
camera/3D subtree. Navigation review corrected stale handles, aborted-loading
notice cleanup and retry history mode. Narrow placement was inspected separately
from absence of page overflow. The evidence records actual controls, ownership,
history, reduced motion and rendered review instead of inferring quality from
skill installation.

## Source and evidence

The new workflows and mechanical examples are original authored guidance. Current
primary specifications and implementation guidance are linked in
[navigation craft](../../skills/page-transitions/references/navigation-craft.md)
and [scene mechanics](../../skills/advanced-motion/references/scene-mechanics.md).
They establish technical basis, with project versions and target support checked
separately. No upstream animation package or artwork was copied.

[The verification record](../evidence/motion-studio-2026-10-08.json) separates
catalog/installation checks, actual reference mechanics, a bounded agent exercise
and independent rendered review. The exercise is a development sample, not a
held-out comparison, measured quality gain or human acceptance. Browser evidence
applies to the exercised renderer; other engines, frameworks and physical devices
need their own relevant checks.
