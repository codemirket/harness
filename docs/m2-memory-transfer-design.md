# M2 manual memory transfer contract

This file documents the implemented same-scope recovery workflow. Its historical
filename is retained for existing links. Focused codec, backup and transfer-admin
unit tests and actual CLI/MCP integration have passed on macOS ARM64.
These checks do not establish portable migration, desktop-client activation or
complete M2 qualification.

The owning contract is [memory backup and recovery](../planning/proposal.md#agreed-memory-backup-and-recovery).
Canonical SQLite remains the sole live memory authority. JSON Lines is an explicit
interchange format, not another live store or journal. No model, external helper,
archive dependency or host configuration change is involved.

## Scope and activation

Export reads all records and retained revisions in the explicitly selected
existing scope. Import admits a validated archive as a managed recovery point;
**import does not replace live memory**. The existing explicit restore workflow
preserves current state, then replaces only the selected scope with that point.

The archive's exact opaque `Scope` must match the selected registered project or
explicit shared-preference scope. Wrong or unknown identities are refused. There
is no merge, revision renumbering, automatic registration, name/remote matching or
checkout remapping. Recorded checkout applicability and sibling-worktree history
remain unchanged; sharing a project identity does not authorize reading sibling
files. Fresh-install and differently identified project/checkout mapping remain
undecided and unimplemented.

## Archive format and limits

The format is UTF-8 JSON Lines, version 1, with one LF-terminated strict typed value
per line. Its sequence is:

1. `{"type":"header","version":1,"scope":...}`.
2. Revision frames ordered by record ID and increasing revision. Each has
   `type:"revision"`, `head_revision`, and a full `record`: record ID, revision,
   `RecordInput` data, recorded time, current-revision flag, `supersedes` and
   `superseded_by`.
3. `{"type":"footer","records":N,"revisions":N}`, followed by EOF.

The footer contains counts, **not an internal checksum**. The CLI binds the exact
archive SHA-256 and native input identity in its private intent; import also binds
archive hash/length in the managed snapshot catalog. Hashes identify bytes and
check drift, not authenticity or permission.

Unknown/duplicate fields, unsupported versions, malformed/truncated lines and
trailing content are rejected. Existing content, source-count, source-string,
selector and timestamp validators apply. Limits are 512 MiB (536,870,912 bytes)
per stream, 256 KiB (262,144 bytes) per line and 1,000,000 revisions. The line cap
covers maximum-size canonical fields even with six-byte JSON escaping. Limits refuse
the operation; they never silently omit history. Frames are processed individually
rather than loading the archive into one vector.

Import validates contiguous revision history, consistent heads/currentness and
the supersession graph. Serialized `superseded_by` claims must agree with the graph;
they are then discarded as derived information. Legitimate content-free links to
forgotten absent records, and retired heads without a surviving successor, remain
representable and are reported in transfer statistics. They do not recreate the
missing records.

Import constructs a new SQLite database using Mirket's current initializer and
parameterized inserts. No supplied SQL or external SQLite schema is executed.
Operation confirmations, mutation intent hashes, restore markers and derived
semantic vectors are not imported. FTS is rebuilt from imported canonical data;
semantic coverage requires a later explicit rebuild after restore.

Codec work has a 120-second deadline checked during processing, with a finite
joined watchdog that interrupts a long SQLite statement. Hash/read passes also
have 120-second checks. This is not a promise that every CLI invocation finishes
within 120 seconds: phases have separate bounds, and synchronous filesystem calls
are checked between calls rather than forcibly interrupted.

## CLI workflow

All commands are under `mirket dev memory`. Supply canonical private `--state-dir`
and `--backup-dir`, explicit `--release-id`, and exactly one of `--project-root`
or `--shared-preferences`. A transfer path must be absolute, bounded, have a
canonical existing parent, and lie outside managed state and backups. Imported
archives must be owned private regular files without symlink/hardlink aliases;
use mode `0600`. New export stages are created with mode `0600`. The implementation
uses Unix native identity checks; only macOS execution is qualified by the current
unit evidence. Windows transfer identity is unsupported.

| Command | Additional argument | Behavior |
| --- | --- | --- |
| `export-prepare` | `--path` | Reserve a five-minute intent for a new output file; existing destinations refuse. |
| `import-prepare` | `--path` | Inspect and bind the selected archive's native identity, size and SHA-256. |
| `transfer-commit` | `--operation-id` | Execute a prepared intent under maintenance exclusion; requires its preparing release. Stop runtime sessions first. |
| `transfer-status` | `--operation-id` | Inspect the receipt and available saved authority without replay. |
| `transfer-reconcile` | `--operation-id` | Finish or clean up only previously started, exactly bound work. It cannot start a prepared intent. |
| `transfer-retire` | `--operation-id` | Retire an expired unused preparation under maintenance after checking that no started stage/binding exists. |

Inspect `receipt_phase`, `confirmed`, `complete`, `needs_reconcile` and `authority`.
A receipt phase alone is not saved-effect proof. `complete` requires a complete
receipt and current verified authority. Missing evidence never permits reusing an
old ID as a fresh operation. Inspect/reconcile uncertainty before considering a
new intent; cleanup consumes the old attempt.

For an admitted import, obtain the eligible `authority.snapshot_id`, then use
`restore-prepare --snapshot-id ...`, followed by `restore-commit --operation-id ...`
with the new restore ID. Use `restore-status`/`restore-reconcile` after uncertainty.
These are separate explicit actions; transfer reconciliation never activates an
import. Restore still requires pre-restore preservation within the backup budget
and leaves other scopes unchanged. Reconnect and inspect/rebuild derived coverage
with `memory_index_refresh` after a successful restore.

## Ownership, publication and storage

Export holds maintenance and exclusive scope access while the codec reads a
consistent transaction directly from current canonical memory. It does not create
a duplicate managed backup or reuse an older daily point. Output first goes to a
private sibling `.part` file with durable ownership recorded before content is
written. A verified completed stage is published with an exclusive hardlink,
checked and synced, then its stage link is removed and the directory synced.
Unexpected destinations, replaced parents or changed file identities are refused.
A partial `.part` file is never presented as a complete export.

A verified hardlink at the selected destination is already a user-owned external
copy even if the process loses its final receipt update. Reconciliation inspects
that exact saved identity; it does not overwrite a newer file. Once published,
external exports are outside managed retention and cannot be erased by later
forgetting. User-owned exports and their sibling staging files are outside the
managed-backup catalog's byte accounting; the archive size limit still applies.

Import retains the selected checked descriptor, verifies bytes before, during and
after reconstruction, and never follows archive source locators. It reserves space
in the existing backup catalog before SQLite growth. The database ceiling is
archive size times four plus approximately 1 MiB, rounded to pages; the reservation
is three times that ceiling plus catalog headroom. `max_page_count` enforces the
ceiling. A valid archive can therefore be refused when its construction cannot fit
the remaining aggregate backup budget or this database ceiling. There is no
unaccounted automatic expansion.

An import moves from `copying` to `eligible` only after saved-byte, canonical and
current-exclusion checks. It creates no artificial daily/weekly references and
uses existing eligible rotation/latest-good/pending-restore protection. A definite
codec/hash/validation/exclusion rejection closes SQLite and removes only the exact
owned stage with directory sync. Abrupt interruption or uncertain cleanup retains
the ineligible reservation for exact-ID reconciliation; unknown files are
preserved. Failed or cleaned attempts are never replayed as fresh imports.

### Unfinished export admission

The maintenance owner permits one unfinished export across the installation,
including retained operations from other scopes and releases. Before creating
staging it inspects typed transfer receipts and refuses while another export is
Unknown, Writing or Staged. Only explicit publication or proven cleanup releases
that slot. This bounds Mirket-created unfinished plaintext to 512 MiB without a
second backup catalog or quota service. A create-before-binding crash can leave
an empty unbound directory entry; it is preserved, so this is not a total inode
or filesystem free-space guarantee. Completed external copies remain user-owned.

## Forgetting and recovery

An import containing a currently excluded lineage is rejected in full. It is not
silently filtered or assigned new record IDs. Known rejected content is removed
from the owned managed stage before cleanup is acknowledged. Eligibility is
checked against the current exclusion epoch under scoped exclusion. Imported
recovery points subsequently participate in normal managed-backup forgetting.
Scope restore never rolls back the separate content-free exclusion authority.

Forgetting also invalidates unused export preparations and cleans exactly owned
unpublished writing/staged files. Changed/unknown stage identity is preserved and
prevents a complete forgetting-cleanup claim. A verified already-published external
copy is preserved; only its owned sibling stage link is removed. Publication of an
unpublished stage requires the original forgetting epoch still to match. These
checks prevent later reconciliation from publishing a pre-forget private stage.

The archive does not transfer deletion authority. Fresh installations cannot infer
exclusions absent from their retained authority, and rewritten external copies are
not recognized through semantic fingerprints. No claim covers external-copy
erasure, operating-system snapshots or forensic remnants.

## Source owners and evidence boundary

- `src/memory_transfer.rs`: strict wire framing, all-history cursor, validation,
  fresh canonical construction and SQL/work deadline.
- `src/memory_backup.rs`: scoped export, import source binding, reservations,
  snapshot eligibility/inspection and existing restore/reconciliation.
- `src/memory_transfer_admin.rs` and `src/memory_admin.rs`: issued intent lifecycle,
  native file ownership, publication, cleanup and forgetting integration.
- `src/memory_lifecycle.rs`: durable exclusions and cleanup completion.
- `src/main.rs` and the memory guide: explicit CLI entry points and workflow.

Passing focused unit evidence covers canonical round trips, graph validation and
malformed inputs; bounded accounting, exact incomplete-stage cleanup and import
followed by restore; no imported confirmations; source/destination drift; expired
unused preparation retirement; and forgetting of unpublished stages versus
preservation of verified external copies. Actual CLI prepare/commit/status/
reconcile/restore integration passed through the disposable MCP fixture using
private Node 24.22.1 on macOS 26.5.2 ARM64. The integration binary was
`03d225a061a113597065f71438baa61a2530ac6a46b7330683914ba81d121f05`.
That same run passed configured native indexing, backups and restore, with OS
network denial and no desktop settings changes. The subsequent single-unfinished-
export admission change passed the actual memory/MCP and release-activation
fixtures on binary `fda50b2df131c2bd7bae82001a795e5de9c2e8446a09929ca85cdbabc23ac9d8`,
alongside 301 native tests and strict all-target Clippy. The 256 KiB line correction
then passed all 12 codec tests, including a maximum escaped canonical record,
strict Clippy and the actual memory/MCP journey on binary
`95c2cc6feab974c5df20ca2c75e612aaf7f1d20bd94ac7db95bd9282f765b87f`.
No portable, Windows, large-archive
performance or actual desktop activation qualification is implied.
