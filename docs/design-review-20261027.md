# Recent design research and implemented review corrections

Scope: 2025-04-27 through 2026-10-27, researched on macOS ARM64. This is a focused follow-up to [design adoption](design-adoption-20261027.md), not completion of the broader 67-skill acceptance program. Hallmark, Taste and HyperFrames remain. No production dependency or installation was added.

## Research coverage and adoption decisions

The retained reports distinguish primary method evidence, community reception, promotional claims, conflicting experiences and unqualified paths:

- [Six dated primary articles and benchmark reports](research-20261027-design-harnesses.md).
- [Ten Reddit/Hacker News discussions, including dissent](research-20261027-design-community.md).
- [Seven qualifying design case studies and an explicitly excluded older example](research-20261027-design-directions.md).
- [Four directly read X discussions and pinned Hue implementation inspection](research-20261027-design-social.md).

These are not an exhaustive popularity survey. Likes, bookmarks, awards and stars locate examples worth studying; they do not measure causal quality gains. Undated pages and pre-window sources are background/exclusions. No upstream prose, executable code, templates, brand assets or fonts were copied into the shipped skills. New article/social observations are comparison evidence informing an original local procedure, not newly admitted catalog reference families. Existing source-method classifications remain intact.

| Demonstrated gap / research lesson | Correction and source owner | Observable outcome / qualification |
|---|---|---|
| A generic moodboard does not tell an implementer which property belongs to which component. Popular reference workflows work at section/property level. | `skills/art-direction/SKILL.md`: task-specific annotated reference/context pack, existing owners/tokens and rights boundaries. | Native guidance; general aesthetic uplift remains unmeasured. |
| Captures can show the wrong page/state; a green tool receipt does not prove visual review. | `skills/design-critique/references/review-packet.md`: explicit target/state, view conditions, located consequence, frozen expected outcome and distinct evidence types. | Human/model inspection still required; the helper below validates records, not visual quality. |
| Review history can lose its baseline, change its expected outcome or reset its budget through a new candidate. | `scripts/check-design-evidence.py` and `tests/test_design_evidence.py`: source/capture hashes, frozen contract, state/rubric/budget binding and one repair ledger. | Repository qualification utility, invoked through Codex shell. Not an installed Mirket MCP tool or an agent orchestrator. |
| The last iteration can be worse; self-healing can weaken the oracle. | Critique resource preserves candidates and rechecks frozen outcomes; checker refuses selecting recorded blockers/regressions/failed checks. | These checks depend on an honestly recorded packet. Rewriting the whole packet or omitting findings cannot be prevented by this utility. |
| Vague motion adjectives give poor repair targets. | `skills/motion-direction/SKILL.md`: relevant named controls, anticipation/hold/settle, decoded-export review; interactive mobile/reduced-motion checks where applicable. | No mandatory aesthetic preset, extra renderer or HyperFrames replacement. |
| Adapter accepted an unrelated selected font plus `src:local(...)`; rendering could depend on host fonts. | `adapters/src/video-export.ts` requires selected relative font URLs in each literal font-face declaration; adapter tests cover nested CSS, host/data sources, comments and mixed invalid faces. | Bounded authored-CSS preflight; not a complete CSS parser or proof that every visible glyph uses the intended face. Actual rendered export remains the final check. |
| The actual film then failed the caption schema and body insertion contract; a later export covered footer copy with captions. | `skills/video-production/references/hyperframes-0.8.197-authoring.md` now includes a complete HTML wrapper, exact cue-array example and caption safe-area inspection. | Adapter-owned requirements documented; the exercise's footer collision remains an observed artifact defect, not a claimed visual pass. |

## Ownership and tool peers

Codex captures and exercises live web interfaces, chooses agents/models and runs project checks. Project Storybook, accessibility, token and interaction tests remain project-owned. Mirket's qualified design operations handle editable vector/design artifacts and bounded raster operations; its private browser is used only for video production. HyperFrames produces video through the existing scoped adapter. The review procedure joins their saved outputs without introducing a second browser, orchestrator, design-system runtime or universal scoring model.

Hallmark/Taste already supply reviewed brief, direction, macrostructure and review methods from the previous pinned adoption. Hue is a promising comparison for observed-versus-derived component provenance, not a replacement or installed dependency. ProofShot illustrates evidence bundling; duplicating its browser/session service would violate the accepted boundary. Proprietary showcase artwork remains reference-only.

## Running the optional repository evidence check

The native `design-critique` skill distributes its Markdown review-packet resource. The Python helper is only available in this repository; installed skills do not promise it exists in arbitrary user projects.

```sh
python3 scripts/check-design-evidence.py --packet /absolute/review/review.json --root /absolute/review
```

The JSON schema and a minimal construction example are exercised in `tests/test_design_evidence.py`. Record the frozen brief/rubric/source/state/budget contract first, retain source snapshots and matched captures for each candidate, then add checks/findings and select a candidate. A `visual_report` records who reports inspecting which capture; success explicitly says `reported_only; pixels_seen_not_certified`. It cannot establish that the right screen was loaded, that somebody looked at it, or that an omitted defect is absent. The root is a trusted local qualification directory, not a hostile concurrent filesystem sandbox.

## Outcome evidence

Private evidence root: `/Users/nazmiyilmaz/.codex/task-state/harness-design-research-round2-20261027/`.

