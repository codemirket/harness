# Private hybrid memory through a Codex model turn

On 2026-10-26 (Europe/Istanbul), one actual Codex Desktop-bundled CLI model
turn used the installed private hybrid-memory tools to answer an English
question from a Turkish fact, exact-read its provenance, and abstain on an
unsupported question despite returned semantic neighbors. The supported answer
was **nine hours**. The expired conflicting **27-hour** record was excluded
from both searches. This establishes this bounded model-use case, not Desktop
GUI activation or general memory quality.

## Exact installation and client

The platform was **macOS 26.5.2 ARM64**. The client was `codex-cli
0.163.0-alpha.11.2`, at
`/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex`,
SHA-256 `248ff9873b6fb35124a86bdffa013a09e1bd17f106385124e3dda73e9eb157ad`.
No model or reasoning override was supplied. The saved JSONL does not expose an
immutable backend model identity; the evaluator's identity is not attributed to
the model under test.

A new task-owned disposable home was installed from previously verified private
payload bytes, without a build or dependency change. Its semantic-only release
exactly reproduced the release used by the
[installed two-client semantic check](m2-semantic-qualification.md):
`sha256:83cd280748a6ab307a69b93e218d8b4577cd707905cb78a60c73dfa90c1c3f6a`.
The current core, Node and text-adapter bytes came from the retained
TypeScript qualification installation; the pinned semantic package came from a
retained disposable installation. Its full receipt inventory was independently
checked before installation and again afterward. The historical installation's
older core was not used: its CLI refused the current `dev doctor-core` command.

| Selected installed input | SHA-256 |
| --- | --- |
| `bin/mirket` | `75e557a48ee52559b9811bd6796e1133d5821ed6f0a119e000bc1b6a7f3d4239` |
| `adapters/main.js` | `3ea3e1a548b5be6b70ece6487c7ce34f05ead1b8ecf1e824eabf8a4637f8be9e` |
| `runtime/node/bin/node` | `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329` |
| ARM64 dynamic-int8 MiniLM graph | `783fea82d71a58179b830a4dbd2d58447e640609e98eedf9ffa12622d375a672` |
| Private ONNX Runtime 1.31.0 library | `2a50a63e7f41f8d4d551d3ee47248df3864f6540109fce8ad520ca3b51af4b70` |

The model used the normal installed `bin/mirket serve --home …` launcher.
`runtime_status` reported the exact release and `embedding_configured:true`;
pre- and post-run `dev doctor-core --json` reported a healthy semantic-only
installation. The embedding model was
`sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2` at revision
`e8f8c211226b894fcb81acc59f3b34ba3efd5f42`, with the existing private CPU runtime.
No development server or runtime fallback served the model turn.

## Frozen facts and observed use

The fresh non-Git project path contains spaces and `İ`. The evaluator seeded
three synthetic facts through direct MCP preparation/commit before model
dispatch; every commit reported `semantic_index.status:"indexed"`. A new
connection then read the existing records and checked the frozen retrieval
oracle without replaying a save.

| Record | Synthetic content | Eligibility at `2000000000` |
| --- | --- | --- |
| Current telescope fact | Turkish: the observatory telescope's portable power pack operates continuously for nine hours | Eligible |
| Expired conflicting fact | Turkish: the same pack operates for 27 hours | Excluded; `valid_until:1700000000` |
| Unrelated fact | Turkish: the kitchen refrigerator's backup power pack operates for six hours | Eligible semantic distractor |

All records retained explicit synthetic provenance. The saved fixture source
revision was
`sha256:1773bc82c2c454d18536fcb85121719a4c080e07e858496c2baaa005494510c8`.
The task prompt supplied the scope, release, two English questions and fixed
search settings; it did not supply any fact contents, record IDs or durations.
It explicitly instructed the model to exact-read support and abstain when a
nearest neighbor did not support the question. This is a public developmental
exercise, not a blind or held-out abstention evaluation.

| Verbatim model query | Observed retrieval and answer |
| --- | --- |
| For the observatory telescope, how long does its portable battery operate? | Current telescope fact ranked first; model exact-read it and answered nine hours |
| What maximum wind speed triggers the observatory dome closure? | Both eligible battery-duration facts returned; model answered unknown from this memory |

Both model searches matched the direct preflight results exactly. They used
`status:"current"`, `at_time:2000000000`, `limit:5` and `max_bytes:16384`.
Both reported zero lexical/literal matches, semantic retrieval available, two
eligible indexed heads, zero missing eligible heads, no truncation and no
critical overflow. The expired record was absent. Returned noncritical hits had
`reason:"hybrid"`, which identifies the active ranking mode; it does not assert
that each hit independently matched both lanes.

The supporting exact read was record `mem-18e1e7b22ae867c8-9999-0`, revision
**1**, source ID `fixture:current`. The final answer cited these exact values,
preserved the fixed applicable time and coverage limits, and did not substitute
the expired or unrelated duration. Independent checks compared every request
and returned record with the frozen oracle, required structured/text result
equality, and checked the final factual answer against the Turkish source.

