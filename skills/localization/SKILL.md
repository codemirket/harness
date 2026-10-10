---
name: localization
description: Translate and localize prose, product interfaces and structured message files while preserving meaning, register, terminology and runtime tokens. Use for substantial multilingual work and translation QA. Handle a simple sentence directly without adding a localization project workflow.
---

# Localization

Produce natural language for the intended locale while preserving the source's
facts and commitments. Establish language/region, audience, domain and register
from the request and project. Resolve an ambiguity only when it changes meaning;
do not ask for a large brief to translate a short passage.

## Preserve what the text means

- Read surrounding context, supplied terminology and existing translations.
  Maintain a small glossary when repeated terms or multiple files warrant it.
  Honor approved names and distinguish names from ordinary words.
- Preserve negation, conditions, quantities, dates, actors, obligations, deadlines
  and uncertainty. Do not improve a claim into a stronger promise or soften a
  prohibition into a suggestion. Flag an ambiguous source rather than silently
  choosing a consequential interpretation.
- Adapt idiom, sentence structure and register to the locale. Keep factual values
  fixed unless conversion is requested. Locale formatting is not currency
  conversion; an ambiguous source date needs resolution before localization.
- Distinguish translation from requested transcreation. If marketing adaptation
  changes the offer or a regulated claim, make that change visible.

## Protect structured messages

For code or resource files, inspect the actual format and consuming framework.
Preserve keys, placeholder identity and type, tags, escapes, URLs, identifiers
and interpolation syntax. Do not translate executable expressions. Parse with
the project's available tooling and compare token inventories; a valid JSON file
alone does not establish valid ICU or correct message behavior.

Use the locale's plural/select rules and the framework's implementation. Retain
required fallback branches and test representative zero, singular, plural and
selection paths. Word order may change; placeholder names and meaning must not.
Use established project terminology for technical/legal concepts and current
authoritative references when uncertain.

For a structured UI translation, read the
[worked semantic and token case](references/message-integrity.md).

## Verify language and interface separately

Review the target text against the source for meaning and then read it naturally
in context. A back-translation can reveal a problem; it cannot prove fluency.
Inspect length, wrapping, truncation, fonts, accessible labels and interaction
states in the actual interface when UI changes are in scope.

For RTL locales, inspect direction, mixed-script text, identifiers and numbers.
Use logical layout and appropriate bidirectional isolation. Mirror directional
affordances only when their meaning changes with reading direction; do not mirror
logos, photographs or every icon by default.

Deliver the requested language or files and identify material unresolved source
ambiguity or unperformed layout checks. Claim native-speaker, domain-expert or
human approval only when that review actually occurred. A translation model's
confidence or a structural checker is not evidence of such approval.
