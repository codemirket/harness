# M3 documents: bounded direct text replacement

> **Historical qualification record.** This PDF editing slice is preserved for
> provenance only and is not part of the current Mirket release surface. The
> approved rewrite assigns PDF, Office, extraction and OCR workflows to Codex
> Desktop and host tools; no Mirket PDF MCP tool is supported. See [the current
> implementation plan](../planning/implementation-plan.md) for the active scope.

Status, 2026-10-25: **the Python candidate and its disposable macOS ARM64
qualification pass**. Rust tool registration, receipt handling, installed release
admission and actual Codex MCP invocation are owned by the integration lane and
are not established by these Python checks. Native Windows remains untested.
The former public workflow was `pdf_text_replace_prepare` followed by the shared
`pdf_start`, then the existing job status/result/cancel operations. The shared
start also handles page transformations.

This slice addresses decision 72's actual old-text removal requirement without
adding an editing backend. It changes one uniquely identified complete `Tj`
text-show object in a simple imported PDF. It preserves the original font,
size, text position, page geometry and Info metadata. It writes a new private
stage; the existing Rust artifact owner decides publication. It does not
implement paragraph wrapping, general imported-PDF reflow, source reconstruction,
OCR, sanitization or redaction.

## Qualified boundary

All pages must pass the existing PDF catalog/page-tree whitelist. This excludes
forms, annotations, links, outlines, XMP, actions, page labels and unsupported
page attributes rather than silently dropping them. Every page must also satisfy
the new finite content validator:

- Unrotated pages; only regular Type1 Helvetica, Courier or Times-Roman core
  fonts. No embedded/subset/custom fonts, width overrides, ToUnicode maps or
  encoding differences. StandardEncoding and WinAnsiEncoding are accepted only
  where every used printable ASCII byte maps to the identical character.
- Each text object is exactly `BT`, explicit `Tf`, one `Td` or identity `Tm`,
  one `Tj`, `ET`. Text is opaque black, at 6–72 points, with 1–256 printable
  ASCII characters. No kerning arrays, text-state overrides, clipping,
  transforms, XObjects, images, marked content or unexamined operators.
- Other admitted operations are gray/RGB fill colors and positive filled
  rectangles. The conservative font-wide target envelope must remain inside
  both page boxes and intersect no other text envelope or rectangle. This
  deliberately refuses tight layouts that could be valid with finer geometry.
- Expected text must be one complete text object occurring exactly once in the
  document's extraction and on the selected page. The replacement must not
  already occur elsewhere or retain the expected text as a substring.
- Replacement advance must not exceed the original advance. More characters
  are allowed when their measured advance still fits; a longer line that needs
  more width refuses. No font resizing, substitution or automatic repagination.

The editor uses the exact pinned pypdf font metrics. It replaces the content
stream through `PageObject.replace_contents()`, performs an ordinary full save,
and reopens the serialized output. It compares all content operators and
operands except the one declared string, all admitted font resources and
ProcSet values, page geometry and metadata. Saved extraction must equal the
original extraction with exactly that one replacement, with no old text and
exactly one new string. This establishes the scoped page-text edit; it is not a
promise that a sensitive string is absent from every PDF byte or metadata field.

Limits remain 10 MiB source, 20 pages, 20 MiB serialized output and 32 KiB request
and response frames. Additional limits are 1 MiB decoded content and 10,000
operators per page, and 1,000 text objects per page. Parsing/decompression happens
inside the supervised worker; these post-decode checks are not a hard memory
quota or a security sandbox. The Rust owner must retain its execution deadline,
owned-process cancellation, source/destination checks and uncertainty handling.

## Frozen private transport

The adapter is `adapters/python/pdf_text_replace.py`, deployed alongside the
existing `adapters/python/pdf_operations.py`. It reuses that sibling's verified
wheel loading and envelope, document validation, byte budgets and response
format. Both authored files must be bound by the release manifest. Invoke the
same selected private interpreter and admitted pypdf wheel:

```sh
/absolute/private/python3.13 -I -S -B /absolute/release/adapters/pdf_text_replace.py \
  --wheel /absolute/release/libraries/pypdf-6.21.2-py3-none-any.whl \
  --wheel-sha256 1d44d1a3a200f25062adff0085b02040e4646431ac2ccac22e42679e91466084
```

Send one newline-terminated JSON request and EOF, using the existing exact
envelope fields `protocol_version`, `release_id`, `job_id`, `input_path`,
`source_revision`, `output_path`, `operation`. Protocol version is `1`; source
revision is `sha256:` followed by 64 lowercase hex digits. The operation has
exactly these fields:

```json
{"kind":"replace_text","page":0,"expected_text":"REPLACE ME","replacement_text":"EDITED"}
```

Page is a zero-based integer in 0–19. Both strings have 1–256 printable ASCII
characters. Identical strings, expected text contained in replacement text,
unknown fields, duplicate JSON keys, nonfinite numbers and malformed frames
are invalid requests. The source must be an identity-stable regular file with
one link in a canonical private `0700` directory; the nonexistent output must
be a distinct sibling stage. The worker rereads source identity and bytes before
exclusive `0600` stage creation, flushes/fsyncs, and rechecks the closed stage.
A failure after stage creation remains an unsuccessful operation with retained
stage evidence; it must not be reported as a no-output refusal or replayed.

