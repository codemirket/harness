# Contributing to Mirket

Start with [AGENTS.md](AGENTS.md) and the [architecture](docs/architecture.md).
A contribution should improve an actual outcome: a clear setup experience,
a reproducible failure, useful expertise or evidence that a task was handled well.
Keep secrets, account state and private prompts out of code, logs and fixtures.

## Bootstrap the development CLI

Install the official Rust toolchain, then install the development CLI from this
full source checkout:

```sh
cargo install --locked --debug --path . --bin mirket
mirket dev check --project .
```

Keep the CLI on PATH outside Cargo's build output directory. Development checks
and builds replace output executables, which Windows cannot do while that same
executable is running. CI installs its bootstrap into a separate temporary prefix.

`mirket dev fmt` formats the Rust sources. `mirket dev check` runs formatting,
Clippy with warnings denied, and all Rust test targets. Use
`mirket dev build --release` to build the CLI and standalone target installers.
When changing owned JavaScript report or export verification, exercise the
affected helpers with the selected runtime. Core Rust checks do not silently
install optional package runtimes. Helper probes do not replace a full browser
or renderer invocation and visual inspection.

The locked dependency graph and pinned toolchain are authoritative. After a
production dependency change has been approved under [AGENTS.md](AGENTS.md),
run `mirket dev lock --project .` to resolve the workspace requirements while
retaining other locked versions where their constraints permit. Review the
resulting `Cargo.lock` diff before running checks; lockfile reconciliation is not
permission to add a dependency. For a reviewed transitive patch, use
`mirket dev lock --project . --package rustls --precise 0.23.46`; this updates only
that package and any dependencies Cargo must reconcile.
`mirket dev test FILTER` runs a focused test selection. `--ignored` explicitly
selects opt-in tests, including real package downloads.
Use `--release` for installed native-package checks so they exercise the optimized
CLI within actual client startup budgets. Debug integrity scans of large native
application bundles are unsuitable for performance conclusions.
`--test NAME` selects one integration target to avoid compiling unrelated native
test binaries. The required `mirket dev check` still runs every standard target.

After publishing the checkout's version, run
`mirket dev test published_update --ignored` to exercise the public GitHub update
in a disposable home. It verifies the published binary's digest, recovery of a
missing managed skill, saved setup choices and unchanged unrelated client settings.

Keep client setup isolated from development. Use existing empty temporary home
and project directories with explicit absolute paths:

```sh
mirket --home /absolute/test-home setup --target all --yes
mirket --home /absolute/test-home doctor --json
mirket --home /absolute/test-home project init --project /absolute/test-project
mirket --home /absolute/test-home project sync --project /absolute/test-project
mirket --home /absolute/test-home project doctor --project /absolute/test-project
```

Installation does not prove native-client activation. Test protocol discovery,
actual tool invocation and useful task output separately. Use host permissions
and the user's chosen provider. Report unavailable native platforms precisely.

## Verify boundaries

Choose meaningful tests from changed behavior. Installer changes must cover
existing configuration, modified managed files, symlink/path escapes, interruption
and recovery. Update changes must check candidate identity and digest before
execution, then verify the new setup or preserve a recoverable installation.
State changes need concurrent-revision, idempotency and stale-evidence probes.
MCP changes need actual SDK-client exchanges and bounded malformed input.

Package changes must cover digest mismatches, archive traversal, links and special
files, extraction limits, managed drift and unexpected generation files. Exercise
setup rollback and update replay with saved package settings. Bridge changes need
discovery and invocation filtering, invalid arguments, process deadlines,
cancellation and child-process cleanup.

Run `mirket benchmark` on an optimized binary for named local workloads; report
sample count, target and binary identity. The output retains raw samples and
separates version startup, fresh-process catalog startup and warm catalog calls.
For startup diagnosis, set `MIRKET_STARTUP_TIMING=1` on `mirket mcp serve`
or `mirket mcp doctor --iterations 2`. It emits bounded, payload-free JSON phases
on stderr for catalog loading, executable identity, database opening, handshake
and background discovery. MCP stdout remains protocol-only. The installed motion
payload regression retains these diagnostics on a timeout and checks that native
model discovery starts after the handshake; its 15-second deadline is unchanged.

Compare unchanged workloads and content identities before attributing a change
to runtime speed. Search relevance changes are a quality/performance tradeoff,
not equivalent-output throughput evidence. Do not turn machine-dependent timing
into flaky correctness gates. Repeat performance tests after relevant changes.

