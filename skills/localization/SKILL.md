---
name: localization
description: "Implement or review multilingual or bidirectional UI where message meaning, expansion and rendered layout must survive locale changes. Skip isolated translation without a product/UI boundary."
---

# Localization

Identify supported locales, scripts, direction, fallback and the task-critical messages. Preserve meaning, variables, plurals, gender/context where relevant and error/action relationships. Separate translatable text from stable identifiers; avoid sentence fragments that cannot be ordered naturally.

Use the project's message/formatting system. Verify date, number, currency, time-zone and unit behavior from actual locale requirements. Do not concatenate untrusted directional text into labels without correct isolation. Mirror only layout/graphics whose meaning changes with direction; inspect icons and mixed-script values individually.

Test long/short messages, missing translations, text expansion, zoom and right-to-left/mixed-script states through the admitted Playwright MCP peer with an isolated profile, or the project’s installed browser test tool when live UI is involved. Check wrapping, truncation, focus order, input/caret behavior and available font glyphs. A translation file alone does not establish a localized journey.

Return changed resources, locale/direction coverage, inspected evidence and linguistic uncertainty requiring review. Stop when required messages and UI paths preserve meaning and use, or report the exact missing translation/font/platform check. Do not invent fluent human approval.
