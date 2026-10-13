# Recovering tasks and verifying specialist work

Mirket records task state and checks saved evidence. The host runs professionals,
checkers and commands. Task records help a later session recover decisions and
reject stale results; they do not authenticate workers or establish that a claim
is correct.

## Recover older checkpoints

`mirket task status TASK` includes the latest eight checkpoints,
`checkpoint_count` and `history_has_more`. When earlier notes are omitted, read
history explicitly:

```sh
mirket task history TASK --limit 10
mirket task history TASK --limit 10 --cursor CURSOR
```

Pass the returned `next_cursor` unchanged. A null cursor ends the listing.
History is newest first, with 1–20 entries per requested page and a 60 KiB
serialized response budget. A page can contain fewer entries to satisfy that
budget. Cursors belong to one task and resume strictly before the last returned
revision. Appends between reads do not shift older pages; start a new listing to
see newer checkpoints. History never truncates the stored note text.

## Evidence remains explicit

Ordinary evidence continues to bind one file's bytes to a caller's summary.
Choose a contract when the task needs stronger structure:

```sh
mirket task validate TASK verification.json --contract verification
mirket task validate TASK verification.json --contract verification --completion
mirket task evidence TASK verification.json --contract verification \
  --kind acceptance --summary 'Checked the declared inputs and result' \
  --revision REVISION --key UNIQUE_KEY
```

Validation is read-only. Without `--completion`, failed checks and unfinished
assignments can be recorded for diagnosis. Completion validation additionally
requires a passed verification result or accepted/waived contributions. The
normal task completion requirements still apply: current skill delivery and
both acceptance and failure-probe evidence. The MCP `task_evidence` contract
option and `task_validate` tool use these same validators.

A registered contract type is retained for that evidence path and evidence kind.
Re-recording it as ordinary evidence cannot disable the contract. Mirket does
not infer a type from arbitrary JSON or execute a checker described by a receipt.

## Verification receipt

A verification receipt binds the explicit inputs, checker and result to SHA-256
identities. For example, create `verification.json` with the following shape,
replacing `TASK` and each digest placeholder with the actual task ID and the
file's 64-character lowercase SHA-256:

```json
{
  "schema_version": 1,
  "task_id": "TASK",
  "inputs": [{"path": "src/example.rs", "sha256": "INPUT_SHA256"}],
  "checker": {"path": "tests/example.rs", "sha256": "CHECKER_SHA256"},
  "result": {"path": ".mirket/check-result.json", "sha256": "RESULT_SHA256"},
  "outcome": "passed",
  "runtime": "Record the actual platform and checker runtime here",
  "limitations": ["This check covers the declared scenario only"]
}
```

`outcome` is `passed`, `failed` or `error`. The last distinguishes a checker or
infrastructure failure from a checked task failure. `runtime` is optional;
`limitations` is required and may be empty. At least one input is required.

Registration validates the receipt and current file identities. Completion
rechecks its unchanged bytes and every declared input, checker and result.
Changing an input invalidates an unchanged report. Changing an unrelated file
does not. After rerunning an affected check, save a new receipt and re-record
its evidence using the current revision and a new idempotency key.

Include all files needed for the conclusion. There is no implicit Git revision,
dependency discovery, environment capture or semantic evaluation. A checker file
can bind a saved configuration that describes a command or runtime; Mirket does
not resolve or execute that description. A matching hash establishes file
identity, not that a test ran, its expectations were useful, or a human approved
its result.

## Specialist contribution receipt

Save the original `team_plan` output as a roster before dispatch. A contributions
receipt binds that roster and accounts for every role exactly once. Use actual
host worker identifiers, observed outputs and an explicit disposition after
integration. The first registered contribution receipt fixes its roster binding:
later updates for that evidence path and kind must retain the same roster path
and digest. The coordinator can own the primary role; record that accurately.

The following is a one-role shape example. The actual `assignments` must cover
all roles in the bound roster; use the original roster objective and select its
lead skills in the task. Replace task, worker and digest placeholders with real
values:

