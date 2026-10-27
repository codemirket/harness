# Video authoring diagnostics and review integrity

2026-10-27. Implements the focused next steps from
[harness engineering research](harness-engineering-research-20261027.md).
This is original repository implementation and guidance, not copied upstream
source or a new dependency admission. HyperFrames remains the renderer.

## Changed behavior

Static video refusal used to reveal one category at a time, with every malformed
caption reduced to `invalid_caption_cues`. Preflight now returns bounded
independent reference/root/font/caption findings, including selected source
path, field, reason, expected bounds and zero-based cue index. Source
identity/revision checks still precede aggregation; unselected references are
never opened. Category precedence remains reference, root, font, caption after
valid source decoding. Lists are capped at 16 items and 8 KiB; they are not an
exhaustive review. Invalid UTF-8, JSON and missing caption insertion points are
explicit refusals. Missing, nonnumeric or infinite root duration no longer
passes a comparison involving NaN.

The Rust boundary validates diagnostic shape, bounds and selected path binding.
`video_status` and `video_result` retain the diagnostics with the durable
receipt. Invalid protocol identity, diagnostic bounds, or child cleanup still
take precedence over leaf claims; a hostile/over-budget stream need not yield
normal authoring feedback. This change does not certify visual quality or
change publication/ownership behavior.

The design evidence checker now rejects one saved artifact path reused for
different frozen states, or for both preview and decoded export of a motion
beat. Different files may have equal hashes: a legitimate hold or parity check
can contain identical pixels. These are record integrity rules. The checker
still cannot prove that pixels were inspected or that an attestation is true.

## Finding-to-fix scope

This follow-up addresses the demonstrated design/video gaps from the research.
The broader [six-area review](deep-review-20261027.md) retains its separate
evidence; these changes do not repeat or promote all of its qualifications.

| Area | Finding and correction | Source and evidence |
|---|---|---|
| Skill authoring/references | Video guidance understated caption output differences and offered only coarse refusal recovery. Clarified format coverage, bounded diagnostics and optional checkout utilities; retained Hallmark, Taste and HyperFrames. | `skills/video-production/references/hyperframes-0.8.197-authoring.md`; repository catalog refresh; four-format caption checks |
| Skill/tool pairing | Adapter detail was lost at the Rust boundary. Validated selected-path diagnostics now survive failed jobs into durable status/result. No new MCP surface or host-owned workflow added. | `adapters/src/video-main.ts`, `src/process_video.rs`, `src/video_session.rs`; fake-process negative tests and real MCP refusal/result check |
| Project memory | No new memory defect was demonstrated by this focused follow-up. Existing memory fixes and measured limits remain in the six-area review. | `docs/deep-review-20261027.md`; not newly requalified here |
| Capability matrix | Test fixtures could select caption metadata without exercising captions. Selection is now explicit and recorded; sidecar coverage is format-specific. | `tests/video-fixture.mjs`, `tests/design-adoption-video-runtime.mjs`, `tests/motion-authoring-runtime.mjs`; fixture tests and decoded four-format exports |
| Agent coordination | A fresh specimen or delegate must not reset a repair allowance. The new pilot freezes a distinct task matrix and shared ledger; earlier exhausted motion trials stay failed. | `tests/video-authoring-pilot.py`; ledger tests and retained model-turn records. Native Codex spawning is still guidance, not Mirket enforcement |
| Feedback/output quality | Missing/nonnumeric root duration could evade a NaN comparison; caption errors hid actionable fields; one capture path could masquerade as distinct review evidence. Added finite-duration checks, bounded multi-finding preflight and capture-path integrity rules. | `adapters/src/video-export.ts`, `scripts/check-design-evidence.py`; discriminating tests, immutable source/refusal probes and independent export review |

Changed files in this follow-up:

- Runtime: `adapters/src/video-export.ts`, `adapters/src/video-main.ts`,
  `src/video.rs`, `src/process_video.rs`, `src/video_session.rs`, `src/server.rs`.
- Authoring/evidence/catalog utilities: `scripts/check-video-source.mjs`,
  `scripts/create-video-seed.mjs`, `scripts/check-design-evidence.py`,
  `scripts/refresh-skill-catalog.py`.
- Skills: `skills/video-production/references/hyperframes-0.8.197-authoring.md`,
  `skills/design-critique/references/review-packet.md`, `skills/catalog.json`,
  `skills/PROVENANCE.md`.
- Tests and qualification runners: `adapters/tests/video-export.test.mjs`,
  `tests/video-authoring-tools.test.mjs`, `tests/video-fixture.mjs`,
  `tests/video-fixture.test.mjs`, `tests/video-mcp-smoke.mjs`,
  `tests/motion-authoring-runtime.mjs`, `tests/design-adoption-video-runtime.mjs`,
  `tests/test_design_evidence.py`, `tests/video-authoring-pilot.py`,
  `tests/test_video_authoring_pilot.py`.
- Documentation: `README.md`, this report,
  `docs/harness-engineering-research-20261027.md`,
  `docs/harness-engineering-sources-20261027.md`,
  `docs/video-authoring-pilot-20261027.md`,
  `docs/video-authoring-pilot-exports-20261027.md`.

## Repository authoring utilities

Use an explicit private Node executable. These utilities are available in the
harness checkout with its compiled adapter; they are not installed MCP tools.

```sh
"$PRIVATE_NODE" scripts/create-video-seed.mjs \
  --destination /absolute/new-project \
  --font /absolute/selected-font.ttf \
  --font-notice /absolute/font-license.txt --with-captions

"$PRIVATE_NODE" scripts/check-video-source.mjs \
  --project /absolute/new-project \
  --manifest /absolute/new-project/video-manifest.json
```

