# Capability matrix review (2026-10-27)

Updated for v0.0.1 on 2026-10-28: 61 catalog entries (9 global / 52 project)
and seven profiles. All 305 public development cases retain their recorded
`planned-not-run` state; representative native outcomes are reported separately.
The six product/marketing skills and content-growth profile were retired after
their tested bundle failed to deliver within budget. Codex retains those host
workflows. This matrix describes route/output contracts, not universal skill
uplift. See the [first-release record](first-release-qualification-20261028.md).

## Workflow to owner, output, failure probe and evidence

| Workflow / skills | Owner and route | Observable output | Discriminating failure case | Evidence required |
| --- | --- | --- | --- | --- |
| Discovery and web research — `browser-research`, `browser-automation`, `browser-qa`, `content-search`, `evidence-research` | Codex browser/web research and project sources; no Mirket crawler | Dated, bounded source notes, scoped interaction/final state and changed information architecture | Page state differs from fetched text, source is stale, interaction submits twice, or a claim has contradictory primary evidence | Reopen the cited page, record URL/date/section, and separate fact, inference and unknown; retain final-state reconciliation for actions |
| Customer discovery and problem framing — `jobs-to-be-done` with Codex research | Host files and consented research | Segment/job synthesis with observations, alternatives and unknowns | Invented participant evidence, missing consent, or segment conclusion changes when one source is removed | Source inventory, consent/use limits, and an independent reviewer tracing each conclusion |
| Product direction and growth — Codex host workflows (retired bundle) | Host planning, analytics and Codex browser for the live journey | Decision options, defensible claims, revised flow/copy and precommitted measurement/rollback | Traffic mix or measurement defect masquerades as conversion lift; unsupported claim reaches copy; rollback leaves changed exposure | Dated evidence, baseline, changed-input/funnel check and retained experiment result |
| Architecture and planning — `task-planning`, `architecture` | Codex owns plan, source inspection and Git | Dependency sequence, ownership map, alternatives and failure tradeoffs | Plan claims completion without a gate or assigns overlapping ownership | Source-bound plan with observable next action and a reviewer checking each gate |
| API and service implementation — `api-integration`, `backend-engineering`, `code-change` | Host code and project toolchain; Codex owns edits | Validated contract/state transition, consumer-compatible change and tests | Duplicate effect after retry, malformed input, or consumer sees an undocumented error | Consumer probe plus a replay/invalid-input test against the saved revision |
| Client implementation — `frontend-engineering`, `interface-composition` | Host code and Codex browser; Mirket design only when explicitly qualified | Loading/empty/error/success/interrupted UI states rendered at target viewports | State is correct in source but focus, responsive layout or interrupted action fails in browser | Reopened build at required viewports and keyboard/failure journey evidence |
| Persistence and data movement — `database-migrations`, `data-pipelines` | Host DB/project tooling and SQL; no Mirket SQL route | Forward migration or replay preserving invariants, lineage and idempotency | Late/duplicate input, rollback, forgotten row revived from backup, or partial checkpoint | Disposable database/project, changed-input replay and independent invariant query |
| Source navigation — `js-ts-navigation`, `sql-navigation` | JS/TS uses Mirket `code_navigate` only with explicit snapshots/imports; SQL uses Codex/project tools | Revision-bound definitions/references or dialect-aware SQL locations with limits | Stale source, same-name scope, missing schema/import, unsupported language, or false whole-project claim | Original UTF-8/CRLF offsets, stale refusal and an independent source lookup |
| Query and data answers — `sql-analysis`, `data-analysis`, `data-visualization` | Host DB/files and spreadsheet/chart tools | Query/result or chart with grain, units, missingness, source and transformation trail | Null/join/grain error, misleading denominator, or chart changes when source is perturbed | Recompute with independent query/oracle and inspect saved chart/export |
| Performance and reliability — `performance`, `reliability-incident`, `root-cause-debugging` | Host profilers/logs and project runtime | Controlled baseline/cause trace and reconciled recovery/rollback | Warm result hides cold regression, response lost after effect, or replay duplicates work | Platform/build/workload record, failure reproduction and post-restart effect inspection |
| Test design and review — `test-design`, `critical-review`, `outcome-verification` | Codex/project test tools and independent oracle | Invariant/property/differential test, ranked finding, and saved-output verification | Green receipt/hash/verdict while artifact is wrong, stale or outside scope | Reopen consumer artifact, execute convincing negative case and retain remaining defects |
| Security and trust — `threat-model`, `security-review`, `privacy-data` | Host code/configuration and authorized disposable environments | Assets/trust boundaries, reachable findings, lifecycle/deletion evidence | Retrieved text grants authority, path escapes project, secret leaks in logs, or restore revives forgotten data | Negative permission/scope tests, support-file review, backup/restore and external-copy limits |
| Accessibility — `accessibility` | Codex browser, host UI/project tests and WCAG/platform references | Criterion-linked keyboard/focus/label/reflow/reduced-motion findings and repaired journey | Keyboard trap, missing name, overflow at reflow, or automated check passes while focus is unusable | Manual keyboard and viewport journey plus saved screenshots/DOM evidence; no automated-only conformance claim |
| Localization and type — `localization`, `typography` | Host source/design tools and Codex browser | Locale message integrity, bidi/expansion layout and legible specimen | Placeholder/plural loss, RTL order corruption, clipping or unlicensed font use | Locale fixture at target sizes/directions and source-linked font/rights record |
| Design direction and composition — `art-direction`, `interface-composition`, `design-critique` | Codex lead owns creative judgment; host/Codex browser and qualified design operations | Brief/references, real-content states and ranked before/after defects | Reference brand/assets copied, subjective preference called a defect, or only a screenshot is checked | Inspect saved pixels at target viewport and retain initial/final artifact plus independent review |
| Design system and editable design — `design-systems`, `openpencil` | Qualified Mirket tokens/OpenPencil only when runtime status and project scope pass; host owns code | Compiled tokens consumed by a real UI; reopened editable `.fig`, changed component and export | Tokens are dead output, OpenPencil cannot reopen/change, or operation writes outside project | Source hash, reopened file/export, rendered inspection and changed-source result |
| Vector, raster and diagrams — `vector-art`, `raster-art`, `diagrams` | Qualified Mirket SVG/SVGO/resvg, Sharp and D2; Codex verifies meaning | Editable source plus optimized/rendered SVG/raster/diagram with dimensions, alpha and semantic layout | Renderer exits with wrong glyph/alpha/layout, output overwrites input, or diagram meaning changes | Decode/open saved export, inspect pixels, compare source semantics and test refusal/size limits |
| Project memory and data/AI systems — `project-memory`, `financial-models`, `statistical-inference`, `ai-tool-design`, `ai-evaluation`, `retrieval-evaluation` | Scoped Mirket memory only through its qualified route; other work uses host spreadsheet/data/model tooling | Source-linked scoped fact/correction/forgetting result, reconciled editable model, uncertainty analysis, bounded tool schema, matched model result and retrieval answer | Wrong project or stale vector leaks, formula not recalculated, dependence/multiplicity ignored, or prompt injection grants authority | Disposable cross-project/restart/forget fixtures, changed-input recomputation, independent statistical/retrieval oracle and scope/stale checks |
| Motion and interactive animation — `motion-direction`, `ui-motion`, `scroll-motion`, `svg-lottie` | Host project code/browser; selected libraries belong to project; Mirket design/video only for qualified assets | Purpose/timing/interruption/reduced-motion spec, responsive playback and editable animation source | Hidden scroll trigger, no reduced-motion path, unsupported Lottie effect or lifecycle leak | Rendered playback at target viewport, interruption/keyboard/reduced-motion checks and source parity |
| Authored video — `video-production` | Qualified Mirket HyperFrames + render-only Chrome + explicit FFmpeg/ffprobe; Codex owns brief/editing | Editable HTML source and decoded MP4/WebM/GIF/PNG output with audio/captions | Host approval blocks render, remote asset is fetched, changed source leaves identical frames, or captions/audio drift | Decode frames/audio/captions independently, inspect saved pixels, and retain refusal/cancellation evidence |
| Documents and communication — `technical-docs`, `document-artifacts`, `workbook-authoring`, `slide-deck-authoring` | Codex host document/spreadsheet/presentation tools and project format tools; Mirket supplies no Office/PDF/OCR engine | Source-accurate task docs, reopened page render, recalculating editable workbook and inspected editable deck | Export succeeds but page/table/chart/slide is wrong, formula stale, scanned value invented or unsupported feature implied | Reopen saved file, render/inspect every relevant page/sheet/slide and compare to independent source values |
| Prototype validation — `prototype-test` | Host design/code/browser tools | Small faithful prototype, observed task and decision rule | Prototype tests decoration instead of riskiest assumption or claims production behavior | Frozen task, observation log, failure decision and retained prototype/source revision |
| Release and operations — `dependency-admission`, `infrastructure`, `ci-release`, `native-desktop`, `runtime-operations` | Codex owns shell/Git/permissions and platform tools; Mirket owns only scoped memory, JS/TS navigation, design/video and receipts | Hash-bound dependency/manifest, drift-safe rollout, native install proof, or reconciled Mirket operation | PATH fallback, stale release, denied permission, wrong project, Windows/GUI path assumed from macOS/source checks | Disposable install/home/project, explicit executable identity, failure/refusal and actual saved effect; Windows and Desktop GUI remain unqualified |
| Coordination — `agent-coordination-strategy` | Codex owns native agents, models, permissions and integration; Mirket can only provide guidance | Disjoint work, capable creative lead/reviewer, lead-owned merge and one shared review/repair budget | Low-capability creative delegate, overlapping ownership, reviewer disagreement, interruption/denial or budget reset | Native model-turn evidence, actual artifacts and independent review; guidance compliance is observed, not hard-enforced |

