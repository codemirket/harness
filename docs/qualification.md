# Local qualification

## 0.0.1 harness review rerelease candidate, 2026-10-31

This candidate was checked on **macOS 26.5.2 (25F84), ARM64**, Rust/Cargo
**1.99.1**, Python **3.15.1**, private Node **24.22.1** (SHA-256
`6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`)
and the exact external Pi **1.4.0** host. The [review](review-20261031.md)
records the proxy metadata, locked-build and explicit project-migration fixes,
source scope, website checks and retained failures. No dependency pins changed.

- Native release: `sha256:2754da67ec1fb2b559ae7c5fcfc981f870fd11dc15b16db914682c4ebcf0a629`.
- Native core SHA-256: `fda20672d11ed228f27c1efd1b134768b20570a60e98354f9ebef12a35fe0754`.
- Package inventory SHA-256: `c09dcefb98ff089d608a5e6979ca6a720c22070568b4bb17a063076fc3912975`.
- Archive: **461,510,645 bytes**, SHA-256 `9b414aa8efa6bbb71df062026ad015342b2571729614fb7e5b53cfb903ca6757`.
- Migration helper SHA-256: `b13f6b6df5f86b21296030f2ca510eac04421e70be8696b419d44b9842352509`.
- `SHA256SUMS` SHA-256: `d0764010d0ee055bac0ea1bba3a5b92f55ba75763e944ac0883556def2d43882`.

The native build-input guard passed before compilation; locked offline Rust
checks passed (**446 tests, 4 ignored**), as did formatting, all-target clippy
with warnings denied, and the release build. The adapter's new direct-dependency
guard, TypeScript build and process tests passed (**146 tests, 2 skipped**).
Full Python discovery passed **199 tests, no skips**, including **23** focused
migration cases and three static website cases. The proxy regression failed
against the original zero-length behavior; the selected-project preservation
test failed against the original global-only helper. Both failed controls are
retained, separately from passing corrected runs.

The fresh assembled package passed **16 Pi integration checkpoints**, including
altered-extension refusal, settings preservation, scoped memory and forgetting
authorization/refusals, navigation/design, reload, native skill filtering and
cleanup. Actual Pi CLI/RPC/TUI passed doctor, reload, fork/resume, running abort
and clean shutdown. Fresh first-tool samples **1713.505, 1761.184, 1797.571 ms**
met the existing **5000/2500/2500 ms** gate; they still fail the historical 1500 ms
gate and do not establish a performance improvement. Saved PNG bytes reached an
offline image-capable Pi fixture exactly; that is transport, not interpretation.
Package evidence is `/private/tmp/mirket-pi-package-G7fAJl/`; `report.json` SHA-256
is `34baf72a14104414d75399b66ee4616000d17cb7937657489db16fc4382ccd0f`,
`cli-lifecycle-report.json` SHA-256 is
`acec102e63e8b9b8af2f93ac410364abf417d32cf7993332986557e4446e0cf6`.

Archive roundtrip checked all **13,357 inventoried files** (13,358 regular files
including the inventory), **15,350** directory/file members, no links or extra
files. Extracted native verification and explicit packaged-core/private-node MCP
smoke passed. The previous seven public assets were downloaded and their six
payload checksums verified before replacement. Source logs, archive reports,
failed first assembly and website screenshots are retained under
`/private/tmp/mirket-review-20261031/` rather than shipped source.

A separate fresh disposable home first installed the previous public package,
then the exact new standalone helper migrated it to this inventoried package
with an explicit project. It returned `updated`, reached backup phase `complete`,
and the final installer check returned `already-installed`. Global/project
unrelated settings, selected skills/extension filters, the project's `0640` mode,
synthetic authentication/session sentinels and the old package were preserved.
Actual Pi CLI doctor from the migrated project reported ready; actual
`/mirket-project init` renewed consent for the new release without losing the
selected skill or unrelated project setting. This is a real disposable
old-to-new migration, not a live-device upgrade or a new canonical-memory
retrieval trial. Its `/private/tmp/mirket-reviewed-migration-ds86fm8e/report.json`
SHA-256 is `311f7212040f85c5b136f0ef2e77576f83d73a5435031359ced718fc1de73272`.

The website was separately checked in private Chromium **156.0.8078.73** at
1440, 768, 390 and 320 CSS pixels with keyboard/navigation, overflow and local
asset checks and saved-pixel inspection. These checks do not qualify every
assistive technology. Fresh optional-browser-peer and video playback/quality,
macOS 27, other platforms, live-provider image interpretation and held-out skill
utility remain unrun for this package. The 305 catalog cases remain planned.

The live session was independently checked as ready on the previous native
release `sha256:144e15c9ef8cd7eb420ff0b9b95c41c5bf3ee3fbd3c21178492abfe8894b1f3b`.
Its runtime lock is busy. Staging another permanent release is not activation;
no live lock may be bypassed to claim a completed installation.

## 0.0.1 macOS ARM64 compatibility-floor rerelease, 2026-10-30

This rerelease package was checked on **macOS 26.5.2 (25F84), ARM64**, Rust/Cargo
**1.99.1**, Python **3.15.1**, private Node **24.22.1** (SHA-256
`6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`)
and exact Pi **1.4.0** peers. It accepts macOS ARM64 26.0 or newer at setup
without an OS-major ceiling. The new `mirket-pi-v0.0.1-macos-arm64.tar.gz`
archive is **462,061,639 bytes**, SHA-256
`9115c5645ec095b539adc0ec4f6a57792988cd5088076ff020d8864b727adf1c`;
all six payloads passed `SHA256SUMS` (file SHA-256
`e3001be9b9b4879a4f06d56864be7f2544a7f91fe33467ecccd84c5fe327e76c`).
The native release ID is
`sha256:144e15c9ef8cd7eb420ff0b9b95c41c5bf3ee3fbd3c21178492abfe8894b1f3b`,
core SHA-256 is
`b45e05e652e257519766ded41d86e4d4de1e0dfe5246da036717bef4f6f2d518`,
and package inventory SHA-256 is
`4631a5cf357401a6b07114521980d909174b5fb0af46c3fe5d8ca5886bd0ab1b`.
A safe extraction checked all **13,357 inventoried files**, including the new
setup helper; the archive has **15,350** file/directory members and **13,358**
regular files including the package inventory itself. There were no links or
extra files. The first roundtrip checker mistakenly rejected the ordinary
`package` root directory; its corrected version passed. The first oracle is
not counted as passing archive qualification.

The build-input guard passed before Rust compilation. Locked offline Rust tests
(**446 pass, 4 ignored**), clippy, formatting, release build, private-node
TypeScript build and process suite (**145 pass, 2 skipped**), Python suite
(**189 pass, 1 skipped**), skill-catalog refresh and explicit-path MCP smoke
passed. Assembled Pi integration passed **16 checkpoints**, including a fresh
setup, altered-extension refusal, scoped memory, selected navigation/design,
reload and skill selection. Actual Pi CLI/RPC/TUI passed doctor, fork/resume,
reload, running codemode abort/recovery and clean shutdown. Fresh first-tool
samples were **3234.666, 1789.684 and 1761.919 ms**, within the current
**5000/2500/2500 ms** gate. A saved PNG was delivered byte-exactly to an
offline image-capable Pi provider fixture; this does not establish provider
interpretation or visual quality.

The exact extracted archive then passed `install-pi.py` into a fresh disposable
home with the pinned acquired host. A separate disposable old-to-new migration
with the repaired `migrate-pi.py` exited `updated`, recorded a complete backup,
kept the old package and preserved a seeded unrelated setting and agent file.
The first attempt at that disposable old install refused an inherited
`PI_CODING_AGENT_DIR`; after unsetting it, a second attempt refused a fabricated
existing `.pi/agent` with no recognized launcher. A third attempt was stopped
by a too-short 300-second external timeout during full native staging. Its
partial state was preserved, not silently resumed or called a pass. A new,
separate fixture with a long enough timeout completed the old installation
and the migration. These failures remain in the private task evidence.

