# Claude Code as a Mirket routed worker

Use Claude only when the task warrants an independent assignment and the current
authenticated model registry exposes an eligible Claude target. Route through
`mirket models route --request FILE` before `mirket agent spawn --request FILE`,
or use the equivalent `route_task` and `spawn_subagent` MCP tools. The same
`SpawnRequest` states the objective, task intent, complexity, project and concrete
`task_needs`. The owned workflows select role/task guidance and scoped tools;
`assigned_tools: []` uses their defaults. An explicit subset cannot widen scope or
omit required production/verification. Optional role, support-skill and reference
choices never grant effects. Do not name a model in that request.
[Model routing](model-routing.md) describes the request and failure
behavior.

Mirket discovers Claude's current selectable model menu through the local
authenticated CLI. It selects a resolved model identity and a supported effort
from the task and tool floor; the source contains no published model-ID list.
The native Claude process receives no general-purpose built-in or ambient MCP
tools; its only reported tool is the CLI's synthetic `StructuredOutput` schema
mechanism. It returns one strict JSON action, and Mirket alone checks and
dispatches an assigned tool.
A role brief, prompt or tool name cannot grant an effect. The host's permissions
still govern the underlying tool and filesystem operations.

Give the worker a bounded question, relevant instructions and source material,
owned paths, allowed effects, dependencies and expected finding format. For a
review, ask for a concrete trigger and file/line evidence; for production, ask
for saved artifacts and checks. Preserve the user's provider, billing route and
authorization. Do not extract credentials, switch accounts to retry a quota
error, or delegate recursively. If discovery, capability evidence, exact model
identity or tool isolation cannot be verified, stop that candidate and report
the gap; do not silently lower the task floor.

The coordinator checks the returned evidence and owns integration. A completed
process or confident verdict is not acceptance. Inspect changed files and
effects before reusing an uncertain result or retrying a mutation. Claude's
[CLI reference](https://code.claude.com/docs/en/cli-reference) describes the
native tool and structured-output switches; Mirket's adapter verifies the
observed process contract for each routed action.
