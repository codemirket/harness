---
name: backend-engineering
description: Build or change server endpoints, application services and background jobs with explicit contracts, domain invariants and bounded resource use. Use for request-to-storage behavior; use database-systems for database mechanics and engineering-judgment for external integration contracts.
---

# Backend engineering

Trace one representative command from the actual entry point through identity,
validation, domain decisions, persistence and response serialization. Reuse the
project's framework, error contract and service boundaries; a new layer needs an
actual responsibility. Read [the command recipe](references/command-service.md)
for state-changing endpoints or jobs.

Before implementation, write the accepted input, caller-visible output, invariant
and failure responses. Separate transport parsing from domain decisions enough
to test both. Derive actor and tenant scope from verified server context; never
trust an editable ownership field. Reject oversized, malformed or unsupported
input before expensive work. Bound collection sizes and server-side work as
well as request bytes.

Locate the commit point and define the meaning of success. Reuse
`database-systems` for concurrency/transaction design and
`engineering-judgment`'s service-integration reference for remote side effects,
retries, webhooks and outboxes. A canceled request or missing response does not
prove that a committed operation was undone.

Set time/resource budgets at the boundary: request deadline, DB pool wait,
remote call timeout, queue admission and cleanup. Propagate cancellation only
where the operation supports it. On shutdown, stop admitting work, bound drain
time and release resources through the framework's lifecycle mechanisms.
Never put an unbounded retry loop behind an endpoint.

Verify the real endpoint or worker adapter as well as core rules: invalid input,
forbidden scope, conflict, dependency failure, duplicate delivery and serialization.
Inspect errors for secret leakage. Record structured outcome, duration and a
non-sensitive correlation identifier; do not log request bodies by default.
Finish with the changed contract, applicable tests and remaining operational gaps.
