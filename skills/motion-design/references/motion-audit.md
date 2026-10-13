# Contextual motion audit

Use for a requested motion review or a pattern affecting several interactions.
Keep findings in the project's existing issue, review or task format. A compact
note may suffice; an HTML audit or before/after specimen is optional. Implement
requested corrections within the authorized scope, then inspect the real result.

## Inventory the interaction, not only the animation

For each affected surface, record the **trigger → state change → frequency →
existing behavior → observed problem**, with component/style/controller locations
and a reproducible state. Include input method, viewport, motion preference and
relevant data. Inspect source owners and their actual mounted consumers; a search
hit in an unused component is not evidence of the running interface.

Compare three concerns in context: restraint (does this frequent action need
motion?), polish (are origin, interruption and settling coherent?), and expression
(does this moment carry a deliberate idea?). They are design concerns, not votes
or simulated approval by named designers. A conditional render without animation
is an inspection opportunity, not inherently a defect. Instant change, opacity-only
feedback and linear travel can all be appropriate.

Look across components for repeated blur entrances, hover scales, staggers and
looping emphasis. Ask whether they compete, repeat on every revisit, obscure the
primary task or imply a status the application does not have. Counts help find
patterns; they do not assign severity. Keep an intentional shared convention when
it supports orientation. Preserve the existing brand and semantic controls.

A frequent command-palette search is a useful counterexample: if every keystroke
replays a result-list stagger, the moving targets can hinder scanning. Compare
identical queries and results with immediate updates. Removing the stagger can be
the right correction; do not replace it with another effect just to show animation.
Keyboard focus and result identity must still follow the actual selection.

## Make each finding reproducible and falsifiable

Give the source location, trigger/state, observed effect and consequence, proposed
correction, and evidence that would disprove it. Distinguish a violated binding
project contract from a usability defect or a design proposal. Reopen the source
and repeat the interaction before finalizing the claim. A stale selector, different
theme, disabled flag or wrong content fixture can invalidate the finding.

For example: “In `SearchResults`, each accepted query reapplies the entrance class
at the mounted list owner; replaying A → AB while arrowing selection moves the
focused row. Keep rows stable and remove the entrance on subsequent result updates.”
The handoff needs the actual file/line, data fixture, ownership/cancellation rule,
and verification steps; this fictional example is not an application finding.

If retaining a presence library already in the project, verify direct children
have stable keys for the identity actually entering/leaving: e.g. `key="copy"` and
`key="confirmed"` for distinct icon states, entity IDs for records, not array indices
or a new random key on each render. Keep accessible status separate from visual
exit/reentry; library presence is not proof that exit handling is needed or correct.

## Inspect costs without property or API bans

Profile the actual interaction before making a performance claim. Use the measured
layout, paint, compositing, main-thread and memory costs to choose a correction.
Batch geometry reads and writes into deliberate phases; invalidate measurements
when layout changes instead of repeatedly alternating reads/writes for each node.
FLIP necessarily measures before and after a layout commit; that bounded read is
not the same as uncontrolled layout thrashing.

A transform can avoid changing layout but does not reproduce a width animation's
sibling reflow, text wrapping or hit geometry. Choose actual size animation,
layout compensation, a clipped visual, or no animation from the required behavior;
then measure and inspect the choice. Prefer existing supported APIs. CSS variables,
scroll handlers, view transitions, blur and layout properties are not categorically
wrong. No fixed blur radius or element count proves acceptable cost. Scope temporary
`will-change` hints to measured needs and release them after use; layer promotion
and compositor execution must be observed, not inferred from a declaration.

## Optional motion specimens

Hold content, viewport, theme, fixture and unrelated properties constant. Show the
current and proposed treatment at normal speed; label approximations and separately
label slow motion. Make the demonstration local and free of real service calls or
writes. A specimen isolates a proposed relationship; it does not prove the actual
component, its keyboard behavior or its application lifecycle was fixed.

[Timing specimen](../assets/timing-specimen.html) is an original, dependency-free
example of a finite **180ms** decorative response with manual replay, pause/resume
and a visible static state. Its optional **0.25×** inspection mode takes **720ms**
of wall time when unpaused; the animation duration remains 180ms. Rest is indefinite
and ends only on explicit replay. There is no automatic looping interval hidden in
the duration label. Movement over 55% of a 3-second CSS cycle would take **1,650ms**;
labelling such playback “180ms” would be incorrect unless explicitly presented as
slow motion with its real scale stated.

Copy the installed asset into a project-owned example directory before adapting
it. Open it using an existing local preview server/browser; it needs no external
assets or package installation. The included asset license covers the original
example. Scope selectors and controllers to each specimen root instead of mixing
selector naming contracts. Keep normal semantic buttons and visible keyboard focus.
Use the project's real reduced-motion path at load and during mounted preference
changes; cancelling an effect must reveal the chosen readable resting state.

Check real playback, important intermediate frames, replay/pause, rapid repeated
input, removal and responsive geometry. Re-run the same scenario in the actual app
after correction, including stale completion and relevant navigation/focus paths.
Build success, screenshots and a “smoothness” score do not replace this evidence.
Human visual acceptance and real performance measurement remain separate.

## Sources and adaptation

- Kyle Zantos, [design-motion-principles](https://github.com/kylezantos/design-motion-principles/tree/4a9ca879f24a361f4dca4174fe2da0f67b5ddee3): contextual review, cross-component pattern search and source-located specimens. MIT, © 2026 Kyle Zantos. This adapts the repository author's synthesis, not endorsement by the named designers. Mandatory designer weighting, branded reports, numerical severity gates and upstream templates are omitted.
- ibelick, [fixing-motion-performance](https://github.com/ibelick/ui-skills/blob/9a0cb1e610e3a1a4f0f2723bf2e7680ac1b89cf5/skills/fixing-motion-performance/SKILL.md): secondary input for measured geometry/layer review. Property/API bans and the nonequivalent width-to-transform shortcut are corrected above. Only this pinned local source is used; the remote registry is not covered.
- [Web Animations](https://www.w3.org/TR/web-animations-1/) defines duration, playback rate and playback controls. The specimen is authored for Mirket rather than copied from the candidate report template.
