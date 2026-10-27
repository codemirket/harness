# Vue, Angular and Astro navigation qualification

Qualification date: **2026-10-25**. This is a finite development experiment on
**macOS 26.5.2 ARM64**, using the explicitly selected private **Node 24.22.1**.
It changes this document only. It does not admit analyzers, change the production
lockfile, activate MCP navigation, qualify Windows or establish Codex acceptance.

## Decision

Keep the official framework engines and their original-source mapping machinery.
The useful next semantic integration is **Angular 22.2.3 with its pinned
TypeScript 6.0.3 API**. **Astro 2.17.3** also passed useful original-source LSP
cases, but its workspace Vue/Svelte integration loading must be excluded before
production. **Vue 3.3.13 needs an explicit TypeScript 6 companion**: its standalone
LSP supplies SFC/CSS structure, while the official TypeScript plugin supplied the
tested template definitions and references. Native TypeScript 7 is not a proven
replacement for that companion.

No candidate is ready for immediate release admission. In particular, successfully
starting the stock binaries against a normal project is insufficient: the
counterexamples below execute project code during otherwise read-only analysis.

## Exact candidates and rights

The npm `latest` metadata was rechecked on the qualification date. Downloaded
tarballs were checked against their registry SHA-512 integrity values; SHA-256
values and package metadata are retained in external evidence. These are
qualification pins, not authorized substitutions for the existing lockfile.

