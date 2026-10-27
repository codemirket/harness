# M3 narrow OpenCV source-build qualification

Qualification date: 2026-10-25. This is a private candidate experiment, with no
production dependency, route, installer or user installation change.

One isolated macOS ARM64 build of OpenCV **4.14.0**, containing `core`, `imgproc`
and the Python/NumPy binding, is feasible within the disk budget. Its staged
extension is **8,253,576 bytes** and imports only macOS system libraries. The
unchanged frozen table passes **16/16 exact cell texts and logical spans, and
8/8 exact amounts**. The independent numeric gate **fails: 96/101 outputs are
bit-exact against the existing broad wheel, with five uint8 linear-resize
mismatches**. The table pass does not waive that failure. This candidate is not
admitted for distribution or seven-language/platform use.

## Source pin and reviewed build

The real official `4.14.0` release resolves to commit
`0654a42e19215ef25b1d367d822f3c630447e7c7`, annotated tag object
`ea8e60798e0319ccbe902f7994153b574f639809`, released 2026-07-19. This matches the
selected wheel's reported OpenCV version and tested API subset; it does not
establish identical compilation or algorithms. The earlier failed tag lookup
remains part of the earlier report; current official metadata resolves the pin.
[Official release](https://github.com/opencv/opencv/releases/tag/4.14.0),
[tag reference](https://api.github.com/repos/opencv/opencv/git/ref/tags/4.14.0),
[resolved commit](https://api.github.com/repos/opencv/opencv/commits/4.14.0).

The commit-addressed source archive is **96,313,878 bytes**, SHA256
`7a75663094ebd50bf212928423a24394e29d751db878044ad015ba28546a59db`.
Full regular source contents would occupy 206,371,016 bytes. Selected build
source occupies 76,997,984 bytes, plus 14,999 bytes of original CMake control
files needed from omitted documentation/data/Python-test directories. Post-run
comparison verifies 4,338 retained regular source files against this archive.
The sole source modification adds a fatal error at `ocv_download` entry; it
changes download policy, not library algorithms.
[Pinned archive](https://codeload.github.com/opencv/opencv/tar.gz/0654a42e19215ef25b1d367d822f3c630447e7c7).

The reviewed path uses CMake configure, the upstream Python binding generators,
Clang compilation, and CMake's **Python-only component staging into the private
task directory**. It does not use `pip`, `setup.py`, package-manager installation,
upstream installers, model helpers or custom hooks. Configure, compile, staging
and inference deny network access through macOS `sandbox-exec`. The download
cache contains only `.gitignore`.

The exact options, environment, reviewed script hashes and reused header hashes
are retained in `build.py`, `build-run.json` and `reviewed-build-inputs.json` in
the private evidence record. The material recipe is:

| Boundary | Selected configuration |
| --- | --- |
| Modules/linkage | `BUILD_LIST=core,imgproc,python3`, `BUILD_SHARED_LIBS=OFF`; build target `opencv_python3`, parallelism 2; stage component `python` only. |
| Toolchain | Existing CMake 4.4.4, Xcode Clang 21.0.0, macOS SDK 26.5, ARM64 deployment target 15.0; actual execution macOS 26.5.2 on M1 Pro, 16 GiB RAM. macOS 15 was not executed. |
| Python/NumPy | Existing private normal-GIL CPython 3.13.17 and NumPy 2.5.4; existing complete CPython headers, with matching `pyconfig.h`; ordinary `cp313` binding, not the broad wheel's limited ABI. No new headers or Python package fetched. |
| Native acceleration | IPP, ITT, OpenCL, Carotene, KleidiCV, ARMPL, FastCV, Eigen and LAPACK disabled; empty `CPU_BASELINE` and `CPU_DISPATCH`; compiler default ARM64 target still applies. |
| Codecs/UI/extras | FFmpeg, media/GUI backends, image codecs, extra modules, ADE, Flatbuffers, Java, apps, examples, tests, documentation, tracing, package registries and custom hook paths disabled. SDK zlib selected explicitly. |
| Parallel backends | TBB, OpenMP and plugins disabled. Upstream enables Apple GCD despite `WITH_GCD=OFF`; `OPENCV_DISABLE_THREAD_SUPPORT=ON` conflicts with that platform default, so the final build uses `OFF`. Runtime requests `setNumThreads(1)`. |

`BUILD_LIST` alone was insufficient: unselected GAPI initialization tried to
acquire ADE. The fatal download guard caught it before acquisition; final
configuration explicitly disables ADE. Two other retained failed configurations
show that omitted directories still need three CMake control files and that the
GCD/thread-disable combination is rejected. These configuration failures preceded
the **single compile invocation**. The relevant upstream owners are
[top-level CMake](https://github.com/opencv/opencv/blob/0654a42e19215ef25b1d367d822f3c630447e7c7/CMakeLists.txt),
[download helper](https://github.com/opencv/opencv/blob/0654a42e19215ef25b1d367d822f3c630447e7c7/cmake/OpenCVDownload.cmake),
[Python binding build](https://github.com/opencv/opencv/blob/0654a42e19215ef25b1d367d822f3c630447e7c7/modules/python/common.cmake)
and [parallel implementation](https://github.com/opencv/opencv/blob/0654a42e19215ef25b1d367d822f3c630447e7c7/modules/core/src/parallel.cpp).

The build supervisor hit a temporary-file rename race while sampling disk use
and exited. The original compiler process continued; it was inspected, not
replayed. Its log reaches `[100%] Built target opencv_python3`, the extension
exists and subsequently imports and executes, and private staging exits zero.
The original compiler exit code and complete elapsed time are unavailable;
continuous compile-time disk minimum cannot be claimed. The failure and process
reconciliation remain in `compile-reconciliation.json`.

## Native and rights closure

The staged module SHA256 is
`ccacfaf176182c65690f6e272c635ee37413c6ba9d23c0e59d761c9a46ecf6e0`.
Its Mach-O imports are exactly `/usr/lib/libz.1.dylib` (1.2.12),
`/usr/lib/libc++.1.dylib` (2100.43.0) and `/usr/lib/libSystem.B.dylib`
(1356.0.0). It embeds static OpenCV core/imgproc and the binding; it does not
redistribute a separate Homebrew, FFmpeg, GUI or image-codec library. The staged
Python loader and conservative notices bring the private stage to **8,706,152
regular-file bytes**. `LC_BUILD_VERSION` records minimum macOS 15.0 and SDK 26.5.
Upstream build-information text still says Cocoa `YES`; absent GUI APIs and the
actual import closure establish the narrower result, rather than that label.

Compiler dependency files enumerate 353 OpenCV source/header inputs with no
unexpected external headers: only the pinned source/build tree, SDK and explicit
Python/NumPy header roots appear. DLPack is an actual header dependency.
OpenCV's root and selected DLPack licenses are Apache-2.0. Actual core/imgproc
source also carries older OpenCV BSD-style, SoftFloat BSD-3-Clause, fdlibm
notice-preservation, MIT and Berkeley qsort attribution text. The qsort comment
contains an advertising clause; it is retained verbatim rather than silently
reclassified. Apache-only attribution would therefore be insufficient.
[OpenCV license](https://github.com/opencv/opencv/blob/0654a42e19215ef25b1d367d822f3c630447e7c7/LICENSE),
[DLPack license](https://github.com/opencv/opencv/blob/0654a42e19215ef25b1d367d822f3c630447e7c7/3rdparty/dlpack/LICENSE),
[SoftFloat/fdlibm notices](https://github.com/opencv/opencv/blob/0654a42e19215ef25b1d367d822f3c630447e7c7/modules/core/src/softfloat.cpp),
[qsort notice](https://github.com/opencv/opencv/blob/0654a42e19215ef25b1d367d822f3c630447e7c7/modules/core/src/datastructs.cpp).

The private stage retains complete captured source notice blocks, root/DLPack
licenses and input hashes. This is deliberately conservative: dependency headers
can contain disabled code, and comment collection is not a final linked-code
rights reconciliation. Release notices, source-modification notice, exact
redistribution terms and toolchain/SDK permissions still require final review.
No commercial-only runtime component was selected by this recipe.

An exact-commit OSV query returns **OSV-2023-444**, a heap read overflow through
`opj_jp2_apply_pclr` / OpenJPEG decoding. Its affected-code path is excluded here:
no `imgcodecs`/OpenJPEG compiled dependency, native import or `imread` API exists
in this build. This is a scoped disposition, not a clean security verdict;
the saved response includes 4.14.0 in affected versions. The original Torch,
tokenizers/Tokio, PDF parser and other native closure remain unchanged and retain
their existing advisory gates.
[Primary OSS-Fuzz record](https://github.com/google/oss-fuzz-vulns/blob/main/vulns/opencv/OSV-2023-444.yaml),
[existing closure dispositions](m3-extraction-release-closure.md#advisory-exposure).

## Frozen numeric oracle: failed

`numeric-inputs.npz`, `numeric-recipes.json` and `reference-outputs.npz` freeze
deterministic inputs, operations and original broad-wheel outputs. The oracle
SHA256 remains
`6a28527b283cebdf7dbeb37d93942d1896124b233584145a91e756323d463ce7`.
There are 65 resize cases (uint8/float32, grayscale/RGB, four interpolation modes,
several sizes/strides, plus `cv2.Mat`) and 36 dilation cases (three inputs, three
kernel sizes, four border modes). **All 36 dilation cases and every float32
resize are bit-exact.** Five uint8 `INTER_LINEAR=1` cases differ:

| ID | Effective input shape; byte strides | Output width × height | Max absolute channel delta |
| --- | --- | --- | --- |
| `004` | RGB `31×37×3`; `(111,3,1)` | `19×23` | 1 |
| `036` | gray `31×37`; `(37,1)` | `19×23` | 1 |
| `037` | gray `31×37`; `(37,1)` | `53×47` | 3 |
| `039` | gray view `16×18`; `(74,2)` from `31×37` base `(37,1)` | `29×11` | **191** |
| `100` | `cv2.Mat` RGB `31×37×3`; `(111,3,1)` | `17×19` | 1 |

Both processes use runtime NumPy 2.5.4, request `setNumThreads(1)`, and report
OpenCL disabled. Both `getNumThreads()` queries return 10: the upstream GCD query
returns a CPU count, not a measured worker count. The source's serial condition
honors the requested setting, but no actual thread-count or performance bound is
inferred from the query.

The broad wheel reports NEON, FP16, NEON_DOTPROD and NEON_FP16 with NEON_BF16
dispatch, custom Carotene/KleidiCV HAL, Clang 15, NumPy 2.0.2 build headers and
limited Python ABI `0x03060000`. Narrow reports no custom HAL, empty CPU feature
line/dispatch, Clang 21, NumPy 2.5.4 build headers and ordinary CPython 3.13 ABI.
Those are confounding build differences; this experiment does not identify a
specific faulty backend.

For each mismatch, converting the effective input with `ascontiguousarray`
produces identical output **within each engine**. The strided case therefore
does not isolate a NumPy-stride wrapper defect. `cv2.Mat` and a plain contiguous
ndarray agree within each engine, and the two Mat-wrapper source files are
identical. An independent half-pixel float bilinear calculation is retained only
as a diagnostic: narrow is within 1 in all five cases; it does not replace or
relax the frozen broad oracle.

The requested cheap control executes only these five cases against the exact
broad wheel after `cv2.setUseOptimized(False)`. That API exists and
`useOptimized()` reports false. **All five remain byte-identical to the original
broad oracle and none converges with narrow**; deltas remain `1,1,3,191,1`.
No second build was run. Numeric admission remains failed.

### Independent diagnosis of the failed comparison

A later read-only probe kept the frozen inputs and both saved output sets
unchanged. It ran OpenCV's `INTER_LINEAR_EXACT` for **all nine uint8 linear
resize cases** in separate processes loading the original broad wheel and the
narrow build. The two processes produced **identical bit-exact output hashes
for all nine**. Each narrow `INTER_LINEAR` result differs from that bit-exact
output by at most one byte value; the broad wheel's five original mismatch
cases differ by `1,1,3,191,1`, respectively. On
case `039`, a separate scalar half-pixel bilinear calculation also agrees with
the narrow result to within 0.70; the broad result's first column differs by as
much as 190.91. At row 3, column 0 the scalar value is 190.91, bit-exact mode
is 191, narrow mode is 191 and broad mode is 0.

This shows that **the broad wheel's ordinary `INTER_LINEAR` result is an
unsuitable correctness oracle for case `039`**. Its build reports custom
Carotene/KleidiCV HALs while the narrow build does not; OpenCV's resize owner
calls a HAL before its generic path. The exact faulty dispatch or algorithm has
not been isolated, so the HAL difference remains a plausible cause, not a
proved attribution. The prior frozen parity result remains failed and intact.
This independent result supports evaluating a replacement against the
bit-exact/scalar contract, with an explicit one-value rounding allowance,
instead of requiring it to reproduce the broad wheel's observed edge error.
The original 92 other resize/dilation cases already match the broad wheel
exactly, but these limited synthetic cases do not qualify all image geometries
or the complete extraction distribution.

Private read-only probe and outputs:
`numeric-exact-diagnosis.py` SHA256
`c4cfe697a6c08ae99af7384e13d9d728b5c3ba0dfbb5702e6e2b936309835c22`,
`numeric-exact-reference.json` SHA256
`4166240fa217fbd45af23ac971886f6c7a93e8244c7626194942440587e75dce`
and `numeric-exact-narrow.json` SHA256
`2358b38338ae3707e19f72c03082e360b1f9878746443db1211333d587fb1ab0`,
under the same private evidence directory. The original numeric input, recipe
and output hashes are embedded in both reports.

## Same frozen table: passed separately

One network-denied inference preloads the staged narrow `cv2` before importing
the existing private Docling coverage adapter. Loaded-image inspection after
inference confirms that no old broad `cv2` binary was loaded. It reuses the
frozen PDF SHA256
`463e4774e060745922c4b602fd24fb902168dd1e4a934111eea72e86a8539e65`,
the same models, fixed Tesseract executable, PSM 11, render scale 3.0 and existing
bounded parser settings. The original PSM 3 failure, PDF and gold are unchanged.

The private adapter reports `complete` execution coverage; raw Docling reports
`SUCCESS`. Independent gold comparison finds all **16/16 texts and logical
spans**, including the merged 3-column header, and all **8/8 amounts**:
`1234.56`, `1400.00`, `250.00`, `300.00`, `15.44`, `20.00`, `1500.00`, `1720.00`.
That is semantic evidence for this page only. Full results and original span
definitions remain in `table-inference/` and the earlier
[engine qualification](m3-extraction-engine-alternatives.md).

Elapsed wall time including process startup is **14.4231 seconds**.
`/usr/bin/time -l` records maximum RSS **1,742,749,696 bytes** and peak memory
footprint **1,479,133,320 bytes**. This run does not sample descendant RSS, and
its cache/host workload is uncontrolled, so it is not a memory/performance
improvement claim or a direct comparison with earlier sampled tree RSS.

## Disk, evidence and next gate

The task record is
`~/.codex/task-state/harness-rewrite-implementation/dependencies/extraction/narrow-opencv-20261025/`.
It retains **241,551,474 regular-file bytes**, including the evidence index,
archive, selected source, one build tree, staged module, notices and numeric/table
evidence. This is below the **1,073,741,824-byte** additional-disk ceiling.
Free space was 4,410,662,912 bytes before acquisition, 4,297,789,440 before source
extraction, and **3,678,240,768 bytes at freeze**, above the **2,147,483,648-byte**
floor. Concurrent work changes free space; the supervisor gap described above
prevents claiming a continuously measured minimum. No second build or broad
sample campaign was used.

Post-run integrity verifies all 74 original wheel archives and their 27,253
members (1,010,372,517 member bytes), seven model files, fourteen OCR payload
files, reused executable/fixture pins, 293 Python/NumPy headers and selected
source files. The failed result/PDF/gold and numeric oracle hashes are unchanged.
Evidence index SHA256:
`ef773383f88c479805ae66f34114d154cddbe4b9bdf1ce122ebdf600741ed7cd`.

The next narrowly owned task is to **diagnose the five frozen resize differences
without replacing the oracle or adding a production route**. Start with case
`039` and inspect the pinned resize/HAL dispatch and broad-wheel build identity;
the no-optimization and contiguous-copy controls already exclude those simple
runtime explanations. Any future rebuild requires a separate bounded decision.

Further admission requires numeric resolution, final notice/advisory closure,
relocatable signed packaging and cancellation/output-failure behavior; seven
language mixed/rotated/degraded OCR, reading order and PDF coordinate transforms;
and clean-machine Mac/Windows executions. No Windows build ran here. Windows
x64 and ARM64 need separate ABI/CRT/zlib/import closure, architecture-compatible
Python/NumPy/OCR children and execution. Upstream's 4.14.0 release specifically
notes Windows ARM PPL auto-partition failures on some cache designs and suggests
OpenMP; adding OpenMP would introduce another runtime/rights/performance choice
that this GCD-only Mac build does not qualify. Native OpenCV availability also
does not qualify the remaining ML stack on Windows ARM64 or supersede the
existing explicitly selected x64-worker route.
[Upstream Windows ARM caveat](https://github.com/opencv/opencv/releases/tag/4.14.0).
