# Turn an observed failure into a useful control

Use this when a harness change addresses a repeated or consequential failure.
Keep the control in the project that owns the behavior unless the same boundary
is genuinely shared. A longer instruction file is not the default repair.

Record the concrete trigger, expected outcome, actual defect and owning layer.
Choose the smallest useful pairing:

| Failure | Guidance before the action | Feedback after the action |
| --- | --- | --- |
| Retry duplicates a committed command | Define commit, key scope and receipt semantics | Independent balance/receipt checks after lost response and replay |
| Visual polish hides broken keyboard behavior | Preserve semantic controls and focus ownership | Real Tab/Escape/focus-return exercise, plus inspected renders |
| Faster query changes the returned rows | State result identity and workload before tuning | Independently computed results and comparable timing/write/storage data |
| Good-looking business brief uses the wrong denominator | Define grain and population from source contracts | Reconcile raw inputs, perturb segment mix, review the actual conclusion |
| Installed skills are reported as demonstrated expertise | Label registration, execution and acceptance separately | Source integrity, installation checks, then task artifacts and reviewer basis |

Implement deterministic feedback in the existing test, linter, CLI or build when
it can observe the invariant. Demonstrate that it rejects a representative broken
case and accepts the correction. Do not reward a test that only repeats the
implementation, checks fashionable words or succeeds after its fixture is weakened.
Prefer small focused regressions to a new generic runner or mandatory hooks.

Where correctness depends on meaning, composition or source applicability, inspect
the actual artifact against explicit criteria. Keep reviewer kind and limitations
visible. Use an independent reviewer when fresh eyes can uncover material gaps;
do not replace a missing human acceptance decision with an agent's score.

Run inexpensive affected checks while editing. Keep expensive integration,
rendering and review checks at the appropriate delivery boundary. Re-run when their
relevant inputs change. Fix the owning source, repeat the failing probe, and retain
the smallest regression that prevents recurrence. A control that never catches
the intended defect needs repair; adding more guidance does not compensate.

For this harness, `capabilities check` checks source coverage, `project doctor`
checks installed copies, `runtime doctor` probes prerequisites, and `eval` plus
the artifact workbench exercise specific results. None substitutes for the other.
Use existing project checks for project architecture and domain behavior; this
portable repository cannot impose one application's fitness functions globally.

## Design rationale

Birgitta Böckeler's [Harness engineering for coding agent users](https://martinfowler.com/articles/harness-engineering.html)
(2026-04-02, read 2026-10-09) distinguishes preventive guidance from feedback,
deterministic checks from model judgment, and maintainability from architecture
and functional behavior. It recommends improving controls from observed failures
and placing checks according to cost and timing. It also cautions that passing
agent-written tests does not settle whether the requested behavior is right.
The table and repository mapping above are this harness's application of those
ideas, not claims of results established by that article.
