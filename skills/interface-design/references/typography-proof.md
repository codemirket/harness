# Prove typography in the consuming renderer

Use when selecting fonts, changing type hierarchy, animating text or exporting
multilingual layouts. Keep the requested brand and languages. A type scale alone
does not prove that the actual text fits or shapes correctly.

Build a small specimen using actual headings, body, controls, numbers and the
longest plausible content. Record the exact font file, weight/style, variable axes,
fallback chain and embedding rights. Inspect the loaded face; synthetic bold,
missing glyphs and substituted fonts can render successfully while changing the
composition. Test normal and narrow delivery sizes, late font loading and zoom
where the consumer supports it. Do not fix an overflow by silently clipping meaning.

Choose relevant cases rather than imposing every language on every project:

| Case | What it exposes |
| --- | --- |
| Turkish: `İstanbul, ıslak, Şişli, öğün` | Language coverage, dotted/dotless casing and diacritics |
| Arabic mixed with `123`, dates, parentheses and an LTR product name | Joining, direction, punctuation and number order |
| Combining marks such as `é`, emoji family `👨‍👩‍👧‍👦` | Grapheme boundaries, fallback and segmentation |
| Narrow display headline, long link, actual product identifier | Wrapping, hyphenation, overflow and hierarchy |
| Currency, tabular numbers and negative values from the real locale | Alignment, decimal/group separators and minus-sign preservation |

For DOM output inspect the actual browser with real fonts. For Satori, Typst,
SVG/resvg or a video renderer, inspect that renderer's saved output too: successful
font loading does not guarantee equivalent shaping, line breaking or bidi support.
The pinned Satori supports WOFF2 and shaping, but some bidi embeddings and mixed
inline arrangements remain limited; choose a browser/typesetter for a required
case it cannot render correctly. Do not flatten essential UI text to conceal a gap.

For animated lettering keep the unsplit accessible text and natural reading order.
Use grapheme-aware segmentation and preserve shaping runs. Verify settled text,
reduced motion and failed-loading visibility. For exact logos or lockups, preserve
the supplied master and rights; do not reconstruct it from guessed font metrics.

Report the actual fonts, language cases and output inspected, including any known
fallback or unsupported shaping. A font list or a screenshot of Latin text is not
proof of a multilingual deliverable.
