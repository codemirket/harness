# Mirket 0.0.1 — macOS ARM64 candidate

This first release is scoped to **macOS 26.5.2 (25F84), ARM64**. Mirket uses
external Pi 1.4.0 as its sole agent host. The archive contains one Rust core,
one compiled TypeScript Pi package, private Node 24.22.1, selected manageable
tools, project license, original third-party notices and required source packets.
It contains no Chrome, Playwright or Raster executable payload.

This rerelease adds `/mirket-project init` and `/mirket-project status`, project
Pi registration, and updates to managed project guidance after successful use of
selected Mirket skills and tools. The native core and compiled Pi extension were
rebuilt from the current source and skill catalog. This remains a macOS
prerelease candidate. [Qualification](qualification.md) separates checks on this
exact archive from earlier source and package evidence; no cross-platform result
is implied. Fresh extracted-archive Pi CLI/RPC/TUI project onboarding, memory,
image delivery, cancellation and ten bundled-domain checks passed. Three
first-tool samples (**1937.586, 1921.059, 1908.535 ms**) met the unchanged
**5000/2500/2500 ms** first/repeat gate without a cache-flush or percentile
claim. The explicit existing-package helper passed 10 focused tests, a real
disposable old-to-new native migration and a JS-only same-native-ID migration;
an initial old-package setup timeout under concurrent staging remains recorded.
Fresh full-package native checks passed bundled design/navigation/memory,
separate optional browser and video Chrome bindings, the Playwright peer probe,
decoded video/audio, reedit, refusal and cancellation. Those optional payloads
remain external. An initial full setup timeout under concurrent staging and a
video fixture input mismatch remain in the private qualification record; their
corrected sequential/fixture runs passed without changing product gates.

## Assets and exact identity

The release contains seven assets:

- `mirket-pi-v0.0.1-macos-26.5.2-arm64.tar.gz` — the complete Pi package.
- `install-pi.py` — the standalone installer for the verified package and exact
  acquired Pi host.
- `migrate-pi.py` — an explicit upgrade helper for an existing Mirket package;
  it verifies the old and new bindings and preserves the old release.
- `acquire-pi-host.py` and `host-lock.json` — obtain the exact external Pi
  1.4.0 dependency tree directly from its pinned publisher archives. They do
  not run npm, installers or lifecycle hooks.
- `LICENSE` — the MIT permission notice for the standalone first-party scripts.
- `SHA256SUMS` — SHA-256 for the six payloads above. Check it before extraction
  or running any script.

This 0.0.1 rerelease replaces the native package archive and checksum file and
adds the migration helper. The standalone fresh installer, acquisition script,
host lock and license notice are unchanged. Earlier installer and route evidence
is retained in [qualification](qualification.md).

The archive is **460,983,272 bytes**; its SHA-256 is
`780eba5b9624063d4c3bde06ffc9f46132c152c705203370b7109a327fc940ea`.
It replaces the same-named archive with SHA-256
`4bdc35bc5933f014485d22260a87d1f059e442a70aa5b54782f97c80b90d4d83`.
The package inventory SHA-256 is
`ee51700253ad7de510d9c4a6d32a837b3e65b2b861f212a28113119b4b4b4984`.
The native release ID is
`sha256:d2aa0c11ff58234c733f8f329047cdbf47c6f5341fbd866b9af712a9ba86c1fc`.
The native executable SHA-256 is
`b8256d8061ab8224b26c94a70d59696c3052a7435cf3de023f0fe4f863629e7d`.
The migration helper SHA-256 is
`296aa33c4f4e457faf18627217d1d4c1ddc3b6d18e5967f052a6074d149b49c5`.
The six-entry `SHA256SUMS` file SHA-256 is
`3cea3a5dc1aa3964573e210bd2cf3c5354a4628647028e2964a3eafa7c80a50d`.
The external host lock SHA-256 is
`c1cee4255aaad64b6dea75b57d8ed484e2c89ea90fbd62cb3818f456ed0cbcfb`.
The acquisition script passed 12 focused refusal/success tests and reproduced
the exact **15,249-file** host tree from **121 unchanged official npm archives**
both from an offline cache and through direct publisher downloads.