No macOS 27 machine was available. macOS 27, live-provider image interpretation,
and a fresh video playback/quality and optional-browser qualification on this
new package remain **unrun**, regardless of the new installation gate or prior
release evidence. The previous 0.0.1 archive retains its exact hash and does
not acquire the new floor retroactively. Private receipts and logs are under
`/private/tmp/mirket-compat-rerelease-0.0.1.rlbmxsfi/` and the assembled Pi
fixture is `/private/tmp/mirket-pi-package-fkigTm/`; neither is shipped source.

All **seven** public `v0.0.1` release assets were downloaded after publication,
matched their exact qualified local hashes and passed the downloaded
`SHA256SUMS`; the tag was moved to the source commit for this rerelease. The
previous archive was removed from the current release assets, not from earlier
installation directories or this historical record. The downloaded archive was
extracted into a **new** permanent private release directory at
`/Users/nazmiyilmaz/.local/share/mirket/releases/0.0.1.jLKH5l9U/` and its
13,357-file inventory and native identity were verified again. The exact
existing Pi host binding independently reverified **15,249** host files.
This was **staging, not installation**: the user's active native release,
Pi package entry and outer launcher remained the previous release when last
inspected. The active Mirket session held the native runtime lock, so a live
migration was not attempted. It must wait until the runtime is idle and be
verified separately; never bypass that lock or call staging a successful live
upgrade.

## Unreleased macOS ARM64 compatibility-floor source change, 2026-10-30

On **macOS 26.5.2 (25F84), ARM64**, with Rust **1.99.1**, Python **3.15.1**,
private Node **24.22.1** (SHA-256
`6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`)
and external Pi **1.4.0** peers, this source tree changed the fresh package setup
gate from exactly `26.5.2` to macOS ARM64 `26.0` or newer. Read-only inspection
of the published candidate's Mach-O deployment commands found a macOS 11.0
floor for the native core, 13.5 for Node, 13.0 for D2, 14.0 for ONNX Runtime
and **26.0 for both FFmpeg and ffprobe**. Those floors do not establish working
behavior on another macOS release.

Synthetic version tests accepted `26.0`, `26.5.2`, `26.7.2`, `27.0`, `27.1.3`
and `28.0`; they rejected pre-26, malformed and unknown input. The local
source setup reached its private-agent argument refusal after passing the
actual 26.5.2 platform gate. Synthetic package assembly checked that the new
version module ships alongside setup in the inventoried package. The build-input
guard, locked offline Rust tests (**446 pass, 4 ignored**), clippy, formatting,
private-node adapter build and process tests (**145 pass, 2 skipped**), Python
suite (**189 pass, 1 skipped**) and explicit-path MCP smoke passed. The first
Python run failed because the changed architecture contract's skill-catalog hash
was stale; the catalog was refreshed and the full suite passed on rerun. This
failure is not counted as a pass.

No changed full package was assembled or installed, and no macOS 27 host was
available for native, Pi CLI/TUI, image or video verification. The published
0.0.1 archive and checksum remain unchanged and its setup still rejects other
macOS versions. The new predicate establishes installation eligibility for a
future assembled package, **not** macOS 27 support or a new published release.
Qualify the same built artifact on macOS 27 before claiming that outcome; a new
OS-specific artifact is warranted only if a real compatibility difference
requires it.

## 0.0.1 skill catalog routing rerelease, 2026-10-30

This package was qualified in private disposable state after the focused skill
adoption candidate below, then published as the current 0.0.1 rerelease. It was
checked on **macOS 26.5.2 (25F84), ARM64**
with Rust/Cargo **1.99.1**, private Node **24.22.1** (SHA-256
`6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`),
Pi **1.4.0** and TypeScript **5.9.3**. The assembled release ID is
`sha256:1ed77a59a1a2e881ab50a321d96172815d8b84059444633176d26b14459ab89a`;
core SHA-256 is
`adb2bd0dd33d4a533189b264c42008be0d2dbece4a74ba3584aeed55eb28c048`;
package inventory SHA-256 is
`c060c4fca31e8f31995067ada463996b3f4634d0b872005bde8403b1b1927e06`.
The package contains **13,356 inventoried files**. Its qualification assembly
and installation used `/private/tmp/mirket-pi-package-BAZuTr`; those tests used
no live user Pi, project settings, `.mirket`, `.pi` or `.agents` state. The later
authorized live installation has separate evidence below.
The new 0.0.1 archive is **461,000,129 bytes**, SHA-256
`5c0f99fe443d66edd77776f5c72d7009743a35d366a8906b1e00df6fcd7eff39`;
its six payload checksums pass. A safety-checked extraction found **15,349**
regular-file/directory members, including **13,357** files with the package
inventory itself. All 13,356 inventoried file sizes and hashes matched the
qualified package; the extracted native core independently verified the same
release ID with debug assertions disabled. Archive creation and roundtrip
receipts are under `/private/tmp/mirket-rerelease-0.0.1-20261030.DV1Uv7VW/`.
The first extraction checker over-rejected tar's standard equal long-path PAX
metadata; the corrected checker allowed only that path field and passed. The
failed checker attempt is retained and is not counted as archive qualification.

The build-input guard passed **before** the new Rust compilation. Locked offline
Rust tests passed (**446 pass, 4 ignored**), as did clippy, formatting and the
release build. The private Node TypeScript build and process suite passed
(**144 pass, 1 skipped**); Python discovery passed (**183 pass**). Catalog
refresh reported `changed: []` and 63 reference families; the route audit and
assembled skill links passed (73 Markdown files, 13 local links). Explicit
private-node MCP smoke passed against the packaged core. The full assembled
Pi integration passed **16 checkpoints**, including native effective selection
under `autoload:false`, selected missing-raster doctor status, local design and
navigation operations, scoped memory authorization, reload and project
isolation. Actual Pi CLI/RPC/TUI checks passed doctor, reload, fork/resume,
running codemode abort/recovery and clean PTY shutdown. Three fresh first-tool
samples were **4125.649, 1845.072 and 1847.170 ms**, passing the current
**5000/2500/2500 ms** gate; the prior 1500 ms gate does not pass.
The saved 640×240 PNG produced by the installed design route was then read by
Pi and delivered byte-for-byte to an offline image-capable provider fixture;
its source and delivered SHA-256 matched. This proves image transport, not a
model's visual interpretation. The receipt is
`/private/tmp/mirket-pi-package-BAZuTr/image-transport-report.json` (SHA-256
`6f4fb80d7fa6a72bdfc14029929318811a57bc00dbdee93f2700cfe9e80744a0`).

A separate **packaged-core Video linkage** trial used this exact release with
explicit packaged Node, adapter, FFmpeg and ffprobe paths and the retained
private external Chrome. It observed the adapter and Chrome running before
cancellation, verified that the cancelled operation left no destination or
scratch output, then completed a two-second supplied-audio MP4 on the same MCP
server. Independent decode found 24 H.264 frames at 640×360 and 96,000 mono
PCM samples from AAC; timing and gain checks passed. A source edit changed
8,310 decoded heading pixels, and the saved receipt matched the output. All
13,356 package files rehashed unchanged and no owned processes survived. The
first cancellation attempt used an insufficient process filter and is retained
as a failed oracle, not counted. The passing report is
`/private/tmp/mirket-final-video-ta3g_hxf/report.json` (SHA-256
`aedbcf065e536ef2963d16cf54755d295c507e8bc483adb3292acbfcfa941cd4`).
This checks the packaged core through development MCP flags; it does not prove
a rendered frame at cancellation, captions, heard audio, playback or visual
quality. An additional **installed Pi Video journey** used this same private
package with its explicit video Chrome binding and an offline deterministic
provider. Pi invoked the real `video_prepare/start/status/cancel` routes:
the two-second MP4 decoded to 24 frames with the expected audio timing/gain;
a changed source changed decoded pixels; a stale revision and remote asset
were refused; and in-flight cancellation left no published output and removed
owned scratch files. The full Pi session report is
`/private/tmp/mirket-pi-package-BAZuTr/domain-session-report.json` (SHA-256
`a7eabf83893f29e54f636d35a96313235d7514267e51e440c55d5b95cecc9a9a`),
with independent decoded, changed-source and failure evidence under
`/private/tmp/mirket-pi-package-BAZuTr/project/video-1793357289394/`.
The exact command, executable and fixture hashes are in
`/private/tmp/mirket-pi-package-BAZuTr/pi-video-qualification.json` (SHA-256
`deff9637ddf259b4aa2ad8ef673e91b5f0a8a1eecf1bc53cb2d23e334a8881d4`).
This establishes the Pi tool path and saved output checks with an offline
scripted provider, not model judgment or playback.

