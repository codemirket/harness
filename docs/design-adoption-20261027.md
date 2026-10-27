# Accepted design adoption — 2026-10-27

Implementation base: `ed5ec1342ca90eb93b43c776fb4989ac595b84be`. This follows
the user-accepted [design research](research-20261027-design-stack.md) and the
[earlier six-area review](deep-review-20261027.md). Hallmark, Taste and
HyperFrames remain selected. No production dependency, renderer, global skill,
second orchestrator or live installation update was added.

The implementation is locally verified. General design-quality improvement is
**not established**. The bounded model comparison exposed source-policy defects
and control contamination; no skill acceptance set is promoted from
`planned-not-run`.

## Six-area finding-to-fix matrix

| Area | Demonstrated gap and correction | Source locations | Evidence and limits |
|---|---|---|---|
| 1. Skill authoring and references | The UI references were mostly comparison locators; useful subject-to-composition, claim/proof and component-state methods were diluted. Added conditional methods to eight project skills, three on-demand resources, and explicit method-adapted provenance. Corrected MengTo paths and HyperFrames Apache-2.0 metadata. Avoided blanket anatomy, proof and competitor-research requirements. | [Mapping ledger](../planning/reviews/2026-10-27/source-design-adoption.md), [art direction](../skills/art-direction/SKILL.md), [interface composition](../skills/interface-composition/SKILL.md), [vector art](../skills/vector-art/SKILL.md), [provenance](../skills/PROVENANCE.md) | Exact pinned bodies/licenses inspected; 60 families / 61 memberships retained. Independent wording and source rights are distinguished. The nine global skills and 58 project skill names/counts remain unchanged. |
| 2. Skill-to-tool pairing | Supporting resources would have been omitted or rejected by the entrypoint-only assembly, release manifest and install fixtures. Added exact bounded resource lists, digest/byte validation, safe path checks, staging checks and release ownership. HyperFrames examples now use the admitted Anime v4 API/registry. | [Assembler](../scripts/assemble-skills.py), [native catalog](../src/native_skills.rs), [sync](../src/skill_sync.rs), [distribution](../src/distribution.rs), [installation](../src/installation.rs), [video authoring](../skills/video-production/references/hyperframes-0.8.197-authoring.md) | Disposable install/recovery tests verify the storyboard resource reaches the project. Drift, unlisted file, symlink, unsafe path and empty-directory regressions pass. Actual MCP inventory and configured/stale/absent/unknown-job cases pass. Qualified video control renders and decoded intended-object movement pass. |
| 3. Project memory | No new memory change was justified by design adoption. Preserve the earlier fix that aligns reported semantic coverage with actually readable chunks. | [Prior memory correction/evidence](deep-review-memory.md), [coverage implementation](../src/memory_index.rs) | Current full Rust regression suite passes. Prior real-model isolation/forgetting/idle evidence remains dated to its recorded binary; it was not relabelled as a fresh design-adoption experiment. SQLite/FTS and lazy MiniLM remain. |
| 4. Capability matrix | Illustration construction and video direction needed specific resources, not more global guidance or duplicate tools. Added MengTo construction and Huashu shot-card resources under existing project skills. | [Workflow/owner/QA map](research-20261027-design-stack.md#capability-and-qa-map), [67-skill/eight-profile matrix](deep-review-capability-matrix.md), [illustration](../skills/vector-art/references/illustration-craft.md), [storyboards](../skills/motion-direction/references/storyboard-and-transitions.md) | Codex/project owns DOM/SVG authoring, general browsing, model choice and permissions. Mirket retains bounded SVGO/resvg/OpenPencil/raster/video operations. No additional runtime or agent framework was admitted. |
| 5. Agent coordination | Model authoring turns did not establish native-spawn enforcement or a full shared-budget repair loop. Read-only independent specialists reviewed disjoint source areas; lead integrated corrections. | [Existing coordination contract](../skills/agent-coordination-strategy/SKILL.md), [managed guidance](../src/codex_guidance.rs), [trial protocol](design-adoption-trials-20261027.md) | Nine bounded authoring turns used matched executable/options, but backend model/reasoning identity is unknown. No additional author repair was run and no budget reset is claimed. Mirket cannot intercept native spawning. |
| 6. Feedback and output quality | Whole-frame movement hid an unregistered frozen Anime subject. Independent replay also rejected all three authored film specimens: two missing no-timeline marker; revised one missing selected font binary. Baseline/UI traces wrote temporary logs outside the requested project. | [Video oracle and sources](design-adoption-video-20261027.md), [model evidence](design-adoption-trials-20261027.md), [scope guidance](../skills/art-direction/SKILL.md), [trial runner](../tests/design-adoption-trials.py) | Saved full illustration pixels and UI browser journeys were independently inspected. Registered/unregistered video controls were decoded and viewed. Frozen specimens were not repaired after their budget. Font and scratch-scope wording was clarified afterward; compliance with those final clarifications remains untested. |

## What is now shipped in the skills

Hallmark/Taste guide subject-specific art direction and interface composition;
Impeccable informs critique; Emil informs purposeful, frequency-aware UI and
scroll motion. MengTo contributes conditional illustration construction and
whole/detail inspection methods. Huashu contributes storyboard, continuity and
acceptance-frame methods. HyperFrames guidance is tied to producer 0.8.197 and
the narrower Mirket no-timeline/local-assets profile.

The three resource files load on demand. The catalog now pins their paths,
byte counts and hashes, and the release manifest and project receipts own them.
The package has 67 skill entrypoints, three resource bodies, catalog and
provenance. Catalog/provenance/audit updates use repository-owned refresh tooling.
Source references are data; their installers, agent policies, assets and runtime
recommendations were not imported.

## Actual artifact outcomes

The nine saved model specimens cover product UI, editable illustration and
video authoring under no-Mirket-skill, previous-skill and revised-skill conditions.
All processes completed inside their individual 180-second authoring caps.
Full initial/final manifests, frozen briefs/skill snapshots, traces and independent
observations are retained under:

`/Users/nazmiyilmaz/.codex/task-state/harness-design-adoption-20261027/`

All three UI specimens passed independently exercised success, empty and error
filter journeys and had no horizontal overflow at 390px. Revised rapid filtering
ended on the correct route. Full saved SVG previews from all three conditions
show the requested watering action, grip, soil contact and grounded feet. These
are useful outcomes, but they do not demonstrate a consistent quality gain over
the controls. Full keyboard traversal, reduced-motion behavior and every visual
state were not qualified in this pass.

The nominal no-skill control could access inherited plugin guidance, and the
baseline read installed Mirket guidance. `HOME` was disposable but `CODEX_HOME`
reused the real authentication location; no isolation claim follows from those
flags. The baseline and no-skill runs wrote temporary render logs outside their
project. No live Mirket installation update was performed. The final scope
clarification explicitly includes logs/profiles/caches and is guidance, not
filesystem enforcement.

Replaying each film through the actual disposable MCP route produced a concrete
pre-render refusal, not an MP4. The revised source's `local('Helvetica')` is not
a selected font file. The reference now makes this distinction explicit and
requires preserving that prerequisite as unresolved when no permitted font file
exists. Frozen sources and their failures remain unchanged. These experiments
must not be called a completed review/repair loop or successful model export.

Separately, the qualified native HyperFrames control produced retained MP4s and
decoded frames. The registered orange subject moved from x=72 to 492 while the
unregistered control stayed at x=32; an independent blue CSS control moved in
both. Saved final PNGs were displayed with an image-capable viewer. This proves
the stated registration failure and correction on this runtime; it does not
establish arbitrary-video quality or full playback/player support.

## Verification and remaining qualification

Actual platform: macOS 26.5.2, ARM64. Rust uses the locked/offline closure; Node
and video executables use explicit private paths recorded in the evidence.
Final frozen-source gates passed:

- `python3 scripts/check-build-inputs.py` before Rust compilation.
- `cargo test --locked --offline`: 505 passed, five manual tests ignored.
- `cargo clippy --locked --offline --all-targets -- -D warnings` and
  `cargo fmt --all -- --check`.
- Locked TypeScript adapter build; 59 process tests passed, one opt-in raster
  test skipped.
- Python discovery: 36 tests passed, including resource integrity and frozen
  trial-input/budget regressions.
- Actual MCP smoke using explicit private Node and rebuilt binary: configured,
  absent, stale-release/source and unknown-job cases passed.
- Video source-policy regression suite, catalog refresh dry-run and diff checks.

Exact commands, runtimes, durations and logs are in `final-gates/results.json`
under the private evidence root. The final rebuilt binary SHA-256 is
`bc3c3c7532c74db25301ce3f956daef1304c7da1eefdabed99f9ab468221c0c6`.
An earlier broad Rust run exposed missing resources in the release/installation
fixtures; those paths were fixed before the successful final run. The first
successful full gate also predates the final guidance clarifications, so the
final frozen-source gate was run again, including after synchronizing the
review-document provenance hashes. Native decoded-video controls and
frozen-film MCP refusals have their own recorded binary/runtime identities and
are not relabelled as executions of this final metadata rebuild.

No Windows qualification, macOS minimum-version qualification or waived Codex
Desktop GUI check is claimed.

Remaining limits are concrete: backend model/reasoning identity and inherited
skill contamination prevent a clean causal comparison; no blind scored review
or author repair was run within the frozen comparison; the authored video
specimens have no exports; final font/scratch clarifications have static and
regression evidence but no post-change model trial. Broader catalog acceptance
remains planned. There is no product decision blocking the local implementation.

The final local commit and changed-file inventory are provided with the handoff.
No push or release publication was performed.

## Changed files

The complete local change inventory relative to the adoption base is:

```text
docs/deep-review-20261027.md
docs/design-adoption-20261027.md
docs/design-adoption-trials-20261027.md
docs/design-adoption-video-20261027.md
docs/research-20261027-design-stack.md
docs/research-20261027-hallmark-taste.md
docs/research-20261027-hyperframes.md
docs/research-20261027-motion-vector.md
planning/references.md
planning/reviews/2026-10-27/reference-audit.json
planning/reviews/2026-10-27/reference-fetches.json
planning/reviews/2026-10-27/skill-reference-audit.md
planning/reviews/2026-10-27/source-design-adoption.md
planning/reviews/2026-10-27/source-visual.md
planning/skill-catalog-design.md
scripts/assemble-skills.py
scripts/fetch-skill-reference-evidence.py
scripts/refresh-skill-catalog.py
skills/PROVENANCE.md
skills/art-direction/SKILL.md
skills/catalog.json
skills/design-critique/SKILL.md
skills/interface-composition/SKILL.md
skills/motion-direction/SKILL.md
skills/motion-direction/references/storyboard-and-transitions.md
skills/scroll-motion/SKILL.md
skills/ui-motion/SKILL.md
skills/vector-art/SKILL.md
skills/vector-art/references/illustration-craft.md
skills/video-production/SKILL.md
skills/video-production/references/hyperframes-0.8.197-authoring.md
src/distribution.rs
src/installation.rs
src/native_skills.rs
src/skill_sync.rs
tests/design-adoption-trials.py
tests/design-adoption-video-runtime.mjs
tests/design-adoption-video.mjs
tests/test_assemble_skills.py
tests/test_design_adoption_trials.py
tests/test_refresh_skill_catalog.py
```
