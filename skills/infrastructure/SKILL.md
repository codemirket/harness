---
name: infrastructure
description: "Change deployment or environment configuration when ownership, drift, rollout and recovery affect a running system. Skip application-only changes and passive configuration reading."
---

# Infrastructure

Identify the desired state, current owner, environment scope and sources of configuration truth. Inspect drift and dependent services before planning a change. Keep secrets in the established secret mechanism and avoid exposing account state in evidence.

Choose the smallest configuration change with clear rollout, observation and recovery criteria. Validate syntax and a plan/diff before mutation. Separate staging evidence from production behavior and preserve explicit authorization for the target environment.

Exercise the relevant failure mode in a safe representative environment: denied access, missing configuration, partial rollout or interrupted recovery. Verify actual service health and state convergence, not only a successful command. Reconcile uncertain external effects before repeating deployment actions.

Return the reviewed change, target identity, drift/rollout evidence and recovery limits. Stop dependent effects at an unauthorized environment or unknown state. Do not replace the user's infrastructure provider or install a new control plane to avoid a missing capability.
