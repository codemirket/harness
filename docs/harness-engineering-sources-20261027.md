# Harness engineering source ledger

Accessed 2026-10-27. Research window: 2025-04-27–2026-10-27. Companion:
[recommendation and proposed tests](harness-engineering-research-20261027.md).

This is a research bibliography, not an admission to `planning/references.md`,
the skill catalog, or production dependencies. Techniques are summarized in
original prose; source material is not copied into skills. Repository licenses
below are observations at the inspected revision, not a blanket license for
all linked assets, trademarks or dependencies.

Evidence labels: **engineering report** = primary account, usually self-reported;
**study** = inspectable experimental method, with scope limitations;
**repository** = source/metadata inspection, not execution;
**discussion** = practitioner observation or preference, not causal evidence.

## Primary engineering reports

| Source and publication date | Useful observation | Limitation |
|---|---|---|
| [OpenAI: Harness engineering](https://openai.com/index/harness-engineering/), 2026-02-11 | Repository knowledge, application observability and architectural invariants support autonomous implementation | Internal account; estimated time saving has no controlled comparison |
| [Stripe: Minions, part 1](https://stripe.dev/blog/minions-stripes-one-shot-end-to-end-coding-agents), 2026-02-09, and [part 2](https://stripe.dev/blog/minions-stripes-one-shot-end-to-end-coding-agents-part-2), 2026-02-19 | Deterministic checks mixed with agent work, curated context/tools and a bounded CI loop | Internal production report; isolated cloud environment does not justify laptop permission bypasses |
| [Anthropic: Writing effective tools for agents](https://www.anthropic.com/research/writing-tools-for-agents), 2025-09-11 | Tool descriptions, bounded useful output, realistic multi-step evaluation | Provider guidance; evaluate each local contract |
| [Anthropic: Harness design for long-running apps](https://www.anthropic.com/research/harness-design-long-running-apps), 2026-03-24 | Independent visual evaluator, browser inspection, rubric calibration, retained intermediate candidates | Unmatched budgets; later aesthetic iterations sometimes regress |
| [OpenAI/deepsense: Harness-aware plugin evaluations](https://developers.openai.com/cookbook/examples/partners/harness_aware_plugin_evals/harness_aware_plugin_evals), 2026-09-23 | Tool tests, standalone loops and actual product-host tests answer different questions | Five synthetic BLS cases; notebook includes replay paths; full host reproduction is separate |
| [Google: Anatomy of harness engineering](https://developers.googleblog.com/en/the-anatomy-of-harness-engineering-how-to-evaluate-iterate-and-guard-ai-coding-agents/), 2026-09-09 | Behavioral assertions plus end-to-end evaluation and regression checks | Engineering guidance, not a Mirket comparison |
| [Vercel: We removed 80% of our agent's tools](https://vercel.com/blog/we-removed-80-percent-of-our-agents-tools), 2025-12-22 | A structured filesystem and general tools simplified a SQL agent | Only five queries; semantic data preparation remained essential |
| [Vercel: AGENTS.md outperforms skills in our agent evals](https://vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals), 2026-01-27 | Compressed documentation index helped tasks using unfamiliar Next.js APIs; optional skills often were not invoked | Narrow framework/version task set, not a verdict on all skills |
| [LangChain: Improving Deep Agents with harness engineering](https://www.langchain.com/blog/improving-deep-agents-with-harness-engineering), 2026-02-17 | Trace-guided environment, verification and loop changes altered performance with the same model | Tuning on the benchmark and reasoning/time choices limit causal claims |
| [Manus: Context engineering lessons](https://manus.im/blog/Context-Engineering-for-AI-Agents-Lessons-from-Building-Manus), 2025-07-18 | Stable context, restorable files and retained failure evidence | Tool masking/cache internals assume control of the model loop, which Mirket lacks |
| [Cursor: Improved token efficiency](https://cursor.com/blog/improved-token-efficiency), 2026-09-23 | Prompt reduction, deferred tools and context changes can reduce overhead | Vendor-reported result; these are primarily host internals |
| [Ramp: Why we built our background agent](https://builders.ramp.com/post/why-we-built-our-background-agent), 2026-01-12 | Full application environments and reviewable previews aid unattended work | Internal adoption claims; cloud architecture is not required for Mirket |
| [Ramp: Building Laminar](https://builders.ramp.com/post/building-laminar), 2026-10-20 | Durable sessions, versioned agent definitions and explicit ownership | Recent internal account, not independent comparative evidence |
| [Birgitta Böckeler: Harness engineering](https://martinfowler.com/articles/harness-engineering.html), 2026-04-02 | Distinguishes the provider's inner harness from user-side guidance and feedback | Conceptual framework, not benchmark results |
| [Mitchell Hashimoto: My AI adoption journey](https://mitchellh.com/writing/my-ai-adoption-journey), 2026-02-05 | Learn limits on known tasks; turn recurring mistakes into fast feedback | One experienced practitioner's account |
| [Integral: Agent harness](https://engineering.integral.de/posts/007-agent-harness/), 2026-07-02 | Artifact-centered work can reduce coordination overhead | Small-team experience, not population evidence |
| [O'Reilly: Agent harness engineering](https://www.oreilly.com/radar/agent-harness-engineering/), 2026-05-15; [Stop overengineering your agent harness](https://www.oreilly.com/radar/stop-overengineering-your-agent-harness/), 2026-07-22 | Practitioner synthesis and counterweight to growing orchestration | Opinion/synthesis; used for triangulation, not technical proof |

## Empirical studies and frontier work

| Source and in-window version | Scope and implication | Qualification |
|---|---|---|
| [Evaluating AGENTS.md](https://arxiv.org/abs/2602.11988), v3 2026-09-29 | 300 SWE-bench-Lite tasks/11 Python repositories plus 138 CTXbench tasks/12 Python repositories, four agent/model pairs. No general significant success gain; generated context increased average cost 20%/23%. Developer context's CTXbench improvement was not significant (`p=0.21`). A documentation-removal ablation found a conditional benefit when context was the only documentation | One completion/configuration, Python-only, test-based outcomes. Use latest version rather than early press summaries |
| [AGENTS.md efficiency study](https://arxiv.org/abs/2601.20404), submitted 2026-01-28 | 124 pull requests/10 repositories; reported 28.64% lower median runtime and 16.58% fewer output tokens with comparable completion behavior | Specific repository/agent comparison, not a general randomized intervention or proof of correctness/safety; instruction content was not isolated |
| [Towards a Science of Scaling Agent Systems](https://arxiv.org/abs/2512.08296), v3 2026-04-08 | 260 configurations, six benchmarks, five architectures, three model families; task structure changes the value of coordination | Relative task-specific gains/regressions are not portable guarantees for a coding team |
| [METR experienced-developer study](https://arxiv.org/abs/2507.09089), 2025-07; [2026 update](https://metr.org/blog/2026-02-24-uplift-update/), 2026-02-24 | Measured developer time can disagree with perceived speed; later evaluation must account for selection and concurrent work | Early-2025 treatment/models; the reported slowdown is not a current universal coefficient |
| [SWE-rebench](https://proceedings.neurips.cc/paper_files/paper/2025/hash/21bec6ace947b1b58967b945c8ac0f10-Abstract-Datasets_and_Benchmarks_Track.html), 2025 proceedings | Fresh repository tasks help reduce static-benchmark dependence | Dataset correctness and representativeness still require checking |
| [Terminal-Bench 2.0 announcement](https://www.tbench.ai/news/announcement-2-0), 2025-11-07; [paper](https://arxiv.org/abs/2601.11868), submitted 2026-01-17 | Reproducible execution environments and heterogeneous tasks provide a useful evaluation model | Leaderboard performance does not qualify Mirket skills or its Desktop integration |
| [MCP Atlas](https://labs.scale.com/papers/mcpatlas), 2025-12-18 | Realistic multi-tool tasks test more than tools/list/schema validity | Benchmark environment differs from the user's runtime and permissions |
| [MemBench](https://arxiv.org/abs/2506.21605), 2025-06; [Mem0](https://arxiv.org/abs/2504.19413), 2025-04-28; [LongMemEval V2](https://arxiv.org/abs/2605.12493), 2026-05 | Useful evaluation ideas for retained facts, evolving information and long-term recall | Neither a vendor result nor a benchmark score justifies replacing scoped SQLite/FTS and lazy MiniLM |
| [Harness²](https://arxiv.org/abs/2610.14224), v1 2026-10-09 | Experimental recursive task/harness improvement across professional-task benchmarks | New author-reported results; no independent Mirket-relevant replication |
| [HarnessPro](https://arxiv.org/abs/2610.26770), v2 2026-10-20 | Candidate harness search using generated tests and quality/cost tradeoffs | New, under review; proxy quality and optimization overfitting need independent checks |

The context studies and Vercel result are not reconciled by declaring one
format universally best. A documentation index may supply otherwise missing
information; a generic repository overview may add cost without information.
Measure task success and cost against a matched no-context baseline.

## GitHub: established projects inspected

Stars are rounded snapshots on 2026-10-27, **not active-user counts or quality
scores**. Exact revisions permit later inspection without trusting a moving
README. No upstream installers, hooks or code were executed for this research.

| Project / inspected revision | License | Approximate stars | Technique worth studying |
|---|---|---|---|
| [Pi](https://github.com/earendil-works/pi/tree/b3a29f1111114fac696e2cfcd65acdb382a6ed5e) | MIT | 118k | Small core, extension boundaries, observable session/agent loop |
| [Superpowers](https://github.com/obra/superpowers/tree/2ce8aa15d05bd57213e6f746f953aafb7ded7a8b) | MIT | 306k | Skill-driven task contracts, testing and independent review |
| [GSD Core](https://github.com/open-gsd/gsd-core/tree/d90964063dda547f97b05f174158ca46ffc516f7) | MIT | 10.9k | Durable phase artifacts and context continuity |
| [GitHub Spec Kit](https://github.com/github/spec-kit/tree/ee94598c3c87ae231027b22fb2be2b5a13e8820f) | MIT | 143k | Specification-to-plan-to-task traceability |
| [OpenCode](https://github.com/anomalyco/opencode/tree/618b89066773e707601577230ba7211605c8b569) | MIT | 222k | Host session/tool architecture and failure handling |
| [Deep Agents](https://github.com/langchain-ai/deepagents/tree/bea2ad54ea872cbc8e026cc92afdde1b4cf40e39) | MIT | 30.6k | Context and evaluation patterns, inspectable orchestration |
| [mini-SWE-agent](https://github.com/SWE-agent/mini-swe-agent/tree/04d809ceab9df28f9adaed044884180159172930) | MIT | 8.9k | A minimal comparison point before adding complexity |
| [agent-browser](https://github.com/vercel-labs/agent-browser/tree/edb40f1878864f90e14f6c11e26fdafd3fecc8dc) | Apache-2.0 | 44.7k | Explicit browser observations and postconditions |

Pi's earlier repository name was `badlogic/pi-mono`.
[gsd-build/get-shit-done](https://github.com/gsd-build/get-shit-done) was archived
at inspection and points to GSD Core; its approximately 64k stars must not be
misrepresented as the successor's stars or current installer recommendation.

[Qoder Better Harness](https://github.com/QoderAI/better-harness) was inspected as
a smaller, recent method-discovery project (MIT, approximately 2.4k stars).
Its [original announcement](https://qoderai.github.io/better-harness/blog) is
2026-07-30; the [Alibaba article](https://www.alibabacloud.com/blog/the-better-harness-evaluation-model-used-by-100000-users-is-now-open-source_603641)
on 2026-10-15 is a recap. The reported 100,000-user figure is a vendor claim,
not independently verified successful deployments. No immutable revision was
retained for reuse; it is not an admitted source package.

### Failure reports used to challenge popularity

These are reported failure cases, not locally reproduced findings. They show
what to test before adopting a technique, not that an entire project is poor.

- [Pi #7053](https://github.com/earendil-works/pi/issues/7053): tool output can
  appear in the UI before durable transcript persistence when a parallel
  sibling stalls. Lesson: distinguish visible progress from recoverable state.
- [Superpowers #2092](https://github.com/obra/superpowers/issues/2092): an invalid
  fixture can make a testing/review workflow falsely green. Lesson: independent
  test validity and discriminating controls matter.
- [Spec Kit #5105](https://github.com/github/spec-kit/issues/5105): update-related
  metadata/hook/link loss. Lesson: distribution/update fidelity needs its own
  tests; authored skill text is not the whole product.
- [agent-browser #1922](https://github.com/vercel-labs/agent-browser/issues/1922):
  stale references can yield apparent action success without the intended
  effect. Lesson: observe postconditions in Codex browser QA; do not add a
  general Mirket browser.
- [Deep Agents #7279](https://github.com/langchain-ai/deepagents/issues/7279):
  requested scoped sharing between selected agents illustrates that generic
  memory abstractions may not match project isolation requirements.

Low-adoption repositories also surfaced in discovery (including Headlong,
TraceTrial-Bench and several small harness/eval projects). They were not
promoted to the shortlist merely because their names matched the query.
Missing reuse rights and proxy/echo evaluations are reasons to exclude, not
to fill a tool-count quota.

## X, Reddit and Hacker News

The lead read the X and Reddit items below in the signed-in browser. Metrics
are rounded visible values at access time; a researcher without that session
had narrower X access. This distinction avoids claiming that all X was blocked
or that web search snippets were equivalent to reading the posts.

| Discussion | Date; visible attention | What it contributes |
|---|---|---|
| [Boris Cherny: clear goals, effort and verification](https://x.com/bcherny/status/2107565388250874193) | 2026-10-06; about 13k likes, 1.2k reposts, 1.4m views | Strong practitioner interest in giving capable agents goals and verification rather than micromanaging every step |
| [Boris Cherny: task-specific verification](https://x.com/bcherny/status/2044847858634064115) | 2026-04-16; about 1.7k likes, 84 reposts, 226.6k views | Tests/browser/computer use should match the task; the post's productivity claim was not independently demonstrated |
| [Motion handoff account](https://x.com/beamnxw/status/2113901317349351694) | 2026-10-24; 99 likes, 16 reposts, about 124k views | Distinguishes editable source, an existing short preview and a longer intended film that was not produced. Disclosed paid partnership; not independent adoption evidence |
| [Pi: what I learned building my own harness](https://www.reddit.com/r/PiCodingAgent/comments/1xd8aks/what_i_learned_while_building_my_own_ai_harness/) | 2026-10-23; score 62, 27 comments | OP criticizes excessive orchestration; replies defend context isolation and independent review. Do not adopt the OP's unsupported percentage as a fact |
| [LLMDevs: orchestration harness](https://www.reddit.com/r/LLMDevs/comments/1u5tij8/orchestration_harness_for_coding_agents/) | 2026-06-14 | Keyword triggers can misfire and miss meaning; skill activation needs examples and counterexamples |
| [LocalLLaMA: long autonomous tasks](https://www.reddit.com/r/LocalLLaMA/comments/1vvsgsp/best_harness_for_long_autonomous_tasks/) | 2026-08-23 | Conflicting experience with delegation and compaction; no single community winner |
| [HN: minimal-agent discussion](https://news.ycombinator.com/item?id=45001051) | 2025-08-24; 472 points, 122 comments | Interest in simpler loops, with objections about production requirements |
| [HN: Ralph loop discussion](https://news.ycombinator.com/item?id=45005434) | 2025-08-24; 425 points, 308 comments | Simple bounded loops can be useful, but larger prompts and unattended repetition can waste work |
| [HN: evidence for agentic coding](https://news.ycombinator.com/item?id=46691243) | 2026-01-20; 461 points, 455 comments | Domain-specific success and regressions; developers ask for observable evidence |
| [HN: GSD discussion](https://news.ycombinator.com/item?id=47417804) | 2026-03-17; 473 points, 253 comments | Checkpoint/context benefits weighed against token and process cost |
| [HN: specification-driven development](https://news.ycombinator.com/item?id=45610996) | 2025-10-16; 128 points, 32 comments | Executable acceptance is useful; large up-front documents can become overhead |
| [HN: agent skills discussion](https://news.ycombinator.com/item?id=48015397) | 2026-05-04; 376 points, 212 comments | Skepticism about prompt packaging alongside support for expert workflows and review |

Highly liked course promotions and viral “end of harness engineering” summaries
were discovery leads only. A specific primary paper was not established for
the latter slogan, so it is excluded as a technical claim. Popularity does not
remove the need to read the method, denominator, limitations and opposing
experience.

## Local evidence and reproducibility

Local review used the approved plan, AGENTS.md, current Git state and affected
source. The strongest implementation-specific conclusion is supported by:

- `adapters/src/video-export.ts:130-178`: caption validation and fail-fast
  static preflight; checks already precede media process launch.
- `src/process_video.rs:278-327`: adapter evidence and refusal boundary.
- `src/video_session.rs`: scoped intents, source rechecks, durable results and
  saved-artifact observation.
- `tests/design-adoption-video-runtime.mjs:39-53` and
  `tests/motion-authoring-runtime.mjs:67-79`: different caption selection.
- [Motion authoring trials](motion-authoring-trials-20261027.md): four repaired
  model outputs still refused at caption preflight, no exported film.
- [Motion examples](motion-authoring-examples-20261027.md) and
  [video route](video-export-route.md): existing controls and qualification
  boundaries, not blanket claims about every authored animation.

Private working notes and authenticated-page observations are retained at
`/Users/nazmiyilmaz/.codex/task-state/harness-engineering-research-20261027/`:
`github.md`, `methods.md`, `community.md`, `local-gaps.md`, `x-*.json`,
`x-motion-handoff.txt`, and `reddit-pi.json`. They are research notes rather than
shipped qualification evidence. Source links, dates, revisions and the
decision-relevant limitations are retained in this public-facing ledger so
the recommendation does not depend on accessing the private scratch files.
