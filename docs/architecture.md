# Architecture

Mirket has one command surface and one Rust implementation of each operation.
The executable embeds the authored guidance and catalog. Target installers call
the same CLI library; coordination MCP tools call the same catalog and state
methods. The optional `design` package launches its pinned local tools and the
official Penpot MCP through one reviewed package contract.

Embedded skill descriptions come from each `SKILL.md` frontmatter, so host
triggers and catalog discovery use the same source. The registry owns selection,
dependencies and provenance; upstream and reference entries retain their reviewed
descriptions there. CLI and MCP selected metadata both expose the source pin and
guidance read route.

| Source | Responsibility |
| --- | --- |
| `src/cli.rs` | Arguments, interactive setup, presentation and dispatch |
| `src/catalog.rs` | Embedded content, capability contracts, bounded discovery, verified upstream payloads |
| `src/catalog/teams.rs` | Professional profiles, validated task facets and bounded assignment briefs for host-run teams |
| `src/catalog/workflows.rs` | Validated task workflows binding default expertise, owned tool choices, required invocation and verification |
| `src/catalog/design_references.rs` | Validated design-language records and bounded instance discovery within the existing embedded skill |
| `src/catalog/design_references/coverage.rs` | Explicit scoped design coverage and conservative gap lookup through existing catalog search |
| `src/catalog/design_references/selection.rs` | Bounded provisional lexical candidates after an empty reference query; separate from ordinary matches and scoped coverage |
| `src/project.rs` | Explicit project selections, target copies and integrity receipts |
| `src/install.rs` | Client configuration, setup transaction and device doctor |
| `src/target.rs` | Shared host identity and path layout for setup, project sync/recovery and catalog selection |
| `src/update.rs` | Release selection, candidate verification and setup replay |
| `src/state.rs` | Registered roots, durable tasks, revisions, idempotency and artifact evidence |
| `src/state/contracts.rs` | Strict opt-in verification and specialist coverage contracts with explicit input bindings |
| `src/role_eval.rs`, `evals/role-workflow/` | Versioned role fixtures, pinned host trial reports and deterministic aggregation |
| `src/mcp.rs` | Typed stdio protocol boundary and bounded tool/resource delivery |
| `src/router/` | Dynamic target/model discovery, capability policy, isolated inference and owned tool dispatch |
| `src/router/assignment.rs` | Shared task-needs resolver, bounded role/skill delivery, workflow floors and non-expanding dispatch scope |
| `src/package_mcp.rs` | Design package MCP session lifecycle and bounded requests |
| `src/packages/mod.rs` | The single package selection, configuration and metadata |
| `src/packages/machine.rs` | Fixed design VM image and shared-workspace preparation |
| `src/packages/services.rs` | Design service start, status, stop and owned data lifecycle |
| `runtime/design/tools/` | Direct design tool schemas and calls inside the shared workspace |
| `src/podman.rs` | Registered Podman client, private VM identity and explicit lifecycle |
| `src/tools.rs` | Explicitly registered executables and CLI-only invocation |
| `src/dev.rs` | Build/check/package operations and performance measurement |
| `src/release.rs` | Verification and assembly of native release manifests |
| `src/util.rs`, `src/paths.rs` | Bounded I/O, integrity, atomic writes and path ownership |
| `setup/*.rs` | Standalone installers using the same CLI entrypoint |
| `registry/`, `skills/`, `instructions/` | Reviewed content and host-neutral working principles |

## Local state

`~/.mirket` holds the managed binary, saved setup, install receipt, verified
payload cache, registered tools and SQLite task database. Package download
caches, prepared generations and mutable data live under `~/.mirket/packages`.
`--home` changes the
user-home boundary for all harness/client installation paths. Each project's
`.mirket` directory contains its own selection, copy receipts and durable sync
journal. Sync recovers interrupted directory publications before new work, checks
all recovery destinations for drift and preserves conflicts. Doctor reports owned
journals and unknown historical temporary recovery trees. Unknown remnants are
never swept automatically.
Setup recovery validates its journal and retained files before rollback or
cleanup, preserving unexpected entries. Rollback cleanup removes the journal
before its before-images; committed cleanup keeps the commit marker until last.
Update candidate checks and setup replay own their process groups/jobs. On Unix,
the entrypoint replaces itself with a verified newer managed runtime so host
termination continues to target the runtime PID.

SQLite transactions are short; WAL permits concurrent reads. Opening an existing
validated schema-3 task database performs no schema write, so CLI readers can
continue while another writer holds a transaction. Valid schema-1/2 databases
migrate transactionally to schema 3, retaining records and retry replies while
updating the task-list index (and adding contracts for schema 1); earlier
runtimes refuse the new version. Unknown or malformed layouts fail closed. Task lists use project-bound cursors
over immutable creation time and ID; checkpoints cannot shift page membership.
Task status identifies omitted checkpoints; task-bound history cursors recover
older notes in bounded revision-descending pages without shifts from new appends.
Opt-in verification contracts bind explicit inputs, checker and result files;
contribution contracts bind the planned roster and account for each specialist
assignment. Registration checks structure and identities, and completion repeats
those checks with terminal outcome requirements. Ordinary evidence remains an
explicit file record; a typed record cannot be downgraded by omitting its type.
See [task evidence](task-evidence.md) for formats, limits and compatibility.
Task mutations use
expected revisions and idempotency keys. An identical retry returns its recorded
result; reusing a key for another intention fails. New successful writes return
versioned compact receipts; explicit status/history calls retrieve full state.
Legacy recorded replies remain unchanged for exact retries. Commands initialize the catalog only when they need it; each process parses it
once and shares it immutably. Notice sections are parsed once for all skills. Startup computes aggregate identities for all embedded
payloads once, including their applicable source notices. Unrelated notice sections do not
change a skill payload; all runtime notices still ship with distributions. Individual document digests are computed
on first read or role plan and reused thereafter. These are distinct costs.
Search ranks positive metadata separately
from description-only matches; exact IDs remain directly addressable. Catalog
validation resolves complete profiles for both hosts, including invocation and
dependency conflicts. Retrieval and downloads have explicit size limits.

