# Codex model-turn qualification

This report records earlier installed releases. The later bounded full-catalog,
default-policy model call is in
[verified installed startup latency](m3-verified-startup-latency.md#installed-deferred-catalog--2026-10-26).

On 2026-10-25, a read-only, ephemeral `codex exec` turn using the executable
bundled with Codex Desktop (`codex-cli 0.163.0-alpha.11.2`) called the installed
rewrite's `mirket_rewrite.runtime_status` MCP tool. The tool returned release
`sha256:254db338a1b0bb03dae043e67c4b163d9060bcdd01cecfa90cb01a9bd4a499c6`
and reported `browser.configured: false`, with Chromium as the sole browser.
The model's final answer matched both values. The trace contains exactly one
MCP tool call, with empty arguments and no error; it contains no shell call.
In a second Desktop-bundled turn, the model called only `file_read` against an
explicit disposable project and returned the exact UTF-8 text
`Mirket Codex saved read: İstanbul ✓` with its trailing newline. The returned
revision was
`sha256:356f9b55f5a5fda33bf2c450046a8203af891eb97ba3d05548674b1f8cee47ea`,
matching an independent SHA-256 of the saved source bytes. Both turns completed
without tool errors.

The turn selected the verified disposable installed home through command-line
MCP overrides and used `--ignore-user-config`, `--ephemeral`, a disposable
project, and `--sandbox read-only`. It did not alter the user's Codex
configuration, active Mirket installation or running Desktop GUI session. A
second turn using the separately installed `codex-cli 0.146.0` also called the
same MCP tool and returned the same release ID. Complete JSONL traces and
stderr are retained outside shipped source at
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/codex-live-20261025/`.

This establishes model-visible MCP discovery and two successful reads through
the actual Desktop-bundled CLI executable on macOS ARM64. It does not establish
GUI activation, a model-driven write or host approval journey, browser package
admission, Windows behavior, or general task quality. The earlier
[bundled-backend fixture](m0-codex-native-qualification.md) separately covers
direct app-server calls, host filtering, artifact inspection and reconnect.

The later four-component navigation release
`sha256:b33f20376b1cc6d6ea2299e97139133a673302703de45912f626fde8e6a0f3e9`
passed direct installed MCP navigation and Codex app-server inventory/tool-call
checks. A first model turn did not see `code_navigate`; the bounded trace does
not identify Codex's exact startup failure, but a timeout is consistent with
the release's verification cost. With a 60-second per-server startup
window, the Desktop-bundled Codex model called `code_navigate` once and returned
`locations` with the saved `answer` definition at UTF-8 bytes `[6,12)`. Its
trace and the unsuccessful timeout attempts are retained in the same external
evidence directory. Mirket's managed Codex entry now writes this startup
window and includes it in drift checking; the setting does not lengthen any
Mirket-owned navigation job deadline.

The subsequent release
`sha256:af5c872947b8a57c903f8283bc12f25a9cd5797c5bde5afa04f55970304b59e7`
activated in a disposable home, passed `doctor` and the installed navigation
fixture, and passed Codex's managed app-server registration/tool-call fixture
with the generated 60-second setting. That fixture used an isolated Codex
configuration and did not run another model turn or alter the user's active
Desktop GUI session.

The later shell-source release
`sha256:0fdf6a45d9d0ec0b51289ec16b7dd8cb23e51cffeaec0bba6dc3f51512693aea`
is healthy and passes the Desktop-bundled Codex app-server's disposable managed
registration, inventory and direct `shell_inspect` call. A fresh read-only
ephemeral model turn using command-line MCP overrides reported that the tool was
unavailable, with **zero tool calls**. Repeating without a tool filter produced
the same result. A workspace-write attempt with only the disposable Mirket home
added as writable reached no tool or agent item within 180 seconds and was
terminated; no process using that home remained afterward. These traces are in
the same external evidence directory. They do not establish model-visible shell
discovery or identify its failure cause, so that gate remains open. No user
configuration, GUI session or source fixture was changed by these attempts.

The follow-up [startup-timing qualification](m3-codex-live-remediation.md)
reproduced this release's `shell_inspect` model call twice when a disposable
Codex invocation waited for optional MCP catalogs. Two otherwise matched turns
with only `required=true` still made no tool call. This resolves the narrow
shell-tool functionality question under the explicit override; ordinary managed
registration remains unqualified while startup exceeds the default catalog
grace. The override is global to Codex and was not written to user settings.
