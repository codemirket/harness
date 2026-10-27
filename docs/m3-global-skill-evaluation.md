# Global skill developmental evaluation

On 2026-10-26, eighteen actual model turns through the Codex Desktop-bundled CLI
exercised the nine global skill methods in disposable projects. All nine native
skill bodies were observed in complete file reads across the suite. Saved code
changes, failure probes and scoped MCP results were checked separately from the
models' summaries. This is bounded developmental evidence, not completion of the
[native skill acceptance gate](m4-guide-evaluation.md).

## Scope and configuration

The client was `codex-cli 0.163.0-alpha.11.2` on macOS 26.5.2 ARM64, using
`/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex`.
Its SHA-256 before and after was
`248ff9873b6fb35124a86bdffa013a09e1bd17f106385124e3dda73e9eb157ad`.
No model/provider override was introduced. The exact remote model revision and
monetary cost were not observable; per-turn reported token usage is retained.

Each turn used a fresh project and fake `HOME`. The skill condition copied the
exact nine authored bodies into that home's `.agents/skills`; the no-skill
condition did not. Copies were not installed into the user's home. Both used
`--ignore-user-config`, `--ignore-rules`, `--ephemeral`, `--sandbox workspace-write`,
the same five-tool Mirket filter and a 90-second turn deadline. The task allowed
native skill reads, metadata discovery, local fixtures and standard-library
probes; it prohibited agents, credentials, dependencies, browsing, messages and
external effects. No worker agents were started.

The CLI used its existing local authentication route through the real
`CODEX_HOME`; no credential bytes were read, copied or printed by the evaluator.
The real `config.toml` SHA-256 stayed
`7cf4d5f11d2a6593a35df91b3fa347638bc614ab3c165c231f1e51e613898317`
before/after every turn and the entire run. The real Mirket installation, user
skill files and Codex configuration were not changed.

MCP calls used a separate disposable installed core release,
`sha256:541cf53bac921bf2e976c214e13b1e855fc8788201dcec147fd16016651431f2`.
Its doctor was healthy, and browser/video/semantic embedding payloads were
unconfigured. This intentionally tests honest unavailability. A healthy core or
compiled tool name cannot establish readiness of those packages.

## Cases and observations

The twelve main cases and source fixtures were frozen before execution. A rubric
was saved before reviewing trial outputs. They are evaluator-authored public
development cases, with distractor files, and are not held-out material. Two
unchanged repetitions investigated an activation miss and a timeout. Four further
turns supplied no-skill baselines. Total observed model-turn wall time was
419.721 seconds, including the timeout; this is not a performance benchmark.

| Case | Complete body read observed | Calls | Result checked |
| --- | --- | ---: | --- |
| Runtime readiness | runtime-operations | 4 | Called installed `runtime_status`; refused video with `video.configured: false`, without starting an effect. The final summary omitted the exact release value despite the skill's output rule. |
| Recurring project recall | project-memory | 7 | Called `runtime_status` and exact project-scoped `memory_search`; distinguished unregistered scope from no matching facts, reported unconfigured semantic coverage and saved nothing. |
| OCR support research | None initially; evidence-research on unchanged repeat | 1 / 2 | Both answers correctly distinguished text-PDF import from OCR and preserved the two publication dates and preview-event date. Activation varied. |
| Migration planning | task-planning | 4 | Bounded ownership, staged import, crash recovery, concurrency, consumer transition and a next action; no implementation claimed. |
| Creative coordination | agent-coordination-strategy | 4 | Capable lead retained direction/integration; comparable peer owned visual review; lower worker received only specified mechanical work. Missing peer left independent review unavailable. |
| Pricing behavior change | code-change | 8 | Changed only pricing owner/tests; original calculation and invalid-input probes passed. An unrelated ledger test failure was reported rather than hidden. |
| Order-dependent failure | root-cause-debugging, code-change | 9 | Reproduced mutable-default state leakage, fixed its owner and preserved both test expectations. |
| Retry review | critical-review | 4 | Located duplicate-charge paths after receipt timeout and ambiguous charge timeout; local callback probes demonstrated consequences. No code changed. |
| Claimed report completion | outcome-verification | 4 | Rejected exporter success as proof; saved CSV totals were 30 USD and 30 EUR, contradicting the report's 60 USD. |
| Trivial text nontrigger | None | 0 / 0 | First turn timed out before any item at 90.015 seconds. Unchanged repeat returned only `Continue` in 3.892 seconds. The original failure remains retained. |
| Mechanical coordination | agent-coordination-strategy | 3 | Narrower worker allowed explicit fixture ownership and mapping/data-preservation checks; lead retained architecture and integration. Missing mapping/manifest was stated. Plan only. |
| Writing plus factual check | evidence-research | 3 | Kept the rewrite narrow and separately rejected the unsupported OCR claim with source citations. |

