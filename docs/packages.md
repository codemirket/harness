# Design package

Mirket exposes the design stack as one local MCP package, `design`. Its tools
share one read/write workspace. The host directory is the selected Mirket home's
`.mirket/execution/podman/staging/design/workspace`; every design container sees
that directory as `/workspace`. `package start` and `package status` report the
host workspace path and assigned loopback host ports. Direct tool file arguments are relative to `/workspace`, so a
single source or output path names the same file across the stack. The official
Penpot `export_shape` and `import_image` `filePath` arguments require absolute
`/workspace/...` paths in their upstream contract; they still refer to the same
shared files. Design calls
do not take `--project` or a configurable port.

## Install and inspect

For an existing installation, replace the saved package selection explicitly
with the new release binary after starting its private Podman environment:

```sh
mirket setup --target all --package design --yes
```

This reconciles the owned client bindings and saves `design` for later updates.
`package install design` also explicitly replaces the saved package selection
with this single stack and removes settings for deselected packages. Both paths
preserve unrelated client configuration. Use these commands to install or repair
the stack:

```sh
mirket package install design
mirket package start design --group core
mirket package status
mirket package probe design --schemas
```

Run `mirket package call design TOOL --arguments 'JSON_OBJECT'` using a tool name
and argument schema returned by `probe`. Inspect the selected package with
`mirket package show design`; package installation, service readiness, a tool
response and a correct artifact are separate results. The design services are
local and require the selected Podman environment. Startup does not infer a
project root or mount a caller's project directory; put authorized inputs in the
reported shared workspace before invoking a tool.

For example, place `design/tokens.json` and `fonts/Inter-Regular.ttf` in the
reported workspace, and create the `artifacts` directory there. Then compile
the canonical tokens and render a static composition:

```sh
mirket package call design tokens_compile --arguments '{"source":"design/tokens.json"}'
mirket package call design static_render --arguments '{"tree":{"type":"div","props":{"style":{"display":"flex","backgroundColor":"#fff","width":"100%","height":"100%"}},"children":"Hello"},"fonts":[{"path":"fonts/Inter-Regular.ttf","name":"Inter"}],"width":640,"height":360,"output":"artifacts/hello.png"}'
```

The output path is relative to the same workspace. Use `probe design --schemas`
for the other direct tools and the official Penpot names forwarded by the
running server.

Core starts only the tools container: tokens, raster/vector processing, static cards,
local Playwright, Typst, HyperFrames and Odiff. Enable heavier groups explicitly:
`mirket package start design --group diagrams --group penpot` (core is always included),
or `--group all`. Omitting groups reuses the saved selection. To change a running
selection, stop it first; data and workspace are retained. Disabled groups do not
block core readiness. Penpot tools appear only when its group is enabled.

Stop the service with `mirket package stop design`. The explicit `--purge-data`
option removes owned service volumes and account state. It leaves the shared
workspace, including user files and models, intact. Stop and remove are different
operations; a stopped service can retain its workspace and persisted design files.

## Route work by output

| Need | Tool domain | Required evidence |
| --- | --- | --- |
| Shared visual tokens | Style Dictionary | Compile `design/tokens.json` before any dependent render; keep CSS, JavaScript, TypeScript, Typst and Penpot token outputs in sync with that source. |
| Static composition with exact text | Satori plus resvg | Use supported Flexbox layout and supplied TTF, OTF or WOFF fonts. Inspect the saved output at its actual use size. |
| Live web page and interaction | Playwright | Inspect full DOM, exercised states and screenshots at 2x device scale. A screenshot alone does not verify an action. |
| Editable design file | Official Penpot MCP and plugin | Use the upstream `penpot/penpot` `develop/mcp` server and its forwarded tool schemas, with the plugin's persistent WebSocket. Inspect saved object identities and exports. |
| Flow, C4, sequence, Vega-Lite or Excalidraw diagram | Kroki | Preserve source nodes, labels, relationships, direction, units and branches; inspect the rendered result. |
| Fixed frame or video | HyperFrames | Run lint, snapshot, then render. Inspect representative frames and normal playback; keep editable source separate from exports. |
| Paged document or deck | Typst | Compile tokens first, then render and inspect every page. |
| Raster input preparation | Sharp | Inspect dimensions, crop, color and transparency before downstream composition. |
| Generated or imported SVG | SVGO | Optimize every SVG, then compare structure and appearance with the required semantic IDs intact. Optimization is not sanitization. |
| Editable vector artwork | SVG authoring, SVGO and resvg | Use `vector_render` for a semantic SVG and native PNG preview. Read back the SVG and inspect its preview; use licensed existing images when raster content is needed. |
| Comparable pixel QA | Odiff | Compare matched dimensions, content and state; inspect the diff and explain material changes. A low difference score is not aesthetic approval. |

Native Penpot SVG exports embed validated fonts from the owned Penpot frontend
before SVG validation and optimization, so saved files retain their typography
without a server connection. CLI scratch directories use private directories on
the shared workspace disk and are removed after completion or cancellation.
HyperFrames and Typst staging reject project assets above the 64 MiB total
before loading them into memory. Typst compiles from the prepared project tree;
absolute imports cannot read other workspace files. Its compiled token file is
copied into that tree before compilation.

