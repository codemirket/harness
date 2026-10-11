# Architecture

Mirket has one command surface and one Rust implementation of each operation.
The executable embeds the authored guidance and catalog. Target installers call
the same CLI library; coordination MCP tools call the same catalog and state
methods. Optional package bridges launch pinned local servers separately; editor
packages use Mirket-owned MCP adapters around pinned native applications.

| Source | Responsibility |
| --- | --- |
| `src/cli.rs` | Arguments, interactive setup, presentation and dispatch |
| `src/catalog.rs` | Embedded content, capability contracts, bounded discovery, verified upstream payloads |
| `src/project.rs` | Explicit project selections, target copies and integrity receipts |
| `src/install.rs` | Client configuration, setup transaction and device doctor |
| `src/update.rs` | Release selection, candidate verification and setup replay |
| `src/state.rs` | Registered roots, durable tasks, revisions, idempotency and artifact evidence |
| `src/mcp.rs` | Typed stdio protocol boundary and bounded tool/resource delivery |
| `src/package_mcp.rs` | Filtered package tool bridges, SQL/document input policies, process lifecycle and request limits |
| `src/editor_mcp/mod.rs`, `src/editor_mcp/process.rs` | Typed editor MCP tools, native startup checks, private jobs, cancellation and bounded process output |
| `src/editor_mcp/inkscape.rs`, `src/editor_mcp/glaxnimate.rs` | Editable SVG and animation operations, native result checks and export semantics |
| `src/editor_mcp/svg.rs`, `src/editor_mcp/files.rs` | Supported vector/document validation, project-relative paths and staged artifact publication |
| `src/packages/mod.rs` | Package registry, selection, configuration and launch profiles |
| `src/packages/store.rs` | Verified downloads, immutable generations and integrity receipts |
| `src/packages/archive.rs` | Bounded extraction and archive path validation |
| `src/packages/dmg.rs` | Private read-only DMG mounts, bounded application copying, internal links and copied-bundle signature checks |
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
`.mirket` directory contains its own selection and copy receipts.

SQLite transactions are short; WAL permits concurrent reads. Opening an existing
validated task database performs no schema write, so CLI readers can continue
while another writer holds a transaction. Task lists use project-bound cursors
over immutable creation time and ID; checkpoints cannot shift page membership.
Task mutations use
expected revisions and idempotency keys. An identical retry returns its recorded
result; reusing a key for another intention fails. The catalog is parsed once
and shared immutably. Each document digest is computed on its first read or role
plan and reused thereafter; unrelated documents add no hashing work to startup.
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
resources and prompts are not forwarded. Editor packages have no upstream MCP
server: Mirket implements their tool schemas and calls the native executable
with fixed operation-specific arguments.

Package launch uses a controlled environment and explicit working directory.
The CLI's `--project` selects that directory; a configured host server uses the
directory chosen by its client. This is not a task-root registration or an
operating-system sandbox. Tools still have the process's actual permissions,
and cancellation cannot undo effects already performed.

Profile policies are enforced before forwarding calls. DBHub SQL is prepared
against the selected SQLite file through a read-only native connection and
authorizer; writes, `PRAGMA`, `ATTACH`, extension loading and requests over
64 statements are refused. Its upstream configuration also enables read-only
queries and a 200-row cap. Safe-Docx rejects cloud document IDs and validates
explicit project-scoped output destinations and overwrite permission. Both clean
and tracked destinations must be explicit for its default dual-output save.
Semiotic exposes six developer tools and limits rendering to static SVG.

MarkItDown uses a private CPython runtime and pinned wheels for local text,
Office and PDF conversion. The launch excludes user-site and `.pth` startup
hooks, working-directory imports, project `.env` loading and plugins. Its single
tool accepts only bounded local files inside the selected project. Python and
MarkItDown definitions cover four macOS/Linux ARM64/x64 targets; Windows is
unavailable. These argument and environment policies do not isolate process
permissions or create a network sandbox.

The editor adapter provides three Inkscape tools for editable static SVG,
inspection and exports, and five Glaxnimate tools for creation, inspection, frame
rendering, animated SVG export and retiming. Static SVG and native animation
documents are validated before opening the application. The animation profile
supports one 60 fps composition with solid-color vector shapes and linear
translation/opacity tracks, preserving once/freeze behavior unless looping is
explicit. This is an offscreen file workflow, not a desktop GUI-control surface.

Each editor request uses project-relative visible paths, a private staging
directory and private HOME/profile/temp directories. Existing outputs require
explicit overwrite intent; the destination snapshot is rechecked before atomic
publication. Exports and retimed documents preserve their input sources. Requests
are limited to 128 KiB, file sources to 2 MiB and generated artifacts to 32 MiB.
Each native process has a 30-second deadline; one job runs per server at a time.
Cancellation reaps the process group and discards unpublished staging, but cannot
retract an already published artifact. Native output is checked beyond the exit
status. These controls do not create an operating-system sandbox or remove the
editor's host permissions.

Editor client startup has a separate 90-second budget in Mirket CLI checks and
managed Codex entries; Codex tool calls receive 120 seconds. Claude entries use a
120000 ms per-server tool timeout, while its global host startup environment is
preserved. These outer budgets leave room for integrity verification and several
native steps without extending the 30-second native process deadline. The
[installation reference](installation.md#editor-connection-budgets) links the
host-specific settings and their different units.

Content hashes establish bytes and detect stale evidence. They do not prove that
an agent applied the guidance, that a check actually ran or that a reviewer is
human. Callers must use the project's real verification tools and report what
they observed. Local state is user-owned and is not an attestation service.

Installation stages all changes, checks ownership/conflicts and preserves
unrelated configuration. Updates run a verified new executable to apply its own
embedded payload and saved setup choices, including selected packages and their
settings. The update and setup locks coordinate writers; failed setup must leave
the installed runtime recoverable.

Package definitions pin source identity, platform artifacts, runtime dependencies,
SHA-256 values, extraction bounds, entrypoints and notices. Setup downloads and
extracts these artifacts, including required private runtimes, without running
lifecycle hooks or package managers.
It publishes a new generation atomically and never overwrites an existing
generation in place. Launch and doctor compare its files with a local receipt;
they do not prove upstream safety or resist an owner who replaces both files
and receipt. Package removal changes selection and client entries while retaining
cached downloads, generations and data.

The macOS ARM64 editor artifacts contain their native runtime closure inside
signed app bundles. DMG extraction copies the selected bundle from a private
read-only mount, rejecting escaping or invalid links and preserving valid
relative links within the app. A system code-signature check must pass on the
copied bundle before the generation is published. Mirket neither strips metadata
nor re-signs applications. File-provider-managed directories can add metadata
that makes a copied signature invalid; this is a failed installation, not a
reason to bypass signature verification. Original notices and reviewed source
inventories accompany each installed bundle. Their documented native
corresponding-source gaps remain relevant to any future binary redistribution;
Mirket's current releases do not contain the editor binaries.

## Extension boundary

`mirket tool` registers explicit local executable paths and hashes. Invocation
uses argv directly, inherits the caller's permissions and observes a timeout.
It does not turn arbitrary executables into MCP tools or resolve packages.

`mirket package` supplies reviewed server and runtime definitions through the
same setup, doctor and update flow. Optional packages are explicit selections;
a fresh unattended setup chooses none. No `npm`, `npx` or `pip` resolution occurs
at runtime. Additional server profiles require pinned artifacts, complete
notices, an appropriate tool policy and relevant invocation tests. See
[local MCP packages](packages.md) for supported profiles and prerequisites.
