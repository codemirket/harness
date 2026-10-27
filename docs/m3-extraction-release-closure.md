# M3 extraction release closure

Status, 2026-10-25: **the Python Docling baseline remains selected, but no
extraction distribution is admitted**. A compatible macOS ARM64 wheel/model
candidate now exists in private state and has run offline. It fails the authored
table-content check and has unresolved native payload, advisory and notice
boundaries. Do not promote successful imports or a top-level conversion success
to complete extraction support.

This report extends [the OCR qualification](m3-extraction-ocr-qualification.md).
It changes no source, distribution manifest, installed runtime, client setting or
public tool. The bounded alternative examined here is reduced OpenCV source
requirements. No docling.rs executable, second extraction engine or new source
build was introduced.

## Exact local candidate

The private record is
`dependencies/extraction/release-closure-20261025/` beneath the existing
`~/.codex/task-state/harness-rewrite-implementation/` task record. It contains
direct PyPI/Hugging Face responses, acquired wheels/models, member and notice
inventories, native imports, commands, authored gold and retained failures.
`wheel-resolution.json` records every wheel URL, filename, version, byte count,
SHA256, marker environment, selected extra and dependency edge. It is a candidate
lock, not an admission manifest or a cross-platform lock.

The roots are `docling-slim[extract-core,format-pdf-docling,models-local]==2.142.1`,
explicit pandas, `docling-ibm-models[opencv-python-headless]==4.0.4`,
`torch==2.14.1` and `torchvision==0.29.1`. The wheel tags and Python constraints
were resolved for normal-GIL CPython 3.13.17, ARM64 and a macOS 15 target floor.
All actual execution used macOS 26.5.2 build 25F84 on an Apple M1 Pro with 16 GiB
RAM. Declared wheel compatibility is not execution on macOS 15.

| Important wheel | Selected version / identity |
| --- | --- |
| docling-slim | 2.142.1; SHA256 `8013da3da0692b51406e6192c3b2aba4c241da92ff055af106dbe6b723b7f528` |
| docling-core / docling-parse | 2.104.0 / 7.23.0; parser is the CPython 3.13 macOS 14 ARM64 wheel |
| docling-ibm-models | 4.0.4; SHA256 `5236a13d440135aa074e211981529424f7fd15acf7938c38431381e623430a3e` |
| Torch / torchvision | 2.14.1 / 0.29.1; CPython 3.13 macOS 14 ARM64 wheels |
| transformers | 5.20.2; local CPU object detection, compilation disabled |
| NumPy / SciPy / pandas / Pillow | 2.5.4 / 1.18.1 / 3.1.0 / 13.0.0 |
| OpenCV candidate, rejected for admission | `opencv-python-headless` 4.14.0.94, `cp37-abi3-macosx_13_0_arm64`; SHA256 `bc7db37dc234f7bb3190a158fd9dd750357246fc7d8adb23698843ef721a993b` |

