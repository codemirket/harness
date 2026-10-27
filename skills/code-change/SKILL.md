---
name: code-change
description: "Implement substantive code behavior changes by finding owners and consumers, selecting the smallest complete design and verifying affected behavior. Skip purely mechanical edits."
---

# Code change

Find the owning implementation and the consumers that depend on its behavior. Read nearby contracts, current changes and project gates before editing. Describe the concrete before/after behavior and the failure that would make a plausible implementation wrong.

Choose the smallest complete design with one clear owner for each responsibility. Preserve public compatibility and data semantics unless the requested change explicitly alters them. Reuse established project tools and patterns; do not add a new dependency, abstraction or fallback merely to avoid understanding the existing path.

Implement error handling at the boundary that can make the recovery decision. Preserve unrelated work. Select tests from externally visible behavior, invariants and consumers; avoid tests that only restate the new implementation. Exercise a meaningful failure case and the required project gates. Broaden checks when new evidence warrants them, not as an endless ritual.

Report the changed behavior, why the design fixes its cause, actual checks and remaining limits. Stop when the requested outcome and affected contracts are verified. If an environmental prerequisite blocks a required check, keep it explicitly unverified rather than weakening the gate or claiming completion from compilation alone.
