---
name: root-cause-debugging
description: "Diagnose an unexplained failure using a controlled reproduction, cause trace and discriminating hypothesis. Skip an already-understood mechanical correction."
---

# Root cause debugging

Capture the failing input, exact revision, environment, expected behavior and actual result. Reproduce with the smallest case that preserves the failure. If reproduction is unavailable, label observations and hypotheses separately instead of inventing a local success.

Trace the failed invariant backward through state and ownership. Compare a working case or last known good revision when it isolates the difference. Form a hypothesis that predicts an observable result and choose a probe that distinguishes it from the closest alternative. Change one causal variable at a time; retain meaningful failed hypotheses so they are not repeated.

Fix the owning cause after the evidence supports it. Avoid stacks of symptom patches, arbitrary sleeps and unrelated cleanup. Verify that the original failure now succeeds and that a nearby counterexample still behaves correctly. Run the affected project gates.

Return the cause chain, decisive evidence, fix and remaining uncertainty. Stop when the failure and its cause are discriminated and the fix is verified, or when a specific missing input/environment prevents progress. Do not keep rerunning an unchanged experiment or claim a cause from correlation alone.
