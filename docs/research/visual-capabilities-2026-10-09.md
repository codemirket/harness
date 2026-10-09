# Visual capabilities: source review, 2026-10-09

## Recommendation

Build a small visual workbench around **one design lead, concrete motion recipes,
editable SVG and browser evidence**. Start from the actual product, copy and visual
references; make and inspect a working screen. Do not install several competing
design personalities and expect their instructions to produce taste.

Recommended first delivery: a responsive product screen with a working drawer,
loading/empty/error states, one custom SVG illustration and one interruptible
transition. Include keyboard behavior, reduced motion, desktop/mobile captures,
motion frames and a short recording. The user chooses whether the composition and
motion feel right. That is a demonstrable capability, unlike a catalog entry.

This review used fresh official documentation, upstream source files, helpers,
license files and test sources. It did not derive the design from this harness.
No packages, skills, binaries, accounts or MCP connections were installed. No
candidate was run against a representative design task, so quality recommendations
below are source-informed judgments, not measured rankings.

## Nine candidates worth deciding about

| Candidate | Useful capability | Runtime and license | Decision |
| --- | --- | --- | --- |
| Impeccable | Product-aware design playbooks, measured browser detection and live variant selection | Native engine plus browser; Apache-2.0 | Optional workshop tool; review runtime separately |
| Emil Kowalski skills | Implementable interaction recipes, interruption, motion purpose and feel review | Markdown plus chosen CSS/WAAPI/Motion; MIT | Adapt `animate` recipes; keep review explicit |
| Interface Design | Product hierarchy, tokens, existing primitives and real UI states | Instruction-only; MIT | Preferred product UI lead after corrections |
| Taste Skill | Extensive marketing compositions and examples | Instruction-only, commonly assumes React/Tailwind/Motion; MIT | Reference only; reject its GPT-specific variant |
| Motion and official AI Kit | Current API guidance, CSS-vs-library choices, layout/gesture motion, optional docs MCP | Browser/project runtime; library MIT; kit package declares MIT; premium tools separate | Preferred motion API source; CSS first for simple cases |
| GSAP | Timelines, scroll choreography, SVG drawing/morphing | JavaScript/browser; custom no-charge license | Add only for a concrete timeline/SVG need |
| Remotion official skills | Deterministic frame-driven video and still exports | Node, React, renderer/browser; commercial eligibility matters | Video-only capability, not UI default |
| SVG + SVGO + resvg | Editable vector source, optional optimized export, independent static rendering | Node for SVGO, resvg binary/Rust; MIT / Apache-2.0 OR MIT | Practical vector pipeline; source stays editable |
| Figma official MCP | Actual design context, tokens, screenshots and source assets | Authorized Figma account and supported MCP client | Connect when the user has a design source |

Playwright is the shared verification tool, not a tenth design personality. Use
the project's existing browser/test setup; installing new dependencies remains a
separate action.

## What the source actually contains

### Impeccable

