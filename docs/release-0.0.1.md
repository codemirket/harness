# Mirket 0.0.1 — macOS ARM64 candidate

This macOS ARM64 prerelease accepts **macOS 26.0 or newer** at setup without a
major-version ceiling. Its current native qualification is on **macOS 26.5.2
(25F84), ARM64**. No macOS 27 host was available to verify runtime behavior;
installation eligibility is not a macOS 27 support claim. Mirket uses external
Pi 1.4.0 as its sole agent host. The archive contains
one Rust core, one compiled TypeScript Pi package, private Node 24.22.1, selected
manageable tools, project license, original third-party notices and required
source packets.
It contains no Chrome, Playwright or Raster executable payload.

This rerelease retains the selected skill-catalog, OpenPencil and HyperFrames
methods. It removes the exact `26.5.2` setup gate, and its asset name identifies
the architecture rather than the OS patch version. The native core and compiled
Pi extension were rebuilt from the current source and skill catalog. The
[qualification](qualification.md) separates checks on this exact package from
earlier evidence; no cross-platform result is implied.

The build-input guard, locked offline Rust tests (**446 pass, 4 ignored**),
clippy, formatting and release build passed. The private-node TypeScript build
and process suite (**145 pass, 2 skipped**), Python suite (**189 pass, 1 skipped**),
explicit-path MCP smoke, package inventory/roundtrip, **16** assembled Pi
checkpoints, Pi CLI/RPC/TUI and byte-exact offline image transport passed in
disposable state. Three fresh first-tool samples (**3234.666, 1789.684,
1761.919 ms**) met the **5000/2500/2500 ms** first/repeat gate without a
cache-flush or percentile claim. The extracted archive passed a standalone
fresh installation and an old-to-new migration in separate private homes;
unrelated settings/files and the previous package were preserved. These checks
do not establish macOS 27 behavior, new-release video playback/quality or
live-provider image interpretation. All 305 catalog skill cases remain planned.
Optional Chrome and Playwright payloads remain external; Raster remains excluded.

## Assets and exact identity

The release contains seven assets:

- `mirket-pi-v0.0.1-macos-arm64.tar.gz` — the complete Pi package.
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

This rerelease replaces the prior `macos-26.5.2-arm64` archive with one
`macos-arm64` archive. The standalone scripts, host lock and license notice
remain byte-identical to the previous release. Earlier outcomes and failures
remain in [qualification](qualification.md).

The archive is **462,061,639 bytes**; its SHA-256 is
`9115c5645ec095b539adc0ec4f6a57792988cd5088076ff020d8864b727adf1c`.
The previous archive was **461,000,129 bytes**, SHA-256
`5c0f99fe443d66edd77776f5c72d7009743a35d366a8906b1e00df6fcd7eff39`.
The package inventory SHA-256 is
`4631a5cf357401a6b07114521980d909174b5fb0af46c3fe5d8ca5886bd0ab1b`.
The native release ID is
`sha256:144e15c9ef8cd7eb420ff0b9b95c41c5bf3ee3fbd3c21178492abfe8894b1f3b`.
The native executable SHA-256 is
`b45e05e652e257519766ded41d86e4d4de1e0dfe5246da036717bef4f6f2d518`.
The migration helper SHA-256 is
`c435719955cb6e80a9fcdeb9d130f9b3b3850a2777988491a211bf171e2ac25c`.
The six-entry `SHA256SUMS` file SHA-256 is
`e3001be9b9b4879a4f06d56864be7f2544a7f91fe33467ecccd84c5fe327e76c`.
The external host lock SHA-256 is
`c1cee4255aaad64b6dea75b57d8ed484e2c89ea90fbd62cb3818f456ed0cbcfb`.
The acquisition script passed 12 focused refusal/success tests and reproduced
the exact **15,249-file** host tree from **121 unchanged official npm archives**
both from an offline cache and through direct publisher downloads.

## Install from the release assets

Download the seven release assets into a temporary directory, verify `SHA256SUMS`,
and extract the package into a new permanent directory. Keep earlier extracted
packages and hosts in place; existing installations bind their exact paths. For a first installation,
acquire the exact host before running the standalone installer:

```sh
set -e
shasum -a 256 -c SHA256SUMS
install_root="$HOME/.local/share/mirket"
umask 077
mkdir -p "$install_root/releases"
chmod 700 "$install_root" "$install_root/releases"
release_root="$(mktemp -d "$install_root/releases/0.0.1.XXXXXXXX")"
tar -xzf mirket-pi-v0.0.1-macos-arm64.tar.gz \
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

An earlier 0.0.1 live migration attempt failed when the Pi settings lock was
lost. The repaired helper passed a separate old-to-new disposable migration for
this archive. A live migration still refuses while the old Mirket runtime is
active; close active Pi sessions before running it and inspect any preserved
failed state rather than replaying an uncertain update. The earlier failure and
recovery evidence remain in [qualification](qualification.md).

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
