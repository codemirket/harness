# Raster private-path review — 2026-10-26

> **Historical custom-build route.** The active closed candidate now selects
> [upstream prebuilt archives](m4-raster-prebuilt-qualification.md).

The original frozen candidate exposes developer paths in executable constants
and debug/source provenance. A separate successor now eliminates those paths
through explicit source defaults and debug stripping, and passed a bounded
source build/relink check. Its runtime/source/notices delivery is frozen with
admission closed. Optional lookup, installed-recipient and release gates remain
separate; the successor evidence and exact identities appear below.

The original classification below was read-only inspection on macOS 26.5.2
ARM64. Subsequent preparation and native execution affected only the separate
private successor; the original and compiled proposal were preserved.
The original dylib remains **14,631,104 bytes**, SHA-256
`1801c6f1cb12363a8d1815d224840001031ed96f6f986e9e91e03dce35ee453a`.
The source archive remains **104,893,453 bytes**, SHA-256
`a3e271a5ce2702bb3d195673cec3a43cc0367eb329b44244d900356613a14d24`.
See the [final gate review](m4-raster-final-gate-review.md) for other blockers.

## Exact binary classification

Here `P` abbreviates the original qualification's developer-home
`raster-qualification/prefix`, retained in the existing private evidence under
`~/.codex/task-state/harness-rewrite-implementation/`. The following offsets
are file offsets and virtual addresses in this exact thin ARM64 Mach-O's
`__TEXT,__cstring` section. Static ARM64 ADRP/ADD references connect every
string to the listed functions; source/configuration inspection establishes
its use. References alone do not prove a branch ran.

| Offset | Constant | Actual use and relocation implication |
| --- | --- | --- |
| `0xaf4509` | `P` | `vips_init`, `vips_guess_prefix`, `vips_guess_libdir`: fallback prefix, `VIPSHOME` value and optional info diagnostics. Sharp calls `vips_init("sharp")`. Prefix guessing checks `P/lib/vips-modules-8.18` even though libvips module loading is compiled out |
| `0xaf4599` | `P/lib` | libvips fallback/diagnostics **and** GLib `_g_locale_get_charset_aliases`: constructs and opens `P/lib/charset.alias`. Missing/unreadable aliases are treated as empty; this is a real optional file lookup, not only provenance |
| `0xaf8541` | `P/share/locale` | GLib `ensure_gettext_initialized` passes it to `bindtextdomain`. The original proxy-libintl compile command has `-DSTUB_ONLY`; its bind function returns `/dummy` and does no catalog lookup. The constant remains disclosable |
| `0xb07142` | `P/lib/gio/modules` | GIO's live module-directory fallback. On macOS it first tries `gio/modules` beside the loaded dylib, then falls back to this constant; `_g_io_modules_ensure_loaded` scans the result. Original GLib uses `G_MODULE_IMPL_DL`. Disabling **libvips** modules does not disable GIO modules |
| `0xb0abc4` | `P/var` | `g_unix_is_mount_path_system_internal` compares this string against a supplied mount path. This use is classification, not a file read |
| `0xb121cb` | `P/var/lib/dbus/machine-id` | `_g_dbus_get_machine_id` conditionally reads it, then `/etc/machine-id`. Errors include the filenames. The reduced raster journey has not been shown to invoke this optional DBus path |

The earlier six-string count was the mapped-string view. Parsing the entire
file and `LC_SYMTAB` finds **54** raw `/Users/` occurrences: these six constants
plus **48 STABS debug entries** in the symbol string table, comprising 24
`N_SO` (`0x64`) source-directory entries and 24 `N_OSO` (`0x66`) libexif archive
object entries. Apple SDK `mach-o/stab.h` and `nlist.h` define these as debug
provenance; they are not runtime filename arguments. `strip -S` is documented
by the installed Apple toolchain to remove debugging symbol-table entries.
It would not remove the six executable constants.

`otool -L` contains the local `@rpath/libvips-cpp.8.18.7.dylib` identity and
Apple system libraries/frameworks, with no developer prefix. Thus these paths
do not make dyld require the old build directory. The exact candidate has
already loaded from separate private receiver layouts. That does not establish
absence of optional filesystem reads or their behavior under every locale/API.

