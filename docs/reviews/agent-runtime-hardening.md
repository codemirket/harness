# Agent runtime patterns and harness hardening

Reviewed 2026-10-08 against the personal harness at `5fa23c4`. Codex desktop
remains the primary runtime; Claude Code remains the supported secondary client.
The useful improvements are bounded source processing, reliable evidence and
clear activation boundaries. They do not require another agent runtime.

## Reviewed sources

These are immutable source snapshots. Root licenses were inspected; a root
license does not settle every dependency or bundled payload's terms. All new
guidance is original prose. No upstream implementation was vendored or executed.

| Source | Reviewed revision | Root license |
| --- | --- | --- |
| [DeerFlow](https://github.com/bytedance/deer-flow/tree/9472579ef0a4e406afa61ec6d6c79fbd79bf1d5d) | `9472579ef0a4e406afa61ec6d6c79fbd79bf1d5d` | [MIT](https://github.com/bytedance/deer-flow/blob/9472579ef0a4e406afa61ec6d6c79fbd79bf1d5d/LICENSE) |
| [OpenHuman](https://github.com/tinyhumansai/openhuman/tree/eef0350e53014b1750ac2ec09f4ae18b8009e1c0) | `eef0350e53014b1750ac2ec09f4ae18b8009e1c0` | [GPL-3.0 text](https://github.com/tinyhumansai/openhuman/blob/eef0350e53014b1750ac2ec09f4ae18b8009e1c0/LICENSE) |
| [DeepAgents](https://github.com/langchain-ai/deepagents/tree/6a3a12bc5b3ac88aeb091e6c77ba81ad4d6732af) | `6a3a12bc5b3ac88aeb091e6c77ba81ad4d6732af` | [MIT](https://github.com/langchain-ai/deepagents/blob/6a3a12bc5b3ac88aeb091e6c77ba81ad4d6732af/LICENSE) |
| [Hermes](https://github.com/NousResearch/hermes-agent/tree/3f97b91aae2bc9a3d533634d54039018e1d224ec) | `3f97b91aae2bc9a3d533634d54039018e1d224ec` | [MIT](https://github.com/NousResearch/hermes-agent/blob/3f97b91aae2bc9a3d533634d54039018e1d224ec/LICENSE) |
| [CodeGraph](https://github.com/colbymchenry/codegraph/tree/b635dd467f0578926a9c01a37b9d28d2b26689f1) | `b635dd467f0578926a9c01a37b9d28d2b26689f1` | [MIT](https://github.com/colbymchenry/codegraph/blob/b635dd467f0578926a9c01a37b9d28d2b26689f1/LICENSE) |

## Decisions supported by implementation

| Source evidence | Adopted principle | Rejected or deferred behavior |
| --- | --- | --- |
| DeerFlow [archive installer](https://github.com/bytedance/deer-flow/blob/9472579ef0a4e406afa61ec6d6c79fbd79bf1d5d/backend/packages/harness/deerflow/skills/installer.py#L115-L177) and [overflow tests](https://github.com/bytedance/deer-flow/blob/9472579ef0a4e406afa61ec6d6c79fbd79bf1d5d/backend/tests/test_skills_installer.py#L167-L239) | Count decoded bytes and members independently of selected skill size. Reject invalid input before registration writes. | Do not copy ZIP checks as proof that tar metadata or decoder allocations are bounded. Our archive format needs its own regression cases. |
| DeerFlow [report contract](https://github.com/bytedance/deer-flow/blob/9472579ef0a4e406afa61ec6d6c79fbd79bf1d5d/backend/packages/harness/deerflow/subagents/report_contract.py#L35-L112), [acceptance checks](https://github.com/bytedance/deer-flow/blob/9472579ef0a4e406afa61ec6d6c79fbd79bf1d5d/backend/packages/harness/deerflow/subagents/acceptance_checks.py#L1-L54), and [status contract](https://github.com/bytedance/deer-flow/blob/9472579ef0a4e406afa61ec6d6c79fbd79bf1d5d/backend/packages/harness/deerflow/subagents/status_contract.py#L1-L104) | Require actual artifacts, observed checks and explicit incomplete outcomes from workers. Reconcile late results against current requirements. | A finished process or `completed` label may still contain a capped partial result. Do not add a second receipt service or claim Markdown enforces host budgets. |
| DeepAgents [blob offload](https://github.com/langchain-ai/deepagents/blob/6a3a12bc5b3ac88aeb091e6c77ba81ad4d6732af/libs/deepagents/deepagents/middleware/_blob_offload.py), [summarization failure path](https://github.com/langchain-ai/deepagents/blob/6a3a12bc5b3ac88aeb091e6c77ba81ad4d6732af/libs/deepagents/deepagents/middleware/summarization.py#L1560-L1603), and [skill trust](https://github.com/langchain-ai/deepagents/blob/6a3a12bc5b3ac88aeb091e6c77ba81ad4d6732af/libs/code/deepagents_code/skills/trust.py) | Save recoverable evidence with provenance; verify the saved location before replacing output with a pointer. Keep changed-input trust checks. | Failed offload can leave history unrecoverable. Do not invent missing context, add a blob store, or adopt implicit permanent-memory updates. |
| OpenHuman [preflight](https://github.com/tinyhumansai/openhuman/blob/eef0350e53014b1750ac2ec09f4ae18b8009e1c0/crates/openhuman-core/src/skills/preflight.rs#L51), [fetching](https://github.com/tinyhumansai/openhuman/blob/eef0350e53014b1750ac2ec09f4ae18b8009e1c0/crates/openhuman-core/src/skills/ops_install/fetch.rs#L183), and [memory provenance](https://github.com/tinyhumansai/openhuman/blob/eef0350e53014b1750ac2ec09f4ae18b8009e1c0/crates/openhuman-core/src/agent/harness/memory_context_safety.rs#L53) | Preserve missing/unavailable/not-checked states and the origin of summarized data. Investigate bounded download time separately from byte limits. | Do not require a Composio connection or equate Git author name with authenticated identity. Post-buffer size checks do not bound peak allocation; agent-authored summaries are not inherently trusted. |
| OpenHuman [approval origin](https://github.com/tinyhumansai/openhuman/blob/eef0350e53014b1750ac2ec09f4ae18b8009e1c0/crates/openhuman-core/src/security/approval/gate_intercept.rs#L226) and [decision handling](https://github.com/tinyhumansai/openhuman/blob/eef0350e53014b1750ac2ec09f4ae18b8009e1c0/crates/openhuman-core/src/security/approval/gate_intercept_decision.rs#L114) | Keep original authorization, affected resource and permitted effect explicit across delegation/resume. | Native host controls own approval enforcement. Do not add another approval engine or infer authorization from a missing UI or timeout. |
| Hermes [skill viewing](https://github.com/NousResearch/hermes-agent/blob/3f97b91aae2bc9a3d533634d54039018e1d224ec/tools/skills_tool.py#L550) and [inline-shell preprocessing](https://github.com/NousResearch/hermes-agent/blob/3f97b91aae2bc9a3d533634d54039018e1d224ec/agent/skill_preprocessing.py#L46) | Make inventory, reading, registration and runtime activation separate operations. Inspect executable metadata before import. | Viewing can ensure dependencies or capture declared credentials; optional shell preprocessing executes code. Keep our catalog passive rather than transplanting those effects. |
| Hermes [scan-cache identity](https://github.com/NousResearch/hermes-agent/blob/3f97b91aae2bc9a3d533634d54039018e1d224ec/tools/skills_guard.py#L693), [source-bound updates](https://github.com/NousResearch/hermes-agent/blob/3f97b91aae2bc9a3d533634d54039018e1d224ec/tools/skills_hub_install.py#L264), and [preserved edits](https://github.com/NousResearch/hermes-agent/blob/3f97b91aae2bc9a3d533634d54039018e1d224ec/tools/skills_sync.py#L331) | Retain content/source/checker identity for reused evidence, explicit source pins and protection of modified installations. | Existing harness hashes and staged replacement already cover these needs. No parallel trust database, regex safety verdict, or same-name source substitution. |
| CodeGraph [resolution metadata](https://github.com/colbymchenry/codegraph/blob/b635dd467f0578926a9c01a37b9d28d2b26689f1/src/resolution/types.ts#L36-L67), [heuristic edges](https://github.com/colbymchenry/codegraph/blob/b635dd467f0578926a9c01a37b9d28d2b26689f1/src/resolution/callback-synthesizer.ts#L1-L23), and [freshness validation](https://github.com/colbymchenry/codegraph/blob/b635dd467f0578926a9c01a37b9d28d2b26689f1/src/mcp/answer-freshness.ts) | Use a ready graph selectively for structural navigation; retain revision, dirty state, exclusions, omissions and edge provenance. Refresh stale/unknown results or inspect live source. | Reject blanket graph-first instructions, universal freshness claims and treating inferred callers as runtime proof. Tool discovery does not establish useful project coverage. |

## Changes at the existing owner

- **Catalog reader:** a streaming 512 MiB decoded budget covers selected and
  unselected content, metadata and padding; 100,000 physical headers and 32 metadata
  parser levels have independent limits. The existing 100 MiB compressed download,
  32 MiB selected payload and 4,096 selected-file caps remain. Invalid sizes,
  incomplete tar framing, nonzero trailing data and corrupt/truncated gzip fail
  before skill publication. Ordinary PAX/GNU names remain supported; sparse-file
  formats are intentionally rejected for GitHub source archives.
- **Skill catalog:** passive discovery and reading cannot implicitly install
  packages, execute inline shell, capture credentials or activate hooks.
- **Context management:** durable output pointers retain their producer, scope,
  revision and completeness. Summaries retain the source's authority level.
  Conditional [repository retrieval](../../skills/context-management/references/repository-retrieval.md)
  guidance covers graph freshness and coverage.
- **Agent coordination:** worker output must identify partial, skipped and
  unverified work; stale or cancelled results need reconciliation before use.
- **Security judgment:** conditional [agent tool boundaries](../../skills/security-judgment/references/agent-tool-boundaries.md)
  guidance applies existing trust rules to loaders, retrieved instructions and
  approval/resume paths.

No new runtime, default MCP, global skill, memory service, hook, dependency or
account connection is introduced. Existing project selections, registration
commands and host responsibilities remain authoritative. Remote downloads still
use a socket timeout rather than an overall elapsed-time deadline; a slowly
progressing response can outlive that timeout. Archive volume limits do not imply
a strict CPU, wall-clock or total-process-memory bound. The continuity trial below
provides bounded behavior evidence, not a universal authority guarantee.

## Optional CodeGraph pilot

Use only when recurring structural discovery justifies an index. The reviewed
package is 1.6.2; its [manifest](https://github.com/colbymchenry/codegraph/blob/b635dd467f0578926a9c01a37b9d28d2b26689f1/package.json)
declares Node >=20 and <25. Standalone distributions bundle their runtime and
native extraction components. Language coverage is broad but uneven; unfamiliar
extensions, exclusions, oversized files and runtime dispatch limit the graph.

1. Select the project and independently verify a pinned binary's source mapping,
   checksum/provenance and dependency notices before any execution. Runtime setup
   and indexing need applicable authorization. Preserve existing configuration.
   The upstream [installer](https://github.com/colbymchenry/codegraph/blob/b635dd467f0578926a9c01a37b9d28d2b26689f1/install.sh)
   defaults to latest and does not itself verify published attestations.
2. Merge only a scoped MCP entry using the stable absolute binary path and
   arguments `serve`, `--mcp`. Do not invoke the automatic installer: its
   [Codex](https://github.com/colbymchenry/codegraph/blob/b635dd467f0578926a9c01a37b9d28d2b26689f1/src/installer/targets/codex.ts)
   and [Claude](https://github.com/colbymchenry/codegraph/blob/b635dd467f0578926a9c01a37b9d28d2b26689f1/src/installer/targets/claude.ts)
   targets also write instructions, and Claude can receive wildcard permissions
   and a prompt hook. Keep existing instructions/permissions and hooks unchanged.
   Check the actual host's supported project configuration; upstream calls its
   target Codex CLI, which does not establish desktop success.
3. Pass `DO_NOT_TRACK=1` to each CLI/MCP process to disable telemetry and update
   checks at this pin. Separate controls are `CODEGRAPH_TELEMETRY=0` and
   `CODEGRAPH_NO_UPDATE_CHECK=1`. Both behaviors are implemented in
   [telemetry](https://github.com/colbymchenry/codegraph/blob/b635dd467f0578926a9c01a37b9d28d2b26689f1/src/telemetry/index.ts#L231-L251)
   and [update-check](https://github.com/colbymchenry/codegraph/blob/b635dd467f0578926a9c01a37b9d28d2b26689f1/src/upgrade/update-check.ts).
4. For a bounded pilot, `CODEGRAPH_MCP_TOOLS=explore,status` exposes
   `codegraph_explore` and `codegraph_status`; only explore is listed by default.
   [Tool definitions](https://github.com/colbymchenry/codegraph/blob/b635dd467f0578926a9c01a37b9d28d2b26689f1/src/mcp/tools.ts#L1490-L1852)
   also define search, callers, callees, impact, node and files queries. Read-only
   annotations do not eliminate index writes, watchers or shared-daemon state.
5. Start with synthetic code. Check known callers, duplicate names, an excluded
   file, edits/deletions before and after sync, and disconnected/restarted
   watching. Match results to current source; no result does not prove no caller.
   A message asking for indexing may have no `isError` while delivering no answer.
6. Confirm fresh-session discovery and a real query in each selected host.
   Compare representative tasks with existing `rg`/direct reads for correctness,
   time, output/context size and index maintenance cost. Retain the integration
   only if the measured benefit justifies it; record rollback and index cleanup.

This recipe is source-reviewed, not executed. It does not register a server or
authorize indexing unrelated projects. [Runtime integration guidance](../runtime-integrations.md)
continues to govern optional tools; local computation is not an OS sandbox.

## Inspected scope and remaining evidence

Read the requested repository pages, root licenses and relevant implementation
and regression-test sections at the pins above. DeerFlow coverage includes
archive limits, delegated reports/acceptance, output limits and durable context;
DeepAgents covers blob offload, summarization, skill trust, memory and filesystem
boundaries. OpenHuman covers preflight/fetch, memory provenance, approval origins
and context ownership. Hermes covers skill viewing/preprocessing, cache/update
identity, sync preservation, traversal, compaction and security guidance.
One Hermes sync-test download returned HTTP 429 and was not reviewed.

CodeGraph coverage includes both host adapters, instruction injection, query
definitions, resolution/freshness, corpus limits, telemetry/updater, path controls,
manifest, shell installer and release workflow. Large modules were inspected at
relevant paths, not exhaustively; its test filenames were inventoried, not fully
reviewed. No upstream runtime, installer, service, hook, test, model benchmark or
account integration was run. No source review establishes universal safety,
improved frontend taste or production readiness in another project.

## Verification

[Exact evidence](../evidence/agent-runtime-hardening-2026-10-08.json) records source
hashes, checks and limits. The final suite passes **441 tests in 35.406 seconds**,
including 15 archive regressions. Initial regressions failed before repair;
independent review found metadata-count, malformed-tail and numeric-size cases
that were also reproduced, fixed and rechecked. Two real pinned GitHub archives
retain their reviewed source and adapted hashes through the final reader.

At that pre-review baseline, a scratch foundation project installed 24 byte-exact
Codex/Claude copies from the then-12-skill profile; doctor
passes and repeat sync preserves all 84 installed files' contents, modes and
timestamps. Global sync/doctor report 30 unchanged live links. Four changed skill
entrypoints pass the official validator's unchanged rules using existing
Ruby/Psych parsing because PyYAML is unavailable. No dependency was installed.

A fresh agent used the installed guidance for a synthetic policy change with an
outdated handoff, cancelled worker report, incomplete graph, missing evidence log
and embedded unauthorized instructions. It implemented the current requirement,
preserved runtime dispatch, retained complete fresh test logs and accurately
reported the missing history. Five local tests and 25 independent behavior
assertions pass; the unauthorized approval marker is absent and input notes remain
unchanged. The primary agent inspected the actual source, report and logs.

This is a guided development trial, not a controlled comparison, held-out benchmark,
general prompt-injection defense or proof of improved frontend design. Host policy
and the trial's explicit scope also constrained the worker. No upstream runtime,
CodeGraph server, new Claude model task, native Windows/device check or production
operation was exercised. Existing projects receive revised managed skills only
after deliberate `project sync` and `project doctor`.
