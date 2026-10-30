# Skill catalog review and improvement plan (2026-10-30)

Two independent Astra High reviewers inspected the catalog. One read all 61
bodies, resources, profiles and provenance for content and overlap; the other
traced Pi selection, doctor and tool policy into the pinned Pi 1.4.0 package
manager. The latter reproduced an inherited-skill leak under `autoload:false`
with private Node 24.22.1 and disposable state. The reviewers made no source
edits. GitHub stars, recent commits and social posts are discovery signals,
not local quality evidence. All 305 catalog cases remain planned.

## Repairs in this worktree

| Priority | Finding | Repair | Acceptance check |
| --- | --- | --- | --- |
| P1 | Explicit profile selection under Pi `autoload:false` left globally selected conditional skills active. | Write Pi-native `!skills/**` followed by exact `+skills/...` includes; preserve an existing filter when project init has no selections. | Resolve the effective Pi skill set with both global and project package entries, not only settings JSON. |
| P2 | Required capability fields were validated but did not gate Pi skill loading; the docs promised refusal. | State the actual contract and have doctor report selected missing native routes. Native operations still enforce their own prerequisites. | Select `raster-art` without `design.raster`; doctor reports missing while unrelated routes remain ready. |
| P2 | Browser skills suggested public browsing, forms and normal-motion review through a peer restricted to one local origin, seven tools, GET/HEAD and reduced-motion mode. | Narrow route wording in browser, design, accessibility, UI/motion and video references. Keep broader actions with established project tools. | Local inspection works; public navigation and text entry remain refused; a missing route is reported. |
| P2 | The design profile selected `raster-art` although Sharp is excluded. | Make it conditional and correct its individual route-audit evidence. | Design baseline omits raster; explicit selection exposes the missing route without installing it. |
| P2 | Source-only HyperFrames authoring was hidden by an export-only trigger and a required `video` pairing. | Include source-only authoring, make `video` conditional on export and add a source-only case. | Author a valid root, font, asset and caption handoff without claiming render or playback proof. |
| P3 | Planned cases named retired Codex routes and treated an unsupported SQL dialect as an out-of-domain task. | Use a matched Pi baseline and project SQL/browser routes; replace the SQL negative case with migration execution. | Catalog refresh and case mix validation pass. |
| P3 | One shipped video reference linked outside the package and prescribed a checkout-only checker. | Link retained pilot evidence to an immutable repository commit, label the checker as checkout-only, and use installed `video_prepare` for operation preflight. | Every relative Markdown link in assembled `skills/` resolves within the package. |
| P3 | Skill assembly required exactly 9/52/7 entries, resisting evidence-led retirement. | Validate bounded structure and profile references instead of a fixed count. | A catalog with one legitimately retired unused skill still assembles; extra unlisted bytes still refuse. |

## Source comparison and retention decision

