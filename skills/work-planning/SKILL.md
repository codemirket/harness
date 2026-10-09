---
name: work-planning
description: Plan and resume substantial work with dependent steps, uncertain scope, or a long execution window. Preserve enough task state to continue accurately after an interruption. Skip routine edits and short questions.
---

# Work planning

Choose planning effort from the cost of losing track, not the number of tool
calls. Use the project's existing task record or the host's plan when it is
sufficient. Do not create parallel records for the same work.

For substantial work with interdependent outcomes or subjective acceptance, read
[delivery review](references/delivery-review.md). It connects the completion
conditions to a working slice, independent inspection and evidence-based repair;
routine edits need no additional review process.

## Establish a useful plan

Describe the intended result and observable completion conditions. Separate
user requirements from assumptions, unresolved choices, and implementation
options. Identify dependencies that constrain the order of work.

Break substantial work into outcomes that can be checked independently. Make
the next action concrete enough to begin; leave later uncertain steps coarse
until investigation resolves them. Include integration and verification in
the plan rather than treating implementation as completion.

Investigate the uncertainty most likely to invalidate the approach early.
Prefer a bounded experiment or representative example before scaling a costly
operation. Update the plan when evidence changes it; record the reason for a
material change so a later session does not revive the abandoned approach.

## Preserve only useful state

Use a durable note when interruptions, context limits, or handoffs could lose
important decisions. Reuse the project's convention. If none exists, choose
one task-specific note in an appropriate workspace location and state its
path; avoid scattering planning files through the repository root.

Keep enough information to answer:

- What result and constraints still govern this task?
- What is completed, with which evidence and relevant revision or inputs?
- What remains uncertain, blocked, or dependent on another result?
- Which decisions and unsuccessful approaches affect the next action?
- Where are the supporting files, sources, and unfinished artifacts?
- What should happen next, and who owns it if work is shared?

Store conclusions and pointers, not a transcript of every tool call. Separate
quoted external material from task instructions. Exclude secrets and avoid
copying unrelated conversation or private account history into the note.
Update at meaningful changes of state and before a planned handoff; a fixed
write-after-every-few-actions rule usually adds noise.

## Resume and coordinate

Confirm the task identity and workspace before reading its record. If several
plans exist, use the assigned task's record; modification time is not evidence
that a different plan belongs to the current task.

Compare the saved state with current files, relevant changes, and available
results. A saved statement that a test passed is useful only while its inputs
remain applicable. Reconcile stale or contradictory notes before relying on
them; preserve valid completed work rather than restarting by default.

If agents are already collaborating, assign one owner to the shared plan and
give workers separate files or clearly bounded updates. A plan does not grant
permission to delegate, install hooks, schedule future work, or access local
session history. Follow the task's actual authorization and available tools.

## Finish against the outcome

Check the requested result and remaining dependencies before marking complete.
Checkboxes document a decision; they do not prove that the product works.
Record the useful verification and any genuine limitation, then deliver the
result. Retain or archive the task note according to project conventions;
do not delete user records or extend the task solely to keep a plan active.

For substantial artifact delivery, use [artifact review](references/artifact-review.md)
to inspect the result independently of the author's claims.
