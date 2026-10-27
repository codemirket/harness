# Sharp-free video export qualification attempt — 2026-10-26

**No Sharp-free export was qualified.** The production installed route has no
Video payload. A separately authorized, single development MCP MP4 attempt then
stopped before execution because macOS rejected the test sandbox profile. No
video operation, output or worker load trace was created. The resource guard
passed, the failed attempt is preserved, and the native lane was released.

This follows the [Sharp dependency review](m3-video-sharp-dependency-review.md),
whose two successful Node-only import/job-creation controls remain valid at
their stated boundary. This attempt does not extend them to rendering,
installation, all four output formats or cancellation. The selected runtime,
release manifests and admission state remain unchanged.

## The actual installed route is unavailable

[VerifiedRelease](../src/distribution.rs) constructs `AdapterPayload` with
`video: None` at the observed line 303. The
[installed payload resolver](../src/installed_payload.rs) can add the separately
owned Raster recipient selection; it has no Video override. Production
[`serve`](../src/main.rs) obtains its payload from that verified release path.
The current release component inventory contains no admitted Video component.

The existing [video MCP test](../tests/video-mcp-smoke.mjs) instead uses
`mirket dev serve` with explicit private Node, adapter, Chromium and media
paths. A successful result through that command would establish development
integration only. Making production installed Video available requires an
owning component/schema/inventory/loader change and its own disposable
installed qualification. Those runtime and manifest changes were outside this
assignment; no override or admission bypass was added.

## Exact prepared inputs

The candidate was the existing frozen private `sharp-absent` copy from the
dependency review: **82 package instances, 5,892 files, 65,459,748 bytes**.
Its exact file-manifest SHA-256 remains
`589fba4abecfd7d80966e22330cb914ddf01c2abc6025c58ed3467f64de6c012`.
All retained files were rehashed before and after this attempt. Sharp,
`@img/colour`, `detect-libc`, `semver`, `@img/sharp-darwin-arm64` and
`@img/sharp-libvips-darwin-arm64` remained physically absent. The earlier
archive-bound receipt and all selected runtime identities were retained.

The proposed single export was a two-second **640×360, 12 fps, 24-frame MP4**.
Its disposable project contained the existing editable Anime.js/CSS/WAAPI
composition, local Noto font and two supplied WAV tracks, plus a task-generated
checkerboard PNG and supplied caption JSON over frames 3–20. The runner prepared
stale-source refusal, saved-file/receipt/source checks, independent full decode,
motion/asset/caption pixel checks and decoded two-tone audio checks. None of
those export assertions reached execution.

The selected existing development tools were:

| Tool | Bytes | SHA-256 |
| --- | ---: | --- |
| Mirket development binary | 86,309,272 | `d338f769e0ce95d45741dfa20dcb57d6512a9af232d11a3a003962ea668cf1de` |
| Node 24.22.1 | 122,545,856 | `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329` |
| Retained Chromium launcher | 68,688 | `18bc94c01fff4c2d9b4db67f5911e748fd1ce0f213b059e6e83d6dce3ca82324` |
| FFmpeg 9.0.2 | 9,647,448 | `8374f88f171b233c072da4acbf9058e9b563ce048d9300edabbe6036f616505e` |
| ffprobe 9.0.2 | 9,454,392 | `8c6f1213081e3805be3e2d69b9db85ff291235771b76b4f35bd64bf5ad280a1c` |

The Chromium entry identifies its existing launcher, not the complete browser
bundle or a newly qualified release artifact. These files were neither rebuilt
nor invoked by this failed attempt. Their separate source, rights, platform and
recipient gates remain in force.

A task-owned Node wrapper and preload hook were prepared to deny all six Sharp
package names and direct module paths, refuse module resolution outside the
candidate, disable native addons, record selected child invocations and retain
module/native-library evidence outside product scratch. The wrapper named the
explicit private Node and left the compiled video adapter bytes unchanged.
The hook included six discriminating denial controls. Because the outer
launcher stopped first, **none of this new instrumentation ran**.

## Resource guard passed; fixture profile failed

The run was coordinated with the Chromium worker, which held no native lane
while this attempt was active. On macOS 26.5.2 ARM64, the initial preflight at
**07:41:20 UTC** observed Normal pressure (`kern.memorystatus_vm_pressure_level`
1), **51% memory free** and **47,469,019,136 disk bytes free**. At launch the
guard again observed Normal pressure, **52% free** and **47,461,720,064 disk
bytes free**. The task had a 25 GiB free-disk floor, 2 GiB task-data cap, bounded
RPC/operation deadlines, a 256 MiB Node heap limit and sampled 512 MiB worker
RSS guard. All 20 retained resource samples remained Normal and at least 52%
free; no resource threshold was relaxed or refused.

The exact profile is preserved at:

`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/video-sharp-free-export-qualification-20261026/offline.sb`

It is **280 bytes**, SHA-256
`16820dab58ac899470af148136960b9730adf6709e4fe549075d149c72dd1ada`.
macOS rejected **line 4**:

```scheme
(allow network-outbound (remote ip "127.0.0.1:*"))
```

The exact diagnostic was
`sandbox-exec: host must be * or localhost in network address`; the launcher
exited **65**. The profile had intended to deny external network access while
allowing the producer's owned loopback server. An unaccepted profile establishes
neither that policy nor successful offline execution.

The runner retained one outgoing MCP `initialize` request and zero responses.
It never called `video_prepare` or `video_start`. The state directory remained
empty, no `worker-trace.jsonl` or `result.mp4` exists, and all seven project
inputs remained unchanged. The launcher exited; subsequent owned-path process
inspection found no remaining process. No browser, media tool or native build
ran. The native lane was explicitly released to the Chromium worker. No profile
repair or repeat attempt was performed in this work window.

## Remaining qualification

A future, separately authorized fixture retry must preserve this failure,
repair and validate the task-only profile, coordinate the native lane and pass
a fresh Normal/at-least-50%-free preflight with the same disk guard. If it
completes the development MP4, report its actual output and persistent-denial
evidence at that scope. WebM, GIF, PNG-sequence tar and cancellation remain
unexercised for the Sharp-free candidate. The production-installed Video=None
blocker remains separate even after a successful development export.

Do not remove the Sharp/libvips release gate based on this attempt. Its required
installed exports and final selected-component/recipient reconciliation remain
described in the [dependency review](m3-video-sharp-dependency-review.md) and
[video route](video-export-route.md).

Private evidence is under
`~/.codex/task-state/harness-rewrite-implementation/video-sharp-free-export-qualification-20261026/`:
the unmodified failed profile, prepared wrapper/hook/runner, exact paths and
identities, project inputs, preflight/samples, stderr, RPC frames and run result.
`final-receipt.json` is **10,345 bytes**, SHA-256
`bb2aab937aff1b0958de07ac9620488f7fb564b121a2b21c7c9da0424c8a2970`.
It binds the failed fixture and reduced candidate, records the installed-route
source identities and confirms the absence of an operation/output/owned process.
This report is the only repository file added for this follow-up; the earlier
Sharp review, README, runtime source and manifests were not changed.
