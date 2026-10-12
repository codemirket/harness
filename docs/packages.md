# Local MCP, editor and animation packages

Mirket installs reviewed local MCP servers and editor applications through the
CLI. Server packages expose a filtered upstream MCP interface; editor and engine
packages use Mirket-owned Rust MCP adapters around the local runtime. Each
selection pins its runtime, dependency archives, launch arguments and notices.
Package installation downloads those exact artifacts; it does not run `npm`,
`npx`, `pip`, upstream installers or package lifecycle hooks.

The shipped profiles require no paid API. General browser navigation can use
the network when invoked; offline render jobs have no network. Native application
and browser prerequisites remain separate from package installation.

## Select packages

Inspect the catalog before choosing a package:

```sh
mirket package list
mirket package show playwright
```

Interactive `mirket setup` asks which packages to select. A fresh
`mirket setup --yes` selects none; subsequent setup and updates preserve saved
choices. Repeated `--package` flags replace the complete selection:

```sh
mirket setup --target all --package playwright --yes
mirket package install motion-canvas
mirket package status
mirket doctor --json
```

`package install` adds to the saved selection and reapplies setup for the saved
clients. Required runtimes are selected automatically. After configuration
changes, start a new Codex Desktop or Claude Desktop Code session.

| Package | Profile | Requirements and limits |
| --- | --- | --- |
| `playwright` | Isolated headless browser QA | Pinned Node and Chrome Headless Shell. Browser sandbox enabled; server-side arbitrary-code and browser-install tools excluded. No native Windows ARM64 browser package. Linux needs compatible system libraries and sandbox support. |
| `inkscape` | Editable static SVG, geometry inspection and PNG/plain SVG/PDF export | Inkscape 1.4.4 and its native bundled runtime. macOS ARM64, minimum macOS 11.3. Self-contained vector profile; live text needs available fonts. |

The catalog's `ready` status means a pinned package definition is available for
installation on a listed target. It does not establish native-client activation,
SDK readiness or successful use on that machine. `package show ID` gives the
exact version, platform artifacts and tool policy.

## Creative and accessibility packages

| Package | Purpose | Initial runtime boundary |
| --- | --- | --- |
| `axe-core` | Rendered accessibility audits with typed state interactions and saved JSON evidence | axe-core 4.14.0, Node and Chromium. Local project HTTP server only. Automated findings plus incomplete/manual-review results; no conformance certificate. |
| `hyperframes` | Editable HTML/CSS/SVG/Canvas films, sampled frames and encoded media | HyperFrames 0.8.149 in offline Podman, Node 24.21, Chromium 155, Linux FFmpeg/FFprobe 9.0.2. Actual producer/player, explicit local assets and finite timeline. See [profile](hyperframes.md). |
| `motion-canvas` | Generator-driven drawn sequences, paths, gradients, text and choreography | Motion Canvas 3.17.2 in offline Podman, fixed local build and browser runtime. Editable typed scene source plus PNG/video exports. See [profile](motion-canvas.md). |
| `penpot` | Self-hosted editable design files and browser-backed typed creation/refinement/export | Penpot 2.18.3, five pinned Linux ARM64 OCI images, PostgreSQL 15.14 and Valkey 8.1.3, a dedicated Podman environment and active native headless Chromium. See [profile](penpot.md). |

These owned adapters initially cover macOS ARM64 only. Use `package show` for
exact artifact pins and platform prerequisites. The CLI and client launchers stay
native; reviewed OCI jobs have enforced container boundaries. Native browser and
application exceptions execute under host permissions. See [Podman](podman.md).

```sh
mirket package install axe-core hyperframes motion-canvas
mirket package probe axe-core --schemas --project /absolute/project
mirket package call axe-core accessibility_audit --project /absolute/project --arguments '{"url":"http://127.0.0.1:3000/","output":"audit.json","width":390}'
```

Axe audits a specific rendered state. Typed click/fill/press actions can expose a
dialog or menu. External requests and missing content are errors; read violations
and incomplete results, then test keyboard, screen-reader and motion behavior as
needed. A zero-violation report is not whole-product accessibility approval.

HyperFrames and Motion Canvas use actual upstream rendering engines through
fixed Mirket runners. Inspect their schemas; capture important intermediate frames,
reverse-seek samples and saved normal-speed playback. Preserve editable source
separately from exports. Limits and supported formats are stated in each profile.
Missing assets, inconsistent seeking or unusable outputs are errors. System fonts
can affect pixel results. Browser-native website QA remains a separate task.

Penpot requires an explicitly selected Podman client and dedicated running ARM64 environment:

```sh
mirket tool register podman --executable /absolute/path/to/podman
mirket environment init
mirket environment start
mirket package install penpot
mirket package start penpot
mirket package probe penpot --schemas --project /absolute/project
mirket package status
mirket package stop penpot
```

