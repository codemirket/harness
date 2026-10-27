# Bounded same-candidate staging recovery

An explicit retry of `mirket dev stage --home HOME --source SOURCE` can resume the same
verified release after an interrupted local copy. The source is fully verified
again and must have the same typed release identity; its absolute location may
change. This does not activate the candidate. The update writer lock, release
capacity, active/previous retention and maintenance/session exclusion remain in
force.

The owner keeps one at-most-4096-byte `state/staging.pending.json` journal and
one at-most-4096-byte replacement record with the `.next` suffix. The strict
schema binds the candidate release identity, the private stage directory's
native device/inode, a checked revision and one phase:

| Phase | Retained contents |
| --- | --- |
| `preparing` | Any subset of the candidate's exact private directories; no payload files or links |
| `copying` | All files before the recorded index are complete; the exact current file's device/inode is recorded before copying bytes; later files and links are absent |
| `linking` | Every file, including the normalized manifest, is complete; only exact admitted interpreter links may exist |
| `ready` | The complete installed inventory and nested directory entries have been verified and flushed; the payload can be renamed to its immutable release directory |

Files use the verified manifest's sorted order, followed by the normalized
manifest itself. Before recovery writes, the owner inspects the entire retained
tree. Completed files must retain their exact digest, length and installed mode.
The current file must remain a private, regular, single-link file with the
recorded device/inode. Its bytes must match a prefix of the newly verified
source, its length cannot exceed the declared length, and its mode must be the
initial `0600` or its installed mode. A full-length current file is checked by
digest too; size alone cannot establish completion.

Only that bound incomplete file can be truncated and copied again, preserving
its inode. A complete matching current file needs no content rewrite; its
installed mode and flush can be finished. Completed earlier files are reused.
Missing owned files, different modes or content, hard links, replaced inodes,
unexpected symlinks, unknown entries and another candidate are refused and
preserved. There is no stage-tree deletion or automatic cleanup of user paths.

A fully written `.next` journal can be consumed only if it is identical to the
current journal or its exact next revision/phase transition with the same
candidate and directory identity. The entire retained tree must satisfy that
selected phase first. An initial preparing record can also be recovered from
`.next`. Torn JSON, unsupported schemas, unowned records, oversized records and
unrelated successors remain for inspection.

The `ready` journal also covers interruption after the directory rename but
before `staged.json` is durable. A retry verifies the immutable release and the
recorded directory identity, finishes the staged pointer, then retires only the
validated staging journal records. A preexisting `staged.json.next` still follows
the activation record's existing exact-byte reconciliation rules. Ordinary
server launch performs no staging recovery or activation.

## Capacity and limits

The disk preflight counts the unfilled payload bytes, the existing entry
allocation allowance (4096 bytes per file/link/directory plus 1 MiB), and
launcher transaction/recovery space: one candidate launcher temporary plus twice
the current stable launcher's byte length for a backup and restoration
temporary. Already completed matching files do not consume a second full-copy
allowance on retry. Arithmetic is checked. `statvfs` remains an observation under
the writer lock, not a filesystem reservation; unrelated writers, allocation
behavior and disk errors can still interrupt copying or later activation.

Recovery is deliberately conservative at ownership establishment. Interruption
after creating a stage directory or new file but before its complete ownership
journal is installed can leave an unbound entry; a torn journal can do the same.
Those cases require inspection and are never guessed away. The feature resumes
copy interruptions with durable ownership; it does not promise recovery from
every instruction boundary or physical power-loss pattern.

The implemented native device/inode binding is qualified on macOS ARM64.
Interrupted recovery is explicitly refused on non-Unix platforms until native
file identity behavior is implemented and qualified. Successful fresh staging
retains the existing platform behavior. Path checks and native identities are
ordinary corruption/drift protection within a private installation, not
containment against a hostile process that can rewrite the installation and its
journals. Per-file durable journal transitions add writes; the isolated buffering
measurement below does not qualify throughput for a complete release.

## Verification

Focused Rust tests simulate copy interruption and verify inode reuse for both
completed and incomplete matching files; complete-current digest refusal;
unknown, corrupt, missing, aliased, mode-changed and substituted files; another
candidate; unjournaled state; complete and torn successor records; ready-stage
and post-rename recovery; and active/previous/launcher preservation while a live
session blocks activation. Existing activation transaction and normalization
tests remain part of the same suite.

On 2026-10-25, the build-input guard and formatting passed with the locked Rust
1.99.1 toolchain on macOS ARM64. `cargo test --locked --offline --lib
release_store::tests` passed all 17 tests. After integration, the full Rust
suite passed 491 library tests with three opt-in tests ignored, and strict
all-target Clippy passed. A fresh four-component release was installed and
reported healthy by `doctor --json`; `tests/installed-maintenance-smoke.mjs`
passed disposable setup, busy staging, idle activation, launcher health and
drift refusal. The native Codex backend and installed MCP smokes passed on that
release. This exercises the new staging path on a complete payload; the
interruption cases remain focused fault-injection tests rather than physical
power-loss trials.

These focused tests do not establish native Windows recovery, actual desktop
activation, physical power-loss guarantees or staging throughput for a complete
multi-component release.

## Targeted buffering measurement

On 2026-10-25, a disposable macOS ARM64 probe compared the staging path's
type-erased, byte-bounded file reader with the same reader wrapped in a 64 KiB
`BufReader`. Across six alternating pairs of fresh 64 MiB destination files,
median copy time was 0.236 seconds for the original generic `io::copy` path and
0.063 seconds for the buffered path. Source bytes were already cached; file
sync was timed separately, and these figures exclude manifest verification,
journaling, permission changes, directory sync and activation. They are an
isolated copy comparison, not an end-to-end release speedup.

Six further alternating pairs ran the complete `dev stage` command with the
baseline and updated debug binaries, cached sources and fresh disposable private
homes. Median elapsed time for the four-file 64 MiB fixture changed from 1.776
seconds to 1.623 seconds; the four-file tiny fixture changed from 0.292 seconds
to 0.290 seconds. Home initialization and cleanup were excluded. These local
fixture results do not predict throughput for a release with thousands of files.

Staging now uses that buffered reader. Destination digests use a 64 KiB scratch
buffer as well, reducing per-file scratch initialization. The copied-byte limit,
destination hash check, final installed verification, private file creation,
inode binding and all file/directory/journal sync calls retain their ordering.
The boundary regression fixture covers a fresh nonuniform payload with a short
tail beyond two buffers and same-inode recovery from prefixes immediately
before, at and after the first buffer boundary.

The build-input guard, formatting check, all 18 focused release-store tests,
locked offline binary build and strict all-target Clippy passed after this
change on macOS ARM64 with Rust 1.99.1. No physical power-loss test or other
platform qualification was performed.
