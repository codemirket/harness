# TypeScript native 7.0.2 release admission

Review date: **2026-10-25**. Scope: the exact three native packages inspected in
[the navigation qualification](m3-typescript-navigation-qualification.md), with
M0b dependency qualification and M3 semantic navigation as the admission boundary.
This review changes no adapter, Rust owner, lock, release inventory or installed
payload. No Go toolchain, upstream installer, hook or source build was executed.

## Decision

**Keep these native prebuilts outside the production release.** Their publisher
and source-tree identities are substantially better established, and the broad
Go advisory matches are narrowed below. The unexplained dirty source state still
prevents treating a tagged-source review as proof of these binaries' complete
execution paths. The notice closure needs supplementation, and the proposed
macOS floor still needs runtime qualification.

This is a bounded static admission investigation. It neither reverses the native
engine's earlier finite semantic results nor promotes those results into installed
MCP, Windows or minimum-OS evidence. Native TypeScript remains a candidate, not an
admitted dependency.

## Exact bytes and publisher identity

All rows are version **7.0.2**. The retained npm tarballs' SHA-512 integrity was
rechecked. Their registry ECDSA signatures verified against the matching key
returned by the official [npm registry keys endpoint](https://registry.npmjs.org/-/npm/v1/keys).
The message verified was `name@version:dist.integrity`; the key ID was
`SHA256:DhQ8wR5APBvFHLF/+Tc+AYvPOdTpcIDqOhxsBHRwC7U`, with no expiry in the
retrieved key record. Verification used the already explicit private Node
24.22.1, executable SHA-256
`6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`.

| Package suffix under `@typescript/typescript-` | Tarball SHA-256 | Executable SHA-256 |
| --- | --- | --- |
| `darwin-arm64` | `902e2fe1cf0799198ef902c6b8c310a450fef629a6baba41d45641ef75c04ebd` | `a82f731365ad69d5c4c15f5e18fba4584bf3b7b839960172a76c3462b5114bf2` |
| `win32-x64` | `61fc4e141d2bc687db580e71bbfa63b9c209f0310645d82ca1b457eb3a24fd19` | `f9ecfbdc93753d2c972d66a8d0d75f5bd737fd4a5f88b422d9091ea282bcb2c7` |
| `win32-arm64` | `0a73534e6ee50cdbb2a29ac48657ca0ad13cf0f424cf63808e4df7baeb87b8be` | `2ec832b3d17ce25676fe92b3810ecc0a7580ffb9287d4972bc9a5acc4da2aa50` |

The three tarball SHA-256s also equal the digests published in the
[official GitHub release API](https://api.github.com/repos/microsoft/TypeScript/releases/tags/v7.0.2).
The Mac executable's embedded signature identifies Microsoft Corporation,
Team ID `UBF8T346G9`; `codesign --verify --strict` passed on macOS 26.5.2 ARM64.
This is static signature verification, without a notarization/revocation or
Gatekeeper acceptance claim. Publisher signatures and matching release digests
authenticate artifact identity; they do not explain the dirty build state.

## Source correspondence: migration resolved, dirty state unresolved

The source mismatch is partly a repository migration:

- npm `gitHead` and every inspected executable identify original
  `microsoft/typescript-go` commit
  `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`.
- That [original commit](https://api.github.com/repos/microsoft/typescript-go/commits/2bd066d87f5bafd315be9f40889d0a60b9e58e0b)
  has source tree `ed2c2c12c401b84bd5888d0b889737495aa93a20`.
- Migrated TypeScript tag `v7.0.2`, commit
  `1e4744d68260a7cb91b62b12edc3f6a2187faaf1`, contains precisely that tree at
  [`tsc/`](https://api.github.com/repos/microsoft/TypeScript/git/trees/ed2c2c12c401b84bd5888d0b889737495aa93a20).
  All root path/mode/type/object-ID entries agree. The release page explicitly
  links its original `typescript/v7.0.2` release.

Consequently, inspecting `tsc/` at the migrated tag is inspecting the original
committed source tree, rather than an unrelated compiler revision. It still does
not identify uncommitted inputs: embedded build information says
`vcs.modified=true` and main module `v0.0.0-20260708042240-2bd066d87f5b+dirty`.

The exact original [Herebyfile](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/Herebyfile.mjs)
was read, not executed. Its `buildTsgo` selects `./cmd/tsgo`, `noembed`,
`-trimpath`, stripped release symbols and per-target `CGO_ENABLED=0`; its package
builder records `git rev-parse HEAD` and copies the notice/default-library files.
These correspond to observed build settings. They are not a build transcript or
a record of dirty files. The public CI fixture builds a development package; it
does not attest the exact production invocation. A query to npm's attestation
endpoint for the exact Mac package returned HTTP 404; no source-to-binary
attestation was obtained in this review.

**Required source gate:** obtain publisher evidence identifying the actual dirty
inputs and production build, or qualify a new clean-source artifact with an
explicitly reviewed build. Do not assume the dirty change was only a version
stamp, generated notice or package metadata. Rebuilding or changing Go/x/text
creates a new payload identity and requires the semantic/passivity corpus again.

## Compiled Go advisories and actual linked code

The earlier corrected `stdlib@1.26.4` query returned 24 records; the exact module
query also found `x/text@0.38.0` records GO-2026-5970 and GO-2026-6629. This review
uses that same-day retained snapshot and the Go project's individual advisory
records. It does not reinterpret the earlier invalid `go1.26.4` version query.

`inspect_native.py` reads each executable as data. It validates the Go 1.20+
`pclntab` header and every function/name offset, then reads both outlined
functions and the complete function-name table. The official
[Go 1.26.4 linker](https://github.com/golang/go/blob/go1.26.4/src/cmd/link/internal/ld/pcln.go)
includes inlined function names in that table. Counts were:

| Target | Outlined function records | Function-name entries, including inline names |
| --- | ---: | ---: |
| macOS ARM64 | 23,307 | 30,152 |
| Windows x64 | 23,040 | 29,682 |
| Windows ARM64 | 22,934 | 29,554 |

Affected package/symbol names were matched after normalizing pointer receivers
and Go's `vendor/` prefix. The machine-readable report retains every advisory,
import path, platform eligibility and actual matching name.

| Advisory slice | Result in all three exact binaries | Admission implication |
| --- | --- | --- |
| GO-2026-5026, 5856, 5972, 6088, 6089, 6090, 6091, 6599, 6600, 6603, 6605, 6607, 6608, 6609, 6610, 6611, 6612, 6613, 6617, 6809 | No function-name entries for the affected HTTP, TLS, X.509, ASN.1, XML, HTML-template, MIME/textproto or related packages. | These 20 version matches have no corresponding affected linked package in this static inventory. Network denial alone was not used to reach this conclusion. |
| [GO-2026-4970](https://pkg.go.dev/vuln/GO-2026-4970), [6604](https://pkg.go.dev/vuln/GO-2026-6604) | Ordinary `os` code is linked; affected `os.Root`/`OpenInRoot`/root helpers and Windows `Mkdirat` are absent. GO-2026-6604 is additionally Windows-only. | No affected-symbol match. Ordinary `os.rootOpenDir` is not the advisory's `os.Root` API. |
| [GO-2026-5942](https://pkg.go.dev/vuln/GO-2026-5942) | `net` and vendored DNS parsing are linked; the listed affected CNAME lookup/cgo resolver and SVCB/HTTPS parsing symbols are absent. | A DNS dependency is not sufficient to establish this defect's reachability. |
| [GO-2026-6218](https://pkg.go.dev/vuln/GO-2026-6218) | `net/url.Parse` and its parser helpers are linked; affected `URL.Parse`, `URL.ResolveReference` and `resolvePath` are absent. | Do not confuse package function `url.Parse` with affected method `(*URL).Parse`. |
| [GO-2026-6629](https://pkg.go.dev/vuln/GO-2026-6629) | No `golang.org/x/text/secure/precis` function-name entries. | No affected linked package in this static inventory. |
| [GO-2026-5970](https://pkg.go.dev/vuln/GO-2026-5970) | `norm.Form.String` and `norm.nextComposed` are present. | A real symbol match remains; do not clear the package based on npm's empty advisory result. |

For GO-2026-5970, a second, separate Mac ARM64 inspection decodes direct `BL`
instructions against the validated function boundaries. The only direct caller
of `norm.Form.String` is `lsutil.removeDiacritics`; its ancestors are natural or
Unicode organize-import collation. The exact
[`organizeimports.go`](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/ls/lsutil/organizeimports.go)
uses `norm.NFD.String` there. The reported defect concerns an iterator over
invalid UTF-8; neither `Iter.Init` nor `Iter.InitString` appears in the name tables.
The decoder found no direct caller of `nextComposed`, which can be kept through
function-pointer tables. This narrows the concern to code outside the proposed
`documentSymbol`, `definition` and `references` request surface.

That is a static exclusion argument for the fixed request boundary, **not a
runtime exploit test, a complete indirect-call graph or a Go vulnerability
scanner clearance**. Dirty-source uncertainty and unqualified Windows runtime
paths remain. Preserve a finite request allowlist and disabled acquisition;
adding code actions, organize-imports, completion, arbitrary preferences or
commands invalidates this narrow argument. A release must record an explicit
reviewed disposition for GO-2026-5970, or use a separately pinned remediated build.

## Rights closure: concrete missing material

Every original native `NOTICE.txt` is byte-identical to the committed source
notice, SHA-256
`f5c708b59114507b8b27b48181b6883d106bbca0c1634bbee45b5e344237b66b`.
Keep it and the complete Apache-2.0 `LICENSE` unchanged, with all 108 matching
default declarations. The declaration/data notices remain part of the selected
payload's rights closure.

The original notice lists the main Go modules, but the x/sync, x/sys, x/term and
x/text sections print `BSD-3-Clause AND LicenseRef-scancode-google-patent-license-golang`
in place of the full license and patent-grant text. Placeholder copyright lines
also occur in its generated license templates. Preserving this notice alone is
not a complete exact-byte license inventory.

The toolset is [Microsoft Go v1.26.4-1](https://github.com/microsoft/go/releases/tag/v1.26.4-1),
tag commit `620545238d8033720ee8512a8c3b82d4bd111d14`, whose Go submodule is
`a9ce111d580581fb925ae88f125c69b7d93504ea`. Its full Go BSD license and patent grant,
Microsoft fork license, and vendored-library notices must be accounted for in
addition to the application module build-info.

There is a concrete missing Windows dependency: both Windows binaries contain
`vendor/github.com/microsoft/go-crypto-winnative` CNG/bcrypt/sysdll functions.
This module is absent from the application's build-info dependency list and its
notice. The exact Microsoft Go
[vendor patch](https://github.com/microsoft/go/blob/620545238d8033720ee8512a8c3b82d4bd111d14/patches/0001-Vendor-external-dependencies.patch)
pins it to **v0.0.0-20260127024749-832b168a84e9** and carries its MIT license.
The patch license bytes equal the license at that upstream revision. Its exact
OSV query returned `{}`; that is an advisory-database result, not a crypto audit.
No such Microsoft crypto function entries occur in the Mac binary.

All three binaries also contain Go-vendored `x/net/dns/dnsmessage`, associated
with **v0.47.1-0.20260417193450-705de46f8788** by the exact Go submodule's
[`src/vendor/modules.txt`](https://github.com/golang/go/blob/a9ce111d580581fb925ae88f125c69b7d93504ea/src/vendor/modules.txt).
Its license/patent files were captured. Do not describe the main-module union as
a complete compiled SBOM.

A private **22,947-byte supplemental-notice proposal** now contains the full
exact-version Go/Microsoft Go, relevant Go x-module, go-json, xxh3, cpuid,
go-winio, go-osstat and Windows native-crypto license/patent text. Its manifest
records every source file, byte count and SHA-256. This provides a concrete
repair input; it was not copied into a production release. Complete the linked
source/data inventory, account for the unexplained dirty inputs, and verify the
assembled release's original plus supplemental notice bytes before admission.

## macOS floor

The exact Mac executable's `LC_BUILD_VERSION` declares platform 1, minimum
**12.0.0**, SDK **12.0.0**, and no build-tool records. Independent `otool -l`
and the parser agree. It links system libraries/frameworks only. The official
[Go 1.26 release notes](https://go.dev/doc/go1.26#darwin) also document that Go 1.26
runs on macOS 12. Both are below the plan's macOS 15 ARM64 floor.
There is therefore no observed metadata conflict requiring a higher floor.

**Runtime gate remains macOS 15 ARM64:** execute this exact package's semantic,
hostile-preparation, Unicode, lifecycle/cancellation and cleanup fixtures there,
then exercise the installed owner/MCP route. The earlier run on macOS 26.5.2
cannot supply that evidence. Keep Windows 11 25H2 x64/ARM64 as separate native
runtime/ownership gates.

## One simpler contingency

If publisher evidence or a controlled native rebuild cannot close the source
gate, use **official TypeScript 5.9.3 direct `tsserver.js` as the sole semantic
engine** on the already private Node 24.22.1. This was the existing qualified
alternative: it passed the same 17 finite groups, uses one analyzer process and
has no registry runtime dependencies. Its unchanged tarball is pinned at
SHA-256 `10e108c9cf7d5f2879053dff18515fb405abf2ccef63eaaf017d9c571687a1d3`.
Its default declarations and original Apache-2.0/third-party notices still belong
to the semantic payload; the previously proposed parser-only subset is not a
semantic-server package.

This removes a native Go build/provenance closure and the wrapper process from
that release, at the measured cost of higher RSS and a finite tsserver-protocol
owner. It is a replacement decision requiring exact admission and integration;
do not ship it behind a silent native fallback. Keep the existing fixed external
project/prepared-model, disabled acquisition, revision checks and plugin refusals.
Windows watcher-helper paths, native Node lifecycle, installed MCP behavior and
the platform floors still need qualification. No second production engine or
`typescript-language-server` wrapper is proposed.

## Exact remaining gates and evidence

1. **Native source:** explain the dirty inputs/actual build with publisher
   evidence, or review and qualify a new clean-source payload.
2. **Advisory disposition:** record the fixed-surface GO-2026-5970 review with
   its indirect-call/source limits, or remediate in a new exact payload. Retain
   all 26 static advisory rows and the toolset-vendored Windows component.
3. **Notices:** finish the compiled source/data closure and integrate/verify
   the original notices plus exact supplemental text in the assembled release.
4. **Runtime/owner:** qualify macOS 15, Windows x64/ARM64 and the finite launch,
   handshake, shutdown-status policy, prepared model/revision checks, tamper,
   cancellation and actual installed MCP contract from the original report.

Private evidence is under
`~/.codex/task-state/harness-rewrite-implementation/typescript-native-release-admission-20261025/`.
It reuses the prior exact tarballs/binaries without duplicating them. Key files
are `source-index.json`, `source-correspondence.json`,
`registry-signature-verification.json`, `binary-inspection.json`, all target
function-name lists, `darwin-arm64.direct-bl-calls.json`, the static command
captures, `notice-supplement-manifest.json`,
`third-party-supplement-proposal.txt`, and `evidence-index.json`. Network source
captures record their exact URLs, UTC retrieval times and SHA-256s, including
unsuccessful provenance/license-path queries. No new runtime semantic result is
claimed by this document. New private evidence is below 20 MiB, within the
500 MB investigation budget; no long build or production installation occurred.
