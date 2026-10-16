# Motion Canvas vector choreography

Mirket runs Motion Canvas core and 2D **3.17.2**, source commit `e735a995aa738ca682feb4204a39ee7db7b5d279`, in a private Linux Chromium job in Mirket’s Podman environment on macOS ARM64. Its actual `Scene2D`, signals, `PlaybackManager` and `Stage` produce every frame. Shared FFmpeg encodes and fully decodes the saved silent H.264 MP4 before publication. No npm lifecycle scripts, project code/configs, Vite, external services or official exporter installer run.

The editable source is a bounded version-1 JSON scene. It describes vector geometry and choreography, and stays unchanged when inspected or exported. Use Inkscape for editable illustration/SVG creation and browser composition tooling for website/UI behavior. This Motion Canvas profile produces PNGs and video; it does not export editable animated SVG, audio or transparent video, or launch the upstream timeline editor.

Install through the harness in the intended home:

```sh
mirket environment start
mirket package install motion-canvas
mirket package probe motion-canvas --project /absolute/project
mirket package serve motion-canvas --project /absolute/project
```

The four MCP operations are `motion_canvas_inspect`, `motion_canvas_frame`, `motion_canvas_verify` and `motion_canvas_render`. Inspect and verify take `{ "source": "scene.json" }`. Frame adds a zero-based `frame` and a distinct `output` ending in `.png`. Render adds an `output` ending in `.mp4`. Output parents must already exist. Existing files require `"overwrite": true`; optional `expected_sha256` protects the version being replaced. Absolute paths, traversal, symlink components and source/output aliases are refused. Publication is atomic after verification, and cancellation before publication removes staging and terminates the captured container.

For a working rich example, copy [`aurora.json`](../runtime/packages/motion-canvas/examples/aurora.json) into a project, inspect it, then export frames 0, 75 and 149 and its video. It includes pinned Inter text, a clipped gradient layer, an SVG path morph, animated gradient stops, a cubic path reveal and separate timed easing tracks. Read the resulting artifact and inspect the intended frame sizes and focal detail; successful rendering and pixel repeatability do not establish visual acceptance.

## Editable scene contract

Required top-level fields are `version: 1`, even `width` and `height`, `fps`, `frames` and `nodes`. Optional fields are `background` (hex color or null), `gradients` and `tracks`. Coordinates use Motion Canvas's centered canvas; positive x points right and positive y points down. Node rotation uses degrees; conic gradient `angle` uses radians, matching the upstream Canvas gradient API. Timeline bounds use frame integers; frame 0 is time 0, and the last exported frame is `frames - 1`.

Dimensions are even integers 64–1920, with at most 2,073,600 pixels. Frame rate is 24, 30 or 60; duration is at most 30 seconds and 900 frames. A source is at most 2 MiB, with 1–256 nodes, at most 32 gradients and 512 tracks. Individual outputs are at most 32 MiB. A job has a 90-second host deadline and an 85-second runner budget covering a bounded reusable worker, diagnostics and frame timeouts. Maximum bounds are admission limits, not a promise that every complex scene completes before its deadline.

Each node has a unique bounded `id`, a `type`, optional `parent`, and `props`. Parents must appear before children. Types use these actual upstream classes:

| Type | Upstream class | Additional properties |
| --- | --- | --- |
| `group` | `Node` | Common transforms only |
| `rect` | `Rect` | Shape properties and `radius` |
| `ellipse` | `Circle` | Shape properties |
| `path` | `Path` | `data`, `fill`, `stroke`, `lineWidth`, `clip`, `start`, `end` |
| `text` | `Txt` | Shape properties, `text`, `fontSize`, `fontWeight`, `letterSpacing`, `textAlign` |

