# Local memory and durable workers

> **Version scope:** This guide includes the verified desktop build 1.0.11. The public download remains 1.0.5 as of 2026-10-20. Check [version availability](release-status.md) before using newer commands.

`agent-runtime` contains Mem0 OSS 2.2.1, LangGraph 1.2.16 and the SQLite
checkpointer 3.1.2. It is an internal package behind the existing Mirket MCP
connection. It does not install a third desktop server, run LangSmith, create a
Mem0 account, or select an inference provider.

```sh
mirket package install design agent-runtime
mirket package probe agent-runtime
mirket package test agent-runtime
mirket package audit agent-runtime
mirket package status
```

Package installation replaces the saved package selection; include `design` to
retain it. Setup/update own installation and preserve unrelated host settings.
The current image targets Linux ARM64 on Mirket's registered private Podman
engine. Serving never downloads dependencies or models and never starts the VM.

## Ownership and containment

| Owner | Responsibility |
| --- | --- |
| Mem0 | Explicit stable facts, local hybrid retrieval, vector records and history |
| LangGraph | Worker/effect/host-verification transitions and SQLite checkpoints |
| Mirket Rust | Project binding, provider/model selection, tool authorization, dispatch, artifact checks and budgets |
| Existing task SQLite | Project registration, coordination and acceptance evidence |
| Host | Permissions, check execution and interpretation of retrieved content |

Each on-demand process has no network, no provider credentials, read-only code
and models, two CPUs, 1536 MiB RAM, 128 PIDs and a bounded temporary filesystem.
Only its selected project's private data directory is mounted. Identity probes
and self-tests mount no installed memory. A registered project retains its original SHA-256 namespace across explicit directory
relocation. Existing registrations migrate without changing namespaces. Unregistered
CLI roots retain path-derived namespaces; a relocated project reserves its old
namespace so another directory at that path cannot inherit its memory. Unregistering
revokes MCP registration and deletes coordination history but retains runtime identity;
re-registering the same directory restores access to the same memory namespace. MCP requires a registered
project; CLI operations require an explicitly selected existing project.

Agent-runtime session startup is queued across desktop processes sharing the engine, with a
120-second queue deadline. The gate ends after the verified identity handshake;
established sessions remain concurrent. Cross-process admission additionally reserves two worker sessions and one short-operation session; long workers cannot consume the operation slot. The permit stays held through process cleanup. The MCP server separately bounds execution, runtime operations and inexpensive control requests, so a full worker pool does not block live identity. These bounds apply to this installation, not unrelated host processes. Project storage operations wait at most
60 seconds on their separate lock, leaving time for execution within the existing
120-second response deadline. Queue cancellation dispatches nothing, and runtime failures
never automatically replay memory or workflow mutations. Early EOF reports the
process exit status without exposing private runtime stderr.

For an already-selected runtime, a candidate CLI can run `mirket package prepare
agent-runtime` followed by `mirket package test agent-runtime` before `mirket update`.
Preparation uses the same pinned installer path without changing the installed
binary, desktop settings or package selection. Old and candidate image receipts
remain separate; stop active runtime sessions before preparing a new generation.

The complete Linux ARM64 Python wheel closure is URL/version/hash locked.
Mem0's default OpenAI extraction is replaced by a fixed adapter that rejects
inference. Dense embeddings use the pinned multilingual MiniLM ONNX model with
FastEmbed 0.9.0. BM25 assets, spaCy and its English model are installed ahead of
use. English preprocessing is not a claim of equal retrieval quality in every
language. Facts and queries over the dense model's 128-token window are rejected,
not silently truncated. Worker recall uses a bounded objective excerpt. `memory_recall` selects `best_effort` (default), `required`, or `off`. An empty store avoids loading embeddings; best-effort failures produce a visible unavailable status while LangGraph continues. Required recall propagates a failure. This choice changes recall only, not durable checkpoint requirements.

## Memory

`memory_read`, `memory_put`, `memory_forget` are host tools, not worker write
permissions. The CLI equivalents are `mirket memory read`, `put`, and `forget`.
Store concise stable facts only when useful within the user's scope. Include a
source/provenance statement, kind, unique idempotency key and optional Unix expiry.
No automatic transcript harvesting occurs. Source files, current instructions
and verification evidence remain authoritative over retrieved text.

Example request for `mirket memory put --request /absolute/path/fact.json`:

