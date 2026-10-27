# Source-to-skill review: D, F and G (2026-10-27)

Reviewed catalog families: `marketing`, `wondel`, `karpathy`, `codex-practices`,
`kingy-motion`, `geist-code`, `atlassian`, `fluent`, `reshaped`,
`apple-liquid-glass`, `govuk`, `vanilla-publishing`, `guardian`, and `spectrum`.

This is the product/documented/reference-library review requested for the
exact pins in `planning/references.md`. It intentionally excludes E
(Animate.css, React Bits and Remotion); that group is owned by the visual
source reviewer. The retrieval date was 2026-10-27 on macOS 26.5.2 ARM64.
URL status, byte counts and response hashes are in
`planning/reviews/2026-10-27/reference-fetches.json`; repository bodies are
not copied into the catalog. A successful HTTP response is retrieval evidence,
not evidence that a technique was adopted or that a license permits reuse.

## D — marketing and discovery methods

The exact pinned repository is `coreyhaines31/marketingskills` at
`dda3841f0b294e01e93b1541486beefbfab0915e`. The following method bodies were
fetched from that commit and inspected (SHA-256 is over the response body):

| source body and inspected sections | hash | local contract / observable output | finding |
| --- | --- | --- | --- |
| `skills/customer-research/SKILL.md`: three modes; asset extraction; synthesis; quality guardrails; interviews/surveys; persona anti-patterns; deliverables | `905e65e67268448d88b3b96cf045c6b3615cdd87c1060e129b148a31aa627669` | `skills/customer-research/SKILL.md`: decision/segment/questions, source dates/consent/bias, observed vs stated vs inferred evidence, contradictions, traceable source trail, uncertainty, and no invented outreach | Retained the evidence discipline and source distinctions. The source's mandatory product-marketing file and fixed “five independent data points” threshold are not adopted: the former conflicts with host-owned context, and the latter would be a false universal prevalence rule. Local guidance instead requires an evidence-supported confidence and reports missing populations. No actionable distortion demonstrated. |
| `skills/product-marketing/SKILL.md`: context workflow and sections for audience, personas, pain, competition, differentiation, objections, switching dynamics, customer language, proof and goals | `6dfd6bd485f62448385844cb9eceefcee0977c843d7a1c9e6077590d4e19d1ec` | `skills/product-strategy/SKILL.md` and `skills/positioning/SKILL.md`: bounded decision, segment/use situation, alternatives, claim/evidence map, tradeoffs, owner, measures and unresolved assumptions | The useful inventory maps to explicit decision and evidence outputs. Local skills do not create or overwrite a context file or campaign account. This is a deliberate Codex/Mirket ownership boundary, not a missing capability. |
| `skills/cro/SKILL.md`: page type, conversion goal and traffic context; value proposition/headline/CTA/hierarchy/trust/objections/friction; quick wins vs high-impact changes vs tests; page-specific checks | `199016bec3b44bc6a7507b111b6028f6550cc852cd54c7d0df789f15d6f814c4` | `skills/conversion/SKILL.md`: event, eligible population, funnel stages and data limits; actual journey and recovery states; causal hypothesis, success/guardrail measure, correct denominator; no live transaction or launch claim | The source’s diagnostic order is represented without “best-practice” causality. A live page review still requires Codex browser evidence; instrumentation success is explicitly not product impact. No actionable gap. |
| `skills/ab-testing/SKILL.md`: hypothesis form; one change; predeclared sample/MDE; primary/secondary/guardrail metrics; no peeking; segment checks; result documentation and inconclusive handling | `1fd229160f6014f3f2d7c92cc7d2f991fbf8dc9ed68aac9793c71b408a6d1144` | `skills/experiments-launch/SKILL.md`: bounded exposure, owner, stop/review/recovery conditions, instrumentation and rollback, operational health vs impact, preserved null/negative result | The local output is safer for this product: it does not promise statistical certainty without an authorized experiment. Sample-size calculators linked by the source are not a license to launch or access external analytics. No actionable gap. |

The marketing README's “mandatory product-marketing context first” graph and
its broad plugin installation workflow were inspected but rejected for this
product. Host-owned Codex context, browser, documents and permissions must not
be shadowed by a Mirket launcher or a mandatory context file.

## Wondel methods

The exact pinned repository is `wondelai/skills` at
`c172996495bed0fcd26896a9416b2093fd7073f0`. The relevant root method bodies
(not merely its catalog README) were inspected:

