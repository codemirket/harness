# Workflows and MCP

Mirket supplies focused expertise and consistent task state. The host chooses
and executes actual tools under its permissions. All operations are available
through the `mirket` CLI; the host starts its MCP endpoint with `mirket mcp serve`.

## Expertise before execution

For a substantive task spanning disciplines, inspect `mirket capabilities facets`
or MCP `team_facets`. Interpret the user outcome and affected sources, then use
`mirket capabilities team ROLE --objective OUTCOME --facet 'ID=REASON'` or MCP
`team_plan` to assign related professionals. Each assignment contains a distinctive
professional method, task-specific contribution, prerequisites and checks. The
host dispatches independent agents and integrates their work; Mirket starts no
models. See [professional teams](roles.md#professional-perspectives-and-team-assignments).

Use `mirket capabilities plan ROLE` for exact single-role lookup, or `mirket catalog
search WORDS` for bounded discovery. That plan selects one lead and only explicitly
requested capability leads. CLI `--with` and MCP `capability_plan.supporting`
compose up to 12 capability names or role aliases. The contract's `support`
IDs are conditional skills; read one with `skill_read` or `mirket catalog read ID` and add it to
the project with `mirket project add --skill ID` only when needed.
For tracked work, start the task from this metadata before the first body read,
then make that read task-bound. Short untracked work reads directly once. Reuse
unchanged guidance and retrieve references only when needed. A skill name or
installation receipt is not evidence of use.

Save a team roster before starting tracked work. `team_plan.task_start` supplies
objective, skill IDs and criteria for MCP `task_start`; add the registered project
ID and operation key. For CLI use `task_start_cli.args` as an argument array and
append `--project` and `--key`; avoid concatenating user text into a shell command.
The returned arguments include `--skill agent-coordination` for the coordinator.
Task creation records those skills and criteria. Bind the separately saved roster
as evidence, then checkpoint real agent IDs, ownership, dependencies and contribution
paths. A retry reuses the same mutation inputs; a changed roster is a new planning
decision and must be recorded. `task status` recovers task/evidence pointers;
it does not regenerate a team or claim assignments have executed.

Search includes explicit guidance access routes. MCP `catalog_show` returns
selected source pins, adaptation, dependencies and caveats without fetching.
`skill_read` reads embedded content only. Explicitly selected upstream guidance
is read from its installed host path after project sync and doctor; reference
entries require manual source inspection under host permissions.

## Track a concrete outcome

Register a project explicitly, then start a capability-scoped task:

```sh
mirket project register --project /absolute/project
mirket task start --project /absolute/project --capability "CFO" \
  --with "Excel Expert" --objective "Reconcile the cash forecast and downside" \
  --key cash-forecast-start
mirket task list --project /absolute/project
mirket task status TASK_ID
```

Use the returned task ID and current revision in subsequent operations. Each
operation needs a unique idempotency key; repeat exactly the same key and input
after an uncertain response. A different intention gets a different key.

The capability supplies default acceptance and failure-probe criteria. When the
requested outcome has a different scope, set both explicitly at task creation:

```sh
mirket task start --project /absolute/project --capability "Accessibility Engineer" \
  --objective "Review and repair the supplied dialog source; no browser is available" \
  --acceptance "Preserve the dialog purpose and text; deliver corrected source and separate observed defects from unverified runtime behavior" \
  --failure-probe "A zero-violations scanner summary must not conceal the source defects or become a keyboard or screen-reader claim" \
  --key dialog-source-review
```

Both options are required together. Criteria describe the user's actual requested
outcome; an unavailable check stays unverified. Do not narrow an already required
outcome merely to mark failed or unfinished work complete.

Task lists use `--limit N` and the returned `next_cursor` as `--cursor TOKEN`.
Tasks are ordered by immutable creation time and ID, so a checkpoint does not
move a task across pages. A cursor belongs to one project. Traversal is not a
frozen snapshot of tasks created later; start a fresh list to discover new work.
Catalog search and project listing use their own offset pagination.

```sh
mirket task read TASK_ID financial-analysis --revision REVISION --key read-finance
mirket task read TASK_ID spreadsheet-analysis --revision REVISION --key read-sheet
mirket task checkpoint TASK_ID --revision REVISION --key cash-basis \
  --note "Receipts and payments use cash dates; unpaid invoices stay separate."
mirket task evidence TASK_ID reports/cash.xlsx --kind acceptance \
  --revision REVISION --key cash-output --summary "Reconciled opening and closing cash."
mirket task evidence TASK_ID reports/downside.md --kind failure-probe \
  --revision REVISION --key cash-downside --summary "Delayed receipts exposed the funding shortfall."
mirket task complete TASK_ID --revision REVISION --key cash-complete
```

`REVISION` is a placeholder for the latest numeric revision; each successful
mutation returns the next one. Evidence paths are relative to the registered
root and must refer to existing regular files. Checksums are recorded and checked
again at completion. Modified evidence, missing selected skill delivery and stale
revisions stop completion. `task reopen` records why work must resume.

These records capture the caller's observations. The server does not run a
spreadsheet calculation, test suite or semantic review. The host must execute
those checks and state their actual results. Hashes establish file identity;
local editable records cannot attest reviewer identity or human approval.

## MCP tools

| Tool | Operation |
| --- | --- |
| `catalog_search` | Bounded reviewed-catalog discovery |
| `catalog_show` | Selected metadata, pins, caveats and correct read route |
| `capability_plan` | Exact role selection and outcome contract |
| `team_facets` | Task boundaries and complementary professional assignments |
| `team_plan` | Bounded roster, professional briefs and task creation inputs; host dispatches workers |
| `skill_read` | Embedded skill/reference content; optional task delivery receipt |
| `project_list` | Paginated roots already registered through the CLI |
| `task_start` | Persist a bounded objective, selected skills and acceptance criteria |
| `task_status` | Inspect a task or recover IDs from a project |
| `task_checkpoint` | Record a meaningful decision at an expected revision |
| `task_evidence` | Bind a project-relative artifact and observed result |
| `task_finish` | Complete or reopen a task after consistency checks |

Resources expose embedded skill content. They do not provide arbitrary filesystem
reads. Tool annotations communicate behavior; they are not permission enforcement.

The stdio boundary limits request frames to 128 KiB and structured results to
64 KiB. Four blocking operations can run concurrently; additional work receives
a busy response. Tool deadlines are ten seconds. A short database operation may
finish after cancellation or a timeout: inspect task state or retry the identical
idempotency key before issuing another mutation.

`mirket doctor` checks the installed server through a real local protocol client.
`mirket mcp doctor --iterations 50` exercises the current executable and reports
startup and request measurements. These checks prove local protocol behavior;
use a fresh Codex or Claude session to observe native discovery and useful calls.
