---
name: backend-engineering
description: "Implement service, background-job or concurrency behavior when state ownership, consistency and failure recovery determine correctness. Skip UI-only work."
---

# Backend engineering

Identify the owner of canonical state and each transition's preconditions, commit boundary and observable result. Trace callers, workers and downstream effects. Make concurrency and consistency expectations explicit before selecting locking, transactions or queues.

Keep durable intent separate from transient process state. Define timeout, cancellation, duplicate and crash behavior at the owner that can reconcile it. Do not equate cancellation acknowledgement with rollback or a transport timeout with no effect. Preserve bounded resources and meaningful error context without leaking secrets.

Exercise real boundaries with representative success and a discriminating race, replay or recovery case. Check that one request/job cannot corrupt another's state or consume unbounded work. Use the project's established runtime and required gates.

Deliver the state/behavior change and observed failure/recovery evidence. Stop when requested invariants and consumers pass, or report the exact unavailable service/environment check. Avoid new infrastructure or hidden retries unless the actual contract requires and authorizes them.