## Source and evidence anchors

These anchors make the route claims above auditable.  They point to the actual
skill body, the approved capability contract, and a saved qualification or
failure record where one exists.  A planning reference or a tool receipt is not
treated as output evidence.

| Domain | Source/contract | Existing evidence and remaining gate |
| --- | --- | --- |
| Discovery, browser and growth | [browser-research](../skills/browser-research/SKILL.md), [browser-automation](../skills/browser-automation/SKILL.md), [content-search](../skills/content-search/SKILL.md), [approved browser boundary](../planning/implementation-plan.md#concrete-capability-and-evidence-contracts) | [project-skill sample](m4-project-skill-evaluation.md) covers nontrigger and bounded turns; live account, permission and production funnel evidence remain task-specific |
| Customer and product decisions | [customer-research](../skills/customer-research/SKILL.md), [jobs-to-be-done](../skills/jobs-to-be-done/SKILL.md), [product-strategy](../skills/product-strategy/SKILL.md), [positioning](../skills/positioning/SKILL.md) | No held-out model comparison is recorded; participant consent, segment bias and current market evidence must be retained per task |
| Architecture and implementation | [architecture](../skills/architecture/SKILL.md), [api-integration](../skills/api-integration/SKILL.md), [backend-engineering](../skills/backend-engineering/SKILL.md), [code-change](../skills/code-change/SKILL.md), [implementation plan](../planning/implementation-plan.md#execution-route) | Rust/adapter repository gates cover source behavior; catalog skill outcome benefit remains `planned-not-run` |
| Data and SQL | [data-analysis](../skills/data-analysis/SKILL.md), [sql-analysis](../skills/sql-analysis/SKILL.md), [sql-navigation](../skills/sql-navigation/SKILL.md), [data-pipelines](../skills/data-pipelines/SKILL.md) | SQL is explicitly Codex/project-owned; no Mirket SQL tool is claimed. Use disposable data and independent recomputation |
| Testing, review and reliability | [test-design](../skills/test-design/SKILL.md), [critical-review](../skills/critical-review/SKILL.md), [outcome-verification](../skills/outcome-verification/SKILL.md), [reliability-incident](../skills/reliability-incident/SKILL.md) | [content review](skill-catalog-content-review.md) verifies static catalog integrity; saved artifact and failure-case inspection remain held-out gates |
| Security and privacy | [threat-model](../skills/threat-model/SKILL.md), [security-review](../skills/security-review/SKILL.md), [privacy-data](../skills/privacy-data/SKILL.md), [memory contract](memory-evaluation-contract.md) | Existing memory smoke/evaluation fixtures cover scoped lifecycle cases; cross-product skill model trials remain open |
| Accessibility and localization | [accessibility](../skills/accessibility/SKILL.md), [localization](../skills/localization/SKILL.md), [WCAG reference](../planning/references.md#reference-wcag) | Browser and host UI checks must include keyboard, reflow, bidi and reduced-motion states; no catalog-wide held-out result yet |
| Design systems and editable design | [design-systems](../skills/design-systems/SKILL.md), [openpencil](../skills/openpencil/SKILL.md), [token contract](m4-token-consumption-qualification.md), [OpenPencil qualification](openpencil-editable-qualification.md) | Token consumer and editable-file qualifications exist; full catalog selection and broader glyph/font coverage remain unqualified |
| Vector/raster/diagram | [vector-art](../skills/vector-art/SKILL.md), [raster-art](../skills/raster-art/SKILL.md), [diagrams](../skills/diagrams/SKILL.md), [diagram qualification](m4-diagram-qualification.md) | Installed raster/diagram checks inspect saved outputs; Windows and untested source feature subsets remain open |
| Motion and video | [motion-direction](../skills/motion-direction/SKILL.md), [svg-lottie](../skills/svg-lottie/SKILL.md), [video-production](../skills/video-production/SKILL.md), [video contract](../planning/implementation-plan.md#concrete-capability-and-evidence-contracts) | [video qualification](m3-motion-video-qualification.md) and installed format smoke tests exist; decoded frame/audio/caption review and host approval-denial case remain required per task |
| Documents and communication | [document-artifacts](../skills/document-artifacts/SKILL.md), [workbook-authoring](../skills/workbook-authoring/SKILL.md), [slide-deck-authoring](../skills/slide-deck-authoring/SKILL.md) | Codex host tools own Office/PDF/OCR; reopen/render and scanned-content uncertainty must be observed, not inferred from export success |
| Data/AI and retrieval | [ai-tool-design](../skills/ai-tool-design/SKILL.md), [ai-evaluation](../skills/ai-evaluation/SKILL.md), [retrieval-evaluation](../skills/retrieval-evaluation/SKILL.md), [semantic qualification](m2-semantic-qualification.md) | Frozen bilingual/scale memory results exist; unseen answer quality, installed-client latency and cross-project distractors remain separate gates |
| Release and native operations | [dependency-admission](../skills/dependency-admission/SKILL.md), [ci-release](../skills/ci-release/SKILL.md), [native-desktop](../skills/native-desktop/SKILL.md), [runtime-operations](../skills/runtime-operations/SKILL.md), [release static review](m4-macos-release-static-review.md) | Disposable Mac install/update checks exist; Windows native ownership, earlier/newer macOS and Desktop GUI activation remain unqualified |
| Coordination and feedback | [agent-coordination-strategy](../skills/agent-coordination-strategy/SKILL.md), [coordination/feedback review](deep-review-coordination.md) | Existing evidence records the out-of-project retry and approval-blocked video; native Codex guidance is advisory; representative held-out comparisons are recorded, while catalog-wide acceptance remains open |

## Catalog coverage and profile comparison

The following coverage partition is intentionally one-to-one: every catalog ID
appears exactly once in this table. This is a static completeness check, not
a test of skill outcomes.

| Area | Catalog IDs |
| --- | --- |
| Discovery/research/growth | `browser-research`, `browser-automation`, `browser-qa`, `content-search`, `evidence-research`, `jobs-to-be-done` with Codex research, Codex host workflows (retired bundle) |
| Planning/architecture | `task-planning`, `architecture` |
| Implementation | `api-integration`, `backend-engineering`, `code-change`, `frontend-engineering`, `interface-composition`, `database-migrations`, `data-pipelines` |
| Navigation/query/analysis | `js-ts-navigation`, `sql-navigation`, `sql-analysis`, `data-analysis`, `data-visualization` |
| Test/reliability/review | `performance`, `reliability-incident`, `root-cause-debugging`, `test-design`, `critical-review`, `outcome-verification` |
| Security/access/localization | `threat-model`, `security-review`, `privacy-data`, `accessibility`, `localization`, `typography` |
| Design | `art-direction`, `design-critique`, `design-systems`, `openpencil`, `vector-art`, `raster-art`, `diagrams` |
| Data/AI | `project-memory`, `financial-models`, `statistical-inference`, `ai-tool-design`, `ai-evaluation`, `retrieval-evaluation` |
| Motion/video | `motion-direction`, `ui-motion`, `scroll-motion`, `svg-lottie`, `video-production` |
| Documents | `technical-docs`, `document-artifacts`, `workbook-authoring`, `slide-deck-authoring` |
| Prototype/operations | `prototype-test`, `dependency-admission`, `infrastructure`, `ci-release`, `native-desktop`, `runtime-operations`, `agent-coordination-strategy` |

The seven profiles are compared below as realistic mixed projects.  A profile is
a selection hint, not a capability receipt or dependency installer.  The
`explicit additions` column records specialist skills that a real project must
opt into; `omissions` records tempting skills that should remain unselected.
Mirket-required entries must refuse or warn when their qualified capability is
absent, while host-only entries continue through Codex/project tools.

| Mixed project | Profile union | Explicit additions | Intentional omissions | Observable acceptance |
| --- | --- | --- | --- | --- |
| Multilingual SaaS with browser onboarding and a service API | `web-product` + `backend-service` + `browser-workflows` | `localization`, `accessibility`, `sql-analysis`, `privacy-data` | `video-production`, `openpencil`, `native-desktop` | Keyboard/RTL/reflow journey, API failure recovery, scoped analytics and changed funnel result |
| Billing and AI assistant platform | `backend-service` + `data-ai` + `library-cli` | `financial-models`, `ai-tool-design`, `ai-evaluation`, `retrieval-evaluation`, `threat-model`, `dependency-admission` | `browser-automation`, `art-direction`, `slide-deck-authoring` | Reconciled billing workbook, prompt/tool authority negative tests, stale retrieval refusal and reproducible package |
| Design-led launch with authored video and campaign pages | `design-studio` + `web-product` | `video-production`, `motion-direction`, `svg-lottie`, `localization` | `sql-navigation`, `native-desktop`, `financial-models` | Reopened design source, inspected saved pixels, decoded frames/audio/captions and precommitted funnel measure |
| Research and authorized browser operations team | `browser-workflows` + `data-ai` | `browser-research`, `browser-automation`, `browser-qa`, `evidence-research`, `security-review` | `openpencil`, `video-production`, `database-migrations` | Page identity, scoped action reconciliation, source dates/disagreement and no account/scope escape |
| Cross-platform desktop SDK and CLI | `library-cli` + `backend-service` | `native-desktop`, `dependency-admission`, `ci-release`, `performance`, `reliability-incident` | `browser-automation`, `openpencil` | Explicit executable identity, installed lifecycle/permission tests and rollback on a native target |
| Analytics and experimentation group | `data-ai` + `backend-service` + `documents` | `sql-analysis`, `data-pipelines`, `statistical-inference`, `data-visualization`, `workbook-authoring` | `openpencil`, `native-desktop`, `video-production` | Independent query/recompute, uncertainty/multiplicity review, editable workbook and changed-input chart |
| Policy/document production team | `documents` + `data-ai` | `technical-docs`, `document-artifacts`, `slide-deck-authoring`, `localization`, `accessibility`, `financial-models` | `browser-automation`, `video-production`, `openpencil` | Reopened pages/slides/sheets, source-consistent figures, locale/reflow checks and explicit scanned/OCR limits |
| Conversion-focused web product | `web-product` + `browser-workflows` | `interface-composition`, `design-critique`, `browser-qa`, `accessibility` | `sql-navigation`, `openpencil`, `raster-art` | Rendered responsive journey, ranked defects, precommitted conversion metric and rollback evidence |

## Findings and correction direction

1. The matrix exposed no missing subject area relative to the approved thirteen
   outcomes: all requested workflow domains have an owner; ordinary product/marketing work stays with Codex without a dedicated Mirket skill.
   A static catalog check cannot establish that those skills trigger correctly
   or produce the stated outputs; that remains the planned held-out gate.
2. The principal demonstrated gap is evidence state, not a missing skill:
   `catalog.json` marks every evaluation `planned-not-run`, while sampled
   installation documents can be mistaken for full acceptance.  This document
   makes the distinction explicit; static coverage was checked against all 61
   catalog IDs. No prose-mirroring test is used to claim runtime qualification.
3. Host ownership is preserved in the matrix: SQL, browser research/automation,
   documents, spreadsheets, slides, ordinary code and native agents are not
   redirected to Mirket.  Mirket operations are limited to the capability names
   reported by `runtime_status`; a receipt, hash or tool list alone is not an
   output check.
4. Windows 11 x64/ARM64 and Codex Desktop GUI activation remain unqualified as
   required by the approved plan.  No profile row treats macOS source tests as
   Windows or GUI evidence.
