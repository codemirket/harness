# Video embedded-source reconciliation — 2026-10-26

**Source-only qualification; release admission remains false.** The selected
esbuild packages now have verified signed source provenance, with their exact
Go/x/sys and vendored xxhash inputs identified. Every inspected Chromium-BiDi
source-map input matches pinned source/package bytes and its bundled dependency
notices reconcile. No runtime, dependency selection or native artifact changed.

This advances the specific esbuild/BiDi source questions in the
[recipient proposal](m3-video-recipient-closure.md) and
[payload review](m3-video-payload-release-review.md). Their frozen packet,
scoped HyperFrames source and unpruned runtime inventory remain unchanged.
The additive supplement below preserves the new evidence; it does not resolve
the separate private Chromium, media, recipient-delivery or platform gates.

## esbuild publisher and declared native sources

Both `esbuild` and `@esbuild/darwin-arm64` **0.28.2** expose SLSA v1 and npm
publish attestations. Their original archive SHA-512 digests match the current
adapter lock and each attestation's subject. Retained compressed identities are:

| Published archive | SHA-256 |
| --- | --- |
| `esbuild` 0.28.2 | `e045f94c235c7adc50e77ba2a579c7bec41b496b6b47c3a7845f7c1e19959a88` |
| `@esbuild/darwin-arm64` 0.28.2 | `1980cde09749094452b20d36ff267585ccb3f72749c7bc97291cd9996ccf5a2a` |

All four bundles verified using the same existing development Sigstore/TUF
toolchain and private-cache method as the recipient proposal. The policy
requires GitHub's Actions OIDC issuer and the exact certificate identity
`https://github.com/evanw/esbuild/.github/workflows/publish.yml@refs/heads/main`,
with certificate-transparency and transparency-log thresholds of one.
Registry package signatures also verified. A changed DSSE payload and unrelated
expected signer identity were refused.

