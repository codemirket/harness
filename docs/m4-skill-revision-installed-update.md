# Revised native skills: installed update qualification

On 2026-10-26, one explicit prepared-candidate core update successfully replaced
the frozen prior `design` + `skills` release in a disposable macOS 26.5.2 ARM64
installation. The revised catalog and all three revised skill bodies match the
installed release. Native global `runtime-operations` and project `vector-art`
copies match their revised bodies; full doctor is healthy, project doctor is
managed, and repeat project sync is unchanged. One bounded actual Codex CLI turn
then read the complete revised vector skill and observed the expected running
release through `runtime_status`.

The first update attempt exposed and preserved a verifier defect before the
owning coordinator repaired it. The successful route invokes the prepared new
candidate executable explicitly. The prior frozen stable launcher predates this
protocol and cannot admit the revised candidate. No automatic launcher handoff,
model-produced design artifact, Desktop GUI activation or installed
post-activation rollback is established by this qualification.

This record extends the [source-level path clarification](m4-skill-path-contract-repair.md),
[earlier native design skill trial](m4-codex-design-skill-tool-qualification.md)
and [native installation qualification](codex-native-installation-qualification.md).
Exact private evidence is retained at
`~/.codex/task-state/harness-rewrite-implementation/skill-revision-installed-update-20261026/`.

## Frozen inputs and installed baseline

The fixture is a new task-owned installation home, fake user home and non-Git
project whose path contains spaces and `İ`. It reuses earlier frozen design
runtime and adapter bytes by filesystem copy. The original fixture and user
installation are not qualification destinations. No upstream helper, new
dependency or source build ran in this evaluator; the coordinator supplied the
separately built candidate cores.

The prior release is
`sha256:76bfecd888ce8c258f7f97ce55bcad29b76ea494cc2f47f967b6df8f1a824c63`.
Fresh top-level `setup` completed uninterrupted in 138.008 seconds. Its full
`doctor` reported healthy, including managed global skills, guidance and Codex
registration. Explicit `skills project-select --include vector-art` installed
that project skill and project doctor reported managed. No recovery or setup
replay was required.

The first revised candidate was
`sha256:2f01d4a3657603aa32e831fc51f1d5538317421b5b2a1d2009453a250c3c6fe2`,
with core SHA-256
`e5b55e46fe5d313bfb92dbc8a9a89cf3275c2cc867aecc174fe2a27b7d4af1da`.
The repaired candidate, retained separately without replacing either earlier
source, is
`sha256:424992211e95bf1d5d02ce6c5a02510e9a63da802887c7850f53bbf7b58658f7`,
with core SHA-256
`ac848c7e3064c5db31d5f8230d510e4cca72af2d655eeb17d3c1f97aba5e24b8`.

Both revised candidates contain 6,212 manifest file entries and the same
`design` and `skills` components. Only the core, skill catalog, provenance record
and three skill bodies differ from the frozen baseline payload. Exact assembly
produced 69 files and 422,059 bytes. Independent comparison confirmed exactly
three changed skill bodies and 64 unchanged bodies. The following are file-body
hashes, distinct from the native package receipt digests.

| File | Prior SHA-256 | Revised installed SHA-256 |
| --- | --- | --- |
| `skills/catalog.json` | `366b6fea56a81ed4fe940c4c60f4b87b4d2938a5d4e1f5254e443218fbe313ad` | `b354ddb4d3579ed5fc1b03b81f6891ae4fcf776ffb96c94bab7d8571ae52e6f7` |
| `skills/runtime-operations/SKILL.md` | `b5fea3912bda85ae83b836508d754d2894a89c1af027e7987ed76c89f4beaf01` | `d14dc9ca35755c7f62258546698b2774134fb2882c3a4631cfe49dcedbe90319` |
| `skills/vector-art/SKILL.md` | `d4143b4125a4ba01f3750d00bb7d75ef0b3154ba93486c0fd7e95e92f95a8ef5` | `5cc2a56b26e27f276e5c255e75f8d33730263da0aedd72127cceea13a6d070ca` |
| `skills/openpencil/SKILL.md` | `3c3c6ca616a8dedf55e2ad9b9634cb3314f8dcb4fd116b89a035090c383c45f7` | `76e8208ab8d568fe2087b0a08ce89c98a3759ddc27aeb113861bb745b5f1a9df` |

## Initial refusals and repaired update

The first revised-core update encountered prior-release verification before
reaching the deliberately edited project skill. The evaluator's expected drift
assertion therefore stopped; the actual refusal is retained. The old installed
launcher still reported that same core and global integration healthy, while
project doctor identified the exact injected skill edit as `drifted`.

A same-release `update --project-root` refused that edited project copy with
`managed skill copy has drifted; refusing replacement or removal`. The injected
bytes, active release, launcher, selection, native files and receipts remained
unchanged. The evaluator removed only its exact known suffix and verified the
recorded native snapshot against the original baseline.

With the project restored, both initial cross-catalog update directions failed:

