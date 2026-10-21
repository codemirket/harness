# Runtime packages

> **Version scope:** This guide describes the working source following Mirket 1.0.11. The selective `package prepare --group` behavior is unreleased. Use the matching release manifest and [version guide](release-status.md); older binaries may not support every command.

`design` supplies production tools. The internal `agent-runtime` package supplies
Mem0 memory and LangGraph worker checkpoints through Mirket itself; see
[local memory and durable workers](agent-runtime.md). To retain both, install
`mirket package install design agent-runtime`. Commands below show design-only
selection and replace any previously saved package selection.

## Design package

Mirket exposes the design stack as one local MCP package, `design`. Its tools
share one read/write workspace. The host directory is the selected Mirket home's
`.mirket/execution/podman/staging/design/workspace`; every design container sees
that directory as `/workspace`. `package start` and `package status` report the
host workspace path and assigned loopback host ports. Direct tool file arguments are relative to `/workspace`, so a
single source or output path names the same file across the stack. OpenPencil
uses these same relative paths for editable `.fig` documents. Design calls do
not take `--project` or a configurable port.

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
for every installed design tool.

Bundled video production is retired. Mirket does not ship HyperFrames or an
FFmpeg encoder. Existing scenes, outputs, profiles and the `motion` workflow ID
remain available; `motion` now explicitly requires selected host tooling and is
unavailable to routed workers. It is not redirected to browser animation.
Old checkpoints retain their original bindings and cannot resume under this
changed source/tool generation. Inspect/export their state before deciding how
to continue the work; no budget reset or tool/provider substitution is implied.

Core starts only the tools container: tokens, raster/vector processing, static
cards, local Playwright, OpenPencil, Typst and Odiff. Prepare/start ordinary
diagrams with `--group diagrams`; this acquires Kroki alone for fresh selections.
Use `mirket package prepare design --group excalidraw`, then
`mirket package start design --group excalidraw` for Excalidraw input; this includes
Kroki and core. `--group all` includes the companion too. Omitting groups reuses
the saved selection. Old diagram selections migrate to `diagrams,excalidraw`,
preserving their formats. Preparation never enables a group. To change a running
selection, stop it first; data and workspace are retained. Disabled groups do not
block core readiness. Unsupported Excalidraw calls fail before rendering with an
explicit preparation/selection instruction.

Stop the service with `mirket package stop design`. The explicit `--purge-data`
option removes current service state and network. Retained legacy state and volumes
are preserved. It leaves the shared
workspace, including user files and models, intact. Stop and remove are different
operations; a stopped service can retain its workspace and persisted design files.

## Route work by output

| Need | Tool domain | Required evidence |
| --- | --- | --- |
| Shared visual tokens | Style Dictionary | Compile `design/tokens.json` before any dependent render; keep CSS, JavaScript, TypeScript, Typst and DTCG outputs in sync. Cached generations bind token bytes, the compiler/format configuration and locked dependencies; source and compiler identities remain separate in the result. |
| Static composition with exact text | Satori plus resvg | Use supported Flexbox layout and supplied TTF, OTF, WOFF or WOFF2 fonts. Inspect the saved output at its actual use size. |
| Live web page and interaction | Playwright | Inspect full DOM, exercised states and screenshots at 2x device scale. A screenshot alone does not verify an action. |
| Editable design file | OpenPencil 0.16.0 official CLI | Create/edit/reopen `.fig` files with validated upstream tools; export and inspect PNG pixels. Canonical DTCG tokens become document variables/styles. |
| Flow, C4, sequence, Vega-Lite or Excalidraw diagram | Kroki | Preserve source nodes, labels, relationships, direction, units and branches; inspect the rendered result. |
| Encoded video | Explicitly selected host encoder | Mirket has no bundled encoder. Preserve editable source; inspect frames, playback, duration and audio where requested. |
| Paged document or deck | Typst | Compile tokens first, then render and inspect every page. |
| Raster input preparation | Sharp | Inspect dimensions, crop, color and transparency before downstream composition. |
| Generated or imported SVG | SVGO | Optimize every SVG, then compare structure and appearance with the required semantic IDs intact. Optimization is not sanitization. |
| Editable vector artwork | SVG authoring, SVGO and resvg | Use `vector_render` for a semantic SVG and native PNG preview. Read back the SVG and inspect its preview; use licensed existing images when raster content is needed. |
| Comparable pixel QA | Odiff | Compare matched dimensions, content and state; inspect the diff and explain material changes. A low difference score is not aesthetic approval. |

