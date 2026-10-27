# M0 verification evidence

This is historical evidence for the removed generic text/file adapter. Current
MCP and memory checks are `tests/mcp-smoke.mjs` and `tests/memory-smoke.mjs` with
only explicit `--mirket` and `--node` paths. The commands and tool claims below
describe the former M0 surface and are not current product instructions.

This file distinguishes implemented test fixtures from executed integration
evidence. The current slice is a unique literal UTF-8 text replacement; it does
not qualify Office/PDF fidelity or the full rewrite.

## Current evidence

- The later native-lock/receipt integration baseline passed 234 library tests,
  one deterministic retrieval fixture and strict all-target Clippy on macOS
  ARM64 with private Node 24.22.1. Three explicit manual tests were ignored;
  the model package was not configured for that Cargo run. The real text MCP
  journey and disposable two-release activation journey also passed against
  binary SHA-256
  `459d66f1d1f5e2a981a85ddba3290dd3b06e8f84fbda2f20ffc52070aface8b3`.
  These are protocol/ownership results, not desktop activation or semantic
  retrieval acceptance. Earlier evidence below retains its original inputs.
- The private TypeScript adapter compiles with TypeScript 5.9.3. Seventeen real
  subprocess tests passed on macOS ARM64 with Node 24.18.1, including UTF-8/BOM/
  CRLF preservation, byte limits, ambiguous matches, release mismatch, exclusive
  creation, symlink refusal, malformed framing, cancellation and idle expiry.
- Five standalone Rust guide-registry tests passed: bounded embedded bodies,
  exact discovery/read IDs, public tool naming and authority/effect boundaries.
- The complete `tests/mcp-smoke.mjs` stdio journey passed on 2026-10-24 at
  approximately 17:12 UTC: macOS 26.5.2 (25F84), ARM64, Node 24.18.1, MCP
  2025-11-25, fixture releases `m0-test-a` and `m0-test-b`. It used the local
  development binary and explicit development Node path; no desktop client was
  involved. The repository remains an uncommitted integration workspace.
- Binary SHA-256 for that run:
  `1341f3def34fe7d0e6cd9521d3e9e5e7e71e7c9d486a69e9f89a925fad39eefd`.
  Compiled adapter SHA-256:
  `3ea3e1a548b5be6b70ece6487c7ce34f05ead1b8ecf1e824eabf8a4637f8be9e`.
  Later source or binary changes require their affected checks to run again.

## Run the real stdio journey

Build the Rust binary and compiled adapter using the admitted project
toolchain. Supply every executable/artifact path explicitly; the fixture does
not install dependencies, discover a global runtime or edit host configuration.
From the repository root, invoke the following with real absolute paths:

```sh
/absolute/node tests/mcp-smoke.mjs \
  --mirket /absolute/mirket \
  --node /absolute/private-node \
  --adapter /absolute/adapters/dist/main.js
```

The fixture launches only its supplied Mirket process with `dev serve --state-dir
ABS --release-id m0-test-a --node ABS --adapter ABS`. It creates a disposable
project and private state directory, requests mode 0700 and checks those mode
bits on POSIX. Windows ACL qualification remains a separate gate. It removes
its disposable files and stops its owned server processes on completion.

The client uses newline-delimited JSON-RPC over stdio, requests MCP 2025-11-25,
completes initialization and uses `tools/list` plus `tools/call`. It prefers
`structuredContent`, with JSON text content as the explicit alternate. RPCs have
a five-second deadline; job polling has a separate fifteen-second deadline.
Frames, aggregate stdout and stderr are bounded. These fixture budgets do not
establish native desktop timeout behavior or production performance targets.

The verified script uses `operation_id`, lowercase `status` and `effect` fields
in job results and `text` in file/artifact reads. Revision-bound artifact reads
send the result's `artifact_revision` as `expected_revision`. Do not weaken a
failing assertion to hide a changed behavioral contract.

## Discriminating checks

1. Negotiate the protocol, list exactly the implemented public tools, discover
   and read the Documents guide, then inspect the selected source.
2. Send a stale release to preparation and compare all disposable project/state
   paths and file hashes before/after. No file effect is allowed.
3. Prepare a unique replacement and confirm no output exists yet. A simultaneous
   second server sharing the state may inspect the receipt, but cannot cancel or
   start that prepared operation; its rejected requests leave all fixture files
   unchanged. The original owner can still start successfully. Start, poll,
   read its result and inspect the saved artifact; independently compare the
   actual UTF-8/CRLF bytes and unchanged original.
4. Modify the saved output and prove inspection rejects the stale saved revision
   without changing the new bytes. Remove the output and repeat the same start. The known operation
   must return without recreating the file; this exposes silent re-execution.
5. Cancel a prepared operation and verify a later start cannot publish it.
6. Change the source after preparation, and separately create the destination
   after preparation. Both operations fail before publication, preserving the
   newer source/destination bytes.
7. Deliberately discard a start response, observe the output, abruptly kill only
   that owned server and reconnect to the same state. Inspect the receipt;
   preserve confirmed/unknown effect knowledge. Remove the output and prove
   replay of that operation cannot recreate it.
8. Reconnect using release `m0-test-b`. Reject release A's guide read and a new
   root-addressed preparation, while keeping the old scoped receipt readable.

The loss fixture covers a start response and an abrupt server exit after an
observed publication. It does not enumerate every crash point or establish
atomic publication/receipt persistence. Removing outputs is deliberate test-only
mutation of disposable artifacts, never a proposed recovery action on user data.

## Remaining release gates

Actual Codex Desktop GUI activation and guide delivery;
Windows x64/ARM64 execution; job execution beyond the native host's default
timeout; and broader filesystem race/recovery qualification remain release gates.
Private runtime assembly, finite owned-child cleanup, competing-owner admission,
maintenance activation and receipt retention have separate qualification
evidence; see [text admission](m1-text-admission-verification.md) and
[receipt retention](m1-receipt-retention.md). They do not establish arbitrary
descendant ownership or native client activation.
This script's process cleanup is fixture hygiene, not proof of the production
ownership mechanism. Record actual machine, OS, client, binary/release and
dependency identities with each completed run.
