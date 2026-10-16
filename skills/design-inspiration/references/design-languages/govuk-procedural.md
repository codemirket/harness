# GOV.UK: reviewable civic procedure

**Language:** Civic procedural clarity. This is a curator interpretation of a coherent reference instance, not a claim that the upstream uses this exact category name. Study this as an alternative direction; do not blend whole systems indiscriminately.

**Use when:** An infrequent visitor needs to inspect and correct a small set of facts before completing a resumable transaction.

**Poor fit:** The primary job is immersive brand storytelling or simultaneous comparison of many data records.

## Brief → concept

A person applying to the fictional Northhaven allotment scheme has already entered preferences. Their immediate task is to check a contact address and finish a review without mistaking local completion for a sent application.

**Concept:** Confidence through recoverable facts. Make the review the centre of the page, and let task status explain what remains. Use a calm civic voice without borrowing government identity.

## Source evidence → transferable grammar

**Observed source and coverage:** The official Check answers documentation places review before submission and uses contextual correction. Complete multiple tasks groups longer work and recommends a small meaningful status vocabulary. The freshly viewed 1280×900 Check answers page exposed its example heading; the attempted deeper example locator was hidden and did not supply a full fresh embedded-example capture. Prior source evidence supplies the pattern detail.

**Interpretation to transfer:**

- Sentence-case task headings describe the immediate job.
- A narrow measure keeps review facts in one reading path.
- Rules separate facts; labels remain close to values.
- Correction names its context and returns to the reviewed fact.
- Status describes actual completion, rather than decoration or reassurance.

These relationships inform the original specimen below. They do not establish task success, accessibility conformance or a right to copy the reference’s identity.

## Located decisions in the original specimen

| Location in `index.html` | Decision | Why it serves the brief |
| --- | --- | --- |
| `.review-content h1`, `.step` | The review heading follows the step indicator; no campaign occupies the default first screen. | The visitor recognises the current decision before reading individual facts. |
| `.fact`, `#address-value` | A definition list pairs labels with values; long plot preference wraps in the value column. | The address and access requirement can be inspected rather than inferred from a progress percentage. |
| `#edit-address`, `#address-form` | Native disclosure exposes an optional editor; Save updates only the displayed address and returns focus to the disclosure. | Correction is recoverable and its scope remains local. |
| `.inset`, `#finish`, `#task-status` | “Not yet completed” and the local-only completion action agree with the three-task status list. | Completion does not imply an external application or reservation. |

The identity, all copy, CSS, JavaScript and inline artwork are original teaching material in this repository. The content is visibly fictional. The page loads no network assets, fonts, libraries or data. Native disclosures preserve their meaning with JavaScript disabled; enhanced controls are hidden without JavaScript and the complete default facts remain readable.

## Attractive wrong variant

Open the same HTML with `?variant=misleading`. The misleading query inserts a spacious seedling campaign before the review, lowers the review heading and spreads the task list below it. The illustration and copy are original and polished. They fail because address verification now requires getting past acquisition messaging. Green or an illustration is not inherently wrong; their precedence is wrong for this brief.

Judge the variant against the brief at the same width and with the same facts. Do not score it by polish alone. The explanatory teaching aside is outside `main`; the actual specimen lives in `main` in both variants.

## Failure probe

Edit the address to “Apartment 204, Willow Lane Community Housing, Northhaven”, save, then finish. The new value must wrap without losing its label; focus must return after correction; completion must state that nothing was sent. At narrow width, task status must wrap without covering task text.

At narrow width also inspect the visible focus ring, long labels, enlarged text and the meaningful order of content. No critical interaction is hover-only; no animation is needed to communicate state.

## Materially different transfer task

Design a university equipment-loan renewal review. The decision concerns an item, return deadline and an exception requiring staff review, rather than an application. Derive a new status model and contextual correction route. Deliver one review screen and a state table; explain which facts must be checked before renewal. Do not reuse allotment fields, the garden illustration or a generic three-step flow.

**Reject the transfer when:** Reject a transfer that says “Complete” while an unresolved exception remains, or that places a persuasive campus hero before the return deadline.

## Exact owner, read and render route

Craft owner: `interface-design`. Embedded package owner: `design-inspiration`. Exact instance ID: `govuk-procedural`.

```sh
mirket catalog read design-inspiration --path references/design-languages/govuk-procedural.md
mirket catalog read design-inspiration --path assets/design-languages/govuk-procedural/index.html
```

MCP reads use `skill_read` with `skill_id: "design-inspiration"` and the exact `path` above. The card resource is `mirket://skills/design-inspiration/references/design-languages/govuk-procedural.md`; the HTML resource is `mirket://skills/design-inspiration/assets/design-languages/govuk-procedural/index.html`. A source read does not render the page.

Render the saved `assets/design-languages/govuk-procedural/index.html` from the installed `design-inspiration` directory in a browser. Project installations ordinarily place it under `.agents/skills/design-inspiration/` for Codex or `.claude/skills/design-inspiration/` for Claude; use the actual installation route returned by Mirket. A loopback static server or a direct local-file browser open is enough: no build step or dependency install is required. Compare the base URL and the same URL with `?variant=misleading`.

## Source edition, rights and exclusions

- [GOV.UK Design System: Check answers](https://design-system.service.gov.uk/patterns/check-answers/) — Live documentation checked 2026-10-15; no immutable version claimed. Link-only reference. Documentation footer states OGL v3 except where otherwise stated; government identity, GDS Transport and excluded assets are not admitted. [Rights evidence](https://www.nationalarchives.gov.uk/doc/open-government-licence/version/3/).
- [GOV.UK Design System: Complete multiple tasks](https://design-system.service.gov.uk/patterns/complete-multiple-tasks/) — Live documentation checked 2026-10-15. Link-only reference; OGL documentation scope is distinct from MIT Frontend software and government-brand/font rights. [Rights evidence](https://www.nationalarchives.gov.uk/doc/open-government-licence/version/3/).
- [GOV.UK Design System: Typeface](https://design-system.service.gov.uk/styles/typeface/) — Live typeface guidance checked 2026-10-15. Official guidance directs use of an alternative typeface outside service.gov.uk. No GDS Transport font is included. [Rights evidence](https://design-system.service.gov.uk/styles/typeface/).

All upstream sources are **link-only** here. Their software or font license scopes remain separate from branding and website/media rights. No upstream file is redistributed by this lesson, no protected mark is imitated, and no upstream package is installed or executed. Source edition and checked date describe the inspected evidence, not a promise that a live URL will remain unchanged.

## Verification scope

Inspected on 2026-10-15 in local headless Chrome 155 on macOS at 1280×900 and 390×900: purposeful and misleading variants, visible keyboard focus and document geometry. The 390px checks also covered JavaScript-off reading and actual 200% text sizing. All seven interface specimens made zero HTTP(S) requests in this local run. The located local interactions were exercised separately: correction and review recovery; filter, empty and batch scope; native navigation/disclosures; reconciled cost and invalid input; full article reading; and the clipboard-denied fallback as applicable.

The table and code block intentionally use labelled local horizontal scroll regions at narrow widths. Source-reference capture limits are stated above. No screen-reader session, physical-device test, successful system clipboard write, human visual acceptance, native-client activation or measured usability gain is claimed.