Both SLSA statements bind source commit
[`609683d892977362a0f99026cb74b96263d728a9`](https://github.com/evanw/esbuild/tree/609683d892977362a0f99026cb74b96263d728a9)
and [publish run 31275288675, attempt 1](https://github.com/evanw/esbuild/actions/runs/31275288675/attempts/1).
That commit's `go.version` selects **Go 1.26.5**. Its
[Makefile](https://github.com/evanw/esbuild/blob/609683d892977362a0f99026cb74b96263d728a9/Makefile)
builds a custom Go compiler after commenting out the linker's `ctxt.buildinfo()`
call. The recipe uses that compiler for platform releases, sets `CGO_ENABLED=0`
and retains stripped/trimmed build flags.

This explains the prior observation: the saved binary contains `go1.26.5` but
no ordinary Go buildinfo magic. The official unmodified Go source contains the
single call targeted by the pinned recipe. Absence of buildinfo is therefore
not evidence that Go is absent. The upstream rationale for suppressing metadata
does not waive Mirket's source or advisory assessment.

The complete official Go source archive matched the SHA-256 and size in Go's
download JSON. The first attempted `.sha256` URL returned an HTML documentation
redirect; that failed checksum-route assumption was retained and replaced by
the actual official metadata. No corrupt-source conclusion followed from it.

`go.mod`/`go.sum` also pin
`golang.org/x/sys v0.0.0-20220715151400-c0bba94af5f8`.
Its source ZIP contents independently matched the exact Go directory checksum
`h1:0A+M6Uqn+Eje4kHMK80dtF3JCXC4ykBgQG4Fe06QRhQ=`.
Saved binary function names include both `golang.org/x/sys/unix` and
`github.com/evanw/esbuild/internal/xxhash`; these are incorporated components,
not merely optional entries inferred from the lock. The pinned source's Darwin
filesystem/logger files import x/sys directly.

| Exact native-source input | Bytes | SHA-256 |
| --- | ---: | --- |
| Official `go1.26.5.src.tar.gz` | 34,140,216 | `495be4bc87176ac567392e5b4116abd98466d33d7b49d41e764ccc6976b2dc42` |
| Locked x/sys source ZIP | 1,798,284 | `3b180937216e93559f16b6076d09baf54a5707378f11b867b6eb914c56b09b91` |

The notice supplement preserves esbuild's MIT text; the separate xxhash MIT
copyright/permission/disclaimer; x/sys's original license/patent/author material;
and **37 unchanged Go root/source license, copyright and patent files**. The
Go collection is a source notice superset, including vendor and crypto
sublibraries. It does not claim that every optional component is linked.
The complete archives and exact build controls preserve the remaining context.

The signed package-to-source identity, declared compiler/dependency versions,
saved binary names and original notice inputs are now verified at that scope.
The custom compiler and esbuild executable were **not independently rebuilt**;
there is no new linked-object inventory, binary reproducibility claim or Windows
execution result. The original native executable remains unchanged.

## Chromium-BiDi source-map and notice closure

The selected `chromium-bidi` **157.0.8090-0** registry record has no `gitHead`
or build attestation. A source version was recovered through the exact
Chromium tag `157.0.8090.0`, resolving to commit
`2ea1b71af70d7fc5d41325dda8df37c1f98ac531`.
Its [BiDi subtree](https://chromium.googlesource.com/chromium/src/+/2ea1b71af70d7fc5d41325dda8df37c1f98ac531/third_party/chromium-bidi/)
was acquired without a browser or build. The **521,658-byte** archive is SHA-256
`19a54101b0a9d54277f31d08989fae47970835ae1b78f59ec2be4f824546426a`.

The pinned `README.chromium` declares a static hard fork, source version
17.0.2, base revision `d3bda5afa8c3fe9d3616bcce09ecaf38596d1ac9` and no
local modifications at that point. Fetching that base as a GitHub archive
returned 404. The actual immutable Chromium subtree is available and was used
for the comparisons; the unavailable base archive was not treated as verified
source.

All **99** package source maps contain `sourcesContent`. Across them, every
one of **199 unique inputs** matches its appropriate original bytes:

| Input kind | Count | Exact comparison |
| --- | ---: | --- |
| BiDi TypeScript | 98 | Pinned Chromium subtree `src/` files |
| Intermediate BiDi JavaScript | 82 | Same published package's generated JS files |
| Bundled Zod JavaScript | 18 | Exact Zod **4.6.5** published archive members |
| Bundled mitt JavaScript | 1 | Exact mitt **3.0.1** published archive member |

**Unresolved source-map inputs: none.** Repeated paths have consistent contents.
The pinned source and published package LICENSE files match byte for byte.
Their parsed `package.json` objects differ only in `version`: source 17.0.2
versus published 157.0.8090-0. This is consistent with the Chromium version
packaging boundary; it is not an authenticated publisher build attestation.

The source subtree's dependency lock matches both embedded dependency archives'
integrity values. Its README identifies mitt revision
`b240473b5707857ba2c6a8e6d707c28d1e39da49` and Zod revision
`59bbc03e10c636b9eb3c393dfeb552819774ec21` in its notices. The verified primary
bytes here are the exact published JS/package inputs, rather than an asserted
independent TypeScript rebuild from those dependency repositories.

| Embedded dependency archive | Bytes | SHA-256 |
| --- | ---: | --- |
| `zod` 4.6.5 | 1,052,733 | `a78c0c533de30dc1c4afc259ac43ac06e390cb0da8d2e32eae355301b50b36fc` |
| `mitt` 3.0.1 | 6,147 | `2ef69c669574ad5b86fdb8b30aed8c8c1705931f19a68bc4b1b234c6d26d7306` |

Each archive's full MIT license equals its Chromium `licenses/LICENSE.*`
counterpart and is present in full inside the shipped
`mapper_bundle_THIRD_PARTY_NOTICES`. The exact notice hashes are:

- mitt: `1cab22f196264195a4caec8ca5630170fdde76ee8f43346e47021d087332d3b0`;
- Zod: `3f1189b28e3866e0d979968d466b78f813f76827cfdca1fbb124cc0a5c8841f8`.

The separately installed Zod **4.6.7** remains a distinct runtime-lock instance.
No dependency pin was changed to match the older bundled code. Source-map
identity and complete original notices are now reconciled. The JavaScript
bundle was not independently rebuilt or executed, and this evidence does not
add signer provenance that the published package does not provide.

## wawoff2 provenance options remain bounded

The [recipient proposal](m3-video-recipient-closure.md#wawoff2-mapping-and-remaining-native-provenance)
already establishes all eleven published wrapper/binding files, exact embedded
WASM hashes, declared WOFF2/Emscripten inputs, historically corroborated Brotli
selection and preserved notices. This source-only follow-up cannot establish
the historical compiler image digest or an independent native build/link map.
The original bindings therefore stay unchanged and the provenance gate stays
visible.

The next-step brief retains three concrete routes without choosing a new
runtime or toolchain:

1. **Publisher evidence:** inspect a historical compiler image digest, actual
   source/link mapping and artifact-bound build provenance or reproducible
   build evidence. A digest fetched for today's mutable tag would not prove
   the old build. No upstream message or issue was sent.
2. **Controlled source candidate after the native lane is free:** first review
   exact local source extraction and compiler inputs, replacing mutable clone/
   checkout steps with preserved inputs. Build only the selected compression/
   decompression bindings; retain source/configuration/notices and new hashes.
   Exercise the actual HyperFrames/font API, Node compatibility, malformed and
   oversized inputs, resource behavior and absence of network. A new candidate
   needs its own explicit artifact selection before replacing current bytes.
3. **Retain the source-only limit:** preserve the current candidate and full
   notice inputs, report the missing native provenance, and keep admission
   closed. Do not silently remove font compression, alter its eager imports or
   introduce another renderer to avoid the unresolved contract.

These permissive components do not create a general corresponding-source offer
obligation. Their licenses still require the applicable notices, while the
project separately requires adequate artifact/source evidence. The existing
legacy Docker recipe was read, never executed; no compiler installation,
container startup or production dependency was selected by this review.

## Frozen additive recipient supplement

`video-embedded-source-notice-supplement-20261026.tar.gz` is **577,529 bytes**,
SHA-256
`1ed8d2a08c3f7146bff7db2df6cf645ad4bf1c085028b5da3de758388168e419`.
It contains **295 regular files, 3,123,852 uncompressed bytes**, including 289
unchanged copied inputs. It preserves source-map inputs and their comparison
index, source/build controls, notices, original esbuild attestation responses,
verified trust evidence and the bounded wawoff2 brief. It contains no newly
selected runtime.

Its source index binds **nine retained reference archives, 44,997,098 bytes**,
to proposed relative same-release paths. The index SHA-256 is
`c4fedf5eb4a3acfe2718126efcb9f6030d0e7f366604594a29e7203c74c13a08`;
the packet manifest is SHA-256
`ede08169af7527a9a0a92e45589339213697efb46ae5efe2b72dad84f08bc2fc`.
Those references are input identity and local assembly evidence, not actual
recipient access or an issued source offer.

Two independent serializations matched. A fresh unpack verified all packet
files and all nine referenced archive identities. Changed notice content,
unexpected files, a corrupt source archive and an altered source-map input
were refused. The prior recipient archive and scoped HyperFrames archive still
match their original frozen hashes. Exported content had no named builder-home
marker match under the bounded scan.

The separate delivery gates remain: choose the final exact native/runtime
components; carry their notices and required source/modification materials
through durable, verified recipient access; finish aggregate terms, signing and
current advisory checks. Private Chromium qualification, installed video/API
and resource behavior, macOS 15, **native Windows 11 x64 and ARM64**, and Codex
Desktop GUI activation are still required. The independently owned Sharp
absence/replacement question is outside this follow-up.

Private evidence is
`~/.codex/task-state/harness-rewrite-implementation/video-embedded-reconciliation-20261026/`:
`provenance-verification.json`, `esbuild-go-recipe-reconciliation.json`,
`esbuild-static-binary-source.json`, `bidi-source-reconciliation.json`,
`bidi-notice-reconciliation.json`, `source-acquisition.json`, the preserved
primary sources/notices, `recipient-supplement/`, `supplement-freeze.json` and
`supplement-verification.json`. No reviewed target import, upstream build/
installer, browser/video process, native compilation, admission or publication
occurred. The repository change for this follow-up is this new document only.
