---
name: design-systems
description: "Create or evolve repeated UI tokens and components when a coherent reusable design language is required. Skip a one-off style change that needs no shared system."
---

# Design systems

Inspect real repeated patterns, current component APIs and any existing token ownership. Separate semantic roles from raw values so themes, states and modes remain understandable. Define only the tokens and variants the product actually needs; avoid a second parallel source of truth.

Use the qualified Mirket Style Dictionary route to compile token sources. Reference generated outputs in authored UI or asset source and verify that changing a source token changes a real consumer. Check names, units, aliases, theme overrides and failure for a missing reference. A generated file that no component consumes does not establish a working system.

Define component anatomy, variants, interaction states and accessibility responsibilities. Exercise representative composition and content stress cases rather than demonstrating only isolated pristine controls. Preserve API compatibility or document the migration for existing consumers.

Separate governing intent from patterns merely observed in the current UI. Identify which token or component source actually reaches the rendered consumer before diagnosing drift. Theme, density and state differences need explicit ownership; avoid copying computed values back into a second handwritten token set. For a migration, identify affected consumers and verify an old/new representative use so renamed aliases or changed defaults cannot silently alter meaning.

Test the system as composition: a form with validation, a dense table with actions or a navigation/content boundary can expose inconsistencies hidden by isolated swatches. Include long labels, disabled/selected states, focus, dark surfaces and relevant locale expansion. Keep visual language flexible enough for the real content; adding a token for every one-off value is not evidence of reuse.

Deliver editable token/component sources, compiled outputs, a consuming specimen and checks for states/themes. Unless the task sets another budget, share 30 minutes and three final repairs across system production and interface review, with one target repair. Stop when the agreed repeated language works through real consumers or report unsupported transforms, missing runtime capability and remaining defects at the limit. Do not install a framework, font or upstream branded component library from a design reference.
