# Historical language-tooling research

Captured 2026-10-25 before applying decision 81. These wider language/analyzer reviews and earlier decisions are historical evidence only. The user separately retained SQL SQLite/PostgreSQL navigation; its current bounded contract and dated qualification remain active, and preserved SQL material here is not a withdrawal. They do not define first-release support, admit dependencies or authorize installation. The current exclusive target list and syntax/navigation distinction are in [decision 81](../../decisions.md#decision-81--exclusive-first-release-language-tooling-scope).


## Preserved source: planning/toolchain-proposal.md

Source file SHA-256 before amendment: `270976f6722c4b829f4c27f02a1c4f38afab4912c7f87baaf233f6ddb606f414`.

## Navigation: open-source candidates and honest gaps

Use one bounded LSP client, package servers privately and launch only the selected project's languages. This preserves the broad agreed coverage; it does not promise every backend is already admissible under the read-only execution rule.

| Coverage | Recommended candidate / disposition | License basis and outstanding work |
| --- | --- | --- |
| JS/TS/TSX/JSX | TypeScript 7 upstream native LSP | Apache-2.0. Release `v7.0.2` is stable; `v7.1-beta` is not the proposed baseline. Verify initialization, navigation, cancellation, shutdown and disabled automatic type acquisition/plugins. |
| Python | Pyright | MIT verified in LICENSE.txt. Prefer the established Node-based server; do not confuse it with the proprietary Pylance distribution. |
| Rust | rust-analyzer | MIT OR Apache-2.0. Toolchain/metadata and implicit execution remain the hard part. |
| Go | gopls | BSD-3-Clause Go tooling. Private Go/toolchain behavior; no automatic downloads or module resolution effects. |
| C/C++ | clangd | Apache-2.0 with LLVM exceptions for the relevant closure. Controlled compilation database, include paths and query-driver behavior. |
| Java | Eclipse JDT LS | EPL-2.0 and dependent components. Private compatible OpenJDK; its distribution/license closure is separate. Verify Maven/Gradle imports and annotation processing. |
| Kotlin | Bounded OSS parser/grammar; exact choice pending | Structural coverage accepted in decision 50: symbol outlines and syntax-aware search. No resolved cross-file definitions/references. Qualify the grammar license, Kotlin syntax coverage and revision-bound locations; no semantic LSP in this baseline. |
| C# | csharp-ls | MIT; .NET SDK has its own OSS/native closure. Current README requires .NET 10. Disable analyzers/generators and verify MSBuild loading does not violate execution policy. |
| Swift | Privately packaged SourceKit-LSP with compatible toolchain | Apache-2.0 family. Decision 52 permits an explicitly selected Apple SDK from user-installed Xcode; SDK is not bundled. Qualify compatibility and project metadata, report missing prerequisites, and do not trigger hidden builds or global developer-tool changes. |
| Ruby | Ruby LSP | MIT. Private Ruby/gems; no automatic Bundler installs or implicit project configuration execution. |
| PHP | PHPantom 0.11.1 as a native qualification candidate | MIT source and native Mac ARM64/Windows x64/ARM64 release leads. Not admitted: test passive configuration, executable overrides, stubs and real definitions/references. Phpactor documents WSL for Windows and is not a valid native Windows baseline. |
| Dart | SDK analysis server | BSD-3-Clause SDK. The server comes from the SDK, not merely the MIT editor extension. Account for project/Flutter metadata. |
| Bash / PowerShell | bash-language-server / PowerShell Editor Services | MIT projects, plus private PowerShell/.NET closure where required. No workspace profile execution by default. |
| Vue / Svelte / Astro | Maintainer language servers | MIT projects. Astro now lives in the Astro monorepo; the old language-tools repo is archived. Check embedded-language engine/version requirements and preprocessors. |
| HTML/CSS/SCSS/JSON/YAML/TOML | Narrow existing language services/parsers and local schemas | Resolve individual licenses/pins. Structural/schema/link operations as agreed; no invented semantic parity. |
| Markdown/HCL/build files | Targeted parsers, tree-sitter grammars where appropriate | Review each grammar's license. Do not install all-language packs indiscriminately. |
| SQLite/PostgreSQL SQL | SQLGlot pure-Python library is the leading qualification candidate | Decision 51: offline table/column resolution from project SQL and declared schemas. Candidate version 30.24.0, MIT, no extras; not yet admitted or runtime-tested. Use strict qualification plus source-index integration, not a database-backed LSP or lineage defaults. See the focused review below. |

[TypeScript 7 announcement](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/), [stable release](https://github.com/microsoft/TypeScript/releases/tag/v7.0.2), [wrapper's own upstream-LSP explanation](https://github.com/typescript-language-server/typescript-language-server), [Pyright license](https://github.com/microsoft/pyright/blob/main/LICENSE.txt), [JDT LS](https://github.com/eclipse-jdtls/eclipse.jdt.ls), [csharp-ls](https://github.com/razzmatazz/csharp-language-server), [Ruby LSP](https://github.com/Shopify/ruby-lsp), [Phpactor](https://github.com/phpactor/phpactor), [Dart license](https://github.com/dart-lang/sdk/blob/main/LICENSE), [Astro relocation](https://github.com/withastro/language-tools/blob/main/README.md).

TypeScript's native port is a meaningful upstream simplification, not a reason to promise the published compiler speedup for our LSP tasks. An upstream issue reports nonzero exit after normal shutdown on 7.0.2/macOS ARM64; test lifecycle behavior rather than treating the release label as proof. Embedded Vue/Svelte/Astro dependencies may still require their own JS TypeScript libraries. They are justified dependencies, not permission to keep two interchangeable generic TS backends. [Shutdown report](https://github.com/microsoft/TypeScript/issues/64742).

**Kotlin coverage is explicitly structural under decision 50.** Provide symbol outlines and syntax-aware search; exclude resolved cross-file definitions/references. The official README's Apache badge coexists with an explicit statement about proprietary Air/Fleet parts. The older OSS server is deprecated. A community MIT fork exists, but the inspected repository metadata showed its last push in November 2024 and no published GitHub releases; this does not establish a maintained modern-Kotlin replacement. These servers are excluded from the baseline, and Mirket will not maintain its own semantic server. Select and qualify a bounded OSS grammar, expose parse limitations, and never label syntax matches as resolved references. [Community fork](https://github.com/kotlin-community-tools/kotlin-language-server), [official status](https://github.com/Kotlin/kotlin-lsp/blob/main/README.md), [deprecated alternative and classpath execution](https://github.com/fwcd/kotlin-language-server/blob/main/README.md).

**OSS is not passive execution.** rust-analyzer explicitly assumes trusted code and documents execution routes beyond build scripts/proc macros. Similar review is needed for project imports, preprocessor plugins and toolchain discovery in other servers. Preserve the agreed host-owned command/build policy and test that prepared metadata can be consumed without replaying hidden build steps. A private path or disabled pair of flags is not a sandbox. [rust-analyzer security](https://rust-analyzer.github.io/book/security.html), [configuration](https://rust-analyzer.github.io/book/configuration), [Swift indexing behavior](https://docs.swift.org/latest/documentation/userdocs/language-features/).

Decision 66 also governs upstream integrations: bundled engines must not independently check for or install updates during ordinary use. Mirket checks its release source only on explicit check/update requests; check-only operations fetch metadata without release payloads. Decision 67 selects one stable channel, with prereleases available only through explicit exact-version selection and no preview subscription. Verify this separately from decision 53 telemetry controls and from authorized browsing/asset downloads.

### PHP and Ruby qualification corrections

Phpactor documents Linux/macOS and WSL for Windows; an x64 PHP interpreter does not resolve that OS conflict. Remove the proposed Windows ARM64 PHP/Phpactor exception. PHPantom 0.11.1 publishes archives for all three targets, but availability is not readiness. Qualify one passive native backend: disable external-tool execution/autodetection and verify project configuration cannot override that policy; prepare complete built-in stubs during controlled assembly without runtime fetching. Test useful navigation on representative PHP projects. If it passes, omit displaced PHP/Composer runtime responsibilities unless another accepted operation needs them. No new dependency is admitted here. [Phpactor requirements](https://github.com/phpactor/phpactor/blob/master/README.md), [PHPantom release](https://github.com/PHPantom-dev/phpantom_lsp/releases/tag/0.11.1), [configuration](https://github.com/PHPantom-dev/phpantom_lsp/blob/0.11.1/docs/configuration.md), [build script](https://github.com/PHPantom-dev/phpantom_lsp/blob/0.11.1/build.rs).

Ruby LSP assumes trusted workspace code and its ordinary composed-bundle route evaluates project inputs and can install/update dependencies. M0b must qualify a preassembled passive route, plugin/add-on suppression, configuration precedence and useful dependency navigation with execution sentinels. A private Ruby binary alone does not satisfy decision 32. If no useful passive route meets the contract, report the precise project-form conflict before expanding Ruby support. Apply the [host preparation contract](../../proposal.md#host-tool-and-evidence-contracts) to all risky analyzers. [Ruby LSP security](https://shopify.github.io/ruby-lsp/security.html), [composed bundle](https://shopify.github.io/ruby-lsp/composed-bundle.html).

### Analyzer runtimes and external SDK prerequisites

**Source review, 2026-10-24; accepted in decision 74, extending decision 52.** Keep managed analyzers and their redistributable runtimes inside `.mirket`. Allow explicitly selected, user-installed platform SDKs/toolchain headers only where the project's target requires them: Apple SDKs for Swift/C/C++ on macOS, and Windows SDK/MSVC components for Windows Swift and Microsoft-targeted C/C++. This accepts the architecture policy, not permission to install prerequisites, redistribute proprietary SDKs or use global analyzer executables. Other features must work without these development components.

The simplest ownership model has three kinds of inputs, recorded through the existing tool inventory and project selection rather than a new SDK manager:

| Owner | Contents | Contract |
| --- | --- | --- |
| Mirket release | Pinned analyzers; compatible private Node, Java, .NET, Go, Swift and other selected runtime dependencies | Resolve by the release inventory, launch on demand, update only with Mirket. An analyzer runtime is not a promise to build every project targeting that language. |
| User's development environment | Explicit platform SDK/toolset selection needed by the project's target | Read selected headers, libraries and metadata; record path, version and architecture. Never install, update or change global selection implicitly. SDK changes invalidate affected analysis. |
| Project | Source, dependency sources/stubs, generated files, target settings and existing build/index metadata | Consume available inputs. Host-authorized project preparation supplies missing material; navigation does not install dependencies or execute project preparation. |

**Windows finding.** Swift's manual requirements name the Windows SDK and architecture-matched MSVC components. Its landing page and manual specify different SDK baselines; do not turn either into Mirket's exact tested version matrix. The manual also describes Python, Git and Developer Mode for the wider development workflow, which does not prove all are necessary for our navigation route. Qualify that route before adding prerequisites; no additional general Python or automatic Developer Mode change is proposed. [Swift requirements](https://www.swift.org/install/windows/manual/), [current landing instructions](https://www.swift.org/install/windows/).

Prefer the standalone Microsoft Build Tools component set if it satisfies the selected Swift/C++ workflow; Microsoft documents that the Visual Studio IDE is unnecessary for its command-line C++ tools. The tools' installation and license terms remain user-owned. An exact compatible component set still needs qualification; a new Visual Studio release must not silently replace it. Environment setup, when required, belongs to an explicit host preparation action, and only its selected results enter the analyzer process. Do not modify global PATH or run developer-shell scripts merely to answer a navigation request. [Microsoft command-line tooling](https://learn.microsoft.com/en-us/cpp/build/building-on-the-command-line?view=msvc-170).

**C/C++ finding.** clangd interprets a compilation database as parser settings; it does not need to run each recorded compiler command. It still needs correct target flags and headers. Its documented defaults can discover compilers through PATH and query `xcrun`; its optional query-driver facility executes a selected compiler. Qualify explicit driver identity, resource headers, SDK and include paths without uncontrolled discovery or query-driver execution. Preserve the original compilation database and use an owned derived configuration if necessary. A missing sysroot produces incomplete analysis, not a silent switch from MSVC semantics to MinGW. Apple SDK use for C/C++ is included in decision 74. [clangd compile-command behavior](https://clangd.llvm.org/design/compile-commands).

**Project loading remains the harder integration gate.** Decision 32 already assigns preparation to the host. The following findings explain why private packaging and prepared files alone do not establish compliance:

| Backend | Evidence | Required integration proof |
| --- | --- | --- |
| SourceKit-LSP | Tagged Swift 6.4.1 documentation exposes background preparation/build settings and lets workspace configuration override initialization options. | Disable background preparation and verify effective configuration, manifest loading, macros and build-server startup. Consuming a prepared index must not silently regenerate it. A supported passive route remains unproven. [Tagged configuration](https://github.com/swiftlang/sourcekit-lsp/blob/swift-6.4.1-RELEASE/Documentation/Configuration%20File.md) |
| rust-analyzer | Upstream explicitly lists project configuration/toolchain overrides alongside macros and build scripts as execution routes. | Prove explicit private toolchain and metadata loading without project executable overrides or implicit Cargo preparation. Two disabled settings are not sufficient evidence. [Upstream execution model](https://rust-analyzer.github.io/book/security.html) |
| csharp-ls / MSBuild | csharp-ls requires .NET 10 or later. Microsoft warns that even opening an MSBuild project can execute extended build logic. | Qualify project loading with custom targets, imports, analyzers and generators disabled. Prepared files are insufficient if opening the project executes again. No owned MSBuild interpreter or replacement C# semantic engine is proposed. [Server requirements](https://github.com/razzmatazz/csharp-language-server/blob/main/README.md), [MSBuild behavior](https://learn.microsoft.com/en-us/visualstudio/msbuild/msbuild-security-best-practices?view=vs-2022) |
| JDT LS | Current README requires Java 21 or later to run, distinguishes supported project Java versions, and advertises Gradle/Maven import and automatic Maven annotation processing. | Use a private compatible JDK and demonstrate imports do not execute build logic, processors or downloads. Prepared classpaths/dependencies must remain data. Do not infer project target compatibility from the server's JDK alone. [Server requirements/features](https://github.com/eclipse-jdtls/eclipse.jdt.ls/blob/main/README.md) |

Other selected language servers retain the same per-backend loading checks from the navigation table. Broad language coverage remains the target; no new semantic downgrade is accepted here. If a backend cannot meet decision 32 through an upstream-supported integration, return that concrete conflict for a product choice. Do not grow a parallel project loader, custom semantic engine or sandbox framework to preserve an unverified promise.

The release qualification fixtures must cover an absent SDK, two conflicting installed SDKs, wrong architecture, SDK replacement, stale build metadata, missing generated code, and workspace configuration attempting to override the private executable or preparation policy. Trace process starts and project writes; a successful definition lookup alone does not prove passive loading. Test positive definitions/references on the supported native target, preserve source revisions, and report unresolved dependencies. Windows Swift support does not imply Apple-framework navigation on Windows.

This review executed no analyzer, SDK installer or project build. Decision 74 accepts the SDK policy; exact compatible versions, private Swift relocation and actual passive startup remain qualification gates. Next specify release operating contracts without reopening the accepted runtime topology.

### Offline SQL navigation qualification

**Research conclusion, 2026-10-24:** prefer SQLGlot's pure-Python library with an explicit schema snapshot and a bounded navigation adapter. This is a candidate implementation of accepted decision 51, not a new accepted dependency or a claim that navigation already works. It preserves both PostgreSQL and SQLite coverage. No upstream package was installed or executed; the findings below come from tagged source, upstream documentation and package metadata.

SQLGlot already supplies the parsing, scope and schema-qualification machinery that would otherwise require custom semantic work. Its tagged README lists both dialects, and its base package has no mandatory third-party runtime dependencies. The tradeoff is a private Python process for SQL analysis and owned source-index integration; SQLGlot is not a ready-made definitions/references server. [Tagged README](https://github.com/tobymao/sqlglot/blob/v30.24.0/README.md), [MIT license](https://github.com/tobymao/sqlglot/blob/v30.24.0/LICENSE), [release metadata](https://pypi.org/pypi/sqlglot/30.24.0/json).

| Alternative | Evidence | Fit judgment |
| --- | --- | --- |
| SQLGlot 30.24.0 | Explicit schema input, scope analysis, qualification and identifier-position metadata. Pure-Python wheel available. | Leading candidate: reuse semantic machinery and the already planned private Python runtime. Still needs original-source mapping and a reverse reference index. [Qualification API](https://github.com/tobymao/sqlglot/blob/v30.24.0/sqlglot/optimizer/qualify.py), [identifier positions](https://github.com/tobymao/sqlglot/blob/v30.24.0/sqlglot/expressions/core.py) |
| Postgres Language Server, native | Official documentation says schema-aware object resolution depends on a database; disabling the connection retains non-schema features. | Does not meet the offline schema contract through this route, and does not cover SQLite. [Database requirements](https://pg-language-server.com/main/guides/configure_database/) |
| Postgres Language Server, WASM | A genuine offline alternative: accepts an exported schema. The documented LSP method list includes completion/hover/diagnostics but not definitions/references. | Do not dismiss it as requiring a live database, but it still leaves the selected navigation and SQLite gaps. Adding another dialect-specific backend is not justified here. [Browser API](https://pg-language-server.com/main/guides/browser/) |
| sqls / joe-re SQL Language Server | Inspected documentation centers database connection/adaptor configuration, including SQLite files; sqls also advertises execution actions. No complete supplied-schema-only navigation contract established. | No demonstrated simplification for this requirement. Do not use a live/local database file as a hidden schema source. [sqls](https://github.com/sqls-server/sqls), [SQL Language Server](https://github.com/joe-re/sql-language-server) |
| sqlc | Supports PostgreSQL/SQLite and a built-in offline analyzer; reads DDL and migration files without running migrations. Its documented output workflow is generated code. | Credible static-analysis alternative, but the reviewed interface does not establish arbitrary cursor definitions/references with source spans. Adapting compiler internals or generator output offers no proven advantage over SQLGlot. [Analysis modes](https://docs.sqlc.dev/en/latest/howto/generate.html), [DDL/migrations](https://docs.sqlc.dev/en/latest/howto/ddl.html) |
| Rust sqlparser | Apache-2.0 parser; explicitly avoids SQL semantic analysis and documents incomplete source-span coverage. | Native placement alone does not compensate for owning table/column binding semantics. Do not build a custom resolver on it merely to avoid the accepted Python runtime. [Maintainer scope](https://github.com/apache/datafusion-sqlparser-rs) |
| Sqruff | Its LSP documents diagnostics and formatting. | Useful lint/format tool, not evidence of the required symbol navigation. Do not add it for this task. [Editor capabilities](https://playground.quary.dev/docs/usage/editors/) |

**Important source finding:** SQLGlot's `qualify` supports strict column validation, and the tagged implementation reports unresolved/ambiguous columns. However, `lineage()` disables that validation by default, and its traversal includes single-source inference when a schema is absent. A plausible lineage graph therefore cannot be treated as proof of a declared column definition. Use strict qualification and explicitly validate each reported base object against the supplied catalog. Unknown tables/columns remain unresolved, even when there is only one plausible source. Do not equate successful parsing or an absence of errors with complete resolution across every statement type. [Validation implementation](https://github.com/tobymao/sqlglot/blob/v30.24.0/sqlglot/optimizer/qualify_columns.py), [lineage behavior](https://github.com/tobymao/sqlglot/blob/v30.24.0/sqlglot/lineage.py).

**Proposed integration contract:**

1. **Declare the schema context.** Select dialect, project query roots and canonical declarative schema files or a supplied schema snapshot. Parse supported `CREATE TABLE`/view declarations as data and retain their file/revision/spans. Multiple files may comprise one declared snapshot; duplicate/conflicting declarations are errors, not “last file wins.” Do not infer the currently deployed schema from arbitrary migration history. Migration-only projects can supply a current snapshot through their host-owned workflow; Mirket neither runs it nor creates a database. No new migration-state interpreter is proposed.
2. **Preserve namespace rules.** Treat PostgreSQL schemas/search-path context and SQLite namespace mappings as explicit analysis inputs where needed. Preserve quoted identifiers and dialect-specific case rules. Never assume a connection's environment, session settings, attached databases or credentials. Unqualified names with unresolved context remain ambiguous. A supplied snapshot is evidence of that snapshot, not of a live database's current state.
3. **Keep original positions separate from transformed SQL.** Parse original text, retain identifier identity and original spans, then qualify a copy for resolution. Normalized identifiers and expanded stars are not new source locations. A definition target must have a genuine declaration origin; a snapshot without source coordinates cannot yield an invented file location. Verify character/byte/UTF-16 conversions, inclusive token ends, CRLF and Unicode. SQLGlot stores token position metadata, but reliable mapping through transformations remains an adapter test obligation. [Parser](https://github.com/tobymao/sqlglot/blob/v30.24.0/sqlglot/parser.py), [position storage](https://github.com/tobymao/sqlglot/blob/v30.24.0/sqlglot/expressions/core.py).
4. **Build navigation from proven bindings.** Return the nearest relevant declaration for aliases/CTEs and the declared schema object for base tables/columns. Keep optional upstream lineage distinct from direct definition/reference relationships. Store reverse references in the existing checkout-scoped derived navigation cache, keyed by symbol, dialect, schema revision and source revision. Report the indexed scope and incomplete files; do not call a partial scan “all references.” Invalidate on schema edits, changed files and branch/worktree changes. No new database service, LSP daemon or parallel cache format is proposed.
5. **Bound execution and unknowns.** Use the existing on-demand helper lifecycle and privately packaged Python; SQL analysis must not import/start the ML document pipeline just to parse SQL. Use SQLGlot's parsing/qualification APIs, not its evaluator, SQL execution, model inference or HTML lineage visualization. Do not evaluate templates, ORM code, stored-procedure bodies or migration scripts. Dynamic SQL, unsupported constructs and missing schemas return partial/unsupported status, while any independently verified results retain their provenance. Embedded SQL requires a valid host-language source map; no guessed spans.
6. **Keep the adapter narrow.** Mirket owns schema selection/provenance, original-span mapping, result filtering, reverse indexing and lifecycle. SQLGlot owns dialect parsing and supported semantic qualification. Do not patch around unsupported semantics by accumulating a custom SQL resolver. Qualify common SELECT/JOIN/CTE/subquery cases first and check DML, views, recursive CTEs and dialect-specific features individually; parsing support is not evidence of navigation support. Report remaining gaps under decision 51 rather than silently promising complete SQL semantics.

**Concrete dependency candidate:** `sqlglot==30.24.0`, MIT, base pure-Python distribution only, with no `c`, `rs` or `dev` extras. PyPI lists `sqlglot-30.24.0-py3-none-any.whl`, SHA-256 `9488828cc018453b7a3ba01e65d2749c9a731314c14583695163ca1f358203fe`; its metadata requires Python ≥3.9 and lists dependencies only behind extras. The hash is published metadata, not a downloaded-file verification. This introduces no extra native SQL extension or database runtime, but adds a Python library/process lifecycle, version-sensitive optimizer APIs and adapter maintenance. It has not been approved or added as a production dependency. [Exact metadata](https://pypi.org/pypi/sqlglot/30.24.0/json).

The universal wheel is an availability lead for macOS ARM64 and both Windows targets. Run it with the selected private interpreter; if that interpreter is x64 on Windows ARM64, this worker is emulated too and must be reported accordingly. Do not claim native execution merely because the package itself is architecture-neutral. Avoid adding a second interpreter solely for SQL unless measured requirements justify it. Neither latency nor memory use has been measured here.

**Discriminating fixtures to run after implementation authorization and dependency admission:**

| Fixture | Required result |
| --- | --- |
| Supplied tables `a(id)` and `b(id)`; `SELECT id FROM a JOIN b ON a.id = b.id` | Qualified join operands resolve separately; unqualified projection `id` is ambiguous, with no fabricated single definition. |
| `SELECT missing FROM a` where supplied `a` lacks that column; then repeat without a schema | No verified column definition in either case. Sole-table lineage inference must not manufacture a declaration. |
| Nested/aliased CTEs, correlated subquery and repeated table aliases in different scopes | Correct binding identity and original spans; direct definitions distinguish local aliases from base-column lineage. |
| PostgreSQL quoted identifiers and conflicting schemas; SQLite namespace/case examples | Dialect-correct resolution or explicit missing context; no cross-dialect normalization shortcut. |
| `SELECT *`, view chains, JOIN USING/NATURAL and recursive CTE | Distinguish explicit source references from derived expansions; unsupported resolution stays incomplete. Never attach synthesized names to nonexistent source spans. |
| UPDATE/INSERT/DELETE and unsupported syntax/templates | Report only actually proven table/column bindings; no inference of complete DML coverage from SELECT tests and no template execution. |
| Schema/query edit, stale snapshot, rename and branch/worktree switch | Old locations and reverse references do not survive as current evidence; schema provenance and indexed coverage are visible. |
| Non-ASCII identifier, emoji in a preceding comment, CRLF and multiple statements | Returned ranges select exactly the original identifier in both host clients. |
| Connection-looking config/env, malicious template and migration side-effect sentinel | No database connection, credential-file lookup, project code execution or automatic package download on the exercised analysis path. |

The next useful evidence is an executed bounded qualification slice, not another round of generic SQL-tool searches. That execution remains after complete-plan agreement and dependency authorization. This review establishes a concrete candidate and integration boundary; it does not mark decision 51 implemented or fully verified.



## Preserved source: planning/platform-qualification.md

Source file SHA-256 before amendment: `6a2bc22b37d988cef5207b04c24ebc4e3d001d6733ad188d640e32c543141bb6`.

## Code intelligence

The accepted language scope is unchanged. Native availability is only the first gate: project loading must also respect decision 32's read-only analyzer policy. Build scripts, proc macros, project plugins, generators, dependency installation and implicit toolchain downloads cannot become hidden prerequisites. Host-authorized preparation must supply needed metadata.

The [focused analyzer review](../../toolchain-proposal.md#analyzer-runtimes-and-external-sdk-prerequisites) documents decision 74’s accepted extension of decision 52: selected user-installed Apple SDKs for Swift/C/C++, and Windows SDK/MSVC components where Windows Swift or Microsoft-targeted C/C++ requires them. The prerequisite policy is accepted; exact compatible versions remain unqualified. Private server runtimes remain Mirket-owned. SourceKit-LSP configuration precedence, MSBuild project opening, JDT LS import/processing and rust-analyzer executable overrides require actual passive-loading tests; supplying a prebuilt index alone does not resolve those gates.

| Coverage / candidate | macOS ARM64 | Windows x64 | Windows ARM64 | Evidence and gap |
| --- | --- | --- | --- | --- |
| JS/TS — TypeScript native LSP | N/G | N/G | N/G | TypeScript 7.0.2 registry metadata lists Darwin ARM64 and Windows x64/ARM64 packages. Verify actual LSP behavior, lifecycle, disabled type acquisition and framework interoperability. [Registry metadata](https://registry.npmjs.org/typescript/7.0.2) |
| Python — Pyright | J/G | J/G | J/G | Private Node route. Qualify imports, source locations and environments without resolving the server from a global installation. [Upstream](https://github.com/microsoft/pyright) |
| Rust — rust-analyzer | N/G | N/G | N/G | 2026-10-19 release includes all three targets. Cargo metadata and workspace configuration remain execution-policy boundaries. [Release](https://github.com/rust-lang/rust-analyzer/releases/tag/2026-10-19) |
| Go — gopls + private Go | N/B | N/B | N/B | Go 1.27.3 download inventory lists all three SDK archives; gopls version/build and disabled toolchain/module downloads require qualification. [Go downloads](https://go.dev/dl/?mode=json), [gopls](https://github.com/golang/tools/tree/master/gopls) |
| C/C++ — clangd | B/G | N/G | B/G | clangd 23.1.0 has a Windows archive; LLVM 23.1.4 lists Windows ARM64 archives, a native packaging lead whose contents still need inspection. Qualify all target builds, compilation databases, headers/SDK requirements and disabled uncontrolled query-driver execution. [clangd release](https://github.com/clangd/clangd/releases/tag/23.1.0), [LLVM release](https://github.com/llvm/llvm-project/releases/tag/llvmorg-23.1.4) |
| Java — JDT LS + OpenJDK | J/G | J/G | J/G | Server direction is retained; an exact private JDK distribution for each architecture is not established here. Adoptium API retrieval returned HTTP 403, so no artifact conclusion is drawn from that query. Qualify native JDK packages and passive project import. [JDT LS](https://github.com/eclipse-jdtls/eclipse.jdt.ls) |
| C# — csharp-ls + .NET | N/G | N/G | N/G | .NET 10 SDK download page provides all three target routes. Resolve csharp-ls compatibility and prove project loading does not run unauthorized MSBuild tasks, analyzers or generators. [.NET downloads](https://dotnet.microsoft.com/en-us/download/dotnet/10.0), [server](https://github.com/razzmatazz/csharp-language-server) |
| Swift — SourceKit-LSP | N/G | N/G | N/G | Windows Swift instructions provide x64/ARM64 installers but require Visual Studio/Windows SDK components. Decision 74 accepts explicit user-installed Windows SDK/MSVC prerequisites. Qualify the private toolchain and selected SDK/toolset compatibility; Apple-framework navigation remains Apple-specific. [Swift Windows requirements](https://www.swift.org/install/windows/), [accepted SDK policy](../../decisions.md) |
| Ruby — Ruby LSP | B/G | N/G | N/G | RubyInstaller lists x64 and ARM64 archives, including distributions without Devkit. Exact private Ruby/gem closure and Mac packaging remain unresolved. Ordinary Ruby LSP trusts/executes project code; M0b must prove a useful preassembled passive route with no Bundler install, add-on or project-code execution. [Downloads](https://rubyinstaller.org/downloads/), [Ruby LSP](https://github.com/Shopify/ruby-lsp) |
| PHP — native PHPantom candidate | N/G | N/G | N/G | 0.11.1 lists all three native archives; no runtime test or dependency admission. Qualify passive execution/configuration, stubs, source spans and actual navigation. Phpactor requires WSL on Windows, so the former x64 PHP exception is removed. [Release](https://github.com/PHPantom-dev/phpantom_lsp/releases/tag/0.11.1), [Phpactor requirements](https://github.com/phpactor/phpactor/blob/master/README.md) |
| Dart — SDK analysis server | N/G | N/G | N/G | Official SDK archive lists platform downloads; a HEAD request confirmed the 3.13.7 Windows ARM64 ZIP exists. No archive contents or analyzer execution were tested. Qualify private configuration, telemetry settings and no implicit pub/toolchain fetching. [SDK archive](https://dart.dev/get-dart/archive), [ARM64 artifact](https://storage.googleapis.com/dart-archive/channels/stable/release/3.13.7/sdk/dartsdk-windows-arm64-release.zip) |
| Bash / PowerShell | J/G + N | J/G + N | J/G + N | Bash LSP uses Node; PowerShell 7.6.8 publishes all three native archives. Resolve Editor Services compatibility and disable profile loading; do not run project shell scripts for analysis. [PowerShell release](https://github.com/PowerShell/PowerShell/releases/tag/v7.6.8) |
| Vue/Svelte/Astro | J/G | J/G | J/G | Qualify exact language-server closures, embedded languages, preprocessors and their TypeScript versions. JavaScript packaging does not establish passive plugin behavior. See [language candidates](../../toolchain-proposal.md). |
| Kotlin; markup/config/build files | B/G | B/G | B/G | Kotlin remains structural only. Select bounded parsers/grammars per accepted file type; compile native parsers for each target where needed. No indiscriminate all-language bundle or invented semantic coverage. |
| SQLite/PostgreSQL SQL | J/G | J/G | J/G | SQLGlot 30.24.0 base pure-Python wheel is the [leading offline candidate](../../toolchain-proposal.md#offline-sql-navigation-qualification). No SQL-native extension is required by that package. Strict bindings, original source ranges, schema snapshots and reverse indexing remain to be implemented/verified; no dependency admitted. Actual process architecture follows the private interpreter, including any selected Windows ARM64 emulation exception. |



## Preserved source: docs/m0-release-qualification.md

Source file SHA-256 before amendment: `4ac7caea5a81576530707fcb6387934bc241186db0a50cb9c65f9a779066d12d`.

## Private release components

The release manifest has one schema with a finite component list. An empty list supports
activation and recovery fixtures. The `documents` component additionally binds the admitted general Python tree,
its notices, the exact SQLGlot and pypdf wheels, and both authored adapters. It is available
only for macOS ARM64; this does not imply Windows runtime qualification.

The fixed layout places the assembled Python tree and its notices beneath
`runtime/python-general/`, the adapter at `adapters/sql_navigation.py`, and the
wheel at `libraries/sqlglot-30.24.0-py3-none-any.whl`. Manifest generation infers
this profile from those optional paths and requires the complete inventory.
The PDF leaf lives at `adapters/pdf_operations.py`, its exact wheel at
`libraries/pypdf-6.21.2-py3-none-any.whl`, and its BSD notice at
`notices/python-documents/LICENSE.pypdf.txt`. A partial documents payload cannot
silently become a core release. The previous SQL-only profile is not accepted.

Regular-file bytes, normalized private modes and the two admitted relative
interpreter link targets participate in release identity. The fixed inventory
includes the required `site-packages` directory with its pinned README and no additional packages; arbitrary extra entries and alternate link targets
are refused. Source files must have the expected owner permissions; staging
normalizes group/other permissions away. Installed files require exact `0600`
or `0700` modes and directories require `0700`. These identity and consistency
checks do not protect against a hostile same-user filesystem mutator.

`tests/documents-installed-smoke.mjs` is the separate fixture for installed SQL
and PDF. It accepts explicit local core, Node, adapter, assembled Python, SQL
script/wheel and PDF script/wheel/admission-receipt inputs; no asset is downloaded and no user installation is changed.
This documents the original documents-only fixture, whose manifest was below
256 KiB. The current distribution permits a 2 MiB manifest and a 512 MiB
payload so the pinned design tree can be represented. The installed combined
design release is described in [local design qualification](../../../docs/m4-design-qualification.md).
Further components require measured inventories and an explicit bounded
distribution-cap revision; native platform/client coverage remains a separate
gate.


The installed SQL fixture passed on macOS 26.5.2 ARM64 with private Node 24.22.1
and the admitted CPython 3.13.17 payload, under OS network denial. It verified
installed-only path resolution, real Unicode navigation, unchanged sources and
payload, aliases/modes, and preservation of the active release when a candidate's
Python bytes or alias target changed. Source and home paths contained spaces and
Unicode. The full installed profile contains 564 regular files and two aliases;
the generated manifest remains below 256 KiB. This is local fixture evidence,
not native Windows, desktop activation or publication qualification.


## Preserved decision 31

### 31. Broad code-language and supporting-file coverage

The user accepted the proposed coverage target: TypeScript/JavaScript including TSX/JSX; Python, Go, Ruby and PHP; Rust, C and C++; Java, Kotlin and C#; Swift and Dart; Bash and PowerShell; SQL with SQLite and PostgreSQL dialects. Also include Vue/Svelte/Astro components, HTML/CSS/SCSS, JSON/YAML/TOML, Markdown, Terraform/HCL, Dockerfile, CMake and Makefiles. Proposal section 5 defines the intended capability levels.

Target symbol search, definitions and references for programming languages, with Kotlin explicitly revised to structural support by decision 50. Supporting files have explicit structural/schema/link capabilities where applicable; do not label structural or text search as complete semantic analysis. SQL analysis uses project files and supplied schemas without database connections; decision 51 clarifies offline schema-aware navigation and incomplete-resolution reporting. Keep original source identity and report incomplete analysis.

Privately package the selected navigation components, start only analyzers required by the current project, bound indexing and release idle resources. Verify each language/file capability and measure combined package size and runtime costs before final dependency selection. Choose the smallest maintained set of analysis tools that satisfies the accepted coverage; this does not approve particular dependencies or adopt Serena wholesale. Scala, Elixir, Lua, R, Julia and Zig are not included by this decision; reconsider additional languages when concrete project needs justify them. No application code or tools are installed during planning.



## Preserved decision 50

### 50. Structural Kotlin support

The user selected structural support for Kotlin under the accepted OSS requirement: symbol outlines and syntax-aware search. Explicitly exclude resolved cross-file definitions and references from Kotlin's coverage commitment. This revises the Kotlin portion of decision 31; other language commitments remain unchanged.

Use a bounded OSS parser/grammar, with its exact selection, license and compatibility still to be qualified. Preserve source identity and revision-bound locations, report parse limitations, and distinguish syntax matches from semantically resolved symbols. Do not bundle a proprietary or deprecated Kotlin language server or create a custom semantic server to preserve the superseded commitment. Any future expansion to full Kotlin semantics requires a separate evidence-backed decision.

Verify representative declarations, nested scopes, incomplete syntax and changed files. Capability reports must clearly identify structural coverage and must not present same-name matches as resolved cross-file references. This is a planning decision, not proof that a parser has been implemented or tested.



## Preserved decision 51

### 51. Offline schema-aware SQL navigation

The user selected offline schema-aware navigation for SQLite and PostgreSQL. Resolve table/column references using project SQL and explicitly supplied schemas where the available information permits. This clarifies the SQL portion of decision 31 and preserves its navigation intent.

Do not connect to databases, introspect live systems or execute migrations to obtain schema state. Preserve dialect, source identity and revision-bound locations. Missing or ambiguous schema information, unsupported syntax and dynamic SQL must produce clearly incomplete results; syntax/search results must remain distinguishable from resolved references. Do not imply complete runtime query resolution.

The exact OSS parser/navigation engine remains to be qualified; this decision accepts the behavior, not a particular dependency or a custom SQL analyzer. Prefer an established engine with a narrow adapter and demonstrate its offline behavior before final selection. Verify local table/column definitions and references, aliases and scope collisions, missing schemas, changed files and dialect differences. A fixture with ambiguous names must not receive fabricated definitive resolution.



## Preserved decision 52

### 52. Explicit Apple SDK prerequisite for Swift

The user selected the recommended Apple SDK exception. Keep Mirket's Swift analyzer privately packaged, while allowing it to use a compatible Apple SDK from an explicitly selected user-installed Xcode. This is a narrow exception to fully self-contained packaging, not permission to resolve managed analyzers from PATH or to use arbitrary host tools.

Do not bundle proprietary Apple SDKs as OSS package contents, automatically install Xcode/SDKs, change global developer-tool selection or launch hidden project builds. Scope SDK selection to the project/operation and pass it explicitly to the owned analyzer. Report missing or incompatible prerequisites and incomplete navigation clearly. Other Mirket capabilities must remain usable without Xcode.

Preserve the Swift navigation commitment, subject to qualified toolchain/SDK compatibility and available project metadata. Required build/index preparation remains a host-owned, authorized project action under decision 32. Verify private analyzer resolution, supported SDK combinations, conflicting global tool selection, missing/changed SDK paths and stale or absent build metadata. Exact versions and Xcode-project integration remain technical qualification work; this acceptance does not prove semantic navigation already works.



## Preserved decision 73

### 73. Separate private Office and general Python runtimes — superseded by decision 79

The user accepted two private Python runtime owners: LibreOffice's matched Python/UNO for Office automation, plus one general Python environment for extraction, PDF libraries and SQL libraries if admitted. This resolves the sharing question in decision 49. Do not install general third-party packages into Office's interpreter or load its native UNO bridge into an unrelated interpreter.

Keep Office and its matching Python native for each release target. The general worker is proposed native on macOS ARM64 and Windows x64, with an x64 worker on Windows ARM64 subject to decision 71's explicit compatibility/resource qualification. This decision accepts the ownership split, not an untested emulation result or specific interpreter distribution/version. Keep architecture-compatible dependencies within each process.

Use the existing job lifecycle and complete private `.mirket` payload. Launch only needed processes, and import only the libraries required by the operation; ordinary PDF/SQL tasks must not initialize the ML pipeline. No third general interpreter, always-running Office service, duplicate fallback interpreter or new scheduler is selected. The additional distribution space, patch maintenance and runtime inventory are accepted tradeoffs; actual size and performance remain to be measured.

Exact interpreter/library pins, redistribution closure, private Windows Office relocation, policy/font handling and all three target workflows remain qualification work. SQLGlot is still a proposed dependency. Existing limits on global installations, host configuration changes and implementation before complete-plan agreement remain unchanged. This former packaging contract is superseded by the [current Office ownership contract](../../toolchain-proposal.md#host-owned-office-and-general-python).



## Preserved decision 74

### 74. Explicit platform SDK prerequisites

The user accepted extending decision 52: allow explicitly selected, user-installed Apple SDKs for Swift/C/C++, and Windows SDK/MSVC components for Windows Swift and Microsoft-targeted C/C++. Mirket retains privately packaged analyzers and their redistributable runtimes. External SDKs are declared project prerequisites, not global analyzer fallbacks or bundled OSS components.

Scope selection to the project/operation and record the selected path, version and architecture. Do not install or update SDKs, modify global PATH/developer-tool settings, enable Developer Mode or redistribute proprietary SDKs through this acceptance. Missing or incompatible prerequisites limit only the affected navigation; other capabilities remain usable. Windows Swift support does not imply Apple-framework navigation on Windows.

Project preparation remains host-owned under decision 32. SDK access does not authorize hidden builds, executable project configuration, generators, macros or dependency installation. Honor existing host authorization for concrete preparation actions. Qualify each backend's effective configuration and loading behavior rather than assuming prepared metadata prevents further execution.

Exact compatible SDK/toolset versions, private toolchain relocation and all three platform workflows remain unverified. This accepts the architecture policy, not an installation, dependency pin or implementation start. See the [analyzer review](../../toolchain-proposal.md#analyzer-runtimes-and-external-sdk-prerequisites). Next specify release operating contracts.
