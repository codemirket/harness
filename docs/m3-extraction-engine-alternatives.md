# M3 extraction engine alternatives and failure diagnosis

Status, 2026-10-25: **the frozen table failure has a passing configuration, but
no extraction distribution is admitted**. Keep the selected narrow Python
Docling baseline while closing its native payload and coverage contract. The
bounded comparison does not establish a smaller replacement that meets the full
seven-language, table, reading-order and coordinate requirements.

No production source, dependency manifest, installation or client setting changed.
No new engine, wheel or model was acquired. All executed inference reused the
existing private CPython, packages, models and fixed-source Tesseract. Raw
metadata, commands, actual OCR input, outputs, scores and integrity checks are in
`~/.codex/task-state/harness-rewrite-implementation/dependencies/extraction/engine-alternatives-20261025/`.
This extends the [OCR qualification](m3-extraction-ocr-qualification.md) and
[release closure](m3-extraction-release-closure.md); their failed inputs and
results remain unchanged.

Initial alternative-investigation evidence was 3,700,885 bytes, including
primary-source metadata and two bounded full runs. Its `evidence-index.json` has SHA256
`f12a802513585ba5163c4af1b1dc44d6198d016e63cec92a37e9a1adc25e04c5`.
An identical second OCR-image copy was removed with its equivalence recorded in
`cleanup.json`; no prior evidence, wheel, model or source artifact was removed.

## What the discriminating case establishes

The original image-only PDF remains SHA256
`463e4774e060745922c4b602fd24fb902168dd1e4a934111eea72e86a8539e65`.
Its separately authored gold remains SHA256
`a21edbc718e2ff6ceabbb06cf489fe81c4c3e47f1e3718068190c36d03fe2142`:
one six-row, three-column table; sixteen populated cells; a three-column merged
header; and eight decimal amounts. Gold geometry was used only for scoring raw
OCR. No gold text, crop, row/column boundary or span was supplied to inference.

| Controlled observation | Exact result |
| --- | --- |
| Original 300-DPI PNG, direct fixed Tesseract, PSM 3 | 16/16 cell texts; 8/8 amounts |
| Same original PNG, PSM 11 | 16/16 cell texts; 8/8 amounts |
| Full Docling, PSM 3, explicit backend render scale 3.0 | **2/16 cell texts and spans**; 0/8 amounts; top-level `SUCCESS` |
| The exact 1786 × 1728 PNG passed by that run to OCR, direct PSM 11 | 16/16 cell texts; 8/8 amounts |
| Full Docling, same backend settings, PSM 11 | **16/16 cell texts and spans; 8/8 amounts**, six rows, three columns and correct merged-header span |

The two new full runs use `ThreadedDoclingParseBackendOptions(render_scale=3.0,
parser_threads=1,max_concurrent_results=1)`. They retain CPU float32 Heron,
accurate TableFormer with cell matching, ordered language `eng`, OCR scale 3.0,
full-page OCR, one thread/batch, bounded queues, and disabled enrichment, remote
services and external plugins. Only PSM changes between their inference
configurations; private output/cache locations differ. Raising backend render
scale alone did not repair the specimen. Sparse OCR segmentation at the actual
render resolution did. This isolates an OCR segmentation/resolution interaction;
it does not prove a generally optimal DPI or PSM.

