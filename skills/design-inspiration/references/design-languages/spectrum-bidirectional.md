# Spectrum: direction-aware relationships and invariant identity

Instance `spectrum-bidirectional` · interface · owner `design-inspiration`.
This language is Mirket’s interpretation of selected Spectrum relationships,
with W3C guidance for mixed-script HTML. It is not a complete localization system.

## Brief → concept → grammar

**Brief.** A support agent reads a fictional Arabic customer request about an
English-named product, retains the exact case reference and saves a local note.
Changing the interface direction must preserve the same case, draft and saved
content. The example makes no network write.

**Concept.** Reading flow changes; identity stays attached to its content. The
sidebar and navigation direction move together, while identifiers and symbols
with an invariant meaning keep their orientation. A note’s declared language
belongs to the note, independently of the surrounding interface language.

**Grammar.** A quiet neutral tool shell, one purple action accent and a restrained
green verification state support a compact request. Logical borders and spacing
group identity beside its detail. Arabic prose and Latin product/reference runs
remain distinct, with explicit boundaries rather than reversed strings. This is
a relationship to transfer into a product’s existing tokens, not an Adobe skin.

## Located choices in the original study

| Location | Choice and purpose |
| --- | --- |
| `#support-workspace`, `.workspace`, `.case-context` | The same DOM order and logical CSS produce a sidebar on the reading-start side. The main region changes `lang` and `dir`; the English teaching context outside it keeps its own language. |
| `.flow-icon`, `#clock-icon`, `#verified-icon` | Only the summary arrow reverses. The asymmetric clock hands and checkmark make indiscriminate mirroring inspectable. |
| `#case-reference`, `.atomic`, customer/product `bdi` elements | `SUP-2047`, an email and the product name have known Latin direction. Customer names use isolation. No stored string is reversed to achieve the layout. |
| `#locale`, `#note`, `#note-language`, `#saved-text` | A language preview changes surrounding copy and flow without resetting data. The native note-language select labels the note and its saved representation; direction alone is not treated as language detection. |
| `#note-form`, `#note-error`, `#note-status` | Empty input creates no saved note, marks the field invalid and focuses it. A corrected submit changes the visible saved record. The status does not claim customer delivery or durable storage. |

**Fit:** direction-sensitive web application layout, mixed-language record
identity and scoped localization working slices. **Poor fit:** proof of Arabic
translation quality, full RTL platform coverage, financial/date localization,
international search, native apps or actual assistive-technology behavior. Those
need their own current guidance and representative testing.

## Contrast and failure probe

Open `?variant=misleading` at the same Arabic preview. A universal CSS mirroring
rule reverses every SVG and `.atomic` run. The finish and data remain orderly,
but the email, reference, checkmark and clock no longer retain their orientation.
Switch both variants to English and back; compare these same objects, not merely
which side the sidebar occupies. The purposeful version mirrors only the arrow.

Then clear the note and save with Enter. Expect `aria-invalid="true"`, a visible
reason, focus on `#note` and no saved record. Enter a mixed-script note, declare
its language, save and switch interface language. Compare actual saved text,
its `lang` and the unchanged reference. A successful-looking status alone is not
evidence that the draft or identity survived.

## Materially different transfer task

Create an RTL library loan detail containing a Hebrew patron name, a Latin book
title with parentheses and an ISBN. Use the library’s existing visual system and
a different content hierarchy. Preserve a draft renewal reason while switching
layout. Accept when the return arrow follows flow, the ISBN remains recognizable,
the title’s punctuation stays within its own run, and the declared content
language survives submission. Require a fluent-language reviewer and relevant
assistive-technology check before claiming localization or nonvisual acceptance.

## Source, edition, inspected relationship and rights

Read [Spectrum bi-directionality](https://spectrum.adobe.com/page/bi-directionality/),
live documentation checked **2026-10-15**. Its rendered component-mirroring figure
was visually inspected in Chrome. The useful relationship is direction-sensitive
component flow with explicit icon exceptions; it does not supply the note model.
No immutable Spectrum documentation release is claimed. The page’s footer links
Adobe terms; a blanket reusable-media license was not established
(`license_url: null`). Keep it link-only and exclude Adobe marks, screenshots,
icons, fonts, interface assets and upstream code.

Read [W3C inline markup and bidirectional text in HTML](https://www.w3.org/International/articles/inline-bidi-markup/),
current article checked **2026-10-15**. It informs the tight directional runs and
unknown-name isolation. The [W3C document license](https://www.w3.org/copyright/document-license-2023/)
describes that source’s rights; no article text, illustration or code is bundled.
The specimen’s text, composition, SVG paths, CSS and JavaScript are original
Mirket work under the library MIT license, with system fonts and no remote assets.

## Read, render and verify

- Card resource: `mirket://skills/design-inspiration/references/design-languages/spectrum-bidirectional.md`.
- CLI: `mirket catalog read design-inspiration --path references/design-languages/spectrum-bidirectional.md`.
- MCP: `skill_read` with `{"skill_id":"design-inspiration","path":"references/design-languages/spectrum-bidirectional.md"}`.
- Executable source resource: `mirket://skills/design-inspiration/assets/design-languages/spectrum-bidirectional/index.html`. Read through `skill_read` with `{"skill_id":"design-inspiration","path":"assets/design-languages/spectrum-bidirectional/index.html"}`; text delivery does not execute the page.
- In this checkout: `python3 -m http.server 8851 --bind 127.0.0.1 --directory skills/design-inspiration/assets/design-languages`, then open `http://127.0.0.1:8851/spectrum-bidirectional/` and its contrast route. An installed skill can serve the corresponding local directory.
- Build owners: `interface-design` for direction-aware hierarchy, `frontend-engineering` for content ownership, and `accessibility-engineering` for input, language and focus evidence.

Authored checks use Chrome on macOS at 1280, 390 and 320 pixels, computed text
doubled at both narrow sizes, a static no-JavaScript route, actual note rejection
and recovery, locale changes, DOM language/isolation and selective icon transforms.
Automated axe observations are separate from these effects. No screen reader,
Arabic linguistic review, native device, browser matrix or conformance audit was
performed. A clean scan and agent inspection do not establish human acceptance.
