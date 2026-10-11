# Glaxnimate package notices and source references

Mirket downloads the unmodified macOS ARM64 application directly from
[KDE's official distribution](https://download.kde.org/stable/glaxnimate/0.6.0/macOS/glaxnimate-0.6.0-arm64.dmg).
Mirket releases contain the adapter, package definition and these notices; they
do not contain or rehost the Glaxnimate binary. The application runs as a separate
process and exchanges SVG, native animation documents and PNG files with the
adapter.

The upstream release is **0.6.0**. This exact application identifies itself as
**0.6.0-git-d2da67a**, so the source pin is
[`d2da67a9c839abf441bd0d1f2048d1af114ee75c`](https://invent.kde.org/graphics/glaxnimate/-/tree/d2da67a9c839abf441bd0d1f2048d1af114ee75c).
KDE publishes the archive's size and SHA256 in its
[mirror metadata](https://download.kde.org/stable/glaxnimate/0.6.0/macOS/glaxnimate-0.6.0-arm64.dmg.mirrorlist).

Glaxnimate's source declares **GPL-3.0-or-later**, copyright Mattia Basaglia and
other contributors identified in the retained source notices. Upstream's
`COPYING` link selects `LICENSES/GPL-3.0-or-later.txt`; that full text is retained
as [GPL-3.0.txt](GPL-3.0.txt). The source's other license texts, contributor list
and REUSE declarations are retained in [SOURCE-NOTICES.txt](SOURCE-NOTICES.txt).
Those declarations apply to their respective files and components, rather than
changing every component to the application's main license.

[BUNDLED-NOTICES.txt](BUNDLED-NOTICES.txt) preserves 33 named notice and attribution
files found in the application, including Python, Pillow, ICU, SDL2 and Qt
Wayland protocol notices. The bundled Python `lottie` 0.7.2 package includes an
AGPL-3.0 license text, which is retained with its original path. Other bundled
libraries retain their individual licenses. The adapter does not expose bundled
interpreters, arbitrary scripts or extension commands.

[SOURCE-INVENTORY.json](SOURCE-INVENTORY.json) records the application and source
submodule pins, build evidence, available notices and observed native library
filenames. ABI labels and filenames do not establish exact source versions or a
complete dependency SBOM. The source identifies shared Craft CI without pinning
its revision. The package/log artifacts for the identified build have expired,
and its job trace was inaccessible. Current Craft definitions are not a
substitute for the exact build's dependency list and options.

## Local use and redistribution

The FSF distinguishes copying for oneself from conveying copies to other people;
private use does not require publishing source. See its
[private copying guidance](https://www.gnu.org/licenses/gpl-faq.html#PropagationNotConveying).
Mirket's current installer obtains the application from its upstream distributor.
This describes the delivery path, not a general exemption for redistribution.

If Mirket later distributes the binary in a release, mirror or shared cache,
complete the native component and corresponding-source review first. GPLv3
sections 1 and 6 cover the applicable dependency sources, modifications and
build/install scripts. Section 6(d) permits source on a different server with
clear directions and equivalent access; the distributor remains responsible for
availability. The FSF confirms that
[unchanged binaries still carry source obligations](https://www.gnu.org/licenses/gpl-faq.html#UnchangedJustBinary).
Preserve applicable notices, license texts and recipients' rights, and assess
the additional bundled components under their own terms.

These files do **not** assert complete native notice coverage, a reproduced
build, or a maintained corresponding-source service. In particular, exact
Qt/KDE/FFmpeg and other native component build options and source closure remain
unverified. A separate process alone is also not a blanket license exception;
the FSF's [aggregation guidance](https://www.gnu.org/licenses/gpl-faq.html#MereAggregation)
considers communication mechanisms and semantics. Mirket does not link against
Glaxnimate libraries or exchange its internal object structures.
