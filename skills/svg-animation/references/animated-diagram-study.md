# Animated diagram reference study

Use this study when motion should explain a relationship in an editable diagram.
It is a small original specimen, not a brand template or a fixed timing scale.
Open [the relay diagram](../assets/relay-diagram/index.html) through the project's
existing local server and inspect it at wide and narrow widths. The HTML contains
two explicitly composed inline SVG layouts; `styles.css` provides the visual
system, and `motion.js` owns one finite clock. The source is under the adjacent
MIT notice. Copy the specimen into project-owned source before adapting it; do
not edit a managed Mirket installation.

## What to study

The brief is a fictional three-stage relationship: Observe → Decide → Act. The
static diagram shows the complete route, direction through stage numbers, and
short descriptions. The wide composition uses two alternating curved edges to
keep the reading path lively; the narrow composition changes to a vertical flow
instead of shrinking the labels to illegibility. Neither layout relies on color
alone for stage identity. The warm paper, technical grid and restrained blue
accent make the focal path distinct from the inactive structure. These are
specimen choices, not a prescribed Mirket palette.

| Beat | Approximate specimen time | Purpose |
| --- | --- | --- |
| Resting route | Before 250 ms | The relationships are already intelligible. |
| First handoff | 250–1030 ms | One packet and trace direct attention from Observe to Decide. |
| Pause | 1030–1230 ms | The first arrival has time to register. |
| Second handoff | 1230–2010 ms | Attention moves from Decide to Act. |
| Settle | After 2010 ms | All information remains; motion ends without a loop. |

The frame update derives packet position from each SVG path's own geometry and
never changes the underlying stage content. Replay cancels the prior frame
owner; a live reduced-motion preference change settles immediately. JavaScript
failure leaves the complete route and labels visible. `window.diagramStudy.seek(ms)`
is a specimen inspection hook for frame capture; watch an ordinary, unpaused
playback before using it. It does not establish smoothness or frame performance.

## Counterexample and critique

Open `index.html?variant=awkward`. It preserves the same static diagram, valid
markup and reduced-motion behavior, but starts both transfers together and
leaves Observe emphasized after the transfer has finished. A checker that only
asks whether paths changed, the page loaded and reduced motion worked will pass
it. At normal speed, the two simultaneous packets compete for the focal point;
the final highlight contradicts the left-to-right narrative. This variant is a
failure probe, never a production recommendation. For a blind comparison, show
only the figure area from each recording with randomized labels; the explanatory
page notes describe the intended sequence and would bias the judgment.

## Transfer to an actual task

1. Write the diagram's factual nodes, relationships, direction and necessary
   labels. Check the static composition at its smallest real placement first.
2. Identify the one relationship that movement should clarify. Sketch opening,
   action, rest and final pose. Adapt the specimen's causal sequencing only when
   it fits that meaning; do not copy its palette, curve shapes or timing blindly.
3. Render the actual integration at desktop and narrow sizes. Watch real-time
   playback, then inspect initial, midpoint, resting and reduced-motion states.
4. Record one located visual observation and repair the editable source. Test
   replay, rapid repeat, no-script fallback and live reduced-motion change.
5. Use independent critique for consequential subjective work. Require a reason
   tied to clarity, hierarchy, rhythm or brand fit; test claims against the actual
   render. Automated checks establish behavior, not visual acceptance.

## Many-to-one website heroes

A product hero may show several sources feeding one destination. Start with the
truth of that relationship and the complete static arrangement. Decide whether
the intended takeaway is **convergence at once**, **distinct contributions**, or
**an ordered process**. Those imply different motion scores; the number of
paths alone does not decide the sequence. Keep the site's actual brand tokens,
copy, responsive geometry and still text rather than adopting this specimen's
palette or topology.

Write a compact score for one representative route: source cue, travel,
destination response and rest. Give each beat a purpose and an approximate time
or progress marker. A response should be distinguishable at the delivered size;
another pulse on an already emphasized destination may be too subtle. When
routes overlap, inspect whether viewers can still tell which source contributed
and when the destination received it. If simultaneity expresses the truth,
compose one clear collective arrival instead of inventing a false order.
When one visual motif recurs across beats, give it the same explanatory job each
time so it links the sequence rather than adding unrelated decoration.

Watch the actual page at normal speed before judging captured frames. At desktop
and narrow placements, sample just before travel, during the busiest overlap,
at arrival, at rest and across the loop seam if the scene repeats. Ask an
independent viewer to identify the progression and arrival without code or a
written explanation of the intended animation. Record the time and element that
support each criticism, repair the editable source, then rewatch the same case.
If playback cannot be viewed, mark pacing unverified; still frames alone cannot
establish its feel.

The semantic-pattern-first approach is informed by [Diagram Design's pinned
diagram guidance](https://github.com/cathrynlavery/diagram-design/blob/3996c1607503ec4bcdb60b018568359d20f71d15/skills/diagram-design/SKILL.md)
and its [animation reference](https://github.com/cathrynlavery/diagram-design/blob/3996c1607503ec4bcdb60b018568359d20f71d15/skills/diagram-design/references/animation.md).
The recurring visual motif and deliberate hold are discussed in [Kingy's
motion-graphics brief](https://kingy.ai/blog/astra-codex-motion-graphics-guide-prompts/).
These are methods adapted to this original example, not copied templates or
evidence that either source improves Mirket output without a transfer test.

For external principles, use the current official [Carbon motion choreography](https://www.carbondesignsystem.com/building-blocks/foundations/motion/choreography),
[Material choreography](https://m1.material.io/motion/choreography.html),
[W3C image alternatives](https://www.w3.org/WAI/tutorials/images/), and
[W3C animation guidance](https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions).
These links are reading references; their images, examples and brand assets are
not bundled here or licensed by this specimen's MIT notice.
