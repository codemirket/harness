# Platform qualification — research matrix

**Historical broad-scope matrix, 2026-10-24.** Decisions 84–93 change the release surface in the [proposed consolidated plan](implementation-plan.md). Browser and design rows remain candidate research after the private-browser decision; document/OCR and broad analyzer rows are not current release requirements. Nothing in this matrix proves native qualification.

Evidence reviewed 2026-10-24, with Office ownership updated under [decision 79](decisions.md#decision-79--host-owned-office-workflows). Scope: macOS ARM64, Windows 11 x64 and Windows 11 ARM64. The original matrix is a source/release review, not a passing runtime test report; observed versions identify evidence rather than the release lockfile. Current admissions and qualification reports own later runtime evidence. Office rows below describe host capability checks, not a bundled Mirket payload.

Codex Desktop is the sole first-release host target under [decision 80](decisions.md#decision-80--codex-desktop-is-the-sole-first-release-host). Historical source reviews do not add another host target.

## Recommended direction

Keep the native Rust core and private Node adapters on all three targets. Most inspected foundational tools have native artifacts. Use decision 71 only for specific process boundaries: Windows x64 browsers are the clearest exception candidate; the Docling/PyTorch worker is the largest likely exception. Do not emulate the whole harness to accommodate a helper.

Office creation, editing, recalculation, rendering and export use the tools available in the actual Codex host under decision 79. Mirket supplies guidance, workflow and verification and reports concrete missing capabilities; no identical self-contained Office support across Codex sessions/platforms is promised. Private Office/UNO packaging is no longer a release gate. Retained PDF/SQL and separate extraction/design qualification are unchanged.

## Reading the tables

| Mark | Meaning |
| --- | --- |
| N | Native upstream artifact or documented native runtime found. Does not mean Mirket integration passed. |
| J | JavaScript, source or data that can be shared, subject to its runtime and resolved native dependencies. |
| B | Controlled native build/integration needs qualification; no complete suitable private package established here. |
| E | Proposed privately packaged x64 process on Windows ARM64; actual emulation remains untested. |
| G | Unresolved capability, packaging or prerequisite gap. |

Combined marks preserve both facts, for example **N/G** means a native artifact exists but the required workflow remains unresolved. A missing artifact in an inspected release does not establish that native compilation is impossible. Managed payload rows require exact source/license closure and actual output tests before release. Host Office rows require actual host capability and saved-output checks; they do not imply redistributed Office components.

## Core, installation, clients and memory

| Component | macOS ARM64 | Windows x64 | Windows ARM64 | Evidence and remaining qualification |
| --- | --- | --- | --- | --- |
| Rust CLI/MCP core | N/B | N/B | N/B | All three Rust target triples are documented with host tools. Build Mirket and its C dependencies for each target; platform support alone says nothing about our installer or process supervisor. [Rust targets](https://doc.rust-lang.org/rustc/platform-support.html) |
| Private Node LTS | N | N | N | Node 24.22.1 distribution lists Darwin ARM64 and Windows x64/ARM64 archives. Qualify compiled adapters and target-specific native modules without global Node/npm fallback. [Distribution checksums](https://nodejs.org/dist/v24.22.1/SHASUMS256.txt) |
| Private general Python | N/G | N/G | N/G + E/G | One general interpreter per target where retained PDF/extraction requires it; decision 79 removes the separate Office-matched runtime. The [macOS admission receipt](../packaging/admission/m3-python-macos.md) owns the selected CPython 3.13.17 tree and local evidence. Windows closure, relocation, rights and any x64-worker exception require their own qualification. An ARM64 interpreter cannot load x64 extensions. |
| Home-folder installation | G | G | G | Shared Rust setup/update owner with thin shell/PowerShell bootstrap remains the plan. The [operating contract](proposal.md#installation-activation-and-recovery-contract) records decision 75’s accepted staged updates and maintenance-window activation. Stable entry points, signing/trust, DLL resolution and actual recovery still need platform qualification. Do not require a global package manager or admin installer. |
| Codex Desktop integration | N/G | N/G | N/G | Official Windows deployment documentation offers x64 and ARM64 clients; local stdio MCP is documented. Verify actual launch/configuration/tool use on each target, preserving client settings and policies. [Windows deployment](https://learn.chatgpt.com/docs/enterprise/windows-deployment), [MCP](https://developers.openai.com/codex/mcp) |
| SQLite/FTS5 + sqlite-vec | B | B | B | Embed architecture-matched C code in Rust. sqlite-vec v0.1.9 lists Mac ARM64 and Windows x64 artifacts but no Windows ARM64 prebuilt artifact; its Rust build compiles sqlite-vec.c. Qualify native ARM64 compilation instead of loading an x64 extension into native memory code. [Release](https://github.com/asg017/sqlite-vec/releases/tag/v0.1.9), [Rust build](https://github.com/asg017/sqlite-vec/blob/v0.1.9/bindings/rust/build.rs) |
| FastEmbed + CPU ONNX Runtime | N/G | N/G | N/G | ONNX Runtime v1.31.0 supplies all three native archives. Select a compatible Rust binding and private library loading strategy, bundle the exact embedding model/tokenizer, and prevent incidental model/binary downloads. Verify multilingual retrieval and resource use. [Release](https://github.com/microsoft/onnxruntime/releases/tag/v1.31.0), [FastEmbed manifest](https://github.com/Anush008/fastembed-rs/blob/main/Cargo.toml) |
| Guidance, curated memory and backups | J/B | J/B | J/B | Guidance data is portable; locking, path identity, atomic persistence and backup/restore behavior are platform-specific implementation obligations. No additional memory service is proposed. |

Codex Desktop and OS prerequisites are external installation targets, not OSS tools redistributed inside Mirket. Runtime redistributables, model assets and other packaged files require their own review.

Decision 76 simplifies the [release workflow](proposal.md#simple-release-and-installation-workflow) to standard hosting, checksums and platform verification, without a custom manifest-signing or channel protocol. Native signing, publisher/provider selection, archive extraction and actual clean-machine installation remain release-preparation checks. Checksums do not independently authenticate their publisher.

## Browser, design, media and documents

| Component | macOS ARM64 | Windows x64 | Windows ARM64 | Evidence and remaining qualification |
| --- | --- | --- | --- | --- |
| Playwright Chromium | N | N | E | v1.64.1 registry lists Mac ARM64 and Windows x64/win64 Chromium artifacts. Propose an x64 browser process on ARM64, including architecture-matched helpers. Test its private profiles, headless/visible modes and cleanup. [Tagged registry](https://github.com/microsoft/playwright/blob/v1.64.1/packages/playwright-core/src/server/registry/index.ts), [browser manifest](https://github.com/microsoft/playwright/blob/v1.64.1/packages/playwright-core/browsers.json) |
| OpenPencil file/SDK route | J/G | J/G | J/G | v0.16.0 publishes CLI/core JavaScript and CanvasKit dependencies; desktop release has native Mac ARM64 and Windows x64/ARM64 installers. Qualify file-mode APIs on private Node first. Desktop WebView2 Evergreen ownership/update behavior is unresolved; desktop availability does not prove private file-mode parity. [CLI manifest](https://github.com/open-pencil/open-pencil/blob/v0.16.0/packages/cli/package.json), [release](https://github.com/open-pencil/open-pencil/releases/tag/v0.16.0), [requirements](https://openpencil.dev/getting-started) |
| Sharp/libvips | N | N | N | Maintainer documentation lists all three prebuilt targets. Resolve the correct platform closure during assembly, verify image codecs and controlled fonts; no install-time download during user jobs. [Installation/targets](https://sharp.pixelplumbing.com/install/) |
| resvg | N | N | N | v0.49.0 lists macOS aarch64 and Windows aarch64/x86_64 archives; resvg-js also publishes relevant native packages. Choose one integration, rather than redundant CLI/binding fallbacks. [Release](https://github.com/linebender/resvg/releases/tag/v0.49.0), [binding metadata](https://registry.npmjs.org/@resvg/resvg-js/2.6.2) |
| D2 + ELK | N/G | N/G | N/G | D2 v0.9.0 lists all three platform archives. Qualify selected ELK layout, fonts, source editability and output; do not infer a working layout integration merely from the D2 executable. [Release](https://github.com/d2lang/d2/releases/tag/v0.9.0) |
| Readability, SVGO, Style Dictionary, Motion/Anime | J | J | J | Shared modules run on the selected Node/browser route. Resolve the exact package closure and exercise extraction, tokens, SVG and motion outputs. No extra daemon or per-library runtime is proposed. See the [component proposal](toolchain-proposal.md). |
| HyperFrames | J/G | J/G | J/G + E browser | Published rendering packages use Node and Puppeteer. Qualify Windows execution, browser selection and FFmpeg integration. Reusing the private browser payload is desirable but not proven; Puppeteer/Playwright version compatibility must be tested before committing to one browser copy. [Tagged package source](https://github.com/heygen-com/hyperframes/tree/v0.8.192) |
| FFmpeg/ffprobe | B | B | B | Controlled native builds remain the proposal. Exact encoders, codec licenses, private dependencies and build recipes are unresolved. No native Windows ARM64 media build was verified by this review; do not silently designate it an x64 exception. |
| Host Office creation/editing/recalculation/render/export | G | G | G | Inspect actual Codex tools, supported formats, effect controls and saved artifacts separately per platform/session. Mirket owns guidance and verification, not the Office runtime. Missing capabilities are explicit gaps; no bundled engine, replacement backend or universal host-support claim. See the [host Office contract](toolchain-proposal.md#host-owned-office-and-general-python) and decision 79. |
| Typst | N | N | N | v0.15.1 lists all three native archives. Bundle selected packages/fonts and prove offline typesetting, pagination and exported artifacts. [Release](https://github.com/typst/typst/releases/tag/v0.15.1) |
| pypdf + PDFium previews/page operations | J + N | J + N | J + N | pypdf is Python code; pypdfium2 5.14.0 publishes Mac ARM64, Windows AMD64 and Windows ARM64 wheels. Use a wheel matching its worker interpreter. Rendering/page operations do not establish existing-paragraph editing. [PDFium binding release files](https://pypi.org/pypi/pypdfium2/5.14.0/json) |
| Source-first PDF editing / bounded direct edits | G | G | G | Decision 72 accepts source-based repagination, bounded direct imported-PDF edits and explicit reconstruction. The [backend review](toolchain-proposal.md#pdf-backend-feasibility-and-proposed-scope-clarification) defines the qualification cases. General imported-PDF cross-page reflow is no longer required; direct-edit backend/output verification remains open. |
| Narrow Docling layout/table pipeline | N/G | N/G | E/G | Inspected torch 2.14.1 and torchvision 0.29.1 release files have Mac ARM64 and Windows AMD64 wheels, but no Windows ARM64 wheels. A complete x64 Python worker is the proposed exception candidate; exact mutually compatible Docling/Python/model versions are still unresolved. [Docling installation](https://docling-project.github.io/docling/getting_started/installation/), [torch files](https://pypi.org/pypi/torch/2.14.1/json), [torchvision files](https://pypi.org/pypi/torchvision/0.29.1/json) |
| Tesseract + seven OCR languages | B | B | B | Upstream documents source builds and external Windows installers, not our private three-target package. Qualify engine/native libraries and bundled traineddata for all seven accepted languages and mixed-language fixtures. [Upstream installation](https://tesseract-ocr.github.io/tessdoc/Installation.html) |
| Fonts/model assets | J/G | J/G | J/G | Architecture-independent files still need exact rights, coverage and integration. Explicit packaged/project font resolution remains required for managed browser, Typst, SVG and video renderers. Host Office font availability/substitution must be inspected through the actual host tool and reported; unavailable fonts do not count as verified fidelity. |

The x64 extraction-worker proposal does not force all document tools to x64. Use architecture-compatible libraries within each managed process; cross-process communication belongs to the existing adapter contract. Keep one general Python per target where required. Office uses host tools under decision 79 and adds no private UNO bridge or second interpreter to Mirket.

## Code intelligence

Decision 81 defines the exclusive first-release targets: Python, Rust, TypeScript, JavaScript, C, C++, Bash, JSON, HTML, JSX, TSX, CSS, TOML, XML, PowerShell, Vue, Angular, Astro, SVG and YAML. SQL navigation uses the five qualified bounded profiles `postgres`, `sqlite`, `mysql`, `tsql` and `duckdb`; per-dialect limits remain explicit. The earlier wider matrix is [historical research only](reviews/2026-10-25/superseded-language-tooling.md). Native availability, syntax/structure, source positions and full navigation are separate gates on macOS ARM64, Windows x64 and Windows ARM64.

| Target family | Platform qualification still required |
| --- | --- |
| Python, Rust, TypeScript, JavaScript, C, C++ | Admitted private runtime/analyzer, passive loading, source-bound syntax and symbols, definitions/references, edits/branch invalidation and missing-metadata behavior on each target. |
| Bash, PowerShell | Grammar/source spans and independently verified navigation where supported; no profile/script execution or guessed dynamic bindings. |
| HTML, JSX, TSX, CSS, Vue, Angular, Astro, SVG | Structure, embedded regions and original source mapping; framework-specific cross-file navigation requires actual fixtures. |
| JSON, TOML, XML, YAML | Per-format structural parsing and original spans; schema/link resolution only when separately implemented and qualified. |
| SQL: `postgres`, `sqlite`, `mysql`, `tsql` and `duckdb` | Preserve the [bounded per-dialect contract](../docs/m0-sql-qualification.md#additional-bounded-dialects--2026-10-25), original spans and explicit ambiguity/refusals. The finite leaf and development MCP fixtures passed on macOS ARM64; native Windows and installed final payloads require their own evidence. No MariaDB or universal SQL claim. |

Use explicitly selected platform SDK/header inputs only for C/C++ where required. No automatic SDK installation, global analyzer fallback, project build, plugin, generator, macro or dependency download is authorized by a navigation request. A successful parser or server initialization cannot pass a definitions/reference gate.

## Exception candidates and choices still open

| Proposed Windows ARM64 exception | Why consider it | Required proof before admission |
| --- | --- | --- |
| Playwright Windows x64 browser processes | Inspected upstream browser mapping supplies Windows x64 packages. | All three engines work on actual ARM64 Windows, including owned profiles, helper executables, headless/visible behavior, cleanup, updates and resource use. |
| x64 Python + Docling/PyTorch worker | Inspected ML wheel releases lack Windows ARM64 wheels. | Resolve one compatible worker closure; run seven-language layout/table fixtures, measure memory/startup/throughput, verify offline models and no native-module architecture mixing. |

These are candidates under the accepted policy, not newly accepted pins. No exception is assigned merely because research is incomplete. Native ONNX Runtime, Typst and D2 leads should be pursued first. Do not add duplicate fallback engines or a new compatibility framework.

## Qualification order during implementation

The [historical implementation plan](implementation-plan-2026-10-24-historical.md#6-implementation-milestones) assigned the original checks to executable milestones. The [proposed replacement](implementation-plan.md) owns current release qualification. This matrix is evidence for selecting experiments, not a passing runtime report.

1. **M0a — Prove the small shared path:** Codex Desktop on each target, exact host/SDK protocol facilities, thin guide discovery, explicit scope, useful child operation/saved-result inspection, long-job reconciliation, abrupt death and synthetic maintenance activation/reconnect. **M0b — Qualify bounded risks:** the selected passive analyzers, then Windows ARM64 extraction, followed by browser/design/video, PDF/SQL and embedded memory. Check actual host Office capabilities separately, preserving explicit gaps instead of adding a private engine. Obtain exact dependency approvals and target access; advance successful slices independently without bypassing any release gate.
2. **M1–M2 — Integrate the core and memory:** Codex sessions, ownership/cancellation, project identity, state consistency, bounded recall and backup/restore/forgetting. Source availability does not establish these behaviors.
3. **M3 — Complete capability coverage:** all selected languages, browsers, managed document/design/media outputs and OCR languages, including refusal cases and actual saved artifacts. For Office, verify the host-supported creation/edit/recalculate/render/export journeys and clearly report missing host capabilities under decision 79. Do not infer broad coverage from M0 samples.
4. **M4 — Complete guides begun in M0a/M1:** focused loading, exact section rights, comparative output and negative-trigger evaluation, accessibility/analytical validity and useful outcomes through host-owned workflows.
5. **M5 — Qualify complete releases:** final inventory, notices, checksums/native verification under decision 76, native installation/client journeys, interruption recovery and measured resource use on all three targets. The proposed minimum OS versions and operating defaults are in the implementation plan; they remain unverified until exercised. No release or performance claim follows from planning.

No benchmark, output-fidelity, emulation-performance or installed-client claim is made by this document. Mutable documentation and release metadata must be rechecked when exact pins are proposed. The current qualification gaps are tracked here and in the existing decision record; this review adds no parallel task tracker or new implementation subsystem.
