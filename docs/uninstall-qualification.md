# Explicit local uninstall and recovery

Observed on macOS 26.5.2 ARM64, 2026-10-26. The rewrite now exposes:

```sh
/absolute/mirket uninstall --home /absolute/private-home --user-home /absolute/disposable-user
/absolute/mirket uninstall-status --home /absolute/private-home --user-home /absolute/disposable-user
/absolute/mirket uninstall-reconcile --home /absolute/private-home --user-home /absolute/disposable-user
```

Every command requires explicit roots. Status is read-only. Uninstall obtains
the existing Codex operation, idle maintenance and release-writer locks; it
checks native receipts, launcher identity, active/previous/staged release
ownership and complete payload manifests before recording its immutable plan.
Pending staging/activation and unresolved child ownership refuse removal.

Registration, managed guidance and global skills are removed through their
existing owners. The skill owner retires only its checked empty committed
receipt after transaction cleanup. Uninstall then removes individually
verified payload entries and empty owned release directories. It never uses
recursive deletion. The stable launcher is the final payload entry.

All canonical state, managed backups, protected recovery data, project-local
skill selections/copies, unrelated client configuration, personal skills and
unknown home files are preserved. Edited managed content or unknown payload
children cause refusal. There is no data-deletion option. The installation
marker, state/backup directories and empty release/staging/bin containers
remain available for inspection.

`state/uninstall.json` binds the original release, user home, exact paths,
native file identities and content pins. Its small progress record binds that
plan and the current native step or single armed payload deletion. Explicit
reconciliation uses the original operation: native pending receipts are
reconciled by their own owners, and only the armed file may be absent after a
lost response. Torn/different records or changed payloads are preserved.
Normal MCP startup, staging, activation and native installation mutations
refuse while the uninstall record exists. Doctor reports `uninstall_pending`
or `uninstalled`; completed status rechecks removed payloads and native
integration rather than trusting the completion record alone.

## Final executable and reinstall boundaries

The installed launcher remains usable until its final unlink. If the process
stops after that unlink and before confirming completion, there is no remaining
installed recovery executable. Use the exact verified external release package
and its installer; do not fetch or execute an arbitrary replacement. The local
macOS installer accepts `--uninstall-reconcile`, with the source manifest and
binary SHA-256 values from the trusted release record, and invokes a verified
private copy of that external binary. The Rust recovery command needs no
remaining installed payload at this stage.

Fresh setup still requires an absent/empty home. A completed uninstall can now
be followed by the separate explicit [same-home reinstall owner](m0-macos-reinstall-qualification.md),
which binds the original completed plan and exact new candidate before staging
into preserved state. An interrupted uninstall remains ineligible. Removing
the marker by hand is not a supported reinstall procedure. Windows filesystem,
process and client behavior remain unqualified.

## Actual installed CLI evidence

A disposable release containing the Rust executable recorded below, the admitted
private Node/text adapter and all exact authored skills passed normal setup and
doctor. Its installed MCP served a real canonical sourced fact save and created
three managed backup files. A live MCP connection caused uninstall to refuse
idle maintenance; an injected native skill edit refused removal before an
uninstall plan was created. The known injected bytes were then restored.

The actual installed launcher completed self-uninstall, removing **150** planned
payload entries. All **16** pre-existing retained state/backup files stayed
byte-identical, including the canonical memory and three backup files.
Unrelated configuration, personal guidance, a personal skill and an unknown
home note survived. Post-uninstall doctor/status and an explicit completed
reconcile reported `uninstalled`. No Codex model, GUI session or actual user
installation was involved.

| Saved source identity | SHA-256 |
| --- | --- |
| Release | `2114162844215fdd7c2dd648e1057f8f7c2cb3888474c65acedb262e8ea5b695` |
| Manifest | `5273751e91bfdd208ee6d1d9cfa7ddb8aac6042edae88870dfa0549fd206d831` |
| Core executable | `4225ce13a0d3eae45852a23d13d2cc18b4109b09a20e855bf484a31960a68b1c` |

Evidence remains outside shipped source in
`~/.codex/task-state/harness-rewrite-implementation/uninstall-qualification-20261026/`:
the qualification script, source package, complete CLI/MCP records, exact
preserved-file hashes and summary. A second disposable installed home remains
healthy for integration checks. These fixtures establish the observed Mac
journeys and ownership behavior; they do not establish Windows, the minimum
macOS floor, release authentication, GUI activation or publication readiness.

The explicit ignored real-release fixture also passed after installing that
actual package and injecting a stop immediately after its final launcher
unlink. No installed launcher or release payload remained. The pinned external
macOS installer then completed `--uninstall-reconcile` in 1.529 seconds against
the original plan and preserved its state sentinel. The injection uses a private
Rust test hook; production exposes no failure-injection option. The saved
`final-unlink-injected-fixture.log`, `final-unlink-fixture/after-final-unlink.json`
and `installer-final-unlink-recovery.json` distinguish the injected boundary
from the actual external installer recovery.

After the final uninstall/status and receipt checks, the build-input guard and
full locked offline Rust suite passed: 523 library tests, five CLI tests and one
integration test; six library tests and one integration test remained explicit
manual/runtime gates. The real-release final-unlink test was separately invoked
and passed. Strict all-target Clippy passed with warnings denied. The installed
CLI package above predates the later read-only completed-uninstall snapshot
helper supplied to the reinstall owner; its exact identity is retained rather
than attributed to a future source build.

An earlier full run retained one browser-session failure: native resource
accounting could not identify a child PID. That unchanged test passed in
isolation in 0.81 seconds, followed by the passing full run. The exact timing
cause remains unconfirmed; the failure, isolated output and final passing logs
are preserved. No browser accounting implementation or test threshold was
changed by this uninstall work.
