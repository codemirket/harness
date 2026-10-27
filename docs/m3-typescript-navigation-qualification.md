# TypeScript family qualification

Qualification date: **2026-10-25**. This is a bounded macOS ARM64 experiment,
not analyzer admission, installed navigation or a Codex acceptance result.
Only this document and task-local evidence were authored. Existing installations,
client settings, adapters, Rust, dependency locks and release inventories were
not changed.

## Decision

Prefer **official native TypeScript 7.0.2 LSP** as the single future semantic owner
for TypeScript, JavaScript, JSX and TSX. It passed the finite semantic corpus with
one process and substantially less sampled resident memory than either tsserver
or the wrapper in this experiment. **Do not admit or advertise the inspected
prebuilt payload yet:** its compiled dependency advisories, exact source mapping,
protocol integration and native platform checks remain gates below. The
[official announcement](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/)
and [7.0.2 release](https://github.com/microsoft/TypeScript/releases/tag/v7.0.2)
establish the supported native direction, not Mirket compatibility or security.

A smaller independent slice can proceed with the **TypeScript 5.9.3 pure parser
API subset** for read-only source structure. Its successfully exercised boundary
is described separately below. It does not provide cross-file semantics.

| Exact compared owner | Evidence and disposition |
| --- | --- |
| Official `typescript@5.9.3` direct `tsserver.js` on private Node 24.22.1 | Passed 17 finite groups; package has no registry runtime dependencies. Uses TypeScript's own request protocol rather than LSP. An explicit external project gives useful control over root files. Retain as a contingency only if the native closure cannot be qualified; adopting it requires a deliberately bounded protocol owner, not a silent fallback. |
| `typescript-language-server@6.0.2` with explicitly selected TypeScript 5.9.3 | Passed the same 17 groups. It converts LSP to tsserver and adds a second process plus a bundled JavaScript closure. Explicit `tsserver.path`, disabled acquisition and syntax-server suppression worked. It is not a stronger semantic implementation than its selected TypeScript server. |
| The same wrapper with TypeScript 6.0.3 | Passed the same 17 groups; no corpus result justified a separate runtime. The wrapper's upstream development dependency is TS 6.0.3. |
| `@typescript/typescript-darwin-arm64@7.0.2` directly | Passed the 17 groups with source-correct configuration and an exact inert registration acknowledgement. Has native LSP and one process. Conditional primary direction; release gates remain open. |

The wrapper's [configuration contract](https://github.com/typescript-language-server/typescript-language-server/blob/v6.0.2/docs/configuration.md)
allows explicit TypeScript paths and plugins and documents
`disableAutomaticTypingAcquisition` and `useSyntaxServer: "never"`.
Its [package](https://registry.npmjs.org/typescript-language-server/6.0.2)
requires Node >=22.22.2 and publishes a bundled `lib/cli.mjs` rather than registry
runtime dependencies. Its Apache-2.0 package label is not the complete rights
closure: the shipped [LICENSE](https://github.com/typescript-language-server/typescript-language-server/blob/v6.0.2/LICENSE)
also contains copied VS Code MIT material, and bundled modules need their own
exact-version notice inventory before redistribution. No wrapper was admitted.

## Experiment and actual results

All payloads were fetched as inert npm tarballs into
`~/.codex/task-state/harness-rewrite-implementation/typescript-navigation-qualification-20261025/`.
Their archive bytes matched npm's exact SHA-512 integrity metadata. Extraction
accepted regular files with bounded package-relative paths; no package manager,
install script, build script, upstream helper or project application was run.
These registry integrity checks bind retrieved bytes; registry signatures were
retained but not independently verified in this experiment.

The platform was **macOS 26.5.2 ARM64**. Node alternatives used the existing
explicit private **Node 24.22.1**, SHA-256
`6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`.
Every analyzer ran beneath an explicit `sandbox-exec` profile denying network
access, with a private HOME/config/cache/temp directory and a PATH containing
only task-owned execution sentinels. A separate socket control failed with
`PermissionError: Operation not permitted`, confirming network denial for this
run. This diagnostic profile is not a proposed Mirket sandbox or portable
network policy.

The corpus lived in a path containing spaces and `İ`. Original files retained
CRLF, Turkish identifiers and an astral emoji preceding a declaration. The
hand-authored positive targets covered:

- Five document-symbol groups: TS, consuming TS, TSX, JS and JSX.
- Eight definition groups: local and cross-file TS, TSX component/use and TS
  imports, JSX component/use and JS imports, a prepared `node_modules` declaration
  package, and a prepared `@types` global declaration.
- Two reference groups: `selam` across TS/TSX and `square` across JS/JSX. Every
  returned reference and positive definition was converted from UTF-16 to
  original UTF-8 bytes and checked against the expected identifier spelling.
- One missing-dependency group and one source-drift group. These demonstrate
  required host incompleteness/staleness handling, not implemented Mirket
  enforcement in this standalone probe.

All four launch combinations passed these **17 groups each** after the probe
was corrected. Earlier attempts are retained. One first assertion incorrectly
expected an unresolved import to return no definition; the servers actually
returned the same-file import alias. Another attempt supplied a string log
verbosity to the native server; its exact API requires a number. The later
contract also corrected configuration timing and parameter omission.

The hostile fixture contained a `tsconfig.json` requesting a local language
plugin and enabled automatic type acquisition, package install/prepare/build
scripts, an executable JavaScript dependency with a side-effect sentinel, and
PATH sentinels for npm/npx/node/tsc/yarn/pnpm/shell/git/make. No sentinel file was
created. Only the selected analyzer process, or wrapper plus its selected
private tsserver child, appeared in sampled process trees. Final process groups
were absent after wait. Process polling and these finite sentinels are not a
complete execution syscall trace; short unknown processes and untested loader
paths are outside this observation. No project code was deliberately executed
as a positive control.

| Launch | Before initialize, KiB | Idle after corpus, KiB | Observed analyzer processes |
| --- | ---: | ---: | ---: |
| Direct TS 5.9.3 | 94,864 | 221,936 | 1 |
| Wrapper + TS 5.9.3 | 64,016 | 303,568 | 2 |
| Wrapper + TS 6.0.3 | 64,544 | 299,904 | 2 |
| Native TS 7.0.2 | 16,464 | 56,416 | 1 |

These are total sampled RSS for the owned process tree in one small corpus run,
after approximately 500 ms awaiting initialization and after two idle seconds.
Sampling targeted 50 ms intervals, with additional `ps` overhead. Disk caches
were not flushed, and the wrapper had not yet spawned tsserver in its
pre-initialize sample. These are neither cold-cache benchmarks, production
memory ceilings nor latency guarantees.

Native `shutdown` returned a valid null result when its parameters were omitted;
`exit` then ended the process with status **1**, stderr `context canceled`.
The process was waited/reaped and its group was gone. Do not claim a clean
zero-status shutdown: integration must qualify this exact native lifecycle and
retain an explicit policy for a known post-shutdown exit status, rather than
blanket accepting analyzer failures.

## Fixed semantic launch and prepared-model contract

A native integration should bind an immutable backend identity and the complete
selected payload, then use exactly:

```text
<absolute private native tsc executable> --lsp --stdio
```

Do not invoke its npm JavaScript dispatcher, PATH `tsc`, shell, npm/npx, workspace
TypeScript, or a fallback server. The current common `LaunchSpec` only supports
no arguments or literal `server`; `--lsp --stdio` requires an explicit finite
native variant. Reuse the existing admission, output/deadline and owned-process
boundaries rather than exposing arbitrary argv.

The effective initialization object exercised successfully was:

```json
{
  "disablePushDiagnostics": true,
  "enableTelemetry": false,
  "logVerbosity": 0,
  "userPreferences": {
    "tsserver": { "automaticTypeAcquisition": { "enabled": false } },
    "customConfigFileName": "mirket-prepared.json",
    "validate": { "enabled": false },
    "suggest": { "autoImports": false }
  }
}
```

Negotiate only UTF-16; advertise no workspace configuration lookup or dynamic
file-watch capability. Reject server requests by default. **One narrow protocol
exception requires implementation and verification:** native 7.0.2 requests
`client/registerCapability` during initialization even when dynamic
configuration was not advertised. The accepted fixture was exactly one
registration with id `typescript-config-watch-id`, method
`workspace/didChangeConfiguration`, and sections
`["js/ts","typescript","javascript","editor"]`. Acknowledge that finite
registration as inert session bookkeeping; do not install a host watcher or
obey arbitrary registration requests. A changed shape should make startup
incomplete. Wait for this configuration handshake before sending `didOpen`.
The native server still answered queries after the generic rejection, but
logged an initialization error; useful locations therefore do not establish
successful initialization.

Omit `params` entirely on `shutdown` and `exit`; native 7.0.2 rejects explicit
`params: null`. Quiet numeric log verbosity is necessary to avoid verbose
project/file logs consuming the finite wire budget. With `0`, the final native
wire contained 22 messages and no `window/logMessage` notification.

The [native launcher](https://github.com/microsoft/TypeScript/blob/v7.0.2/tsc/cmd/tsgo/lsp.go)
has an npm execution callback. The
[preference implementation](https://github.com/microsoft/TypeScript/blob/v7.0.2/tsc/internal/ls/lsutil/userpreferences.go)
gives `tsserver.automaticTypeAcquisition.enabled` precedence over its deprecated
setting. Acquisition must be fixed off by the owner; project
`typeAcquisition.enable: true` must never override it. No arbitrary configuration
updates, plugins, builds, install requests, edits or commands belong to the
navigation interface.

A custom config filename is **not** a filesystem boundary. In an early race,
opening documents before configuration completed caused the native service to
load the ordinary hostile `tsconfig.json` before selecting the prepared model.
A missing custom config can also fall back to ordinary discovery. The integration
must require the exact existing prepared file before launch and use a verified
private project view, or equivalently validate and bind the entire effective
model and dependency graph. Pin all config/extends/project-reference/package
metadata, all source and declaration inputs, default libraries and dependency
closure. Reject escapes, unrecorded ancestor models, mutable dependency links,
unsupported resolution modes and plugin requirements. The experiment exercised
Bundler resolution, ESNext modules, allowJs/checkJs, JSX preserve and explicit
files; it does not qualify every Node/export-map/project-reference form.

Every response must recheck the prepared model and original-source revisions,
including non-returned files, and map only proven original locations. A server
may retain open-document text after disk changes: the drift probe changed a
source declaration and still received a plausible old location. The standalone
probe detected the changed SHA-256 and classified it `Stale`; production must
perform that check in its owner before exposing locations.

Missing packages/types/configuration produce **Incomplete** or a refusal. In all
four candidates, a use of `absent()` imported from an unavailable package
returned the import-alias declaration in the same file. That is a valid local
binding observation, not proof that dependency navigation is complete. Neither
an empty result nor a nonempty alias result proves a complete graph. Coverage
must remain the verified declared corpus; references are not universal
whole-repository references.

## Payload, rights and advisory gates

The exact native platform packages were downloaded, inventoried and statically
inspected; only macOS ARM64 was executed. Each has **113 regular files**,
including its executable, **108 default `.d.ts` files**, `LICENSE`, `NOTICE.txt`,
README and package metadata. Preserve the matching library layout and the full
upstream license/notice bytes. The macOS binary linked only macOS system
libraries/frameworks in `otool -L`; that does not establish the oldest supported
macOS version.

| Native package, version 7.0.2 | Executable bytes | Executable SHA-256 |
| --- | ---: | --- |
| `@typescript/typescript-darwin-arm64` | 23,653,616 | `a82f731365ad69d5c4c15f5e18fba4584bf3b7b839960172a76c3462b5114bf2` |
| `@typescript/typescript-win32-x64` | 24,520,544 | `f9ecfbdc93753d2c972d66a8d0d75f5bd737fd4a5f88b422d9091ea282bcb2c7` |
| `@typescript/typescript-win32-arm64` | 22,956,896 | `2ec832b3d17ce25676fe92b3810ecc0a7580ffb9287d4972bc9a5acc4da2aa50` |

All three notices have SHA-256
`f5c708b59114507b8b27b48181b6883d106bbca0c1634bbee45b5e344237b66b`.
The Apache-2.0 primary package includes additional declaration/data notices and
compiled Go dependency notices: BSD-2-Clause, BSD-3-Clause, MIT, Apache-2.0 and
Go patent terms. Preserve full notices rather than replacing them with one SPDX
label. Confirm the precise linked Go runtime/toolset license closure as part of
admission; the package notice alone is not a verified SBOM.

Static executable build information identifies **Microsoft Go 1.26.4**, the
module revision `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`, and
`vcs.modified=true`. It records the following union of platform-linked modules:
`go-winio 0.6.2`, `go-json-experiment/json
v0.0.0-20260623181947-01eb4420fa68`, `cpuid/v2 2.2.10`, `go-osstat 0.2.7`,
`xxh3 1.1.0`, `x/sync 0.21.0`, `x/sys 0.46.0`, `x/term 0.44.0`,
`x/text 0.38.0`. Resolve how the shipped dirty source revision corresponds to
reviewed tagged source before claiming that tag inspection proves the exact
binary's execution paths.

The exact npm advisory bulk query returned `{}` for all inspected package
versions. A separate OSV query over the compiled Go closure produced matches
for `x/text 0.38.0`:

- [GO-2026-5970](https://pkg.go.dev/vuln/GO-2026-5970): invalid-input normalization
  infinite loop, fixed in 0.39.0.
- [GO-2026-6629](https://pkg.go.dev/vuln/GO-2026-6629): crafted `secure/precis`
  input panic, fixed in 0.41.0.

A correctly normalized `stdlib` version query for **1.26.4** returned **24**
records, including [net/url complexity](https://pkg.go.dev/vuln/GO-2026-6218)
and [Windows `os.Root` junction escape](https://pkg.go.dev/vuln/GO-2026-6604).
An earlier query used `go1.26.4` and returned overly broad results; retain it as
a failed query, not the advisory conclusion. Official Go vulnerability JSON
records and the corrected query are saved in evidence. Matching module versions
is not proof of reachable defects: symbol/call-path/platform analysis has not
been completed. Conversely, npm's empty result does not clear compiled Go code.
**Exact native prebuilt admission stays blocked pending reachability/remediation,
source provenance and complete notices.** Do not silently rebuild or upgrade;
any remediated exact payload needs its own identity and corpus rerun.

## Independently usable TypeScript 5.9.3 parser subset

`typescript@5.9.3` has no registry runtime dependencies. The unchanged
`lib/typescript.js` compiler API can parse supplied in-memory text using only
`createSourceFile` and `forEachChild`. Four TS/TSX/JS/JSX source cases passed;
identifier ranges preserved the original CRLF/Unicode text, and malformed input
produced a parser diagnostic. After loading the trusted library and reading
fixture text, the probe replaced filesystem and child-process methods with
throwing sentinels. Parsing/walking caused **zero** sentinel calls. This proves
the exercised parse path, not every compiler API.

A minimal production selection proposal is exactly:

| Unchanged selected file | Bytes | SHA-256 |
| --- | ---: | --- |
| `lib/typescript.js` | 9,112,572 | `3ae902c92cc44dace175c0e69e13a4b0899f6983c6121d76b9ab8dd5795e7675` |
| `LICENSE.txt` | 9,197 | `a7d00bfd54525bc694b6e32f64c7ebcf5e6b7ae3657be5cc12767bce74654a47` |
| `ThirdPartyNoticeText.txt` | 37,824 | `1af3c68039c57e539422da82a4faada506ce6d0ea6f90e0b699d02dbcdb7a90c` |

Total selected payload: **9,159,593 bytes**, on the already private Node runtime.
The full tarball is 4,377,468 bytes, expanding to 132 files / 23,625,066 bytes;
its SHA-256 and npm integrity are in the task inventory. The package's
[Apache-2.0 license](https://github.com/microsoft/TypeScript/blob/v5.9.3/LICENSE.txt)
and full third-party notices must travel with the selected module. Do not
package npm/npx, the `tsc`/`tsserver` launchers or default type libraries for this
pure source-structure use. Default libraries are needed for a separate semantic
program, not AST parsing.

Bind the absolute trusted module and hash; never resolve a workspace
`typescript` package. Pass an explicit `ScriptKind.TS`, `.TSX`, `.JS` or `.JSX`
and preserve immutable original text. Bound bytes, tree depth, returned items,
output and execution time. Return parse diagnostics/partial or unsupported
constructs explicitly. Convert parser UTF-16 offsets against the exact original
input and recheck its expected revision. This slice may expose structure and
syntactic declaration occurrences; do not label them resolved definitions or
references.

The fixed leaf must not expose `createProgram`, language services, config
parsing/discovery, module resolution, filesystem-backed hosts, emit/transpile,
plugins or commands. Do not execute package scripts or load source imports.
These restrictions preserve the observed passive path; merely installing the
full compiler and accepting arbitrary API calls would not.

## Evidence and remaining release work

The task-state directory retains `qualify.py`, exact corpus/hostile files,
`qualification-results.json`, per-backend wire/memory records, earlier failed
attempts, `cleanup-observation.json`, the network-denial control, package metadata,
tarballs, every package-file hash, primary source captures, native build-info,
advisory queries/responses and `structure-probe.cjs` / `structure-probe.json`.
`evidence-index.json` binds the task-local files available at finalization.

Before semantic activation, implement and exercise the finite launch/handshake,
parameter omission and lifecycle policy in the common owner; verify prepared
model correspondence, dependency closure, revision/stale handling and scope
through actual installed MCP. Complete the compiled-source/advisory/notice
closure first. The parser subset needs its own finite owner and installed
fixture; this experiment did not add that tool.

Native Windows **x64 and ARM64**, Job Object/descendant cleanup, cancellation,
Unicode paths/URIs, prepared dependencies, offline behavior and hostile-config
sentinels remain untested. macOS minimum-version compatibility, actual Codex tool
selection/model turns, installed relocation/tamper checks and release packaging
also remain unverified. No result here qualifies Vue/Angular/Astro source maps,
framework plugins, build preparation, broad diagnostics or whole-project
semantic completeness.