A separate [focused routing runner](../tests/skill-trials-routing.mjs) passed
six paired offline Pi cases (12 arms) on this release: source-only Video static
handoff, unconfigured public research refusal, local wide/narrow browser
snapshots and PNGs, account/form route refusal, selected missing-raster doctor
status and inherited skill exclusion under `autoload:false`. The native arms
used one fresh private installation with separate HOME, project, settings and
session state per arm. The browser grant was test-only. Its [private
report](/private/tmp/mirket-skill-trials-routing-CLsi82/REPORT.md) retains the
four corrected harness failures, exact private executables and source pins;
`report.json` has SHA-256
`df54d6dc94e63a1fe4a062d25197e7f59ec04f0c23aa36a523c64845d0f8d0d8`.
The deterministic provider measured route/context behavior, not model utility,
token savings or visual quality.

Three failed attempts were retained while repairing the fixture: the old Rust
binary correctly refused changed embedded catalog bytes; a refreshed binary
then reached a package assembler call missing the required Video source/notice
root; the first installed doctor call exposed Pi's `skill:` command prefix.
The Rust fixed-count check, fixture argument and doctor ID mapping were
corrected before the final full pass. These attempts are not counted as
qualification. Raw final evidence is at
`/private/tmp/mirket-pi-package-BAZuTr/report.json` (SHA-256
`24cf874f42d5d76e16c10544138d9a18db56df8b682ab8dc4704d716958b60d9`)
and `cli-lifecycle-report.json` (SHA-256
`9661bbdecd12dc931bde3b3b81b233f4792223b9142e8cb227e5de10152810ee`)
in the same directory. Reviewer source/build
logs are at `/private/tmp/mirket-review-b-gates-o0oe7klc/` and
`/private/tmp/mirket-skill-review-verify-ze0i6ex4/`. These private paths are
disposable; the identities and bounded results above remain the record.

The 305 catalog cases remain planned. This run established source/package
consistency and selected installed Pi routes, plus the bounded packaged-core
and installed Pi Video operation/cancellation above. It did not run held-out skill-utility,
image interpretation or visual-quality trials. Windows and other platforms
require their own native
qualification. The later publication and live installation do not expand these
bounded qualification results.

### Live installation and migration failure

After publication, the authorized live upgrade used the same archive and native
release identified above. The earlier `migrate-pi.py` helper (SHA-256
`296aa33c4f4e457faf18627217d1d4c1ddc3b6d18e5967f052a6074d149b49c5`)
failed after publishing the new installation because its `settings.json.lock` disappeared
during the long operation. It retained `failed-state-preserved` at phase
`outer-new-final-check-pending`, with a recovery record in
`/Users/nazmiyilmaz/.local/share/mirket/migration-backups/migration-ky84lqxt/record.json`.
No rollback was attempted. The failed invocation remains a failure; it does not
qualify that helper's complete migration or locking behavior.

A subsequent rerun of the exact `install-pi.py` installer reported
`already-installed`, and native doctor reported healthy with the new release
ID. A read-only audit verified all **13,356 package files** and **15,249 host
files**, and Pi reported **1.4.0**. Settings changed only the old Mirket package
source to the new one; unrelated settings and skill selections were preserved.
Authentication, model-store and managed Pi launcher target bytes and modes were
unchanged. The outer launcher matched the new installation, backup settings and
launcher matched their pre-migration hashes, and the previous active release
remained retained. These checks establish the resulting installation's identity,
preservation and native health, rather than a passing end-to-end
migration-helper run. Live-provider interpretation, visual quality and
unexercised live tool routes remain outside this installation check.

Raw failure and recovery receipts remain under
`/private/tmp/mirket-rerelease-0.0.1-20261030.DV1Uv7VW/`:
`migration-stderr.log` has SHA-256
`178da973e602bfe74683d77cf83932ed2131e363986a2c87598a02fe643720fe`;
`live-before.json` has SHA-256
`63ec212f81abfb9228736fa558e9357426b6675fb7362e5d2b5b71bb83f9a10c`;
`live-final-installer.json` has SHA-256
`a5e45bc88963fe422cbd44006f014967ff1da213728e8f149f57fa0f84ffb458`.
The later read-only `live-install-report.json` has SHA-256
`883221f52cf5f31059d203053a9b09ac20187404e89e5b799d7b4e60be97215d`.
Its first audit oracle incorrectly expected the backup launcher copy's mode to
be `0700`; the helper deliberately writes that private backup copy as `0600`
and records the original mode separately. The corrected oracle passed. This
failed audit attempt remains in `live-report-attempt-1.txt`; it is not a failed
installation or a migration-helper pass.

The repaired standalone helper has SHA-256
`c435719955cb6e80a9fcdeb9d130f9b3b3850a2777988491a211bf171e2ac25c`.
It renews the cooperative Pi settings lock during long operations and checks
lock ownership before publication. On macOS **26.5.2 ARM64**, Python **3.15.1**
and private Node **24.22.1**, its focused suite passed **46 tests, no skips**, in
**17.063 seconds**. Real Pi lock checks verified reclamation of a stale bare
lock and contention against the renewed lock after **11 seconds**. These repair
checks do not change the failed live invocation's result. Full Python discovery
with the explicit private package and host then passed **190 tests, no skips**,
in **59.358 seconds**. Its retained `python-final.log` in the rerelease evidence
directory has SHA-256
`ef468505368b7ab23ea83b83da7bbbc889b7f4889c445147b3274fbf75b06b3a`.

A separate disposable installation then exercised the repaired helper from
native release
`sha256:d2aa0c11ff58234c733f8f329047cdbf47c6f5341fbd866b9af712a9ba86c1fc`
to this rerelease. The helper exited zero with `status: updated` and backup
phase `complete`. New native doctor was healthy, and an independent exact
installer rerun reported `already-installed`. A restarted installed MCP server
read the old canonical fact with its original source revision. Authentication,
unrelated files, runtime selection and canonical SQLite bytes and modes were
preserved, as were unrelated settings, skill filters and the settings-file
mode. This is an end-to-end passing disposable migration, separate from the
failed live run. The repaired helper was not rerun against the live
installation.
The final disposable `qualification-result.json` is retained under
`/private/tmp/mirket-migration-real-20261030-j4370eit/`, alongside `REPORT.md`
and 21 hashed logs; its SHA-256 is
`27cd27cb6f0c524d40c725270883188d4b18f21d89af848305f7127c522bb7d7`.

## Focused skill adoption candidate, private full package, 2026-10-30

This is a new **unreleased** candidate from HEAD
`7437db7dcd386f6d65fa9ae044fbad2e1e09b7fe` plus the current working-tree
changes. It was checked on **macOS 26.5.2 (25F84), ARM64**, with Rust 1.99.1,
private Node 24.22.1 (SHA-256
`6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`)
and external Pi 1.4.0. The fresh native release ID is
`sha256:fecfeae99d324259a2e836039d442439922fe8d18fc7d7e5e3904c4f65d0a125`,
core SHA-256 is `255a40ac41580e33a947d4d01fa2149a7ff83a6af9cbe0ff47ba3602cb8c8d91`,
and package inventory SHA-256 is
`df50945aa32fb2fbadd3b212add630fd4aff1e1968c052fde48f1f1bc6bcee4c`.
All 13,356 inventoried package files matched their recorded bytes and hashes;
no links were present. The package was assembled and tested in
`/private/tmp/mirket-post-adoption-native-cix6a895/package`, not installed on
the user's device or published. The version remains 0.0.1.

The build-input guard ran before Rust compilation. Locked offline Rust tests
passed (**445 pass, 4 ignored**); clippy, formatting and release build passed.
The locked TypeScript build and process suite passed (**143 pass, 1 existing
skip**), as did the repaired Python suite (**181 pass**), 63-reference catalog
consistency, native/package verification and explicit packaged-core MCP smoke.
The first Python run failed on an obsolete expected resource list; the fixture
was updated for the new OpenPencil and frame-review resources before the full
rerun. An initial private package copy omitted a nested retained notice and
failed assembly; the complete notice tree was restored before the final
independently checked package. Both failed attempts remain in the private
report; neither is counted as a pass.