```json
{
  "project": "/absolute/path/project",
  "text": "The product supports keyboard navigation.",
  "kind": "constraint",
  "provenance": "User's accessibility requirement in the current task",
  "expires_at": null,
  "key": "accessibility-requirement-v1"
}
```

```sh
mirket memory read --project /absolute/path/project --query 'accessibility requirements'
mirket memory forget --project /absolute/path/project --id UUID --expected-sha256 OBSERVED_SHA256
```

Retries with the same key and input reconcile the actual Mem0 record. Reusing a
key for different content fails. Forget requires the observed content digest and
removes active vectors and Mem0 history; existing run traces, checkpoints and
backups remain. This is not forensic erasure. A query/list is bounded, not an
export of all memory. Expired facts are excluded from retrieval, not erased. Lists return `next_cursor`
and scan at most 256 candidates per call; follow a cursor even when the page is empty.
The CLI accepts `memory read --cursor UUID`. Search still filters a bounded candidate
set after ranking; dense expired/superseded neighbors can reduce returned recall.

To replace a decision explicitly, add `supersedes: {"id":"OBSERVED_UUID",
"sha256":"OBSERVED_DIGEST"}` to a new keyed put request. The old fact remains in
history/export but is excluded from active reads after publication. An interrupted
replacement stays hidden until an identical retry reconciles it; competing
replacements fail. Forgetting a replacement does not resurrect the old decision.

## Worker recovery

When selected, `agent-runtime` is used by actual `spawn_subagent`/`agent spawn`
runs. Failure of that runtime does not silently switch to ephemeral execution.
Installations without the package remain explicitly ephemeral and cannot resume.
Memory recall is labeled untrusted context in the worker prompt. Retrieved facts
are sent to the already-selected inference provider as task context; local memory
storage does not imply local inference. LangGraph saves
the task contract and serialized worker state, including verification obligations,
retained image identities, model selection, review counters and original deadline.
It records tool intent before dispatch and the resulting state after dispatch.

```sh
mirket agent status --project /absolute/path/project --run-id routed-ID
mirket agent resume --project /absolute/path/project --run-id routed-ID --expected-checkpoint-id CHECKPOINT
```

The MCP equivalents are `workflow_status` and `resume_subagent`. Resume requires
explicit authorization to continue that worker and the observed checkpoint. It
revalidates the compiled generation, contract/guidance/tool identities, original
provider/model capabilities, artifact/image hashes and remaining budget. It does
not reset turns, design correction counts or elapsed time. For a pending
`workspace_write`, the saved intent binds exact before/after digests, including
edits. Resume compares current bytes: intended bytes restore the artifact
obligation; original bytes allow the worker to reconsider the action. Neither
case replays a write. Other bytes, symlinks and other pending effects remain
blocked. Matching bytes do not establish authorship or correctness; readback,
design review and host checks remain required. There is no generic "mark effect
successful" bypass.

## Host verification

Source, web-interface, interactive-motion and editable-interface runs require
host verification. For UI, the host checks the real application, including relevant
API/authentication/failure paths; contained screenshots alone cannot cover these.
The existing `verification_handoff` now uses explicit attempts:

1. Inspect the trace and required check names.
2. Before executing checks, submit `begin: {key, inputs, commands}` with the
   observed `expected_trace_sha256`. Inputs are project-relative files/directories;
   commands maps every required name to its actual runner/command identity.
   Optional `design_inputs` are relative to the assigned design workspace.
   Produced artifacts are included automatically. Keep generated logs outside
   declared input directories. Each workspace is bounded to 64 input roots,
   20,000 entries and 256 MiB; symlinks and repository internals are refused.
3. Run checks through host tools, save logs, then submit all named `checks` with
   their evidence digests, returned `attempt_id`, and current trace digest.
4. For a repair/recheck, begin a new keyed attempt. Old attempts are immutable;
   replacing an old log no longer prevents a new attempt. Up to 64 attempts are
   retained per run.

Status rechecks inputs (including added/deleted directory entries) and evidence,
and persists `verification_stale` when they differ. Older unbound evidence is
`verification_unbound` until a new attempt runs. Command identities and result
summaries are host testimony, not proof the command executed.

The host executes
checks and supplies hash-bound evidence; the handoff synchronizes the same graph
only after validation. If synchronization fails after recording evidence, retry
the handoff with the same input: the trace is idempotent and graph state is
inspected first. A checkpoint or reported check is not human acceptance.

