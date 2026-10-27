# Video Sharp dependency review — 2026-10-26

**Sharp is absent from the inspected producer graph and from the observed
import/configuration/job-creation phase. Its removal from a released video
payload remains unqualified.** Two bounded Node-only controls passed with the
unchanged HyperFrames 0.8.197 packages: one retained Sharp, while the other
physically omitted its entire six-package subtree and refused its resolution.
Neither control called `executeRenderJob`, captured a browser frame, or encoded
media. Keep the selected runtime and release admission unchanged.

This narrows the optional-dependency question in the
[payload review](m3-video-payload-release-review.md). The
[video route](video-export-route.md) owns the operation and fidelity contract;
the [native source review](video-native-source-closure.md) still owns the
Sharp/libvips obligations if that artifact is ultimately shipped. This review
does not transfer the separate Design/Raster candidate's source or replacement
qualification to the original video libvips binary.

## Exact source boundary

The current [video adapter](../adapters/src/video-export.ts) selects the private
media/browser paths before dynamically importing `@hyperframes/producer`. It
then calls `resolveConfig`, `createRenderJob` and `executeRenderJob`. The selected
configuration uses screenshot capture, one worker, software rendering, no
browser pool and no streaming encoder.

The prior TypeScript AST graph was reused only after all **6,017** selected
runtime files were rechecked against its recorded installed-file manifest and
adapter identities. It contains 961 source-file nodes, including literal
dynamic import/require targets; it is not a control-flow proof. In that graph:

- Producer imports Studio's `manual-edits-render-script` and `encoder-options`
  subpaths. Those resolve exactly two helper files and two shared chunks.
  Studio's root `dist/index.js` is absent.
- No import/reexport/literal-require edge reaches `sharp`, `@img/colour`,
  `detect-libc`, `semver`, or either selected `@img/sharp-*` package.
- The inspected Studio root bundle contains the lazy `import("sharp")` at line
  5051, in its JPEG thumbnail route. That route verifies JPEG input, rotates,
  resizes within 240×135 and re-encodes JPEG. It is separate from the two helper
  subpaths imported by producer. The package declares Sharp optional.
- The lock-closure's only incoming edge to Sharp is Studio's optional
  dependency; all incoming edges to the other five selected instances originate
  inside that subtree. No other producer-closure consumer shares these
  particular instances.

The earlier qualification's separate pixel-inspection script imports Sharp.
That observation establishes the inspector's use, not a dependency of the video
worker. Conversely, literal graph absence does not cover computed imports,
configuration-file execution or an actual render's control flow.

## Frozen private copies and identities

No assembled current private video package existed in the earlier payload
receipt: it explicitly described a source-only skeleton. The older
`motion-video-qualification-20261025/node_modules` candidate has the same
HyperFrames version but predates the Mirket policy patches. It was preserved.
This check instead copied the exact current conservative closure into two new
private directories, without npm, lifecycle scripts, installers or downloads.

The lock is **132,863 bytes**, SHA-256
`c4c72361ef9a83d2e030e6e2338caa66e50cdd755571db794eca9587808dad8b`.
All **86 distinct published archives**, totaling **24,361,214 compressed bytes**,
were rehashed against the earlier retained archive inventory. Its 88 installed
package instances have 6,012 regular files; the compiled video adapter, root
package metadata and policy receipt add five files. The original installed-file
manifest remains SHA-256
`2ddcdb87fc05d7838e018380ea929a29b650fdff947f7d80ed3050a0bef03f73`.

| Private control copy | Package instances | Regular files | Bytes | File-manifest SHA-256 |
| --- | ---: | ---: | ---: | --- |
| Unpruned | 88 | 6,017 | 85,158,415 | `ff7540e899c9c86b72dfbc4a361eb812378cc23f50bb77b1d22353519cba56ae` |
| Entire Sharp subtree absent | 82 | 5,892 | 65,459,748 | `589fba4abecfd7d80966e22330cb914ddf01c2abc6025c58ed3467f64de6c012` |

The omitted **125 files / 19,698,667 bytes** belong to these exact instances:

| Package | Version | Files | Bytes |
| --- | --- | ---: | ---: |
| `sharp` | 0.35.5 | 47 | 962,168 |
| `@img/colour` | 1.1.0 | 6 | 73,264 |
| `detect-libc` | 2.1.2 | 8 | 26,437 |
| `semver` | 7.8.5 | 53 | 101,065 |
| `@img/sharp-darwin-arm64` | 0.35.5 | 5 | 292,231 |
| `@img/sharp-libvips-darwin-arm64` | 1.3.4 | 6 | 18,243,502 |