## Add expertise

Read [source review](docs/source-review.md). Keep authored skills focused on a
recognizable deliverable, prerequisites, useful references and failure probes.
Preserve upstream attribution and licenses. Test the selected payload and its
project installation, then exercise the intended task where tools are available.
A skill's presence or an agent repeating its name is not evidence of quality.

The embedded browser-motion examples also have an optional rendered acceptance
probe in `tests/browser_motion.cjs`. Select an installed Chromium executable
through `MIRKET_CHROMIUM_PATH` and make `playwright-core` resolvable (or set
`MIRKET_PLAYWRIGHT_MODULE` to its absolute installed module path); the probe never
downloads a browser. Serve the **installed**
`advanced-motion/assets` directory with an existing local server, then run:

```sh
node tests/browser_motion.cjs http://127.0.0.1:8765/ /absolute/evidence/motion
```

It records normal playback and frames at desktop/narrow sizes, checks reduced
motion at load and live changes, rapid input, scroll reversal, static/Canvas
fallbacks, and explicit lifecycle events. An optional fourth argument selects a
scenario prefix for a focused rerun, such as `signal-390-normal`. Inspect the
recordings and frames against the brief; the assertions do not accept aesthetics,
prove native back/forward-cache behavior, or benchmark a physical mobile device.
Keep generated evidence outside shipped source and stop the preview server.

The design-language and interaction lessons use the existing `design-inspiration`
delivery. After selecting/syncing it in a disposable project, serve its installed
skill directory and run the optional rendered probe (serving the skill root also
keeps example-to-card links usable):

```sh
node tests/browser_design_languages.cjs http://127.0.0.1:8765/assets/design-languages/ \
  /absolute/evidence/design-languages \
  /absolute/installed/design-inspiration/references/design-languages/library.json
```

Select an installed Chromium executable through `MIRKET_CHROMIUM_PATH`. The
probe installs nothing. An optional final comma-separated ID
list selects affected instances. The script checks actual wide/narrow rendering,
no-script readability, enlarged-text reflow, resource independence and access
scan findings, and captures the plausible misleading versions. Inspect every
changed composition, focal detail and interaction against its lesson; these
checks do not establish conceptual quality, native material parity or a measured
improvement in a model. Keep source/licensing exceptions explicit in the typed
canonical per-instance record. Keep `library.json` a bounded index; do not
duplicate complete records into it. Scope coverage cells to one actual example
and platform; source guidance alone is not a worked native example. Exercise a
request whose native/RTL/offline requirements occur only in separate cells and
confirm it cannot become represented. `tests/design_language_payload.rs` covers individual routing, bounded
reads, both-host delivery and actual installed MCP exchanges without a browser.

`evals/design-coverage/README.md` defines the paired transfer pilot through the
existing `mirket dev role-eval` contracts. Freeze briefs/configuration/rubrics
before actual host trials, retain every planned result and grade saved artifacts
independently. Protocol validation does not execute or grade a model. Keep local
trial artifacts and private task history outside shipped source. No gain or
universal-coverage claim follows from fixture validation or a small pilot.

`evals/design-quality/README.md` defines the harder follow-up: separate continuous
craft ratings, functional/access guards, two calibrated render-first judges and
repeated trials on independently sealed briefs. Keep source, output, judge and
analysis identities explicit. The public controls are synthetic collector/judge
calibration material, not model executions. A reported practical gain must meet
every frozen threshold; retain inconclusive waves, regressions and judgment
limits. Local run history and sealed evaluation contexts belong outside shipped
source. `design_reference_candidates` is a provisional no-match recovery path;
test that it cannot alter ordinary keyword results or certify missing coverage.

Run `tests/browser_design_language_motion.cjs` against the same assets URL and an
evidence directory for the four motion lessons. It exercises actual geometry,
truthful/misleading state contrasts, rapid reversal, live reduced motion and
preference restoration, finite rest, narrow reflow and no-script content. An
optional final ID selects one lesson. Inspect the saved beats;
the probe does not claim native platform rendering or real-device performance.

For the optional timing specimen, install `motion-design` in that disposable
project and serve its installed `.agents/skills` directory (or the Claude
equivalent). The same probe
accepts this skills-root URL with the `craft` selection:

```sh
node tests/browser_motion.cjs http://127.0.0.1:8765/ /absolute/evidence/craft craft
```

It exercises labelled timing, keyboard playback controls, rapid replay, cleanup
and initial/live reduced motion using the explicitly selected Chromium executable.
Use normal
playback and the saved frames for inspection; a timing assertion or visible static
fallback is not a performance or human visual-acceptance result.

For illustration changes, `evals/illustration-quality/README.md` supplies varied
illustration, asset-family, diagram and composition briefs using the existing
role-eval report contract. Its separate craft review compares concept,
composition, construction, coherence and placement fit against the brief and
multiple relevant examples. Templates are not observed trials; freeze actual
producer/model settings and preserve failures before evaluating a quality claim.


## Add a local MCP package

Use the existing package registry and setup flow (`design` for design tools,
`agent-runtime` for memory and worker persistence). Its definition
needs exact source and version pins, supported targets, complete runtime and
dependency closure, artifact digests, extraction bounds, launch arguments and
retained license notices. Keep dependency resolution locks and reviewed notices
with the definition. The pinned OpenPencil CLI and its transitive dependencies are installed from the
reviewed npm lock in the prepared image; package serving must not resolve dependencies or run
upstream installers. Ask before adding a production dependency as required by
[AGENTS.md](AGENTS.md).

Review the actual entrypoint, optional native modules, runtime downloads, network
behavior and tool effects. Restrict a profile only when its promised capability
still works; hold an incomplete profile instead of presenting it as available.
New selections must remain explicit, and setup/update must preserve saved choices
and unrelated client settings.

Use an isolated home for real package checks:

```sh
mirket --home /absolute/test-home setup --target all --yes
mirket --home /absolute/test-home package install design
mirket --home /absolute/test-home package start design
mirket --home /absolute/test-home package status
mirket --home /absolute/test-home package probe design --schemas
```

Then use `mirket package call` with a meaningful fixture and a discriminating
failure case for the selected profile. Check written artifacts and effects, not
only a successful response. Run networked downloads and native SDK/device tests
explicitly; report missing prerequisites and untested targets. A registry entry
or successful probe alone does not establish useful runtime behavior. Keep
temporary packages and fixtures out of shipped source and clean task-owned
scratch when finished. The [package guide](docs/packages.md) defines the product
workflow and current limitations.

## Exercise the design stack

The standard Rust gate covers the controller and package bridge. It does not
launch design services or run the JavaScript tool suite. With the
locked tool dependencies available under Node 22, run `npm test` in
`runtime/design/tools` for the owned JavaScript tests. These tests require Linux
and the owned guard: compile it with
`cc -O2 -Wall -Wextra -Werror runtime-guard.c -o runtime-guard` in an isolated copy
of the tool directory before testing. CI uses Node 22, the exact npm lock and
`npm ci --ignore-scripts`. Keep `node_modules` and generated binaries outside the
checkout because runtime source identity includes this directory.

On macOS with an explicitly selected, running Mirket engine, the matching candidate
can build the reviewed design image and run the suite, including browser fixtures,
in an isolated container:

```sh
mirket dev design-check --project . --engine-home /absolute/engine-home
```

This does not change package selections, service receipts, running services or
mount a user project. It can prepare a new dependency image in the engine cache. Provision dependencies
only as an explicit development step; package serving never installs them.

For live acceptance, use an existing disposable home, register the approved
Podman executable there, initialize/start its private environment, then run:

```sh
mirket --home /absolute/test-home setup --target all --package design --yes
mirket --home /absolute/test-home package start design
mirket --home /absolute/test-home package status
mirket --home /absolute/test-home package probe design --schemas
```

Place bounded fixtures in the reported shared workspace. Invoke the actual
MCP tools with `package call` or a native SDK client: compile tokens, render a
Satori card, capture a full-DOM page at 2x, create/edit/export an OpenPencil file,
render Kroki diagrams with explicit Excalidraw selection when needed, compile a
multi-page Typst PDF, transform a raster with Sharp, render editable vector art,
and compare equal and changed pixels with Odiff. Exercise core alone and optional
diagrams group and core OpenPencil tools, input and animation states, cancellation and memory bounds.

