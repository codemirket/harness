# Sharp prebuilt source provenance — 2026-10-27

This verifies signed package-to-source identity for the two selected macOS
ARM64 native Sharp archives. It advances the
[upstream prebuilt candidate](m4-raster-prebuilt-qualification.md) and
supersedes the older [source review's](video-native-source-closure.md)
statement that its decoded npm provenance had not yet received signature and
transparency verification. **It does not complete native source, notice,
recipient-rights or release admission.** No upstream helper, installer,
native source build or changed runtime package ran.

| Selected archive | Local tarball SHA-256 | Signed source commit |
| --- | --- | --- |
| `@img/sharp-libvips-darwin-arm64` 1.3.4 | `e8350e62344d027995ef46cdd97a5ea1c88bc54ab82ab8c3fcb0c076d839675f` | [`lovell/sharp-libvips` `ebb95f8…`](https://github.com/lovell/sharp-libvips/tree/ebb95f8add54eee8bed840e3fb587e4cbec857d7) |
| `@img/sharp-darwin-arm64` 0.35.5 | `52168af984c322e5548bebfb437a9976244824338ea5b8a5b1e0cdaaf06480cb` | [`lovell/sharp` `51a990f…`](https://github.com/lovell/sharp/tree/51a990faa26ade5586a4934ac9673c98d8893326) |

The official npm registry's exact version records supply SLSA v1 bundles for
both packages. The saved local tarballs match their registry SHA-512 integrity
and the sole subjects in those bundles. Using the existing development Sigstore
4.1.1 client and a new private TUF cache, certificate and transparency-log
verification passed with the GitHub Actions OIDC issuer, certificate-transparency
and log thresholds of one, and these exact publisher identities:

- `https://github.com/lovell/sharp-libvips/.github/workflows/ci.yml@refs/tags/v1.3.4`;
- `https://github.com/lovell/sharp/.github/workflows/ci.yml@refs/tags/v0.35.5`.

The signed statements identify the same commits as the npm `gitHead` values.
Their pinned workflows exist in the corresponding
[sharp-libvips](https://github.com/lovell/sharp-libvips/blob/ebb95f8add54eee8bed840e3fb587e4cbec857d7/.github/workflows/ci.yml)
and [sharp](https://github.com/lovell/sharp/blob/51a990faa26ade5586a4934ac9673c98d8893326/.github/workflows/ci.yml)
source trees. A changed DSSE payload and an unrelated expected signer identity
were both rejected for each package. Verification used inert registry responses
and archives; it did not import Sharp or trust an archive-supplied key.

The two commit-addressed repository archives are retained privately:

| Source archive | Bytes | SHA-256 |
| --- | ---: | --- |
| `sharp-libvips-ebb95f8….tar.gz` | 24,490 | `65fdb633c2d97626352a3771b7a0ce30b6a89af03d2a1238491c8b068e6b8dad` |
| `sharp-51a990f….tar.gz` | 41,802,569 | `d600bcbdd550ea096a384391a474907e6aa8138f97ea472bf5228b07ae75f779` |

The selected libvips package's `versions.json` names 28 native component
versions. All 28 match the signed source commit's `versions.properties`, after
normalizing the two underscore-versus-hyphen names. The exact cross-check is
retained as `native-version-crosscheck.json`. The source commit includes its
original 4,230-byte `THIRD-PARTY-NOTICES.md` (SHA-256
`6f0d5086e3bcbee12bcdacf5878b31d1104002aeecaeb241d3fff466108082fe`),
the 23,877-byte native build recipe and both repository licenses. The npm
libvips tarball itself contains the binary, headers, metadata and README, but
no complete native source kit or notice file.

This correspondence is not an independently reproduced binary or a linked
object map. The publisher's [pinned native recipe](https://github.com/lovell/sharp-libvips/blob/ebb95f8add54eee8bed840e3fb587e4cbec857d7/build/posix.sh)
fetches and patches native sources, runs a mutable Rust/Cargo update for
librsvg, and fetches `THIRD-PARTY-NOTICES.md` from `main` during its build. The
earlier source follow-up inertly retained 25 named dependency archives for this
same recipe commit and 357 pristine librsvg-lock crates; the exact final edited
lock, compiled crate/linked-component closure, build-environment identities and
complete applicable notice/source packet for the **upstream full binary** remain
unverified. A separate reduced, source-built libvips candidate has different
bytes and does not qualify this prebuilt artifact.

All primary registry responses, attestation bundles, verifier, private TUF
cache, source archives, `verification.json` and version cross-check are retained
outside shipped source at
`~/.codex/task-state/harness-rewrite-implementation/sharp-prebuilt-provenance-20261027/`.
The verification report is SHA-256
`b38fc3a37a3835e4a205850706a70042975d8dafa73cc668235817ecd9324bf3`.
No source offer, recipient download, public release or license conclusion
followed this check. The current Raster admission flag stays closed.
