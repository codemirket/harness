# Image remediation, Penpot inventory and probe accounting — 20 October 2026

This follows the [dependency and accounting report](harness-gaps-2026-10.md).
The candidate updates owned images, restores an inspectable final Penpot OS,
and records vision-probe costs. It is not deployed, released or security-cleared.
The image gate remains red. The approved review branch now has current Windows,
macOS and Linux CI evidence, including packaged installer and MCP journeys.
No release or desktop deployment was performed.

The [evidence manifest](harness-image-remediation/checks.json) binds the candidate,
component tests, image identities, failure records and retained raw evidence.

## Priorities and disposition

| Priority | Boundary | Implemented result / remaining condition |
| --- | --- | --- |
| P1 | Owned image advisories | Frozen Debian security snapshot advanced to 20 October; Chromium patched; esbuild rebuilt with patched Go and scannable metadata; stale pip bundle removed. Remaining findings retain the failing gate. |
| P1 | Penpot backend OS inventory | New pinned Debian runtime retains its package database. Final scan detects 228 Debian packages; application payload comparison and isolated fresh-database/restart tests pass. Existing-data migration is not approved by these tests. |
| P1 | Windows validation | Native CI runs even when another prerequisite fails, while preserving those failing gates; manual dispatch is available. Current Windows x86-64 CI passed the full Rust gate and all four packaged installer/update/MCP journeys; retained receipts bind the reviewed source. |
| P2 | Probe usage and cost | Durable pre-dispatch intents, success/failure receipts, distinct resumed probes and CLI inspection implemented. Live native fixture accounts for probe plus worker calls. Authoritative billing remains unavailable. |

## Image changes and discriminating evidence

