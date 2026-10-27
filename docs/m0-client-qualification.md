# M0 Codex qualification routes

> **Historical fixture guidance.** The `guides_list`/`guide_read` wrappers and
> canonical guide registry described below were replaced by nine native global
> skills and 58 opt-in project skills. Current discovery and installed-client
> evidence live in [native skill evaluation](m4-guide-evaluation.md) and
> [Codex installation qualification](codex-native-installation-qualification.md).
> Retain the dated observations below as evidence, not instructions for a new
> release or test fixture.

Codex Desktop is the sole first-release host target. The 2026-10-25 [Desktop bundled-backend qualification](m0-codex-native-qualification.md) is the current native-executable result; GUI/model acceptance remains open. Earlier standalone CLI backend runs below retain their dated scope and evidence. No model turn or existing client configuration change was performed.

## Installed surfaces

| Surface | Observed version | Evidence and limitation |
| --- | --- | --- |
| Codex CLI | `codex-cli 0.146.0` | `/Users/nazmiyilmaz/.nvm/versions/node/v24.18.1/bin/codex`; version/help exit 0 |
| Codex Desktop app | `26.1016.62342`, build `21477` | `/Applications/ChatGPT.app/Contents/Info.plist`; bundle ID `com.openai.codex`, executable `ChatGPT`; CLI/backend parity not established |
| Codex Desktop bundled backend | `codex-cli 0.163.0-alpha.11.2` | Actual app-bundled executable, verified against the active Desktop process; isolated installed-release journey passed. [Evidence and limits](m0-codex-native-qualification.md) |
| Native Windows | Unavailable | Neither Windows architecture nor Windows desktop behavior qualified here |

Raw command outputs, invocation metadata and fetched Codex documentation are retained outside shipped source in `/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/client-inspection/`. The CLI's development Node 24.18.1 is separate from the proposed private Mirket Node 24.22.1 payload.

## Common disposable fixture

Use an absolute disposable project, a fresh Mirket state directory and a pinned qualification release. The current development entry point is:

```text
/ABS/RELEASE/mirket dev serve --state-dir /ABS/FIXTURE/state --release-id RELEASE_ID --node /ABS/RELEASE/runtime/node/bin/node --adapter /ABS/ADAPTER/main.js
```

Confirm the compiled adapter's actual path before invocation. These are placeholders; the executed paths and result are recorded below. Windows uses its selected `node.exe` path. The state directory must be mode 0700 on macOS; the first probe failed because the disposable directory was too open. All qualification output and configuration belong under the fixture, never existing client homes.

The seven selected inspection tools for this bounded fixture are `runtime_status`, `guides_list`, `guide_read`, `file_read`, `artifact_read`, `job_status`, and `job_result`. Exclude effect tools from this fixture. Preparation reserves operation state and is not a pure read. `readOnlyHint` annotations are descriptions, not authorization enforcement.

The canonical registry now contains the shared workflow, the twelve capability guides and a separate Memory operations guide. The bounded Codex fixture below exercised `shared-workflow`, `engineering`, `documents` and `memory` before the other guides were authored. Native skill wrappers should remain thin: metadata identifies the skill, then requests the current canonical guide through `guide_read` with `expected_release_id`. Do not embed another evolving copy of the full corpus in a test wrapper.

## Codex CLI backend route

