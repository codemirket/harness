# Remotion comparator admission and rights inventory

Status: `cancelled_by_user_hyperframes_only` (2026-10-27).

The user dropped Remotion from the plan. All comparator, host-installation and
replacement proposals below are superseded and retained only as historical
research. Do not execute their proposed steps. HyperFrames is the sole selected
engine; the [implementation plan](../planning/implementation-plan.md) governs
further work. No Remotion bundle or render was performed in this evaluation.

Bundled-shipping status: `excluded_pending_explicit_redistribution_grant`.
The published package license grants eligible users permission to use the
software for creating videos/images and to modify it for custom use; it does
not explicitly grant redistribution of the unmodified package or the
`@remotion/compositor-darwin-arm64` binary/dylibs as part of another tool. No
such grant was established in the inspected package archives, license page, or
FAQ. Consequently Mirket must not ship this closure or these native binaries.
An alternative candidate in which the user acquires the exact packages into a
user-owned project is described below; that candidate is not admitted and does
not eliminate license obligations.

Private effects already performed before the stop: the exact lock was resolved;
`npm ci --ignore-scripts --no-audit --no-fund --fund=false` installed the
locked comparator closure in private task state; direct tarballs were read as
metadata; and a private staging directory was populated with the compositor's
`remotion`/dylibs plus the previously qualified private `ffmpeg`/`ffprobe`.
No upstream install hook, browser launch, Remotion bundle, render, or model call
was executed. The private files are evidence only and are not release inputs.