One planned model trial completed with exit **0** in **21.380 seconds**, within
its 120-second deadline. It made exactly **four successful MCP calls**: one
`runtime_status`, two `memory_search` calls and one `memory_read`. The search
calls overlapped. There were no shell, file, mutation or other tool items, and
no model retry. Canonical SQLite, identity, lifecycle and operation-receipt
file hashes/modes, and the complete project file set, were unchanged.

## Configuration preservation and teardown limit

The invocation used `exec --strict-config --ignore-user-config --ignore-rules
--ephemeral --skip-git-repo-check --sandbox read-only --color never --json`,
`project_doc_max_bytes=0` and `approval_policy="never"`. Command-line MCP
overrides selected only the task-owned installed launcher and the enabled tools
`runtime_status`, `memory_search`, `memory_read`, with a 60-second startup
timeout and `required=true`. No catalog-grace override was supplied. The exact
argv, prompt and environment overrides are retained in `request.json`; the
runner passed an argv array without shell interpolation. These one-run
configuration overrides follow the fetched official
[Codex advanced configuration documentation](https://learn.chatgpt.com/docs/config-file/config-advanced#one-off-overrides-from-the-cli).

The process used a task-owned `HOME`; Codex itself consumed the existing real
authentication `CODEX_HOME`. The evaluator did not read, copy or print credential
bytes. Passive guards before and after the model process and during independent
verification found real `~/.codex/config.toml` unchanged: SHA-256
`7cf4d5f11d2a6593a35df91b3fa347638bc614ab3c165c231f1e51e613898317`,
mode **0600**. No restoration was needed. These observations do not enforce
future client behavior or prove isolation of inherited guidance and other real
client state.

Post-run process inspection found no owned Mirket, Node or model process. One
empty embedding-session directory remained at
`~/.codex/task-state/harness-rewrite-implementation/memory-codex-model-20261026/mirket-home/state/jobs/op-embed-18e1e7cc63fd18a0-9ae0-1`.
It is owned by UID **501**, GID **20**, mode **0700**, and was created at
**2026-10-25T22:56:31.708616Z**, during the model turn. Its encoded server PID
is **39648**, distinct from the Codex PID **39627** and earlier seed server.
Attribution to the model's warm embedding session is an inference from its
name, creation time and implementation; exact attribution to either overlapping
query is unproven.

The inspected implementation's normal awaited shutdown removes empty session
scratch only after confirmed process cleanup. The model trace exposes neither
the server's shutdown report nor exit status, so abrupt client teardown versus
a missed cleanup path remains unresolved. The directory and its ownership
metadata are preserved for investigation. This run establishes process absence
at inspection, not complete scratch cleanup.

A subsequent [controlled cleanup audit](m2-embedding-scratch-cleanup-audit.md)
reproduced removal after graceful MCP EOF and empty-directory retention after
SIGTERM to the server. It found no missed graceful cleanup on this platform;
the original Codex server's termination mechanism remains unobserved.

## Evidence and limits

Private evidence is retained outside shipped source at
`~/.codex/task-state/harness-rewrite-implementation/memory-codex-model-20261026/`:
preparation and seed scripts, exact release/payload checks, `seed.json`, tool
schemas, the synthetic source, `frozen-inputs.json`, `request.json`,
`process.json`, complete `events.jsonl`, `stderr.log`, `final.txt`,
`summary.json`, `independent-checks.json`, `post-doctor.json`,
`payload-and-cleanup.json`, `cleanup-residue.json`, source excerpts and
`evidence-hashes.json`. The trace SHA-256 is
`432fc721dbeb82a5cdae90624584d25cece4d8c0a043e87ff7ff7ce7e41012e7`.

Before model dispatch, a direct preflight assertion incorrectly expected the
public match label `semantic`; inspection established the documented
`hybrid` mode label. The existing seed was then inspected read-only and the
unchanged content/ranking/eligibility checks passed. The initial full preflight
trace was not retained; its assertion and the complete continuation trace are
recorded. The first post-run cleanup assertion also failed on the empty
directory described above. Both diagnostics remain explicit; neither caused
another model turn or a product-source change.

The client reported **78,648 input tokens**, including **63,104 cached input
tokens**, and **496 output tokens**. Monetary cost and immutable backend model
revision are unknown. Its 134-byte stderr contained one rollout state-database
discrepancy warning; completed tool calls were error-free. Network silence and
complete isolation of real client state were not measured.

This case adds actual model use to the separate
[canonical memory verification](m2-memory-verification.md) and
[frozen semantic quality qualification](m2-semantic-qualification.md). It does
not establish model-driven writes or approval, automatic memory capture,
arbitrary stale-source detection, broader multilingual or abstention reliability,
10k/100k-record behavior, combined-load performance, Windows, macOS 15, Desktop
GUI activation or human acceptance. No current user installation, real Codex
configuration, old harness, production dependency, commit, push or publication
was changed. Only this document was authored in the repository by this check.
