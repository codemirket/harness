# Executable document and illustration capabilities

Research date: 9 October 2026. Scope: a fresh comparison for a personal harness used from Codex Desktop and Claude Desktop Code. No existing harness selections were used as evidence that a capability is useful or operational. No dependency, plugin, account connection, or MCP server was installed.

## Recommendation

Build a small portable authoring and inspection layer around independently licensed libraries and real rendering programs. Keep native Codex artifact tools as an optional host adapter. Use Python for DOCX, PPTX and XLSX when it covers the requested features; add PptxGenJS only for a demonstrated slide requirement. Keep a separate image-generation adapter because a skill file cannot provide an image model or paid API access.

The valuable shared behavior is executable: write an editable artifact, render the exact saved version, inspect every page or slide, check content and calculations, repair the source, then repeat affected checks. A file existing, a library importing, or a renderer returning success is insufficient evidence of a good deliverable.

Minimal initial capabilities:

1. Office authoring: `python-docx`, `python-pptx`, `openpyxl`; existing templates and normal OOXML inspection.
2. Rendering: explicitly resolved LibreOffice and Poppler executables, separate conversion output and temporary LibreOffice profiles.
3. Inspection: page count, extracted text, all-page PNGs, presentation element bounds, spreadsheet formulas and computed values. A vision-capable host must actually inspect the PNGs.
4. Markdown: the project's build and examples first; optional pinned markdownlint-cli2 and lychee for syntax and links.
5. Illustration: source SVG plus a verified rasterizer for vectors; available native image generation or an explicitly selected credentialed API for bitmap work.

## Eight candidate routes

