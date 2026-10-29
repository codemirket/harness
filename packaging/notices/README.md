# Supplemental component notices

`sources.json` records each retained file's exact source and hash. These are
original upstream texts or exact selected payload declarations, not generated
MIT/ISC templates. The package keeps its other component notices beside their
selected native code and in the Video/FFmpeg source packets.

NotoSans-OFL.txt is the original Noto project's full OFL, with the same 2022
copyright statement recorded in the selected font. The retrieval revision is
not asserted to be the font's build revision. Apache-2.0.txt supplies the full
license for the Apache-declared model and packages that ship a declaration but
omit a standalone license. Their original declarations/author notices remain.

The `design/` directory supplements the nine packages without standalone notice
files. The bundled mirrors name their embedded modules in generated source
comments. Component notices are supplied conservatively for that entire union,
including original memfs 4.17.0 and deepmerge 4.3.1 attribution. Current selected
component versions in sources.json identify the notice texts' source; they are
not claimed as the unknown embedded bundle versions. `stream` already contains
its full MIT license in README and its Joyent/Node header in source. The
path-unified project says it copies Node 21.5.0 path utilities, so that version's
original LICENSE and path source/header are supplied instead of an invented
package-specific copyright statement. Boolbase's official ISC notice was added
upstream after its 1.0.0 npm release; its original ISC declaration is retained.

The Emscripten directory supplies exact AUTHORS and library license texts from
the 2.0.34 source archive identified by WAWOFF2's publisher build recipe.
The full compiler source and its root/musl notices are in the Video source packet.

The rust directory maps original notice texts from all 195 reachable locked
macOS ARM64 build-input crates, including build dependencies. Every archive was
checked against Cargo.lock before extraction. All 618 original notice members
are retained through 295 distinct byte texts. The mapping identifies the exact
crate, archive checksum and member for every text. This includes unused archive
source notices conservatively; it does not claim all archived code is linked
into Mirket. `scripts/collect-rust-notices.py` reproduces this offline collection.
