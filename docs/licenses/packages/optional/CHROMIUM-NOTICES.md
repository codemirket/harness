# Chrome Headless Shell

The unmodified browser archives contain `LICENSE.headless_shell` and `ABOUT`.
Mirket retains these files alongside the browser. The license file contains the
Chromium and bundled third-party copyright and license texts, including the
platform-specific notices. The package manifest pins each archive by SHA-256.

Version: 155.0.8059.12, the browser version in Playwright core
1.64.0-alpha-1790635538000 (`browsers.json`, revision 1247).

Source: https://chromium.googlesource.com/chromium/src/+/refs/tags/155.0.8059.12

Targets: macOS ARM64/x64, GNU/Linux ARM64/x64 and Windows x64. No native Windows
ARM64 browser archive is declared. Linux requires compatible system libraries;
Mirket does not invoke package managers or disable the browser sandbox to install them.
