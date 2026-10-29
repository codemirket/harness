---
name: browser-qa
description: "Test a live web journey, responsive layout or stateful interaction using rendered evidence and failure cases. Skip backend-only tests or a static source review."
---

# Browser QA

Bind the test to the actual source revision, application build, URL, account/data state and intended journey. Use the admitted Playwright MCP peer with an isolated profile, or the project’s installed browser test tool for rendered inspection; run project-owned browser tests when they are the established reproducible check. Keep personal profiles and test data separate.

Choose representative viewport sizes and the states that can invalidate the journey: empty, loading, error, success, overflow and interrupted/repeated action when applicable. Exercise keyboard access, visible focus, labels and reduced-motion behavior alongside pointer interaction. Inspect captures for clipping, wrapping, missing assets and hierarchy; screenshots alone do not establish behavior.

Use a discriminating failure such as rejected input, network failure, expired session or repeat submission. Verify the user can understand and recover from it without losing unrelated work. Keep live external effects within authorization and use test fixtures where the project provides them.

Return observed results, reproducible defects, captures bound to viewports/states and untested boundaries. Recheck evidence affected by repairs. Stop after the required journey and cases pass or the agreed repair budget is reached; report remaining defects and any unavailable client or project browser capability.
