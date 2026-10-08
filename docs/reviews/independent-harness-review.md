# Independent harness review — 2026-10-08

**Astra approves the final snapshot with limits. Opus approved R2 with limits;
its final reassessment is blocked by the Claude session limit. Agreement on the
final revision has not yet been obtained.** The user requested a ready handoff,
with no scheduled follow-up.

| Reviewer | Model and effort | Latest completed opinion | Final-revision status |
| --- | --- | --- | --- |
| Astra | `gpt-6-astra`, `ultra` | [R5: APPROVE_WITH_LIMITS](independent-harness/astra-r5.md) | No remaining concrete issue identified; R4 includes the final code review. |
| Claude Code CLI | `claude-opus-5-5`, `max` | [R2: APPROVE_WITH_LIMITS](independent-harness/opus-r2.md) | Final R4 attempts returned no accepted opinion; pending on the same implementation plus R5's README correction. |

The official CLI reports: “You've hit your session limit · resets 7:30pm
(Europe/Istanbul).” This was observed on 2026-10-08; a reset time is not a guarantee
of later availability. [Ready continuation instructions](independent-harness-handoff.md)
retain the exact model/effort and reviewer isolation.
The reviewed implementation is snapshot
`4f0e8ec6540686a1a95978b03897ef0b4c1d2efe842d78bfc6051b26428362b2`
(202 source files, including uncommitted work based on `5fa23c4d6f8204dd0ee84ef20d15bac841e9df2d`).

## Method

Two separate review contexts received identical immutable source and acceptance
criteria. Each received only its own previous findings. Claude Opus 5.5 used the
official Claude Code CLI at explicit `max` effort; Astra used a native Codex worker
configured as `gpt-6-astra` at `ultra`. No substitute model or billing fallback was
used. CLI model metadata and native dispatch settings establish the reported or
configured model identity; they are not cryptographic attestation.

The reviewers inspected code, tests, routing, substantive skills and evidence.
They did not execute project tests or domain work. Primary execution evidence was
labelled separately. Claude had Read/Glob/Grep only, with command execution, writes,
MCP, web tools, user/project hooks, skill invocation and recursive agents disabled
by the bounded runner controls. Administrator-managed policy still applies. Astra had a read-only assignment and separate review scope. This is context
and workspace isolation, not an operating-system sandbox.

The initial non-maximum attempts were cancelled and do not count. A later Opus
pass was cancelled before accepting any output when another diagnostic repair
became necessary. Two later Opus final-pass attempts exited unsuccessfully; a separate bounded
diagnostic confirmed the session limit. Those are failures, not review opinions.
Only completed opinions are recorded as reviews. No reviewer was
required to approve or shown the other's opinion to manufacture agreement.

[Acceptance criteria](independent-harness/acceptance-criteria.md) and original
capability matrices: [Astra R1](independent-harness/astra-r1.md),
[Opus R1](independent-harness/opus-r1.md). Follow-up reports:
[Astra R2](independent-harness/astra-r2.md), [R3](independent-harness/astra-r3.md),
[R4](independent-harness/astra-r4.md), [R5](independent-harness/astra-r5.md),
[Opus R2](independent-harness/opus-r2.md). Historical findings are superseded only
by explicit later dispositions; no final Opus report exists yet.

## Repairs

- The normal foundation now supplies four project specialists alongside the 14
  globals. The original 12 fundamentals remain an explicit portable profile.
  This avoids eight duplicated default names while preserving absent-global use.
- Specialist selections warn about configured global-name overlaps. Plan, sync
  and doctor report preserved unselected copies, including unknown, malformed,
  linked and unreadable paths. They do not delete them or claim cleanup occurred.
- Root and per-copy diagnostic filesystem errors and overcomplex JSON are bounded.
  Existing selected-copy and name-collision preflight remain strict.
- Role search covers designer, documenter, tester, planner, debugger, marketer,
  illustrator, animator and researcher. Authored descriptions match frontmatter;
  documented invocation/profile names have consistency checks.
- Project readiness is based on requested project checks, with client readiness
  separately reported. Explicit authentication failures still fail when requested.
- Evaluation summaries expose reviewer kind and identity, keeping agent and human
  acceptance distinct. Original installation provenance remains separate from the
  current catalog selection when installed content is unchanged.
- Unpinned global exports exclude Python/Finder residue without weakening reviewed
  payload contracts. Claude delegation supports validated explicit effort levels.
- Current evidence is durable and fingerprinted; old counts and results remain
  explicitly historical.

The prior archive/trust/context hardening was retained and reviewed. No new default
runtime, service, MCP server or production dependency was added.

## Verification

[Execution record](../evidence/independent-harness-verification-2026-10-08.json):

- 471 automated tests pass in 35.759 seconds.
- 30 normal-plus-specialist and 24 portable copies match authored payloads across
  Codex and Claude. Repeat sync preserves 110 file states.
- Portable-to-normal migration preserves 84 skill-file states and reports 16
  unselected copies. Actual CLI discovery still exposes preserved copies.
- Real non-root POSIX permission and nested-receipt checks exercise diagnostic
  recovery; the selected-target collision negative control still fails safely.
- All 14 eligible archive-mode sources pass full parsing and one original/adapted
  pinned payload each. Reader/source/selected-entry inputs remain applicable.
- Export checks preserve 40 original global payload files and exercise the bundled
  wrapper. Generated documentation, local links, shell syntax and whitespace pass.

## Scope and limits

The review covers routing and guidance for frontend engineering, interface design,
illustration/motion, QA/testing, backend/integration, databases, DevOps, marketing,
research, documentation, planning/product, debugging and analysis. Initial reports
contain each reviewer's capability matrix; later reports amend those findings.

Approval concerns this configuration layer. It does not establish automatic skill
activation, universally excellent output, or production readiness for another
project. Actual desktop/Claude selection, native Windows, physical devices, other
browsers, live providers and recovery, bitmap craft and human visual acceptance
remain unverified. Existing guided domain trials retain their original scope and
fingerprints; no new all-domain quality experiment was run.

Frontend Vue/Nuxt and DevOps worked examples remain thinner than React, visual and
motion guidance. These are sensible future depth improvements, not evidence that
installing more tools by itself will improve design quality. Existing projects
still require deliberate sync, inspection of preserved duplicates, runtime checks
and verification of actual delivered work.

[Review provenance](../evidence/independent-harness-review-2026-10-08.json) records
snapshots, completed reports, configured models, failures and the outstanding step.
These status records were published after freezing the isolated source; they were
not exposed to either reviewer. At completion of this review, the changes were
uncommitted; subsequent publication is recorded by Git history.
