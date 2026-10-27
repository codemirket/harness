# Skill guidance and catalog status: local qualification

On 2026-10-27, a fresh disposable macOS 26.5.2 ARM64 installation activated
release `sha256:85a9dc580bf6aaa10df1ba53bbe239479601eef875d50ea204e2d689706e2473`.
Its source was assembled from the current 69-file native skill corpus, the
revised art-direction body and Codex guidance, and the previously qualified
seven-component local Mac runtime. Its source manifest SHA-256 is
`23edeef221a2f515657f446a479e47e6a52a3c01591c2fef642319fae44a46bb`;
its Rust executable SHA-256 is
`56c278ff9b4185f8b8f5c966132a8fa34bbd7a913b767d7f5c39816c1b1e8988`.
No real Mirket home or Codex configuration was used as an installation target.

First-party `setup` installed the core, nine global skills, managed Codex
guidance and MCP registration in a task-owned user home. `skills project-select
--include art-direction` installed the exact project body. Installed `doctor`
reported all seven components, global skills, guidance, registration, manifest
and launcher healthy. Project doctor reported the selected skill managed. The
installed body SHA-256 is
`0591e59891f86d82ad37820128160283ceb47b21850f65b43317564bbf7411dd`,
identical to source; the managed guidance contains the explicit saved-pixel
inspection rule. The read-only qualification summary is SHA-256
`57670f4d4852cab80decc214af588dfd6acc938ba6934aca02f42d9367433469`.
The real Codex config SHA-256 stayed
`a24841bc83964a74a4c138a338609c88cd43bb7604d1040bd9efd85e920062bf`.

A status wording change in `skills/catalog.json` initially made an earlier
disposable installation fail health inspection with the interim verifier,
although its release bytes were intact. The verifier had required one exact
English evaluation-status sentence. It now checks a bounded, nonempty status
without control characters; source pins, manifest hashes, skill identities and
the rest of catalog validation remain in force. Six focused native-skill tests,
including a status-change regression, pass. Strict all-target Clippy,
formatting, the assembly tests and the pre-build guard pass. The complete
locked Rust suite passed immediately before this narrow validator change:
492 library tests, five CLI tests and one active integration test; the new
validator path has the focused six-test result.

A newer development binary with that validator repair (SHA-256
`d7a9e0e56bd290b0f030f3efb7bc89bdee95c7fc562020a96c4b28dce25b318a`)
also reported the just-installed release healthy without changing it. A
separate complete source manifest was created for that newer binary as release
`sha256:551dbc9d6754ba9003734c5b171c96878f4e09483421f56b602cf625e122d5d9`
(manifest SHA-256
`e586bc8c019d7a34fc4d6592bf7ff8c571c80f467383e6a5263dd20df3f82580`).
That second candidate was not activated; the passive doctor and focused tests
qualify only the changed catalog-verification boundary.

The [sampled Codex skill trials](m4-skill-acceptance-sample-20261027.md)
reported visual inspection without image-view items in JSONL. Later controlled
image and missing-file probes show that this CLI trace omits image access, so
the missing items cannot establish that the model's claims were false. One
guide retry wrote its render outside the requested project; that scope
violation remains. The installed bytes and healthy doctors establish
distribution integrity, not model adherence or design acceptance. The user's
waived Desktop GUI test,
Windows native gates, Mac OS 15 floor and public source/notice routes remain
under their separate decisions. No commit, push or publication followed.

Private source, installed state, doctor outputs and exact trial traces are
under `~/.codex/task-state/harness-rewrite-implementation/simplification-20261027/`
and `~/.codex/task-state/harness-rewrite-implementation/skill-acceptance-20261027/`.
