---
name: motion-direction
description: "Define a motion language or choreography when purpose, sequence, timing, interruption and reduced motion need a coherent specification. Skip implementing an already-specified isolated transition."
---

# Motion direction

Identify what motion should communicate: state change, continuity, hierarchy, feedback or authored narrative. Inspect the static composition and content before adding movement. Define the viewer's focus and the relationship between the first frame, key beats and final resting state.

When quality is disappointing, separate static art direction, motion staging and implementation faults. Compare a still before changing timing; hold geometry fixed when comparing choreography. A technically valid SVG can still be generic, and a successful export can hide a dropped transform behind a moving neighbour. A reference's beat sequence must fit the actual rig's layers and space; more motion is not evidence of better direction.

Specify trigger, sequence, duration, delay, easing, overlap, repeat and interruption behavior in terms production can implement. Use contrast and restraint to preserve reading/action priority. Design reduced-motion alternatives that retain meaning and access, not merely a universal zero-duration switch.

Give each major beat a job and a resting state. Decide which element leads attention, which follows and when reading or input becomes possible. Distinguish a response to input from a sequence the viewer watches. Feedback should follow the user's current action; an authored sequence can use anticipation and pauses when they support comprehension. Repeated identical entrances can flatten hierarchy even when each transition looks polished alone.

Describe timing in milliseconds or seconds and anchor beats to events or a timeline. Define the relevant edges: enter, hold, exit, reverse, cancel and finish. Choose acceleration, overshoot or a spring because it communicates the intended weight or relationship. Keep the same visual property under one animation owner and ensure the logical state is clear after interruption. For exported work, reconcile timing with the requested frame rate and caption/audio beats.

Create a small playable specimen or timed storyboard with the actual type/assets. Inspect pacing at normal speed and representative frames; test whether the sequence remains understandable when interrupted, replayed, resized or reduced. Use qualified runtime capabilities for any Mirket playback/export claim.

For authored video source, including a handoff without an immediate export, use the selected `video-production` skill's pinned authoring contract before construction. Direction alone does not define composition-root or caption wire formats. Report a missing producer contract instead of guessing those fields.

When the deliverable is editable SVG/DOM motion or needs deterministic handoff, keep a small motion contract: stable element IDs, one owner per animated property, explicit base-versus-keyframe values, pivot/origin, timing units, trigger/reverse/cancel behavior, target context and final resting state. Use the project's source adapter (HyperFrames where applicable); this contract does not create a second renderer or preview API. Read [editable-motion.md](references/editable-motion.md) for the compact contract and original native-animation examples with externally controlled `Animation.currentTime`.

Expose only the named timing, easing, spacing and type controls that affect this sequence, with one owner for each. For an interactive target, review anticipation, hold and settle at sampled frames and recheck the same beats on mobile and in reduced-motion mode when those modes exist. For an exported video, scrub the normal decoded export when available; do not invent a reduced-motion requirement for an offline deliverable. A successful render or hash is not playback evidence.

For a finite sequence with a plausible midpoint or transition defect, read [numeric motion observations](references/motion-observations.md) and use the repository checker only for source-bound geometry relations. It can fail a midpoint-only defect, hold/settle violation, containment failure or overlap; its passing receipt is not decoding, pixel inspection or aesthetic judgment.

For a multi-beat sequence, read [storyboard and transition direction](references/storyboard-and-transitions.md) before implementation. Use its compact shot card when it clarifies focal subject, setup/payoff, reading holds, motivated camera movement or continuity between scenes. It is a decision aid, not a requirement for three directions, fixed quotas, hard-cut bans or extra approvals.

A storyboard specifies a sequence but cannot prove its feel at playback speed. Retain that distinction if playback is unavailable. Check reading time, focus continuity, loop seams and whether motion hides an actionable or task-critical element. A reduced-motion version may use immediate state changes, static composition or a restrained transition; preserve the same information and controls.

For a concrete unresolved choreography question, consult only the relevant portion of the approved pinned [LottieFiles motion](https://github.com/LottieFiles/motion-design-skill/tree/f9a8a041b85185ee4881b3471d3415e939aac772) or [Kyle motion principles](https://github.com/kylezantos/design-motion-principles/tree/4a9ca879f24a361f4dca4174fe2da0f67b5ddee3) source. These are comparison locators, not installed players, adopted prose or authority to add a dependency.

Deliver the motion rationale, timing/state specification and inspected specimen. Unless the task sets another budget, share 45 minutes and four final repairs across motion production and review, with one target repair. Stop when the choreography is concrete and verified enough for implementation or report missing playback, assets and unresolved defects at the limit. A new sequence does not reset the budget.