Tesseract's own guidance identifies table recognition as a weak point and defines
PSM 11 as sparse text without reading-order guarantees. Therefore Docling's layout
and reading-order stages must still establish document order independently.
[Tesseract quality and segmentation guidance](https://tesseract-ocr.github.io/tessdoc/ImproveQuality.html).

The successful JSON preserves the sixteen original text/span records. Markdown
repeats the merged heading across its three display columns, as that serializer
did before; it is not the span-scoring representation. Title, introduction,
table and footer appear in expected order in this simple page. Multicolumn and
ambiguous reading order remain untested.

| Single prepared macOS ARM64 run | Elapsed, including startup | Sampled process-tree RSS sum peak |
| --- | ---: | ---: |
| New PSM 3 / render scale 3.0 | 18.90 s | 1,662,205,952 bytes |
| New PSM 11 / render scale 3.0 | 12.60 s | 1,661,009,920 bytes |

These observations are on the Apple M1 Pro, 16 GiB, macOS 26.5.2 build 25F84,
with private CPython 3.13.17, network denied during inference and offline model
settings. Cache/background state was uncontrolled. RSS sums can double-count
shared pages and 0.1-second target sampling can miss peaks. No latency, memory
improvement, minimum RAM or concurrent-job claim follows.

Post-probe comparison verifies all **27,253 wheel members**, seven model files
and fourteen OCR runtime files against their frozen inputs. The original failed
result remains SHA256
`4837a4288186cefce14aa8a34244c87d6fa43766556cda525d50e2cb98fac39c`.
The successful result is SHA256
`1b808f088133012ae84cf7f86b5e6cc228d9e4e907bac599986e7fbf26d2b8d7`.
The raw comparison is `psm11/score.json`; `post-probe-integrity.json` records the
preservation check. Artifact hashes establish identity, not general accuracy.

## Architecture and dependency alternatives

The following are bounded choices, not a new multi-engine runtime. Published
model metrics and package metadata were not treated as this project's fidelity
measurements. Only the selected Docling candidate ran full inference here.

| Path and inspected version | Rights and concrete tradeoff | Decision for this boundary |
| --- | --- | --- |
| Existing `docling-slim` **2.142.1**, `docling-parse` **7.23.0**, IBM models **4.0.4**, Torch **2.14.1**, torchvision **0.29.1** | Docling code MIT; Heron Apache-2.0; selected TableFormer has CDLA-Permissive-2.0/Apache-2.0 declarations. It already supplies the complete layout/order/table assembly and now passes the failed specimen. The 74-wheel/native closure is large and still rejected for release. | **Retain as the measured baseline.** Freeze PSM 11 plus explicit render settings as a candidate configuration, without admitting it from one page. [Exact closure and rights](m3-extraction-release-closure.md). |
| **PDFium + Tesseract**, through `pypdfium2` **5.14.0** and the already qualified fixed-source OCR | pypdfium2 is Apache-2.0/BSD-3-Clause; PDFium is BSD-style with its own third-party notices. Its macOS ARM64 wheel is 3,542,294 compressed bytes, beside the existing 34,193,421-byte OCR specimen. These are component observations, not a full installed total. PDFium supplies rendering/text coordinates; Tesseract supplies words/boxes. | **Smallest credible render/OCR foundation, insufficient full engine.** PDFium explicitly supplies no layout analysis. Replacing Docling with this pair would leave table detection/structure and document order to another maintained implementation or a new custom semantic engine. [API boundary](https://pypdfium2.readthedocs.io/en/stable/python_api.html#pypdfium2.PdfTextPage), [versioned metadata and licensing](https://pypi.org/pypi/pypdfium2/5.14.0/json). |
| **docling.rs v2.51.3**, source `59698ebc6cec2bb2b69822923eeb66cf2055f16c` | MIT code; exported Heron/TableFormer model rights remain separate. Native ONNX inference removes the Torch/Python ML closure and broad OpenCV wheel, while retaining layout, tables and order. The already frozen CPU archive is about 22.6 MB compressed; selected fp32 models total 435,591,838 bytes, before ORT/OCR/fonts/notices and extraction/staging. Upstream documents table/order/model-render differences and OCR-disabled reference scoring. | **Best focused smaller challenger, still unproven locally.** Do not replace the baseline or call it mature parity from its release receipt. A single same-PDF OCR-enabled JSON comparison is the useful future challenger experiment. [Frozen comparison](../docs/m3-extraction-ocr-qualification.md), [conformance at the inspected pin](https://github.com/docling-project/docling.rs/blob/59698ebc6cec2bb2b69822923eeb66cf2055f16c/docs/PDF_CONFORMANCE.md). |
| **RapidTable 3.0.2**, source `22592283c1f9d7c5a014c96c4ecc57de0b3ebfce`, SLANet-family ONNX | Apache-2.0 wrapper; exact selected model provenance/license still needs independent closure. It accepts external OCR boxes/text and returns HTML, cell boxes and logical spans. The package still requires ORT, OpenCV, NumPy, Pillow and other libraries. Inspected preprocessing uses `cv2.resize`; default missing-model handling downloads models. Explicit local model paths and disabled automatic OCR are required for a private probe. | **Useful table-only experiment, incomplete replacement.** It does not supply whole-page layout/order. Adding a layout model, matcher and reading-order owner would create a new assembly that is not yet smaller or more mature than the baseline. No model acquired. [Versioned dependency metadata](https://pypi.org/pypi/rapid-table/3.0.2/json), [pinned source interface](https://github.com/RapidAI/RapidTable/blob/22592283c1f9d7c5a014c96c4ecc57de0b3ebfce/rapid_table/main.py), [model selection](https://github.com/RapidAI/RapidTable/blob/22592283c1f9d7c5a014c96c4ecc57de0b3ebfce/rapid_table/default_models.yaml). |
| **Microsoft Table Transformer / TATR**, final inspected source `16d124f616109746b7785f03085100f1f6247575`; structure model `v1.1-all` | MIT code and model-card declaration. It produces structure from pixels and requires separate OCR/text input. The repository is archived as of 2026-09-21; a fresh maintained-product claim would be inaccurate. Existing Torch could run it, but that does not remove the heavy closure; an ONNX export needs its own equivalence check. | **Reference model, low-priority replacement.** It supplies neither OCR nor whole-document order and is no longer the strongest maintenance choice. [Official repository](https://github.com/microsoft/table-transformer), [model card](https://huggingface.co/microsoft/table-transformer-structure-recognition-v1.1-all/blob/main/README.md). |
| **PaddleOCR 3.7.0 / PaddleX 3.7.0 / PaddlePaddle 3.3.1**, PP-StructureV3 | Apache-2.0 code; each selected model still needs rights/pin review. This is a maintained complete pipeline with layout/OCR/table modules, but its chosen closure is not minimal: PaddleX core includes hub clients and its OCR extras pin `opencv-contrib-python==4.10.0.84`. Current PaddlePaddle CPython 3.13 ARM64 Mac and Windows x64 wheels are about 105 MB compressed each, before models/dependencies. | **Credible full-engine alternative, not the next small step.** No executed seven-language equivalence or exact-cell result; no published native Windows ARM64 PaddlePaddle wheel in inspected metadata. [Module contract](https://github.com/PaddlePaddle/PaddleOCR/blob/main/docs/version3.x/pipeline_usage/PP-StructureV3.en.md), [PaddleX closure](https://pypi.org/pypi/paddlex/3.7.0/json), [runtime artifacts](https://pypi.org/pypi/paddlepaddle/3.3.1/json). |

OpenDataLoader's current Apache-2.0 local Java path is another order/text engine,
but its documented scanned/complex-table route adds Docling and a local backend
server. It therefore does not resolve this failure with a smaller private
distribution. No JVM or service was introduced.
[Hybrid architecture](https://opendataloader.org/docs/hybrid-mode).

## Platform, advisory and admission consequences

Current metadata supplies pypdfium2 5.14.0 wheels for macOS ARM64, Windows x64
and **native Windows ARM64**. ONNX Runtime 1.31.0 also publishes normal-GIL
CPython 3.13 Windows ARM64 CPU wheels. These facts make a native component path
plausible; they do not qualify an end-to-end native pipeline or change the
approved x64 ML-worker exception. Keep every worker's Python/native libraries,
OCR executable and children architecture-compatible. docling.rs's inspected
release and the Torch/Paddle paths still need the explicit x64 worker route on
Windows ARM64 unless separately qualified native selections exist.
[PDFium platform inventory](https://pypdfium2-team.github.io/pypdfium2/platforms.html),
[ORT versioned artifacts](https://pypi.org/pypi/onnxruntime/1.31.0/json).

The fixed-source Tesseract pin and seven language assets remain those in the
existing OCR report. Current upstream releases still stop at affected 5.5.3;
do not replace the verified fixed-source pin with that release. The exact-version
OSV screen here returned no records for the six top-level packages RapidTable,
pypdfium2, PaddleOCR, PaddlePaddle, PaddleX and ORT. That limited screen does not
clear their native or model closure. The existing Docling bundle's libpng and
tokenizers/Tokio exposures and broad OpenCV rights/import problems remain open.
[Tesseract releases](https://github.com/tesseract-ocr/tesseract/releases),
[frozen advisory dispositions](m3-extraction-release-closure.md#advisory-exposure).

No Windows binary, ownership/cancellation journey, private relocation, CRT/zlib
closure or native Windows ARM64 execution was tested here. No production leaf
should be exposed before those gates and the remaining seven-language mixed/
rotated OCR, degraded scans, original PDF crop/rotation transforms, reading order,
blank/malformed/encrypted inputs and output-finalization gates pass. The existing
Portuguese box, mixed-language order and rotation failures remain discriminators.

## Private coverage adapter follow-up

A **private fixture adapter, without a production route or new dependency**, now
exercises that coverage boundary in `coverage-adapter/` beneath the same private
record. It accepts only five named, hash-bound cases. Requested language assets,
the interpreter/OCR executables and selected models are checked before ML imports.
A local pipeline subclass selects an audited Tesseract stage that retains typed
page/crop outcomes before the upstream stage catches and skips OCR failures.
No installed or extracted upstream source file was patched.

| Required case | Adapter result and discriminating evidence |
| --- | --- |
| Unchanged frozen table | `complete` execution coverage; independent gold comparison again passes **16/16 text/span records and 8/8 amounts**. All 35 full-page OCR words are retained in crop evidence. |
| Requested `eng,tur` with only the private fixture's Turkish asset absent | `unavailable`, code `requested_language_unavailable`; no ML import, pipeline start or page work. The original tessdata is untouched. A direct bypass still returns exit zero, logs failed `tur` loading and emits `1234.56`, showing why exit code alone is insufficient. |
| One-page table, injected OCR-crop subprocess exit **42** | `failed`; the typed crop error preserves exit 42 and the actual raster. Raw Docling still returns `SUCCESS` with no conversion errors. |
| Two copies of the frozen raster, second-page OCR exit **42** | `partial`; page 1 remains `complete` and independently passes the same **16/16 cells/spans and 8/8 amounts**; page 2 is `failed`. Raw Docling still returns `SUCCESS`. |
| Authored blank PDF | `blank`; actual full-page raster is pure white in every RGB channel, OCR recognition succeeds with zero words, and no table is emitted. OSD exits 1 on this blank input; that outcome remains recorded and does not turn positively observed blank coverage into missing text. |

All five cases passed sequentially under macOS network denial with private
cwd/HOME/temp/cache and the same pinned interpreter/packages/models. The fault
injector is task-owned code that forwards to the exact real executable except on
one recognition ordinal, where its subprocess deliberately exits 42. It is not a
claim that real Tesseract spontaneously failed. The missing-asset fixture uses
task-owned symlinks to the original assets while omitting Turkish. No original
file was deleted, changed or replaced.

Three additional failure checks pass: nonwhite input with successful but empty
OCR is `failed`; zero processed pages is `failed`; one usable page with another
requested page absent is `partial`. `complete` here describes requested execution
coverage, not semantic accuracy or user acceptance. The independent gold checks
provide the narrower fidelity evidence. Pure-white detection is deliberately a
finite fixture rule, not a qualified degraded-scan blankness classifier.

The follow-up retains **757,712 regular-file bytes**, including its own evidence
index, below the 250,000,000-byte ceiling. No dependency was fetched or installed.
Post-test comparison again verifies all 27,253 original wheel members, seven
model files, fourteen OCR runtime files, the executable pins and input fixtures.
The original failed result/PDF/gold hashes above remain unchanged.

| Follow-up evidence | SHA256 |
| --- | --- |
| `coverage-adapter/test-results.json` | `cefbe1f280f45f83fa4984347c74540f9f3d8734f54faaf972cb35a8f50dc55b` |
| `coverage-adapter/post-test-integrity.json` | `f02bf5db04a4bf974840972f96fcee13e5372b77eb08a8fd998044fb2de42a6d` |
| `coverage-adapter/evidence-index.json` | `7b671c653e2b397beacb8c46a97b70438bf11f277348692b7679deb5b9371c9f` |

This private adapter does not qualify a production job lifecycle, cancellation,
arbitrary input handling, output publication, Windows, seven-language fidelity,
complex reading order or original PDF crop/rotation transforms. Those and the
existing native rights/advisory gates still block admission.

## Smallest remaining qualification step

The native payload owner can prepare the previously scoped
core/imgproc-only OpenCV build with a real source pin, disabled downloads and
numerical resize/dilation checks. Keep the successful table for that build's
integration check; a smaller OpenCV binary must preserve its result. Do not add a
second table engine merely to avoid closing the current native/coverage boundary.
