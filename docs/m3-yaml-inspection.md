# Revision-bound YAML occurrence structure

`src/yaml_inspect.rs` implements read-only structure for one explicitly selected
local YAML source. It reuses `ProjectScope`, the 1 MiB source limit and full SHA-256
revisions. It writes no source/state, starts no process, resolves no schema and
performs no network access. The exact admitted parser and closure are recorded in
[the YAML admission](../packaging/admission/m3-yaml-inspect.md).

An empty pointer selects the document set. Its ordinal children are documents;
each document contains one root. Mapping children are entry occurrences, each
with key child `0` and value child `1`; sequence children are item occurrences.
Thus `/0/0/2/1` selects the value of the third mapping entry in the first
document. Paths are opaque structural occurrence identities, not YAML key lookup,
JSON Pointer, schemas or semantic symbols. Duplicate and complex keys retain their
original occurrences and order. Key uniqueness/equivalence is not evaluated.
Every nonempty pointer and pagination continuation requires the exact source
revision. Changed sources return no structure. A second scoped read checks observed
drift; this is not an atomic snapshot or a hostile-filesystem sandbox.

Scalars expose style and original lexeme locations, never parsed Boolean, null,
numeric, timestamp or string values. Quoted escape sequences, numeric spelling,
CRLF and Unicode therefore remain source text. Flow collection spans are emitted
only after checking the opening and closing source delimiters. Anchor presence is
metadata; preceding anchor properties are excluded from scalar/flow spans. Document
and mapping-entry envelopes, block collections, implicit empty scalars and full
literal/folded block-scalar lexemes have explicit unavailable locations. The pinned
parser reports block-scalar content rather than a proven full lexeme. Comments,
directives and anchor names/property locations are omitted explicitly in coverage.

Aliases (including recursive aliases), merge-key spellings and all explicit tags
return explicit refusals without children. This includes quoted `<<` keys and
standard explicit tags as a conservative boundary. No alias is expanded and no
merge/tag is evaluated. Malformed input returns `invalid_yaml` with only a checked
original byte position when available; parser diagnostic text is not echoed.

Streaming parser events feed a flat arena, bounded to depth 64 and 32,768 nodes,
including synthetic documents/entries. Pages expose at most 100 immediate children
and fit the caller's serialized budget up to 32 KiB, including escaped metadata and
continuations. Incomplete, invalid, refused, stale, missing, unavailable and budget
outcomes are distinct; `complete` describes immediate enumeration only.

The first Unicode test caught an upstream contract defect: `Marker` documentation
says bytes, while the scanner counts characters. Selecting the character-iterator
API and mapping every original character boundary to UTF-8 bytes fixed the
truncated `şehir🙂` key range. A bounded table preserves CR and LF independently;
no normalized/serialized parser output is used as source evidence. Quoted-scalar
and flow-delimiter event extents also include trailing spaces/comments in some
cases; bounded original delimiter checks exclude that ignored syntax. Plain
scalar trailing comments/spaces and both quote escape styles are tested. NUL input is
explicitly rejected rather than allowing a scanner sentinel to hide trailing data.

Development verification passed on macOS 26.5.2 ARM64 (`aarch64-apple-darwin`)
with locked Rust 1.99.1 (58cf8f9c3, LLVM 23.1.1).
Focused tests cover Unicode/emoji/combining escapes, Unicode comments, CRLF and
multiline lexemes, repeated/complex mapping keys, multiple/empty documents,
anchors, alias/merge/tag refusals, invalid/NUL input, depth/node/source/result
bounds, revision drift, pagination, scope and symlink refusals and unchanged source
bytes. `python3 scripts/check-build-inputs.py` passed;
`cargo test --locked --offline yaml_inspect::tests` passed all 14 focused tests,
and `cargo clippy --locked --offline --all-targets -- -D warnings` passed. MCP integration, final packaged payloads, native Codex activation
and Windows qualification remain separate evidence gates. This slice establishes
source structure only; it does not establish semantic language navigation.
