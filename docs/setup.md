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
and run `/mirket setup`. Chrome is the sole external runtime step:

1. **Install pinned** downloads Chrome for Testing 156.0.8078.73 directly from
   Google after displaying its source URL, size, SHA-256 and terms. Mirket verifies
   the archive and exact file/link inventory, installs it privately, and runs a
   local JavaScript capability probe. No upstream installer or hook executes.
2. **Use an installed executable** accepts an explicit absolute executable path,
   asks to run a disposable capability test, and records its identity. Jobs recheck
   the binding before use. Mirket does not search PATH or silently replace it.

**Skip** preserves an explicit skipped state and leaves video unavailable.
**Cancel setup** or Ctrl+C cancels unfinished work; completed choices remain.
Failures provide Retry, Skip and Cancel choices. A changed executable or package
requires setup again. Reload Pi after a completed selection, then run
`/mirket doctor` to inspect readiness. A tool listing alone does not establish it.

Node, SQLite, semantic runtime/model, D2, selected Design/OpenPencil code,
HyperFrames and FFmpeg/FFprobe are bundled. Raster/Sharp is omitted while its
native closure/source obligations remain unresolved. The short exact-version,
source and rights table is in [packaging](../packaging/README.md#runtime-choices);
the full original notices and source packets accompany the assembled package.

The [qualification](qualification.md) records this package's local onboarding
and installed-route evidence, including a failed first-tool timing gate. The
earlier package's results remain separate; they do not transfer automatically.
