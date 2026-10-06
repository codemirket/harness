# Portable Codex preferences

[registry/codex-settings.json](../registry/codex-settings.json) is the shared preference source. It captures configured values from this Mac, rather than inventing values for absent defaults. The initial snapshot contains 43 leaves:

| Area | Included values |
| --- | --- |
| Model behavior | `gpt-6-astra`, `xhigh` reasoning, pragmatic personality, low verbosity, concise reasoning summaries |
| Appearance | Codex light and GitHub dark code themes; light/dark chrome colors, contrast, transparency, semantic colors and configured code font |
| Interaction | Conversation detail, follow-up queuing, context usage display, full browser URLs and visible reasoning choices |
| Plugins | Enabled flags for 13 official bundled/runtime plugins, including browser, documents, PDF, spreadsheets, presentations and visualization |

The configured font list retains fallbacks. Device, app version and account availability still determine supported models, reasoning levels, fonts and plugins. No model is silently substituted. `appearanceTheme` was not explicitly configured on the source device, so this snapshot does not force light/dark/system mode on other devices.

## Capture and apply

```sh
python3 ai.py settings capture
python3 ai.py settings capture --output registry/codex-settings.json
python3 ai.py settings plan
python3 ai.py settings apply
python3 ai.py settings doctor
```

Capture prints only allowlisted values unless an output is selected. An explicit output refreshes that file; review the repository diff before sharing it. Use `--settings /path/to/preferences.json` to plan/apply/check another reviewed source. `--home /existing/user-home` targets that home's `.codex`; otherwise the command respects `CODEX_HOME` and defaults to `~/.codex`.

Only allowlisted leaves are merged into `config.toml`. Unselected values, tables, comments and line endings are retained. Comments inside a replaced selected value may change. Missing source keys do not delete destination settings. The merge supports selected scalar and homogeneous array values; an unsupported selected inline table or multiline value stops with a manual-edit instruction. It never guesses at or rewrites unknown structured configuration.

Close Codex desktop and other active Codex clients before applying a change. The CLI detects the desktop process conservatively and refuses writes if it is running or process state cannot be established. It checks again before replacement, uses a lock against another settings sync, and detects concurrent file changes. An unchanged configuration needs no closure and produces no writes. The CLI does not terminate clients or restart them.

An existing config is backed up to a private `backups/settings-<id>/config.toml` under the target Codex home before atomic replacement. Backups contain the full previous local config and stay on the device; they are not repository artifacts. Inspect before restoring, and close clients first. A stale `.personal-ai-settings.lock` requires checking that no sync is active before removal. These checks are not a transaction against arbitrary concurrent writers.

## Deliberate exclusions

The source omits authentication, tokens, account identifiers, connector state, permissions, sandbox and approval settings, browser access policies, trusted projects, filesystem paths, worktrees, remote devices, windows, conversation history, recent models, environment variables, commands, MCP servers, hooks, experiments and private/local marketplaces. It never copies desktop global-state or persisted-atom files. One private/local plugin was excluded from the initial capture.

Plugin synchronization copies only `enabled` flags for approved public marketplace IDs and the personal marketplace namespace. It does not copy installation directories, permissions, configuration secrets or accounts. A flag alone does not install a plugin. Codex may fetch configured plugins during its own marketplace refresh; availability and account connections must be verified in the target app. Direct global skills and plugin bundles can overlap, so choose one route for the same skill set.

## Contract and verification

The root model preferences use the [Codex configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference). Appearance and interaction keys were verified against this installed app's settings schema and migration code; they are version-dependent and may change. The snapshot records that contract review version and date. See [app settings](https://learn.chatgpt.com/docs/reference/settings) and [plugin configuration](https://developers.openai.com/plugins/build/plugins).

`doctor` verifies configured values only. It does not establish visual rendering, model access, account entitlement or plugin execution. `ai.py check` additionally checks the three runtimes, global registrations and the daily OS schedule. Native Windows execution and visual parity need confirmation on Windows; fixture tests on macOS establish file behavior only.
