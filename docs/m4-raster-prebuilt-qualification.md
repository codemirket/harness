# Upstream Sharp prebuilt candidate — 2026-10-26

This is historical candidate qualification. The exact Mac Raster payload later
passed local first-party installation and installed output checks; see the
[current qualification](simplification-qualification-2026-10-27.md#final-local-mac-integration).
Later [signed source provenance](m4-raster-prebuilt-provenance-20261027.md)
verified both selected Mac native packages and matched all 28 reported native
versions to the pinned build recipe; it did not close the source/notice route.
Public source and notice delivery remains a separate gate. Statements below
that normal installation refuses Raster describe the earlier build.

The official [Sharp installation guide](https://sharp.pixelplumbing.com/install/)
lists prebuilt macOS ARM64 and Windows x64/ARM64 binaries. This is the active
closed Mac Raster candidate. The custom source-built recipient route was
removed from active code. No npm install, lifecycle script, global library
lookup or host package modification was performed.

The exact registry archives inspected were:

| npm package | Version | Archive bytes | SHA-256 | Observation |
| --- | --- | ---: | --- | --- |
| `@img/sharp-libvips-darwin-arm64` | 1.3.4 | 8,341,921 | `e8350e62344d027995ef46cdd97a5ea1c88bc54ab82ab8c3fcb0c076d839675f` | Upstream libvips 8.18.7 dylib, 18,231,688 bytes |
| `@img/sharp-darwin-arm64` | 0.35.5 | 110,737 | `52168af984c322e5548bebfb437a9976244824338ea5b8a5b1e0cdaaf06480cb` | The `.node` hash matches the unchanged addon in the current candidate |
| `@img/sharp-libvips-win32-x64` | 1.3.4 | 8,346,896 | `3d6d4c909ffcf72d70b7d42a29fd8c69b71fd8afa786ce46cd88ebadfd187114` | PE x64; native execution untested |
| `@img/sharp-libvips-win32-arm64` | 1.3.4 | 7,678,274 | `f06011a56e6e50684c6b9e410b877edc7a7731d6273643619b7965dc2bdcde23` | PE ARM64; native execution untested |
| `@img/sharp-win32-x64` | 0.35.5 | 8,575,353 | `16a804181ddd258ca6fe7079b2faa0c0356654c9db3d93ab0a80ec5e5161c3d4` | PE x64 addon and native libraries; native execution untested |
| `@img/sharp-win32-arm64` | 0.35.5 | 7,895,045 | `7a8e306ca14032a49efa5e0d3d5bb6af4da0c4bb18c7e070ee0a5253d40e5603` | PE ARM64 addon and native libraries; native execution untested |

Each downloaded archive matched its registry SHA-512 integrity value. The
Windows `@img/sharp-win32-*` archives already contain their libvips DLLs;
their separate libvips archives are inventory candidates, not an instruction
to install duplicate runtime files. The
macOS upstream libvips archive was extracted as inert data into a disposable
copy of the current pinned Sharp 0.35.5 module tree. The selected private Node
24.22.1 loaded it with a cleared environment, rendered a 20×10 SVG into a
40×20 PNG and decoded the result; dimensions and PNG signature matched.
This establishes basic ABI and rendering compatibility on the observed Mac,
not parity for all Raster operations or a normal Mirket installed job.

The prebuilt dylib links only Apple system libraries, but strings inspection
found `/Users/runner/work/sharp-libvips/...` and `/usr/local/...` built-in
resource/configuration paths. The npm libvips tarball declares LGPL-3.0-or-later
in package metadata but does not contain a license file or complete build source
kit. The old custom source-built [Raster candidate](m4-raster-final-gate-review.md)
has a different binary, selected source kit and recipient route. Its receipt
cannot be reused for these upstream bytes. Before admitting the prebuilt route,
qualify private-path behavior for every admitted raster operation, build an
exact source/notice/recipient story for the selected native closure, generate
a new fixed release inventory and run actual Mac installed and native Windows
jobs. Raster remains unadmitted in normal setup while those gates are open.

The inert archives and temporary compatibility tree are under
`~/.codex/task-state/harness-rewrite-implementation/sharp-prebuilt-20261026/`.

## Follow-up: actual Mac raster job and selected-format checks

On macOS 26.5.2 ARM64, the copied upstream package ran through the existing
`tests/raster-mcp-smoke.mjs` path with Mirket's development server and private
Node 24.22.1. Preparation refused a stale source revision. WebP, JPEG and a
3,151,914-byte PNG-to-WebP resize reached successful terminal receipts with
matching saved-output hashes, source preservation and complete owned scratch
cleanup. The 8×8 WebP output was 104 bytes, SHA-256
`216ab42f4809255844e3007b0c1a27103a2abec293bb6eb676a6c1263687bb50`.
This is an actual Rust/MCP/Node worker test, still using the explicit
development module path rather than a normal installed Raster component.

A separate private-Node probe decoded 48×32 output from all four requested
formats under a macOS profile that denied file reads below `/usr/local` and
`/Users/runner`. PNG, JPEG, WebP and AVIF each completed; Sharp reports the
decoded AVIF container as `heif`. The probe exercised representative encode
and decode work while those two hard-coded path prefixes were denied. It does
not prove that every native resource lookup stays private, nor does it qualify
an OS sandbox for production.

These results remove the basic compatibility concern for this Mac. The
remaining work is rights/notice handling, integration into normal setup,
installed Raster jobs, and native Windows x64/ARM64 execution. The old custom
source-built recipient inventory is a different binary and cannot validate
this prebuilt package.

## Exact upstream archive closure for setup design

The [proposed prebuilt selection](../packaging/admission/m4-raster-prebuilt-macos.runtime.json.proposed)
now records all six package archives used by the Mac module: `sharp` 0.35.5,
`@img/colour` 1.1.0, `detect-libc` 2.1.2, `semver` 7.8.5,
`@img/sharp-darwin-arm64` 0.35.5 and `@img/sharp-libvips-darwin-arm64` 1.3.4.
Each official npm archive matched the exact SHA-512 integrity in the locked
adapter dependency graph. All archive members were regular files or
directories under `package/`; there were no links or traversal names. A
read-only controlled extraction of the six archives produced 125 files, all
byte-identical to the private module tree used in the tests. The complete
runtime tree is 19,698,667 bytes. The proposal itself is 28,886 bytes, SHA-256
`705cd057ebc0cc25c6cffe92d05d442663dad544dcd62b0e67cc30adae74e41b`.

This inventory is a **proposal**, not an admitted release component. The Rust
layout now pins this upstream tree and still rejects normal Raster installation
until setup acquisition, source/notice handling and installed behavior are
qualified. The custom-build recipient CLI was removed from active code; its
earlier evidence remains in historical records. The official
[sharp-libvips v1.3.4 release](https://github.com/lovell/sharp-libvips/releases/tag/v1.3.4)
lists the native dependency versions and identifies the source tag; it does
not by itself settle the recipient's complete license/source obligations.

The Rust distribution owner pins that proposal hash and its exact 125-file
private module inventory. A qualification fixture made only from the six
verified archives matched every compiled file pin. `layout` still refuses
Raster before manifest creation or installation, and Windows targets still
refuse the component. The inactive custom-build recipient module,
import/rollback CLI and doctor branch were removed; an installed release will
use its immutable, verified private module once setup acquisition and the
remaining gates are admitted. The old proposal is retained only as
[historical evidence](historical-admission/m4-raster-source-recipient.json.proposed).

## Reproducible offline assembly

`scripts/assemble-raster-prebuilt.py` verifies the six pinned npm archives by
size, SHA-256 and registry SHA-512 integrity. It rejects links, special tar
members, unsafe or extra paths, duplicate entries and files that differ from
the 125-file proposal. It writes into separate private staging and moves only
the verified Raster tree into an existing design release stage. It never
invokes npm or upstream package code.

The assembler's two negative tests passed. With the six downloaded official
archives, it reproduced 125 files and 19,698,667 bytes under a disposable
release stage. The Rust `raster_private_fixture_matches_every_frozen_candidate_pin`
test then verified every assembled file against its compiled release pin. A
separate run with an altered `sharp-0.35.5.tgz` refused before publishing a
Raster destination. The real Raster MCP development smoke then passed using
the newly assembled private module, including stale-source refusal, WebP and
JPEG saves, a 3,151,914-byte PNG resize, output hashes and owned scratch
cleanup. This is a qualification result, not normal setup acquisition, a
release admission or recipient-rights clearance.

As a regression check for removing the old mutable recipient owner, a fresh
disposable **Design-only** release was rebuilt with the current Mirket binary,
staged and activated through `dev setup-core`. `doctor-core` reported a healthy
manifest and launcher, and `tests/design-installed-smoke.mjs` passed the
installed token and vector jobs. The earlier `dev activate` check correctly
reported a missing launcher because that command intentionally does not
publish one; `setup-core` completed the normal core ownership path. No user
installation or Codex configuration was changed.

## Isolated installed Raster qualification — macOS 26.5.2 ARM64

An isolated source fork under
`~/.codex/task-state/harness-rewrite-implementation/raster-installed-qualification-20261026/`
opened the Raster admission check only when
`MIRKET_RASTER_QUALIFICATION_FORK=1` was set. The shipping source and binary
retain the refusal. The fork passed the build-input guard and an offline,
locked build without new dependencies. Its payload combined the already
qualified Design tree with the exact 125-file Sharp tree assembled from the
six pinned upstream archives. Without the qualification flag, manifest
creation refused Raster before writing a manifest.

With the flag, `dev manifest` and `dev setup-core` produced and activated a
disposable Design+Raster release, ID
`sha256:2036b3eca9f43ac36d2ab8bf1028e06fa2a9363d31a04c4e0960c8ad1d37b572`.
`dev doctor-core` reported a healthy launcher and manifest with both components.
The installed `mirket serve` and private Node then passed
`tests/raster-mcp-smoke.mjs --home` through MCP. It refused a stale source
revision, saved and decoded WebP, JPEG, PNG and AVIF outputs with matching
artifact hashes, preserved the source, resized a 3,151,914-byte PNG, and
reported complete owned scratch cleanup. AVIF decoded as Sharp's `heif` with
AV1 compression. The same expanded smoke passed through the development
route. The normal shipping binary still refused `dev manifest` on the
Design+Raster source with the expected unadmitted-component error.

This establishes installed Mac behavior for the selected prebuilt tree. It
does not admit Raster into normal setup. The source/notice and recipient
rights route, normal release admission, native Windows x64/ARM64 behavior and
Codex Desktop GUI activation remain open.

## First-party Mac setup acquisition in the isolated fork

The macOS installer now has a conditional Raster acquisition branch. It
accepts an explicit directory containing the six exact npm archives, or
downloads those six pinned URLs over HTTPS. It verifies each copied archive's
size and SHA-256, extracts only after verification into private staging,
normalizes file modes, and lets Mirket verify the complete release inventory
before activation. It invokes no npm lifecycle scripts or upstream helpers.
The branch is inert for normal releases because the shipping binary still
refuses Raster admission.

Using the same isolated qualification fork, a release source with the Raster
tree omitted and the Design and Skills payloads present passed the first-party
`scripts/install-macos.sh --raster-archive-root` route. Release ID
`sha256:4c54a9a8e8152e1ccb574af16767d498e9c45431c4b5a60f3e6fbfd3e9233b96`
activated in a disposable user home. `mirket doctor --json` reported healthy
Design, Raster, nine global skills, managed guidance and MCP registration.
The installed MCP Raster smoke passed stale-revision refusal, PNG, JPEG, WebP,
AVIF and large-input output checks with saved hashes and complete scratch
cleanup. A same-size, one-byte change to `colour-1.1.0.tgz` was rejected on
SHA-256 before update; doctor still reported the original release healthy.
The default HTTPS acquisition branch installed the identical release ID into
a second disposable home, and doctor reported it healthy. The Mac tests ran
on macOS 26.5.2 ARM64. Neither disposable home is the user's installation.

The upstream [Sharp installation guide](https://sharp.pixelplumbing.com/install/)
lists macOS ARM64 and Windows x64/ARM64 prebuilt support. The selected
[`sharp-libvips` v1.3.4 notices](https://github.com/lovell/sharp-libvips/blob/v1.3.4/THIRD-PARTY-NOTICES.md)
identify LGPLv3 and other third-party grants for the native closure; the npm
binary archive includes that summary in its README but no complete source kit
or full license-text packet. The tagged [v1.3.4 source and build
scripts](https://github.com/lovell/sharp-libvips/releases/tag/v1.3.4) are a
source lead, not a verified corresponding-source delivery for these exact
prebuilt bytes. The final notice/source route and recipient terms remain a
release gate. Native Windows jobs and Codex Desktop GUI activation also remain
unqualified.
