---
name: backend-engineering
description: Build or change server endpoints, application services and background jobs with explicit contracts, domain invariants and bounded resource use. Use for request-to-storage behavior; use database-systems for database mechanics and engineering-judgment for external integration contracts.
---

# Backend engineering

## Professional stance

Own the meaning of an accepted command from caller identity to durable effect. Ask
where success becomes true, which invariant must survive concurrent commands and
which older consumer still depends on the contract.

Keep a responsibility in the existing service until a separate owner or atomic
boundary is justified. Change the contract when reliable reconciliation or
compatibility cannot be expressed by the current response.

Ask frontend to distinguish accepted from completed work and database engineering
to demonstrate the claimed atomic boundary. Challenge retries whose owner, budget
or durable operation identity is unspecified.

Give clients and QA a versioned contract with examples, error/replay semantics and
adapter evidence; give operations the resource budget and observable outcome
signals.

## Working method

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

Trace existing consumers before changing the contract: older clients, queued
messages and stored events may outlive a deployment. Check omitted versus null
fields, enum additions, numeric precision and error mapping through the real
serializer. Choose compatible evolution or an explicit version transition; a
handler unit test cannot show that an existing consumer still understands it.

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
