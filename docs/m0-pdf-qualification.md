# PDF qualification

> **Historical qualification record.** The PDF MCP surface and installed
> workflow described on this page belong to an earlier rewrite slice. They are
> retained as evidence and are not shipped or supported by the current product.
> The approved scope leaves PDF, Office, extraction and OCR work to Codex
> Desktop and host tools. See [the current implementation plan](../planning/implementation-plan.md)
> for the active boundary.

The former public PDF surface was `pdf_inspect`, `pdf_transform_prepare`,
`pdf_text_replace_prepare` and shared `pdf_start`, followed by the existing
job status/result/cancel tools. This document records the historical page-transform
qualification; [direct text replacement](m3-documents-next-slice.md) has its own
bounded contract and evidence.
Preparation binds the exact source revision, destination and transformation for
five minutes. Typed receipts prevent a text start permission from starting PDF
work or the reverse. Pending text/PDF source snapshots share a 64 MiB bound.

The finite PDF worker shares the existing admission and ownership lifecycle.
It writes only its private stage; the Rust artifact owner rechecks the result and
source, persists the publication boundary, and creates a new output without
replacing an existing destination. Inspection can read 20 MiB, including the
largest allowed saved output. Transformation still accepts at most 10 MiB input.

The implementation uses explicit supported catalog, page-tree and page attribute
sets. Unknown features refuse before staging, including outlines and attributes
that the original baseline blacklist could have silently dropped. Accepted Info
metadata is preserved except for the explicit Title/Author patch. Output is
serialized, reopened and checked before staging, then independently hash-checked
by Rust. Raw PDF bytes and Info values are not stored in operation receipts.

The complete `documents` component payload binds the same private Python used
by SQL, both exact wheels, their notices and the authored adapters. The installed
fixture exercises inspect → prepare → start → result → saved-revision inspect,
preserving source bytes and metadata. Modified pypdf wheel/notice bytes, Python
bytes and interpreter aliases are refused before activation. There is no partial
SQL-only installed profile or host interpreter fallback.

The probe uses pypdf 6.21.2 from the unchanged pure Python wheel, SHA256
`1d44d1a3a200f25062adff0085b02040e4646431ac2ccac22e42679e91466084`,
inside the selected private CPython 3.13.17 runtime. All 611 runtime inventory
entries match before and after execution. No PDFium native module is involved.

An independently authored three-page PDF is reduced to pages GAMMA and ALPHA,
with GAMMA rotated clockwise 90 degrees. Title and author metadata change while
the fixture's unrelated metadata remains intact. Saved-file inspection verifies
page order, text, rotation and boxes. Independent macOS PDFKit/CoreGraphics
inspection agrees. Visual review of all three source pages and both saved pages
finds the expected labels, colored rectangles and composition preserved without
clipping. The original PDF remains unchanged.

Five discriminating refusal cases also pass: encrypted input, malformed input,
input over the byte limit, an existing destination and a stale source revision.
The existing destination is preserved. The six cases finish in approximately
3.3 seconds under a network-denied execution policy; this single fixture run is
not a performance benchmark.

Standalone baseline limits are 10 MiB input, 20 pages, 20 MiB output, six seconds
per Python operation and bounded captured output. The supervisor samples total
scratch size and enforces a whole-run deadline. These controls do not constitute
a hard native-memory quota or a same-user security sandbox.

This baseline deliberately excludes forms, annotations, actions and other
unsupported document structures. It makes no general preservation, sanitization,
OCR, redaction or arbitrary-PDF editing claim. Direct text replacement has a [separate bounded qualification](m3-documents-next-slice.md). Broader format fidelity, parser-specific interruption
and supported-platform qualification remain separate work. Frozen sources, native evidence and the
separate visual-review record are retained in the implementation task record.

## Actual MCP lifecycle verification

The native MCP fixture passes all eleven groups, including wrong-kind/foreign
start refusal, duplicate start, unsupported outlines, late destination/source
changes, lost response and reconnect without replay. Independent saved-PDF text,
geometry and two-page visual inspection agree with the expected transformation.

Controlled private Python leaves make cancellation observable before parsing
and after an exclusive partial stage has been flushed. Explicit cancellation,
graceful client EOF and abrupt MCP server termination all stop the leaf, release
heavy admission and remove supervisor temporary state. The partial stage retains
its exact identity and bytes; no destination is published. A real PDF inspection
then succeeds in the same state. Reconnection inspects the original receipt and
cannot restart it. Abrupt server loss leaves cleanup confirmation unknown rather
than inventing a persisted cleanup result. These are process-ownership tests,
not evidence of interrupting every possible parser instruction.

The production PDF leaf has a 60-second execution deadline and 32 KiB request/
response limits. Its process ownership does not impose a hard memory quota.
