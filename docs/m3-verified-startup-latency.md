# Verified installed startup latency

The installed startup path verifies the entire exact release before admitting
MCP traffic or opening an engine. On 2026-10-25, profiling showed that the
5.0–5.8-second startup recorded in
[the Codex discovery investigation](retired-evidence.md#retired-docs-m3-codex-live-remediation-md) was dominated
by byte hashing in a development build. An optimized build helps substantially,
but its serial installed catalog still exceeded Codex's observed one-second
optional-server catalog grace.

`VerifiedRelease` now verifies large file inventories using at most four scoped
read-only workers. It retains the complete manifest, compiled inventory,
ancestry, byte-length, SHA-256 and file-identity checks. Installed file modes
are checked using each file's existing hash metadata snapshot; the existing
post-read identity/Unix ctime check rejects changes during that read. This
avoids another ancestry/stat walk for every installed file. Final manifest and
directory mode checks remain. No verification cache, dependency, global Codex
setting or early operational readiness was added. Release-store session
admission and full verification before launcher handoff remain unchanged.

## Baseline and attribution

The original disposable installed release was
`sha256:0fdf6a45d9d0ec0b51289ec16b7dd8cb23e51cffeaec0bba6dc3f51512693aea`.
It contains 6,852 regular files, two exact symbolic links and 551,208,448 payload
bytes. Its saved manifest is 1,619,692 bytes. The largest files are Node
(122.5 MB), the admitted ONNX model (118.4 MB) and the development core (80.6 MB).
All sizes here are decimal bytes/MB.

A temporary private unit-test profiler used the real parser, compiled layout,
inventory walker and hashing functions against that unchanged installed
release. It was removed from shipped source after measurement. The initial
profile was built with the normal test profile and no optimization override.
The table reports one process's first pass and two subsequent passes; these
are diagnostic observations, not cold-filesystem or throughput guarantees.

| Phase | First pass | Subsequent passes |
| --- | ---: | ---: |
| Select root and read/parse bounded manifest | 178 ms | 14 ms |
| Validate compiled inventory/layout | 147 ms | 137–141 ms |
| Inspect complete saved inventory | 143 ms | 125–138 ms |
| Read and hash every payload file | 4,365 ms | 3,783–3,822 ms |
| Compute canonical manifest identity | 35 ms | 34–35 ms |
| Additional installed-file mode/path walk | 160 ms | 159–161 ms |
| Total measured phases | 5,026 ms | 4,253–4,310 ms |

The file-mode phase reproduces that walk's path/stat work; directory-mode and
engine work were not included. The first manifest read includes initialization
of the process-local compiled maximum-manifest bound.

Private Cargo profile overrides established the compilation contribution
without altering dependencies or release bytes. Optimizing only `sha2` reduced
the phase total to 1,662 ms initially and 1,227–1,228 ms subsequently.
Optimizing both Mirket and `sha2` reduced it to 1,499 ms initially and
982–989 ms subsequently. Three additional fresh processes, each verifying the
same original release, took 1,083, 1,077 and 1,307 ms for their first pass.
These experiments did not justify a subsecond installed-startup claim.

## Actual optimized installed path

A normal `cargo build --release --locked --offline --bin mirket`, with no
profile override, produced a 25,234,288-byte core. A task-owned copy of the
original payload changed only its core and exact manifest. It was installed
through `setup` in a separate disposable home as release
`sha256:b44d8a864c1856ca96c51f1d7584c16e7f651700db74bcacbe3a2667587f3794`.

Five fresh processes invoked that installation's actual stable launcher,
completed initialize and requested the complete 48-tool catalog. Their
spawn-to-catalog times were 1,502, 1,015, 1,053, 1,008 and 1,051 ms.
`tools/list` itself took 1.4–2.1 ms. All processes exited successfully with
empty stderr. Optimized packaging alone therefore still missed the observed
one-second grace in every sample.

The bounded-worker/mode-check candidate was subsequently compiled with the
same standard release profile. Against the unchanged original 551 MB release,
its read-only `doctor` took 1,472 ms on first invocation and 471, 460, 453 and
450 ms on subsequent fresh-process invocations. The serial release executable
had taken 1,821 ms initially and 1,084, 1,078, 1,110 and 1,145 ms subsequently;
durable staging was active during that serial diagnostic. These are complete
`doctor` timings, including launcher-byte comparison, rather than isolated
hash times.

That candidate was then installed through `update` in the same task-owned home
as release
`sha256:4e86ef5ffa27581b3ccaa0cd780639f2bf6fa7d45f06d8be38f179ce0aa0d6e7`.
Its five actual-launcher spawn-to-catalog times were **1,139, 566, 535, 572 and
501 ms**. Every process returned all 48 tools, exited successfully and emitted
no stderr. The reduction is material, but the first-invocation one-second
default-grace gate remains open. Qualification must retain that first sample.

## One bounded stripped-binary control

One additional experiment applied the local macOS `/usr/bin/strip` to a copy
of the bounded-worker release executable. The original 25,279,824-byte binary
was retained. Its SHA-256 is
`10e2fb9deb3295fb3f19cb78eb2f6669afd2219399065015b40e2f7dd8d470e7`.
The stripped copy is 20,164,856 bytes, SHA-256
`f0798c992ddb5039f4919d2a15d4cdeff992e3926b8f25889e4a2259ac925595`.
`codesign --verify` accepted both copies without resigning. The stripped core
was installed through the same disposable `update` path as release
`sha256:ae92150d8be2c54e383b1216820b1fb118a3da916433f78544ecde25eba415e3`.

For each of three samples, separate fresh executable copies measured the first
`--help` invocation and the first full installed catalog invocation. Help did
not run in the catalog copy. All catalog processes returned the complete
48-tool catalog and an exact `runtime_status` release identity, then exited
successfully with empty stderr.

| Fresh-copy sample | First help | First installed catalog |
| --- | ---: | ---: |
| 1 | 553 ms | 1,005 ms |
| 2 | 374 ms | 870 ms |
| 3 | 394 ms | 852 ms |

This one experiment still missed the one-second catalog threshold in its
first sample. No production strip/profile change was adopted, no additional
tuning campaign was run, and the first-invocation discovery gate remains open.
The final disposable home retains this experimental release; its preceding
unstripped release is retained as the previous activation.

## First executable invocation is a separate limit

A control copied the same optimized candidate to three fresh executable paths
and invoked only `--help`. The first invocations took 1,089, 407 and 399 ms;
the second invocations took 4.9, 4.8 and 5.7 ms. This command performs no
installed-release verification or engine opening. The observation establishes
additional first-invocation cost; it does not identify its exact macOS loader,
security or filesystem cause. No OS cache was evicted and no device setting
was changed.

Consequently, faster verification does not establish a universal one-second
catalog guarantee. A lazy catalog inside the same executable also cannot
honestly guarantee that deadline solely from these measurements.

## Installed deferred catalog — 2026-10-26

The macOS installed launcher now acquires and retains the existing session
guard, binds the active pointer, and advertises the compiled MCP schema while
one connection-owned task verifies the complete exact installed release. This
early schema is discovery metadata, **not** a payload-health or release-identity
claim. Every tool call, including `runtime_status`, waits for verification and
running-core comparison. The first operation then opens the engine exactly
once; concurrent calls share it, and an opening failure is cached for that
connection. A catalog-only disconnect waits for the read-only verifier but
never opens the writable engine. The old eager path remains for non-installed
and as-yet-unqualified Windows launchers. No new state format, global Codex
setting, dependency or private backend handoff was added.

The final complete installed catalog test after this change used fresh
disposable macOS ARM64 release
`sha256:541cf53bac921bf2e976c214e13b1e855fc8788201dcec147fd16016651431f2`
with the private Node payload and 40 compiled tools. Its first executable
invocation answered initialize in 912 ms and `tools/list` in 922 ms from spawn.
A later invocation answered in 7 and 12 ms; its first `runtime_status` call
completed at 1,894 ms after full verification. Two simultaneous first calls
completed at 1,999 ms and opened exactly one engine owner. The catalog-only
connection created no engine owner record. Same-length adapter tampering still
let the client inspect the compiled catalog, then produced an explicit
verification error on the first call and no healthy result. These observations
are from one fixture, not a cold-start or one-second guarantee.

A separate task-owned stripped, ad-hoc-signed launcher with SHA-256
`c67d3ad1ceb04e25c30c67177d63886b4d4f1f14da8a1552097a8e32c2aeb893`
was executable but differed from the active release's original core. It
advertised the static catalog, then refused its first operation after exact
verification and running-core comparison. Restoring the original launcher
returned the disposable home to `doctor-core: healthy`, with the original core
hash intact.

The bundled Codex CLI also completed a read-only model turn against the full,
unfiltered installed catalog with `required=true`, the normal 60-second server
timeout and **no global catalog-grace override**. It called `file_read` once and
returned release
`sha256:541cf53bac921bf2e976c214e13b1e855fc8788201dcec147fd16016651431f2`,
the independently checked source revision
`sha256:a3fb28b7b6f8afcfdfb16dbe6c7f3cef37ef87517f53a5ff7c78d23090764e12`,
and the exact UTF-8 fixture including its trailing newline. The prompt allowed
metadata-only tool discovery. Earlier restrictive prompts that prohibited
other tools reported `file_read` unavailable even with zero grace; paired
discovery-allowed runs succeeded both with a filtered catalog and the full
catalog. That contrast shows a model-selection/prompt sensitivity, not a
measured server failure. The actual Desktop GUI remains untested.

Early catalog metadata can precede discovery of a corrupted payload or a
running-core mismatch; both are refused before the first operation. A stricter
pre-catalog core-byte check would reintroduce the measured startup delay and is
not claimed here. Native Windows, tamper varieties beyond the tested adapter
and core changes, and a cold-start timing guarantee remain qualification limits.

## Evidence and limits

Private profiles, exact build identities, source snapshots, timing scripts,
JSON samples, setup/update receipts and focused-test output are retained at
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/verified-startup-latency-20261025/`.
The bounded-worker regression exercises both one and four workers, last-file
corruption, exact installed modes, ctime rejection after a mode change and
stable manifest-order selection between distinct failures. The distribution
suite (20 tests), release-store suite (17 tests) and all-target Clippy with
`-D warnings` passed. The existing suites cover strict inventories,
symlink rejection, source/installed mode distinction, staging, activation and
recovery behavior.

The final task-owned stripped installation also passed two actual startup
failure probes. Same-length adapter-byte corruption and installed file-mode
drift each caused exit code 1 with zero MCP stdout, the expected integrity
error and unchanged writable-state file bytes. The original adapter bytes and
mode were restored after each probe; `doctor` then reported that exact release
healthy. Every recorded catalog probe process was absent afterward.

The deferred-catalog disposable installations, direct MCP and Codex model
traces are retained separately at
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/codex-fast-catalog-20261026/`.
The actual Codex test reused the existing local authentication route without
reading or copying credentials; the real configuration hash was unchanged.
After the final connection-owned change, the build-input guard passed, the
offline Rust run passed 495 library tests (three ignored), three CLI tests and
one non-ignored integration test, and all-target Clippy passed with warnings
denied. The private-Node adapter build and suite passed 89 tests with one
actual-browser test skipped because Chromium has no admitted release payload.
The explicit development MCP stdio smoke passed source, ownership,
lost-response and cancellation checks. The disposable final installed home
passed `doctor-core` again after restoring the deliberate tamper.

The platform was macOS 26.5.2 ARM64 with Rust 1.99.1 and the locked offline
dependency closure. Every Cargo build/test was preceded by the project's build
input guard. The original installed payload and user configuration were
preserved. All installation mutations used the task-owned disposable home.
No commit, push, publication or new dependency was introduced. Fresh-process
samples do not establish cold-disk, Windows, GUI activation, model-driven
approval or general performance guarantees. The bounded model turn above is
evidence of one default-policy discovery and read, not a promise for every
prompt or GUI turn.

## Later installed startup-path check — 2026-10-26

After the Raster recipient resolver was wired under the session guard, the
separate healthy uninstall fixture's exact installed core passed an MCP
`runtime_status` call through both launch paths. The installed launcher
initialized in 1,201 ms and answered the first call in 2,663 ms; the same
release's external source executable initialized in 2,350 ms and answered in
2,358 ms. Both listed 44 tools and returned release
`sha256:2114162844215fdd7c2dd648e1057f8f7c2cb3888474c65acedb262e8ea5b695`.
`tests/deferred-catalog-smoke.mjs` now accepts an optional absolute `--binary`
to exercise the external eager path; its default remains the installed
launcher. These are one-off Mac observations, not a comparative benchmark or
Codex Desktop GUI check. Raster remained unadmitted in this release.
