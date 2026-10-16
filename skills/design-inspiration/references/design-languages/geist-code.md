# Geist: code provenance and emphasis

**Language:** Typographic developer information. This is a curator interpretation of a coherent reference instance, not a claim that the upstream uses this exact category name. Study this as an alternative direction; do not blend whole systems indiscriminately.

**Use when:** A developer must understand a code or configuration change, its file context and its resulting behavior.

**Poor fit:** Literary reading, warm lifestyle identity or an infrequent-user form has no code artifact to inspect.

## Brief → concept

A developer reading the fictional Relay Notes guide needs to inspect a configuration change from one total attempt to three, understand that this allows at most two retries, and distinguish resulting source from a displayed diff.

**Concept:** Make the technical artifact the focal content. Prose explains behavior; mono locates exact identifiers and code; changed-line emphasis identifies what deserves inspection. A clean black-and-white theme alone would not teach this relationship.

## Source evidence → transferable grammar

**Observed source and coverage:** Official typography guidance distinguishes heading, label, copy, mono and tabular roles. Code Block names filenames, line references and diff/line emphasis, and advises selective highlighting. Fresh 1280×900 Code Block viewing exposed a filename header, numbered code and surrounding prose. No upstream integrated keyboard, accessibility or performance result is inferred.

**Interpretation to transfer:**

- Use sans for explanation and mono for exact source and identifiers.
- Keep filename/provenance adjacent to the code it describes.
- Emphasise only the lines under discussion.
- Preserve added versus removed meaning beyond color.
- Keep units and behavioral definitions beside ambiguous numeric limits.
- Separate display diff from copyable resulting code.

These relationships inform the original specimen below. They do not establish task success, accessibility conformance or a right to copy the reference’s identity.

## Located decisions in the original specimen

| Location in `index.html` | Decision | Why it serves the brief |
| --- | --- | --- |
| `.lead`, `.inline-code`, `.file` | Distinct prose, identifier and filename roles surround one original configuration diff. | The reader can distinguish explanation from an exact source token. |
| `.line.added`, `.line.removed`, `.legend` | Only line 2 changes. Plus/minus, text equivalents and surface differences redundantly identify additions and removal. | The diff is meaningful without relying only on red/green. |
| `.callout`, `#attempts .flow` | Three total attempts are reconciled as one initial run plus up to two retries. | A developer cannot safely substitute the ambiguous word “retries” for the configured total. |
| `#copy`, plain-text disclosure | The copy action returns only resulting source; the native disclosure provides a readable fallback. | A pasted snippet cannot accidentally contain the removed line or diff markers. |

The identity, all copy, CSS, JavaScript and inline artwork are original teaching material in this repository. The content is visibly fictional. The page loads no network assets, fonts, libraries or data. Native disclosures preserve their meaning with JavaScript disabled; enhanced controls are hidden without JavaScript and the complete default facts remain readable.

## Attractive wrong variant

Open the same HTML with `?variant=misleading`. The misleading query gives a dramatic true “3” metric priority, applies equal strong treatment to every code line and moves filename context below the code. The dark showcase remains polished. It fails because the change is harder to locate and source identity is separated from the first read. Dark theme itself is not the defect.

Judge the variant against the brief at the same width and with the same facts. Do not score it by polish alone. The explanatory teaching aside is outside `main`; the actual specimen lives in `main` in both variants.

## Failure probe

Identify the changed property without reading every line: maxAttempts changes 1 → 3, while 1000 ms backoff remains. Read the resulting-source disclosure and reconcile the attempt sequence. Exercise clipboard-denied feedback separately from a successful clipboard claim. At 390px, code must scroll within its labelled region without widening the page.

At narrow width also inspect the visible focus ring, long labels, enlarged text and the meaningful order of content. No critical interaction is hover-only; no animation is needed to communicate state.

## Materially different transfer task

Write a migration note for a fictional pagination API changing from offset to cursor. The important artifact is a request/response pair and a compatibility boundary, not retry counts. Derive line emphasis, provenance and copy behavior for that pair; include an old/new comparison and a failure case for an expired cursor. Do not retain the attempt-limit story or treat code aesthetics as migration correctness.

**Reject the transfer when:** Reject a transfer where copied source still includes removed syntax, or where code emphasis conceals an incompatible old/new response contract.

## Exact owner, read and render route

Craft owner: `interface-design`. Embedded package owner: `design-inspiration`. Exact instance ID: `geist-code`.

```sh
mirket catalog read design-inspiration --path references/design-languages/geist-code.md
mirket catalog read design-inspiration --path assets/design-languages/geist-code/index.html
```

MCP reads use `skill_read` with `skill_id: "design-inspiration"` and the exact `path` above. The card resource is `mirket://skills/design-inspiration/references/design-languages/geist-code.md`; the HTML resource is `mirket://skills/design-inspiration/assets/design-languages/geist-code/index.html`. A source read does not render the page.

Render the saved `assets/design-languages/geist-code/index.html` from the installed `design-inspiration` directory in a browser. Project installations ordinarily place it under `.agents/skills/design-inspiration/` for Codex or `.claude/skills/design-inspiration/` for Claude; use the actual installation route returned by Mirket. A loopback static server or a direct local-file browser open is enough: no build step or dependency install is required. Compare the base URL and the same URL with `?variant=misleading`.

## Source edition, rights and exclusions

- [Vercel Geist: Typography](https://vercel.com/geist/typography) — Live official documentation checked 2026-10-15; no component version pin claimed. Link-only documentation. No blanket website/component reuse license established; no font, component source or mark included. No general reuse license is established for the selected page.
- [Vercel Geist: Code Block](https://vercel.com/geist/code-block) — Live official documentation and visible code specimen inspected at 1280×900 on 2026-10-15. Link-only documentation/reference. No upstream component code, icon, media or site asset copied. No general reuse license is established for the selected page.
- [Geist Font: Independent font license](https://github.com/vercel/geist-font/blob/main/LICENSE.txt) — Official font repository license checked 2026-10-15. SIL OFL applies to that font family, independently of component/site/media rights. No Geist font files are admitted; system mono/sans are used. [Rights evidence](https://github.com/vercel/geist-font/blob/main/LICENSE.txt).

All upstream sources are **link-only** here. Their software or font license scopes remain separate from branding and website/media rights. No upstream file is redistributed by this lesson, no protected mark is imitated, and no upstream package is installed or executed. Source edition and checked date describe the inspected evidence, not a promise that a live URL will remain unchanged.

## Verification scope

Inspected on 2026-10-15 in local headless Chrome 155 on macOS at 1280×900 and 390×900: purposeful and misleading variants, visible keyboard focus and document geometry. The 390px checks also covered JavaScript-off reading and actual 200% text sizing. All seven interface specimens made zero HTTP(S) requests in this local run. The located local interactions were exercised separately: correction and review recovery; filter, empty and batch scope; native navigation/disclosures; reconciled cost and invalid input; full article reading; and the clipboard-denied fallback as applicable.

The table and code block intentionally use labelled local horizontal scroll regions at narrow widths. Source-reference capture limits are stated above. No screen-reader session, physical-device test, successful system clipboard write, human visual acceptance, native-client activation or measured usability gain is claimed.
