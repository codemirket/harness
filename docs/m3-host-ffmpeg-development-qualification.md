# Explicit host FFmpeg candidate — Mac development qualification

Observed on macOS 26.5.2 ARM64, 2026-10-26. The user now permits an external
tool dependency when private packaging is disproportionately troublesome.
This is an experiment with an **explicitly selected executable**, not a PATH
fallback or an installed-release admission. Normal Mirket releases still
refuse Video.

The Mac had Homebrew FFmpeg 9.0.2 at
`/opt/homebrew/Cellar/ffmpeg/9.0.2_1/bin/ffmpeg` and a matched `ffprobe`.
`/opt/homebrew/bin/ffmpeg` and `ffprobe` are symlinks to those versioned paths;
Mirket's development path canonicalizes the selected files. The executable
SHA-256 values observed through those links were:

| Executable | SHA-256 |
| --- | --- |
| FFmpeg | `768609fb3f463ff9272bebac61478515f19e43c5343b0c60dde2e6230c40013e` |
| ffprobe | `b8d6a076993f4269f71ed090af524093d38a810d4481b38bc9bc1853d91b4612` |

The existing Rust/MCP/Node/HyperFrames development smoke selected this pair,
private Node 24.22.1, the same reduced Sharp-free Video component and a
development Chromium executable. Four fresh, sequential disposable jobs
passed source-revision refusal, saved artifact publication and source
preservation:

| Output | Saved bytes | SHA-256 |
| --- | ---: | --- |
| MP4 | 55,326 | `755853d167a04c36502b68506c0e4160a8ec4d2f6d00d7422186abf72eac6ad5` |
| WebM | 48,429 | `9d5908146ab801f2a113c2a9fdc8f7d4b00e2400b2c71436b0a9a03e6019c81b` |
| GIF | 174,408 | `ad4eea5bc460614c30472af4ff299f99dac8fe2e872263668c812a3e59ea249e` |
| PNG-sequence tar | 431,104 | `b0f0046bfba6d7df0e2e47c39823de26c313740912b8683e908de135e020970b` |

An initial MP4 fixture selected an HTML file that referenced `checker.png`
but did not include that asset in the smoke's selected inputs. The worker
correctly refused `missing_or_unselected_asset` and published nothing. The
four passing jobs used the original five-file fixture with no unselected
asset; the failed disposable scratch fixture was removed afterward.

The observed Homebrew FFmpeg links to its Cellar libraries and other
`/opt/homebrew/opt/*` libraries, so the executable hash alone is **not** a
complete runtime identity. A reliable installed binding would have to check
required codecs and the changing linked-library closure, then refuse drift
before a job. On Mac, the already qualified neutral private FFmpeg pair is
self-contained and simpler to keep stable; this host pair is **not selected**
for the Mac release. The development test does not qualify Windows
package-manager builds, minimum macOS 15 behavior, offline containment,
actual installed Video, or Codex Desktop GUI activation.
