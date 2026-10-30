# Mirket 0.0.1 — macOS ARM64 candidate

This first release is scoped to **macOS 26.5.2 (25F84), ARM64**. Mirket uses
external Pi 1.4.0 as its sole agent host. The archive contains one Rust core,
one compiled TypeScript Pi package, private Node 24.22.1, selected manageable
tools, project license, original third-party notices and required source packets.
It contains no Chrome, Playwright or Raster executable payload.

This published rerelease carries the focused skill-catalog repairs and selected
OpenPencil and HyperFrames methods. Pi profile selection now excludes inherited Mirket
skills under `autoload:false`; doctor names selected skills whose required
native routes are unavailable. Browser guidance matches the optional local
peer's actual scope, raster remains conditional, and source-only Video authoring
is separately described. The native core and compiled Pi extension were rebuilt
from the current source and skill catalog. This remains a macOS prerelease
candidate. [Qualification](qualification.md) separates checks on this exact
package from earlier source and package evidence; no cross-platform result is
implied.

Locked offline Rust tests passed (**446 pass, 4 ignored**), TypeScript process
tests passed (**144 pass, 1 skipped**), and the Python suite passed (**183**).
The build-input guard, clippy, formatting, release build, catalog/route audits,
packaged MCP smoke, Pi CLI/RPC/TUI, image delivery, six paired offline routing
cases, and an installed Pi Video journey passed with disposable state. Three
fresh first-tool samples (**4125.649, 1845.072, 1847.170 ms**) met the
**5000/2500/2500 ms** first/repeat gate without a cache-flush or percentile
claim. The Video journey independently decoded two-second H.264/AAC output,
observed changed-source pixels and refusal/cancellation cleanup. Separate
packaged-core evidence observed Chrome running before cancellation. The routing
cases used a deterministic provider and do not establish model skill utility;
all 305 catalog cases remain planned. Optional Chrome and Playwright payloads
remain external, and Raster remains excluded.

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

This 0.0.1 rerelease replaces the native package archive. Its subsequent
migration-helper repair leaves that archive unchanged. The standalone fresh
installer, acquisition script, host lock and license notice are unchanged.
Earlier installer and route evidence is retained in
[qualification](qualification.md).

The archive is **461,000,129 bytes**; its SHA-256 is
`5c0f99fe443d66edd77776f5c72d7009743a35d366a8906b1e00df6fcd7eff39`.
It replaces the same-named archive with SHA-256
`780eba5b9624063d4c3bde06ffc9f46132c152c705203370b7109a327fc940ea`.
The package inventory SHA-256 is
`c060c4fca31e8f31995067ada463996b3f4634d0b872005bde8403b1b1927e06`.
The native release ID is
`sha256:1ed77a59a1a2e881ab50a321d96172815d8b84059444633176d26b14459ab89a`.
The native executable SHA-256 is
`adb2bd0dd33d4a533189b264c42008be0d2dbece4a74ba3584aeed55eb28c048`.
The migration helper SHA-256 is
`c435719955cb6e80a9fcdeb9d130f9b3b3850a2777988491a211bf171e2ac25c`.
Its lock-renewal repair separately passed **46 focused tests** and the
**190-test Python suite**, with no skips; the archive was unchanged.
A disposable old-to-new native migration passed with canonical memory,
authentication, unrelated settings and skill filters preserved. The failed live
attempt remains recorded separately in [qualification](qualification.md).
The six-entry `SHA256SUMS` file SHA-256 is
`0fbf531a2f0cfd231ebd314758e4416fd23e43d27bdfb4c588460850751f78f7`.
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

The first live attempt to install this rerelease with the earlier migration
helper failed after publishing the new installation because its `settings.json.lock` disappeared
during the long operation. A subsequent exact `install-pi.py` rerun reported
`already-installed`, and native doctor reported healthy. This establishes the
resulting live installation and recovery check; it does not count the failed
migration-helper invocation as a pass. The failure and recovery evidence remain
in [qualification](qualification.md).

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