CLI scratch directories use private directories on
the shared workspace disk and are removed after completion or cancellation.
Typst staging reject project assets above the 64 MiB total
before loading them into memory. PNG, JPEG, WebP and AVIF assets are fully decoded
for validation in an isolated worker, then staged byte-for-byte (at most 32 MiB
and 17 million source pixels per raster). Staging preserves dimensions, encoding
and metadata; use `raster_transform` explicitly for resizing or re-encoding.
Multi-page animated rasters in those formats require explicit frame preparation
or a video asset instead of silently retaining only their first frame.
SVG optimization preserves hidden states, empty attachment groups and unused
definitions as well as IDs; inspect the exported structure and resting pixels.
Typst compiles from the prepared project tree;
absolute imports cannot read other workspace files. Its compiled token file is
copied into that tree before compilation.

`static_render` accepts local TTF, OTF, WOFF and WOFF2 fonts supported by pinned
Satori, checking their signatures. Supply exact licensed faces and inspect the
actual glyphs and wrapping. This does not expand `vector_render`'s resvg font-file
support or establish browser-equivalent shaping for every script and inline layout.

The owned [tool budgets](../runtime/design/tools/tool-budgets.json) set each
design call's bridge deadline. They allow long renders, including 25 minutes for Typst, including page-image inspection.
The 64-page bound is a limit, not a promise that every job fits the deadline. The host MCP request ceiling is the largest
of these budgets; cancellation can still end an individual call earlier.

OpenPencil runs through its pinned official CLI in file mode, inside the core
tools container. `openpencil_inspect` lists supported upstream operations and
schemas, or reads a document. `openpencil_document` creates/imports an editable
`.fig`; `openpencil_edit` validates and applies an upstream write operation;
`openpencil_export` exports the saved source unchanged to PNG (2x) or SVG.
Document creation/import adds canonical variables/styles. Edits compile tokens
and preserve the saved file's IDs until serialization; their result contains the
reopened tree with the IDs to use next. Upstream error objects fail the operation
even when the CLI exits zero. The parent bounds inputs and publishes outputs with
revision checks. The child
can read only installed dependencies/fonts and its copied scratch inputs, and
write only scratch outputs. Arbitrary evaluation, network tools, app-only state
and remote font/asset downloads are excluded. No editor session or public port
is needed. Open the resulting `.fig` manually in OpenPencil when desired.
Headless PDF export is not exposed: the pinned upstream exporter requires browser
APIs. Use the existing Typst workflow for paged PDF output.

For example, after authoring canonical tokens and creating `artifacts/`:

```sh
mirket package call design openpencil_document --arguments '{"output":"artifacts/design.fig"}'
mirket package call design openpencil_inspect --arguments '{"tool":"create_shape"}'
mirket package call design openpencil_edit --arguments '{"source":"artifacts/design.fig","tool":"create_shape","arguments":{"type":"RECTANGLE","name":"Card","x":0,"y":0,"width":240,"height":120},"output":"artifacts/card.fig"}'
mirket package call design openpencil_export --arguments '{"source":"artifacts/card.fig","output":"artifacts/card.png"}'
```

The source order is deliberate: token compilation precedes dependent rendering;
raster inputs pass through Sharp before composition; generated and imported
SVG passes through SVGO; Playwright
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
Actions target only the local workspace. External requests and WebSockets are
blocked during capture; no arbitrary evaluation tool is exposed.

The package runs no local image-generation model. Art direction remains the
agent's work, supported by SVG creation, scene composition, UI/motion guidance
and actual rendered evidence. User-supplied images and any separately authorized
host generation tools remain outside Mirket's runtime and billing.

## Updates and recovery

Stop the design package with the currently installed release before a runtime
generation change, then run `mirket update` and explicitly start the desired
groups. Installation refuses an active incompatible generation before replacing
the executable that can stop it. The OpenPencil cutover archives exact legacy
private state under `packages/data/design/retained/` and retains all old volumes
and workspace files. It never opens or migrates a Penpot database. Export required
legacy files before switching; automatic Penpot-to-OpenPencil conversion is not
provided. The new stack creates no database, cache or assets volume.

