# Browser release path proposal

Research observed 2026-10-25. Scope is the first **Codex-only** release on macOS
ARM64, Windows x64 and Windows ARM64. This proposal changes no admission, product
scope, runtime, installation or client configuration. No proposed browser below
is presently admitted as a complete release payload.

**Recommended next repair: source-build unbranded Chromium, generate its actual
credits, and freeze the complete required notice/source/build materials with the
result.** This retains the current Playwright `chromium`/explicit-executable
contract. It removes reliance on a snapshot whose credits were deliberately not
generated. Decision 83 selects Chromium as the sole browser. Its required notice/source,
platform and actual desktop activation gates remain in force.

There is no verified ready-to-ship replacement covering all three targets in this
investigation. A feasible upstream source route is different from a completed
binary, licensing and platform admission. In particular, building open-source
Chromium does not establish that every copied Windows SDK/runtime file is OSS.

## Selected release route

Source-built unbranded Chromium keeps the existing Playwright Chromium API and
explicit private executable contract. It requires a controlled build, complete
required notices/source materials and native target checks. No runtime downloader,
alternate browser or automatic fallback is part of the release route.

The measured existing macOS Chromium candidate occupies 378,761,212 regular
bytes. This measures the blocked snapshot, not a source-built payload or a
promised final size. Source archives and notices still need separate accounting.

## Concrete source freezes to investigate

The 2026-10-25 Chromium Dashboard capture reported different Stable source
versions for Mac and Windows. Their metadata and the corresponding source files
were retrieved in that investigation; no browser was built or installed. Use these exact candidates
for a controlled build experiment, then recheck security status at the actual
release freeze. They are not a silently selected replacement for the admitted
Playwright package or the blocked snapshot.

| Target | Candidate source version / immutable Chromium revision | Captured DEPS SHA-256 |
| --- | --- | --- |
| macOS ARM64 | 156.0.8078.48 / `d0c02bfad08ec92a6ffcf989e53cd995e6381a4c` | `3535b3175635f9984c82a3f0c2a8e70f0ed2cfdb2a352f40446c9fddf2490703` |
| Windows x64 and ARM64 | 157.0.8103.15 / `1aa07b638799e59656559ae6c6649d4b197f555e` | `c6e5bf72ba196a184fd9753f699530b4446a9af980b15818105d79a302158d11` |

