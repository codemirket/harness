# Bash and PowerShell navigation qualification

Observed **2026-10-25**, for the Codex-only Mirket 0.0.1 rewrite. This is a
qualification report, not dependency admission or a production capability claim.
Only this document was added. Production source, lockfiles, release payloads,
user installations, client configuration and the old harness were not changed.

## Decision

**Neither tested backend qualifies for general binding-complete definitions and
references.** Both run offline and return useful original-source locations, but
discriminating fixtures expose incorrect or overbroad bindings. Bash definitions
have a useful narrower result; its references and PowerShell's cross-module
results must remain explicitly incomplete source candidates until their actual
limits are represented and tested by the Mirket owner.

| Candidate | Useful observed capability | Release gate still open |
| --- | --- | --- |
| [bash-language-server 5.8.1](https://registry.npmjs.org/bash-language-server/5.8.1), MIT, existing private Node direction | Literal sourced-file function/variable definitions, local versus global variable definitions, and source occurrences across a prepared graph. | References merge shadowed variables; dynamic/missing sources produce empty results requiring explicit incomplete/refused outcomes; exact bundled WASM notices and integration/platform qualification remain open. |
| [PowerShell Editor Services 4.7.0](https://github.com/PowerShell/PowerShellEditorServices/releases/tag/v4.7.0) with [PowerShell 7.6.8](https://github.com/PowerShell/PowerShell/releases/tag/v7.6.8), MIT plus runtime/component notices | Static function declarations and same-spelling source occurrences across supplied `.ps1`/`.psm1` files, without importing the project modules. | Local scope definitions can be wrong; references merge different bindings; an unimported module is a definition candidate; exact component/notice reconciliation and native Windows qualification remain open. |

An editor extension wrapping these same servers is not an independent alternative.
ShellCheck is a diagnostics owner, and direct Tree-sitter or PowerShell AST parsing
supplies syntax and occurrences; neither is a replacement cross-file binding
engine. No second resolver or fallback stack was implemented to conceal these
failures. The existing structural inspectors and five SQL profiles are unaffected.

## Exact payloads and provenance

Latest stable metadata was retrieved from the primary npm registry and official
GitHub release APIs. PSES 4.7.0 was published 2026-06-25; PowerShell 7.6.8 was
published 2026-10-22. These are observations on the research date, not a floating
version policy. All commands used disposable private state. npm extraction used
`--ignore-scripts --no-audit --no-fund`; no upstream install/build helper ran.

| Observed archive | SHA-256 |
| --- | --- |
| [bash-language-server-5.8.1.tgz](https://registry.npmjs.org/bash-language-server/-/bash-language-server-5.8.1.tgz) | `2ae910de8c50f4ec148222bdc580c121f2c621bdacc1225716218ca700f8ec79` |
| [Bash server source, server-5.8.1](https://api.github.com/repos/bash-lsp/bash-language-server/tarball/server-5.8.1) | `7df1436bc278b1450a69db2e111f37da0be9ecaf8c0911433082ba135acc9066` |
| [PowerShellEditorServices.zip](https://github.com/PowerShell/PowerShellEditorServices/releases/download/v4.7.0/PowerShellEditorServices.zip) | `5084f0326cc88539e9d880b08f6c52ce5be4672bb8ecee8921e35f8895a6b9d7` |
| [PSES source, v4.7.0](https://api.github.com/repos/PowerShell/PowerShellEditorServices/tarball/v4.7.0) | `dd5692c696bf7407d3735fe372a7ac85bf04c8714d012996bd69383a07c4cc53` |
| [powershell-7.6.8-osx-arm64.tar.gz](https://github.com/PowerShell/PowerShell/releases/download/v7.6.8/powershell-7.6.8-osx-arm64.tar.gz) | `c16a3f433966c6fd5a9c8a660a95c92cb5dbc4b1813416501d1ba30ca160da32` |
| [PowerShell source, v7.6.8](https://api.github.com/repos/PowerShell/PowerShell/tarball/v7.6.8) | `5acd0b0fcc27af8bf73591cbabe3b655e5dc02b0a7e5d5106760996e61bed09f` |

The npm metadata records `gitHead` `dd90f3d2c10ddce889b456a9ac4a1d0f079a9558`.
The fetched source tag's archive root identifies `3c0f67a`; every file under its
`server/src` matches the npm tarball's included `src` byte for byte. Qualification
inspected the shipped npm source, rather than assuming that tag identity implied
binary correspondence.

The Bash tarball's `parser.info` records Tree-sitter Bash source
[`a06c2e4415e9bc0346c6b86d401879ffb44058f7`](https://github.com/tree-sitter/tree-sitter-bash/tree/a06c2e4415e9bc0346c6b86d401879ffb44058f7),
Tree-sitter CLI 0.27.0 and parser ABI 15. The packaged grammar WASM SHA-256 is
`1c14dc240687e25a1cd8a11ae5135a6c780986d724c488d74d3dd83c20954511`.
Its exact grammar source archive was captured, SHA-256
`71f092ac81d0b63380ac3caa623e04d9de3156a969f8e041159476dc9e9543ff`.
The grammar's [MIT license](https://github.com/tree-sitter/tree-sitter-bash/blob/a06c2e4415e9bc0346c6b86d401879ffb44058f7/LICENSE)
must accompany distribution; the server's top-level MIT declaration is not its
substitute. The npm server tarball itself contains no license file; the server
[repository license](https://github.com/bash-lsp/bash-language-server/blob/server-5.8.1/LICENSE)
was separately captured.

## Passive finite corpus

Execution platform was **macOS 26.5.2 (25F84), Darwin 25.5.0, ARM64**. The explicitly
selected development Node was 24.18.1, outside the user's Mirket installation.
PowerShell was the downloaded private ARM64 archive; `pwsh --version`/the LSP
response identify 7.6.8. Its bundled runtime configuration identifies
`Microsoft.NETCore.App` **10.0.13**, `net10.0`. No SDK or C# analyzer was installed.

The client used stdio LSP with explicit absolute executable/script paths,
private homes/caches/logs, a PATH containing only task-owned failing sentinels,
and macOS `sandbox-exec` network denial. For Bash, a task-owned Node preload
recorded and rejected child-process attempts. Backend files were not modified.
No actual host executable was available through PATH. This establishes the
observed finite route, not a portable OS sandbox or proof about unrequested LSP
methods.

Initial fixtures contained eight Bash and ten PowerShell source/configuration
files. Additional failure probes introduced shadowing, unrelated modules and
unnotified saved changes. Source text included Turkish letters, an astral emoji,
CRLF and paths containing spaces/Unicode. Hand-selected expected declarations
were checked independently of the server; returned UTF-16 locations were converted
against exact source snapshots and their UTF-8 slices compared with expected
original text.

| Probe | Bash 5.8.1 | PSES 4.7.0 / PowerShell 7.6.8 |
| --- | --- | --- |
| Cross-file function definition | `source "./lib.sh"; greet` returns the original function declaration in `lib.sh`. | An explicit module-use fixture returns `Get-Çay` in `Tea.psm1` without importing it. |
| Cross-file function references | Returns the original declaration and call in the literal source graph. | Returns the original declaration and call in the module-use fixture. |
| Variable definition | Imported global resolves to `lib.sh`; separate local/global definitions resolve to the appropriate assignments. | Simple unique variable resolves; the shadowing case below fails. |
| Unicode/CRLF | An occurrence after an emoji maps exactly to `value` at UTF-16 columns 20–25, UTF-8 bytes 22–27 of that line. Definition locations cover the entire declaration, not necessarily only its name. | `Get-Çay` and `$Ad` spans map to exact original text; the `$Ad` query also exposes unrelated matches below. |
| Missing source/module metadata | Missing literal source returns no definition and an explicit source-resolution diagnostic. | Missing module/command returns no definition. Empty results alone do not identify why binding is unavailable. |
| Dynamic command/source | Command-substitution source, including a dynamic directory prefix, returns no definition; no source code executes. | Dynamic invocation/`Invoke-Expression` input yields no proven definition; no source code executes. |
| Ambiguous declarations | Two sourced `choice` functions return both alternatives. | Two same-named module functions return both alternatives. |
| Changed in-memory dependency | Replacing the function through `didChange` removes the old result. | Replacing the original function removes that declaration; another same-named unrelated module then becomes the result. |
| Saved dependency changed without notification | Returns the cached old declaration. | Returns the cached old declaration. Neither backend provides Mirket's saved-revision guarantee. |

The first Bash attempt returned early nulls because the probe had not serviced
`workspace/configuration` while the server awaited initialization. Correcting the
client's request handling produced the positive results above. An elapsed delay
alone did not repair that protocol mistake. A production readiness barrier must
complete fixed configuration before admitting document operations.

### Discriminating counterexamples

For Bash:

```bash
item=global
outer() { local item=local; printf "%s" "$item"; }
printf "%s" "$item"
```

A references query on the global declaration returns all four occurrences,
including the different local declaration and use. Definitions on the local use
and final global use correctly distinguish the assignments. Thus a positive
cross-file references example is insufficient for a binding-complete reference
claim. The shipped `onReferences` calls `findOccurrences(uri, word)`; its result
is consistent with source-name occurrence coverage.

For PowerShell:

```powershell
$item = "global"
function Get-Local { $item = "local"; $item }
$item
```

Global references include all four occurrences. Definition on the local use
returns the outer script assignment. In a separate fixture, `Get-Lonely` returns
a declaration in `Lonely.psm1` even though the calling script never imports that
module. A unique candidate therefore does not prove the runtime/import binding.
References to a script's `$Ad` additionally include the same-spelled parameter
and use in `Tea.psm1`. The pinned
[`SymbolsService`](https://github.com/PowerShell/PowerShellEditorServices/blob/v4.7.0/src/PowerShellEditorServices/Services/Symbols/SymbolsService.cs)
scans workspace identifiers and possible declarations; this matches the observed
limits. These results must not be silently relabeled complete semantics.

### Execution sentinels and fixed policy

No task-owned execution marker, PATH sentinel hit or Bash child-process attempt
was observed. The corpus included top-level project writes, a sourced-file
command-substitution write, PowerShell profile writes, project/module writes,
custom analyzer rules and an autoload module. A second module outside the
workspace but explicitly present in the private qualification `PSModulePath`
was not imported by a definition query; its command returned no definition.
These observations cover these requests and expected execution routes only.

The service/runtime's own approved code is executable backend code. Project
scripts, modules, profiles and analyzer rules were source inputs and remained
unexecuted. PSES startup necessarily imports its own private service module.

The fixed Bash configuration was:

```json
{
  "shellcheckPath": "",
  "shfmt": { "path": "", "ignoreEditorconfig": true },
  "explainshellEndpoint": "",
  "backgroundAnalysisMaxFiles": 0,
  "includeAllWorkspaceSymbols": false,
  "enableSourceErrorDiagnostics": true
}
```

A nonempty private PATH is required by the server even for this passive route.
It must contain no user/host fallback. Source inspection found that hover may
invoke Bash/man and completion may run the bundled `get-options.sh`; neither was
admitted or tested as a passive operation. Disabling ShellCheck and shfmt does
not disable those separate routes. Forward only approved request methods.

PSES launched private `pwsh` with `-NoLogo -NoProfile -NonInteractive`, the private
`Start-EditorServices.ps1`, `-Stdio -LanguageServiceOnly`, explicit private log,
session and bundled-module paths, and no console/debugger/additional modules.
Initialization fixed `enableProfileLoading=false`, `shellIntegrationScript=""`
and the private initial working directory. Server settings fixed
`enableProfileLoading=false`, `scriptAnalysis.enable=false`, no settings path,
and disabled references code lenses before opening any project file.

The pinned
[`PsesLanguageServer`](https://github.com/PowerShell/PowerShellEditorServices/blob/v4.7.0/src/PowerShellEditorServices/Server/PsesLanguageServer.cs)
defaults its internal runspace profile loading to **true** when the initialization
option is absent. Outer `pwsh -NoProfile` is insufficient. Script analysis also
defaults on. Reject project settings that change these choices, the executable,
module roots, integration script or analyzer rule policy. Arbitrary execute,
completion, hover, debugging, REPL and formatting methods remain unqualified.

## Archive, source and notice closure

A disposable npm lock resolved **36 packages**. All 36 tarballs were captured and
checked against their lockfile SHA-512 integrities; none declared an install
script. Their exact versions are:

```text
@mixmark-io/domino 2.2.0; @nodelib/fs.scandir 2.1.5;
@nodelib/fs.stat 2.0.5; @nodelib/fs.walk 1.2.8; @one-ini/wasm 0.2.1;
balanced-match 4.0.4; bash-language-server 5.8.1; brace-expansion 5.0.12;
braces 3.0.3; commander 14.0.3; editorconfig 3.0.2; fast-glob 3.3.3;
fastq 1.20.3; fill-range 7.1.1; fuzzy-search 3.2.1; glob-parent 5.1.2;
is-extglob 2.1.1; is-glob 4.0.3; is-number 7.0.0; merge2 1.4.1;
micromatch 4.0.8; minimatch 10.2.6; picomatch 2.3.2;
queue-microtask 1.2.3; reusify 1.1.0; run-parallel 1.2.0; semver 7.8.5;
to-regex-range 5.0.1; turndown 7.2.4; vscode-jsonrpc 9.0.2;
vscode-languageserver 10.1.1; vscode-languageserver-protocol 3.18.3;
vscode-languageserver-textdocument 1.0.14; vscode-languageserver-types 3.18.3;
web-tree-sitter 0.27.0; zod 3.25.76
```

Registry declarations are 30 MIT, four ISC, one BSD-2-Clause and one
BlueOak-1.0.0. The full evidence inventory records each archive URL, version,
license declaration, integrity, SHA-256 and hook flag in `bash-closure.json`
(SHA-256 `a79c2e8973549031bba53a17fe22411d91943acdc76ecf0cb7c6037d043005b0`).
Those declarations are not by themselves complete linked-code notice evidence.

In particular, `editorconfig` includes `@one-ini/wasm` even though formatting is
disabled. Its exact source commit is
[`a9374aa5b8a193223f6248bee100f8207e69388f`](https://github.com/one-ini/core/tree/a9374aa5b8a193223f6248bee100f8207e69388f),
and the captured source archive SHA-256 is
`734086d868971430dd287fc676331e0200277aeceb1016571087751c61b8a0f1`.
The source has Rust dependencies, including Pest, Serde, wasm-bindgen and
wee_alloc, but no `Cargo.lock`; the npm package has its own MIT file without a
linked-crate inventory. Exact linked revisions and applicable notices therefore
remain unresolved. The Web Tree-sitter runtime WASM's linked runtime/build
closure must likewise be recorded before admission. A package-level MIT label
cannot close these embedded binaries. No production dependency batch is proposed
as already qualified by this report.

PSES includes `LICENSE`, `NOTICE.txt`, `.deps.json` manifests and bundled
PSScriptAnalyzer 1.25.0 with its own license/notices. The PSES notice SHA-256 is
`6dafb4e4b76d2979900369c92a012cc19a5ac4bf13b56f2655a3e0f2b635bfec`.
Its notices name Microsoft.Extensions components 8.0.0 and Newtonsoft.Json
13.0.3, while shipped dependency manifests identify 10.0.5 and 13.0.4.
This does not establish a license violation; it establishes that the exact
shipped assembly/package/source/notice mapping has not been closed. Build-only
manifest entries must be distinguished from actual shipped runtime assemblies.
Retain and supplement the upstream notices using exact package evidence before
redistribution; never infer a full closure from the root MIT license.

The PowerShell archive includes `LICENSE.txt`, `ThirdPartyNotices.txt`, module
notices, runtime `.deps.json` and a self-contained .NET runtime. Its main
third-party notice SHA-256 is
`275443457d6f9eac61ef2367da89b1420b94f68c1cc94ed305d2591514ea88d1`.
These files were preserved. Exact .NET/native/module component sources and
platform-specific notice reconciliation remain packaging gates. Excluding .NET
languages does not remove the necessary PowerShell runtime dependency.

As extracted, the npm payload occupies 21,796,903 file bytes, PSES 312,529,239,
and the macOS PowerShell payload 201,133,098. These are raw source/archive
extractions including unused components and symbols, not measured installed
release sizes or memory/latency benchmarks.

## Platform and integration boundary

| Platform | Evidence | Remaining limit |
| --- | --- | --- |
| macOS ARM64 | Exact Bash package on explicit Node 24.18.1 and native `pwsh` 7.6.8/PSES 4.7.0 executed through the finite offline corpus. `file` identifies ARM64 `pwsh`. | No oldest-macOS, signed/relocated installed release or actual Codex activation test. |
| Windows x64 | Bash JavaScript/WASM has no platform-specific npm native dependency. Official PowerShell 7.6.8 ZIP asset exists. PSES supplies common managed payloads. | No native execution, private Node closure, filesystem/URI/CRLF/process ownership or packaging qualification. |
| Windows ARM64 | Same Bash JavaScript/WASM direction; official native PowerShell 7.6.8 ARM64 ZIP asset exists. | No native execution or proof that every service/native transitive component works on ARM64; no x64 emulation assumption. |

Official release metadata reports SHA-256
`b9f074f923c499b51ec99d313da8e9ae90d17a9f3adf00c73d6189f34d908528`
for [PowerShell-7.6.8-win-x64.zip](https://github.com/PowerShell/PowerShell/releases/download/v7.6.8/PowerShell-7.6.8-win-x64.zip),
and `e889c9bfa6c3a9796bd6268cbeeb389e3968f77c6db46cb282178274d71c160b`
for [PowerShell-7.6.8-win-arm64.zip](https://github.com/PowerShell/PowerShell/releases/download/v7.6.8/PowerShell-7.6.8-win-arm64.zip).
Those two ZIPs were not downloaded/executed here; asset metadata is not platform
qualification or a closed Windows dependency inventory.

A future bounded integration must reuse the common connection-owned analyzer
lifecycle and enforce all of the following before exposing results:

1. Bind the exact approved runtime/server/grammar bytes and complete license/
   source inventory. Use explicit private paths and a fixed environment; no
   executable, module, configuration, network or package-manager fallback.
2. Complete fixed configuration before document admission. Present an immutable,
   finite in-scope source/model snapshot; reject escaped source paths/symlinks
   and implicit host modules. Treat missing/dynamic source edges as incomplete
   metadata, not a successful empty binding set.
3. Admit only tested methods. For Bash, a conservatively bounded definition
   operation may use literal sourced files and verified local/global declaration
   results; reference results remain name occurrences unless an independently
   tested boundary proves the requested binding. PSES results remain possible
   source declarations/occurrences with the documented scope/import failures.
   A singleton result does not upgrade their certainty.
4. Convert UTF-16 positions against each exact original file to UTF-8 byte ranges;
   preserve whole-declaration ranges when the backend supplies them. Reject
   out-of-scope, malformed or stale correspondence. Do not fabricate identifier
   ranges by string search inside a larger declaration.
5. Include backend/version, source/model revisions, coverage, missing edges,
   ambiguity and truncation in the response. Rehash every participating saved
   input before returning current results. Retire/rebuild a session after unknown
   dependency changes; LSP file caches are not revision verification.
6. Qualify those actual wrappers through development and disposable installed MCP
   journeys, cancellation/queue/output/deadline/process cleanup, relocation and
   native platform/Codex checks. This experiment did not implement that wrapper.

The reproducible scripts, raw responses, exact source snapshots, downloaded
archives, notices and verification summary remain outside shipped source at
`~/.codex/task-state/harness-rewrite-implementation/shell-navigation-qualification-20261025/`.
`probe.py` regenerates the finite initial corpus, `extra-probe.py` reproduces the
counterexamples, and `verify-results.py` checks expected original text, known
failure results and zero observed sentinels. The final summary records that
production admission is **not qualified**. No broad support claim follows from
successful initialization, a parser load, or these useful positive examples.