Anyone receiving the bytes can recover the private username and workspace
structure. Info/debug logging or DBus errors can additionally expose paths
under their conditions. The Mirket design child clears its environment and
sets only private job HOME/TMP variables, so verbose environment variables and
GIO overrides are not inherited by that normal path. This is not proof that
every optional lookup is suppressed, or that all private paths remain absent
from ordinary output. No syscall or loader probe ran during this original
classification.

## Source-kit metadata

All 369 regular archive members were enumerated. Inspecting every non-nested
member, including binary members, found **964** `/Users/` occurrences, confined
to these five JSON files. Seventeen nested archives (15 native inputs, Sharp
and its native recipe) were not recursively privacy-scanned.

| Member under `sharp-raster-source-kit/` | Occurrences / field | Meaning |
| --- | --- | --- |
| `artifact-identities.json` | 4 / `[].path` | Historical baseline/modified qualification artifact locations |
| `build-tool-acquisition.json` | 1 / `ninja.origin` | Historical private Ninja acquisition/copy location |
| `linked-object-inventory.json` | 955 / `objects[]` | Link provenance: static archive paths with member names, not runtime libraries to locate |
| `source-changes.json` | 3 / `patches[].path` | Qualification patch locations; preparation currently regenerates absolute paths |
| `tool-identities.json` | 1 / `tools[].path` | Historical Ninja tool location; the executable is supplied as `tools/ninja` |

These are provenance, not executable path configuration. The supplied build,
preparation, staging and relink scripts do not consume those metadata paths
to select a tool/library. The preparation script does write the absolute
`patches[].path`, so editing only the archived JSON would leave its generator
unfixed. Preserve source/archive/member identities and hashes when replacing
private prefixes with relative labels; retain the original full provenance
privately. Scanning all uncompressed candidate files found no other `/Users/`
occurrences outside the dylib.

## Smallest concrete successor

The corrected `build-raster.py` (`ce617db9…`) defines
`PREFIX = ROOT / 'prefix'`; its Meson helper supplies
`--prefix=<that absolute directory> --libdir=lib --buildtype=release` and static
libraries except shared libvips. The original generated configs and compile
commands confirm these inputs produced `P`. `-Dnls=disabled` did **not** remove
the original GLib/libvips `ENABLE_NLS` definitions; the compiled proxy is what
makes the translation binding inert. `-Dmodules=disabled` applies to libvips.

Use **`N = /__mirket_raster_no_resources__`**, a deliberate neutral, absent
root namespace, for runtime defaults only; no installer should create it.
Keep physical build/install `PREFIX`, pkg-config search paths and archive
locations under `ROOT` unchanged. Do not substitute an ambient package prefix
such as `/usr/local`. Supply small dated source/preparation patches at these
exact producers, then rebuild the affected GLib and libvips code:

| Delivered source producer | Successor runtime value |
| --- | --- |
| `source/vips/meson.build:649–650`, `VIPS_PREFIX` / `VIPS_LIBDIR` | `N` / `N/lib` |
| `source/glib/meson.build:2322,2324`, `GLIB_LOCALE_DIR` / `GLIB_LOCALSTATEDIR` | `N/share/locale` / `N/var` |
| `source/glib/glib/libcharset/meson.build:11`, `GLIB_CHARSETALIAS_DIR` | `N/lib` |
| `source/glib/gio/meson.build:4–5`, `GIO_MODULE_DIR` / `LOCALSTATEDIR` | `N/lib/gio/modules` / `N/var` |

This retains the physical source/build/runtime layout and public API, but
**changes optional filesystem defaults**, not merely debug metadata. Expected
codec behavior and symbol compatibility must be verified, not assumed. GIO's
loader-relative mechanism remains; no external module directory is part of
the selected runtime. Future external resources require an explicit verified
runtime design rather than populating `N`.

Add `/usr/bin/strip -S` to the supplied staging helper after local load-ID
staging and before its existing ad-hoc `codesign`. Avoid broader symbol
stripping. This targets the 48 debug entries without intending to change the
11,331 defined exported interfaces. Recheck those interfaces and signing;
do not patch frozen binary bytes or silently reuse its hash.

