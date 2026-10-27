# FFmpeg private paths and relocation review

Read-only review, 2026-10-26, macOS 26.5.2 ARM64. **Both selected binaries expose
the builder's account path in ordinary runtime strings.** The frozen recipient
kit additionally retains developer paths and a host identifier in historical
build/configuration evidence. No binary, archive, admission or installed state
was changed. No native build or FFmpeg execution ran.

The smallest successor policy is a fresh build from a neutral, owned build
root, with neutral source-filename mapping, followed by a deliberate recipient
evidence export. Preserve the original evidence privately. Stripping debug
symbols or hiding the banner would not remove these strings.

## Exact inputs and method

| Input | Bytes | SHA-256 |
| --- | ---: | --- |
| `ffmpeg` | 9,647,448 | `8374f88f171b233c072da4acbf9058e9b563ce048d9300edabbe6036f616505e` |
| `ffprobe` | 9,454,392 | `8c6f1213081e3805be3e2d69b9db85ff291235771b76b4f35bd64bf5ad280a1c` |
| `ffmpeg-9.0.2-darwin-arm64-recipient-source-20261026.tar.gz` | 29,434,536 | `f420e0ea8e047fa01a3ae0bcc0d4df29f97a87712ee6f72e338c49ae83c616ba` |

Inputs are under external task state
`~/.codex/task-state/harness-rewrite-implementation/ffmpeg-source-review-20261026/`:
the exact pair in `local-recipient-fixture/bin/`, plus the frozen source archive.
The earlier [final gate review](m3-ffmpeg-final-gate-review.md) verified all
65 kit inputs and all 29 local delivery files. This audit rehashed both binaries
and read the frozen archive directly without extracting or executing it.

The audit scanned whole-file ASCII printable runs of at least six bytes,
counted raw named markers, parsed Mach-O sections/load commands, used `otool -l`
and `nm -j`, and inspected the exact nested sources and generated configuration.
Marker terms were `/Users/`, the observed builder login, `task-state`,
`/opt/homebrew`, `/Applications/`, `/Library/Developer`, `/private/var` and
`/var/folders`. Exact UTF-16LE/BE `/Users/` markers were also checked. This is a
bounded private-path review, not a general credential or arbitrary-encoding scan.
This document substitutes `<builder>` for the account name and omits the host
identifier's value.

## Binary findings

Each binary contains **65 private-bearing printable strings**, accounting for
**67 raw occurrences each** of `/Users/`, the builder login and `task-state`.
One additional `/opt/homebrew` occurrence is inside the FFmpeg configuration
string. No other listed path marker, UTF-16 `/Users/` marker or captured builder
host identifier occurs in either binary.

| String class, per binary | Count | `ffmpeg` file offsets | `ffprobe` file offsets | Meaning |
| --- | ---: | --- | --- | --- |
| Compiled data directory | 1 | `0x7c9898` | `0x748771` | `<builder-root>/private/share/ffmpeg`; shared literal for `FFMPEG_DATADIR` and `AVCONV_DATADIR`. Real preset-search fallback. |
| FFmpeg configure line | 1 | `0x7d18e1` | `0x74956a` | Builder prefix, codec include/library paths and existing Homebrew pkgconf path. Configuration/banner/API metadata; not a loader instruction. |
| libvpx configure line | 1 | `0x80ded6` | `0x7883e2` | Builder codec prefix. Library build-configuration metadata. |
| Opus source filenames | 62 | `0x80e011`–`0x812739` | `0x78851d`–`0x78cc4d` | Absolute source filenames in assertion/fatal diagnostics. No source-file loading behavior was identified. |

Here `<builder-root>` is the observed
`/Users/<builder>/.codex/task-state/harness-rewrite-implementation/ffmpeg-qualification-20261025`.
The configure string contains three account-root paths, explaining why 65
strings contain 67 root occurrences.

All 65 string starts lie in `__TEXT,__cstring`: offset 8,149,152, size 328,868
for `ffmpeg`; offset 7,630,428, size 300,039 for `ffprobe`. Neither file has a
`__DWARF` segment or `__debug*` section. The binaries also contain Opus's fatal
message format and assertion text; selected Opus configuration retains
`ENABLE_HARDENING=1`. Removing diagnostic filenames must preserve hardening.

Both files have five `LC_LOAD_DYLIB` entries, all Apple system facilities:
libSystem, zlib, CoreFoundation, CoreVideo and CoreMedia. There is no
`LC_RPATH` and no developer/Homebrew dynamic-library load path. The compiler,
pkgconf and codec prefixes in the configuration strings therefore do not
establish runtime dependency on those build locations.

## Runtime defaults that need a deliberate policy

Exact FFmpeg source separates these paths from ordinary provenance:

