# Python navigation: ty 0.0.91 release qualification

Observed **2026-10-25** on **macOS 26.5.2 (25F84), ARM64, Apple M1 Pro**.
**Do not admit or advertise ty as a released Mirket capability.** The pinned
candidate passes the bounded direct-import navigation and passive-loading cases,
but its redistribution notice closure is incomplete, native Windows execution
is untested, and generic reference completeness is not established. This report
does not change the release inventory or expose a language tool.

Initial raw evidence is under the private task-state directory
`ty-qualification-20261025/release-admission/`; the independent follow-up is in
`python-ty-admission-20261025/` under the same implementation task-state root.
No user installation, old harness,
client configuration, source lockfile or release manifest was changed. Downloads
were inert files; no upstream installer, hook, build script or helper was run.
Cargo metadata/tree downloaded and inspected source packages without compiling
them. The pre-existing signed ty binary was executed only in disposable probes.

The official [0.0.91 release](https://github.com/astral-sh/ty/releases/tag/0.0.91)
was published on 2026-10-22. Its tag resolves to
`67fbd28745e4bf2f1afb97886c60960bac54f2c5`; its Ruff gitlink is
[`59a9e85a7b6dc4e4592cd29de32a214c150d663c`](https://github.com/astral-sh/ruff/tree/59a9e85a7b6dc4e4592cd29de32a214c150d663c).
`gh attestation verify --repo astral-sh/ty --format json` succeeded independently
for all three archives. The signed statement contains all their artifact
subjects. This establishes the checked artifact provenance, not a reproducible
build, complete linked-component inventory or redistribution permission.

| Official artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| macOS ARM64 `.tar.gz` | 12,881,796 | `8ec3e0315cab3f8969a974db1fb1d431ef38525793455dda411089e733c71fbe` |
| macOS ARM64 executable | 26,599,936 | `39ab93d3aefb55f00338770e33509291569040c20111f52a2bc43abb77850459` |
| Windows x64 `.zip` | 13,709,743 | `e25d02eba8a3d3a850a2775ae5a391f86485ec8aeace6ea02872ca77a183812a` |
| Windows x64 `ty.exe` | 32,220,464 | `9820580cd8ff717f382d474e627deb4d2b1746cc20d6e2aa65ea2ca5296b705b` |
| Windows ARM64 `.zip` | 13,268,815 | `42fcca1fd15b6c18159f1be83cd4b6600105837a353638537e3f43bca0b33614` |
| Windows ARM64 `ty.exe` | 30,723,376 | `ef25ae2a8dae38190c79a202624f2c03b105593b25ed7bf3019048b8299b2cbf` |
| Official `source.tar.gz` | 17,633,227 | `8e841a976d1b599d22026c67b492181a55942d67de539270ae07744f0e13fb80` |
| Official `dist-manifest.json` | 42,772 | `1480b821f9cc77b892ccd84fdf3e1a1ff7629a0b51ce032501ccee8e69b01343` |

Each binary archive contains only its executable. The macOS strict signature
check passed in the [initial qualification](m3-python-ty-qualification.md).
Windows PE headers identify COFF x86-64 and ARM64 respectively. Both contain
Authenticode certificate material identifying OpenAI OpCo, LLC; parsing those
certificates on macOS is **not** Windows trust/signature validation.

The x64 PE imports Windows system DLLs without a dynamic VC runtime import.
ARM64 additionally imports **`VCRUNTIME140.dll`** and UCRT API sets. The pinned
[build workflow](https://github.com/astral-sh/ty/blob/67fbd28745e4bf2f1afb97886c60960bac54f2c5/.github/workflows/build-binaries.yml)
explicitly verifies static CRT only for x64. The archive supplies no ARM64 VC
runtime. Its exact clean-machine runtime availability or private redistribution
closure, and the Windows compiler-runtime provenance/rights, remain open.
PE subsystem metadata is not evidence of the supported Windows floor.

The official source archive includes the complete nested Ruff tree rather than
an empty submodule. Its `Cargo.toml` and `Cargo.lock` match independent reads at
the pinned Ruff commit. Their respective SHA-256 values are
`500a8379affa70d415c341cb31912c9cde750d2a449fab9778c5da65c1f52468`
and `46f8a4fa82d5ac6dfd4ebd9eabb399179f02ce5127a7addc1780e8fb4ff9c7d2`.

`cargo +1.99.0 tree --locked -p ty --target TARGET --edges normal,build`
and target-filtered metadata identify 282 packages for macOS ARM64, 287 for
Windows x64 and 286 for Windows ARM64: **300 distinct normal/build packages**,
including 34 workspace packages and 266 registry packages. Every registry
archive was retained unchanged and independently matched to the Cargo.lock
checksum; the archives total 31,275,233 bytes. This is a conservative build
closure, including proc macros and assembly-only dependencies. It does not
assert that every listed package contributes linked bytes to each binary.

`source-inventory.json` records each package, selected targets, declared license,
source archive and recovered notice hashes. Notices were recovered for 296 of
300 packages, including pinned-upstream recovery where crate archives omitted
them. Four declared-MIT packages still have **no original notice text** in the
crate archive or inspected pinned source:

- `collection_literals` 1.0.2, source commit `5c5e88a5cfbf5603123187039c4382366d46edd1`.
- `quote-use` and `quote-use-macros` 0.8.4, commit `05096f346f8b17fba8a57a235b958917ad9d99e0`.
- `seahash` 4.1.0, declared source commit `94b632aeac099031c373599313d5b5f0acbbaec0`.

A second read-only retrieval obtained the complete pinned upstream archives for
all four entries. All **20 published Rust source files** matched their upstream
bodies byte for byte. Source availability is established; none of those archives
contains an original license/notice file, and a text scan recovered only Cargo
MIT labels and `include` entries, not copyright/permission text:

| Original-notice gap | Pinned upstream source body |
| --- | --- |
| `collection_literals` 1.0.2 | [Available at `5c5e88a5`](https://github.com/staedoix/collection_literals/tree/5c5e88a5cfbf5603123187039c4382366d46edd1), 4/4 Rust files match |
| `quote-use` 0.8.4 | [Available at `05096f34`](https://github.com/ModProg/quote-use/tree/05096f346f8b17fba8a57a235b958917ad9d99e0), 1/1 Rust file matches |
| `quote-use-macros` 0.8.4 | [Available at `05096f34/quote-use-macros`](https://github.com/ModProg/quote-use/tree/05096f346f8b17fba8a57a235b958917ad9d99e0/quote-use-macros), 6/6 Rust files match |
| `seahash` 4.1.0 | [Available at `94b632ae`](https://gitlab.redox-os.org/redox-os/seahash/-/tree/94b632aeac099031c373599313d5b5f0acbbaec0), 9/9 Rust files match |

A manifest's MIT label does not recover its missing original copyright notice.
The candidate is not cleared by substituting generic license boilerplate or
assuming that proc-macro/build dependencies impose no incorporated-code terms.
Determine the actual incorporation boundary or recover the original notice
before closing those entries.

The retained source includes ty/Ruff MIT terms, the full typeshed license and
additional terms, Unicode data licenses and native crate subcomponent notices.
Rust 1.99.0 library copyright/license documents are retained separately as
supporting evidence. Three selected packages declare MPL-2.0: `colored` 3.1.1,
`option-ext` 0.2.0 and `version-ranges` 0.1.1. Their exact unmodified source
archives and terms are retained, but a release must actually provide the
required source/notice arrangement. The prepared notice collection is evidence,
not an approved packaged notice bundle or an exhaustive binary SBOM.

The 2026-10-25 OSV exact-version query of all 266 selected registry packages
returned these findings. Primary records and the input/response are retained:

| Selected component | Finding | Relevance to the selected server |
| --- | --- | --- |
| `crossbeam-epoch` 0.9.18 | [RUSTSEC-2026-0204](https://rustsec.org/advisories/RUSTSEC-2026-0204.html), invalid pointer dereference during pointer formatting; patched in 0.9.20 | The exact source implements the unsafe path in `fmt::Pointer`. The advisory body says Display while its title and source identify Pointer. Selected callers use the concurrency machinery, but the callsite review found no formatting invocation on the affected Atomic/Shared values. No exploit reachability demonstrated. |
| `zip` 8.6.0 | [RUSTSEC-2026-0367](https://rustsec.org/advisories/RUSTSEC-2026-0367.html), unsafe privileged permission preservation during extraction | Selected callers read embedded typeshed entries or create an archive at build time; no ZIP extraction API caller found. |
| `zip` 8.6.0 | [RUSTSEC-2026-0368](https://rustsec.org/advisories/RUSTSEC-2026-0368.html), extraction of escaping symlinks | Same bounded callsite conclusion. |
| `zip` 8.6.0 | [RUSTSEC-2026-0369](https://rustsec.org/advisories/RUSTSEC-2026-0369.html), extraction permissions through symlinks; rated critical | Same bounded callsite conclusion. A scanner severity alone does not establish reachable impact in this use. |

The upstream ty security record also identifies
[GHSA-vxvm-j4xq-q7m4](https://github.com/astral-sh/ty/security/advisories/GHSA-vxvm-j4xq-q7m4):
crafted Python input could exploit incremental-analysis use-after-free without
executing Python. It affects ty before 0.0.84; this 0.0.91 pin is outside that
range. Its pinned `salsa` 0.28.5 is the patched version named by
[Salsa's advisory](https://github.com/salsa-rs/salsa/security/advisories/GHSA-xc3w-55vh-cw3w).
The read-only public Ruff advisory endpoint returned no entries. This dated
review is not a universal vulnerability or memory-safety guarantee. Reachability
must be revisited if archive handling, dependencies or the server version change.
The independent 266-version OSV refresh at **2026-10-25 12:42 UTC** returned
the byte-identical response. Independent local checks also reverified every
retained registry archive and notice, both Windows ZIP inventories and PE import
tables: 300 selected packages, 266 registry sources, 296 notice-covered packages,
and the same ARM64 `VCRUNTIME140.dll` dependency. Those checks close no native
Windows execution, trust or redistribution gate.

The independent unmodified server ran with private HOME/config/cache/temp paths,
an explicit executable path, `untrustedWorkspace: true`,
`experimental.useUv: off` and diagnostics disabled. macOS denied network,
process fork and process execution except the selected analyzer executable.
A separate control returned `PermissionError`/`EPERM` for child execution and a
localhost connection and created no marker. The upstream
[untrusted executor](https://github.com/astral-sh/ruff/blob/59a9e85a7b6dc4e4592cd29de32a214c150d663c/crates/ty_server/src/system.rs)
rejects external commands, and
[initialization options](https://github.com/astral-sh/ruff/blob/59a9e85a7b6dc4e4592cd29de32a214c150d663c/crates/ty_server/src/session/options.rs)
force uv off while preserving an untrusted setting on unrelated parse errors.
These controls are not a portable production sandbox.

The direct-import corpus passed **66/66** assertions across prepared environment,
absent environment, missing environment, an absolute interpreter sentinel and
explicit configuration-file cases. It verifies same-file and cross-file
definitions, all four expected direct references, document symbols, prepared
dependency `.pyi` resolution through an ordinary `.pth` path, unresolved-name
handling, saved declaration changes, restoration, unchanged original source
bytes, framing and clean shutdown. CRLF, Turkish identifiers/paths and a
non-BMP emoji before a call site are checked by translating UTF-16 ranges back
to the original UTF-8 byte intervals.

Fixtures also contain executable `.pth` lines, project module top-level code,
startup/sitecustomize/usercustomize/setup/build-backend files, fake build
requirements, PATH command sentinels and an absolute interpreter sentinel.
No marker appeared. Successful bounded navigation under execution denial shows
that these cases do not require successful subprocess execution; it does not
prove no attempted absolute-path command or all project forms are passive.
Missing dependency metadata yields a definition at the local import statement,
not a resolved external declaration. Production must report this as unresolved
or incomplete rather than treating any nonempty LSP response as success.

One initial direct-explicit assertion used the wrong scenario-name comparison;
the unchanged explicit-config behavior passed the corrected 13-assertion probe.
Initial traces and results remain retained. Other initial null expectations were
also corrected to distinguish import fallback from stale original definitions;
the alias-transitive reference failure below remains a failed requirement.

For `from kütüphane import selam as greet`, ty correctly resolves `greet(...)`
to the original function. References queried at that original declaration return
the declaration, same-file call and import `selam`, but omit `greet(...)`.
The identical source and prepared-environment fixture against **Pyright 1.1.414**
returns the identical three-location subset. The held criterion requiring the
alias call therefore fails for both candidates. This may reflect their symbol
and alias scopes rather than an upstream defect, but **neither result qualifies
Mirket's alias-transitive complete-reference contract**. A backend switch alone
does not repair it. Return incomplete/refused unless the integration establishes
and communicates a narrower supported semantic scope; do not silently label
this subset complete or repair it with lexical search.

`ty.toml` wins over a neighboring `[tool.ty]` configuration in the tested case;
an explicit `configurationFile` selects its configured target. Combining that
file with conflicting inline `extra-paths` still selected the file's earlier
path in this fixture, so an inline-array override is not qualified. Use one
explicit frozen configuration artifact and bind it to the project model.
The [editor-setting documentation](https://docs.astral.sh/ty/reference/editor-settings/)
and current source should not substitute for this actual search-path observation.

The configuration invalidation probe changed only `ty.toml`: source SHA-256 stayed
`5515acd4bebd5f83c5f30aaa7680367750f80bb9d02dda31c768e154fa005539`,
while model/config SHA-256 changed. With no watcher notification, the same
process retained the former definition. A fresh process resolved the new target.
The owner must invalidate a changed model before accepting a response and
restart or explicitly reload/revalidate the server; a source hash alone cannot
establish currentness.

A follow-up ran **9 scenarios / 76 assertions** against the same unmodified
binary with network denied and child execution allowed. The trusted-uv control
executed the harmless uv sentinel twice; untrusted mode suppressed it with
conflicting `TY_UV`/`useUv` settings and with an unrelated malformed setting.
An absolute interpreter sentinel, executable `.pth` lines and project/startup
sentinels also stayed unexecuted in untrusted mode. A separate network control
returned `EPERM`. This distinguishes ty's command suppression from an OS rule
that simply prevents every child process.

The same probes demonstrate limits of that control. With untrusted mode and an
explicit configuration, `PYTHONPATH` added an external search root; a variable
in `configurationFile` selected an external configuration; and an extra-path
symlink resolved to an external file. These are file reads, not command execution.
The [pinned search-path implementation](https://github.com/astral-sh/ruff/blob/59a9e85a7b6dc4e4592cd29de32a214c150d663c/crates/ty_project/src/metadata/options.rs#L433-L470)
also appends `PYTHONPATH` directly. An invalid explicit Python version produced
a project-load error and fallback **Python 3.15**, yet local definitions and
symbols still looked correct; dependency definitions fell back to local import
names. An unrelated malformed initialization setting preserved untrusted mode
but discarded other settings and restored diagnostics. Correct local output
therefore cannot attest the requested model or configuration. These negative
cases pass as observations and remain **production refusals**.

The direct prepared process initialized in 17.130 ms; its first definition took
31.138 ms, later cross-file definition 0.614 ms and references 0.763 ms. During
15 seconds of idle observation its sampled RSS reached and remained at
**41,712 KiB (40.73 MiB)**; the last samples reported 0.0% CPU and the server
remained responsive. These are one process-cold startup and a short idle
observation on this small corpus, with filesystem cache and concurrent desktop
load uncharacterized. They are not a cold-install benchmark, peak RSS, p95,
large-project result or idle-expiry qualification. No Windows runtime or oldest
macOS floor was tested.

The eventual leaf should reuse the existing private Rust [LSP owner](../src/lsp_client.rs)
(`Invocation::Server` and its three finite read operations), with no second
protocol bridge. Its generic launch and position checks do not qualify the
ty-specific configuration model. The required integration contract is:

1. Verify the fixed private executable identity before launch; pin 0.0.91 and
   never discover an analyzer/interpreter on PATH. Use `env_clear` followed only
   by explicit private runtime/config/cache/temp variables. Fix strict boolean
   `untrustedWorkspace: true`, `experimental.useUv: off` and diagnostics off in
   trusted integration code; accept no project/caller overrides.
2. Before launch, construct an immutable prepared configuration/source projection
   from admitted original snapshots and stubs, with a finite original-to-private
   path map and no escaping symlinks. Bind source/configuration/`.pth` identities,
   Python version/platform, dependency roots and analyzer identity to one model
   revision. Normalize accepted paths into the projection; do not pass ambient
   `PYTHONPATH`, expandable configuration paths, discovered user configuration or
   arbitrary original `.pth` roots to ty. An incomplete model refuses before spawn.
3. Accept only document symbols, definition and references requests, bounded by
   time, bytes and result count. Do not expose commands, installation, builds,
   edits or code actions as a side effect of navigation. Any configuration parse,
   project-load or initialization fallback must refuse the operation; successful
   LSP initialization and a correct local location cannot override that refusal.
4. Convert original UTF-8 byte positions to UTF-16 and validate returned ranges
   through the projection map against original saved bytes. Check approved paths
   and identifier slices; reject invalid encodings, split surrogate/codepoint
   boundaries and locations
   outside the admitted project/model roots. Preserve CRLF in the byte mapping.
5. Revalidate source and project-model revisions before publishing results.
   Invalidate/restart on model changes and classify unresolved import fallback
   explicitly. Results must carry revisions and an honest completeness state;
   reference truncation and unsupported alias coverage cannot become success.
   Keep alias-transitive references incomplete/refused until an independently
   qualified semantic path satisfies that held requirement; a location subset
   is not evidence of a complete reference set.
6. Own shutdown/exit, deadlines, cancellation, process cleanup and idle policy.
   This qualification is a backend experiment, not evidence of those Mirket
   production lifecycle paths or desktop activation.

Ty is the simpler development candidate because this path uses one native
analyzer with explicit untrusted-workspace behavior. Its
[version policy](https://github.com/astral-sh/ty/tree/0.0.91#version-policy)
still describes a beta API that can change across `0.0.x` versions. Pinning and
rerunning the corpus are required on every upgrade. The previously inspected
[Pyright candidate](m3-python-navigation-qualification.md) has unresolved
embedded WASM/native source-notice correspondence and reachable TOML parser
advisory concerns, and did not improve the exact alias-reference case here.
That comparison supports continued ty development work; it does not waive any
ty admission gate.

**Admission decision remains no.** A private integration is technically plausible
through the existing LSP owner after the above preflight is implemented and
verified. A production admission of these pinned archives is not justified yet:
recover the four original notices or establish their actual incorporation/rights
boundary, prepare the required source/notice delivery, close the ARM64 compiler
runtime closure, run native platform/floor checks, and resolve the held reference
coverage. A newer exactly pinned upstream candidate may be evaluated if it closes
those boundaries; switching to the already blocked Pyright payload closes none
of them. This task adds no public Python navigation tool or release admission.

Evidence identities:

| Evidence | SHA-256 |
| --- | --- |
| `source-inventory.json` | `20af7556776073e863ab4cced898cfbacafa71ab51f541b2c347f7759aef5828` |
| `notice-inventory.json` | `eda0425d086f079f954ffdf1319ba48b4895415699427b3b4c02cd2ca0309b53` |
| `direct-import-result.json` (retains initial explicit-name expectation error) | `a6c9b183ab3f86bfdc3a17ab7e52598616573ed1e65545e202b9b3c922aaaf07` |
| `direct-explicit-corrected-result.json` | `124456098bfa106695babeb4266d20496950d7a7b3e03a19df23f5903bfd99e8` |
| `extended-result.json` (alias and initial expectations) | `6697b1aac6ff1cc41266e9f9db0703455cb022ea51c6f3ba9013a81ad1e240d2` |
| `pyright-comparison-result.json` | `490b8f75aac2fcdcccf67ac114e4719740fbb4b2a954206c0537dc3948fab275` |
| `model-change-result.json` | `04527a9ca50110c702a77fbc3ff3a325a83bb6cdfbf2a0f05b76b47f06301f8e` |
| `osv-response.json` | `0d477da5e22f1006b2eccd6a34fed2f801bfa6eff290de939101ffaba4918bb3` |

Follow-up evidence in `python-ty-admission-20261025/`:

| Evidence | SHA-256 |
| --- | --- |
| `verified-evidence.json` | `dcd9bb908b6f42cc851fbd20e3063ad26f1f5462245104fc117cc6de883dfd4b` |
| `osv-summary-verified.json` | `b668aadf9d63614959f9ef998b867b47b1d3f4ae1e6454068dc412fba453f21e` |
| `notice-gap-upstream-archives.json` | `ca8e6ee210c65a967b7a9ea36f014f84e5159ee8dbbfcd4efcd764b300788755` |
| `passive-boundary-result.json` | `581a662d640ce112d13979c8f8b927770c4b81502b06da498f7ce8b452d13ba3` |
| `network-denial-control.json` | `d36b7422156c5905e968b3ee6a0e88dfe08a5eac382e19a68767d92b7d6d1b5b` |

The reproducible probe sources, all request/response traces, sampled memory,
PE imports, certificates, source archives, Cargo metadata/trees, advisory bodies
and attestation verification results remain with those task-state reports.
No generic Python capability, redistribution admission, performance guarantee,
native Windows support or complete alias-reference support is established.
