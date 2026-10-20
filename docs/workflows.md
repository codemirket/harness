# Workflows and MCP

> **Version scope:** This guide includes the verified desktop build 1.0.11. The public download remains 1.0.5 as of 2026-10-20. Check [version availability](release-status.md) before using newer commands.

Mirket supplies focused expertise and consistent task state. The host chooses
and executes actual tools under its permissions. All operations are available
through the `mirket` CLI; the host starts its MCP endpoint with `mirket mcp serve`.

## Expertise before execution

Routine work is direct. For substantive multidisciplinary work or explicit
collaboration, use [professional roles](roles.md) and the
[task-execution reference](../skills/skill-catalog/references/task-execution.md):
select one lead and add only affected support. `team_facets`/`team_plan` are
read-only; dispatch useful independent assignments through the
[Mirket-owned routed spawner](../skills/agent-coordination/references/model-routing.md). `capabilities plan ROLE` handles focused lookup and
`catalog search WORDS` handles bounded discovery. `--with` and
`capability_plan.supporting` compose explicit capability names; contract
`support` IDs are conditional skills to read and add only when needed.

Use durable tracking only when continuity, recovery, coordination or explicit
user/project tracking requires it. Reuse a sufficient existing host/project
record and do not create a parallel tracker. For tracked work, task-bound skill
delivery, revisions and idempotency keys are evidence; a skill name or
installation receipt is not. `catalog_show` returns pins, adaptations and
caveats; `skill_read` reads embedded content. After explicit upstream selection,
project sync and doctor, read upstream guidance from its installed host path;
reference entries are read manually from their pinned source under host
permissions.

## Track a concrete outcome

`project init` records target hosts without selecting project skills. Add
`project-foundation` explicitly for an engineering or release boundary, then add
specialties justified by the project. When hosts have no shared installation,
choose the explicit `portable-foundation` profile to copy shared guidance into
the project.

When durable tracking is warranted, register the project if needed, then start a
capability-scoped task:

```sh
mirket project register --project /absolute/project
mirket task start --project /absolute/project --capability "CFO" \
  --with "Excel Expert" --objective "Reconcile the cash forecast and downside" \
  --key cash-forecast-start
mirket task list --project /absolute/project
mirket task status TASK_ID
```

New task writes return compact `receipt_version: 1` replies with task ID, revision,
status and an operation-specific `result` (start includes `result.skills`). Use
`task status` for complete task criteria, evidence and recent history. Exact
retries of old keys can still return their originally recorded full snapshot.
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
| `runtime_status` | Live MCP process, compiled source/catalog fingerprints and installed-binary match (newer desktop build) |
| `catalog_search` | Bounded reviewed-catalog discovery |
| `catalog_show` | Selected metadata, pins, caveats and correct read route |
| `capability_plan` | Exact role selection and outcome contract |
| `team_facets` | Task boundaries and complementary professional assignments |
| `model_registry` | Dynamic target/model capabilities with TTL refresh; no inference |
| `route_task` | Task/tool capability floors and eligible models; no inference |
| `spawn_subagent` | Mirket-owned model and tool execution with verification gates |
| `team_plan` | Bounded roster, professional briefs and task creation inputs; spawn_subagent dispatches Mirket workers |
| `skill_read` | Embedded skill/reference content; optional task delivery receipt |
| `project_list` | Paginated roots already registered through the CLI |
| `task_start` | Persist a bounded objective, selected skills and acceptance criteria |
| `task_status` | Inspect a task or recover IDs from a project |
| `task_checkpoint` | Record a meaningful decision at an expected revision |
| `task_evidence` | Bind a project-relative artifact and observed result |
| `task_finish` | Complete or reopen a task after consistency checks |

The newer desktop build also exposes these operations through the same main MCP
connection. Memory and durable worker operations require `agent-runtime` and a
registered project; backup and relocation remain explicit CLI operations.

| Tool | Operation |
| --- | --- |
| `memory_read` | Read an explicit fact, list a page, or search active project facts |
| `memory_put` | Store an explicit fact with provenance and an idempotency key; optionally supersede an observed fact |
| `memory_forget` | Delete an observed fact and its history using its digest; traces/backups remain |
| `workflow_status` | Inspect the latest durable worker state and recovery limits |
| `resume_subagent` | Resume eligible checkpointed work within its original bindings and budget |
| `workflow_forget` | Delete one inactive run's checkpoints using its observed checkpoint ID |
| `verification_handoff` | Begin a bound host-check attempt and submit named evidence |
| `runtime_data` | Export a cursor page or preview retention without deleting anything |
| `runtime_data_apply` | Apply an explicitly reviewed, unchanged retention plan |

Lifecycle MCP cursors are objects, not JSON strings. CLI `--cursor` takes JSON
text. A preview/export is not a restorable backup. See [memory, recovery and typed
contracts](agent-runtime.md) for request examples, retention and privacy boundaries.

Resources expose embedded skill content. They do not provide arbitrary filesystem
reads. Tool annotations communicate behavior; they are not permission enforcement.

The stdio boundary limits request frames to 128 KiB and structured results to
64 KiB. The newer desktop build separates control, execution and runtime
admission, with four slots in each class; the contained agent runtime has its
own cross-process limits. Ordinary state operations have a ten-second deadline;
agent-runtime responses have a 120-second deadline and workers retain their
assigned execution budgets. See [runtime containment](agent-runtime.md#ownership-and-containment).
The public 1.0.5 server uses one four-operation pool; inspect its tool description
for exceptions to the ordinary deadline. A short database operation may
finish after cancellation or a timeout: inspect task state or retry the identical
idempotency key before issuing another mutation.

`mirket doctor` checks the installed server through a real local protocol client.
`mirket mcp doctor --iterations 50` exercises the current executable and reports
startup and request measurements. These checks prove local protocol behavior;
use a fresh Codex or Claude session to observe native discovery and useful calls.
