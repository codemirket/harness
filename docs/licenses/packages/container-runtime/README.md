# Linux container runtime provenance

Mirket prepares local Linux ARM64 and x64 package images from the approved,
digest-pinned Microsoft Playwright 1.61.1 Ubuntu Noble system-library image.
The package adapters, Node 24.21.0 and Chromium 155 remain separately pinned in
Mirket's package definitions. Image preparation does not execute the upstream
Dockerfile, npm, pip or browser installation helpers. The upstream Dockerfile is
retained as provenance for the already built base layers.

The exact OCI index, platform manifests, configuration digests and compressed
layer sizes are recorded in `registry/package-locks/container-runtime.json`.
Microsoft Playwright's Apache-2.0 license and NOTICE are retained here. The OCI
base also contains Ubuntu packages, fonts, browsers and Node; their package
copyright files must stay with any copied system-library closure. OCI digests
pin bytes and do not establish license completeness or runtime acceptance.

Motion Canvas uses esbuild 0.18.20; HyperFrames uses esbuild 0.25.12. Their exact
Linux ARM64/x64 npm artifacts are recorded in the lock. Both releases use the
same MIT license text, retained in `ESBUILD-LICENSE.md`.

## Linux media encoder

FFmpeg and FFprobe 9.0.2 are the approved Martin Riedl Linux release builds:

- ARM64: https://ffmpeg.martin-riedl.de/info/detail/linux/arm64/1789931697_9.0.2
- x64: https://ffmpeg.martin-riedl.de/info/detail/linux/amd64/1789931100_9.0.2
- FFmpeg source commit: `946fcce07b6dcd0331c8cc609192aeff5e1924f8` (`n9.0.2`).

The retained architecture-specific version reports list every enabled codec and
build option. These binaries enable `--enable-gpl`, `--enable-version3` and
`--enable-nonfree`, including DeckLink. The reported configuration is **not a
redistributable GPL-only build**. Mirket prepares these approved binaries locally
for personal execution; do not publish or redistribute images containing them.
Retaining license texts does not remove that restriction. The upstream FFmpeg
license description and GPLv3/LGPLv3 texts are retained alongside the build
reports. Corresponding-source and component license completeness would require
separate review before any different, redistributable build is published.

Mirket uses these Linux media encoders only in its reviewed Podman render jobs.
The Rust CLI, desktop MCP launchers and native application packages keep their
own execution paths.