In a separate disposable Pi installation of this exact package, native resource
filtering loaded nine globals plus the selected OpenPencil and video-production
skills. Pi read both bodies and all five declared references byte-for-byte;
the TUI read the two new method references. CLI/RPC/TUI doctor, reload,
fork/resume, running abort/recovery and clean exit passed. Three first-tool
samples were **2883.358, 1894.737 and 1886.863 ms**, meeting the current
**5000/2500/2500 ms** first/repeat budget; the prior 1500 ms target remains
false. Exact PNG image transport, a source edit changing 24,894 decoded pixels,
stale-source refusal, running D2 cancellation/cleanup and recovery passed with
no owned MCP survivors. Seeded unrelated Pi settings were preserved.

One real supplied-audio MP4 operation through the **packaged core and adapter**
returned `audio_end_seconds: 2` and a matching saved receipt. Independent decode
found 24 H.264 frames at 640×360 over two seconds and AAC decoded to PCM; onset,
tail and two-tone gain checks passed. A separate current-adapter trial produced
a PNG sequence with caption boundary checks and showed the new stream-end guard
rejecting real 0.5-second AAC metadata against a two-second target. Two initial
captioned MP4 strict pixel-oracle attempts failed because the test strip also
contained an animated footer and lossy output variation. Those failures remain
recorded; the lossless PNG cue check passed. Frame review also found a
synthetic footer/caption overlap and insufficient reading hold, so it is not
presented as visual acceptance.

The focused OpenPencil typed trial completed 16 operations, reopened and patched
native source, and exposed then repaired a clipped long label. The HyperFrames
frame trial decoded two 13-frame clips with matching endpoints but a detectable
midpoint overshoot. These are bounded method checks, not held-out model
comparisons. All 305 catalog cases remain planned. Audible playback, live
provider image interpretation, native editor fidelity, Windows and other
platforms require separate tests. No live installation or client settings were
used, and Mirket-owned processes are not an OS sandbox.

Exact commands, executable identities, failures and raw outcomes are retained
outside shipped source in
`/private/tmp/mirket-post-adoption-native-cix6a895/REPORT.md` (SHA-256
`d1096693c13a457d43579135ffbc2b88d8cccae1e03a8ead2b235cfaae0a26ca`),
`/private/tmp/mirket-post-adoption-pi-q86l200_/REPORT.md` (SHA-256
`8a23f7ec81cc94602c3d5373042a217f5f73c39e46bc9d21b44f655808086d26`),
and `/private/tmp/mirket-packaged-video-link-iqeyghsp/report.json` (SHA-256
`7e178f0c643f87ff35ea623cbf157036c27b00368b595962d9fadffa2da7b38d`).
Source/rights receipts and the exact accepted/rejected method mapping are in
`packaging/provenance/skills/`.

## 0.0.1 project-onboarding rerelease, full package, 2026-10-30

This exact archive was checked on **macOS 26.5.2 (25F84), ARM64** with private
Node **24.22.1** and external Pi **1.4.0**. Its native release ID is
`sha256:d2aa0c11ff58234c733f8f329047cdbf47c6f5341fbd866b9af712a9ba86c1fc`,
core SHA-256 is `b8256d8061ab8224b26c94a70d59696c3052a7435cf3de023f0fe4f863629e7d`,
package inventory SHA-256 is `ee51700253ad7de510d9c4a6d32a837b3e65b2b861f212a28113119b4b4b4984`,
and release archive SHA-256 is
`780eba5b9624063d4c3bde06ffc9f46132c152c705203370b7109a327fc940ea`
(**460,983,272 bytes**). All six payloads matched the release `SHA256SUMS`.
Independent extraction checked all **13,352 inventoried files** plus the
inventory itself. The safe archive scan found **15,343 members**, with no links,
special files or unsafe paths; extracted native verification passed. The exact
assembly and roundtrip receipt is at
`/private/tmp/mirket-v0.0.1-head-f80eb55-nv30qoon/release-artifact-report.json`.
Archive assembly ran at source commit `f80eb553061edf43e2345fef0d4da5fa618a840f`.
The subsequent release commit adds only the standalone migration helper, its
tests and release documentation; none is an assembled package input.

A fresh native setup from that extracted full archive completed before the
unchanged **600-second** subprocess limit. Actual Pi CLI/RPC project onboarding
passed extension-filter repair, reload into **15 selected skills**, native
project memory prepare/commit/read, selected skill read, slash expansion,
idempotency, and changed-wiring refusal/recovery. Three fresh Pi CLI doctor
processes measured **1937.586, 1921.059 and 1908.535 ms**, passing the
**5000/2500/2500 ms** first/repeat gate. RPC fork/resume, running abort and
recovery passed. Pi TUI `/mcp`, literal `/reload`, doctor, project init/status,
repeat init and Ctrl-D passed with no owned process survivors. The TUI project
scenario used a second disposable project/session with the same successful
private agent; it is not counted as a second native install.

Ten installed domain checks passed: decoded D2 Dagre/ELK/TALA PNGs, exact Pi
image delivery, **24,894** changed decoded pixels after a source edit, observed
running D2 cancellation and recovery with no destination or owned process left,
UTF-8/CRLF navigation and stale-revision refusal, tokens/SVG/raster design,
and native memory. Saved original and changed PNGs were visually inspected for
legible Turkish labels, a connected arrow and the intended blue-to-peach fill
change. Full commands, raw results and failed fixture attempts are retained in
`/private/tmp/mirket-full-package-pi-head-bwvvy6te/`; the final `REPORT.md` has
SHA-256 `0bd3a57274db543aea47d7bf76773b449960ab62702c315fd309eef4339fbd31`.
Initial fixture errors (premature tests during extraction and an interrupted
redundant TUI installation) remain recorded and are not counted as passes.
These timing samples are neither cache-flushed nor percentile evidence. Offline
Pi image delivery is not live-provider interpretation, and this section does
not establish optional browser/video or other-platform behavior.

The standalone migration helper has **10 focused passing tests** for setting
and state preservation, busy/pending/unknown-layout refusals, failed-phase
records, installer-byte verification, duplicate JSON keys and a same-native-ID
package change. A separate real disposable fixture upgraded native release
`sha256:f7d5ca0b43729d222db840bba0b7e2e2c0192f0f9a763a7e119ee48f728d1800`
to the exact full-package release above in **313.363 seconds**. Native doctor,
Pi SDK registration and runtime status passed; a canonical memory record with
source/revision, selected skills, unrelated settings, synthetic auth/session
files and private backups were preserved. Repeating the exact installer reported
already installed without rewriting settings. A subsequent real JS-only
same-native-ID variant changed Pi package bytes while leaving native pointer,
executable bytes and mtimes unchanged; Pi registration and memory still passed.
Its initial old-package standalone setup timed out at the unchanged
**600-second** native gate during concurrent staging. The supported `--resume`
in that **same fresh fixture** passed in **50.490 seconds** after I/O contention
cleared; the failed first attempt remains recorded and is not a fresh-installer
pass. This fixture did not import a previous installation or touch live Pi.
Exact commands, failure logs and receipt hashes are under
`/private/tmp/mirket-install-path-audit-5q0_0__w/`; `evidence.json` SHA-256 is
`35cd3f9e92e6b5d1cd060fdf28981a97e368540ca729f2f86c066a08d4172f70`.
After publication, this device's existing Pi installation was migrated from
native release `sha256:26d6c76e7c25c8e690f9c6b7b4014134891c49b2c67a844a7d3c166deb834fe3`
to the exact new release above using the independently downloaded public assets.
All seven public assets passed `SHA256SUMS`; the public archive matched the
qualified archive byte for byte. The migration helper completed with a private
backup, native doctor reported healthy and `pi --version` returned **1.4.0**.
The Pi settings object differed only in the selected Mirket package path;
authentication retained its prior SHA-256, and the old package stayed in place.
The installed core and package inventory matched the release identities. The
post-install report is
`/private/tmp/mirket-v0.0.1-head-f80eb55-nv30qoon/live-install-report.json`
(SHA-256 `92126b4d5ce54d69381a3708c5413d2cf3be347421fc3f4f146cd9134ee37095`).
This device check establishes installation and registration, not a live-provider
answer or another platform.

