# Penpot: direct manipulation with edit ownership

Instance `penpot-direct-manipulation` · interface · owner `design-inspiration`.
The language name is Mirket’s interpretation of selected Penpot workspace
relationships, not a complete Penpot design system or client implementation.

## Brief → concept → grammar

**Brief.** A person arranging a small walking-event label must move either its
text or its round date stamp, inspect exact coordinates and recover an edit
after selecting another object. The finite document is fictional and local.

**Concept.** Point at an object; keep its identity. The canvas, layer list,
property fields and history describe the same two stable objects. Changing
selection must not change the owner of an earlier edit.

**Grammar.** A quiet green tool shell surrounds a cream, dotted work surface.
The selected layer receives one rust outline; the inspector names its stable ID.
Direct movement and exact numeric entry produce the same document change.
History records applied actions, while selection and unapplied form fields have
separate ownership. One pointer drag produces one undoable move.

## Located choices in the original study

| Location | Choice and purpose |
| --- | --- |
| `#layer-title`, `#layer-stamp`, `#shape-title`, `#shape-stamp` | Two representations use stable IDs `title` and `stamp`. The pressed state and visible words agree after selection and editing. |
| `#board`, `.canvas-window` | A finite 560 × 350 unit document uses actual coordinates. Its scrollable region preserves this working surface at narrow widths instead of silently scaling the meaning of a unit. |
| `#properties`, `#selected-id` | Labeled words/X/Y fields provide a keyboard alternative. Applying changes commits to the named layer. Switching layers explicitly discards unapplied fields and announces that result. |
| `#undo`, `#redo`, `#action-history` | Commands retain the edited object ID and its before/after values. Undo uses that identity even when another object is selected. A new edit clears redo. This is page-session undo, not file-version storage or collaboration. |
| Object key/pointer handlers | Arrow keys nudge one unit; Shift uses ten. Escape, pointer cancellation, lost capture, blur and page exit restore an interrupted move. Document shortcuts skip text inputs so browser text editing retains ownership. |
| `#fact-title-x`, `#fact-stamp-x` and corresponding Y/word cells | The current-document table exposes exact data independently of the spatial surface, including without JavaScript. |

**Fit:** finite layout tools, object placement, small creative workspaces and
recoverable document edits. **Poor fit:** workflows needing a full vector editor,
multiplayer conflict resolution, version restoration, off-canvas navigation or
an assumption that spatial input alone is accessible. The specimen implements
none of those additional capabilities.

## Contrast and failure probe

Open `?variant=misleading`. Its typography, canvas and controls keep the same
finish, but an inverse edit is applied to the current selection instead of the
edited object. This is an edit-ownership error.

Probe with keyboard: focus `#shape-title`, press Right, select
`#layer-stamp`, then activate `#undo`. The purposeful route restores title X from
61 to 60, leaves stamp X at 360 and keeps selection `stamp` in
`#studio-main[data-selected-id]`. The misleading route leaves title X at 61 and
replaces stamp’s state with the title’s earlier values. A clean-looking action
log does not establish correct recovery. Redo in the purposeful route restores
the title edit; a new edit after undo must disable redo. During a pointer move,
press Escape and verify that coordinates and history return to their prior state.

## Materially different transfer task

Build a seating-plan tool for a small outdoor workshop. Use stable seat IDs,
row/seat labels and a different composition; allow moving a selected chair and
changing its accessibility note. Select another chair before undo. Accept only
when the inverse targets the edited chair, the note remains attached to its own
ID, an interrupted move leaves no history entry and the complete operation is
possible without dragging. Recoloring Marker does not demonstrate this transfer.

## Source, edition, inspected relationship and rights

Read [Penpot workspace basics](https://help.penpot.app/user-guide/designing/workspace-basics/),
live documentation checked **2026-10-15**; no immutable release is claimed. Its
viewport image and action/history illustrations were rendered and visually
inspected in Chrome. The transferred relationship is a central work surface
surrounded by layer identity, editable properties and recoverable actions. The
source’s keyboard, numeric-input and action-history descriptions were also read.
The reference includes much more than this bounded study; source screenshots do
not establish native app behavior or a complete Penpot interaction audit.

The help page’s copyright and open-source label do not establish a blanket
license for documentation media (`license_url: null`). Keep the source link-only.
Exclude Penpot marks, screenshots, icons, fonts, interface artwork and upstream
code. This lesson’s composition, text, CSS and JavaScript are original Mirket work
under the library’s MIT license, use system fonts and request no network assets.

## Read, render and verify

- Card resource: `mirket://skills/design-inspiration/references/design-languages/penpot-direct-manipulation.md`.
- CLI: `mirket catalog read design-inspiration --path references/design-languages/penpot-direct-manipulation.md`.
- MCP: `skill_read` with `{"skill_id":"design-inspiration","path":"references/design-languages/penpot-direct-manipulation.md"}`.
- Executable source resource: `mirket://skills/design-inspiration/assets/design-languages/penpot-direct-manipulation/index.html`. Read with `skill_read` using `{"skill_id":"design-inspiration","path":"assets/design-languages/penpot-direct-manipulation/index.html"}`; text delivery is not execution.
- Render the installed skill’s `assets/design-languages` directory locally. In this checkout: `python3 -m http.server 8847 --bind 127.0.0.1 --directory skills/design-inspiration/assets/design-languages`, then open `http://127.0.0.1:8847/penpot-direct-manipulation/` and its contrast route.
- Build owner: `frontend-engineering` for state, input and history; `interface-design` for the working hierarchy and intended-size inspection.

Chrome on macOS was used for the authored pointer, keyboard, cancellation,
undo/redo and field-ownership journeys, narrow reflow, doubled text and static
no-JavaScript content. The finite board intentionally has its own scroll region.
No screen reader, touch device, native Penpot client, collaboration or persistent
file history was verified. Automated checks and an agent’s visual inspection do
not establish human acceptance or universal accessibility.