The owned design image previously shipped Chromium 154.0.8037.92. Its refreshed
frozen snapshot supplies 155.0.8059.79; the contained regression checks the Debian
security floor and exercises a real browser. For example,
[CVE-2026-102322](https://security-tracker.debian.org/tracker/CVE-2026-102322)
affects older Chromium Site Isolation code. Scanner matching is not an
exploit demonstration in this harness.

Simply taking the newest esbuild npm binary was insufficient. Version 0.28.2
contained Go 1.26.5 and lacked recognizable Go build metadata; Trivy stopped
reporting the old Go findings. The first candidate's archive, binary hash and
scan are retained as evidence of that false improvement. The final recipe
rebuilds the same esbuild 0.28.2 source using SHA-256-pinned
[Go 1.27.3 archives](https://go.dev/dl/) and the upstream locked module checksums.
It retains the Go metadata and license texts, and ships no Go compiler.
The regression checks the actual binary's metadata and version, then bundles
and executes TypeScript. The scanner now requires the known Go inventory for
this image, so a stripped binary cannot silently appear fixed again.

The agent image's owned locks already selected urllib3 2.8.0 and setuptools
84.0.0. Vulnerable extra copies were vendored in pip, including urllib3 2.7.0,
msgpack 1.1.2 and setuptools 70.3.0. After installation and `pip check`, the image
removes pip. Its contained regression checks pip is absent and the actual
runtime package versions remain correct; the real agent-runtime suite passes.

The Penpot MCP final layer receives the same frozen OS security upgrade. Its
initial candidate unnecessarily installed ca-certificates/OpenSSL packages and
introduced 14 additional scanner matches. The final recipe uses Node's existing
CA roots for the bounded build-time APT operation, retaining the original runtime
package scope. Both candidate scans remain in the private evidence.

The final owned-image comparisons and unresolved advisories are recorded in
[image results](harness-image-remediation/images.json). Counts are scanner finding
occurrences, not unique exploitable defects. No image exceptions were added.
Across the three previously scanned owned images, occurrences fell from 2,279
to 1,598, including 261 fewer high/critical matches. Design tools now has 1,116
matches (309 high/critical), Penpot MCP 228 (57 high/critical), and agent-runtime
254 (63 high/critical). These remaining high/critical matches have no fixed
version in the scanner's current report; that does not establish safety.
The rebuilt backend has 685 matches, including newly visible OS findings, so
its count cannot be compared with the old image's missing OS inventory.

Retaining Go metadata also exposes the existing x/sys match
[GO-2026-5024](https://pkg.go.dev/vuln/GO-2026-5024). Its affected symbol is
`windows.NewNTUnicodeString`, with `goos: windows`; the shipped esbuild artifact
is Linux ARM64. This is a platform-specific reachability distinction, not an
exception, and the scanner match remains in the report.
The separate source-lock audit queries 828 coordinates, including all 299 Penpot
MCP coordinates; zero new untriaged findings and three existing ZIP exceptions.

## Closing the Penpot OS inventory gap

The pinned upstream backend performs package upgrades/installations and then
removes `/var/lib` in the same layer. Earlier-layer dpkg files therefore describe
a different package state. Restoring those files would manufacture an inventory.
The digest-bound producer SPDX attestation is also insufficient: its 185 entries
include only seven Debian source/binary detections, not a final installed inventory.
The registry attestation, provenance, exact layer observations and hashes are retained.

The owned backend copies `/opt/jre`, `/opt/imagick` and `/opt/penpot/backend` from
the exact pinned upstream payload onto a pinned official Debian trixie layer,
installs the upstream runtime package set from a frozen snapshot and retains
`/var/lib/dpkg/status` and copyright metadata. A read-only archive comparison
honoring whiteouts confirms all 407 payload entries match in content, modes
and link targets. Ownership changes intentionally to the service UID.

`mirket dev backend-check` creates a private internal network and fresh tmpfs
Postgres/Valkey/backend instances with the normal service environment and resource
limits. No host port or saved volume is exposed. The check observed 60 migrated
public tables, the expected anonymous profile before and after backend restart,
and functioning Java 26.0.2/ImageMagick 7.1.2-27 binaries. Cleanup succeeded.
[Backend runtime result](harness-image-remediation/backend-runtime.json).

The final backend scan detects 228 Debian and 167 JAR packages. This closes the
final image's missing OS-package inventory, not every possible binary dependency
blind spot. The original payload input still lacks a complete OS inventory and
remains visible as such. The rebuilt backend retains 11 fixable high Java matches
in Netty, Jackson and lz4-java. Those need a reviewed upstream rebuild and application
regression evidence; blindly replacing jars inside the application is not a fix.

The production generation/data-upgrade guard remains in place. A fresh database
and restart do not prove migration of existing user data, GUI/editor workflows,
or acceptance of a new live design generation.

## Probe accounting and billing boundary

Probe intents are saved before inference under each run's existing private
`probes/` directory. Completion writes a separate exclusive result with numeric
provider usage, SDK estimate provenance and bounded diagnostics. A crash leaves
a pending/interrupted intent with unknown cost. Failed preflight retains a run
path even when no worker checkpoint exists. Resume generates another probe ID;
reading the same receipts repeatedly never adds another attempt.

`mirket agent accounting --run-id routed-ID` derives separate worker/probe
subtotals and a combined total. Missing costs or older untracked probe history
keep the total null. Discovery uses authenticated account/control/model-list
requests and sends no inference prompt; that protocol fact is not account-wide
billing evidence. `billing_usd` is explicitly null. Native Codex USD and account
invoices remain unavailable through the existing adapters.

The live Claude Sonnet fixture inspected four synthetic color quadrants correctly.
It recorded one probe at $0.01031 plus two worker calls at $0.0369128, totaling
$0.0472228 **SDK estimate**, not billed spend. Two independent CLI reads returned
the same three attempts and totals. Regression tests cover failed native dispatch,
resume attempts, interruption, unknown and legacy history, receipt attribution,
exclusive publication, and the existing native usage/cache/estimate semantics.
[Native accounting evidence](harness-image-remediation/native-accounting.json).

## Platform evidence and remaining work

The candidate-bound macOS gate passed 416 Rust tests; native Linux ARM64 passed
415 (one test is macOS-only). Both report zero failures and seven intentionally
ignored tests. Linux also passed catalog, disposable setup/doctor and actual MCP.
The design container passed all 82 tests, and the real agent-runtime suite passed.
The manifest binds those component checks and the final packaged macOS journeys.
The first packaging attempt timed out locating standard-library license notices.
A subsequent attempt was sampled blocked in a `read` of a 3,372-byte source
reference. Both attempts and the sample are retained. Packaging succeeded from
the already verified frozen source snapshot with the same compiled fingerprint,
without increasing timeouts or omitting notices. The underlying filesystem cause
was not established.
A final rehash of the original checkout also exceeded its 60-second read bound;
an independent Git whitespace check stalled reading a Git object. The current
checkout was therefore not freshly certified equal to the frozen candidate.
The passing platform evidence binds the frozen source fingerprint, and the
two final documentation contract tests passed after recompiling against that
snapshot. For the review branch, a fresh isolated clone received the manifest-verified
snapshot and final public evidence. All 689 committed files matched that prepared
checkout byte for byte, and the 391 compiled inputs retained the same fingerprint.
The original checkout was not used as the source for publication.
Windows cannot be established from a cross-compile, static inspection or an older
CI run. The workflow preserves all required checks and artifacts, adds manual
invocation, and prevents an unrelated prerequisite failure from skipping native
platform validation. After explicit user authorization, commit
`bb6956245e083662c04fad975eddedbe8d7136b9` was pushed to
`review/harness-remediation-windows-20261020`. This publishes a review snapshot,
not a release or deployment. No native CI failure required a source fix.

The next security work is an upstream-backed patch set for remaining Penpot Java
and remote service images, with their own runtime checks and existing-data migration
plans where applicable. Keep source-image coverage gaps and unsupported binary
inventory separate from final runtime OS coverage; do not replace either with an
exception or a green receipt. Account-wide billing needs an authoritative provider
billing source; inferred token prices cannot close that gap.

## Hosted Windows and native CI evidence

[Run 39469703515](https://github.com/codemirket/harness/actions/runs/39469703515)
executed the approved snapshot. The [CI record](harness-image-remediation/ci.json)
retains job identities, exact commit and source fingerprints, executable hashes,
artifact metadata, native receipts and command-output hashes.

| Hosted target | Rust passed / failed / ignored | Packaged native journeys |
| --- | --- | --- |
| Windows x86-64 MSVC | 385 / 0 / 7 | 4 variants; 21 commands, all successful |
| macOS ARM64 | 416 / 0 / 7 | 4 variants; 21 commands, all successful |
| Linux x86-64 GNU | 415 / 0 / 7 | 4 variants; 21 commands, all successful |

Each native job passed formatting and Clippy with warnings denied, built the
optimized CLI and three standalone installers, then exercised disposable setup,
doctor, update, updated doctor and MCP. Downloaded evidence and all twelve
packaged executable hashes were checked against their receipts. All three
platforms report source fingerprint
`712494f47832c1ae5268d4d8bf4806221e66582c2356f17c860f4026069767cc`.
Different Rust totals reflect platform-conditional tests; seven explicit opt-in
tests remain ignored per runner. Hosted Linux x86-64 supplements the earlier
local Linux ARM64 evidence. These checks do not establish desktop GUI activation,
contained service acceptance or migration of existing data.

The JavaScript, source-dependency and website checks passed. The source audit
queried 828 coordinates with no new untriaged findings and the three retained ZIP
exceptions. The image job completed all ten remote scans and failed as required
on untriaged advisories and incomplete coverage, including the upstream backend
payload's absent OS inventory. Its remote-only scope explicitly excludes four
built images; prior local immutable-image evidence remains separate. The overall
workflow is therefore **failed**, despite all native platform jobs passing.
No thresholds, exceptions, tests or release gates were weakened.
