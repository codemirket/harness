# Raster final gate review — 2026-10-26

> **Historical custom-build route.** The recipient CLI and source-built
> selection discussed below were removed from active code when the pinned
> upstream prebuilt became the closed Raster candidate. See the
> [current prebuilt qualification](m4-raster-prebuilt-qualification.md).

Raster remains **unadmitted**. A separate macOS ARM64 successor now removes
private native paths and has preserved source/build/relink evidence sufficient
to continue closeout without repeating that build. It does not have final delivery terms, an intended
recipient's access receipt, or a normal installed Mirket modification journey.
The recipient owner and its explicit CLI are implemented; the owner's private
synthetic binding is not that journey. Windows remains gated.

The initial review performed read-only source, artifact and license inspection
on macOS 26.5.2 (25F84), ARM64. Its authorized follow-up implements the recipient
CLI in `src/main.rs` and a narrow maintenance inspection API in
`src/release_store.rs`. A later authorized private-path successor completed the
bounded native build and delivery freeze recorded in the
[private-path review](m4-raster-private-path-review.md#executed-successor-and-frozen-delivery).
No installed Raster trial, user installation/client setting change or publication
was performed. The owning records are the
[corrected recipient closure](m4-raster-recipient-closure-20261026.md),
[historical release boundary](m4-raster-installed-release-qualification.md),
[source qualification](video-native-source-closure.md) and
[historical proposed inventory](historical-admission/m4-raster-source-recipient.json.proposed).

## Exact evidence that can be reused

The initial review independently read and hashed every file in the corrected private
candidate: **198 regular files, 123,139,141 bytes**, with no path, size, SHA-256
or normalized `0600` mode mismatch against its frozen inventory. These are
the reduced source-built runtime's bytes, not the original publisher dylib.

| Bound item | Identity or existing evidence | What it establishes |
| --- | --- | --- |
| Current proposal | SHA-256 `e1127db8741e5e349e347a3b8230ad32c36db55c2cba6a63c26eaff5c7551f82` | Current compiled proposal identity; `release_admitted=false` remains authoritative |
| Runtime | 124 files / 16,087,960 bytes; addon `791921f9…`; baseline dylib `1801c6f1…` | Frozen Sharp 0.35.5 / reduced libvips 8.18.7 selection |
| Notice tree | 71 files / 2,151,327 bytes | Preserved selected notice bytes, not blanket rights clearance |
| Source kit | 104,893,453 bytes; SHA-256 `a3e271a5ce2702bb3d195673cec3a43cc0367eb329b44244d900356613a14d24` | Corrected archive identity; 369 regular members / 368 manifest inputs |
| Root instructions | 5,325 bytes; SHA-256 `477b28b05332b6cd83ad07382af69e2f5c9fcc22c6952a64a5afacedbcf70219` | Supplied source-local commands exist; no missing fixture helper is promised |
| Exact candidate archive | 111,438,046 bytes; SHA-256 `d500d16faa6015e24cc1e01fb2db4826478c42752f6151cb01142ec5c438bc43` | Local co-delivery is assembled and independently unpacked |
| Fresh corrected-kit run | 8,693 source entries reconstructed; 42 native stages completed in 349.898 seconds without control edits or resume | Supplied corrected controls have completed a fresh native build on the observed host |
| Source modification/relink | Baseline `02f44e08…`, modified `255ad238…`; 11,331 defined exported symbols equal; 1,259-member baseline link-map multiset preserved | Supplied source patch produces the exact red-channel change, preserves controls and permits baseline reselection in separate Node layouts |
| Recipient owner | `src/raster_recipient.rs`; private native result uses modified `255ad238…` against the fixed runtime | Release-bound copy/import/rollback and corruption refusal have synthetic private tests; ordinary installed setup/MCP was not established |

The full pins and receipts remain under the existing private evidence root
`~/.codex/task-state/harness-rewrite-implementation/`, particularly
`m4-raster-recipient-closure-20261026/recipient-v2/` and
`raster-recipient-owner-20261026.md`. The review rehashed the existing inventory
and source/relink receipts; it did not rerun their native operations.

That table records the preserved predecessor. The separate successor now has
198 files / 123,523,665 bytes and a baseline of 14,608,256 bytes, SHA-256
`c5de3c4af11bfb017fb1e632d2df08e98e96c989d551e5b76f3fa84ddfd56044`.
Its complete source kit is 105,298,010 bytes, SHA-256
`84e5366f7fb118225ec34b6a8d5ac97bbb128b74423996f6979aa6d9fc000d4e`.
The separate proposed inventory is 50,070 bytes, SHA-256
`e2f89a4b11f54f88ec2abe7fd2d8a104284fddb30b886b475d8b9aed88d700ef`;
it retains `release_admitted=false`. It has not replaced the compiled proposal
in this review. Exact transport/manifest identities and native limits are in
the linked private-path review. Its root instructions include the implemented
CLI contract while explicitly retaining the unqualified installed-success gate.

The earlier failed control run, repaired/resumed build and 209-file development
MCP fixture remain historical evidence. They are not substituted for the
corrected 198-file source delivery or normal installed operation. Conversely,
the historical missing-helper section and the proposal's
`development_installed_release_wiring_remaining` array are not a current code
inventory: corrected pins and a `RasterRecipient` owner now exist. Only the
normal installed journey remains unproven among those implementation items.

## Rights and delivery decisions still missing

The original grants govern each component. A package's license label, retained
license text, file hash or successful modified-library load does not establish
that final delivery satisfies those grants. The official pinned
[libvips source header](https://github.com/libvips/libvips/blob/v8.18.7/libvips/resample/reduceh.cpp)
contains a later-version grant; the exact
[libheif source license](https://github.com/strukturag/libheif/blob/v1.23.5/COPYING)
states its library's LGPL terms. The proposed election must remain tied to
the actual linked-source grants, with permissive terms, patent notices and
excluded components kept distinct.

[LGPLv3 §§3–5](https://www.gnu.org/licenses/lgpl-3.0.html) separately address
header material, application/library combination and side-by-side combined
libraries. Section 4 preserves modification/debugging rights and requires a
working relink route or suitable shared-library mechanism. If §5 is used for
the statically combined native library, designate its accompanying uncombined
work and give the required notice. Merely demonstrating Sharp's dynamic swap
does not settle that election. This review does not declare that an uncombined
binary is invariably required, or that the existing source archives already
satisfy §5: the final designation and rationale are absent.

[GPLv3 §6](https://www.gnu.org/licenses/gpl-3.0.html#section6) supplies the source
delivery routes and conditional Installation Information rules referenced by
LGPLv3. For a network delivery, equivalent same-place source access can use
the same downloadable package; a separate source service is not intrinsically
required. Actual product/transaction facts determine Installation Information,
including necessary procedures or authorization details. Private ad-hoc
signing is not an exemption. The candidate's GNU LGPL text exactly matches the
official GNU text fetched during this review; the official pinned libheif
source also supplies the LGPLv3/GPLv3 texts inspected here.

Sharp's wrapper/addon retain their
[pinned Apache-2.0 license](https://github.com/lovell/sharp/blob/51a990faa26ade5586a4934ac9673c98d8893326/LICENSE).
Retain the applicable notices and changed-file statements under those terms;
helper MIT terms do not replace native or wrapper terms. This is a bounded
release-gate assessment, not a patent-clearance conclusion.

## Implemented recipient path and remaining installed journey

`src/main.rs` now implements explicit recipient inspect/import/rollback commands
over the existing `RasterRecipient` APIs. Ordinary inspection uses
`ReleaseStore::open_active()` and its session guard. Import, rollback and
record-only inspection use the exact installation's exclusive maintenance
guard and `ReleaseStore::inspect_active_for_maintenance()`, which preserves the
existing pending uninstall/reinstall refusals. `src/installed_payload.rs`
resolves the selection for installed startup, and doctor conditionally
inspects health. The closed Raster admission still makes this binding
unreachable through normal setup today; the new CLI does not bypass it.

A recipient can manually execute the delivered source-local Python staging
and Node relink check today. That produces new standalone module layouts and
proved changed output. The frozen root instructions still describe that local
journey, so final delivery must add the actual installed commands below after
the release gates are resolved. Overwriting the release dylib would fail its
immutable inventory checks; editing selection JSON or using a development
override is not the installed recipient mechanism.

LGPL does not prescribe a command called `import`. A documented and effective
manual mechanism could be relevant evidence under the selected license route.
The product has chosen immutable releases plus explicit mutable selection, so
the CLI exposes the existing owner. These commands are implemented in source;
they have not established a successful admitted installed Raster journey:

```text
mirket raster-recipient inspect --home /absolute/home --json
mirket raster-recipient inspect --home /absolute/home --record-only --json
mirket raster-recipient import --home /absolute/home \
  --expected-revision REVISION --library /absolute/rebuilt.dylib \
  --bytes EXACT_BYTES --sha256 EXACT_SHA256 --json
mirket raster-recipient rollback --home /absolute/home \
  --expected-revision REVISION --json
```

- Commands derive and verify the active installed release through the existing owner.
  Require admitted Raster+Design, its fixed runtime and exact installation
  path. Accept no supplied release root, manifest admission override, ambient
  module lookup or direct payload replacement.
- Ordinary inspection holds the installation session guard and creates no
  recipient state. Return release ID, revision, generation, selected variant,
  exact library identity and selected module/library paths, using the existing
  `RasterSelection` result.
- Import and rollback require the exclusive maintenance guard and matching
  revision. Preserve existing busy, unsafe-input, bounds, stale-state,
  corruption and immutable-baseline refusals. Import changes selection for
  subsequent sessions and does not execute or certify the selected native code.
- `--record-only` obtains maintenance, reads the valid bound record even if
  selected bytes are corrupt, and returns `RasterSelectionRecord` without
  launch paths. Its JSON has `kind="record_only"` and
  `runtime_health="not_checked"`; healthy inspection/import/rollback instead
  have `kind="verified_selection"`, an explicit operation and a selection.
  This permits inspection after a lost response and an explicit
  revision-bound rollback; it must not repair an invalid record silently.
- Keep the installed session guard through all raster child lifetimes. Update
  delivered instructions to the actual compiled spelling and demonstrate
  baseline, modified output, rejected corruption and byte-exact rollback in
  the normal installed MCP path. The existing four-object/pending-stage limits
  may remain finite refusals; the instructions must explain any exposed limit
  instead of prescribing manual state deletion.

The CLI follow-up passed these checks on the observed macOS ARM64 host:

```sh
python3 scripts/check-build-inputs.py
cargo test --locked --offline --bin mirket raster_recipient_cli_tests -- --test-threads=1
cargo test --locked --offline --lib release_store::tests::maintenance_active_inspection_preserves_lifecycle_refusals_and_guard_binding -- --exact --test-threads=1
cargo clippy --locked --offline --all-targets -- -D warnings
```

All **five CLI tests** and the **one focused lifecycle test** passed. They
exercise required paths/identities, rejected overrides and invalid scalars,
distinct JSON variants, missing-home refusal without creation, core-only
release refusal, busy maintenance, exact maintenance-root binding and all
four pending uninstall/reinstall marker names. Integration fixtures use
explicit disposable private homes and leave the active pointer unchanged.
The complete Rust suite remains the initiating owner's integration gate.
Neither the parser/refusal tests nor strict Clippy qualifies an admitted
installed Raster setup/import/rollback or actual client execution.

## Artifact privacy and platform evidence

Read-only `otool` inspection found `minos 11.0` in both candidate Mach-O files.
The selected Node receipt declares `minos 13.5`; the approved implementation
plan's **product qualification floor is macOS 15**, not 11. These are different
facts. Execution on macOS 26.5.2 does not qualify macOS 15, and no source build
on macOS 15 was observed. Source-kit tool prerequisites remain independently
obtained build prerequisites, separate from the installed runtime floor.

The predecessor baseline dylib has six mapped `/Users/…` runtime constants
and 48 STABS debug entries, **54 raw occurrences** in total. Its constants
include real optional charset/GIO/DBus lookups as well as diagnostic or
classification use; details are in the private-path review. The unchanged
addon contains none. The predecessor source archive also contains private-prefix strings in these top-level
metadata files:

| Archive-relative file | `/Users/` occurrences |
| --- | ---: |
| `sharp-raster-source-kit/artifact-identities.json` | 4 |
| `sharp-raster-source-kit/build-tool-acquisition.json` | 1 |
| `sharp-raster-source-kit/linked-object-inventory.json` | 955 |
| `sharp-raster-source-kit/source-changes.json` | 3 |
| `sharp-raster-source-kit/tool-identities.json` | 1 |

This bounded scan covered regular non-archive members and the two selected
native binaries. It did not recursively privacy-scan pristine nested archives.
The predecessor `prepare-raster.py` wrote absolute patch paths. The successor
fixes its writer, preserves provenance using relative labels, supplies neutral
runtime defaults and strips only debug entries before signing. Both successor
libraries have zero private user-path bytes and no STABS entries, retain all
11,331 exported names and passed the exact source-local RGB/RGBA modification
and baseline restoration checks. The original old-root-denied relocation check
passed; a later [source/build/prefix-denied check](m4-raster-private-path-review.md#sourcebuildprefix-denied-relocation-check)
also passed for the separately staged baseline, modified and rollback packages.
Neither check exercised all optional resource paths or a normal installed release.
The final delivery's 198 files and source archive's 369 manifest inputs were
statically verified. All 17 nested archives remain exact predecessor bytes
and were not recursively privacy-scanned. A generic `/Users/` rejection-detector
literal in the staging script is not private developer provenance.

Source authentication remains distinct from digest correspondence. The source
closure records decoded npm provenance subject checks but no certificate,
signature or transparency verification. Final dependency/advisory review,
release authentication and selected signing/notarization behavior are not
closed by this report.

Windows x64 and ARM64 have no Raster candidate or native recipient test in this
evidence. The compiled layout rejects both, and the recipient owner is limited
to macOS ARM64. Upstream platform availability cannot replace target-specific
payload/source/rights, process ownership, installation, client and output
qualification. The [Windows gate](windows-qualification-status.md) stays closed.

## Minimal closeout checklist

| Order / owner | Exact missing action or evidence | Can proceed before publication without repeating this source build? |
| --- | --- | --- |
| 1. Delivery/rights owner | Record the actual conveyance election against the linked-source grants; identify the §5 uncombined work/notice if that route is used; review all effective product/distribution terms and any runtime copyright display; state product/transaction facts and supply applicable Installation Information | **Yes**, for the documents/decision. Do not assert that the missing designation is already satisfied. A newly required native artifact would need its own qualification |
| 2. Artifact owner | Reuse the successful private-path successor build and frozen delivery; review exact successor pins and the stronger relocation receipt, complete current dependency review and final authentication | **Yes**. Native path removal, build/relink proof and source/build/prefix-denied source-local relocation are complete. Final delivered/installed relocation, nested-archive privacy and broader optional subsystem behavior remain unproved |
| 3. Implementation/delivery owner | The CLI above and successor root instructions now provide explicit inputs, tagged results and guards/refusals. Retain focused checks and record actual installed results when qualified | **Yes**. No native source rebuild or new dependency is needed; documented commands alone do not establish admission or successful installed execution |
| 4. Delivery owner + intended recipient | Select the actual same-release access route. Receive the final runtime/source/notices through it, verify every frozen identity and retain the access/copy receipt and source-access terms | **Yes**, including a real pre-publication recipient transfer when authorized. Another copy between developer directories is insufficient. No recipient destination or receipt is established here |
| 5. Admission/integration owner | Bind reviewed successor pins while admission stays closed; only after release gates are met, deliberately admit and normally install a disposable final release; run baseline raster MCP, explicit import of the already-built successor `7deee832…` compatible modification, changed saved output, corruption-before-write refusal, record reconciliation, rollback and exact baseline recovery; inspect loader paths, cleanup and unrelated payload integrity | **Yes**, reuse the 14,608,256-byte modified library with exact SHA-256 `7deee8320edb38fa4f8b5d2aead5ee8c80551ee379381d46ab73508efba2d765`. This is new installed-operation evidence. No admission bypass or synthetic binding counts |
| 6. macOS/client owner | Exercise the final packaged release and recipient mechanism on the proposed macOS 15 floor and actual Codex Desktop; qualify chosen authentication/signing/notarization and modified-library execution behavior | Requires target/client access, not inherently a rebuild. A failure may require a new candidate; the observed macOS 26 run cannot close it |
| 7. Windows owner | Keep both Windows admissions closed until their separate native implementation and qualification gates pass | **No macOS evidence can close Windows**; do not broaden the macOS admission |

The existing corrected native build need only be repeated if native inputs,
patches, meaningful build controls or the selected binary change, or if new
evidence exposes a defect in that proof. A changed archive identity alone
requires renewed inventory/extraction/source-correspondence checks, not an
automatic full native rebuild. Preserve the prior exact archive and receipts
when making any successor candidate. No step above authorizes publication.
