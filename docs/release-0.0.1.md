# Mirket 0.0.1 — macOS ARM64 candidate

This first release is scoped to **macOS 26.5.2 (25F84), ARM64**. Mirket uses
external Pi 1.4.0 as its sole agent host. The archive contains one Rust core,
one compiled TypeScript Pi package, private Node 24.22.1, selected manageable
tools, project license, original third-party notices and required source packets.
It contains no Chrome, Playwright or Raster executable payload.

The package is functionally qualified on this platform: fresh installed Pi
CLI/TUI, D2 PNG and image delivery, video, memory/navigation/design and 24/24
optional browser cases passed. The **1.5-second first-tool startup gate failed**
with 4.097, 1.911 and 1.839 second samples. This is an initial prerelease candidate,
not a performance-qualified or cross-platform release. Full results and the
tested artifact identity are in [qualification](qualification.md).

## Assets and exact identity

The release contains six assets:

- `mirket-pi-v0.0.1-macos-26.5.2-arm64.tar.gz` — the complete Pi package.
- `install-pi.py` — the standalone installer for the verified package and exact
  acquired Pi host.
- `acquire-pi-host.py` and `host-lock.json` — obtain the exact external Pi
  1.4.0 dependency tree directly from its pinned publisher archives. They do
  not run npm, installers or lifecycle hooks.
- `LICENSE` — the MIT permission notice for the standalone first-party scripts.
- `SHA256SUMS` — SHA-256 for the five payloads above. Check it before extraction
  or running either script.

The current 0.0.1 rerelease adds the standalone installer and license notice and
updates the checksum file. The native package archive, host acquisition script
and host lock are unchanged. Adding the installer does not resolve the native package's failed
startup timing gate or extend its platform qualification.

The installer separately passed 29 focused tests and the 165-test Python suite.
On macOS 26.5.2 ARM64, disposable existing-Pi and fresh-Pi installs both reached
actual Pi CLI doctor `ready`; the existing-Pi route also passed a Pi TUI check.
The failed first probe and its successful recovery were retained outside the
shipped source. Other platforms remain untested.

The package inventory SHA-256 is
`275e91c346e6a80225735892b0c1cfc0fb4acdfe62eba72e37b34fe4d0592973`.
The native release ID is
`sha256:f7d5ca0b43729d222db840bba0b7e2e2c0192f0f9a763a7e119ee48f728d1800`.
The native executable SHA-256 is
`404bec4d75f68f91fec45b61a583d6902492fa1f88287b93663e4527bf27d97a`.
The external host lock SHA-256 is
`c1cee4255aaad64b6dea75b57d8ed484e2c89ea90fbd62cb3818f456ed0cbcfb`.
The acquisition script passed 12 focused refusal/success tests and reproduced
the exact **15,249-file** host tree from **121 unchanged official npm archives**
both from an offline cache and through direct publisher downloads.

## Install from the release assets

Download the six release assets into a temporary directory, verify `SHA256SUMS`,
and extract the package into a permanent location. Acquire the exact host before
running the standalone installer:

```sh
shasum -a 256 -c SHA256SUMS
install_root="$HOME/.local/share/mirket"
umask 077
mkdir -p "$install_root/releases/0.0.1" "$install_root/host"
chmod 700 "$install_root" "$install_root/host"
tar -xzf mirket-pi-v0.0.1-macos-26.5.2-arm64.tar.gz \
  -C "$install_root/releases/0.0.1"
python3 acquire-pi-host.py \
  --package "$install_root/releases/0.0.1/package" \
  --lock host-lock.json \
  --output "$install_root/host/pi-1.4.0"
python3 install-pi.py \
  --package "$install_root/releases/0.0.1/package" \
  --host "$install_root/host/pi-1.4.0" \
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

Keep the extracted package and host tree in place: Pi settings bind the package
path, and Mirket verifies the host and native bytes. The ordinary managed Pi
1.4.0 installation may have a different transitive closure and bundled entry;
the release's exact host check will refuse it. The host acquisition script
verifies each archive's lock integrity and the complete package-bound host
inventory before making the private tree available.

In Pi TUI, run `/mirket doctor`. `/mirket setup` offers **Install pinned
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