Private state lives under `.mirket/execution/podman/staging/agent-runtime/data`.
A 512 MiB per-project preflight budget bounds accumulation. This is a guard,
not an exact filesystem quota. For explicitly authorized cleanup, `mirket agent forget --project PROJECT
--run-id ID --expected-checkpoint-id CHECKPOINT` (MCP `workflow_forget`) deletes
one inactive run's checkpoints and compacts its database, retaining evidence traces.
Status, lifecycle export and deletion remain available at the storage limit.
Task traces are evidence projections, not another resume engine. Do not manually
rewrite checkpoint databases to force completion.

## Restorable backups and project relocation

```sh
mirket backup create --output /absolute/new-backup.tar
mirket backup inspect --archive /absolute/new-backup.tar --sha256 REPORTED_SHA256
mirket backup restore --archive /absolute/new-backup.tar --sha256 REPORTED_SHA256 --yes
```

The private, uncompressed archive contains a SQLite snapshot of coordination
records, the complete Mem0 and LangGraph data trees (including history and all
checkpoints), and run traces. A manifest binds every file's size and SHA-256,
home, source generation and runtime recipe. Creation is bounded to 20,000 files
and 2 GiB; it refuses links, special files and existing archive destinations.
It contains private facts and worker context; store it under your own backup policy.

This is a **durable-state backup**, not a project or machine backup. Application
source, design workspaces, external evidence files, Penpot databases, installed
packages, credentials and desktop settings are excluded. Back up those through
their existing owners. Restored records do not prove that external effects or
verification evidence still hold. Original source, path and deadline bindings
continue to govern worker resumption.

Reconnect every desktop client to the updated CLI before maintenance. Updated
state calls and workers hold shared leases; backup, restore and relocation require
an exclusive lease and fail while any are active. This is cooperative coordination,
not protection against direct database edits or old binaries. No worker is killed.
SQLite creates its own consistent snapshot including committed WAL pages. Runtime
files are copied only after their sessions have closed.

Restore validates the entire archive, schema and file hashes before publication.
It currently requires the original canonical home and matching runtime recipe,
with empty durable destinations; it never merges or overwrites an existing store.
An interrupted publication blocks state calls and retains a journal. Repeat the
same restore command to finish it; changed staged or published files are preserved
and reported. Existing installation files and unrelated host settings are untouched.
Do not delete live stores as a test: exercise restore in an explicitly disposable
fixture. Recovery after real data loss needs the original project directories and
separately backed-up source/design assets as well.

After physically **renaming the original directory**, rebind its registration:

```sh
mirket project list
mirket project relocate PROJECT_ID --expected-root /absolute/old-root --to /absolute/new-root --yes
mirket project doctor --project /absolute/new-root
```

Relocation checks the original filesystem identity and atomically updates the
registered root. Task IDs, memory scope, checkpoints, selected skills and the
assigned design namespace remain intact. It does not move application files,
copy between filesystems or accept a replacement directory. The old root must be
absent. An identical retry is harmless. A copy to another filesystem requires a
separate migration design; this command will reject it.

Historical worker paths and evidence are not rewritten. Status marks old path- or
source-bound runs non-resumable; start a new worker at the new root. Historical
checkpoints remain inspectable and memory remains usable. Recheck source/evidence
through the new root before treating old completion claims as current.

## Export and retention

```sh
mirket agent export --project /absolute/project --output /absolute/new-export.jsonl
mirket agent retention-plan --project /absolute/project --expired-before UNIX_TIME --completed-before UNIX_TIME
mirket agent retention-apply --request /absolute/reviewed-plan.json
```

Exports include **all active and expired facts and the latest snapshot of each
workflow**, with the loaded runtime identity. They exclude historical checkpoints,
Mem0 history, trace files and backups, and are not a restorable database image.
The CLI follows bounded pages, rejects changes between pages, and publishes only
a complete export to a new private file. MCP `runtime_data` with `action: export`
returns one page; follow `next_cursor` until null. Pages use indexed cursors, a project mutation revision and counts, with at most
20 records/3 MiB per page and a 3 MiB per-record bound. There is no count cap that
prevents cleanup of a larger inventory. Every owned mutation advances the revision
before execution, including uncertain/failed operations; direct database edits are
unsupported. No previous export page is rescanned. Large or busy projects should pause workers
before exporting. Exports can contain private facts and worker context.

