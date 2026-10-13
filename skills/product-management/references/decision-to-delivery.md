# From evidence to a useful product slice

Read when a request is ambiguous, several options compete, or a feature must
cross product and engineering boundaries. This is a decision aid, not a required
document template. Keep a small decision in a few paragraphs; use the project's
existing issue or product document for a larger one.

## Start with the decision that could change

Record the affected actor and situation, current behavior, evidence and desired
outcome. Ask what finding would change the proposed action. Separate:

| Input | What it supports | What it does not establish |
| --- | --- | --- |
| Support tickets | Reported friction in those cases | Prevalence among all customers |
| Repeated observed failures | A reproducible workflow problem | That the proposed feature is the best fix |
| Interview statement | The participant's account and context | Purchase, adoption or representative demand |
| Usage events | Recorded behavior under the event contract | Intent, satisfaction or unrecorded users |
| Signed requirement | A commitment within its actual scope | Unlimited priority for unrelated additions |
| Stakeholder estimate | An assumption to compare or test | A measured benefit or delivery date |

Source counts should preserve their unit: reports, distinct accounts, users and
events answer different questions. Look for counterexamples and costs imposed on
other users. Do not contact people or publish a roadmap merely to complete a plan.

## Map unresolved discovery to its evidence

When the choice still depends on uncertain customer needs or competing solutions,
connect **outcome → evidenced opportunity → alternatives → assumptions → tests
and results** in the existing issue, table or diagram. This is optional reasoning
support, not another required artifact, interview cadence or approval step. Keep
accepted scope visible; do not manufacture an opportunity tree to rationalize a
predetermined solution or reopen a settled implementation request.

- **Outcome:** name the actor, context and observable change, with a baseline or
  an explicit measurement gap. A shipped feature is not itself a user outcome.
- **Opportunity:** describe a difficulty, need or desired progress in that actor's
  terms. Attach dated source locators, represented population and counterevidence.
  An interview quote supports that account; a feature request does not establish
  prevalence or prove the requested solution is necessary.
- **Alternatives:** link plausible approaches to the same opportunity, including
  repairing existing behavior or retaining it where credible. Keep important
  costs and accepted constraints visible. An alternative can serve several needs;
  a strict tree is unnecessary when a small linked table is clearer.
- **Assumptions:** state what could make each approach fail. Distinguish desire or
  adoption, usability, technical feasibility and business viability as useful
  prompts, without requiring a quota in every category. Record current support,
  contradiction and consequence if wrong; repeated citations of one source are
  not independent evidence.
- **Test and result:** choose a proportionate observation or experiment that can
  change the choice. Before running it, state what it can establish, relevant
  population, comparison and the result that would change the decision. Retain
  actual results, source artifacts, limits and supported next action. Mark an
  unrun test as proposed and an ambiguous result as inconclusive.

Select the next test using consequence, uncertainty, available evidence, exposure
and cost. A reversible working slice can itself be a useful test. Do not ban
building before every assumption is tested or assign universal durations and
thresholds. Use `customer-research` for new customer evidence and
`experiment-design` when the decision needs a causal comparison. Clicks can
indicate interest under the observed conditions; they do not demonstrate adoption,
and checkout initiation is not payment. One successful prototype task cannot prove
demand, sustained use or viable operating cost.

## Worked example: the request is broader than the evidence

All names and numbers here are illustrative. A team requests a bulk importer after
ten support tickets. Inspection finds three accounts behind the tickets; eight
reports concern duplicate rows and two concern unsupported file formats. The
product has 800 active accounts, but only 40 used the existing import workflow
last month. Neither 10/800 nor 3/800 is a valid import failure rate: tickets and
accounts use different units, and non-importing accounts had no such exposure.

The evidence supports investigating import reliability for participating accounts.
It does not yet support demand for every file format or a fully general import
platform. A stated outcome could be “operators can import supported records and
understand rejected rows without creating duplicate records.” A target rate needs
a valid baseline before it becomes a business commitment.

Compare a repair to the current import, a preview-and-commit flow for the existing
format, and a general import platform. Suppose the project already has a stable
parser and a recoverable server job. Reusing those for preview and commit is a
plausible bounded option. If duplicate identity is not defined, the decision owner
must choose whether a matching row updates, skips or rejects; code cannot infer
that policy from the word “deduplicate.”

The first slice could support the existing format, give actionable row validation,
show the proposed changes, and commit one confirmed import with a recoverable
status. Defer new file formats and scheduled synchronization unless evidence or
prior commitments justify them. A prototype that does not commit data may resolve
preview usability while the duplicate policy is still open.

### A compact discovery map for that decision

Continue the fictional case above. The following locators and results are teaching
examples, not real customer evidence. In a real task, replace them with retrievable
sources and dates; do not carry these findings into a product plan as facts.

