# W3C access patterns: equivalent input with stable focus

Instance `aria-alternative-input` · interface · owner `design-inspiration`.
The identifier names a teaching route, not a requirement to build an ARIA widget.
The original finite queue uses native buttons and an ordinary ordered list.

## Brief → concept → grammar

**Brief.** A workshop organizer arranges three field sessions after a required
orientation. The full route must be operable by clicking, tapping or keyboard,
including boundary rejection and undo. Changes remain local to this page.

**Concept.** A move belongs to a named session, not the array slot it occupied.
Equivalent inputs must cause the same data change. Focus, visible order and
feedback should continue describing that same identity after the move.

**Grammar.** A warm paper surface, dark green text and a large editorial title
frame an outdoor learning route. A numbered vertical sequence makes order the
main relationship. Short native actions sit with each session; the required
opening has a different structural treatment. A small factual summary supplies
the time and place without competing with the ordering task.

## Located choices in the original study

| Location | Choice and purpose |
| --- | --- |
| `#queue`, `li[data-id]`, `#order-proof` | One canonical ordered array contains stable IDs. Existing DOM rows move to match it. The ID order in the teaching aside exposes actual state independently of a friendly confirmation; these implementation details stay outside the organizer’s flow. |
| `#up-habitat`, `#down-habitat` and sibling buttons | Native buttons accept single clicks and keyboard activation. Their accessible names include the session title. No drag gesture, custom listbox, roving tabindex or emulated arrow-key model is required here. |
| `move()`, `render()` | A successful move resolves the current index from the stable ID, renders the real order and focuses the same button on the moved session. Repeated commands retain ownership. |
| Boundary buttons and `#move-status` | `aria-disabled` exposes an unavailable move while keeping its explanation reachable. The handler still prevents it. Rejection retains order and focus; a later Down operation recovers normally. |
| `#undo`, `#reset` | History retains the actual prior ID order. Undo restores that order without moving focus away from its command. Empty undo explains its rejection; reset restores the initial route. |
| Narrow `.queue li`, `.row-actions` | Controls move below their own session rather than into an unrelated toolbar. Number columns can grow with text enlargement, preserving the row association. |

**Fit:** finite sortable web queues, step priorities and small ordering tasks with
explicit constraints. **Poor fit:** large virtualized grids, spatial canvases,
cross-window dragging, multiplayer order reconciliation or proof of assistive
technology support. Those need a different interaction design and evidence.

## Contrast and failure probe

Open `?variant=misleading`. The same polished controls retain an initial numeric
array index instead of resolving the named item’s current position. Focus also
returns to the old row position. Neither the orderly list nor a positive status
reveals which object actually moved.

In the purposeful version, focus `#up-habitat` and press Enter. The real order
becomes `orientation,habitat,soil,water`; focus remains `up-habitat`. Enter again
must reject crossing orientation without moving any item. Tab then Enter moves
the same habitat session down. Undo restores each actual order in turn. In the
wrong version, the first move sends focus to `up-soil`; activating the named
habitat Down control then moves the stale-index soil row instead. Compare IDs,
active element and actual order, not the reassuring sentence.

## Materially different transfer task

Build an exhibition audio-tour queue with a compulsory access introduction,
six tracks and a different composition. Offer an explicit “move before track”
selector instead of row arrows. Retain stable track IDs through reorder and undo;
reject moving before the introduction with a recoverable explanation. Accept
only after keyboard and single-pointer users can produce the same final order,
focus remains meaningful, and a compatible screen reader has been tested before
making announcement claims. Recoloring Field School does not prove this transfer.

## Source, edition, inspected relationship and rights

Read [WCAG 2.2 Understanding SC 2.5.7, Dragging Movements](https://www.w3.org/WAI/WCAG22/Understanding/dragging-movements.html),
current informative support page checked **2026-10-15**. Its key distinction is a
non-dragging single-pointer route, separately assessed from keyboard access. The
specimen omits dragging entirely; it does not claim a tested drag implementation
or whole-page WCAG conformance. The source uses the
[W3C document license](https://www.w3.org/copyright/document-license-2023/).

Read [APG example listboxes with rearrangeable options](https://www.w3.org/WAI/ARIA/apg/patterns/listbox/examples/listbox-rearrangeable/),
live APG checked **2026-10-15**. The rendered list/action grouping was inspected
in Chrome. Its moved-item focus relationship informs this study; its custom
listbox interaction and example code are not copied. The page’s footer links the
[W3C software and document license](https://www.w3.org/copyright/software-license-2023/).
No immutable APG release is claimed. Retain source attribution; no W3C logos,
screenshots, code or documentation passages are bundled. This specimen’s text,
composition, SVG, CSS and JavaScript are original Mirket MIT work, with system
fonts and no remote assets. A reference pattern does not establish support in
every assistive-technology combination.

## Read, render and verify

- Card resource: `mirket://skills/design-inspiration/references/design-languages/aria-alternative-input.md`.
- CLI: `mirket catalog read design-inspiration --path references/design-languages/aria-alternative-input.md`.
- MCP: `skill_read` with `{"skill_id":"design-inspiration","path":"references/design-languages/aria-alternative-input.md"}`.
- Executable source resource: `mirket://skills/design-inspiration/assets/design-languages/aria-alternative-input/index.html`. Read through `skill_read` with `{"skill_id":"design-inspiration","path":"assets/design-languages/aria-alternative-input/index.html"}`; retrieval is not execution.
- In this checkout: `python3 -m http.server 8851 --bind 127.0.0.1 --directory skills/design-inspiration/assets/design-languages`, then open `http://127.0.0.1:8851/aria-alternative-input/` and its contrast route. Use the corresponding local directory in an installed skill.
- Build owners: `frontend-engineering` for stable state and focus recovery; `accessibility-engineering` for equivalent inputs and actual assistive evidence; `interface-design` for intended-size relationships.

Authored Chrome/macOS checks cover keyboard moves, boundary rejection and recovery,
single-click moves, consecutive undo, empty-undo rejection, reset and the wrong
variant’s actual identity/focus defect. Renders cover 1280, 390 and 320 pixels,
computed doubled text at both narrow sizes and static no-JavaScript content.
Automated axe observations do not establish screen-reader output. No screen
reader, touch device, switch or speech input, native platform, browser matrix or
conformance audit was performed; visual direction has no human acceptance result.
