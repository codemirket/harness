# Architecture

Mirket has one command surface and one Rust implementation of each operation.
The executable embeds the authored guidance and catalog. Target installers call
the same CLI library; MCP tools call the same catalog and state methods.

| Source | Responsibility |
| --- | --- |
| `src/cli.rs` | Arguments, interactive setup, presentation and dispatch |
| `src/catalog.rs` | Embedded content, capability contracts, bounded discovery, verified upstream payloads |
| `src/project.rs` | Explicit project selections, target copies and integrity receipts |
| `src/install.rs` | Client configuration, setup transaction and device doctor |
| `src/update.rs` | Release selection, candidate verification and setup replay |
| `src/state.rs` | Registered roots, durable tasks, revisions, idempotency and artifact evidence |
| `src/mcp.rs` | Typed stdio protocol boundary and bounded tool/resource delivery |
| `src/tools.rs` | Explicitly registered executables and CLI-only invocation |
| `src/dev.rs` | Build/check/package operations and performance measurement |
| `src/release.rs` | Verification and assembly of native release manifests |
| `src/util.rs`, `src/paths.rs` | Bounded I/O, integrity, atomic writes and path ownership |
| `setup/*.rs` | Standalone installers using the same CLI entrypoint |
| `registry/`, `skills/`, `instructions/` | Reviewed content and host-neutral working principles |

## Local state

`~/.mirket` holds the managed binary, saved setup, install receipt, verified
payload cache, registered tools and SQLite task database. `--home` changes the
user-home boundary for all harness/client installation paths. Each project's
`.mirket` directory contains its own selection and copy receipts.

SQLite transactions are short; WAL permits concurrent reads. Task mutations use
expected revisions and idempotency keys. An identical retry returns its recorded
result; reusing a key for another intention fails. The catalog is parsed once
and shared immutably. Retrieval and downloads have explicit size limits.

## Trust and execution

Only explicit CLI project registration admits a canonical root to task tracking.
MCP cannot register a filesystem root or execute a command. Skill resources come
from the embedded catalog. Evidence reads stay inside registered project roots
and reject symlink traversal. Host permissions remain the real execution boundary.

Content hashes establish bytes and detect stale evidence. They do not prove that
an agent applied the guidance, that a check actually ran or that a reviewer is
human. Callers must use the project's real verification tools and report what
they observed. Local state is user-owned and is not an attestation service.

Installation stages all changes, checks ownership/conflicts and preserves
unrelated configuration. Updates run a verified new executable to apply its own
embedded payload and saved setup choices. The update and setup locks coordinate
writers; failed setup must leave the installed runtime recoverable.

## Extension boundary

`mirket tool` registers explicit local executable paths and hashes. Invocation
uses argv directly, inherits the caller's permissions and observes a timeout.
There is no MCP execution tool or implicit package installer. Additional package
sources and binaries can use this registry without creating separate setup flows.
