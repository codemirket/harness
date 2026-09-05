# Containers MCP

A local MCP server for sharing persistent Docker services across projects and AI
agents. It exposes interactive tools, a catalog resource, and a setup prompt.
Codex and Claude start it on demand through stdio. Every client uses the same
machine-local catalog, credentials, project allocations, and containers.

Setup migrates owned `development` registrations to `containers`. Existing catalog,
volumes, Docker labels, and `DEVELOPMENT_MCP_*` environment variables retain their
original identities so existing services remain reusable.

## Requirements and installation

- Node.js 24 or newer with npm.
- Docker Desktop running Linux containers, using a local Unix socket or Windows named pipe.
- Codex CLI and Claude Code CLI on PATH (or select just the installed agent).
- Internet access for initial npm dependencies, image pulls, and the MinIO source build.

From the shared `.ai` repository:

```sh
# macOS: install for both agents
sh setup/MacOS/containers.sh
# Or: sh setup/MacOS/containers.sh codex
```

```powershell
# Windows 11: install for both agents
& .\setup\Windows11\containers.ps1
# Or: & .\setup\Windows11\containers.ps1 -Agent codex
```

The existing per-agent setup scripts also install and register containers MCP.
You can alternatively run `npm run setup -- --agent all` from this directory.
Restart existing agent sessions after setup. Codex receives a 30-minute per-tool
timeout for initial image pulls and MinIO source builds. The MCP server starts automatically
when the agent connects; no listening HTTP port or separately managed daemon is
required. Containers use Docker's `unless-stopped` restart policy and outlive MCP
clients. Setup seeds the catalog but does not start containers.

Setup installs a versioned runtime outside the shared repository, so npm packages
are not synced through iCloud or Git:

| Location | Purpose |
| --- | --- |
| macOS `~/.local/share/containers-mcp/releases/<hash>` | Installed runtime snapshots |
| Windows `%LOCALAPPDATA%\containers-mcp\releases\<hash>` | Installed runtime snapshots |
| `~/.dev.mcp/catalog.sqlite` | Catalog, credentials, project allocations, registration metadata |
| `~/.dev.mcp/operations.sqlite` | Automatic cross-process operation lock |
| `~/.dev.volumes/<service-alias>` | All managed container bind data and service configuration |

Run setup again after updating source. It reuses unchanged runtime snapshots and
updates its own registrations. It refuses to overwrite an unrelated server already
named `containers`. Older runtime snapshots remain available to running clients;
remove unused snapshots only after those sessions have ended.

Codex receives a global `containers` MCP entry through its CLI. Claude Code
receives a **user-scoped** entry, available across projects (also used by the
Claude Desktop Code tab). When Claude Desktop's configuration directory exists,
setup also merges a stdio entry into its separate chat configuration.

## Initial catalog

| ID / default alias | Service | Default host ports | Container data path |
| --- | --- | --- | --- |
| `postgres` | PostgreSQL 18 | 5432 | `/var/lib/postgresql` |
| `redis` | Redis 8, append-only persistence | 6379 | `/data` |
| `minio` | S3-compatible MinIO | 9000 API, 9001 console | `/data` |

Published ports bind to `127.0.0.1` only. Set a port to `0` when creating or recreating a removed
service to let Docker allocate an available port. Always obtain the actual endpoint
from `get_connection`; automatically allocated ports can change on recreation.
Containers also share a managed Docker network. Containerized projects can join
the network returned by `inspect_service` and use `containerHost`/`containerPort`.
The server does not attach unrelated project containers automatically.

PostgreSQL and Redis major versions are fixed. The resolved image digest is saved
on creation and reused for recreation. Automatic database major upgrades are not
performed. Register a new template and alias when a different version is needed.

