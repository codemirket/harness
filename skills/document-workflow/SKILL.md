---
name: document-workflow
description: Write or revise repository documentation, API guides, ADRs and substantial document artifacts. Verify facts, commands, examples and editorial intent with the relevant project or format workflow. Skip simple text corrections.
---

# Document workflow

Identify the authoritative input, intended reader, requested edit, and final format.
For document artifacts, use the installed format-specific skill or native application
for the actual edit. A file operation and a change to a live shared document need
different tooling; choose from capabilities actually available in this session.

For repository documentation, API guides or ADRs, use `skill-catalog`'s task
routing to find the technical documentation workflow and preserve the project's
owning documents. For DOCX, PDF, spreadsheet and slide artifacts, select the
available format-specific skill; portable office guidance is a fallback when
native capabilities are absent. These routes serve different deliverables.

For substantial repository/API documentation, read
[technical documentation](references/technical-documentation.md) to trace claims
to the running interface and verify runnable examples. Keep the existing README,
API reference, ADRs, and their owners authoritative; do not introduce a competing
documentation set merely to follow an example.

## Preserve meaning while editing

- Separate author-supplied facts from proposed language. Keep names, dates, units,
  obligations, and numerical assumptions traceable to their sources.
- Establish the edit boundary: wording, structure, visual styling, calculations,
  or conversion. A request for one does not silently authorize changes to another.
- Match the existing document's styles, terminology, audience, and template.
  For a new document, organize it around what the reader must understand or decide.
- For consequential revisions, keep a readable change summary or tracked edits
  when the format and request support them. Do not accept earlier revisions or
  remove comments merely to obtain a clean appearance.
- Preserve formulas, links, annotations, and accessibility structure unless their
  modification is part of the task. A rendered preview cannot prove they survived.
- Resolve material missing facts before presenting them as settled. Use clearly
  labeled assumptions when the user authorizes an estimate or scenario.

## Check the deliverable at two levels

**Meaning and structure:** compare the affected content with the source. Check
numbers with their units and periods, cross-references, headings, links, and any
calculation or revision behavior touched by the change. For a decision document,
check that a reader without chat history can identify the decision, evidence,
uncertainty, and next action.

**Presentation and operation:** use the format-specific workflow to reopen,
render, recalculate, or inspect the output as appropriate. Inspect the affected
pages, slides, or sheets for clipping, missing content, unreadable labels, and
broken hierarchy. Broaden the inspection when a template or global style changes.

Report the delivered file or live document, the meaningful changes, and the
checks actually completed. Distinguish a successful save from a verified result.

For repository Markdown, use the project's checker or an approved registered tool
through `mirket tool run`. Check local links, heading structure and fences, then
inspect the actual renderer and exercise changed examples. Structural checks do
not establish prose quality or technical correctness.
