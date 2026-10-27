# HTML and CSS original-source inspection admission

Reviewed 2026-10-25 under the user's standing dependency approval. This batch
adds the pinned `vscode-html-languageservice` 5.6.2 and promotes the already
admitted `css-tree` 3.2.1 to a direct adapter dependency. Four exact overrides
pin the HTML closure. No CSS language-service package, native binary, runtime,
formatter operation, network client or project plugin is added.

| Package | Exact version | Admission | License |
| --- | --- | --- | --- |
| vscode-html-languageservice | 5.6.2 | New | MIT, with bundled notices below |
| @vscode/l10n | 0.0.18 | New | MIT |
| vscode-languageserver-textdocument | 1.0.16 | New | MIT |
| vscode-languageserver-types | 3.17.5 | New | MIT |
| vscode-uri | 3.2.0 | New | MIT |
| css-tree | 3.2.1 | Reused from SVG production | MIT |
| mdn-data | 2.27.1 | Reused unchanged | CC0-1.0 |
| source-map-js | 1.2.2 | Reused unchanged | BSD-3-Clause |

The [exact inventory](m3-web-source.inventory.json) records dependency edges,
registry archive URLs, verified SHA-512 integrity, archive SHA-256, compressed and
unpacked sizes, file counts, every package script, and embedded notice hashes.
All eight packages lack `preinstall`, `install`, `postinstall` and `prepare`
scripts. Installation used `--ignore-scripts`; no upstream build, prepack,
publishing, data-update or formatter helper ran. The existing TypeScript compiler
and adapter runtime selection are unchanged. The complete adapter production
tree contains 131 packages after the five additions; the finite release inventory
must be reconciled against the changed lockfile before assembly.

All embedded license and notice text is retained in
[the notice record](m3-web-source.notices.json). The HTML archive includes JS
Beautifier MIT and W3C HTML 5.1 notices, even though source inspection does not
call formatting. The `@vscode/l10n` npm archive omits a license file. Its manifest
declares MIT; the missing notice is supplemented from upstream's immutable
[LICENSE commit](https://raw.githubusercontent.com/microsoft/vscode-l10n/524fe8f6aa9628744bb40ecdc73dae51e6471cfd/LICENSE).
The provenance limit is explicit: this is a retained current upstream license,
not a recovered source commit for the 0.0.18 npm archive. Preserve this supplement
when assembling the installed runtime.

The [advisory record](m3-web-source.advisories.json) contains the exact isolated
eight-package `npm audit --omit=dev --json` result: zero reported advisories on
the review date. This lookup does not establish absence of vulnerabilities or
constitute a source/security audit.

The official [HTML archive](https://registry.npmjs.org/vscode-html-languageservice/-/vscode-html-languageservice-5.6.2.tgz)
exposes public scanner offsets and element hierarchy. The leaf supplies only a
fixed local HTML void-element provider and no filesystem provider. It uses the
public scanner to preserve duplicate attribute occurrences, comments, doctype,
opaque text, and original tag boundaries. It does not claim browser DOM repair,
schema evaluation, rendering, document symbols, definitions or references.

The candidate CSS service 6.3.10 exposes `Stylesheet = {}` in its public type
contract; its public document symbols omit ordinary declarations/comments and
can duplicate a rule's extent for multiple selectors. The leaf therefore uses
the existing css-tree package's public parser/tokenizer exports. The pinned
[parser contract](https://github.com/csstree/csstree/blob/8a6caba481be4cae4b0e8690af643ff8e59271f2/docs/parsing.md)
and [AST contract](https://github.com/csstree/csstree/blob/8a6caba481be4cae4b0e8690af643ff8e59271f2/docs/ast.md)
provide positioned original-source nodes, comment callbacks and explicit Raw
nodes. Recursive delimiter nesting is bounded with its public tokenizer before
parsing. Recovery callbacks and malformed tokens/delimiters refuse the tree;
valid unparsed Raw values remain explicitly incomplete.

The [source/require graph](m3-web-source.sources.json) pins the inspected compiled
HTML source/type files, css-tree implementation files, and 119 observed CJS
runtime files with sizes and hashes. Package exports/metadata and retained notices
also remain necessary to assemble a working installation. A narrow import does
not justify omitting the declared package closure. The HTML root loads bundled
formatter/service modules, although the leaf exposes no formatter operation.

Qualification covers the pure leaf on macOS 26.5.2 ARM64 with Node 24.18.1 and
npm 11.16.0: locked TypeScript compilation, 18 focused tests, and the existing
adapter suite (59 passes plus one existing live-browser skip). The focused suite includes original saved-byte slices,
Unicode/CRLF/BOM, malformed input, comments, no filesystem/network/process calls
during analysis, revision changes, pagination, quotas and explicit metadata
blocking. Rust source scope/rechecks, adapter transport, MCP, installed operation,
Windows and native Codex Desktop activation require their own integration checks.