This document records a bounded, disposable comparison project. It is not a
production admission, does not change the Mirket release or live installation,
and keeps downloaded dependencies in private task state outside this repository.
The comparison target is
Remotion 4.0.539 at the immutable tag target
[`e50b41a410933aa807515335d1c04dbba1b8b93f`](https://github.com/remotion-dev/remotion/tree/e50b41a410933aa807515335d1c04dbba1b8b93f),
verified from the GitHub ref API for `v4.0.539`.
The qualification platform is macOS ARM64 only. Windows remains unqualified.

## Scope and product boundary

The private comparator may exercise the existing explicit Chrome-for-Testing
and FFmpeg bindings already qualified for Mirket, but only after the exact
Remotion API contract is probed. It may not download another browser, select a
PATH executable, or modify the HyperFrames runtime. Codex's general browser is
permitted for a project-owned preview; Mirket's private Chrome remains
render-only. The comparator is a project-owned specimen used to compare source
preview and rendering behavior; it is not a Mirket tool and is not a release
artifact.

If production adoption is later approved, Remotion is a **sole replacement**
for HyperFrames. That would require removing HyperFrames direct and transitive
dependencies, routes, payloads, production skills and runtime assumptions, and
migrating HTML/CSS/JS sources to React/TypeScript compositions. Existing
rendered artifacts may be retained as artifacts, but there will be no dual
engine, hidden fallback, or compatibility layer. One explicit browser and one
explicit FFmpeg binding would be requalified. The comparator's success is not
approval for that migration.

## Exact comparator dependency inventory

The private project is
`/Users/nazmiyilmaz/.codex/task-state/harness-motion-feedback-20261027/remotion-admission/lock-minimal`.
Its `package.json` has the following direct dependencies:

| Package | Version | Registry tarball | Integrity (npm lock) | Declared license | Use |
|---|---:|---|---|---|---|
| `@remotion/bundler` | `4.0.539` | [`bundler-4.0.539.tgz`](https://registry.npmjs.org/@remotion/bundler/-/bundler-4.0.539.tgz) | `sha512-TK+O9Xd/mdPQuXvXaxND8QLW5QyAVLkD38CuB1UKVpVJEEMBxVVIP6iqdCoVm3fvwlys+KKWyXzhd8ZiyV8CIA==` | `SEE LICENSE IN LICENSE.md` | Bundle a React composition |
| `@remotion/player` | `4.0.539` | [`player-4.0.539.tgz`](https://registry.npmjs.org/@remotion/player/-/player-4.0.539.tgz) | `sha512-ww8vwLgLfXD7FzOmrzO8uLMnTQWWv9LIiSmm4zQ2GRkc/9/1k6Xe708dhjMAJYo5DrQF+LHCAbh5paUuJPWGXg==` | `SEE LICENSE IN LICENSE.md` | In-browser preview/seek |
| `@remotion/renderer` | `4.0.539` | [`renderer-4.0.539.tgz`](https://registry.npmjs.org/@remotion/renderer/-/renderer-4.0.539.tgz) | `sha512-vMRQm88TiMzwmUlsTgc3g1GYo2U/HznWbrFoo8rhvU3wnwTr5eHRSp3SzLxn48WISn7QyE1Odh8IB7yPy/MhtQ==` | `SEE LICENSE IN LICENSE.md` | Deterministic frame/media render |
| `react` | `19.1.1` | [`react-19.1.1.tgz`](https://registry.npmjs.org/react/-/react-19.1.1.tgz) | `sha512-w8nqGImo45dmMIfljjMwOGtbmC/mk4CMYhWIicdSflH91J9TyCyczcPFXJzrZ/ZXcgGRFeP6BU0BEJTw6tZdfQ==` | MIT | Composition runtime |
| `react-dom` | `19.1.1` | [`react-dom-19.1.1.tgz`](https://registry.npmjs.org/react-dom/-/react-dom-19.1.1.tgz) | `sha512-Dlq/5LAZgF0Gaz6yiqZCf6VCcZs1ghsu84Q/GT0gV+mCxbfmKNoGRKBYMJ8IEdGPqu49YWXD02GCknEDkw==` | MIT | Browser/player runtime |
| `remotion` | `4.0.539` | [`remotion-4.0.539.tgz`](https://registry.npmjs.org/remotion/-/remotion-4.0.539.tgz) | `sha512-CC97/XczqcJDJq0za0lU/kc7XnOocs8L9ADHFPWPMEXLm2aOxzdy3E0qnNNwqcW6q98JoKovtxc6YHQGZsZY/Q==` | `SEE LICENSE IN LICENSE.md` | Core frame/composition API |

The exact resolved transitive closure is the lockfile's `packages` map (not a
handwritten approximation): lockfile version 3, 227 package entries including
the root, 27 `@remotion/*` entries, SHA-256
`86c251908624a11af6e8a58b08b52a5bf2b8ee4894d2cb8679c4e491dbb6d62a`, and
`package.json` SHA-256
`1081922de757eac7aab07ce7785c3f712e05aab25c048fcc18cb5aef14de8386`.
The lock is retained outside the repository at the path above so it can be
reviewed without shipping upstream packages. It includes all optional
platform compositor packages and pulls Studio/studio-shared through the
bundler; this is not a claim that the closure is small.

The lock was resolved with:

```text
npm install --package-lock-only --ignore-scripts --no-audit --no-fund --fund=false
```

It completed without creating `node_modules` and without executing package
scripts. Registry metadata and integrity values were read from the official npm
registry. This is a reproducible resolution record, not proof that every
archive's notice file or source license has been inspected.

As a separate archive-level check, the six direct tarballs were downloaded to
the private comparator state directory and read as tar metadata only (no module
was installed or executed). The relevant observations are:

| Archive | SHA-256 | Archive evidence |
|---|---|---|
| `@remotion/bundler` | `997ac67144bf33b0eaf2d51ea8d62052d54ddd3b0df16b39f62fcc436ac5178c` | `package/LICENSE.md`; development-only formatting/lint/test/make scripts |
| `@remotion/player` | `7de7e733dc8445ce5b30a47a87216c901bd876fc8dfde394cfda40d310244fde` | `package/LICENSE.md`; no install hook; development scripts and `prerelease` only |
| `@remotion/renderer` | `58aff67a9d69a739c60d13a0517eeb76d162a2b6e89ffeef1f6eb520e8cb4013` | `package/LICENSE.md`; development-only formatting/lint/test/make scripts |
| `remotion` | `be57b25afee19853461362048ad94ba7350010cedfe472e0b5fc39ee889b81d5` | `package/LICENSE.md`, `package/THIRD_PARTY_LICENSES.md`; `prepublishOnly`, not install-time |
| `react` | `754c21ed5078d3d211a443c9e4137031ecd72e1725d0ef3d235443c294eea2a5` | `package/LICENSE` (MIT); no package scripts |
| `react-dom` | `d72ebd28017f0ccfee96e4c3d9b2f69a0ab8109edfd51f17316a1fd68bd0bdd8` | `package/LICENSE` (MIT); no package scripts |

The platform compositor archive required by this closure was also inspected:
`@remotion/compositor-darwin-arm64@4.0.539` contains `remotion`, `ffmpeg`,
`ffprobe`, and native dylibs, has no `LICENSE` or `NOTICE` file and declares no
license in `package.json`; its README calls it an internal package. This is an
unresolved provenance/rights item, not evidence that it is prohibited. The
private archive metadata is retained at
`.../remotion-admission/archive-metadata/` outside the repository.

Notable transitive obligations and open inventory items are:

* `@mediabunny/*` and `mediabunny` are declared MPL-2.0; `caniuse-lite` is
  CC-BY-4.0 and needs attribution if distributed.
* `@remotion/sdk` and `@remotion/studio-protocol` are custom Remotion-license
  packages. Several compositor/media/web-renderer entries have no useful
  license field in the lock, so their package archives' LICENSE/NOTICE files
  must be inspected before any release admission.
* The closure includes optional compositor packages for multiple host
  platforms. Their presence in a lock does not qualify those platforms or grant
  permission to redistribute them.
* The existing private browser and FFmpeg records remain separate obligations:
  Chrome-for-Testing 156.0.8078.47 Mac ARM64 and the existing FFmpeg/ffprobe
  9.0.2 bindings are the only candidate executables. Their existing admission
  and notice gates still apply.

The CLI and Studio are intentionally not direct dependencies. The bundler
already brings Studio-related packages into the closure; adding the CLI would
expand the comparator without testing a required Mirket boundary.

## Rights and provenance gate

Remotion packages carry a custom Remotion License rather than a conventional
permissive open-source license. The exact text observed in the package archives
is also published in the upstream repository's
[`packages/core/LICENSE.md`](https://github.com/remotion-dev/remotion/blob/e50b41a410933aa807515335d1c04dbba1b8b93f/packages/core/LICENSE.md).
It distinguishes individual use, use by up to three employees, nonprofit use,
and evaluation from larger for-profit use. The private, non-commercial
comparator may fit the stated evaluation permission, subject to keeping it an
evaluation and not using it commercially. However, that use permission is not
an explicit redistribution grant for an unmodified package or native
compositor binary. The text explicitly disallows copying or modifying Remotion
code for the purpose of selling, renting, licensing, relicensing, or
sublicensing a derivative; this document makes no broader claim about that
sentence. Bundled production adoption is therefore excluded unless a grant
covering the exact package closure and native compositor is established.
The separately installed runtime candidate below does not depend on Mirket
redistributing those packages.

The upstream Remotion skills repository was inspected for a license file at its
current raw `LICENSE` and `LICENSE.md` locations; neither was available. No
redistribution grant was established for its instruction prose. A comparator
may use the public method links as research references, but no upstream skill
body, prompt, or copied text is shipped. Any production skill would require an
explicit rights grant or independent re-authoring from observed techniques.

The custom package licenses, packages with missing registry license metadata,
MPL-2.0/CC-BY obligations, Chrome terms, FFmpeg distribution/codec obligations,
and any local font/asset notices are separate gates. A lockfile hash is not a
license approval.

## Candidate: user-owned upstream acquisition (not admitted)

Remotion's current [brownfield installation guidance](https://www.remotion.dev/docs/brownfield)
explicitly documents adding exact-version `remotion` and `@remotion/*`
packages to an existing project, and separately adding `@remotion/player` for
embedding or `@remotion/renderer` for Node.js rendering. That supports a narrow
alternative architecture: Mirket could keep no Remotion npm tarballs, native
compositor binaries, or copied Remotion code in its release, and a user could
acquire the exact lock into a user-owned project from the upstream registry.
Mirket would record and verify the selected lock, package provenance and native
binary identities before invoking explicit paths. The recipient remains
responsible for holding the applicable license; a successful installation or
render does not prove that eligibility.

This is not a license workaround. The official [contributing guidance](https://www.remotion.dev/docs/contributing)
says the project cannot be redistributed. For the pinned 4.0.539 release, the
[version-4 Terms and Conditions](https://www.remotion.pro/terms-4-0) say that
the software's specific license prevails over the generic terms: users receive
copy/distribution rights only to the extent that license permits, and the terms
also prohibit reproducing, selling, reselling, or exploiting Remotion or its
service without express prior written permission. The newer
[`remotion.dev/docs/terms`](https://www.remotion.dev/docs/terms) page is marked
upcoming for Remotion 5.0 and is not applied to this 4.0.539 candidate.
Direct upstream acquisition may avoid Mirket redistributing the closure, but it
does not grant Mirket or its user permission to use Remotion without the
applicable Free/Company terms. The candidate is an independently authored
integration operating the recipient's local project and runtime, with no sale
or sublicense of Remotion itself. This is an architectural inference from the
published use permission, not an express upstream approval of Mirket. It does
not authorize a hosted rendering service or distribution of Remotion code to
other users. The compositor archive's missing license metadata and third-party
codec obligations remain separate inventory findings; missing metadata alone
does not establish that the binary is unlicensed for its upstream intended use.

The candidate remains unqualified until all of the following are established:

* the recipient acquires the exact package closure directly from upstream,
  either separately or through an explicit setup step after exact admission;
  Mirket ships no tarballs, native binaries, package caches, container layers,
  or generated bundles carrying Remotion code, and provides no mirror, hidden
  install or fallback;
* the user's applicable 4.0 license covers the selected use, including any
  automation, Player preview, render/service, and commercial restrictions; the
  current FAQ's later-version billing language must not be silently substituted
  for the 4.0 terms;
* the user-owned lock resolves the same version set and all native packages
  have a recorded provenance/notice result;
* one explicit browser/FFmpeg/compositor binding is identity-checked and
  cancellation/update/rollback behavior is qualified; and
* the product migration still satisfies the sole-replacement rule: direct
  user acquisition is not a second HyperFrames route, fallback, or compatibility
  layer.

No user installation, license purchase, package refresh, or runtime execution
was performed for this candidate. The official docs establish that upstream
installation is a supported project workflow, not that Mirket's proposed
integration is admitted.

## Disposable comparator design

The proposed specimen is a small React/TypeScript composition with explicit
input props, a fixed 640x360, 12 fps, 36-frame profile, local selected font,
and captions. It should use `@remotion/bundler.bundle()` and
`@remotion/renderer.selectComposition()`/`renderMedia()` against the same
source. `@remotion/player` should preview the bundled composition and exercise
frame seeking, using Codex's general browser for the project-owned preview when
useful. The CLI and Studio are not part of this bounded comparator.

The official [`binariesDirectory` option](https://www.remotion.dev/docs/options/binaries-directory)
says the directory contains `ffmpeg`, `ffprobe`, a Rust binary, and shared
libraries; when unset, Remotion selects the platform package under
`node_modules/@remotion/compositor-*`. The official [`browserExecutable` option](https://www.remotion.dev/docs/options/browser-executable)
supports an absolute browser path but otherwise may download a browser. The
old separate `ffmpegExecutable`/`ffprobeExecutable` options were removed in v4.
Therefore a user-owned installation still needs one identity-checked
`binariesDirectory` containing the complete runtime. The existing private
FFmpeg directory contains only `ffmpeg` and `ffprobe`, while the Remotion macOS
compositor package supplies a separate `remotion` binary and libraries. Until
that complete arrangement is proven, the candidate must not claim that the
existing FFmpeg binding is compatible or silently fall back to bundled
binaries. If no single-explicit-directory arrangement is accepted, this is an
integration blocker and not a reason to download another runtime.

The table below defines the technical comparator's required evidence. Private
evaluation uses the evaluation permission; a successful comparison would still
not qualify production licensing or packaging. No such run was completed:

| Check | Observable evidence | Failure is a blocker |
|---|---|---|
| Bundle and source contract | Bundle succeeds from a fresh disposable project; props and input paths are explicit | Yes |
| Preview/seek | Saved preview pixels at selected frames, including a seek after a rewind | Yes |
| Render parity | Rendered PNG/frame samples and decoded media from the same props/source | Yes |
| Readiness | Local font and captions finish deterministically; no remote fetch | Yes |
| Existing runtime binding | Explicit qualified Chrome/FFmpeg paths are accepted and identity checked | Yes; do not download a replacement |
| Stale/mismatched input | Changed source/props are refused or explicitly re-bundled; no silent stale result | Yes |
| Cancellation/concurrency | One cancellation and bounded concurrent render are observed without leaked jobs | Yes |
| Independent comparison | HyperFrames fixture and Remotion specimen judged by an independent pixel/media/code oracle | Yes |

The oracle must inspect saved pixels and decode output metadata; a successful
render, file hash, package receipt, or model self-report is not visual or media
verification. Record platform, Node, browser, FFmpeg, frame/fps/dimension,
resource-use, and failure evidence. No comparator run has yet qualified these
checks, so the table is a gate, not a completion claim.

## Production replacement gate

No production dependency or release admission follows from this comparator.
Before a replacement proposal could be approved, it would need:

1. Rights confirmation for the exact Remotion closure, every missing
   LICENSE/NOTICE item, local assets/fonts, Chrome, and FFmpeg.
2. A source migration plan from HyperFrames HTML/CSS/JS to React/TypeScript,
   with no compatibility adapter and no dual engine; removal of HyperFrames
   routes, dependencies, payloads, skills, and runtime assumptions.
3. Requalification of one explicit browser and one explicit FFmpeg binding,
   including cancellation, update, rollback, uninstall, and resource cleanup.
4. Independent preview/render parity, caption, font, audio, dimensions, fps,
   frame-count, and saved-pixel evidence on the supported macOS target.
5. A separately designed Windows milestone. Local macOS source/protocol tests
   do not qualify Windows, and the Codex Desktop GUI check remains waived and
   untested for this release.
6. Locked build/test evidence with disposable homes/projects and a complete
   provenance/notice record. No upstream install hooks may run incidentally.

Concrete unresolved prerequisites are the custom-license eligibility decision,
archive-level notices for the null-license packages, compatibility of the
existing Chrome/FFmpeg bindings with this exact Remotion release, the cost and
behavior of source migration, and production qualification of React/TypeScript
authoring. These are reasons to keep the comparator private, not reasons to
silently widen the release.

## Primary technical sources

The immutable source and release notes reviewed for this inventory are:

* [Remotion v4.0.539 release](https://github.com/remotion-dev/remotion/releases/tag/v4.0.539)
  and [immutable source tree](https://github.com/remotion-dev/remotion/tree/e50b41a410933aa807515335d1c04dbba1b8b93f).
* [Published Remotion package license](https://github.com/remotion-dev/remotion/blob/e50b41a410933aa807515335d1c04dbba1b8b93f/packages/core/LICENSE.md)
  and [version-4 Terms and Conditions](https://www.remotion.pro/terms-4-0).
  These establish the package-specific use/evaluation terms and say that any
  copy/distribution right is limited by that license; they do not establish an
  explicit grant to redistribute the unmodified package and native compositor
  as part of Mirket. The current [licensing FAQ](https://www.remotion.pro/faq)
  is retained as a pointer only; its later-version billing language is not
  silently applied to this 4.0.539 candidate.
* [useCurrentFrame](https://raw.githubusercontent.com/remotion-dev/remotion/e50b41a410933aa807515335d1c04dbba1b8b93f/packages/core/src/use-current-frame.ts),
  [Sequence](https://raw.githubusercontent.com/remotion-dev/remotion/e50b41a410933aa807515335d1c04dbba1b8b93f/packages/core/src/Sequence.tsx),
  [Series](https://raw.githubusercontent.com/remotion-dev/remotion/e50b41a410933aa807515335d1c04dbba1b8b93f/packages/core/src/series/index.tsx),
  [interpolate](https://raw.githubusercontent.com/remotion-dev/remotion/e50b41a410933aa807515335d1c04dbba1b8b93f/packages/core/src/interpolate.ts),
  [spring](https://raw.githubusercontent.com/remotion-dev/remotion/e50b41a410933aa807515335d1c04dbba1b8b93f/packages/core/src/spring/index.ts),
  and [delayRender](https://raw.githubusercontent.com/remotion-dev/remotion/e50b41a410933aa807515335d1c04dbba1b8b93f/packages/core/src/delay-render.ts).
* [Renderer frame seeking](https://raw.githubusercontent.com/remotion-dev/remotion/e50b41a410933aa807515335d1c04dbba1b8b93f/packages/renderer/src/seek-to-frame.ts),
  [frame rendering](https://raw.githubusercontent.com/remotion-dev/remotion/e50b41a410933aa807515335d1c04dbba1b8b93f/packages/renderer/src/render-frames.ts),
  and [composition selection](https://raw.githubusercontent.com/remotion-dev/remotion/e50b41a410933aa807515335d1c04dbba1b8b93f/packages/renderer/src/select-composition.ts).
* [Binaries directory](https://www.remotion.dev/docs/options/binaries-directory)
  and [browser executable](https://www.remotion.dev/docs/options/browser-executable)
  option docs, including the v4 removal of separate FFmpeg executable flags.
* [Parameterized rendering](https://www.remotion.dev/docs/parameterized-rendering),
  [fonts](https://www.remotion.dev/docs/fonts), [captions](https://www.remotion.dev/docs/captions),
  [flicker troubleshooting](https://www.remotion.dev/docs/flicker), and
  [rendering](https://www.remotion.dev/docs/renderer).
* [Remotion best-practices skill source](https://raw.githubusercontent.com/remotion-dev/remotion/e50b41a410933aa807515335d1c04dbba1b8b93f/packages/skills/skills/remotion-best-practices/SKILL.md),
  used as a research reference only; its prose is not copied into shipped
  Mirket skills.

The detailed dated source ledger and implementation notes are retained privately
at `/Users/nazmiyilmaz/.codex/task-state/harness-remotion-research-20261027/remotion.md`.
