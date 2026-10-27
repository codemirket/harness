# M3 Vue and Astro finite snapshot qualification

Observed **2026-10-25**, **macOS 26.5.2 build 25F84 ARM64**, explicit private
**Node 24.22.1**. This source slice adds useful offline Vue and Astro navigation.
It is **not registered in `lib.rs`, MCP, a release, the adapter package or an
installed home**. The old harness and client settings were not changed.

## Decision and owners

Use the upstream **Vue 3.3.13 language-core transform** or the upstream
**Astro language-server 2.17.3 core transform**, with their matching Volar mapping
APIs and **TypeScript 6.0.3** in a single fresh Node process. The standard
TypeScript language service and Volar's original-source proxy provide definitions
and references. This closes the tested Vue companion boundary without a
second server, a new tsserver protocol client or a handwritten semantic engine.
The [earlier framework qualification](m3-framework-navigation-qualification.md)
remains useful counterevidence about stock workspace/configuration loading.

The isolated files are:

- [`src/vue_astro_navigation.rs`](../src/vue_astro_navigation.rs): immutable saved
  source capture, strict payload identities, bounded finite child, embedded
  bootstrap, original-byte verification/currentness and conservative cleanup.
- [`tests/vue_astro_navigation.rs`](../tests/vue_astro_navigation.rs): real native
  runtime fixtures, negative model/span cases and inherited existing process
  owner tests. It includes the new source directly, with no public route.

The existing `process.rs` supplies the child/process-group owner, bounded frame
reader, stderr drain, cancellation and reaping. No shared process implementation
was changed. The embedded JS is an internal finite leaf, not a second installed
adapter package. Integration should move its execution into the existing owned
adapter/worker and release lifecycle after the remaining admission checks.