- `fftools/cmdutils.c:924` searches legacy file presets in `FFMPEG_DATADIR`,
  `$HOME/.ffmpeg`, then the compiled data directory. `-fpre` opens the supplied
  path directly. `fftools/ffmpeg_mux_init.c:130` similarly searches
  `AVCONV_DATADIR`, `$HOME/.avconv` and the compiled directory for `.avpreset`
  files. These are distinct from libx264's named `-preset` speed setting.
  The shared helper is present in `ffprobe`, but the inspected `ffprobe` source
  has no preset caller; string presence alone does not prove execution.
- `libavutil/file_open.c:141` constructs `/tmp/%sXXXXXX` on this selected
  `HAVE_MKSTEMP=1` macOS path. That function does not consult `TMPDIR`.
  Its two in-tree consumers are the cache protocol and libxvid encoder, both
  disabled in the selected configuration. The exported helper/string remains;
  enabling those consumers later would need a separate private-temp review.
- `FFREPORT`/`-report` can direct report-file creation. It is a caller-controlled
  runtime feature, not a leaked builder path. The owning operation must control
  inherited report/data-directory variables and private HOME; this static audit
  does not establish that its launcher does so.

Relocation does not require the original developer root for dynamic loading.
It does break the meaning of the compiled preset fallback. Accepting that
fallback is reasonable only for an explicitly qualified profile that does not
offer file presets and rejects their options. A profile offering file presets
must supply approved preset data and explicit per-process `FFMPEG_DATADIR` and
`AVCONV_DATADIR` locations, with tested missing-preset behavior. A relative
`--datadir` resolved against an arbitrary working directory is not a relocation
fix. Neither policy was executed here.

## Frozen source-kit findings

Excluding the four compressed source archives, **17 supplied files** contain
the marker set: **3,095 `/Users/`**, **3,099 builder-login**, **23 Homebrew** and
**12 Xcode `/Applications/` occurrences**. These are mostly optional provenance
and generated configuration, not the recipient-relative source/build helpers.

| Kit path | `/Users/` count |
| --- | ---: |
| `selected-binaries.json` | 2 |
| `selected-build-evidence/build-commands.json` | 35 |
| `selected-build-evidence/build-input-review.json` | 3 |
| `selected-build-evidence/ffmpeg-signature.json` | 3 |
| `selected-build-evidence/final-commands.json` | 4 |
| `selected-build-evidence/final-configure.log` | 1 |
| `selected-build-evidence/final-ffmpeg-otool-L.txt` | 1 |
| `selected-build-evidence/final-ffprobe-otool-L.txt` | 1 |
| `selected-build-evidence/link-map-verification.json` | 18 |
| `selected-build-evidence/native-identity.json` | 15 |
| `selected-configuration/ffmpeg/config.h` | 5 |
| `selected-configuration/ffmpeg/ffbuild/config.log` | 2,965 |
| `selected-configuration/ffmpeg/ffbuild/config.mak` | 16 |
| `selected-configuration/ffmpeg/ffbuild/config.sh` | 5 |
| `selected-configuration/libvpx/config.mk` | 4 |
| `selected-configuration/opus/config.status` | 16 |
| `selected-configuration/x264/config.mak` | 1 |

The FFmpeg `config.log` additionally retains one 20-character host identifier
at line 9375. Private-valued shell/configuration assignments include
`BASH_ARGV`, `BASH_SOURCE`, `HOME`, `HOSTNAME`, `PWD`, `TMPDIR`, `_`, prefix,
flags and pkg-config state. They are captured configure evidence, not evidence
that the rebuilt program needs the builder's HOME. `config.status` is retained
generated executable configuration; recipient directions use the portable
`build.py`, not replay of that old script. `build.py`, `rebuild.py`, README and
the notice index have no named private-marker matches.

All **12,409 regular files** inside the four complete upstream archives were
scanned. None contains this builder login or `task-state` marker. Three generic
`/Users/` occurrences are upstream fixtures/CI configuration: one in FFmpeg's
`tests/ref/fate/mxf-probe-dnxhd` and two in x264's `.gitlab-ci.yml`. Preserve
these exact upstream archives and their source identity; those upstream paths
are not this distributor's private build evidence.

## Smallest successor-candidate closeout

1. Rebuild the unchanged selected sources/configuration in a new private,
   neutrally named root such as `/private/tmp/mirket-ffmpeg-build`; preserve the
   original candidate and evidence. Use the same explicit existing tools and
   static targets. Choose staging/prefix/data paths that contain no account,
   host or task identifier.