The seed creates editable 640×360, 12 fps, 36-frame native-WAAPI source and
copies only the caller-selected font and notice. It refuses existing
destinations and does not download assets or establish their reuse rights.
Caption selection is optional. The sample is a valid starting point, not an
art direction or a requirement to use its motion/palette in finished work.

The read-only checker reuses the compiled production validator. It reads only
the explicit manifest and selected sources, starts no media child and writes
no project files. Manifest shape:

```json
{
  "schema_version": 1,
  "entry_file": "index.html",
  "profile": {
    "width": 640, "height": 360,
    "fps": {"numerator": 12, "denominator": 1},
    "frame_count": 36, "format": "mp4", "captions_path": null
  },
  "sources": [
    {"path": "index.html", "revision": "sha256:<64 lowercase hex digits>", "bytes": 123},
    {"path": "SelectedFont.ttf", "revision": "sha256:<64 lowercase hex digits>", "bytes": 456}
  ]
}
```

The example hashes and byte counts are placeholders, not executable evidence.
After an intentional source edit, explicitly recompute SHA-256 and byte count
for the selected files and save a new manifest. The checker deliberately does
not silently refresh stale revisions. A valid result means static checks only;
font rendering, intended motion, audio, playback and visual quality still need
actual exported-artifact checks.

## Qualification inputs

The motion/design trial runtime scripts now require an explicit
`--captions PATH|none|metadata-only` choice. `PATH` selects burned captions;
`metadata-only` retains `captions.json` while recording that caption rendering
was not tested. `none` omits the conventional caption asset. Shared test-only
request construction preserves exact selected bytes and limits. Failed
multi-variant qualification retains partial receipts rather than losing all
records when a later variant fails.

Burned captions apply to MP4/WebM/GIF and PNG frames. SRT/WebVTT sidecars are
retained in PNG-sequence tar exports; other formats do not promise sidecars.

## Evidence status

Verified on macOS 26.5.2 ARM64 with private Node 24.22.1 and locked Rust 1.99.1.
Evidence root:
`/Users/nazmiyilmaz/.codex/task-state/harness-video-diagnostics-20261027/gates/`.

| Check | Observed result |
|---|---|
| Build-input guard | Passed admitted source/build-script guards before compilation |
| `cargo test --locked --offline` | 505 passed across suites, 5 explicitly ignored; `MIRKET_TEST_NODE` supplied for private process-boundary probes |
| All-target locked/offline Clippy, warnings denied | Passed |
| Rust formatting | Passed |
| Locked adapter TypeScript build and adapter/authoring/fixture/preview tests | Build passed; final combined suite: 87 passed, 1 skipped |
| Python repository tests | Final suite after the usage-reporting correction: 71 passed, including the actual model-facing diagnostic helper test with explicit dependencies |
| Exact MCP inventory/runtime and negative smoke | Passed with explicit binary/private Node paths |
| Actual MCP diagnostic refusal | Missing root marker and both malformed cues returned together; selected bytes preserved, no destination, owned cleanup confirmed, same diagnostics read from durable result |
| Actual captions in MP4/WebM/GIF/PNG tar | All four exported and independently decoded; observed caption-region changes below |
| Catalog refresh/provenance | Repository-owned refresh is idempotent; acceptance values and dependency lockfiles unchanged |

Opt-in native tests that do not receive their complete executable environment
do not establish native qualification merely by returning from a Rust test.
The video evidence above comes from the explicit real MCP runs instead.
The Node skip is the private raster-module probe. Rust's five ignored checks
require separate Sharp, semantic-model, real-uninstall or scale fixtures; this
follow-up does not newly qualify those paths.

| Format | Changed caption pixels during cue | After cue versus before | Additional evidence |
|---|---:|---:|---|
| MP4 | 13,215 | 0 | Saved decoded during-cue PNG inspected |
| WebM | 12,982 | 5 | Saved decoded during-cue PNG inspected |
| GIF | 11,993 | 0 | Saved decoded during-cue PNG inspected |
| PNG tar | 11,993 | 0 | Before/during/after saved PNGs inspected; SRT/VTT timing and Unicode checked |

The viewed samples show legible supplied Turkish caption text in the reserved
lower region, with no collision in this fixture. These checks do not qualify
arbitrary captions, complete normal-speed playback, other players or alpha
support. Initial compressed-format extraction attempts failed because the
test requested the unavailable `image2pipe` muxer. The extractor was corrected
to the private build's `image2` file output; original failure logs remain, and
all three affected formats were rerun successfully. No renderer fallback was
introduced.

The separate
[diagnostic authoring pilot](video-authoring-pilot-20261027.md) has its own frozen
inputs and budget. It completed 24 cells: 17 met the authoring contract and
budget, 19 passed static checks, and all 15 available video sources exported
and decoded. Four refusal cells and five missing-output failures remain
separate. Baseline completed 9/12 and diagnostics+seed 8/12; this small bundled
comparison does not establish creative or repair-efficiency uplift. The
[independent pixel review](video-authoring-pilot-exports-20261027.md) records
saved-frame observations, a corrected review misread and a font-use coverage
limit in the minimal controls. The subsequent reference clarification requires
applying the selected family, while keeping static font checks distinct from
actual glyph fidelity. The frozen pilot was not rerun for that clarification.

The earlier four-repair motion trial remains exhausted and
failed at caption preflight. No catalog acceptance is promoted from
`planned-not-run` by these changes. Windows and the waived Desktop GUI check
remain unqualified. No live installation changes or publication are included.