| source body and inspected sections | hash | local contract / observable output | finding |
| --- | --- | --- | --- |
| `jobs-to-be-done/SKILL.md`: progress in circumstances; functional/social/emotional dimensions; job statement without a product name; push/pull/anxiety/habit; Big/Little Hire; non-obvious alternatives and non-consumption; purchase-timeline interviews; job-completion metrics; quick diagnostic | `cc227aa837bda4ae0910324e52a90d338354501d005f7ef2fbd0cff9fd73aaf1` | `skills/jobs-to-be-done/SKILL.md`: circumstance/progress/outcome, source-supported dimensions, current workarounds, forces and segment differences, evidence-linked alternatives, next uncertainty, and explicit no-demand claim | Local guidance preserves the method while rejecting the source's numeric “10/10” score as false precision. It does not claim a job sentence validates demand or willingness to pay. No actionable gap. |
| `cro-methodology/SKILL.md`: nine-step evidence-led process; quantitative WHERE vs qualitative WHY; persuasion proof; objection/counter-objection table; research-grounded hypothesis and ICE; predeclared sample/duration; honest inconclusive reporting | `52bfedca17750ad117d838bfdcb5c5db190fb8a7c6e9b82abe44f939830e3897` | `skills/conversion/SKILL.md` and `skills/experiments-launch/SKILL.md`: measurement/traffic/flow distinction, research-linked causal hypothesis, guardrail, recovery and no unrun-launch claim | The local contracts retain evidence before optimization and prevent live launches outside authorization. No actionable gap. |

The Wondel README's plugin/metaskill journeys were inspected as context but are
not adopted: Mirket does not become a second agent orchestrator or install a
competing plugin system.

## F — coding and moving comparison references

The exact pinned repository is `multica-ai/andrej-karpathy-skills` at
`2c606141936f1eeef17fa3043a72095b4765b9c2`. The README and the actual
launcher bodies were inspected:

| source body and inspected sections | hash | local contract / observable output | finding |
| --- | --- | --- | --- |
| `README.md`: “The Four Principles in Detail” — Think Before Coding (state assumptions, present interpretations, push back, stop when confused); Simplicity First; Surgical Changes; Goal-Driven Execution and testable success criteria | `daf314efb31894b86d68b67786c820bd8470bb2126e578ba9cd0e3e2667883f8` | `skills/code-change/SKILL.md` lines 8–14: owner/consumer tracing, smallest complete design, boundary recovery, preserved unrelated work, externally visible tests, failure case, gates, actual checks and environmental limits. `skills/task-planning/SKILL.md` handles bounded ambiguity and decisions. | The local pair covers the four useful decisions without copying the source launcher. No license file was found at the pin, so this remains comparison/link-only. No actionable gap. |
| `CLAUDE.md`: same four principles, per-project merge caveat, explicit tradeoff toward caution | `694a2d721e41c385f3db492838c23226df5ba9809e3b0721aac70021e196a` | Host-owned AGENTS/Codex guidance remains authoritative; no Claude plugin or first-use installer is added. | No local source reuse or ownership conflict demonstrated. |
| `.cursor/rules/karpathy-guidelines.mdc`: same principles, `alwaysApply: true` | `259cf32ac1a493b7bf863992bbb968079a2b6ffdce99dad092ec6f4e2764717d` | No Cursor rule is installed or implied. | Comparison-only; no actionable gap. |

The selected Codex best-practices and Kingy motion URLs are moving pages. They
were captured on 2026-10-27 (receipts and hashes in the fetch record), but
have no stable revision/license in the selection. They remain dated link-only
comparison evidence, not completed future coverage.

## G — visual/reference library

All multiple locators listed for each family were opened and the relevant
sections or license terms were read. These are reference links, not package,
font, brand or asset admissions.

