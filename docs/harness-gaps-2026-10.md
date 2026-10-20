# Accounting, dependency coverage and platform validation — 20 October 2026

This follows the [completion report](harness-completion-2026-10.md). Native
accounting, Penpot lock coverage, image scanning and repeatable native-platform
receipts are implemented. The image audit now exposes substantial untriaged
findings and a missing OS inventory; this is not a clean security bill.
The new candidate has not been deployed or published.
The [verification manifest](harness-gaps/checks.json) binds this report, the final
candidate, regression results and retained evidence.

## Priorities and disposition

| Priority | Gap | Result and next concrete action |
| --- | --- | --- |
| P1 | Image and OS dependencies were outside the audit | All nine pinned base/service images and three installed built images scanned. Eleven OS inventories usable; Penpot backend inventory missing. Keep the audit red until findings and missing coverage are resolved. |
| P1 | Native adapters discarded received usage | Numeric usage and estimate provenance now survive in per-attempt traces and run receipts. Actual Codex and Claude fixtures completed. Unknown costs remain null. |
| P1 | Platform results lacked durable candidate binding | Shared native validator checks host, checksum, compiled source, four installer variants, updates and actual MCP. Current macOS package passes; Linux result is recorded separately below. Current Windows and hosted Linux runs remain pending. |
| P2 | Penpot MCP lock closure was omitted | All 299 exact coordinates across both lock documents now enter the OSV gate. Source archive, lock and license are pinned. |
| P2 | Complete billing and probe accounting | Worker-attempt accounting is implemented. Native Codex USD cost, account-wide billing and discovery/vision-conformance costs remain outside the available evidence. |

## Native usage and cost