Response envelope is exactly `protocol_version`, `release_id`, `job_id`,
`result`. A staged result has exactly `status`, `source_revision`, `source_bytes`,
`output_revision`, `output_bytes`, `document`, `change`; `status` is `staged`.
`document` is the unchanged existing PDF inspection schema. `change` has exactly:

```json
{"page":0,"operation_index":4,"font":"Helvetica","font_size":16.0,
 "original_advance_points":103.136,"replacement_advance_points":58.672,
 "change_box":[27.344,166.4,149.136,184.89600000000002]}
```

These are actual shorter-Helvetica fixture values. Page and operation index are
zero-based; index addresses the parsed content operator. Font is `Helvetica`,
`Courier` or `Times-Roman`. Sizes, advances and boxes are finite PDF points;
`change_box` is `[left,bottom,right,top]` in unrotated PDF coordinates. No source
or replacement text is returned or written to adapter logs.

A semantic refusal returns exactly `{"status":"refused","code":"..."}`
and creates no stage. The finite codes are:

```text
ambiguous_text clipping content_budget destination_exists encrypted input_budget
invalid_pdf output_budget overlap page_range replacement_exceeds_original_width
result_budget serialization_mismatch source_changed text_integrity text_not_found
unsupported_catalog unsupported_content unsupported_crypto unsupported_encoding
unsupported_font unsupported_metadata unsupported_page unsupported_resources
unsupported_rotation unsupported_text_color unsupported_text_object unsupported_version
```

Malformed transport/runtime identity failures exit 2 without stdout and log only
`pdf_text_leaf_failed:<exception class>`. Rust must validate the typed operation,
receipt and response, bind the new script's identity, route `replace_text` to
this leaf, independently verify stage bytes, and preserve the existing artifact
publication/recovery boundary. A text-replacement receipt must bind both exact
strings without persisting document body text in ordinary receipts or logs;
keep any needed request body in the existing private transient preparation
state. No separate installer, Python runtime or job supervisor is required.

## Dependency and evidence record

No dependency was installed or added. The existing admitted pure-Python pypdf
6.21.2 wheel is 407,766 bytes, SHA256
`1d44d1a3a200f25062adff0085b02040e4646431ac2ccac22e42679e91466084`.
The qualifier verifies its existing BSD-3-Clause license against the admission
receipt and verifies the unchanged source retains the Adobe Core 14 AFM rights
notice used by the metric data. No font data was copied into authored source.
The full admitted wheel must remain unchanged, retaining those notices. No
fontTools, crypto, image, ML or network dependency is needed by this slice.
The pinned font helper is an internal pypdf API, so a future pypdf upgrade needs
the same qualification rather than an assumed compatible import.

Qualification ran on macOS 26.5.2 build 25F84 ARM64, private CPython 3.13.17,
and explicitly selected host Poppler 26.10.0. Poppler is an independent test
observer here, not an admitted runtime dependency or installed-tool fallback.
The private Python executable SHA256 is
`71d176d8e02702c2ad568148caf36cfae92cd862f6a72c94c03643a61c04bd54`.

`tests/test_pdf_text_replace.py` passes all 16 test groups against the admitted
wheel, including an actual isolated empty-environment child process. It covers
the supported fonts, identity text matrices, escaped ASCII, actual old-stream
removal in the fixture, metadata and unaffected page preservation, longer
width-fitting text, and discriminating refusal cases. Stale source/output changes
during preparation preserve the current data. Injected post-write fsync failure
retains the uncertain stage. The seven existing `test_pdf_operations.py` groups
also pass unchanged.

`scripts/qualify-pdf-text-replace.py` requires explicit interpreter, wheel,
renderer, extractor and fresh evidence-root paths. Four two-page source/result
cases pass independent Poppler extraction and 72-DPI rendering: shorter
Helvetica, equal-width Courier, shorter Times-Roman and Helvetica with more
characters but lower advance. Each has nonzero changes inside the declared
region and zero changed pixels outside it, with a one-pixel mask margin. Every
unaffected second page is pixel-identical. All 16 rendered source/result pages
were visually inspected: the edits are legible, the preserved text/rectangle
remain intact, and no clipping or overlap is visible. Final rendered PNG hashes
match the inspected bytes exactly.

The machine report, inputs, outputs, PPMs and PNGs are retained outside shipped
source in
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/pdf-text-next-slice-20261025-final/`.
The report SHA256 is
`2c2a6b36df4a27980ad38fc61fa38745a91e25b3dbb4fe6c0bc6660cb5050c27`.
Its exact executable, source, input/output and rendered-image identities make
the observations reproducible; the hash itself is not semantic proof.

The adapter is now wired through public `pdf_text_replace_prepare` and `pdf_start`
using the existing PDF receipt owner. Rust unit tests and the disposable installed
`documents` fixture pass, including saved-text verification by independent
Poppler extraction, source preservation, a too-wide replacement refusal, exact
wheel/notice checks and no destination on refusal. The combined macOS ARM64
release also passes installed SQL navigation and PDF inspection alongside design
and semantic payload discovery. These checks do not establish parser-instruction
interruption, a Codex model turn, native Windows behavior, arbitrary-PDF fidelity,
automatic reflow, reconstruction or complete M3 documents.
