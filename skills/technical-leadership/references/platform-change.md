# Worked technology investment and integration decision

This is a hypothetical example, not a recommendation to adopt a particular
platform or a claim about real prices. Use its reasoning when a proposed
foundation competes with a near-term delivery obligation.

## CTO decision: replace a synchronization platform

A product must add offline order capture next quarter. The existing backend works
but requires repeated adapter code. The team proposes replacing it with a generic
event platform. The supplied planning assumptions are 90 available engineering
days after routine support and 55 days for the agreed customer journey.

| Option | Additional engineering days | Total with journey | Decisive uncertainty |
| --- | ---: | ---: | --- |
| Continue existing integration | 0 | 55 | Can the current retry contract prevent duplicate orders? |
| Add one durable operation adapter | 12 | 67 | Does the existing store support the required atomic write? |
| Replace platform this quarter | 60 | 115 | Migration/dual-running estimate and staffing availability |

The rewrite exceeds the stated capacity before considering risk or preferences.
It is not made feasible by assigning a higher architecture score. The adapter
leaves 23 planned days unallocated; those days are not guaranteed contingency if
the underlying estimates are wrong. Existing payroll is not counted again as a
new cash outflow, although displaced delivery remains an opportunity cost.

The first useful experiment is narrow: can one order intent be atomically stored
with its result, then safely replayed after a client timeout? If that invariant
cannot be supported, the adapter estimate and recommendation must change. A new
platform's diagram or benchmark does not answer this question for the actual store.

The supported decision could be to implement the adapter and defer the broader
platform decision until the measured incident and maintenance costs justify it.
Record the strongest objection: continuing the current platform may accumulate
adapter complexity. Give that objection a review trigger, such as measured effort
across additional integrations, rather than an invented universal rewrite date.
The owner still decides any expenditure or changed business commitment.

## Senior lead decision: make the adapter usable across boundaries

Define the contract before splitting work. A concrete example is:

- Client persists an operation ID with account, operation kind and immutable
  payload before sending; retrying the same intent reuses that ID.
- Server scopes the key to its authenticated principal and operation, binds it to
  a payload digest and commits the effect and retrievable result atomically.
- Reusing a key with a different payload returns a defined conflict; it does not
  silently replace the previous order.
- An uncertain response permits status reconciliation or replay under that
  contract. It does not prove the first write failed.
- Old clients remain supported during the agreed compatibility window; migration
  of stored operations has an explicit reader/writer order.

This is one possible contract, not a mandatory architecture. Use the actual
database's atomicity and provider limits. If the effect crosses an external
system, distinguish local acceptance from the external effect and reconcile it.

Keep the contract/schema with one owner. A backend worker can implement atomic
deduplication while a mobile worker implements durable local intent after both
agree on states and test fixtures. A read-only reviewer can inspect compatibility
and the failure path. Do not split two workers across the same shared schema file
to create apparent parallelism.

## Evidence that closes the work

The critical probe commits an order, drops the response, restarts the client and
retries the same operation. Observe one durable order and the same eventual
result. Also probe a changed payload with the same key and an account switch.
A happy-path HTTP 200 or a mocked invocation count cannot establish these facts.

Check the relevant consumer versions, not only the new endpoint. If integration
changes the error enum after a worker's tests pass, rerun affected callers and
dispatch branches. Do not rerun an unrelated unchanged rendering check merely
because another agent finished.

A useful handoff says which contract was implemented, the source revision and
dirty inputs tested, the actual failure/recovery result, and what remains outside
the tested environment. If only a local service ran, record local acceptance;
deployment and production reconciliation require their own authority and evidence.
