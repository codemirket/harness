# Capability review — second pass

Reviewed 2026-10-24 against the six files frozen in this directory's `baseline.json`. Independent Capability Reviewer; requested model/effort: Astra High. Planning and source review only.

## Verdict

**Ready for complete-plan agreement from the capability perspective, with one small evaluation-method correction below. No remaining P0/P1 capability finding.** The revised plan preserves the full 0.0.1 scope and now gives host agents usable contracts for discovering guidance, operating tools, expanding evidence, resuming work and inspecting saved results. It does not need another framework, runtime, guide family or catalog expansion.

The remaining P2 concerns what will count as comparative guide evidence. It can be corrected in M4's paragraph without reopening architecture or blocking unrelated qualification. Runtime readiness, client activation, output fidelity and quality gains remain untested.

## Remaining finding

### C2-1 — P2: Preserve comparison controls before accepting guide quality claims

**Location:** `planning/implementation-plan.md:163`; related `planning/proposal.md:358` and `planning/platform-qualification.md:103`.

The revised M4 correctly separates activation from output quality, compares the shared workflow with and without a guide, includes held-out and mixed tasks, and inspects correctness, calls/questions, bytes and time. It does not carry forward the first review's clean-session/configuration controls, independent outcome assessment, or repeated variable cases. Recording client versions in M0 does not identify which host/model/settings/tools/budget were used in each later comparison.

**Realistic failure:** an author tunes on a case, runs the candidate in a context that already contains its solution, and compares that result with a fresh baseline or a different model/tool configuration. Alternatively, one lucky guide run beats one unlucky baseline run. Both can satisfy the current M4 wording while producing no reliable evidence that the guide helped. A self-assessment can further reward a polished but analytically wrong output. This is an evaluation-invariant gap, not a request to design report schemas during planning.

**Smallest correction:** add a short requirement to M4: run matched comparisons in clean isolated contexts with the same task inputs, host/model settings, tool access and budget; record available configuration identities and report opaque provider versions as unknown. Freeze acceptance criteria before held-out runs, use deterministic checks or independent artifact/decision assessment rather than worker self-report, retain failed trials, and repeat cases where observed variability could change promotion. Keep exploratory case design separate from held-out acceptance. Do not require a new judge service, paid calls, fixed universal repetition count, or a new framework.

**Discriminating acceptance:** a deliberately contaminated candidate context is excluded from the comparison; a candidate that wins only one variable trial cannot be described as reliably better. A polished analysis that treats repeated observations as independent fails the artifact-level criterion even if the guide loaded and its worker reports success.