The evidence helper was exercised on an actual retained SVG and its saved 1000×900 render. The lead opened the saved pixels; XML/PNG checks were recorded separately. The valid record passed, while wrong route/state, changed expected outcome, stale image digest, blanket `verified`, failed automated check, and a three-cycle/two-repair ledger were refused (`evidence-check/results.json`). This tests record integrity against real files, not new generation quality. It deliberately does not claim that a hash proves the render corresponds to the source or that an attestation proves inspection.

The actual video MCP exercise copied the prior current-condition film into disposable state and reproduced `explicit_local_font_required`. Local font selection exposed two further authoring errors: `invalid_caption_cues`, then `caption_injection_requires_body`. Font, caption-array and body-wrapper corrections produced a 640×360, 12-fps, 36-frame, three-second H.264 MP4 through HyperFrames 0.8.197. The lead independently decoded the saved MP4 with the selected FFmpeg and checked its streams with ffprobe. Saved frames at 0.25/1.50/2.50 seconds were opened; the tide glyph changes location and the main messages are visible. Burned captions overlap the footer labels, so this is not a clean visual acceptance.

The delegated exercise also failed its process contract: three edit/retest cycles exceeded two allowed repairs, and its reported source manifest had an incorrect BRIEF digest. The lead compared every initial file with the old archive: the copied files are intact, but the reported manifest is invalid (`video-repair/lead-manifest-audit.json`). Preserve the failed protocol and amendments; do not recast the extra repair as the same pass or accept a rewritten freeze. Its technical export is separate evidence, not a successful frozen trial. The critique resource now states how cycles and amendments are counted. The checker catches recorded inconsistencies; it cannot intercept native Codex delegation or stop an agent before it writes.

The previous nine authoring trials remain historical evidence with their original limits; they are not relabelled as new controlled comparisons. No new matched no-skill aesthetic uplift claim is made, and no broad `planned-not-run` catalog acceptance is promoted by this work. Normal-speed playback, audio synchronization and alpha/player compatibility were not qualified by this film exercise.

### Final source gates

Actual platform: macOS 26.5.2 ARM64; private Node 24.22.1 and the repository's locked dependency closure. Final frozen-source checks:

| Gate | Actual result |
|---|---|
| `python3 scripts/check-build-inputs.py` | Passed before Rust compilation. |
| `cargo fmt --all -- --check` | Passed. |
| `cargo test --locked --offline` | 505 passed across suites; five manual tests ignored. |
| `cargo clippy --locked --offline --all-targets -- -D warnings` | Passed. |
| `cargo build --locked --offline` | Passed. |
| `python3 -m unittest discover -s tests -p 'test_*.py'` | 52 passed, including 16 evidence-packet cases. |
| Private Node → `adapters/node_modules/typescript/bin/tsc -p adapters/tsconfig.json`; private Node → `--test adapters/tests/*.test.mjs` | Build passed; 60 tests passed, one optional raster-runtime test skipped. No lockfile/dependency changes. |
| `tests/mcp-smoke.mjs --mirket /Users/nazmiyilmaz/Projects/harness/target/debug/mirket --node <explicit-private-node>` | Passed inventory/runtime, stale release, missing payload, unknown job, exact skill pairing and stale source cases. |
| `python3 scripts/refresh-skill-catalog.py` and `git diff --check` | Passed with no stale generated metadata or whitespace errors. |

The smoke Node was `/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/codex-gui-core-20261026/release-source/runtime/node/bin/node`. Smoke execution used a disposable home and temporary directory. Logs are under `gates-final/`; adapter logs and exact commands are in `gates/nonrust-results.json`. An earlier Rust run overlapped the last resource/catalog update and correctly refused two fixture manifests with `TooLarge` against the older compiled byte pins. Its failure log is retained in `gates/rust-1.log`; the complete frozen-source rerun above passed. No production bound was loosened.

The lead reran the actual video MCP operation after the final build, with explicit private Node/Chrome/FFmpeg/ffprobe paths recorded in `video-repair/runtime-final-head/runtime-initial.json`. It succeeded and independently decoded all 36 frames. Mirket binary SHA-256: `d2d780033489179af2d9f4f32afcbd13aa0ca3ca081f138aaf0415022fb4731f`; compiled `video-export.js`: `ca8af0eee5c294917babe00e0734abe249a16a26ed474315196913d2568bcee3`. The MP4 SHA-256 remains `97ea7a6a161a33f0851b2af43c1914a6844e0c8b685bd018982ba36a6c5d7482`. The lead opened all three final-run PNGs, confirming the same footer collision. This is a no-edit runtime recheck, not another creative repair or a reset of the failed trial budget.

### Changed source groups

- Adapter behavior and regressions: `adapters/src/video-export.ts`, `adapters/tests/video-export.test.mjs`.
- Optional evidence utility and regressions: `scripts/check-design-evidence.py`, `tests/test_design_evidence.py`.
- Native guidance: the art-direction, design-critique and motion-direction bodies; critique review-packet and video authoring resources.
- Repository-owned refresh: `scripts/refresh-skill-catalog.py`, `skills/catalog.json`, `skills/PROVENANCE.md`, `planning/reviews/2026-10-27/reference-audit.json`.
- Research/current-product documentation: this report, four linked research reports and `README.md`.

No release was published or pushed; no live installation, external account state or dependency admission was changed.

Windows qualification and the user-waived Desktop GUI check remain unverified.