| Invocation | Exit | Observed refusal |
| --- | --- | --- |
| Prior installed stable launcher, first revised `--source` | 1 | `release exceeds its compiled inventory capacity` |
| First revised core, prior `--home`, first revised `--source` | 1 | `Codex installation is not healthy; reconcile before update: saved release bytes do not match the manifest` (reported twice) |

Neither refusal staged or activated the revised release. The owning source
repair separates strict current-source admission from verification of retained
installed skill revisions. Retained catalog, provenance and bodies bind to their
own manifest and catalog; rollback staging is restricted to the active pointer's
recorded previous release. Source admission remains compiled and pinned. This
addresses the observed skill-corpus change; it does not qualify arbitrary future
dependency-inventory changes or retrofit the already built prior launcher.

The repaired core first reported the retained prior installation healthy. Its
explicit `update --home … --source repaired-source --user-home … --project-root …`
then completed in 150.953 seconds with the repaired candidate active and the
exact prior release recorded as previous. Global skills, guidance, registration
and the saved project selection synchronized through their normal owners. Full
and project doctors passed; `skills project-sync` returned `unchanged`.

The evaluator subsequently used an incorrect read-only activation-record path.
It reconciled the saved successful update results with the actual
`home/active.json`, installed bytes and receipts, then continued checks without
replaying update. No old-source admission probe or reverse activation ran.

A second exact suffix was injected into the revised native vector skill. The
repaired stable launcher's same-release update refused it before staging with
`managed skill copy has drifted; refusing replacement or removal`, preserving
the injected bytes and complete recorded native snapshot. Only that exact test
suffix was removed. Final doctors again reported healthy and managed. These are
preflight preservation checks; automatic failure rollback was not entered.

The revised `openpencil` body is verified in installed release content. Project
planning reports missing `design.openpencil`; an explicit unavailable selection
update refuses before replacing the pinned vector selection or its receipt.
No native OpenPencil project copy or editor behavior is claimed for this fixture.

## One read-only actual Codex turn

The client was macOS ARM64 `codex-cli 0.163.0-alpha.11.2`, at
`/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex`,
SHA-256 `248ff9873b6fb35124a86bdffa013a09e1bd17f106385124e3dda73e9eb157ad`.
No model, provider or reasoning override was supplied. The immutable backend
model identity was not observable in the saved JSONL.

One `exec` turn used strict configuration, ignored user configuration and rules,
was ephemeral, skipped the Git requirement and selected the host's read-only
sandbox. It used a fresh fake `HOME`; existing real `CODEX_HOME` was consumed
only by the client for its existing authentication route. The evaluator did not
read, copy or print credentials. One-run MCP overrides selected this fixture's
stable launcher with `serve --home …`, `required=true`, a 60-second startup
timeout and only `runtime_status` enabled. No design mutation tool was exposed.

The turn exited zero in 11.995 seconds, below its 120-second deadline. The trace
contains the complete installed revised vector body, a successful local SHA-256
command with the expected body hash, and exactly one successful MCP call:
`runtime_status`. Its ordinary and structured results agree on the repaired
release ID and `design.configured=true`; OpenPencil remains unconfigured.
Independent before/after project hashes are identical. No design artifact,
render, optimization, external-reference probe or image inspection was requested
or performed by this turn.

The 134-byte stderr contains one rollout state-database discrepancy warning.
This qualification does not establish network silence, Desktop GUI activation,
human acceptance or useful skill-to-design production. The earlier observed
host approval refusal and missing model-produced design artifacts remain open;
the 335 catalog development cases remain planned.

## Preservation, evidence and limits

Passive guards before preparation, after mutation commands, around the model
and at final verification preserve the real `~/.codex/config.toml` SHA-256
`7cf4d5f11d2a6593a35df91b3fa347638bc614ab3c165c231f1e51e613898317`
and mode `0600`. Selected existing user-installation files also retain recorded
hashes and modes: `setup.json`, `tools.json`, `update-activation.json`, `state.db`
and `bin/mirket`. This is a guard of those named files, not a full 8.4 GiB user
installation audit. The fake config's unrelated exact comment is preserved.
All installation mutation arguments name task-owned homes and project paths.

The final installed release and project remain healthy, the prior release is
retained, no staging or activation journal is pending, and no task-owned server
process remains. The exact installed `ac848c7e…` core predates the coordinator's
later recovery-journal candidate-identity binding repair. This journey did not
exercise recovery; final source tests do not requalify a different installed
binary. Windows and general future runtime-admission changes remain separate
gates. Observed durations are individual runs, not performance benchmarks.

Private evidence includes `task-state.json`, source pins, assembly and manifest
commands, all setup/update command records, baseline snapshots,
`update-blocker.json`, `repaired-candidate.json`, `update-summary.json`,
`repaired-drift-probe.json`, model request/JSONL/summary, final doctor commands
and `independent-checks.json`. The evaluator changed only this new document and
its private fixture. The owning coordinator's verifier repair is separate;
original fixtures, old harness, user settings, commits, pushes and publication
were not changed by this evaluator.
