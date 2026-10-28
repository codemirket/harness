# Ending the design timeout investigation

Research/access date: 2026-10-28. Repository baseline: `a75dea2`.
Purpose: revise the remaining work after the user rejected another multi-hour
investigation. This is a decision record, not a new qualification or dependency
admission. The [revised execution plan](../planning/remaining-design-work.md)
contains the work cap, owners and acceptance.

## Recommendation

Use the existing Codex Desktop production path, working incrementally from the
accepted Daybreak artifact. Separate static composition, motion implementation,
deterministic checking and independent visual review. Allow one repair, then
deliver or stop with a concrete defect. Pause the isolated creative benchmark
campaign. Do not build another runner, orchestration layer or rendering engine.

The strongest established engineering reference for the **stopping rule** is
Stripe's bounded agent/deterministic-check workflow. The closest established
reference for **frontend quality** is Anthropic's generator/evaluator work.
LangChain's newly public **HyperFrames design-agent implementation** is a useful
concrete comparison, but its recent publication and internal results do not
establish broad independent adoption. None proves a guaranteed one-hour route
to premium motion design.

## Local evidence: what is and is not broken

- D6 delivered an actual 12-second, 360-frame film, targeted pixel regressions,
  decoded frame correspondence and a browser review pack. The user accepted
  that candidate's direction. [D6 evidence](design-review-pack-20261028.md).
- Both Lantern authors produced saved interfaces that passed the recorded
  independent journeys, but both exceeded the 240-second author deadline. One
  also violated output scope. These are failed executions with separately
  reviewed artifacts, not successful benchmark cells.