Normalize the five provenance files using kit-relative patch/tool/archive
labels and explicit historical-artifact labels; fix the preparation writer as
well. No new dependency, source acquisition or admission override is needed.
Freeze a new source kit and dylib, preserving this candidate and all receipts.
The changed native controls require a new supplied-control build/relink proof;
metadata-only changes would not, by themselves, require native compilation.

Before considering that successor qualified: check all raw binary/metadata
bytes for the developer prefix, debug entries, load commands, exports and
signatures; verify source reconstruction and exact supplied commands; exercise
relocation with the old build prefix inaccessible, baseline/modified raster
outputs and rollback; and test that absent charset/module resources remain
optional. The separate execution below establishes only the stated successor
checks. Recipient rights, normal installed operation, macOS floor, client and
Windows gates remain separate.

## Executed successor and frozen delivery

The private successor lives under
`~/.codex/task-state/harness-rewrite-implementation/m4-raster-private-path-successor-20261026/`.
Its original input archive and original dylib still match the two hashes above.
No release admission, installed release, user installation or client configuration was
changed. This is a separate candidate for review and integration.

Preparation reproduced 8,693 source entries. Exactly six entries differ from
the predecessor: the four Meson producers above and the GLib/libvips dated
modification notices. The physical build recipe remains 13,213 bytes, SHA-256
`ce617db97f2962aaa8782ede9632a717f63ad7d82c9c93a0abaa321038a7d1ee`.
The exact executed preparation and staging controls are respectively:

| Control | Bytes | SHA-256 |
| --- | ---: | --- |
| `prepare-raster.py` | 11,216 | `960510331e2df327ef364d4530ee876dd6b04aea3e2dfe78d23a16132317a50a` |
| `install-private-sharp.py` | 4,871 | `f100b5ddbfd4c8015e20a52186119dd1f480291f724397561a45d3458dbb31ca` |
| `expected-source-patched-manifest.json` | 1,373,667 | `304cefb42667b7159be11594c441ed3319a786380edd155069762e386eb7bd4d` |

All 368 then-supplied inputs were reverified before execution. The fresh
baseline completed all **42 configure/compile/install stages in 335.549 s**.
Staging, applying the supplied resize patch, the two-stage modified relink,
second staging and source-local output check all completed within the same
**344.601 s / 360 s** phase. Its guard retained a 25 GiB floor and 1 GiB private
regular-byte cap. Minimum sampled free space was **48,691,646,464 bytes**;
peak sampled owned process-tree RSS was **745,701,376 bytes**, with memory
pressure level 1 throughout. RSS is sampled, not an instantaneous peak.
Native children exited and the native lane was explicitly returned.

| Staged library | Bytes | SHA-256 |
| --- | ---: | --- |
| Delivered baseline | 14,608,256 | `c5de3c4af11bfb017fb1e632d2df08e98e96c989d551e5b76f3fa84ddfd56044` |
| Private modified-library CLI input | 14,608,256 | `7deee8320edb38fa4f8b5d2aead5ee8c80551ee379381d46ab73508efba2d765` |

Both have **zero raw `/Users/` occurrences, zero current kit-path occurrences
and zero STABS entries**. All six neutral runtime constants are present.
Both retain the predecessor's exact **11,331 defined exported symbol names**,
local rpath identity and Apple system dependencies; strict signature checks
passed. The remaining modified link map has the same 1,259 archive-member
multiset. Its baseline map was overwritten by the modified relink, so no
separate baseline-map comparison is claimed.

Separate Node processes loaded the selected staged addon/dylib. The supplied
check observed the exact RGB red decrease of 16, unchanged green/blue, an
unchanged RGBA control, and baseline reselection restoring exact encoded/raw
bytes. All those fixture hashes equal the predecessor's source-local results.
The initial sandbox denied reads from the **old predecessor build root**.
A later, separate check also denied the successor's source/build/prefix
directories, as recorded below. Neither check covers every codec, locale,
GIO/DBus branch or external resource.

`bounded-native-phase-result.json` is the full private receipt (115,737 bytes,
SHA-256 `ccc49c94080b8f095069ba5292f2eedb9237df67a75c1775c2c719a17daadac1`).
`postbuild-static-review.json` and `modified-linkmap-static-comparison.json`
record the subsequent static checks. No further native process ran while
assembling the final delivery.

