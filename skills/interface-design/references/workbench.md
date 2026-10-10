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
