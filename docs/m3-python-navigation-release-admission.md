# Python navigation: exact macOS release materials

Observed **2026-10-25** on macOS **26.5.2 (25F84), ARM64**. The exact
CPython/Jedi/parso source, dependency and notice materials are complete for the
selected macOS component. A clean layout with installed permissions passed the
real backend and public development MCP fixtures. Subsequent release-store
staging, activation and installed MCP payload resolution passed in disposable
homes, as recorded below.
This does not qualify Windows, macOS 15, Codex GUI activation or complete Python
semantics. The [runtime integration contract](m3-python-runtime-integration.md)
continues to bound the selected-parser-name capability.

## Exact selection

[m3-python-navigation-macos.json](../packaging/admission/m3-python-navigation-macos.json)
defines every added file, its bytes, SHA-256 and normalized installed mode. Its
embedded notice text is encoded as UTF-8 without newline conversion. The
[source manifest](../packaging/admission/m3-python-navigation-macos.sources.json)
defines eleven unchanged accompanying source archives and their original
license/notice locations. Neither document authorizes publication.

| Entry | Bytes | SHA-256 |
| --- | ---: | --- |
| `adapters/python_navigation.py` | 20,803 | `384c0c55d27ece7613c20b1f2f87f648163b51636632dd2aa9fb0c46c44f087f` |
| `runtime/python-navigation/jedi-0.20.1-py2.py3-none-any.whl` | 4,885,980 | `0fb16d86c4a4c73c37ba518c77419975e30fcc620658a8d14fbb5720cdd34142` |
| `runtime/python-navigation/parso-0.8.7-py2.py3-none-any.whl` | 107,025 | `a8926eb2a1b915486941fdbd31e86a4baf88fe8c210f25f2f35ecec5b574ca1c` |

The component adds **30 regular files / 94,903,095 bytes**: the adapter, two
unchanged wheels, **16 notice files / 74,961 bytes**, and **11 source archives /
89,814,326 bytes**. These are file-content totals, not compressed release size,
allocated disk space, memory consumption or staging reservations. All added
regular files install as `0600`; directories install as `0700`. No new symlink,
executable or Python runtime is introduced by this component.

Reuse the exact
[general-Python selection](../packaging/admission/m3-python-macos.selection.json)
and [native notice mapping](../packaging/admission/m3-python-macos.notices.json).
It contributes 545 original regular files plus thirteen supplemental notices,
**46,698,348 bytes**, and two fixed relative aliases. The executable is
`runtime/python-general/python/bin/python3.13`, SHA-256
`71d176d8e02702c2ad568148caf36cfae92cd862f6a72c94c03643a61c04bd54`.
The combined Python material content is **141,601,443 bytes**, excluding other
release components and filesystem overhead.

Independent inspection verified every one of the original archive's **1,657**
members against the selected/omitted inventory, including hashes, modes, types
and alias targets. All retained private-runtime bytes matched. The reused
development directory additionally contained four undeclared `.pyc` files under
`__pycache__`; those files were preserved as failed-candidate evidence and were
not copied into the new specimen. Assemble the runtime from the admitted archive
or an exact selected inventory; an unrestricted copy of that directory fails
the release contract.

## Dependency and source closure

Jedi's wheel metadata requires only **parso >=0.8.7,<0.9.0** without extras;
parso has no base dependency. No development, documentation, QA or testing extra
is selected. Both wheels contain regular source/data files only, with no native
binary, shared library, symlink or executable helper. They contain 5,626 and 37
files, respectively, with 14,939,387 aggregate logical bytes.

The audit independently checked every wheel `RECORD` entry and exact membership:
5,625 Jedi and 36 parso records have content hashes; each wheel's own `RECORD`
has the prescribed empty self-hash fields. All **5,620 Jedi package members**
and **31 parso package members** are byte-identical to their unchanged source
distributions. The source distributions are:

| Source | Bytes | SHA-256 |
| --- | ---: | --- |
| `jedi-0.20.1.tar.gz` | 3,121,738 | `2f71208c3f9c1bca057c0e90d3f272aba44ace88fc4067d7587e9e069331b7e5` |
| `parso-0.8.7.tar.gz` | 401,824 | `eaaac4c9fdd5e9e8852dc778d2d7405897ec510f2a298071453e5e3a07914bb1` |

The source tags resolve to Jedi
`ab89beb06cc4ad51ec3b99d7f2023401a8d11815` and parso
`bfe30584415cde2e5a686512c2b109e1d06da3b6`. Both observed tags are unsigned;
hashes and source equality do not authenticate a signed publisher. Jedi's
vendored gitlinks identify typeshed
`4bb9d8351d0795fbc7dee3da069013a48641689d` and django-stubs
`fd057010f6cbf176f57d1099e82be46d39b99cb9`. The actual complete copies retained
in the wheel were compared to the published source distribution, rather than
assuming a repository snapshot contains its submodules.

