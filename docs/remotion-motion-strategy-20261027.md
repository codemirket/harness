# Remotion, motion quality and Codex model control

Research date: 2026-10-27. Repository baseline:
`2543bc4854d7a3bb9fe680de0eccb6799d1f4be0`. This is a research finding and
proposed strategy, not renderer admission, a qualified migration, or a new
model-quality result. The approved implementation plan still governs delivery.

Final user direction, 2026-10-27: **HyperFrames only.** Remotion adoption,
comparator trials, host-runtime integration and migration work are cancelled.
The Remotion research below and the [rights inventory](remotion-comparator-admission-20261027.md)
remain historical evidence. The active work is to improve author-visible
feedback and independently verified output using HyperFrames. Redistribution
and permitted host-acquisition requirements still apply to its dependency closure.

The current implementation does not establish the best authoring architecture.
Equally, its recent failures
do not establish a HyperFrames defect or an inferior model: the frozen pilot
explicitly used Astra/high, restricted authoring to 120 seconds, and provided
the author no preview/render feedback.

## What the local evidence actually says

The [24-cell pilot](video-authoring-pilot-20261027.md) selected
`--model gpt-6-astra -c model_reasoning_effort="high"` in every cell. This is
verified configuration, not provider-side model identity; the retained JSONL
does not supply that identity. Earlier trials do not establish their model and
cannot support a weaker-model explanation.

| Observation | Consequence | What it cannot establish |
| --- | --- | --- |
| Baseline completed 9/12; diagnostics/seed treatment 8/12 | No demonstrated creative uplift from that bundle | That diagnostics harmed quality; several variables changed together |
| Complex Cinder task produced no deliverables in all four cells | Investigate pre-artifact time/context costs | A rendering-engine defect: there were no Cinder frames to render |
| Initial authoring cap 120s; optional repair cap 60s | A high-reasoning creative workflow may exhaust time before construction | That more effort, or more time alone, fixes the task |
| Authoring had no Mirket runtime, browser preview or renderer | The skill's requested feedback loop could not execute | A complete end-to-end evaluation of the skill |
| Fifteen available sources later exported and decoded successfully | Existing finite export route worked for those specimens | Design quality, full playback, or parity for missing specimens |
| Seventy-five decoded PNGs were inspected after authoring | Saved-pixel evidence exists | That the author saw those pixels and repaired from them |

See the [independent export evidence](video-authoring-pilot-exports-20261027.md).
The previous incorrect clipping finding was retracted after reopening the exact
frame. Keep that correction: independent review itself needs an observable oracle.
No claim here infers missing visual inspection from absent CLI image events.

## What Remotion does well