| Candidate | Inspected evidence and practical value | Reuse and availability | Decision |
| --- | --- | --- | --- |
| 1. Anthropic official office skills | Actual DOCX/PPTX/XLSX/PDF skill bodies describe authoring, OOXML manipulation, LibreOffice conversion, and spreadsheet recalculation. They contain environment-specific assumptions, including preinstalled libraries and sandbox helpers. | Their individual licenses are proprietary and restrict copying, derivatives, retention and redistribution. The repository expressly separates these from its open-source examples. Public visibility is not a portable redistribution permission. | Reference their documented behavior; do not vendor these skills or helpers. Use an available Anthropic product capability under its own terms, or independently implement against library documentation. [Repository and per-skill license](https://github.com/anthropics/skills/tree/683bc88e56f3e09ba94f7055977f3d3aa499f202/skills/docx). |
| 2. OpenAI official public skills and native runtime | The pinned public PDF skill uses ReportLab/extraction libraries and Poppler visual checks. Public ImageGen includes an API CLI. The current public plugins archive was searched for the office skill directories; it contains a Google Slides routing skill, but the native `documents`, `presentations` and `spreadsheets` packages inspected locally were not present at those public paths. | The public PDF and ImageGen skills have per-directory Apache-2.0 licenses. The old `openai/skills` repository now says it is deprecated and points to `openai/plugins`. Local bundled files and internal artifact packages are not thereby licensed for copying to Claude. | Reuse only individually reviewed public payloads with notices. Prefer native tools when actually exposed; preserve a portable fallback. [Deprecation](https://github.com/openai/skills/blob/49f948faa9258a0c61caceaf225e179651397431/README.md), [PDF source](https://github.com/openai/skills/tree/49f948faa9258a0c61caceaf225e179651397431/skills/.curated/pdf), [current plugin collection](https://github.com/openai/plugins/tree/0722921d5542fc593105c27bd52630babd8b8c2a). |
| 3. Independent Word and slide libraries | `python-docx` creates and updates Word content/styles/tables; `python-pptx` provides a Python path for editable decks. PptxGenJS provides Node-based editable PowerPoint generation and native charts. These are authoring libraries, not rendering engines. | All three inspected repositories carry MIT licenses. Python is the smaller initial runtime set here; PptxGenJS adds Node/package resolution but can justify that cost for rich decks. Arbitrary OOXML fidelity requires fixture tests, especially when editing existing files. | Start Python-first; keep PptxGenJS a bounded alternative, not a second mandatory slide pipeline. [Word documentation](https://python-docx.readthedocs.io/en/latest/), [python-pptx source](https://github.com/scanny/python-pptx/tree/278b47b1dedd5b46ee84c286e77cdfb0bf4594be), [PptxGenJS saving API](https://gitbrent.github.io/PptxGenJS/docs/usage-saving/). |
| 4. openpyxl versus XlsxWriter | openpyxl supports workbook reading/writing; it does not evaluate formulas. XlsxWriter's formula documentation also distinguishes formula writing from calculation and cached results. A cached zero or absent value must not pass as a computed result. | openpyxl is MIT/Expat and its maintained source is Heptapod, not the old GitHub mirror. XlsxWriter is BSD-2-Clause. Excel/LibreOffice remains a separate calculation dependency. | Use openpyxl for create/edit continuity. Consider XlsxWriter for a generation-only requirement after measuring benefit. Recalculate a copy, then assert known results and formula preservation. [openpyxl license/source](https://openpyxl.readthedocs.io/en/stable/), [formula limitations](https://openpyxl.readthedocs.io/en/stable/simple_formulae.html), [XlsxWriter formulas](https://xlsxwriter.readthedocs.io/working_with_formulas.html). |
| 5. LibreOffice plus Poppler | LibreOffice documents command-line export filters; Poppler provides PDF rasterization. This is the common executable QA path for both hosts. It also enables simple XLSX recalculation, tested below. | LibreOffice is distributed under MPL-2.0 with components under other licenses; inspect installed notices before redistributing binaries. Keep renderers as detected external dependencies rather than vendored payloads. Output is LibreOffice's interpretation, not proof of identical Microsoft Office rendering. | Required when claiming portable rendered office QA. Record renderer version, fonts, source hash, page count and warnings. [Conversion filters](https://help.libreoffice.org/latest/en-US/text/shared/guide/convertfilters.html), [LibreOffice licensing](https://www.libreoffice.org/licenses/), [Poppler project](https://poppler.freedesktop.org/). |
| 6. Pandoc plus Markdown checks | Pandoc can turn Markdown into DOCX/PPTX and apply a reference document; its AST conversion cannot promise arbitrary original formatting fidelity. markdownlint-cli2 checks Markdown conventions; lychee checks links. Neither checks whether an instruction is true or useful. | Pandoc's repository has GPL licensing; markdownlint-cli2 is MIT and lychee is Apache-2.0. Use optional executable dependencies with pinned versions and retained notices when redistributing anything. | Add Pandoc for repeatable prose-to-document exports, not as the universal office editor. Prioritize runnable examples and reader testing over stylistic lint scores. [Pandoc manual](https://pandoc.org/MANUAL.html), [markdownlint-cli2](https://github.com/DavidAnson/markdownlint-cli2/tree/916ad0aaa108c64d294101002066f530ea170b10), [lychee](https://github.com/lycheeverse/lychee/tree/c11d7794e9be55fc57d5ffd35543bf6221799fb3). |
| 7. Editable SVG plus resvg and a coherent icon family | resvg is a standalone/static SVG rasterizer with a published test suite and explicit support boundaries. It is not an animation/browser-interaction renderer. Lucide provides reusable SVG icons with a coherent visual vocabulary; it does not replace bespoke illustration design. | resvg offers MIT or Apache-2.0. Lucide's license is ISC with notices for Feather-derived icons; preserve the complete applicable notices. Neither executable was assumed present. | Keep SVG as editable source; render at actual delivery sizes and on light/dark backgrounds. Use an existing icon system for UI icons; use deliberate original geometry for illustrations. [resvg source/license](https://github.com/linebender/resvg/tree/617cebab98f8f08e354fb4664656d9a575db70e6), [Lucide license](https://github.com/lucide-icons/lucide/blob/a04f228cd01185e09c188b7227b9600c08c565ec/LICENSE). |
| 8. Native image generation versus explicit API CLI | The public ImageGen skill prefers Codex's built-in tool and makes CLI fallback explicit. Its actual script has generate/edit/batch commands, an API-key check, output handling and retries. A local skill cannot manufacture that native tool in Claude Code. | The public skill/script are Apache-2.0; the model service, account access and billing are separate. No API key was read and no generation was purchased or run during research. | Codex: use exposed native generation. Claude Code: use a user-selected provider/CLI only when credentials and billing are authorized, otherwise report unavailable. Evaluate actual generated images in their intended crop and context. [Skill and script](https://github.com/openai/skills/tree/49f948faa9258a0c61caceaf225e179651397431/skills/.system/imagegen). |

Anthropic's public doc-coauthoring workflow additionally provides a useful reader-testing idea, but it is a conversational workflow, not an executable document engine. Its individual license was not located at the guessed path during this review, so this report does not approve importing it. An independently authored reader test can ask someone without task history to execute the document's instructions. [Inspected workflow](https://github.com/anthropics/skills/blob/683bc88e56f3e09ba94f7055977f3d3aa499f202/skills/doc-coauthoring/SKILL.md).

## Hands-on host evidence

The Codex workspace dependency loader returned bundle `26.1007.11041`. This is evidence for this session and machine, not a portable installation guarantee.

| Probe | Observed result |
| --- | --- |
| Default interpreter | `/usr/bin/python3`; imports for docx, pptx and openpyxl unavailable. |
| Bundled interpreter | `/Users/nazmi/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3`; python-docx 1.2.0, python-pptx 1.0.2, openpyxl 3.1.5, ReportLab 4.4.9, pypdf 6.10.0, Pillow 12.3.0, pdf2image 1.17.0. |
| Bundled Node resolution | `pptxgenjs`, `docx`, `sharp` and `@oai/artifact-tool` resolved through the dependency directory. `@resvg/resvg-js` did not. Package resolution alone was not counted as an operational authoring test. |
| `soffice --version` | Wrapper resolves a real bundled LibreOffice executable: LibreOfficeDev 26.8.0.0.alpha0, revision `2c87e51eeaa2b413ff4ae097b2705eea1995d8e5`. |
| `pdftoppm -v` | Wrapper resolves bundled Poppler 26.05.0. |
| Optional PATH tools | Pandoc, resvg, Inkscape, lychee and markdownlint-cli2 not found. |
| DOCX smoke | Created a one-page editable document with heading, Turkish characters and a two-column table using python-docx. LibreOffice produced PDF; Poppler produced PNG. The PNG was opened and inspected: text, table headers and Turkish glyphs were visible and unclipped. |
| PPTX smoke | Created a one-slide editable presentation using python-pptx. LibreOffice produced PDF; Poppler produced PNG. The PNG was opened and inspected: title and Turkish text were visible and unclipped. |
| XLSX calculation smoke | Wrote A1=10, A2=15, A3=`=SUM(A1:A2)` with openpyxl. LibreOffice converted into a separate output directory. Reopening showed formula retained and cached A3=25. |

Conversion subprocesses returned zero and created nonempty outputs. LibreOffice emitted Fontconfig warnings despite successful rendering; warnings should remain in evidence rather than be silently discarded. Fixtures ran in a temporary directory. These deliberately tiny tests establish working author/render/recalculate paths; they do not establish attractive document design, chart support, multi-page pagination quality, complex workbook compatibility or quality improvement over a baseline.

The locally inspected native Codex document `render_docx.py` resolves a runtime, creates isolated profiles and rasterizes PDF. Its bundled skill also refers to host-specific operation markers and import tools. This confirms why copying that skill body into another host is not a complete implementation. No bundled helper was copied or executed for the independent smoke tests.

## Concrete cross-host availability contract

A portable runner should accept explicit Python, Node, LibreOffice, Poppler and optional SVG executable paths. Resolve paths before invocation; use argument arrays, bounded subprocess durations and dedicated output directories. Do not rely on shell aliases, ambient package lookup or `npx` fetching code during a quality check.

For each capability, expose distinct evidence:

- `unavailable`: missing executable, package or host tool; name the missing item.
- `available`: path/version/import succeeded; no claim about output yet.
- `smoke_passed`: a known fixture produced expected files or calculation results.
- `artifact_checked`: the exact artifact hash was structurally inspected and all required render outputs created.
- `visually_reviewed`: a reviewer actually inspected those outputs; record reviewed page/slide coverage separately from mechanical checks.

Codex may resolve dependencies through its workspace loader and use native artifact/image tools when exposed in that session. Claude Desktop Code should receive the same portable workflow but resolve an independent existing environment or a separately authorized installation. Do not bake a user-specific Codex cache path into shared defaults or imply Claude has OpenAI's artifact library/MCP tools. A discovered Codex runtime can be an explicitly selected local development path, not a distribution strategy.

Credentials and remote image providers belong to adapter configuration; log only presence and provider identity, never tokens. No background package installation or network image generation belongs in a read-only capability check.

## Minimal executable workflows

The following are command shapes, not claims that optional programs are installed. A wrapper should supply resolved paths and a fresh temporary profile URI built with `Path(...).as_uri()`.

```text
<PYTHON> author.py
<SOFFICE> -env:UserInstallation=<FRESH_PROFILE_URI> --headless --convert-to pdf --outdir <PDF_DIR> <INPUT_DOCX_OR_PPTX>
<PDFINFO> <PDF>
<PDFTOPPM> -r 150 -png <PDF> <PAGE_PREFIX>
```

Check the actual expected output file and page count, not stdout alone. Inspect every PNG, plus full-resolution crops where small text needs them. Verify that extracted content covers required titles, table values and captions. For slides, inspect a contact sheet for consistency and individual slides for labels, overlap and clipping. Preserve native text/charts when editability is required. Rerender after every source change affecting the delivered artifact.

For spreadsheets, retain the original source and recalculate into a separate directory:

```text
<SOFFICE> -env:UserInstallation=<FRESH_PROFILE_URI> --headless --convert-to xlsx --outdir <RECALCULATED_DIR> <INPUT_XLSX>
```

Then open the saved copy twice with openpyxl: once with `data_only=False` to inspect formulas, once with `data_only=True` to inspect caches. Assert known results and input-change behavior; enumerate spreadsheet errors and externally linked formulas. Do not overwrite the original during recalculation. Complex Excel-only functions, macros, spill ranges and unsupported objects need representative tests or Excel verification; LibreOffice success cannot establish that those features survived. Render meaningful print areas rather than assuming all workbook cells appear in a PDF.

For Markdown, run the project's documentation build and relevant command examples in a disposable fixture. Optional syntax/link checks can use pinned local `markdownlint-cli2` and `lychee`; distinguish authenticated, rate-limited and unreachable links from objectively broken relative paths. Review title, intended reader, prerequisites, executable examples, failure cases and source attribution. Use Pandoc only when an exported office document is requested and verify its rendered output.

For SVG, parse XML, check the viewBox and intrinsic dimensions, detect embedded raster images when true vectors are required, and render at final sizes. Inspect silhouettes, stroke consistency, whitespace and contrast. For generated raster assets, validate dimensions, alpha when requested, crop, text fidelity and artifacts; inspect in the actual page/document mockup. An API success or XML-valid SVG does not pass aesthetic review.

## Representative benchmark set

Run the same inputs and constraints against the old baseline and proposed workflow, with a blinded final-artifact review where feasible. Record wall time, manual interventions, cost when a model call occurs, defects, editable-source fidelity and reviewer preferences. Keep mechanical correctness separate from aesthetic preference. None of these larger benchmarks was executed in this research pass.

| Benchmark | Fixture and acceptance evidence |
| --- | --- |
| Word decision memo | Four pages from supplied facts, a branded reference, Turkish names, a table crossing a page boundary, header/footer and citations. No invented facts; styles editable; headings logical; every page inspected; no isolated table header, clipped row, missing glyph or overflow. |
| Existing Word revision | Change two clauses and add one comment in a supplied document with tables and existing comments. Preserve unrelated paragraphs/styles and relationship parts; verify exact text changes, comment anchors and before/after render differences. Explicitly report tracked-change limitations. |
| Presentation | Six-slide proposal with a decision, native chart, source footnotes, diagram and image crop. All required facts represented; chart data editable; slide elements within bounds; every slide reviewed; visual hierarchy assessed independently of file validity. |
| Financial workbook | Inputs, monthly model and summary tabs, cross-sheet formulas, date boundary and negative/zero cases. Known outputs checked independently; one input mutation propagates; formulas retained; no unexplained errors; print areas legible. Add a modern-function workbook as a compatibility stress test, not an assumed supported case. |
| Repository guide | Install and troubleshooting guide for a tiny real fixture project. A fresh reader can run it from a clean checkout; commands and named files verified; broken links detected; no unstated prerequisites or contradictions. |
| Illustration set | One editable SVG illustration, four coherent UI icons and an optional raster hero. Verify vector purity where required, notices, small-size legibility, dark/light contexts, consistent style and usable crops. Raster generation only with an authorized available provider. |

## Source revisions and limits

These are inspected source revisions, not suggested dependency pins. Production pins should be released package versions selected after compatibility tests.

| Source | Inspected revision or version |
| --- | --- |
| anthropics/skills | `683bc88e56f3e09ba94f7055977f3d3aa499f202` |
| openai/skills | `49f948faa9258a0c61caceaf225e179651397431` |
| openai/plugins | `0722921d5542fc593105c27bd52630babd8b8c2a` |
| python-openxml/python-docx | `e45454602b53e8e572b179ccf1c91093ec9f4ed7` |
| scanny/python-pptx | `278b47b1dedd5b46ee84c286e77cdfb0bf4594be` |
| gitbrent/PptxGenJS | `3c9ec1b687c174952166f6a34b5e87ebf69fa469` |
| jmcnamara/XlsxWriter | `5d4606d89a955226d2d0825a0f44309043ae7251` |
| jgm/pandoc | `e51c9c6054c8f4ec5c3209d5abe10939dfe2963e` |
| DavidAnson/markdownlint-cli2 | `916ad0aaa108c64d294101002066f530ea170b10` |
| lycheeverse/lychee | `c11d7794e9be55fc57d5ffd35543bf6221799fb3` |
| linebender/resvg | `617cebab98f8f08e354fb4664656d9a575db70e6` |
| lucide-icons/lucide | `a04f228cd01185e09c188b7227b9600c08c565ec` |
| openpyxl | Installed 3.1.5; official stable documentation observed at 3.1.3. Exact current Heptapod revision not verified. The 2014 `ericgazoni/openpyxl` GitHub mirror was rejected as a current-source candidate. |

GitHub unauthenticated tree API access hit a rate limit after initial metadata reads. Raw pinned files and the public OpenAI plugins archive were inspected instead. Inkscape's manual returned an access error and was not used to substantiate a recommended command. No Microsoft Word/PowerPoint rendering, Excel calculation, native image generation, Claude Code session, cloud upload or provider artifact-library authoring was exercised. License observations identify why payloads were or were not selected; this report does not infer rights for uninspected files from a repository label.
