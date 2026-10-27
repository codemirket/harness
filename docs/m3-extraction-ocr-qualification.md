# M3 extraction and OCR qualification

Status, 2026-10-25: **no complete extraction pipeline is admitted**. Retain the
selected narrow Python Docling baseline. The bounded docling.rs challenger has
source/metadata evidence, not an executed layout/table comparison. This slice
qualifies a private native OCR component on macOS ARM64 and exposes important
quality and admission limits. It adds no public tool, runtime fallback, dependency
manifest entry, or second document framework.

Raw inputs, authored fixtures, commands and results live in the existing task
record under `dependencies/extraction/ocr-qualification-20261025/`. This report
supplements `dependencies/extraction/candidate-comparison.md` and its frozen
`candidate-metadata.json`; it does not replace their model and platform inventory.
The source-only comparison did not establish actual extraction fidelity. The
final native report SHA256 is
`06abc3bfaa054be7122c9eca6b88a14384e6e21e8cf6d509fd4970b0757693f3`;
the source/runtime/evidence index SHA256 is
`22174f1a8f106dceeafae68887db77e3325e0ff54532043d046348390b793d3e`.

## Selected boundaries

| Candidate | Exact input inspected | Consequence |
| --- | --- | --- |
| Python Docling | `docling-slim` 2.142.1; source `498c1b60fc2071b41e6da1719d0a85eb604113e0` | Keep `extract-core,format-pdf-docling,models-local` with explicit pandas for the CLI OCR module, CPU Heron and accurate TableFormer, explicit Tesseract CLI. These extras are not a resolved wheel/native/model lock. |
| docling.rs | v2.51.3; source `59698ebc6cec2bb2b69822923eeb66cf2055f16c` | One comparison remains required on identical scanned/merged-cell tables, reading order and original coordinates. Published OCR-disabled conformance does not qualify seven-language scans. No promotion or Python-removal claim follows here. |
| General Python | Existing admitted CPython 3.13.17 normal-GIL payload | Reuse the one private interpreter. PDF/SQL leaves must continue without importing Docling/Torch or loading models. |