2. Apply component/source-relative filename mapping while retaining Opus
   hardening. The installed Apple Clang help advertises `-ffile-prefix-map`
   and `-fmacro-prefix-map`; upstream Clang documents macro/file mapping versus
   debug-only mapping. A debug-only map cannot fix these `__FILE__` strings.
   Adding a map containing the current private root is insufficient by itself:
   FFmpeg records the map argument in its configure string. Building at the
   neutral root avoids reintroducing the private path that way.
   [Clang argument reference](https://clang.llvm.org/docs/ClangCommandLineReference.html#cmdoption-clang-ffile-prefix-map).
3. Export recipient source/recipe/configuration material with a truthful
   neutral-path mapping. Omit unnecessary historical host logs or create clearly
   labeled redacted copies; retain originals privately. Do not label changed
   receipts as byte-identical originals. Recompute the successor manifest and
   kit hash; preserve exact upstream archives and all applicable notices.
4. Document generic system paths and the nonpersonal pkgconf/Xcode tool paths
   as acceptable build provenance if retained. They are not observed loader
   dependencies. Apply the explicit preset policy above; do not describe a stale
   neutral fallback as a working relocated data directory.
5. After the native lane is released, scan the successor pair and kit again for
   the original root/login/host identifiers, verify the exact new binding, and
   run source rebuild plus media/relocation/failure checks. This creates a new
   candidate requiring the [final release gates](m3-ffmpeg-final-gate-review.md),
   not an in-place byte patch or inherited admission.

## Prepared successor controls

A separate private source/control candidate is now prepared at
`/private/tmp/mirket-ffmpeg-build-20261026`, mode 0700. No native build has
started. The original pair and frozen archive were rehashed after preparation
and retain the exact identities above. All 12,409 upstream source files are
unchanged. Old host-bearing selected configuration/logs were omitted from the
successor inputs; their originals remain preserved separately.

The complete helper delta is `control/recipe.diff`: a native-lane guard and
`-ffile-prefix-map=<neutral-root>=.` in global CFLAGS/CXXFLAGS and FFmpeg's
extra-cflags. Feature selections, static targets and Opus hardening are retained.
The 43-input prebuild manifest has SHA-256
`e7a49ea81cc5219de403f3d0baa314b6effb9052d1b34ece9facd9342e256518`.
Read-only verification of its inputs, all extracted sources and four Python
control files passed; `rebuild.py --verify-only` also passed. There are no native
output directories or successor executable hashes.

The additional owner inspection pinned `src/process_video.rs`,
`adapters/src/video-main.ts`, `adapters/src/video-export.ts` and 108 HyperFrames
JavaScript files in `control/video-owner-static-review.json`. It verified the
eight required policy-patch hashes. Rust clears the environment and sets private
HOME/TMP state; video-main supplies explicit FFmpeg/ffprobe paths after the
policy check. The finite request does not expose arbitrary FFmpeg flags. The
reviewed HyperFrames chunk/streaming builders use codec `-preset`; no quoted
file-preset option was found across the scanned files. This supports retaining
the neutral compiled data-directory fallback for that inspected profile. It
does not establish an executed no-file-open result or all possible future
HyperFrames routes.

The safe current command is:

```sh
/opt/homebrew/bin/python3.14 /private/tmp/mirket-ffmpeg-build-20261026/verify_prebuild.py
```

Only after the native owner releases the lane, `run_native.py` accepts an
explicit release flag and existing pkgconf path, bounds the build to 600 seconds
and samples the 25 GiB disk floor every 250 ms. Its exact command is in the
private candidate README and `control/successor-controls.json`. It refuses
existing output trees and retains failed evidence. Actual compilation,
successor string/load-command/configuration checks and relocated media/failure
qualification remain pending. No release-package mutation, integration, signing,
publication or release admission follows from this preparation.

## Deterministic prebuild source export

A separate source-only export now contains 47 manifest-listed inputs plus its
manifest. The four exact upstream archives, signature/key and all 24 notices
remain byte identical. Historical host-bearing logs/configuration are omitted;
their private originals are preserved. The archive
`ffmpeg-9.0.2-darwin-arm64-neutral-prebuild-source-20261026.tar.gz` is 29,900,557
bytes, SHA-256
`827cf33595b814ba8544492f71eeb4f93a1aaf8c42186f0a58ec34d6a681e42b`.
Two independent deterministic serializations matched; a fresh unpack verified
all inputs, and a changed recipe was rejected before extraction/native output.
Exported nonarchive records have no observed builder-login, captured-host,
account-root or task-state marker match in UTF-8/UTF-16LE/UTF-16BE. This is a
bounded path/identity scan, not a general credential audit.

The export and receipts are under external task state
`ffmpeg-private-path-successor-20261026/recipient-source-export-20261026/`.
It is explicitly prebuild material with no new executable correspondence claim.
Actual native outputs/configuration, path-mapping effectiveness, relocated
media/failure behavior and final binary/source/notice binding still wait for
the native lane. Original kit/binary identities remain unchanged; admission
remains false.

## Built neutral successor follow-up

After explicit native-lane release, the staged controls produced a separate
successor pair and passed source/configuration/privacy, relocated media and five
real disposable Mirket/HyperFrames MCP smokes. The final source-only kit is
29,939,250 bytes, SHA-256
`f40269afa8a45026f7051fce0b78ef5e314122d436ecea7eed9f101d084f61c3`.
The original pair/kit above remain unchanged. Exact new identities, archive
verification, residual paths and open admission gates are recorded in
[the neutral successor qualification](m3-ffmpeg-neutral-successor-qualification.md).
The native lane was released before final source/evidence freezing; release
admission remains false.
