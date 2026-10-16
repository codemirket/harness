# Vanilla: paced technical publishing

**Language:** Warm technical publishing. This is a curator interpretation of a coherent reference instance, not a claim that the upstream uses this exact category name. Study this as an alternative direction; do not blend whole systems indiscriminately.

**Use when:** A technical service or open-source product needs substantial prose, clear ownership and a coherent explanatory sequence.

**Poor fit:** An operator needs many concurrent controls or a campaign depends on theatrical media rather than reading.

## Brief → concept

A community group is deciding whether it could operate the fictional Fieldnote archive. Before considering features, it needs to understand where original files live, who manages access and who tests recovery.

**Concept:** A readable technical publication with an engineered grid. Let the page move from proposition to responsibility, architecture and a first operating exercise. Warmth supports reading; it does not replace precise obligations.

## Source evidence → transferable grammar

**Observed source and coverage:** Official brochure guidance uses 50/50 and 25/75 relationships and discusses section pacing; it is work in progress. Strip guidance distinguishes meaningful background-changing strips. Fresh 1280×900 viewing showed the document and its layout headings. A later cookie overlay obstructed the detailed split capture; no unobstructed embedded example interaction is claimed. The published grid relationship is supported by official documentation reading.

**Interpretation to transfer:**

- A small set of shared alignment lines organises the entire page.
- A 50/50 opening relates a proposition to a concrete object.
- 25/75 sections let short scanning anchors introduce substantial prose.
- Paper and dark surfaces mark a change of subject, not every paragraph.
- Text width and section spacing create pace without a carousel or scroll effect.

These relationships inform the original specimen below. They do not establish task success, accessibility conformance or a right to copy the reference’s identity.

## Located decisions in the original specimen

| Location in `index.html` | Decision | Why it serves the brief |
| --- | --- | --- |
| `.hero`, `.archive-sheet` | The proposition shares the opening with an original three-row catalogue sheet. | The visitor sees what the service describes before reading architecture. |
| `#responsibility .split-quarter` | A short left anchor introduces a wider responsibility explanation, followed by application/team obligations. | Who maintains files and backups is a first-order decision, not footnote copy. |
| `#architecture`, `.spec-list` | A dark strip contains a four-part account of catalogue, originals, backup and restore test. | The surface shift marks technical structure and keeps related facts together. |
| `#guide .guide-list` | Three concrete operating steps end in a recovery exercise. | The narrative supplies a testable next action instead of a download-shaped dead end. |

The identity, all copy, CSS, JavaScript and inline artwork are original teaching material in this repository. The content is visibly fictional. The page loads no network assets, fonts, libraries or data. Native disclosures preserve their meaning with JavaScript disabled; enhanced controls are hidden without JavaScript and the complete default facts remain readable.

## Attractive wrong variant

Open the same HTML with `?variant=misleading`. The misleading query turns the opening into a centred promise and three spacious “Collect / Keep / Discover” tiles. Operating responsibility moves to a low-priority disclosure after exploration. The rhythm and warm typography remain coherent. They fail because a self-hosting decision can now precede the storage and backup obligations.

Judge the variant against the brief at the same width and with the same facts. Do not score it by polish alone. The explanatory teaching aside is outside `main`; the actual specimen lives in `main` in both variants.

## Failure probe

At 390px, the 50/50 and 25/75 sections must become one column in the same explanatory order, with no empty grid tracks. Follow the Responsibilities and Operating guide anchors and open the unresolved-questions disclosure. Explain whether backup is supplied: the answer must be unambiguously “maintained by your team”.

At narrow width also inspect the visible focus ring, long labels, enlarged text and the meaningful order of content. No critical interaction is hover-only; no animation is needed to communicate state.

## Materially different transfer task

Explain an offline field-survey application to a research coordinator. The key relationship is between local capture, sync conflict and data ownership; it is not an archive. Use repeated alignment lines to stage those responsibilities, then a new concrete recovery exercise. Deliver the opening, one operating section and the actual unresolved boundary.

**Reject the transfer when:** Reject a transfer that substitutes attractive feature tiles for the sync/conflict obligation, or that inherits Fieldnote’s fictional storage architecture without evidence.

## Exact owner, read and render route

Craft owner: `interface-design`. Embedded package owner: `design-inspiration`. Exact instance ID: `vanilla-publishing`.

```sh
mirket catalog read design-inspiration --path references/design-languages/vanilla-publishing.md
mirket catalog read design-inspiration --path assets/design-languages/vanilla-publishing/index.html
```

MCP reads use `skill_read` with `skill_id: "design-inspiration"` and the exact `path` above. The card resource is `mirket://skills/design-inspiration/references/design-languages/vanilla-publishing.md`; the HTML resource is `mirket://skills/design-inspiration/assets/design-languages/vanilla-publishing/index.html`. A source read does not render the page.

Render the saved `assets/design-languages/vanilla-publishing/index.html` from the installed `design-inspiration` directory in a browser. Project installations ordinarily place it under `.agents/skills/design-inspiration/` for Codex or `.claude/skills/design-inspiration/` for Claude; use the actual installation route returned by Mirket. A loopback static server or a direct local-file browser open is enough: no build step or dependency install is required. Compare the base URL and the same URL with `?variant=misleading`.

## Source edition, rights and exclusions

- [Vanilla Framework: Brochure site layouts](https://vanillaframework.io/docs/layouts/brochure) — Vanilla 4.59.0 identified in prior source evidence; live page checked 2026-10-15 and marked work in progress. Link-only reference. Framework LGPLv3 is not a blanket license for Canonical/Ubuntu marks, fonts, photography or website media; no framework code is included. [Rights evidence](https://raw.githubusercontent.com/canonical/vanilla-framework/main/LICENSE).
- [Vanilla Framework: Strip](https://vanillaframework.io/docs/patterns/strip) — Live documentation checked 2026-10-15; paper/light/dark themes and deprecated variants distinguished. Link-only reference. No Canonical assets, Ubuntu fonts or source files are copied. [Rights evidence](https://raw.githubusercontent.com/canonical/vanilla-framework/main/LICENSE).

All upstream sources are **link-only** here. Their software or font license scopes remain separate from branding and website/media rights. No upstream file is redistributed by this lesson, no protected mark is imitated, and no upstream package is installed or executed. Source edition and checked date describe the inspected evidence, not a promise that a live URL will remain unchanged.

## Verification scope

Inspected on 2026-10-15 in local headless Chrome 155 on macOS at 1280×900 and 390×900: purposeful and misleading variants, visible keyboard focus and document geometry. The 390px checks also covered JavaScript-off reading and actual 200% text sizing. All seven interface specimens made zero HTTP(S) requests in this local run. The located local interactions were exercised separately: correction and review recovery; filter, empty and batch scope; native navigation/disclosures; reconciled cost and invalid input; full article reading; and the clipboard-denied fallback as applicable.

The table and code block intentionally use labelled local horizontal scroll regions at narrow widths. Source-reference capture limits are stated above. No screen-reader session, physical-device test, successful system clipboard write, human visual acceptance, native-client activation or measured usability gain is claimed.
