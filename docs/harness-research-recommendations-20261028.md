# Research-led recommendations for the harness

Research date: 2026-10-28. Repository baseline: `7da476b55873e1b104bde2944c9ac63b0d299273`.
Status: recommendations approved by the user on 2026-10-28. Implementation and
qualification are tracked in [the approved follow-up plan](../planning/research-improvements.md);
approval itself is not outcome evidence.
Source window: 2025-04-28 through 2026-10-28; current repository/documentation
observations are distinguished from dated publications.

## Decision

Keep Codex Desktop as the agent host and Mirket as the narrow capability layer.
The next investment should be better research procedure, stronger independent
oracles, reusable project context and fewer gaps between existing evidence
records. There is no demonstrated reason to install another agent framework,
research crawler, marketing service or video engine.

This is not a claim of worldwide consensus or a universal best architecture.
The defensible pattern is to measure final task outcomes, simplify where
coordination adds no value, and retain only interventions that help on new
tasks under comparable budgets.

## Research and evidence quality

Four independent substantive lanes covered QA/engineering, product/marketing,
parallel work and art direction, plus a lead source-audit lane. The lead researched search/research quality,
inspected source, challenged duplicate recommendations and checked selected
social discussions in the authenticated browser. The companion
[source register](harness-research-sources-20261028.json) retains source-level
dates, inspected material, access limits, relevance and evidence class.

Discovery included GitHub implementations/issues, Reddit discussions, X,
LinkedIn, Hacker News, practitioner publications, official documentation and
research papers. Examples of following less familiar leads:

- A Reddit implementation discussion led to `evrenesat/asky`; source inspection
  exposed useful acquisition/retrieval separation and original-chunk provenance,
  but also illustrated why a README description is insufficient qualification.
- A Reddit maintenance survey led to fresh GitHub checks: LangChain's
  `open_deep_research` is currently archived, while LearningCircuit's project
  is active. May's discussion was not treated as October's repository state.
- An X discussion led to Anthropic's October 22 report on simplifying
  subagents. The replies included a counterargument to adding more agents,
  rather than just enthusiasm for harnesses.
- Hamel Husain's LinkedIn post and comments exposed concrete evaluator failures:
  lost retrieval context, an extractor returning an empty result, and a judge
  returning constant PASS. These are practitioner reports, not reproduced
  experiments, but they suggest discriminating local tests.

The X post by Gaurav Sen showed 906 likes and 1,083 bookmarks when inspected;
the linked article was dated October 23 and updated October 26. This establishes
attention, not effectiveness. Its primary citations were followed where relevant.
The authenticated LinkedIn page and comments were read; other lane-specific X
entries were only indexed or mirrored and remain explicitly weaker evidence.
No messages, reactions, follows or account changes were made.

The sample is purposive, mostly English-language and subject to publication,
commercial and ranking bias. We did not reproduce external benchmarks, run a
new model cohort, inspect new creative output or measure customer outcomes.
Source counts are an audit aid, not a depth or quality metric. Out-of-window
leads and non-authoritative mirrors are retained as excluded discovery records.

## What is already implemented

Several attractive external patterns are already present. Reimplementing them
would increase maintenance without demonstrating improvement.

| Existing behavior | Current source | Implication |
| --- | --- | --- |
| Frozen brief/rubric/source manifests, shared repair budget, candidate lineage, scoped artifacts and finding dispositions | `scripts/check-design-evidence.py:312`, `:500`, `:613` | Extend this packet; do not invent another design evidence system. |
| Preview/export beat pairing and explicit reported-only visual attestations | `scripts/check-design-evidence.py:273`, `:646`, `:703` | Record integrity is intentionally separate from media measurement and visual judgment. |
| Immutable operation finalization and artifact revision/effect reconciliation | `src/operation_store.rs:593`; `src/engine.rs:870` | Consume these receipts read-only; do not replace their lifecycle. |
| Frozen model trials, runtime probes, decoded samples and negative controls | `tests/matched_design_study.py`; `tests/motion-authoring-runtime.mjs`; `tests/motion-feedback-pilot.py` | Reuse the existing mechanisms; improve coverage and evidence joins. |
| Disjoint native ownership, capable creative lead/reviewer, one repair budget, completed-turn and interruption caveats | `skills/agent-coordination-strategy/SKILL.md`; `tests/codex-coordination-trials.mjs:67` | These are not missing policies. Test their generalization; host lifecycle remains outside Mirket. |
| Source-aware research, truthful marketing, product alternatives and experiment stop rules | `skills/evidence-research`, `browser-research`, `customer-research`, `positioning`, `marketing-copy`, `experiments-launch` | Add reusable project artifacts and decision methods rather than more global prose. |

