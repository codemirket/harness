# Local MCP and editor packages

Mirket installs reviewed local MCP servers and editor applications through the
CLI. Server packages expose a filtered upstream MCP interface; editor packages
use Mirket's Rust MCP adapter around the application's native command line. Each
selection pins its runtime, dependency archives, launch arguments and notices.
Package installation downloads those exact artifacts; it does not run `npm`,
`npx`, `pip`, upstream installers or package lifecycle hooks.

The shipped profiles require no paid API. Local execution does not imply an
air-gapped workflow: documentation crawling and browser navigation use the
network when invoked. Native SDKs, devices and host permissions remain separate
prerequisites.

## Select packages

Inspect the catalog before choosing a package:

```sh
mirket package list
mirket package show jscpd
```

Interactive `mirket setup` asks which packages to select. A fresh
`mirket setup --yes` selects none; subsequent setup and updates preserve saved
choices. Repeated `--package` flags replace the complete selection:

```sh
mirket setup --target all --package doc-scraper --package jscpd --yes
mirket package install safe-docx semiotic
mirket package status
mirket doctor --json
```

`package install` adds to the saved selection and reapplies setup for the saved
clients. Required runtimes are selected automatically. After configuration
changes, start a new Codex Desktop or Claude Desktop Code session.

| Package | Profile | Requirements and limits |
| --- | --- | --- |
| `doc-scraper` | Stored-document search, provenance, freshness and explicit crawling | Native executable. Crawls use a configured HTTPS site; stored content can be searched locally. |
| `jscpd` | Exact, renamed and similar duplicate-code checks | Native executable. No model downloads; semantic modes and cross-folder comparison are excluded. |
| `playwright` | Isolated headless browser QA | Pinned Node and Chrome Headless Shell. Browser sandbox enabled; server-side arbitrary-code and browser-install tools excluded. No native Windows ARM64 browser package. Linux needs compatible system libraries and sandbox support. |
| `dbhub` | Local SQLite inspection and bounded SQL | Pinned Node. Requires an existing database selected through the CLI. macOS and Linux only; remote database connections are outside this profile. |
| `safe-docx` | Local document editing, comments and redlines | Pinned Node. Reads and explicit output paths are scoped to the launch working directory. Existing outputs require explicit overwrite permission. Google Docs integration is excluded. Reopen and render saved documents to assess layout. |
| `semiotic` | Static-data chart creation, explanation and validation | Pinned Node and React dependencies. Six developer-profile tools with SVG rendering; PNG and interactive rendering are excluded. Inspect the chart and reconcile its data. |
| `agent-device` | Local device and simulator interaction | Pinned Node. Requires user-provisioned SDKs, simulators/devices and permissions. Cloud, plugin installation and arbitrary batch commands are excluded. Its managed web backend is not included. |
| `markitdown` | Local text, Office and PDF extraction to Markdown | Private CPython and pinned wheels with the `docx`, `pdf`, `pptx` and `xlsx` extras. macOS 14+ and Linux glibc 2.28+ on ARM64/x64; Windows is unavailable. Network inputs, audio, image conversion and plugins are excluded. |
| `inkscape` | Editable static SVG, geometry inspection and PNG/plain SVG/PDF export | Inkscape 1.4.4 and its native bundled runtime. macOS ARM64, minimum macOS 11.3. Self-contained vector profile; live text needs available fonts. |
| `glaxnimate` | Editable vector animation, frame inspection and animated SVG export | Glaxnimate 0.6.0 distribution, native build `0.6.0-git-d2da67a`, with its bundled runtime. macOS ARM64, minimum macOS 13.0. Solid-color shapes and linear translation/opacity tracks. |

The catalog's `ready` status means a pinned package definition is available for
installation on a listed target. It does not establish native-client activation,
SDK readiness or successful use on that machine. `package show ID` gives the
exact version, platform artifacts and tool policy.

## Configure data sources

Doc Scraper initially uses `https://modelcontextprotocol.io`. Change the site
through the CLI:

```sh
mirket package configure doc-scraper --site https://modelcontextprotocol.io
```

The site must be a credential-free HTTPS URL without a fragment. Generated crawl
configuration limits depth, request concurrency, page size and duration, skips
images and disables private-network access. Configuration alone does not crawl;
`crawl_site` is an explicit tool invocation.

DBHub needs a selected existing SQLite file before its server can start. Replace
the example paths with absolute paths on your machine:

```sh
mirket package install dbhub
mirket package configure dbhub --database /absolute/database.sqlite
mirket package probe dbhub --project /absolute/project
mirket package call dbhub execute_sql --arguments '{"sql":"SELECT 1 AS ready"}' --project /absolute/project
```

The database path must be ASCII, contain no whitespace, `%`, `?` or `#`, and
contain no symlink components. These restrictions prevent the pinned DBHub
release from interpreting an encoded path as a different file. Mirket resolves
the accepted path to one canonical absolute path for both validation and the
server configuration. Windows SQLite
path handling is unavailable until verified.