[Versioned slim metadata](https://pypi.org/pypi/docling-slim/2.142.1/json),
[IBM model package metadata](https://pypi.org/pypi/docling-ibm-models/4.0.4/json),
[OpenCV wheel metadata](https://pypi.org/pypi/opencv-python-headless/4.14.0.94/json).

The first 73-wheel selection imported successfully but failed full pipeline
initialization with `ModuleNotFoundError: No module named 'cv2'` in
`docling_ibm_models.tableformer.data_management.tf_predictor`. Its OpenCV
dependency is an optional extra in package metadata but mandatory in this
selected TableFormer implementation. The initial lock and failure remain in
`failed-no-opencv/`. Explicitly selecting the headless extra produces 74 wheels;
it does not make the resulting native payload suitable for admission.

Every acquired wheel matched its published size and SHA256. All 27,253 extracted
members were compared byte for byte with their wheel members after inference.
The selected wheels' actual METADATA has no unsatisfied active dependency. Raw
JSON/wheel requirement-string differences are retained separately: psutil has a
development-extra spelling difference, and Torch has duplicate/order-only
differences. They do not add an active macOS dependency. No sdist was built, pip
installer was run, `.pth` file was processed, or upstream hook/helper was invoked.

## Model and native payload boundary

The selected model tree contains exactly seven files, totaling 384,434,803 bytes:
the Heron card, configuration, image preprocessor and safetensors weight; and the
TableFormer card, accurate configuration and safetensors weight. The configuration
contains the selected table vocabulary; this exercised path needed no separate
tokenizer model, ONNX graph or external-data sidecar. Unselected fast, VLM, ASR,
enrichment and alternative OCR assets were not acquired.

| Model | Frozen revision and weight SHA256 |
| --- | --- |
| Heron | `8f39ad3c0b4c58e9c2d2c84a38465abf757272d8`; `00333a43451945aaf89db8ca9c0a17e75d1537c17db60fdb91aa95f4c7929e0c` |
| Accurate TableFormer | `2199320848bb9a8a519d22e4b528185a4f9a6f64`; `2a7d6c924b3cd12fb99a09280ca9c33a89c5d60b93253617d2e088c1a40374d9` |

Small files were checked against their Git blob IDs and independently hashed with
SHA256; weights matched their upstream LFS SHA256 values. `model-manifest.json`
records every asset. The Heron card declares Apache-2.0; the TableFormer
repository card declares CDLA-Permissive-2.0 and Apache-2.0. Preserve the cards,
both applicable license texts and model attribution rather than relying on the
Python code's MIT license. The texts were captured in private supplemental
notices. This is an upstream declaration review, not an independent assessment of
all training-data rights.
[Heron card](https://huggingface.co/docling-project/docling-layout-heron/blob/8f39ad3c0b4c58e9c2d2c84a38465abf757272d8/README.md),
[TableFormer card](https://huggingface.co/docling-project/docling-models/blob/2199320848bb9a8a519d22e4b528185a4f9a6f64/README.md).

The final wheel tree contains 333 Mach-O files totaling 673,513,000 bytes. Native
inspection records actual load targets separately from dylib install IDs; no
non-system absolute load target remains in that observation. This does not prove
relocation, signing, an older OS floor or complete runtime closure on another
platform. NumPy/SciPy report Apple Accelerate; SciPy also carries Fortran runtime
libraries. Torch carries its CPU runtime and OpenMP. Pillow, torchvision, lxml,
rtree, tokenizers, safetensors and other binary wheels have their own native
closure.

The OpenCV wheel adds **93 dylibs**. These include FFmpeg and its filters,
x264/x265, GnuTLS/OpenSSL, X11/SDL, and another Tesseract/Leptonica pair. Its
`cv2/version.py` says `headless=True`, while `cv2.getBuildInformation()` reports
`GUI: COCOA`, FFmpeg enabled, OpenCL enabled and broad modules including highgui,
videoio and dnn. The package name and metadata therefore cannot establish the
intended narrow build. These binaries do not replace the separately qualified
Tesseract executable or establish its language/model identity.

The wheel includes wrapper/OpenCV/third-party notices, but the native inventory
and required redistribution obligations are not fully reconciled. In particular,
the FFmpeg path and x264/x265 require a component-specific source/license review;
the generic LGPL notice is insufficient evidence for the complete observed
binary combination. The selected torchvision wheel supplies its BSD notice but
does not include a complete notice set for its separate codec/libc++ dylibs.
The rtree wheel's wrapper MIT notice does not establish the notice closure of its
libspatialindex binaries. The latex2mathml wheel and inspected sdist have no full
license file despite MIT metadata. Tokenizers' Apache license was recovered from
its exact sdist; its Rust dependency notices still require reconciliation.

The private audit indexes 251 notice-like wheel members, with exact hashes.
Pillow's 70,307-byte combined notice and its SBOM, lxml's additional library
notices, Torch's per-component notices, and SciPy's runtime-exception notices are
retained. A notice count alone does not establish that every incorporated library
is covered. `source-inventory.json` records upstream project links and published
sdist identities for the 74 packages; it does not claim every sdist was acquired
or every binary's build inputs reproduced.

## Advisory exposure

The exact-version OSV Python screen returned no records for the 74 selected
packages. A separate screen of the 127 Cargo components declared in tokenizers'
embedded SBOM matched Tokio 1.53.1 to `RUSTSEC-2026-0386`, an I/O-safety issue in
Unix `AsyncFd` constructors, fixed on the relevant line at 1.53.3. It also matched
paste 1.0.15 to `RUSTSEC-2024-0436`, an unmaintained notice rather than a demonstrated
runtime vulnerability. These records describe SBOM components; compiled feature
presence and reachability from this offline image path remain unverified. Do not
turn a Python-only screen into a clean native-runtime claim.
[Tokio advisory](https://rustsec.org/advisories/RUSTSEC-2026-0386.html),
[paste maintenance notice](https://rustsec.org/advisories/RUSTSEC-2024-0436.html).

Direct native version calls identify Pillow's libpng as 1.6.58 and torchvision's
as **1.6.56**. The latter is in the range of CVE-2026-34757, fixed in 1.6.57. The
advisory requires an application to pass particular getter results back to
setters on the same PNG object; a crafted PNG alone does not trigger it. No such
call sequence, exploit or hostile input was exercised here. The affected bundled
version is a release-review blocker until an exact safe selection or supported
reachability disposition exists. The native closure has additional libraries
whose versions/advisories are not fully mapped; it is not cleared by these two
screens.
[libpng advisory](https://github.com/pnggroup/libpng/security/advisories/GHSA-6fr7-g8h7-v645).

## Offline inference and the discriminating failure

Imports and inference used the previously admitted private CPython 3.13.17 with
`-I -S -B`, an explicit package path, private cwd/HOME/temp/cache, offline Hugging
Face settings, telemetry disabled and macOS network denial. CPU float32 Heron,
accurate TableFormer, one thread/batch, bounded queues, the private fixed-source
Tesseract executable and its explicit data directory were selected. Remote
services, external plugins, compilation and enrichment were disabled. Models
were loaded from explicit local artifacts. Nothing was installed into the user's
home or existing Mirket release.

All thirteen import probes passed, including the PDF parser, Torch/torchvision,
pandas, converter, standard PDF pipeline and Tesseract CLI stage. The input PDF
is an authored, image-only page with a six-row, three-column revenue table and a
three-column merged header. Its independent gold specifies all sixteen cell
texts and spans, eight exact decimal amounts, the page size and raster/page
transform. The PNG was visually inspected before inference. The PDF SHA256 is
`463e4774e060745922c4b602fd24fb902168dd1e4a934111eea72e86a8539e65`.

| Observation | Outcome |
| --- | --- |
| Initial full pipeline without OpenCV | Import failure; retained in `failed-no-opencv/` |
| First full inference, Tesseract PSM 6 | Top-level `SUCCESS`, no errors, one table, zero rows/columns/cells; table text omitted |
| One correction to normal automatic PSM 3 | Top-level `SUCCESS`, no errors, six rows and three columns, correct merged-header span; only **2 of 16** cell texts and spans exact |
| Amounts and column labels | All eight amounts and Q1/Q2 missing or replaced with `_`/`—`; content check fails |
| Post-run integrity | All 27,253 package files, seven model files and fourteen existing OCR runtime files match their frozen inputs |

The first inference remains in `failed-psm6-empty-table/`; the PSM 3 result and
independent comparison are saved without replacing its gold. A successful model
load or correct table dimensions cannot establish table-content fidelity.
The observed top-level success needs an honest coverage/failure contract before
exposure. The earlier source finding that crop subprocess failures can be logged
and skipped remains a separate, untested integration failure path.

The existing exact `eng,tur,deu,fra,spa,ita,por,osd` assets remain unchanged.
This slice runs an English table; it does **not** claim full-pipeline fidelity
for all seven languages. The prior mixed-language ordering, Portuguese box and
rotation failures remain open. This specimen supplies page points and table text
boxes, but does not qualify original PDF crop/rotation transforms, exact text
coordinates, multicolumn reading order, degraded scans, encrypted/malformed
inputs, missing assets, cancellation or output finalization.

## Footprint, memory and limits

| Measured regular-file bytes | Bytes |
| --- | ---: |
| 74 compressed wheels | 279,560,884 |
| Extracted wheel members | 1,010,372,517 |
| Selected model files | 384,434,803 |
| Wheels + extracted members + models | **1,674,368,204** |
| Private closure record including metadata/source/notices/results at measurement | 1,753,699,836 |

The preparation guard used a conservative 2,000,000,000-byte ceiling and at least
1 GB free reserve; the recorded free space after the run was about 9.04 GB.
Existing Python/OCR specimens were reused without copying them. A hypothetical
installed combination of the selected extracted wheels/models, previously
selected Python and OCR bytes starts at 1,475,699,089 bytes before filesystem
overhead, additional notices, staging, caches or scratch. This is not an assembled
or admitted installation-size result.

The import process reported approximately 452 MB maximum RSS. The PSM 3 PDF run
reported approximately 1.49 GB process maximum RSS; 0.1-second target sampling of
the owned process tree observed a 1,639,251,968-byte RSS-sum peak. RSS sums can
double-count shared pages, and sampling can miss shorter peaks. Its elapsed wall
time was 12.38 seconds including startup. Cache state and concurrent host work
were uncontrolled. These are single prepared-workload observations, not cold/warm
baselines, minimum RAM, percentiles or evidence for two concurrent heavy jobs.

Windows x64, the approved x64 worker exception on Windows ARM64, CRT rights,
private relocation and Windows ownership remain unexecuted. This macOS lock
cannot be reused as a Windows native closure. No docling.rs layout/table or
reading-order comparison was executed, so no engine winner or Python-removal
claim follows.

## Smallest next step

Keep extraction unavailable in the production manifest. The bounded alternative
inspection found that the selected TableFormer source needs OpenCV resize,
interpolation/border constants, thread/OpenCL toggles and `cv2.Mat`; Docling's OCR
base additionally uses dilation. These are **core/imgproc plus the Python
binding**, not an image-codec, video, GUI or second OCR stack. Exact source
hashes and the API checklist are in `reduced-opencv-requirements.json`.

Prepare one separately reviewed, pinned CPython 3.13 ARM64 OpenCV build containing
only core/imgproc and the required Python/NumPy binding. Inspect its build files,
disable configure-time downloads, and verify actual native imports and numeric
resize/dilation behavior; module flags alone are insufficient. A source-tag lookup
for the wheel's reported OpenCV 4.14.0 returned 404, so this report does not invent
an exact source pin or claim that build complete. Do not implement a replacement
`cv2` shim or substitute a different interpolation algorithm without numerical
and document-quality evidence.

Then rerun this frozen failed table, diagnose OCR text loss at the raster/crop
boundary, and require exact amounts plus honest incomplete-coverage reporting.
Complete the remaining notices/advisory dispositions and frozen seven-language,
layout/reading-order, coordinate and failure corpus before admission. A reduced
OpenCV build alone cannot repair the demonstrated semantic failure. No further
dependency search, alternative engine campaign or source compilation was run in
this slice.

Frozen evidence SHA256 values:

| Private evidence | SHA256 |
| --- | --- |
| Wheel resolution | `e1073cb3d59a746a6842f5d7078b64732cdf801b09c53f08debc263d6470779a` |
| Model manifest | `8ecebeed19c8818b3aa10ef802a7a5fb0a7e286545898eda3398cc9a2f8c39d8` |
| Final inference result | `4837a4288186cefce14aa8a34244c87d6fa43766556cda525d50e2cb98fac39c` |
| Post-run verification | `232124000fa19bb9dc744332f051ea64db97d3991b504496a142acbf2442b925` |
| Evidence index | `9023796906dc1930ce2f064f42a3aa04f5bad6a30964205bc54ef8df3ffa9005` |
