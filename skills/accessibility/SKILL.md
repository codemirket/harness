---
name: accessibility
description: "Implement or review interaction and visual access using criterion-linked keyboard, focus, labels, reflow and motion evidence. Skip unrelated code changes without an access boundary."
---

# Accessibility

Identify the affected task, users, platform and requested conformance target. Inspect actual rendered behavior and semantic structure; use the admitted Playwright MCP peer with an isolated profile, or the project’s installed browser test tool for web checks. Consult the applicable WCAG criteria or platform guidance rather than copying a generic checklist into the result.

Exercise keyboard navigation, visible/unobscured focus, accessible names and relationships, error identification, reading order, zoom/reflow and reduced motion where relevant. Include content pressure and state transitions such as a modal opening, a validation error or dynamic status. Check contrast against the real rendered background. Automated scans can locate candidates but cannot establish complete conformance.

Repair the owning semantics or interaction rather than adding labels that hide a broken control. Recheck assistive-technology behavior when available and report the exact combinations observed. Keep untested modes explicit.

Return criterion-linked findings or changes, reproduction conditions and manual/automated evidence. Stop when the affected access requirements pass within the agreed surface, or report the concrete unavailable assistive technology or capability. Do not claim universal accessibility from a clean scanner report.
