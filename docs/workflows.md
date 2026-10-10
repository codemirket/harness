# Workflows and MCP

Mirket supplies focused expertise and consistent task state. The host chooses
and executes actual tools under its permissions. All operations are available
through the `mirket` CLI; the host starts its MCP endpoint with `mirket mcp serve`.

## Expertise before execution

Use `mirket capabilities plan ROLE` for exact role/ID lookup, or `mirket catalog
search WORDS` for bounded discovery. A plan selects one lead and only explicitly
requested support. Read the selected skill body and relevant references before
substantive work. A skill name or installation receipt is not evidence of use.

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
| `capability_plan` | Exact role selection and outcome contract |
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