```json
{
  "schema_version": 1,
  "task_id": "TASK",
  "roster": {"path": ".mirket/team.json", "sha256": "ROSTER_SHA256"},
  "assignments": [
    {
      "role": "backend-engineering",
      "attempt": 1,
      "result_attempt": 1,
      "status": "completed",
      "host_agent_id": "ACTUAL_HOST_WORKER_ID",
      "inputs": [{"path": "src/example.rs", "sha256": "INPUT_SHA256"}],
      "evidence": [{"path": ".mirket/backend-result.json", "sha256": "RESULT_SHA256"}],
      "depends_on": [],
      "validation": "Describe the observed check and its result",
      "unverified_paths": [],
      "reason": null,
      "disposition": "accepted"
    }
  ],
  "limitations": ["Worker identities and dispositions are caller declarations"]
}
```

Record it with `--contract contributions`, and preview completion with
`mirket task validate TASK contributions.json --contract contributions --completion`.

Supported statuses are `planned`, `running`, `unknown`, `completed`, `reused`,
`failed`, `interrupted` and `waived`. They describe recorded observations, not
live worker liveness. `attempt` is positive. A supplied `result_attempt` must
match it. Mirket also retains each role's latest recorded attempt and worker
identity for the evidence record. An update cannot decrease an attempt, and
replacing a recorded worker requires increasing it. This rejects a late older
report even when its own `attempt` and `result_attempt` agree. Assigning the
first worker to a previously unassigned attempt is allowed.

Completed or reused contributions require a host worker ID, matching result
attempt, nonempty input and evidence bindings, and a validation account.
`disposition` is `pending`, `accepted`, `rejected` or `waived`; only completed or
reused work can be accepted. Waivers require a reason and `waived` disposition.
Completion permits accepted completed/reused assignments and explicit waivers.
Dependencies name roster roles and must form an acyclic graph; missing, repeated
or extra roles are rejected. Failed, interrupted and unresolved assignments stay
visible and prevent completion until resolved or explicitly waived.

Neither a worker ID nor an accepted disposition proves independent execution,
expertise, semantic correctness or human approval. Preserve the actual worker
outputs and integration judgment. The contract does not dispatch workers, poll
processes, rerun dependencies or recursively validate referenced contracts.

## Bounds and database compatibility

Contract JSON and the roster are limited to 128 KiB, with versioned strict fields.
Every bound path must be a normal project-relative path to a regular file inside
the registered root; traversal and symlinks are rejected. Each file is at most
16 MiB. There are at most 32 entries in a binding list, 64 bindings per contract,
and 64 MiB of unique bound files per contract validation. Completion also caps
the sum of bytes validated across contracts at 64 MiB (shared files count again
across contracts). Contribution rosters contain
1–13 roles. These limits bound work and output; they do not create a sandbox.

Opening a valid schema-1 or schema-2 task database migrates it transactionally to
schema 3. Schema 1 gains the contract table; both older versions replace the
outdated task-list index with creation-time/ID ordering. Existing projects, tasks,
evidence, checkpoints, deliveries, contracts and idempotency replies are retained.
Opening schema 3 validates it without rewriting it. Unknown or malformed schemas
are rejected. Older runtimes refuse schema 3; restart existing MCP connections
at cutover and keep a consistent pre-upgrade SQLite backup. A binary rollback
alone cannot downgrade the database. `--home` isolates development checks.

## Compact mutation replies

New successful task writes return `receipt_version: 1`, `operation`, `id`,
`project_id`, `revision`, `status`, `updated` and an operation-specific `result`.
For example, start returns selected `result.skills`; checkpoint returns the note's
revision, digest and byte count without repeating recent history. Record the
returned revision for the next mutation. Retrieve full criteria, deliveries,
evidence and recent notes with task status; retrieve older notes with history.
The CLI skill-read command pairs its compact `receipt` with the requested document.
MCP skill delivery likewise returns content plus its delivery receipt.

Exact idempotent retries preserve the original reply bytes, including old full
snapshots recorded before 3.3.0. Consumers must tolerate those legacy replies
when retrying old keys. A new key uses the versioned receipt. Snapshot size limits
still apply to the underlying task, and history and evidence checks are unchanged.

Applicable source notices remain part of complete skill payload integrity.
The 3.3.0 change from shared to scoped notices requires one fresh delivery of
selected guidance for older active tasks. Unrelated future notice changes do not
invalidate an unchanged owner's payload. Do not bypass a meaningful body or
reference change merely because a previous delivery succeeded.
