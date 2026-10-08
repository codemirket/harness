# Navigation craft

Select a technique from the relationship between views and the actual router.
These recipes preserve navigation semantics; they do not supply a replacement
router, require a framework or prescribe a universal visual style.

## Compose a transition from identity

A collection-to-detail transition can carry the selected object's artwork into
its expanded placement while the supporting metadata enters quietly. Give only
that selected artwork a shared identity. Unselected siblings remain stable or
leave as a group. Preserve text legibility: isolate artwork from a title whose
font size and line wrapping differ, rather than scaling one giant screenshot.

For unrelated destinations, a short restrained context change may be clearer
than a fabricated shared element. For a workflow step, preserve the form shell
and show progress; never animate entered values into another user's state. For
back navigation, follow the project's actual history relationship and restored
position. Timing follows communication and frequency, not a compulsory table.

Before building, inspect first, middle and settled compositions. Check image
cropping, corners, backgrounds, elevation, masks and stacking. Choose one focal
action and enough rest to recognize it. A long stagger across every label makes
routine navigation feel slow; text must remain readable and actionable.

## Same-document snapshots

`document.startViewTransition(update)` captures the old view, invokes `update`,
then captures the committed new view for animation. Feature-detect the API and
provide the same application update without it. In a framework, integrate with
its actual commit boundary and supported routing API; resolving a data promise
or calling a state setter alone may not establish rendered content.

Use the three promises for their different boundaries:

| Boundary | Meaning and consequence |
| --- | --- |
| `updateCallbackDone` | The update callback settled. Propagate a real update error through the application error path. |
| `ready` | The transition's visual capture/pseudo-elements are ready. Skipping or invalid capture can reject this even after a successful update. |
| `finished` | The visual work ended or was skipped; an update rejection can also reject it. Do not gate navigation state on it. |

Do not replay an update merely because `ready` rejected. Calling `skipTransition()`
skips the visual work; it does not cancel the update callback or a data request.
Starting another transition skips the previous visual transition, so asynchronous
application work still needs its own ownership checks.

The following mechanical example accepts an already prepared, single-owner DOM
commit. It is not a router or concurrency controller. The optional visual promises
are observed to avoid unhandled rejection; actual commit errors still propagate.

```js
async function commitWithTransition(commit) {
  const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
  if (reduced || typeof document.startViewTransition !== 'function') {
    return await commit();
  }
  const transition = document.startViewTransition(commit);
  void transition.ready.catch(() => {});
  void transition.finished.catch(() => {});
  return await transition.updateCallbackDone;
}
```

The caller owns fetching, stale-request guards, actual DOM commitment, history,
focus, scroll and cleanup. If the caller needs live preference handling, retain
its owned transition, skip it when the preference changes, and release the media
listener on removal. Never swallow commit errors in a broad visual fallback.

## Shared elements and snapshot styling

Use a stable entity identity, not a list index. A `view-transition-name` must be
unique among rendered elements in each capture. Multiple cards named `hero`
invalidate the visual capture. Temporarily name the selected source, then name
the destination representation consistently. Clean up by ownership: a finishing
old transition must not clear names installed by its successor.

Capture geometry after needed images/fonts and layout are ready within a bounded
application policy; don't hide a broken loading state behind an arbitrary wait.
Inspect object-fit, image dimensions, card radius and text separately. Use the
root group and named groups deliberately: animating both root travel and a shared
element can produce unwanted doubled movement. Inspect pseudo-element styles
in the actual browser, including blending, clipping and stacking behavior.

Handle snapshot-only effects in the `::view-transition-*` pseudo-elements rather
than changing application layout merely to get an animation. Verify target
support for names, classes, types or newer APIs before using them. A shared
snapshot is a visual representation; do not make a clone a second focus target
or leave it covering real controls after the effect.

