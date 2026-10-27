---
name: outcome-verification
description: "Verify a claimed completion against actual saved behavior, artifacts or effects, including a convincing failure case. Skip isolated facts with no completion claim or changed outcome."
---

# Outcome verification

Translate the requested outcome into observable evidence at the consumer boundary. Verify the actual saved output or running behavior at the current revision; a prior capture becomes stale when its source changes. Keep required project gates intact.

Choose a discriminating failure case that could expose a convincing fake: a changed input, denied permission, duplicate submission, stale revision, missing asset or broken formula as appropriate. Prefer an independent oracle, alternate extraction or downstream consumer where it meaningfully checks the result. A receipt, file hash, successful process exit, screenshot or agent verdict alone cannot establish semantics, access, editability or human acceptance.

For interfaces, exercise the requested journey and inspect relevant viewports/states. For paged or visual artifacts, reopen and inspect the saved file through the suitable host/renderer. For data, reconcile units, totals and formulas. For effects, observe the resulting system state and reconcile uncertainty before replaying.

Report what was checked, actual results, platform/access limits and untested paths. Stop when the requested outcome and required gates are demonstrated; repeat only checks affected by subsequent changes or new failures. If a required observation is unavailable, state the concrete gap and avoid an unqualified completion claim.
