# Installed Python navigation through the Codex native backend

On 2026-10-25, the Codex Desktop bundled backend passed an isolated app-server
qualification of installed Python `code_navigate`. The exact installed release
was `sha256:5d8c2d9d1697a8b353926610bf864d5991b2aff046fa81426cfded23489318ba`.
This establishes useful direct tool calls through the Desktop-shipped client
backend. GUI interaction and a model-driven Python journey remain unqualified.

The client executable was
`/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex`,
version `codex-cli 0.163.0-alpha.11.2`, SHA-256
`248ff9873b6fb35124a86bdffa013a09e1bd17f106385124e3dda73e9eb157ad`.
The installed Desktop app was `26.1016.62342`, build `21477`. The platform was
macOS 26.5.2 ARM64. This is the same client binary identity recorded in the
[earlier native qualification](m0-codex-native-qualification.md).

The child process ran `app-server --strict-config --listen stdio://` with a
fresh disposable `CODEX_HOME`. No authentication files were copied and no
`turn/start` request was sent. The fixture generated the experimental app-server
JSON schema from this exact executable, then used `initialize`, `initialized`,
`config/read`, `skills/list`, `mcpServerStatus/list`, an ephemeral `thread/start`
and `mcpServer/tool/call`. The configured tool inventory contained exactly
`runtime_status`, `guides_list`, `guide_read`, `file_read` and `code_navigate`.
`startup_timeout_sec` was 60; `mcp_optional_startup_grace_ms` was unset in the
saved configuration and the effective client configuration. No global catalog
grace override was supplied.

The historical qualification-only stdio observer, preserved outside this
repository under `simplification-20261027/removed-sources/`,
forwarded the original MCP bytes to the installation's normal stable launcher,
`bin/mirket serve --home …`. It retained dispatch timestamps and owned-process
exit evidence. It is outside the installed runtime implementation.

| Boundary | Observed result |
| --- | --- |
| Native discovery | The backend discovered the enabled repository skill wrapper and exactly the five configured tools. |
| Runtime and guide identity | `runtime_status`, `guides_list` and the canonical Engineering guide returned the exact installed release. Both saved Python files were read with independently checked revisions. |
| Useful Python navigation | `main.py` imported `şehir` as `go` from the selected `library.py`; querying `go()` returned the original library definition at UTF-8 bytes 15–21. |
| Backend identity | Jedi `0.20.1`, Parso `0.8.7`, private runtime `cpython-3.13.17-macos-arm64`. |
| Bounded completeness | The response was `complete`, coverage `selected_parser_names`, six examined names and no reasons. This applies to the selected parser-name pass only. |
| Original coordinates | The definition source included a UTF-8 BOM, an emoji comment and CRLF newlines. The saved byte slice at 15–21 was independently checked as `şehir`. |
| Stale source | After changing the library while keeping the original request revisions, the same client call returned `changed` with current revisions and no locations. The fixture then restored the exact original bytes. |
| Stale release | A guide request bound to `stale` returned the release-changed error. |
| Passive fixture state | Both selected source files, a raising `sitecustomize.py` and a `pyproject.toml` naming an unavailable build requirement were preserved byte for byte; no extra project file appeared. |
| Shutdown and installation | Codex exited 0. Both observed Mirket processes were terminated by the host with `SIGTERM` and were absent afterward. A subsequent `doctor --json` reported the exact release healthy. |
| User configuration | The live `~/.codex/config.toml` SHA-256 was unchanged. The disposable Codex home contained no `auth.json` and was removed after success. Synthetic sources and bounded evidence were retained. |

The original library revision was
`sha256:701590fb7f39f26fd0641969920acfddad1e4f99e997a42c448fa5055ca45932`;
the caller revision was
`sha256:61922eb8cca388b867dec7a13481bdbe675d3320c8f8d4bab03434fef21e11c6`.
The returned navigation model revision was
`sha256:1f31a61017ca64fe8dbe74cf67cc4fac2169c573e8ba59bb9927344898ff6e26`.
The source path included spaces and Unicode. Structured and parsed text results
were checked for equality.

Two MCP connections were observed: inventory discovery and the ephemeral
thread. Their initialize request-to-response intervals were **2,320 ms and
2,398 ms**. Both exceeded the one-second default optional-server catalog grace
discussed in [the startup investigation](m3-verified-startup-latency.md).
A successful direct app-server call therefore does not establish default
model-visible discovery. No workaround was introduced and no model turn was
attempted in this unauthenticated disposable client state.

Private evidence is retained outside shipped source at
`~/.codex/task-state/harness-rewrite-implementation/python-navigation-codex-client-qualification-20261025/`:
`summary.json`, `qualify.mjs`, the exact client schema, `config.toml`,
`effective-config.json`, complete synthetic app-server request/response traces,
MCP dispatch/exit timestamps, `runtime-status.json`, `python-definition.json`,
`python-stale-revision.json`, stderr and the saved synthetic project.
The fixture used the task-owned installation at
`~/.codex/task-state/harness-rewrite-implementation/python-navigation-production-install-20261025`;
that path records the tested location and may subsequently be relocated by the
owning installation qualification.

The isolated backend emitted failed featured-plugin and response-WebSocket
requests with HTTP 401, and unrelated downloaded-plugin metadata warnings.
No model generation was requested. This run establishes neither network
silence nor authenticated account readiness. It does not qualify GUI activation,
model approval behavior, references or stub navigation through Codex, Windows,
oldest-platform compatibility, whole-project Python completeness or general
performance. It used only disposable task state, with no user installation
switch, commit, push or publication.

The protocol framing follows the fetched official
[Codex app-server documentation](https://learn.chatgpt.com/docs/app-server).
The generated installed schema and actual returned source locations establish
the bounded result above.
