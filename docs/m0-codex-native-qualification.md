# Codex Desktop bundled-backend qualification

Historical evidence for the removed M0 text/file surface. The obsolete fixture
and stdio observer are preserved outside the repository under the rewrite task's
`simplification-20261027/removed-sources/`; they are not current tests. Use
`tests/codex-app-server-smoke.mjs` for the current backend path.

On 2026-10-25, the Codex-only fixture
passed against the executable shipped inside the installed Codex Desktop app:
`/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex`.
OS process inspection confirmed that the active Desktop app used this same
executable. Its version was `codex-cli 0.163.0-alpha.11.2`; the app was
`26.1016.62342`, build `21477`, bundle ID `com.openai.codex`.

This qualifies the Desktop-shipped backend through its real app-server API.
It does not qualify a GUI or model-driven journey. The already connected
Desktop session continued serving the previous installation throughout the
test; its passive runtime identity remained unchanged. No existing installation,
client configuration, authentication file or model setting was modified.

The reference machine was macOS 26.5.2, ARM64, `MacBookPro18,1`, with
17,179,869,184 bytes of RAM. Four isolated backend processes negotiated MCP
`2025-06-18` with the admitted `rmcp` SDK reporting `3.7.0`. Each child received
the fixture's disposable `CODEX_HOME`; no authentication files were copied. The
project and installed Mirket paths included spaces and Unicode.

## Observed behavior

| Boundary | Verified observation |
| --- | --- |
| Native skill metadata | The actual backend's `skills/list` discovered enabled repository wrappers for Engineering and Documents. |
| Canonical guide delivery | `guides_list` and `guide_read` returned the current Shared Workflow, Engineering and Documents guides with their exact release ID. |
| Inspection | `runtime_status` and `file_read` returned the installed release and the expected saved source bytes/revision. |
| Host denial | A valid, same-connection text preparation reached `prepared`; disabled `text_replace_start` returned Codex's own disabled-tool error. The receipt stayed unchanged and no destination appeared. |
| Dispatch evidence | A transparent fixture-only stdio observer recorded preparation and receipt inspection, with no `text_replace_start` call forwarded to Mirket. This establishes the host filter boundary, rather than inferring denial from an empty inventory or a service error. |
| Reconnection | The same policy passed in a fresh app-server process with a fresh valid preparation and again after release activation. |
| Useful allowed operation | A separate connection with start enabled completed the real private Node adapter operation, returned `effect: confirmed` and `scratch_cleanup: complete`, and exposed the saved result through `artifact_read`. |
| Saved artifact | The unchanged source contained `Mirket Codex native backend artifact: İstanbul ✓`. The new copy contained `Mirket Codex verified backend artifact: İstanbul ✓`; its exact UTF-8 bytes, final newline and receipt SHA-256 were checked independently. |
| Release transition | After all A sessions stopped, explicit update activated B. B returned current guides, refused A-bound guide and preparation calls, and inspected A's retained completed receipt under B's current release binding. No stale destination appeared. |
| Shutdown | After fixture EOF, Codex exited successfully and terminated its MCP process using `SIGTERM`. The observer recorded Mirket's actual exit and the fixture checked that PID was absent. This is host-owned termination, not a normal-EOF claim for Mirket. |
| Cleanup | All four app-server processes exited; the successful fixture's private installation/project/configuration tree was removed. Only bounded evidence and copies of synthetic source/result files were retained. |

The historical stdio observer forwarded the original
bytes unchanged. It records request names, tool names, initialization identity
and owned-process exit. It is qualification instrumentation and is not part of
the installed launcher or Mirket's protocol implementation.

## Historical invocation (no longer runnable against current source)

Use immutable core/adapter inputs so a concurrent development build cannot
replace the qualified bytes. Supply a new, nonexistent evidence directory;
the fixture refuses to overwrite an earlier run.

```sh
/absolute/private/node tests/codex-native-qualification.mjs \
  --codex /Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex \
  --mirket /absolute/frozen/mirket \
  --node /absolute/private/node \
  --node-license /absolute/private/LICENSE \
  --adapter /absolute/frozen/main.js \
  --evidence /absolute/new-evidence-directory
```

The fixture assembles two private four-file core releases with the same compiled
core and Node. B adds an inert adapter comment to obtain a distinct release
identity. This exercises binding/activation, not compatibility between changed
application schemas or independently compiled cores. The separate
[release activation fixture](m0-release-qualification.md) owns that core-handoff
boundary. Individual CLI and app-server requests and shutdown waits are bounded.
Failures preserve their disposable tree and report its location.

Recorded evidence is outside shipped source under the implementation task's
`codex-native-qualification-20261025-03/`: `summary.json`, complete synthetic
app-server request/response traces, per-session MCP dispatch/exit traces,
stderr, both manifests, saved receipt/source/result and passive Desktop identity.
The successful run took approximately 48.3 seconds; this single run is not a
performance benchmark.

| Qualified input | SHA-256 |
| --- | --- |
| Desktop bundled Codex | `248ff9873b6fb35124a86bdffa013a09e1bd17f106385124e3dda73e9eb157ad` |
| Frozen Mirket core | `e5b841cb13cc43ef232eabdf74b65bc52ea36e32eff862ed3ac438b4bfbfee13` |
| Private Node 24.22.1 | `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329` |
| Compiled text adapter | `3ea3e1a548b5be6b70ece6487c7ce34f05ead1b8ecf1e824eabf8a4637f8be9e` |
| Saved result | `b944fe1e9d29732b94977ec434fe18b25bf0ab9e3d6924e1b6f0da84fd20e291` |

Release A was `sha256:5f162f8e13c0db9e2503f3af5b359baf0cc40c073cf0b639dd3c07ecb2acffbb`;
B was `sha256:8b76dd63b9e2c3fc698ee8e020814b4a9c9cd35df9f445b7c6efc249ffdb4c3b`.
Later changed core/adapter bytes require affected checks again.

## Remaining acceptance limits

No `turn/start`, model generation, GUI discovery/interaction, production config
switch, default host timeout overrun or abrupt death during an active worker was
exercised. Low-level app-server filtering does not establish model-visible
approval behavior. A non-disruptive live temporary-config route for the active
GUI was not established, so the existing client was preserved and the isolated
bundled-backend route was used. This four-file slice is not the full private
distribution. Native Windows and the oldest supported macOS remain unqualified.

The isolated backend emitted failed featured-plugin and response-WebSocket
requests with HTTP 401 despite no model turn. These tests establish neither
network silence nor authenticated account readiness. Codex Desktop is the sole first-release host target.

The configuration and framing follow the fetched official
[Codex MCP documentation](https://developers.openai.com/codex/mcp/) and
[app-server documentation](https://developers.openai.com/codex/app-server/).
The observed installed implementation, saved artifact and dispatch evidence
determine the bounded result above.
