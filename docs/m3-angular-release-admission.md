# M3 Angular finite snapshot owner and admission boundary

Observed **2026-10-25** on **macOS 26.5.2 ARM64**, explicit private
**Node 24.22.1**. This advances the [framework qualification](m3-framework-navigation-qualification.md)
into an isolated, exercised source owner. **It is not release admission or a
public tool.** No shared runtime, release/distribution inventory, installed home,
client settings or old harness was changed.

## Result and source owners

A **single Angular language-server process** provides useful external/inline
template definitions, class-property/template references and HTML template
symbols with the pinned Angular 22.2.3/TypeScript 6.0.3 pair. No companion
TypeScript server is required for these cases. Ordinary TS definitions remain
outside this owner's qualified operation, with an explicit incomplete result.

The isolated implementation is:

- [`src/angular_navigation.rs`](../src/angular_navigation.rs): explicit source and
  payload capture, immutable projection, fixed compiler metadata, original-byte
  result mapping, currentness checks and conservative scratch cleanup.
- [`src/angular_snapshot_bootstrap.cjs`](../src/angular_snapshot_bootstrap.cjs):
  embedded internal Node bootstrap that narrows the pinned TypeScript system
  host and starts the stock Angular server. This is not a second adapter package
  or installed standalone command.
- [`tests/angular_navigation.rs`](../tests/angular_navigation.rs): source-only
  integration through the new module, with real pinned-runtime cases and
  failure probes. It also compiles the existing LSP/process helpers and their
  focused tests. `src/lib.rs` does not expose this backend.

Transport and process handling reuse the **existing `lsp_client.rs` source**
through an isolated module include. No alternate LSP parser or process owner was
introduced. The current owner performs one finite query per fresh process; a
second query on the same prepared value is refused. The separate measurement
prototype demonstrates several useful queries in one server process, but
connection reuse is not implemented by this finite source slice.

## Prepared inputs and execution boundary

The owner accepts an explicit canonical project root and a saved model revision.
The strict version-1 model contains **1–32 selected `.ts`/`.html` sources**, each
with its original absolute path, language and full SHA-256 revision. It requires
at least one TS source; source bytes are bounded at 256 KiB each. Unknown model
fields, duplicate sources, stale bytes, symlinks and out-of-scope paths refuse.
There is no inferred project, glob scan, tsconfig inheritance or workspace
package discovery.

Sources are copied byte-for-byte into an empty private job's projection. A
separately selected, revision-bound **declaration-only tree** supplies matching
framework types. Its files must be `.d.ts` or sanitized package metadata with a
small declarative schema: name/version, type, types/typings and types-only
exports. Runtime exports, scripts, hooks and arbitrary metadata are refused.
The source tree's `tsconfig.json`, package scripts, `.pnp.cjs`, plugins and other
workspace files are not copied or executed.

The generated compiler model fixes ES2022, ESNext/Bundler resolution,
experimental decorators, strict mode, strict templates, `skipLibCheck`,
`noEmit`, `types: []` and disabled acquisition. Its file list contains only the
selected TS files. These are **explicit finite-model semantics**, not a claim
that an arbitrary application's existing compiler configuration has been
reproduced. Unsupported path aliases, inherited options, generated inputs and
library versions need an expanded, separately qualified model.

Before starting Angular, the bootstrap uses the pinned compiler's parser and
resolver over the selected source snapshots. Unrepresented imports, nonliteral
dynamic imports, triple-slash source/type directives, missing external
`templateUrl`, nonliteral template/resources, computed metadata properties and
style resource fields refuse. The style/computed-property checks are currently
conservative across the parsed source; an ordinary object using those shapes
can therefore also be refused. This is a known finite-profile limit rather
than a silent interpretation change.

