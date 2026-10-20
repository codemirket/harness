---
name: skill-catalog
description: Select specialist guidance and compose professional teams from task boundaries; inspect role methods, assign complementary expertise and register needed project skills. Skip trivial edits and reuse unchanged selections.
---

# Mirket expertise

Use Mirket to discover and deliver relevant expertise. A role supplies
professional judgment, not authority over accounts, spending, tools, deployment
or model execution. User choices, project instructions and host permissions
remain authoritative.

## Entry and selection

Handle routine, single-session work directly. Use a durable Mirket task only
when continuity, recovery, coordination or explicit user/project tracking
warrants it; reuse a sufficient host or project record and do not create a
parallel tracker. Compose a team only when useful independent work would
materially help or the user/project explicitly requests it. A team plan is
read-only and starts no agents.

For a Mirket-owned worker, use `capability_plan`'s task workflows to declare only
the actual deliverables as `task_needs` with concrete reasons. The router selects
task-specialist guidance and scoped tools, loads the selected embedded bodies,
and enforces canonical production and verification. A role override never expands
tools. Select support skills and reference files only when they help the task;
see [the worker request contract](../agent-coordination/references/model-routing.md).
For native host tools, use these workflow choices as planning guidance under the
host's permissions; Mirket does not intercept native tool calls.

Select a lead from the deliverable and add support only for concrete affected
boundaries. An explicitly chosen skill ID can be read directly without a role
plan. Use `mirket capabilities facets` and `capabilities team` for
substantive multidisciplinary work, or `capabilities plan` for a focused role:

```sh
mirket capabilities team "Frontend Engineer" \
  --objective "Add a recoverable checkout flow" \
  --facet "web-interface=The checkout screen and browser state change" \
  --facet "service-contract=The order endpoint must reconcile retries"
mirket capabilities plan "CFO" --with "Excel Expert"
mirket catalog search accessibility --limit 3
```

Facets and explicit `--with` names produce bounded, deterministic assignments;
repeated roles merge their contributions. Up to eight facets and thirteen
distinct roles form one plan; broader work needs bounded phases. Do not invent a
role from a skill name. `support` entries are optional skill IDs: read and add
one only when its boundary is real. Profiles group installations and are not a
request to load every member.

Load the lead first and retrieve a reference when the next decision needs it.
Keep project facts in owning files, current work in one record when tracking is
warranted, and source captures in retrievable evidence. Read selected guidance
only; more context does not resolve conflicting sources, so state which source
wins.

For unfamiliar work, choose the narrow reference that resolves the next
decision: use [task routing](references/task-routing.md) when ownership or
boundaries are unclear, [task execution](references/task-execution.md) when
independent roles or durable coordination would help, and the affected
[delivery standards](references/delivery-standards.md) for substantial-work
verification. These references explain composition, tool boundaries and
evidence; they do not add a mandatory workflow to routine work.

## One default workflow, explicit alternatives

Use Mirket's authored methods by default. Upstream whole-workflow alternatives
(Matt, Superpowers and Addy) are opt-in: search their source name or exact skill
ID when requested. Their specialist techniques remain discoverable normally.
Do not stack competing planning, TDD or execution suites on the same assignment.

`mirket catalog profiles` lists eight starter bundles. Use
`mirket catalog profiles --all` for specialist and alternative bundles. Existing
profile IDs and saved selections keep their original membership; discovery
simplification does not reinstall, remove or substitute project skills.

## Discover without loading everything

```sh
mirket catalog search database
mirket catalog search motion --limit 10
mirket catalog show database-systems
mirket catalog profiles --all
mirket catalog read database-systems --path references/postgresql.md
```

Search is bounded and paginated with `--offset` and `--limit`. Query relevance
comes first: exact IDs, positive names, role aliases and tags precede weaker
matches. Delivery is a final tie-breaker only; catalog ranking does not inspect
whether a skill is installed in a home, project or host. Ranking never selects
a task automatically. Search reports each entry's read route.
`catalog_show` provides pins, adaptation, dependencies and caveats;
`skill_read` returns embedded guidance only. Upstream entries include pinned
source and license information: read adaptation, companions, runtime needs and
caveats before explicit selection. `mirket catalog fetch ID` verifies a package;
it does not execute scripts, install dependencies or authenticate tools. After
explicit upstream selection and project sync/doctor, read the installed host
skill. Reference entries are read manually from their pinned source under host
permissions.

## Register project expertise

Inspect the actual project, its instructions, `.mirket/project.json`, current
changes, stack and available tools. `project init` records targets without
selecting project skills; add `project-foundation` explicitly for an engineering
or release boundary, then add a specialist for actual current work or an
established later stage. When hosts have no global installation, choose the
explicit `portable-foundation` profile.

```sh
mirket project init --project /absolute/project --target all
mirket project add --project /absolute/project --profile project-foundation
mirket project add --project /absolute/project --capability "Backend Engineer"
mirket project plan --project /absolute/project
mirket project sync --project /absolute/project
mirket project doctor --project /absolute/project
```

`init` registers the canonical root. For tracking without installation use
`mirket project register`. Check project listings before deciding a root is
unregistered and follow pagination. Only explicit CLI registration admits a root;
coordination MCP cannot register or expand roots, and registration grants no
additional host permissions. Managed-copy conflicts require inspecting
the changed file; never delete edits to silence a check. Project commands and
native tools remain under host permissions. After renaming the original registered
directory, use `mirket project relocate PROJECT_ID --expected-root OLD --to NEW --yes`.
This preserves task IDs and memory/design namespaces; it does not move files or
rewrite historical worker bindings. A copied replacement is rejected.

## Track and verify inspectable evidence

When durable tracking is warranted, start one task from the selected metadata:

```sh
mirket task start --project /absolute/project --capability "Backend Engineer" \
  --objective "Implement and exercise an idempotent endpoint" --key endpoint-start
mirket task status TASK_ID
```

Record selected guidance delivery, current revisions and idempotency keys.
Retry an uncertain mutation with the same key and identical input; a changed
intention gets a new key. Checkpoint meaningful decisions and blockers rather
than every tool call. Bind project-relative acceptance and failure-probe
artifacts before completion. When task criteria are overridden, the CLI requires
both `--acceptance` and `--failure-probe`; preserve the requested outcome and
leave unavailable checks unverified. Stale revisions, missing delivery,
escaped paths or changed evidence bytes stop completion.

These records capture caller observations. Hashes establish file identity, not
semantic correctness, reviewer identity or human approval. Verify the result
through the project's tests, native tools, rendered artifacts or observed
system effects. If delegating, use actual host agents with bounded briefs,
disjoint ownership and integrated verification; record worker IDs and
contributions only when the chosen task record needs them.

## Check the environment

`mirket setup` guides setup, `mirket doctor` checks installation and registered
executables, and `mirket update` refreshes the CLI. Registration, installation,
startup, invocation and useful output are separate facts. Use native artifact
and browser capabilities when available. Additional executables can be
explicitly registered through `mirket tool`; Mirket pins file bytes and detects
drift but does not approve dependencies, create a sandbox or grant account
access.

After a harness update, call `runtime_status` in the actual client. Compare its
compiled source/catalog fingerprints and `installed_binary.matches_process`;
`mirket info` checks a separate CLI process. A successful call observes only that
connection. Direct design success responses carry `runtime` in ordinary structured
and text content, as well as `mirket/runtime` metadata, for the running container's
modules. Use ordinary content when the client hides metadata. Forwarded Penpot
service identity is separate. Do not infer activation from a version string or setup receipt.