**Evidence:** the selected [Agent Skills output-evaluation guide](https://agentskills.io/skill-creation/evaluating-skills) explicitly calls for clean contexts and same-prompt comparisons; its [description-evaluation guide](https://agentskills.io/skill-creation/optimizing-descriptions) explains repeated runs for nondeterministic activation. These source observations support the correction, but their example thresholds, scripts and provider calls are not adopted. Independent assessment and matched configuration are also consistent with the local AI-system-evaluation guidance used for this review. No guide-quality experiment was run.

## First-pass closure

The current plan was assessed before reading the earlier capability report and synthesis. Their source observations were then reused only for closure and unchanged provenance; no peer second-pass report was read.

| First-pass finding | Second-pass disposition and current evidence |
| --- | --- |
| C1 — Guide/tool entry contract | **Closed in planning.** `proposal.md:215` defines canonical portable bodies, exact IDs, thin host entries, release/revision matching, collision/drift handling and explicit stale-entry behavior. `implementation-plan.md:112` exercises thin Engineering/Documents journeys before expansion; `:128` assigns the operation inventory to M1. Exact schemas and qualified host mechanisms are correctly deferred. |
| C2 — Evidence expansion and incomplete retrieval | **Closed in planning.** `proposal.md:237` identifies source/revision/range, bounded expansion, changed/unavailable evidence, incomplete indexes and budget truncation. Applicable constraints bypass semantic ranking; unclassified/overflow coverage is visible. `implementation-plan.md:136` includes the decisive outside-excerpt exception, changed sources, overflow, answerability and downstream decisions. |
| C3 — Agent-usable browser | **Closed in planning.** `proposal.md:233` binds actions to owner/session/tab/frame/revision and covers structured observations, screenshots, stale targets and action outcomes. Dialog/upload/download/new-tab qualification is conditional on advertised operations. `implementation-plan.md:120` moves stale interaction into M0b; `:140` completes engine and visible/headless coverage. No second browser-control stack is needed. |
| C4 — Comparative guide evaluation | **Substantially closed; C2-1 remains.** `implementation-plan.md:163` and `proposal.md:358` now require baseline, trigger/nontrigger, missing-tool, mixed-task and held-out evaluation. The initial two or three cases are explicitly a starting sample, not a release coverage waiver. Preserve the comparison controls above. |
| C5 — Technique-level reuse and rights | **Closed in planning.** `references.md:119–125`, `proposal.md:358` and `implementation-plan.md:146–163` require exact portion/revision, reuse basis, removed assumptions and acceptance cases. Restricted DOCX material is link-only; source selection does not license a whole repository or adopt its helpers. Section mappings can be completed during authoring before adaptation. |
| C6 — Useful continuation | **Closed in planning.** `proposal.md:239` specifies objective, constraints, task/scope, evidence pointers, completed work, questions, next actions and unknown effects, with observations/inferences/proposals separated. `:217` avoids inferred host-thread identity. `implementation-plan.md:136` owns the implementation. A second transcript journal is unnecessary. |
| C7 — Specialist depth | **Closed in planning.** The six authorized focused references strengthen testing/review, retrieval, analysis and accessibility while retaining the twelve guides. `implementation-plan.md:159–163` requires analytical-validity and keyboard/focus checks. Guide bodies must demonstrate the methods during M4; source count alone is not capability. |
| C8 — Tool descriptions/results/errors | **Closed in planning.** `proposal.md:219` requires a finite task-oriented inventory, prerequisites/effect classes and useful errors; `:231` supplies saved-revision inspection routes and coverage limits. `:235` gives the analyzer preparation handoff. `implementation-plan.md:124–130` makes these slice outputs and M1 work rather than an unbounded API-design prerequisite. |

## Focused primary-source verification

All six new method entries were checked against primary bodies on 2026-10-24. Source-observed statements below establish method content and declared terms, not tool behavior, scientific correctness of every example, or a blanket reuse clearance.

| Selected reference | Observed content and appropriate boundary |
| --- | --- |
| Agent Skills | The [specification](https://agentskills.io/specification) defines frontmatter/body/reference disclosure. The two evaluation pages distinguish activation from delivered output. This supports portable authoring, not a guarantee of either desktop host's loader or permissions. The plan correctly requires actual M0 delivery tests and terms review before copying. |
| Trail of Bits, `196a9073ab10…` | [Property testing](https://github.com/trailofbits/skills/blob/196a9073ab10d35bacf111f945e02c00b2c6883a/plugins/property-based-testing/skills/property-based-testing/SKILL.md) distinguishes meaningful invariants/oracles from tautological or vacuous tests. [Differential review](https://github.com/trailofbits/skills/blob/196a9073ab10d35bacf111f945e02c00b2c6883a/plugins/differential-review/skills/differential-review/SKILL.md) has useful evidence/context methods but also mandatory reporting, risk heuristics and dispatch machinery. The pinned [license](https://github.com/trailofbits/skills/blob/196a9073ab10d35bacf111f945e02c00b2c6883a/LICENSE) is CC-BY-SA-4.0. The plan's link-only/independent-synthesis default and explicit compatible treatment for adaptations are appropriate; do not import its whole workflow. |
| QMD, `86d2365d17fc…` | The [skill body](https://github.com/tobi/qmd/blob/86d2365d17fc682e491c2c53d1e4a884aee8682a/skills/qmd/SKILL.md) declares MIT and separates candidate search from attributable source retrieval. It also contains runtime/install/model assumptions that Mirket has not selected. Borrowing the bounded search/read interaction does not justify replacing SQLite memory or adding model-driven query expansion. |
| Sentry, `d12f250460ff…` | [Skill scanner](https://github.com/getsentry/skills/blob/d12f250460ffd961f549dbebf7eeb0cf4afd4c2e/skills/skill-scanner/SKILL.md) distinguishes malicious behavior from discussion of threats, and inspects scripts/support files and lasting effects. [Find bugs](https://github.com/getsentry/skills/blob/d12f250460ffd961f549dbebf7eeb0cf4afd4c2e/skills/find-bugs/SKILL.md) requires contextual verification but includes broad per-file checklists and remote-default-branch assumptions. The pinned [license](https://github.com/getsentry/skills/blob/d12f250460ffd961f549dbebf7eeb0cf4afd4c2e/LICENSE) is Apache-2.0. Select techniques; no scanner, `uv`, exhaustive ritual or remote access is authorized by reference selection. |
| Statistical analysis, `593377bb0487…` | The [selected body](https://github.com/K-Dense-AI/scientific-agent-skills/blob/593377bb048752abb0152deab9fbb1083a6087df/skills/statistical-analysis/SKILL.md) declares MIT and explicitly covers target effects, sampling units, dependence, missingness, uncertainty and multiplicity. It also prescribes a scientific environment and helper scripts. The plan properly selects the methods and requires independent example validation. I did not validate its package versions, calculations, all supporting files or the entire catalog; the attempted root `LICENSE` URL returned 404, so the claim remains “MIT-declared,” not a blanket repository license audit. |
| W3C WCAG | The [2.2 quick reference](https://www.w3.org/WAI/WCAG22/quickref/) links criteria and techniques for keyboard access, focus, labels, reflow and more. This is a sound primary basis for criterion-linked checks, not a promise that a screenshot or accessibility tree proves conformance. No new runtime is implied. |

The retained [Anthropic DOCX license](https://github.com/anthropics/skills/blob/683bc88e56f3e09ba94f7055977f3d3aa499f202/skills/docx/LICENSE.txt) was rechecked: it expressly restricts derivatives and distribution. The revised link-only rule accurately addresses that source-observed restriction. No restricted text was adapted here.

The source-to-guide strategy is sufficient to begin authoring: twelve bounded owners, short entries, focused support references, one shared workflow, and section-specific acceptance/rights records. It intentionally does not require reading every file in 58 selected entries. The author must inspect the exact portions actually used and avoid turning reference helpers, scientific environments, competing agent machinery or branded assets into accidental dependencies.

## Scope and integrity

- At entry, all six frozen SHA-256 hashes matched `baseline.json`. They were checked again after writing this report and remained unchanged.
- Reviewed the implementation plan and reference catalog, the capability/guide/memory/ownership contracts and accepted matrices in proposal/decisions, and relevant qualification/toolchain sections. This is not a second full installer, native dependency or analyzer feasibility audit.
- Used the local skill-catalog, research-and-synthesis and AI-system-evaluation guidance. No recursive delegation or peer-report convergence was used.
- Only this report was written. No implementation, installation, upstream helper, engine invocation, client-setting change, commit, publication or paid model experiment was performed.
- Exact operation fields, guide support-file organization, host-native loader integration and test-report formats are assigned implementation work, not additional planning blockers. Complete-plan agreement and existing dependency/access gates remain necessary before that work.

**Baseline integrity: six of six matched. Planning readiness does not establish runtime readiness.**

## Post-review closure — 2026-10-24

**C2-1 is closed in planning.** The final `planning/implementation-plan.md:163` now requires clean isolated and matched comparisons, recorded configuration identities with opaque versions marked unknown, frozen held-out criteria, retained failures, deterministic or independent outcome assessment, variability-sensitive repetition, and exclusion of contaminated/lucky-trial claims. It keeps exploratory tuning separate and adds no judge service or universal repetition quota. Line 165 preserves the model-call/helper authorization boundary.

This was a focused correction check, not a renewed audit of other reviewers' refinements. All six current files match `baseline.json`'s `final_files` hashes; all six original frozen inputs under `/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-review-2/reviewed` match its original `files` hashes. The earlier integrity statements describe the original review point; this section records the later authorized amendment.

**Final capability verdict: ready for complete-plan agreement, with no open capability findings.** Actual guide quality and runtime readiness still require the planned implementation and evaluation evidence. Only this closure was appended; no implementation or runtime work was performed.
