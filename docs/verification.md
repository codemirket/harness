# Verification evidence

Completed on macOS, 2026-10-06. These checks establish reviewed bytes, registration, reconciliation and packaging. They do not certify every upstream workflow or every target device. Detailed per-entry results and input hashes are preserved in the [registration evidence](evidence/registration-2026-10-06.json).

| Check | Result |
| --- | --- |
| Automated behavior tests | 146 tests pass across registrar, source remaps/replacements, registry data, global/project reconciliation, wrappers and plugin export. |
| Source inventory | 28 pinned sources, 4,186 physical discovery records; the six added sources contribute 78 canonical bodies and 12 planning host mirrors. Per-source reports distinguish full body reads from partial runtime/reference review. |
| Authored skill structure | All 29 authored skill folders pass the skill-creator validator. |
| Project payloads | All 195 selections pass current source/adapted hash, name, license/extra-file and payload preparation checks. |
| Actual isolated registration | All 195 entries install for every declared provider, with unchanged-repeat, receipt, content and executable-mode checks. 194 support both hosts; Claude-only Git guardrails installs for Claude and rejects Codex. |
| New real downloads | All 17 Addy/Vercel project payloads fetched through two pinned archives match the reviewed local payloads, including shared-reference copies and literal corrections. |
| Existing upstream downloads | Prior checks verified the unchanged 154 upstream selections using 12 archives plus bounded tree/raw-file retrieval for two large sources. Current local payload/registration checks were rerun after the installer refactor. |
| Profiles | All 60 resolve to installable selections with required companions and no internal conflicts. |
| Fresh usage simulation | An existing Next.js/PostgreSQL fixture produced 11 skills / 22 proposed registrations through the catalog skill's real commands, preserving project source, conventions and custom skills. No installation or dependency action was taken by that simulation. |
| Global reconciliation | Link/copy behavior, managed updates, user-edit preservation, legacy migration, predictable preflight failures and backup recovery exercised with isolated homes. Real Mac setup now verifies 30 unchanged destinations: 14 skills and one guidance file for each host. |
| Project update regressions | New executable helpers, old executable-mode changes, missing/stale lock, unsupported metadata, parent file collisions and parent-link changes during preparation are covered. |
| Plugin export | All six declared bundles export together in an isolated build. Tests verify deterministic manifests/file locks, licenses, executable modes, collisions, failed-payload cleanup and atomic no-replace publication. The exported catalog CLI runs after its original fixture checkout is deleted. |
| Native plugin validation | Claude Code's installed `plugin validate` accepts the marketplace and all six plugin manifests without manifest warnings. This validates packaging, not live activation. |
| Shared guidance | AGENTS.md and CLAUDE.md match at 75 lines each; skill pickup is required for substantial work with unchanged selections reused. |
| Documentation/configuration | Generated documentation drift check, repository-relative links, shell syntax and Git whitespace checks pass. |

The global set was applied to this Mac. The old `components/AGENTS.md` was removed after its installed links migrated to `instructions/`. Unrelated registrations and scheduled jobs were preserved. Existing scheduled wrapper calls use the same new declarative installer. No cron schedule or other device was changed.

The registration API verifies source and adapted hashes separately. Extra files outside an upstream skill directory must be individually declared, mapped to safe relative paths and included in the source hash. Literal corrections require exact occurrence counts. Downloads and exports retain size/count limits; no limit was relaxed to pass a check. Upstream helpers are copied when reviewed but never executed during these checks.

Known preflight failures leave requested skill destinations unchanged. An OS failure after installation starts can leave earlier completed items, with receipts and per-item recovery. Parent rechecks narrow concurrent mutation risks; the filesystem operations are not a general transaction or a security sandbox against a hostile concurrent process. Unselected or locally edited skills are preserved for deliberate cleanup.

Native Windows execution, live Codex/Claude discovery after refresh, paid APIs, browser binaries, cloud deployment and full runtime packages remain unverified. Windows managed-copy logic and portable paths were exercised on macOS, which is not native Windows evidence. Marketplace generation does not activate a plugin or change app settings. Instruction files cannot guarantee model behavior.

Reproduce repository checks:

```sh
python3 scripts/render_registry.py --check
python3 -m unittest discover -s tests
sh -n setup/macos.sh
git diff --check
python3 ai.py doctor --target both
```

After an upstream update, re-review changed bodies, companions, licenses and helpers; refresh hashes and rerun affected source/registration checks. After source changes, regenerate the marketplace rather than reusing a stale build. The build's own lock records the exact packaged bytes.
