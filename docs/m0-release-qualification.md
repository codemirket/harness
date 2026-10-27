# Local release activation qualification

> **Historical qualification record.** This page preserves an earlier
> documents/PDF payload experiment for provenance. It is not the current
> Mirket product contract or a supported release component. In the approved
> rewrite, Codex Desktop and host tools own PDF, Office, extraction and OCR
> work; Mirket does not ship the `documents` component described below. Use
> [the current implementation plan](../planning/implementation-plan.md) for
> the active scope.

The disposable `tests/release-activation.mjs` fixture passes on macOS 26.5.2
ARM64 with private Node 24.22.1. It uses two independently compiled Rust cores,
the same Node/adapter bytes, actual stdio MCP and saved artifact operations.
It changes no existing installation or client configuration.

## Active-core handoff

`serve --home` acquires its session guard, then verifies the active manifest
once and reuses that verified release for core comparison and engine startup. If
its running executable differs, it replaces itself with the verified active core
using macOS exec. The PID and MCP streams remain unchanged; no wrapper process
continues running. No Engine or MCP input reader is opened before verification.

The child receives an exact expected release ID, reacquires its session guard,
and independently verifies both that release and its running core. A mismatch refuses rather
than recursively launching again. File/identity errors are errors, not a reason
to hand off. The old guard remains held through exec; close-on-exec releases its
locks. Activation in the reacquisition gap therefore causes refusal, not mixed
release state. Reconnection is explicit. No inherited-lock protocol or daemon is
introduced.

The test observes the original PID executing the installed B path through the
OS process table, checks B's saved executable hash, and completes a B-bound text
operation. It also rejects direct wrong-core and stale-release handoff claims
without changing project/home bytes. This verifies changed compiled bytes, not
compatibility between different application schemas or versions.

## Reproduce

Supply explicit paths to two built cores, private Node and its matching license,
and the compiled adapter:

```sh
/absolute/node tests/release-activation.mjs \
  --mirket /absolute/core-a \
  --next-mirket /absolute/core-b \
  --node /absolute/node \
  --node-license /absolute/node/LICENSE \
  --adapter /absolute/adapters/dist/main.js
```

The recorded A build uses the normal development profile; B uses
`cargo rustc --locked --offline --bin mirket -- -C debuginfo=0` after the build
input guard. Their code has the same intended behavior and their compiled bytes
differ without patching binaries or signatures. Node/adapter bytes are identical.
The fixture refuses equal core hashes before creating temporary state.

| Input | SHA-256 |
| --- | --- |
| Core A | `e1c6450cde1c8629bcdf8b3a1ea941fd2a20d8ac36119e739fdc65d8b6fcf277` |
| Core B | `2381583af0bfb47db7f2e0fa293e79469f2839b7f8548dfda97ea46e1a4406a7` |

Recorded release A is `sha256:20ee553e6965e6808759c6c16a76c8213126bc6fcb2b1fe564e0437850a17514`;
B is `sha256:20ccdea4f509af121f9cc85e6e040915991992664df7c62a8de8a89361f3072e`.
Build and native evidence remain in the implementation task record. These pins
identify this run; later changed code requires affected checks.

After the single-verification startup change, the same disposable handoff
fixture passed again using core A digest
`bb5be9e8759990a9bae998f53b1ff2504669e43df28de1dceb6385ea96f08bb3`
and core B digest
`e2a51a1f37f5a34d54f9e8d2971ffaa2999750354e27a92bb67973cbf89632af`.
It exercised live-session exclusion, same-PID handoff, saved artifact creation,
stale-release refusal, and interruption recovery on macOS ARM64. This does not
measure startup latency or qualify Windows handoff.

## Observable checks

- Unrecognized nonempty homes are refused with existing files preserved.
- Both active cores produce a confirmed saved text artifact using their installed
  runtime/adapter paths, preserving source bytes and completing scratch cleanup.
- Live sessions block activation. Staging, disconnect and ordinary reconnect do
  not automatically activate a candidate; a later explicit request does.
- B rejects cached A guidance and prepared-operation requests before effects.
- Differing activation residue is preserved/refused. Exact matching pending
  records and post-commit retries finish only on explicit activation. These are
  constructed saved crash states, not injected power failures.
- Unresolved heavy ownership and modified staged payloads preserve the active
  pointer and refuse activation.

The fixture deletes only its owned temporary tree after all checks and every
connection's normal EOF/reap succeed. Failures retain evidence and report its
location; forced termination is not labelled confirmed cleanup. Individual CLI,
MCP and artifact waits are bounded. Deleting a test-created recovery record is
not a product recovery procedure.

## Limits

This four-file core payload is not the complete releasable tool distribution.
The handoff is qualified on macOS ARM64 only. Windows, desktop activation,
native trust/distribution rights, oldest supported macOS, real power-loss
recovery and broader migration behavior remain separate gates. Hashes do not
establish publisher authenticity. Executable checks assume private immutable
release paths, not hostile concurrent pathname replacement.

## Private release components

The release manifest has one schema with a finite component list. An empty list supports
activation and recovery fixtures. The `documents` component additionally binds the admitted general Python tree,
its notices, the exact SQLGlot and pypdf wheels, and both authored adapters. It is available
only for macOS ARM64; this does not imply Windows runtime qualification.

The fixed layout places the assembled Python tree and its notices beneath
`runtime/python-general/`, the adapter at `adapters/sql_navigation.py`, and the
wheel at `libraries/sqlglot-30.24.0-py3-none-any.whl`. Manifest generation infers
this profile from those optional paths and requires the complete inventory.
The PDF leaf lives at `adapters/pdf_operations.py`, its exact wheel at
`libraries/pypdf-6.21.2-py3-none-any.whl`, and its BSD notice at
`notices/python-documents/LICENSE.pypdf.txt`. A partial documents payload cannot
silently become a core release. The previous SQL-only profile is not accepted.

Regular-file bytes, normalized private modes and the two admitted relative
interpreter link targets participate in release identity. The fixed inventory
includes the required `site-packages` directory with its pinned README and no additional packages; arbitrary extra entries and alternate link targets
are refused. Source files must have the expected owner permissions; staging
normalizes group/other permissions away. Installed files require exact `0600`
or `0700` modes and directories require `0700`. These identity and consistency
checks do not protect against a hostile same-user filesystem mutator.

`tests/documents-installed-smoke.mjs` is the separate fixture for installed SQL
and PDF. It accepts explicit local core, Node, adapter, assembled Python, SQL
script/wheel and PDF script/wheel/admission-receipt inputs; no asset is downloaded and no user installation is changed.
This documents the original documents-only fixture, whose manifest was below
256 KiB. The current verifier derives finite bounds from each compiled target
and component inventory, with exact file pins and named unpinned roles; see
[distribution capacity](m0-distribution-capacity.md). The installed combined
design release is described in [local design qualification](m4-design-qualification.md).
Further components require measured inventories and explicit admission; native
platform/client coverage remains a separate gate.


The installed SQL fixture passed on macOS 26.5.2 ARM64 with private Node 24.22.1
and the admitted CPython 3.13.17 payload, under OS network denial. It verified
installed-only path resolution, real Unicode navigation, unchanged sources and
payload, aliases/modes, and preservation of the active release when a candidate's
Python bytes or alias target changed. Source and home paths contained spaces and
Unicode. The full installed profile contains 564 regular files and two aliases;
the generated manifest remains below 256 KiB. This is local fixture evidence,
not native Windows, desktop activation or publication qualification.
