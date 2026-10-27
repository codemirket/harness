# Selected-source TypeScript semantic navigation

`code_navigate` provides definitions and references for JavaScript, TypeScript,
JSX and TSX in an explicitly supplied saved-source graph. The fixed private
TypeScript **5.9.3** language service supplies binding and scope information.
This extends the existing selected-source engine using the unchanged selected
`lib/typescript.js`, its notices and private Node. There is no new analyzer,
server protocol, fallback engine or project loader.

Select **1–8 project source snapshots** and **0–8 declaration snapshots**, at
most 16 saved files under one project root. Local variables, parameters, private
names, object/interface properties and selected script globals retain their
scope. Static imports and re-exports resolve only through the caller's explicit
source-specific binding table. Every result keeps `model_complete: false`:
the selection does not establish a complete project or valid types. `complete`
describes query-list truncation only.

The graph extension is verified in source-level adapter and owned-worker tests,
through a disposable installed MCP release and in one bounded
[Codex model turn](m3-typescript-codex-model-qualification.md). Desktop GUI and
broader client outcomes remain separate gates.

## One finite request model

Supply `expected_release_id`, absolute `project_root`, and `request` with:

- `sources`: distinct absolute saved `path`, explicit `language`, and required
  `expected_revision` (`sha256:...`) for every file. Project kinds are `js`,
  `ts`, `jsx` and `tsx`. `ts_declaration` identifies a selected declaration
  snapshot; it is not another supported language family.
- `typescript_model`: required for every TypeScript-family request, even with
  an empty `imports` array. Other language backends are outside this route.
- `typescript_model.imports`: at most 128 objects containing exactly `source`,
  literal `specifier` and `target`. Indices identify selected snapshots; each
  source/specifier key is unique. Specifiers contain 1–256 UTF-8 bytes with no
  controls or malformed Unicode.
- `source`: the query snapshot's zero-based index; `position_byte`: an original
  UTF-8 byte offset inside an identifier/private identifier. EOF, byte interiors
  and non-identifiers refuse.
- `operation`: exactly `definitions` or `references`; `limit`: 1–100.

A declaration at index 0 and importing source at index 1 use
`typescript_model: {"imports":[{"source":1,"specifier":"prepared","target":0}]}`.
This declares the graph; it does not discover or install a package named
`prepared`. The caller owns the mapping, including aliases.

Every static import/re-export in **every selected snapshot**, including unrelated
inputs, needs exactly one entry. Repeated uses of one literal in one source
share an entry. Missing entries, duplicate keys, unselected indices, unused
entries and unsupported forms refuse the entire query without locations.
Named/default/namespace imports and named re-exports must resolve to declarations
in the selected model; a broken imported export elsewhere cannot be hidden by
querying a valid local.

Triple-slash path/type/lib directives, project references, dynamic imports,
`require`, import-equals, import-type expressions, import attributes, `export =`
and string-named ambient modules refuse. No package exports/conditions,
configuration, `node_modules`, default library, type root, plugin, automatic
acquisition, build or source execution is discovered or invoked.

`model_revision` is SHA-256 over an ordered JSON pair: snapshot
`[language,revision]` tuples and import `[source,specifier,target]` tuples. Node
returns it; Rust independently verifies it against the request. Changed/reordered
edges or a declaration revision change this identity. Release/job identity
separately binds the fixed engine and options.

The mapping remains a caller preparation responsibility. The engine rejects a
missing export, but cannot prove that two plausible selected declarations with
the same export identify the intended real package. Correct lookup in a
declared graph is not package provenance or whole-project completeness.

## Saved-source and response boundary

`ProjectScope` reads each path and rejects traversal, detected symlinks,
duplicate canonical paths, nonregular files, malformed UTF-8 and NUL. Existing
path checks do not provide directory-handle containment against hostile
concurrent ancestor replacement.

The response is `{release_id,navigation:{project_root,sources,source,
position_byte,operation,outcome}}`. Ordered metadata binds each index to its
saved path, kind, revision and byte length. Successful outcomes contain fixed
backend `{"name":"typescript-language-service","version":"5.9.3"}`, every source
identity, `model_revision`, fixed coverage, `model_complete:false`, locations and
query `complete`.

Locations carry selected source index/revision and original UTF-8
`start_byte`/`end_byte`; references also carry the service's `is_write` boolean.
They are deduplicated/sorted by source/start/end. Every returned filename must
belong to the selected model, and each UTF-16 offset must prove an original
UTF-8 boundary. BOM, CRLF, Unicode, escaped identifiers and astral prefixes are
preserved without normalization.

`changed` carries all current revisions/nulls and no locations.
`refused`/`budget_blocked` carry bounded reasons and no partial locations.
Truncation has `complete:false`; no continuation cache exists. The pinned
runtime's `failedAliasResolution` and unverified-definition guards prevent a
same-file unresolved import alias being presented as a dependency definition.

## Passive host, resources and ownership

Rust retains paths. Node receives texts, kinds, revisions, indices and the import
table. Names are synthetic `/mirket/source-N.<kind>` or
`/mirket/declaration-N.d.ts`, avoiding actual `node_modules` paths in the
compiler's package-scope heuristics. Snapshot/version and `fileExists`/`readFile`
callbacks consult only that immutable map. Directory callbacks return
empty/false; module resolution is an exact table lookup; type-reference
resolution remains unresolved. No filesystem-backed default host is created.

Options fix `noLib`, `noResolve`, `noEmit`, empty `types`, `allowJs`, `checkJs`,
ESNext target/modules, Bundler mode and preserved JSX. No option, module-path or
callback override is accepted. No plugin loader, tsserver, LSP, emitter, package
manager, project execution or output-file API is exposed.

