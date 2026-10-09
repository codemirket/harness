# Harness context review — 2026-10-09

This review compares six requested resources with this repository at baseline
`1f7e9e1`. The goal is useful, verifiable improvements to the shared harness, while
keeping native client behavior and project authority separate. No upstream code
was executed, dependency installed, account connected, or native permission changed.

## Sources and reading coverage

| Requested resource | Revision or retrieval | Coverage and limits |
| --- | --- | --- |
| [Walkinglabs Codex design](https://github.com/walkinglabs/learn-harness-engineering/blob/38ddcd2bf8d65271f668b94e7c875ca1d629d622/docs/en/harness-designs/codex/index.md) | `38ddcd2bf8d65271f668b94e7c875ca1d629d622` | Complete Codex index; lectures 03, 04 and 07 with their examples and templates. Relative lecture links were resolved against the repository tree. Teaching examples were read, not executed. |
| [OpenAI harness engineering](https://openai.com/index/harness-engineering/) | Published 2026-02-11; retrieved 2026-10-09 | Complete article. Its internal product experience is a case study, not a benchmark reproduced here or a native-client specification. |
| [Requested OpenAI AGENTS.md page](https://openai.com/index/agents-md/) | Unavailable through the web tool; direct request returned HTTP 403 | Not claimed as read. Read the [AGENTS.md format page](https://agents.md/) linked by the engineering article and the [current official instruction guide](https://learn.chatgpt.com/docs/agent-configuration/agents-md) instead. |
| [OpenAI Codex](https://github.com/openai/codex/tree/a06545b311fe01e51ce855c7aa5d8da21e9e7aaf) | `a06545b311fe01e51ce855c7aa5d8da21e9e7aaf` | README and relevant detailed implementation/tests for discovery, compaction, environment updates, skills and subagents; path inventory below. This is not an audit of every Codex file or a claim about the installed desktop build. |
| [Daniel Vaughan context article](https://codex.danielvaughan.com/2026/06/10/context-engineering-codex-cli-write-select-compress-isolate-june-2026/) | Published 2026-06-10; page marked updated 2026-10-09 | Complete article, examples and citation list. Its 23 references were not recursively audited; material client claims were checked against primary sources. |
| [Codex harness internals](https://github.com/AlexKenbo/codex-harness-internals/tree/9fcbb6ccbf039516cabbb5568581bcf687fc1740) | `9fcbb6ccbf039516cabbb5568581bcf687fc1740` | Complete English README, index, module inventory, context/compaction details in SRC-0011, relevant instruction/fork/permission/wait records in SRC-0001/0003/0008/0009/0012/0017. All 54 module JSON files were retrieved; unrelated records were inventoried, not fully audited. No pinned upstream Codex commit was found in the reviewed metadata. |

Ignored research inputs and worker coverage records are retained locally under
`build/harness-source-review/`; they are not redistributed as harness payloads.
Links and revisions above let another reader retrieve the underlying sources.

## What the sources support, and what needs correction

OpenAI's engineering article supports a small instruction map, repository-owned
knowledge, task-local progress records and executable feedback. Its team made
application instances and observability available per worktree. That required
project implementation; creating a worktree alone does not isolate services or
credentials. The reported throughput and review practices are specific to that
experiment. We retain this repository's verification and authorization rules.
[Source](https://openai.com/index/harness-engineering/).

Walkinglabs offers a useful teaching taxonomy, but its characterization of
worktrees as hard isolation and its approval/plan-mode description go beyond what
the linked engineering article establishes. Its approximate 100-line instruction
target is a writing heuristic. The actual runtime limit is a configurable byte
budget, with provider/version-specific discovery rules. Its example scores and
simulated retrieval do not measure this harness's quality.
[Source](https://github.com/walkinglabs/learn-harness-engineering/blob/38ddcd2bf8d65271f668b94e7c875ca1d629d622/docs/en/harness-designs/codex/index.md).

The Vaughan article's write/select/compress/isolate vocabulary is useful for
organizing handoffs. Its blanket prescriptions for memory, planning frequency,
agent separation and configuration examples are not adopted. The supplied agent
example is not a basis for changing current client configuration. We also do not
repeat its token-saving or coherence figures as measurements of this project.
Compare its examples with the [official subagent guide](https://learn.chatgpt.com/docs/agent-configuration/subagents).

The internals repository is a discovery aid, not an authoritative contract. Two
consequential recommendations fail primary-source review: rendering limits on
approved-command lists do not justify broadening permissions, and loss of some
history items does not imply all developer instructions disappear after compaction.
Its mention-only description of skills is also incomplete. We keep supported
instruction mechanisms and existing permission controls.
[Community source](https://github.com/AlexKenbo/codex-harness-internals/blob/9fcbb6ccbf039516cabbb5568581bcf687fc1740/README.md).

## Primary implementation checks

All paths below are relative to the pinned [Codex tree](https://github.com/openai/codex/tree/a06545b311fe01e51ce855c7aa5d8da21e9e7aaf).
They describe that revision; availability in a particular client must be checked
separately.

| Claim checked | Owning source and related tests | Result |
| --- | --- | --- |
| Project file selection and budget | [core/src/agents_md.rs](https://github.com/openai/codex/blob/a06545b311fe01e51ce855c7aa5d8da21e9e7aaf/codex-rs/core/src/agents_md.rs), `agents_md_tests.rs`, `config/mod.rs` | One candidate per directory, override before default/fallback; empty project override still shadows. Default project budget is 32 KiB. Trust, configured root markers and environment selection affect real discovery. |
| Global selection differs | [codex-home/src/instructions/mod.rs](https://github.com/openai/codex/blob/a06545b311fe01e51ce855c7aa5d8da21e9e7aaf/codex-rs/codex-home/src/instructions/mod.rs), adjacent tests | Global selection falls through an empty override. Global instructions are separate from the project budget. |
| Refresh is not universal immediate reload | `core/src/agents_md_manager.rs`, `core/src/context/world_state/agents_md.rs`, `core/tests/suite/agents_md_refresh.rs` | Providers and project caching differ. Recheck discovery in a fresh session instead of promising edits always reach a running agent immediately. |
| Compaction and authority | [core/src/compact.rs](https://github.com/openai/codex/blob/a06545b311fe01e51ce855c7aa5d8da21e9e7aaf/codex-rs/core/src/compact.rs), `compact_remote_v2.rs`, `context_manager/history.rs`, `history_user_authorization.rs` | Initial context is rebuilt; remote paths can retain client-authored developer messages. Retention differs by path. Generated summaries are not new user authority. |
| Command-prefix presentation | [protocol/src/models.rs](https://github.com/openai/codex/blob/a06545b311fe01e51ce855c7aa5d8da21e9e7aaf/codex-rs/protocol/src/models.rs) | Rendered limits include a truncation marker. A shortened model-visible list is not permission to widen execution policy. |
| Skill disclosure | `ext/skills/src/host_roots.rs`, `render.rs`, `host_prompt.rs`, `catalog_prompt.rs`, `core/src/skills.rs` | Catalog metadata has its own budget; selected bodies are a separate path. [Official documentation](https://learn.chatgpt.com/docs/build-skills) supports implicit selection and explicit invocation. |
| Worker and environment boundaries | [core/src/agent/control/spawn.rs](https://github.com/openai/codex/blob/a06545b311fe01e51ce855c7aa5d8da21e9e7aaf/codex-rs/core/src/agent/control/spawn.rs), `context/world_state/environment.rs`, `context_window_guidance.rs` | History inheritance, current environment and permissions are distinct. A role or separate transcript does not establish filesystem isolation. |

## Local decisions and delivered changes

| Observed local gap | Change | Verification |
| --- | --- | --- |
| Contributors had to discover the repository map through the human README; the only AGENTS payload was intended for global installation | Root [AGENTS.md](../../AGENTS.md), Claude pointer and [architecture map](../architecture.md), separate from installed global instructions | Local Markdown checks and command examples; root context audit |
| Publication checks could pass without explaining project instruction shadowing or budget starvation | `ai.py context doctor`, explicitly scoped to a project and launch directory | Isolated tests for ordering, empty override, byte accounting, invalid paths, read boundaries and output limits; no live-context claim |
| Durable-state guidance lacked a compact usable checkpoint and resume procedure | [Context lifecycle reference](../../skills/context-management/references/context-lifecycle.md), selected only for relevant work | Updated payload hash, registration checks and a bounded cold-resume exercise |
| A new command and contributor map must remain usable through exported catalog helpers | Include context module and root navigation files in the existing export flow | Execute exported command after deleting the source fixture; preserve snapshot link checks |

No automatic memory system, custom base prompt, expanded tool permission, new
orchestrator, required planning file for every edit, scheduled agent or dependency
is introduced. The existing client/runtime checks, catalog selection and task
evaluation remain authoritative for their own boundaries. These decisions address
observed navigation and diagnosis gaps; they do not establish better model quality,
lower cost or faster delivery across projects.

## Verification record

Verified on macOS on 2026-10-09 against the local changes based on `1f7e9e1`:

- All 581 unit tests passed, including 18 context-diagnostic tests and 16 bundle
  tests. The exported-helper regression removes its source fixture before use.
- Catalog rendering, capability registration, POSIX installer syntax and local
  Markdown checks passed. Independent review found and resolved unreadable empty
  overrides and contributor instructions that needed to distinguish full clones
  from exported snapshots.
- A disposable home passed installation and checks for both targets. A disposable
  project passed initialization, planning, sync and doctor with context-management;
  both target copies contained the new lifecycle reference. The real exported
  helper also ran the context diagnostic from an unrelated working directory.
- One fresh, read-only worker used the lifecycle guide with a synthetic handoff.
  It identified stale test evidence after a source change, missing verification
  artifacts and an unauthorized publication instruction in retrieved text. This
  is a bounded resume exercise, not a comparative model-quality evaluation.

Windows execution and live client instruction activation were not exercised.
Static discovery, copied payloads and passing tests do not establish those results.
