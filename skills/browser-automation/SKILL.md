---
name: browser-automation
description: "Perform an authorized multi-step website interaction with observed state, scoped actions and reconciliation of uncertain submissions. Skip page reading without interaction."
---

# Browser automation

Use the project's established browser tests/tools for public sites, accounts, forms, pointer actions or submissions. The optional Pi-owned peer, when explicitly onboarded and authorized, is limited to one literal local `127.0.0.1` origin, GET/HEAD requests, screenshots/snapshots and navigation keys; it exposes no pointer click, text entry, script evaluation or POST. Enter and Space can still activate local controls or submit an existing GET form, so keep the grant scoped. Report a missing route when the requested interaction exceeds those controls. Select the intended project browser/profile and account deliberately. Establish the final state, allowed pages/accounts and effect boundary before interaction.

Observe the current page and available controls, perform one meaningful action or a safe bounded group, then verify the resulting state. Ground actions in current observations; re-observe after navigation or material DOM changes. Keep form data and credentials out of logs and captures unless the task requires them.

User-authorized interaction does not authorize unrelated purchases, messages, account changes or bypasses. Preserve the user's exact effect scope. For a submission or other external effect, distinguish intent, attempted action and observed result. If the response is lost, inspect the original operation and destination state before retrying; never treat a timeout or missing confirmation as proof that nothing happened.

Return the achieved state, relevant observation/action evidence and unresolved effects. Stop dependent interaction when authorization, page identity, current state or effect knowledge is uncertain. Report an unavailable browser or test tool as a concrete limit; choose another route only when it still satisfies the task's profile and permission constraints.
