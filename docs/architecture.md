# Architecture

Mirket has one command surface and one Rust implementation of each operation.
The executable embeds the authored guidance and catalog. Target installers call
the same CLI library; coordination MCP tools call the same catalog and state
methods. Optional package bridges launch pinned local servers separately; editor
and engine packages use Mirket-owned MCP adapters around pinned local runtimes.

| Source | Responsibility |
| --- | --- |
| `src/cli.rs` | Arguments, interactive setup, presentation and dispatch |
| `src/catalog.rs` | Embedded content, capability contracts, bounded discovery, verified upstream payloads |
| `src/catalog/teams.rs` | Professional profiles, validated task facets and bounded assignment briefs for host-run teams |
| `src/project.rs` | Explicit project selections, target copies and integrity receipts |
| `src/install.rs` | Client configuration, setup transaction and device doctor |
| `src/update.rs` | Release selection, candidate verification and setup replay |
| `src/state.rs` | Registered roots, durable tasks, revisions, idempotency and artifact evidence |
| `src/state/contracts.rs` | Strict opt-in verification and specialist coverage contracts with explicit input bindings |
| `src/role_eval.rs`, `evals/role-workflow/` | Versioned role fixtures, pinned host trial reports and deterministic aggregation |
| `src/mcp.rs` | Typed stdio protocol boundary and bounded tool/resource delivery |
| `src/package_mcp.rs` | Filtered browser package tools, process lifecycle and request limits |
| `src/editor_mcp/mod.rs`, `src/editor_mcp/process.rs` | Typed editor MCP tools, native startup checks, private jobs, cancellation and bounded process output |
| `src/editor_mcp/inkscape.rs`, `src/editor_mcp/axe_core.rs` | SVG geometry/exports and rendered local accessibility evidence |
| `src/editor_mcp/hyperframes.rs`, `src/editor_mcp/motion_canvas.rs`, `runtime/packages/` | Scoped web/drawn compositions, actual upstream playback and verified media publication |
| `src/editor_mcp/penpot.rs`, `src/packages/services.rs` | Typed editable design documents, real browser editors/plugins and owned local service lifecycle |
| `src/editor_mcp/svg.rs`, `src/editor_mcp/files.rs` | Supported vector/document validation, project-relative paths and staged artifact publication |
| `src/packages/mod.rs` | Package registry, selection, configuration and launch profiles |
| `src/packages/store.rs` | Verified downloads, immutable generations and integrity receipts |
| `src/packages/embedded.rs` | Bounded first-party adapter files, generation identity and collision-free staging |
| `src/packages/archive.rs` | Bounded extraction and archive path validation |
| `src/packages/dmg.rs` | Private read-only DMG mounts, bounded application copying, internal links and copied-bundle signature checks |
| `src/podman.rs` | Registered Podman client, private VM identity, explicit lifecycle and owned-worker recovery |
| `src/packages/container.rs`, `src/podman_api.rs` | Pinned OCI recipes, bounded worker admission, fixed Unix API execution and single-artifact retrieval |
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
Legacy recorded replies remain unchanged for exact retries. The catalog is parsed once
and shared immutably. Startup computes aggregate identities for all embedded
payloads once, including their applicable source notices. Unrelated notice sections do not
change a skill payload; all runtime notices still ship with distributions. Individual document digests are computed
on first read or role plan and reused thereafter. These are distinct costs.
Search ranks positive metadata separately
from description-only matches; exact IDs remain directly addressable. Catalog
validation resolves complete profiles for both hosts, including invocation and
dependency conflicts. Retrieval and downloads have explicit size limits.

## Trust and execution

Only explicit CLI project registration admits a canonical root to task tracking.
The coordination MCP server cannot register a filesystem root or execute a
command. Skill resources come from the embedded catalog. Evidence reads stay
inside registered project roots and reject symlink traversal. Host permissions
remain the real execution boundary.

Each selected package receives a separate `mirket-package-ID` client entry
running `mirket package serve ID`. Server bridges expose permitted upstream
tools, filtering both discovery and invocation and managing child-process
cancellation and shutdown. Upstream sampling, elicitation, roots requests,
resources and prompts are not forwarded. Editor and engine packages do not forward
an upstream MCP server: Mirket implements their tool schemas and calls the runtime
with fixed operation-specific arguments.

Package launch uses a controlled environment and explicit working directory.
The CLI's `--project` selects that directory; a configured host server uses the
directory chosen by its client. This is not a task-root registration or an
operating-system sandbox. Tools still have the process's actual permissions,
and cancellation cannot undo effects already performed.

The owned adapter dispatches task-oriented Inkscape, axe-core, HyperFrames,
Motion Canvas and Penpot tools. Inkscape validates self-contained SVG
before native geometry inspection and exports. The browser adapters use pinned
Node/Chromium and fixed local runners; media exporters use shared FFmpeg/FFprobe.
Editable project inputs remain distinct from rendered output. Each saved result
is reopened or independently inspected before publication.

Requests are limited to 128 KiB. Project-relative visible file paths reject
traversal and symlinks; output parents must exist. Existing destinations require
explicit overwrite intent, with optional revision hashes and a final snapshot
check. A private job workspace holds ephemeral browser state outside synchronized
project folders. Generated artifacts are bounded to 32 MiB. One job runs per
server; deadlines are 30 seconds for Inkscape, 60 for axe-core and 90 for
media/Penpot. Cancellation terminates the child process group and discards
unpublished staging. It cannot retract an artifact already published or a remote
Penpot edit already persisted. These controls do not create an OS sandbox.

`podman` owns the native registered client, private control directories and dedicated
VM connection. `packages::container` builds pinned Linux OCI images from the same
immutable package generations and stores the image identity in their receipt.
`podman_api` uses the private native Unix socket for bounded attached execution and
output retrieval; the socket is never mounted into workloads. Image IDs, recipe
labels, machine shares and connection identity are verified before launch.

Motion Canvas and HyperFrames use bounded session workers with read-only root,
non-root UID, dropped capabilities, private temporary storage, resource limits and
no network. Each receives only validated job inputs through a read-only staging
mount. Rendered output stays in bounded guest temporary storage; a single validated
regular artifact is retrieved before the existing project publisher runs. Queueing,
startup, staging, rendering and retrieval share the job deadline. Cancellation
removes the immutable owned worker ID and discards unpublished output. The VM is
started explicitly, never per call. See [Podman](podman.md) for platform limits.

Penpot adds five digest-pinned Linux ARM64 services. Explicit setup pulls images
through the registered Podman client. `package start` creates only labeled per-home
containers, one internal network and private VM volumes; it never pulls. Only the
frontend is exposed on loopback. Services use their reviewed non-root users,
read-only roots, bounded private temporary storage and resource limits. Readiness
requires real backend JSON, not a running process. Failed start removes new
containers, restores prior running state and retains durable data. `package stop`
removes owned containers/network and preserves designs. Private credentials stay
in the selected home. The fixed plugin runs in an active native headless browser;
arbitrary Plugin API scripts are not exposed through Mirket tools.

Managed Codex connections have a 90-second startup and 120-second tool budget.
Claude entries have a 120000 ms tool timeout; unrelated host settings are preserved.
These outer budgets allow integrity checks without extending job deadlines.
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
a fresh unattended setup chooses none. No `npm`, `npx` or `pip` resolution occurs
at runtime. Additional server profiles require pinned artifacts, complete
notices, an appropriate tool policy and relevant invocation tests. See
[local MCP packages](packages.md) for supported profiles and prerequisites.
