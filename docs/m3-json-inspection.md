# Revision-bound JSON configuration inspection

`json_inspect` locates a strict JSON value by JSON Pointer and lists its immediate
children in source order. It returns saved SHA-256 identity and exact UTF-8 byte
spans, allowing `file_read_ranges` to inspect original source without rewriting
large numbers, escaped strings or formatting. It reads only the explicit project
and file through the existing source owner. It creates no state, subprocess,
network request or source edit.

Initial discovery starts at child zero and may omit the expected revision.
Continuation requests include the exact revision and child offset; a changed
source returns no children. Keys use JSON Pointer escaping (`~0` for a tilde,
`~1` for a slash); array indices are canonical zero-based decimal indices. A
scalar is an inspected value with no children. Missing values and invalid
pointer syntax remain distinct.

The complete document is checked before any page: duplicate decoded keys,
including escape-equivalent keys, are refused. Parsing uses serde_json's borrowed
raw values rather than a second JSON parser or floating-point conversion. Input
is bounded to 1 MiB, depth to 64 and value count to 32,768. Pages contain at most
100 children and fit 32 KiB including JSON escaping and the release wrapper.
MCP may also carry a text copy of that result. Oversized child metadata produces
an explicit blocked result and source span, not a continuation that loops without
progress. Source checks are revision-bound reads, not a multi-file snapshot or
filesystem sandbox.

This is structure/key inspection only. Schema resolution, JSONC, YAML, TOML,
remote `$ref` fetching and semantic language navigation are separate capabilities.
The feature-only parser admission is documented in
[the admission record](../packaging/admission/m3-json-inspect.md).

On macOS 26.5.2 ARM64, all nine focused tests and the actual MCP JSON journey
passed with private Node 24.22.1. The fixture inspected nested Unicode and escaped
keys, expanded an exact large-integer source span, paged escaped child metadata
within the complete response budget, refused duplicate decoded keys and detected
revision drift. It also verified no filesystem changes from inspection. Native
cases include depth and node boundaries, large exponent lexemes, scope escapes,
JSONC refusal and explicit oversized-metadata outcomes.

The combined native suite passed 331 library tests and one deterministic retrieval
fixture, with three manual tests ignored and strict all-target Clippy passing.
Core SHA-256 is
`d115785773dde1c23eac0d00b7a6bc0bd7880746c507ad6811b9fa1e9ef03b49`.
Final MCP confirmation on that build is recorded in the task-state
`enumeration-json-mcp-final.log`; Windows and native desktop activation remain
unqualified. These cases do not establish schema or broader language support.