## Install from the release assets

Download the seven release assets into a temporary directory, verify `SHA256SUMS`,
and extract the package into a new permanent directory. A rerelease uses the same
version and asset name, so keep any earlier extracted package and host in place;
existing installations bind their exact paths. For a first installation,
acquire the exact host before running the standalone installer:

```sh
set -e
shasum -a 256 -c SHA256SUMS
install_root="$HOME/.local/share/mirket"
umask 077
mkdir -p "$install_root/releases"
chmod 700 "$install_root" "$install_root/releases"
release_root="$(mktemp -d "$install_root/releases/0.0.1.XXXXXXXX")"
tar -xzf mirket-pi-v0.0.1-macos-26.5.2-arm64.tar.gz \
  -C "$release_root"
python3 acquire-pi-host.py \
  --package "$release_root/package" \
  --lock host-lock.json \
  --output "$release_root/pi-host"
python3 install-pi.py \
  --package "$release_root/package" \
  --host "$release_root/pi-host" \
  --install-root "$install_root"
pi
```

`--install-root` defaults to `$HOME/.local/share/mirket`; it must already be
private (`0700`) and contain the permanent package and host paths. `--host` is
the acquisition output containing `node_modules`. If standard `$HOME/.pi/agent`
exists, the helper
registers Mirket there and makes that directory mode `0700` if needed. Otherwise
it creates a fresh private `<install-root>/agent`. Authentication, sessions,
managed files and unrelated settings are preserved. Unsupported layouts fail
closed. The supported existing managed Pi 1.4.0 launcher is `$HOME/.local/bin/pi` pointing
to `../../.pi/agent/bin/pi`. Only that outer symlink is backed up and replaced;
its managed target stays intact. Unknown launcher layouts are refused. The new `pi` launcher uses
the selected agent directory, package-private Node and exact acquired host without
searching PATH or silently falling back to another executable. Ensure
`$HOME/.local/bin` is on the shell's PATH to invoke that launcher as `pi`.

The helper records backups and checks native health and Pi package/MCP registration before
replacing the launcher. See [setup](setup.md) for failure recovery and the
package's lower-level fresh-agent setup.

For an existing Mirket installation, use the verified `migrate-pi.py` asset and
the [explicit upgrade procedure](setup.md#upgrade-an-existing-mirket-installation).
The fresh installer refuses to replace a different registered package path.

Keep the new release directory, extracted package and host tree in place: Pi
settings bind the package path, and Mirket verifies the host and native bytes. The
ordinary managed Pi 1.4.0 installation may have a different transitive closure and bundled entry;
the release's exact host check will refuse it. The host acquisition script
verifies each archive's lock integrity and the complete package-bound host
inventory before making the private tree available.

In Pi TUI, ask Pi to run the `mirket_doctor` tool once, then run
`/mirket doctor`. Before that tool runs, the slash command can show
`Awaiting actual runtime_status identity`. `/mirket setup` offers **Install pinned
version**, **Use installed tool** and **Skip** for video Chrome, a separate
browser QA Chrome and the optional Playwright MCP peer. Bundled tools work when
all three are skipped. No tool downloads on first use or searches PATH during a
job. The selected browser pair needs `/mirket-browser open
http://127.0.0.1:PORT` and an explicit scoped TUI confirmation before use.

The browser peer remains external-only because one embedded dependency declares
MIT without supplying its license text. Raster/Sharp remains excluded. FFmpeg's
source and notices are included with the binary, although its replay recipe has
not been freshly executed. The browser proxy and Mirket-owned processes are not
an OS sandbox. Other platforms and live-provider image interpretation need
separate qualification.