[Accounting before removal](https://github.com/codemirket/harness/blob/10246ad172879979bc8d1c58912023e55ea1d9ee/src/router/accounting.rs) retains only allowlisted numeric
counters. Native [Codex](../src/router/codex.rs) obtains usage from the validated
terminal Responses object rather than trying to reconstruct it from child
progress events. [Claude](../src/router/native.rs) records its terminal result
after tool-boundary and model identity confirmation, including failed terminal
results. Raw provider text is not copied into accounting.

The counters preserve cached input, cache writes and reasoning output when
reported. Anthropic's uncached input counters remain distinct from inclusive
Responses input counts. Repeated Claude assistant frames are not added to a
cumulative terminal result. Each native invocation starts a new session.

The run summary is derived from the durable attempt list, including retries,
restored attempts and interrupted turns without receipts. It separates known
estimated cost from unknown attempts and leaves the total null when any attempt
has unknown cost. Integer validation rejects negative/string counters; missing
values and arithmetic overflow do not become zero.

Claude's amount is explicitly `native_sdk_estimate`. Its SDK uses a local price
table, and the vendor warns that these estimates can differ from billing.
Codex supplies token usage through this transport but no USD amount, so its
cost remains null. Neither subscription usage nor an unpriced model is assumed
free. [Vendor cost semantics](https://code.claude.com/docs/en/agent-sdk/cost-tracking),
[Responses usage contract](https://developers.openai.com/api/reference/resources/responses/methods/retrieve).

The two actual native fixtures read the same synthetic `ready.txt` and completed:

| Provider | Recorded attempts | Usage observations | Estimated total |
| --- | --- | --- | --- |
| Codex, `gpt-6.1-sol`, high | 3 | 2 completed responses; one failed attempt remains unknown | null |
| Claude, `claude-sonnet-5-5`, high | 2 | 2 terminal results | $0.0332268, SDK estimate only |

The Codex transport failure remained visible and the bounded retry completed;
it was not silently removed from accounting.
[Live observations and failure receipt](harness-gaps/native-accounting-summary.json).
These live calls used the development candidate identified in that record.
Subsequent Rust changes concerned Linux verification and image-export
bookkeeping; final native/accounting regressions passed in the complete gate.
The final optimized package was separately tested through installation and MCP.

Regression coverage includes native cache/reasoning counters, unknown and
invalid values, estimate provenance, private-field exclusion, failure-result
accounting, interrupted/older turns, overflow, retries and the actual Codex
capability-proxy response path. Existing HTTP receipt tests still pass. This is
worker observability, not a monetary budget, invoice or account-wide meter.

## Penpot lock and image coverage

The OSV audit now queries **855 unique coordinates** across Cargo, owned npm,
agent Python and the Penpot MCP lock. It found zero new untriaged advisories;
the three existing source-bound ZIP exceptions remain.
[Lock audit](harness-gaps/dependency-audit.json).

The unchanged Penpot lock was extracted from the archive whose digest is pinned
in `registry/design-stack.json`. The capture retains its license and provenance.
The coordinate reader covers both YAML documents and refuses unfamiliar key
forms, empty inventories or changed source/lock/license identities. It inventories
every locked package, including build/dev packages, without installing them.
This distinction matters: a separate Trivy filesystem probe reported only 153
packages for the same lock, whereas the complete pinned coordinate inventory
contains 299. That partial scanner result was retained.
[Capture and source binding](https://github.com/codemirket/harness/blob/3bcfac091f6a34021e148f4500c0262e8c24a417/evals/dependency-audit/penpot/provenance.json).

The new [image audit](../evals/dependency-audit/image_audit.py) uses a pinned,
separately fetched development scanner. It records the scanner digest and
database metadata, requests all severities and both OS/library coverage, and
does not filter unfixed findings. Private per-run caches avoid concurrent scanner
locks. A changed target, incorrect architecture, missing OS inventory,
unsupported/end-of-life OS or scanner failure cannot pass.

`mirket dev image-export` exports only an installed, verified image identity
through the existing registered engine. It does not start image contents or
change runtime selection. Docker archives are hashed before scanning, and the
scanner's image ID must equal the export receipt. Publication of the archive is
exclusive; existing output and unknown aliases were exercised as refusal cases.
The final export implementation also produced the same Penpot archive digest
as the scanned archive; its saved manifest and platform were independently read
back and checked.
[Export refusal evidence](harness-gaps/export-rejections.json),
[saved archive verification](harness-gaps/export-atomic-verification.json).

All **12** required image targets were scanned. The results contain **10,523
finding occurrences** across **9,157 package/version/advisory tuples**, including
84 critical and 1,964 high occurrences. These are scanner matches, not that many
confirmed exploitable vulnerabilities. Shared packages and repeated occurrences
must not be interpreted as independent application defects.
[Image inventories, counts and raw-report hashes](harness-gaps/image-triage.json).

The Penpot backend scan found 167 Java packages and 16 language findings but no
OS packages despite recognizing Debian. Its OS coverage remains incomplete.
The final evaluator preserves language findings alongside that coverage failure.
A base-image inventory alone cannot establish the contents of this final image.
Obtain an exact-image SBOM from the producer or generate and retain an inventory
before package metadata is pruned; then reconcile it to the final image digest.

Raw image reports, scanner/database metadata, verified export receipts and all
failed attempts are retained in ignored project evidence under
`.mirket/implementation/harness-gaps-2026-10-20`. The compact public summary binds
each original report by hash. No scanner exception was added.

## Actionable image triage

1. **P1: refresh the active design browser in a reviewed candidate.** The installed
   image contains Chromium `154.0.8037.92-1~deb12u1`; Debian lists
   `155.0.8059.39-1~deb12u1` as the fixed floor for CVE-2026-102322, a
   crafted-HTML SiteIsolation vulnerability. Rendering supplied HTML is an intended
   harness path, so this is a stronger triage candidate than a severity-only
   ranking. Actual exploitability inside Mirket's containment was not tested.
   Update the frozen Debian security snapshot in the owned Dockerfile, build an
   isolated candidate image, verify its browser version, rerun the full browser/
   renderer suite and scan, then complete acceptance before changing live services.
   [Debian advisory](https://security-tracker.debian.org/tracker/CVE-2026-102322).
2. **P1: repair Penpot backend inventory coverage.** Keep the missing OS inventory
   visible and do not substitute its base image or a zero-finding language scan.
   Audit exact final-image package metadata or producer SBOMs, then triage the
   retained Java findings. This is required before claiming full Penpot coverage.
3. **P1: review newer pinned bases and optional service images separately.**
   Review fixed versions and applicable input paths, rebuild the owned images
   with explicit source pins, and repeat scans. PostgreSQL or retained Penpot data
   require migration/restore checks before any runtime upgrade. No database or
   service image was changed by this task.
4. **P2: resolve applicability before granting exceptions.** For example, the
   scanner marks CVE-2026-106195 critical, while the vendor describes a Mac-specific
   issue with Chromium severity Low; these images are Linux ARM64. Debian also
   notes that the affected contrib/minizip code for CVE-2023-45853 is not built by
   Bookworm's zlib source. CVE-2025-7458 requires crafted arbitrary SQL; its
   reachability through the agent runtime needs a separate review. These
   observations inform prioritization but did not silently remove findings.
   [Chromoting scope](https://security-tracker.debian.org/tracker/CVE-2026-106195),
   [zlib binary scope](https://security-tracker.debian.org/tracker/CVE-2023-45853),
   [SQLite prerequisite](https://security-tracker.debian.org/tracker/CVE-2025-7458).

## Platform validation and CI

[The shared native validator](../evals/harness-remediation/native_validate.py)
requires the exact host target, verifies the executable and all installer
checksums, binds the build record to the running source identity, and checks that
packages did not change during validation. Each variant gets a fresh disposable
home for setup, doctor, managed update, doctor and actual MCP SDK exchanges.
Both stdout and stderr plus a hashed receipt survive a failure.

The current optimized macOS ARM64 package passed all four variants and catalog
validation. Its source identity is
`59ed3c2c4ce3acf3f17d8bae86447584e574aa1b180a7452455c9e30e0a248a4`;
the binary SHA-256 is
`eef6811fb2cea64d25d406fc2000c30f28026a4d50e514a9bc2fd440b6341034`.
[Native package receipt](harness-gaps/native-macos.json),
[candidate build record](harness-gaps/candidate-build.json).

The local native Linux ARM64 check uses an explicitly selected existing engine
and a digest-pinned Rust 1.99.1 compiler image. It snapshots only public source
inputs, mounts that snapshot read-only, and keeps home directories, credentials,
engine sockets and private task state out. It runs the full Rust gate, catalog,
disposable setup/doctor and MCP. It is Linux process execution in a macOS-hosted
VM, not Windows, x86 Linux, GUI activation or a Linux package-controller claim.
The final run passed **411 tests**, zero failures, seven opt-in tests ignored,
formatting, Clippy, catalog, disposable installation/doctor and actual MCP
exchanges, with 7.5 GiB free at completion. Its compiled source matches the final
macOS candidate. The one-test count difference is a macOS-only prerequisite check.
[Linux receipt](harness-gaps/linux.json), [complete log](harness-gaps/linux.log).

The first full Linux attempt failed when two concurrent installer tests exhausted
the VM disk while copying debug executables. That failed receipt and log are
retained. The runner now disables debug symbols and incremental caches for its
disposable builds, records those settings and disk capacity, and preserves debug
assertions, all tests and the existing timeout.
[Original storage failure](harness-gaps/linux-initial-disk-failure.json),
[failure log](harness-gaps/linux-initial-disk-failure.log).

That rerun passed the installer tests with 7.6 GiB free, then reproduced a second
runner defect: its source snapshot omitted `website/`, required by the local-link
gate. The corrected snapshot includes it. A new Rust regression captures the
real checkout, compares its compiled-source fingerprint and required public
files, and confirms that `.git` and `.mirket` are excluded. A separate review
found no missing snapshot targets across 236 authored Markdown files.
[Snapshot failure](harness-gaps/linux-snapshot-failure.json),
[failure log](harness-gaps/linux-snapshot-failure.log),
[link review](harness-gaps/snapshot-link-review.json).

CI now preserves native gate logs and failed installer receipts on every
platform, pins Python for the common validator, and runs a separate image audit
job for the nine remote base/service pins. That job explicitly excludes locally
built layers; its scoped result cannot establish complete runtime coverage.
The image job is expected to fail on the retained untriaged matches.

Read-only inspection found Windows, Linux and macOS jobs passing on committed
baseline `d8013f53969db336d813274d1fbfba53a89ad0e2`. Those results do **not** verify
the current dirty working tree. Current Windows and hosted Linux validation
remain pending a permitted commit/push and CI run; no such publication action
was taken.
[Baseline run](https://github.com/codemirket/harness/actions/runs/39406745581),
[saved job evidence](harness-gaps/remote-baseline-jobs.json).

## Verification and limits

The current macOS Rust gate passed **412 tests**, zero failures, seven opt-in
tests ignored, with formatting and Clippy passing.
[Full log](harness-gaps/rust-check-final.log).
All **7 dependency/scanner** and **5 feedback/platform** Python regressions
passed; workflow YAML parsed successfully. The verification manifest records
final counts and hashes.

Failed attempts remain available: the initial Linux snapshot named a nonexistent
NOTICE file, concurrent scanner caches collided, and the scanner rejected OCI
tar input before the export switched to Docker archives. The corrected exports
scanned successfully. The existing accepted macOS package was also used first to
validate the new native-check harness, followed by the current candidate.

The 81-test JavaScript suite and prior contained acceptance were not rerun:
owned JavaScript, agent Python and runtime recipes were unchanged. They are
historical evidence, not new passes for this candidate. This candidate has no new
contained release-acceptance receipt and was not installed into the desktop.
The live desktop remains the previously accepted generation.

Remaining work is explicit: image applicability/remediation and the missing
Penpot OS inventory, current Windows/hosted Linux CI, complete native billing
and conformance/discovery accounting. No test threshold or timeout was lowered,
no image finding was waived, and no commit, push or release was performed.
