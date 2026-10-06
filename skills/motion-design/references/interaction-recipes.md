# Interaction motion recipes

Use the sections matching the changed interaction. Keep the project's primitives,
tokens, supported platforms and actual state/API contracts. These recipes describe
choices to tune and exercise; they do not authorize a library, new feature or
different business behavior.

## Specify the transition before choosing an effect

For each changed interaction, identify its trigger, semantic state change, visual
feedback and interruption behavior. Distinguish feedback that confirms an action
from movement that explains a spatial relationship. Focus, keyboard selection and
error information must be available when needed; animation completion must not
gate them.

Reuse existing timing first. Without project tokens, useful initial tuning ranges
are roughly 80–150ms for a press/color response, 150–250ms for a small menu, and
200–350ms for a dialog/disclosure. Larger travel or a quieter brand may differ.
Exits often feel better shorter than entrances. These are candidate ranges, not
pass/fail thresholds; repeated actions should not feel delayed.

Choose curves by the motion: deceleration helps an element settle into place;
acceleration can communicate departure; a balanced curve can fit a state change.
A spring can preserve continuity in a direct manipulation when the installed API
supports position/velocity interruption. Reuse existing curves and inspect the
result. No easing name or CSS property proves smooth performance.

## Controls and immediate feedback

**Button press.** Keep geometry stable and focus visible. A small color change,
shadow change or subtle press displacement can distinguish pointer-down from hover.
Choose the effect that fits the existing control. On release or pointer cancellation,
return from the current visual state; do not wait for an old animation to finish.
Keyboard activation should expose the actual action/state immediately.

**Toggle/check.** Commit the logical change at the normal control event. A moving
thumb or check appearance can connect old and new positions, but rapid toggles must
retarget toward the latest state. Keep the state recognizable without color or
movement. If a server accepts/rejects the change, use the existing pending and
reconciliation contract rather than assuming every toggle can update optimistically.

**Async action.** Put feedback where the action occurs: a button's pending label,
an inline status or a progress region. Keep label width/neighbor positions stable
when possible. Avoid a spinner flash on near-instant work; an existing delayed
indicator can help, but do not prolong a completed action to display it. Prevent
duplicate writes using the established behavior, and distinguish a failed response
from an unknown server outcome. Motion cannot resolve that uncertainty.

**Copy feedback.** After confirmed clipboard success, a brief local label/icon change
can suffice. Expose failure if the operation rejects. Clear any restore timer on
unmount; a repeated success should not be immediately erased by an earlier timer.
Announce meaningful feedback through the existing accessible status mechanism.

**Validation.** Make the error message and field association available promptly.
Use subtle optional emphasis on the relevant region; a shake or color animation
must not be the sole signal. Preserve entered text, stable input geometry and the
form's validation timing. Do not hide an error while an exit effect is playing.

## Menus, popovers, tooltips and dialogs

**Anchored menu/popover.** A short opacity change and small movement/scale from its
anchor can explain where it came from. Set the visual origin from actual placement;
an edge-flipped menu should not animate from the wrong side. Keep the project's
keyboard, focus, dismissal and positioning behavior. Reopening during exit should
reverse from the current visual state and cancel stale exit cleanup.

**Tooltip.** Hover may need a short delay to avoid appearing during casual pointer
travel; keyboard focus should expose useful information promptly. Use the
project's current delay policy. Cancel a scheduled show when the target loses the
trigger. Honor hoverable/persistent/dismissible behavior where the tooltip contains
additional content. Required labels/actions must not live only in a tooltip.

**Dialog/sheet.** Open from an explicit state change using the existing accessible
primitive. A restrained crossfade, short displacement or small scale can mark entry;
directional travel can help a sheet relate to its edge. Coordinate backdrop and
content so neither leaves an unexplained blocking surface. Use focus placement,
background interaction control, Escape/dismiss and return focus from the primitive's
contract, independent of a decorative animation timer.

During close, the visual layer may finish an exit only while the application can
keep its semantic/focus state correct. On a rapid reopen, invalidate the old close
completion. On unmount/navigation, release listeners/timers/background locks through
the component lifecycle. Do not let a stale completion hide a newly opened dialog
or move focus back to a trigger after the user has moved elsewhere.

## Disclosure, tabs and changing content