The optional `design-inspiration` payload contains original language
and interaction lessons with standalone contrastive examples. A bounded schema 2
index locates one canonical typed source record per instance; those records bind
URLs, editions, capture identities, rights limits and exact local resources.
Catalog search matches all query terms within one instance and returns at most
five summaries on the owning skill result; source bodies are never searched,
fetched or executed. Show links the complete index and manifest. Catalog loading
validates routes, bounded readable files, source/rights records and original
example license scope. Normal project sync delivers these files to both hosts;
there is no parallel installer, tool or state store. Structural validity and a
source digest do not establish visual quality, legal clearance or a measured
model capability improvement. Optional structured design context on the same
catalog search validates exact registry axis IDs and matches one scoped cell to
the complete request. It reports represented/partial/unrepresented inventory,
per-candidate gaps and `current_task_validated:false`. Source-only native guidance
cannot satisfy worked-example coverage. It neither pools unrelated cells nor
launches retrieval, a browser or a model. The existing role-eval checker binds
the paired transfer pilot; actual host execution and independent artifact grading
remain outside catalog bookkeeping.

Reference routing supplies optional discovery, not a design policy engine. The
host/model judges fit, can compare several sources for one question, seek sources
outside the inventory or use no reference. Contrast exercises teach relationships;
they are not required production steps. Team planning and durable tracking are
available when useful for independent work, coordination or resumption; a
sufficient host/project workflow needs no duplicate record. Choosing a typed task
contract still enables its exact binding and completion checks. User/project
requirements, source rights and host permissions remain authoritative.

## Trust and execution

Only explicit CLI project registration admits a canonical root to task tracking.
The coordination MCP server cannot register a filesystem root or execute a
shell command. Its `spawn_subagent` tool executes schema-bound inference and
explicitly assigned file/design tools through the [capability router](../skills/agent-coordination/references/model-routing.md).
Discovery runs at MCP initialization and uses a bounded TTL cache; raw model IDs
are accepted only from target metadata. Concurrent refreshes share work per home;
at most two targets are probed at once, with target and refresh deadlines. The
32-target limit includes automatically discovered targets, and both paginated
metadata and the persisted cache have aggregate byte limits.
Skill resources come from the embedded catalog. Evidence reads stay
inside registered project roots and reject symlink traversal. Host permissions
remain the real execution boundary.

Owned worker requests must declare bounded task needs from the catalog's task
workflows. The shared assignment resolver selects default specialist roles,
loads their exact embedded skill bodies and selected references, and derives a
small tool set with mandatory invocation and verification. An explicit role can
add its professional perspective without removing task-specialist guidance or
expanding tools. Explicit tool selections can narrow the owned scope, never add
unrelated tools or omit required producers. The same contract drives preview,
inference context, intelligence floors, dispatch checks, completion and traces.
Workflow IDs and reasons are a structured host interpretation of the task, not
proof of semantic fit or authorization. Mirket cannot intercept direct native
host tools or infer human approval from role text.

Owned source writes accept either whole-file content or bounded, unique exact
replacements. Both require the observed digest for an existing file and subsequent
readback. Responses commentary is never dispatched as an action: the complete
output must validate before its single final action can execute. Generated Odiff
diagnostics are recorded as artifacts and invalidate prior mutation verification.

The single design package receives a client entry running `mirket package serve
design`. Its direct tool schemas use paths relative to one fixed `/workspace`
mount, backed by the selected home's
`.mirket/execution/podman/staging/design/workspace`. It forwards the official
Penpot MCP tool schemas and calls without adapting them into another design-object
API. `package start` and `package status` report the host workspace path.
Cancellation cannot undo effects already performed. See [packages](packages.md).

`podman` owns the native registered client, private control directories and
dedicated VM connection. The design package prepares its pinned tool and service
images and mounts the same workspace read/write in each container. The private
Podman control socket is not part of that shared workspace. Service identity,
image pins and workspace sharing must be checked during startup and status.

Runtime readiness, actual invocation, artifact correctness and human acceptance
are separate facts. See [packages](packages.md) for target-specific contracts.

## Extension boundary

`mirket tool` registers explicit local executable paths and hashes. Invocation
uses argv directly and inherits the caller's permissions. Foreground invocations
own their Unix process group or Windows job; timeout, cancellation and launcher
exit clean up owned descendants. Deliberate Unix session/group escapes are outside
this ownership contract. It is not an OS sandbox or rollback of completed effects.
It does not turn arbitrary executables into MCP tools or resolve packages.

`mirket package` supplies reviewed server and runtime definitions through the
same setup, doctor and update flow. Optional packages are explicit selections;
a fresh unattended setup chooses none. Official Penpot MCP source is built with
pinned pnpm dependencies during package preparation; serving performs no package
resolution. Additional server profiles require pinned artifacts, complete
notices, an appropriate tool policy and relevant invocation tests. See
[local MCP packages](packages.md) for supported profiles and prerequisites.
