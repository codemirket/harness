# Verification evidence

Verified on macOS, 2026-10-06, for the version 2 Codex desktop harness. Python 3.9.6 is the system interpreter used for the suite. These checks establish file behavior, runtime accessibility and bounded CLI operation; they do not certify every upstream workflow or target device.

| Check | Result |
| --- | --- |
| Automated behavior tests | 283 tests pass across catalog registration, global/project reconciliation, wrappers, runtime diagnostics, settings, Claude delegation, combined installation, OS scheduling, Git maintenance and Codex plugin export. |
| Installed runtimes | Codex desktop `26.930.61225` (build `13232`) is recognized and its executable accessible; Codex CLI `0.160.1` and Claude Code CLI `2.1.287` execute successfully. Desktop was not launched by diagnostics. |
| CLI authentication | Both official status commands report authenticated. Account IDs, emails and credentials are omitted. Status is not a general model entitlement or quota test. |
| Actual full installation | `ai.py install` passes on this Mac with native midnight schedule registration. All 30 guidance/skill destinations match. The 43 portable configured values already match, so no live app preferences were rewritten. |
| Native scheduling | The actual Mac user crontab now contains one owned `0 0 * * *` maintenance job. The exact legacy installer line was migrated, unrelated cron bytes were compared and preserved, and a private backup was retained. Repeated schedule checks report unchanged. 34 isolated scheduler tests cover migration, quoting, inherited Python environment, concurrency and Windows task contracts. |
| Scheduled update flow | 22 focused maintenance tests exercise local bare Git repositories, actual fast-forward plus fresh installer launch, dirty/ahead/divergent states, hooks/filter/submodule guards, credential configuration preservation, process locks, timeouts and bounded logs. A real launch of the registered argv from `/` with a minimal cron-like environment recorded `skipped_dirty`, preserving the current uncommitted work without fetching. Future cron delivery is not established by this manual run. |
| Portable settings | Isolated tests cover allowlist exclusions, unrelated TOML/comment preservation, Unicode/CRLF handling, structural conflicts, open/unknown-app deferral, no-op while open, backup/replace failures, concurrent edits and symlink refusal. Focused settings tests also run on a newer Python with full TOML validation. |
| Installer failures | Missing runtimes, unmanaged guidance, invalid settings and an open app block known writes. Tests cover settings failure after global setup, failure partway through global installation, authentication attention and unsupported custom `CODEX_HOME`. |
| Live Claude delegation | Subscription route verified. A bounded connectivity request returned the requested text; a separate Read-tool request returned a random value present only in a temporary fixture file. A larger code-review request exceeded its 120-second limit and was cancelled without publishing a result. No automatic retry or billing fallback occurred. |
| Delegation boundaries | Tests cover unsupported CLI controls, API opt-in, sensitive-path denials, preserved positive Read-deny rules, disabled updates, malformed/failing results, bounded output, timeouts/process-tree cancellation, partial write cleanup and no-overwrite publication. |
| Changed authored skills | Both changed skills pass the skill-creator validator. The updated coordination payload and its Claude reference install and repeat unchanged in isolated Codex and Claude Code project directories. |
| Global guidance | AGENTS.md is 80 lines; Claude Code guidance is 34 lines. Codex desktop is primary, native Codex/ChatGPT workers are preferred, and Claude Code remains a selective supporting CLI. |
| Codex packaging | All six declared bundles export as version 2 with Codex marketplace/plugin manifests, verified skill bytes, retained licenses and deterministic file locks. No Claude marketplace or plugin manifests are generated. The foundation includes the settings source and complete installation/delegation engine. |
| Documentation | Generated tables, local Markdown references, shell syntax and Git whitespace checks pass. |

The prior [registration evidence](evidence/registration-2026-10-06.json) records review and actual isolated installation of all 195 project selections across their declared providers, 60 resolved profiles, 28 pinned sources and 4,186 inventory records. It also records the source/adapted hashes and executable modes for that baseline. Upstream pins and payloads are unchanged by version 2; the changed authored coordination selection has a newly reviewed catalog hash and was reinstalled separately. Historical Claude marketplace validation describes the previous exporter, not current Claude desktop support.

The current six-bundle export is `build/personal-marketplace-v2` (ignored generated output). Its `build-lock.json` records exact packaged bytes and modes. Older generated directories are not updated in place; use the version 2 export deliberately. Neither generation nor preference capture activates a marketplace, connects an account or publishes remotely.

Known preflight failures preserve target files. Global installation has per-item receipts/recovery, and settings use a local backup and atomic replacement; together they are not a whole-install transaction. Process/path checks are not an OS security sandbox. Unselected or modified skills remain for deliberate cleanup. This Mac’s legacy midnight cron entry was replaced by the managed maintenance job. No other device was changed. The checkout must become clean before automatic pulls proceed; it is not stashed or committed by maintenance.

Native Windows Task Scheduler execution, Windows process cancellation, visual theme parity and full plugin discovery after client refresh remain unverified. Windows branches were exercised with fixtures/static review on macOS. Paid API routes, browser binaries, cloud deployment and upstream runtime packages were not provisioned. CLI controls and instruction files cannot guarantee model behavior or detect every secret path.

Reproduce local checks:

```sh
python3 scripts/render_registry.py --check
python3 -m unittest discover -s tests
sh -n setup/macos.sh
git diff --check
python3 ai.py runtime doctor --check-auth --json
python3 ai.py settings doctor
python3 ai.py check
python3 ai.py schedule check
python3 ai.py maintenance status
```

After source updates, re-review changed bodies, companions, licenses and helpers; refresh hashes and run affected registration checks. Finish source changes before generating a new export. Check actual client discovery and one bounded workflow before distributing a build to another device.
