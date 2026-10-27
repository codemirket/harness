# Finite distribution capacity

> **Historical qualification record.** The `documents` component and general
> Python/PDF payload references on this page describe an earlier distribution
> experiment. They are not current release components. The approved rewrite
> keeps document, Office, extraction and OCR work in Codex Desktop and host
> tools. See [the current implementation plan](../planning/implementation-plan.md)
> for the active release inventory.

The release verifier derives capacity from the compiled target and component
inventory. It no longer applies the former global 512 MiB payload, 256 MiB
file, 8192-entry, 1024-directory or 2 MiB manifest limits to every release.
The supported targets and four component variants remain explicit; this is
neither a package resolver nor an admission path for additional tools.

Each pinned file has its exact admitted byte length as its read limit. Its
digest, path and normalized mode must also match. The few unpinned paths have
closed roles with explicit limits:

| Unpinned role | Maximum per file |
| --- | ---: |
| Core/native executable | 256 MiB |
| JavaScript adapter | 16 MiB |
| Python adapter | 1 MiB |
| Core license notice | 1 MiB |
| Component receipt | 64 KiB |

Only named core, adapter and receipt paths can use those roles. The core-only
qualification layout uses the native and notice budgets for Node; the
documents component retains its exact admitted Node pins. An additional
unpinned path is rejected until its owner deliberately assigns a finite role.

The payload limit is a checked sum of the selected file limits. The entry limit
is the exact number of compiled files, links and derived/required directories,
plus one manifest. Traversal still accepts only those exact entries. Capacity
does not permit an extra file, a partial component, a different executable mode,
a case/path collision or an unapproved symbolic link. The manifest name is
reserved against collisions too.

Before parsing JSON, the reader uses a cached ceiling derived solely from the
largest currently supported compiled layout. Each layout's manifest ceiling is
twice its largest standard pretty-printed envelope, plus one newline byte. The
envelope uses compiled paths, fixed-length digests and maximum permitted file
lengths. After parsing, the selected layout's smaller ceiling is enforced as
well. Manifest input cannot increase either ceiling; excessive whitespace is
rejected. Overflow in capacity derivation or byte accounting is an error.

The current compiled four-component macOS ARM64 layout has 6,843 files, two
links and 955 directories. Its payload ceiling is 874,469,697 bytes and its
manifest ceiling is 3,235,553 bytes. The HTML/CSS admission added 117 pinned
package files, one supplemental notice and two bounded authored adapters;
JavaScript/TypeScript syntax inspection adds four exact TypeScript package files
and one bounded authored adapter. Selected-snapshot semantic navigation adds
two bounded authored TypeScript adapters. Python source inspection adds one authored
private Python leaf beside the existing SQL/PDF scripts. The earlier SQL/YAML qualification candidate contains
533,226,987 payload bytes and a 1,587,628-byte manifest. Its original bytes and
release identity need no change for this policy.

Future browser or analyzer work must add a reviewed, target-specific compiled
selection with exact pinned files and approved rights before it can affect the
layout or its ceiling. The synthetic scaling test is not browser admission.
Browser redistribution remains blocked by its separate rights requirements.

## Verification

On macOS ARM64, the focused distribution tests exercise all four actual
components together; an inventory larger than the former entry/directory,
payload/file and manifest limits; checked-sum overflow; unknown file roles;
case, file/directory and manifest-name collisions; traversal; exact aliases;
corrupt and missing files; and selected-layout JSON/read limits. The scaling
fixture contains synthetic compiled pins and does not allocate or hash a
multi-gigabyte payload. Native Windows execution, browser rights and desktop
activation require their own evidence.

Checked on 2026-10-25 with the locked Rust toolchain on macOS ARM64:

- `python3 scripts/check-build-inputs.py` passed before compilation.
- `cargo test --locked --offline distribution::tests` passed all 19 tests.
- `cargo clippy --locked --offline --all-targets -- -D warnings` passed.
- `rustfmt --edition 2024 --check src/distribution.rs` passed.
- `cargo build --locked --offline --bin mirket` passed. Its read-only `doctor`
  returned `healthy` for the existing disposable SQL/YAML candidate home,
  including manifest and launcher integrity. The release stayed
  `sha256:e29fbc09218821c99ec69c05518ae2928d819191bf162dc71a61e6c022f8147b`.

That final check verified the saved installed payload through the revised
verifier; it did not replace the candidate binary, alter client configuration,
or establish new desktop activation evidence.
