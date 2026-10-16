# Android adaptive: keep the selected content through layout change

Instance `android-adaptive-continuity` · interface · owner `design-inspiration`.
This is a **web teaching analogue informed by native Android guidance**. It is
not a Compose implementation, Android runtime test or device coverage claim.

## Brief → concept → grammar

**Brief.** A site coordinator reviews three fictional visits and keeps a draft
note for one of them. Switching between a wide two-pane view and a narrow
single-pane view, returning to the route or reloading the tab must not silently
replace the selected visit or attach its draft to another record.

**Concept.** Space changes the arrangement; identity preserves the work. The
selected visit and each record’s draft belong to state outside the pane layout.
A return to the route retains the last selected visit as context.

**Grammar.** Use a scannable route beside one detailed briefing when space permits.
At a constrained width, show the route or the selected detail as successive
places. A selected-row marker, explicit visit ID, contextual Back action and
record-owned note maintain continuity. The original green notebook palette and
restrained surfaces support field information; they are not a reconstruction of
Android’s visual identity.

## Located choices in the original study

| Location | Choice and purpose |
| --- | --- |
| `#fieldlog[data-selected-id][data-pane]` | Stable IDs `quay`, `orchard`, `library` own selection independently of the one/two-pane media query. `data-pane` records the single-pane navigation place. |
| `#select-orchard` and sibling visit links | The selected record has `aria-current="true"`, a visual marker and a full wrapping name. The link remains the focus-return target after going back to the route. |
| `#detail-panel[data-record-id]`, `#entry-orchard`, `#title-orchard` and siblings | The selected briefing, accessible heading and actual content agree on identity. Narrow navigation focuses the new heading; returning focuses the selected route link. |
| `#back-to-list`, `#route`/`#entry-<id>` URL fragments | The in-page return goes to the route while retaining selected identity. Browser back/forward follows the fragment history. This is a declared web policy, not Android’s navigator/back-stack implementation. |
| `#note-orchard` and sibling `textarea[data-note]` fields | Drafts are keyed by visit ID, updated synchronously and saved to this tab’s session storage when available. Changing selection preserves each visit’s draft. Storage failure is announced and leaves an in-memory page draft. |
| `#layout-status`, `#continuity-status` | The layout mode and result of navigation/restoration remain explicit. There is no claim of server submission, cross-device sync or a saved report. |

Without JavaScript all three briefings remain readable, and real fragment links
reach their sections. With script, the complete data remains in the document
while only the selected detail is exposed. At wide sizes the route and detail
are visible together; at narrow sizes focus is moved out of any pane that becomes
hidden. No transition animation is necessary to express this relationship.

**Fit:** reviewing a collection and a selected detail, adaptive navigation and
record-owned interrupted drafts. **Poor fit:** a native Android release claim,
fold posture handling, task/process restoration, Predictive Back, multi-window
concurrency or a workflow whose complete information must always be compared
side by side. Those require their own implementation and evidence.

## Contrast and failure probe

Open `?variant=misleading`. It has the same finished notebook and reflow, but a
layout-width change resets selection to `quay`. The draft still belongs to its
original record; the currently displayed visit silently changes. A responsive
screenshot therefore appears successful while continuity fails.

In the purposeful route at 1280 px, choose `#select-orchard` and type a distinctive
note into `#note-orchard`. Narrow to 390 px: `#fieldlog[data-selected-id]` and
`#detail-panel[data-record-id]` must remain `orchard`, and the note must be
unchanged. Activate `#back-to-list`; route focus returns to `#select-orchard`.
Reopen the orchard, reload this tab and check both identity and draft. Browser
Back returns to the route; Forward reopens the detail. Widen again and verify the
selected row, briefing and draft agree. The misleading route becomes `quay` at
the width change, which is a semantic failure even though nothing overlaps.

## Materially different transfer task

Build an original reference-book reader: a chapter list and one selected passage
with a local annotation. Test a long chapter title, two annotations on different
passages, navigation to the list and a constrained-width change during editing.
Choose and document a return policy; keep passage identity and annotation
ownership through interruption. If delivered as native Android, implement with
the project’s actual navigation/state tools and verify process/configuration and
Predictive Back behavior on the declared native runtime. A resized web page
cannot establish that transfer’s native acceptance.

## Source, edition, inspected relationship and rights

Read [Android Developers: Build a list-detail layout](https://developer.android.com/develop/ui/compose/layouts/adaptive/list-detail),
**last updated 2026-10-01**, checked **2026-10-15**. Its wide and narrow figures
were rendered and visually inspected in Chrome. The transferred relationship is
one selected content identity across side-by-side and successive panes. The
current guide’s content-key restoration and explicit back-policy distinctions
inform the teaching invariant; its Kotlin code is not copied or executed.

The page links [Google Developers site policies](https://developers.google.com/terms/site-policies):
documentation generally uses CC BY 4.0 and code examples Apache 2.0, with stated
exceptions for brand features and some media. This reference stays link-only;
exclude Android/Google marks, screenshots, icons, Roboto, upstream layouts/code
and platform media. The specimen’s content, CSS and JavaScript are original
Mirket work under the library’s MIT license and use system fonts with no network
assets. A documentation license does not create native runtime coverage.

## Read, render and verify

- Card resource: `mirket://skills/design-inspiration/references/design-languages/android-adaptive-continuity.md`.
- CLI: `mirket catalog read design-inspiration --path references/design-languages/android-adaptive-continuity.md`.
- MCP: `skill_read` with `{"skill_id":"design-inspiration","path":"references/design-languages/android-adaptive-continuity.md"}`.
- Executable source resource: `mirket://skills/design-inspiration/assets/design-languages/android-adaptive-continuity/index.html`. Read with `skill_read` using `{"skill_id":"design-inspiration","path":"assets/design-languages/android-adaptive-continuity/index.html"}`; source delivery is not runtime verification.
- Render the installed skill’s `assets/design-languages` locally. In this checkout: `python3 -m http.server 8847 --bind 127.0.0.1 --directory skills/design-inspiration/assets/design-languages`, then open `http://127.0.0.1:8847/android-adaptive-continuity/` and its contrast route.
- Build owner: `frontend-engineering` for this browser analogue and `interface-design` for pane composition. An actual Android deliverable requires the project’s native Android implementation owner and runtime checks; this card supplies a bounded reference, not that expertise.

Chrome on macOS exercised selection, per-record drafts, one/two-pane resize,
return focus, hash Back/Forward and same-tab reload restoration; narrow and
doubled-text layouts and static briefings were inspected. **Unverified native
paths:** Compose adaptive library behavior, lifecycle/process death, device
rotation/fold/hinge posture, physical-device input, IME/safe areas, Predictive
Back and TalkBack. No screen reader or native device test is implied. Automated
web checks and agent visual inspection do not establish human acceptance.
