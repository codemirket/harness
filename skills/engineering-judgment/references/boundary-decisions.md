# Boundary, state, and failure decisions

Read for changes to an interface or state transition whose consequences reach
another caller, process, or stored record. Use the project's owners, contracts,
and existing implementation; these examples are decision aids, not a new
architecture to install.

## Find the owner before adding a layer

Suppose two screens create the same export. Disabling each button prevents a
double click in that screen, but cannot coordinate a second tab or a retry after
a timeout. The export service owns duplicate-operation handling; the screens
own pending feedback and recovery. A shared client helper can carry the operation
key, while the server must enforce its meaning. Moving both into a generic UI
state abstraction would hide the distinction without solving it.

Trace where the invariant is enforced and which callers bypass that location.
Keep validation near the boundary that receives untrusted input, domain rules
with their owner, and display choices with the consumer. Add a seam when it
contains a real responsibility such as provider error translation; a pass-through
wrapper with one caller needs another concrete benefit.

## Work through states, including uncertain effects

An illustrative export operation has these observable states:

| State | Evidence | Permitted next action |
| --- | --- | --- |
| Not submitted | No send attempted | Submit the intended operation. |
| Pending | Accepted operation ID; no terminal result | Poll or receive completion under the existing contract. |
| Completed | Terminal status and an accessible artifact | Offer the artifact; verify its contents where the task requires it. |
| Rejected | Definitive validation/authorization failure before acceptance | Correct the input or access; preserve the original request. |
| Outcome unknown | Connection lost after the send; no definitive result | Reconcile the same operation before deciding whether to resend. |

Do not infer rejection from a transport timeout or completion from an accepted
request. Choose state names from the real contract. For optimistic UI, distinguish
displayed intent from confirmed durable state; an older failure must not undo a
newer successful edit. Define which owner resolves uncertainty and what the
caller can safely do while it remains unresolved.

For concurrent edits, identify the conflict policy: an expected version, a lock,
a domain merge, or a deliberate last-write rule. Do not silently adopt the last
response to arrive as that policy. For a storage migration, trace old readers,
new readers, partial conversion, rollback, and interrupted execution before
choosing when old data can be removed.

## Compare a contract change through callers

If a lookup previously returned `None` for a missing record and will now raise
`NotFound`, inspect callers that branch on the return value, adapters that map
the result to HTTP, and command-line exit behavior. A test of the new exception
alone misses the caller that now crashes. Decide whether compatibility requires
an adapter, a versioned interface, or an authorized breaking change; preserve
distinctions between missing data, forbidden access, and provider failure.

Keep numbers, units, precision, nullability, ordering, and identity intact across
the boundary. An amount of zero is not missing; an empty page is not necessarily
the end of a cursor traversal. Validate these against the actual schema rather
than normalizing away unfamiliar but legitimate states.

## Verification and completion example

For the export above, choose checks that separate plausible wrong implementations:

- Two concurrent submissions for one logical operation create the allowed number
  of exports under the server's deduplication contract, not merely one UI call.
- A provider accepts the export and the connection drops. Reconciliation finds
  that export; retry logic does not create a different operation.
- A second account cannot inspect the first account's operation or artifact,
  even when it supplies a valid-looking operation ID.
- A terminal success can be consumed by the real caller. If the user requested
  an exported file, inspect that file rather than stopping at the job status.

Use controlled fixtures or a sandbox for effects; do not exercise production
writes solely to prove the example. State what a substitute establishes: a fake
provider can verify retry decisions, but cannot prove the real provider retains
idempotency keys for the assumed duration. Name that contract or runtime gap.

Completion means the requested consumer outcome and applicable project gates
are satisfied. Evidence may include tests, a rendered result, a scoped request,
or a delivered artifact. Configuration written, service started, operation
accepted, and outcome verified are separate claims. Continue the authorized
steps needed for the outcome; do not broaden the task after it is established.
