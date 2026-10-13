---
name: brand-guidelines
description: Create, extract or refine a usable brand system with color and typography tokens, mark and asset rules, voice examples and rendered applications. Use for brand guidelines, identity consistency and brand handoff; a single screen polish uses interface-design.
---

# Brand guidelines

Make decisions another designer or agent can apply without guessing. A moodboard,
palette list or attractive cover is not a working brand system.

## Professional stance

Treat the brand as a recognizable promise expressed through repeatable choices.
Prioritize continuity, meaningful distinction and rules that survive new channels.
Ask which visual or verbal cues the audience should recognize without the logo,
which conventions have actual approval, and where consistency would impair use.
Derive a proposed rule from inspected applications, then try it on a contrasting
application before making it part of the system.

Challenge graphic design when a compelling asset cannot belong to a repeatable
family; challenge interface design when a local shortcut erases a useful identity
cue. In return, revise a brand rule when application evidence shows unreadable
type, inaccessible states or misleading language. Consistency alone cannot defend
a harmful rule. Give collaborators semantic tokens, allowed variants, source
assets and examples of the decisions they can make independently. Keep proposed
rules distinguishable from approved identity so a teammate can question them.

## Establish the source of authority

Identify the audience, promise, useful personality traits and actual channels.
Inspect approved marks, fonts, copy, product tokens and existing applications.
Separate observed conventions, user-approved rules and proposed repairs. Do not
quietly redesign an established identity while documenting it. Name unresolved
asset ownership, font licensing or approval instead of inventing authority.

For an extraction, read source tokens and computed styles, then compare real
applications. Repeated CSS values are evidence of usage, not proof of a rule.
For a new identity, start with an actual application and a compact specimen;
extend the system only after the core choices work together.

## Build the usable system

- Explain the brand's distinctive behavior in concrete terms: what it says,
  shows and prioritizes. Turn adjectives into choices and examples.
- Separate primitive colors from semantic roles: surface, text, action, focus,
  success and destructive state. Specify allowed foreground/background pairs,
  state variants and relevant light/dark behavior. Brand accent is not the only
  way to communicate status. Measure actual contrast rather than labeling a
  whole palette accessible.
- Keep semantic token references connected to the delivered source: confirm they
  resolve in each supplied theme and that example components consume those same
  values. A renamed token needs an explicit consumer mapping; duplicating a new
  palette beside unchanged product tokens produces contradictory guidance.
- Define type roles and fallbacks with sample headings, paragraphs, labels,
  numerals and long content. Record licensed font source/weights where supplied.
  System fonts vary by platform; a token name does not promise identical metrics.
- Preserve editable marks. Specify variants, intended backgrounds, tested minimum
  size and clear-space rule derived from the mark. Show real misuse examples only
  where they prevent likely errors. A favicon may need a separate simplified asset.
- Describe imagery and illustration through composition, palette, detail, crop and
  subject choice. Keep editable vector masters separate from raster exports;
  asset names should identify role, variant and format.
- Show voice in context: a useful action label, empty state, error and public
  claim. Keep meaning and evidence intact; personality cannot obscure recovery.

Read [the worked Fieldwork reference](references/fieldwork-system.md) for a
token/usage/voice/export example. Its numbers are a specimen, not universal rules.

## Test the brand in use

Render the actual product, a relevant communication and the asset at its intended
size. Choose only channels in scope. Compare hierarchy, mark, color roles, type,
copy and crop across them. Test long/localized content and relevant accessibility
states. Screen contrast does not establish print reproduction; print needs the
requested color profile, material and proofing process.

The Mirket CLI measures opaque sRGB color pairs:

```sh
mirket craft contrast '#647066' '#fffefa'
```

This calculation does not inspect images,
gradients, transparency, focus geometry or the whole page. Read the reference's
WCAG source before interpreting thresholds; report exact measured pairs.

Deliver the concise guidelines, machine-readable tokens in the project's existing
format, editable assets plus required exports, and an inspected specimen. Record
source provenance, proposed versus accepted decisions and concrete inconsistencies.
Use `interface-design` for application changes, `svg-creation` for vector craft,
and the available image-generation workflow for raster assets. This skill does not
provide a font license, image provider or publishing authority.
