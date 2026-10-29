# Preview and beat correspondence

Use a project-owned preview to locate changes, then verify the actual HyperFrames
export. Preview interaction uses the admitted Playwright MCP peer or installed project browser tests. Mirket's private
Chrome remains reserved for finite video capture. Do not install another
renderer or make preview controls part of the exported film.

## A finite clock with visible units

Record rational FPS, total frame count, and named review beats before editing.
Frame zero is the first output; the last is `frame_count - 1`. Convert an output
frame to milliseconds with `frame * denominator * 1000 / numerator`. A 36-frame,
12-fps film lasts 3 seconds, but its final sampled frame is at 35/12 seconds.
For a requested event at a time between output frames, choose and document the
rounding rule; do not silently substitute seconds for frame indices. Caption
ranges are zero-based and end-exclusive.

| Beat | Frame at 12 fps | Time | Expected visible state |
| --- | --- | --- | --- |
| Start | 0 | 0 s | Initial composition and relevant label |
| Reveal | 6 | 0.5 s | Subject entering; check competing text |
| Hold | 12 | 1 s | Main message legible |
| Last | 35 | 2.9167 s | Intended resting/exit state, no lost caption |

These are sample review points, not prescribed pacing. Add boundaries and
just-before/at/after caption or scene changes when those are material.

## Edit the source owner

Use stable element IDs and named animation rules. Record the animated property,
its base state, overriding keyframes and relevant parent transforms/pivots.
Changing a base value does not change a value still owned by a keyframe.
Resolve competing owners before seeking; moving a layer must not change which
object an edit addresses. Keep text, geometry and design controls editable in
the project source rather than rebuilding them from a rendered frame.

The pinned HyperFrames 0.8.197 runtime discovers native CSS and WAAPI handles.
For a bounded document-timeline specimen, a separate same-origin preview can pause those
handles and set their `currentTime` to the selected frame's milliseconds.
Reuse the authored keyframes; do not independently interpolate replacement
geometry. Fonts and images must finish loading before capture. Reject or
explicitly leave unqualified scroll timelines, CSS transitions, arbitrary
timers, canvas, media synchronization or another timeline engine. A finite
CSS/WAAPI fixture was checked at five matching preview/export frames; that
does not qualify every API or target. Registered Anime uses its separate
pinned contract, not this native-animation preview.

The repository's optional `scripts/create-motion-preview.mjs` generates such a
review page when working in the harness checkout. It is a development utility,
not an installed Mirket tool or a file promised in other projects. A project
with an existing preview should use that instead where it can expose the same
timing and source context.

For a subject whose motion matters, the utility's motion JSON can declare
`"expected_properties": [{"target": "heart", "property": "transform"}]`.
Use stable target IDs and the browser's camelCase keyframe property names.
The preview refuses a declared property absent from the accepted keyframes,
including transforms silently dropped for invalid CSS syntax while a neighbour
or the same target's opacity still animates. Without declarations, intended
property coverage remains untested. This check establishes presence only:
constant values, occlusion, bad timing and weak art direction still require
saved-pixel and playback review.

## Compare corresponding states

Seek forward, backward and back to the same beat; inspect the visible subject,
not only the displayed time. Change one authored control and confirm that only
the intended source owner and output change. Review normal-speed playback
separately: matching still frames cannot establish pacing or smoothness.

Retain saved preview pixels and independently decoded export frames at the
same named frame/state and viewport. Inspect both with pi’s read tool with a vision-capable active model.
Check geometry, type, clipping, caption safe area, loop seam and final/resting
state as applicable. A missing comparison remains untested. Compare a suspected
unmanaged track directly; a moving neighboring element can hide a frozen one.

Freeze beat expectations with the brief, source manifest and rubric. Keep one
repair ledger across specimens and reviewers, preserve initial artifacts, and
recheck affected beats after a repair. The optional repository evidence checker
can bind frame correspondence and hashes; it cannot prove visual equality,
inspect pixels or certify unrecorded defects absent.