Common properties are `position: [x,y]` or separate `x`/`y`, `scale` (number or two-number vector), `rotation` and `opacity`. Shape properties are `width`, `height`, `fill`, `stroke`, `lineWidth` and `clip`. Fill/stroke accept a hex color, null or `{ "gradient": "gradientId" }`. A clipped shape clips its children to its actual geometry. Path `start`/`end` are fractions from 0 to 1. `data` is bounded SVG path commands, not SVG markup or an asset URL; numeric path parameters must be finite within −100000 to 100000, and actual upstream parsing must succeed.

Text uses the bundled Inter Variable 4.1 font, with weights 100–900. This profile admits Latin/Turkish text and common punctuation; it refuses emoji and unsupported scripts instead of depending on host font fallback. It accepts line breaks and bounded text up to 4096 characters. Font loading must succeed before rendering.

A gradient has a unique `id`, `type` (`linear`, `radial` or `conic`), optional `from`/`to` vectors, `angle`, `fromRadius`/`toRadius`, and 2–16 initially sorted `stops` of `{ "offset": 0..1, "color": "#hex" }`. The scene uses the actual upstream `Gradient` object; shared references share its animated signals.

A track has `target`, `property`, `start`, `end`, `to` and optional `easing`. Bounds satisfy `0 <= start < end <= frames`; start is inclusive and the endpoint is reached at end. The default easing is `easeInOutCubic`; also supported are `linear`, `easeOutCubic` and `easeOutBack`. Tracks run concurrently when they affect different channels. Overlaps on the same target/property are refused, including position versus x/y. Sequential tracks inherit the previous signal state. Node tracks support the listed properties except `clip`, `fontWeight` and `textAlign`; path `data` uses upstream morphing. Fill/stroke color tracks require explicit initial and target hex colors.

Gradient tracks use `"target": "gradient:id"` and a property such as `from`, `to`, `angle`, `fromRadius`, `toRadius`, `stop.0.color` or `stop.0.offset`. Stop color interpolation uses Motion Canvas `Color.lerp`. Unsupported keys, assets, caller scripts/configs, conflicting transforms or out-of-profile values fail explicitly.

## Verification and provenance

Inspect establishes actual node/gradient state, font readiness and editable source identity; it reports at most 64 node details. Verify seeks five bounded sample frames forward and backward and compares exact PNG hashes. Render additionally decodes every saved video frame and reconciles dimensions, timebase, timestamps, H.264 codec and frame count; a video frozen while source samples change fails. PNG can preserve a transparent scene background; MP4 uses opaque YUV420p and does not preserve alpha.

[`SOURCE-INVENTORY.json`](licenses/packages/motion-canvas/SOURCE-INVENTORY.json) records the stable upstream lock hash, full declared production archive closure, archive integrity/digests, Linux compiler, Playwright reuse and Inter font. [`docs/licenses/packages/motion-canvas`](licenses/packages/motion-canvas/README.md) retains originals and explicit fallback provenance when a published archive lacks a license file. Runtime files are fixed Mirket-owned adapter code, compiled by the directly extracted ARM64 esbuild 0.18.20 CLI. Node, Chromium and FFmpeg retain shared package definitions/notices. The official Motion Canvas exporter is not installed.

The opt-in runtime contract test is `tests/packages_motion_canvas_live.rs`. It requires an explicitly marked disposable installed-package home, exercises rich actual rendering and complete video decoding, rejects convincing failure inputs, verifies source preservation, cancels a live Chromium/FFmpeg render, confirms immutable-container/staging cleanup, and checks the next request succeeds. Set `MIRKET_MOTION_CANVAS_EVIDENCE_DIR` to an existing absolute directory to retain the project and trace. Live client discovery/activation remains a separate host-session check.

The container runs without network, as UID 1000 with no capabilities and a read-only root. Only validated inputs are mounted read-only; output is retrieved and validated before publication. Chromium’s inner sandbox is disabled in this constrained profile; the Podman boundary supplies process, filesystem and network isolation. This is distinct from the native browser sandbox. See [execution environment](podman.md) for quotas, machine setup, recovery and platform limits.
