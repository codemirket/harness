# Setup-acquired browser qualification — 2026-10-26

The browser build from Chromium source is stopped under the user's revised
installation choice. Mirket will keep one browser for browser workflows and
HyperFrames, but its release download need not contain the browser bytes.
The first-party setup flow obtains an exact prebuilt browser archive, verifies
its pinned size and SHA-256 before extraction, assembles it in a private staging
directory, and submits the complete, manifest-verified release to Mirket's
existing setup/update transaction. An already downloaded archive can be supplied
explicitly and receives the same verification. No user browser profile or
global app installation is selected.

The selected browser is Google's [Chrome for Testing](https://developer.chrome.com/docs/automation-and-testing/chrome-for-testing),
which is designed for fixed-version automation. The official [availability
catalog](https://github.com/GoogleChromeLabs/chrome-for-testing) lists
`mac-arm64` and `win64`, but no `win-arm64`. Windows 11 ARM64 would use the
`win64` executable through x64 emulation only after native qualification.
The executable is Chrome: [Google's Chrome terms](https://www.google.com/chrome/terms/)
apply. This is free to download, but is not an all-open-source-binary route.
The setup prompt must disclose the vendor and terms before acquisition.

| Target | Official archive | Download bytes | SHA-256 |
| --- | --- | ---: | --- |
| macOS ARM64 | [Chrome for Testing 156.0.8078.47](https://storage.googleapis.com/chrome-for-testing-public/156.0.8078.47/mac-arm64/chrome-mac-arm64.zip) | 194,932,023 | `ad7f2b49a4dc679af286b1b1b386670d93ca7868079f69012d21ba0ac8293df3` |
| Windows x64 and provisional Windows ARM64 emulation | [Chrome for Testing 156.0.8078.47](https://storage.googleapis.com/chrome-for-testing-public/156.0.8078.47/win64/chrome-win64.zip) | 209,154,801 | `a4e0d57245e167e98a9d465af1e3774d2b4ec9f04e483d19bcd97a6f63cf46dd` |

The Mac ZIP has 673 entries, 344 regular files and five internal framework
symlinks, totaling 383,539,409 uncompressed regular bytes. The Windows ZIP
has 305 regular files totaling 462,211,788 uncompressed bytes; `chrome.exe`
is PE x64 (`0x8664`). Both archives had no absolute or parent-traversing
entry names in the inspected metadata. These are local observations, not a
general ZIP extractor proof. The Mac executable reported version
`156.0.8078.47` and ARM64 Mach-O. Its app bundle has an ad-hoc signature,
and `codesign --verify --deep --strict`/`spctl --assess` refused it with
“code has no resources but signature indicates they must be present.” The
verified official archive hash, not a Developer ID assertion, is the current
byte-authentication mechanism. This trust behavior must be explicit in the
installer and release review; no Gatekeeper exception was installed.

On macOS 26.5.2 ARM64, a disposable Mirket development MCP journey with the
private Node 24.22.1, Playwright 1.64.1 and the extracted Mac executable passed
local navigation, article extraction, click, saved screenshot, lost-start
response reconciliation, temporary-profile cleanup and reaping nine observed
owned processes. The saved 1280×720 screenshot was inspected. A separate
HyperFrames/FFmpeg MP4 public MCP journey using the same browser passed source
preservation, stale-source refusal and publication checks. The local browser
evidence is under
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/browser-prebuilt-20261026`.

The Mac release inventory now binds 344 regular browser files, five links,
the Node/Playwright/Readability closure and the three authored adapter files.
The corresponding Windows x64 archive has a separate 305-file inventory;
Windows ARM64 currently selects that x64 build for native emulation tests. The
first-party installers acquire or accept only these exact archives and stage a
complete release for the existing setup transaction.

On macOS 26.5.2 ARM64, a disposable release source intentionally omitted the
browser archive and extracted tree. `install-macos.sh --browser-archive` verified
the supplied official archive, completed setup, installed the managed Codex
skills and MCP entry in a disposable user home, and `mirket doctor --json`
reported a healthy active `browser,skills` release. Its installed launcher
passed a public MCP journey with navigation, article extraction, click,
screenshot, lost-response reconciliation, reconnect and reaping nine observed
browser processes. A same-size, one-byte-mutated archive was rejected on
SHA-256 before update; doctor reported the original release still healthy.
The default HTTPS download branch installed the same release into a second
disposable user home; doctor reported it healthy with managed registration.
Evidence is under
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/browser-prebuilt-20261026/installed-route-test-20261026`.

Windows x64/ARM64 native installation, final combined browser/design/video package, four-format
installed video export, and Codex Desktop GUI activation remain release gates.
The older Chromium source and rights evidence is retained privately but is no
longer the selected release route.