Explicit installation pulls only pinned manifests; start/serve never downloads.
Services are named and labeled per selected home, use one internal network, and
expose only the frontend on loopback. Start verifies a real backend JSON response;
failed start removes new containers and restores prior running state, retaining
durable data. Stop preserves
private Podman VM volumes and private account/credential state. Package removal changes
selection and host entries, not durable design data; stop services explicitly.
The fixed plugin works in an active headless browser without a manual GUI. Its
accepted content permissions, persistence and export checks are part of readiness.
Use `mirket package stop penpot --purge-data` only to permanently delete this
home's owned design volumes, accounts and service state, such as after a disposable
test. The CLI checks ownership of all resources before deleting them.

## Create vector graphics and animation

Inkscape runs through its native command line with the complete signed application
bundle. The profile covers macOS ARM64, with a declared minimum macOS 11.3. Live
text needs available fonts; operation without a logged-in macOS session remains
unverified. No unrestricted editor actions or bundled interpreter scripts are tools.

```sh
mirket package install inkscape
mirket package probe inkscape --schemas --project /absolute/project
```

All source and output paths in editor arguments are relative to that selected
project. Parent directories must already exist. Absolute paths, hidden path
components, parent traversal and symlinks are refused. Existing output files
require `overwrite: true`; an optional `expected_sha256` checks the current
destination revision before replacement. A detected concurrent destination
change stops publication. Results identify the editable source separately from
an exported artifact and include saved artifact paths and hashes.

### Static SVG with Inkscape

| Tool | Purpose and principal arguments |
| --- | --- |
| `svg_save` | Create or refine editable SVG from `path` and inline `svg` markup; preserves the supplied source after validation and a native geometry check. |
| `svg_inspect` | Inspect `path`: canvas, viewBox, object IDs, declared font families and native bounding boxes. |
| `svg_export` | Export `source` to a distinct `output` with `format: "png"`, `"svg"` or `"pdf"`; optional `object_id` selects one drawable object. PNG `width` is 1–2048 pixels, default 1024. |

The static profile supports shapes, paths, gradients, clipping and text with an
explicit SVG `viewBox`. Scripts, CSS stylesheets, images, filters, linked
resources and active animation are outside this profile. Available fonts affect
live text; inspect the exported appearance. Plain SVG and PDF 1.5 are delivery
artifacts. PDF export reopens the generated file with the bundled PDF importer
and checks a rendered PNG before publishing; this verifies a usable renderer
result, not print-production requirements.

Create an editable source containing a named group, inspect it, then render a
separate preview:

```sh
mirket package call inkscape svg_save --arguments '{"path":"mark.svg","svg":"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"640\" height=\"240\" viewBox=\"0 0 640 240\"><g id=\"box\"><rect x=\"20\" y=\"60\" width=\"80\" height=\"80\" fill=\"#e04040\"/></g></svg>"}' --project /absolute/project
mirket package call inkscape svg_inspect --arguments '{"path":"mark.svg"}' --project /absolute/project
mirket package call inkscape svg_export --arguments '{"source":"mark.svg","output":"mark-preview.png","format":"png","width":640}' --project /absolute/project
```

For vector choreography, retain SVG geometry and integrate it into the actual
page with CSS/WAAPI or a selected HyperFrames composition. Motion Canvas produces
editable drawn scene documents; it does not promise an animated SVG export.
See the [SVG animation guidance](../skills/svg-animation/SKILL.md) and the
[local video workflow](../skills/advanced-motion/references/local-video.md).

## Update, remove and recover

```sh
mirket update
mirket package remove motion-canvas
mirket setup --no-packages --yes
```

Updates use the new Mirket release's embedded package definitions and reapply the
saved selections and settings. Upstream releases are not resolved at runtime.
Package removal removes that selection, its settings and its managed client
entries. `--no-packages` clears all package selections. Download caches, prepared
generations and package data remain on disk; removal does not purge them.

Artifacts are downloaded over HTTPS, checked against curated SHA-256 values and
extracted with size and path validation. Setup prepares a new generation before
changing client registrations. Published generations are never overwritten in
place. Failed setup preserves the previous active configuration.

Editor DMGs are downloaded from the official upstream distributors and mounted
read-only. Mirket copies only the selected application, retaining valid relative
links inside its bundle, and checks the copied app with macOS code-signature
verification before publishing the generation. It does not run a DMG installer,
strip signing metadata or re-sign applications. File-provider-managed locations,
including some Documents folders, can add metadata that invalidates this check;
installation then fails without publishing the app. Use a normal local directory
for a disposable `--home`, outside those managed folders. Signature verification
does not establish notarization, upstream safety or visual correctness.

Mirket releases contain editor adapters and package metadata, not these editor
binaries. Direct upstream installation and redistribution by Mirket are different
delivery paths. The retained notices identify known licenses and exact source
pins, but do not assert complete native component/source coverage. Rehosting an
editor binary or supplying a shared binary cache requires completing that review
and the applicable corresponding-source obligations; see the
[editor license scope](third-party-notices.md#editor-application-distribution).

If `doctor` or launch reports drift, inspect and preserve the changed files before
reconciling them. Do not edit receipts to suppress a mismatch. A receipt is local,
user-owned state: matching hashes establish consistency, not a security
attestation or proof that the upstream code is safe.

Source pins and dependency locks are in `registry/packages-*.json` and
`registry/package-locks/`. [Package notices](third-party-notices.md#runtime-packages)
are embedded and retained with the installed payloads.