Sources: [Mac Stable metadata](https://chromiumdash.appspot.com/fetch_releases?channel=Stable&platform=Mac&num=1),
[Windows Stable metadata](https://chromiumdash.appspot.com/fetch_releases?channel=Stable&platform=Windows&num=1),
[Mac candidate DEPS](https://github.com/chromium/chromium/blob/d0c02bfad08ec92a6ffcf989e53cd995e6381a4c/DEPS),
[Windows candidate DEPS](https://github.com/chromium/chromium/blob/1aa07b638799e59656559ae6c6649d4b197f555e/DEPS).
The different Chromium majors make Playwright 1.64.1 compatibility a real check;
its [explicit-executable option](https://playwright.dev/docs/api/class-browsertype#browser-type-launch-option-executable-path)
does not guarantee arbitrary Chromium revisions. Do not claim compatibility from
shared CDP ancestry or downgrade the Windows candidate merely to align versions.

The source route supports the required CPU targets. Chromium documents
[macOS ARM64 builds](https://github.com/chromium/chromium/blob/d0c02bfad08ec92a6ffcf989e53cd995e6381a4c/docs/mac_arm64.md)
and [Windows builds including ARM64 tools](https://github.com/chromium/chromium/blob/d0c02bfad08ec92a6ffcf989e53cd995e6381a4c/docs/windows_build_instructions.md).
The latter requires a substantial Windows build environment: at least 100 GB
free disk and recommends more than 16 GB RAM. Those are upstream build-machine
requirements, not Mirket's measured runtime requirements. This Mac session does
not establish Windows build or execution success.

### Build configuration to qualify

Use a separate source/build workspace and pinned private tool paths. Review the
selected DEPS, hooks, GN actions and toolchain acquisition before invocation;
do not execute upstream quick-start installers or automatic hooks incidentally.
Freeze all fetched Git revisions and CIPD/package instances, not only top-level
DEPS bytes. Preserve the build scripts and the exact configuration used.

The following is a **proposed GN configuration for validation**, not a tested
build script. Select `target_os` and `target_cpu` per target; the non-CPU options
are supported by the inspected Mac candidate source and must also be checked
against the Windows candidate before use.

```gn
is_debug = false
is_official_build = false
is_component_build = false
generate_about_credits = true
is_chrome_branded = false
is_chrome_for_testing_branded = false
enable_widevine = false
bundle_widevine_cdm = false
proprietary_codecs = false
ffmpeg_branding = "Chromium"
is_component_ffmpeg = true
symbol_level = 0
# target_os = "mac" or "win"
# target_cpu = "arm64" or "x64"
```

This uses the complete browser for headless and visible sessions. It does not
select a second headless shell, ChromeDriver, an updater or a runtime downloader.
The pinned [credits build rule](https://github.com/chromium/chromium/blob/d0c02bfad08ec92a6ffcf989e53cd995e6381a4c/components/resources/BUILD.gn)
sets `generate_about_credits` from `is_official_build` by default, invokes the
notice generator when true, and copies `sample_credits.html` when false. That
explains the observed snapshot defect and supplies a concrete repair.

[Widevine's build configuration](https://github.com/chromium/chromium/blob/d0c02bfad08ec92a6ffcf989e53cd995e6381a4c/third_party/widevine/cdm/widevine.gni)
enables and bundles the CDM for branded Chrome/CfT. The proposed unbranded flags
avoid that component; verify the final runtime inventory rather than assuming
flags establish the binary contents. Disabling proprietary codecs also means
qualifying the resulting media behavior; it does not itself prove a complete
license or patent clearance.

The Mac candidate pins Chromium FFmpeg revision
`9db86ce5b5b454dfc96c435f9f0723012980c37f`. Its
[options](https://chromium.googlesource.com/chromium/third_party/ffmpeg/+/9db86ce5b5b454dfc96c435f9f0723012980c37f/ffmpeg_options.gni)
explicitly support an independently shared FFmpeg library via
`is_component_ffmpeg`; its [build rule](https://chromium.googlesource.com/chromium/third_party/ffmpeg/+/9db86ce5b5b454dfc96c435f9f0723012980c37f/BUILD.gn)
otherwise uses a static library in non-component builds. A shared boundary is a
practical candidate for LGPL library replacement, but the actual loading,
signing, ABI and replacement journey must work. Provide the exact FFmpeg source,
configuration and compilation scripts, its
[LGPL-2.1 text](https://chromium.googlesource.com/chromium/third_party/ffmpeg/+/9db86ce5b5b454dfc96c435f9f0723012980c37f/COPYING.LGPLv2.1),
and the complete required materials for the chosen distribution mechanism.
Repeat the mapping for the Windows candidate's own DEPS; do not reuse the Mac
FFmpeg pin by assumption.

### Required release evidence

1. Freeze the resolved source tree and the actual build recipe for each target,
   including patches, GN arguments and toolchain identities. Build from those
   inputs; keep generated notices, license metadata and build-input receipts.
2. Inventory the produced runtime and map every shipped component to its actual
   license/notice/source obligations. Keep Chromium BSD notices and the complete
   relevant third-party notices. Generated credits are evidence, not an exemption
   from reciprocal-source obligations or an authoritative audit of copied SDK files.
3. Assemble the corresponding-source materials required by each component with
   accessible recipient instructions. Prefer a durable same-distribution source
   artifact over a promise to reconstruct source later. Measure those artifacts;
   the assembler's current 16 GiB source bound has not been qualified for a full
   Chromium source export and must not be silently raised or bypassed.
4. Verify a recipient can replace the LGPL library or use the applicable relink
   materials. Preserve the required modification/reverse-engineering rights in
   any distribution terms. Do not use signing/update rules as an undisclosed
   prohibition on the recipient's permitted modified copy.
5. Run the actual private-path Playwright workflow on macOS 15/current and both
   Windows targets: headless/visible navigation, extraction, interaction/capture,
   saved profiles, cancellation/process cleanup and operation failure recovery.
   Qualify the source/notice delivery and library replacement separately from
   ordinary browser execution. Keep the existing gate blocked until reviewed.

The build-input records support review of the selected binary and its required
source materials. They do not by themselves establish the legal adequacy of a
source delivery mechanism. A controlled build is recommended because it makes
Mirket's actual inputs and configuration inspectable.

## Windows-specific closure

Native ARM64 Chromium with an explicit executable remains a separate build and
runtime qualification target. An upstream download route or a successful macOS
launch does not establish native Windows execution. Keep any individually
qualified x64-helper exception distinct from native ARM64 evidence.

A source build can also copy vendor redistributables. The Mac candidate's DEPS
pins ANGLE `3b89d84e9d59f75675ac4e2bef58fc068a9f0577`; that exact
[ANGLE build](https://github.com/google/angle/blob/3b89d84e9d59f75675ac4e2bef58fc068a9f0577/BUILD.gn)
copies `d3dcompiler_47.dll` from the Windows SDK. Chromium's
[CRT configuration](https://github.com/chromium/chromium/blob/d0c02bfad08ec92a6ffcf989e53cd995e6381a4c/build/config/win/BUILD.gn)
selects static or dynamic Microsoft runtime linkage, and its
[toolchain helper](https://github.com/chromium/chromium/blob/d0c02bfad08ec92a6ffcf989e53cd995e6381a4c/build/vs_toolchain.py)
can copy runtime DLLs. The candidate Windows revision requires its own inspection.

Consequently, the Windows build experiment must identify copied **and linked**
vendor material. An SDK build prerequisite does not authorize bundling its
runtime files under an OSS label. Prove that any omitted application-local DLL
is supplied by the supported OS through safe system resolution, or establish a
separate permitted distribution-policy/license treatment before including it.
No such omission or treatment was qualified here. This remains a concrete
Windows blocker even after Chromium credits are repaired.

## Practical next action and stopping condition

Prepare one reviewed Chromium source-acquisition/build-input receipt for the Mac
candidate in a separate workspace: resolve DEPS/toolchain pins without running
unreviewed hooks, inventory required source/license material, validate the GN
proposal and measure the source export. Then run a controlled macOS ARM64 build
with generated credits and the shared FFmpeg boundary. Its browser/notice/source
artifacts must be reviewed and tested before replacing any blocked candidate.
In parallel planning, secure a Windows x64 build environment and native x64/ARM64
verification machines, and resolve vendor-runtime distribution before promising
Windows admission. This proposal creates no installation or build job.

The remaining first-release blockers are the actual source builds and complete
required materials, legal/OSS treatment of Windows vendor runtime code,
Playwright compatibility of the selected revisions, signing/replacement behavior,
and all required native Windows/Codex journeys. No browser admission gate was relaxed.