The upstream APIs selected here create virtual source from supplied snapshots;
configuration parsing and full language-server construction are not required.
[Vue pinned language plugin](https://github.com/vuejs/language-tools/blob/f2d92d3f1cb5513268fff358993f163e78d4821a/packages/language-core/lib/languagePlugin.ts),
[Astro pinned core](https://github.com/withastro/astro/blob/34c953555749223022e6bb2f02019d3515749a24/packages/language-tools/language-server/src/core/index.ts),
[Volar pinned proxy](https://github.com/volarjs/volar.js/blob/4d4d6c6df14bff46cdb99c3fc2f017c80273d27c/packages/typescript/lib/node/proxyLanguageService.ts).

## Selected snapshot contract

A trusted caller explicitly selects one canonical project root, **1–8 saved
sources**, each source's full SHA-256 revision and exact language, a trusted
payload and an empty private job directory. Each model contains Vue **or** Astro,
plus selected `.ts`/`.js` helpers. Source bytes retain their original names,
UTF-8, BOMs, Unicode and line endings; no source projection is written to disk.
Rust passes only relative source names, source text and revisions to the leaf.
Original absolute source paths remain with Rust.

The filesystem-backed project host is replaced by immutable maps under synthetic
`/mirket/` names. A fixed TypeScript compiler model uses ESNext, Bundler module
resolution, strict mode, preserved JSX, `allowJs`, `checkJs`, `noEmit`, empty
`types`, matching immutable TypeScript libraries and the selected declaration
profile. Imports resolve only through that map. There is no discovery of existing
tsconfig/package metadata, workspace declarations or aliases.

Vue uses `getDefaultCompilerOptions(3.5, 'vue', true, ...)` with the selected
Vue 3.5.44 declarations and the official immutable helper types. It does **not**
call the executable Vue configuration resolver. Astro registers only its `.astro`
core plugin; the Vue/Svelte integration plugins, content collections, formatter,
commands and language-server plugin list are never constructed. Astro's Volar
2.4.30 proxy is resolved through its pinned `@volar/kit` dependency; Vue uses
Volar 2.4.28. These copies are deliberately not flattened.

Before navigation, the owner checks the whole selected model. Original script
syntax, source/type/lib directives, nonliteral dynamic imports, missing imports,
foreign renderers, Vue external `src` blocks, custom blocks/generic SFCs and
preprocessor languages refuse. Astro script-tag extra service scripts are not
selected and refuse. Generated TS syntax must also be valid before semantic
results return. Unsupported imports do not become plausible local alias results.

`Structure` is separate from semantic navigation: official SFC/frontmatter block
ranges, original top-level script declaration names and the upstream HTML
service's document symbols. Embedded style tags are structure; CSS selector or
cascade references are not claimed. `Definitions` and `References` use the
upstream TypeScript/Volar source mapping. Only nonempty, exact original UTF-8
boundaries in selected original sources are accepted. Known declaration/helper
files, generated targets, zero-width locations, unverified/failed aliases and
unknown targets make the entire query incomplete; they are not silently dropped.

Successful results retain every source identity and the Node/runtime/declaration/
bootstrap hashes, plus `model_complete: false`. `complete` describes only whether
the **bounded query** was truncated. It never claims an entire application or all
framework semantics. Reference `is_write` is the upstream classification; it
does not mark every declaration as a write. Results are ordered and deduplicated.
Stale results contain no locations. Backend failures have fixed bounded reasons;
raw exception messages or source contents are not returned as diagnostics.

Bounds are **256 KiB/source**, **1 MiB total source**, **8 sources**, **32,768
original script AST nodes**, **64 original AST levels**, **100 result rows**,
**2 MiB request JSON**, **32 KiB result JSON** and **16 KiB counted stderr**.
The complete Rust envelope also has a 32 KiB bound. TypeScript/framework parsing
can fail before an AST walk; the finite Node process has a **30-second maximum
execution deadline**, shortened by the caller's remaining deadline. Payload
manifests are limited to 8,192 regular files and 192 MiB/tree; the Node executable
is separately bounded. Preparation and full payload hashing are synchronous in
this isolated API and need the existing admitted worker boundary on integration.

## Passive loading and lifecycle evidence

Trusted modules and the selected immutable library/declaration assets load before
source processing. Before any supplied source is transformed, the bootstrap
constrains the TypeScript system host to the in-memory map, denies filesystem
writes, restricts module/file reads to the selected payload, and denies common
network, DNS, child-process, Worker, native-addon and directory-change entry
points. No source is evaluated, no package manager runs, and no missing input is
fetched. The source model never supplies executable plugin or runtime paths.

These are **application boundaries over trusted candidate runtime code, not an
OS sandbox**. They do not establish arbitrary malicious-code containment or a
complete syscall trace. The official WASI loader is trusted code, has no supplied
filesystem preopens and uses shared WebAssembly memory. Its configured initial
memory remains 4,000 64-KiB pages (250 MiB); that is distinct from measured RSS.

The exact direct prototypes ran under the prior qualification's macOS
network-denial profile. Both frameworks returned useful results with zero denied
calls. A separate positive-control bootstrap blocked ten deliberate outside-read,
write/open-for-write, spawn, Worker, socket, HTTP, DNS, fetch and external-module
attempts. The tests also leave hostile tsconfig/plugin/PnP/Prettier/Astro config
files in the original project and analyze an explicit filesystem-writing source
statement without creating its sentinel. Those cases establish the exercised
path, not every API or possible source construct.

One finite query consumes each prepared execution. The process starts through
an absolute private Node path with an empty inherited environment, private
HOME/temp paths, no shell/PATH fallback and the existing Unix process-group
owner. Cancellation and failure terminate/reap before returning. All original
sources and all payload inputs are checked again after successful process
reaping; a change to **any** selected source discards the whole result. These
are sequential checks, not an atomic multi-file snapshot.

The only scratch file is the embedded bootstrap. Explicit cleanup verifies the
job directory's device/inode, the exact file set and bootstrap bytes before
removing that file; it leaves the caller's job directory empty. Unknown/changed
scratch is preserved. A dropped future leaves state unconfirmed and cannot
claim cleanup or authorize immediate replay. No recursive deletion runs from
Drop. Concurrent hostile ancestor replacement and abrupt-owner-death recovery
remain existing native ownership gates.

## Exact candidate payload and rights

Private evidence is retained at
`~/.codex/task-state/harness-rewrite-implementation/vue-astro-qualification-20261025/`.
Registry latest metadata was rechecked on this date: Vue language-core **3.3.13**
and Astro language-server **2.17.3**. The exact development installation used
lifecycle scripts disabled and optional peers omitted; it explicitly added
`@emnapi/runtime` **1.11.3** after the first Astro probe exposed that required
peer. That failed probe is preserved, not counted as a pass. No project build or
upstream helper ran.

| Candidate | Archive SHA-256 | Source / rights |
| --- | --- | --- |
| `@vue/language-core` 3.3.13, 82,624 bytes | `f3da753a82257eda3c411afec2bd4dec647f2eb3d7dd650ee6af935508469aa6` | Vue source `f2d92d3f1cb5513268fff358993f163e78d4821a`, MIT. |
| `@volar/typescript` 2.4.28, 42,634 bytes | `489fc8eb2c67da21636a9b8c4cbfe5f48a2100b177489868195e5872685ce96a` | Source `882cd56d46a13d272f34e451f495d3d62251969a`, MIT. |
| `@volar/typescript` 2.4.30, 45,213 bytes | `9e778dc3eae800a30e3d488cb1fc25612450eb3abb1aeb417dffc340bcb1af5a` | Official `v2.4.30` tag resolves to `4d4d6c6df14bff46cdb99c3fc2f017c80273d27c`; registry has no gitHead. MIT; this identifies the tag, not a reproducible build attestation. |
| `@astrojs/language-server` 2.17.3, 49,617 bytes | `4b8e4a98e6aeec0ee72e4db4cdf0723da28994efc6cb371ac868ea777f019f88` | Official package tag source `34c953555749223022e6bb2f02019d3515749a24`, MIT. |
| `@astrojs/astro2tsx` 0.1.3, 504,488 bytes | `ecba34fe481da75b92a838df1631863010ebcf68391edd58f48b0db75d5e62bc` | Source tag `342e26603bf6416545a85b506b434d25226e1435`, MIT, compiled Rust/WASI boundary. |
| `@emnapi/runtime` 1.11.3, 68,501 bytes | `e6eb8913a08b551436d65f9a5cfbdcc8006f58e6798226d24e695c25679b5e49` | Source `b2011ea96d28fe60a6b8eabb82c2d6264daf7460`, MIT. |
| TypeScript 6.0.3, 4,515,854 bytes | `33cd0ee1beaa8c9e9d15a9da836c62ddea4c34a42d7c2d349dbc80d94165d22a` | Source `050880ce59e30b356b686bd3144efe24f875ebc8`, Apache-2.0. Preserve LICENSE and ThirdPartyNoticeText. |
| Existing private Node 24.22.1 executable | `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329` | Existing explicitly selected macOS ARM64 runtime; Node notices remain its separate admission input. |

[Vue metadata](https://registry.npmjs.org/@vue/language-core/3.3.13),
[Astro metadata](https://registry.npmjs.org/@astrojs/language-server/2.17.3),
[astro2tsx metadata](https://registry.npmjs.org/@astrojs/astro2tsx/0.1.3),
[TypeScript metadata](https://registry.npmjs.org/typescript/6.0.3).

The full development install contains **88 package entries / 84 unique archives**.
Every archive's registry SHA-512 integrity was checked and its SHA-256, metadata,
source hints, scripts and embedded notice identities were retained in
`closure-inventory.json`, `metadata/` and `archives/`. Metadata licenses comprise
79 MIT, 4 ISC, 2 BSD-3-Clause, 1 BSD-2-Clause, 1 0BSD and 1 Apache-2.0 package
entry. Full packages and notices were preserved, rather than inferring a smaller
closure from exercised imports.

The immutable copied runtime is **4,334 regular files / 70,188,615 bytes**,
manifest SHA-256
`9d75890fde71bf73d90a1f9bd55f12cd6cf74aab30df0a8ca98ae050228fd316`.
The separate Vue declaration profile is **25 files / 1,105,505 bytes**,
manifest SHA-256
`e30c9d835dff2746d3516a6893e51efa4458557f3b85d822b108d3e051cf7442`.
It copies declarations/notices from Vue, runtime-core, runtime-dom, reactivity and
shared **3.5.44**, plus **csstype 3.1.3** as pinned by `types-provenance.json`, and writes
only strict types-only package metadata. No Vue runtime JavaScript is copied into
that tree. Node and archive/staging bytes are outside these regular-file totals.

Two npm archives omit standalone license bodies. The exact
`@napi-rs/wasm-runtime` 1.2.4 source MIT body is retained from source
`7e3f293e2d6a3032eabfe51ff38bcaa82d342a2f`, SHA-256
`3f1ce66533302df3a32edbfdfc0b78f0dd34659e4c1f5817162e5ea3c2297215`.
For `@vscode/l10n` 0.0.18, the already-reviewed
[web-source supplement](../packaging/admission/m3-web-source.md) was copied to
private evidence, SHA-256
`c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383`.
Its caveat remains: that immutable upstream MIT body is **not a recovered source
commit for the published 0.0.18 archive**.

Npm's bulk advisory endpoint returned **no matched advisories** for the selected
package/version set on this date. The exact request/response are retained; this
is not a security audit or review of embedded compiled Rust dependencies.
The pinned compiler-rs source archive, Cargo manifests/lock and original MIT
body were captured without building. The final compiled WASM dependency/license
closure and source-to-artifact evidence still need release review. JavaScript
package SPDX fields do not close that compiled boundary.

## Actual checks and limits

The build-input guard passed before compilation. The final native focused suite
passed **19 tests in 142.21 seconds**; strict focused Clippy and the format check
also passed. Exact logs are retained in private evidence. Native tests explicitly selected the private Node and complete
payload paths. Inherited generic adapter tests that require a separately selected
adapter return early when it is absent; they are not claimed as real adapter
activation. Vue/Astro real runtime cases did execute.

The discriminating corpus includes:

- Unicode project paths containing a space and `İ`, original CRLF and Turkish
  identifiers, an astral emoji before queried spans, exact original source hashes
  and UTF-8 slices. Both frameworks resolve template `başlık` locally and `selam`
  through the selected original helper.
- Parent component property `title` resolves to the child's original property
  token: Vue **52–57**, Astro **23–28**. References include the parent binding and
  the child property; a selected, unimported component with the **same property
  name** contributes no result. This rejects a string-search substitute.
- Structure contains original script/frontmatter, top-level binding and markup
  spans. A component-tag definition that maps to the generated default export at
  zero width returns `unmapped_or_generated_target`, with no plausible replacement.
- Missing packages, foreign renderers, external Vue blocks, invalid byte interiors,
  NULs, duplicate/symlinked/mixed models, stale revisions, generated/outside targets,
  invalid reference flags, forged complete-model claims and malformed refusal
  shapes refuse. A changed unrelated source invalidates the result.
- Actual running Node cancellation records its PID and verifies ESRCH after
  return; in-flight source drift returns stale. Successful and cancelled queries
  leave the job empty after verified cleanup. Unexpected scratch files are
  preserved. Consumed execution cannot be replayed.

One small network-denied direct sample measured **0.66 s / 350,306,304 bytes max
RSS for Vue** and **0.52 s / 354,369,536 bytes for Astro**. These are single fresh
leaf observations, not p95 targets, minimum RAM, whole-owner performance or
concurrency qualification. Rust preparation and before/after payload hashing add
work; the complete sequential native test corpus takes roughly 2–3 minutes.

The concrete remaining integration sequence is to freeze/admit the exact runtime,
declaration and compiled WASI rights closure; route this leaf through the existing
release-bound worker, analyzer/heavy admission, lifeline and disconnect owner;
then verify installed MCP tamper/relocation, cancellation/death recovery and Codex
native/model turns. There is no fallback server after a failure.

Component-tag file definitions remain an explicit missing behavior under the
current nonempty-span contract. A concrete next option is to qualify an upstream
component-source navigation API, or review a separate revision-bound **file-target**
result variant; inventing a span for a generated zero-width export is not valid.
Vue macros beyond the tested props, inherited configuration, Pug/preprocessors,
content collections, Astro client script services, foreign integrations, broad
framework diagnostics and whole-project reference completeness remain unqualified.
Native Windows 11 x64/ARM64 and oldest-supported macOS are untested. Windows
execution refuses before effects. No release admission, public route, user-home
activation, commit, push or publication occurred.
