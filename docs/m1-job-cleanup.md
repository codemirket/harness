# M1 finite artifact-job scratch cleanup

`src/engine.rs` now disposes known private text/PDF-job scratch at a confirmed
terminal boundary and releases expired in-memory preparation capacity. It does
not scan historical jobs or run a cleanup daemon.

## Ownership and disposal

Each execution exclusively creates `state/jobs/<issued-operation-id>`; an
existing directory is never adopted. The source snapshot is created exclusively
with Unix mode `0600`. The live execution retains native identity and metadata
for the state/jobs directories, source file and fully verified adapter output.
The source file's opened-handle identity must still match its path after writing.
Text output must match the exact prepared replacement. PDF output must pass its
typed result, revision, size and saved-byte checks. Both must remain unchanged
during verification before they become eligible for disposal.

The worker's `cleanup_confirmed` result survives the Engine boundary. Disposal
requires all of the following:

- The child has confirmed cleanup, or no child was launched.
- A terminal receipt has been saved successfully.
- The external publication effect is `not_attempted` or `confirmed`.
- The original private directories and known files still match their ownership
  evidence, without symlink or hardlink aliases or unexpected directory entries.

The Engine preflights every entry before removing either known scratch file,
then removes the empty operation directory. It never uses recursive deletion.
Project source/output paths, project publication stages, operation receipts,
other jobs and ownership/admission markers are not cleanup targets.

Unconfirmed child termination, unknown publication, partial/unverified output,
substituted entries, extra files and failed terminal persistence retain recovery
material. `scratch_cleanup` is an optional bounded receipt/report field recording
completion or the reason for retention. A cleanup failure does not change a
successfully published artifact into a failed or unattempted operation. If the
secondary cleanup-notice save fails, the earlier terminal receipt conservatively
continues to report retained material pending confirmation.

Receipt saves reject terminal-state changes and publication-effect downgrades
under the operation-store lock. This prevents a late cancellation holding an
older snapshot from erasing the evidence that authorized scratch disposal.

## Expired preparations

A pending intent retains its original five-minute expiry. Before applying the
64-intent connection capacity limit, a new valid preparation drops expired
in-memory plans. An explicitly attempted expired start also drops its pending
plan and refuses execution. Durable receipts remain available; expiry does not
reset an ID, authorize replay, reclaim durable receipt capacity or clear an
unresolved ownership marker.

## Verification

On 2026-10-24, macOS ARM64:

```sh
cargo test --locked --lib engine::tests
cargo clippy --locked --lib -- -D warnings
```

All **15 Engine tests passed**, and production-library Clippy passed with
warnings denied. Tests distinguish confirmed disposal from retained unknown
child/publication outcomes; preserve project outputs, original source and other
jobs; retain partial/unrecognized/substituted entries; refuse symlink/hardlink
aliases and adoption of existing job directories; retain scratch when receipt
persistence fails; preserve confirmed success despite incomplete cleanup; refuse
late evidence downgrades; exercise cancellation before worker launch; and recover
64-slot pending capacity without deleting receipts or replaying expired IDs.

These focused tests did not rebuild the production binary. Native MCP text-job
regression checks must use the subsequently rebuilt binary. The same-binary
worker's separate process tests own proof of child termination; a scratch path
or a successful deletion is not such proof.

## Limits

Ownership evidence is held by the live execution. Abrupt Engine death can leave
scratch for explicit reconciliation; this change does not infer ownership from
filenames or reclaim abandoned directories. Path-based standard-library checks
detect ordinary substitutions but do not provide containment against hostile
concurrent ancestor replacement. Native directory-handle qualification remains
separate. Disposal without qualified native file identity is refused, and no
Windows cleanup qualification is claimed.

## PDF integration extension

The same lifecycle now owns binary PDF jobs. Native MCP tests cover controlled
partial output followed by explicit cancellation, graceful client disconnect and
abrupt server termination. All preserve the unverified stage, avoid destination
publication/replay, stop the leaf and permit a subsequent real PDF inspection.
The abrupt case reports an interrupted receipt with unknown scratch-cleanup
confirmation; it does not reclaim abandoned recovery material.
