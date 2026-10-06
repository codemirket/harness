# Service and API integration reliability

Read when an application calls an external service, receives events, or exposes
a contract to another service. Reuse the repository's connector, credential,
queue, and contract conventions. A protocol specialist such as MCP adds its own
requirements; this reference addresses the underlying service behavior.

Resolve the actual API/SDK version, account scope, endpoint/environment, and
provider documentation before choosing behavior. Record a missing capability
instead of assuming that a SDK wrapper supplies pagination, retries, or
idempotency. Use the existing reviewed connector or contract workflow when it
fits; this reference does not justify a second integration architecture.

## Contract and authentication at the boundary

For an inventory connector, retain provider IDs and quantities with their units,
and preserve missing, null, zero, and empty values as the schema distinguishes
them. Validate responses before converting them into the domain model. Map
provider failures into stable caller errors without turning malformed data or
partial retrieval into a successful empty list. Keep correlation information
useful for diagnosis without returning credentials or private response bodies.

Authentication proves a principal; authorization determines which account and
resource that principal may use. Bind configured credentials and resource IDs
to the intended tenant, and verify the real provider enforces required scopes.
Use the existing secret mechanism. If a documented token-expiry response permits
refresh, coordinate concurrent refreshes and replay only operations whose replay
is safe. A forbidden response is not evidence that refresh will repair access.

Do not send credentials to an arbitrary URL supplied in a response or caller
argument. Resolve documented pagination/download destinations against the
provider's allowed authorities and credential rules before following them.

## Complete retrieval without inventing coverage

Suppose page one contains 100 items and `next_cursor="p2"`, while page two
contains no items and `next_cursor="p3"`. Follow the documented cursor contract;
page size or emptiness alone may not indicate completion. Preserve filters,
account scope, ordering, and snapshot/version parameters across pages. Detect
a repeated cursor or a violated traversal bound rather than looping forever.

If retrieval stops at a deadline or provider cap, return/report partial coverage
explicitly. Deduplicating by stable ID can handle repeated items when appropriate,
but cannot recover records skipped while the collection changes. Use a provider
snapshot or watermark only when supported, and state the remaining consistency
limit. Test a multi-page traversal, repeated cursor, partial failure, and changing
collection where those cases affect the consumer.

## Rate limits, timeouts, and retries

Assign a bounded request deadline and retry budget at the layer that owns the
operation; nested SDK and application retry loops can multiply traffic. Apply
the provider's limit scope, concurrency rules, and documented backoff. Interpret
`Retry-After` according to its supported seconds/date form, and account for clock
handling. If the required wait exceeds the task's deadline, defer or report the
limit instead of retrying immediately. Use the project's jitter policy for
transient retries to avoid synchronized bursts.

Classify retries by the operation and failure evidence, not HTTP status alone.
Retrying a read may be appropriate; an invalid request or forbidden access usually
needs a different action. Cancellation and a client timeout do not undo a remote
write that the server already accepted.

## Worked example: a write whose response is lost

A shipping request uses a durable local operation ID and the provider's documented
idempotency key. The provider creates shipment `S42`, then the connection closes
before the client receives a response.

1. Retain the same logical operation, key, and request payload. A new key would
   represent a new operation; a changed payload may conflict with the existing key.
2. Mark the outcome unknown. Reconcile through a documented key/status lookup,
   or repeat the same request only if the provider contract guarantees safe replay.
3. Record the confirmed shipment ID and reconcile local state. Do not report a
   failure merely because the original response was lost.

Check the key's scope and retention window. If the provider has no reliable
deduplication/status mechanism, do not manufacture an exactly-once guarantee.
Stop automatic replay when it could create another shipment and expose the
unresolved operation for the repository's recovery path.

A meaningful fixture commits the remote write before dropping the response.
Verify recovery yields one intended shipment and the caller sees its confirmed
status. A test where the request fails before any send exercises a different case.

## Worked example: duplicate and reordered webhooks

Verify the provider's signature against the required raw bytes and its documented
timestamp/replay rules before trusting the payload. Bind the event to the expected
provider account and resource; a valid signature alone does not grant access to
every local tenant. Preserve the event identifier and schema/version.

If event `E7` is delivered twice, processing should produce the contract's single
logical effect. Record deduplication and the effect atomically where the existing
storage supports it, or use the project's durable inbox/outbox recovery pattern.
Acknowledging receipt after durable enqueue is different from claiming processing
succeeded; a failed enqueue must not receive a misleading success acknowledgment.

If a shipped event arrives before an older packed event, follow a documented
resource version or retrieve authoritative state rather than letting arrival
order move the shipment backward. Provider timestamps from different clocks are
not automatically a safe conflict rule. Check duplicate delivery, reordered
events, signature failure, and a crash between effect and acknowledgment.

## Evidence for the final claim

Separate schema/contract checks, sandbox behavior, credential configuration, and
live operational evidence. A mocked success does not establish working scopes,
provider key retention, webhook reachability, or a remote side effect. Exercise
only authorized environments and operations; report material limits alongside
the confirmed consumer outcome.
