# Motion authoring implementation — 2026-10-27

This implements the approved [Remotion/SVGator comparison](research-20261027-motion-authoring-comparison.md) around the existing HyperFrames route. No production dependency, renderer, browser MCP, agent orchestrator or installation change is introduced. Hallmark and Taste remain the UI reference owners. The new repository helpers are development utilities, not installed Mirket tools.

## Authoring preview

`scripts/create-motion-preview.mjs` creates a separate same-origin HTML review page for an explicit project entry and finite motion manifest. It supports document-timeline CSS keyframes and native WAAPI, stable unique element IDs, per-property ownership context, integer frame seeking, named beats and playback controls. It refuses CSS transitions, scroll timelines, external animation runtimes, ambiguous owners and changed profile metadata. The source remains editable; reload after source edits. The generation-time entry digest is context, not a frozen source snapshot or an export-parity claim.

Run with an explicit Node path and absolute paths:

```sh
/private/node scripts/create-motion-preview.mjs --project /project --entry /project/index.html --motion /project/motion.json --output /project/preview.html
```

Serve the project through its existing local server and open the preview in Codex's browser. The utility does not start a server or acquire anything. Existing outputs are refused rather than overwritten. Keep preview HTML out of the source selection for video export.

The manifest has `fps: {numerator: 12, denominator: 1}`, `frame_count: 36`, and `beats: [{id: "hold", frame: 12, expected: "message readable"}]`. Root dimensions/duration must match the composition. Shorter tracks and deliberate delays/holds are valid; they need not equal the full film duration. The inspector exposes their timing rather than imposing a universal pace.

## Original resources and evidence

The [three original examples](motion-authoring-examples-20261027.md) cover live save feedback, an editable SVG route and a typography beat. Vector/type examples use finite CSS; the live Save interaction uses WAAPI; its state semantics are tested in the browser, independently of the finite video fixture. The video skill now records frame/seconds arithmetic, named beat correspondence and the distinction between native WAAPI seeking and the separately registered Anime adapter. No upstream source prose, code, branded assets or fonts were copied into these resources.

The existing evidence checker now binds an optional motion manifest to the frozen contract and requires paired preview/decoded-export records at the same beat/frame/state when parity is claimed. It preserves the task's shared repair budget. It verifies record integrity, not pixels or visual quality.

Catalog hashes, resource inventories and original-method provenance are refreshed by `scripts/refresh-skill-catalog.py`; these three changed skill families cite this local synthesis. The comparison reports retain exact source/license pins. No new runtime or upstream skill body is distributed, and no planned-not-run acceptance is promoted.

## Qualification

Private evidence: `/Users/nazmiyilmaz/.codex/task-state/harness-motion-authoring-20261027/`.

Actual platform is macOS 26.5.2 ARM64. Private Node is 24.22.1; HyperFrames producer remains 0.8.197. The exact private Node/Chrome/FFmpeg/ffprobe paths and file hashes are in each `runtime/*/runtime-receipt.json`. The binaries came from disposable qualification payloads, not the live installation.

### Finding-to-fix and observed outcomes

| Finding / need | Change | Actual evidence and limit |
| --- | --- | --- |
| Timing edits lacked a usable common review surface | `scripts/create-motion-preview.mjs`, `tests/motion-preview.test.mjs` | Codex in-app browser sought 0→6→12→24→35→12, and back to zero. CSS and native-WAAPI subjects appeared at x=32,80,128,224,312. Property-owner conflicts and external Anime were refused by this bounded preview. Playback controls advanced and stopped; smoothness is not certified by still captures |
| Reordering could make ordinal edit targets ambiguous | Stable IDs/property owners, generation-time entry digest, actual effect/keyframe context | Moving the subject's source position retained x=128 at frame 12; halving authored travel yielded x=80. Live source reload is intentional, not a frozen-source promise |
| Preview agreement could be asserted from a receipt | Motion extension to `scripts/check-design-evidence.py` and critique reference | Actual 5-beat/10-capture packet accepted; stale hash, wrong frame, missing counterpart, expectation change and budget reset refused. Retrospective record-integrity exercise, not a new frozen quality trial |
| Unregistered library animation can silently freeze | Existing Anime registration guidance retained; matched negative added to runtime qualification | Registered Anime moved x=32,80,128,224,312; unregistered Anime stayed x=32 at all five frames despite successful export and moving neighboring CSS. Both saved outputs were retained |
| Partial source inspection incorrectly suggested native WAAPI would freeze | Corrected analysis and qualified native WAAPI separately | Its separate runtime adapter discovers and seeks native handles. The actual paused `element.animate()` specimen moved at all five expected export positions and matching preview positions. The earlier unsupported-WAAPI hypothesis is rejected |
| Source-only model handoffs omitted required producer fields | `motion-direction` now routes authored video source to `video-production`; video entry point requires the contract before authoring and names root/caption fields | Four initial specimens omitted `data-no-timeline`. Each received one root-marker repair; all then failed `invalid_caption_cues`. Final guidance correction is unmeasured in another model trial; no extra trial or repair resets the exhausted budget |

Three original snippets are retained under the motion resource: live Save acknowledgement, vector route draw-on and staggered typography. The lead exercised the Save click in Codex's in-app browser and observed `Saved.`; the CSS examples were extracted into the technical film and rendered by the real disposable Mirket MCP route. Saved preview and decoded export pixels were opened independently. The static scenes are teaching examples, not a claim of art-direction superiority.

### Actual export and pixel evidence

