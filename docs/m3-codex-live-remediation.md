# Codex model discovery: startup timing remediation

This is the 2026-10-25 qualification record. The deferred installed-catalog
implementation and later default-policy Codex model result are recorded in
[verified installed startup latency](m3-verified-startup-latency.md#installed-deferred-catalog--2026-10-26).

On 2026-10-25, the Desktop-bundled Codex CLI successfully called the installed
shell-source release's `shell_inspect` tool in two read-only model turns when
the invocation supplied `-c mcp_optional_startup_grace_ms=0`. Each turn made
exactly one MCP call, returned the expected saved-source revision and completed
without a tool error or shell call. This closes the narrow shell model-turn
gate left open in [the earlier report](m3-codex-model-turn.md) for the explicitly
overridden qualification invocation. It does not establish default managed
registration or GUI activation.

The qualified release was
`sha256:0fdf6a45d9d0ec0b51289ec16b7dd8cb23e51cffeaec0bba6dc3f51512693aea`,
served from the existing disposable home
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/homes/codex-shell-source-clean-20261025`.
The client was `codex-cli 0.163.0-alpha.11.2` on macOS 26.5.2 ARM64, executable
`/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex`,
SHA-256 `248ff9873b6fb35124a86bdffa013a09e1bd17f106385124e3dda73e9eb157ad`.
This matches the binary identity in the earlier successful Desktop-bundled
qualification; a client executable change does not explain these observations.

## Discriminating observations

All four model probes used the same prompt, exact JSON arguments, disposable
32-byte UTF-8/CRLF Bash fixture, `enabled_tools = ["shell_inspect"]`,
`startup_timeout_sec = 60`, `--strict-config`, `--ignore-user-config`,
`--ephemeral` and `--sandbox read-only`. No model override was introduced.
A qualification-only byte-preserving stdio observer recorded the MCP exchange.
The table reports observed wall time, not a performance benchmark.

| Invocation difference | Initialize request to reply | `tools/list` round trip | Completed model MCP calls | Entire turn |
| --- | ---: | ---: | ---: | ---: |
| Global catalog grace `0`, first probe | 5.531 s | 7 ms | 1 | 15.206 s |
| Mirket entry `required=true`, default global grace | 5.459 s | 7 ms | 0 | 15.813 s |
| Same `required=true` case, repeat | 5.798 s | 8 ms | 0 | 15.881 s |
| Global catalog grace `0`, repeat | 5.777 s | 6 ms | 1 | 15.892 s |

Process spawn to the initialize request took 2–7 ms. Almost all measured catalog
startup time preceded the initialize response; the protocol trace does not
separate release verification, state opening and other work inside that interval.
The two successful `shell_inspect` calls then took 159 and 162 ms.

Both successful results reported `status: page`, backend `tree_sitter_bash`
`0.25.1`, `structure_complete: true` and revision
`sha256:d883045c9ab80540b4a0a01b17d5e19be9622dd8f251af7b29d56a96b754d6a5`.
That revision matches an independent SHA-256 of the unchanged saved fixture.
The structured result and parsed text result were also identical.

Both `required=true` probes received successful initialize and `tools/list`
responses, but the model reported that the tool was unavailable and made no
tool call. A separate disposable app-server `config/read` confirmed that
`required: true`, the 60-second timeout and the tool filter were present in the
effective configuration; the global grace was unset. This build's observed
behavior therefore does not support `required=true` alone as a sufficient fix.
The internal client reason for that result remains unmeasured.

An independent control used an unauthenticated disposable `CODEX_HOME` and no
model generation. With the default optional-server configuration, the actual
app-server returned the expected two-tool inventory and completed a direct
`shell_inspect` call. Its inventory and new-thread MCP connections initialized
in 4.981 and 4.977 seconds. The full control took 10.285 seconds and exited
successfully. Direct app-server inventory waits can therefore pass while a
model turn still reports an unavailable tool.

## Cause and configuration tradeoff

The current official [MCP documentation](https://learn.chatgpt.com/docs/extend/mcp)
and [configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference)
describe two distinct deadlines. `startup_timeout_sec` bounds server startup.
The global `mcp_optional_startup_grace_ms` controls the initial optional-server
tool catalog, defaults to 1000 milliseconds, and waits for server startup
deadlines when set to zero. The measured 5.0–5.8-second startup exceeds that
default catalog grace while remaining well within Mirket's 60-second timeout.
These paired probes support startup/catalog timing as the current discovery
failure boundary. A successful call in the same read-only sandbox, with the
same prompt and tool filter, rules out those settings as an inherent obstacle
to this shell inspection.

The docs say required servers use their startup deadline, but the two observed
`required=true` turns above did not expose a usable model tool. Preserve this
version-specific discrepancy rather than assuming the documented alternative
works. No per-server catalog-grace option was found in the current documented
MCP options or configuration reference. The exact cause of every older failed
turn, including the earlier 180-second workspace-write attempt, is not proved
by these new observations.

The tested qualification workaround is the invocation override
`-c mcp_optional_startup_grace_ms=0`. It is a **global Codex setting**: unrelated
optional MCP servers may also hold initial catalog construction until their
individual startup deadlines. Mirket must not silently write this setting or
present it as part of its own server entry. No user setting was changed here.
Choosing a persistent global override needs an explicit configuration decision;
default managed registration remains a separate acceptance gate.

## Unimplemented startup alternative

A focused Mirket alternative is to make initialize and the compiled tool
catalog available before verifying and opening the entire runtime payload.
The current startup owners are `src/main.rs` (`serve`),
`src/release_store.rs` (`open_active`), `src/distribution.rs`
(`VerifiedRelease::open_installed`) and engine/server construction. The inspected
path completes full installed-release verification before starting MCP. The
trace identifies the interval; it does not quantify each owner's contribution.

A possible design would acquire the existing session guard, bind the active
release and verify the small manifest plus running core first, then answer
initialize and `tools/list` using compiled schemas. Backend readiness would
remain unverified. A connection-owned, single shared verification result would
perform the current complete payload checks and engine opening lazily before
the first tool operation or any healthy-runtime claim. Failure would refuse
the operation; no private executable or backend would start before its required
verification. The session guard, release identity and activation exclusion must
remain continuous, and concurrent first calls must not duplicate verification
or create engines against different releases.

This design is **not implemented or qualified**. It needs a measured cold
catalog response within Codex's default grace and tests for modified payloads,
concurrent first calls, cancellation/shutdown during verification, activation
exclusion and unchanged integrity guarantees. It must not replace byte
verification with an mtime cache or declare an unverified payload healthy.

## Evidence and limits

Sanitized argv/prompt records, JSONL model traces, MCP timestamps, effective
configuration, the direct app-server control, source fixture and independently
checked summaries are retained outside shipped source at
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/codex-live-remediation-20261025/`.
`verification-summary.json` records exact timings and validation.

Model probes reused the previously authorized local authentication home with
`--ignore-user-config`; no credential bytes were read, copied or printed.
Each probe checked the real user configuration's SHA-256 before and after and
found it unchanged. The app-server control used its own unauthenticated home.
All six observed Mirket process IDs were absent after client shutdown. The
source fixture remained unchanged. No shared Rust source, user installation,
GUI session, commit, push or publication was changed by this investigation.

This evidence establishes a bounded passive shell inspection through the
Desktop-shipped CLI under the explicit global override. It does not establish
GUI activation, default-startup reliability, a model-driven write/approval
journey, a fully shipped distribution, Windows behavior or general model task
quality.
