---
name: interface-design
description: Build and refine usable product screens and public pages with deliberate visual direction, real content, working states and inspected browser evidence. Use for UI design, redesign, visual polish and interaction work. Skip backend-only changes.
---

# Interface design

Make the screen work, look at it, and improve the specific things that are wrong.
The deliverable is the working interface, not a declaration of design principles.

## Professional stance

Judge the interface by whether someone can understand the current state, choose
the next action and recover from a mistake. Prioritize task hierarchy, legibility
and coherent state changes. Ask what competes with the main action, which fact
the user needs before committing, and what the screen implies while waiting.
Compose with real content; use a small working specimen to test the riskiest
layout or interaction decision before propagating it across screens.

Challenge product scope when equally prominent actions obscure the user's goal,
brand guidance when it weakens hierarchy, and frontend implementation when its
state model produces an ambiguous screen. Supply a working flow with content,
responsive rules and named interaction states so engineering and QA can inspect
the same behavior. Revise the direction when observed task use, content extremes
or accessibility checks contradict its apparent clarity. Distinguish a located
usability defect from a stylistic preference when reviewing another designer.

For an anti-slop or less generic design request, use the existing
[art direction and critique](references/art-direction-and-critique.md): tie each
visible choice to this brief and repair located defects in the actual render.
Do not replace the product's identity with a checklist of fashionable effects.

When the requested deliverable is an editable design document, use
`graphic-design` and its design-document reference with the optional headless
Penpot tools (shared design files). A static design export does not implement responsive behavior,
accessible controls or a working website.

## Before composing

Identify the person, their immediate task and the important action. Inspect the
existing screen, components, tokens, content and target devices. Preserve the
project's system unless the requested change includes a new direction.

For a revision, name the visible problem and retain a comparable baseline before
editing. Identify approved constraints such as labels, brand assets, information
order and data behavior. Change the responsible layout or component rule, then
check its other affected instances; a local polish request should not silently
rewrite content or alter the flow. Compare the gain and any loss at matching
content, viewport and state rather than judging a new screenshot in isolation.

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

For a new screen, meaningful redesign or design QA, use
[brief to proof](references/brief-to-proof.md) to connect the actual content,
composition, state matrix and final inspection. Reuse the current task record;
the reference does not require another planning document.

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

Name the purpose: feedback, continuity, state change, explanation or a requested
expressive visual narrative. Reuse the project's timing conventions. CSS or WAAPI
may be enough; springs, layout changes
or timelines may justify the existing motion library. Do not install a library
for a simple fade. Test reversal, rapid repeated input and exit while entering.

Choose a lead from the changed result: `motion-design` for control feedback,
`page-transitions` for navigation, `svg-animation` for vector art, or
`advanced-motion` for a coordinated landing-page/scroll/procedural scene. The latter's
art-direction reference connects brand, typography, material and rhythm before
effects are chosen. An ordinary UI repair does not need an advanced scene.

Provide a reduced-motion path that preserves meaning. Inspect normal-speed
playback as well as intermediate frames. Performance claims require measurement;
`transform`, a spring setting or a library name alone does not prove smoothness.

## Inspect and critique

Run the real screen and exercise the changed task. Compare matching content,
viewport, theme and state with the baseline/reference. Inspect desktop and narrow
layouts, then repair observed clipping, weak hierarchy, unreadable text, missing
assets and broken behavior. A screenshot capture or passing build is not visual
acceptance.

Use [browser evidence](references/workbench.md) for reproducible scenarios,
motion frames and inspection criteria. Keep diagnostic tools
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
