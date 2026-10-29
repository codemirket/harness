# FFmpeg and FFprobe corresponding source

The separate `native/runtime/video/bin/ffmpeg` and `ffprobe` programs use
FFmpeg 9.0.2, x264 commit `0480cb05fa188d37ae87e8f4fd8f1aea3711f7ee`,
libvpx commit `6df3ec34557879fff673706f4a1d9fbd0f3a6f0e`, and Opus 1.6.1.
This build enables GPL and x264; it is GPL-2.0-or-later. It is also based in
part on the work of the Independent JPEG Group. Applicable license and patent
texts are in `notices/`; complete source archives are in `source/`.
The source is supplied alongside the programs under GPLv2 section 3(a),
without a separate download, account, expiring offer, or additional charge.
These programs remain separate from Mirket's Rust/TypeScript implementation.

`source-files.json` identifies the selected programs and every source,
notice and original build-log file. `build/SOURCE-URLS.json` records upstream
sources. Source archive members were previously compared against all 12,409
post-build regular source files with no observed edits. This is historical
source-correspondence evidence, not a new independent build.

The original configure scripts and Makefiles controlling compilation are in
the archives. `build/*config.log` and the retained configure/build logs record
the actual flags and original machine paths. `rebuild-ffmpeg.py` transcribes
those commands into a portable sequential recipe with a new private prefix.
It is new recipient guidance; it was not the original interactive command wrapper.
No fresh execution of this replay recipe or byte-reproducible result is claimed.

On a macOS ARM64 development host with existing Xcode command tools, GNU Make
and pkg-config, explicitly select their canonical absolute paths. Start outside
the real user's home and use an absent output directory:

```sh
python3 /absolute/package/third-party/ffmpeg/rebuild-ffmpeg.py \
  --packet /absolute/package/third-party/ffmpeg \
  --output /private/tmp/new-media-build \
  --cc /absolute/clang --ar /absolute/ar --ranlib /absolute/ranlib \
  --strip /absolute/strip --make /absolute/make \
  --pkg-config /absolute/pkg-config --sdk /absolute/MacOSX.sdk
```

This explicitly requested build executes upstream configure scripts and
Makefiles; package setup does not run it. It does not download or install build
tools. The deployment floor transcribed from the selected build is macOS 26.0.
The original observation used macOS 26.5.2 ARM64, Xcode 26.6, Apple Clang 21.0.0
and the macOS 26.5 SDK. Other compilers/SDKs can produce different bytes.
Rebuilt programs require fresh capability/output/cancellation qualification
before replacing the selected pair. Codec patent rights depend on the activity
and territory; GPL source permissions do not themselves grant them.
