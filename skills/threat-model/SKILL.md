---
name: threat-model
description: "Model a new or changed trust boundary or sensitive workflow through assets, attacker capability, misuse paths and mitigations. Skip unrelated routine edits."
---

# Threat model

Identify assets, owners, actors, entry points, trusted components and external effects. Trace actual data/control flow from source and deployment context. State attacker capabilities and access assumptions explicitly; a model that assumes away the affected boundary is not useful.

Describe realistic misuse/failure paths, including authority confused by untrusted input, replay, stale identity, credential exposure and recovery abuse where relevant. Prioritize by impact and plausible reachability. Keep facts, assumptions and unanswered architecture questions separate.

Choose mitigations at the component that can enforce the boundary. Define a negative test or observation for each material control, along with residual risk and an owner. A diagram or tool schema cannot establish authorization, sandboxing or human approval.

Return the scoped threat model, prioritized paths, mitigation/verification plan and gaps. Stop when the changed boundary is actionable or missing architecture/access evidence prevents a conclusion. Do not broaden into an unrelated audit or perform offensive external actions from a modeling request.