**Accordion/disclosure.** Connect the header's expanded state with the relevant
content. A short size transition can explain expansion when it is worth the layout
work; instant expansion can be better for frequent use. Use a supported technique
already in the project and inspect layout/performance instead of substituting a
fixed oversized maximum height. Retarget from the current height on reversal and
handle late-loading text/media. Hidden content must not remain an invisible focus
target. Preserve text legibility and reduced-motion behavior.

**Tabs.** Move the selection indicator or crossfade content when that clarifies the
change. Keyboard selection/focus and the active panel follow the established tab
model immediately. Avoid queuing several transitions during rapid navigation;
show the latest selected panel. A panel-height change should not unexpectedly move
the user's focused control or scroll position. An instant switch is appropriate
when the panels are information the user is comparing rapidly.

**Search/filter results.** Keep input and usable results responsive while the actual
request is pending. A short insertion/removal change can show the new result set;
do not animate every row on every keystroke. Protect against stale results through
the data layer. Preserve current focus and the real selection/query semantics.
Announce a settled result state/count rather than every animated intermediate value.

**Number/data change.** Stable numeric alignment or brief emphasis on a changed
value may explain an update better than counting from zero. Keep the confirmed
value and units accessible; a decorative interpolation is not new source data.
Avoid announcing every frame or making a financial/operational value wait for a
counter effect to settle. Show final values immediately under reduced motion.

## List updates and direct manipulation

**Insert/remove/reorder.** Use stable identity so an item moves as itself rather
than inheriting another row's appearance. A brief position transition can explain
where surrounding items went. Keep focus with the affected control or use the
project's fallback when it is removed. Do not replay entrance effects for unchanged
items, and account for virtualization if the project uses it.

Visual exit must not imply that a server deletion succeeded. Follow the actual
mutation/undo contract. A reversible optimistic change needs reconciliation that
cannot undo a newer edit; do not capture a snapshot and blindly restore it after
any later failure. Use the project's engineering workflow for that boundary.

**Drag/swipe.** Direct manipulation should track input without an ornamental lag.
Use the project's axis constraints, thresholds and gesture ownership. Settle from
the current position and velocity if supported; allow re-grab without snapping to
the initial position. Account for pointer cancellation, scrolling conflicts and
unmount. Provide the supported keyboard or non-gesture alternative and preserve
the same action semantics. Check gesture-specific behavior on actual hardware.

## Reduced motion and verification

Choose the reduced-motion treatment for the interaction, not an indiscriminate
duration rule for the whole app:

- Remove unnecessary spatial movement, parallax and scale travel. Use an instant
  state change or a restrained opacity change when useful.
- Retain information about expanded/selected/pending/error states. A loader may
  need a static status/progress alternative rather than an infinite visual loop.
- Preserve the final content/position when an entrance is removed. An element
  initially styled invisible must not stay hidden when its animation is skipped.
- Use the project's preference handling; account for preference changes during
  a mounted session when that is part of the supported behavior.

Inspect actual frames and changed interactions. Useful checks include rapid
repeat/reversal, leave during a scheduled show, close/reopen during exit,
navigation during pending work, keyboard use, narrow container, text enlargement
and reduced motion. Profile when jank or a performance claim is involved. Keep
observed behavior separate from assumptions about compositor or library internals.

## Worked choices

**A repeatedly opened filter popover.** The problem is a hard jump, not a missing
animation library. Use the existing popover and timing tokens for a small anchored
opacity/position transition. Keep filter input focus immediate. Open → close → open
before exit finishes must leave one visible, interactive popover; old exit cleanup
must be invalidated. Under reduced motion, open directly or crossfade without travel.

**An async save button.** The real form prevents duplicate submits and reports server
validation failures. Keep those contracts. Make the pending label stable in width,
show local status, and reveal any error near the relevant field. A successful save
can return to the normal label when the resulting state is visible. Do not add a
celebratory sequence, fake loading delay or optimistic write merely to demonstrate
motion. Verify a slow response, rejected save and navigation while pending.

**An expanding detail section.** The section contains user-controlled text and media.
Use instant expansion first if comparison speed matters. If motion clarifies the
relationship, use a supported short size transition that adapts to real content.
Test expand → collapse → expand, a long paragraph and an image arriving mid-transition.
Expanded semantics and content accessibility follow the state, not the completion
callback. Reduced motion reaches the final size immediately.
