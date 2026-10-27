---
name: reliability-incident
description: "Diagnose an operational failure or implement resilience/recovery when state transitions, replay or rollback determine the result. Skip feature work without an operational failure boundary."
---

# Reliability and incidents

Establish the affected users, current severity, exact environment/revision and known effect state. Preserve decisive logs/state without collecting unnecessary private data. Separate immediate containment from root cause and permanent repair; keep production effects within authorization.

Trace state transitions, ownership and commit boundaries. Identify what is safe to retry, what may have partially happened and which evidence can reconcile it. Missing acknowledgements or receipts do not prove no effect. Inspect actual destinations before replay or rollback.

Use a representative disposable/staging path to test interruption, crash, duplicate request, resource exhaustion or unavailable dependency as relevant. Verify recovery preserves invariants, cleans up only owned resources and does not affect unrelated jobs. A restart or green health check alone does not establish data/effect consistency.

Return impact, causal evidence, mitigation/repair, observed recovery and remaining risks. Stop when service outcome and consistency are verified or a specific authority/access gap blocks recovery. Do not silently discard damaged state, replay uncertain external effects or lower release gates to end an incident.
