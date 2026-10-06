> Historical first-pass review. The broad selection policy and final catalog are in [source-review.md](../source-review.md); minimal-selection recommendations below are superseded.

# Design source review

Review date: 2026-10-06. Source snapshots are pinned below. No upstream installer,
program, or test suite was executed, and no installed skills were changed.
Recommendations are from source inspection, not popularity or claims of benchmark
wins. A skill being permitted by a license does not prove that it improves output.

## Decision

Use a small authored global `interface-design` skill, available only when the task
involves interfaces. None of these four entire upstream packages belongs in the
global registry unchanged. Their triggers overlap substantially, while their
visual rules conflict. A global catalog selector should choose a single applicable
workflow rather than load several anti-generic-design packages together.

The strongest first project opt-ins are Emil's `mobile-native`, `break-ui`, and
`ask-sonner` (the latter only for a Sonner project). `prototype` is a useful second
wave for explicitly requested design exploration. All are MIT, self-contained
Markdown directories with the companion files specified below. This is a static
compatibility judgment, not proof that every technical statement is correct.

The [global interface-design skill](../../skills/interface-design/SKILL.md) and
[project motion-design skill](../../skills/motion-design/SKILL.md) retain
useful techniques without blanket process, aesthetic, API, or library mandates.

## Snapshot and coverage

| Repository | Commit | Tracked files / bytes | Unique skills / SKILL words |
|---|---|---|---|
| nextlevelbuilder/ui-ux-pro-max-skill | `477bcb28c9812b385cb51a4605ddf30d7b2266e2` | 682 / 21,992,419 | 7 / 7,951 |
| Nutlope/hallmark | `13ac0ec7e148655948100b6396439e481361d690` | 286 / 17,787,446 | 1 / 9,804 |
| Leonxlnx/taste-skill | `ce26fc25c0e5e8cab638f883de62d9a86ee5e45b` | 66 / 4,875,890 | 13 / 45,619 |
| emilkowalski/skills | `e8a175de22ae1e49370fc144c1f3bb9aeedf988d` | 28 / 285,193 | 14 / 32,456 |

Counts include tracked demo/media files; SKILL words exclude duplicated UI Pro Max
CLI copies. Inventoried every SKILL entry, its trigger, requirements, dependencies,
and restrictive instructions. Read the full main UI Pro Max bundle instructions,
Hallmark entry, and recommended Emil workflows plus companion CATALOG. Read the
main taste workflow and its smaller presets; large image-direction and specialized
Swift/Expo/Apple material received targeted instruction/API/risk review. Linked
references and scripts were selectively inspected for packaging, dependencies,
side effects, and conflicts; not every visual recipe, external documentation
page, or data row was individually validated. Do not relabel this as an exhaustive
runtime or security audit.

## nextlevelbuilder/ui-ux-pro-max-skill

**Recommendation:** retain the core local search tool as a project opt-in after
adapting path resolution and its broad trigger; do not install the entire plugin.
The plugin's manifest registers the whole `.claude/skills/` directory, including
six ancillary design skills. Exact inventory is in the appendix.