Reviewed revision `d631a8827f99414d2b6daba4ef08b7f8701751d7`:
[skill](https://github.com/pbakaus/impeccable/blob/d631a8827f99414d2b6daba4ef08b7f8701751d7/.claude/skills/impeccable/SKILL.md),
[live variants](https://github.com/pbakaus/impeccable/blob/d631a8827f99414d2b6daba4ef08b7f8701751d7/.claude/skills/impeccable/reference/live.md),
[launcher](https://github.com/pbakaus/impeccable/blob/d631a8827f99414d2b6daba4ef08b7f8701751d7/.claude/skills/impeccable/scripts/impeccable),
[browser differential test](https://github.com/pbakaus/impeccable/blob/d631a8827f99414d2b6daba4ef08b7f8701751d7/crates/browser/tests/differential.rs),
[license](https://github.com/pbakaus/impeccable/blob/d631a8827f99414d2b6daba4ef08b7f8701751d7/LICENSE).

This is now an executable design environment: the launcher locates a native
engine or downloads a pinned release and verifies its SHA-256 sidecar. Live mode
uses a local dev server/static page, browser overlay, generated variants and
accept/discard events. The detector tests compare implementations and exercise
browser findings; they establish detector behavior, not better visual taste.

Conflicts to remove from a personal adaptation: compulsory context invocation,
a universal two-round verification ceiling, ten-minute polling instructions,
and unconditional completion rhetoric. Do not run its launcher merely because a
skill was loaded. It can download executable code. Keep the real variant picker
as an optional, separately approved tool rather than copying its entire workflow
into every task. Its audit reference explicitly distinguishes verified detector
findings from visual judgment and false positives; retain that distinction.

### Emil Kowalski: animate and review-animations

Revision `e8a175de22ae1e49370fc144c1f3bb9aeedf988d`:
[animate](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/animate/SKILL.md),
[recipes](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/animate/RECIPES.md),
[review](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/review-animations/SKILL.md),
[license](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/LICENSE).

The recipes contain actual CSS/React examples for press feedback, anchored
popovers, modal/drawer, toast, accordion, tab indicator, stagger, scroll reveal,
drag-dismiss and WAAPI. The construction sequence asks what motion communicates,
then selects properties, timing, interruption and reduced-motion behavior.
The final output points to slow playback and real-device feel checks.

Use these as starting implementations, not immutable laws. The absolute bans on
keyboard-triggered animation, `ease-in`, pure fades and most layout properties
are opinions, not universal defects. Performance statements need measurement;
official Motion guidance permits small height animations and independent
transforms. Remove branded greeting-only responses and mandatory report formats.
Retain existing project tokens, interruption tests, pointer gating and explicit
reduced-motion behavior. No comparative visual-quality trial was found in the
reviewed files.

### Interface Design

Revision `2f9be3206855bcb2d1d0af262c8bae25cba6658d`:
[skill](https://github.com/Dammyjay93/interface-design/blob/2f9be3206855bcb2d1d0af262c8bae25cba6658d/.claude/skills/interface-design/SKILL.md),
[system template](https://github.com/Dammyjay93/interface-design/blob/2f9be3206855bcb2d1d0af262c8bae25cba6658d/reference/system-template.md),
[license](https://github.com/Dammyjay93/interface-design/blob/2f9be3206855bcb2d1d0af262c8bae25cba6658d/LICENSE).

A stronger fit for tools and dashboards than marketing art direction. Concrete
material covers hierarchy, three text tiers, spacing, surfaces, native controls,
headless primitives, existing components, loading/empty/error states and rendered
desktop/mobile review. It recommends showing specimens instead of only describing
them. It has no executable renderer or browser harness.

Adapt away per-component intent rituals, a required five-element signature and
the automatic offer to save a new system file. Use the project's existing design
record. Correct its attribution of 44×44 to WCAG: WCAG 2.2 AA criterion 2.5.8 uses
24×24 CSS pixels with exceptions; larger targets can be a design preference.
[W3C criterion](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html).

### Taste Skill

Revision `18dfc928b135629e0eddfdd445a06400d04ed439`:
[main skill](https://github.com/Leonxlnx/taste-skill/blob/18dfc928b135629e0eddfdd445a06400d04ed439/skills/taste-skill/SKILL.md),
[GPT variant](https://github.com/Leonxlnx/taste-skill/blob/18dfc928b135629e0eddfdd445a06400d04ed439/skills/gpt-tasteskill/SKILL.md),
[license](https://github.com/Leonxlnx/taste-skill/blob/18dfc928b135629e0eddfdd445a06400d04ed439/LICENSE).

The main file contains concrete layout and scroll-stack recipes, real-copy checks,
responsive rules and reduced-motion handling. Its many aesthetic bans can become
another template: fixed motion/variance defaults, short copy quotas, compulsory
generated imagery and strongly discouraged hand-authored illustration.

Reject the GPT variant. It requires simulated RNG execution and mock output,
forces an AIDA structure and GSAP, and forbids static interfaces. That would
replace real judgment with manufactured evidence and conflict directly with
product UI and vector work. Example images are demonstrations, not controlled
evidence that the instruction set improves results across models.

### Motion and official AI Kit

Library revision `5c5073af2ae92a2d7e17c17ef67f0e5db7ec96a7`:
[SVG example](https://github.com/motiondivision/motion/blob/5c5073af2ae92a2d7e17c17ef67f0e5db7ec96a7/dev/react/src/examples/SVG-path.tsx).
Kit revision `d1c5c26f424adfd47c112d894e9d424b57338c7e`:
[skill](https://github.com/motiondivision/ai-kit/blob/d1c5c26f424adfd47c112d894e9d424b57338c7e/plugins/motion/skills/motion/SKILL.md),
[CSS choice](https://github.com/motiondivision/ai-kit/blob/d1c5c26f424adfd47c112d894e9d424b57338c7e/plugins/motion/skills/motion/best-practices/css-or-motion.md),
[React patterns](https://github.com/motiondivision/ai-kit/blob/d1c5c26f424adfd47c112d894e9d424b57338c7e/plugins/motion/skills/motion/best-practices/react.md),
[package license declaration](https://github.com/motiondivision/ai-kit/blob/d1c5c26f424adfd47c112d894e9d424b57338c7e/packages/motion-ai/package.json).

Useful exact guidance includes stable keys in `AnimatePresence`, unique `layoutId`
per instance, Radix `forceMount`, `MotionConfig reducedMotion="user"`, keyboard
alternatives to pointer reordering and when CSS is sufficient. The SVG example
implements stroke/pathLength motion, but its clickable SVG is an animation demo,
not an accessible checkbox to copy unchanged.

The installer needs Node 18+. Standalone best-practice files work without MCP.
The kit declares MIT in package metadata and official docs, but the inspected
tree has no top-level license file; retain provenance and resolve notice packaging
before vendoring. Hosted docs are free; premium source, editor and MotionScore
have account/tier requirements. The docs and skill differ on easing-generation
tiering, so check actual tool availability rather than promise it.
[Official installation and tiers](https://motion.dev/docs/ai-kit-install).

### GSAP

Revision `13e2b790546426a1a2e0e9b409f3f8dc6d6611f2`:
[MorphSVG implementation](https://github.com/greensock/GSAP/blob/13e2b790546426a1a2e0e9b409f3f8dc6d6611f2/src/MorphSVGPlugin.js).
It performs path normalization and segment matching rather than assuming SVGs
have compatible point counts. This is concrete value for illustration morphs.
For UI, prefer an existing runtime; GSAP is justified for coordinated timelines,
scroll scenes and complex SVG work. `matchMedia` supplies breakpoint/reduced-motion
branches and reverts collected animations when conditions change.
[Official lifecycle recipe](https://gsap.com/docs/v3/GSAP/gsap.matchMedia/).

The custom license permits ordinary commercial websites and AI-generated code at
no charge, but restricts competing visual animation builders and requires notices.
Do not call it MIT or unrestricted open source.
[License](https://gsap.com/community/standard-license/).

### Remotion

Skills revision `32b241b97f4e0e4ab61fe9a41b05e6e64503f8c5`, version 4.0.534:
[markup](https://github.com/remotion-dev/skills/blob/32b241b97f4e0e4ab61fe9a41b05e6e64503f8c5/skills/remotion-markup/SKILL.md),
[render](https://github.com/remotion-dev/skills/blob/32b241b97f4e0e4ab61fe9a41b05e6e64503f8c5/skills/remotion-render/SKILL.md).
The source correctly separates frame-driven `useCurrentFrame`/`interpolate`
animation from browser-time CSS animation. The render skill supports stills and
selected frames, which makes intermediate visual review reproducible.

Use only for videos, animated explainers or exported motion graphics. It requires
a Remotion project and renderer; UI transitions should not require it. The skills
repository exposes no standalone license in the inspected tree, so do not assume
redistribution rights from public availability. Remotion's current license permits
individuals and for-profit organizations with up to three employees to use it
free; larger for-profit organizations need the company license. Recheck for the
version and actual legal entity. [Official license](https://www.remotion.dev/license).

### Editable SVG, SVGO and resvg

SVGO revision `e4cb29bebcc9820ac979dfc05106b512cc5de986`:
[preset source](https://github.com/svg/svgo/blob/e4cb29bebcc9820ac979dfc05106b512cc5de986/plugins/preset-default.js),
[ID cleanup](https://github.com/svg/svgo/blob/e4cb29bebcc9820ac979dfc05106b512cc5de986/plugins/cleanupIds.js).
resvg revision `617cebab98f8f08e354fb4664656d9a575db70e6`:
[render regression method](https://github.com/linebender/resvg/blob/617cebab98f8f08e354fb4664656d9a575db70e6/crates/resvg/tests/README.md),
[license declaration](https://github.com/linebender/resvg/blob/617cebab98f8f08e354fb4664656d9a575db70e6/Cargo.toml).

The useful capability is source geometry plus rendering, not a prompt that says
"make beautiful vectors." Author named groups, intentional viewBox, reusable defs,
consistent stroke rhythm and accessible title/description. Keep an editable
master. SVGO defaults clean IDs, collapse groups and merge paths; those operations
can break external animation selectors or destroy useful editing structure.
Optimize a derived export with an explicit configuration. resvg renders static
SVG and has image regression fixtures; it does not verify scripted SVG animation.
Use a browser for that. SVGO is MIT; resvg is Apache-2.0 OR MIT. Fonts and embedded
assets have separate licenses.

### Figma MCP

Official hosted documentation reviewed 2026-10-09; no public server revision or
server implementation inspected. `get_design_context`, variables, component
mappings and `get_screenshot` provide source-grounded implementation inputs.
`download_assets` can supply SVG exports/original images; screenshots are for
inspection, not substitutes for source assets.
[Tool contract](https://developers.figma.com/docs/figma-mcp-server/tools-and-prompts/).

The official installation guide supports Codex and Claude Code and requires
OAuth. No account was connected or client activation tested. Preserve the
project's framework and actual components instead of copying default generated
React/Tailwind indiscriminately. Asset rights depend on the source file; account
access is not an asset redistribution license.
[Supported setup](https://developers.figma.com/docs/figma-mcp-server/remote-server-installation/).

## Small implementation set

1. **Design/build:** adapt Interface Design's hierarchy, states and primitive reuse.
   Keep a short working brief with audience, primary action, real copy and two
   reference crops naming what is borrowed. For a major new direction, show two
   real coded specimens before expanding the whole page. Do not force alternatives
   for routine repairs. Use a custom asset only where it serves the product.
2. **Motion:** bring in a narrow subset of Emil recipes: anchored popover, drawer,
   accordion, toast and tab indicator. Couple them to official runtime guidance.
   Retain CSS/WAAPI default, interruption and reduced-motion variants. Add GSAP
   or Motion only when the existing stack/interaction requires it.
3. **Vector:** ship one original editable SVG example and a browser preview page
   with size/background controls and playback/scrub controls. Keep SVG authoring
   distinct from raster generation; icons/charts/marks should retain editable
   geometry. Static raster export is supplementary.
4. **Review:** one runnable browser scenario and a short independent critique of
   the actual output. Critique must identify location, observed problem and a
   concrete repair; separate functional defects from aesthetic preferences.
   Human visual acceptance remains a distinct result.

Keep Impeccable live mode, Figma and Remotion optional. This is enough for useful
daily work without installing every source reviewed above.

## Executable verification workflow

Implement these commands in the consuming project, using already installed tools.
The following examples are an implementation contract, not executed evidence.

**UI:** create `tests/visual/ui.spec.ts` with stable fixtures and existing selectors.
Visit the real screen at 1440×1000 and 390×844; wait for fonts/images and fixture
readiness. Exercise the primary task, loading/empty/error states, keyboard focus,
Escape and focus return. Capture screenshots only after those states exist.
Use the repository's installed runner, for example:

```sh
./node_modules/.bin/playwright test tests/visual/ui.spec.ts --project=chromium
./node_modules/.bin/playwright show-report
```

Use `toHaveScreenshot` for stable resting states after a human has reviewed the
baseline. Do not automatically update golden images to make a failing test pass.
Pin browser/environment for comparisons; a pixel match establishes consistency,
not good design. [Playwright visual comparisons](https://playwright.dev/docs/test-snapshots).

**Motion:** include normal and `page.emulateMedia({reducedMotion: 'reduce'})`
scenarios. Capture entry, midpoint, settled and exit frames. Rapidly toggle a
drawer twice and reverse a drag to expose interruption bugs. Save a real-time
recording as well: evenly sampled stills cannot establish timing or feel.

For deterministic CSS/WAAPI sampling, trigger the interaction first, then pause
the target's animations and set their `currentTime` before each screenshot:

```js
await page.getByRole('button', {name: 'Open details'}).click();
for (const ms of [0, 90, 180]) {
  await page.locator('[data-motion-review]').evaluate((element, time) => {
    for (const animation of element.getAnimations({subtree: true})) {
      animation.pause();
      animation.currentTime = time;
    }
  }, ms);
  await page.screenshot({path: `artifacts/motion-${ms}.png`});
}
```

This captures only animations exposed through Web Animations. GSAP/rAF-driven
scenes need a development-only deterministic timeline handle, or Playwright clock
control before app initialization. Do not claim CSS screenshots test every engine.
Test real playback separately; do not use screenshot animation disabling for
midpoint inspection. [Playwright clock](https://playwright.dev/docs/clock).

**Vector:** render the editable master at intended small, normal and large sizes
on light/dark backgrounds; inspect clipping, stroke weight, joins, optical balance,
label legibility and duplicated-ID collisions when multiple instances coexist.
With resvg already available:

```sh
resvg --width 256 assets/illustration.svg artifacts/illustration-256.png
resvg --width 1024 assets/illustration.svg artifacts/illustration-1024.png
```

Render both master and optimized export before accepting optimization. Test
animation after optimization, including IDs referenced by external code. Record
fonts used for deterministic static export. For Remotion projects only, the
review equivalent is a contact sheet of selected frames plus the rendered video:

```sh
./node_modules/.bin/remotion render Demo artifacts/frames --frames=0,30,90 --image-format=png
./node_modules/.bin/remotion render Demo artifacts/demo.mp4
```

**Acceptance:** deliver the actual screen/preview, paired reference/result crops,
normal/reduced-motion recording, editable SVG, and a brief defect list. Ask the
user to judge visual direction and feel only after the implementation is usable.
Record accepted, revise, or not-reviewed; never translate automated checks into
human approval. Fix material defects and rerun affected captures. Stop when the
requested quality is met, not when a compulsory polish counter expires.

## User references and the working example

The user admires GitHub, Airbnb, Apple, WhatsApp, OpenAI and Slack, and supplied
HIG, Carbon, Base, Polaris and Primer as design-system references. These are
different products and tasks, not a palette to blend. For the small Fieldwork
materials library, the applicable references are predictable workspace navigation,
visible selection/result state and reversible preview/save actions.

The actual [Primer dialog example](https://primer.style/product/components/dialog/)
includes a trigger reference passed through `returnFocusRef`; that is a concrete
behavior to retain using the demo's native dialog. [Carbon's filtering
pattern](https://www.carbondesignsystem.com/building-blocks/core/patterns/filtering)
distinguishes instant from batch filtering and requires a way to clear applied
filters. Its instant update pattern fits this small local collection. These pages
were read on the research date; no component package was copied or installed.

[Current Polaris documentation](https://shopify.dev/docs/api/polaris) describes
web components and different Shopify application surfaces. It should be consulted
for a Shopify task, not treated as a mandate to add a Shopify runtime to an
unrelated app. Apple's motion page and Uber Base were only partially retrievable
through the text browser during this source pass; this report does not pretend to
have evaluated their full component implementations.

`examples/craft-lab/index.html` is an original, dependency-free demonstration with
editable SVG studies and fictional records. Its code and the workbench scenario
allow direct testing of desktop/mobile layout, filtering, saving and modal motion.
It is a reproducible review subject, not proof that any skill improves design.
Rendered evidence and independent review are recorded separately by the workbench.

## What this research does not establish

No benchmark showed that any reviewed prompt package consistently improves this
user's outputs. Test-source inspection proves that checks exist, not that they
passed here. No live Figma/Motion MCP, Impeccable binary or paid source was
evaluated. The subsequently implemented Fieldwork sample was exercised in an
installed Chromium browser at desktop and mobile sizes, including reduced motion,
and its captured layouts were inspected. That validates this sample's tested
behavior, not the quality claims of the upstream packages. A meaningful next
comparison is the same concrete screen,
same model, same input assets and time budget, evaluated by visible result and
behavior, with and without the selected adapted guidance.
