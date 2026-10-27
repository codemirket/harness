# Hallmark and Taste source-to-skill audit

Research-only audit requested on 2026-10-27. No skill, catalog, provenance,
dependency, or installation files were changed by this review.

## Evidence and rights

The approved pins are:

- [Hallmark](https://github.com/Nutlope/hallmark/tree/13ac0ec7e148655948100b6396439e481361d690), source
  `skills/hallmark/SKILL.md`, SHA-256
  `59469635bbbd21acddbc664d95e8d97147f454b2d8f4a0a89c7d2c5afbe67577`, and
  `LICENSE`, SHA-256
  `06088a8b94598626f27612dea42300154bf7be967c85d9d3eee4490cb056af7d`.
- [Taste](https://github.com/Leonxlnx/taste-skill/tree/f624f37a8c650b9cc550253104925efe4c8de246), source
  `skills/taste-skill/SKILL.md`, SHA-256
  `ee847084e58f5a81a46120912be9cd15f083570a6ca8f3d1a30372a720633043`, and
  `LICENSE`, SHA-256
  `4575a543ab88dad12ccea7d97e563d0bce5b448b06072e65d3264497dad326df`.

Both licenses are MIT and require the copyright and permission notices in
copies or substantial portions. The current catalog correctly classifies both
as comparison-only and says no upstream prose, code, assets, fonts, or runtime
was adopted. This report independently summarizes techniques; it does not copy
source text or authorize upstream scripts, bundled examples, fonts, images,
Together AI, Node/Chrome launchers, or registries.

Raw receipts and bodies are retained outside the repository at
`/tmp/mirket-reference-raw-2026-10-27/` and recorded in
`planning/reviews/2026-10-27/reference-fetches.json`.

## Findings

### F1 — High source-fidelity gap / predicted failure risk: public-page proof is
not explicitly protected from invention

Hallmark’s cross-verb discipline says not to invent metrics, testimonials,
logos, or case-study counts, and to use a labelled placeholder or a different
structure when proof is unavailable (pinned source §§42–50, especially source
lines 46–48). Taste makes the same decision before markup: every section is a
claim with proof, sections without a claim are deleted (lines 62–67), and
sample details must be marked for confirmation while proof is never invented
(lines 170–176).

Current local coverage is close but not explicit: `art-direction` says to use
real copy and available assets (`skills/art-direction/SKILL.md:8-12`), while
`interface-composition` asks for evidence near public-page claims
(`skills/interface-composition/SKILL.md:12`). Neither says what to do when a
metric, logo, testimonial, review, or case-study result is missing. A model can
therefore satisfy “real content” with plausible but unsupported proof. This is
not a demonstrated Mirket model/runtime failure: the host’s general honesty
guidance already forbids fabrication, and no matched acceptance trial was run
for this report. It is a source-to-skill fidelity gap with a predictable failure
mode worth testing.

Recommended preservation: add a conditional rule to `art-direction` and
public-page `interface-composition`: map each claim to supplied/verified proof;
never invent proof; use a labelled placeholder or remove/restructure the claim;
mark illustrative sample details for confirmation. `design-critique` should
rank an unsupported claim as a concrete content-truth defect when observed, not
as a visual preference. Do not require this marketing-page rule for private
dashboards or user-supplied copy whose truth is outside the visual task.

### F2 — High source-fidelity gap / predicted failure risk: the subject-specific
idea method is under-specified

Taste’s distinctive anti-slop method is not merely “use real content.” It asks
for an inventory of 8–12 facts belonging to this client and no competitor,
including place, materials, rituals, constraints, and printed artefacts (lines
38–60). It then deals three structurally different directions, chooses against
the inventory, records why, and records every justified redraw (lines 69–117).
The idea must pass a nearest-competitor test (lines 121–147).

Hallmark independently reinforces the same quality bar: find the idea in the
subject rather than a palette or adjectives (`SKILL.md` lines 203–209 and the
six disciplines at lines 46–50), choose structural variety rather than colour
swaps (lines 11–13), and preserve an existing project’s direction and content
intent (lines 27–36).

`art-direction` currently asks for a visual idea from the subject
(`skills/art-direction/SKILL.md:16`) and allows a small number of structurally
different specimens (`:18`), but it does not require a subject inventory,
competitor test, a one-sentence idea, or a reason for selecting/redrawing a
direction. This predicts a generic “editorial/modern/premium” failure mode
despite the anti-trend language, but is not a demonstrated model/runtime
failure: no matched model trial was run for this report.

Recommended preservation: for an open-ended art-direction brief, record a
short subject inventory, formulate “subject fact becomes compositional device,”
test it against a real competitor, and compare only a few materially different
directions. Record why the chosen direction fits and why any redraw was needed.
Do not import Taste’s random seed, sector library, direction IDs, or script;
the decision method can be independently authored and routed through Codex or
qualified Mirket design operations.

### F3 — Medium: existing-project preflight and edit boundary are not visible

Hallmark’s redesign safety rail preserves routes, component ownership, copy
intent, brand, and information architecture; defaults to in-place/additive
changes; requires a file-level plan before edits; and requires confirmation for
deletions (source lines 27–36). Its preflight reads the existing design system,
font stack, palette, motion dependencies, spacing, and framework before asking
or changing anything, reports the findings with file/line citations, and flags
conflicts (lines 147–201).

Local skills do inspect the existing product and accepted direction
(`interface-composition:8,16,20`; `art-direction:18,22`), and the project’s
Codex/AGENTS rules already prohibit unsafe destructive edits. However, the
skill output does not require an explicit preservation boundary or preflight
record. In a redesign, a capable model can still touch more files or override
an existing token/font/motion owner than the user intended while remaining
visually plausible.

Recommended preservation: before a non-greenfield visual change, report the
existing route/component/token owners and exact files in scope; preserve them
by default; honor an already authorized file/component scope without asking
again. Ask only when a deletion, broader multi-component replacement, or other
destructive scope is new or genuinely ambiguous. For an existing design system,
report the detected typography, palette, spacing, framework and motion
ownership before choosing a direction. This should remain a compact boundary
check, not a Hallmark clone or a second project scanner.

### F4 — Medium: component state coverage misses Hallmark’s deliberate 8-state
contract

Hallmark switches to a component-specific flow and requires every interactive
component to cover default, hover, focus-visible, active, disabled, loading,
error, and success, with a labelled side-by-side state wrapper (source lines
60–137). Taste’s semantic rule independently requires real links/buttons,
visible focus, alt text, and a phone composition (lines 228–229).

`interface-composition` currently asks for empty/loading/error/success/
selected/disabled states and focus (`skills/interface-composition/SKILL.md:10`),
then exercises important controls and transient states (`:18`). It does not
explicitly require hover and active, nor a component-scoped specimen that makes
all states inspectable. This matters for a button/input/card request where a
default screenshot can pass while the pressed, hover, or focus treatment is
broken.

Recommended preservation: when the task is a single interactive component,
exercise the eight states (or document why a state is inapplicable), including
keyboard focus and the real loading/error/success semantics. Keep the rule
conditional; do not force page-level states or Hallmark’s temporary wrapper
files into every interface task.

### F5 — Medium: explicit “study this reference” requests lack a source-mode
boundary

Hallmark’s `study` branch distinguishes screenshot and URL input, extracts
structure/type/colour rather than pixels, refuses template-marketplace URLs,
requires a diagnosis before an optional rebuild, and requires an attestation
before emitting a portable design file from a URL (source lines 27–30 and
§`hallmark study` lines 490–550). URL mode also reports that HTML/CSS cannot
establish visual rhythm. This is a useful fidelity and rights boundary, not a
request to install Hallmark’s WebFetch or Together route.

Local `art-direction` permits references and says to preserve the user’s chosen
direction (`skills/art-direction/SKILL.md:18,24`), but has no explicit branch
for “study this screenshot/site.” If such a request arrives, the agent could
silently move from comparison to implementation or imply a visual inspection
that URL HTML cannot support.

Recommended preservation: only for an explicit study request, first return a
structural diagnosis and source mode; never copy pixels, source assets, fonts or
brand text; apply the DNA when the user has already authorized that scope, and
ask only if applying it would add an ambiguous or destructive scope; state URL
rhythm and access limits. Codex remains the owner of general browsing, and no
Mirket study/browser tool is justified by this finding.

### F6 — Low/optional: compact self-critique would strengthen the final artifact

Hallmark requires a pre-emit review on Philosophy, Hierarchy, Execution,
Specificity, Restraint, and Variety; any score below 3 triggers repair and the
scores are stamped in the artifact (source lines 42–48). Local `design-critique`
already requires ranked, observed consequences and paired before/after evidence
(`skills/design-critique/SKILL.md:8-18`), and `art-direction` freezes a target
and checks focal point, reading order, contrast and material character
(`:16,20,26`).

The underlying judgment is covered, so this is not a demonstrated correctness
defect. If a compact final-artifact self-check is added, use the six questions
as prompts and evidence fields, not as a mandatory numeric style gate or copied
Hallmark stamp.

## What is already faithful, and what should remain optional

- Actual saved-pixel inspection is explicit in `art-direction:14,26`,
  `interface-composition:16,18,22`, and `design-critique:8,18`; a renderer,
  source diff, or screenshot receipt is not treated as visual proof.
- Existing local responsive pressure, lifecycle, states, and design-token
  ownership are more product-appropriate than importing Hallmark’s exact
  320/375/414/768 gates or Taste’s exact 1440/768/390 inspector widths.
- Asset rights are now explicit in `art-direction:10`, and the local report
  must retain creator/source/license/notices for incorporated assets. A source
  locator, private raw download, or inspiration reference grants no asset/font
  rights.
- Taste’s open-collection image ladder, four-image minimum, one moving image,
  eight-motion-element floor, 60–90 ms stagger, and 16–32 px travel are
  source-specific policy/thresholds. They are useful comparison evidence but
  not universal Mirket requirements; local motion and raster/video skills own
  qualified operations and actual inspection.
- Hallmark’s 21-theme rotation, no-italic/no-gradient/no-monospace rules, exact
  section spacing, and nav/footer archetype rotation are its house style. Do not
  present them as accessibility, correctness, or product capability gates.
- Neither source authorizes a Mirket dependency or tool. Keep Codex/project
  ownership for browsing, shell, files, Git, permissions and general web
  review; keep Mirket scoped to its qualified design/raster/vector operations.

## Current-upstream drift (candidate evidence, not a pin replacement)

The approved review remains historical and is the source-to-skill basis. A
read-only `git ls-remote`/shallow inspection on 2026-10-27 found these current
heads:

Repository popularity is not method evidence: the same-day GitHub metadata
snapshot recorded approximately 37,977 Hallmark stars and 103,367 Taste stars;
these counts do not change the pinned selection or rights analysis.

- Hallmark `7ec868225c6caa040558c321ee0fefa0e73f83a8`, source SHA-256
  `25a1ff92d925b2389e8409f42fb8ad7e033fceca8b5d1f07ed4ca4c28da53dc3`,
  version 2.0. The current source reorganizes the method into a hard Floor and
  Flow, explicitly floors invented metrics/testimonials/logos/case-study
  counts, material/proof inventory, real-product capture, and a fresh review
  sweep. These strengthen the pinned F1/F2 observations, but they are later
  source data and were not silently treated as approved-reference content.
  Current source: [`skills/hallmark/SKILL.md`](https://raw.githubusercontent.com/Nutlope/hallmark/7ec868225c6caa040558c321ee0fefa0e73f83a8/skills/hallmark/SKILL.md).
- Taste main `12af471f627f866411b4bd785b19646d3672784c`, source SHA-256
  `3435ed963b15a1b6447746ec8844ab2cd38d8d2e8ead3451937f42df01e408c4`,
  released as 2.1.5. It adds a page-versus-app-screen route, real data and
  accessibility/keyboard requirements, production-build inspection, screenshot
  retention fixes, and safe escaping for chart/split-heading text. These are
  useful candidates for a future reference refresh; they are not claims about
  the approved `f624f37a8c65` body.
- The current Codex/GPT-specific classic is in the separate
  [taste-skill-classics](https://github.com/Leonxlnx/taste-skill-classics/tree/5e08c9383dc4ad7fc0df70e1ac8cfa09ec356305)
  repository at `5e08c9383dc4ad7fc0df70e1ac8cfa09ec356305`, with
  `skills/gpt-tasteskill/SKILL.md` and
  `skills/gpt-tasteskill/agents/openai.yaml` (`allow_implicit_invocation:
  false`). Its distinctive methods are a simulated deterministic Python
  randomization, mandatory AIDA structure, GSAP/ScrollTrigger motion, a
  `prefers-reduced-motion` static fallback, and a mandatory design-plan block.
  It also mandates Picsum image URLs, a floating-glass/pill-oriented AIDA
  arsenal, and “static interfaces are forbidden.” Treat this as an opt-in
  historical candidate only: mandatory GSAP, randomization, external stock
  URLs, and rigid AIDA/creative bans conflict with the Codex/Mirket ownership
  boundary and the approved Taste method. Do not add it to the catalog or
  silently merge it into `art-direction`/`interface-composition`.

Current-upstream observations are drift evidence, not model outcome evidence;
no current-head behavior was run in this repository.

## Current association and verification limits

`skills/catalog.json` and `skills/PROVENANCE.md` associate Hallmark and Taste
with `art-direction` and `interface-composition`; Hallmark is also associated
with `design-systems` and `openpencil`, while Taste is associated with
`raster-art` and `marketing-copy`. All are currently comparison-only with
planned acceptance cases, not runtime receipts. Local body hashes at this
review are:

```text
skills/art-direction/SKILL.md         6f42d61f144460613b086aab20a52ccb2b6a1f276a41acffa062d16606370ac8
skills/interface-composition/SKILL.md 3b89bf4f5ee400b84c44650030e53675e09eeccdd74d80a1ab5e595656c979a2
skills/design-critique/SKILL.md       0e3441fef6e4154f195774c2260a2b740792a6a0f0f4e319f06490d984c85ff6
```

This is an exact pinned-source body/license review with static local mapping;
it did not run model acceptance trials, browser inspections, or an external
asset-license qualification. Those remain required evidence before claiming
the catalog’s planned acceptance complete.