The owned [tool budgets](../runtime/design/tools/tool-budgets.json) set each
design call's bridge deadline. They allow long renders, including 15 minutes
for HyperFrames and up to 90 minutes for a 64-page
Typst job with per-page inspection. The host MCP request ceiling is the largest
of these budgets; cancellation can still end an individual call earlier.

Penpot's MCP is the official upstream implementation. Inside the private design
network its MCP endpoint is `/mcp` on port 4401, the plugin manifest is served
on port 4400, and its WebSocket is on port 4402. Host access to published
services uses the assigned loopback ports reported by `package start` and
`package status`; the internal WebSocket port is not separately published.
Mirket forwards the official MCP tool names and schemas
without custom design-object wrappers. Penpot writes can persist independently of
a static export, so inspect the saved file and its preview separately.

The source order is deliberate: token compilation precedes dependent rendering;
raster inputs pass through Sharp before composition; generated and imported
SVG passes through SVGO; HyperFrames runs lint, snapshot and render; Playwright
checks web behavior and 2x appearance; Odiff checks comparable pixels. A tool
response or file hash identifies an output, but the user journey and visual result
need their own inspection.

PNG QA retains the original pixels and digest without re-screenshotting them.
Playwright captures actual DOM/SVG browser rendering at 2x and refuses screenshots
above 16 Mi pixels. `browser_screenshot` accepts up to eight local workspace
`actions` (`click`, `hover`, `focus`, `press`), `reduced_motion`, and
`animation_time_ms` for CSS/WAAPI samples. Authored JS scenes may provide
`window.__mirketSeek(milliseconds)` to seek and settle their own timeline.
Seeking checks composition; normal playback and interruption need separate tests.
HyperFrames accepts `snapshot_times`, one to eight unique timestamps in seconds,
for named beats. Every requested sample must be returned and lie within the encoded
duration; the default remains three evenly spaced samples.
Actions never target the Penpot origin. External requests and WebSockets are
blocked during capture; no arbitrary evaluation tool is exposed.

The package runs no local image-generation model. Art direction remains the
agent's work, supported by SVG creation, scene composition, UI/motion guidance
and actual rendered evidence. User-supplied images and any separately authorized
host generation tools remain outside Mirket's runtime and billing.

## Updates and recovery

Stop the design package with the currently installed release before a runtime
generation change, then run `mirket update` and explicitly start the desired
groups. Installation refuses an active incompatible generation before replacing
the executable that can stop it. The one-way state migration retains Penpot
account/file identities and volumes; it refuses database/backend image changes.

`mirket update` reapplies saved package selections from the new release. A fresh
unattended setup selects no optional package. `package status` checks the selected
generation and service state; `doctor` checks installation and configured tools.
Unexpected file or service drift needs inspection before reconciliation. Keep
credentials, downloaded models and private design files out of repository source.

Source and image pins are in [the design stack registry](../registry/design-stack.json).
Dependency locks are in [the tool runtime](../runtime/design/tools/package-lock.json),
pinned Penpot source. [Runtime notices](third-party-notices.md#runtime-packages) record the
shipped license provenance.

Host MCP timeouts derive from the longest design tool budget plus a cleanup margin. The owned routed-spawner binding allows two hours for multi-turn work; individual inference turns and design phases retain their own bounded deadlines. Existing desktop MCP sessions must reconnect to load changed bindings.

### Concurrent rendering and resource bounds

Design sessions share workspace files. Mirket publishers use an OS-held lock for
one output path across processes; competing overwrites cannot both consume the
same revision. A busy writer receives a conflict and must re-read before retrying.
Locks release on process exit. Files written outside Mirket are not coordinated
by these locks. Cold token generations are immutable: a concurrent compiler
validates and reuses the winning generation instead of failing its render.

Workspace and renderer reads use bounded file descriptors, including when a file
grows during the read. Static compositions allow at most 512 nodes, 16 images,
16 Mi aggregate requested image pixels and 12 MiB of processed raster bytes.
Odiff compares stable PNG snapshots capped at 16 Mi decoded pixels per image;
other image formats can first be prepared through Sharp.
Page/frame QA batches cap total stored PNG bytes and decoded pixels at
64 Mi each. Typst requests at most 65 pages and rejects a 65th page.
Sharp runs in an owned child with one native thread, no retained decoder cache,
a 17-megapixel/32-MiB source bound, 4096-pixel output dimensions and one frame.
A cross-process render slot rejects competing native jobs as busy before loading
renderers. Clients can retry after the current job; a busy response is not a crash.
Image preparation is sequential and identical transformations are reused within
the request. Satori/resvg and the browser client load only when their operations
need them.

The Linux design container builds a small owned `runtime-guard` helper in an
isolated compiler stage. Its subreaper owns detached renderer descendants and
cleans them on success, failure or cancellation. Its flock mode serializes file
publication. The compiler is absent from the final image; no new runtime package
is installed for these primitives. This resource lifecycle is not a sandbox or
an authorization mechanism.

Raster transforms accept PNG, JPEG, GIF, WebP and AVIF inputs; animated inputs use
the first frame. They preserve EXIF orientation, flatten JPEG transparency onto
white, and do not enlarge by default (`enlarge: true` opts in). Output formats are
PNG, JPEG, WebP and AVIF. SVG sources use `vector_render` instead of a raster decoder.
