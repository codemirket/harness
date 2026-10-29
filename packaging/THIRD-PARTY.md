# Third-party scope and source access

This document describes the local package's actual component choices. The
original license texts apply. No blanket license here replaces their terms or
grants rights to unrelated artwork, fonts, models or source references.

The package retains Node's exact upstream LICENSE; native npm component license
files remain beside their selected code. The external Pi host is not copied.
[Node's license](https://github.com/nodejs/node/blob/v24.22.1/LICENSE) permits
redistribution subject to its notices. [SQLite's dedication](https://sqlite.org/copyright.html)
places its deliverable code in the public domain. The actual Rust and npm
closures are bound by Cargo.lock, pi/package-lock.json and the selected runtime
file inventories, rather than an eight-crate build guard. The supplemental Rust
closure maps all 195 reachable locked target input crates to original notice
texts extracted from checksum-verified crates.io archives (including build
dependencies and conservative unused-source notices).

[ONNX Runtime 1.31.0](https://github.com/microsoft/onnxruntime/blob/v1.31.0/LICENSE)
is MIT licensed; its archive supplies original third-party notices.
The pinned [MiniLM card](https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/blob/e8f8c211226b894fcb81acc59f3b34ba3efd5f42/README.md)
declares Apache-2.0. The complete card and selected files remain in the payload;
the supplemental directory supplies the full Apache license text.

Design npm attribution includes the exact selected upstream texts and additional
component notices for the code bundled inside `@bundled-es-modules` mirrors.
The wrapper declarations are retained. This union includes BlueOak, ISC, BSD,
MIT, Apache and 0BSD texts as applicable; a wrapper's own MIT/Apache declaration
does not replace its bundled components' licenses. Exact embedded dependency
versions were not recoverable from the published bundles/trees, so no independent
source rebuild or version correspondence is claimed. The separate
`supplemental/README.md` explains the precise evidence scope.

The unchanged Noto Sans font identifies Copyright 2022 The Noto Project Authors
and OFL-1.1 in its metadata. Its full project-specific OFL text is supplied in
`supplemental/NotoSans-OFL.txt`. The [license retrieval revision](https://github.com/notofonts/latin-greek-cyrillic/blob/0ef94805d19e98ed658543a7c48c0a4f19cc0ac0/OFL.txt)
does not prove the source revision from which this font was originally generated.

D2 0.9.0 uses [MPL-2.0](https://github.com/d2lang/d2/blob/v0.9.0/LICENSE.txt).
The executable is unchanged; its LICENSE and THIRD_PARTY_NOTICES files remain
beside it. Its corresponding source is available at [the exact source commit](https://github.com/d2lang/d2/tree/d5a51743b6d5c00f1ec6f8340003f5d5a6ba4eda),
including the upstream source build instructions. Recipients may obtain that
source directly without an account or extra charge. Preserve these source
instructions when redistributing the executable.

HyperFrames 0.8.197 is [Apache-2.0](https://github.com/heygen-com/hyperframes/blob/v0.8.197/LICENSE).
The Video source packet supplies the selected npm archives and component notices,
policy modification diffs, exact embedded font mappings/OFL texts, Puppeteer
vendored headers, WAWOFF2/WOFF2/Brotli sources, and Emscripten source/runtime
notices. The supplemental Emscripten compiler-rt/libc++/libc++abi/libunwind
license texts and AUTHORS come from the exact 2.0.34 source archive. The publisher
build recipe identifies this compiler version; no independent generated-binding
rebuild or container digest correspondence is claimed. Selected esbuild source,
Go runtime and x/sys source/license/patent material are included as well.

FFmpeg/FFprobe are separate GPL-2.0-or-later programs in this build because x264
and GPL support are enabled. Their complete selected sources, license/patent
texts, source mapping, original configure/build records and portable replay
instructions are delivered in `third-party/ffmpeg/`. This supplies the source
alongside the programs under [GPLv2 section 3(a)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.html).
See that directory's README; the new replay recipe has not been freshly executed.
These programs are based in part on the work of the Independent JPEG Group.
Source-code permissions do not themselves settle codec patents for a particular
use or territory.

Chrome for Testing is acquired directly from Google or independently selected
by the recipient. [Google's Chrome terms](https://www.google.com/chrome/terms/)
apply to the executable. The exact archive includes a proprietary Widevine module
whose license requires a separate Google agreement for distribution; Mirket does
not bundle or mirror it. The installed binding is checked at each job.

Sharp/libvips Raster is excluded while its LGPL/MPL native closure and recipient
source/replacement obligations remain unresolved. [Sharp's Apache license](https://github.com/lovell/sharp/blob/v0.35.5/LICENSE)
does not cover that entire binary closure; [the exact libvips build's notices](https://github.com/lovell/sharp-libvips/blob/v1.3.4/THIRD-PARTY-NOTICES.md)
identify the additional components.
