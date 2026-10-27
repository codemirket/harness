# TypeScript in-memory semantic qualification

Qualification date: **2026-10-25**, macOS **26.5.2 ARM64**. This is a private
extension experiment around the existing `adapters/src/code-navigation.ts`
engine and its selected-source contract. No shared source, dependency lock,
release inventory, installed payload or client configuration was changed by
this experiment.

## Decision

**The existing TypeScript 5.9.3 in-memory language service is viable as the sole
TypeScript/JavaScript engine for a bounded, explicitly prepared dependency
graph.** Both the complete selected standard-library case and the case with
no default libraries passed the earlier **17-group** semantic corpus. There
were zero filesystem, process, network, TypeScript-system or module-loader trap
hits during semantic work, zero unexpected module bindings, no discovery
callbacks, and no hostile sentinel files.

The no-default-library case needs only **10 snapshots / 1,003 original text
bytes / 216 AST nodes** for the full original corpus and both dependency
declarations. It extends the already selected compiler module and host; it needs
no native Go rebuild, tsserver protocol, LSP wrapper or second semantic engine.
The current eight-source schema still needs an explicit bounded extension for
selected declarations and import bindings. This experiment does not activate
that extension.

Keep `model_complete: false`. Passing selected definitions/references does not
establish complete project resolution, standard types or whole-repository
references. The missing standard-type counterexample below is a real coverage
loss, not a failed positive test hidden from the result.

## Exact engine and model

The unchanged selected `typescript@5.9.3` `lib/typescript.js` is **9,112,572
bytes**, SHA-256
`3ae902c92cc44dace175c0e69e13a4b0899f6983c6121d76b9ab8dd5795e7675`.
Its existing full npm archive SHA-256 is
`10e108c9cf7d5f2879053dff18515fb405abf2ccef63eaaf017d9c571687a1d3`.
The already private Node is **24.22.1**, executable SHA-256
`6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`.
No package manager, build, install hook or upstream helper ran.

The host follows the existing owner rather than introducing another service:

- Immutable selected texts and their SHA-256s become `ScriptSnapshot`s. The
  original eight project files and two prepared declaration files retain the
  prior corpus's CRLF, Turkish text, astral prefix and original bytes.
- Every filename exposed to TypeScript is an opaque synthetic
  `/mirket/source-N.<kind>` or `/mirket/declaration-N.d.ts`, with a fixed mapping
  back to the selected original. No actual project/dependency path reaches the
  compiler host.
- Compiler options keep `noLib: true`, `noResolve: true`, `types: []`,
  `allowJs`, `checkJs`, preserved JSX and ESNext modules. The experiment fixes
  target ES2022 and Bundler mode to match the prior corpus's prepared options.
- The existing explicit module-resolution callback is extended with a lookup
  table keyed by **selected source ID plus literal module specifier**. Its four
  entries bind `main.ts`/`Widget.tsx` → `a.ts`, `View.jsx` → `logic.js`, and the
  prepared package import → its exact selected declaration. A separate selected
  declaration supplies `FixtureGlobal`; no type-root search is performed.
- The two deliberately missing imports are explicitly identified as absent.
  Unknown edges never trigger fallback resolution. No project reference,
  config/extends/package lookup, automatic type acquisition, plugin, source
  execution, emit or command API is provided.
- One language service/document registry is created and disposed for each
  operation, as in the existing owner. This probe sends several operations to
  one Node process for the corpus/resource observation; it does not establish a
  production warm-cache lifecycle.