- One handoff file was saved around 239.3 seconds; process exit followed the
  cutoff promptly. The retained traces cannot identify model/service/tool time
  separately. The later native readiness probe completed in 7.94 seconds, but
  had no design task or Mirket tools. [Timing evidence](design-acceptance-sample-20261028.md#timing-investigation-after-the-closed-handoff-pair).
- Storyboards, a dependency-free seed, source checks, native-animation preview,
  finite video jobs, decoded checks and repair lineage already exist. Adding
  them again under new names would add maintenance rather than fix a gap.

Our inference is therefore a **planning correction**, not a discovered runtime
root cause: repeated attempts to qualify an isolated end-to-end author under a
short deadline have become the critical path after a usable artifact already
received acceptance. Product usability, operational completion and comparative
model/skill evidence must stay distinct.

## Public sources and the decisions they support

Articles below are within 2025-04-28–2026-10-28. Live documentation is explicitly
undated; it is checked for present behavior, not used as dated popularity
evidence. Primary reports were opened and read, including their limitations.

| Source / date | Actual evidence | Adopt here / limit |
| --- | --- | --- |
| [Stripe: Minions, part 2](https://stripe.dev/blog/minions-stripes-one-shot-end-to-end-coding-agents-part-2), 2026 engineering report | Its workflow alternates agent steps with deterministic checks. It allows two CI runs with “one more chance to fix the failing test locally” between them, then hands back to a human. The authors cite diminishing returns from indefinite loops. | Our one-repair design limit is an adaptation, not Stripe's design-specific result. Use existing scripts and Codex ownership; do not copy its infrastructure, permission policy or push behavior. |
| [Anthropic: harness design for long-running application development](https://www.anthropic.com/research/harness-design-long-running-apps), 2026-03-24 | Frontend generation is separated from an evaluator using explicit design/craft/function criteria and actual page navigation. The reported 5–15 iteration design runs reached four hours; another application run reached six hours. | Adopt a narrow contract and one independent reviewer. Reject the unbounded repetition and broad planner expansion for this task. It does not prove Claude outperforms our selected Codex model. |
| [Anthropic: effective harnesses for long-running agents](https://www.anthropic.com/research/effective-harnesses-for-long-running-agents), 2025-11-26 | Incremental feature work and durable progress artifacts address one-shot overreach and poor session handoffs. End-to-end browser tests catch defects missed by source-only checks. | Save one completed stage and next action in the existing project. Use native Codex continuity; no new initializer service, feature database or Claude SDK. |
| [OpenAI: harness engineering](https://openai.com/index/harness-engineering/), 2026-02-11 | The internal account emphasizes inspectable applications, small building blocks and a short navigation map rather than an oversized instruction file. It also describes six-hour runs. | Connect existing tools through a concise entry point. Do not add more global guidance or claim this report promises fast creative work. |
| [LangChain: improving Deep Agents](https://www.langchain.com/blog/improving-deep-agents-with-harness-engineering), 2026-02-17 | With GPT-5.2-Codex fixed, harness changes improved its reported Terminal-Bench score. Maximum reasoning caused more timeouts than high reasoning in that setup. Environment context, verification and trace review helped. | Keep current user model/effort choices; remove unnecessary setup work before considering model changes. Do not transfer its numerical results or reasoning schedule to Astra/design tasks, or add Mirket model middleware. |
| [Anthropic: demystifying agent evals](https://www.anthropic.com/research/demystifying-evals-for-ai-agents), 2026-01-09 | Distinguishes environment outcomes from transcripts, capability from regression tests, and objective checks from calibrated subjective judgment. Descript is discussed as separating quality benchmarking and regression checks. | Record execution, artifact validity and visual findings separately. Preserve old timeout failures. Pause broad skill comparisons without silently treating them as passed. |
| [Anthropic: infrastructure noise](https://www.anthropic.com/research/infrastructure-noise), 2026-02-05 | Controlled resource changes affected Terminal-Bench outcomes; allocation guarantees and kill limits are different concepts. Beyond headroom for transient failures, extra resources also change the task being measured. | A deadline-constrained result is not a pure model-quality measurement. This is not evidence our timeouts were infrastructure faults, and not justification to increase their frozen limits. |
| [LangChain: how we built our design agent](https://www.langchain.com/blog/design-agent), 2026-10-27 | Describes static storyboard → animatic → production, persistent project source, rendered-image review and deterministic checks. It uses HyperFrames for editing **and a separate canvas engine for original animation**; its film examples cannot all be attributed to HyperFrames. | Reuse accepted static art and check selected moments before expensive full renders. Keep only our selected HyperFrames route. No second engine, Slack service, hosted runtime or Paper bridge. Internal usage claims are self-reported; large jobs still take hours. |
| [HyperFrames deterministic rendering](https://hyperframes.app/docs/2-concepts/4-determinism), current undated docs | Describes frame-driven capture, finite dimensions/fps/duration, preloaded assets and preview/export expectations, with platform caveats. | Supports the selected deterministic-render approach. Our [pinned 0.8.197 contract](../skills/video-production/references/hyperframes-0.8.197-authoring.md) and adapter remain authoritative for admitted APIs. No Docker migration, new `seek()` interface or promise of universal parity. |
| [METR: revising its productivity experiment](https://metr.org/blog/2026-02-24-uplift-update/), 2026-02-24 | Reports that selection effects and concurrent-agent time measurement undermine conclusions from its newer productivity experiment. The old slowdown figure is not a reliable current universal estimate. | Measure our own wall time to a checked artifact. Neither vendor benchmark scores nor old productivity headlines justify another broad comparison here. |

## Concrete open-source cross-check

The GitHub connector retrieved the LangChain repository at
[`9fd563be8be0f3a2c956599f8ed4672ae0d1f281`](https://github.com/langchain-ai/design-agent/tree/9fd563be8be0f3a2c956599f8ed4672ae0d1f281),
committed 2026-10-27. Its
[README](https://github.com/langchain-ai/design-agent/blob/9fd563be8be0f3a2c956599f8ed4672ae0d1f281/README.md),
[instructions](https://github.com/langchain-ai/design-agent/blob/9fd563be8be0f3a2c956599f8ed4672ae0d1f281/instructions.md)
and [Apache-2.0 license](https://github.com/langchain-ai/design-agent/blob/9fd563be8be0f3a2c956599f8ed4672ae0d1f281/LICENSE)
were inspected. They make storyboard approval, render inspection, machine checks
and a limit on blocking correction rounds concrete. We have not executed its
code or validated its claimed asset throughput. These are independently
summarized workflow ideas; no code, assets or instruction bodies are copied,
installed or admitted. Existing catalog reference/provenance records therefore
do not change in this planning pass.

Public Reddit discussions were checked as practitioner evidence, including
[the September 2026 overbuilt-harness retrospective](https://www.reddit.com/r/AI_Agents/comments/1w8h85a/i_spent_883_commits_and_8_months_building_an_llm/).
It is one author's account, useful as a warning about spending more time on the
harness than its outputs, not proof of a general failure rate or a technical
root cause. Reddit search also surfaced conflicting Codex design experiences;
we do not infer model superiority from those anecdotes. X searches produced no
additional inspected primary report that changed the decision. No claim of
worldwide consensus or authenticated X coverage is made in this pass.

## What changes in the repository plan

1. Replace rolling cohort/diagnostic work with one local production closure,
   capped at 60 minutes including verification and one repair. This cap is our
   scope choice, not a measured speed guarantee from the sources.
2. Use the existing seed, source checker, preview, Mirket finite jobs and
   Daybreak checks. Make their execution order easy to discover. Do not create
   another production-packet schema or generalized visual validator.
3. Prove one small edit through the whole path on a disposable Daybreak copy.
   Keep the accepted original intact. The goal is a repeatable edit/export/QA
   journey, not another blank-page creative comparison.
4. Keep failures honest and separate: native completion, file/scope validity,
   actual export integrity, visual review and user acceptance. Correct files
   after a timeout do not make that model trial pass.
5. End the closure with deliverables or a specific blocker. Keep release rights,
   OS-floor/Windows qualification and broad skill/model acceptance in their
   existing backlog; none is implied by a successful local maintenance journey.

## Rejected expansions and unresolved limits

We rejected larger timeouts as the first intervention, weaker creative models,
fresh forced context resets, a new multi-agent framework, new hosted/browser
dependencies, general collision detectors, automatic aesthetic grading and
further instrumentation without a production failure. Some public projects use
these techniques successfully; they either violate our boundary, duplicate
existing work or lack task-specific evidence here.

The research does not resolve the causal allocation of the old 240 seconds.
It does not establish premium quality on arbitrary future briefs. The smaller
maintenance journey must still be run; this document is not its outcome. We
can stop spending on those open research questions while preserving them and
finishing the useful path the user already accepted.
