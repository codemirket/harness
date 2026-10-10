# Harness and runtime source review — 2026-10-10

Fresh public snapshots of twelve primary resources; ECC is covered in [workflow review](workflows.md). Review was by repository inventory, relevant documentation, implementation paths and licenses. Listed paths include focused sections of large files, not an assertion that every line or transitive dependency was audited. Retrieval used inert Git blobs or exact-commit raw files; no upstream installer, hook, SDK, service or model was executed. Three large Git transfers timed out and were replaced with complete GitHub tree inventories plus pinned raw-file reads. Temporary source copies are removed after integration.

The reusable architecture is a small host-independent contract layer: select expertise, establish real tools, preserve task state, deliver a result and verify the consumer. Full SDK/runtime projects remain references or explicit future integrations. Original guidance is written independently; it does not vendor their implementation.

## learn-harness-engineering

**Decision: adapt.** course and scaffolder. Snapshot [38ddcd2bf8d6](https://github.com/walkinglabs/learn-harness-engineering/tree/38ddcd2bf8d65271f668b94e7c875ca1d629d622).

Inspected paths: [README.md](https://github.com/walkinglabs/learn-harness-engineering/blob/38ddcd2bf8d65271f668b94e7c875ca1d629d622/README.md), [LICENSE](https://github.com/walkinglabs/learn-harness-engineering/blob/38ddcd2bf8d65271f668b94e7c875ca1d629d622/LICENSE), [skills/harness-creator/SKILL.md.en](https://github.com/walkinglabs/learn-harness-engineering/blob/38ddcd2bf8d65271f668b94e7c875ca1d629d622/skills/harness-creator/SKILL.md.en), [skills/harness-creator/scripts/validate-harness.mjs](https://github.com/walkinglabs/learn-harness-engineering/blob/38ddcd2bf8d65271f668b94e7c875ca1d629d622/skills/harness-creator/scripts/validate-harness.mjs).

Findings:
- The creator connects instructions, state, verification, scope and lifecycle; its first move inspects existing conventions and scales complexity to observed problems.
- The validator delegates to structural scoring and exits at a configurable score, default 70. The skill itself correctly calls its benchmark structural and asks for real before/after sessions.
- The course inventory includes explicit maker/checker, graph and session-handoff exercises. Those are separate candidate patterns, not proof a graph or additional agent is needed for every task.

Boundaries:
- Mandatory feature_list/progress/init filenames and a fixed one-feature policy can duplicate an existing project task system.
- MIT source. Scaffolder and benchmark programs were inspected, not executed; a passing scaffold score would not prove professional output quality.

Applied decision:
- Rewrite shared guidance around outcome, source, execution, verification and resumable state.
- Use existing task records and independently useful failure probes; do not import a new scaffold or scoring threshold.

Additional inspected examples: [loop lecture](https://github.com/walkinglabs/learn-harness-engineering/blob/38ddcd2bf8d65271f668b94e7c875ca1d629d622/docs/en/lectures/lecture-13-loop-engineering/index.md)
(focused sections), [maker/checker graph](https://github.com/walkinglabs/learn-harness-engineering/blob/38ddcd2bf8d65271f668b94e7c875ca1d629d622/docs/en/lectures/lecture-14-graph-engineering/code/maker_checker_graph.py)
and [observability SOP](https://github.com/walkinglabs/learn-harness-engineering/blob/38ddcd2bf8d65271f668b94e7c875ca1d629d622/docs/en/resources/openai-advanced/sops/observability-feedback-loop.md).
The useful method separates finite goals from periodic monitoring and repeats the
same workload against logs, metrics or traces. The graph is an explicit teaching
skeleton: its test and approval checks are substrings, attempts never increment,
unknown review routes to merge, and in-memory checkpoints cannot establish
process-death recovery despite the comment. These placeholders and the lecture's
unverified product-command/unattended-merge claims are not adopted. Artifact-bound
checks, explicit failure/unknown handling and actual persistence remain necessary.

## harness-engineering-guide

**Decision: adapt.** conceptual guide with example code. Snapshot [86fec9bea430](https://github.com/nexu-io/harness-engineering-guide/tree/86fec9bea430cecb29ff10afaae36b96496a8f8e).

Inspected paths: [README.md](https://github.com/nexu-io/harness-engineering-guide/blob/86fec9bea430cecb29ff10afaae36b96496a8f8e/README.md), [LICENSE](https://github.com/nexu-io/harness-engineering-guide/blob/86fec9bea430cecb29ff10afaae36b96496a8f8e/LICENSE), [guide/guardrails.md](https://github.com/nexu-io/harness-engineering-guide/blob/86fec9bea430cecb29ff10afaae36b96496a8f8e/guide/guardrails.md), [guide/context-engineering.md](https://github.com/nexu-io/harness-engineering-guide/blob/86fec9bea430cecb29ff10afaae36b96496a8f8e/guide/context-engineering.md), [guide/memory-and-context.md](https://github.com/nexu-io/harness-engineering-guide/blob/86fec9bea430cecb29ff10afaae36b96496a8f8e/guide/memory-and-context.md), [guide/multi-agent-orchestration.md](https://github.com/nexu-io/harness-engineering-guide/blob/86fec9bea430cecb29ff10afaae36b96496a8f8e/guide/multi-agent-orchestration.md), [guide/skill-system.md](https://github.com/nexu-io/harness-engineering-guide/blob/86fec9bea430cecb29ff10afaae36b96496a8f8e/guide/skill-system.md).

Findings:
- Useful decomposition of tools, state, context and orchestration; progressive retrieval reduces irrelevant material.
- The context assembler example places all sections into system-role messages and promotes conversation summaries to system text. This erases source trust and is unsuitable as a copied implementation.
- The permission examples use lexical shell matching and fnmatch paths. They illustrate concepts, not canonical-path enforcement or robust command authorization.

Boundaries:
- MIT source. Claims that context matters more than model choice and generalized token arithmetic are not measurements for this harness.
- A wrapper marking content untrusted is not enforcement; strict tiered approvals would repeat permissions already established by the user.

Applied decision:
- Preserve origin through summaries and keep host security authoritative.
- Use exact explicit capability composition, no hidden authority promotion or shell guardrail imitation.

## openai-agents-python

**Decision: adapt.** application agent SDK. Snapshot [125efa029b4b](https://github.com/openai/openai-agents-python/tree/125efa029b4bfd84238bd2c4fd69c3406f802663).

Inspected paths: [LICENSE](https://github.com/openai/openai-agents-python/blob/125efa029b4bfd84238bd2c4fd69c3406f802663/LICENSE), [docs/guardrails.md](https://github.com/openai/openai-agents-python/blob/125efa029b4bfd84238bd2c4fd69c3406f802663/docs/guardrails.md), [docs/testing.md](https://github.com/openai/openai-agents-python/blob/125efa029b4bfd84238bd2c4fd69c3406f802663/docs/testing.md), [docs/sessions/index.md](https://github.com/openai/openai-agents-python/blob/125efa029b4bfd84238bd2c4fd69c3406f802663/docs/sessions/index.md), [.agents/skills/implementation-final-review/SKILL.md](https://github.com/openai/openai-agents-python/blob/125efa029b4bfd84238bd2c4fd69c3406f802663/.agents/skills/implementation-final-review/SKILL.md), [.agents/skills/implementation-final-review/references/reviewer-brief.md](https://github.com/openai/openai-agents-python/blob/125efa029b4bfd84238bd2c4fd69c3406f802663/.agents/skills/implementation-final-review/references/reviewer-brief.md).

Findings:
- Input checks attach to the initial agent, output checks to the final output; function-tool checks have a separate execution boundary. Parallel input checks can finish after tools have already run.
- Output rejection cannot undo completed tool effects; session handling preserves or sanitizes replay-valid tool records according to the specific failure path.
- Deterministic model substitutes test orchestration; real model behavior remains a separate evaluation. Repository reviewer guidance requires artifact identity and concrete failure scenarios, but is much heavier than normal personal work.

Boundaries:
- MIT source. This is an API runtime with accounts, models, tracing and session integrations, not a Codex Desktop configuration layer.
- The SDK is not installed or executed here. Its repo-specific review ledgers and fixed protocol should not be copied into every project.

Applied decision:
- Keep capability planning free of model calls; separate local structural tests, fresh agent exercises and owner acceptance.
- Carry effect/replay and artifact-identity distinctions into task execution guidance.

## CowAgent

**Decision: adapt.** multi-channel agent runtime. Snapshot [c8b1dd8c148c](https://github.com/zhayujie/CowAgent/tree/c8b1dd8c148c4d49449a69965f35977866110cbb).

Inspected paths: [LICENSE](https://github.com/zhayujie/CowAgent/blob/c8b1dd8c148c4d49449a69965f35977866110cbb/LICENSE), [agent/skills/frontmatter.py](https://github.com/zhayujie/CowAgent/blob/c8b1dd8c148c4d49449a69965f35977866110cbb/agent/skills/frontmatter.py), [agent/skills/loader.py](https://github.com/zhayujie/CowAgent/blob/c8b1dd8c148c4d49449a69965f35977866110cbb/agent/skills/loader.py), [docs/memory/context.mdx](https://github.com/zhayujie/CowAgent/blob/c8b1dd8c148c4d49449a69965f35977866110cbb/docs/memory/context.mdx), [docs/memory/self-evolution.mdx](https://github.com/zhayujie/CowAgent/blob/c8b1dd8c148c4d49449a69965f35977866110cbb/docs/memory/self-evolution.mdx).

Findings:
- The skill loader keeps parse signatures and diagnostics, detects recursive directory ancestry and separates discovered metadata from later invocation.
- The documented context pipeline trims full turns, summarizes older context and has increasingly aggressive overflow recovery. Restored sessions retain user/final text rather than full tool chains.
- Self-evolution reviews idle conversations to consolidate memory and alter skills. Its default description is inconsistent: on by default prose versus false in the configuration table.

Boundaries:
- MIT-form license notice. Automatic memory/skill mutation and background follow-up broaden authority and retention beyond this requested harness.
- Summary injection and emergency clearing can lose original tool evidence; documentation is not proof that recovery preserves the user intent. Runtime not launched.

Applied decision:
- Keep task state distinct from memory, retain retrievable full evidence, and make workflow improvement a reviewed change.
- No always-on observer, channel connector or self-modifying background service.

## pydantic-ai

**Decision: adapt.** typed agent SDK and evaluation framework. Snapshot [4c6fc3ce0ae2](https://github.com/pydantic/pydantic-ai/tree/4c6fc3ce0ae28d4a69533ff1aa20695210b03d97).

Inspected paths: [LICENSE](https://github.com/pydantic/pydantic-ai/blob/4c6fc3ce0ae28d4a69533ff1aa20695210b03d97/LICENSE), [docs/durable_execution/overview.md](https://github.com/pydantic/pydantic-ai/blob/4c6fc3ce0ae28d4a69533ff1aa20695210b03d97/docs/durable_execution/overview.md), [docs/testing.md](https://github.com/pydantic/pydantic-ai/blob/4c6fc3ce0ae28d4a69533ff1aa20695210b03d97/docs/testing.md), [docs/evals/evaluators/llm-judge.md](https://github.com/pydantic/pydantic-ai/blob/4c6fc3ce0ae28d4a69533ff1aa20695210b03d97/docs/evals/evaluators/llm-judge.md), [pydantic_ai_slim/pydantic_ai/usage.py](https://github.com/pydantic/pydantic-ai/blob/4c6fc3ce0ae28d4a69533ff1aa20695210b03d97/pydantic_ai_slim/pydantic_ai/usage.py).

Findings:
- Durable execution is explicitly different from conversation storage, and an agent may have only one durable engine.
- UsageLimits separates acted-on model responses, successful tool calls, tokens and cost. Token counts are generally checked after a response; unknown cost can make a configured cost bound unenforceable.
- TestModel is procedural schema-shaped data, not a model-quality test. LLM judges are reserved for semantic judgments while exact values, formats and deterministic logic have mechanical evaluators.

Boundaries:
- MIT source. Eight optional durability backends and provider integrations are application choices, not a reason to add dependencies to a desktop harness.
- No SDK or paid model call was run. A request-count limit does not cover every provider retry or guarantee a dollar cap.

Applied decision:
- Expose actual evidence levels and context bytes without pretending to measure model tokens or billing.
- Use deterministic financial/localization contract checks plus separate qualitative review.

## agentmemory

**Decision: defer.** memory server and capture integrations. Snapshot [9d70369f015b](https://github.com/rohitg00/agentmemory/tree/9d70369f015b0ec22eae8754ba2b24cc45d7bbe4).

Inspected paths: [LICENSE](https://github.com/rohitg00/agentmemory/blob/9d70369f015b0ec22eae8754ba2b24cc45d7bbe4/LICENSE), [SECURITY.md](https://github.com/rohitg00/agentmemory/blob/9d70369f015b0ec22eae8754ba2b24cc45d7bbe4/SECURITY.md), [packages/mcp/README.md](https://github.com/rohitg00/agentmemory/blob/9d70369f015b0ec22eae8754ba2b24cc45d7bbe4/packages/mcp/README.md), [src/hooks/_capture-filter.ts](https://github.com/rohitg00/agentmemory/blob/9d70369f015b0ec22eae8754ba2b24cc45d7bbe4/src/hooks/_capture-filter.ts), [src/mcp/tools-registry.ts](https://github.com/rohitg00/agentmemory/blob/9d70369f015b0ec22eae8754ba2b24cc45d7bbe4/src/mcp/tools-registry.ts).

Findings:
- The standalone MCP package forwards to the full agentmemory runtime; its short launcher is not a dependency-free documentation tool.
- Capture filtering defaults to allowing most tools, excludes memory/discovery patterns, and truncates output at a configurable limit (default 8000). Truncation is not secret redaction.
- The security document states there is no committed dependency lockfile and that CI resolves a fresh one, which limits source-build reproducibility.

Boundaries:
- Apache-2.0. Runtime, hooks, storage, embedding/provider configuration and retention deserve a dedicated setup decision.
- Reading hooks does not authorize session-history collection or permanent personal memory changes. No server or capture hook run.

Applied decision:
- Use the existing task record and host memory policy; preserve source identities and bound retrieval.
- Defer a dedicated memory service until a measured retrieval failure warrants its operational and data costs.

## ZCode

**Decision: adapt.** desktop and CLI runtime. Snapshot [29628c9acdb8](https://github.com/zai-org/ZCode/tree/29628c9acdb81b703bbd4080c207a0e7ce5e276e).

Inspected paths: [LICENSE](https://github.com/zai-org/ZCode/blob/29628c9acdb81b703bbd4080c207a0e7ce5e276e/LICENSE), [README.en.md](https://github.com/zai-org/ZCode/blob/29628c9acdb81b703bbd4080c207a0e7ce5e276e/README.en.md), [CONTEXT.md](https://github.com/zai-org/ZCode/blob/29628c9acdb81b703bbd4080c207a0e7ce5e276e/CONTEXT.md), [architecture-policy.yaml](https://github.com/zai-org/ZCode/blob/29628c9acdb81b703bbd4080c207a0e7ce5e276e/architecture-policy.yaml), [apps/zcode-cli/packages/bootstrap/src/app/dynamic-workflow-gate.ts](https://github.com/zai-org/ZCode/blob/29628c9acdb81b703bbd4080c207a0e7ce5e276e/apps/zcode-cli/packages/bootstrap/src/app/dynamic-workflow-gate.ts).

Findings:
- Its domain glossary separates marketplace listing from functional manifest and treats discovery, installation, configuration, invocation, update and recovery as distinct lifecycle stages.
- The dynamic-workflow gate disables specific skill paths rather than whole roots, avoiding collateral removal of unrelated skills.
- Architecture policy distinguishes managed modules from legacy modules, names public contracts and checks dependency direction. It also embeds numeric file/method limits specific to that project.

Boundaries:
- Apache-2.0 with third-party notices. An Electron/Node application is a separate product runtime, not a portable harness skill.
- No app was launched. A manifest can advertise capabilities without proving account access or runtime activation; fixed size limits are not architecture quality.

Applied decision:
- Expose exact selected sources and preserve explicit activation/readiness limits.
- Keep local CLI owners and generated documentation contracts; use risk-based checks rather than copying a fixed architecture.

## jcode

**Decision: adapt.** Rust coding-agent runtime. Snapshot [9e153ed56033](https://github.com/1jehuang/jcode/tree/9e153ed56033eee104e04beb92beac2e87ac4e3e).

Inspected paths: [LICENSE](https://github.com/1jehuang/jcode/blob/9e153ed56033eee104e04beb92beac2e87ac4e3e/LICENSE), [README.md](https://github.com/1jehuang/jcode/blob/9e153ed56033eee104e04beb92beac2e87ac4e3e/README.md), [crates/jcode-app-core/src/server/durable_state.rs](https://github.com/1jehuang/jcode/blob/9e153ed56033eee104e04beb92beac2e87ac4e3e/crates/jcode-app-core/src/server/durable_state.rs), [crates/jcode-app-core/src/agent/compaction.rs](https://github.com/1jehuang/jcode/blob/9e153ed56033eee104e04beb92beac2e87ac4e3e/crates/jcode-app-core/src/agent/compaction.rs).

Findings:
- The README describes semantic memory/skill injection and background extraction plus same-repository change notifications for collaborating agents.
- Compaction resets cache tracking and provider session identity, showing why a summary replacement is also a runtime-state transition.
- Durable-state helpers expire stale records and log failed persistence as warnings; a successful foreground action does not by itself prove persistence.

Boundaries:
- MIT source. Automatic memory consolidation, autonomous swarms and self-development require stronger user/runtime authority than a skill can provide.
- Change notifications are useful coordination signals, not proof all conflicts resolve correctly. Runtime and claimed memory savings were not measured.

Applied decision:
- Preserve one integration owner and refresh evidence after input changes.
- Keep role resolution explicit and inspectable; do not import autonomous self-updating behavior or semantic-injection machinery without a measured need.

## open-code-review

**Decision: adapt.** review CLI and integrations. Snapshot [357c3f09a264](https://github.com/alibaba/open-code-review/tree/357c3f09a264563ab64d8f4fc3b25b186cd3c4a4).

Inspected paths: [LICENSE](https://github.com/alibaba/open-code-review/blob/357c3f09a264563ab64d8f4fc3b25b186cd3c4a4/LICENSE), [ASSURANCE_CASE.md](https://github.com/alibaba/open-code-review/blob/357c3f09a264563ab64d8f4fc3b25b186cd3c4a4/ASSURANCE_CASE.md), [internal/pathutil/path.go](https://github.com/alibaba/open-code-review/blob/357c3f09a264563ab64d8f4fc3b25b186cd3c4a4/internal/pathutil/path.go), [internal/tool/comment_collector.go](https://github.com/alibaba/open-code-review/blob/357c3f09a264563ab64d8f4fc3b25b186cd3c4a4/internal/tool/comment_collector.go), [internal/tool/code_comment.go](https://github.com/alibaba/open-code-review/blob/357c3f09a264563ab64d8f4fc3b25b186cd3c4a4/internal/tool/code_comment.go).

Findings:
- Path validation and per-agent comment collection are concrete boundaries. The parser separates comment category/severity and handles model JSON failures.
- The assurance document maps trust boundaries, but its high-level claim of git-only external commands is qualified later by configured credential scripts, shell tools, MCP processes and browser launch.
- Comment parsing has a path that discards repair diagnostics; successful parsing alone does not establish sound findings or complete review.

Boundaries:
- Apache-2.0. The CLI sends code to a configured provider and may store plaintext credentials with restrictive file modes; neither effect is needed for this local redesign.
- No reviewer runtime or configured provider was invoked. Review comments need source-supported trigger and consequence, not just schema validity.

Applied decision:
- Use bounded native reviewers on exact artifacts; retain defects/preferences/unverified distinctions.
- Preserve independent verification and test consumer behavior rather than introducing another review service.

## grok-build

**Decision: adapt.** Rust coding-agent runtime. Snapshot [2bdd1d6a6369](https://github.com/xai-org/grok-build/tree/2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8).

Inspected paths: [LICENSE](https://github.com/xai-org/grok-build/blob/2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8/LICENSE), [README.md](https://github.com/xai-org/grok-build/blob/2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8/README.md), [crates/codegen/xai-grok-pager/docs/user-guide/16-subagents.md](https://github.com/xai-org/grok-build/blob/2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8/crates/codegen/xai-grok-pager/docs/user-guide/16-subagents.md), [crates/codegen/xai-grok-pager/docs/user-guide/18-sandbox.md](https://github.com/xai-org/grok-build/blob/2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8/crates/codegen/xai-grok-pager/docs/user-guide/18-sandbox.md), [crates/codegen/xai-grok-pager/docs/user-guide/13-memory.md](https://github.com/xai-org/grok-build/blob/2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8/crates/codegen/xai-grok-pager/docs/user-guide/13-memory.md), [crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md](https://github.com/xai-org/grok-build/blob/2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8/crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md).

Findings:
- Agent definitions control the session while personas are behavioral overlays; role labels do not create tools or isolation.
- Sandbox docs explicitly say sandboxing is off by default and child-network restriction differs: Linux can enforce it while the described macOS path does not.
- Global hook/config write-denials and canonical-path checks illustrate that protecting agent policy requires runtime enforcement, distinct from workspace separation.

Boundaries:
- Apache-2.0 and third-party notices. Rust/DotSlash builds and provider authentication are separate dependencies; no installation or runtime probe occurred.
- Host-specific permission inheritance and model overrides cannot be transposed to Codex/Claude tools. Documentation claims remain version-specific.

Applied decision:
- Keep the existing desktop runtimes authoritative and describe actual readiness/platform limits.
- Use roles as task contracts, not a new execution hierarchy or provider switch.

## univer

**Decision: defer.** spreadsheet/document SDK. Snapshot [bfe5d8167783](https://github.com/dream-num/univer/tree/bfe5d81677830a27f63c35a7b8515c28b615a5f9).

Inspected paths: [LICENSE](https://github.com/dream-num/univer/blob/bfe5d81677830a27f63c35a7b8515c28b615a5f9/LICENSE), [README.md](https://github.com/dream-num/univer/blob/bfe5d81677830a27f63c35a7b8515c28b615a5f9/README.md), [docs/API_STABILITY.md](https://github.com/dream-num/univer/blob/bfe5d81677830a27f63c35a7b8515c28b615a5f9/docs/API_STABILITY.md), [docs/ISOMORPHIC.md](https://github.com/dream-num/univer/blob/bfe5d81677830a27f63c35a7b8515c28b615a5f9/docs/ISOMORPHIC.md), [packages/sheets-formula/src/facade/f-formula.ts](https://github.com/dream-num/univer/blob/bfe5d81677830a27f63c35a7b8515c28b615a5f9/packages/sheets-formula/src/facade/f-formula.ts), [packages/core/src/services/command/command.service.ts](https://github.com/dream-num/univer/blob/bfe5d81677830a27f63c35a7b8515c28b615a5f9/packages/core/src/services/command/command.service.ts).

Findings:
- The public facade and command architecture distinguish workbook data, operations and the formula calculation lifecycle; calculation completion must be observed.
- The API stability policy is explicit about pre-1.0 breaking changes, coordinated package versions and internal versus public exports.
- Browser, Node and worker composition have different plugin availability; an SDK import does not supply a live Microsoft Excel session or guarantee Excel round-trip fidelity.

Boundaries:
- Apache-2.0 core with separately scoped commercial/pro functionality. Do not infer every advertised feature has the same license or runtime.
- No spreadsheet SDK was installed. Saved formulas, cached values, recalculation, rendering and preservation of workbook features require separate checks.

Applied decision:
- Prefer native spreadsheet tools; new spreadsheet-analysis guidance verifies calculations and preservation.
- Defer Univer to a project explicitly building an embedded spreadsheet product.

## Trellis

**Decision: adapt.** project workflow and context framework. Snapshot [f089cb328607](https://github.com/mindfold-ai/Trellis/tree/f089cb3286071199e84118dee86b6d76aab032f1).

Inspected paths: [LICENSE](https://github.com/mindfold-ai/Trellis/blob/f089cb3286071199e84118dee86b6d76aab032f1/LICENSE), [README.md](https://github.com/mindfold-ai/Trellis/blob/f089cb3286071199e84118dee86b6d76aab032f1/README.md), [.agents/skills/trellis-meta/references/local-architecture/context-injection.md](https://github.com/mindfold-ai/Trellis/blob/f089cb3286071199e84118dee86b6d76aab032f1/.agents/skills/trellis-meta/references/local-architecture/context-injection.md), [.agents/skills/trellis-meta/references/local-architecture/task-system.md](https://github.com/mindfold-ai/Trellis/blob/f089cb3286071199e84118dee86b6d76aab032f1/.agents/skills/trellis-meta/references/local-architecture/task-system.md), [.agents/skills/trellis-meta/references/local-architecture/workspace-memory.md](https://github.com/mindfold-ai/Trellis/blob/f089cb3286071199e84118dee86b6d76aab032f1/.agents/skills/trellis-meta/references/local-architecture/workspace-memory.md), [.codex/hooks/inject-workflow-state.py](https://github.com/mindfold-ai/Trellis/blob/f089cb3286071199e84118dee86b6d76aab032f1/.codex/hooks/inject-workflow-state.py).

Findings:
- Task requirements, design/execution plans, research, context manifests and workspace journals have distinct ownership.
- Active task pointers are scoped by session identity; absent identity must not fall back to a shared current-task pointer. Parent-child task position is explicitly not dependency scheduling.
- Context injection supports hook push and agent pull. Per-task referenced context can be small and explain why each source is relevant.

Boundaries:
- AGPL-3.0 at this revision; source behavior and terms were inspected, no templates/hooks/code copied.
- Hook support and session identifiers are host-dependent. Automatically promoting session notes into spec must still follow project/user authority.

Applied decision:
- Keep a single project-owned task record and explicit worker context with ownership.
- Use pull-based selected read paths and exact role contracts without imposing .trellis or lifecycle hooks.
