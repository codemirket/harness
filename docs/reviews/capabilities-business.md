# Business capability assurance — 2026-10-09

The authored guidance supports disciplined product, research, marketing and data
work. This review adds a specific company-level allocation workflow and exercises
two existing development cases. It does **not** establish broad business expertise,
conversion lift, live SEO performance or superiority over another harness.

## Scope and inspected sources

Read the complete skill bodies for product-management, data-analysis,
search-visibility, search-audit, marketing-writing, research-and-synthesis and
work-planning, plus their relevant product, data, editorial, claim-verification and
delivery-review references. Read catalog routing/delivery standards and the
ai-system-evaluation development workflow. This is source inspection of the local
authored skills, not execution of every upstream catalog entry. No provider,
account, dependency, external publication or fixture input was changed.

| Capability | Concrete guidance already present | Evidence and remaining limit |
| --- | --- | --- |
| SEO and AI search visibility | [search-audit](../../skills/search-audit/SKILL.md) traces responses, canonical/redirect/robots interactions and distinguishes indexing from incomplete public search; [search-visibility](../../skills/search-visibility/SKILL.md) separates crawl, training, search retrieval, citation and conversion, and raw/rendered evidence | Source-reviewed. No real site, crawler, Search Console, analytics or AI answer sample was exercised in this assignment. These skills do not supply authenticated tools or establish search outcomes. |
| Deep research | [research-and-synthesis](../../skills/research-and-synthesis/SKILL.md) requires claim decomposition, disconfirming queries, source origins, applicability and stopping conditions; [claim verification](../../skills/research-and-synthesis/references/claim-verification.md) handles version conflicts and observed behavior | One supplied-source version-conflict brief produced. It tests synthesis within four local sources, not internet search breadth, primary-source discovery, access failures, corrections or research saturation. |
| Planning | [work-planning](../../skills/work-planning/SKILL.md) preserves requirements, dependencies, unresolved choices and verification state; [delivery review](../../skills/work-planning/references/delivery-review.md) separates defects, preferences and unverified outcomes | Source-reviewed and used to define this bounded review. No separate interrupted-session/resume trial; a written plan does not prove durable continuation. |
| Product management | [product-management](../../skills/product-management/SKILL.md) connects evidence and decision authority to alternatives, coherent slices, acceptance, rollout and measurement; [worked reference](../../skills/product-management/references/decision-to-delivery.md) rejects ticket-count demand inference | The rollout brief supplies an actionable next product decision, but does not test stakeholder negotiation, live discovery, delivery or business outcomes. No mandatory PRD/interview ceremony is introduced. |
| Marketing writing | [marketing-writing](../../skills/marketing-writing/SKILL.md) ties message to audience, mechanism, proof, conditions and action; [editorial preservation](../../skills/marketing-writing/references/editorial-preservation.md) preserves evidence qualifiers and offer scope through cuts | Source-reviewed. No new channel-rendered campaign or independent copy trial here; the guidance cannot establish conversion improvement. Paid media, positioning and SEO are specialist routes, not automatically included in copy drafting. |
| CEO/business allocation | New [executive-strategy](../../skills/executive-strategy/SKILL.md) compares customer evidence, incremental economics, collection/payment timing, capacity, opportunity cost, concentration and reversibility; [worked allocation](../../skills/executive-strategy/references/allocation-decision.md) makes constraints and decision-changing thresholds explicit | Authored and arithmetic checked. It is a recommendation workflow, not delegated business authority. Existing catalog `gstack-openclaw-ceo-review` is a selected upstream plan/scope review with execution marked untested; its name alone does not demonstrate cash/capacity analysis. |
| Data understanding | [data-analysis](../../skills/data-analysis/SKILL.md) defines grain, denominators, units, time windows, join cardinality, missingness and source reconciliation; [worked reference](../../skills/data-analysis/references/grain-and-comparisons.md) shows fanout and mix reversal | Reproducible account-conversion calculation passed the existing exercise, including varied populations, duplicate conflicts and missing-stratum nulls. This does not test SQL engines, large datasets, monetary joins, statistical inference or causal identification. |

## Material gap addressed

Product scope guidance already considers operating costs and reversibility, but it
does not provide an explicit company-wide cash schedule, capacity feasibility or
capital-allocation method. Treating a product priority score as sufficient for
that decision can choose an opportunity the company cannot fund or deliver.