Mirket generates a two-tool profile: `search_objects` and `execute_sql`.
Before forwarding `execute_sql`, Mirket opens the selected file read-only and
prepares every statement against its actual schema without executing validation
SQL. SQLite's native authorizer permits reads and rejects writes, `PRAGMA`,
`ATTACH`, `DETACH` and extension loading. Requests must contain between one and
64 statements, and every prepared statement must be read-only.

The generated upstream `execute_sql` configuration also sets `readonly = true`
and a 200-row cap. These are SQL restrictions, not an operating-system sandbox
or a read-only filesystem boundary for the server. SQLite work is synchronous,
so the bridge's process deadline is the outer time limit. Keep consequential
databases protected by independent file permissions and use a disposable copy
for untrusted workloads.

## Work with local documents

MarkItDown exposes only `convert_to_markdown`. It accepts a host-free `file:` URI
under the selected project directory, with no query or fragment:

```sh
mirket package install markitdown
mirket package call markitdown convert_to_markdown --arguments '{"uri":"file:///absolute/project/brief.docx"}' --project /absolute/project
```

The supported inputs are UTF-8 TXT, Markdown, HTML, CSV, TSV, JSON and XML;
DOCX, XLSX and PPTX containers; and PDF. Inputs must be regular files no larger
than 64 MiB. Mirket checks text content, PDF headers and Office container identity;
Office expansion is bounded to 256 MiB. Remote URIs, audio, images and arbitrary
archives are refused. The private Python launch excludes user-site and `.pth`
startup hooks, working-directory imports, project `.env` loading and plugins.
It does not use the machine's Python installation or install dependencies at
launch. Check extracted text and tables against the source; conversion does not
preserve document layout.

Safe-Docx maintains an editing session inside its server process. Use a native
client or an MCP client connected to one `mirket package serve safe-docx` process
for read/edit/save workflows; separate `package call` commands do not share that
session. Read the current tool schemas with
`mirket package probe safe-docx --schemas --project /absolute/project`.
Google document IDs are removed from discovery and refused at invocation.

Every document output must use an explicit project-relative or absolute path
inside the selected project. Home expansion, parent traversal and symlink paths
are refused. The bridge checks all requested destinations before forwarding:

| Tool or save mode | Required output fields | Existing output behavior |
| --- | --- | --- |
| `save`, `save_format: "clean"` | `save_to_local_path` | Requires `allow_overwrite: true` |
| `save`, `save_format: "tracked"` | `save_to_local_path` for the tracked document | Requires `allow_overwrite: true` |
| `save`, `save_format: "both"` or omitted | Distinct `save_to_local_path` and `tracked_save_to_local_path` | Both are checked; each existing output requires `allow_overwrite: true` |
| `export`, `convert_to_odt` | `output_path` | Requires `allow_overwrite: true` |
| `compare_documents` | `save_to_local_path` | Always refuses an existing output |

An explicitly supplied `tracked_save_to_local_path` is checked even when the
chosen save mode does not need it. `batch_edit` accepts only text replacements
and paragraph insertions; it cannot dispatch save or export operations. Verify
the saved file's contents, tracked changes and source preservation, then render it
with a separate capable renderer when layout matters. Path checks are not an
operating-system sandbox.

## Render static charts

Semiotic runs its developer profile with exactly six tools: `suggestCharts`,
`getSchema`, `diagnoseConfig`, `renderChart`, `auditAccessibility` and
`groundChart`. `renderChart` returns SVG content; an omitted `format` becomes
`svg`, and other formats are refused. Interactive rendering is unavailable.

```sh
mirket package install semiotic
mirket package probe semiotic --schemas --project /absolute/project
mirket package call semiotic renderChart --arguments '{"component":"BarChart","props":{"data":[{"region":"North","revenue":120},{"region":"South","revenue":80}],"categoryAccessor":"region","valueAccessor":"revenue","width":640,"height":420},"format":"svg"}' --project /absolute/project
```

Save the returned SVG, inspect its labels and marks, and reconcile their values
with the input data. An accessibility audit or successful render does not by
itself establish visual acceptance.

## Create vector graphics and animation

Inkscape and Glaxnimate run through native command lines, with Glaxnimate using
Qt's offscreen platform. The tools work with saved files and do not provide a GUI
editing session. They use the complete pinned application bundles, including
their private native libraries and interpreters; system Python, Node and a
separate editor installation are unnecessary. Bundled interpreters and arbitrary
editor actions are not exposed as tools.

These profiles cover macOS ARM64 only. Their declared minimum versions are
macOS 11.3 for Inkscape and 13.0 for Glaxnimate. Native CLI and Qt offscreen
checks were run in a logged-in macOS 26.6.2 ARM64 session, without an external X11
server. Operation without a logged-in session or WindowServer is unverified.
These checks do not establish every supported macOS version or actual activation
in a Codex Desktop or Claude Desktop Code session.

Read the installed schemas before creating files. Replace `/absolute/project`
with an existing writable project directory and use new output names for the
first run:

```sh
mirket package install inkscape glaxnimate
mirket package probe inkscape --schemas --project /absolute/project
mirket package probe glaxnimate --schemas --project /absolute/project
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

### Vector animation with Glaxnimate

| Tool | Purpose and principal arguments |
| --- | --- |
| `animation_create` | Import a static SVG `source`, add typed `tracks` on named groups, and save an editable `.rawr` `output`. |
| `animation_inspect` | Validate a `.rawr` `path` and report its canvas, duration, named groups and supported keyframes. |
| `animation_frame` | Render one zero-based `frame` from a `.rawr` `source` to a distinct PNG or static SVG `output`. |
| `animation_export` | Export a `.rawr` `source` to a distinct animated SVG `output`; `loop_animation` defaults to false. |
| `animation_retime` | Scale the keyframe times to `duration_seconds` and save a distinct editable `.rawr` `output`, preserving the source. |

The animation profile is narrower than the static SVG profile: solid-color
shapes and paths, without text, gradients, clipping, filters, images or dashed
strokes. Rounded rectangles must use equal horizontal and vertical corner radii.
Use named `<g>` elements for motion targets. Each animation has one
composition at 60 fps and lasts 0.05–30 seconds. Creation accepts 1–64 group
tracks, each with 2–64 linear translation or opacity keyframes. Keyframe `at`
values are increasing fractions of the duration, starting at 0 and ending at 1.
A translation target cannot already have a transform; place any static transform
on an inner group. Unsupported native `.rawr` features are refused.

Continue from the static source above:

```sh
mirket package call glaxnimate animation_create --arguments '{"source":"mark.svg","output":"mark.rawr","duration_seconds":2,"tracks":[{"target_id":"box","translate":[{"at":0,"x":0,"y":0},{"at":1,"x":240,"y":0}],"opacity":[{"at":0,"value":1},{"at":1,"value":0.5}]}]}' --project /absolute/project
mirket package call glaxnimate animation_inspect --arguments '{"path":"mark.rawr"}' --project /absolute/project
mirket package call glaxnimate animation_frame --arguments '{"source":"mark.rawr","output":"mark-frame-60.png","format":"png","frame":60}' --project /absolute/project
mirket package call glaxnimate animation_export --arguments '{"source":"mark.rawr","output":"mark-animated.svg","loop_animation":false}' --project /absolute/project
mirket package call glaxnimate animation_retime --arguments '{"source":"mark.rawr","output":"mark-slow.rawr","duration_seconds":4}' --project /absolute/project
```

Keep `mark.svg` and `mark.rawr` as editable sources. Frame exports are static;
animated SVG preserves the supported motion. Export plays once and freezes its
final state unless `loop_animation: true` explicitly requests indefinite motion.
The embedding page must supply any reduced-motion behavior. Inspect the first,
middle and final available frames, then inspect actual browser playback. Frame
indices must be less than `last_frame`: a two-second, 60 fps composition permits
frames 0–119. Neither successful export nor changed pixels establishes visual
acceptance.

### Editor execution limits

Editor requests are capped at 128 KiB; inline `svg_save` markup is capped at
96 KiB. Existing SVG and `.rawr` sources are limited to 2 MiB, generated artifacts
to 32 MiB, and structured inspection responses to 1 MiB. Each native editor
process has a 30-second deadline and bounded diagnostic output. A server handles
one job at a time; concurrent requests receive a busy response. Jobs use private
HOME/profile/temp directories and staged output, validate the resulting file,
then publish it. A zero process exit alone is insufficient.

Host connection and per-tool budgets are separate from those native process
limits. See [editor connection budgets](installation.md#editor-connection-budgets)
for the managed Codex and Claude settings and Claude's distinct host startup
timer.

Cancellation stops active editor process groups and discards unpublished
staging; it cannot undo an artifact already published. Path validation, controlled
arguments and a private environment are not an operating-system sandbox. The
native editor retains its host process permissions.

## Verify invocation

Use checks that answer different questions:

```sh
mirket package status
mirket package probe doc-scraper --project /absolute/project
mirket package call doc-scraper list_sites --arguments '{}' --project /absolute/project
```

`status` and `doctor` compare package files with their local receipts without
starting the servers. `probe` starts the selected process and lists its permitted
MCP tools. `call` invokes one permitted tool using JSON object arguments. A tool
error gives a nonzero exit status. `probe --schemas` includes the filtered input
schemas. Each probe or call starts and closes its own package process. A
successful discovery probe does not prove a browser task, a device interaction,
a correct chart or an acceptable document.

`--project` sets the working directory and applicable path scope; its default is
the current directory. Use it explicitly for CLI checks. The client owns the
working directory of a configured host server. That directory is not a verified
project registration or an operating-system sandbox; inspect it during native
client testing.

Configured clients start `mirket package serve ID`. Server profiles run a stdio
bridge exposing permitted upstream tools; editor profiles serve Mirket's typed
editor tools. Both bound requests and process lifecycles. Server bridges do not
forward upstream sampling, elicitation, roots requests, resources or prompts.
They are separate from Mirket's coordination MCP server. Host permissions remain
authoritative; tool filtering and an isolated environment do not create an
operating-system sandbox.

## Update, remove and recover

```sh
mirket update
mirket package remove semiotic
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
