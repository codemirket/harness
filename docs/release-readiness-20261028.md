# macOS release readiness — 2026-10-28

This advances the approved plan from `7ffe47a`; it does not authorize publication
or changes to the user's installation. Qualification uses disposable homes on
macOS 26.5.2 ARM64. The proposed macOS 15 floor remains untested; the requested
platform access or revised first-release scope is still pending. Windows is a
separate later gate. Codex Desktop GUI activation remains waived and untested.

Raw evidence is retained at
`/Users/nazmiyilmaz/.codex/task-state/harness-release-readiness-20261028/`.
The final candidate is assembled locally. The platform-floor decision remains
open; this record does not claim a published or macOS 15 qualified release.

## Demonstrated defects and corrections

| Consequence | Correction / source | Outcome evidence |
| --- | --- | --- |
| The installed Video payload retained old Mirket JavaScript because a vendor inventory also pinned three first-party adapters. Current authoring diagnostics could not reach the release. | `src/distribution/video_candidate.rs` identifies only those three first-party files; `src/distribution.rs` keeps their 16 MiB bound. Their current bytes are bound by the release manifest, as with other first-party adapters. All vendor/media pins remain exact. | Current installed MCP diagnostics report the root fault and both bad caption cues, retain bounded diagnostics across result reads, preserve source bytes and publish no artifact. `installed-video/diagnostics-mp4-supplied-fixture.log`. |
| An interrupted fresh Codex setup left an owned staged home that another setup refused; update also required a healthy installation. | `src/installation.rs` resumes a recognized owned home only when its core doctor reports ready, after source and native-destination preflight. Complete, foreign and drifted homes remain refused. | The actual interrupted fixture and baseline refusal are retained. Two discriminating regression tests cover staged recovery and refusal to reopen a healthy home. Actual setup was stopped after staging at 312.917 seconds; the first-party installer resumed the same candidate in 16.451 seconds and doctor reported healthy. A different pending candidate was refused without adoption. |
| The Daybreak wordmark covered the live desktop CTA. | Head-only flow repair in `specimens/daybreak/index.html`; the film builder supplies its own head. | Original: −77.48px gap and `DIV.wordmark` intercept. Repaired: 32px desktop gap; desktop and 390×844 mobile clicks reach `#sessions`, no horizontal overflow. Saved desktop/mobile pixels inspected; accepted film unchanged. `ui-outcomes.json` and saved PNGs. |
| The recipient proposal still named predecessor FFmpeg binaries and a larger superseded Video closure; metadata alone could accept changed payload bytes. | Updated `packaging/admission/m3-ffmpeg-source-closure.json.proposed`; added `scripts/assemble-video-recipient.py` and its focused tests. | Exact archive, source, notice and binary verification; corrupt bytes, stale manifest, linked roots and unsafe paths are refused. Current packet selects 82 instances / 80 npm archives and the neutral FFmpeg successor. |

## Skill acceptance and the six-area review

The [six-area review](deep-review-20261027.md), its source locations and retained
failure cases remain the baseline. No new creative cohort or budget reset was
introduced during this release pass.

| Area | Current evidence and limit |
| --- | --- |
| Skill authoring/references | All 67 bodies and their pairings retain the exact-reference static review. Repository catalog refresh reports no drift. No reference was reintroduced or new authoring claim added. |
| Skill/tool pairing | Fresh installed navigation, tokens/SVG, raster and OpenPencil operations passed. Installed Video diagnostics now exercise the current adapter. Host-owned routes remain separate. |
| Project memory | Fresh two-client installed semantic journey passed; maintenance and D2 fixtures also passed. Before use, the two server trees totaled 22,848 KiB RSS; after use 1,688,448 KiB and after 30 seconds 1,095,248 KiB. These are sampled summed RSS, not physical peaks. The 30-second sample precedes the 60-second idle retirement deadline and does not establish unloading. |
| Capability matrix | The [workflow matrix](deep-review-capability-matrix.md) remains scoped to retained capabilities. Platform and rights checks are independent of tool receipts. |
| Agent coordination | The [native coordination trials](native-coordination-qualification-20261028.md) retain the mechanical stall and interruption failure. Mirket does not enforce native spawning/model/approval behavior. Lead integration and shared review budgets remain guidance. |
| Feedback/output quality | The accepted D6 direction and bounded maintenance journey remain closed. Four new noncreative model turns compare dependency admission and CI release decisions with matched no-skill controls; independent artifact review retains both positive and negative cases. |