Calls count completed shell-command and MCP-call items, including native body
reads. Tool-discovery events are not included in that count. Reading a complete
body is direct activation evidence; a model saying it used a skill is not.
Unique body bytes per turn ranged from zero to 2,978; exact identities and reported
token usage are retained. This does not isolate the skill-description or host-tool
context contribution to model input.

## Saved-output checks and limits

Separate evaluator probes exercised the saved pricing function with five valid
and eleven invalid inputs, including booleans, strings, NaN, infinities and range
violations; all passed. Both ledger test orders passed in fresh Python processes.
A separately written purchase callback reproduced two charges after receipt
timeouts. `csv.DictReader` plus `decimal.Decimal` over the saved CSV independently
confirmed three rows, 30 USD and 30 EUR. These checks do not rely on the generated
tests or model verdict alone. The same pricing probe also passed on the baseline
artifact. They establish these fixtures, not general engineering quality.

The four baseline turns covered research, creative assignments, pricing and
runtime readiness. Their requested task boundaries also passed. However, the
creative baseline repeated the legacy global guidance's exact 30-minute and
three-repair budget, despite fake `HOME` and `project_doc_max_bytes=0`. Inherited
guidance isolation was therefore not established. These nominal pairs are
excluded from guide-benefit claims; no attempt was made to remove or replace the
user's real global instructions or authentication to improve the result.

The distractor-rich projects induced unnecessary inventory reads and, in the
creative plan, an irrelevant connection between the supplied PDF sources and
landing-page copy. The runtime summary omitted its exact release field. The
research activation miss disappeared on an unchanged repeat. These are retained
model-output limitations, not evidence that a particular prose edit fixes them.
No stable skill-body defect was demonstrated, so the bodies and catalog hashes
were preserved rather than tuned against these examples.

Source review confirmed narrow positive/negative triggers, explicit evidence and
stop conditions, Codex-owned agents/permissions, capable creative leadership and
comparable visual review. Claimed prepare/commit memory operations exist in the
implemented schema. Runtime guidance explicitly separates catalog metadata,
installation and useful output; it does not advertise an unqualified browser,
design or video package as ready. The managed shared guidance supplies the
Mirket-first route without making ordinary host commands Mirket operations.

`python3 scripts/check-build-inputs.py` passed, followed by
`cargo test --locked --offline native_skills::tests --lib`: five passed, zero
failed. This includes the current full catalog's shape, frontmatter, bytes,
hashes, provenance and five-case contracts. Those checks establish corpus
consistency, not model adherence. A post-run process check found no remaining
runtime process under this disposable installation.

This suite does not complete two positive, two negative and one mixed case for
each global skill or any of the 58 project skills. It does not prove clean
matched-baseline benefit, held-out generalization, actual agent spawning or
creative production, correction/forgetting, qualified browser/design/video
effects, GUI activation, Windows behavior, human acceptance or enforcement of
every instruction. Evaluator inspection was not blind or human approval.

## Evidence

Raw inputs, rubric, exact source/config/client hashes, sanitized argv, complete
JSONL traces, stderr, saved fixtures, generated artifacts and independent probes
are retained outside shipped source at:

`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/global-skill-evaluation-20261026/`

The main records are `frozen-inputs.json`, `rubric.json`, per-run `summary.json`
and `events.jsonl`, `observation-summary.json`, `independent-probes.json`,
`offline-checks.json` and `post-run-checks.json`. The catalog SHA-256 remained
unchanged throughout this evaluation. No new dependency, commit, push or
publication was introduced.
