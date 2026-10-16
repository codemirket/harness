# Apple Liquid Glass: content and control layers

**Adaptive native control layer** is a Mirket interpretation of the WWDC25 presentation. Liquid Glass is an Apple platform language; this original web study teaches one relationship and does not reproduce its optical renderer.

## Brief → concept → grammar

**Brief.** A fictional estuary journal needs optional view controls. Opening them should keep the landscape and article available as stable reading content.

**Concept.** A small control island expands above a calm reading surface. The reader keeps their place while the tools acquire room.

**Grammar.** Treat material, motion and hierarchy together. Navigation/controls can occupy a distinct functional layer over content; making the content itself another transient material confuses its role. Preserve legibility and the content/control distinction rather than applying a generic “glass” finish to every surface.

## Located choices in the original study

| Location | Choice and purpose |
| --- | --- |
| `.landscape`, `#article[data-layer="content"]` | An original SVG estuary and a stable text column form the content layer. Opening tools leaves their position unchanged in the purposeful study. |
| `#control-dock`, `#view-tools`, `#toolset` | A local tool group expands from its own anchor. Its border, near-opaque backing and shadow preserve a clear surface role over the reading context. CSS blur is progressive decoration, with a solid backing retained. |
| `#labels`, `#save`, `#view-status` | The actual view state changes at activation and is described in text. No simulated optical effect owns the selected value. |
| Escape and Close controls | These dismiss only the transient controls. Focus returns to the trigger when it came from the hidden group; the article never becomes a modal or dismissible object. |

**Fit:** reasoning about controls above content in an actual Apple-platform app, followed by the platform’s native components and accessibility behavior. **Poor fit:** generic glass decoration, stacked glass content cards or a web renderer being presented as native Liquid Glass. The fictional journal is a teaching composition, not an Apple UI clone.

## Contrast and failure probe

`?variant=misleading` keeps the same landscape and type. Opening tools also lifts the article into a polished “Floating preview.” The body copy remains readable, but the material/movement falsely promotes reading content into a temporary surface. The problem is the layer relationship, not an accessibility stunt.

Probe: record `#article`’s position, open View controls, and inspect the result. Purposeful behavior keeps `data-layer="content"` and unchanged article bounds. The counterexample changes it to `transient` and moves the reading into another floating panel. Close → reopen quickly must leave the latest tool state visible; Escape from inside the tools must close only that group and restore trigger focus. Save and trail-label state remain true through dismissal.

## Transfer task

Design a native reading app’s text-size and bookmark controls. Keep the article’s stable content identity while the tools expand from their own anchor, then test actual platform legibility over light and dark content and the platform’s accessibility settings. For a web product, translate the hierarchy with appropriate web surfaces and describe its limits. Do not transfer the fictional landscape, composition or CSS blur as a substitute for native material behavior.

## Source, edition, observation and exclusions

[WWDC25 Meet Liquid Glass, session 219](https://developer.apple.com/videos/play/wwdc2025/219/) (2025), especially Dynamics, Adaptivity and Principles, is the primary source. Checked 2026-10-15. Its transcript describes a distinct control/navigation layer, contextual legibility and avoiding glass in the content layer. The prior research read that transcript. During implementation, the official video was played at sampled positions around 1:45, 11:15 and 13:35: visible material/rounded-layer studies were inspected. These samples do not establish full choreography or real native behavior. The [HIG Materials link](https://developer.apple.com/design/human-interface-guidelines/materials) is retained as a next reading route; the prior text reader exposed only its JavaScript shell, so its full body is not claimed as read.

The presentation, Apple UI media, Apple marks, SF Symbols, fonts and native rendering code remain excluded and link-only (`license_url: null`). No redistribution license for those materials was established. Lagoon uses original HTML/CSS/JS/SVG and system fonts. It implements neither native lensing nor automatic native contrast adaptation. Its solid-backed web controls are an original approximation of the structural relationship only.

## Read, render and build owner

- Study owner: `design-inspiration`. Build owner: `motion-design` for this web control group; an actual native app must use its platform owner and native materials rather than this HTML as an implementation specification.
- Card resource: `mirket://skills/design-inspiration/references/design-languages/apple-liquid-glass.md`.
- CLI: `mirket catalog read design-inspiration --path references/design-languages/apple-liquid-glass.md`.
- MCP: `skill_read` with `{"skill_id":"design-inspiration","path":"references/design-languages/apple-liquid-glass.md"}`.
- Executable source: `mirket://skills/design-inspiration/assets/design-languages/apple-liquid-glass/index.html`; retrieve using the same owner and `path: "assets/design-languages/apple-liquid-glass/index.html"`. A resource read returns source, not native rendering.
- Render the installed owner’s `assets/design-languages` through a local static server. Checkout command: `python3 -m http.server 8843 --bind 127.0.0.1 --directory skills/design-inspiration/assets/design-languages`. Visit `http://127.0.0.1:8843/apple-liquid-glass/` and `?variant=misleading`.

The static journal and landscape remain available without JavaScript; inactive tools stay hidden. Controls support keyboard activation, visible focus, narrow reflow and Escape dismissal. Finite resizing is canceled/replaced on rapid input. Reduced motion at load and during use removes spatial effects immediately while keeping current state. Reduced-transparency CSS supplies an opaque control backing where supported; it is not proof of native accessibility-setting parity. Integration evidence must distinguish Chromium checks from Apple-platform validation.
