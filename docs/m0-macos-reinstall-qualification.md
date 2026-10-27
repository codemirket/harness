# Same-home reinstall after complete uninstall

Same-home reinstall is an explicit installation lifecycle operation. It preserves
the existing `state` and `backups` directories and project skill selections/copies.
It does not turn an interrupted uninstall or an arbitrary nonempty home into a
fresh setup destination. The existing absent/empty-home setup rule remains.

The implementation owner is `src/installation_reinstall.rs`. The public actions
are `reinstall`, `reinstall-status` and `reinstall-reconcile`, each with explicit
`--home` and `--user-home`; mutations also require `--source`. Reconciliation must
receive the exact originally selected release identity, although the same source
bytes may be supplied from another canonical source directory. No download,
publication, dependency installation or admission override is part of this path.

## Admission and state transitions

The uninstall owner must first prove its original operation complete: valid
plan/progress records, their actual payload and native integration absent, the
same explicit user home, and no unfinished atomic-record writes. The reinstall
owner also requires empty owned release/staging containers and passes the new
verified release/catalog through native destination preflight.

Before any installation effect, a bounded private `state/reinstall.json` intent
binds the home/user, prior release, original uninstall-plan/progress hashes and
new candidate release. It advances through staging, activation, global skills,
guidance, registration, verification and archival. Only the exact next phase is
accepted from an interrupted `.next` write. The original uninstall records remain
unchanged until the candidate's complete installed manifest, launcher and all
native receipts/copies have been verified.

The existing native-operation lock and maintenance guard remain held across the
journey. ReleaseStore writer guards protect intent changes, its existing release
effects and final archival. Narrow nonconstructible permits authorize only the
saved candidate and correct staging/activation phase; they do not bypass source
admission, payload hashes or existing release journals.

The live reinstall marker blocks ordinary startup, setup/update mutation and old
uninstall reconciliation. This remains true after new activation and during
partial archival. Errors preserve the original intent and affected owners'
staging/activation/native journals; an explicit reconciliation resumes those
owners rather than silently starting another candidate or undoing unknown edits.

After full verification, the exact uninstall plan/progress move to
`state/reinstall-history/<uninstall-plan-sha256>/`. Moving the committed reinstall
intent there is the final admission point. The old uninstall can then no longer
act on the new payload. If the response is lost after that rename, status and
explicit reconciliation verify the archived receipt against actual current
installation health and return the completed result without another effect.
History is bounded to 32 retained operations, with an explicit receipt sequence
for repeated same-candidate reinstalls. Unknown entries in the archive being
finalized and malformed completed history records are preserved and refused
rather than silently pruned.

## Verification scope

Focused tests in the owning module cover a SQLite sentinel row, protected backup
bytes and unrelated files/configuration surviving the complete lifecycle;
lost responses after staging, activation and native effects; interruption after
either uninstall-record archive move; and a lost final completion response.
They also cover pending-uninstall refusal, changed candidate identity, edited
managed native files, changed original uninstall records and unknown archive
entries. Pending intents must block normal sessions, ordinary staging and the
old uninstall before reconciliation. Foreign pending native receipts are
rejected before their reconciliation, preserving their exact bytes. Seven
focused reinstall tests passed as part of the final full source checks.

These fixtures exercise the saved-state owners on macOS ARM64 with synthetic
release core bytes; the separate actual CLI/MCP journey below verifies the
installed executable and real canonical recall/forgetting path. Codex Desktop GUI
activation, the proposed macOS floor, Windows ownership, cross-version state
migration and semantic-model behavior remain separate release gates. Torn or
different journal bytes fail closed and remain available for inspection;
arbitrary storage corruption is not automatically repaired. The user's existing
installation is outside this work.

## Actual installed CLI and canonical memory

On macOS **26.5.2 ARM64**, the reusable
[`reinstall-installed-smoke.py`](../tests/reinstall-installed-smoke.py) assembled
a disposable current-core release with the existing pinned private Node/text
adapter and authored native skills. Its runtime inputs came from the retained
uninstall qualification source, verified against that source manifest before
copying. No upstream installer or dependency download ran.

The installed MCP saved two real sourced project facts, forgot one through the
actual prepare/commit owner and closed normally. The installed launcher then
completed self-uninstall. The verified external source executable ran
`reinstall --home ... --user-home ... --source ...` against that same private home.
All **17** preexisting retained state/backup files stayed byte-identical,
including the canonical store and **three managed backups**. Installed doctor
reported healthy; status and completed reconciliation reported `reinstalled`.
The reopened installed MCP returned the retained fact's actual content and
continued to refuse the forgotten record. Personal configuration, guidance and
the personal skill survived.

| Final saved source identity | SHA-256 |
| --- | --- |
| Release | `5172ece5a624946ceab8f24b9568924ab1d8af5a2af696c261e9d5441e39f4b4` |
| Manifest | `adb4015b3f1b13f68963f5dc86714aa348601f10d5487d271679f94c216ac779` |
| Core executable | `09a9d51d4144b6cb573a83bcca6943e1f5d3099baa1fa3bd02a2c6b0d81d04ad` |

The final successful run is retained privately at
`~/.codex/task-state/harness-rewrite-implementation/reinstall-qualification-20261026/same-home-reinstall-1ly03xa3/`.
It contains the source package, CLI records, incremental MCP exchanges, source
record identities, exact retained-file hashes and summary. The initial fixture
correctly refused a 0644 config before claiming any installation; a subsequent
successful reinstall exposed a fixture-only assertion using `record.content`
instead of `record.data.content`. Both are retained beside the corrected runs;
production permission or memory checks were not relaxed.

The final build-input guard, locked offline Rust suite and strict all-target
Clippy passed: **530 library tests, six CLI tests and one integration test**.
Six library tests and one integration test remain explicit manual/runtime gates.
The final core above was rebuilt and used by the actual CLI/MCP smoke after the
native receipt-binding correction. These results establish this observed local
lifecycle, rather than GUI activation, Windows execution or release admission.

The first-party macOS installer now exposes `--reinstall` and
`--reinstall-reconcile` using the same explicitly pinned local source checks as
setup/update. Against the completed disposable fixture above, the installer
copied and hash-checked that exact core, then `--reinstall-reconcile` returned
`reinstalled` with the original plan hash and no new effect. `sh -n` passed.
Invoking `--reinstall` again against the now-active home refused with
`completed uninstall is missing`; a subsequent installed `doctor` remained
healthy. These wrapper checks do not substitute for the full CLI/MCP journey
above.
The corresponding Windows switches parsed under private PowerShell 7 on this
Mac; native Windows execution remains unqualified. Exact wrapper arguments,
hashes, completed receipt, refused repeat and unchanged active pointer are
retained outside shipped source in
`~/.codex/task-state/harness-rewrite-implementation/reinstall-qualification-20261026/installer-wrapper-check.json`.