The pinned [language-service implementation](https://github.com/microsoft/TypeScript/blob/v5.9.3/src/services/services.ts)
and [host API](https://github.com/microsoft/TypeScript/blob/v5.9.3/src/services/types.ts)
support explicit snapshot and resolution callbacks. Synthetic names matter:
the pinned compiler's `getImpliedNodeFormatForFileWorker` can inspect package
scope for filenames containing `node_modules` even with a custom resolver.
The experiment's strict virtual-host callbacks would fail on any unselected
file or directory request; none occurred.

## Corpus results and refusal evidence

Each finalized case passed:

| Group | Verified outcome |
| --- | --- |
| Five symbol groups | The expected original TS, consuming TS, TSX, JS and JSX declarations are present. Every returned name span is mapped to original UTF-8 boundaries; identifier spellings are checked. |
| Eight definition groups | Local and cross-file TS, TSX and JSX bindings resolve to the hand-selected original declarations. Prepared package and prepared global-type definitions map to their exact selected `.d.ts` files. |
| Two reference groups | `selam` returns exactly six identifier locations in `a.ts`, `main.ts` and `Widget.tsx`; `square` returns exactly three in `logic.js` and `View.jsx`. Every spelling and original byte span is checked. |
| Missing dependency | The service's same-file alias result carries `failedAliasResolution`; the inherited guard refuses it as `unresolved_import`, with the manifest reporting the missing edge. No alias is presented as a resolved dependency declaration. |
| Saved-source drift | The parent changes `a.ts` after snapshots are frozen. A current-revision mismatch discards the query before another service is created. The original fixture is restored afterward. |

The prior server corpus's two additional document openings were also mirrored.
The resulting process creates/disposes **18** services for the 16 semantic and
missing-dependency groups plus those supplementary openings; the seventeenth
group is the parent-side drift refusal and creates no service.

After loading only the exact trusted compiler and explicit manifest, throwing
traps replaced filesystem and promise-filesystem functions, child-process APIs,
DNS/network entry points, TypeScript `sys` functions, worker construction and
subsequent CommonJS module loading. Five deliberate controls confirmed the
filesystem/process/network/system/loader traps throw; their calls are recorded
separately from the **zero** calls made by the semantic corpus.

Every observed host read named a selected synthetic snapshot. The compiler
called neither the host's `fileExists`/`readFile` nor its directory callbacks.
The same hostile config, enabled type-acquisition request, plugin, executable
dependency, package scripts and PATH sentinels were present on disk and absent
from the selected model. A `sandbox-exec` profile also denied networking. These
are finite interception/process observations, not a syscall trace or a general
JavaScript sandbox.

The initial harness acknowledged exit but retained its parent-owned stdin pipe;
two lifecycle attempts required termination. They are retained as failed
attempts. The final harness closes that owned input after the acknowledgement;
both final cases exited **0**, were waited/reaped, and had no remaining process
group. No blanket analyzer-error acceptance was added.

## Default-library bounds and actual lost coverage

The full case follows the exact pinned `lib.es2022.full.d.ts` reference closure:
**63** unchanged default-library snapshots, **2,389,540 bytes** and **124,589 AST
nodes**, including the DOM declarations. All are supplied explicitly as roots;
`noLib`/`noResolve` prevent discovery. Their individual hashes and original
reference closure are retained in the selected-file inventory.

| Prepared model | Snapshots | Text bytes | AST nodes | Maximum AST depth |
| --- | ---: | ---: | ---: | ---: |
| Original corpus + two declarations; no default libraries | 10 | 1,003 | 216 | 7 |
| Same corpus + exact ES2022 full library closure | 73 | 2,390,543 | 124,805 | 12 |

The full case exceeds the existing 8-source, 1 MiB original-text, 256 KiB
per-source, 32,768-node and 2 MiB leaf-request budgets. It is not an honest drop-in
change. The no-default case fits the byte/node/depth/transport budgets, but still
needs a finite selected-declaration count and import-edge schema beyond the
current eight undifferentiated sources. No existing bound was lifted in shared
source for this experiment.

Removing **all** default libraries is a coherent coverage boundary, not a
per-symbol minimization. It preserves the required explicitly declared corpus
bindings, but standard-library/DOM APIs are absent unless declarations are
explicitly selected. A discriminating query on the existing `kişi.length`
expression proves the difference:

- Full closure: `length` resolves to pinned `lib.es5.d.ts`, original bytes
  **21,174–21,180**.
- No libraries: the same query refuses as `unresolved_symbol`; type diagnostics
  report missing `String` and that `length` does not exist on `string`.

The no-library checker additionally reports missing global `Array`, `Boolean`,
`Function`, `IArguments`, `Number`, `Object` and `RegExp`. Thus primitive-member,
array/callable built-in, Promise/iterator and DOM type relationships cannot be
advertised from this model. Promise/iterator/DOM examples were not separately
exercised here; their declaration inputs are absent. These limits can affect
inference as well as navigation to library declarations. Compiler-option
diagnostic 5055 also reflects the inherited `allowJs` options without `noEmit`;
no emit operation or write callback is exposed and no file was written.

## Process RSS comparison

These are one-run sampled RSS observations in KiB on the same Mac/private Node
as the earlier experiment. Samples target 50 ms intervals, with `ps` overhead.
The new pre-query sample follows compiler/manifest bootstrap and a 500 ms pause;
the final sample follows the corpus and two idle seconds. The auxiliary
`length`/diagnostic probes run **after** the reported idle sample.

| Engine/model | Before initialize/queries | After corpus | Observed analyzer processes |
| --- | ---: | ---: | ---: |
| Prior direct tsserver 5.9.3 | 94,864 | 221,936 | 1 |
| Prior native TypeScript 7.0.2 | 16,464 | 56,416 | 1 |
| In-memory 5.9.3, full explicit libraries | 247,488 | 678,800 | 1 |
| In-memory 5.9.3, no default libraries | 239,040 | 260,416 | 1 |

The in-memory prototype is **not a measured memory improvement over tsserver**.
Its benefit is the simpler already-owned API and explicit input boundary. The
new process repeatedly creates/disposes programs, while the prior servers
retain their initialized projects; bootstrap instrumentation/model loading also
differ. V8 heap/RSS reclamation was not forced. No cold-cache, p95, throughput,
production memory ceiling or minimum-RAM claim follows from these samples.

## Integration conditions and remaining limits

Extend the existing selected-source owner with one finite schema: explicit
immutable source/declaration snapshots and a duplicate-free, source-specific
import binding table. Keep synthetic IDs and require each resolved target to
belong to the selected model. Reject missing/ambiguous edges, unsupported
resolution forms, unselected targets and failed/unverified alias resolution.
Bind the graph, options and every original source/declaration revision; after
worker cleanup, recheck all selected saved inputs, including non-returned ones.

This does not implement a trusted project preparer. The host must establish
that a provided edge identifies the intended package/export/declaration and
that required model inputs are present. Exact lookup tables can faithfully
resolve an incorrectly prepared graph. Relative paths, package exports,
conditions, Node16/NodeNext, path aliases, triple-slash references, project
references, JavaScript package metadata and framework transforms need explicit
preparation/qualification or refusal; this corpus does not prove those forms.

Keep the existing original-span validation, whole-model stale suppression,
finite location/output budgets, `model_complete:false`, owned process deadline,
cancellation and cleanup. No API should accept arbitrary compiler options,
module callbacks or `typescript` module paths. Default-library scale is a
separate asset/transport/resource decision, not an automatic budget increase.

The experiment is source-level API evidence only. Installed MCP/worker schema,
host revalidation, tamper/relocation, cancellation and failure propagation must
be verified after implementation. Windows x64/ARM64, macOS 15, framework source
maps, broad diagnostics and actual Codex Desktop activation are untested here.

Private evidence is retained in
`~/.codex/task-state/harness-rewrite-implementation/typescript-in-memory-semantic-qualification-20261025/`:
the frozen existing owner capture, exact manifests/inventories, original/hostile
fixtures, `semantic-probe.cjs`, `run_probe.py`, per-case results/wire/RSS records,
failed lifecycle attempts, verification and `evidence-index.json`. New bytes
remain below the 250 MB budget. The earlier server/native evidence is reused
without duplicating its payloads.
