# Local Penpot design workspace

Mirket packages Penpot 2.18.3 as a local service and exposes four typed design tools.
It runs the real editor in an active, headless Chromium session, installs a fixed
Mirket plugin with Penpot's normal permission checks, and verifies that changes
persist before publishing a project handle. It does not require a model, cloud
account, paid API or visible editor window.

## Install and start

The supported slice is macOS ARM64 with Mirket's dedicated rootless Linux ARM64
Podman environment. The native Rust CLI and desktop MCP launch entry remain on the
host. Register the approved native Podman executable, initialize its pinned VM,
then select and start Penpot explicitly:

```sh
mirket tool register podman --executable /absolute/path/to/podman
mirket environment init
mirket environment start
mirket setup --target all --package penpot
mirket package start penpot
mirket package status
mirket package probe penpot --project /absolute/project
```

`environment init` uses the reviewed machine image and shares only Mirket's private
staging directory. Environment start is explicit; neither package calls nor MCP
initialization restart the VM. An absent or stopped environment produces a
recovery command. The Podman client uses a private home, configuration and explicit
connection; it does not adopt another application's engine or mount its socket.

Setup downloads the exact service image closure only during installation.
Service start uses installed digests with pulls disabled. All five services join
one internal bridge. Only the frontend publishes the selected loopback port;
internal Podman networking was exercised with a successful host HTTP response.
No service mounts a host home, project, credentials, application data or socket.
The database, cache and assets reside in this environment's owned named volumes.
The frontend sees assets read-only. Each process has a numeric non-root user,
read-only root, dropped capabilities, no new privileges, private bounded temporary
storage, CPU/memory/PID limits and a private IPC namespace. Rootless isolation is an
engine boundary; these configuration checks are not a security attestation.

| Service | UID/GID | Memory limit | CPU ceiling | PID limit |
| --- | --- | --- | --- | --- |
| PostgreSQL | 999/999 | 512 MiB | 1 | 128 |
| Valkey | 999/999 | 256 MiB | 1 | 128 |
| Backend | 1001/1001 | 2048 MiB | 2 | 512 |
| Exporter | 1001/1001 | 1024 MiB | 2 | 512 |
| Frontend | 1001/1001 | 256 MiB | 1 | 128 |

These are ceilings, not reservations or throughput claims. All processes share the
VM's fixed resources. Swap growth is disabled. Temporary mounts permit the reviewed
applications' necessary writes without making their installation writable. A fixed
Mirket frontend launcher generates configuration from pinned upstream templates in
private memory; it disables external templates and Google Fonts routes. Bundled
fonts work locally. Database credentials are limited to PostgreSQL/backend; the
exporter receives its required service key and cache URI, never the database key.
Each service writes bounded `k8s-file` logs in its private container storage, with
a 4 MB ceiling. Exact driver, ceiling and storage path are checked before reuse;
the managed log path is not a host mount or caller-selected destination.
The backend's approved JAR requires native zstd JNI compression for editable
documents. General temporary storage remains `noexec`; only a separate private
8 MiB `nosuid,nodev` temporary mount permits executable library mapping, selected
by the pinned library's `ZstdTempFolder` property. The library loads its bundled
Linux ARM64 bytes and deletes the extracted file after mapping. This exception
does not make the application installation writable or mount host data.
The pinned HTTP multipart parser receives a separate 64 MiB `noexec,nosuid,nodev`
temporary mount at its fixed `/tmp/undertow` path for thumbnail and rendered-export
uploads. The rest of `/tmp` stays on the read-only root.

Private service state is bound to both the selected Mirket home and its private
engine. Unbound or differently owned data is preserved and refused; export portable
designs from its owning environment before provisioning a separate service.
The first start creates the database and runs application migrations. Readiness
requires a bounded JSON anonymous-profile response and exact running service
configuration. A process exit code, open port or running Java process is insufficient.
Only an actual design call establishes successful browser/plugin operation.

```sh
mirket package stop penpot
# Explicitly destroy this Mirket home's Penpot designs, accounts and owned volumes:
mirket package stop penpot --purge-data
```

Ordinary stop retains editable service data. Saved package selection is replayed
by `mirket update`; process activation remains explicit. A frontend port can be
selected with `mirket package start penpot --port 19080`. The service can also be
opened locally for human editing; a human edit changes the revision and requires
fresh inspection before a refinement.

## Design contract