Inspect saved outputs and persisted identities. Exercise a meaningful rejected
input, deadline and transport disconnect; confirm child cleanup, no partial
publication and preserved service health. Retain useful evidence outside source.
There is no checked-in ignored end-to-end suite or marker-file authorization
shortcut for these service journeys; selecting a real user home is not a test
fixture. Stop/purge only the exact disposable profile and remove its private VM
and task-owned files after verifying preservation. Report native platforms and
paths that were not exercised.

## Build release packages

```sh
mirket dev dist --project . --output /absolute/new-dist
```

This builds native release binaries, copies standalone installers and writes
checksums plus the release manifest. Each distribution also contains Mirket's
`LICENSE`, source and package `NOTICE.md`, and `RUST-NOTICES.txt` with the locked
Rust dependency and standard-library notices. Preserve these notice files when
publishing binaries. Notice collection fails if a dependency's license evidence
is missing. It does not publish. Native CI runners test and package their target.
Combine their manifests through the CLI:

```sh
mirket dev manifest --input /absolute/macos/mirket-release.json --input /absolute/windows/mirket-release.json --acceptance /absolute/evidence/acceptance.json --output /absolute/release/mirket-release.json
```

The command verifies the listed local binaries against their checksums, requires
one version and unique targets, verifies adjacent `mirket-build.json` records,
requires a successful contained-acceptance receipt whose executable is one of the
candidates and whose source identity matches every candidate, and writes a new
manifest without executing the
binaries. Upload that manifest, the packaged executables, checksums and notices
only after publication is explicitly authorized. If native notice files differ,
publish each with its executable target in the filename rather than dropping one.
The operating system must match the executable target.

## Contained release acceptance

`mirket dev acceptance` is part of the [0.0.1 release gate](docs/release-status.md).
Use the candidate executable with its matching source and prepared runtime generation.

Before deploying a change to desktop contracts or owned runtime code, run both
the standard gate and the explicit contained gate with the candidate CLI outside
Cargo output. The latter requires a dedicated or explicitly selected Linux ARM64
Podman engine, selected `design` and `agent-runtime` packages, and matching prepared
images. It does not start the VM, install dependencies or change package selection.

```sh
mirket dev check --project .
mirket package prepare agent-runtime
mirket package prepare design
mirket package start design
mirket dev acceptance --project . --engine-home /absolute/engine-home --output /absolute/evidence/acceptance.json
```

Use `--home /absolute/engine-home` on package commands when the selected engine
belongs to another home. Before preparing a changed design generation, stop its
services using the currently installed CLI; retain service groups and restart
those groups with the candidate. Never purge its data. The candidate's source
identity must match the checkout tested by Cargo. Complete disposable setup,
doctor and catalog checks as described above before updating the desktop.

Acceptance reuses native Rust integration fixtures and the contained design
checks: typed MCP schemas and fresh executable/skill identities; actual
paginated export and typed retention preview/apply; parallel native memory and
lifecycle calls; scoped design token compilation and SVG/PNG output; native
fact-store and execution-journal recovery and relocation. Fixtures contain
synthetic data, use unique scopes, clean up their own files, and never register
or read existing user projects. Missing prerequisites or stale package
identities fail the gate rather than skipping checks.

This gate is an automated local release check. It does not claim that either
already-running desktop connection has reconnected, or judge aesthetic quality.
After deployment call `runtime_status` in each actual client, inspect ordinary
`runtime` from a direct design response, and verify loaded skill hashes. Never
substitute an installed binary or fresh SDK session for those observations.

## Dependency reporting

CI queries OSV for every locked Cargo and design npm coordinate. Native agent
artifacts and image contents are covered by their separate pinned inventories
and receipts. Source and image advisories remain visible under the report
policy; choose fixes for reachability, long-term stability and performance,
while retaining unresolved findings. Incomplete queries, invalid inventory,
scanner failures and identity mismatches remain errors.
[Reachability reviews](evals/dependency-audit/exceptions.json) bind exact versions,
advisories, source bytes, owner and expiry. Stale reviews become visible warnings
and stop suppressing findings. Do not renew them merely to clear a check.

The design lock covers OpenPencil, codecs, CanvasKit and Yoga. Pinned remote-image
scans cover detected OS/language packages; candidate layers additionally require
verified image exports. See the [audit workflow](evals/dependency-audit/README.md)
for coverage and strict opt-in policy. Neither a successful report nor a hash
establishes exploitability or safety.

Run with Python 3.11 or later:

```sh
python3 evals/dependency-audit/check.py --output /absolute/evidence/dependency-audit.json
```