| family and exact material inspected | useful technique mapped to local output/evidence | rights and boundary | finding |
| --- | --- | --- | --- |
| Geist Typography (hierarchy/roles) and Code Block (grouped code presentation); Geist font `LICENSE.txt` | `typography` returns a real-text specimen, wrap/measure/fallback/glyph evidence; `interface-composition` checks hierarchy and code/table pressure | Font license is SIL OFL 1.1 with reserved-name, notice and font-only-sale conditions. No Geist font/assets are bundled. | No gap; local guidance explicitly requires independent rights and glyph checks. |
| Atlassian Comment examples; Confluence page restrictions; Share Confluence content; Create/edit/publish a page | `interface-composition` and `accessibility` exercise author/content/visibility states; output includes permission, draft/publish, version and recovery evidence | Retrieved support pages expose product examples and marks; page metadata/terms were not treated as a blanket asset license. No Atlassian UI or copy is shipped. | No gap; local state/permission evidence is stronger than a static example. |
| Fluent 2 Motion and Material: functional/natural/consistent/appealing motion, duration/easing, hierarchy and no-motion setting; material/transparency hierarchy | `motion-direction`, `ui-motion`, `accessibility` and `interface-composition` return timing/easing, interruption, reduced-motion and settled-pixel evidence | Official guidance is link-only; no Fluent package, brand or material asset is added. | No gap demonstrated. |
| Reshaped Overview and Principles: core components, composition/theming, semantic tokens, accessibility/dark mode, long-term maintenance/browser-native dependencies, headless/styled balance | `design-systems` returns token source → compiled output → consuming specimen, state/theme/locale pressure and migration evidence | Moving docs have no admitted source revision or implementation license; no Reshaped API is promised. | No gap demonstrated. |
| Apple WWDC25 “Meet Liquid Glass” and HIG Materials | `native-desktop`/`interface-composition` report platform, material choice, adaptation and untested host behavior; no Apple API/asset claim | Apple pages/marks and Liquid Glass assets remain link-only. | macOS-only local checks do not qualify Windows or a native host; this remains explicitly unverified, not a skill defect. |
| GOV.UK Check answers (summary list/change links/confirmation), Complete multiple tasks (task state/completion), Typeface (public-service type hierarchy) | `accessibility`, `interface-composition` and `typography` return criterion-linked focus/order/reflow and real-content hierarchy evidence; change links and completion state are tested in the journey | GOV.UK material is used as a pattern locator under applicable OGL/site terms; no brand/markup/font is copied. | No gap demonstrated. |
| Vanilla brochure layouts and Strip pattern: structural content grouping and strip/layout boundaries | `interface-composition` returns real-content responsive composition and pressure viewport evidence rather than a framework dependency | No Vanilla component/CSS is installed. | No gap demonstrated. |
| Guardian typography MDX (headline/text presets and font loading), International front, and Guardian fonts usage restrictions | `typography` checks editorial roles, loading/fallback, mixed scripts and saved-output metrics; `accessibility` checks directionality/readability | Guardian font notice says fonts are Commercial Type property and may not be reproduced without permission. No font/page asset is copied. | No gap demonstrated. |
| Adobe Spectrum Bi-directionality and W3C Inline bidi markup: Unicode bidi, direction isolation and inline mixed-script markup | `typography` and `accessibility` return Arabic/Hebrew/mixed-script specimens, shaping, direction and focus/order evidence; `vector-art` preserves text semantics in exports | Link-only guidance; no Spectrum implementation or W3C/Adobe prose/assets are redistributed. | No gap demonstrated. |

The visual references therefore support existing local decisions rather than
justifying a new design library. E's Animate.css/React Bits/Remotion source
fidelity is intentionally not assessed here; see the visual source review.

## Findings and limits

1. **Provenance/evidence status (actionable, catalog-level):** the original
   authoring record's statement that reference bodies were not fetched remains
   historically true for that pass. This 2026-10-27 review supersedes the
   *current review status* with dated body hashes, section mapping and license
   classifications; it does not rewrite the authoring history. The
   independently-authored classification remains correct: these comparisons
   did not adapt source prose, code, assets or launchers into the local skills.
   The catalog/provenance refresh must consume the exact review receipts and
   must never turn HTTP 200 into “reviewed” or “license verified”.
2. **No demonstrated local body defect in D/F/G:** the source methods map to
   local triggers, decision methods, outputs and evidence rules. Mandatory
   third-party context files, plugin/metaskill orchestration, fixed JTBD scores,
   and upstream installers/assets were deliberately rejected because they
   conflict with the approved Codex/Mirket ownership boundary or would make
   unsupported claims.
3. **Unqualified paths:** moving pages (Codex practices, Kingy motion, current
   Reshaped/Guardian fronts and other web docs), Windows/native GUI behavior,
   and the waived Codex Desktop GUI check remain unverified. The 67-skill
   acceptance matrix is not complete merely because these source bodies were
   retrieved; its planned-not-run cases still require their own evidence.
