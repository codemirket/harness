# Inkscape package notices and source references

Mirket downloads the unmodified Inkscape 1.4.4 macOS ARM64 application from
[Inkscape's server](https://media.inkscape.org/dl/resources/file/Inkscape-1.4.4_arm64.dmg).
Mirket releases contain the adapter, package definition and these notices; they
do not contain or rehost the Inkscape binary. The application remains a separate
process and exchanges ordinary SVG, PNG and PDF files with the adapter.

The complete Inkscape binary is **GPL-3.0-or-later**. Most individual Inkscape
source files are GPL-2.0-or-later, but included GPL/LGPL version 3 code changes the
binary's license. This is stated in the pinned upstream [COPYING](COPYING).
The separately bundled Ghostscript 10.05.1 is **AGPL-3.0-or-later**, with font
embedding and CMap provisions described in [its license](GHOSTSCRIPT-LICENSE.txt).
Other components retain their respective licenses.

## Retained evidence

- [BUNDLED-NOTICES.txt](BUNDLED-NOTICES.txt) preserves 65 named notice files from
  the signed application, grouped into 45 distinct texts without discarding their
  original paths.
- [SOURCE-INVENTORY.json](SOURCE-INVENTORY.json) records the exact Inkscape,
  submodule, macOS packaging, build-tool and customized Python source revisions;
  74 reviewed build-module definitions include their upstream archives, available
  hashes and patch references. These definitions include build dependencies and
  are not a binary SBOM.
- [GPL-3.0.txt](GPL-3.0.txt) and [AGPL-3.0.txt](AGPL-3.0.txt) retain the full license
  terms. The package lock records artifact identity, observed bundle structure
  and the scope of native verification.

The bundle includes native libraries, a customized Python 3.10.19 framework,
Python packages, Ghostscript and resources. All original application files and
links remain present. The editor profile exposes only native SVG inspection and
SVG/PNG/PDF exports; it does not expose bundled interpreters, extensions or
arbitrary Inkscape actions. Inkscape's macOS command-line restriction on Python
extensions remains in effect.

## Local use and redistribution

The FSF distinguishes private copying and use from conveying copies to others.
Its [FAQ](https://www.gnu.org/licenses/gpl-faq.html#PropagationNotConveying)
identifies copies made for oneself as propagation without conveying; private
use does not require publishing source. Mirket's current installation obtains
the binary directly from its upstream distributor. This describes the current
delivery path, not a general exemption for downloaders or redistributors.

If a release, mirror or shared cache supplies copies of the application to
others, assess that distribution separately. GPLv3 sections 1 and 6 require
complete corresponding source, including applicable dependency sources and
build/install scripts. Section 6(d) permits another source server with clear
directions and equivalent access, while retaining the distributor's availability
obligation. Merely linking the main Inkscape repository is insufficient for a
bundle with additional components. The FSF also confirms that
[unchanged binary redistribution has source obligations](https://www.gnu.org/licenses/gpl-faq.html#UnchangedJustBinary).
Preserve copyright notices, license texts and recipients' license rights.

The retained named notices and pinned recipes do **not** establish a complete
native dependency notice audit or a maintained corresponding-source service.
The inventory identifies these limits explicitly. No complete component SBOM,
source archive closure or independent rebuild of the official app is asserted.
Before Mirket rehosts these binaries, complete that component review and provide
the applicable source access for the actual redistributed build.

The FSF's [aggregation guidance](https://www.gnu.org/licenses/gpl-faq.html#MereAggregation)
considers both communication mechanisms and semantics. A separate executable
alone is not a blanket license exception. Mirket does not link against Inkscape
libraries or exchange its internal object structures.