The final source tree's build-input guard passed for
`aarch64-apple-darwin` before any Rust compilation in this release pass. The
full Python suite passed **181 tests** with Python **3.15.1**, disposable HOME,
Pi agent and session directories; its retained stderr log is
`/private/tmp/mirket-release-final-source-tests.odpXH0/stderr.log` (SHA-256
`10f2078abdf31836763a2351762127271a5d606e93ffa0cf2e58e9195620930c`).
The earlier source candidate below records locked Rust, clippy, TypeScript and
MCP gates for this same core SHA and compiled modules. The full package's fresh
MCP and Pi outcomes above and below are separate checks of its assembled bytes.

Independent installed native MCP checks on the same full package passed
navigation, tokens/SVG/PNG, D2, semantic memory and SQLite backup integrity,
OpenPencil editing, stale/remote refusals, cancellation and clean EOF. All
three optional tools began absent with bundled routes ready, then exact
external Chrome-for-video, separate browser and Playwright peer bindings passed
real capability probes. The optional payloads were copied from previously
publisher-qualified private fixtures; they are not in the default package.
Video output independently decoded to **24 H.264 frames at 640×360 and 2.000 s**
with AAC audio; PCM checks found a **2.0008:1** voice/music gain ratio, and a
changed heading changed **4,929 decoded pixels**. Remote-asset refusal and
in-flight cancellation published no output. The initial full native setup
timed out under concurrent staging at the unchanged **600-second** gate; the
same fresh fixture's supported `--resume` passed in **62.783 seconds** once
staging ran sequentially. A first video reedit fixture used the wrong heading
and failed its input prerequisite; the corrected input passed the unchanged
test. Both failed attempts remain retained. Exact commands, outputs, pixels and
media are under `/private/tmp/mirket-full-runtime-qualification-3czvt0ei/`;
`report.json` SHA-256 is
`411a54ab7716e34d9f1a5a39c72e7a3e3ca5f66bf40a04fb527e8d64c39e5542`.
These native checks do not establish whole-browser authorization, live-provider
visual interpretation, source replay, code signing or another platform.

## Project setup source candidate before this rerelease, 2026-10-30

The project setup change was checked on **macOS 26.5.2 (25F84), ARM64** with
Rust **1.99.1**, Python **3.15.1**, private Node **24.22.1** and the exact
external Pi **1.4.0** host. The skill catalog was refreshed after the
architecture/skill documentation changed, then the native core was rebuilt.
At the time, this was a fresh, disposable **minimal source package**. Its
results did not qualify the full 0.0.1 archive or every bundled creative runtime.

- Native release: `sha256:dfa61e5648a1a54cb52514c7f8ccccb53022109eee84c585899553abeab297a6`.
- Native core SHA-256: `b8256d8061ab8224b26c94a70d59696c3052a7435cf3de023f0fe4f863629e7d`.
- Minimal package inventory SHA-256: `d8a3808d1743e27f2e93d4998455f112961ad21070768dbdc2e1a1b8b153b85a`.
- Private Node SHA-256: `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`.

The build-input guard passed before Rust compilation. Locked offline Rust
tests, all-target clippy with warnings denied, formatting, the release build,
**171 Python tests**, the locked TypeScript build and process suite (**142 pass,
one existing skip**) against the admitted Pi peer tree, catalog consistency and
MCP smoke passed using
explicit executables. Focused project setup and observation tests cover
selection preservation, duplicate/malformed guidance, drift, symlinks and hard
links, scoped opt-in, successful-use filtering and failure isolation.

An actual Pi CLI/RPC session in a private project passed project init, native
Pi reload into **15 selected skills**, native project memory prepare/commit/read,
verified selected skill read, idle slash-skill expansion, idempotent init and
status. The next provider turn contained the updated managed `AGENTS.md`
section; manual guidance and unrelated global Pi settings stayed byte-identical.
An existing project package entry with `extensions: []` was repaired to the
exact Pi extension filter; its skill filter, prompts and unrelated settings were
preserved, and actual reload retained Mirket commands and tools.
Removing the project Pi package object changed sync status to `changed`; an
otherwise successful new native search did not rewrite guidance. Restoring the
settings recovered `ready` without reinitialization.
The CLI exited cleanly with no owned process-group survivors. Its report is at
`/private/tmp/mirket-pi-project-3aLToK/project-setup-report.json` (SHA-256
`19e08e166669de99bcd9a8704d52840d34edd9f4b113446cadea877572b6382d`).

Actual Pi TUI in a second private home/project visibly reported disabled,
initialized the project, completed literal `/reload`, reported ready, and
repeated init without changing source or settings. It preserved manual guidance,
the selected Pi filter and unrelated settings, then exited via Ctrl-D with no
owned survivors. Its raw PTY transcript and report are at
`/private/tmp/mirket-pi-project-tui-ugbhpdx5/`; the report SHA-256 is
`839294b2134ea56a52b24a687971f81f359a63181eea13e5b4c822941bef6b1f`.

Failures remain in the private fixture history: an initial TypeScript build
omitted `MIRKET_PI_PEERS`, a type error in the new context-file hook was fixed,
the first CLI init exposed a trailing-slash package-root bug, RPC `/reload`
proved TUI-only and the fixture switched to Pi's public `ctx.reload()`, and the
first TUI assertion needed to account for Pi's own changelog setting. Corrected
fresh runs above passed; those failed attempts are not counted as passes.
Review also found and fixed an initially stale `ready` result after removal of
the project Pi package entry; the final negative CLI case above checks that
refusal. The extension filter repair was subsequently checked through actual
Pi reload against the final source candidate.

Project sync records observed Mirket use, not output quality or current tool
readiness. The optional browser peer's successful-call recording passed its
controller lifecycle tests with synthetic bindings; a fresh real browser-peer
operation with this changed package was not run. Plain idle slash activation
passed in real Pi; queued slash activation has mocked
hook-sequence evidence only, and slash prompts with image-normalization hints
are conservatively skipped. No live installation, release, other platform,
paid provider or image-delivery qualification is implied.

## Prior 0.0.1 rerelease candidate and revised startup budget, 2026-10-30

This candidate targets **macOS 26.5.2 (25F84), ARM64**, external Pi **1.4.0**
and private Node **24.22.1**. The native core was rebuilt because it embeds the
skill catalog; the earlier native identity does not qualify these bytes.
Fresh setup and Pi CLI/RPC/TUI lifecycle checks passed on the extracted final
archive, as did ten installed domain checks. Fresh clones of the same
inventoried package bytes passed 24/24 optional browser cases and five video
commands. Standalone-installer fresh and existing-managed-Pi fixtures also
passed.

- Native release: `sha256:26d6c76e7c25c8e690f9c6b7b4014134891c49b2c67a844a7d3c166deb834fe3`.
- Native core SHA-256: `38f81be0ba6261b6b194b514641483ca0938cfb445105b089eb0d7861ecb8489`.
- Assembled package inventory SHA-256: `396991cfe581e8fb3a3cf67909430ae42d6afc16027c71e27a5b5f3067ce8abf`.
- Release archive SHA-256: `4bdc35bc5933f014485d22260a87d1f059e442a70aa5b54782f97c80b90d4d83`; **462,041,409 bytes**, **13,350 inventory entries**.

All five payloads matched `SHA256SUMS`; the extracted package matched its core
and inventory identities. The archive was extracted into a new private fixture.
Its safe member scan found
15,341 members: 13,351 regular files including the inventory and 1,990
directories, all under `package/`, with no links or special members. The replaced
0.0.1 archive had SHA-256
`72c3c78951d4e153e0592170e1b25d50038f3ffaef91a48444f57e1d2443b106`;
its package and original failed startup result remain separately recorded below.

The build-input guard passed for `aarch64-apple-darwin` before Rust compilation.
Locked offline Rust tests passed (**441 library, 3 binary and 1 fixture pass**;
ignored cases remain ignored), as did all-target clippy with warnings denied,
formatting, catalog consistency, **171 private-HOME Python tests**, the locked
TypeScript build and process suite (**112 pass, 1 existing skip**), and MCP
smoke against the explicit release binary above. Rust was 1.99.1 and Python
3.15.1; integration state stayed in disposable private directories. Supplemental
private-HOME guard attempts with missing toolchain/cache environment failed;
explicit paths restored a pass, with both failed logs retained.

