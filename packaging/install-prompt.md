# Codex Desktop installation prompt

Use this prompt only with a published Mirket release record that supplies the
exact macOS or Windows package, installer script and SHA-256 values. This
repository has no published 0.0.1 release yet. Fill in the release location and
hashes from that record; do not derive trusted hashes from the downloaded package.

> Install Mirket for Codex Desktop from this exact release: **[release URL]**.
> Its package manifest SHA-256 is **[manifest hash]** and Mirket executable
> SHA-256 is **[executable hash]**. Download the matching package and the
> first-party installer for my OS, verify their release-record hashes, and
> inspect the package target and notices. Use the first-party local installer
> with those explicit hashes; it must install under my home `.mirket` and use
> only its verified private runtimes. Preserve unrelated Codex settings,
> project skills and files. If Mirket is already installed, inspect it first
> and use an explicit update only when the release is newer and the existing
> managed files have no drift. If the home records a completed uninstall with
> retained state, use the verified release's explicit reinstall path and
> reconcile that same candidate after any interruption; do not remove the
> retained marker by hand. After installation, run the installed
> `mirket doctor` and check the actual Codex Desktop connection with
> `runtime_status`. Report the release identity, managed skill and MCP health,
> enabled capabilities, any unqualified platform/tool path and any action that
> needs me. The first-party setup flow may acquire release-pinned bulky tools
> such as Chrome for Testing from their official publisher into a private
> `.mirket` staging directory; show their vendor terms and verify the published
> archive and complete release-manifest hashes before activation. The Mac semantic
> model and ONNX Runtime may use the same setup acquisition route. If Raster is
> admitted in the selected release, setup may acquire its six pinned Sharp
> archives from npm into private staging and verify every extracted file. Do not use my
> existing browser, publish, modify project dependencies or run upstream
> installers as part of this installation.

The macOS entry point is `scripts/install-macos.sh`. It takes an absolute
canonical unpacked release directory, `--manifest-sha256`,
`--binary-sha256` and `--user-home /absolute/home`. For an offline video Chrome
archive already obtained from the pinned publisher URL, it also accepts
`--video-chrome-archive /absolute/chrome-mac-arm64.zip` and verifies the same exact
size and SHA-256. Without that option it fetches the pinned official archive.
For a semantic release, the source retains only the pinned semantic receipt;
setup restores the selected model and ONNX Runtime before activation. Optional
`--semantic-runtime-archive /absolute/onnxruntime-osx-arm64-1.31.0.tgz` and
`--semantic-model-root /absolute/model-root` select locally supplied exact
assets; omitted inputs are downloaded from their pinned publisher URLs. The
model root contains the eight selected files at their local package paths.
For an admitted macOS Raster release, the installer accepts
`--raster-archive-root /absolute/directory` with all six exact npm archives;
otherwise it downloads the pinned archives. The current local Mac candidate
admits only its exact pinned Raster payload. The lean Mirket archive excludes those six upstream packages; setup acquires
them directly from the publisher or verifies the recipient's supplied copies.
Rebundling the native archives would require their separate corresponding-source
and notice delivery route.
Before any Raster acquisition, present the selected
[Sharp license](https://github.com/lovell/sharp/blob/v0.35.5/LICENSE) and
[libvips third-party notices](https://github.com/lovell/sharp-libvips/blob/v1.3.4/THIRD-PARTY-NOTICES.md),
including the listed LGPL components. The release record must distinguish publisher acquisition from bundled
components and retain the selected notices. Bundled FFmpeg must have its
matching source and notices available alongside the runtime package.
`--update` explicitly
selects the update path. After a completed uninstall, `--reinstall` starts the
exact-source same-home preservation path; `--reinstall-reconcile` resumes that
same candidate after an interruption. Neither takes `--project-root`. The
script runs the release's own CLI. Retain an external verified copy of the
release and installer through uninstall or reinstall recovery;
`--uninstall-reconcile` can resume an interrupted removal if the installed
launcher is gone. The release record must independently authenticate the
installer script itself. The Windows entry point is `scripts/install-windows.ps1`
with `-Source`, `-ManifestSha256`, `-BinarySha256`, optional `-UserHome` and
`-ProjectRoot`, optional `-VideoChromeArchive` for the pinned offline ZIP, plus one of `-Update`, `-Reinstall`,
`-ReconcileReinstall` or `-ReconcileUninstall`. It has passed a source
parse on macOS but no native Windows
execution. Both Windows architectures require native qualification before this
prompt may claim that target ready.
