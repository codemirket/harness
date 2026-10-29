# P0 integration proof specification

Status: **offline host-contract proof passed**, under the exact
[approved test admission](../packaging/admission/pi-p0.md). Owner: migration lead.
This file is the case specification. Actual assertions, raw evidence locations,
platform identity and limits are in the
[qualification record](../packaging/admission/pi-p0.qualification.json) and the
[execution checkpoint](../planning/pi-migration.md#execution-checkpoint--2026-10-29).
Caller-side response discard is the tested loss model; abort covers running
codemode, not a native creative job. TUI interaction has transcript evidence,
not saved-pixel visual QA. Aggregate RSS was sampled, not continuously profiled.

## Fixture boundaries

Create a fresh disposable HOME, pi agent directory, independently located session
directory, two project directories and local package. Do not inherit user auth,
MCP settings, extensions or `.agents/skills`. Explicitly set the offline and
telemetry controls from the admission. Never use the live Mirket installation.

The original extension imports the actual host-provided TypeBox value and pi API
types, compiles as ESM with external peers, and registers `mirket` on
`session_start`. Config: absolute freshly built Mirket executable; args
`dev serve --state-dir <agent-dir>/mirket --release-id pi-p0 --node <verified-node>`;
explicit project cwd; `exposure: deferred`; `toolExposure.runtime_status: direct`.
Release id `pi-p0` is a fixture identity, not a release attestation.
No workers on factory load. Unregister idempotently on shutdown. No direct
subprocess launcher or duplicate wrapper for domain tools.

For SDK cases explicitly load `createCodemodeExtension`,
`createToolSearchExtension` and `createMcpExtension` via `DefaultResourceLoader`,
and call `session.bindExtensions`. Actual CLI cases load the same assembled
fixture package and built-in extensions. A deterministic local provider may
emit fixed tool calls to exercise the genuine agent pipeline without paid model
calls; this proves protocol integration, not model judgment.

## Cases and independent oracles

| ID | Action | Required observation / failure oracle |
| --- | --- | --- |
| P0-01 | Load compiled package with external peers, then bind a session. | Real peer value works; extension factory alone starts zero children; one verified Rust process after binding. |
| P0-02 | Call direct runtime_status in loaded CLI session; inspect `/mcp`. | Expected release, binary/cwd/state tuple and extension origin agree. Shell `pi mcp list` is not a substitute. |
| P0-03 | Search for deferred memory tools, prepare/create/commit/read a unique fact. | Tool exposure changes after discovery; actual read equals frozen input. One model-visible route per operation. |
| P0-04 | Repeat a distinct fact through codemode; deliberately use stale release. | Commit/read persistence through real MCP; `isError` inspected even when the JS promise resolves. Stale call produces no mutation. |
| P0-05 | Restart and reconnect; lose a commit response deliberately. | Read saved fact; reconcile original operation, never blindly replay the write. |
| P0-06 | `/reload`, resume/fork and orderly shutdown. | New binding is active, old connection closes; no duplicate/orphan service, including descendant checks after exit. |
| P0-07 | Global filter selects skill A; project replaces it with B; then `autoload:false` delta. | Observe actual loaded resource names/origins, including unfiltered package behavior and collision case. No host copies. |
| P0-08 | Same-name file-configured alternate server, disabled MCP, filtered tools, missing extension. | Distinct observed unavailable states; never attest overridden server as verified Mirket; do not probe it with writes. |
| P0-09 | Change project and independently relocate session storage. | Memory remains scoped to actual project and agent state root, not session path. Cross-project read cannot return the first fact. |
| P0-10 | Two clients; interrupt a call; shutdown and inspect processes. | Canonical result and ownership remain consistent; no false cancellation assertion from turn completion. |
| P0-11 | Status-only idle at 30/60/120s; repeat cold starts. | Capture startup/first-tool latency, RSS and process tree. Zero embedding/Chrome/FFmpeg startup for status-only use. Freeze later regression budgets from observations. |
| P0-12 | Actual TUI `/mcp`, `/reload`, exit; actual CLI/RPC invocation. | Saved transcript plus process evidence; SDK pass cannot waive TUI/CLI. No claim of image delivery or visual QA from this test. |

Memory input is independently authored: `kind: fact`, `basis: observed`, unique
nonce content and source revision, `status: current`, no applicability/expiry
restriction. Use the existing `memory_prepare` schema, then its returned
operation ID for commit, then returned record ID/revision for read. Check the
structured result and error flag, not only a tool completion event. Keep raw
responses bounded and redact no nonexistent credentials into misleading passes.

Stop P1 if a required case fails or is unrun. A native MCP failure should first
produce the smallest reproducible counterexample; custom IPC is not the default
repair. Current-service protocol smoke is only a prerequisite.