MinIO community upstream was archived and its legacy images are no longer updated.
The included Dockerfile builds official MinIO release
`RELEASE.2025-10-15T17-29-55Z` and mc release `RELEASE.2025-08-13T08-35-41Z` from source.
The first build can take several minutes. This supports the requested local MinIO
service but does not provide ongoing upstream maintenance. See
[MinIO's source/distribution notice](https://github.com/minio/minio) and its
[security-fix release](https://github.com/minio/minio/releases/tag/RELEASE.2025-10-15T17-29-55Z).

## Agent workflow

Example request:

> Use containers MCP to find the shared PostgreSQL service, allocate a database
> for `shop-api`, and configure my local tests with its project credentials.

The agent should call:

1. `list_services` and `list_catalog` to find a compatible existing service.
2. `ensure_service` with `{"template":"postgres","alias":"postgres"}`.
3. `provision_project` with `{"alias":"postgres","project":"shop-api"}`.
4. `get_connection` with `{"alias":"postgres","project":"shop-api"}`.

A second project repeats steps 3–4 with another stable project ID. It uses the
same container and its own database/login. Project IDs and aliases are lowercase
letters, digits, and hyphens, start with a letter, and contain at most 40 characters.

| Tool | Behavior |
| --- | --- |
| `list_catalog` | Show available service templates, without credential values |
| `add_catalog_entry` | Add a custom Docker service template |
| `remove_catalog_entry` | Remove an unused template; never deletes bind data |
| `list_services` | Discover all local containers and distinguish managed services |
| `inspect_service` | Show state, endpoints, volume path, network, and projects |
| `ensure_service` | Reuse, create, start, or recreate a service |
| `provision_project` | Reuse or create project-specific service resources |
| `get_connection` | Explicitly return project credentials or administrator access |
| `stop_service` | Stop a shared service after confirming its alias |
| `remove_service` | Remove a shared container after confirming its alias; preserve data |

`containers://catalog` provides a read-only catalog resource. The
`use-local-service` prompt accepts `service` and `project` arguments.

### Project separation

- **PostgreSQL:** separate database and login per project. Public database
  connections are revoked for each managed project database.
- **Redis:** project ACL users can access only keys with their returned prefix.
  Use `keyPrefix` in your client. The allowed command set supports common data
  operations; administrative commands, Lua, scanning, and unrestricted Pub/Sub
  are excluded. This is shared trusted development infrastructure, not a security
  boundary between hostile applications.
- **MinIO:** separate bucket and user with a bucket-specific policy. Use the
  returned endpoint, access keys, region, and path-style S3 addressing.
- **Custom templates:** shared service credentials and endpoints. Automatic
  project provisioning is limited to the three built-in service kinds.

Omit `project` from `get_connection` only when administrator credentials are
needed. Its result is sensitive. Do not log it, commit it, or put it in shared
project documentation. Listing and inspection never return container environment
variables or credential values.

### Expand the catalog

For example, register Mailpit:

```json
{
  "id": "mailpit",
  "description": "Shared local mail capture",
  "image": "axllent/mailpit:v1.27",
  "dataPath": "/data",
  "ports": [
    { "name": "smtp", "container": 1025, "host": 1025 },
    { "name": "web", "container": 8025, "host": 8025 }
  ],
  "environment": { "MP_DATABASE": "/data/mailpit.db" },
  "secretEnvironment": [],
  "command": []
}
```

Supply this object to `add_catalog_entry`, then call `ensure_service` with
`{"template":"mailpit"}`. Use `secretEnvironment` for environment variable names
that need generated random secrets. Custom commands are argument arrays executed
by the selected container image; no host shell or raw Docker flags are exposed.
Custom services report container running state; their application readiness must
be checked by the consuming project. Only install images intended by the user.

## Persistence and removal

Stopping or removing a service affects every project using it. Tool descriptions
require the agent to obtain authorization and repeat the alias in `confirmAlias`.
The repeated alias is an accidental-action guard, not proof of human consent.
There is intentionally no data-purge or project-delete tool.

`remove_service` keeps bind data, credentials, and project records. A subsequent
`ensure_service` recreates the container. Catalog entries can be removed after
their active containers are removed; retained service records still carry their
original template, allowing recovery even after catalog removal. Catalog IDs
referenced by retained services cannot be reassigned to different software; use
a new ID for a new definition.

Back up `~/.dev.mcp` together with `~/.dev.volumes`. Stop services before copying
raw database files, or use each database's native backup procedure. Do not sync
live state, credentials, volumes, or SQLite files into the shared `.ai` directory.

Unmanaged containers appear in discovery but cannot be adopted, stopped, or removed
by this server. The server labels its containers with a catalog owner ID and verifies
ownership before changes. A name collision causes an error. A remote Docker context
is rejected because local bind paths would refer to another machine.

## Configuration and diagnostics

Setup registers absolute Node and Docker paths so GUI-launched clients do not
require your interactive shell setup. Rerun setup after moving those executables.
These optional environment variables must be set **before setup** to register
non-default local paths:

- `DEVELOPMENT_MCP_HOME`: machine-local catalog directory; default `~/.dev.mcp`.
- `DEVELOPMENT_MCP_VOLUMES`: bind root; default `~/.dev.volumes`.
- `DEVELOPMENT_MCP_DOCKER`: Docker executable for direct server launches; setup
  resolves the installed executable from PATH.

The original Docker endpoint is recorded. Restore that local context if the server
reports a context mismatch. Catalog browsing works while Docker is stopped;
container inspection and operations require Docker Desktop.

From this source directory, after `npm ci`:

```sh
node src/index.mjs --check
npm test
npm run test:docker
```

`--check` checks local Docker connectivity and prints paths and template IDs only.
Unit/protocol tests use temporary catalogs. Docker integration tests create
uniquely named temporary containers and bind directories under `~/.dev.volumes`,
verify two-client sharing, project separation, and recreation durability, then
remove only those test artifacts. Images remain cached for reuse.

Global registration uses the documented
[Codex MCP CLI](https://learn.chatgpt.com/docs/extend/mcp?surface=cli) and
[Claude Code user scope](https://code.claude.com/docs/en/mcp#user-scope).

### Standalone Claude Desktop chat

Setup preserves existing settings and merges `mcpServers.containers` in:

- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`.
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`.

If that configuration directory does not exist yet, open Claude Desktop once,
then rerun setup. Claude Code CLI remains a prerequisite for the Claude setup
path. Restart Claude Desktop to load its new MCP server.