`mirket update` reapplies saved package selections from the new release. A fresh
unattended setup selects no optional package. `package status` checks the selected
generation and service state; `doctor` checks installation and configured tools.
Unexpected file or service drift needs inspection before reconciliation. Keep
credentials, downloaded models and private design files out of repository source.

Source and image pins are in [the design stack registry](../registry/design-stack.json).
Dependency locks are in [the tool runtime](../runtime/design/tools/package-lock.json). [Runtime notices](third-party-notices.md#runtime-packages) record the
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
A cross-process render slot waits up to 30 seconds for competing native jobs
before loading renderers, within the existing tool deadline. Cancellation removes
a waiting request without starting its operation. Direct-tool schema, read-only
output preflight, disabled-group and admission rejections carry a tool-bound `not_started` receipt, so a routed
worker can correct the call without claiming an artifact was produced. Handler
failures, missing receipts and upstream execution errors retain uncertain-effect
semantics and require inspection before recovery. Output-path locks still reject
conflicting writers; queueing render admission does not authorize overwrites.
An overwrite requires the current `expected_sha256`; a missing or stale revision
is recoverable before dispatch. The publisher still checks the revision again,
and a conflict after rendering retains uncertain-effect semantics.
Image preparation is sequential and identical transformations are reused within
the request. Satori/resvg and the browser client load only when their operations
need them.

Assigned tools with explicitly disabled availability fail routed preflight before
model inference. Service activation remains an explicit `mirket package` operation.
`vector_render` and `svg_optimize` require `.svg` output paths; vector rendering
returns a separate PNG preview path in its receipt.
Native vector previews use the pinned image's DejaVu serif, sans-serif and
monospace fallbacks when an authored family is unavailable. Supply licensed local
font files through `fonts` when exact typography matters; a fallback preview does
not establish a match to a browser's host fonts. The editable SVG retains its text.

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

Direct owned design tool success responses include `structuredContent.runtime`,
the same identity in ordinary text content, and `_meta["mirket/runtime"]`: the process-captured
runtime digest, module/lock-file digests and Node version. Use it to compare actual
calls after a container rebuild. The package bridge's MCP version also includes
the compiled Mirket source identity; these identify different layers. OpenPencil results additionally report the pinned
engine version and headless file mode.

If a native Podman upgrade removes its registered executable, inspect and explicitly
replace that registration through `mirket tool remove/register`. For the resulting
owned helper-path mismatch, inspect `.mirket/execution/podman/containers.conf`, then
run `mirket environment repair-helpers --expected-sha256 OBSERVED_SHA256`. This
accepts only Mirket's exact helper-only configuration, checks the registered
installation's helpers, and leaves VM identity, volumes and workspace unchanged.

## Worker design scope

Routed workers use a project subdirectory of the single design workspace. Each
process receives that directory as its root; file tools, tokens, renderer inputs,
outputs and browser requests use it. Absolute file writes are accepted only within
that assigned design directory. Sibling project paths and browser requests are
rejected, including encoded traversal. The native renderer admission lock remains
shared across projects. Responses report the process workspace and host/project
scope alongside loaded source identity.

Host design MCP calls retain the host-owned shared workspace. Project scoping is
an enforced tool boundary, not a separate container/kernel sandbox per project.
OpenPencil documents share this filesystem namespace and are available to routed
workers through the `editable-interface` workflow. Within one project, use
separate output names and revision hashes for concurrent work.

The Typst tool deadline is 25 minutes, below the 30-minute paged-work budget;
inner compiler and page-image limits remain in force. This ceiling does not
estimate job duration or guarantee that 64 complex pages fit the budget.


## Selective diagram preparation

A fresh design installation prepares the core image and its pinned build base.
It does not pull Kroki or the Excalidraw companion. To add diagrams explicitly:

```sh
mirket package prepare design --group diagrams
mirket package stop design
mirket package start design --group diagrams
```

Preparation downloads missing pinned images but does not start services or change
saved groups. Startup never downloads. Setup/update preserve saved diagram groups;
a previously prepared full image receipt remains valid. Core and optional diagram
receipts are immutable within a generation: preparation can finish after a failed
pull, but conflicting or modified receipts require inspection, never overwrite.
Prepared additions do not invalidate a running core. Changing enabled groups or
runtime generations still requires stopping the owned services first. Existing
images and user artifacts are not deleted by this change.

The diagram group retains Excalidraw format support. Encoded video remains a
specialist capability of the explicitly selected design package; no claim of
natural adoption or production-quality gain follows from its availability.