The retention policy consists of two fixed past Unix timestamps. A plan selects
at most 100 expired facts and 100 completed workflows older than the cutoffs;
Each preview scans at most 256 records and returns `next_cursor`; follow it even
for an empty batch. The CLI accepts `retention-plan --cursor JSON`. Each batch must
be reviewed separately. Apply rechecks the selected records and their eligibility
without rebuilding the whole inventory. Only terminal
verified workflows qualify. Pending effects, blocked workers and outstanding
host verification remain, even after their execution deadline. Nonexpired facts,
trace evidence and backups are never selected automatically.

MCP `runtime_data` with `action: plan` takes `policy: {expired_before,
completed_before}`. Inspect the returned `result.plan` and `result.sha256`.
`runtime_data_apply` and the CLI request take `{project, plan, expected_sha256}`.
All candidates are rechecked before deletion; active-worker locks and changed
records reject the batch. Retry an interrupted application with the exact same
plan: missing entries reconcile as deleted and remaining history cleanup finishes.
The operation compacts SQLite. It does not claim forensic erasure.

These are explicit host operations, not a background scheduler or default purge.
Select policy dates for the project and review the batch before applying it.
The installation does not automatically delete existing memories or user evidence.

## Dependency and retrieval evaluation

`package audit agent-runtime` returns a CycloneDX 1.6 Python distribution
inventory and recorded installed-file bytes without mounting private data or
using the network. It reports upstream license metadata, not verified licensing
conclusions. Base OS packages and models are outside this inventory; consult the
digest-pinned image and model manifests separately. Vulnerability queries are an
explicit host task, not hidden network activity during runtime calls.

The real package tests report a public synthetic retrieval baseline using
English, Turkish, German and Spanish queries, current/older facts, instruction-like
text, irrelevant distractors and an unanswerable query. The report retains every
ranked result and latency, plus fixture-process peak RSS. It measures retrieval
on these fixtures, not answer quality, universal multilingual quality or model
prompt-injection resistance. No ranking threshold was tuned on these examples.

## Reproducing and verifying the dependency lock

`runtime/agent/requirements.in` declares direct dependencies. Resolve its complete
binary closure with Python 3.13/pip for Linux ARM64, save the pip JSON report, and
review it before replacing `requirements.lock`:

```sh
python -m pip install --dry-run --ignore-installed --only-binary=:all: \
  --platform manylinux_2_28_aarch64 --platform manylinux_2_17_aarch64 \
  --implementation cp --python-version 3.13 --abi cp313 --abi abi3 \
  --report linux-resolve.json -r runtime/agent/requirements.in
```

Each locked line is `NAME @ download_info.url --hash=sha256:HASH` from that report.
Retain prior versions as constraints when doing a narrow update. Installation
uses `--require-hashes --only-binary=:all:` and cannot run source build hooks.
Model manifests pin immutable revisions, every file's size and SHA-256. The
Python base image is digest pinned. Changes require a new recipe/image and real
`package test agent-runtime` execution. Its synthetic fixtures exercise semantic
recall, project isolation, retries, expiry, deletion/history, process restart,
stale checkpoints, effect/verification transitions and runtime containment.

The runtime reports loaded module/model-manifest/dependency-lock hashes and
package versions in the structured result. Rust checks them against its embedded
recipe on every call. Desktop adapters that drop MCP metadata still receive this
identity. `runtime_status` identifies the main process separately.

## Desktop contracts and bounded recall

Lifecycle MCP cursors are typed objects, passed back unchanged from `next_cursor`;
do not JSON-encode them into strings. Export cursors include `sha256`; retention
preview cursors omit it. Retention apply accepts the exact typed plan with its
scope, policy, memory IDs/hashes and workflow IDs/checkpoints. Unknown fields and
incorrect cursor types are rejected. CLI `--cursor` still takes JSON text because
it is a shell argument; the CLI deserializes it before dispatch.

Semantic recall grows its candidate pool when expired, superseded or unpublished
replacement facts occupy the initial results. It stops after enough active facts,
an exhausted candidate set, or 256 candidates. `retrieval.incomplete: true` means
the ceiling prevented filling the requested result count; an empty result in that
case is not evidence that no matching active fact exists. History and exports keep
the original facts. This is bounded retrieval, not a claim of exhaustive semantic
relevance. See [release acceptance](../CONTRIBUTING.md#contained-release-acceptance).
