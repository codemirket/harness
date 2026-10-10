# Worked mobile lifecycle case: interrupted order submission

Consider an app that lets a field user submit an order with intermittent service.
The user must not get two orders because the app was suspended after the server
committed but before the response arrived. This example defines behavioral
invariants; it does not prescribe a database, queue library or platform framework.

## Distinguish intent from its visible screen

An order draft remains editable until the user submits it. At submission, create
a durable operation containing the account/tenant, operation ID, payload and
payload version. The operation is an immutable attempt to perform that intent;
editing a draft later must not change a payload already in flight. Show a state
the user can understand, such as waiting to send, checking result or completed.

| Event | Durable interpretation | Incorrect shortcut |
| --- | --- | --- |
| Request times out | Outcome may be unknown | Treat timeout as rejection and create a new ID |
| User navigates away | Screen is gone; operation may continue | Delete the only record of the pending intent |
| OS kills the process | Restore from committed local state | Depend on an in-memory callback to finalize |
| Connection returns | Reconcile or retry under the contract | Assume connectivity means the API is reachable |
| Account changes | Pending work belongs to the original identity | Replay old work using the new user's credential |

Do not promise exactly-once delivery across the network. Define an idempotent
business effect: the same authenticated principal, operation kind and ID map to
the same accepted payload and result. The server must reject incompatible reuse
and commit its deduplication record with the local effect atomically. If an external
provider owns the final effect, local acceptance alone does not prove completion;
use its supported identity/reconciliation contract.

Choose operation retention and retry limits from the business workflow. If the
server forgets a deduplication key before a queued client may replay it, retries
can recreate the effect. When a limit expires, expose a recoverable unresolved
state or reconcile before creating a new intent; do not silently reset identity.

## Bind recovery to the account and lifecycle

On foreground or startup, load durable pending operations for the active account.
Schedule work through the platform's supported mechanisms, but keep correctness
independent of a background scheduler running promptly. Back off transient failures
without retrying a permanent validation rejection. Avoid multiple screens/workers
claiming the same operation concurrently unless deduplication handles them.

At sign-out, determine the product's explicit choice: retain encrypted pending
work for that identity, cancel unsent work, or require reconciliation. Do not
invent a data-loss policy. Clear or retain credentials using the approved account
lifecycle without placing tokens in the operation log. A canceled local future
cannot revoke a request already accepted by the server.

A notification or deep link arriving during sign-in should retain only the
validated intended destination, then re-check resource access after authentication.
Opening an order for another account must not switch identities silently or expose
cached details. Re-delivering the link should not resubmit the order.

## Verification that exposes the real defect

Use an isolated fixture with an observable order count and operation result.
Arrange for the server to commit, suppress the response, kill the app process,
relaunch and recover. The proof is one durable order plus the recovered stable
result, not merely one client HTTP invocation. The probe should fail if a new
operation ID is generated on every retry.

Vary the nearest consequential boundary:

- Retry the same ID and payload, then the same ID with a changed payload.
- Change accounts before replay; confirm no work crosses identities.
- Interrupt a local schema upgrade with an older pending operation fixture.
- Deliver the deep link while logged out, already on the screen and after process
  recreation; confirm one destination and no unintended write.
- Exercise permission denied/revoked only if this order flow uses a gated feature.

Use the platform's process/lifecycle controls, not only a React component remount
or an activity recreation, when the claim concerns whole-process death. Record
which event actually occurred. A simulator can establish much of this contract;
hardware checks remain necessary for claims tied to radio conditions, background
execution policy, release responsiveness or physical peripherals.

Report local database state, server state and UI state separately. For example,
"server has one order; app recovered pending status but has not reconciled its
receipt" is incomplete recovery even if all transport retries finished.
