# Bounded external reviews

Use the current host's subagents first. An external Claude Code review is useful
when a specific independent perspective can resolve a consequential uncertainty
and the user's provider/model choices permit it. The initiating agent retains
implementation and integration ownership. Do not delegate recursively.

Supply the question, allowed paths, relevant instructions, observed evidence and
expected finding format. For example: inspect a named retry implementation and
its tests for a reproducible duplicate write; return the triggering input and
file/line evidence without editing, executing or contacting external services.

Prefer the host's provider integration when available. If a separate CLI is the
only route to the needed review, use the [shared missing-tool route](../../skill-catalog/references/task-execution.md#establish-the-execution-boundary)
to inspect it and its documented controls. Optional registration can pin the
executable when that improves reproducibility; invocation remains subject to the
host's bounded controls. Preserve the user's provider, model, billing route,
authority and permissions.

Keep the selected provider, model, billing route, authentication and permissions
unchanged. Do not extract credentials, bypass host approvals or retry quota errors
through a different account. Restrict the review using actual host/provider
controls; an instruction saying “read-only” is not enforcement. Avoid passing
secrets or unrelated files in the prompt. Missing access is an explicit limit.

Verify each returned finding against the actual source and behavior. A finished
process or confident verdict is not acceptance. Record actionable evidence and
uncertainty, then apply only changes warranted by the user's task.
