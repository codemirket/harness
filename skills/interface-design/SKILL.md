---
name: interface-design
description: Build and refine usable product screens and public pages with deliberate visual direction, real content, working states and inspected browser evidence. Use for UI design, redesign, visual polish and interaction work. Skip backend-only changes.
---

# Interface design

Make the screen work, look at it, and improve the specific things that are wrong.
The deliverable is the working interface, not a declaration of design principles.

## Before composing

Identify the person, their immediate task and the important action. Inspect the
existing screen, components, tokens, content and target devices. Preserve the
project's system unless the requested change includes a new direction.

For substantial visual work, inspect a small number of useful references. Name
what each contributes: navigation density, type hierarchy, grouping, material,
interaction, or an asset treatment. Do not blend whole design systems. A product
library might borrow Primer's predictable actions and Carbon's visible filter
state while keeping its own palette. Apple HIG is platform guidance, not a reason
to imitate macOS in every website. User references take priority over a fashionable
style. Read only the relevant component/pattern guidance.

Use actual copy and believable data shapes. Label fictional sample content. Keep
claims and customer evidence supplied or verified. If the direction is uncertain
and costly to change, show a small coded specimen before extending it. Routine
repairs do not need speculative alternatives or a permission ceremony.

## Build a coherent result

- Put the primary task first. Group related actions and demote supporting detail.
  Use weight, space, contrast and alignment together; do not make everything large.
- Choose type roles, spacing, surfaces and accent usage that fit this product.
  Use existing tokens. There are no universally banned colors, layouts or fonts.
- Use native controls and the project's accessible primitives. A new visual
  treatment does not justify rewriting focus, keyboard and popup behavior.
- Implement real state transitions: filters change results; save controls change
  saved state; dialogs open, close and restore focus. Include empty/error/loading
  states when the real data contract needs them, not as decorative screenshots.
- Choose assets deliberately. Use source SVG for editable icons, marks, diagrams
  and vector illustration; use supplied/licensed/generated raster when the work
  needs photography or painterly imagery. Inspect the asset in its composition.
- Let content wrap and reflow intentionally. Test long labels, narrow containers,
  touch targets, visible focus and enlargement. Do not hide overflow to conceal
  a broken layout or make required actions hover-only.

## Motion that survives use

Name the purpose: feedback, continuity, state change, or explanation. Reuse the
project's timing conventions. CSS or WAAPI may be enough; springs, layout changes
or timelines may justify the existing motion library. Do not install a library
for a simple fade. Test reversal, rapid repeated input and exit while entering.

Provide a reduced-motion path that preserves meaning. Inspect normal-speed
playback as well as intermediate frames. Performance claims require measurement;
`transform`, a spring setting or a library name alone does not prove smoothness.

## Inspect and critique

Run the real screen and exercise the changed task. Compare matching content,
viewport, theme and state with the baseline/reference. Inspect desktop and narrow
layouts, then repair observed clipping, weak hierarchy, unreadable text, missing
assets and broken behavior. A screenshot capture or passing build is not visual
acceptance.

Use [the workbench recipe](references/workbench.md) for reproducible browser
scenarios, motion frames and the working Fieldwork example. Keep diagnostic tools
outside the product interface. For large changes, obtain an independent review of
the rendered result when a reviewer is available. Ask for located defects and
specific repairs, separating functional defects from preferences. Otherwise do a
fresh inspection and state that independent review was unavailable.

Show the actual result and relevant captures. State what was exercised and what
remains unverified. Human approval of visual direction and feel is its own result;
record accepted, revise, or not reviewed. Do not turn automated checks into it.

For deeper craft questions, use the relevant existing references:
[application composition](references/application-composition.md),
[public pages](references/public-page-composition.md), and
[art direction and critique](references/art-direction-and-critique.md).
