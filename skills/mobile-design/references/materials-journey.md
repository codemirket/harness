# Worked journey: find a material, inspect it, save it, return

Example domain: the fictional Fieldwork materials collection. Goal: save a useful
reference while keeping the current query and collection position. This is a
design/verification example, not an unbuilt native application's acceptance record.

## State and navigation decisions

| State | Expected behavior | Failure probe |
| --- | --- | --- |
| Collection | Search and category jointly narrow results; active criteria remain visible. | Long query, zero results and clearing criteria. |
| Preview/detail | Name, useful attributes and save state are available without relying on art. | Open a deep link or return after rotation/window resize. |
| Save | One user intent changes one material's saved state; state is announced. | Repeated tap, failed persistence and delayed response. |
| Return | Query, category and a useful location/focus survive. | Back while entering; dismiss with keyboard present. |
| Saved view | Saved items are recoverable; removing the last gives a useful empty state. | Focus cannot remain on a removed control. |

In the current web specimen, a native HTML dialog provides transient inspection.
The app keeps the collection context and returns focus to the trigger. A larger
native product might instead use a detail destination with a shareable ID. Decide
from the content, deep-link and back-stack contract; do not copy the web dialog
onto every platform.

## Platform-specific handoff

| Concern | iOS/iPadOS | Android | Mobile web |
| --- | --- | --- | --- |
| Navigation | Reuse the app's native tabs/sidebar and navigation stack; back follows hierarchy, dismiss ends a presentation. | Reuse the app's navigation/back stack and system Back; adapt top-level controls to window size. | Use real links for destinations; preserve browser Back/forward and deep entry. |
| Touch | Use the native control hit area; 44×44pt is Apple's published general design target. | Material/Compose guidance uses 48×48dp; check expanded targets cannot overlap. | WCAG 2.2 AA target-size criterion is 24×24 CSS px with exceptions; choose larger controls where touch usability requires them. |
| Input | Native input purpose, Dynamic Type and safe-area behavior; verify keyboard and external keyboard. | Appropriate input/IME actions, font scaling and window/IME insets; verify system gesture navigation. | Labels, input type/autocomplete and visible focus; test actual browser/keyboard behavior separately from viewport emulation. |
| Assistive use | VoiceOver labels, values, order and focus after presentation/dismissal. | TalkBack semantics, grouping, state and focus after navigation. | DOM semantics, keyboard traversal and target browser/screen-reader pairing. |

These are distinct guidance/measurement systems, not numeric translations.
The WCAG target criterion includes spacing and other exceptions; a 32px web close
button is not automatically a WCAG failure because it is below an iOS target.
The product can still choose a larger hit region based on touch testing.

Android's adaptive navigation documentation shows `NavigationSuiteScaffold`
switching between navigation bar and rail according to window/posture information.
Use the project's supported API/version; this example does not require that library
or mean every two-destination app needs a bottom bar. System insets and keyboard
insets represent different occlusions: applying a fixed bottom padding cannot
reliably model both. Prefer the framework's inset contract and check consumption
when nested containers also apply padding.

## A bounded responsive-web exercise

Use an existing local server and configured browser. For a materials catalog,
capture 390×844 and a wider layout in both motion modes.
Exercise filter Paper → search Cotton → preview → save → dismiss → Saved.
Assert the rendered result and saved state, then remove the last item and recover.
Inspect the actual mobile capture; an overflow number alone misses tiny labels,
bad hierarchy and unreachable controls.

Then test what the simple happy path misses:

- Search a long value and clear it; observe whether the query/result summary wraps.
- Increase text size and narrow the available viewport; check actions remain usable.
- Open/close rapidly and change reduced-motion preference during entry; inspect
  focus, backdrop, final state and whether old completion work runs afterward.
- Focus the search field on a real phone; show/hide keyboard, scroll and dismiss
  detail. Emulation alone leaves browser chrome, occlusion and touch unverified.
- For native delivery, run the real build on its supported simulator/emulator,
  then test the consequential gestures and assistive path on device.

Acceptance record example: “Chrome desktop engine, 390×844 viewport: filter,
preview and focus return observed; no horizontal overflow. iOS Safari keyboard,
Android Back, VoiceOver/TalkBack and native builds not exercised.” Do not shorten
this to “mobile-ready”. A failed optional device check is a remaining limitation;
a failed required supported-platform journey is a delivery defect.

## Primary sources checked 2026-10-09

- [Apple UI design tips](https://developer.apple.com/design/tips/) for touch design
  and its 44pt target. Consult current platform HIG for a specific component.
- [Android accessibility API defaults](https://developer.android.com/develop/ui/compose/accessibility/api-defaults)
  for 48dp, semantics and expanded touch bounds.
- [Android adaptive navigation](https://developer.android.com/develop/ui/compose/layouts/adaptive/build-adaptive-navigation)
  for navigation patterns by available window space.
- [Android window insets](https://developer.android.com/develop/ui/compose/system/insets)
  for system and IME occlusion boundaries.
- [W3C target size minimum](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html)
  for CSS-pixel requirements and exceptions.

No platform SDK, device runner or native app is supplied by this skill. Source
guidance does not establish that the user's implementation meets it.
