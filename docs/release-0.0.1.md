# Mirket 0.0.1 — macOS ARM64 candidate

This first release is scoped to **macOS 26.5.2 (25F84), ARM64**. Mirket uses
external Pi 1.4.0 as its sole agent host. The archive contains one Rust core,
one compiled TypeScript Pi package, private Node 24.22.1, selected manageable
tools, project license, original third-party notices and required source packets.
It contains no Chrome, Playwright or Raster executable payload.

This rerelease candidate includes bounded-memory executable hashing and updated
workflow guidance. The core was rebuilt to embed the current skill catalog.
Fresh extracted-archive setup and Pi CLI/RPC/TUI lifecycle checks passed.
First-tool samples were **2854.792, 1984.948 and 1889.654 ms**, passing the
revised budget of **5000 ms** for the first observed process and **2500 ms**
for each repeat fresh process. All three still fail the former **1500 ms**
target. Ten installed domain checks passed, including decoded D2 PNG, exact Pi
image delivery, cancellation/recovery, navigation, design and memory.
Fresh clones of the same inventoried package bytes passed 24/24 optional
browser cases and five video commands, including decoded media and running
cancellation checks. Standalone fresh and existing-managed-Pi installer fixtures
passed CLI readiness, repeat-install preservation, and the existing-route TUI
check. This remains a macOS prerelease candidate. [Qualification](qualification.md) retains historical failures and exact
limits; this revision does not establish a speed improvement or cross-platform
result.

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

This 0.0.1 rerelease replaces the native package archive and checksum file.
The standalone installer, acquisition script, host lock and license notice are
unchanged. Fresh setup, lifecycle, domain, optional browser, video and installer
checks passed. Earlier installer and route evidence is retained in
[qualification](qualification.md).

The archive is **462,041,409 bytes**; its SHA-256 is
`4bdc35bc5933f014485d22260a87d1f059e442a70aa5b54782f97c80b90d4d83`.
It replaces the same-named archive with SHA-256
`72c3c78951d4e153e0592170e1b25d50038f3ffaef91a48444f57e1d2443b106`.
The package inventory SHA-256 is
`396991cfe581e8fb3a3cf67909430ae42d6afc16027c71e27a5b5f3067ce8abf`.
The native release ID is
`sha256:26d6c76e7c25c8e690f9c6b7b4014134891c49b2c67a844a7d3c166deb834fe3`.
The native executable SHA-256 is
`38f81be0ba6261b6b194b514641483ca0938cfb445105b089eb0d7861ecb8489`.
The external host lock SHA-256 is
`c1cee4255aaad64b6dea75b57d8ed484e2c89ea90fbd62cb3818f456ed0cbcfb`.
The acquisition script passed 12 focused refusal/success tests and reproduced
the exact **15,249-file** host tree from **121 unchanged official npm archives**
both from an offline cache and through direct publisher downloads.

## Install from the release assets

Download the six release assets into a temporary directory, verify `SHA256SUMS`,
and extract the package into a new permanent directory. A rerelease uses the same
version and asset name, so keep any earlier extracted package and host in place;
existing installations bind their exact paths. Acquire the exact host before
running the standalone installer:

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
