# Revision-bound XML and SVG structure

`xml_inspect` inspects the original UTF-8 bytes of one explicitly selected local
XML or SVG source. It shares the existing `ProjectScope` source reader and full
SHA-256 revisions. It creates no state, source changes, subprocesses, resource
loads or network requests. The parser is locked `roxmltree` 0.21.2, exposed by the
existing `resvg::usvg` dependency; no dependency admission or manifest amendment
is needed for this slice.

An empty pointer selects the document. Returned pointers are opaque occurrence
paths in this structural model, not XPath, element names, XML `id` bindings or
semantic symbols. Every nonempty pointer requires the observed source revision,
including its first page. This prevents an ordinal from silently selecting a
different occurrence after an edit. Pagination also requires that revision. A
changed source returns no children. A second bounded scoped read detects observed
drift during analysis; this is neither an atomic snapshot nor a filesystem
sandbox against hostile concurrent ancestor replacement.

Element entries list attributes and namespace declarations in their original
start-tag order, then content in document order. Repeated names retain distinct
occurrence pointers. Names preserve source prefixes; namespace URIs come from the
parser and are never fetched. Namespace declarations have their own kind, rather
than being mistaken for ordinary expanded-name attributes. Comments, ordinary
processing instructions and element whitespace text are included. The response
explicitly reports that XML declarations and document-level whitespace are omitted
by the parser. These coverage statements describe successful structural pages;
refusal and malformed results return no partial tree.

Locations refer to original UTF-8 byte intervals, preserving Unicode and CRLF.
Attribute locations include the full original attribute, its qualified name and
its unquoted original value. Start-tag ranges are checked against parser-provided
full attribute ranges; they do not use the parser's documented short subrange
bookkeeping for extremely long names or spaces around `=`. Adjacent text and CDATA
are coalesced by the parser. Their returned source interval covers original text
and CDATA delimiters only after checking sibling and enclosing-element bounds;
unproved intervals are explicitly unavailable. No decoded text, normalized value,
reserialization or guessed parser-error position is returned. Exact ranges can be
expanded through the existing revision-bound source-range reader.

The source limit is 1 MiB. Depth is limited to 64, and the total model to 32,768
entries including the document, elements, attributes, namespace declarations,
text, comments and processing instructions. Pages contain at most 100 immediate
entries and fit the caller's budget, up to 32 KiB including JSON escaping. The
integration caller reserves release-wrapper overhead. Oversized names or
namespace metadata produce an explicit blocked result with source locations,
rather than skipped entries or a continuation without progress.

All DTDs and all entity/character references outside literal comments, CDATA and
processing instructions are refused before parsing. This deliberately includes
predefined references such as `&amp;` and numeric references. Parser options also
disable DTDs and supply no external entity resolver. Recognized resource-bearing
forms are refused: stylesheet processing instructions, XInclude, external
`href`/`src`, schema-location attributes, `xml:base`, and recognizable CSS imports
or non-fragment `url(...)`. Namespace URIs and arbitrary XML string data remain
inert names/data. This conservative check is not a validator for every embedded
language or custom XML vocabulary; the implementation never resolves any
resource, irrespective of its spelling. Fragment references are inspected as
source only and are not resolved.

The focused tests cover Unicode, CRLF, mixed text/CDATA, namespaces and duplicate
expanded names, repeated element identity, comments and whitespace coverage,
DTD/entity/resource refusal, malformed input, long attribute subranges, output
budgeting, pagination, revision drift, depth/node limits, oversized/invalid UTF-8
sources, scope escapes and symlinks. On macOS 26.5.2 ARM64 with Rust 1.99.1, the
build-input guard, all 13 focused tests (`cargo test --locked --offline
xml_inspect --lib`) and strict library Clippy (`cargo clippy --locked --offline
--lib -- -D warnings`) passed. This module establishes structural inspection only;
schema processing, rendering, symbol search, definitions, references, Windows
behavior and actual Codex Desktop activation require separate qualification.