The accompanying general-Python sources are the exact CPython 3.13.17,
bzip2 1.0.8, libffi 3.8.0, XZ/liblzma 5.8.4, Expat 2.9.0, mpdecimal 4.0.0,
OpenSSL 3.5.9 and SQLite 3.53.1 archives from the frozen build-recipe inputs,
plus the full python-build-standalone recipe/patch snapshot at
`0c5e581b54a4736cdbbe68bdd6d7e571a1dca11e`. All eight preexisting native-source
archive pins matched independently. The recipe snapshot is 461,690 bytes,
SHA-256 `11307e854a65298fbb986f5d758e1c60fc1f5da77f187b5329c4dd45a3b870f2`.
The manifests preserve source URLs and per-archive identities.

These full source archives retain their original per-file terms, including
build, test and other-platform material that is not selected runtime code.
They are accompanying inert source files; Mirket does not invoke their helpers.
The native sources were not rebuilt, and no reproducible native build or minimal
linked-object SBOM is claimed. The retained runtime's build metadata, dependency
pins and exact native inventory remain the provenance boundary.

## Original rights and notices

[Jedi's exact license](https://github.com/davidhalter/jedi/blob/ab89beb06cc4ad51ec3b99d7f2023401a8d11815/LICENSE.txt)
is MIT, with its original AUTHORS file retained. Its vendored typeshed copy
contains Apache-2.0 and MIT terms; the django-stubs copy has its original MIT
grant. [Parso's exact license](https://github.com/davidhalter/parso/blob/bfe30584415cde2e5a686512c2b109e1d06da3b6/LICENSE.txt)
also preserves PSF terms for its stdlib-derived parser/tokenizer files alongside
MIT contributions. All six wheel license/AUTHORS files remain inside the
unchanged wheels and are copied separately without rewriting their text.

The existing runtime admission retains CPython's historical license bundle,
incorporated-software acknowledgements, macholib terms, and the selected native
component notices. The archive's CPython LICENSE is byte-identical to the full
[CPython source LICENSE](https://github.com/python/cpython/blob/64e080d38154391ff2d2caa1161c298bb7cd8f30/LICENSE).
Source inspection found that python-build-standalone's bzip2 notice uses an older
copyright year/version label and its libffi notice differs from the exact newer
source. This component preserves those existing notices and adds complete,
source-authentic LICENSE/COPYRIGHT text for every bundled native-source archive.
The bzip2 source attribution is 1996–2019; libffi's is 1996–2026. No existing
runtime notice was silently replaced or removed.

The complete recipe/patch snapshot preserves python-build-standalone's MPL-2.0
source terms. Its original MPL license is copied separately, and
`notices/python-navigation/REPACKAGING.txt` identifies the accompanying source
paths and briefly describes Mirket's changed selection/permissions, omitted
facilities, added notices and unchanged interpreter/stdlib bytes. This also
supplies the PSF change-summary boundary. Original upstream patch sources remain
available in the accompanying recipe archive. No upstream source is relabeled
as Mirket-authored MIT code.

The concrete conclusion is that the identified source/notice obligations have
an exact accompanying-material arrangement. It is not universal legal or
advisory certification. Preserve the actual files in the release; a receipt,
URL or path to private evidence alone does not perform that inclusion.

## Assembly and runtime verification

`scripts/assemble-python-navigation-macos.py` assembles only the new thirty-file
component tree. Prepare a stable private artifact directory containing the two
wheels and eleven source archives with their exact manifest artifact names.
Then run:

```sh
python3 -B scripts/assemble-python-navigation-macos.py \
  --artifacts-dir /absolute/private/python-navigation-artifacts \
  --output /absolute/private/new-component-tree
python3 -B tests/test_python_navigation_assembly.py
```

The output must be absent beneath an existing stable private parent. The
assembler validates the source-manifest identity, resolves every selected file,
rejects unsafe/colliding paths, links and special inputs, checks exact bytes
before copying and again before publication, writes original notice text, and
normalizes private modes. Darwin's exclusive rename preserves even a destination
created during publication. It never reads or copies the general Python runtime,
unpacks archives, runs upstream code, fetches packages or changes host settings.
These path-based checks require a stable private parent; they do not protect
against hostile concurrent same-user mutation or establish crash durability.

Seven focused tests passed: exact output/modes and exclusion of an ambient
runtime, corrupt wheel/adapter refusal, notice/source-inventory drift, traversal
and path collisions, symlink/FIFO refusal, input mutation after copy, and
preservation of existing/racing destinations. Assembly of the real component
then independently matched all thirty output hashes, lengths, modes and member
names, with no `runtime/python-general` subtree.

A separate clean runtime was assembled through the existing reviewed
general-Python assembler, then combined with the exact navigation files in a
path containing spaces and Unicode. All regular files/directories used their
installed permissions. Fourteen real-backend checks passed, followed by the
public stdio MCP fixture: definitions, references, stubs, original Unicode/BOM/
CRLF spans, passive project state, explicit syntax/limit results, source drift,
mixed-backend refusal, cancellation cleanup and absent-candidate refusal.

The isolated interpreter identity probe returned CPython 3.13.17 ARM64 with the
exact private `sys.executable`, prefix, module origins and search paths despite
poisoned `PYTHONHOME`/`PYTHONPATH` values. The runtime uses `-I -S -B`; the Rust
owner clears the environment and supplies only owned temporary/home paths plus
a non-searchable PATH. The three retained Mach-O files were inspected again:
their load commands name the documented macOS system libraries/frameworks and
no Tcl/Tk library. macOS 11.0 deployment metadata is not macOS 15 execution
evidence. No host Python/analyzer fallback or package installation is admitted.

Private evidence is retained under
`~/.codex/task-state/harness-rewrite-implementation/python-navigation-release-admission-20261025/`:
`package-audit.json`, `wheel-members.json`, `runtime-members.json`,
`source-closure.json`, `native-inspection.json`, `interpreter-identity.json`,
`checks.json` and `verification.json`. The old private directory's extra bytecode
identities are recorded in `package-audit.json`. All source acquisition was data
retrieval; no upstream installer, hook, helper or source build ran.

## Installed-release acceptance contract

The current Rust distribution layout admits exactly the thirty new files only
alongside the macOS Documents component. A five-component manifest contained
6,882 exact files and 972,234,806 payload bytes; missing or changed navigation
files failed validation. The disposable source release with ID
`sha256:5d8c2d9d1697a8b353926610bf864d5991b2aff046fa81426cfded23489318ba`
staged and activated through both the development release store and the normal
`mirket setup` launcher. The source assembly initially exposed four stray
CPython bytecode files from the old task-owned runtime copy; manifest creation
refused them. Only the new disposable candidate was cleaned, and the exact
manifest then validated. Full `cargo test --locked --offline`, strict all-target
Clippy and Rust formatting checks passed after this integration.

`tests/python-navigation-installed-smoke.py` passed against the normal launcher
in the disposable installed home on macOS ARM64. It confirmed healthy `doctor`
before and after, real stdio MCP navigation through the installed private
Python/Jedi/parso payload, the original Unicode byte span, saved-source drift
refusal and unchanged project files. No host interpreter or developer override
was selected. This is an installed protocol test, not a Codex GUI activation or
a complete Python semantics claim.

After the native Codex client check stopped, that same home was renamed to a
task-owned path containing spaces and `İ`. The installed smoke passed again
without rebuilding or remanifesting; `doctor` remained healthy and all navigation
paths resolved beneath the relocated home. The [Codex-native app-server
qualification](m3-python-codex-client-qualification.md) also passed for the
pre-relocation home through its disposable client state. It did not run a model
turn or change the user's live Codex configuration.

The release owner must include the unchanged general runtime inventory plus all
thirty component files, preserve the two exact interpreter aliases, reject
unknown/missing entries, and verify installed permissions. Resolve all four
navigation payload paths beneath the verified selected release. Enable the
payload only when this complete component is present; never infer readiness from
a filename, development override or a host interpreter. Recheck the release
before jobs and after the owned child lifecycle as required by the owning
release/currentness contract.

Further installed acceptance should run the full definitions/references/stubs fixture through installed
`mirket serve`. Verify cutoff behavior, cancellation/reaping and immutable
installed bytes after every job. An altered wheel, adapter, source archive, license,
runtime stdlib/native file, permission or escaping alias must fail verification
without changing the active release or selecting a fallback. A component-absent
release must refuse Python navigation explicitly. Release activation/rebinding
must invalidate stale release ownership before effects.

Native Windows x64/ARM64 runtime/rights/lifecycle qualification, actual macOS 15
execution, Codex GUI activation, broader capability completeness and whole-release
resource/space reservations remain open. This audit modified only its new
admission documents, component assembler/tests and private evidence; it did not
install into the user's `.mirket`, alter the old harness/client settings, commit,
push or publish.