| Candidate | Observed release/source identity | Runtime and rights boundary |
| --- | --- | --- |
| `@vue/language-server` and `@vue/typescript-plugin` **3.3.13** | npm publication: **2026-10-10**, respectively 14:35:10 and 14:33:28 UTC; source `f2d92d3f1cb5513268fff358993f163e78d4821a`. [Server metadata](https://registry.npmjs.org/@vue/language-server/3.3.13), [plugin metadata](https://registry.npmjs.org/@vue/typescript-plugin/3.3.13). | MIT. The server's exact Volar dependencies include **2.4.28**; the broader experiment also resolved newer Volar copies for Astro. Do not flatten them into one guessed version. The Vue server tarball omits the original license body; retain the [pinned source license](https://github.com/vuejs/language-tools/blob/f2d92d3f1cb5513268fff358993f163e78d4821a/LICENSE) in distribution notices. |
| `@angular/language-server` and `@angular/language-service` **22.2.3** | npm publication: **2026-10-14**, 23:42:28 and 23:40:40 UTC; source `ec3bef4616cbd95e3c338bc96ce47831fdfb211c`. [Server metadata](https://registry.npmjs.org/@angular/language-server/22.2.3). | MIT; server pins exactly **TypeScript 6.0.3** and language-service **22.2.3**. Node requirement is `^22.22.3 || ^24.15.0 || >=26.0.0`. Both tarballs omit a separate original license body; preserve the [pinned source license](https://github.com/angular/angular/blob/ec3bef4616cbd95e3c338bc96ce47831fdfb211c/LICENSE) and review the bundled compiler/server notices. |
| `@astrojs/language-server` **2.17.3** | npm publication: **2026-10-21 14:13:05 UTC**; npm has no `gitHead`; the exact upstream package tag resolves to `34c953555749223022e6bb2f02019d3515749a24`. [Metadata](https://registry.npmjs.org/@astrojs/language-server/2.17.3), [source tag](https://github.com/withastro/astro/tree/%40astrojs/language-server%402.17.3/packages/language-tools/language-server). | MIT; its tarball includes LICENSE. The range `@astrojs/astro2tsx:^0.1.2` resolved to **0.1.3**, with a Rust/WASI payload and emnapi runtime. This is a real additional compiled dependency boundary. |
| `typescript` **6.0.3** | npm publication **2026-04-16 23:38:27 UTC**; source `050880ce59e30b356b686bd3144efe24f875ebc8`. [Metadata](https://registry.npmjs.org/typescript/6.0.3). | Apache-2.0, no registry runtime dependencies; include LICENSE and third-party notices. A shared copy served this fixture's three frameworks. That proves these cases, not all framework/API compatibility. |
| `@astrojs/ts-plugin` **1.10.12** | npm publication **2026-09-16 19:39:10 UTC**. [Metadata](https://registry.npmjs.org/@astrojs/ts-plugin/1.10.12). | MIT; installed only in disposable research. It adds `@astrojs/compiler` **2.13.1** and its Go WASM payload. The tested `.astro` LSP path does not require this plugin. Do not add it merely because it is an official companion; `.ts` imports of `.astro` remain a separate unqualified operation. |

The disposable combined closure installed **138 packages** with lifecycle scripts
disabled and optional peers omitted. Inspected manifests had no preinstall,
install or postinstall hook; some packages contain prepare/development scripts,
which were not executed. Its metadata licenses were MIT, ISC, BSD-2-Clause,
BSD-3-Clause, 0BSD and Apache-2.0. npm's audit endpoint reported **zero known
advisories** for that closure on this date. This does not review embedded Rust/Go
dependencies, establish notice completeness, or select the smaller eventual
production closure. Preserve each package's exact integrity and notice body,
including bundled-source rights, before admission.

## Actual offline fixture results

External evidence is under
`~/.codex/task-state/harness-rewrite-implementation/framework-navigation-qualification-20261025/`.
`qualify.py` runs the finite corpus; `qualification-results.json` retains full
responses, converted original byte spans, commands, baseline sentinel effects and
sampled memory. `counterexamples.py` / `counterexample-results.json` retain the
deliberately unsafe cases. `verify-results.py` passed **21 discriminating
assertions**, recorded in `assertion-results.json`.

The project path contains a space and Turkish `İ`; sources use CRLF, Turkish
strings/identifiers and an astral emoji before selected spans. All analyzer
processes ran under a macOS network-denial profile. An explicit fetch control
failed as expected. That profile is development instrumentation, not a proposed
product sandbox. The environment omitted `NODE_OPTIONS`, `NODE_PATH` and inherited
runtime/configuration paths. PATH held task-owned npm/npx/node/tsc/package-manager,
shell, Git and make sentinels. Baseline runs produced **zero sentinel effects**.

| Engine/path | Demonstrated result | Limit exposed by the same corpus |
| --- | --- | --- |
| Angular stock stdio LSP | An external `app.html` interpolation maps to `app.ts` property `title` at original bytes **179–184**. An inline template following an emoji maps to the property in `inline.ts`, bytes **159–164**. References include the declaration and exact external-template use. External template document symbols include the original tags. | A plain TypeScript imported-function definition returned null; this engine is a template owner, not the general TS owner. `app.ts` document symbols returned empty in this finite case. |
| Astro stock stdio LSP | Template `başlık` maps to its original frontmatter declaration, bytes **40–73**; a template `selam` call maps to the original helper declaration, bytes **22–83**. References map to the exact `başlık` tokens at **46–54** and **102–110**. Symbols include frontmatter and embedded `.kart` CSS. | Definitions are declaration ranges, not necessarily identifier ranges. A missing-package call returns its import declaration; that is not a successfully resolved dependency definition. |
| Vue standalone stdio LSP, explicit no-companion response | SFC symbols contain `script setup`, `template`, `style`, original bindings and embedded `.kart` CSS. | Template definitions and references were empty. Its `tsserver/request` notification requires a companion; startup capability flags do not prove semantic navigation. |
| TypeScript 6.0.3 tsserver with only the official Vue global plugin | Template `başlık` maps to original `.vue` bytes **67–75**; its two references map to **67–75** and **159–167**. Template `selam` maps to helper identifier bytes **38–43**. | Definitions also include immutable helper type `__VLS_unwrap`; return coverage/filtering must be explicit. `navtree` supplies only the file root in this fixture, so it does not replace Vue SFC structure. Missing-package navigation returns an alias plus the helper type. |

All four final baseline processes exited with code zero. The sampled peak RSS was
approximately **438 MiB Angular**, **418 MiB Astro**, **159 MiB Vue LSP** and
**317 MiB Vue tsserver**. These are one small sequential corpus's process samples,
not a benchmark, hard memory requirement or aggregate concurrency claim. No child
process was observed; that does not measure WASM worker threads or prove absence
of short-lived children.

The initial Angular fixture used `başlık`/`sayı` as template expression identifiers.
Angular emitted lexer/parser errors and omitted the intended bindings. The final
positive Angular corpus uses `title`/`count` while retaining Turkish/astral source
text. Preserve the original run as a **language-syntax refusal case**, not a
Unicode-position pass. A separate initial Vue LSP bridge stub used the wrong
nested notification shape and crashed; the corrected explicit no-companion run
above is the relevant standalone result. Neither earlier failure is a passing run.

## Execution boundary and counterexamples

1. **Vue configuration is executable input unless constrained.** Setting
   `vueCompilerOptions.plugins:["evil-plugin"]` executed the task-owned module
   while the official TypeScript plugin loaded the project. The same definition
   request still returned plausible original locations. The upstream
   [compiler-options resolver](https://github.com/vuejs/language-tools/blob/f2d92d3f1cb5513268fff358993f163e78d4821a/packages/language-core/lib/compilerOptions.ts)
   calls `require` for plugins while processing extended configs. Clearing plugins
   only in the root is insufficient if an inherited config is processed first.
2. **Astro foreign integrations execute workspace code.** Importing `App.vue`
   from an `.astro` fixture executed the task-owned
   `@astrojs/vue/dist/editor.cjs`. Definition then returned `App.vue` at **0:0–0:0**,
   which establishes neither a semantic target nor a useful source mapping. The
   [pinned package loader](https://github.com/withastro/astro/blob/34c953555749223022e6bb2f02019d3515749a24/packages/language-tools/language-server/src/importPackage.ts)
   resolves workspace editor integrations; their generated foreign code has no
   demonstrated original mappings. Svelte uses the analogous path.
3. **Astro formatting crosses another boundary.** An explicitly issued formatting
   request loaded the task-owned project Prettier module. Baseline symbols,
   definitions and references did not load it. Deny formatting, code actions,
   commands and edit operations at the actual adapter method boundary; omitting
   optional peers from Mirket's payload does not disable workspace resolution.
4. **Angular has useful existing restrictions.** Its pinned
   [server host](https://github.com/angular/angular/blob/ec3bef4616cbd95e3c338bc96ce47831fdfb211c/vscode-ng-language-service/server/src/server_host.ts)
   refuses arbitrary plugin names; the
   [session](https://github.com/angular/angular/blob/ec3bef4616cbd95e3c338bc96ce47831fdfb211c/vscode-ng-language-service/server/src/session.ts)
   uses `nullTypingsInstaller`, `allowLocalPluginLoads:false` and a selected Angular
   plugin location. The hostile compiler plugin, package scripts, Astro/Vite/Nuxt
   configs and PnP sentinel did not execute in baseline analysis. Fixed private
   probe locations and prepared project metadata are still required.

The counterexamples intentionally ran only task-owned sentinel modules in a
disposable project. They demonstrate a reachable execution path; the passive
contract must prevent that path before a real workspace is analyzed.

## Smallest implementation and admission sequence

1. **Prepare one finite project model before any framework launch.** Bind original
   saved documents, templates, declarations, package metadata and all inherited
   configuration to revisions. Supply a finite file graph and fixed compiler
   options; disable acquisition and reject unsupported plugins/integrations,
   missing dependencies and unrepresented generated inputs. Preserve the original
   config and report any unsupported semantics. Do not read a raw config first
   and sanitize it after a framework resolver has already loaded plugins.
2. **Use a private prepared input projection, or equivalently bounded upstream
   filesystem hosts.** A projection must contain verified unchanged source bytes,
   declarative sanitized configs and pinned type inputs only; map each temporary
   file identity back to its original path/revision. This is temporary analyzer
   input, not another project-state format. Recheck original source/model revisions
   before returning results. The existing test-only LSP owner does not yet enforce
   this correspondence or remap projected URIs: implementation is still required.
3. **Integrate Angular first.** Use one manifest-bound private Node process and
   exact server/service/TS6 payloads. Fix `--stdio`, private `--tsProbeLocations`,
   `--ngProbeLocations` and `--tsdk`; do not search workspace TypeScript or PATH.
   Keep document symbols, definitions and references as separate declared
   capabilities. Route plain TS positions to the selected TS owner by explicit
   language/region policy, without silently converting an Angular failure into a
   successful result from a different engine. Require prepared matching Angular
   declarations; the fixture used `@angular/core` 22.2.3 as type data only.
4. **Integrate the Astro `.astro` path.** Pin server 2.17.3 and the actual
   astro2tsx/WASI closure, use private TS6 `initializationOptions.typescript.tsdk`,
   `contentIntellisense:false` and `disableAutoImportCache:true`. Admit only declared
   `.astro`/JS/TS/type inputs to this owner; refuse foreign `.vue`/`.svelte` source
   processing unless fixed, admitted integrations and useful original mappings
   are independently established. Do not include the old compiler/TS plugin for
   an unrequested operation. Preserve CSS structure as structure, not cascade or
   selector-reference semantics.
5. **Close the Vue companion protocol before admission.** The concrete working
   semantic path is one private TS6 tsserver with fixed `--globalPlugins
   @vue/typescript-plugin`, private `--pluginProbeLocations`, disabled acquisition
   and no arbitrary local plugins. A bounded owner must support that finite
   protocol or prove a pinned standard-LSP composition with the same official
   mapping engine. The official Vue LSP remains useful for SFC/CSS structure.
   Do not claim its notification bridge or native-TS7 integration is implemented.
   A two-process Vue composition consumes two analyzer process slots and must
   reserve them together; it cannot bypass the global two-analyzer limit.
6. **Apply existing ownership and release gates.** Bind every payload hash and
   release input, launch absolute private paths with controlled HOME/cache/temp
   locations, reserve analyzer/heavy admission, bound initialization and execution,
   reject out-of-scope/unmapped/zero-width synthetic targets, and retain explicit
   incomplete/missing-metadata/stale outcomes. Filter known helper/generated targets
   only with declared coverage; do not silently drop them and call the result
   complete. Reject server edits/commands/configuration expansion. Test actual
   installed MCP, relocation, tamper, revision change, cancellation and descendant
   cleanup before Codex activation. No runtime npm/npx or project build is needed.

## Remaining unsupported and platform cases

This corpus does **not** qualify component prop/event/slot bindings, directive or
pipe resolution, Vue external `src` blocks/Pug/preprocessors/macros, complex
Angular control flow and library versions, Astro content collections/generated
types, `.ts` imports of `.astro`, Astro foreign renderer integrations, CSS
reference semantics, broad diagnostics, whole-project reference completeness,
workspace plugins, build generators or arbitrary monorepo inheritance. These
remain explicit unsupported/incomplete operations pending independent cases;
the agreed language scope is not reduced by a syntax-only result.

Native **Windows 11 x64 and ARM64** remain entirely untested, including private
Node execution, Angular watcher file locks/polling, URI drive/case/UNC behavior,
Unicode paths, offline loading, WASI shared-memory/thread behavior, cancellation,
Job Object ownership and installed packaging. The astro2tsx 0.1.3 loader creates
a shared memory with **4,000 initial 64-KiB pages (250 MiB)** and maximum 65,536
pages; that is configured WASM memory, not measured resident memory. Its source
tag resolves to `342e26603bf6416545a85b506b434d25226e1435`; complete compiled
dependency/notices and minimum-platform checks remain open. A WASM artifact and
pure-JS package metadata do not qualify either Windows architecture.

The oldest supported macOS, actual Codex tool/model turns, full release rights,
and production transport/admission integration are also unverified. No release,
publication, user-installation change or client-configuration change occurred.
