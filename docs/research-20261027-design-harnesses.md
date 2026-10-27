# Research: design harnesses and visual verification (2026-10-27)

This second-round pass covers primary engineering articles, a production case
study, and a visual/interactive benchmark published or submitted from
2025-04-27 through 2026-10-27. It excludes Reddit/X and aesthetic showcase
roundups. A source is evidence only when its publication/submission date is
visible; repository pages without a dated revision are background, not adoption
evidence.

## Search ledger

| Pass | Route | Result |
|---|---|---|
| 1 | Broad searches for `2025 2026 agent visual QA harness screenshots`, `browser visual verification coding agent`, `long-running app evaluator harness`, and `held-out agent evaluation` | Found dated primary design workflows and papers before familiar project names. |
| 2 | Publisher pages and arXiv records | Verified dates, concrete procedures, artifacts, costs, and stated limits. |
| 3 | Parent-provided Anthropic, Pixelhop, Agentic Design School, and WebGen-Agent leads | Added the three dated workflows; excluded WebGen-Agent because its supplied PDF says it remains under ICLR 2026 review. |

Source URLs and access metadata are retained under
`/Users/nazmiyilmaz/.codex/task-state/harness-design-research-round2-20261027/articles/`.
No upstream prose, code, brand asset, or font was copied into Mirket.

## Dated sources

### 1. Anthropic: evaluator/generator loop for long-running apps

