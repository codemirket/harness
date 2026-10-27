# Raster release boundary — qualification only

> **Historical candidate.** The `dec135…` proposal and 123,560,526-byte
> selection below were superseded by the corrected 123,139,141-byte source kit.
> The later custom-build pin was `e1127db8741e5e349e347a3b8230ad32c36db55c2cba6a63c26eaff5c7551f82`;
> that route and its CLI were subsequently removed. See the
> [current upstream candidate](m4-raster-prebuilt-qualification.md).
> Raster admission remains closed. The figures below document the original
> qualification boundary and must not be used to assemble a current release.

Observed 2026-10-26 on macOS ARM64. Raster is a separate, **unadmitted**
release component. It requires the admitted Design component and the
`aarch64-apple-darwin` target. Normal manifest creation, release opening and
installation reject Raster while recipient source delivery and rights gates
remain open. No user installation or client configuration changed.

At this qualification, `src/distribution.rs` bound the proposed candidate by the SHA-256 of
`docs/historical-admission/m4-raster-source-recipient.json.proposed`:
`dec13575023f65a9dd0573a029b9ff72f61f39126eb8b2e5e5aa63763fd7b30b`.
The owner verifies its explicit false admission state, target, Sharp/libvips
versions, file counts, byte totals and finite paths before returning the closed
admission error. Editing a proposal or supplying a manifest flag cannot open
that gate. Admission requires a deliberate change to the compiled owner and
the owning admission evidence.

## Frozen candidate boundary

The candidate selection contains these exact, hash-bound files:

| Payload | Regular files | Bytes |
| --- | ---: | ---: |
| Private Sharp runtime | 124 | 16,087,960 |
| Raster notices | 71 | 2,151,327 |
| Same-candidate source kit | 1 | 105,315,672 |
| Recipient instructions and helper MIT terms | 2 | 5,567 |
| Total candidate selection | 198 | 123,560,526 |

The private module is fixed at
`runtime/design/raster/node_modules/sharp/dist/index.mjs`. A Raster-bearing
Design payload resolves that exact path under the verified release root;
Design alone supplies no raster module. There is no PATH or host-package
fallback and no manifest-selected module, executable, source-kit or notice
path. Candidate files use normalized mode `0600`. The ordinary compiled
inventory supplies directory, byte and manifest limits; candidate selection
cannot replace an existing component's file pins.

The unchanged Sharp 0.35.5 addon is 280,256 bytes, SHA-256
`791921f975371be33aed727e06283282cc001c960792eb52505b579fdaaf1b55`.
The reduced source-built libvips 8.18.7 library is 14,631,104 bytes, SHA-256
`1801c6f1cb12363a8d1815d224840001031ed96f6f986e9e91e03dce35ee453a`.
The source-kit SHA-256 is
`fb1f835e96de79e2a0b15f35df8aa051181b2b09de0d976c612873914025f284`.
This frozen qualification selection co-locates the source archive. A later
separate recipient download needs its own durable-access qualification and
release binding; a path in a receipt alone does not establish delivery.

The earlier [source closure and recipient journey](video-native-source-closure.md)
records native build inputs, notices, the functional library replacement and
rollback through development MCP. That evidence remains distinct from release
admission and successful installed operation.

## Checks performed

The build-input guard passed before Rust compilation. Focused distribution
tests passed: **25 passed, one explicit private-fixture test ignored**. The
ignored test was then invoked with the restored private recipient fixture and
passed verification of all 198 selected files against their compiled sizes and
SHA-256 pins. This read included the exact source-kit bytes and did not execute
upstream scripts, installers or native builds.

The new checks establish:

- The candidate adds exactly 198 files to Design without replacing its inventory.
- Both native pins and the private module path match the frozen candidate.
- Raster requires Design and rejects Windows x64 and ARM64.
- Normal manifest creation fails without writing a manifest.
- `Installation::setup` refuses a manifest naming Raster before creating the
  disposable installation home.
- Existing admitted release inventory, capacity, identity, mode and corruption
  checks still pass.

Reproduce the focused checks from the repository root:

```sh
python3 scripts/check-build-inputs.py
CARGO_BUILD_JOBS=1 cargo test --locked --offline --lib distribution::tests
MIRKET_RASTER_QUALIFICATION_FIXTURE=/absolute/private/restored-fixture \
  CARGO_BUILD_JOBS=1 cargo test --locked --offline --lib \
  distribution::tests::raster_private_fixture_matches_every_frozen_candidate_pin \
  -- --ignored --exact
```

The fixture environment variable belongs only to the ignored Rust test.
Production release selection has no qualification override.

## Remaining installed-release gate

An actual successful Raster `setup`/installed MCP/import/rollback trial was
**not performed**. The normal installer reopens the source and installed
release through the same closed compiled admission boundary. Bypassing that
boundary for a trial would not establish the intended normal integrity path.
The private fixture hash check is not an installed-release or native-client
success claim.

To complete the installed journey, first resolve durable same-release source
access and recipient delivery; final modification/debugging rights, combined
library obligations and applicable Installation Information; artifact
authentication, private build-path review and current dependency review.
Replace the proposed record with a reviewed admission and explicitly open the
Raster component in its compiled owner. Then assemble a disposable release
with this exact candidate and the admitted Design prerequisites, install it
through the normal owner, and exercise saved raster output, corruption
refusal, recipient modification/import and rollback without weakening release
integrity. Required Codex activation, minimum macOS, signing/notarization and
native Windows qualification remain separate gates. Chromium/HyperFrames and
combined-runtime obligations remain outside this Raster boundary.