The user authorized replacing the existing 0.0.1 assets and revising the startup
budget. The lifecycle driver now requires exactly three finite, nonnegative
first-tool measurements: the **first observed process must return its first-tool
result within 5000 ms**, and **each of two repeat fresh processes must do so
within 2500 ms**.
The first observation is simply the first process in a disposable fixture;
there is no OS page-cache flush or percentile claim. The driver retains the
raw timings and the previous **1500 ms** result, and exits nonzero for missing,
invalid or over-budget evidence. Six focused driver tests passed with Python
3.15.1 on this platform; receipts are outside shipped source at
`/private/tmp/mirket-startup-gate-tests-kfyqx1uf/`.

Initial concurrent browser and video fixture setups both exceeded the package's
600000 ms native setup timeout before any Pi functional operation. Their failed
attempts and raw output remain at
`/private/tmp/mirket-browser-candidate-20261030-rerelease-n07oof00/` and
`/private/tmp/mirket-video-final-20261030-2a5my0za/`. Both setups subsequently
completed through the supported `--resume` route (**50.212 seconds** for
browser, **92.596 seconds** for video). Browser and corrected video functional
checks passed. The initial video scratch TUI driver race ended Pi early and
left one Chrome `--version` process, which was reaped. That failed attempt and
the correction remain retained; only the corrected rerun counts as a pass.

Fresh first-tool samples were **2854.792, 1984.948 and 1889.654 ms**. All
passed the revised budgets; all still failed the historical 1500 ms target.
The driver exited zero and retained both outcomes. Actual Pi CLI doctor,
RPC reload/fork/resume, running codemode abort/recovery, and PTY
`/mcp`/reload/doctor/Ctrl-D passed in new disposable HOME, agent/session and
project directories. The fixture changed only the selected final agent paths;
its diff, raw JSONL, stderr and PTY transcript remain at
`/private/tmp/mirket-v0.0.1-overwrite-20261030.4Zx6RD/`.
`cli-lifecycle-report.json` has SHA-256
`4aaf13756ecbaf09b84942bbda72b8139ad018fb5e38783e95f29c864053609a`.

The exact extracted archive also passed **10 installed domain checks** with
zero errors and zero owned MCP processes after shutdown. D2 Dagre/ELK/TALA
outputs decoded as PNG; a palette edit changed **24,894 pixels**. Pi `read`
delivered exact PNG bytes (SHA-256
`166af748c1995f7e7189efd4790ee73c9cdb0f566c3ff71643b2ac52a32e6036`)
to an offline image-capable provider fixture. Stale input refused, running
cancellation cleaned up without a destination, and a fresh PNG recovered.
Selected navigation, token/SVG/PNG design exports and canonical project memory
passed their bounded checks. Image transport does not establish live-provider
visual interpretation. `combined-pi-report.json` at the same root has SHA-256
`34dbf0666135cf0372c5b57b30df987a833553cf9e7163f08156eeaa3124c26b`.

A fresh clone of the **same inventoried package bytes** passed **24/24
optional browser cases** in its separate fixture above. Actual Pi CLI/TUI
covered Skip, installed selection and pinned publisher acquisition, scoped
allow/deny/revoke, local responsive navigation, wide/narrow viewport PNGs and
exact Pi image delivery, origin/method refusals, and path/byte/resource drift.
Saved wide/narrow and pinned-path PNGs were inspected.
Escape reaped all **eight** observed old MCP/browser identities in **460.904 ms**;
zero owned survivors were observed before or after harness cleanup. No PATH trap
ran; unrelated settings were preserved, with only Pi's own
`lastChangelogVersion` added. This covers a synthetic loopback app and seven
selected tools; the application proxy and owned processes are not an OS sandbox.
The browser `report.json` has SHA-256
`f4ea0b7aa2dd02cdb6b3f5d2107606cf5c96eb1fa30c3a1d073e00b1c80bd851`;
its `REPORT.md` has SHA-256
`66ff8c7b16895f4a601565e4599083852f6722822909e1a1dea867cd0644acc6`.

The separate video clone of the **same inventoried package bytes** passed
**five commands** with empty stderr. Pi TUI selected installed Chrome for video
while both browser tools stayed skipped, and doctor reported ready. The actual
Pi export decoded to **24 H.264 frames at 640×360, 12 fps, over two seconds**
with AAC audio. Independent PCM checks confirmed timing and a **2.000798**
voice/music gain ratio. Changed source changed **30,839 decoded pixels** and
the inspected heading. Stale input and remote assets refused; running
cancellation published no output and cleaned scratch. No survivors remained
among the **76** sampled owned process identities after the corrected run. The video
`run-report.json` has SHA-256
`129397541762ede8bbbf3cf768df4edf3cc2d11da484675981db6f1def690968`;
its `REPORT.md` has SHA-256
`e2df3a631d0c6026699b57abe4e12dda497e39c59fa9122e5b86a2af67f9d831`.

The standalone installer passed both **fresh no-Pi** and **synthetic existing
managed-Pi** fixtures using the final package. Both returned `installed`, then
`already-installed` on repeat, and their actual outer launchers reached CLI
doctor `ready`. Settings, authentication, sessions and original launcher backups
matched the seeded bytes. The existing route also passed actual TUI `/mcp`
with 25 tools and clean Ctrl-D exit. Initial slash doctor rendered `not_ready`
with `Awaiting actual runtime_status identity` before the actual doctor tool ran.
An enhanced small-viewport assertion then timed out after the actual tool had
returned ready; its transcript remains retained as a failed harness attempt.
The final larger-viewport TUI showed actual `mirket_doctor` ready, subsequent
slash doctor ready, and Ctrl-D exit zero. Zero owned survivors and no unresolved
failures remained. These tests used disposable HOME
and project directories, with the exact acquired Pi 1.4.0 host; they did not
modify the user's installation. Receipts are at
`/private/tmp/mirket-final-installer-20261030-lrtrnilb/`; `report.json` has SHA-256
`1df1bf9f85c278238c880d0e3c2db3eb8f2058b49a6ecee61c65d7435fe2478a`;
its `REPORT.md` has SHA-256
`a65dd509f74a42fc1b39c0b4eca83ca855169b01d4accacef766936f8ab007f4`.

Appending a byte to the installed Rust core made actual Pi doctor report
`not_ready` with `Host verifier identity mismatch`. Exact restoration recovered
`ready`. The retained
`/private/tmp/mirket-v0.0.1-overwrite-20261030.4Zx6RD/installed-byte-mutation.json`
records the original, changed and restored byte identities.

This prospective budget revision does not turn earlier 1500 ms failures into
passes or establish a speed improvement. The historical samples and their
original outcomes remain below. Other platforms, live-provider visual
interpretation and the FFmpeg source replay need separate qualification.

## Pragmatic method and startup memory candidate, 2026-10-30

This **unreleased** macOS 26.5.2 (25F84) ARM64 source candidate was checked
with Rust 1.99.1 and Python 3.15.1. It keeps native release
`sha256:f7d5ca0b43729d222db840bba0b7e2e2c0192f0f9a763a7e119ee48f728d1800`,
Pi 1.4.0 and private Node 24.22.1. Its assembled package inventory is
`c2e826eb99f860985e407298648762891a4de04edf28be0cfe5d3e927e8ff66a`.
Against the published package, only the compiled `host.js` and `runtime.js`,
three existing skill bodies and catalog changed. Ponytail's pinned method was
adapted into existing guidance; no plugin, mode or runtime dependency was added.
An independent review found and fixed a real Pi Escape-buffer race in the TUI
qualification driver. This is practical quality evidence, not a measured
model-efficiency comparison.

