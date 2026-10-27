# Codex Desktop GUI activation path

This is a read-only path review dated 2026-10-26, not a completed GUI
qualification. A fresh GUI chat in a trusted disposable project is the
recommended first journey. Its project MCP configuration can select a separate
installed Mirket without replacing the user's global registration, restarting
Desktop, or refreshing the current chat's MCP connection. Actual GUI discovery
and calls must still be observed before closing that gate.

The inspected app is `/Applications/ChatGPT.app`, bundle ID
`com.openai.codex`, version `26.1016.62342`, build `21477`, on macOS 26.5.2
ARM64. Its running main process and two bundled Codex backend processes were
visible. The earlier [bundled-backend qualification](m0-codex-native-qualification.md),
[startup measurements](m3-verified-startup-latency.md), and CLI skill trials
remain separate evidence. None demonstrates this GUI journey.

## Disposable project route

Official OpenAI documentation supports MCP entries in trusted
project `.codex/config.toml` files. Project configuration overrides the same
keys in the user configuration; untrusted project layers are ignored. See
[MCP configuration](https://learn.chatgpt.com/docs/extend/mcp#connect-codex-to-an-mcp-server)
and [project configuration](https://learn.chatgpt.com/docs/config-file/config-advanced#project-config-files-codexconfigtoml).
This supports the route below, but does not establish that the installed GUI
has loaded the intended layer until a real chat returns the expected identity.

Prerequisites are an immutable, admitted release source; an absolute frozen
Mirket executable; a new private installation home; a separate disposable
project outside the old/new harness repositories; and a GUI operator able to
select and trust that project. Retain the exact release ID, source identity,
core/adapter hashes and available capability inventory. Do not rebuild over
the candidate during the journey. A core-only fixture proves only the selected
slice; use the complete admitted candidate for a complete-release claim.

1. Record the current main app/backend PIDs, the global config hash and the
   existing Mirket process identity without reading credentials. Keep the
   ongoing chat active and do not send it a new task or reload request.
2. Create a fresh private project and `.codex` directory. Place a small saved
   TypeScript probe there. Freeze its expected bytes and SHA-256 outside the
   model's write scope. A small local Git
   repository gives the client an unambiguous project root; it needs no remote.
3. Install the candidate in its own disposable home, then register that
   installation directly in the project's config. The existing explicit CLI
   supports this without a hand-written launcher or copied registration:

   ```sh
   /absolute/frozen/mirket dev setup-core \
     --home /absolute/private/gui-mirket \
     --source /absolute/frozen/release
   /absolute/frozen/mirket codex-register \
     --home /absolute/private/gui-mirket \
     --config /absolute/private/gui-project/.codex/config.toml
   /absolute/frozen/mirket codex-doctor \
     --home /absolute/private/gui-mirket \
     --config /absolute/private/gui-project/.codex/config.toml
   ```

   These are instructions for a later authorized run; this review did not
   execute them. Use a fresh installation home: one installation's native
   registration receipt belongs to its explicitly selected config path and
   must not be rebound casually between test homes or configs.
4. In the actual Desktop GUI, add/open the disposable local project, complete
   the project trust decision if requested, and start a fresh chat using that
   exact directory. Preserve a screenshot of the project/chat entry state.
   Keep the existing harness chat open. Do not use a fork carrying old tool or
   instruction context for this first activation check.
5. In that GUI chat, allow metadata-only discovery and request the Mirket
   `runtime_status` call followed by `code_navigate` on the TypeScript probe.
   Require the exact independently frozen release ID, saved-source revision and
   definition span. General file reading belongs to Codex and is no longer a
   Mirket tool.
   Inspect the actual tool results, not only the assistant's final assertion.
   Retain the chat ID, tool calls/results, elapsed observations and a screenshot
   showing the real Desktop chat. If tools are unavailable or the identity is
   old, stop that trial and retain the failure. Check trust, effective config
   and new-thread connection selection before trying a different route.
6. Return to the original chat without reloading its configuration. Confirm
   its previous process/connection identity is retained and the global config
   hash is unchanged. Retain both successful and failed attempts.

The rewrite's registration owner is `src/codex_config.rs`. It writes exactly
`command = <home>/bin/mirket`, `args = ["serve", "--home", <home>]`,
`startup_timeout_sec = 60`, and `required = true`. It does not write global
catalog-grace, approval, model, credential or plugin settings. It requires an
absolute regular config path, a real user-owned config parent that is not
group/other writable, and private installation/receipt roots. Do not bypass its
ownership or drift refusal by editing receipts.

The inspected global registration currently selects the old executable
`/Users/nazmiyilmaz/.mirket/bin/mirket` with arguments
`["--home", "/Users/nazmiyilmaz", "mcp", "serve"]`. Its only other entry key is
`tool_timeout_sec`, consistent with the old owner's `src/install.rs`.
It has no `enabled_tools` or `disabled_tools` list to inherit. Project overrides
may still inherit that tool timeout; record effective settings instead of
calling this a factory-default timing comparison. Current global config bytes
have a private baseline hash in the review evidence. Do not publish config or
authentication contents.

This first journey needs a fresh GUI chat and possibly an explicit trust
decision. An authorized GUI operator can perform it without restarting the app;
if the host cannot select/trust a new project without affecting the active
chat, the user must do those steps. The read-only review itself required no user
intervention. No credential copying or sign-in change is required by this
project route; actual account/model readiness still needs observation.

## Evidence needed beyond a successful navigation

The discriminating failure is the wrong Mirket returning a plausible location:
freeze the new release ID and source revision outside the worker's scope, require both,
and retain the original connection identity as a control. A visible catalog
alone is insufficient: the installed deferred catalog can precede payload
verification, while every tool call waits for exact release verification.
Record the first actual call and its failure if present. Do not warm the
candidate with unrecorded calls or change global grace to obtain a pass.

This route qualifies a GUI connection and a useful read, not complete managed
global installation or guide benefit. The existing `$HOME/.agents/skills` and
user instructions remain visible. Selected project skills can be installed in
the disposable project through `skills project-select`, but the resulting
trial is still contaminated by unrelated global guidance unless its loaded
sources are independently established. Follow the
[native evaluation contract](m4-guide-evaluation.md) for frozen matched cases,
guide reads, host permission observations, saved artifacts and independently
checked outcomes. A read-only trial does not prove approval behavior or a
successful design write. Record the actual host policy and any refusal; do not
weaken the current chat's policy to complete another trial.

Rollback is local: let the test chat finish, close/archive only that test chat
through the native GUI, verify its Mirket PID has exited, and run
`codex-remove` against the same disposable config using its owning installation.
Retain evidence before removing task-owned project/payload files. The temporary
project trust/recent-project entry may persist in client state; remove that
specific entry through the UI if desired. Do not restore a whole global config
snapshot over concurrent edits or kill processes by the generic app name.

## Alternate isolated Desktop instance

Read-only inspection of the installed `app.asar` found an internal demo
launcher that starts another instance of this same app using:

```sh
/usr/bin/open -n \
  --env CODEX_HOME=/absolute/private/gui-user/.codex \
  --env CODEX_ELECTRON_USER_DATA_PATH=/absolute/private/gui-electron \
  /Applications/ChatGPT.app \
  --args --user-data-dir=/absolute/private/gui-electron
```

The installed bootstrap resolves Electron `userData` from
`CODEX_ELECTRON_USER_DATA_PATH` before requesting the single-instance lock.
With that explicit path it enables the lock; it does not disable it. The
startup environment loader preserves an explicitly paired `CODEX_HOME`, and
the app resolves Codex state from that variable. These observations support a
version-specific isolated-instance candidate. The demo UI additionally checks
for an internal release track, so this source evidence is not a promise of a
public profile feature or proof that manually launching succeeds.

This review did not launch that command. A future run must first prove a new
main PID, distinct user-data/config roots and unchanged original PIDs and
global-config hash. Do not infer isolation from a second window. Stop if it
forwards into the current app, opens the user's existing state, or requests an
installation/update. Quit only the identified test instance after checking its
owned children; `killall`, app-wide Quit and a Desktop restart are inappropriate
while the current task runs.

`CODEX_HOME` relocates config/history and file-backed authentication, not the
OS account, global `$HOME/.agents/skills`, system requirements, keychain,
network permissions or registered URL scheme. A separate instance can also
trigger normal startup networking or OS integration. Do not change `HOME`,
copy authentication files, change protocol handlers, or infer clean native
skill isolation from these two variables. Official
[state-location](https://learn.chatgpt.com/docs/config-file/config-advanced#config-and-state-locations)
and [credential-storage](https://learn.chatgpt.com/docs/auth#credential-storage)
documentation describes these separate stores. A clean instance may require
interactive sign-in or OS keychain approval; the user must perform credential
or authentication prompts if encountered. Keep credentials out of evidence.

For the complete native setup path, a disposable user home must actually be
the app's effective user skill home: setup writes
`<user-home>/.agents/skills`, whereas `CODEX_HOME` alone does not redirect that
location. Do not report full setup/skill activation from a disposable
`--user-home` paired only with `CODEX_HOME`. A separate macOS login account is
the stronger clean-home option and requires user session management. Replacing
the real old installation is another separately staged transition, after this
task's active sessions are safely finished. Neither is necessary for the first
project-scoped GUI read.

Only this document and private review evidence were written. No client config,
installation, skill file, account state or GUI was changed, and no app-server,
model turn, commit, push or publication was performed by this review.

## Coordinator preparation — 2026-10-26

A separate disposable **core-only** release was assembled from the frozen
private Node/adapter pair and the post-release-store-fix Mirket executable.
Its four-file manifest has SHA-256
`69b643df74ad1010aa42e4f0e9e6b05245fbc7c1e9a63a404ad6efb9f54b4087`;
the executable has SHA-256
`b14034bc5693e3a8039179cc444b13a87090af593412a3a029ecb0790a216324`.
`dev setup-core`, project-scoped `codex-register`, `codex-doctor` and
`dev doctor-core` all returned success. The active release is
`sha256:4c3541c0321be149cb2d6780c9b53bf65acea358b9802eaee79da37a2b728900`.
This qualifies fixture preparation and managed registration only; it contains
no optional component and has not been opened by the Desktop GUI.
It was frozen before the later Video-only source preparation; the rebuilt
current development executable has SHA-256
`99990dc27ea9518df0b3bb3af485d93175b554367880be8074521a2263d94b3e`.
Any eventual complete-release GUI acceptance must use its own exact final
release identity.

The new project is
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/codex-gui-core-20261026/gui-project`.
Its saved UTF-8 `nonce.txt` has SHA-256
`2df5a70fd95493b12645390e77c8feda0f8e5fafeab557b53b20d3d2d3bceb7e`.
The preparation receipt is private alongside the source, home and project. The
real global Codex config SHA-256 stayed
`7cf4d5f11d2a6593a35df91b3fa347638bc614ab3c165c231f1e51e613898317`.

Direct computer-use binding to `com.openai.codex` was rejected with the tool
message “Computer Use is not allowed to use the app 'com.openai.codex' for
safety reasons.” No GUI input was sent and no alternate UI automation route
was attempted. A user-operated fresh GUI chat is still required. A separate
installed-MCP control then initialized the exact disposable launcher, listed
44 tools, called `runtime_status` and read the independently frozen Unicode
nonce with the expected release ID. The text hash matched the fixture; its
private result has SHA-256
`82a854e8e35a01f67006d8788a1af8037eb63bf347bf9440e478041f431233b2`.
That is protocol evidence, not a Desktop GUI pass. The original global config
hash remained unchanged after the control.

## Current 2026-10-27 GUI fixture

The 2026-10-26 core-only `file_read` fixture above is historical; that public
tool was removed in the accepted simplification. The current disposable project
is
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/simplification-20261027/gui-project-fts-first`.
Its project-local `.codex/config.toml` was registered with the latest installed
Design release; `codex-doctor` reported `managed` and no pending transaction.
That release ID is
`sha256:440b5f9f64d9d43f7281834c3fd730950c93415e81985c880dd3f48c2f9e5495`.
The unrelated global Codex configuration and real `.mirket` installation were
not changed.

The saved `probe.ts` is exactly:

```ts
export const guiProbe = "MIRKET-GUI-2026-10-27-FTS";
guiProbe;
```

Its SHA-256 revision is
`sha256:09a9bd96e45d12993529af0c52b6cfc792eba559cdbefd82f28746ce686c980c`.
For `code_navigate` `definitions`, use this file as the sole selected `ts`
source, `position_byte: 53` at the second `guiProbe`, and
`typescript_model: {"imports":[]}`. The expected definition occupies original
UTF-8 byte span `[13,21)`. A direct installed MCP call on this exact probe
returned that span with TypeScript language service 5.9.3. The broader
installed navigation fixture and a managed Codex app-server call also passed
on the same release.

A noninteractive Codex CLI model turn then used an explicit, limited MCP
override to call `runtime_status` and `code_navigate` against that installed
release. It returned the exact release ID and definition byte span `[13,21)`
for the frozen source revision. Exit code was zero; elapsed time was 14.926
seconds; the unrelated global Codex config hash was unchanged. Retained private
JSONL is `simplification-20261027/codex-model-probe-20261027/events.jsonl`,
SHA-256 `b6d1619b2d01f6646afcf4494596d30387511611258aff573f01e01152644ed1`.
The explicit CLI override does not prove that Desktop loads this project's
`.codex/config.toml` or activates its skills. The actual Desktop GUI gate
remains open because this host's computer-use tool refuses control of
`com.openai.codex`.
