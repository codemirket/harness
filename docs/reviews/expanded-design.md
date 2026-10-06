> Domain review evidence. Final IDs, status and counts are authoritative in [the catalog](../../registry/catalog.json); source-provider descriptions are not endorsements.

# Expanded design-source review

This review expands the earlier minimal shortlist into **74 useful candidates across eight pinned sources**, with **201 canonical skill entries** available for discovery. Of those candidates, **32 have complete instruction/required-companion review and can be registered directly or with the listed adaptation**, while **42 remain manual integration or further-review choices**. A manual entry is a useful capability with a concrete unresolved packaging, runtime, license, or review boundary; it is not a rejection of its style or size. No upstream program, installer, daemon, generated HTML, or binary was executed.

`candidates.json` separates source facts, recommendation, adaptation, dependencies and review coverage. `inventory.json` includes every canonical SKILL under the chosen source trees and SHA256 of each SKILL. The inventory is discovery metadata, not certification that every body and script has been audited. Provider copies and OpenDesign plugin/example duplicates are intentionally not counted repeatedly.

## Sources and meaningful coverage

| Source | Pinned commit | Canonical skills | Finding |
|---|---|---:|---|
| [UI UX Pro Max](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/tree/477bcb28c9812b385cb51a4605ddf30d7b2266e2) | `477bcb28c9812b385cb51a4605ddf30d7b2266e2` | 7 | Substantial data/search plus brand, tokens, banners and slides. More than a single design prompt. |
| [Hallmark](https://github.com/Nutlope/hallmark/tree/13ac0ec7e148655948100b6396439e481361d690) | `13ac0ec7e148655948100b6396439e481361d690` | 1 | Opt-in design governance with dozens of gates and persistent project artifacts. |
| [Taste](https://github.com/Leonxlnx/taste-skill/tree/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b) | `ce26fc25c0e5e8cab638f883de62d9a86ee5e45b` | 13 | Different styles and image-first workflows; directory names often differ from frontmatter names. |
| [Emil](https://github.com/emilkowalski/skills/tree/e8a175de22ae1e49370fc144c1f3bb9aeedf988d) | `e8a175de22ae1e49370fc144c1f3bb9aeedf988d` | 14 | Strong focused mobile, stress-testing, variant comparison, motion and library guidance. |
| [Diagram Design](https://github.com/cathrynlavery/diagram-design/tree/3996c1607503ec4bcdb60b018568359d20f71d15) | `3996c1607503ec4bcdb60b018568359d20f71d15` | 1 | Current SKILL says **44** visual types (some README copy says 42); large editable SVG/HTML library. |
| [OpenDesign](https://github.com/nexu-io/open-design/tree/53231d40b778d88eba23f35547bf99485d3ae9fc) | `53231d40b778d88eba23f35547bf99485d3ae9fc` | 163 | **85 advertisement-only wrappers**, 78 substantive or copied entries. Canonical scope is `skills/`, not all app/plugin duplicates. |
| [Impeccable](https://github.com/pbakaus/impeccable/tree/4e8504f10106a1cd7a99e37a401aa368d1d55576) | `4e8504f10106a1cd7a99e37a401aa368d1d55576` | 1 | Current v4.5 consolidates commands inside one runtime-backed skill; historical lists of separate skills are misleading. |
| [Archify](https://github.com/tt-a1i/archify/tree/73aaa0696e8f72c232ea710e6fa94fd953f3e773) | `73aaa0696e8f72c232ea710e6fa94fd953f3e773` | 1 | Typed JSON → validated interactive diagrams; complete Node/browser package needed. |

Previous full-body source findings remain in `docs/reviews/design.md`; this review changes their earlier overly selective registry recommendation without discarding their evidence.

## Useful project profiles

These are choices the catalog can offer, not bundles to load into every task:

- **UI robustness:** Emil `mobile-native` + `break-ui`; add `ask-sonner` only when the project uses Sonner. Mobile web checks complement native iOS/Android skills; they do not replace them.
- **Compare directions:** Emil `prototype` for real, isolated variants and fixed comparison chrome; OpenDesign `reference-design-contract` when a durable evidence-backed visual handoff is wanted. Do not make three contract files or three variants for a prescribed one-line CSS correction.
- **Style exploration:** choose ONE Taste `minimalist-ui`, `industrial-brutalist-ui`, or `high-end-visual-design`, or a fitting HTML template. These have mutually incompatible palette/type/shape/motion mandates, so co-installation must not imply applying all simultaneously.
- **Existing app redesign:** Taste `redesign-existing-projects` with the adaptation preserving identity and actual facts. Treat its audit as a menu of candidates, not a command to replace Inter, flat surfaces or a sidebar regardless of usability.
- **Research and handoff:** OpenDesign `research-decision-room` creates a shareable evidence ledger and decision artifact from real material. Existing authored `customer-research` can route to it when a substantial HTML synthesis is requested. `reference-design-contract` covers visual evidence, not customer interviews.
- **HTML presentation/publishing:** choose Swiss, editorial/e-ink, open-canvas, Keynote-style, parchment report, magazine article or resume template. These output HTML and must not masquerade as an editable PPTX/DOCX tool. Native document and presentation handlers retain ownership of file editing and conversion.
- **Marketing graphics:** banner-design for art direction/sizing; quote/share cards, social carousels, tall hero poster, or device showcase for specific formats. No automatic posting, fake endorsements, or fabricated metrics.
- **Data storytelling:** data-report for provided datasets with Chart.js/ECharts; frame-data-chart-nyt for a focused SVG editorial chart. These are illustrative publishing recipes, not statistical validation or scientific plotting tools.
- **Diagramming:** Diagram Design for many controlled editorial SVG grammars and imports; Archify for typed models, validation, interaction and traceability. Prefer ordinary Mermaid or existing inline visualization for a simple explanatory diagram unless the user selects the richer artifact workflow.
- **Advanced design runtime:** Impeccable or Hallmark as deliberate project governance packages. Do not layer both plus all Taste profiles as coequal global instructions.
- **GSAP motion:** the eight OpenDesign GSAP entries are substantive source material; original upstream license notices and library version should be verified before direct copied registration. Three full-body reviews are recorded. Existing motion stack wins.

## Important concrete integration findings

### OpenDesign: distinguish a registry advert from a skill

The 85 wrappers say: “This catalogue entry advertises the skill in OpenDesign” and instruct the agent to install the original upstream bundle. For example [threejs](https://github.com/nexu-io/open-design/blob/53231d40b778d88eba23f35547bf99485d3ae9fc/skills/threejs/SKILL.md) contains no Three.js technique, assets or scripts. The same applies to `color-expert`, `shader-dev`, `d3-visualization`, `swiftui-design`, `flutter-animating-apps`, `hand-drawn-diagrams`, and many others. Preserve them as upstream leads in source inventory; do not advertise a wrapper copy as functioning support.

Substantive OpenDesign recipes are often portable despite `od:` metadata. The actual body matters: `research-decision-room` needs only files/browser and local evidence/checklist references, while [brand-extract](https://github.com/nexu-io/open-design/blob/53231d40b778d88eba23f35547bf99485d3ae9fc/skills/brand-extract/SKILL.md) requires `od brand preview` / `od brand finalize`, daemon-rendered `brand.html`, a registry ID and an `agent-browser` session. A short provider-name substitution cannot make that whole integration standalone.

The [reference contract](https://github.com/nexu-io/open-design/blob/53231d40b778d88eba23f35547bf99485d3ae9fc/skills/reference-design-contract/SKILL.md) is an especially useful original workflow: observed/provided/inferred evidence, keep/change/do-not-copy boundaries, one coherent visual direction, implementation handoff. Existing DESIGN.md must be preserved/merged, not silently replaced.

The [research evidence model](https://github.com/nexu-io/open-design/blob/53231d40b778d88eba23f35547bf99485d3ae9fc/skills/research-decision-room/references/evidence-model.md) explicitly separates stakeholder opinion, repeated observation and weak evidence. Its numeric opportunity scores remain judgments, not statistical estimates. An adapter should not fabricate an experiment owner or success threshold just to fill the queue.

Template caveats are fixable, not reasons to reject whole visual families. The Reddit card demands generated usernames/vote counts; X card implies a blue verification badge; Spotify card assumes duration; sticky-flowchart demands at least five nodes. Treat these as visibly fictional demo affordances only or omit them. Do not manufacture content or alter topology for decoration. Some templates prescribe Tailwind/Chart.js CDNs; locally renderable HTML does not mean fully offline.

OpenDesign root is Apache-2.0. Copied Taste/Emil/GSAP entries preserve upstream attribution or MIT declarations. Prefer already-reviewed original sources for Taste/Emil rather than duplicate installs with the same name. The GSAP directories contain only SKILL.md and no full MIT notice; do not silently reclassify them all as Apache-2.0.

### Diagram Design: excellent breadth, real package boundaries

[SKILL.md](https://github.com/cathrynlavery/diagram-design/blob/3996c1607503ec4bcdb60b018568359d20f71d15/skills/diagram-design/SKILL.md) distinguishes semantic patterns from visual types, imports meaning rather than renderer coordinates, and requires a fidelity ledger when merging/dropping source nodes. It provides accessible SVG title/description contracts, static/reduced-motion behavior and type-specific complexity budgets. These are useful techniques worth retaining.

First use asks for style-guide customization unless a project profile resolves. Preserve user-supplied brand and previously answered decisions rather than forcing redundant intake. The normal node cap is nine, but import mode `faithful` explicitly permits up to 24 zoned nodes; do not incorrectly summarize every path as nine nodes maximum.

The skill bundles five Python helpers: Mermaid extractor (~52KB), draw.io extractor (~31KB), Excalidraw extractor (~26KB), self-check (~19KB), SVG export (~15KB). These and 44 type references are **not fully audited in this bounded expansion**, therefore candidate is manual pending full package review. Its `self_check.py` is inside the installed subtree, but geometry and motion verification commands point to repository-root scripts. Do not claim those checks are installed by copying the skill folder alone.

[Third-party notices](https://github.com/cathrynlavery/diagram-design/blob/3996c1607503ec4bcdb60b018568359d20f71d15/THIRD_PARTY_LICENSES.md) distinguish MIT code/Tabler/log-z/Devicon, CC0 Simple Icons and OFL Instrument Serif logo outline. Google-font links and brand marks have different offline/trademark implications. Preserve the notice file alongside root MIT license.

### Impeccable: current source is a runtime, not just 20 prompts

[Current Codex SKILL](https://github.com/pbakaus/impeccable/blob/4e8504f10106a1cd7a99e37a401aa368d1d55576/.agents/skills/impeccable/SKILL.md) is one 12KB entry with 20+ playbooks, new-work/mode routing, context loading, platform references, live iteration, hooks, agent definitions and a large font index. Its explicit “brief wins”, preserving refinement scope, and reporting stale context without silently repairing it are good improvements over rigid aesthetic-only prompts.

The [shell launcher](https://github.com/pbakaus/impeccable/blob/4e8504f10106a1cd7a99e37a401aa368d1d55576/.agents/skills/impeccable/scripts/impeccable) executes an override, sibling binary, cached engine or PATH binary, otherwise downloads a versioned release to `~/.impeccable`. A SHA256 sidecar is verified for a new download; this does not pin an overridden/cached engine to the catalog source SHA. Windows `.cmd` launcher exists, and Windows ARM can fall back to x64. The main skill has a manual context-reading fallback if the launcher fails.

The 500KB live browser implementation, native engine, provider agent definitions and all playbooks were not fully audited; manual package status is deliberate. Do not split command-table rows into fake standalone install paths. Root Apache-2.0 and `NOTICE.md` for MIT-derived platform guidance must travel together.

### Prior sources, broadened recommendations

Taste's explicit strong visual styles are now retained as project options rather than excluded for being opinionated. The four fully reviewed style/redesign bodies need small overriding notes for existing brand, reduced motion, scope and truthfulness. `gpt-taste` contains a request to simulate a Python random result: never retain fabricated tool evidence. Image-first skills remain useful explicit capabilities, but require actual image tools and review of their long workflows; an image mockup is not a production interactive screen.

Emil's `prototype/PICKER.md` and `ask-sonner/API.md` were fully read in this expansion. The picker is intentionally fixed harness chrome, can move to the top to avoid a bottom toast, respects reduced motion and ignores shortcuts while editing input. Its fixed-number variants are appropriate when comparing directions; not for every component request. Sonner is an existing React-library skill, not authorization to install that dependency globally. The vocabulary is useful for naming effects but some glossary statements simplify rendering/performance behavior; do not treat them as benchmark evidence.

UI UX Pro Max banner references have useful dimension/safe-zone/art-direction tables but include an outdated “Meta penalizes 20% text” assertion and unsupported “highest CTR” claims. The adapter must ask current source evidence for changing platform facts. Its main BM25 search and token/brand/image siblings remain valuable optional bundles, with Python/Node/API dependencies declared. `ui-styling` has conflicting MIT frontmatter vs local Apache-2.0 LICENSE.txt and needs reconciliation.

Hallmark is a consciously comprehensive project process, not rejected merely for its 58 gates. However a skill-only install omits `../../site/css/tokens.css`; the documented preflight cache does not capture all rendered input changes. Scope and packaging must be concrete before installing.

Archify creates diagram-specific work folders, explicitly separates automated/browser/perceptual evidence, and supports five typed modes. Its [skill](https://github.com/tt-a1i/archify/blob/73aaa0696e8f72c232ea710e6fa94fd953f3e773/archify/SKILL.md) calls a complete Node renderer/schema/browser package and a bounded update check in delivery. Retain it as a full integration with exact package content and asset licenses, not merely SKILL.md.

## Gaps to keep visible

Native macOS/iOS/Expo coverage is being reviewed by the parent against current OpenAI plugin sources. OpenDesign wrappers do not close Android/Flutter, Three.js, shader, color-science or D3 gaps until their actual upstream bundles are reviewed. A rich inventory can offer those leads honestly without loading empty or unaudited skills. Production-auth implementation, accessibility certification, scientific plotting, document-format fidelity and statistical research validity remain responsibilities of the relevant engineering/provider tools, not claims of these visual recipes.
