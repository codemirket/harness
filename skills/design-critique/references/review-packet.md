# Review packet

Use this compact packet for a high-risk, matched visual or motion review, a handoff between specialists, or a claim that a repair improved the saved artifact. It is an evidence record, not a design template. Keep it project-owned and task-scoped; do not copy upstream prose, code, assets, fonts or branded examples into the packet.

## Source and context

- **Source/revision:** project path or exact URL, commit/ref or source revision, retrieval date, and applicable license/rights note. Record which reference was actually loaded; a catalog receipt is not proof of use.
- **Intended target:** project, route or file; expected loaded state and task entry point. Record an error/blank page as a failed setup, not as the target.
- **View conditions:** viewport and device class, pixel ratio when material, theme, zoom, locale, content/data fixture, browser/runtime identity, and whether fonts/media were ready.
- **Context pack (only when needed):** annotated task-specific crops with the desired property, source/revision, target token/component, and a do-not-copy/prohibited-copy note; existing product tokens/components and constraints; owned good/bad cases; and a short subject/material/typography/motion rationale.

## Evidence ledger

For each baseline, candidate and final artifact, record the saved path or URL, hash when useful, timestamp, and the evidence kind:

- **Pixel inspection:** an image-capable tool displayed the saved pixels at the target size. Record the view/tool and what was inspected (hierarchy, legibility, clipping, state, or motion frame).
- **Structural check:** source, DOM, dimensions, XML, hash or detector result. It supports only that structural claim and must not be reported as visual inspection.
- **Runtime/playback observation:** the loaded route/state was exercised. For interactive motion, record normal-speed playback or sampled anticipation/hold/settle frames and mobile/reduced-motion observations when those modes exist. For exported video, scrub the normal decoded export when available; an offline deliverable is not automatically a reduced-motion target.

## Findings and repair budget

For each material finding, capture:

1. A stable location/state and the observed symptom.
2. The user consequence (blocked task, mistaken interpretation, reading effort, access loss, or visual defect) and confidence/unknowns.
3. The responsible owner and a frozen expected outcome stated before the repair.
4. The repair, matched before/after evidence, regressions checked, and remaining defect.

Keep one task-level review and repair budget across visual specialists. Preserve the best verified candidate with its source and evidence. A new specimen, delegate or replacement view does not reset the budget. If the target route, asset, runtime or inspection capability is unavailable, record the concrete limit rather than claiming the outcome.

Count each edit followed by a new test/review as a repair cycle. A further edit
after that check reveals another defect consumes another cycle; calling it the
same pass does not restore budget. Preserve the original frozen contract and
append dated scope amendments or overruns rather than rewriting its history.

For motion, name only the timing/easing/spacing/type controls relevant to the sequence and their owner. Retain the first frame, anticipation, hold, settle and final resting state where they matter; interactive mobile/reduced-motion variants should preserve meaning and controls when applicable. Do not infer smoothness, codec support, or visual quality from a successful render, file hash, receipt or model self-report.

## Optional sampled-motion fields

When a packet makes a preview/export parity claim, add one frozen `motion` object
and include its canonical SHA-256 as `contract.motion_sha256`. Keep timing
explicit and unit-bearing: `fps` is a positive rational object with only
`numerator` and `denominator`, and `frame_count` is a positive integer. Name
each sampled `beat` with an integer `frame` in `[0, frame_count)`, its expected
state, and the existing state id. The frozen `contract.states_sha256` binds the
route, viewport and state expectations; `contract.motion_sha256` binds the
sample names and expected beat states.

Motion captures extend the ordinary hashed capture with `beat_id`, the matching
`frame`, and `capture_kind` (`preview` or `decoded_export`). With
`parity_claim: true`, every beat needs both kinds, matched by the same beat and
frame, and each kind must point to a distinct saved artifact path for that
beat. Equal SHA-256 values are allowed because identical pixels can be a
legitimate result; a missing counterpart, reused path or changed expectation is
a refusal. Across distinct states, captures must likewise use distinct saved
artifact paths. A static packet need not include these fields. These records
distinguish correspondence and file integrity only: they do not certify that
pixels or playback were visually inspected, and they cannot prove that an
honestly authored packet was not later rewritten in full.