The build-input guard, locked offline Rust tests (441 library pass, two ignored;
three binary pass; one fixture pass, two ignored), all-target clippy, formatting,
catalog consistency, 169 Python tests, locked TypeScript build and process tests
(112 pass, one existing skip), and explicit release-binary MCP smoke passed.
Fresh disposable Pi CLI, RPC and TUI functional journeys passed, as did ten
installed domain checks: D2 PNG decoding, 24,894 changed pixels, exact PNG image
delivery to an offline image-capable provider fixture, running cancellation
without published output, recovery, navigation, design and memory. Zero owned
MCP service processes remained after shutdown. First-tool samples were
**2730, 1865 and 1873 ms**, so the unchanged **1500 ms startup gate failed**
and the lifecycle driver correctly exited nonzero after saving the functional
report. A separate paired frozen-package Pi comparison measured about **277 MiB
lower median peak RSS** with streaming executable hashes; its interleaved timing
samples did not establish a speed improvement. Other platforms and live-provider
visual interpretation remain untested. Raw candidate receipts are under
`/private/tmp/mirket-ponytail-candidate.IhVzg9/` and
`/private/tmp/mirket-startup-profile-final-1hd73l1w/`; they are not shipped.

## Browser onboarding and direct D2 PNG candidate, 2026-10-30

This is fresh installed-product evidence for the **parser-fixed final package**
on macOS **26.5.2 (25F84), ARM64**, external Pi **1.4.0**, Rust **1.99.1**
and private Node **24.22.1** (SHA-256
`6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`).
Two independent installations used new disposable HOME, Pi agent/session and
project directories under `/private/tmp`; neither imported an earlier Mirket
installation or touched the user's live settings. Superseded package trials and
failed test-harness attempts remain in separate scratch evidence, not as passes
for this artifact.

- Native release: `sha256:f7d5ca0b43729d222db840bba0b7e2e2c0192f0f9a763a7e119ee48f728d1800`.
- Native core SHA-256: `404bec4d75f68f91fec45b61a583d6902492fa1f88287b93663e4527bf27d97a`; release verification reports `debug_assertions: false`.
- Assembled package inventory SHA-256: `275e91c346e6a80225735892b0c1cfc0fb4acdfe62eba72e37b34fe4d0592973`, **13,350 files**. Its compiled extension and native/source/notice inputs were independently matched to the frozen source. No Playwright, Chrome or Raster publisher runtime bytes are in the default package.

The build-input guard ran before Rust compilation. Locked offline Rust tests
passed (**441 library pass, 2 ignored; 3 binary pass; 1 fixture pass, 1
ignored; manual scale test ignored**), as did all-target clippy with warnings
denied, formatting, the release build and explicit-release MCP smoke. The
locked TypeScript build and full process suite passed (**108 pass, 1 existing
skip**); the Python suite passed **124 tests**. A new Rust regression covers a
ready video Chrome alongside the two optional browser choices and bounded
browser tree metadata. The package assembler rejects debug-assertions builds.

Fresh setup of the exact package preserved unrelated seeded Pi settings. Actual
Pi CLI/RPC/TUI passed doctor, MCP, reload, fork/resume, running RPC abort and
recovery, and Ctrl-D shutdown. Five installed tamper cases refused changes and
returned healthy after exact restoration. With **all three optional tools
skipped**, the native runtime status reported video Chrome missing rather than
changed; bundled navigation, design, memory and D2 remained usable. D2 produced
decoded Dagre, ELK and TALA PNGs without Chrome onboarding. Pi editing changed
**24,894 decoded pixels**; saved images retained legible Turkish labels and a
connected arrow. Pi delivered the exact PNG to an offline image-capable provider
fixture. Stale input and a running native D2 cancellation published no failed
output; a fresh job recovered. This proves image transport, not a live model's
visual interpretation.

The video-only choice with both browser tools skipped passed actual Pi
`video_prepare`, directly exercising the prior Rust choice-parser failure. With
independent browser and peer trees also bound, all three choices reported ready
and an actual Pi video export decoded to **24 H.264 frames at 640×360 over two
seconds**, with AAC audio. Independent PCM checks covered timing and a roughly
2:1 voice/music gain ratio; a changed source changed **30,839 decoded pixels**
and the inspected heading. Stale revision and remote asset requests refused,
and a running video cancellation cleaned up without publishing an output.

The separate browser fixture passed **24/24 actual packaged Pi CLI/TUI cases**.
TUI `/mirket setup` exercised **Skip**, **Use installed tool** with an exact
selected Playwright tree and separate browser app, and **Install pinned
version** using user-confirmed publisher URLs, sizes, hashes and terms. The
pinned private output was independently checked against the twelve selected
Playwright files and the browser's 344 regular files/five links. Both ready
choices navigated a real local page; the installed path checked accessible
Turkish text, reduced motion, keyboard state change, wide/narrow layouts,
viewport PNGs, Pi image delivery and isolated browser storage. The pinned path
also produced a fresh snapshot and PNG. Explicit TUI authorization, denial,
revoke, origin/redirect/CONNECT/POST/attachment refusals, hidden extra tools,
full-page screenshot refusal, namespace aliases, changed bytes/resources and
same-byte alternate path refusal passed. No PATH trap ran. An actual Pi
**Escape** interruption ended all **eight** observed old MCP/browser process
identities in **181.859 ms**, before the held HTTP response was released or a
replacement registered; a fresh authorized grant then navigated. Zero owned
survivors were observed at the end. Pi added only its own
`lastChangelogVersion` setting; pre-existing client fields were preserved.

**Startup timing still fails.** The quiet final-package first-tool samples
were **4.097, 1.911 and 1.839 seconds**, all above the unchanged **1.5-second**
gate. Every sample is retained. This package is a functional local candidate,
not a performance-qualified release; no gate was waived. The optional browser
route was exercised only against a synthetic loopback HTTP app and seven
selected tools. The application proxy and owned-process cleanup are not an OS
sandbox. No live provider/token benefit, remote or authenticated browsing,
whole-app codesign, or other-platform result is claimed.

The browser peer stays **external-only**: its `proxy-agent-negotiate@1.1.0`
bundle declares MIT but supplies no license text, so redistribution clearance
has not been established. Raster/Sharp remains excluded. The delivered FFmpeg
source packet and replay recipe were checked as package inputs, but the replay
has not been freshly executed; generated wrapper/source correspondence remains
conservatively scoped. Windows and other native platforms require their own
packaging, ownership and capability tests.

Raw final receipts are outside shipped source at
`/private/tmp/mirket-combined-pi-qualification-20261030-vxddb7xe/` and
`/private/tmp/mirket-browser-candidate-20261030-h-n4xz1ic4/`.
The core `REPORT.md` has SHA-256
`fb866f32c4d40ff5d01f3b7181710b68aa24e9b2454be207a88a1c1bae1ffbef`;
its `SUMMARY.json` has SHA-256
`f7c644fb2c43252eb3a639da38890c3093dd275655f6460140ba85c5731d81e0`.
The browser `REPORT.md` has SHA-256
`c34cd3a7988af518a7150dd88fa0efaa74fa7aecab3dbc443f46296929218439`;
its 24-case `report.json` has SHA-256
`4f9826db79ad7ace04cb82174c47dbec1b8b9210eb1585a0ecb8897f4b67e078`.
Earlier candidate packages and their failures are retained in their own roots;
their outcomes do not qualify this package.

The subsequent standalone-installer addition kept this original package archive
unchanged. The installer passed 29 focused tests and the 165-test Python suite.
Disposable existing-Pi and fresh-Pi installations both reached actual Pi CLI
doctor `ready`; the existing-Pi route also passed a Pi TUI check. The failed
first probe and successful recovery were retained outside shipped source. These
installer checks belong to the original package, not the rebuilt rerelease.

## Packaging and onboarding candidate, 2026-10-29

This records fresh evidence for the packaging/onboarding change set on macOS
**26.5.2 (25F84), ARM64**. It does not extend to Windows or another
native platform. The test used disposable HOME, Pi agent/session directories and
projects under `/private/tmp/mirket-onboarding-qualification.MNBJjZ`; it did
not touch the user's installation. The tested local package was assembled from
the final runtime and notice inputs without Chrome or Raster in its default
payload. Pi 1.4.0 remained the external product host.

- Native release ID: `sha256:1e42a96ef7c776646969f1c3d0c2f57c9852efe62afb8b6b981d6c6cbe478708`.
- Package inventory SHA-256: `1b7066dec5a4f58e183607b6f56feb7a991e6786960bb062aa09b478d8efd40d`.
- Private Node 24.22.1 executable SHA-256: `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`.
- Selected and pinned Chrome for Testing 156.0.8078.73 executable SHA-256:
  `9c5ab4582272b583d209b854fd7c494c50f11026c5728f84b4ffb7001a9e5e45`.
