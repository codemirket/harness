# IBM hybrid illustration — controlled fidelity explains a relationship

Study language: **Engineered conceptual illustration**. This label is a curator interpretation, not an upstream standard name.

## Brief → concept → grammar

**Brief.** Explain what a client should do after a receipt write times out. A saved write and a lost response are both possible. The viewer must understand why checking the original reference comes before repeating the request.

**Concept.** Three original UI fragments carry one unanswered question across a workflow. The middle fragment names the uncertainty; the last preserves the two conditional outcomes. The connector expresses the reasoning rather than merely decorating three cards.

**Grammar.** IBM’s source combines UI fragments, conceptual pictograms and connectors, adjusting fidelity to the message. The official hybrid guidance uses an 8 px construction rhythm; its line illustration guidance has a separate 4 px grid. Do not collapse these into one universal specification. The transferable relationship is selective detail in service of a causal story, not a mandate to reproduce IBM’s palette, type or product UI.

## Located decisions in the original study

- `main.recon .flow` puts request, observation and reconciliation in reading order. At a narrow width that order becomes a vertical sequence.
- `.sheet .ghost` abstracts nonessential interface copy. The actual reference `R-208`, uncertainty label and branch outcomes remain readable HTML, because they carry the decision.
- `.condition` explicitly says “Read before you repeat”; `.fork` retains both Found and Absent. Neither colour nor a success tick substitutes for the branch.
- `.receipt` is a newly drawn pictogram. The sample service and its records are illustrative, not claims about an existing Mirket API.

## Source, edition and inspection

The named upstream instance is the bill-payment assistant composition, `example_assistant.svg`, with the fidelity comparison described in official MDX. The live design page timed out in the reader. The official MDX was read and the assistant SVG was rendered as a local image for visual inspection on 2026-10-15. It shows simplified UI blocks, genuine local labels, icons and a connector. Prior unchanged research also inspected Line Design and Hybrid Usage. No live product or internal kit was accessed.

The original study borrows the relationship between controlled abstraction and a legible workflow. It omits the assistant scene, IBM identity, IBM Plex, IBM product panels, brand pictograms and source assets. The w3id-gated kit is excluded. A separate icon license cannot authorize those illustrations.

## Attractive wrong variant and discriminating failure

Open `?variant=misleading`. The same panel proportions, grid, shadows, type and original pictograms now connect a missing response to a new write `R-209`, without checking `R-208`. This is a polished conceptual error, not a contrast or spacing failure.

Assume `R-208` was saved just before the response was lost. Follow the visible steps: the correct version checks and returns the saved receipt; the misleading version can duplicate it. A visual check that counts three aligned cards or recognises an enterprise aesthetic must fail to accept that variant. This diagram teaches the uncertainty; it is not an executable retry-policy specification.

## Transfer to a different brief

Explain a museum loan whose dispatch acknowledgement is missing. Preserve the accession and loan references, show “shipment state unknown,” and inspect the dispatch record before arranging another shipment. Invent the objects and labels for that domain; do not reuse the receipt illustration or replace every node with a generic checkmark.

**Suitable:** technical overviews, workflow explanation, architecture editorial graphics. **Poor fit:** a faithful interface tutorial, human gesture, a legal or operational guarantee of exactly-once delivery.

## Primary sources and rights boundary

- [IBM Design Language: Hybrid UI style design](https://www.ibm.com/design/language/illustration/hybrid-ui-style/design/) — Living guidance, checked 2026-10-15. Link-only study. No scene, kit, font or brand asset is admitted; no open grant for the selected illustration established. No open redistribution license is established for this selected reference.
- [Official Hybrid UI source and named assistant illustration](https://github.com/carbon-design-system/design-language-website/blob/master/src/pages/illustration/hybrid-ui-style/design.mdx) — Unpinned official source read 2026-10-15; named example_assistant.svg. Source content is used for inspection and citation only. Separate Carbon or icon code licenses do not clear these IBM scenes. No open redistribution license is established for this selected reference.
- [IBM line style construction](https://www.ibm.com/design/language/illustration/line-style/design/) — Living guidance, checked in research 2026-10-15. Link-only; line style is a separate construction reference, not an asset license. No open redistribution license is established for this selected reference.

## Owner and exact routes

**Craft owner:** `graphic-design` for the concept, composition and source-to-form check; `svg-creation` for extending editable vector construction. Use `interface-design` when integrating the asset into a working product interface. The reference card is a decision aid, not an instruction to mix design systems.

- Card: `references/design-languages/ibm-hybrid-illustration.md` in the `design-inspiration` skill.
- CLI read: `mirket catalog read design-inspiration --path references/design-languages/ibm-hybrid-illustration.md`.
- MCP resource: `mirket://skills/design-inspiration/references/design-languages/ibm-hybrid-illustration.md`.
- Original source: [editable HTML/CSS/SVG](../../assets/design-languages/ibm-hybrid-illustration/index.html), at `assets/design-languages/ibm-hybrid-illustration/index.html` inside that skill.
- Source read: `mirket catalog read design-inspiration --path assets/design-languages/ibm-hybrid-illustration/index.html`; resource `mirket://skills/design-inspiration/assets/design-languages/ibm-hybrid-illustration/index.html`.
- Render the saved HTML in a real browser. From this repository: `python3 -m http.server 8765 --bind 127.0.0.1 --directory skills/design-inspiration`, then open `http://127.0.0.1:8765/assets/design-languages/ibm-hybrid-illustration/`. For an installed copy, serve that skill’s root directory instead. Append `?variant=misleading` for the intentional failure. No download or build step is required.

The specimen is the `<main>` region. Teaching and comparison notes live in a separate `<aside>`. The correct content, labels and editable SVG are complete without JavaScript; JavaScript only selects the query-based misleading comparison. Inspect the saved output at desktop, narrow width and the stated smallest use size. Rendering establishes visibility, not human acceptance.
