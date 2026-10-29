# Set up the local Pi package

The current target is macOS 26.5.2 ARM64. Pi 1.4.0 remains an external host.
Obtain the assembled local package described in [packaging](../packaging/README.md).
Use its bound private Node executable; do not substitute a PATH runtime.

Create or explicitly select a fresh private Pi agent directory. Run:

```sh
/absolute/package/native/runtime/node/bin/node \
  /absolute/package/scripts/setup.mjs \
  --agent-dir /absolute/fresh-private-pi-agent-directory
```

Setup verifies package/native identities, stages the core installation, and edits
only this package's Pi resource filter. An existing Mirket home is preserved;
`--resume` applies only to an interrupted setup of this same package. It does not
import an old installation or alter unrelated package settings.

Open that agent directory in Pi's terminal interface, reload the local package,
and run `/mirket setup`. It presents video Chrome, a separate browser QA
executable, and the optional Playwright MCP peer. Each has three choices:

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
and installed-route evidence, including a failed first-tool timing gate. The
earlier package's results remain separate; they do not transfer automatically.
