---
name: frontend-engineering
description: "Implement component or client-state behavior when rendering, accessibility and browser lifecycle affect correctness. Skip backend-only changes and mechanical CSS edits."
---

# Frontend engineering

Inspect component ownership, state sources, routes, data contracts and existing UI patterns. Separate derived state from persisted/user state. Define loading, empty, error, success and interrupted transitions before wiring the happy path.

Keep asynchronous work bound to the current request/view lifecycle. Reconcile stale results, cancellation, repeated submission and navigation so old responses cannot overwrite current intent. Preserve accessible semantics, keyboard focus and stable input values through re-rendering. Use project dependencies and established state tools.

Exercise the actual component/journey with the admitted Playwright MCP peer with an isolated profile, or the project’s installed browser test tool, supplemented by required project tests. Verify a discriminating case such as out-of-order results, rejected mutation or unmount during loading. Inspect responsive layout and relevant content pressure; a DOM snapshot alone does not prove the interaction works.

Return the behavior change, source and observed browser/state evidence. Stop when affected contracts and project gates pass, or state the specific untested runtime path. Keep visual direction with the design owner when separate from this behavior task.
