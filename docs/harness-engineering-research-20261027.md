# Harness engineering: evidence and a practical direction for Mirket

Research date: 2026-10-27. Source window: 2025-04-27 through 2026-10-27.
Repository reviewed at `0fdad609fccfc58b97c1dda68f373fa0dd2c3c91`.
Status: research and proposed next implementation; not new qualification.

Implementation follow-up: [video authoring diagnostics and review
integrity](video-authoring-diagnostics-20261027.md) records the subsequent
changes and measured results. The recommendations below preserve the research
decision point; they are not themselves evidence that the implementation or
model outcomes passed.

## Recommendation

Keep Codex Desktop as the agent harness and Mirket as its narrow project
capability layer. Improve the quality and speed of feedback at that boundary:
precise authoring diagnostics, executable examples, selective context,
artifact-based review, and evaluation inside the actual host. Do not install a
second orchestrator to compensate for unclear operation contracts.

The strongest recurring practical pattern is:

> Give the agent a clear outcome and the relevant environment; make errors
> observable early; verify the resulting artifact independently; preserve the
> evidence needed to resume or reject the work.

This is a synthesis, not a claim that one framework won a worldwide survey.
GitHub stars, X likes, Hacker News votes and Reddit scores establish attention.
They do not establish task success, paid adoption, or developer consensus.

The [source ledger](harness-engineering-sources-20261027.md) separates primary
engineering reports, empirical studies, repository inspection and practitioner
discussion. It includes counterevidence and source-specific limitations.

## Research method and limits

The research used independent streams for repositories, engineering/evaluation
methods, community experience, and the current Mirket implementation. The lead
read primary reports and cross-checked findings against source and retained
local trials. GitHub inspection included current revisions, licenses,
implementation structure and failure reports, rather than README popularity
alone. X and Reddit posts were also read through the user's authenticated
ordinary Chrome session. No posts, messages, reactions or follows were sent.

Searches covered minimal harnesses, context and skills, specification-driven
work, long-running recovery, tool design, multi-agent coordination, evaluation,
visual verification and motion handoffs. Older projects are included only as
current repository observations; older articles outside the source window are
not evidence for current trends. Recent papers are labeled experimental rather
than treated as established practice.

The sample is purposive and primarily English-language. Public success stories
have selection and commercial incentives; issue reports have failure selection
bias. Engagement is a volatile snapshot. We did not independently reproduce
external benchmark results or install the inspected frameworks. Public issue
reports are not locally reproduced defects. No new model or renderer trials
were run for this research. Existing local evidence is identified as such.

## What the evidence actually supports

### 1. A small harness can be powerful when the environment is rich