Every retained file's size/hash was checked before and after the probes; final
file modes also match the source. Both copies have no symlinks. These are
private control directories, not new selected components, installed releases or
recipient archives. The original adapter files, policy patches, lock,
admission proposal, older candidate and user installation were unchanged.

## Bounded execution result

Both probes used macOS 26.5.2 ARM64 and the explicit private **Node 24.22.1** at
`reinstall-qualification-20261026/same-home-reinstall-1ly03xa3/source/runtime/node/bin/node`.
Its 122,545,856 bytes retain SHA-256
`6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`.
The preflight observed 50% memory free and 47,698,227,200 bytes disk free.

Each sequential process had a 15-second outer timeout, 256 MiB V8 heap limit
and a sampled 512 MiB RSS abort guard. The RSS guard is not an atomic OS memory
limit. Node permissions denied addon/child capability and restricted filesystem
reads to its copy and control script. Module hooks refused non-file and
out-of-copy modules; global search paths were disabled and the inherited
environment was cleared. A fixture-only macOS profile denied network, process
forks and writes. Additional instrumentation recorded/refused `dlopen`, native
module loads, child-process APIs and `fetch`. None is a product policy change.

The discriminating controls were:

1. In the unpruned copy, `createRequire(...).resolve("sharp")` successfully
   resolved its actual `dist/index.cjs` without loading it.
2. In the absent copy, all six package directories were confirmed missing.
   Resolution of each package name and a direct Sharp module URL was rejected
   by the explicit denial hook. These seven expected denials occurred before
   the producer phase.
3. Each copy then imported the unchanged `dist/video-export.js` and producer
   entry, called the same `resolveConfig` options as the adapter, and created
   `mp4`, `webm`, `gif` and `png-sequence` jobs. All four jobs had `queued`
   status. `executeRenderJob` was deliberately not called.

Both processes exited zero. Their producer-phase traces are identical after
normalizing only the private root: **1,512 resolution events, 656 load events,
650 unique loaded module URLs**, including the same four Studio helper/chunk
modules. This phase produced **zero Sharp resolution/load edges, denial events,
native-addon/`dlopen` attempts, child API attempts or `fetch` attempts**. Node's
shared-object reports contain the explicit Node executable and macOS system
libraries, with no Sharp addon or libvips library. These observations establish
absence at this phase, not absence throughout a completed video export.

The outer runs took 0.460 and 0.392 seconds respectively; maximum reported RSS
was 143,280 and 140,544 KiB. These are bounded fixture observations, not video
performance measurements. No browser, FFmpeg/ffprobe, native build, upstream
helper or publication ran.

## What removes the video Sharp/libvips gate

The 5,892-file copy is a concrete candidate for the next qualification. Before
omitting Sharp from a selected video component:

1. Bind that exact inventory, the eight policy patches and explicit qualified
   Node/Chromium/media tools into a disposable installed layout. Run actual
   offline exports for MP4, WebM, GIF and PNG-sequence tar, with local fonts,
   JPEG/PNG input assets, supplied audio and caption inputs.
2. Keep all six packages physically absent, refuse resolution outside the
   component, and retain total Sharp-denial controls throughout each real
   worker. Record module/file and native-library loading. Recheck the installed
   configuration loader, final output content, cancellation, source preservation
   and owned cleanup. Import success and a queued job are insufficient.
3. If those results support omission, select and inventory the reduced video
   component explicitly and reconcile its actual recipient notices/source
   packet. A manifest edit alone does not establish recipient delivery, release
   rights, authentication or acceptance. Separate platform and capability gates
   continue to apply.

If Sharp remains shipped, its original full libvips stack still needs its own
complete source/notice/replacement/recipient qualification. Alternatively, a
separately selected shared native image artifact needs matching video behavior
and component qualification. The reduced Design/Raster artifact's evidence does
not clear the different original full-feature video library. This check neither
chooses that substitution nor ships a second native image stack.

Private evidence is retained under
`~/.codex/task-state/harness-rewrite-implementation/video-sharp-dependency-review-20261026/`:
`preflight.json`, `static-and-copy-verification.json`, both exact file manifests,
both complete resolution/load/shared-object traces, control sources and the
archive-bound `final-receipt.json`. The receipt is **48,935 bytes**, SHA-256
`902258ec41eea902abbf3e1b574888b2f1b72fabe45f63fa8d2a0433f42f83cc`.
It retains all 86 archive identities, source/fixture identities and the observed
limits. The repository change for this assignment is this report only.