**Prithvi Rajasekaran, “Harness design for long-running application
development,” Anthropic, published 2026-03-24**
([source](https://www.anthropic.com/research/harness-design-long-running-apps)).

Anthropic used planner, generator, and evaluator agents with structured
handoffs, a Playwright evaluator, and criteria for design quality, originality,
craft, and functionality. Few-shot scored examples calibrated the evaluator;
the generator could refine or pivot after each assessment. Runs used 5–15
iterations, improved then plateaued, and sometimes had a preferred middle
iteration rather than a best final iteration. A retro-game comparison reports
solo Opus 4.5 at 20 minutes/$9 versus a full harness at six hours/$200. An
updated DAW run took 3h50m/$124.70; QA still found display-only and stubbed
features.

This is first-party evidence, not independent replication. Criteria can steer
style, evaluator/model costs are specific, and no broad superiority claim
follows. The useful Mirket method is a capable disjoint reviewer, frozen rubric,
retained iterations, and best-artifact review under one shared budget. Do not
blindly repair the last iteration or treat evaluator prose as proof without
saved-artifact inspection.

### 2. Pixelhop: visible proof in the pull request

**Jozef, “Our agentic coding workflow: how we make coding agents check their
own work,” Pixelhop, 2026-10-16**
([source](https://pixelhop.io/writing/agentic-coding-workflow-visual-verification/)).

Pixelhop’s rule is “show me, don’t tell me”: changed routes receive 390px and
1440px screenshots; interactive flows receive a recording; evidence is attached
with commit, route, and viewport metadata. One feature produced 22 captures and
one interaction a 29-second recording. Before upload, the agent must open each
capture and recapture after clipping, overlap, wrong state, or visible errors.
The workflow caught a passing check that exercised an error page instead of the
signed-in product. Screenshot/video annotations return structured coordinates
or timestamps.

This is a first-party workflow report, and its “million times better” claim is
explicitly unmeasured. Media existence does not prove inspection. The direct
Mirket gap is a behavioral test requiring saved-pixel inspection, route/
viewport identity, and decoded video review for motion claims; hashes and
receipts alone must not become “verified.”

### 3. Agentic Design School: executable design-system harness

**Agentic Design School, “Build a Design Harness Before You Prompt,” published
2026-06-02, updated 2026-10-09**
([source](https://agenticdesign.school/articles/build-a-design-harness-before-you-prompt)).

The case study separates lean always-loaded project rules from design tokens,
on-demand skills, examples, and executable checks. A real audit failed a
scratch component on four hardcoded colors and passed after semantic tokens
were used. In a same-brief comparison, the unharnessed run took four rounds
and roughly 50 minutes; the harnessed run took two rounds and roughly 15
minutes, reused existing components, and passed the token audit. Both runs
missed awkward title wrapping, which human review caught.

The author labels this a reconstruction, not a benchmark, and warns that
audits catch only encoded rules and stale harnesses can be worse than none. The
useful Mirket method is lean global guidance, project-specific references on
demand, concrete recurring-failure checks, and mandatory human review for
hierarchy/density/taste. It supports a method to test, not a general uplift
claim.

### 4. Google: behavioral tests for harness regression

**Taylor Mullen and Christian Gunderman, “The Anatomy of Harness Engineering,”
Google Developers Blog, 2026-09-09**
([source](https://developers.googleblog.com/en/the-anatomy-of-harness-engineering-how-to-evaluate-iterate-and-guard-ai-coding-agents/)).

Google recommends small behavioral evaluations that assert observable actions:
clarify an underspecified prompt, run a validator after a build-file edit, or
provide canonical documentation links. It distinguishes strict tool-call
assertions for simple behaviors from fuzzier outcome/LLM judgments for complex
ones and recommends batches because agent behavior is nondeterministic.

For Mirket, the discriminating cases are: inspect saved pixels before a visual
claim; refuse video export when font/timeline prerequisites fail; preserve the
requested project; and distinguish denied, stale, unsupported, and unavailable
operations. Assert effects and artifacts, not explanations. This article gives
engineering guidance, not a published effect size.

### 5. Code23: production design-system ownership and failures

**Code23, “How We Rebuilt the Code23 Website With AI Agents,” 2026-07-15,
updated 2026-10-08**
([source](https://code23.com/blog/how-we-built-with-ai-agents-build-log/)).

This first-party build log reports a public site launched in August 2026 after
a three-month-to-seven-day rebuild. Humans owned architecture, brand, claims,
accessibility/performance approval, and release; agents handled bounded
components, migrations, test scaffolding, and repetitive checks. Reported
failures were invented brand facts, scope creep, tests asserting broken
behavior, flaky previews, and adjacent-agent changes without a merge plan.
The team retained broken-state screenshots and used confidence/sample
thresholds before creating post-launch evidence tasks.

There is no independent audit or matched causal comparison, so the speed claim
is not transferable. The actionable method is lead-owned integration, claims
provenance, test-intent review, explicit scope boundaries, and a human release
decision. It is useful counterevidence against treating a polished launch or
agent summary as verification.

### 6. ArtifactsBench: visual plus interaction evidence

**Zhang et al., “ArtifactsBench,” arXiv:2507.04952, submitted 2025-07-07,
revised 2025-09-29**
([paper](https://arxiv.org/abs/2507.04952)).

ArtifactsBench renders generated artifacts, captures temporal screenshots of
dynamic behavior, and combines source with per-task checklists for multimodal
judging. It reports 1,825 tasks, more than 30 models, 94.4% ranking consistency
with WebDev Arena, and over 90% pairwise agreement with human experts.

Those are benchmark authors’ results: judge agreement does not establish local
correctness, and visual judging cannot replace deterministic interaction or
accessibility checks. The relevant Mirket method is to retain intent/reference,
fixed-width and state captures, accessibility output, interaction traces, and
independent saved-pixel review. Do not convert benchmark agreement into a
claim that Mirket’s visual quality improved.

## Excluded background

Current GitHub repositories (`design-harness`, `veriharness`,
`coding-agent-eval-harness`, `praxen`, and `agent-execution-harness`) and the
undated OpenAI Sandbox Agents guide were inspected for comparison, but are not
adoption evidence here because a dated revision/publication or independent
outcome was not verifiable in the requested window. The supplied WebGen-Agent
PDF is explicitly under ICLR 2026 review and is likewise excluded.

## Bounded recommendations

1. Freeze brief, rubric, budget, skill snapshot, and evidence packet for
   high-risk trials; retain initial artifact, findings, repair, final artifact,
   remaining defects, and cost/latency.
2. Require saved-pixel inspection for visual claims and decoded-export review
   for video claims. Keep deterministic failures separate from heuristic and
   subjective review findings.
3. For creative work, use a capable disjoint reviewer, calibrated criteria,
   retained iterations, and one shared review/repair budget. The final iteration
   is not automatically best.
4. Keep global guidance lean and project-specific; encode recurring failures,
   not every isolated miss, and retain human review for taste and hierarchy.
5. Treat tests, hashes, receipts, tool listings, and model self-report as
   supporting evidence only, never as proof of visual quality or completion.

These are methods to test, not claims that Mirket has already improved visual
quality. Windows qualification and the waived Desktop GUI check remain
unverified.