The CSS, native-WAAPI and changed-travel fixtures each produced a 640×360, 12-fps, 36-frame, three-second MP4. Decoded CSS coordinates matched the browser observations exactly. The changed-travel output produced x=32,56,80,128,172, demonstrating the intended control effect rather than merely a changed file hash. Source snapshots, MP4s, FFprobe output and PNG/RGB samples are retained under `runtime/private-*`.

The five saved browser samples are JPEG payloads; original captures were initially named `.png`. Byte-identical `.jpg` copies and corrected packet paths retain the provenance. Comparing lossy browser JPEG pixels with decoded H.264 RGB yielded mean absolute channel differences of 2.33–3.11 on a 0–255 scale. These are measurements, not a universal visual threshold or exact color-parity claim. Geometry and readable content were inspected; color-management equivalence, audio, transparency, other players and broad smooth-playback quality remain unqualified.

The selected FFmpeg build rejected the first decoder's `select` filter. Failed receipts are retained under `runtime/failed-select`; the runner now uses timestamp seeking and independently checks stream dimensions/FPS/frame count. This was a qualification-script repair, not a change to model specimens or a HyperFrames defect.

### Held-out model outcome

See [the retained trial report](motion-authoring-trials-20261027.md). Four initial model turns compared baseline/current motion resources on two frozen briefs. Exactly four repair turns were then spent, one per specimen, all changing only the missing root marker. All repaired exports were refused for captions using camelCase/inclusive fields or a wrapper document instead of the required snake_case, end-exclusive cue array. No model MP4 exists, and there is no decoded model visual-quality comparison. Initials, repairs, findings, manifests and unresolved defects remain retained. The backend model/reasoning identity is unavailable, which independently prevents causal skill-uplift claims.

The original code reviewer suggested requiring every track's duration to equal the film duration. That was not adopted: short entrances followed by holds and delayed/staggered tracks are intentional. Instead the inspector exposes effect timing and per-keyframe easing separately. The accepted source-copy race finding was fixed: the runtime runner writes exactly the bytes it hashed to evidence and the disposable project.

### Source gates

Final results:

| Gate | Result |
| --- | --- |
| Build-input guard before Rust compilation | Passed |
| `cargo test --locked --offline` | 505 passed, five explicit manual tests ignored |
| `cargo clippy --locked --offline --all-targets -- -D warnings` | Passed |
| `cargo fmt --all -- --check` and locked/offline build | Passed |
| Locked adapter TypeScript build and adapter tests | 60 passed, one optional raster runtime test skipped |
| Preview utility tests after final clock guard | 8 passed |
| Full Python suite; affected suite after final skill routing correction | 56 passed; 34 affected tests passed |
| MCP smoke with explicit private Node and rebuilt binary | Passed in disposable HOME/TMPDIR |
| Final rebuilt-binary native video recheck | Succeeded; 36 decoded frames and five expected subject positions |
| Repository catalog refresh / diff checks | No stale hashes; clean whitespace |

The optional system skill-creator `quick_validate.py` could not run: PyYAML is absent in both system and bundled Python. No dependency was installed for it. Repository-owned native skill/catalog/assembly validation passed instead.

Final Mirket binary SHA-256: `1bdbcbd0b4ec0e058b975d2e9cc090e3afc2d5ed96a47b6015d56b4c12d8f676`. Compiled `video-export.js`: `ca8af0eee5c294917babe00e0734abe249a16a26ed474315196913d2568bcee3`. The last native output is under `runtime/final-source-css/`; its decoded middle frame was opened after that run. Full command logs are `rust-final-source.log`, `clippy-final-source.log`, `adapter-frozen.log`, `preview-final.log`, `python-frozen.log`, `python-final-source.log` and `mcp-final-source.log`.

A read-only source-preview check also exposed a competing autoplay clock in one model specimen: it could change animation time after a frame was selected. The helper now continuously checks that its paused native handles retain the selected time and disables controls on interference. The actual source triggered that refusal in Codex's browser; the technical fixture still sought and played successfully. This does not detect arbitrary JavaScript changes outside tracked animations. The original source-preview screenshots are observational only, not a controlled aesthetic comparison or decoded export evidence. One intermediate Rust run overlapped a skill-resource correction and rejected five stale resource-digest fixtures; its failure log is preserved (`rust-final.log`). The later run uses the refreshed catalog and frozen resources. Windows execution and the waived Desktop GUI check remain unverified. A local Codex browser preview is not the waived installed-client GUI qualification.


## Changed files and remaining scope

- Preview utility and tests: `scripts/create-motion-preview.mjs`, `tests/motion-preview.test.mjs`.
- Evidence utility/schema/tests: `scripts/check-design-evidence.py`, `tests/test_design_evidence.py`, critique review-packet resource.
- Native guidance/resources: motion-direction and video-production bodies; editable-motion, preview-and-beats and pinned HyperFrames references.
- Actual runtime fixture: `tests/motion-authoring-runtime.mjs`.
- Catalog/provenance tooling: refresh script, generated catalog/provenance/audit and exact resource-inventory regression expectations.
- Research/evidence/current documentation: this report, example/trial reports, the three prior-turn research reports, and README.

No dependency, release, push or live installation change occurred. Source helpers and native guidance are implemented; the held-out model export acceptance remains failed, and the final authoring-route correction has not received another bounded model comparison. Color parity, audio/alpha/player breadth, arbitrary JS clocks and Windows remain unqualified. No broad catalog acceptance status was promoted.
