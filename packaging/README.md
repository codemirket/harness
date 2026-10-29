# Local package assembly

The current package is a local Pi package. Nothing in this directory publishes
it, imports a previous installation, or grants approval for new dependencies.
Pi remains external and owns models, authentication and sessions.
Mirket's project license is MIT. Its noncommercial operating intent does not
restrict recipients from commercial use under that open-source license.

`runtime/` contains only current build/payload/source identity inputs consumed
by first-party code. They are not an approval catalog. Rust and TypeScript use
their lockfiles. The eight-crate build-script guard is deliberately smaller than
the actual Rust dependency closure. Historical reviews remain recoverable from
the exact Git revision in [history](../docs/history.md).

## Runtime choices

| Runtime | Current source/version | Package route | Rights/materials |
| --- | --- | --- | --- |
| Node | Official Node 24.22.1 archive; `runtime/node.json` | Bundled private executable | Exact upstream LICENSE includes Node and bundled dependency notices. |
| SQLite | Official 3.54.1 amalgamation; `runtime/sqlite.json` | Statically linked in Mirket | Public-domain dedication; unchanged selected source. |
| Semantic search | ONNX Runtime 1.31.0; MiniLM revision `e8f8c211226b894fcb81acc59f3b34ba3efd5f42` | Private runtime/model assets | Runtime MIT/third-party notices and full model card; supplemental Apache-2.0 text. |
| Navigation, tokens, SVG | TypeScript 5.9.3, Style Dictionary 5.7.1, SVGO 4.1.0, sax 1.6.1; resvg 0.49.0 linked in Rust | Bundled selected JS/Rust closure | Native notices plus the supplemental exact attribution union for wrapper bundles. Embedded dependency versions are not independently reconstructed. |
| Editable design | OpenPencil 0.16.0 selected subset; `runtime/openpencil-selection.json` | Bundled selected JS subset | Selected component notices; exact Noto project OFL for the shared font. |
| Diagrams | D2 0.9.0 official macOS ARM64 archive | Bundled private executable | MPL-2.0, third-party notices and source instructions in the delivered third-party document. |
| Video | HyperFrames 0.8.197, FFmpeg/FFprobe 9.0.2 with x264/libvpx/Opus | Bundled separate private programs | Required Video source/notice packet, complete separate FFmpeg source packet and replay instructions. |
| Rendering browser | Google Chrome for Testing 156.0.8078.73 | Explicit onboarding: direct publisher download or select existing executable | Google terms; includes proprietary Widevine. Mirket does not redistribute this archive. |
| Raster | Sharp 0.35.5 / sharp-libvips 1.3.4, libvips 8.18.7 | Excluded from the final package while rights work remains open | LGPLv3/MPL and native closure obligations are not resolved by Sharp's Apache-2.0 license. |

The [third-party scope](THIRD-PARTY.md) links primary rights evidence and states
source-correspondence limits. The [qualification](../docs/qualification.md)
records current macOS packaging/onboarding evidence and a failed startup timing
gate. Windows and other platforms require separate native qualification.

## Assemble

Build the compiled TypeScript extension and verify the native source payload
first, using the locked [build instructions](../docs/native-build.md). Select
absolute canonical paths. The output must not exist:

```sh
python3 scripts/assemble-pi-package.py \
  --mirket /absolute/selected/mirket \
  --source /absolute/verified-native-source \
  --video-source-root /absolute/video-source-notice-packet \
  --output /absolute/new-pi-package
```

The Video packet must match `runtime/video-source.json`; the assembler verifies
every copied file. Its `source/ffmpeg/` supplies the exact media source packet.
`--ffmpeg-source-root` can select that packet separately. Video/media source
delivery cannot be omitted while their binaries are bundled. The output includes
source archives, original notices, exact build logs, a portable replay recipe,
source-file inventories and `THIRD-PARTY.md`; links alone do not substitute for
the GPL source delivery. The replay recipe has not been freshly rebuilt.

The package's bound `runtime-setup.json` describes Chrome's exact publisher
archive, target, member hashes and links. Setup never performs an incidental
upstream installation or uses PATH as a fallback. Run the package's first-party
setup using its private Node and an explicitly chosen fresh private Pi agent
directory, then follow the Pi onboarding command. Existing installations and
unrelated settings are preserved.
