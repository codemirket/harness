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
The locked dependency graph and pinned toolchain are authoritative. After a
production dependency change has been approved under [AGENTS.md](AGENTS.md),
run `mirket dev lock --project .` to resolve the workspace requirements while
retaining other locked versions where their constraints permit. Review the
resulting `Cargo.lock` diff before running checks; lockfile reconciliation is not
permission to add a dependency.
`mirket dev test FILTER` runs a focused test selection. `--ignored` explicitly
selects opt-in tests, including real package downloads.

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

## Add a local MCP package

Use the existing package registry and setup flow. Every installable definition
needs exact source and version pins, supported targets, complete runtime and
dependency closure, artifact digests, extraction bounds, launch arguments and
retained license notices. Keep dependency resolution locks and reviewed notices
with the definition. Installation must not run upstream helpers, lifecycle hooks,
`npm`, `npx` or `pip`. Ask before adding a production dependency as required by
[AGENTS.md](AGENTS.md).

Review the actual entrypoint, optional native modules, runtime downloads, network
behavior and tool effects. Restrict a profile only when its promised capability
still works; hold an incomplete profile instead of presenting it as available.
New selections must remain explicit, and setup/update must preserve saved choices
and unrelated client settings.

Use an isolated home and project for real package checks:

```sh
mirket --home /absolute/test-home setup --target all --yes
mirket --home /absolute/test-home package install jscpd
mirket --home /absolute/test-home package status
mirket --home /absolute/test-home package probe jscpd --project /absolute/test-project
```

Then use `mirket package call` with a meaningful fixture and a discriminating
failure case for the selected profile. Check written artifacts and effects, not
only a successful response. Run networked downloads and native SDK/device tests
explicitly; report missing prerequisites and untested targets. A registry entry
or successful probe alone does not establish useful runtime behavior. Keep
temporary packages and fixtures out of shipped source and clean task-owned
scratch when finished. The [package guide](docs/packages.md) defines the product
workflow and current limitations.

## Exercise installed packages

The live package suites are ignored by normal checks. They start the actual
`mirket package serve` processes through an MCP client and use disposable
fixtures. Run them only on a supported native target after installing the
required packages in an explicit disposable home:

```sh
mirket --home /absolute/test-home setup --target all --yes
mirket --home /absolute/test-home package install jscpd playwright dbhub safe-docx semiotic markitdown
```

The home must contain a file named `.mirket-package-test-home` with exactly
`disposable package test home` followed by a newline. Create that marker only in
the disposable installation. The suites refuse an absent or different marker;
an environment variable pointing to a real user home is not sufficient consent.

On supported macOS or Linux targets, run the suites explicitly:

```sh
MIRKET_PACKAGE_TEST_HOME=/absolute/test-home mirket dev test packages_live --project . --ignored
MIRKET_PACKAGE_TEST_HOME=/absolute/test-home mirket dev test packages_documents_live --project . --ignored
MIRKET_PACKAGE_TEST_HOME=/absolute/test-home mirket dev test packages_python_live --project . --ignored
MIRKET_PACKAGE_TEST_HOME=/absolute/test-home mirket dev test packages_lifecycle_live --project . --ignored
```

The browser/SQLite suite changes a real local page, reads a fixture database and
checks denied operations. The document suite edits, saves and reopens DOCX files,
checks source preservation and overwrite refusal, and reconciles SVG chart marks
with their data. The Python suite checks local conversion and refused network,
escape and unsupported-content inputs. The lifecycle suite verifies removal,
reinstallation, update repair, drift refusal and preservation of unrelated client
settings. The SQLite suite changes the disposable home's database selection;
configure another existing fixture database before running a full doctor on
that home. These checks do not establish native
client activation, device readiness or document layout acceptance.

To retain the document suite's outputs, set `MIRKET_PACKAGE_TEST_OUTPUT` to an
existing absolute disposable evidence directory. It writes its fixed artifact
names there, replacing earlier test artifacts with the same names. Keep evidence
outside shipped source, inspect the saved files and retain only useful artifacts.
The suites create and remove their own task fixtures; remove the separately
provisioned test home and unneeded evidence when verification is complete.

For the editor profile on macOS ARM64, use a local disposable home outside
file-provider folders. Install both signed applications, create the same explicit
test-home marker, then run:

```sh
mirket --home /absolute/test-home package install inkscape glaxnimate
MIRKET_PACKAGE_TEST_HOME=/absolute/test-home MIRKET_PACKAGE_TEST_OUTPUT=/absolute/evidence mirket dev test packages_editors_live --project . --ignored
```

The editor suite exercises all eight tools against native applications, distinct
editable/exported artifacts, malformed or unsupported input, source preservation,
overwrite refusal, saved-choice update replay and managed-copy drift. Its SVG
fixture is durable test source; generated PNG, PDF, native animation and browser
evidence belong outside shipped source. Inspect the saved illustration at intended
and small sizes, animation samples and actual browser playback. Neither byte
checks nor differing frame hashes establish visual acceptance. Normal gates also
cover fake-success outputs, transport disconnects, deadlines, request cancellation,
descendant cleanup and archive failures.

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
mirket dev manifest --input /absolute/macos/mirket-release.json --input /absolute/windows/mirket-release.json --output /absolute/release/mirket-release.json
```

The command verifies the listed local binaries against their checksums, requires
one version and unique targets, and writes a new manifest without executing the
binaries. Upload that manifest, the packaged executables, checksums and notices
only after publication is explicitly authorized. If native notice files differ,
publish each with its executable target in the filename rather than dropping one.
The operating system must match the executable target.