The retained native trial reported a completed parent turn despite missing
mechanical integration files; the lead itself reported the missing work
honestly. Another interrupted turn left a command writing later. The local
outcome checker already rejects these cases. Do not recast them as a new
unfixed Mirket orchestrator defect. See
[native qualification](native-coordination-qualification-20261028.md).

## Prioritized improvements

### 1. Give research an explicit search-and-evidence procedure — P0

**Gap:** Current skills correctly require primary sources and counterevidence,
but do not provide a reusable coverage map or claim-to-source artifact. A
plausible answer can still emerge from a narrow famous-site shortlist.

**Proposal:** Add an on-demand reference to `browser-research`; keep the global
`evidence-research` skill short. For consequential research, record:

1. The decision, date window, subquestions and what evidence would change it.
2. Query families: implementation, primary measurements, practitioner use,
   failure/criticism, alternatives, current maintenance and reuse rights.
3. Query-to-source links; follow citations and issue discussions; deliberately
   inspect unfamiliar implementations when relevant. Deduplicate syndicated
   articles and shared upstream claims.
4. Per claim: supporting source and locator, contradictory evidence, publication
   and access dates, inspected revision, and observed/reported/inferred/unknown.
5. An evidence-gap list. An inaccessible page is unavailable, not negative
   evidence. Stop when decision-relevant gaps are resolved or explicitly
   bounded; do not optimize for a page quota.

Use Codex's search, GitHub connector and ordinary browser. Mirket's private
Chrome remains video-only. A second search provider is worth evaluating only
after held-out questions show material recall gaps in the host routes.

**Acceptance:** Six frozen niche questions with hidden must-find facts,
counterexamples, duplicate stories, stale versions and one inaccessible source.
Compare current versus proposed procedure at equal time/model budgets. Measure
supported claim coverage, missed decisive counterevidence, stale claims, citation
precision and cost. Require fewer unsupported conclusions without padding.

