# Material 3 Expressive: responsive shape

The language label **responsive expressive geometry** is a Mirket interpretation. The reference is Material 3 Expressive motion and interaction-driven shape, rather than a general “playful” style.

## Brief → concept → grammar

**Brief.** A fictional reading-room control should feel responsive when its sound cushion switches on. The separate room measurement is a fixed 42 dB sample. A reader must distinguish a changed control state from changed measured data.

**Concept.** A soft object responds; an instrument tells the truth. The large purple sound object owns the spatial response. The cream reading surface, label and calibrated mark remain still.

**Grammar.** Keep the control identity and hit area invariant. Let its geometry settle toward the latest selected state with a short, bounded spring. Spatial movement may overshoot; color and opacity are effect changes and follow their destination directly. Do not assign a universal meaning to a decorative abstract shape.

## Located choices in the original study

| Location | Choice and purpose |
| --- | --- |
| `#sound-toggle`, `#room-status` | The pressed value and visible status commit at activation; no animation callback owns the logical state. The control’s hit target stays still. |
| `#shape`, `.sound-mark` | Border radii, scale and a small rotation express a soft control. A damped position/velocity spring retargets from its current state; color uses a separate finite linear transition. These are original tuning choices, not Material parameter values. |
| `#meter`, `#meter-marker` | The fixed sample and calibrated marker stay at 42 dB in the purposeful route. Control expression never changes an instrument’s apparent reading. |
| Bottom measurement rule | Space and the rule separate the reading from the expressive object; the task does not require a second animated focal point. |

**Fit:** tactile consumer controls, soft tools and selected moments of response. **Poor fit:** deforming quantitative marks, precision comparison, or a repetitive operational action where spring travel becomes tiring. The control is fictional and plays no actual audio.

## Contrast and failure probe

Open `?variant=misleading`. It has the same composition, typography and finish. The spring also moves the calibrated marker, so a toggle momentarily implies a changed room reading even though the data remains 42. This is a semantic error, not an intentionally ugly version.

Probe: activate sound and inspect the first 90–400 ms. `#meter[data-reading="42"]` must agree with `#meter-marker[data-visual-reading]` throughout the purposeful animation. A polished moving marker with a different reading fails. Then reverse on → off → on → off before settling: the final `aria-pressed`, visible status and `#shape[data-spatial]` must agree with off, without an old sequence replaying.

## Transfer task

Build an original drawing-mode control for a maker’s app. Let the selected mode’s silhouette respond, but keep a 12 px calibrated stroke-width preview unchanged. Change the palette, control geometry and composition. Accept the transfer only when a viewer can identify the active mode and read the actual stroke width during an interrupted transition. A recolored copy of Tide is insufficient evidence of transfer.

## Source, edition, observation and exclusions

[Material Motion: How it works](https://m3.material.io/styles/motion/overview/how-it-works), including the named expressive/standard switch and stiff/soft FAB demonstrations, describes the May 2025 physics update. [Shape principles](https://m3.material.io/styles/shape/overview-principles) relates morphing to interaction and state. Checked 2026-10-15. The prior research read official text/accessibility trees and sampled loaded motion players near rest; it did not establish their complete bounce trajectory. Shape videos were identified, not fully inspected. This implementation transfers the described relationship and verifies its own playback; it does not claim a faithful timing reenactment.

The guideline media are link-only; no blanket reuse license was established (`license_url: null`). Exclude Google marks, UI artwork, videos and source geometry. Material Web’s Apache code license is a separate question and grants no blanket guideline-media rights. This self-contained example uses original CSS/SVG/JS and system fonts, with no Material package or copied runtime.

## Read, render and build owner

- Study owner: `design-inspiration`. Build owner: `motion-design`, especially `references/interaction-recipes.md` → Controls and immediate feedback; use `advanced-motion` only if an actual brief expands into a coordinated scene.
- Card resource: `mirket://skills/design-inspiration/references/design-languages/material-responsive-shape.md`.
- CLI: `mirket catalog read design-inspiration --path references/design-languages/material-responsive-shape.md`.
- MCP: `skill_read` with `{"skill_id":"design-inspiration","path":"references/design-languages/material-responsive-shape.md"}`.
- Executable source resource: `mirket://skills/design-inspiration/assets/design-languages/material-responsive-shape/index.html`. Read it with the same owner and `path: "assets/design-languages/material-responsive-shape/index.html"`; reading returns text, not playback.
- Render the installed owner’s `assets/design-languages` directory through a local static server. In this checkout: `python3 -m http.server 8843 --bind 127.0.0.1 --directory skills/design-inspiration/assets/design-languages`, then visit `http://127.0.0.1:8843/material-responsive-shape/` and its `?variant=misleading` route.

The static page contains the complete off state and fixed reading; inactive controls stay hidden without JavaScript. Keyboard activation, visible focus and 320 px reflow are part of the example. Reduced motion at load or live change commits the exact final shape, cancels frames and removes color travel; state remains readable. The spring rests finitely. `pagehide` stops pending work. Actual browser checks belong to the integration evidence; this source card is not human visual acceptance or a performance benchmark.
