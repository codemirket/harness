# Fiori: complex object continuity

**Language:** Complex object continuity. This is a curator interpretation of selected Fiori relationships, not an upstream category or a claim that this is an SAP application.

**Use when:** An operator moves between parent objects and their items, with distinct draft and commit boundaries that must survive interruption.

**Poor fit:** A dashboard of unrelated facts, side tools around one object, or a short form without a meaningful parent/item workflow. More columns do not automatically make work clearer.

## Brief → concept

In the fictional Helix procurement workspace, an operator prepares a field order with two items. They need to edit a child quantity and note, apply it to the order draft, then save the parent locally. Switching to another order must preserve or explicitly discard only the current order's draft. A parent-level availability failure must lead back to its child without losing the change.

**Concept:** Stable identity across a changing transaction boundary. Collection, parent and child each have a named place; applying a child is visibly different from saving its parent. Recovery follows that same hierarchy.

## Source evidence → transferable grammar

**Actually observed:** SAP Fiori's versioned v1.151 global-flow guideline was read through web retrieval on 2026-10-15. It distinguishes child Apply from main-object Save and describes linked/grouped errors. SAP's edge denied the fresh Chrome guideline and flow-image requests, so no fresh guideline pixels or authenticated product behavior are claimed. The official OpenUI5 flexible-column-layout sample was separately rendered at 1360×900 and exercised through list → mid column → end column. That primary sample demonstrates linked navigation/layout; it does not demonstrate business save semantics. These are complementary, explicitly limited sources.

**Grammar to transfer:** Keep the active object identifiable as a child opens; use related panes for a real hierarchy; locate each action at the boundary it changes; mark draft state accurately; return an error to the affected item; preserve work during a deliberate navigation choice. Pane count, SAP themes and blue chrome are not the lesson.

## Located decisions in the original specimen

| Location in `index.html` | Decision | Purpose |
| --- | --- | --- |
| `#context-path`, `#order-id`, `#item-parent`, `[data-order]` | Collection selection and parent/child IDs remain explicit as objects change. | “Item 20” cannot silently migrate to the other order that also has an item 20. |
| `#item-form`, `#item-status`, `#apply-item` | Unapplied form buffers belong to the exact order/item; Apply updates only the parent draft. | Item navigation does not overwrite another item's text or imply a parent save. |
| `#items`, `#order-status`, `#save-boundary`, `#save-order` | Rows say “Staged · not saved” until the parent saves. The total uses applied quantities only. | The visible state names the real local transaction boundary. |
| `#save-errors`, `#open-error-item` | A parent rejection names `PO-4201 / Item 20` and opens that item with its quantity intact. | The error is recoverable in context rather than a detached toast. |
| `#switch-dialog`, `#keep-switch`, `#discard-switch` | Interruption can keep the draft, continue editing, or discard only that order's changes. Native modal plus explicit Tab wrapping and Escape focus return is used. | Switching objects does not erase work or discard another order's state. |

All identity, content, CSS and JavaScript are original. The saved baseline, drafts and form buffers are in page memory only; reload resets them. There is no SAP runtime, backend, supplier transaction or durable draft. Default parent/item facts and both order identities remain readable without JavaScript. The teaching aside is outside `main`.

## Attractive wrong variant

Open `?variant=misleading`. The composed workspace uses calm green reassurance and labels an applied child **“Saved”**. Its parent still says **“Order draft · not saved”** and the actual local save is still required. The polish collapses a boundary the workflow depends on. This fails through inaccurate state semantics, even though its navigation and local totals still work.

## Discriminating failure probe

On `PO-4201`, edit item 20 to quantity **9**, then Apply. The row must say “Staged · not saved” and the order total must be **$204.00**. Save must reject the local pouch limit of 8 and name the exact parent/item; linked correction must preserve 9. Correct to 8, Apply, then Save: the total becomes **$192.00** and the parent baseline changes locally.