The four new turns used `gpt-6-astra`, high reasoning, with a frozen protocol and
independent oracles. All completed with unchanged protected inputs and no
out-of-scope writes. Dependency admission passed both exact oracles. Both CI
conditions made the correct release/hold decisions, but failed a frozen oracle
that required literal enum strings absent from the task brief. These remain
**two strict failures**, explained as an overspecified oracle; the oracle was
not rewritten and no skill improvement is inferred. No repair turn was used.
Model execution totaled 247.003 seconds, 275,450 input tokens (238,464 cached)
and 5,840 output tokens. See `acceptance/protocol.json`, raw turns and
`acceptance/independent-evaluation.json`. These decisions did not activate or
publish anything and do not establish general skill superiority. Catalog-wide
`planned-not-run` acceptance remains unchanged.

## Delivery boundary

The lean release excludes Chrome, the Sharp native package closure and semantic
model/ONNX Runtime files. First-party setup obtains exact publisher archives
or explicitly supplied matching files into private staging. Their identities
and notices still matter; this is distinct from Mirket redistributing those
archives. No live installation is used for qualification.

FFmpeg binaries are bundled, so their corresponding source and notices must
accompany the release under the chosen delivery route. The current recipient
packet includes selected FFmpeg binaries, the neutral source kit, npm archives,
package notices and supplemental embedded-font/Puppeteer/native notices. Its
21 unavailable full upstream archives are supplementary provenance for the
selected permissively licensed payload, not a replacement for required notices.
All original OFL font notices are retained. MIT notice preservation and Apache-2.0
redistribution terms are distinct from GPL corresponding-source delivery; see
[esbuild MIT terms](https://github.com/evanw/esbuild/blob/v0.28.2/LICENSE.md),
[Apache-2.0 section 4](https://www.apache.org/licenses/LICENSE-2.0) and
[GPLv2 section 3](https://www.gnu.org/licenses/old-licenses/gpl-2.0.html). The source kit's prior guarded
rebuild evidence is preserved; a notice-only packet assembly is not another
native rebuild. A loopback HTTP download-and-unpack rehearsal verified all 12,922 delivered
files and all 64 FFmpeg source-kit inputs. The combined archive is 220,066,227
bytes, SHA-256 `a81f24c36a645cd1a4f3de14426d963ee8c4d19cc12dab60124f815b9d58ba1d`.
This demonstrates local source/notice access, not public distribution.
Public publication/authentication and actual public recipient access have not
occurred. Generic packet limitations remain in its frozen source index; unchanged
source does not require another native build merely because notices were repackaged.

## Security review

OSV checks covered 395 public package/version pairs: 225 crates.io and 170 npm.
The [`deepmerge@4.3.1` advisory](https://github.com/advisories/GHSA-gcqx-w48m-w2h4)
is retained, but the concrete accepted token path
rejects dangerous object keys recursively before Style Dictionary. A hostile
input was refused without output; a benign control compiled. Arbitrary project
configuration is not exposed. This is a transitive maintenance item, not a
demonstrated reachable exploit in the selected route. `paste@1.0.15` is reported
unmaintained, without a demonstrated exploit. Empty advisory results do not
prove absence of vulnerabilities. Raw responses and reachability evidence are
under `security/`.

The browser security review identified [52 fixes in the October 27 Chrome
update](https://chromereleases.googleblog.com/2026/10/stable-channel-update-for-desktop_0562499947.html),
after the existing renderer pin. The Mac pin is now **156.0.8078.73**, with its inventory generated by
`scripts/generate-chrome-admission.py`: archive 194,963,618 bytes, SHA-256
`a46be48ee2e33b3edc1a98a39668b9f0abb8f23f031bc4df195e2388e6de28d1`;
344 regular files / 383,457,057 bytes and five verified framework links.
Only the Mac patch changed; Windows retains its old unqualified pin. [Node 24.22.1](https://nodejs.org/en/blog/release/v24.22.1)
is the selected official security release. FFmpeg advisory/source-ancestry
evidence remains separate from binary reproducibility.


## Final candidate and source gates

Release ID: `sha256:6651f54ef26bb7ac30d388060e0dbc0d12ffb1b374d4f8f62ab1a15a7d75b61c`.
Full manifest SHA-256:
`1a5ac5bc81cd40395adee45de437b5eb2a17bec329de772b3ef9f2780f5cc521`.
Optimized Mirket SHA-256:
`88bbe8f3fb47334d5a1db7ba650ca76b100e52e86a954a61afd4afe0e523c8e0`.
First-party installer SHA-256:
`dc87c51f32f60c4d1bd05a03993a4394719daa0764a1c6b94570d062c5fe2a1b`.
The full installed manifest identifies 13,082 regular files and five links;
the lean delivery omits setup-acquired assets.

- Build-input guard, locked/offline Rust tests (508 passed, five ignored),
  all-target Clippy with warnings denied, formatting and optimized build passed.
  One additional test-only pending-activation regression was then added; the
  affected ten setup tests and all-target Clippy passed again.
- Locked TypeScript compilation passed. Adapter tests: 68 passed, one optional
  raster test skipped; the separate installed raster journey covers actual
  PNG/JPEG/WebP/AVIF operations.
- Python discovery passed 180 tests with one skip before the final generator
  additions; the final recipient/Chrome generator subset passed all 17 tests.
- Explicit-private-Node/optimized-binary MCP smoke passed; catalog refresh
  reported no hash/provenance drift.
- Actual same-home reinstall retained canonical recall and forgetting, backed
  up 17 state files, and reconciled completion without replay.
- Fresh preliminary full installation took 352.009 seconds. A first harness
  timeout at 300 seconds exposed the setup recovery defect; its failed attempt
  and refusal remain retained. The final same-candidate interrupted setup
  journey described above used the first-party installer and exact current pins.

The initial final-candidate attempt ran Design and Video fixtures concurrently
against the same installation. Both observed the documented operation-store
busy response on inspection; those harness failures are retained. The final
operation matrix is run sequentially. This is not a claim that the concurrent
attempt passed, or a reason to replay a potentially completed effect. Prior
combined-load evidence remains separately scoped.


The exact final installed candidate passed navigation, Design tokens/SVG,
PNG/JPEG/WebP/AVIF Raster, editable OpenPencil and Codex app-server managed
skill/memory/navigation checks. Video passed current bounded diagnostics,
captions in MP4/WebM/GIF/PNG sequences, decoded audio gain/timing, re-editing,
active cancellation, remote-script refusal and a PNG archive above 20 MiB.
See `final-video/diagnostics-mp4.log` and `final-sequential/results.json`.
Saved decoded MP4 frames at 0.0833, 0.75 and 1.9167 seconds were viewed: the
three animation paths progressed consistently, the local-font heading remained
legible, and the caption appeared during its cue and was absent afterward.
These are fixture pixel checks, not a new creative-quality or continuous-viewing
claim. The user-accepted Daybreak film itself remains unchanged.

## Remaining release decision

The sole product prerequisite for the first Mac release is the **support floor**:
provide a macOS 15 ARM64 host for actual qualification, or explicitly scope the
first release to the tested macOS 26.5.2 ARM64 host. No additional creative study,
engine replacement or new model trial is needed to answer that decision.
Windows, Desktop GUI and broader catalog/model-quality generalization remain
explicitly unqualified rather than being silently included in first-release
claims. Publishing, public release authentication and live installation changes
have not been authorized or performed. The prepared combined package supplies
local recipient source/notice access; its local checksum is not a substitute
for the trusted release record at publication.


## Local commits and changed files

Runtime, packaging and regression tests are committed locally as `f358379`
(`Fix release payload freshness, setup recovery and Chrome security pin`).
The accompanying maintained-page repair and qualification documentation are
committed separately. No commit was pushed. Relative to `7ffe47a`, the final
change inventory is:

```text
README.md
docs/design-production-closure-20261028.md
docs/m3-browser-setup-acquisition.md
docs/release-readiness-20261028.md
packaging/admission/m3-browser-cft-macos-arm64.json
packaging/admission/m3-ffmpeg-source-closure.json.proposed
packaging/install-prompt.md
planning/release-readiness.md
scripts/assemble-video-recipient.py
scripts/generate-chrome-admission.py
scripts/install-macos.sh
specimens/daybreak/README.md
specimens/daybreak/index.html
src/distribution.rs
src/distribution/chrome_admission.rs
src/distribution/video_candidate.rs
src/installation.rs
tests/test_assemble_video_recipient.py
tests/test_generate_chrome_admission.py
```