One service/registry is created per immutable model and disposed in `finally`.
All selected files must be present; syntax errors refuse. An iterative AST walk
bounds depth/nodes, validates all edges and checks imported aliases before the
requested operation. Deep input may fail inside TypeScript before that walk;
service failure refuses and remains subject to the owned deadline.

Original budgets remain **256 KiB per snapshot, 1 MiB aggregate text, depth 64,
32,768 AST nodes, 100 locations and 32 KiB complete MCP result**, including the
reserved path envelope. Counts are eight project plus eight declaration
snapshots. The transport retains 2 MiB request frames, 64 KiB response frames,
16 KiB counted stderr and the existing 30-second leaf deadline. Escaped JSON
and path envelopes can hit transport/output limits below raw source limits;
those requests refuse without silently raising budgets.

The existing same-binary supervisor, connection cancellation, admission lease,
lifeline and owned-child cleanup remain. After cleanup Rust re-reads **every
selected saved input**, including non-returned declarations/files. Any changed,
removed or unreadable input discards all locations. This is sequential checking,
not an atomic multi-file snapshot. No production warm-cache lifetime was added.

Default libraries are absent unless explicitly selected within every bound.
The no-library corpus passes cross-file queries but cannot resolve
`string.length`; Array/Boolean/Function/IArguments/Number/Object/RegExp/String
are missing. Standard-library, Promise/iterator and DOM inference/navigation
remain incomplete. The tested full standard-library closure exceeds the
existing byte/node/transport limits; this change does not lift those limits.

## Verification and current limits

The [in-memory semantic qualification](m3-typescript-in-memory-semantic-qualification.md)
freezes both 17-group experiments, exact runtime/module, standard-library
counterexample, spans, host calls and resource measurements. Its no-library
case was 10 snapshots / 1,003 bytes / 216 AST nodes. The unchanged module SHA-256
is `3ae902c92cc44dace175c0e69e13a4b0899f6983c6121d76b9ab8dd5795e7675`;
the explicit private Node was 24.22.1 on macOS 26.5.2 ARM64.

For the implemented schema, locked TypeScript compilation and **22 focused
adapter tests passed**: eight cross-file/declaration definition cases, exact
six/three-reference groups across the four project kinds, transitive/default/
namespace/named re-export bindings, whole-model unresolved exports,
missing/duplicate/unused/unselected edges, stale declarations, default-library
refusal, unsupported forms, bounds and passive I/O. Prepared graph cases made
zero filesystem/child-process/TypeScript-system trap calls; the focused suite
also retains its network traps.

After the required guard, **12 focused Rust tests passed**, with explicit private
Node, compiled adapter and same-binary paths, and no skips. They cover a real
private prepared-declaration leaf, Node/Rust graph digest, saved declaration
drift, forged result/model identity, scope, transport bounds, cancellation,
cleanup and admission release. A guarded debug binary build passed. Root owns
the final serial broad Rust/Clippy gates.

The first source-MCP smoke reached the tool but refused before dispatch:
`private code navigation runtime is not configured; no host Node fallback is used`.
`dev serve --node --adapter` leaves the design payload unconfigured, and this
owner selects its private runtime from that payload. That earlier refusal was
correct; no fallback was added.

On 2026-10-26, a fresh disposable macOS ARM64 release containing the current
compiled adapter, private Node 24.22.1 and the exact admitted design package
was assembled and activated as
`sha256:888cd610a31b0b433ed29eb96ac7f8d4c7e2fc782d186d2a30fb1b8816d27d8d`.
The package assembler verified 6,128 selected design-package files and copied
only its 6,139 admitted design files including adapters, font and notices.
`doctor-core` reported a healthy installed launcher and manifest. The updated
[`code-navigation-installed-smoke.mjs`](../tests/code-navigation-installed-smoke.mjs)
then passed through the actual installed MCP server, private Node and owned
worker. It checked JS/TS/JSX/TSX locations with Unicode/CRLF byte spans,
lexical shadowing and selected globals; explicit cross-file import definitions;
selected `.d.ts` definitions and declaration revision drift; unresolved binding,
stale release/input, symlink, outside-project and duplicate-snapshot refusal;
and unchanged project files without loading its planted plugin or package hooks.
The output report at task-state `typescript-new-schema-installed-20261026/mcp-result.json`
has SHA-256 `44379e13f040ac4e32e5be4c37ff82f71ff41df9f846d8c2ce1182684297963f`;
the final smoke source SHA-256 is `301cb5f44e8c89b6a6be56fa3138c6913e36393c0c3fd9b85b9e4fc590639827`.
The first installed run failed only because the old test expected
`unresolved_import` for a missing explicit mapping; the new schema correctly
returned `missing_import_binding`. After correcting that assertion, the whole
installed smoke and healthy doctor passed. The failed synthetic project was
inspected for unwanted effects and removed.

One subsequent [Desktop-bundled Codex CLI model turn](m3-typescript-codex-model-qualification.md)
called the installed `code_navigate` tool four times against a frozen TS/JS and
selected-declaration graph. Independent checks confirmed the imported definition,
six exact cross-file references excluding a shadowed local, the `.d.ts`
definition and a whole-model missing-binding refusal. The disposable release
and real user config stayed unchanged. This is one developmental model-use case,
not Desktop GUI or held-out reliability evidence.

Implementation evidence is under
`~/.codex/task-state/harness-rewrite-implementation/typescript-selected-graph-implementation-20261025/`.
Earlier local-only and older-schema Codex model-turn results are historical
evidence for their exact releases. Windows, macOS 15, installed tamper/relocation, an
actual Desktop GUI journey with the new selected graph, framework transforms,
broad diagnostics, performance ceilings and user acceptance remain open. This
existing-engine extension does not admit the native TypeScript 7 executable.