For a CSS 3D scene, choose the capture boundary carefully. A non-`none`
`view-transition-name` creates a stacking context/backdrop root and flattens that
node's 3D rendering even outside active navigation. Do not name a node required
to preserve depth between faces. Consider a flat leaf or an outer presentation
wrapper around a self-contained camera/3D subtree, then inspect both settled and
captured depth. See the current draft's
[rendering consolidation](https://drafts.csswg.org/css-view-transitions-1/#rendering-consolidation).

## Async navigation and error ownership

For a late response after A-to-B-to-C navigation, abort or disregard B according
to the existing router/request policy. Check ownership immediately before commit,
not only before fetching. Preserve request errors when they belong to the active
destination. An obsolete completion must not change URL, content, focus or scroll.

Prepare data outside the native update callback where possible; a long awaited
request inside that callback can hold the captured render while the user waits.
The application still needs visible pending, timeout and error recovery behavior.
For streaming or concurrent rendering, use the documented framework integration
and test the actual mount/commit lifecycle instead of assuming synchronous DOM.

History updates once per accepted navigation, at the existing router boundary.
Back/forward must restore the original entry and its intended scroll/focus policy.
Do not convert a history traversal into a new push to make the animation easier.
Preserve modified-click/new-tab behavior and same-page anchors. Direct links and
refresh must work without a previous source element or transition state.

## Cross-document navigation

For supported same-origin multi-page navigation, opt both documents in with:

```css
@view-transition {
  navigation: auto;
}
@media (prefers-reduced-motion: reduce) {
  @view-transition { navigation: none; }
}
```

Keep real anchors and browser navigation. Same-origin includes scheme, host and
port. A cross-origin destination or unsupported browser uses ordinary navigation;
no click interceptor is needed merely for the opt-in. Reload/direct entry may
have no matching transition, so pages must stand on their own.

The descriptor gates initiation; it does not by itself implement live preference
handling for an already-running incoming transition. Where the lifecycle API is
supported, retain the incoming event's transition, skip owned visual work when
reduced motion becomes active, and release listeners/handles after settlement or
teardown. Verify this document's real lifecycle rather than assuming a media rule
cancels a transition already in progress.

If assigning ephemeral identities in `pageswap`/`pagereveal`, first verify support
and lifecycle in the target browser. Release temporary state correctly for both
navigation directions and back/forward cache restoration; do not persist a
selected name on every restored card. Avoid storing sensitive destination data
in transition names or unnecessary navigation state.

## Accessibility and completion evidence

Use the project's accepted focus/announcement/scroll conventions. On a committed
new SPA screen, a meaningful heading or landmark can receive deliberate focus;
on history restoration, preserve the prior position/control when the existing
policy supports it. Stable application shells must not be recreated just to
animate them. Reduced motion must retain complete content and navigation.

In a development check, exercise native motion and forced no-API paths, a skipped
capture, an actual update failure, rapid requests, history traversal and direct
entry. Inspect unique captured identities, unhandled rejections, console errors,
settled overlays, focus, titles and narrow/long-content layout. Inspect actual
playback plus intermediate beats; profiling and device checks are separate.

For substantial visual work, compare continuity against the baseline using the
same content and viewport, refine visible defects, and report the evidence and
limits. One development example cannot certify every router or browser.

## Technical sources

Checked 2026-10-08; verify current support and installed framework versions for
the target project. Craft recipes and the mechanical example above are authored.

- [CSS View Transitions Level 1](https://www.w3.org/TR/css-view-transitions-1/): snapshots, promises, skip behavior and pseudo-elements.
- [CSS View Transitions Level 2](https://www.w3.org/TR/css-view-transitions-2/): cross-document and lifecycle extensions; implemented support needs separate verification.
- [Same-document guidance](https://developer.chrome.com/docs/web-platform/view-transitions/same-document): native update and shared-element mechanics.
- [Cross-document guidance](https://developer.chrome.com/docs/web-platform/view-transitions/cross-document): same-origin opt-in and navigation lifecycle.
- [React ViewTransition](https://react.dev/reference/react/ViewTransition): use only when applicable to the project's installed React/framework integration.
- [Reduced motion](https://www.w3.org/TR/mediaqueries-5/#prefers-reduced-motion): preference semantics; verify load and live changes in the actual renderer.
