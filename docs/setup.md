# Set up the local Pi package

The current target is macOS 26.5.2 ARM64. Pi 1.4.0 remains an external host.
Obtain the assembled local package described in [packaging](../packaging/README.md).
For the tagged 0.0.1 archive and its exact external Pi host acquisition, see
the [release instructions](release-0.0.1.md).
Use its bound private Node executable; do not substitute a PATH runtime.

After verifying and extracting the release package and acquiring its exact host
under a permanent install root, run the standalone `install-pi.py` downloaded
with the release assets and verified against `SHA256SUMS`:

```sh
python3 install-pi.py \
  --package /absolute/mirket-install-root/releases/0.0.1-unique/package \
  --host /absolute/mirket-install-root/releases/0.0.1-unique/pi-host \
  --install-root /absolute/mirket-install-root
```

`--host` names the acquisition output containing `node_modules`. The install root
must already be private (`0700`), with both package and host beneath it at
permanent canonical paths. Extract a rerelease into a new directory rather than
overwriting a package or host used by an existing installation; the release
instructions create a unique permanent directory. `--install-root` defaults to
`$HOME/.local/share/mirket`.
The helper detects the standard `$HOME/.pi/agent` directory and registers Mirket
there. If it is absent, setup creates a fresh
private `<install-root>/agent` directory. Existing Pi authentication, sessions,
managed files and unrelated settings are preserved; the existing agent directory
is made private with mode `0700` if needed. Unsupported layouts fail closed.
The supported existing managed launcher is Pi 1.4.0 at `$HOME/.local/bin/pi`
pointing to `../../.pi/agent/bin/pi`. Only that outer symlink is backed up and
replaced; its managed target is preserved. Unknown layouts and versions are
refused. The resulting `pi`
launcher binds the selected agent directory, package-private Node and exact
package-bound Pi host. It does not search PATH or silently substitute a host.

Before publishing the launcher, the helper checks native doctor and Pi package/MCP
registration without requesting a model. The first `/mirket doctor` can show
`Awaiting actual runtime_status identity` until the actual tool has run. Ask Pi
to run `mirket_doctor` once, then repeat `/mirket doctor` to inspect verified
readiness. Original settings bytes/mode, the outer symlink and installation
phases are recorded under
`<install-root>/install-backups/install-*/`. A setup or registration failure leaves
the original launcher intact and preserves any installed state for inspection;
it does not delete the Mirket home or automatically overwrite settings during
recovery. Repeating an identical completed installation rechecks native health
and Pi registration without running setup or rewriting settings. An exact
`setup-complete` backup allows a failed post-setup check to continue without
copying the payload again. Launcher replacement excludes cooperating installers;
an unrelated process writing the same launcher at the same instant is outside
that lock's guarantee. A recognized earlier Mirket launcher is backed up and
updated after the same checks.

The 0.0.1 release has six assets: the native package archive, `install-pi.py`,
`acquire-pi-host.py`, `host-lock.json`, `LICENSE` and `SHA256SUMS`. The checksum
file covers the five payloads. `LICENSE` supplies the MIT permission notice for
the standalone first-party scripts. The current rerelease changes the native
package and startup budget; see [qualification](qualification.md) for its exact
identity, fresh outcomes and earlier failed timing evidence.

For an explicitly selected fresh agent directory, the package's lower-level
setup remains available:

```sh
/absolute/package/native/runtime/node/bin/node \
  /absolute/package/scripts/setup.mjs \
  --agent-dir /absolute/fresh-private-pi-agent-directory
```

Setup verifies package/native identities, stages the core installation, and edits
only this package's Pi resource filter. An existing Mirket home is preserved;
`--resume` applies only to an interrupted setup of this same package. It does not
import an old installation or alter unrelated package settings.

## Project setup

These commands require a local package built from the current source. The
published 0.0.1 archive predates project onboarding; its tagged artifact must be
updated before release users can use this workflow.

Once that local package is installed, start `pi` from the intended project root
and run:

```text
/mirket-project init
```

This ensures `.pi/settings.json` contains the installed Mirket package and adds
a managed Mirket section to the project's `AGENTS.md`. It explicitly enables
Mirket's extension in the project package entry so it remains loaded after Pi
reload, including when that entry previously disabled extensions. Other fields
and package entries are preserved. With no arguments, an
existing Mirket skill filter is preserved, including an unfiltered package.
If no project package entry exists, initialization selects the baseline global
skills. For an explicit selection, pass
profile names and optional exact skill IDs:

```text
/mirket-project init web-product +js-ts-navigation
```