| Tool | Input | Result |
| --- | --- | --- |
| `penpot_create` | Typed document and distinct project output | Native service design plus `.penpot.json` handle; persisted geometry reopened and compared |
| `penpot_inspect` | Project `.penpot.json` handle | Actual file revision, object keys, hierarchy, geometry, fills and text |
| `penpot_update` | Handle, current `expected_revision`, typed keyed patches, distinct output | Cloned/refined native document; original document preserved |
| `penpot_export` | Handle, output, `png`, `svg` or `penpot`; object key for PNG/SVG | Plugin API PNG, native SVG renderer, or current backend archive export |

Use `mirket package call penpot TOOL --project /absolute/project --arguments
'JSON_OBJECT'` for CLI calls. `mirket package serve penpot --project
/absolute/project` supplies the same filtered MCP contract to the host.

The concise profile supports boards, rectangles, ellipses, closed SVG paths, text and
board flex layout. Coordinates are global canvas coordinates. Stable keys contain
letters, digits, underscores or hyphens. Parents must be earlier boards; array order sets the stacking order (later objects on top). Designs
have at most 256 objects; dimensions and raster exports are at most 2048 pixels per
side. Solid six-digit hex fills, opacity, rotation, rounded rectangles and bundled
Source Sans Pro text are supported. Font size is 8–256; weights are 400, 600 and 700.
The installed `examples/field-notes.json` is an editable editorial composition.
Images, linked resources, custom scripts, arbitrary API endpoints, external font
loading and arbitrary plugin code are outside this profile.

A `.penpot.json` file is a scoped handle, **not** the editable drawing bytes. Native
editable data resides in Penpot's database/assets volumes. Export `format:penpot`
to save a portable `.penpot` native ZIP archive; Mirket validates its manifest,
editable shape records and geometry against the observed design. Reimport of the
saved archive into this Podman environment has not been verified. PNG and SVG are
separate exports. PNG uses the Plugin API; SVG uses the actual native `render.html`
DOM with verified fonts, avoiding a second exporter browser. SVG export embeds the editor's bundled
WOFF fonts, preserving editable text without depending on the running service.

Output parents must already exist inside the selected project. Absolute paths,
hidden components, traversal, symlinks and hard-link aliases of the input are
refused. An existing output needs explicit `overwrite:true`; `expected_sha256`
provides a revision check. A separate private local account is derived for each
canonical project path. Moving a project changes this account scope; retain a
portable native export when transferring a design.
Private generated passwords satisfy the pinned editor's required character
classes. A rejected pending account can be repaired only after registration and
authentication succeed; working credentials and existing-account refusals retain
their original file. RPC diagnostics include bounded error codes, never password,
registration token or arbitrary response bodies.

## Runtime limits and verification

A design call starts a disposable sandboxed browser with service workers blocked
and requests constrained to the local service origin. Jobs use bounded input,
output and process time, cancel their browser descendants, and publish artifacts
only after validation. Interrupted create/refine calls can leave an unreferenced
native file in the local service; they publish no project output. Deliberate purge
removes this home's owned data. Project artifact bounds are not a Podman VM disk
quota; large ongoing service histories need normal local storage management.

The supported raster ceiling has been exercised with a 2048×2048 board at scale 1
and a 1024×1024 board at scale 2: both produced readable 2048×2048 PNGs and the
service accepted subsequent inspection calls. These representative designs used
nine objects; they do not establish the memory envelope for every 256-object
design or concurrent workload. The exporter's 1 GiB cgroup budget includes image
file cache. The verified workloads reached that budget and triggered reclaim
without an out-of-memory kill. Native adapter Node/Chromium memory and the Podman
VM are outside the service budgets, so those limits are not a whole-device bound.

The five pinned services are Penpot frontend/backend/exporter 2.18.3,
PostgreSQL 15.14 and Valkey 8.1.3, each pinned to a Linux ARM64 manifest.
They require no display server. The desktop's managed Node and Chromium design
adapter remains native: it connects to the published loopback service and enforces
the same bounded typed operations. A container browser reaching that loopback origin
has not been established as equivalent; no broader network route is advertised.
The registered Plugin
API workflow does not require an upstream MCP or admin-console service.
The browser driver pins Playwright Core 1.61.1 and reuses Mirket's managed Node and
Chromium. No npm install, lifecycle hook or runtime download helper runs. Source
commit, manifest digests, integrity values and retained notices are recorded in
`registry/package-locks/penpot.json` and `docs/licenses/packages/penpot/`.

The editor's bundled fonts work headlessly. Other fonts and native multimedia or
external assets are not advertised. macOS Intel, Linux ARM64/x64 and Windows service/adapter activation have not been
verified. A Linux image pin alone does not establish those installation targets. Installation checks do not
establish fresh desktop-client activation or human visual approval. Inspect the
saved exports before accepting a design.