The frozen `successor-delivery/` contains a separate 198-file candidate of
**123,523,665 bytes**. All regular files are mode `0600`, with no symlinks or
extra paths. Among 124 runtime files only the dylib changes; all 71 notice
files remain byte-identical. Root instructions now describe neutral defaults,
debug stripping, the actual implemented recipient CLI contract and its closed
installed-success gate. Historical metadata is labeled and path-normalized;
the preparation writer emits relative patch paths.

| Frozen delivery artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `source/sharp-raster-darwin-arm64-source-kit-private-path-successor-20261026.tar.gz` | 105,298,010 | `84e5366f7fb118225ec34b6a8d5ac97bbb128b74423996f6979aa6d9fc000d4e` |
| `candidate-198-inventory.json` | 41,509 | `8d74f8fbc2af95eb0c35913fb13654c7c0c1a7eda54e608a77658f5e2e18a75f` |
| `private-path-successor-raster-candidate-198-files.tar.gz` | 112,392,539 | `92c6651e3c937354ddb3c141a044a681ef38d8d6bce99eb9c29fa6f0ada64fd4` |
| `m4-raster-private-path-successor.json.proposed` | 50,070 | `e2f89a4b11f54f88ec2abe7fd2d8a104284fddb30b886b475d8b9aed88d700ef` |

The final source archive has **370 regular members**, including 369 exact
manifest inputs. Its manifest is 61,888 bytes, SHA-256
`49c71d465a3f09f55c3bd57bf7cadee4aa14a4eeae198006b95126b028ce135e`.
The added `successor-native-qualification.json` carries sanitized exact native
evidence; source controls did not change after execution. Only documentation,
evidence and manifest bytes changed for the delivery freeze. Static verification
decompressed both transports, checked every member's identity/mode and neutral
headers, checked supplied command inputs and parsed Python syntax. It did not
repeat a build from the final repacked archive.

Delivered nonnested candidate/source-kit files contain **no private user
paths**. One generic `/Users/` literal remains intentionally in the staging
helper's rejection detector. The **17 nested archives** are exact predecessor
bytes and were not recursively privacy-scanned; this is not a whole-recursive-
archive privacy claim. `successor-delivery/static-delivery-verification.json`
records these limits. The final private allocation was 857,926,677 regular
bytes and verification left more than 44 GiB free.

The modified input remains only at
`sharp-raster-source-kit/rebuilt-modified/node_modules/@img/sharp-libvips-darwin-arm64/lib/libvips-cpp.8.18.7.dylib`
inside this private successor root. Its exact byte count/hash above are ready
for a later revision-bound `raster-recipient import`; the current revision
must come from that installation's inspection. It is not included prebuilt
in the source archive. Admission remains false and no installed import ran.

## Source/build/prefix-denied relocation check

After the first native phase released its lane, the source-local baseline,
modified and rollback packages passed `recipient-relink-check.py` under
`/usr/bin/sandbox-exec`. The fixed profile denied network access and reads
from the predecessor build root plus the successor's `source/`, `build/` and
`prefix/` directories. Writes were limited to the successor source-kit tree.
The profile has SHA-256
`efc17c870dbe2b42e19e2eb2ddaecf2ab899d581232df071f03c9cc8f24b06d9`;
the new `relink-check-no-build-tree/result.json` is SHA-256
`03ae7cc494f89248ab14512c0b2f4f1224b11f529d039195776ecb3f2d39989f`.
The check exited successfully, loaded only the selected staged Sharp addon and
dylib paths, reproduced the exact red-channel decrease of 16, left green/blue
and the RGBA control unchanged, and restored byte-identical baseline outputs
on reselection. This demonstrates the exercised raster operations do not need
those source/build/prefix trees. The final delivered archive and normal
installed recipient path were not executed by this check; optional resource
paths outside the exercised operations remain unqualified.

Integrating exact successor pins and exercising normal installed CLI/MCP
import, corruption refusal and rollback remain owner work. Actual
intended-recipient same-release access,
final conveyance election/uncombined-work designation, effective modification
and debugging terms, applicable Installation Information, artifact/dependency
review, native client, signing/notarization and minimum-product-OS execution
remain release gates. The product floor is **macOS 15**; the dylib's `minos 11.0`
does not qualify that floor. Windows remains gated. None of the hashes or local
checks settles legal recipient facts or authorizes publication.
