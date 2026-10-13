# Browser evidence you can act on

Start the project's app through its established commands. Prefer the host's
available browser automation tools. If a separate browser CLI is needed, inspect
and explicitly register the approved executable with `mirket tool register`,
then invoke it with `mirket tool run`. Registering a skill supplies no browser,
dependency, authentication or remote publishing permission.

## Exercise real interactions

Use real components and representative content. Cover the intended desktop and
mobile sizes, keyboard navigation, empty states, recovery and reduced motion.
For a search/filter interface, exercise selection, long labels, no results, clear,
preview, dismissal and focus return. Capture meaningful checkpoints and inspect
the page itself. Do not count a screenshot as evidence of an unexercised action.

Watch playback to judge interruption and timing; frames alone do not establish
either. An animation already running at capture time has no precise zero sample.
For exact timeline comparisons, use a development-only seek handle or set time
on the relevant animation engine.

## Optional component comparisons

Use a workbench when comparing existing component variants or states resolves a
real question. Reuse Storybook, a test route or the existing preview system when
it is sufficient. A pan/zoom canvas is an optional viewing aid, not a prerequisite
for UI work. Keep preview code outside the product's component implementation.

Before mounting, isolate the component through its public providers, fixture
services and callback props so reads and writes cannot reach real accounts or
services. Preserve access controls. An iframe alone does not isolate network
effects; omit an unsafe live example and explain the boundary. Static fixture
assets can remain local. Do not use production credentials or real private data.

Choose meaningful supported states from the actual API and task. Compare one
dimension at a time: hold content, fixture data, theme, container and unrelated
props constant. A button-size comparison keeps the same label, icon and variant;
a theme comparison keeps those and the size fixed. Do not enumerate every prop
combination, invent unsupported variants or style fake focus states. Exercise
hover, keyboard focus, pressed state and validation through real input. Record
what a snapshot shows and which interactions it cannot establish.

The preview shell must preserve typing, selection, native keyboard activation,
focus, touch and nested scrolling. Shell shortcuts must not intercept Space,
arrows, Escape or undo intended for the preview. Open dialogs individually and
test their focus containment, dismissal and focus return. Use component public
APIs rather than patching rendered DOM or library internals to fit the workbench.

If the shell transforms its content, check portal geometry independently. Native
portals can render outside the transformed tree; matching the trigger's visual
position does not guarantee matching hit areas or popup scale. Keep the library's
positioning and modal behavior, use a supported container option if appropriate,
and verify theme inheritance, focus and pointer targets at relevant scales and
after resize or nested scrolling. Prefer an untransformed preview or separate
document when the component cannot safely support the transformed host. That
document still needs service isolation before mounting.

For a custom camera, start gestures on shell space rather than inside controls.
Avoid broad event cancellation or disabling native touch behavior on previews.
Handle pointer-count changes, pointer cancellation, lost capture, window blur and
hidden tabs. Release captures and remove listeners, observers, animation frames
and timers on teardown. Batch geometry reads and writes; remeasure after content
expansion or viewport changes. Keep keyboard focus reachable and provide visible,
keyboard-operable reset/fit controls. If camera persistence is useful, validate
stored values and tolerate unavailable storage; resetting it must not erase a
component's draft input.

Start with one real component and a small failure probe: type in an input, open a
portaled popup by keyboard, scroll its nested region, resize the preview, then
remove it while active. Check focus, hit targets, unexpected requests and leftover
work. Add further comparisons only after this slice works. Report limitations
of the actual runtime; an illustrative specimen does not prove the application
was repaired.

## Interpret observations

Keep automated checks, agent visual review and human acceptance distinct. Tie
findings to visible defects: a selected filter disappearing below the mobile
search, overlapping dialog buttons, lost focus or unreadable labels. Repair the
cause and inspect the affected states again. Do not change screenshot baselines
solely to silence a failure.

For substantial tracked work, attach actual captures and behavioral output with
Mirket's task evidence tools. File presence and hashes identify an artifact;
they cannot establish that someone reviewed it or that a design is good.

## Place illustrations in context

Write an asset brief covering subject, composition, palette, material, dimensions
and placement. Specify negative space for text and crop constraints. Use supplied
or licensed assets, an available image tool, or an explicitly approved provider.
Inspect the source at full size, then the actual component at intended widths.
Check crop, text contrast, seams, transparency and unwanted lettering.

Retain editable paths, shapes and text for a vector deliverable. A bitmap inside
an SVG wrapper is not editable vector artwork. Render the actual master through
an available browser or registered renderer and inspect fine details.

Use one relevant visual reference for the surface's purpose. These sources can
inform particular choices; they are not a combined brand recipe:

- [Primer dialogs](https://primer.style/product/components/dialog/) for actions
  and focus return.
- [Carbon filtering](https://carbondesignsystem.com/patterns/filtering/) for
  selection visibility and understandable result changes.
- [Apple motion](https://developer.apple.com/design/human-interface-guidelines/motion)
  for platform interaction and accessibility behavior.
- [Polaris](https://shopify.dev/docs/api/polaris) and
  [Base](https://base.uber.com/) for relevant workflow and component guidance.

## Technique provenance

The optional comparison method adapts
[component-canvas](https://github.com/ibelick/ui-skills/blob/9a0cb1e610e3a1a4f0f2723bf2e7680ac1b89cf5/skills/component-canvas/SKILL.md)
at `9a0cb1e610e3a1a4f0f2723bf2e7680ac1b89cf5`, MIT, Julien Thibeaut;
the notice accompanies installed guidance. Mandatory canvas construction,
cosmetic details and report headings are omitted. The upstream remote registry
is outside this adaptation.
