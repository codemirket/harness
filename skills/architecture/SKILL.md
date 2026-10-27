---
name: architecture
description: "Choose or revise system topology and module boundaries when ownership, dependencies or failure tradeoffs change. Skip a local implementation with an established owner."
---

# Architecture

Map the existing components, data ownership, consumers and trust/failure boundaries from source evidence. Identify the concrete pressure requiring a structural decision: coupling, incompatible lifecycles, recovery, scale or changing contracts.

Compare the smallest plausible alternatives against that pressure. Make state ownership, dependency direction, concurrency and failure propagation explicit. Consider migration and rollback costs alongside steady-state behavior. Avoid introducing an abstraction or service without a demonstrated responsibility it owns.

Use a focused decision record or source diagram only when it helps future readers. Prototype the highest-risk boundary with a discriminating failure/recovery case rather than presenting a topology as proof. Keep implementation and documentation consistent.

Deliver the selected boundaries, alternatives/tradeoffs, migration path and verifying evidence. Stop when the structural decision is justified and implementable, or name the missing constraint/measurement. Do not silently replace the user's chosen architecture or expand a bounded change into a rewrite.