[Versioned slim metadata](https://pypi.org/pypi/docling-slim/2.142.1/json),
[Python model catalog](https://github.com/docling-project/docling/blob/498c1b60fc2071b41e6da1719d0a85eb604113e0/docs/usage/model_catalog.md),
[Rust conformance method](https://github.com/docling-project/docling.rs/blob/59698ebc6cec2bb2b69822923eeb66cf2055f16c/docs/PDF_CONFORMANCE.md).

The Python standard local-model route still includes Torch/torchvision. Selecting
an ONNX extra does not replace the complete table pipeline; the inspected Windows
extra selects `onnxruntime-gpu`. Keep the selected CPU reference and resolve its
complete compatible closure before assembly. Do not invoke upstream model helpers.
Selected weights, configuration/tokenizer files, native runtimes and notices must
all be present and hash-bound before a job can launch. Heron and TableFormer have
separate weight rights; the Python package's MIT license is insufficient.

## Native OCR specimen and security boundary

The frozen Tesseract 5.5.3 source was built and exercised only on authored fixtures.
Exact-commit OSV screening then matched eight upstream advisories,
CVE-2026-88047 through CVE-2026-88054. They concern crafted `.traineddata`
components; several affect the default LSTM path. Fixed private language assets
reduce that entry surface but do not justify admitting an affected source pin.
Upstream identifies main-branch fixes and a planned 5.5.4 release. There was no
fixed release in the inspected release inventory. The original 5.5.3 specimen
remains rejected for admission.

One fixed-source route was compared, retaining the same fixtures and tessdata:

| Component | Qualification pin | Rights / selected behavior |
| --- | --- | --- |
| Tesseract | Unreleased source `f139e578a37c5c18717195c7d603ddb357717e23` | Apache-2.0. All eight advisory fix commits are verified ancestors. Keep legacy code for OSD, LSTM recognition, NEON and upstream `TESSERACT_DISABLE_DEBUG_FONTS`. No OpenMP, training tools, ScrollView, curl, archive or TIFF input support. |
| Leptonica | 1.87.0; `13275a278eb55b5746e33f95fbf5a2c8f604b3ab` | BSD-2-Clause. Static PNG-only external codec selection; no JPEG/GIF/TIFF/WebP/OpenJPEG dependencies. |
| libpng | 1.6.59; `cd952f49f95bb27154ae77dbb103032d95f6e580` | PNG Reference Library License v2 and preserved complete notice. Static library. Earlier trial 1.6.56 matched CVE-2026-34757 and is rejected. |
| tessdata_fast | `87416418657359cb625c412a48b6e1d6d41c29bd` | Apache-2.0. Exact `eng,tur,deu,fra,spa,ita,por` assets plus `osd`, and the upstream `configs/tsv` file. Git blob identities and independently computed SHA256 values are recorded. |
| System libraries | macOS `libSystem`, `libc++`, `libz` | OS-provided dependencies observed with `otool`; no redistributed Homebrew runtime library. This is not a Windows zlib/CRT closure. |

[Default-LSTM advisory](https://github.com/tesseract-ocr/tesseract/security/advisories/GHSA-q44c-23p6-5mw6),
[legacy-model advisory](https://github.com/tesseract-ocr/tesseract/security/advisories/GHSA-5j2p-r5vc-q7f3),
[PNG advisory and fix](https://github.com/pnggroup/libpng/security/advisories/GHSA-6fr7-g8h7-v645),
[Tesseract source](https://github.com/tesseract-ocr/tesseract/tree/f139e578a37c5c18717195c7d603ddb357717e23),
[debug-font compile boundary](https://github.com/tesseract-ocr/tesseract/blob/f139e578a37c5c18717195c7d603ddb357717e23/src/ccstruct/debugpixa.h).

The source archives, exact hashes, build commands, notices and advisory/fix
ancestry responses are retained. Exact-commit OSV queries returned no records for
the final three source pins; this is an observed advisory screen, not a complete
security audit. No malicious model proof of concept was executed. Caller-supplied
traineddata, configuration files, plugins, source URLs and executables must not
be accepted by the extraction operation.

The initial PNG-only trial logged errors while constructing Leptonica bitmap
fonts. An intermediate TIFF-only addition did not solve them: Leptonica's TIFF
implementation also depends on JPEG. The final route uses Tesseract's existing
debug-font compile switch, avoiding that unused codec expansion. Disabling the
legacy engine is not equivalent: it prevents the selected `osd` asset from
working. These failed closure hypotheses remain in the raw logs.

CMake configure/build used explicit existing compiler/SDK paths, private build
roots and a controlled environment under macOS network denial. No package
manager, upstream installer, model downloader, source hook or global install ran.
Only authored fixtures were processed. Private source compilation and observable
native imports do not establish packaged release assembly, reproducible-build
identity, signing, general hostile-document safety or the macOS 15 runtime floor.

The final relocated specimen has 14 runtime files and 34,193,421 regular-file
bytes before filesystem overhead, including all eight traineddata files and
notices. Its 5,303,144-byte ARM64 executable has SHA256
`6e8cb969b229d52f67dd03acf80badeefba6bcbbdb2ba2dcd624b7658c8d7dd6`.
The interpreter, source/build trees, fixtures, Docling dependencies and models are
outside that component count. This is not the size of the complete extraction
installation. No Homebrew prefix appears in its observed runtime library imports.

## Recognition and source-coordinate observations

The machine is an Apple M1 Pro with 16 GiB RAM, macOS 26.5.2 build 25F84, ARM64.
The fixture generator used the existing host Python/Pillow and the already
selected Noto Sans font; the OCR driver used the admitted private CPython with
`-I -S -B`. The font hash is
`bfb7bb691513f12e734dc346c03a03f784912432d7e3fa8e56efcf906fe86b3d`.

The frozen corpus contains one clean, two-line, 300-DPI image per language with
accented text and exact numbers, and one English–Turkish image. Each input is
2480 × 900 pixels. Gold text and separately authored raster word boxes precede
execution. This small synthetic set is a discriminating specimen, not a language
benchmark, degraded-scan evaluation or proof of representative document quality.

| Language | Recognition on its individual specimen | Minimum word-box IoU against gold |
| --- | --- | ---: |
| English | Exact word sequence | 0.9204 |
| Turkish | Exact word sequence | 0.9036 |
| German | Exact word sequence | 0.8679 |
| French | Exact word sequence | 0.7742 |
| Spanish | Exact word sequence | 0.7857 |
| Italian | Exact word sequence | 0.8276 |
| Portuguese | Exact word sequence | **0.2288** |

The Portuguese `e` box is inaccurate despite high OCR confidence. With ordered
languages `eng+tur`, the mixed specimen loses the diacritics in `İstanbul`,
`Ödeme` and `için`; reported word confidence remains approximately 0.89–0.95.
A separate `tur+eng` probe reads that same specimen exactly. Preserve language
order as part of the request and evidence. Do not silently normalize the wrong
text, change language order, treat confidence as a calibrated probability, or
claim all mixed-language/coordinate requirements passed.

TSV boxes use top-left raster pixels. They are OCR estimates, not exact PDF text
spans. Raster dimensions, crop offset, render scale, page rotation and the inverse
orientation transform must accompany any conversion back to original page
coordinates. A supplemental native `--psm 1` probe uses the same mixed image
rotated by 90/180/270 degrees. Separate OSD calls report all three angles correctly, with
orientation confidence around 2.6–2.8. Exact recognition fails at 90 and 180
degrees and passes at 270; all emitted boxes remain inside the original raster
frame, but the 180-degree word alignment has zero minimum IoU. This does not
exercise Docling's separate OSD rotation and inverse-coordinate mapping. No PDF
crop/rotation or native text-coordinate mapping is qualified here. Source
snippets must remain available for reviewing uncertain names, identifiers, numbers
and bounding boxes.

The negatives distinguish blank input from failure: blank PNG returns no words;
malformed input and an absent sole language fail. Tesseract **succeeds with
`eng+zzz`** while logging the missing language and recognizing with English.
Consequently, a zero exit code does not establish all requested languages loaded.
An intentionally small output cap kills and reaps the owned process; explicit
SIGTERM is reaped with the process group gone. The runtime/asset tree is unchanged
across the recorded clean/failure run after relocation to a path with spaces and
Unicode. Eight single-run OCR observations take approximately 0.098–0.145 seconds
and report approximately 30–62 MB maximum resident set size through macOS
`/usr/bin/time -l`. These are process observations on tiny prepared images, with
OS cache state uncontrolled; they are not cold/warm baselines, percentiles, full
Docling process-tree measurements or performance promises. These checks cover
the native process, not Mirket job cancellation or installed artifact publication.

## Integration contract before exposing extraction

This is a proposed leaf boundary, not an already registered MCP schema.

- Invoke the admitted general Python with `-I -S -B`, an exact release-bound
  adapter, explicit verified package paths, and owned private cwd/HOME/temp/cache.
  Keep Docling imports behind the extraction entry point. Bounds and heavy-job
  admission belong to the existing Rust job owner.
- Accept identity-bound local inputs with sniffed supported formats, bounded page
  selection, ordered languages from the seven admitted codes, and an explicit
  extraction mode. Reject unknown fields, remote URLs, user model/config paths
  and implicit format conversion. Request output is staged by the existing
  artifact owner; no source mutation occurs during extraction.
- Preflight all language files, `osd`, `configs/tsv`, every selected model/config
  and ONNX external-data sidecar by exact hash before loading. Missing assets
  fail with a typed unavailable result. Do not concatenate every bundled language
  automatically or select an alternative engine.
- For Python Docling, choose the Tesseract CLI model explicitly, local
  `artifacts_path`, CPU, bounded workers and full selected TableFormer behavior;
  disable remote services, external plugins and unselected enrichment. Set both
  the OCR option's private path and controlled `TESSDATA_PREFIX`: the inspected
  `_set_languages` and `_perform_osd` do not forward the OCR option path.
- Return source revision, engine/package/model identities, requested and loaded
  ordered languages, page dimensions/coordinate origin/units/transforms,
  extraction method (`text_layer` or `ocr`), raw text and raw OCR confidence,
  reading order, table row/column spans, and per-page/per-region coverage.
  DOCX/PPTX/XLSX structure needs honest paragraph/slide/sheet/cell locators; do
  not invent rendered page coordinates when that format path supplies none.
- Make `complete`, `partial`, `failed`, `unsupported` and `unavailable` distinct.
  Preserve region errors, uncertain orientation, low-confidence text and output
  truncation. Neither a successful top-level conversion nor empty text establishes
  complete coverage. Preserve originals and source crops for review.

Both inspected implementations currently discard some crop-level failures:
[Python OCR loop](https://github.com/docling-project/docling/blob/498c1b60fc2071b41e6da1719d0a85eb604113e0/docling/models/stages/ocr/tesseract_ocr_cli_model.py#L376)
logs a failed OCR subprocess then continues;
[Rust `recognize_all`](https://github.com/docling-project/docling.rs/blob/59698ebc6cec2bb2b69822923eeb66cf2055f16c/crates/docling-pdf/src/tesseract.rs#L684)
substitutes empty words after an OCR error. Resolve observable failure propagation
before relying on either engine for complete extraction. A log receipt alone is
not a stable typed coverage contract.

## Open release gates

No production leaf is added because full extraction closure and fidelity remain
unqualified. Finish one exact compatible Docling wheel/native/model inventory,
its rights/advisory review and actual offline imports/inference. Then execute
the frozen paired Docling/docling.rs layout/table/reading-order corpus with seven
languages, mixed/rotated scans, original PDF coordinates, missing assets,
malformed/encrypted inputs and crop failure injection. Preserve the failed OCR
specimens above. Measure whole-process-tree memory, footprint and resource
contention independently from recognition scores.

Windows x64 and Windows ARM64 remain unexecuted. The inspected Torch/torchvision
pins and docling.rs release do not supply native Windows ARM64 coverage. The
approved x64 worker exception is a candidate only: keep Python, ML libraries,
Tesseract and children architecture-consistent; close Windows CRT/zlib/native
rights and private relocation; test actual x64 execution and ownership on Windows
ARM64. macOS OS-library imports and source portability establish none of those
facts. No additional general interpreter or global fallback is selected.