| Source inspected | Useful method or signal | Decision |
| --- | --- | --- |
| [Hallmark](https://github.com/Nutlope/hallmark/tree/4f6a46c6dc325d40ab268c93e25c7b96676ebccd/skills/hallmark), [Taste](https://github.com/Leonxlnx/taste-skill/tree/12af471f627f866411b4bd785b19646d3672784c/skills/taste-skill), [Impeccable](https://github.com/pbakaus/impeccable/tree/02523aec3e4c4580102d996d1b6a71fbe062e03f) | Recently maintained and widely watched design families; structure, brief-led direction and rendered critique already map to Mirket's three design roles. | Keep pinned, focused reference methods. Do not import their launchers, browsers, default aesthetics or project state. A newer commit alone is not a reason to change pins. |
| [Emil animation review](https://github.com/emilkowalski/skills/blob/a81dbca07f2217b7103e510c0f13f8d5b017afc9/skills/review-animations/SKILL.md), [GSAP skills](https://github.com/greensock/gsap-skills) and [LottieFiles motion](https://github.com/LottieFiles/motion-design-skill) | Specific lifecycle, scroll and delivery checks can be compared with current UI/scroll/vector/video guidance. | Keep current focused roles. Verify API advice against the installed project's version and official docs; do not add an engine merely because a skill is popular. |
| [HyperFrames](https://github.com/heygen-com/hyperframes/tree/279d40dcd75e3d074149b18dc5b297608dcf0540/skills) and [OpenPencil](https://github.com/open-pencil/open-pencil/tree/master/skills) | Useful authoring methods already have reviewed, bounded local adaptations and actual runtime contracts. | Keep the pinned local producer and editable-file subsets. Upstream `npx`, installers and broad editor features do not become Mirket routes. |
| [Addy Osmani instrumentation](https://github.com/addyosmani/agent-skills/blob/eb07f19f52c2669f2364897aaa293e2aaddd7b1c/skills/observability-and-instrumentation/SKILL.md) | Current [MIT source](https://github.com/addyosmani/agent-skills/blob/eb07f19f52c2669f2364897aaa293e2aaddd7b1c/LICENSE) gives a focused question-to-signal and induced-failure verification method absent from backend/reliability guidance. | Best next *trial*, not an admitted dependency or automatic catalog addition. Adapt a small optional resource only if local outcomes justify it. |
| [Intent journey](https://github.com/ghaida/intent/blob/7111cb85894feaeb7e98938b1ed34597d40e3b4c/skills/journey/SKILL.md) | [CC0 source](https://github.com/ghaida/intent/blob/7111cb85894feaeb7e98938b1ed34597d40e3b4c/LICENSE) maps entry, branch, interruption and return states that could clarify a missed cross-screen recovery task. | Comparison only until an actual interface/prototype failure calls for it; avoid another broad skill. |
| [Vercel agent skills](https://github.com/vercel-labs/agent-skills/tree/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills) | Popular React and web-guideline locators, but broad optimization defaults and dynamically fetched rules do not fit a framework-neutral, offline-capable catalog. The inspected repository has no clear root license for blanket redistribution. | Link for exact project-specific comparison only; no copied bundle or automatic new dependency. |

The [SkillsBench v4](https://arxiv.org/abs/2602.12670v4) and
[SWE-Skills-Bench v2](https://arxiv.org/abs/2603.15401v2) results differ by
workload and both favor local matched evaluation over a larger catalog by
default. A [Reddit design comparison](https://www.reddit.com/r/ClaudeCode/comments/1syachi/best_skill_for_uxui_impeccable_vs_uxui_pro_max_vs/)
and [Hacker News Impeccable discussion](https://news.ycombinator.com/item?id=46587284)
provide practitioner examples and counterexamples, not controlled ranking.
A [LinkedIn same-brief comparison](https://www.linkedin.com/posts/appic-by-hiflylabs_uxui-skills-activity-7500844776343298049-ru7n)
reports a preference for Hallmark and Impeccable across ten dashboard builds,
while still finding alignment and spacing defects in every skill configuration;
the posted account is an anecdote without Mirket-matched inputs or independent
artifact scoring. A [GitHub Agent Skills discussion](https://github.com/agentskills/agentskills/discussions/304)
raises context cost for thin CLI-wrapper skills, and [maintainer issue reports](https://github.com/addyosmani/agent-skills/issues)
include missing references and activation problems; these motivate our package
link and effective Pi filter checks, but do not prove a new host feature is
needed. Medium results did not add an inspectable outcome study. X did not
yield enough inspectable evidence for a sentiment claim. No rights conclusion
is inferred from stars or repository popularity.

## Next measured improvements

1. **Complete package acceptance.** The [current macOS/ARM64 source, package,
   MCP, Pi CLI/RPC/TUI, image transport and profile/doctor checks](qualification.md#skill-catalog-routing-candidate-private-full-package-2026-10-30)
   have passed with explicit private binaries and disposable state. A separate
   real Video operation and active-process cancellation passed through the
   exact packaged core's development MCP route. An installed Pi Video journey
   also passed with an offline scripted provider, saved output decode, changed
   pixels, refusal and cancellation checks. Model judgment and playback remain
   outside that proof. Other platforms need their own native qualification.
2. **Run focused held-out routing cases.** A [six-pair offline Pi
   trial](/private/tmp/mirket-skill-trials-routing-CLsi82/report.json) passed
   source-only HyperFrames static handoff, unconfigured public research
   refusal, real wide/narrow local browser viewport PNGs, account-form/public
   route refusal, missing raster doctor status and inherited skill exclusion
   under `autoload:false`. Each arm had separate project/settings/session state;
   native cases shared one explicit private installation. The scripted provider
   held inputs and tools constant and captured context byte deltas, but it
   cannot measure model judgment, token benefit or output quality. An actual
   model cohort remains a future acceptance trial; preserve the earlier failed
   trial-harness attempts separately from these passing results.
3. **Trial instrumentation as one optional resource.** A [four-pair exploratory
   trial](/private/tmp/mirket-instrumentation-trial-g9865_st/REPORT.md) compared
   current backend/reliability guidance with a concise adaptation of the pinned
   Addy Osmani method. Both conditions passed retry, origin, cardinality and
   redaction checks, with matched business results and SQLite journals. A
   posthoc missing-code negative control caught one unsupported baseline claim,
   but the primary endpoints tied. No resource or production dependency was
   added. Keep the source/rights receipt and revisit only on an independent
   consequential miss; this non-Pi trial cannot establish installed skill
   utility.
4. **Ablate overlap before pruning.** A [private static and native Pi loader
   audit](/private/tmp/mirket-overlap-ablation-20261030-o9n05syv/REPORT.md)
   found distinct phase/domain safeguards, no name collisions in nine loadouts
   and a 1,483-byte formatted metadata cost for the global trio. Fresh private
   Pi had no available model, so this is not a utility comparison. Retain the
   three globals and backend/frontend/API skills for now. A frozen
   three-family, four-condition trial and independent oracles are ready for a
   later matched model cohort; retire or merge only after repeated consequential
   non-contribution or a harmful route collision. Keep design and motion roles
   distinct while their different outputs remain useful.
5. **Revisit cross-screen journey guidance only on a missed task.** If a real
   interrupted-return or cross-channel flow fails current interface/prototype
   guidance, adapt the smallest journey map into an existing resource and test
   that exact failure. Keep source rights and host/tool scope separate.

No new production dependency, runtime tool, account or upstream installer is
approved by this review. The source review does not establish visual quality,
skill utility, browser capability beyond the admitted peer or a release on an
untested platform.