The inspected release is [v4.0.539](https://github.com/remotion-dev/remotion/releases/tag/v4.0.539),
dated 2026-10-26, commit
`e50b41a410933aa807515335d1c04dbba1b8b93f`. The architecture observations below
are from that immutable source and current documentation, not a locally executed
Remotion benchmark.

| Mechanism | Why it helps an agent | Transfer to this harness |
| --- | --- | --- |
| Composition with dimensions, fps, frame count and input props | The output has explicit boundaries and a reusable input contract | Freeze those inputs before authoring; keep content editable |
| Zero-based frame clock and sequence-local frames | A scene can be evaluated at a requested time without playing from the start | Name beats with absolute frames and local offsets; define rounding and end-exclusive ranges |
| Frame-derived interpolation and springs | Motion is inspectable mathematics rather than elapsed wall-clock state | One timing owner per property; independently test repeated/backward seeking |
| Studio/Player tied to the composition | Author can inspect a working sequence early and make precise revisions | Put preview and saved frame feedback inside the authoring loop |
| Explicit resource readiness and cancellation | A loaded page is distinguishable from a frame whose fonts/media are ready | Report source checks, readiness, capture and export as separate evidence states |
| Concurrent frame rendering | Efficient exports, but forces order-independent visual state | Test random-order and repeated samples rather than only a sequential playback |
| Focused create/markup/studio/render/caption skills | Load the relevant operation rather than a large universal recipe | Keep project skills small and make exact producer references available on demand |

The relevant source includes
[`useCurrentFrame`](https://github.com/remotion-dev/remotion/blob/e50b41a/packages/core/src/use-current-frame.ts),
[`Sequence`](https://github.com/remotion-dev/remotion/blob/e50b41a/packages/core/src/Sequence.tsx),
[`interpolate`](https://github.com/remotion-dev/remotion/blob/e50b41a/packages/core/src/interpolate.ts),
[`spring`](https://github.com/remotion-dev/remotion/blob/e50b41a/packages/core/src/spring/index.ts),
[`delayRender`](https://github.com/remotion-dev/remotion/blob/e50b41a/packages/core/src/delay-render.ts),
[`seekToFrame`](https://github.com/remotion-dev/remotion/blob/e50b41a/packages/renderer/src/seek-to-frame.ts),
and the renderer's [page/frame pool](https://github.com/remotion-dev/remotion/blob/e50b41a/packages/renderer/src/render-frames.ts).
The seeking implementation waits for render readiness and font readiness;
parallel capture makes arbitrary mutable animation history unsafe.

Three distinctions matter:

1. **Frame purity is a producer-specific implementation contract.** Remotion's
   [flicker guidance](https://www.remotion.dev/docs/flickering) rejects dependence
   on wall time or sequential frame order. Do not copy a ban on CSS animation into
   HyperFrames indiscriminately: its WAAPI/Anime registration and
   `data-no-timeline` contract are different. Test the same desired outcome on
   each producer.
2. **Preview is not export proof.** Player, Studio and server rendering share
   composition semantics, but asset loading, codecs, browsers and concurrency
   differ. Match named preview frames to decoded exports; retain full-timeline,
   audio and playback limits separately. See [Player](https://www.remotion.dev/docs/player/player)
   and [rendering](https://www.remotion.dev/docs/render).
3. **The library supplies structure, not art direction.** Layout, type, reference
   choice, shot rhythm, camera intent and editorial restraint still require
   judgment. Retain Hallmark/Taste and the current design references. Use
   licensed or independently authored shot examples as optional starting points,
   without turning them into a compulsory visual style.

The [v4.0.539 release notes](https://github.com/remotion-dev/remotion/releases/tag/v4.0.539)
specifically list spring overshoot clamping, multi-keyframe interpolation,
Player seeking and a concurrent-render audio deadlock fix. Remotion is
established, but not an oracle free of bugs. Its
[core license](https://github.com/remotion-dev/remotion/blob/e50b41a/packages/core/LICENSE.md)
is a custom license with commercial eligibility conditions, not unrestricted
MIT/Apache licensing. The external skills repository's inspected license URLs
returned 404; the [earlier same-day rights review](research-20261027-remotion.md)
also records `license: null` from its repository API and no license/notice file
in the inspected tree. That is insufficient permission to redistribute its prose. No
upstream code, skill text, assets or dependency was adopted in this research.

## Does Claude outperform Codex at motion graphics?

The reviewed evidence does not establish a general ranking. It does establish
that polished examples frequently combine a model with specialized skills,
reference assets, shot recipes, preview, frame inspection and repeated repair.
Those variables must be controlled before attributing a result to the model.
Remotion itself provides [plugins for both Codex and Claude Code](https://www.remotion.dev/docs/ai/plugins)
with its agent skills.

Recent practitioner observations point in different directions:

| Source/date | Observed report | Evidential limit |
| --- | --- | --- |
| [Codex vs Claude motion question](https://www.reddit.com/r/codex/comments/1wue2vp/how_to_get_codex_to_make_motion_graphic_videos_as/), 2026-09-30 | Author favored Claude; Codex Astra was later switched from HTML/CSS/JS to Remotion | Engine changed; missing exact Claude model, shared assets, budgets and independent scores |
| [Direction versus layout report](https://www.reddit.com/r/codex/comments/1uovwrs/codex_creates_better_remotion_videos_but_claude/), 2026-07-06 | Author favored Codex for direction and Claude for responsive layout | One uncontrolled workflow |
| [Same-reference recreation](https://www.reddit.com/r/ClaudeCode/comments/1tech8y/codex_vs_claude_code_who_recreated_higgsfields/), 2026-05-15 | Author leaned Codex on the same recreation challenge | No exact budgets, versions, rubric or independent scoring |
| [Authenticated X discussion](https://x.com/NIxinhai0/status/2114998031837774011), 2026-10-27 | Author describes assembling storyboarding/motion/QA resources for Codex; replies praise Claude too | Read full post and replies; no matched exports or scoring. 95 likes/130 bookmarks at observation indicate interest only |
| [Opus/Sonnet motion study](https://github.com/JakeB-5/motion-graphic-skill), observed 2026-10-27 | Retains critique rounds and reports 41–52 minutes for a 30-second film | Its separate Codex example uses a different brief; not a host comparison |

Useful implementation references include
[claude-motion](https://github.com/whaleyxbt/claude-motion) (MIT; 688 stars/91
forks at research time),
[video-shotcraft](https://github.com/Vincentwei1021/video-shotcraft) (Apache-2.0;
14.0k stars/1.3k forks), and [demovie](https://github.com/enszrlu/demovie) (MIT).
Their cross-host approaches make shot reuse, real app capture and iterative QA
worth evaluating. Popularity and showcase claims are not comparative quality
scores. No assets or implementation were imported.

Further counterchecks included
[FreeMotion's feedback-friction discussion](https://news.ycombinator.com/item?id=46732672),
[a motion benchmark discussion](https://news.ycombinator.com/item?id=50011813),
[a React-video production account](https://news.ycombinator.com/item?id=46374391),
[aeiadev/motion](https://github.com/aeiadev/motion), and
[a diagnosis-focused Remotion skill](https://github.com/fernandokaraka/remotion-motion-graphics-skill).
None supplies a sufficient matched Codex/Claude comparison.

The leading local hypothesis is **inadequate authoring feedback and an
unrepresentative time budget**, followed by excessive setup/context cost. It is
not yet a demonstrated explanation for all weak motion. OpenAI's
[2026-09-11 Astra guidance](https://developers.openai.com/blog/rethinking-skills-and-prompts-for-gpt-6-astra)
also advises revisiting accumulated instructions and narrowly triggering skills.
That supports testing shorter context; it does not prove that our prose caused
the observed timeouts.

## Model and effort controls: what is possible

The inspected Desktop-bundled executable is
`/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex`,
version `0.164.0-alpha.11.1`, SHA-256
`a04752e4d46a4d89e53e6f61c59c5349a638c8ce172ffbf391fd482b4d902a4e`.
The separate PATH executable under `.nvm/versions/node/v24.18.1/bin/codex`
reports `0.146.0`; do not substitute it in qualification commands.
The Desktop-bundled executable's model catalog
advertises Astra with text/image inputs, default effort `low`, and
`low/medium/high/xhigh/max/ultra` options. Availability advertised by this local
catalog is not an account entitlement check or backend execution receipt.
The [API model page](https://developers.openai.com/api/docs/models/gpt-6-astra)
does not advertise native video input. Decoded frames can support visual review;
they do not prove uninterrupted viewing, audio judgment or motion smoothness.

Recommended policy: explicitly select **Astra/high for creative lead and
comparable reviewer**, then evaluate xhigh as a separate treatment if high
produces a concrete unresolved judgment defect. Do not silently choose a smaller
creative delegate. Do not assume raising effort solves missing tools or short
wall caps. Mechanical work can remain separate. Explicit user model/effort
choices take precedence over this recommendation.

| Control | Real scope | Limit |
| --- | --- | --- |
| Explicit root model and effort | CLI supports model and config flags; Desktop owns its selector | Does not retroactively change this conversation or prove backend identity |
| Named native agent TOML with both model and effort | Current docs say role-file values take precedence when that role is selected | Does not guarantee that every creative task selects that role |
| Default subagent model/effort | Reduces accidental weak defaults | Explicit spawn values can override defaults |
| Host `PreToolUse` guard for `spawn_agent`/`Agent` | Current documented local path can deny/rewrite arguments | Specialized paths can opt out; this Desktop collaboration surface has not been qualified |
| `SubagentStart` hook | Adds context/observability | `continue: false` does not prevent the subagent from starting |
| Mirket skill/guidance | Communicates selection, budget and evidence rules | Cannot change the native model or enforce host spawning |

Sources: [native subagent configuration](https://learn.chatgpt.com/docs/agent-configuration/subagents),
[configuration layers](https://learn.chatgpt.com/docs/config-file/config-advanced),
and [hook coverage and limitations](https://learn.chatgpt.com/docs/hooks).
Host-native guardrails preserve the accepted boundary; a Mirket model router
or second orchestrator would not. Do not label hook documentation as an observed
Desktop enforcement result.

Illustrative native role, **not installed or runtime-qualified**:

```toml
name = "motion-reviewer"
description = "Review saved motion artifacts against the frozen brief and rubric."
model = "gpt-6-astra"
model_reasoning_effort = "high"
developer_instructions = """
Review only the assigned project's saved artifacts. Do not edit production
files. Cite exact frames and paths for findings. Distinguish still-frame
inspection from playback and audio evidence. Use the lead's remaining shared
review budget; do not create a new budget or delegate. Return remaining defects
and uncertainty to the lead for integration.
"""
```

The current official documentation and inspected `0.164.0-alpha.11.1` CLI help
describe a named profile as a separate
`motion.config.toml` under the selected Codex configuration home, with top-level
`model` and `model_reasoning_effort`; the old `[profiles.motion]` form is obsolete.
Project and explicit CLI settings can override a profile. Do not install a
global profile to force unrelated tasks onto Astra. Both native role selection
and explicit user overrides need a disposable-host test before activation.

## Proposed strategy and implementation order

The active work improves the HyperFrames authoring and feedback workflow.
There is no engine comparison or replacement track.

1. **Make the existing feedback loop executable.** The motion/video skills
   already require a short specimen, preview correspondence and decoded output
   QA. Supply the author with the qualified runtime and saved-pixel tools during
   its turn, or label the run source-only. Separate time spent on environment
   readiness from creative authoring. Do not add more mandatory prose to repair
   a missing-tool environment.
2. **Use a minimal, editable composition and beat manifest.** Keep the selected
   project source, assets, fonts, aspect ratio and fps; list opening, anticipation,
   transition midpoint, hold, settle and ending samples. Bind every capture to
   its source revision and frame. Inspect a short segment before expanding;
   verify asset/font application, not just declaration. This can use current
   project files and tools; no new Mirket DSL or browser is needed.
3. **Close the visual loop before final export.** Save initial specimen,
   independent findings, targeted repair and final specimen. Recheck the same
   samples and adjacent transition frames after repair, then check decoded
   exports. Timing, audio and full playback retain their own evidence labels.
   Lead owns integration; one task-level budget spans all specialists.
4. **Qualify native model controls separately.** Test the named role with an
   attempted weaker override, explicit user choice, unavailable Astra, inherited
   effort and approval denial. Test any hook on the actual Desktop spawn path.
   Never silently fall back or bypass denial. Until then report configured,
   host-observed and provider-confirmed model identities separately.
5. **Measure HyperFrames outcomes with equivalent task inputs.** Freeze the
   brief, assets, Astra setting, CLI/config/runtime identities and total budget.
   Compare source-only authoring with author-visible preview and QA on the same
   engine. Measure edit precision, time to accepted output and reproducibility;
   do not attribute an improvement to the model or renderer without evidence.

The cancelled Remotion private inventory installed an exact lock with
scripts disabled before the redistribution stop, but performed no Remotion
bundle or render. Those packages remain outside the repository and release;
the [inventory](remotion-comparator-admission-20261027.md) records these effects.

## Finite evaluation proposal

Do not resume or reset the completed pilot. Freeze a separately identified
experiment before any new model calls. Suggested first stage: three held-out
briefs (kinetic type, editable vector choreography, real-product UI sequence),
two repetitions, Astra/high, current source-only setup versus the same qualified
producer with author-visible preview/QA. Use the existing 45-minute task budget
and four-repair ceiling across lead/reviewer, with one targeted repair as the
normal path. Record actual time/usage; do not interpret a timeout as poor taste.

Only after that isolates the feedback effect, compare high versus xhigh on
HyperFrames if warranted. A Claude
comparison needs an exact available Claude model/client and the same brief,
assets, renderer, skill content, feedback and budget; no access or result is
assumed. These are proposed cells, not acceptance evidence. Freeze their total
budget in the new experiment manifest; the completed pilot remains unchanged.

Use blinded saved-artifact review where feasible. Score composition/type,
readability, originality relative to brief, motion hierarchy, continuity,
timing/holds, editability and export fidelity separately. Retain failure rate
and time per accepted artifact. Include backward/repeated seeking, stale source,
missing fonts, wrong-project output, false verification, unnecessary repair,
budget-reset attempts and regressions. A source check, render receipt or model
self-rating cannot pass the visual rubric. Reuse already measured outcomes
where valid; expand only on a shared new failure.

## Scope, evidence and remaining limits

Research covered immutable Remotion source, current official Remotion/OpenAI
documentation, GitHub implementations, Reddit, HN and authenticated X. Dated
discussion/release evidence is within 2025-04-27–2026-10-27; undated/current
documentation and repository observations are labeled as such. X was read
without posting; an article card that did not open was not used as full-article
evidence. Popularity counts are point-in-time discovery signals only.

Three independent read-only streams covered Remotion implementation, community
comparisons and the local pilot; the lead checked decisive claims and current
host controls. Detailed working ledgers remain under
`/Users/nazmiyilmaz/.codex/task-state/harness-remotion-research-20261027/`.

This change adds research/proposal documentation only. No runtime, catalog,
provenance, installed skill, live Codex setting or `~/.mirket` file changed.
No new model comparison, Remotion render, visual inspection of a new artifact,
hook enforcement test, Windows qualification or Desktop GUI test occurred.
The existing Desktop GUI waiver remains untested. Runtime build/test gates are
not rerun for a documentation-only change, and prior test results are not
represented as validation of the proposed workflow. No new commit or release
is created by this research.
