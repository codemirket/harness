# Experiment and recurring-loop contract

Use the requested artifact/layout; the path below is a default when none is specified. Embed the relevant records and share evidence links in a combined deliverable instead of creating extra sidecars.

Keep a project-local `product/experiment-contract.json` with the decision,
hypothesis, eligible population, exposure unit, primary outcome, guardrails,
assignment, baseline, stop rule, rollback owner, review date, and approval
state. For recurring reviews, also record cadence, the signal that triggers the
run, input digest, intended action ids, idempotency keys, and whether an action
is staged or executed.

Cadence follows signal speed; a slow product decision does not need a daily
loop. A repeated review of the same input must produce a no-op or staged
finding, never a second copy of an intended external action. External sends,
publication, spend, or production rollout remain human-approved. Keep null and
negative results and do not reset the review budget by creating a fresh
specimen or delegate.

Repository qualification tooling verifies arithmetic, predeclared structure,
approval gates, and duplicate prevention. It cannot prove statistical power,
user value, or a successful launch. See the loop mechanics in [Corey Haines's marketing-loops
skill](https://raw.githubusercontent.com/coreyhaines31/marketingskills/main/skills/marketing-loops/SKILL.md).