OpenAI's internal engineering account emphasizes repository knowledge,
observable application behavior and mechanically enforced architecture. Stripe
describes interleaving agent work with deterministic checks and limiting CI
retries. Neither establishes that every team needs its cloud execution system.
[OpenAI, February 11](https://openai.com/index/harness-engineering/);
[Stripe, February 19](https://stripe.dev/blog/minions-stripes-one-shot-end-to-end-coding-agents-part-2).

Vercel's SQL-agent report found better results after removing most specialized
tools and exposing structured files and shell access. The comparison contained
only five queries: it supports testing simplification, not a general 100%
success claim. For Mirket, it reinforces the existing decision to leave shell,
SQL and general browsing with Codex/project tools.
[Vercel, December 22, 2025](https://vercel.com/blog/we-removed-80-percent-of-our-agents-tools).

The transferable choice is to improve the tools already justified by Mirket's
scope. More tools or more agents should need evidence of a missing capability.

### 2. Select context; do not maximize it

Vercel's Next.js evaluation found a compressed documentation index in AGENTS.md
more effective than optionally invoked skills for its new-API tasks. That is a
retrieval/activation result for a particular framework, not proof that every
skill body belongs in global guidance.
[Vercel, January 27](https://vercel.com/blog/agents-md-outperforms-skills-in-our-agent-evals).

The latest ETH repository-context study found no general significant resolution
gain and increased cost. Context helped conditionally when it supplied
documentation otherwise unavailable. The joint implication is a short routing
map plus non-obvious constraints, with
task-specific resources loaded when relevant. Treat the exact format as a
testable choice, not a doctrine.
[ETH study, v3 September 29](https://arxiv.org/abs/2602.11988).

Keep the nine global skills lean. Do not preload 58 project skills. Test whether
representative requests activate the right skill and whether its instructions
actually improve the outcome. A source-only motion handoff and a video-export
request should not silently share the same completion claim.

### 3. Early deterministic feedback is the most defensible next investment

Stripe's reported workflow runs quick local checks before expensive CI and
caps CI rounds. Anthropic's tool-design guidance recommends meaningful,
bounded results and realistic multi-step evaluation. Mitchell Hashimoto's
practitioner account describes turning recurring mistakes into better tools
and checks. These are independent descriptions of a similar mechanism, not
independent measurements of the same intervention.
[Stripe](https://stripe.dev/blog/minions-stripes-one-shot-end-to-end-coding-agents-part-2);
[Anthropic, September 11, 2025](https://www.anthropic.com/research/writing-tools-for-agents);
[Hashimoto, February 5](https://mitchellh.com/writing/my-ai-adoption-journey).

Mirket already checks video inputs before launching renderers. The demonstrated
gap is the quality and authoring accessibility of those checks, especially the
single `invalid_caption_cues` response. Adding another reasoning loop would
not explain the invalid field to its caller.

### 4. Visual review helps, but repeated review is not monotonically better

Anthropic's long-running application experiments use a separate evaluator,
browser inspection and a calibrated design rubric. The author also reports
preferring intermediate candidates over some final candidates. Examples with
different time/cost budgets do not prove a fair superiority over a solo agent.
This supports independent review, preservation of candidates and regression
checks; it does not support unlimited repair or a score-maximizing loop.
[Anthropic, March 24](https://www.anthropic.com/research/harness-design-long-running-apps).

Boris Cherny's highly engaged X posts emphasize clear goals and task-specific
verification. Those posts show influential practitioner preference, not a
controlled productivity estimate. The community discussions include both
successful browser/test feedback and objections about cost and false green
results.
[April 16 post](https://x.com/bcherny/status/2044847858634064115);
[October 6 post](https://x.com/bcherny/status/2107565388250874193).

For Mirket, a successful export remains separate from correct intended motion,
and an image hash remains separate from visual inspection. A reviewer must
inspect saved pixels or decoded exports and report the states/frames reviewed.

### 5. Evaluate the host workflow, not only the tool

OpenAI/deepsense's harness-aware plugin evaluation separates direct tool tests,
a standalone agent loop and the actual product harness; results differ. Its
small synthetic dataset is not a universal ranking, but the methodological
point directly applies here. MCP smoke tests cannot establish whether native
Codex selects and uses a skill successfully.
[OpenAI/deepsense, September 23](https://developers.openai.com/cookbook/examples/partners/harness_aware_plugin_evals/harness_aware_plugin_evals).

Google's engineering account likewise separates behavioral assertions from
end-to-end tasks. LangChain's report shows that trace-driven harness changes
can alter benchmark performance with the same model, while its tuning and
reasoning/time settings limit causal interpretation.
[Google, September 9](https://developers.googleblog.com/en/the-anatomy-of-harness-engineering-how-to-evaluate-iterate-and-guard-ai-coding-agents/);
[LangChain, February 17](https://www.langchain.com/blog/improving-deep-agents-with-harness-engineering).

Use deterministic tests, disposable MCP journeys and actual Codex model turns
as separate evidence layers. Retain failed runs and denied approvals. Codex
CLI observations do not qualify the waived Desktop GUI path.

### 6. Delegate according to task structure, with explicit integration

The multi-agent scaling study evaluates 260 configurations across six
benchmarks and reports large differences by task structure, including gains
for decomposable work and regressions for sequential work. It does not support
mandatory delegation. Practitioner discussions similarly value isolation and
independent review while objecting to coordination overhead.
[Scaling study, v3 April 8](https://arxiv.org/abs/2512.08296);
[Pi community discussion, October 23](https://www.reddit.com/r/PiCodingAgent/comments/1xd8aks/what_i_learned_while_building_my_own_ai_harness/).

Preserve Codex-owned spawning. A capable lead owns creative direction and
integration; use a comparable reviewer when independent judgment helps.
Mechanical parallel work needs disjoint ownership. Small sequential work needs
no delegate. One task-level review/repair allowance follows the task across
delegates and specimens. Mirket guidance cannot hard-enforce native spawning.

## Recommended Mirket workflow

This is a proposed application of the evidence to the accepted architecture,
not an externally proven recipe.

| Stage | Concrete output | Owner and completion evidence |
|---|---|---|
| Brief and art direction | Audience, intended effect, references, required states/frames, delivery format, constraints and budget | Codex lead; selected references and explicit acceptance criteria |
| Capability selection | Available scoped route, supported operation and selected source/profile | Codex selects; Mirket reports runtime/capability readiness; absence remains a concrete limit |
| Editable starting point | Small valid project-native source, selected assets/fonts, explicit caption contract where needed | Codex/project toolchain; Mirket-owned operations only where qualified |
| Cheap checks | Bounded, actionable diagnostics before expensive work | Project toolchain or Mirket's existing validator, according to ownership |
| Implementation and preview | Working route/states, vector render or motion preview | Codex browser for UI/live review; Mirket's private Chrome only for video rendering |
| Independent review | Specific findings tied to pixels, behavior or a test oracle | Comparable creative reviewer when useful; otherwise appropriate independent checks |
| Bounded repair | Changed artifact plus regression comparison | Lead integrates; task allowance does not reset |
| Delivery | Saved editable source, actual export where requested, evidence and remaining defects | Observe saved artifacts; execution success, semantic quality and visual review remain distinct |

Do not require this entire ceremony for a trivial edit. The amount of planning
and review should match consequence and uncertainty.

For design, keep Hallmark and Taste as the chosen art-direction resources.
They provide direction and judgment prompts, not a substitute for examining
the actual result. Keep HyperFrames for the accepted video route. This research
does not reopen the renderer choice or assert new source-to-skill fidelity for
those projects. Remotion and SVGator remain workflow references under their
respective reuse conditions, not newly admitted runtime dependencies.

## Smallest next implementation, in order

### P1: Make the demonstrated video authoring failure actionable

**Evidence:** `adapters/src/video-export.ts:130-178` collapses different caption
errors to one code and stops static validation at the first error.
`src/process_video.rs:315-327` carries a short error string. The
[retained trial](motion-authoring-trials-20261027.md#repaired-source-runtime-oracle)
records four repaired model outputs refused with `invalid_caption_cues`, with
no MP4 produced. This demonstrates poor diagnostics and failed authoring; it
does not prove that richer diagnostics alone would make the models succeed.

**Proposed correction:** retain stable refusal codes and add bounded structured
static diagnostics: phase, selected relative path, cue index, field, reason and
expected range. Do not emit source bodies, captions or arbitrary stacks.
Aggregate independent format errors only after scope/revision checks succeed;
never inspect an unselected path to provide a nicer diagnostic. Define a stable
top-level precedence if several error categories coexist.

Reuse that validator for a cheap authoring check through a repository-owned
utility or the existing appropriate route. Select the least additional API
surface during implementation; do not duplicate rules or add a generic tool.
Provide a minimal valid seed from the current contract, including explicit
font selection and, for fixtures that intentionally select captions, the exact
caption keys. It is an example, not a new scene language.

**Acceptance:** a malformed root and two independently malformed cues yield
bounded precise findings; no browser/encoder starts, no destination is written
and source bytes remain unchanged. Test bounds, malformed text and scope/stale
source refusal. Valid controls still export: MP4/WebM/GIF controls exercise
burned-caption pixels, while `png_sequence_tar` also exercises SRT/WebVTT
sidecars. Then measure first-pass acceptance in new held-out authoring tasks.

### P2: Make qualification input selection explicit

**Evidence:** `tests/design-adoption-video-runtime.mjs:39-53` selects JSON assets
but always sets `captions_path: null`; `tests/motion-authoring-runtime.mjs:67-79`
selects a present `captions.json`. These can legitimately represent different
tasks, but their results cannot be treated as equivalent caption coverage.

**Proposed correction:** use an explicit test fixture field for selected
captions versus metadata-only captions, and a shared test-only request builder
where the runtime contract is identical. Preserve each scenario's differences.
Hash exact selected input bytes. Do not automatically scan production projects
or change intentional omissions into selected captions.

**Acceptance:** the same frozen fixture builds the same relevant request;
selected malformed captions fail; intentionally unselected captions are
recorded as untested. This corrects evidence ambiguity, not renderer behavior.

### P2: Qualify existing design review against retained candidates and exports

**Evidence:** external visual-loop experience reports regressions; local video
trials contain successful operation/control evidence alongside failed
model-authored exports. A single “verified” label would conflate these facts.

**Proposed work:** first test the existing review packet and evidence checker;
add fields or tests only for demonstrated gaps. Coverage should include frozen
brief and references, artifact revision,
reviewed states/frames, functional findings, visual findings, repairs, regressions
and remaining defects. Keep the initial and best passing candidate. A reviewer
sees the task and artifact before the author's assessment. Blind candidate order
where practical. Lead adjudicates disagreement with cited observations.

Keep structural gates separate from aesthetics: a visually attractive result
does not compensate for inaccessible controls, broken interactions or a wrong
export. UI checks cover representative routes/viewports/states and reduced
motion; vectors need rendered inspection plus editability; video needs decoded
samples and temporal/audio/caption checks appropriate to the brief. Sampling
does not prove every frame or device.

**Acceptance:** the checker rejects malformed or unsupported verification
records, while an independent reviewer/oracle challenges a false claim against
the artifact. The checker validates structure and reported attestations; it
cannot certify that pixels were seen or that an attestation is true. Detect an
intentional post-repair regression, preserve an earlier passing candidate, and
enforce the recorded task budget in the evaluation runner. Do not claim hard
enforcement of native Codex behavior or infer missing inspection solely from
the absence of image events in CLI logs.

### P3: Improve evidence readability only where consumers need it

`src/process_video.rs:278-313` accepts the adapter's evidence object, while the
server separately checks result identity and bytes; `video_result` also observes
the saved artifact. There is no reproduced unsafe-publication defect here.
If packets are copied outside their receipts, a server-owned binding to the
operation, source/profile and artifact would make interpretation clearer.
Add it only with a concrete consumer; never promote a leaf
`visual_verified: true` claim to an independent verification.

Similarly, owner-loss status could carry a concise recovery explanation, but
existing durable scoped receipts and non-replay behavior should remain. Resume
by inspecting effects and sources, not blindly rerunning a lost operation.

## How to prove the next changes help

The following is a proposed experiment, not completed acceptance:

1. Freeze six new authoring tasks spanning product motion, vector animation,
   captions and a refusal case. Include malformed inputs and valid controls.
   Keep held-out tasks distinct from diagnostic development fixtures.
2. Compare current behavior with the proposed diagnostics/seed bundle, using
   two repeats per task and condition: 24 total runs. This is a small diagnostic
   pilot, not a statistically decisive study. It measures the bundle; isolate
   diagnostics versus seeds in a later ablation only if attribution is needed.
3. Match Codex model/reasoning, tool availability, platform, permissions,
   task input and per-run resource allowance. Predeclare a finite total budget.
   Do not borrow a new delegate/specimen to reset an individual allowance.
4. Record first-pass valid output, final accepted output, scope violations,
   refusal accuracy, repair turns, tokens where available, elapsed time and
   human review time. A denied host operation remains blocked, not successful.
5. Independently validate output files, captions and specified motion. Score
   visual quality only for artifacts actually inspected. Preserve initial,
   intermediate and final artifacts, findings and remaining defects.
6. If the technical pilot helps, run a separate matched visual-quality subset
   with a no-skill baseline and blind comparable review. Report disagreement
   and regressions, not just the preferred candidate's score.

The old four-repair authoring trial remains exhausted and failed at caption
preflight. A separately frozen experiment must retain that history and must
not relabel old specimens as successful. Catalog `planned-not-run` acceptance
stays unchanged until its own required evidence exists.

For implementation, run the build-input guard before Rust compilation, then
the affected locked/offline Rust, Clippy, formatting and locked adapter gates,
and MCP smoke with explicit private executables. Use disposable homes/projects.
Actual model trials follow those gates. Windows and the waived Desktop GUI
check remain unqualified until their distinct checks are performed.

## What to borrow, and what to avoid importing

| Reference | Useful technique | Mirket decision |
|---|---|---|
| Pi / mini-SWE-agent | Minimal core, inspectable trajectories, simple comparison baseline | Learn from simplicity; do not replace Codex or add a second agent loop |
| Superpowers | Explicit task contracts, independent review, disciplined verification | Adapt narrowly where current skills lack a proven behavior; do not install every workflow |
| GSD Core | Durable phase artifacts and resumable context | Use project-owned task/evidence files; avoid a parallel scheduler |
| GitHub Spec Kit | Trace intent through plan, implementation and acceptance | Apply for substantial ambiguous work, not every small edit |
| Deep Agents | Trace-based evaluation and selective context experiments | Borrow evaluation methods; host owns agents and context internals |
| agent-browser | Observable browser actions and explicit postconditions | Lessons for Codex browser QA; no second general Mirket browser |
| Better Harness | Identify a concrete gap, make a small change, measure later tasks | Use as a research checklist, not an automatic rewrite or a maturity score |
| Harness² / HarnessPro | Experimental search over harness designs | Watch; no self-modifying production harness based on proxy scores |

Keep scoped SQLite/FTS plus lazy MiniLM unless a matched correctness/performance
test establishes a better alternative. Memory benchmarks do not justify a
database migration by themselves. Do not copy prompt-cache controls, tool
masking, spawning middleware or approval bypasses from systems that own their
entire model loop; Mirket does not own those controls.

## Work performed and remaining limits

This change adds research documents only. It does not change skills, provenance,
catalog hashes, runtime behavior, dependencies or the live installation; no
new reference admission or release is implied. No new implementation defect is
claimed fixed by this document. Research documents were checked for local-link
integrity and whitespace. Runtime gates and model evaluations were not rerun
for these documentation additions.

The practical next step is P1 plus P2 input selection, followed by the frozen
pilot. That offers a measurable route to better authoring without violating
the accepted Codex/Mirket boundary or replacing HyperFrames on speculation.
