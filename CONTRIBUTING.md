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

`mirket dev fmt` formats the Rust sources. `mirket dev check` runs formatting, Clippy with warnings denied, and all Rust test
targets. Use `mirket dev build --release` to build the CLI and standalone target
installers. The locked dependency graph and pinned toolchain are authoritative.
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

Run `mirket benchmark` on an optimized binary for named local workloads; report
sample count, target and binary identity. Do not turn machine-dependent timing
into flaky correctness gates. Repeat performance tests after relevant changes.

## Add expertise

Read [source review](docs/source-review.md). Keep authored skills focused on a
recognizable deliverable, prerequisites, useful references and failure probes.
Preserve upstream attribution and licenses. Test the selected payload and its
project installation, then exercise the intended task where tools are available.
A skill's presence or an agent repeating its name is not evidence of quality.

## Package

```sh
mirket dev dist --project . --output /absolute/new-dist
```

This builds native release binaries, copies standalone installers and writes
checksums plus the release manifest. It does not publish. Native CI runners test
and package their target. Combine their manifests through the CLI:

```sh
mirket dev manifest --input /absolute/macos/mirket-release.json --input /absolute/windows/mirket-release.json --output /absolute/release/mirket-release.json
```

The command verifies the listed local binaries against their checksums, requires
one version and unique targets, and writes a new manifest without executing the
binaries. Upload that manifest and the packaged executables only after publication
is explicitly authorized.
The operating system must match the executable target.
