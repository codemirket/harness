# Design-direction research (2025-04-27 through 2026-10-27)

Status: bounded research, not a product qualification. This note records reusable methods from seven qualifying recent examples outside the approved Taste/Hallmark pins, plus one explicitly excluded context example. It separates what the project makers or an independent publication actually describe from my own transfer judgment. An award, a studio claim, or a screenshot is not proof of usability, accessibility, performance, or business outcome.

## Method and rights

I searched award records and practitioner/editorial case studies published in the window above. I opened the source pages, captured exact publication/award dates where available, and retained HTML and any small reference images privately under:

`/Users/nazmiyilmaz/.codex/task-state/harness-design-research-round2-20261027/directions/`

I inspected saved pixels with an image-capable tool for the Exat, Dropbox, and Elimar reference images. I did not run the sites, copy assets, or reuse source text/code. The pages and visual assets are proprietary unless their respective terms say otherwise; the recommendations below are method abstractions only. Do not ship client logos, type, imagery, code, or screenshots from these sources without permission and a per-asset license check.

## Examples and observed methods

### 1. Heron AI — Bearplus / Kenny Ho (creative AI product narrative)

Sources: [Codrops case study, 12 Oct 2026](https://tympanus.net/codrops/2026/10/12/heron-ai/) (method and implementation account); [independent Rickcreates teardown, accessed 27 Oct 2026](https://rickcreates.com/r/2026-10-12-heron-ai) (award/performance observations). The Codrops article identifies Awwwards SOTD (12 Oct), FWA SOTD (1 Oct), and CSSDesignAwards WOTD (12 Sep); Rickcreates reports its own Awwwards/FWA scores and browser measurements. Those are reception/teardown evidence, not a guarantee for another build.

Observed mechanics: the product had no finished UI, so the team made architectural floor plans, dimensions, annotations, violation marks, and metadata the interface itself. The opening asks the visitor to drag a crosshair/box around a building problem; an orange card then reports a code violation. Loading, geometry, labels, and explanation reveal in a deliberate sequence. The project account also says prototypes and mobile should have been developed earlier.

Transferable method: for a technical or invisible subject, make the subject's actual evidence format (diagram, trace, measurement, status) the visual grammar and let one small interaction perform the core promise. Map every claim to real proof; if the product is only a concept, label the specimen as invented rather than implying a deployed result. This is a candidate extension of `skills/art-direction` and `skills/interface-composition`, not a reason to copy its orange palette, architecture, or animation stack.

Contraindications: a fictional assistant can invent a convincing “result”; a real product cannot. A drag gesture needs a keyboard/touch/static path, and an elaborate loader must not conceal the task. The independent teardown's load/CLS/FPS numbers are observations of that site only.

### 2. Dropbox Brand Studio — Daybreak Studio (living guidelines)

Sources: [Webflow Awards announcement, last updated 15 Sep 2025](https://webflow.com/blog/webflow-awards-2025) (independent platform award/editorial description); [Webflow customer case](https://webflow.com/customers/dropbox) (team-reported 3x testing velocity, 2x interactivity, and two-week landing-page launches); [Communication Arts Webpick](https://www.commarts.com/webpicks/dropbox-brand-guidelines) (independent editorial observation). The Rive technical post is dated 17 Jan 2025, before this review window, so it is deliberately not used as a qualifying source or basis for new adoption here.

Observed mechanics: the guidelines are a playground, not a static PDF. The in-window sources connect scrolling, navigation and micro-interactions to the brand story, with interactive color/iconography content. Communication Arts notes persistent navigation and keyboard paths. A saved Webflow image was inspected: a strict grid holds a bright blue message block, large color fields, and editor chrome; the grid is content structure, not decoration.

Transferable method: treat a design system as an explorable set of relationships. Pair each interactive example with its name, use, state and source; let motion demonstrate the rule rather than merely decorate a card. Keep the CMS/content owner and the animation owner explicit. This supports the current `interface-composition` state matrix and `art-direction` owner/proof rules; it does not justify a requirement for Rive or a specific density.

Contraindications: brand guidelines are unusually controlled content. Proprietary Dropbox marks, colors, illustrations and Rive files are not reusable assets. Report Webflow's metrics as customer claims, not independent causal evidence.

### 3. Exat Typeface microsite — Studio Size / RISE2 (type as spatial system)

Sources: [Codrops case study, 10 Apr 2026](https://tympanus.net/codrops/2026/04/10/the-exat-microsite-pushing-a-typography-showcase-to-new-creative-extremes/) (design/development method); [Studio Size project page](https://studio-size.com/work/exat/) (project context); [CSS Design Awards 2025 WOTY announcement, 12 Feb 2026](https://www.cssdesignawards.com/blog/2025-website-of-the-year-winners/430/) (independent award reception). The Codrops article describes a 21-style, 1,715-glyph system, visibility-gated internal animations, reversible scroll state, sparse 3-D rotation, parallel design/development, and mobile simplification. The saved `exat-top4.jpg` was inspected: giant orange/black forms, stacked function labels and circular glyph markers make the type specimen itself the navigation/content scaffold.

Transferable method: when the subject is a variable type system, let glyph behavior, specimen hierarchy and reversible scroll state carry the narrative. Gate costly internal animation to visible panels, and simplify mobile composition instead of shrinking desktop. These are conditional techniques for `art-direction`, `ui-motion`, and `scroll-motion` when the content itself is a type specimen; they are not general motion thresholds or a reason to force 3-D.

Contraindications: a showcase can afford abstraction that an application cannot. Do not copy EXAT's glyphs, references to EXAT 51/New Tendencies, wordmark, or visual assets. The award score is reception, not evidence that every visitor understood the specimen.

### 4. The Symphony of Vines — Unseen Studio / Château Oublié (material-to-story interaction)

Sources: [live project](https://symphonyofvines.unseen.co/) (accessed 27 Oct 2026; four-chapter interaction description); [CSS Design Awards 2025 Website of the Year, published 12 Feb 2026](https://www.cssdesignawards.com/blog/2025-website-of-the-year-winners/430/) (independent award reception, listed 8.88). The project maps four material forces to chapters: press/drag to carve a glacier, conduct rivers, wield tides, and alter soil; the chapters resolve into a terroir fingerprint and bottle.

Transferable method: make an input metaphor correspond to a real subject operation, then let chapters accumulate into a concrete conclusion. Define what each interaction proves before styling it; preserve a readable chapter sequence and a non-gesture path. This gives `art-direction` and `scroll-motion` a useful content-to-motion test beyond a generic “hero animation.”

Contraindications: four full-screen gesture scenes and a cursor metaphor are not a default web pattern. Touch, keyboard, reduced-motion and direct navigation must retain the meaning. The CSSDA listing establishes reception only, not conversion or accessibility.

### 5. KHUFU'S Digital Experience — Monarq (heritage without pastiche)

Sources: [Monarq case study](https://monarqeg.com/case-studies/khufus/) (published five months before 27 Oct 2026; method and self-reported Cloudflare reach); [Monarq award notice, 7 May 2026](https://monarqeg.com/news/khufus-earns-awwwards-honorable-mention/) (official date and linked Awwwards record). The project uses place, restraint, negative space, typography and pacing to establish anticipation; an interactive photographic menu keeps decision-making inside the editorial world; a third-party reservation path remains explicit; mobile reorders content and controls rather than simply compressing desktop. Monarq reports 100,000+ unique launch-week visits from Cloudflare, which is a self-reported metric.

Transferable method: translate a physical subject into proportion, silence, sequence and content operations rather than repeating its obvious symbols. Put the practical action (menu/reservation) inside the same story but give it a clear owner and handoff. This supports `art-direction`, `interface-composition`, and `design-critique` when a dramatic direction must still prove task completion.

Contraindications: do not imitate Egyptian imagery, pyramid motifs, AI-generated camel film, or luxury pacing. Treat AI imagery as a claim/artifact requiring provenance and review, not proof of place. The award and Cloudflare count do not establish usability or performance in another project.

### Excluded from the qualifying window: Elimar — The Vincent van Gogh Experience — OddCommon / LMI

The method is useful, but I could not verify a project/case-study publication or award announcement on or after 27 Apr 2025. The Webby ceremony was 12 May 2025, but that does not establish the announcement date; the official announcement page is 22 Apr 2025 and therefore outside the window. Sources retained for context only: [OddCommon project case](https://www.oddcommon.com/work/elimar_experience) (accessed 27 Oct 2026), [Webby winner record](https://winners.webbyawards.com/winners/websites-and-mobile-sites/features-design/best-use-of-animation-or-motion-graphics), and [Webby announcement](https://www.webbyawards.com/press/press-releases/29th-annual-webby-awards-announce-2025-winners/). The saved `elimar-zoom.jpg` was inspected: a sparse museum-like white room places framed images and small explanatory text in a wide spatial composition. The case says the particle field resolves at a distance and disperses on closer examination, while an interactive timeline anchors the story in paintings and letters. This example is not counted among the seven qualifying cases and does not drive a new adoption recommendation.

Transferable method (context only): pair an atmospheric visual metaphor with a hard evidence spine. The dream-like field can make uncertainty felt, but the timeline and correspondence prevent the subject from becoming empty spectacle. Use this as a `ui-motion`/`design-critique` test: does the expressive layer clarify the source trail, and can a visitor still read the evidence at rest?

Contraindications: do not treat a historical or scientific reconstruction as established fact without provenance. Particle/WebGL effects need a static and reduced-motion representation; museum-like typography must be tested for legibility. OddCommon's awards establish reception, not research validity.

### 6. Saudi Now identity — Alphabetical with Shaqa Bovand / F37 (bilingual parity)

Sources: [Creative Bloq independent feature, 20 Dec 2025](https://www.creativebloq.com/design/branding/saudi-now-has-the-best-bilingual-logo-ive-seen) (practice description and reception); [Saudi Now About page](https://saudinow.co/en/page/about-us) (client purpose and bilingual platform). Creative Bloq reports a custom Arabic calligraphic form paired with a British sans, equal legibility, an angled Arabic-calligraphy structure, and a system extending into monogram, website, film, animation, signage, merchandise and toolkit. It also reports that early explorations were tested with Arabic and English viewers of different ages for legibility.

Transferable method: treat localization as co-authored form, not a late translation. Test both scripts with real readers before expanding the mark into motion or a toolkit; preserve parity of prominence and meaning. This is a concrete extension for `art-direction`, `interface-composition`, and localization-facing project skills, not a reason to add a generic “make it culturally authentic” checklist.

Contraindications: do not redraw Arabic or another script from a Latin analogue, borrow the mark's angled construction, or claim bilingual accessibility without script-specific readers. The feature's “best” framing is editorial opinion; the reported testing is the useful method, not a universal pass.

### 7. 4WIDE — Tomoya Okada (expressive effect with explicit constraints)

Sources: [Codrops case study, 23 Apr 2026](https://tympanus.net/codrops/2026/04/23/building-4wide-turning-distortion-blur-and-motion-into-a-coherent-experience/) (method, launch 31 Mar 2026); [CSS Design Awards WOTD, 14 Apr 2026](https://cssdesignawards.com/sites/4wide/49150/) (independent judging, 8.20 public/final score detail). The author explicitly frames the problem as expressiveness versus business clarity. A curved/distorted About transition, parallax/RGB-shift showcase and default/focus hero modes are tied to content. The article documents mobile constraints: optimized WebP images, half-size/segment geometry, blur disabled on mobile, and low-power renderer settings. A focus mode slows the background video and changes the information shown.

Transferable method: treat visual effects as a budgeted content mode, not a permanent layer. Define what focus mode reveals, give mobile a deliberate treatment, and record which effect is removed when its cost is too high. This is a useful concrete refinement to `ui-motion` and `scroll-motion`'s existing lifecycle/performance guidance; it does not introduce blanket device thresholds or require a WebGL engine.

Contraindications: the article is an author case study and includes code excerpts; no code was copied. Blur, RGB shift, and slow video can reduce readability or motion comfort. A mode switch is only useful if the non-effect state still communicates the business proposition.

## Cross-example findings for Mirket skills

The current local skills already cover the broad guardrails: subject inventory and competitor-substitution test (`skills/art-direction/SKILL.md:20`), claim-to-proof handling and owner boundaries (`art-direction:10,16`; `interface-composition:8,12`), real component hover/focus/active states (`interface-composition:10`), lifecycle/reversal/reduced-motion checks (`ui-motion:16-18`), and interior reload/orientation/late-media checks (`scroll-motion:14-18`). The examples above do not justify copying Taste/Hallmark slogans, rigid viewport/animation thresholds, mandatory pre-plans, or universal confirmation gates.

What they do add is a small set of conditional tests that remain worth preserving in project-specific work:

1. **Subject evidence test:** can a visitor see the subject's real evidence format (plan, type specimen, correspondence, menu, bilingual script) rather than a generic moodboard?
2. **Metaphor-to-action test:** does the gesture or motion perform the subject's operation, and is its meaning retained without a pointer or motion?
3. **Mode/owner test:** which owner supplies content, motion, external reservation/CRM, and proof, and what remains in the default state when an effect is disabled?
4. **Responsive proof test:** what was deliberately simplified or re-ordered for mobile, rather than merely scaled down?
5. **Reception boundary:** record awards/editorial comments as independent reception, and record studio metrics as claims with their source; neither is a substitute for a matched user, accessibility, performance, or pixel trial.

These are refinements to existing guidance, not evidence of a runtime failure in this repository. No new dependency, tool, catalog entry, or shared asset is proposed from this research.

## Not counted as proof / unresolved limits

- I did not run the live sites in a browser or measure their current mobile, keyboard, reduced-motion, screen-reader, or network behavior. The saved image inspections establish only what those pixels show.
- No source page supplied a blanket license to reproduce its visual system. Rights, fonts, client marks, and asset provenance remain per-project checks.
- Webflow/Monarq/agency metrics are self-reported; CSSDA/Webby/Webflow awards are external reception, not a causal quality certificate.
- Windows behavior, Codex Desktop GUI checks and Mirket runtime behavior remain outside this research and unqualified.