**Useful substance.** The core entry selects the smallest relevant search mode,
detects the installed stack, distinguishes a search match from general fallback,
limits retry to one narrower query, and treats returned guidance as subordinate
to user/project instructions. A new design direction uses a product/style/color/
typography search; a small component defect can use one domain. This is much more
compatible with economical operation than blindly generating a design system on
every UI change. Source: [core workflow](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/477bcb28c9812b385cb51a4605ddf30d7b2266e2/.claude/skills/ui-ux-pro-max/SKILL.md#L49).

**Execution boundary.** Core search uses Python standard-library CSV/BM25 code,
with data resolved relative to the script. No external service is required for
this search. Persistence is opt-in, slugifies project/page names, and normally
avoids overwriting prior output; it writes a temporary file and atomically
publishes it. `--force` changes that behavior. Source: [search entry](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/477bcb28c9812b385cb51a4605ddf30d7b2266e2/.claude/skills/ui-ux-pro-max/scripts/search.py),
[persistence](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/477bcb28c9812b385cb51a4605ddf30d7b2266e2/.claude/skills/ui-ux-pro-max/scripts/design_system.py#L988).

**Portability.** Python 3 exists for macOS/Windows; instructions offer `python`,
`python3`, and `py -3`. The displayed commands nonetheless rely on
`${CLAUDE_PLUGIN_ROOT}/.claude/skills/...`; a copied Codex/project directory cannot
assume that environment variable or plugin layout. Use discovered skill location
and include the complete scripts/data/references subtree. Python Windows encoding
is explicitly handled. No runtime test was run on either OS.

**Do not import all extras.** `design` reads API keys and can send prompts to
Gemini, Atlas Cloud, or MuAPI; setup includes `google-genai` and Pillow. It routes
to sibling brand/token/style skills and prescribed questions. `design-system`
combines useful primitive/semantic/component token architecture with mandatory
Chart.js, center-aligned, persuasion-oriented HTML slide behavior. `slides`
overlaps the installed presentation skill. `banner-design` offers a useful
editable-copy plus exact-size-export flow, but includes a dated Meta text-ratio
claim and instructions forbidding any file-path disclosure, conflicting with its
own deliverable format and this user's practical reporting preferences. Source:
[design](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/477bcb28c9812b385cb51a4605ddf30d7b2266e2/.claude/skills/design/SKILL.md),
[logo API code](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/477bcb28c9812b385cb51a4605ddf30d7b2266e2/.claude/skills/design/scripts/logo/generate.py#L39),
[banner rules](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/477bcb28c9812b385cb51a4605ddf30d7b2266e2/.claude/skills/banner-design/SKILL.md).

**License.** Root [LICENSE is MIT](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/477bcb28c9812b385cb51a4605ddf30d7b2266e2/LICENSE), copyright Next Level Builder.
The `ui-styling` directory has an [Apache-2.0 LICENSE.txt](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/477bcb28c9812b385cb51a4605ddf30d7b2266e2/.claude/skills/ui-styling/LICENSE.txt)
despite MIT frontmatter: record this inconsistency and defer copying that subtree
until provenance is resolved. Font metadata includes a separate
`google-font-licenses.json`; the repo's MIT license is not a license for every font
or third-party image/mark an agent may download. Preserve applicable notices when
vendoring MIT resources.

## Nutlope/hallmark

**Recommendation:** adapt selected ideas, exclude unchanged automatic install.
One skill, `skills/hallmark/SKILL.md`, with an extensive reference library.
The package itself is Markdown, without an execution dependency; the npm
`serve` command is for its demonstration site. “Powered by Together AI” is present
in branding; this review did not find a requirement to call that provider in the
main skill workflow. [Manifest](https://github.com/Nutlope/hallmark/blob/13ac0ec7e148655948100b6396439e481361d690/package.json).

**Useful substance.** Existing-design preflight, distinguishing component scope
from page scope, respecting existing production structure, honest metrics and
endorsements, semantic tokens, observing responsive output, and documenting
reference-derived properties separately from guesses. The study workflow treats
remote HTML/CSS as inert, calls out screenshot font-identification limitations,
and distinguishes extraction from reproduction. These ideas suit an original
interface skill without the large machinery.

**Main conflicts.** [Lines 203 onward](https://github.com/Nutlope/hallmark/blob/13ac0ec7e148655948100b6396439e481361d690/skills/hallmark/SKILL.md#L203)
explicitly require asking audience/use/tone even when a detailed brief already
contains them. That directly conflicts with the user's request for useful
judgment without unnecessary friction. It mandates all eight states even for
components that do not have those states, 58 pass/fail design gates and critique
scores, a `tokens.css` artifact on every build, `.hallmark` persistence, and
rotation of layout/theme/nav/footer across outputs. These are artistic/workflow
preferences, not demonstrated universal quality controls. The preflight cache
only watches package/Tailwind mtimes even though it also relies on design.md,
HTML, CSS, and token sources; that can reuse stale design evidence.

**Internal inconsistency and cost.** Theme rotation first says one axis must
differ, then rejects an example that differs on one axis. The before-build preview
requires a score from the after-build test. Typography-only is encouraged, unlike
taste's mandatory images. The blanket ban on italic headings conflicts with
other packages' recommendations. Several useful rules are swamped by prescribed
questions, stamps, outputs, and long catalogs. The global stylesheet is called
append-only; blindly appending conflicting CSS is not a substitute for the
smallest correct edit. [Full entry](https://github.com/Nutlope/hallmark/blob/13ac0ec7e148655948100b6396439e481361d690/skills/hallmark/SKILL.md),
[contract](https://github.com/Nutlope/hallmark/blob/13ac0ec7e148655948100b6396439e481361d690/skills/hallmark/references/contract.md).

**Packaging.** SKILL references `../../site/css/tokens.css` for named-theme data,
but npm `files` includes only `skills`. A single skill-folder install cannot
resolve that sibling repository-site path. This is another reason to avoid an
unmodified registry copy. `WebFetch` is a named Claude capability; a Codex adapter
must map to an available browser/HTTP tool and report the resulting evidence limits.

**License:** [MIT, Hallmark contributors](https://github.com/Nutlope/hallmark/blob/13ac0ec7e148655948100b6396439e481361d690/LICENSE).

## Leonxlnx/taste-skill

**Recommendation:** reference/adapt, not a global workflow. Retain only explicit
style presets as optional catalog inspiration if the user wants those aesthetics;
no need to install the whole 13-skill plugin. [License is MIT](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/LICENSE), copyright Leonxlnx.

**Current v2 merits.** `design-taste-frontend` now starts with the brief/audience,
asks only when there is a meaningful divergence, treats access needs as higher
priority than aesthetics, distinguishes official design systems from visual
trends, audits before redesign, and preserves functionality/brand. It corrects
fake-precise numbers by requiring actual or labeled example data. This is better
than its legacy preset's instruction to substitute organic-looking fake numbers.
Its narrow stated scope is landing pages, portfolios, and redesigns.

**Still not general guidance.** The current entry is 1,206 lines/87 KB; despite
its contextual preamble it contains mandatory generated imagery, even on minimal
sites; default React/Next/Tailwind/Motion; mandatory two color modes on consumer
pages; blanket icon/font/color/layout/punctuation bans; and up-front library
installation direction. These can spend time or disrupt a brand without improving
the user's requested outcome. Source: [current entry](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/taste-skill/SKILL.md#L262).

**Exclude unequivocally.** `gpt-taste` tells the model to simulate Python random
selection and emit mock execution output, forces GSAP, and forbids static
interfaces. This is not trustworthy engineering evidence. `full-output-enforcement`
treats every task as production-critical, bans legitimate concise examples/TODOs,
and demands continuation rather than selecting efficient file edits. v1 mandates
perpetual motion. `high-end-visual-design` forces nested bezels/pills/slow reveals
and even demonstrates `transition-all`. [gpt-taste](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/gpt-tasteskill/SKILL.md#L14),
[output enforcement](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/output-skill/SKILL.md).

**Image workflows.** `image-to-code` requires generating the design before any
visual code task, one image per section, and fresh generation rather than cropping.
`imagegen-frontend-web` always makes separate images and defaults an unspecified
landing page to six images. `imagegen-frontend-mobile` and `brandkit` are concept
image briefs, not editable app/identity implementations. Their screen consistency,
readability, and meaningful-logo prompts are useful references; mandatory image
counts and aesthetic rules are poor defaults when installed image tooling exists.
[Image-to-code](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/image-to-code-skill/SKILL.md#L112),
[image web](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/imagegen-frontend-web/SKILL.md#L6).

**Portability.** Markdown bodies are readable by either agent on either OS.
Execution still depends on installed web stack, image generation, browser, or
Stitch capability. `stitch-design-taste` specifically requires Google Stitch
access. Do not run the repository's convenience shell installer for selection;
copy only a reviewed pinned skill with notices if one is explicitly adopted.

## emilkowalski/skills

**Recommendation:** best source here for narrow actionable opt-ins, with a small
original global synthesis instead of installing the broad design-engineering
skill. [All reviewed directories fall under root MIT](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/LICENSE), copyright Emil Kowalski; no per-directory license override was found.

**First three exact install units:**

- `skills/mobile-native/`: `SKILL.md`. Its symptom table and conditional fixes
  target real mobile-web failures; distinguishes stable hero height from dynamic
  app-shell height, preserves zoom, scopes selection suppression to controls,
  and explicitly reports what requires real hardware. Some blanket statements
  about browser emulation/API behavior still require checking against the actual
  support matrix. It is not proof of performance. [Source](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/mobile-native/SKILL.md).
- `skills/break-ui/`: `SKILL.md` plus `CATALOG.md`. Traces data to schema limits,
  exercises plausible extremes through actual component inputs, and separates
  visual observations from inference when a browser is unavailable. Dev-only
  fixtures are reusable regression evidence. Default is report-before-fix; invoke
  `<component> + fix` when the task already includes fixes. [Source](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/break-ui/SKILL.md),
  [catalog](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/break-ui/CATALOG.md).
- `skills/ask-sonner/`: `SKILL.md` plus `API.md`. Useful for projects that actually
  use Sonner, with setup/state/style diagnosis from its author. Check API names
  against the installed version. Skill text has no external execution script.
  [Source](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/ask-sonner/SKILL.md).

**Secondary.** `prototype` with `PICKER.md` creates functioning alternatives in an
isolated preview and promotes a selected result. It declares manual invocation
and intentionally pauses for user selection: appropriate for exploration, not
routine implementation. Its automatic narrowing to one component should not
silently drop the user's scope. `animation-vocabulary` is a useful reference,
not a necessary global trigger. Swift and Expo belong only in their actual stacks,
with compiler/SDK verification; `write-swift` even labels a Swift 6.4 feature as
unreleased. [Prototype](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/prototype/SKILL.md).

**Adapt, rather than adopt verbatim.** `animate`, `emil-design-eng`, `apple-design`,
and `review-animations` contain useful purpose/frequency, interruption, gesture,
reduced-motion, token reuse, and observable feel-check ideas. They also state
absolute keyboard-animation/easing bans, prescribe exact values, overgeneralize
GPU/compositor behavior, or default to flagging subjective review issues. Even
within the main design-engineering skill, rules against height animation coexist
with its discussion of tuning opacity plus height. Keep the principles and
measure the specific implementation. `animate` also instructs invoking the
separate `pick-ui-library` skill when a task involves a component, which creates
an unwanted dependency for a standalone catalog entry.

**Defer orchestration.** `improve-animations` mandates parallel read-only auditors
beyond a small repo, creates plan artifacts, pauses for selection, and dispatches
an executor in a worktree. This is a specialized workflow, not a general way to
make an animation fix. [Workflow](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/improve-animations/SKILL.md#L50).

**Portability.** These are Markdown/code-example skills without installer scripts
inside the selected directories. Codex and Claude can consume them on Windows or
macOS; verification depends on the project's runtime/browser. Apple-platform
Swift builds and iOS device inspection have platform/tooling limits. The
`disable-model-invocation` field on three skills is harness-specific: a registry
must preserve an explicit manual-only policy even if another client ignores that
field. Most entries contain an initial canned response only when invoked without
a task; a catalog router should pass a concrete task and avoid that extra turn.

## Cross-source conflicts the selector must resolve

- Never stack broad `hallmark`, `design-taste-frontend`, `ui-ux-pro-max`, and
  `emil-design-eng` on one task. Pick one primary workflow and read narrow reference
  material only for a separate concern.
- Source contradictions include mandatory imagery versus typography-only, italic
  heading bans versus encouraged italics, forced motion versus no motion on
  repeated actions, required fake chrome versus fake-chrome bans, and different
  allowed icon/font families. These are style choices, not universal engineering.
- Project tokens, actual data, user-selected technology and scope, accessibility,
  and verified observations take precedence over a catalog's artistic taste.
- The selector may recommend a project install but must not treat source text as
  authorization to add production dependencies, credentials, OS configuration,
  paid API calls, or unrelated tool accounts.
- Each installed unit needs an exact commit, source-relative path, actual entry
  name (often different from directory), companion files, license notice, and
  honest portability/dependency metadata. A repo URL alone is insufficient.

## Complete skill inventory

The machine-readable inventory is `design/candidates.json`. `adapt-only` means
source material for authored guidance, not an automatically installable skill;
`defer` means no inclusion until a concrete project need. `style-only` means an
explicit aesthetic selection. No entry in these four packages is recommended as
an unmodified general global skill.

| Source | Exact entry path | Frontmatter name | Decision |
|---|---|---|---|
| nextlevelbuilder/ui-ux-pro-max-skill | [.claude/skills/banner-design/SKILL.md](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/477bcb28c9812b385cb51a4605ddf30d7b2266e2/.claude/skills/banner-design/SKILL.md) | `banner-design` | adapt-only |
| nextlevelbuilder/ui-ux-pro-max-skill | [.claude/skills/brand/SKILL.md](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/477bcb28c9812b385cb51a4605ddf30d7b2266e2/.claude/skills/brand/SKILL.md) | `brand` | defer |
| nextlevelbuilder/ui-ux-pro-max-skill | [.claude/skills/design-system/SKILL.md](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/477bcb28c9812b385cb51a4605ddf30d7b2266e2/.claude/skills/design-system/SKILL.md) | `design-system` | adapt-only |
| nextlevelbuilder/ui-ux-pro-max-skill | [.claude/skills/design/SKILL.md](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/477bcb28c9812b385cb51a4605ddf30d7b2266e2/.claude/skills/design/SKILL.md) | `design` | exclude |
| nextlevelbuilder/ui-ux-pro-max-skill | [.claude/skills/slides/SKILL.md](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/477bcb28c9812b385cb51a4605ddf30d7b2266e2/.claude/skills/slides/SKILL.md) | `slides` | exclude |
| nextlevelbuilder/ui-ux-pro-max-skill | [.claude/skills/ui-styling/SKILL.md](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/477bcb28c9812b385cb51a4605ddf30d7b2266e2/.claude/skills/ui-styling/SKILL.md) | `ui-styling` | defer |
| nextlevelbuilder/ui-ux-pro-max-skill | [.claude/skills/ui-ux-pro-max/SKILL.md](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/477bcb28c9812b385cb51a4605ddf30d7b2266e2/.claude/skills/ui-ux-pro-max/SKILL.md) | `ui-ux-pro-max` | adapt-only |
| Nutlope/hallmark | [skills/hallmark/SKILL.md](https://github.com/Nutlope/hallmark/blob/13ac0ec7e148655948100b6396439e481361d690/skills/hallmark/SKILL.md) | `hallmark` | adapt-only |
| Leonxlnx/taste-skill | [skills/brandkit/SKILL.md](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/brandkit/SKILL.md) | `brandkit` | project-secondary |
| Leonxlnx/taste-skill | [skills/brutalist-skill/SKILL.md](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/brutalist-skill/SKILL.md) | `industrial-brutalist-ui` | style-only |
| Leonxlnx/taste-skill | [skills/gpt-tasteskill/SKILL.md](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/gpt-tasteskill/SKILL.md) | `gpt-taste` | exclude |
| Leonxlnx/taste-skill | [skills/image-to-code-skill/SKILL.md](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/image-to-code-skill/SKILL.md) | `image-to-code` | exclude |
| Leonxlnx/taste-skill | [skills/imagegen-frontend-mobile/SKILL.md](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/imagegen-frontend-mobile/SKILL.md) | `imagegen-frontend-mobile` | adapt-only |
| Leonxlnx/taste-skill | [skills/imagegen-frontend-web/SKILL.md](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/imagegen-frontend-web/SKILL.md) | `imagegen-frontend-web` | exclude |
| Leonxlnx/taste-skill | [skills/minimalist-skill/SKILL.md](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/minimalist-skill/SKILL.md) | `minimalist-ui` | style-only |
| Leonxlnx/taste-skill | [skills/output-skill/SKILL.md](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/output-skill/SKILL.md) | `full-output-enforcement` | exclude |
| Leonxlnx/taste-skill | [skills/redesign-skill/SKILL.md](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/redesign-skill/SKILL.md) | `redesign-existing-projects` | adapt-only |
| Leonxlnx/taste-skill | [skills/soft-skill/SKILL.md](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/soft-skill/SKILL.md) | `high-end-visual-design` | exclude |
| Leonxlnx/taste-skill | [skills/stitch-skill/SKILL.md](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/stitch-skill/SKILL.md) | `stitch-design-taste` | exclude |
| Leonxlnx/taste-skill | [skills/taste-skill-v1/SKILL.md](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/taste-skill-v1/SKILL.md) | `design-taste-frontend-v1` | exclude |
| Leonxlnx/taste-skill | [skills/taste-skill/SKILL.md](https://github.com/Leonxlnx/taste-skill/blob/ce26fc25c0e5e8cab638f883de62d9a86ee5e45b/skills/taste-skill/SKILL.md) | `design-taste-frontend` | adapt-only |
| emilkowalski/skills | [skills/animate-expo/SKILL.md](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/animate-expo/SKILL.md) | `animate-expo` | defer |
| emilkowalski/skills | [skills/animate/SKILL.md](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/animate/SKILL.md) | `animate` | adapt-only |
| emilkowalski/skills | [skills/animation-vocabulary/SKILL.md](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/animation-vocabulary/SKILL.md) | `animation-vocabulary` | project-secondary |
| emilkowalski/skills | [skills/apple-design/SKILL.md](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/apple-design/SKILL.md) | `apple-design` | adapt-only |
| emilkowalski/skills | [skills/ask-sonner/SKILL.md](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/ask-sonner/SKILL.md) | `ask-sonner` | project |
| emilkowalski/skills | [skills/break-ui/SKILL.md](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/break-ui/SKILL.md) | `break-ui` | project |
| emilkowalski/skills | [skills/emil-design-eng/SKILL.md](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/emil-design-eng/SKILL.md) | `emil-design-eng` | adapt-only |
| emilkowalski/skills | [skills/find-animation-opportunities/SKILL.md](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/find-animation-opportunities/SKILL.md) | `find-animation-opportunities` | defer |
| emilkowalski/skills | [skills/improve-animations/SKILL.md](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/improve-animations/SKILL.md) | `improve-animations` | exclude |
| emilkowalski/skills | [skills/mobile-native/SKILL.md](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/mobile-native/SKILL.md) | `mobile-native` | project |
| emilkowalski/skills | [skills/pick-ui-library/SKILL.md](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/pick-ui-library/SKILL.md) | `pick-ui-library` | defer |
| emilkowalski/skills | [skills/prototype/SKILL.md](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/prototype/SKILL.md) | `prototype` | project-secondary |
| emilkowalski/skills | [skills/review-animations/SKILL.md](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/review-animations/SKILL.md) | `review-animations` | adapt-only |
| emilkowalski/skills | [skills/write-swift/SKILL.md](https://github.com/emilkowalski/skills/blob/e8a175de22ae1e49370fc144c1f3bb9aeedf988d/skills/write-swift/SKILL.md) | `write-swift` | defer |
