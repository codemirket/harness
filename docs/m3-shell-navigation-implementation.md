# Bounded shell navigation implementation checkpoint

Observed **2026-10-25** for the Codex-only Mirket 0.0.1 rewrite on macOS 26.5.2,
ARM64. This checkpoint applies the selected-source boundary to the exact private
payloads in [the shell qualification](m3-shell-navigation-qualification.md).
It does not admit dependencies or add a production navigation capability.

## Decision

**No shell navigation owner is implemented at this checkpoint.** The tested
payloads still need a source-loading boundary and a shell-specific launch and
configuration barrier. A post-response location check cannot prevent a backend
from reading an unselected file before it returns that location. Marking such a
result incomplete describes binding uncertainty but does not repair source scope.

The existing shell syntax owner remains the qualified structural route. No stub
`shell_navigation` module, public registration, alternate resolver, release
payload, dependency, installation or client setting was added.

## Discriminating selected-source fixtures

The fixture supplies only `main.sh` or `main.ps1` through `didOpen`. Each selected
root is disposable and separate from a task-owned unselected directory. Source
files include CRLF, Unicode paths and an emoji comment. The unselected file has
a distinct declaration and a top-level write sentinel. It is never opened or
sent as source text by the client.

| Exact backend | Controlled trigger | Observed result |
| --- | --- | --- |
| bash-language-server 5.8.1, explicit Node 24.18.1 | `source "/absolute/task-owned/unselected/private.sh"` followed by `outside_selection` | Definition and references include the unselected file. Definition covers its original `outside_selection() { :; }` declaration. |
| PSES 4.7.0, private PowerShell 7.6.8 | A workspace symlink points to the task-owned unselected directory; `main.ps1` calls `Get-OutsideSelection`. | Under the qualification's fixed settings, workspace scanning follows the symlink and returns the unopened file's declaration and reference. Its returned URI retains the workspace symlink path; canonical resolution reaches the unselected directory. |

The PowerShell result establishes workspace discovery through this symlink. It
does **not** establish that the absolute dot-source statement also present in
the fixture caused a read. Its script/module code did not execute.

An independent verifier rehashes both saved inputs, resolves the returned file
URIs and converts the exact UTF-16 ranges against original UTF-8 bytes. Bash's
definition is bytes **13–39** of the unselected file; PowerShell's identifier is
bytes **22–42**. The verifier also checks an unselected reference for each backend.
This discriminates source discovery from a guessed name or a declaration copied
from the selected caller.

No project-write marker, private PATH sentinel hit or Bash child-process attempt
was observed. Launch reused the earlier passive settings, explicit paths,
private homes/caches/logs, child-process rejection preload for Bash and macOS
network-denying `sandbox-exec` profile. Those observations cover these finite
requests, not a portable sandbox or unrequested LSP methods. The direct client
children were closed through the qualification client's shutdown/wait route;
this checkpoint does not qualify a new Rust cancellation owner.

## Owning implementation gaps

`src/lsp_client.rs` currently sends the original project root to the backend.
`PreparedProject` captures only declared files and validates returned locations,
but it does not constrain server filesystem reads to those files. The Bash
server's pinned `ensureUrisAreAnalyzed` explicitly reads reachable files not
already analyzed, including files outside the workspace. PSES's pinned
`ScanWorkspacePSFiles` enumerates workspace files when
`AnalyzeOpenDocumentsOnly` is false; the tested policy uses that value.

The shared launch owner also does not yet express either exact shell startup:

- Bash needs the literal `start` argument, a nonempty private PATH without host
  executables, the fixed passive settings and a serviced `workspace/configuration`
  barrier before opening documents. The shared owner accepts bare Node scripts,
  clears PATH and refuses server configuration requests.
- PSES needs the exact private PowerShell/service-module arguments, `-NoProfile`,
  `-LanguageServiceOnly`, private module roots, profile loading disabled in the
  initialization options and script analysis disabled before source admission.
  Its launch cannot be represented by the existing finite invocation variants.

A future implementation can investigate a fresh projection containing only
selected snapshots and an independently enforced source-read policy for Bash.
For PSES, `AnalyzeOpenDocumentsOnly=true` and a projection without symlinks are
plausible narrower controls, but they were not qualified here. A projected tree
alone does not stop Bash's absolute source reads. Disabling tool execution,
rejecting returned URIs or matching keywords in source text cannot substitute
for the required loading boundary. No custom filesystem shim, source rewrite or
second resolver was introduced to conceal the failure.

The binding limitations from the earlier qualification also remain: Bash
references merge shadowed bindings; PSES definitions and references can select
different scopes or unimported modules. A future source-candidate response must
retain explicit incomplete/refused status, backend and policy identity,
source/model revisions, original spans, missing edges, ambiguity and truncation.
Exact source/notice closure and native platform/release journeys remain separate
gates even after a process owner exists.

## Verification and retained evidence

The exact native finite probe and independent result verifier passed. No Rust
source changed, so this checkpoint did not run or contend with the integration
owner's Cargo suite. No Windows, oldest-macOS, installed MCP or Codex activation
claim follows from these native development requests.

Evidence is private and outside shipped source:

`~/.codex/task-state/harness-rewrite-implementation/shell-navigation-implementation-20261025/`

`selected-source-probe.py` reuses the preserved qualification client with new
disposable state and unchanged runtime payloads. `selected-source-summary.json`
records runtime/platform and exact saved-input hashes; per-backend result and
transcript files retain the original responses. `verify-selected-source.py` and
`selected-source-verification.json` establish the original-byte correspondence
and finite sentinel observations. The initial probe verifier assumed every LSP
definition result was an array; accepting the protocol's single-Location form
corrected that harness error before the successful run.
