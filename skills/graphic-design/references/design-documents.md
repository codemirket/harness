# Editable design documents

Use the `design` package's OpenPencil file tools for layered, editable design
files. The pinned official CLI runs headlessly in the existing tools container;
no desktop app, login, database or plugin connection is required. Tool paths
are relative to the reported workspace (project-scoped for routed workers).

Compile `design/tokens.json` first. `openpencil_document` creates a `.fig` file or
imports `.fig`/`.pen` source and imports canonical DTCG tokens as variables and
styles. `openpencil_inspect` without source lists supported upstream tools; with
only `tool` it returns that tool's argument schema. Use a read tool plus `source`
to inspect layers, geometry, text, variables and component identities.

Use `openpencil_edit` with source, upstream tool, arguments and a `.fig` output.
Use the actual schema; do not invent a design-object API. Editing an existing
output requires `overwrite: true` and its current `expected_sha256`. Canonical
tokens are compiled before the edit; document creation/import adds variables and
styles. To refresh them later, use the upstream `import_design_tokens` operation
with the compiled DTCG contents, then read back the saved IDs. Unsupported code execution, network tools
and desktop state operations are omitted. No provider credentials are passed.

Reopen the saved file with `openpencil_inspect`; export it with
`openpencil_export` to PNG (2x) or SVG. Export leaves the source unchanged.
Inspect PNG pixels independently for hierarchy, typography, clipping and layout;
compare visible text and IDs with the saved document. Use embedded assets and
bundled fonts; remote font/asset downloads are disabled. A saved file, successful
export or dimension receipt does not prove visual quality or font fidelity.

Keep `.fig` source and previews together. Open the `.fig` in OpenPencil for manual
editing. The runtime does not control a user's open desktop document. Existing
Penpot volumes are retained during the stopped-generation cutover, with no claim
of automatic conversion; export required legacy designs before switching.
