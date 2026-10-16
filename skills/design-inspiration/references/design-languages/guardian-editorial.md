# Guardian Source: editorial reading modes

**Language:** Editorial typographic contrast. This is a curator interpretation of a coherent reference instance, not a claim that the upstream uses this exact category name. Study this as an alternative direction; do not blend whole systems indiscriminately.

**Use when:** A publication, report or knowledge collection needs both content discovery and sustained reading.

**Poor fit:** A small control-dense tool has no editorial hierarchy and several type roles would add noise.

## Brief → concept

A reader of the fictional Breakwater Review needs to discover the issue’s central explanation and then read it. The lead river field note is conceptually broader than two short observations; the newest item is not necessarily the most significant.

**Concept:** Let typography signal reading mode, and let editorial judgement determine space. A serif voice introduces and sustains the argument; a quieter sans layer identifies who, when and where. Build a publication rather than a dashboard of equal cards.

## Source evidence → transferable grammar

**Observed source and coverage:** Source’s official typography document assigns headline, text and interface roles and relates presets to front/article use. Fresh 1280×900 International-front viewing showed a strong masthead, lead story, grouped secondary content and quieter utilities. This is an in-situ hierarchy observation, not a claim that Source alone governs the entire front page or that the live news copy may be reused.

**Interpretation to transfer:**

- Headline and continuous-reading type may share a voice while retaining different scale and measure.
- Metadata and controls use a quieter sans role.
- A lead story gets space according to its explanatory function.
- Rules and alignment relate stories without making them interchangeable cards.
- Captions identify what an image proves and what it does not.

These relationships inform the original specimen below. They do not establish task success, accessibility conformance or a right to copy the reference’s identity.

## Located decisions in the original specimen

| Location in `index.html` | Decision | Why it serves the brief |
| --- | --- | --- |
| `.story-title`, `.article-copy`, `.meta` | Georgia carries headline and reading voice; Arial carries byline/date and controls. | Reading mode is visible without loading restricted Guardian fonts. |
| `.stories`, `.lead-story`, `.briefs` | A wide explanatory field note leads two narrower observations. | Space follows editorial significance rather than publication time. |
| `.lead-image`, `figcaption` | An original conceptual river map is captioned as imagined rather than geographic data. | The reader can use the diagram to follow a relationship without mistaking it for survey evidence. |
| `.article-disclosure`, `.article-copy h3` | A native disclosure opens the full original essay; subheads and a bounded measure support sustained reading. | The specimen includes actual reading content rather than only a magazine-shaped cover. |

The identity, all copy, CSS, JavaScript and inline artwork are original teaching material in this repository. The content is visibly fictional. The page loads no network assets, fonts, libraries or data. Native disclosures preserve their meaning with JavaScript disabled; enhanced controls are hidden without JavaScript and the complete default facts remain readable.

## Attractive wrong variant

Open the same HTML with `?variant=misleading`. The misleading query places a large date-led “latest” treatment above uniform sans cards. The newest bench observation moves first and receives the strongest surface; the field note is compressed. The design is coherent and readable. It fails the issue’s editorial purpose by making chronology determine prominence and flattening the difference between a central argument and a short note.

Judge the variant against the brief at the same width and with the same facts. Do not score it by polish alone. The explanatory teaching aside is outside `main`; the actual specimen lives in `main` in both variants.

## Failure probe

Open the full field note by keyboard. At 390px, follow headline, standfirst, byline, map caption and article in that order without clipped text. Identify why the older lead is more significant than the newest brief. A successful inspection should refer to the argument, not only its type size.

At narrow width also inspect the visible focus ring, long labels, enlarged text and the meaningful order of content. No critical interaction is hover-only; no animation is needed to communicate state.

## Materially different transfer task

Design a public research digest containing a major evidence review, a late correction and three short updates. Determine whether the correction must interrupt the lead before applying editorial hierarchy. Create new original copy and a reading specimen; document headline, reading and metadata roles. Do not carry over the river story or assume every older article deserves the lead.

**Reject the transfer when:** Reject a transfer where chronology hides a material correction, or where all entries become equal cards despite different evidence and consequences.

## Exact owner, read and render route

Craft owner: `interface-design`. Embedded package owner: `design-inspiration`. Exact instance ID: `guardian-editorial`.

```sh
mirket catalog read design-inspiration --path references/design-languages/guardian-editorial.md
mirket catalog read design-inspiration --path assets/design-languages/guardian-editorial/index.html
```

MCP reads use `skill_read` with `skill_id: "design-inspiration"` and the exact `path` above. The card resource is `mirket://skills/design-inspiration/references/design-languages/guardian-editorial.md`; the HTML resource is `mirket://skills/design-inspiration/assets/design-languages/guardian-editorial/index.html`. A source read does not render the page.

Render the saved `assets/design-languages/guardian-editorial/index.html` from the installed `design-inspiration` directory in a browser. Project installations ordinarily place it under `.agents/skills/design-inspiration/` for Codex or `.claude/skills/design-inspiration/` for Claude; use the actual installation route returned by Mirket. A loopback static server or a direct local-file browser open is enough: no build step or dependency install is required. Compare the base URL and the same URL with `?variant=misleading`.

## Source edition, rights and exclusions

- [Guardian Source: Typography stories in csnx](https://raw.githubusercontent.com/guardian/csnx/main/libs/@guardian/source/src/foundations/typography/stories/typography.mdx) — Mutable csnx main documentation checked 2026-10-15; current Source location, not archived guardian/source. Link-only source reference. CSNX software is Apache-2.0; Guardian font files and news/media have separate rights. No source code is copied. [Rights evidence](https://raw.githubusercontent.com/guardian/csnx/main/LICENSE).
- [The Guardian: International front](https://www.theguardian.com/international) — Live International front inspected at 1280×900 on 2026-10-15; content is time dependent. Link-only editorial reference. No news copy, photographs, marks, page code or font files are reused; no general reuse grant is established for this page. No general reuse license is established for the selected page.
- [Guardian fonts: Usage restrictions](https://github.com/guardian/fonts) — Official repository statement checked 2026-10-15. Official repository limits these fonts and CDN fonts to Guardian websites and apps. The original specimen uses Georgia and system sans instead. [Rights evidence](https://github.com/guardian/fonts).

All upstream sources are **link-only** here. Their software or font license scopes remain separate from branding and website/media rights. No upstream file is redistributed by this lesson, no protected mark is imitated, and no upstream package is installed or executed. Source edition and checked date describe the inspected evidence, not a promise that a live URL will remain unchanged.

## Verification scope

Inspected on 2026-10-15 in local headless Chrome 155 on macOS at 1280×900 and 390×900: purposeful and misleading variants, visible keyboard focus and document geometry. The 390px checks also covered JavaScript-off reading and actual 200% text sizing. All seven interface specimens made zero HTTP(S) requests in this local run. The located local interactions were exercised separately: correction and review recovery; filter, empty and batch scope; native navigation/disclosures; reconciled cost and invalid input; full article reading; and the clipboard-denied fallback as applicable.

The table and code block intentionally use labelled local horizontal scroll regions at narrow widths. Source-reference capture limits are stated above. No screen-reader session, physical-device test, successful system clipboard write, human visual acceptance, native-client activation or measured usability gain is claimed.
