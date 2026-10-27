# Original-source HTML and CSS structure

`adapters/src/web-source-inspect.ts` exports a pure
`inspectWebSource(source: Uint8Array, request: WebSourceInspectRequest)` leaf.
It inspects one supplied UTF-8 byte snapshot and returns
`WebSourceInspection`. The calling source owner must scope/read the saved file,
reserve transport-wrapper bytes and recheck the saved revision before returning
the result. This leaf adds no MCP server, file reader, source edit, persistent
state, project configuration lookup, plugin callback or execution path.

```ts
const page = inspectWebSource(savedBytes, {
  language: "html", // or "css"
  pointer: "",
  start_child: 0,
  limit: 100,
  max_response_bytes: 30_000,
});
// Select an observed child, or page its siblings, using page.revision.
const selected = inspectWebSource(savedBytes, {
  language: "html",
  pointer: "/0",
  expected_revision: page.revision,
});
```

Pointers are opaque zero-based occurrence paths, such as `/3/0`, in this exact
structural model. They are not XPath, selectors, DOM identifiers or semantic
symbols. Every nonempty pointer and every nonzero `start_child` requires the
full observed `sha256:` revision. Revisions hash the supplied original bytes,
including BOM, CRLF and Unicode. A mismatch returns `changed` with the current
revision and no selected node or children. Missing pointers and invalid child
offsets have separate statuses. Changing language changes the structural model;
the caller must preserve the explicit requested language along with a pointer.

Successful `page` results identify the exact backend/version, revision and source
size. `selected` and `children` include `pointer`, `kind`, optional original HTML
`name`, `child_count`, and either an original UTF-8 `location` with half-open
`start_byte`/`end_byte` or a null location with `location_reason`.
`complete` describes pagination; `structure_complete` describes the finite
structural model. A finished page may still describe incomplete source. A page
continuation carries the pointer, revision, next child offset and limit; retain
the language and caller's result budget when using it.

The HTML backend is `vscode-html-languageservice` 5.6.2. It uses the public element
hierarchy and scanner with a fixed local void-element data provider. Attributes
retain their original order and duplicate occurrences, followed by comments,
text and elements in source order. Attribute ranges include the name, any
assignment/spacing and the original value token. `name_location` selects the
original attribute name; `value_location` preserves quotes when present. Boolean
attributes have no value location. Unclosed quoted values have no full attribute
or value location; the separately proved name location remains available.

The hierarchy is the language service's source hierarchy. Optional end tags,
unclosed/mismatched tags and scanner recovery make structure incomplete. The
leaf does not invent browser-created `html`, `head`, `body` or table nodes.
Unproved whole-element ranges are null. Unmatched closing tags are explicit
source entries. Unterminated comment/doctype ranges are unavailable. Document
whitespace text, complete comments and doctype spellings are included; whitespace
inside tags is represented only as part of attribute/tag source ranges. HTML
entity spellings, script contents and style contents remain original opaque
source. Resource-bearing attributes and declarations are inert data; nothing is
resolved, loaded, executed or fetched.

The CSS backend is `css-tree` 3.2.1, reused from the existing SVG dependency
closure. It exposes its public positioned stylesheet AST with source-ordered
children, including selector/value structure, nested rules and declarations.
Comment callbacks restore ordinary comments that the AST normally omits;
exclamation comments are deduplicated. Each comment belongs to the narrowest
enclosing positioned AST node. Whitespace and redundant separators are omitted
from the model but remain in the original source. Hexadecimal escape terminator
whitespace remains inside an original token's span when the parser includes it.

CSS parser recovery, malformed tokens, unclosed strings/comments/URLs and
unbalanced delimiters return `refused` without a partial tree. Raw nodes, including
custom-property values and some function arguments, retain exact source intervals
but make structure incomplete. Empty ordinary declaration values and missing
parser locations have explicit unavailable locations. This is structural syntax
inspection, not CSS property/value validation, rendering, cascade evaluation,
symbol search, definitions or references. No schema, stylesheet import, URL,
project plugin, formatter or custom provider is resolved or called.

Source is limited to 1 MiB of valid UTF-8 with no NUL, model depth to 64, model
nodes to 32,768 and scanner tokens to 65,536. The CSS tokenizer checks delimiter
nesting before recursive parsing. Pages return at most 100 immediate children.
The complete leaf result is bounded by `max_response_bytes` between 1,024 and
32,768 bytes including JSON escaping. Oversized selected/child metadata yields
`budget_blocked` with an expandable original span, never a skipped child or a
continuation without progress. Pointer/request metadata that cannot fit the
budget refuses explicitly. Diagnostics are at most 16 representative issues;
`issues_truncated` marks a full or further budget-trimmed issue sample. Source
location conversion preserves original UTF-16 parser boundaries through a single
UTF-8 byte table, rejecting surrogate-interior or otherwise unproved offsets.

On macOS 26.5.2 ARM64 using Node 24.18.1, TypeScript compilation and all 18
focused tests passed. The tests read saved HTML/CSS bytes and check independent
expected slices for emoji, Turkish characters, CRLF, BOM, duplicate attributes,
comments, strings and escapes; they also exercise stale/missing pointers,
malformed source, Raw coverage, pagination, byte/depth/node/token limits and
metadata blocking. A sentinel test replaces filesystem, HTTP/fetch and process
APIs with failures during resource-bearing source analysis and observes zero
calls. This verifies those exercised paths; it is not an OS sandbox claim.

The dependency closure and retained notices are in
[the admission](../packaging/admission/m3-web-source.md). These tests qualify the
pure leaf. Source scope/recheck, transport/MCP and macOS ARM64 installed
packaging subsequently passed the [separate integration qualification](m3-web-inspection-transport.md).
Windows and actual Codex Desktop activation remain separate observations.
