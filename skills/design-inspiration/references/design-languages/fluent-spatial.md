# Fluent 2: spatial continuity

**Spatial continuity and surface roles** is a Mirket interpretation of the named Fluent 2 specimens. It distinguishes a container becoming a larger version of itself from navigation between peer destinations.

## Brief → concept → grammar

**Brief.** A field-note workspace needs more space for selected note FN-028. Notes and Collections are peers. The note’s identity and the destination hierarchy must survive both kinds of change.

**Concept.** Keep the thought; give it room. A pale inspection pane expands inside a stable workbench. The outer workspace stays a common frame of reference.

**Grammar.** Resizing/repositioning can preserve the identity of one container. A short fade between top-level destinations avoids inventing a directional parent/child relationship. Material has a role: base, transient and blocking surfaces should not all receive identical depth treatment.

## Located choices in the original study

| Location | Choice and purpose |
| --- | --- |
| `#workspace`, `.workspace-body` | The outer frame and neighboring list survive the inspector’s width change. Responsive reflow stacks the same content at narrow sizes. |
| `#inspector[data-object-id="FN-028"]`, `#note-title` | The identifier and title stay with the same object. The pane grows from a compact share of the workspace to a larger share; it does not teleport into a new document. |
| `#expand`, `#note-detail` | Expanded semantics and usable detail commit immediately. Native CSS transitions reverse from current dimensions; no obsolete expansion callback can hide a newer state. |
| `.workspace-nav`, `#workspace-crumb` | Notes and Collections use a finite fade and a common Workspace parent. The selected note remains available in either view. |
| Pane borders and restrained surface color | Selection and containment remain readable without relying on blur or Windows-native Mica behavior. This is an original web workbench, not a native Fluent material implementation. |

**Fit:** workspaces, document/detail relationships and responsive panes. **Poor fit:** flat comparison graphics where depth would invent rank, or a character story needing expressive acting. Do not infer that every route is a child because a slide is pleasing.

## Contrast and failure probe

`?variant=misleading` keeps the same finished workbench but slides Collections in as if it were a child of Notes. Its breadcrumb becomes “Notes / Collections,” even though the brief declares peer destinations. The defect is the invented hierarchy, not timing polish.

Probe: expand FN-028, then select Collections. In the purposeful route `#workspace[data-hierarchy="peer"]` and “Workspace / Collections” must agree with the declared hierarchy. A smooth slide plus `data-hierarchy="child"` fails. Rapidly alternate destinations and reverse the pane size; the latest selection and FN-028 must survive, with no stale panel or focus jump. At 320 px, all content must reflow rather than clip behind an expanding pane.

## Transfer task

Adapt this relationship to an original photo-review tool. Preserve an asset identifier while an inspector grows; treat Photos and Albums as peers. Build different content and layout, then check that an interrupted size change retains the selected asset and that navigation never implies Albums belongs inside Photos. Test actual geometry and hierarchy, not just matching easing names.

## Source, edition, observation and exclusions

[Fluent 2 Motion](https://fluent2.microsoft.design/motion), checked 2026-10-15, names Container transform, Elevation and Top level transitions. The current page distinguishes resize/reposition from quick fades at the top level and includes no-motion guidance. [Material](https://fluent2.microsoft.design/material) describes surface roles. The prior source inspection played the 3.03-second Container transform clip and observed different pane proportions within the same window; other named clips were source-described. Current official motion/material text was read during implementation. No source application keyboard, reduced-motion or native material behavior was tested.

Both live pages are unversioned; retain the check date. Their videos, marks, illustrations, icons and fonts remain link-only (`license_url: null`); no site-media license was established. Fluent UI’s repository MIT license does not clear every design-site asset. Folio contains original HTML/CSS/JS, system type and fictional notes, with no Fluent dependency.

## Read, render and build owner

- Study owner: `design-inspiration`. Build owner: `motion-design` for pane/controls; `page-transitions` owns actual route identity and top-level navigation in a product. Read their relevant interaction/navigation craft references.
- Card resource: `mirket://skills/design-inspiration/references/design-languages/fluent-spatial.md`.
- CLI: `mirket catalog read design-inspiration --path references/design-languages/fluent-spatial.md`.
- MCP: `skill_read` with `{"skill_id":"design-inspiration","path":"references/design-languages/fluent-spatial.md"}`.
- Executable source: `mirket://skills/design-inspiration/assets/design-languages/fluent-spatial/index.html`; use the same owner and `path: "assets/design-languages/fluent-spatial/index.html"` to retrieve it. Text delivery is not rendered evidence.
- Render the installed owner’s `assets/design-languages` with a local static server. Checkout command: `python3 -m http.server 8843 --bind 127.0.0.1 --directory skills/design-inspiration/assets/design-languages`. Visit `http://127.0.0.1:8843/fluent-spatial/` and `?variant=misleading`.

The static workspace, selected object and teaching content remain readable without JavaScript; inactive controls are hidden. Keyboard buttons have visible focus. Reduced motion removes size travel/fades at load and on live preference change, while preserving selection and expanded semantics. Finite animations cancel on new navigation or `pagehide`; CSS size transitions reverse toward the newest state. This teaches a relationship, without claiming native Windows material parity or measured performance.