Then type an unapplied preparation note, choose `PO-4202`, and keep the draft while switching. Returning to `PO-4201` must recover item 20, quantity 8 and that note. Escape/Keep editing must restore focus; deliberate discard must affect only the named order's unsaved work. `data-testid` values expose `item-state-20`, `item-line-20`, identity, total and messages. Reject the misleading specimen when child “Saved” contradicts the unsaved parent. Inspect 390px, 200% text and keyboard focus.

## Materially different transfer task

Design a university course-and-assessment editor. A course owns assessment weights and feedback instructions; applying an assessment stages it, while publishing the course requires weights totaling exactly 100%. Show an 110% parent rejection with a route to a named assessment, and preserve an unapplied feedback draft while switching courses. Derive which edits invalidate publication, and label any persistence/approval boundary honestly. Do not reuse procurement amounts, quantities or supplier wording.

**Reject the transfer when:** Applying one assessment claims the course is published, or opening an assessment with the same number in another course overwrites the first course's draft.

## Exact owner, read and render route

Craft owner: `interface-design`. Embedded owner: `design-inspiration`. Instance ID: `fiori-complex-workspace`.

```sh
mirket catalog read design-inspiration --path references/design-languages/fiori-complex-workspace.md
mirket catalog read design-inspiration --path assets/design-languages/fiori-complex-workspace/index.html
```

MCP: `skill_read {"skill_id":"design-inspiration","path":"references/design-languages/fiori-complex-workspace.md"}` and `skill_read {"skill_id":"design-inspiration","path":"assets/design-languages/fiori-complex-workspace/index.html"}`.

Resources: `mirket://skills/design-inspiration/references/design-languages/fiori-complex-workspace.md` and `mirket://skills/design-inspiration/assets/design-languages/fiori-complex-workspace/index.html`.

Render the saved HTML from the actual installed `design-inspiration` directory through a local-file browser open or loopback server. No build or dependency install is required. Common project routes are `.agents/skills/design-inspiration/` for Codex and `.claude/skills/design-inspiration/` for Claude; use Mirket's returned location. Compare the base and `?variant=misleading`. Reading the source is not rendering.

## Source edition, rights and exclusions

- [SAP Fiori v1.151: Complex Objects — Global Flow](https://www.sap.com/design-system/fiori-design-web/v1-151/foundations/best-practices/global-patterns/object-handling/manage-objects-with-the-global-flow) — Versioned guideline text read 2026-10-15. Fresh Chrome page/image requests were edge-denied. [SAP copyright terms](https://www.sap.com/about/legal/copyright.html) cover website content/media; no general reuse right is inferred.
- [Official OpenUI5 SDK: Flexible Column Layout with Event Bus](https://sdk.openui5.org/#/entity/sap.f.FlexibleColumnLayout/sample/sap.f.sample.FlexibleColumnLayoutSimple) — Mutable official sample inspected 2026-10-15. Three related columns and navigation were exercised in Chrome 155; no immutable runtime version or parent-save behavior claimed. [OpenUI5 software license](https://github.com/SAP/openui5/blob/master/LICENSE.txt) is Apache-2.0, separate from SAP branding, themes and website media.

All upstream material is **link-only**. No SAP/OpenUI5 code, runtime, screenshot, logo, icon, font, theme or distinctive product identity is redistributed. The original example uses the repository's own license. Navigation evidence and source text establish only their stated scope.

## Verification scope

On 2026-10-15, local Chrome 155 on macOS rendered both variants at 1280×900 and 390×900. Checks cover narrow reflow, text at 200%, JavaScript-off default reading, zero external requests and axe checks of purposeful defaults. Exercised effects cover child Apply versus parent Save, unapplied-buffer preservation, parent rejection and linked correction, per-order recovery, scoped discard, invalid quantity, reset, modal Tab wrapping and Escape/focus return. No OData/backend contract, durable drafts, screen-reader session, physical device, native-client activation, human visual acceptance or measured usability gain is claimed.
