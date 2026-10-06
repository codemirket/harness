# Property editor hierarchy

Work only inside this scratch `workspace/`. This is a fictional local interface; it has no account, network, or production connection. Use the supplied static HTML, CSS, and JavaScript without adding dependencies.

Improve the editor so its content locale, current publication state, and main Save action form a clear hierarchy in light and dark themes. The present screen feels generic. Preserve the fictional Vale identity: a restrained navy wordmark, one amber accent, quiet neutral surfaces, and clear action blue. Keep the existing content and behavior: changing locale updates the editing context, editing a field marks the draft unsaved, Save stores the current local values, and the state remains Draft. Do not imply that Save publishes the property.

Open the page in a browser and inspect the actual result. Check desktop and narrow widths in both themes. Save four screenshots as `evidence/light-desktop.png`, `evidence/dark-desktop.png`, `evidence/light-narrow.png`, and `evidence/dark-narrow.png`. Write a short `evidence/review.md` describing the baseline weakness, changes made, one finding from rendering, and the resulting refinement. Check label associations, keyboard focus, readable state meaning, and narrow-screen overflow. Keep all work and evidence in this workspace.

You may run the local structural checker with `python3 -I ../verify.py .` from this workspace. Its passing result does not certify visual quality; a reviewer must inspect the browser renders.