The new executive skill fills that narrow gap. Its fictional example distinguishes
proposed EUR 120,000 enterprise sales from only EUR 40,000 collected inside the
three-month horizon. The enterprise option reaches EUR 144,000 cash against an
assumed EUR 150,000 floor and needs 14 person-weeks against 10 available. The
recommendation weighs customer evidence rather than pretending the highest
forecast or cheapest option automatically wins. Downside balances and a EUR 46,000
collection feasibility threshold are recalculated in local evidence. These are
illustrative assumptions, not facts about the user or any real company.

Catalog identity, routing, payload hashes and installation are owned by the main
integration task. Creating these source files by itself does not prove registration,
fresh-session selection or successful invocation in either client.

## Executed development exercises

Runs live under ignored `build/capability-business/`; they are local evidence,
not shipped capability payloads. Both were prepared with the model label
`host model exact identifier unavailable`, condition `capability-assurance`, and
explicit current-host/no-provider-switch settings. Preparation records the source
fingerprint. The agent read relevant existing guidance and solved only the copied
workspace; task, verifier, rubric and immutable fixture sources were not changed.
Verifiers were executed after producing the outputs, not read as answer keys.
Rubrics were read afterward to prepare the independent review handoff.

```sh
python3 ai.py eval check --run build/capability-business/analysis
python3 ai.py eval check --run build/capability-business/research
```

**Analysis:** nine checks passed. Current conversion is 14/40 (35%); pilot is
21/40 (52.5%). Mobile declines from 25% to 12.5% and desktop from 75% to 62.5%.
At the current 80% mobile/20% desktop mix, pilot conversion is 22.5%, a descriptive
difference of -12.5 percentage points. The brief recommends a bounded stratified
randomized pilot rather than all-account expansion, without claiming observed
causal harm or inventing ROI/significance. Artifacts:
`analysis/workspace/analyze.py`, `results.json`, `decision.md` and `analysis/checks.json`.

**Research:** three structural checks passed. The brief applies the supplied 4.2.1
deployment contract, preserves the contrary stale-read observation without adopting
its unsupported diagnosis, and proposes a direct-origin/edge/client comparison
with synthetic organizations. It does not claim that this runtime test happened.
Artifacts: `research/workspace/research.md` and `research/checks.json`.

**Executive example:** all monthly cash balances, downside balances, floor and
collection thresholds were checked with standard-library Python. The reproducible
check is `executive_check.py`; output is `executive-arithmetic.json`. There is no
claim that the fictional customer demand or forecast is validated.

No evaluation review/acceptance was self-recorded. The parent reviewer must read
the actual artifacts against the copied rubrics and sources before recording an
agent review. Deterministic passing checks and arithmetic are narrower evidence
than accepted decision quality. These public cases are development exercises;
there was no baseline comparison, held-out trial or measured quality improvement.

## Concrete assurance gaps and next useful tests

These are coverage limits and targeted follow-ups, not instructions to install
more skills or tools:

1. **SEO lacks exercised behavior here.** Use a local fixture with conflicting
   canonical/redirect/noindex/robots states and a rendered-only content section.
   Require evidence per affected URL and preserve owner bot/training preferences.
   A live search outcome then needs separately authorized site and measurement
   access. A passing static audit must not become a ranking claim.
2. **Research checks cannot grade support.** The current verifier checks links,
   dates and document presence; a plausible but contradictory brief can still
   satisfy those mechanics. Retain source-by-source independent interpretation
   review. Add a later separate retrieval exercise with duplicated origins,
   unavailable sources and a correction; do not label the local cache case a
   deep-web research benchmark.
3. **Product/marketing/planning have no dedicated trial in this pass.** A useful
   product trial supplies repeated tickets from few accounts, competing authority
   and a constrained release. A copy trial gives measured versus self-reported
   proof and a limited trial offer, requiring a finished channel artifact without
   stronger claims. A planning trial interrupts work after a dependency changes,
   then checks that resumption preserves completed evidence and changed scope.
4. **Business forecasts need actual evidence.** The executive example checks
   reasoning and arithmetic only. Before applying it to a real allocation, inspect
   the actual cash schedule, payment terms, capacity and customer evidence. A
   useful future trial should make a late collection or scarce capacity reverse
   an apparently attractive recommendation, while retaining decision authority.
5. **Analysis coverage is deliberately narrow.** Retain this cohort/mix regression;
   complement it with an independently controlled monetary fanout/missingness
   case when strengthening business analytics. No additional library is needed
   for these small deterministic controls. Connected SQL/spreadsheet access must
   be verified in its actual environment before claiming live-data readiness.

No concrete contradiction requiring an edit to the inspected existing authored
skills was established. Prefer these representative output checks over lengthening
their already explicit evidence and authority rules. New guidance should follow
an observed failure or a demonstrably missing decision boundary.