- Publisher Chrome ZIP SHA-256:
  `a46be48ee2e33b3edc1a98a39668b9f0abb8f23f031bc4df195e2388e6de28d1`.
- Rust 1.99.1; private Node v24.22.1. The assembly fixture contains 13,347
  files and occupies about 852 MiB. Its inventory has no Chrome or Raster
  payload. The shipped license, third-party notices, Video packet and FFmpeg
  source packet were verified in the assembled artifact.

The build guard passed before Rust compilation. `cargo test --locked --offline`
passed (435 library tests, 3 binary tests, 1 memory fixture; 2 library and 1
scale/manual cases ignored), as did `cargo clippy --locked --offline
--all-targets -- -D warnings`, `cargo fmt --check`, `cargo build --locked
--offline`, the locked TypeScript build, 96 TypeScript process tests (1
skipped), 124 Python tests, and `tests/mcp-smoke.mjs` using the explicit private
Node and native paths. Source/package manifest verification and a fresh setup
of this final package passed; an earlier setup of the same native release also
passed. The installed Pi CLI/RPC/TUI checked
doctor, reload, fork/resume, abort/recovery, `/mcp` and Ctrl-D shutdown.
An offline image-capable Pi provider fixture received the exact PNG bytes from
an installed design output through Pi's `read` tool (SHA-256
`11ad05a70be560cb79243cff441867307294d3a13d99ead599c404a5036118a4`).
This checks image transport, not a live provider's visual interpretation.

Actual Pi TUI `/mirket setup` exercised **Skip**, **Use installed executable**
and **Install pinned version**. The installed choice ran a local JavaScript DOM
capability probe; ten independent fresh bindings passed. The pinned choice
downloaded the exact Google ZIP from the publisher, verified its archive and
every member/link, then passed the same capability probe. Pi doctor reported
Chrome `missing` after no selection and `ready` after the pinned selection.
Process tests covered cancellation, bad or corrupted download, changed bound
bytes, Retry and no PATH fallback. The changed-byte prepare/start refusal also
passed in Rust. The original Chrome probe intermittently failed during process
teardown; the final probe configuration passed ten fresh bindings. Both the
failed and passing receipts are retained in the disposable evidence directory.

The default package passed installed D2 Dagre/ELK/TALA and Pi navigation,
token, SVG and decoded resvg PNG journeys without Chrome onboarding, including
after Skip. A separate installed Pi journey with Chrome explicitly skipped
passed cross-language semantic recall, independent SQLite backup inspection,
correction, authorized forgetting and managed backup restore/reindex. With an
explicitly selected installed Chrome binding, an actual Pi video export produced
a decoded 24-frame MP4. An independent ffprobe/PCM oracle
checked audio timing and gain; changed source changed decoded pixels. Stale
revision, remote asset and in-flight cancellation checks passed with no
published failed output. This qualifies those observed paths, not all possible
media content or provider interpretation.

**Startup timing failed.** Three fresh installed Pi CLI first-tool samples of
the final package were **3.867, 3.682 and 3.793 seconds**. A quiet repeat after
setup I/O stopped measured **4.004, 3.687 and 3.740 seconds**. Both cohorts exceed
the recorded 1.5-second gate (and the earlier 2.5-second full-package gate).
The CLI/RPC/TUI functions passed, but this candidate does not meet that timing
target. Profiling found approximately 0.85 seconds in Pi initialization,
0.71 seconds verifying the external Pi host and 2.96 seconds in full native
payload verification in a run with competing setup I/O; these are diagnostic
samples, not a controlled
attribution. The new descriptor/selection/Chrome identity status check averaged
0.489 ms over 20 runs (1.900 ms cold). No startup gate was waived.

The delivered FFmpeg replay recipe was transcribed from the exact build records
but has **not** been freshly rebuilt; generated-source correspondence for some
bundled wrapper bytes remains conservatively attributed rather than
independently reconstructed. Raster/Sharp remains excluded pending its native
rights/source closure. Windows packaging, ownership and capability fidelity,
other platforms, live installation, larger corpora and unexercised routes need
separate qualification. Mirket-owned processes are not an OS sandbox.

The raw local receipts are outside the shipped source, principally at
`/private/tmp/mirket-onboarding-qualification.MNBJjZ`,
including its `memory-domain-report-1793289743087.json` and decoded video
operation files and `image-delivery-report.json`,
`/private/tmp/mirket-onboarding-final-network-evidence.json`,
`/private/tmp/mirket-stable-bind-evidence.json`,
`/private/tmp/mirket-chrome-process-trial.json`, and
`/private/tmp/mirket-packaging-evidence-oyazpoc1/checks.json`. These are
disposable local paths; the hashes and results above retain the identity and
limits of this trial if those paths are removed.

## Earlier Pi package candidate

The refactored Pi package was tested on **macOS 26.5.2 (build 25F84), ARM64**
on 2026-10-29. The tested working tree was HEAD
`9ddf25cef144b680aa741fef03f340315f88c81b` plus uncommitted changes;
the package identity below binds the tested artifact more precisely than HEAD.
Private receipts are retained outside the repository. The later repository
cleanup and subsequent onboarding/package changes are separate revisions; the
results below do not qualify them.

## Identity

- Package: `@codemirket/pi-harness` 0.0.1; one Rust crate and one compiled
  TypeScript package. External unbundled host: Pi 1.4.0 with Undici 8.12.2;
  exact historical host review is recoverable via [historical evidence](history.md).
- Native release: `sha256:6a80efe8b79381b2c26d7d10f4a02169e44820f860a10969573dd73c09accd11`.
- Source and installed Rust executable SHA-256:
  `c79475802ccc5a0c1afa6bf3798bacc2d3c4e4b44d7c045fb93c2985c8c8a746`.
- Assembled package inventory SHA-256:
  `c64393f4ebf97c13830abb3cc10924a6835369ab65b34e6cc7404673bff91066`.
- Private Node 24.22.1 SHA-256:
  `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`.

## Earlier results

The build guard, locked offline Rust tests, clippy, formatting and release build,
explicit-path MCP smoke, TypeScript process suite (**84 pass, 1 skip**) and
Python suite (**123 pass**) passed. Exact assembled-package Pi integration
passed **15/15 checkpoints**. Four sequential installed journeys checked
domain operations, semantic memory, editable design and video with file,
SQLite, decoded media and cancellation oracles. Package/source inventory and
native binding matched the installed fixture.

Three fresh full-package Pi CLI first-tool samples were **2.002, 2.066 and
2.059 seconds**, under the 2.5-second full startup gate. At 30, 60 and 120
seconds after doctor, sampled Pi-plus-Rust RSS was **291.42, 174.55 and
174.55 MiB**; Rust RSS was **31.13 MiB** throughout. These pass the 450 MiB
aggregate, 40 MiB Rust and at most 16 MiB growth gates for warm idle.
The startup 0-second aggregate sample was 467.91 MiB; it is outside the
30-second warm-idle gate. Pi EOF left no observed owned children after
**0.092 seconds**, under the 3-second shutdown gate. These are local samples,
not percentile or cross-platform guarantees. The separate 1.5-second core
startup gate was not measured in this full-package resource cohort.

Installed Pi consumer checks passed CLI/RPC doctor, reload, abort/recovery and
PTY shutdown; visible TUI denial/acceptance of a memory-forget operation; and
byte-exact PNG delivery to an offline image-capable provider fixture. The TUI
result qualifies that tested effect, and image transport does not establish
provider interpretation. These checks used disposable HOME, Pi agent/session
directories and project paths outside the real home ancestry.

## Limits

The offline image check covered transport, not provider-native image
interpretation. Individual effect routes not exercised in the current journeys,
larger memory corpora, ignored manual Rust cases and a separate reduced-motion
video export remain unqualified. All 305 catalog skill cases are planned, not
completed trials. The synthetic memory retrieval result has its own
[evaluation contract](memory-evaluation-contract.md).

The recipient source manifest is unrun. Windows ownership, packaging and native
capability fidelity need separate qualification; other platforms and live user
installation were not tested here. Mirket-owned processes are not an OS sandbox.

See [native build instructions](native-build.md) for reproducible checks. Do
not use the user's existing installation or settings as a qualification fixture.