The TypeScript `sys` host admits reads/file existence only for the prepared
source/type files and the selected private TS declaration libraries. Directory
results are derived from that finite set, source writes are refused, and
watchers are no-ops because the projection is immutable. Unknown paths are not
read as an implicit dependency. The Node module loader accepts executable
modules only beneath the verified private three-package runtime. Common Node
network, child-process, worker and native-addon entry points are denied; the
server is fixed to stdio, with private TS/Angular probe locations. The stock
Angular server additionally restricts its plugin loader to the selected Angular
language service and disables local plugins/typings installation.
[Pinned server host](https://github.com/angular/angular/blob/ec3bef4616cbd95e3c338bc96ce47831fdfb211c/vscode-ng-language-service/server/src/server_host.ts),
[pinned session](https://github.com/angular/angular/blob/ec3bef4616cbd95e3c338bc96ce47831fdfb211c/vscode-ng-language-service/server/src/session.ts).

These are **defense-in-depth application boundaries, not an OS sandbox** or a
claim that arbitrary hostile trusted runtime code cannot bypass JavaScript
hooks. The actual prototype also ran under the previously qualified macOS
network-denial profile; no network is needed for its tested operations. There
is no runtime npm/npx, shell, project build, generator or installer invocation.
Source code containing a top-level filesystem-writing statement was analyzed
without executing that statement.

## Exact candidate payload and rights

Private evidence is under
`~/.codex/task-state/harness-rewrite-implementation/angular-release-admission-20261025/`.
The original archive/source captures remain in the linked framework
qualification's evidence directory. All copied candidate files are bound by
fresh per-file manifests; no archive installer or lifecycle hook ran.

| Input | Exact identity / measured scope |
| --- | --- |
| Angular server | `@angular/language-server` **22.2.3**, source `ec3bef4616cbd95e3c338bc96ce47831fdfb211c`; tarball **2,087,319 bytes**, SHA-256 `68f13ad6c28e4372b5550d04c84845215157d8f7c5dd1011d585719024e7d8ec`. |
| Angular service | `@angular/language-service` **22.2.3**, same source commit; tarball **2,949,342 bytes**, SHA-256 `539d7f4390b5a007359074cc7ccfbb3eb9cc8a7931a3122fcdba6473857c2d8d`. |
| TypeScript API | **6.0.3**, source `050880ce59e30b356b686bd3144efe24f875ebc8`; tarball **4,515,854 bytes**, SHA-256 `33cd0ee1beaa8c9e9d15a9da836c62ddea4c34a42d7c2d349dbc80d94165d22a`. |
| Three-package runtime tree | **198 files, 52,855,329 regular bytes**; manifest SHA-256 `6ff1ce577fbe8c4509b2cdb75f847002be76410b4817972d75a45aef532aa165`. No package or localized diagnostic pruning was inferred. |
| Framework type data | Angular core **22.2.3** declarations plus **RxJS 7.8.2** declarations and generated types-only package metadata: **269 files, 1,201,659 regular bytes**; manifest SHA-256 `bf6a37a051fcf3cf535317041208e2e490673ad81bc7c68297c6b345398f7b6e`. No Angular/RxJS executable runtime is copied into this tree. RxJS is an additional qualification input, not a production dependency admission. |
| Existing Node | Private **24.22.1**, executable SHA-256 `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`; reused explicitly. Node bytes/notices are outside the runtime/type totals. |

The Angular server directly pins the service and TS versions above; its Node
range includes the selected 24.22.1. [Server metadata](https://registry.npmjs.org/@angular/language-server/22.2.3),
[service metadata](https://registry.npmjs.org/@angular/language-service/22.2.3),
[TypeScript metadata](https://registry.npmjs.org/typescript/6.0.3),
[core metadata](https://registry.npmjs.org/@angular/core/22.2.3),
[RxJS metadata](https://registry.npmjs.org/rxjs/7.8.2).

Angular is MIT, TypeScript Apache-2.0 and RxJS Apache-2.0. The original Angular
server/service tarballs omit a standalone upstream LICENSE body; retain the
exact pinned source MIT text. TypeScript's LICENSE and third-party notices
remain in its copied package; the RxJS archive's license body was retained in
external evidence. Declaration files still need their applicable notices.
[Angular pinned license](https://github.com/angular/angular/blob/ec3bef4616cbd95e3c338bc96ce47831fdfb211c/LICENSE),
[TypeScript pinned license](https://github.com/microsoft/TypeScript/blob/050880ce59e30b356b686bd3144efe24f875ebc8/LICENSE.txt).

**Three npm packages do not mean three original code owners.** The server bundle
has 16 distinct embedded package/version labels, including vscode-jsonrpc
8.2.0, language-server 9.0.1, HTML/CSS services 5.6.2/6.3.10, source-map-support
0.5.21, source-map 0.6.1 and several versions of protocol/type/text-document
packages. Its internal Angular label `0.0.0` is a build label, not a selected
registry version. `identity-summary.json` retains all observed labels. These
comments help identify a bundled closure; they are not complete license or
source-to-binary attestation. Root release review must resolve all embedded
rights/notices before admission.

On 2026-10-25, npm's bulk advisory endpoint returned **no matched advisories** for
the five selected package versions and, separately, the 15 non-internal bundled
package labels. The request and raw results are retained. That is bounded npm
advisory evidence, not a security audit, native Node clearance or proof that the
bundle-label inventory is exhaustive.

## Original locations, ownership and cleanup

Existing LSP framing/budget checks, UTF-16-to-original-UTF-8 conversion and strict
URI matching run before remapping. The Angular owner then accepts only locations
in its selected project sources; generated/private type targets, zero-width,
invalid-boundary or unknown locations refuse. Every returned location retains
its original source revision and nonempty byte span. Current results include
model and Node/runtime/declaration/bootstrap identities, explicit bounded
coverage, and a final **32 KiB** result limit. Empty bindings do not become a
successful general-TS result.

All original sources, the model, runtime/type manifests and created snapshot
bytes are checked before and after execution. A change during analysis returns
stale. An owned child runs with a cleared environment, private HOME/config/temp
paths, no PATH fallback and the existing finite Unix process-group cleanup.
Server-requested edits, commands and configuration expansion remain refused by
the existing transport. No projection URI is returned as an original location.

`cleanup_after_reap` checks the job's directory identity, exact owned-file set
and saved bytes before removing files, then removes empty directories. It
leaves the caller's job root empty and never removes source/runtime/type inputs.
Unexpected files, changed snapshot bytes or unknown process cleanup preserve
the tree for review; Drop does not recursively delete it. A dropped calling
future marks execution unknown while the existing transport owner cancels and
cleans its child. That state does not authorize a replacement run or immediate
scratch deletion.

Path-based checks inherit `ProjectScope`'s documented concurrent ancestor
replacement limits. Native directory-handle and abrupt-owner-death recovery
remain separate host gates; private state and hashes are not an OS containment
claim.

## Actual verification and measurements

The final focused suite passed **32 tests** with explicit real Node/Angular
selection and serial test execution, including inherited transport/process
checks and the Angular cases below. The unrelated real generic adapter fixture
was explicitly skipped because `MIRKET_TEST_ADAPTER` was not selected; Angular
real-runtime tests did run. Focused Clippy with `-D warnings` passed. The build
input guard passed before compilation. This worker did not claim the root's
full-repository, installed MCP or desktop activation gates.

| Case | Observed outcome |
| --- | --- |
| External interpolation | `app.html` `title` maps to `app.ts` original bytes **137–142**, exact text `title`. |
| Inline template after astral text | Template `title` maps to the class member at original `inline.ts` bytes **152–157**. |
| References | Exactly the class declaration **137–142** and external HTML use **11–16** in this corpus. |
| Template symbols | Original `<h1>` and `<p>` ranges plus original selection spans; `.ts` structure is not claimed. |
| Unicode/path correspondence | Selected paths contain a space and Turkish `İ`; source text has CRLF, Turkish characters and an emoji before the external interpolation. Returned paths/revisions/spans remain original. |
| Workspace execution sentinels | Hostile raw tsconfig plugin, package module and PnP files remain outside the projection and do not execute. A top-level `require('node:fs').writeFileSync(...)` statement in selected TS is parsed but does not execute. |
| Missing inputs/unsupported source | Unprepared package import, escaping/missing template and nonliteral template URL fail before returning plausible navigation. Ordinary TS function definition returns incomplete. |
| Stale and bounded inputs | Model/source/payload drift, duplicate/symlinked sources, executable metadata/scripts in type packages and extra runtime files refuse. Source mutation while Angular is running returns stale. |
| Cancellation and cleanup | Already-cancelled/expired work remains incomplete; actual running Angular cancellation records a PID and verifies it is gone after return. Successful/cancelled/stale runs leave an empty job root after explicit cleanup. Changed/extra scratch files are preserved. |

The direct guarded prototype used **one observed process**, exited zero and
resolved four queries under network denial. Two small samples recorded peak RSS
of **354,352–442,128 KiB** (approximately **346–432 MiB**). Sampling can miss brief
peaks, and this is not a minimum requirement or concurrency benchmark. The observed first definitions took about **508–560 ms**; subsequent
inline/reference/symbol calls were roughly **1–32 ms**. Individual timings are
retained in `probe.log` and `guarded-results.json`.

The **Rust debug-build owner** has additional identity/projection/cleanup work.
The final sequential corpus measured navigation around
**3.53–7.96 s** per fresh process,
preparation around **1.19–1.22 s**, and cleanup
around **43–47 ms**. These are sampled small-corpus observations, not p95
targets or release-build performance. It hashes the large Node/runtime closure
before and after execution; removing those checks to improve a smoke-test time
is not an accepted optimization. Measure a release build and larger real
prepared graphs before choosing validated caching/session-reuse policy.

## Smallest remaining integration sequence

1. Root reviews the exact three-package and declaration closure, embedded
   licenses/notices, trusted manifests and source pins. Freeze the selected
   declaration profile and its input provenance; none of these task-state
   manifests is an existing production admission.
2. The shared LSP owner now supports a **trusted native Node argv launch mode**.
   The isolated Angular owner uses a verified private Node executable and a
   verified script path; a path-with-spaces test confirms it needs no shebang,
   shell or PATH search. Qualify this same launch mode with an installed release
   and on native Windows before admitting Angular publicly.
3. Bind one process slot, source model and owned scratch to the existing
   analyzer/heavy admission and receipt lifecycle. Retain resource/ownership
   until reaping is confirmed, including dropped calls and disconnects. Inspect
   actual snapshot cleanup at the integration boundary.
4. Keep the TS/template routing explicit. Angular handles the qualified
   template binding path; ordinary TS stays with its selected owner. Extend
   and test directive/input/output/pipe/control-flow/other-version behavior
   before claiming those operations; do not add a fallback server after an
   Angular failure.
5. Qualify installed MCP identity/tamper/relocation, public request validation,
   cancellation/death recovery and Codex tool/model turns. Run macOS 15/current
   and native Windows x64/ARM64, including Job Objects, drive/case/UNC handling,
   file locks and executable-in-use updates, before platform admission. Windows
   currently refuses before effects; no Windows process execution occurred.

External evidence retains exact per-file payload/type manifests, selected
archive metadata, `identity-summary.json`, pinned license bodies, advisory
requests/results, prototype wire/stderr/RSS captures, Rust measurements,
`build-inputs-final.json`, `tests-final-source.log`, targeted follow-up logs and
`clippy-final.log`. No commit, push, public route or publication was made.