Installed help supports `--strict-config`, `-c key=value`, `-C /absolute/project`, and `app-server --listen stdio://`. `--profile` is an overlay on the base configuration, not isolation. Create a disposable `config.toml` and set `CODEX_HOME` **only in the child-process environment**, leaving the host environment and existing directory untouched. `CODEX_HOME` relocates auth/history/config state, but does not prove isolation from managed configuration or skills in `$HOME/.agents/skills` and `/etc/codex/skills`. Do not copy authentication files. [Configuration](https://developers.openai.com/codex/config-advanced/), [skills](https://developers.openai.com/codex/skills/).

Candidate configuration, with concrete fixture paths substituted:

```toml
[mcp_servers.mirket_m0]
command = "/ABS/RELEASE/mirket"
args = ["dev", "serve", "--state-dir", "/ABS/FIXTURE/state", "--release-id", "RELEASE_ID", "--node", "/ABS/RELEASE/runtime/node/bin/node", "--adapter", "/ABS/ADAPTER/main.js"]
startup_timeout_sec = 60
enabled_tools = ["runtime_status", "guides_list", "guide_read", "file_read", "artifact_read", "job_status", "job_result"]
disabled_tools = ["text_replace_prepare", "text_replace_start", "job_cancel"]
```

The official MCP configuration supports these allow/deny lists; deny entries take precedence. CLI and desktop share configuration conventions, but this does not establish that their currently installed implementations are identical. [Codex MCP](https://developers.openai.com/codex/mcp/).

The 2026-10-25 probe used the installed app-server's generated JSON schema to send `initialize`, `initialized`, `skills/list`, `mcpServerStatus/list`, an ephemeral `thread/start`, and read-only `mcpServer/tool/call` requests. It did not call `turn/start`. The isolated client still attempted to warm a featured-plugin cache on one run and received HTTP 401; do not claim network silence from the absence of model generation. [App-server protocol](https://developers.openai.com/codex/app-server/).

Put the test wrapper at `/ABS/FIXTURE/project/.agents/skills/mirket-engineering/SKILL.md`; use `skills/list` with that cwd to establish metadata discovery. A backend tool inventory and successful direct `guide_read` establish protocol integration only. A low-level tool-call API might have different authorization semantics from model-visible tools, so catalogue absence alone must not be labelled an enforced start denial. If a direct call reaches Mirket despite the filter, report that boundary explicitly.

## Codex CLI backend result — 2026-10-25

With `CODEX_HOME` scoped to a disposable child directory, installed `codex-cli 0.146.0` discovered the fixture's project skill and Mirket server. `mcpServerStatus/list` exposed exactly the seven configured read tools. An ephemeral thread used the installed `mcpServer/tool/call` route to read the current Engineering guide, `runtime_status` and a fixture file; the file's returned SHA-256 matched its saved bytes. A stale release ID was refused. No `turn/start` or model generation occurred. The complete bounded probe, installed schema and responses are retained under `/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/client-qualification-20261025/` outside shipped source.

The repeatable [Codex app-server fixture](../tests/codex-app-server-smoke.mjs) passed against the rebuilt local core with a fresh disposable home and project. Read mode checks the exact seven-tool inventory, guide list and saved file revision. Policy mode enables preparation, creates a valid operation, then calls the disabled start tool through Codex's low-level host API. Codex returned its own disabled-tool error; the source stayed unchanged, no destination appeared and the preparation receipt remained inspectable. Both modes remove their disposable fixtures. The local `target/debug/mirket` identity in these runs was SHA-256 `36f5dbb8ccc85f15b118024543506e5bddccd352c35bfc9ce48de876b1021bff`. This is a development binary, not an installed release.

On 2026-10-25 the same fixture also exercised an actual local `setup --home --source` release using a private Node 24.22.1 executable, compiled adapter and current Rust core. `doctor --json` reported `healthy`, the exact active release and `aarch64-apple-darwin`. With `--home` instead of development `--node`/`--adapter` arguments, both read and policy modes passed through `serve --home`; the host filter denied a valid prepared start. This tested a separate 167 MiB disposable installed home, not the user's existing `.mirket` or Codex configuration. It still uses the Codex CLI app-server backend and no model turn, so it does not establish native desktop activation. The fixture now accepts either the explicit development runtime pair or an already verified installed home.

The first attempt returned an empty server inventory because the fixture state directory had group access. Mirket's own stderr identified the mode error; after mode 0700, the same installed route succeeded. This verifies CLI backend discovery, explicit read calls and one genuine denied start at the low-level host API. It does not prove model-visible tool permissions, persistent denial after reconnect, or desktop activation. The isolated client made one failed featured-plugin request (HTTP 401) during an ephemeral-thread run, so the probe is not evidence of an entirely offline host session.

## Required evidence before acceptance

| Check | Discriminating observation | Status here |
| --- | --- | --- |
| Native discovery | Expected wrapper metadata appears in actual host's skill list; canonical guide and release identity retrieved | Desktop bundled-backend passed; GUI not run |
| Inspection allowed | Actual host calls `runtime_status`, `guides_list`, `guide_read`, and a valid bounded `file_read`; responses retain identity | Desktop bundled-backend calls passed; model-visible route not run |
| Start denied | A valid prepared-operation start is denied by host policy, with no call reaching Mirket and no child or published artifact | Desktop bundled-backend filter refused valid preparations; proxy trace confirms no start dispatch and no artifact appeared. Model-visible path remains untested |
| Persistent denial | Repeat in a fresh/reconnected host session with the same policy and release; retain rejection evidence | Desktop bundled-backend passed with fresh same-owner preparations |
| Release transition | After explicit disposable activation, old expected release fails and new guide retrieval succeeds in the real host | Desktop bundled-backend passed; GUI/model path not run |
| Native desktop journey | Capture actual Codex Desktop discovery, tool inventory, inspection and denial | Not run |
| Codex Desktop bundled backend | `codex-cli 0.163.0-alpha.11.2` | Actual app-bundled executable, verified against the active Desktop process; isolated installed-release journey passed. [Evidence and limits](m0-codex-native-qualification.md) |
| Native Windows | Repeat platform process and host checks on Windows ARM64/x64 | Unavailable |

For the denial test, prepare a valid operation through a separately authorized fixture setup before enabling the read-only host policy. An invalid operation ID, missing file, expired revision, or service-side validation error is a false-positive denial. Record the host error, effective tool inventory, Mirket dispatch/effect evidence, operation receipt and artifact hashes before/after, and absence of a spawned child. Denied-start evidence must show the host boundary; a direct service request is a control test, not that evidence. These proposed checks are distinct from already available Rust, adapter and direct MCP tests.

The current inspection establishes versions and candidate supported routes. It does not establish authentication readiness, desktop activation, policy enforcement, background-network behavior, or Windows compatibility.

## Installed maintenance and managed-registration repeat — 2026-10-25

The repeatable [installed maintenance fixture](../tests/installed-maintenance-smoke.mjs) created two private macOS ARM64 releases in a disposable home. With a live MCP session holding the active release, top-level `update` left the second release staged and the launcher unchanged. After a clean disconnect, the same update activated it; `doctor` verified the exact active manifest and launcher. The fixture then registered that installation in a separate disposable Codex config and passed the real Codex CLI app-server read journey: skill discovery, guide/runtime/file inspection and stale-release refusal. It removed only its managed config entry afterward. A changed launcher caused a later update to refuse before staging or activation, preserving the changed file for inspection.

The first maintenance run exposed an OS path alias: macOS reported a running executable through `/var` while the verified home was under `/private/var`. The running-core check now canonicalizes only that OS-reported parent and still rejects a symlinked executable leaf. Rust unit tests, the locked strict Clippy check and the repeated installed maintenance/Codex fixture passed after that fix. This remains a CLI-backend qualification, not a native Codex Desktop or Windows result.

## Combined release Codex repeat — 2026-10-25

The fresh four-component disposable release
`sha256:510a84eed465a4cbe35c6287cfbcefba7911ffdef27e338cecf42957cb637346`
passed `doctor --json`, the installed HTML/CSS source-inspection smoke and the
combined five-dialect SQL/TOML/XML/YAML/PDF smoke. The same installed home
passed all three modes of `tests/codex-app-server-smoke.mjs`: read, policy and
managed registration. Codex's low-level host filter denied a valid prepared
start without changing the source or creating a destination. This tests the
Codex CLI app-server backend with disposable configuration and no model turn;
the GUI/model path and Windows remain open.
