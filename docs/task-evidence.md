# Task evidence

Coordination tasks record an objective, selected guidance, decisions and acceptance
or failure-probe artifacts. The [task recovery and evidence contract guide](../skills/agent-coordination/references/task-evidence.md)
owns their formats and limits and ships with the agent-coordination skill.

## Worker verification handoff

The [newer desktop build](release-status.md) also binds worker results to host
verification. These are separate records: task completion checks coordination
evidence; a worker handoff records checks against that worker's actual inputs.

1. Inspect the worker trace and required check names.
2. Begin an attempt with source/configuration/test inputs and the command identity
   for every required check. Produced artifacts are bound automatically.
3. Execute those checks with host tools and save their evidence.
4. Submit all named results with the attempt ID and current trace digest.

A source change, added/deleted input, or changed evidence makes the verification
stale. Begin a new attempt after a repair. Earlier attempts remain inspectable.
Retry an interrupted handoff with the same arguments rather than inventing a new
successful result. See the [runtime handoff contract](agent-runtime.md#host-verification)
for bounds and examples.

Hashes bind bytes, not correctness, command execution or human approval. The host
must actually run the relevant application/tests and inspect the resulting output.
