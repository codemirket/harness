# Verification record — 9 October 2026

This records what ran on the current macOS host. Generated evidence lives under
ignored `build/`; source and repeatable commands live in Git. Artifact receipts
include hashes. A report's automatic `visual_review: required` field stays intact;
the human-readable review below is separate evidence, not a machine-awarded pass.

## Runnable capabilities

- Browser: public `ai.py workbench browser` invocation against the local specimen.
  Four isolated contexts: desktop 1440×1000 and mobile 390×844, each normal/reduced
  motion. Twenty assertions passed; 24 PNGs produced; no recorded console, page,
  HTTP, transport or overflow findings in this scenario. `build/browser-final/`.
- Office: public `workbench documents demo` generated editable DOCX/PPTX/XLSX,
  two brief pages, three slides and one workbook preview. LibreOffice recalculated
  subtotal 11,600, contingency 1,160 and total 12,760. A separate changed-input
  run (20→21 discovery hours) produced 12,870 with the total formula preserved.
  `build/office-final/`. Termination cleanup changed subsequently; artifact
  generation/layout inputs were unchanged and were not rerendered for that fix.
- Vector: public `workbench vector` preserved the original SVG and rendered
  24/64/256/1024px on light/dark backgrounds plus a contact sheet.
  `build/vector-proof/`.
- Markdown: public `workbench markdown` verified the specimen README's supported
  local link/heading/fence subset. `build/markdown-proof/`. Technical commands in
  that README were run through the same entrypoints with fresh output names.
- Tool discovery: Playwright 1.62.1, sharp 0.35.5, python-docx 1.2.0,
  python-pptx 1.0.2, openpyxl 3.1.5, LibreOfficeDev 26.8 alpha and Poppler 26.05
  were already available. Existing paths are configured in the user's local
  toolchain file, not committed as host assumptions. Chrome 154 rendered the
  public browser scenario. No optional dependencies were installed.

## Actual inspection and repairs

The primary agent and visual reviewer opened desktop library/mobile modal
captures. The visual implementer also exercised filters, Saved/unsave, empty-state
recovery, Escape and focus restoration, rapid close and reduced motion. Fixed
illustration letterboxing and focus fallback after removing the last saved item;
the final automated capture used the repaired source. This does not substitute
for a screen-reader audit or real mobile hardware.

The vector contact sheet has a coherent notebook/botanical silhouette at intended
illustration sizes. Paper lines and leaf detail collapse at 24px; the README states
that a separate simplified icon would be needed. No icon-quality claim is made.

The document authoring peer inspected all six pages individually. The primary
reviewed every page across final and unchanged equivalent outputs. Content,
figures, labels and hierarchy were legible with no observed clipping. Inherited
slide card shadows were removed and the updated deck rerendered. Editable source
and formula checks are separate from these visual observations.

A fresh integration review found and repaired missing `pdfinfo` configuration,
missing examples in self-contained plugin snapshots, and renderer descendant
cleanup on interruption. The final interruption regression launches an actual
renderer/grandchild fixture and verifies that it stops producing output.

## Gates and packaging

The final full suite passed **552 tests** in 51.464 seconds, with no skips.
The log is `build/test-results-complete.txt`.
Tests cover seeded browser assertion failures, console/HTTP/blocked requests,
invalid scenario budgets, vector active/external/duplicate content, broken
Markdown links/anchors/fences, OOXML macro/external/formula errors, missing
renderers, missing expected output and process termination.

Registry generation, shell/Node syntax and Git whitespace checks passed. An
isolated home rehearsal installed both Codex and Claude targets in copy mode,
then `check` passed. A temporary project added SVG/motion plus its foundation;
project sync and doctor passed. No real project manifests were altered.

The `personal-workbench` Codex bundle packages the 14 globals plus SVG/motion,
executable helpers and specimen. A relocation regression runs the exported
launcher from an unrelated directory after removing its source checkout. See
[plugin instructions](../plugins.md) for build/import and duplicate-skill guidance.
The generated bundle's 268 file hashes matched its build lock, and its exported
launcher checked the bundled specimen README from an unrelated working directory.
Plugin manifest/CLI checks are not a test of discovery in a fresh desktop session.

## Remaining limits

The existing global skill links resolve to these authored sources on this Mac.
The revised native target configuration has pending changes; applying it requires
closing Codex. It was rehearsed in an isolated home, not applied to the active app.
No client restart, plugin activation, new MCP connection, raster image-provider
connection, or Chat/Cowork upload is claimed. Chat/Cowork cannot inherit local
shell tools merely by uploading a skill.

No blinded equal-budget old-vs-new agent trial was run, no human aesthetic
acceptance was obtained, and no performance/a11y/Office-compatibility certification
is claimed. A successful fixture proves its exercised path. The workbench is
usable evidence-producing machinery; production work retains its own tests,
design requirements and acceptance decisions.