**Basis:** [DeepResearch Bench](https://arxiv.org/html/2506.11763v1) separates report
quality from citation support; [ReportBench](https://arxiv.org/html/2508.15804v1)
also checks uncited statements. Their academic tasks and model judges do not
establish a general product-research winner.

### 2. Verify conclusions against evidence, including the evidence checker — P0

**Gap:** Valid citations can support an outdated or selectively framed claim.
An internally consistent packet can also contain an incorrect reviewer verdict.

**Proposal:** Add a small held-out verifier calibration set: known defect,
unchanged control, plausible false finding, stale source, swapped attribution
and a compound claim whose source supports only half. A consequential reviewer
finding must name the artifact/region/claim and an observable failure. Permit
abstention. Retain disagreements and their resolution instead of voting them away.

Promote only reproduced failures into regression cases. Give every proposed
harness change a predicted effect, baseline, held-out check and rollback/delete
condition. Never let the same agent quietly rewrite the oracle after failing it.

**Acceptance:** The checker must reject planted defects and accept controls;
report false-positive and false-negative counts separately. A constant PASS
judge must fail calibration. Repaired artifacts must pass prior obligations too.

**Basis:** [DeepFact](https://aclanthology.org/2026.acl-long.1586/) distinguishes
citation support from broader factuality and preserves auditable challenges to
labels. [Rethinking Harness Evolution](https://arxiv.org/html/2607.12227v4)
finds that gains can disappear under held-out, matched-budget evaluation. Its
positive game results are conditional, not a reason to reject all iteration.

### 3. Connect existing design packets to actual execution evidence — P0

**Gap:** The generic packet, runtime receipt and specialized trial identity can
each be valid but disconnected.

**Proposal:** Extend the existing packet with references to: (a) the final
Mirket operation receipt when a Mirket operation produced the artifact, (b)
the independent media probe/decoder output for media claims, and (c) the frozen
execution identity already retained by specialized pilots. Compare operation,
release, project and exported artifact revisions; bind decoded captures to the
export and decode invocation. A PNG digest is not the video's artifact revision.

Keep Codex-native output possible without inventing a Mirket receipt. Missing
binding means that specific provenance claim is unqualified. Preserve existing
record-only attestations; a new field cannot authenticate pixels or provider-side
model execution. This is an evidence relationship, not a compatibility layer,
runtime rewrite or new tracing database.

Extend the same packet's reference/asset rows with creator, license, attribution
and permitted use when external assets are involved. Existing hashes establish
identity, not reuse rights. A host-provided image-view observation may improve
traceability if the host actually exposes one; do not manufacture that evidence
or infer absent inspection from missing CLI events.

**Acceptance:** Reject a valid packet paired with another operation/project,
changed export bytes, failed/unknown effect, contradictory probe dimensions/FPS,
and silently mixed skill/model revisions. Valid metadata must still never claim
visual quality. Reuse `tests/motion-authoring-runtime.mjs:163` for probe evidence.

### 4. Test skill selection and useful outcomes separately — P0

**Gap:** Catalog validation and named-skill prompts do not establish correct
activation or benefit. Representative trials exist; catalog-wide acceptance
remains `planned-not-run`.

**Proposal:** Extend current trial fixtures with natural requests that omit the
skill name, adjacent requests that should not select it, and unavailable/stale/
wrong-project routes. Separate selection, successful execution, output quality
and improvement over baseline. Use current tests and runners; do not import
another eval framework merely to obtain a dashboard.

**Acceptance:** Start with research, product/marketing, visual/motion and one
engineering workflow. Hold out briefs, preserve model/effort and inherited-skill
identity, inspect actual artifacts, and retain failures. Do not aggregate a
selection success with a failed deliverable into an overall PASS.

**Basis:** [Coder Eval](https://github.com/UiPath/coder_eval) is a useful
skill-evaluation implementation reference; the
[Reddit discussion](https://www.reddit.com/r/ClaudeCode/comments/1wlkcbx/skills_are_the_new_code_evals_discussion/)
provides practical activation-test criticism. Neither proves Mirket skill uplift.

### 5. Make motion intent executable around HyperFrames — P1

**Gap:** Named beat correspondence and decoded output exist. The remaining
increment is a reusable oracle for intended spatial/temporal relationships,
especially errors that leave endpoints unchanged.

**Proposal:** Use stable subject IDs and small project assertions: object A is
visible before B; title remains inside its safe area during its readable hold;
transition overlap stays within a declared range; final frame settles; one owner
controls each animated property. Sample beat neighborhoods and risky midpoints,
not just first/last frames. Keep the static composition gate and independent
normal-speed playback review. Preserve the accepted direction, Hallmark/Taste
references and the same task-level repair budget.

Stable IDs and CSS/WAAPI ownership checks already exist in the preview tooling;
reuse them. The new work is semantic checking of the rendered HyperFrames/Anime
motion, not another ownership checklist.

**Acceptance:** Plant a clipped descender, midpoint jump with matching endpoints,
unregistered Anime subject, wrong time units and a property collision. Detect
each without rejecting intentional motion. Compare false alarms and repair time.

**Basis:** [MoVer](https://github.com/jama1017/MoVer) supplies a concrete
spatiotemporal verification approach and dataset; its SIGGRAPH publication is
within the source window although its first preprint is older. Borrow the
verification idea, not its GSAP renderer or model pipeline. HyperFrames remains
the sole video engine. [motiscope](https://github.com/KumarSashank/motiscope)
is a smaller experimental reference for separating measured timing from visual
interpretation, not a qualified dependency.

### 6. Reuse region-aware visual and interaction checks — P1

**Gap:** Existing specimen-specific checks do not constitute a reusable
cross-project visual acceptance suite. A video may pass while the live CTA is
occluded, as the retained Daybreak case demonstrated.

**Proposal:** Parameterize the useful existing checks around named regions,
font/media readiness, viewport/DPR/theme/locale, DOM hit targets, line wrapping,
focus, reduced motion and saved pixels. Pair image differences with geometry
and style evidence. Masks need a reason and review; an unhealthy capture cannot
be accepted as a baseline. For vectors inspect silhouette, crop and readability
at the actual target size, not merely valid XML.

**Acceptance:** An 8px regional shift, missing font, clipped descender, hidden
focus target, mobile overflow and clean control. Require defect localization;
do not use a global similarity or model “polish” score as the release gate.

**Basis:** [Playwright](https://playwright.dev/docs/test-snapshots) documents
environment-sensitive captures; [Storybook](https://storybook.js.org/docs/writing-tests)
provides component-state testing. Use these in projects that already have them.
[TestivAI](https://github.com/testivai/testivai-oss) and
[PixelJury](https://github.com/gchahal1982/pixeljury) offer useful report shapes,
but their small adoption signals do not justify adding their runtimes.

### 7. Give product work a durable evidence-to-decision map — P1

**Gap:** Product/customer skills specify sound decisions, but their outputs do
not share an initiative-level map across research, prototype and delivery.

**Proposal:** Maintain a project artifact connecting desired outcome → observed
customer opportunity → candidate solution → riskiest assumption → test → result
→ decision. Mark real observations, synthetic fixtures and inference distinctly.
Use small valuable slices; an interview quote is not a prevalence estimate and
a synthetic persona is not demand evidence. Retain rejected hypotheses.

**Acceptance:** Given conflicting interview excerpts and a large feature request,
choose a supported opportunity and the cheapest informative test. The artifact
must preserve the contradiction and avoid inventing a customer consensus.

**Basis:** [Teresa Torres, May 21, 2025](https://www.producttalk.org/ship-value-weekly/)
connects opportunity decomposition and small delivery slices. Her cadence is
aspirational and constrained by deployment/regulation. The small
[product-pipeline implementation](https://github.com/lenar-amirov/product-pipeline-public)
illustrates persistent hypothesis state; it is not evidence of broad adoption.

### 8. Establish shared marketing context and diagnose before writing — P1

**Gap:** Existing positioning, copy, conversion and experiments skills lack a
common versioned context that carries actual customer language and approved proof.

**Proposal:** Add a project-local context template read by these existing skills:
audience/use case, actual alternatives, differentiated value, objections,
switching friction, supported claims, voice examples and dated learning. Keep
today's product positioning separate from future market hypotheses. Before a
growth task, identify the weak funnel stage and tracking validity; improve the
stage supported by data rather than defaulting to landing-page copy.

Use source-linked claim checks on the saved copy. A recurring marketing review
may have a trigger, state, idempotency key and stop rule, but Codex owns any
authorized scheduling and publication. No default outreach or ad-spend agent.

**Acceptance:** Feed strong acquisition/weak activation data, an invented
testimonial, an expired metric and a fashionable competitor absent from buyer
shortlists. Expect an activation experiment, blocked unsupported claims and
positioning against actual alternatives. A repeated review must not duplicate
an external action. Offline checks cannot establish conversion lift.

**Basis:** [Corey Haines's product-marketing skill](https://github.com/coreyhaines31/marketingskills/blob/main/skills/product-marketing/SKILL.md)
demonstrates shared context; [April Dunford, May 2026](https://aprildunford.substack.com/p/positioning-in-the-age-of-ai)
distinguishes current competition from roadmap threats;
[Krithika Shankarraman's May 2025 interview](https://www.lennysnewsletter.com/p/growth-tactics-from-openai-and-stripes-first-marketer)
offers diagnosis before tactic selection. Adapt selected methods in original
prose; do not bulk-import the collection or its integrations.

### 9. Evaluate parallelism against a strong single-lead baseline — P1

**Gap:** Disjoint ownership and lead integration are already policy. What is
unqualified is when native fan-out improves verified delivery in this host.

**Proposal:** Keep independent research/review parallel; keep tightly dependent
implementation with one lead unless interfaces are stable. Serialize shared
setup, bound fan-out by integration capacity, and retain requested versus
observed model/effort. Reuse the existing output oracle and host job controls.
After interruption, do not reassign overlapping work until the owning host's
process/job state is reconciled. Mirket cannot hard-enforce Codex spawns or kill
host commands by adding guidance.

**Acceptance:** A small serial/2-worker comparison measuring verified completion,
elapsed time, tokens and integration/reviewer effort, plus existing missing-file,
overlap, denial and delayed-write failures. Preserve explicit user model choices
and comparable creative review; do not spend additional trials without a new
bounded protocol.

**Basis:** [Anthropic, October 22](https://claude.com/resources/articles/building-agents-when-to-fold-subagents-into-your-main-agent-loop)
reports benefits from consolidating tightly coupled specialists, while retaining
independent research and verification. This is vendor field evidence, not a
controlled universal result. Our own retained failures are more directly relevant.

### 10. Make maintenance and context growth earn their cost — P2

**Gap:** More skills, reference files and traces can slow the host and make
obsolete guidance harder to identify even when each file looks reasonable.

**Proposal:** Track why each nontrivial instruction/check exists, its responsible
owner, triggering failure and re-evaluation condition. Test removing or narrowing
unused guidance after model/runtime changes. Keep only a small route map globally;
load specialized references on demand. Maintain current product docs separately
from dated qualification history. Retain bounded/redacted trace references, not
duplicated unbounded transcripts or automatic promotion of web claims to memory.

**Acceptance:** A task must still find its relevant resource after cleanup;
measure token/latency and task outcomes before/after. Expired product/source
claims must be flagged, while historical failed evidence remains discoverable.
No automatic rewrite of architecture or memory based solely on model feedback.

**Basis:** [OpenAI's harness account](https://openai.com/index/harness-engineering/)
supports repository legibility and mechanical constraints. The recent
simplification evidence above supports measuring the cost of context and roles.

## Tool and resource disposition

| Resource | Useful part | Recommended disposition |
| --- | --- | --- |
| Hallmark, Taste, HyperFrames | Existing direction references and selected video production | Keep existing pinned adoption; no renderer replacement or bulk skill import. |
| MoVer (Apache-2.0 top-level) | Spatial/temporal predicates and planted motion cases | Reference-only prototype over existing HyperFrames outputs. Full dependency closure is not admitted. |
| Playwright / Storybook | Stable captures, interaction and component states | Use the project's existing installation where available; no new bundled browser/testing stack. |
| Corey Haines marketing skills (MIT top-level) | Shared context, bottleneck diagnosis, bounded recurring loops | Adapt a few methods into existing project skills; preserve attribution if code/prose is reused. |
| DeepResearch Bench / ReportBench / DeepFact | Coverage, support checking, uncited claims, challengeable labels | Evaluation-method references. Dataset/code rights must be checked separately before redistribution. |
| LearningCircuit/local-deep-research (MIT top-level) | Current research implementation, strategy and citation handling | Study as a comparator. Its fact-finding benchmark is not long-report quality proof; do not embed its agent/search stack. |
| asky | Session-scoped research evidence and replay/live test separation | Architecture/code reference only; package metadata declares MIT but that is not a completed redistribution review. |
| Coder Eval / Harbor / Inspect / LoopsBench | External outcome grading and long-horizon evaluation shapes | Borrow test patterns first; external qualification tools only if current runners cannot answer a measured question. |
| Langfuse / OTel conventions | Trace relationships and failure diagnosis | Reuse a few field names only if needed; no new service, exporter or telemetry dependency. |
| Archived LangChain Open Deep Research | Historical implementation | Do not choose it as a new maintained dependency merely because it has many stars. |

Top-level licenses are screening information, not admission of transitive code,
models, fonts, assets, browser binaries or services. This pass imports none.
No new production dependency is recommended for immediate admission.

## Finite implementation sequence

1. **Research and evaluator pilot:** improve the existing project research
   resource, add claim/coverage fixtures and calibrate reviewers. This directly
   addresses the user's shallow-search complaint without expanding Mirket scope.
2. **Evidence joins and reusable motion checks:** extend the current packet and
   existing probes; test planted failures before any new creative generation.
3. **Product/marketing artifacts:** shared context, opportunity/assumption map,
   current/future positioning and bottleneck-first experiment briefs.
4. **Held-out qualification:** compare current/proposed behavior and a matched
   baseline across representative domains; test serial versus useful parallel
   work. Promote only the individual cases supported by actual evidence.

Stop an intervention if it adds cost without improving supported outcomes, hides
a failed case, needs a new owner boundary, or duplicates an existing mechanism.
Do not grow the nine global skills or reset the accepted D6 creative budget.
Windows, the waived Desktop GUI check and the proposed macOS floor retain their
existing unqualified status. This research does not change release readiness.

## Deliverables and verification

Only this report and the companion source register were added. No source,
catalog, dependency, installation or runtime configuration was changed. The
register is parsed and deduplicated by URL; relative report links and whitespace
are checked. Rust/adapter/MCP gates were not rerun for research-only documents.
There is no new model-quality, visual-quality or conversion result in this pass.