| Link | Evidence, uncertainty and decision |
| --- | --- |
| Outcome → opportunity | Operators should recover an interrupted supported-format import without duplicate records. Fictional ticket register `IMP-01…10`, reviewed October 1, separates eight duplicate reports from two format requests; replay `R-04` reproduces a duplicate after a lost response. This supports recovery work, not broad format demand. |
| Opportunity → alternatives | A: repair retry identity and status recovery; B: add preview and commit as well; C: build a general importer. All retain the supported format and chosen duplicate policy. A costs less scope but may leave ambiguous row validation; B adds a user step; C carries parser and maintenance work unsupported by the duplicate reports. |
| Alternative → feasibility assumption | A assumes one intended import can be identified across a lost response. Before probe `P-01`, define failure as any duplicate persisted row after retrying the same intended operation. Its fictional before/after record comparison finds no duplicates for the tested supported fixture after the repair. This supports that recovery path, not every concurrency case. |
| Alternative → usability assumption | B assumes a preview resolves row confusion that recovery alone cannot. In fictional task observation `U-02`, an operator recovers status but cannot explain a rejected row. Keep this counterexample to A's sufficiency. It supports clearer row feedback; it does not yet distinguish inline validation from a separate preview. |
| Result → next decision | Retain the demonstrated recovery repair. Compare inline row feedback with preview only if that unresolved interaction warrants it. Defer the general importer; the two format requests remain recorded and could revive format work with supported obligations or broader evidence. |

This map can narrow the original preview proposal: recovery and row feedback may
meet the outcome without an extra confirmation screen. A passing technical probe
does not erase the usability counterexample, and the observation does not establish
how common that problem is. Keep this distinction when handing off the slice.

## Turn the slice into observable acceptance

Write scenarios around user and data outcomes, not class names or checkbox labels.
For the example, adapt the following to the policy the project actually chooses:

| Scenario | Required result | Useful evidence |
| --- | --- | --- |
| Supported valid rows | Preview accurately describes the proposed changes | Source rows reconciled to preview counts and sampled values |
| Invalid row | The row and reason are identifiable without losing input | Rendered failure state and corrected retry |
| Duplicate business key | The explicitly chosen update/skip/reject policy applies | Before/after records and documented policy |
| Confirmation request repeats | One intended operation is applied | Duplicate-request check against persisted state |
| Commit succeeds but response is lost | User can recover truthful status | Interrupted response followed by status lookup |
| User lacks write access | No records change; next action is clear | Permission check and unchanged data |

Decide whether the operation is atomic or permits partial success, and how the
preview remains valid if source data changes before confirmation. These are
product-visible semantics with engineering consequences. Do not quietly select
them as convenient implementation details.

Acceptance establishes that the implementation meets the contract. It does not
establish adoption or the business outcome. Include visual inspection only for the
changed interface states; backend evidence remains necessary for data claims.

## Make measurement answer the same question

A possible metric is the fraction of eligible import attempts that finish with
the intended records and no unresolved validation errors within a stated window.
Define “attempt,” “eligible,” completion and the window before comparing periods.
Use a stable import identifier so retries do not create extra attempts. Keep
attempt-level completion separate from account-level adoption. Reconcile event
counts with recorded job states on a sample before trusting the dashboard.

Choose guardrails from plausible harms: wrong updates, duplicate creation,
support burden, time to usable data or runtime cost. An event saying “success”
cannot prove imported records are correct. If detection is incomplete, state that
measurement limit and retain a direct integrity check.

For prioritization, distinguish a policy constraint from an uncertain estimate.
If two options exchange rank under reasonable effort or adoption estimates, the
ranking is fragile. Prefer a small test or reversible slice that resolves the
uncertainty instead of reporting a precise score as certainty.

End with the supported decision, open decisions and owner, next slice, acceptance
evidence and follow-up trigger. Do not mark the outcome achieved at code completion.

## Technique provenance

The optional discovery map adapts
[continuous-discovery](https://github.com/wondelai/skills/blob/c172996495bed0fcd26896a9416b2093fd7073f0/continuous-discovery/SKILL.md)
and [assumption-mapping](https://github.com/wondelai/skills/blob/c172996495bed0fcd26896a9416b2093fd7073f0/continuous-discovery/references/assumption-mapping.md)
from Wondelai at `c172996495bed0fcd26896a9416b2093fd7073f0` (MIT,
©2025 Wondel.ai sp. z o.o.; notice accompanies installed guidance). The upstream
synthesis attributes discovery methods to Teresa Torres. This adaptation does not
reproduce the underlying book or establish its empirical claims. Fixed cadence,
scores, assumption counts and illustrative demand thresholds are omitted. The
import case and its probes are original fictional teaching material.