Profiles select their baseline skills; conditional skills require an explicit
`+SKILL`. Resources stay in the installed package. No skills are copied into the
project or `.agents`, and no second selection database is created. Project Pi
settings remain the active selection source.
The package source in `.pi/settings.json` is an absolute path on this device.
Keep that settings file local; teammates should install Mirket and run project
initialization on their own devices. The managed `AGENTS.md` guidance can be
shared with the project.

The generated `AGENTS.md` section is bounded by
`<!-- mirket-project:start -->` and `<!-- mirket-project:end -->`. It records
Mirket's project guidance and observed use. Initialization opts this project
into ongoing updates from verified Mirket skill use and successful Mirket tool
calls, bound to this canonical project and runtime identity. Observed use does
not attest tool readiness,
successful output, complete project coverage or capability availability. It does
not install tools, dependencies or accounts. Keep project-specific instructions
outside the managed section.

Review both files, reload Pi, ask Pi to run `mirket_doctor` once, then run
`/mirket doctor` for actual runtime readiness. Use `/mirket-project status` to
inspect project synchronization separately. The existing `/mirket-profile
project PROFILE ... [+SKILL ...]` command remains available for skill selection.

Initialization and subsequent updates preserve existing `AGENTS.md` content
outside the managed section and unrelated Pi settings. Conflicting ownership,
changed managed content, duplicate or malformed section markers, symlinks,
hardlinks, aliased ancestors, invalid or oversized files, busy locks and
concurrent edits are refused rather than overwritten. Status also changes if
the project Pi package entry is removed or points elsewhere; automatic guidance
updates stop until that wiring is repaired. Inspect the reported
conflict and reconcile the files before retrying. Repeating initialization with
no selection arguments preserves the current selection.

Project setup does not change memory identity or discover project sources.
Canonical memory remains explicitly scoped; navigation still requires selected,
revision-bound JS/TS source snapshots.

## Optional runtime tools

Launch `pi` after using the helper, or open the explicitly selected agent directory
in Pi's terminal interface. Reload the local package and run `/mirket setup`.
It presents video Chrome, a separate browser QA executable, and the optional Playwright MCP peer. Each has three choices:

1. **Install pinned version** displays the publisher source, size, SHA-256 and
   terms before downloading. Chrome for Testing 156.0.8078.73 comes directly
   from Google. The optional peer uses the exact `@playwright/mcp` 0.0.83 and
   `playwright-core` 1.64.0-alpha-1790635538000 npm archives. Mirket verifies
   archives and selected members, installs private files, and runs a real local
   capability probe. It does not run npm, upstream installers or hooks.
2. **Use installed tool** accepts an explicit absolute executable path, or for
   Playwright an exact selected tree root containing only the twelve reviewed
   publisher files under `node_modules`. The browser QA choice also
   asks for its `.app` tree root. Mirket checks versions, exact bytes and a
   disposable local browser/MCP capability probe before storing the selected
   paths. A changed binding is refused; no job searches PATH or silently
   switches tools.

**Skip** preserves an explicit skipped state for that tool. Skipping video Chrome
leaves video rendering unavailable; skipping the QA browser or peer leaves
browser QA unavailable. D2, design, navigation and memory remain available.
**Cancel setup** or Ctrl+C cancels unfinished work; completed choices remain.
Failures provide Retry, Skip and Cancel choices. A changed executable or package
requires setup again. Reload Pi after a completed selection, then run
`/mirket doctor` to inspect readiness. A tool listing alone does not establish it.
The Playwright peer and its separate browser are absent from the default package.
The Playwright bundle includes one embedded module that declares MIT but lacks
its own supplied license text; Mirket does not redistribute that bundle.

After both QA choices are ready, use `/mirket-browser open
http://127.0.0.1:PORT` in Pi TUI. Confirm the displayed project, origin and
runtime identities. The grant lasts at most five minutes or 64 calls. The peer
permits seven navigation and inspection tools and bounded GET/HEAD requests
through an application proxy to that one local origin. Opening the page also
authorizes its subresource requests; even GET requests can change the local
application. Use `/mirket-browser status` to inspect it or
`/mirket-browser revoke` to end it. A cancelled call revokes the grant and
requires a fresh explicit open. This proxy is not an OS sandbox.

Node, SQLite, semantic runtime/model, D2, selected Design/OpenPencil code,
HyperFrames and FFmpeg/FFprobe are bundled. Raster/Sharp is omitted while its
native closure/source obligations remain unresolved. The short exact-version,
source and rights table is in [packaging](../packaging/README.md#runtime-choices);
the full original notices and source packets accompany the assembled package.
The bundled D2 route can save SVG or a bounded, directly decoded PNG without
browser onboarding.

The [qualification](qualification.md) records this package's local onboarding
and installed-route evidence, the revised startup budget and historical timing
failures. Earlier package results remain separate; they do not transfer
automatically.
