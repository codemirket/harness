# Route the task to a capability

Use the row that matches the requested outcome, then inspect the chosen entry
with `catalog show <id>`. Commands run through `scripts/catalog.py` relative to
the skill directory. IDs register catalog entries; invocation names identify the
installed skill. Profiles are reusable installation groups, not instructions to
load every member for every task. Existing project instructions and explicit
user choices remain authoritative.

| Requested outcome | Primary workflow and catalog additions | Selection boundary |
| --- | --- | --- |
| Improve an existing interface's hierarchy, composition or visual polish | Global `interface-design`; consider `taste-redesign-skill` (invocation `redesign-existing-projects`) for a requested redesign. The `design-redesign` profile also supplies content, mobile and motion checks. Search `redesign` or `ui-design`. | Preserve the actual brand, tokens, content and behavior. `frontend-web`, `break-ui` and accessibility checks alone do not provide art direction. Read the redesign adaptation before using aesthetic prescriptions. |
| Explore alternative UI directions | Global `interface-design`; `emil-prototype` (invocation `prototype`) when the user requests exploration; `open-design-reference-design-contract` (invocation `reference-design-contract`) for an explicit durable visual handoff. | `prototype` preserves explicit invocation semantics. Do not trigger a multi-variant picker on every UI fix or create competing design documents. The `design-prototyping` profile contains several separate recipes; select those needed. |
| Fix UI overflow, extreme content or mobile platform behavior | `emil-break-ui` (invocation `break-ui`) for realistic content stress; `emil-mobile-native` (invocation `mobile-native`) for viewport, touch and safe-area problems. | These diagnose concrete failures. They do not establish visual quality. Use existing fixtures/review tools; diagnostic controls stay outside production. |
| Implement or debug application behavior | Global `engineering-judgment`; authored `debugging`, `test-design` and `architecture-review` as needed. Add `frontend-engineering` for browser rendering, forms and data flow; choose a matching backend/platform profile from actual dependencies. | Reuse the project's selected implementation workflow. React/Next, Vue/Nuxt and SvelteKit guidance are different branches; do not select React recipes for a Nuxt application. A mechanical edit needs no specialist registration. |
| Write repository documentation, API docs or an ADR | `addy-documentation-and-adrs` (invocation `documentation-and-adrs`); global `document-workflow` for substantial revisions that need source/content comparison. | Follow existing owners, templates and examples. Office-authoring and PDF skills do not substitute for technical documentation. Do not create an ADR for a routine edit. |
| Create or revise DOCX, spreadsheet, slide or PDF artifacts | The available native format skill, supported by `document-workflow`, `office-authoring` and `document-parsing` when relevant. The `documents` profile provides portable authoring/extraction and `openai-pdf` (invocation `pdf`). | Check the actual editor, renderer and calculation engine. A file skill does not control a live Excel session. Save, render and inspect the requested deliverable. |
| Implement or review interface animation | `motion-design`; add `vercel-react-view-transitions` only for a matching React View Transition task and supported installed versions. `emil-animation-vocabulary` (invocation `animation-vocabulary`) is for naming an effect. Search `motion` or `animation`. | General animation does not imply React View Transitions, Remotion video or a new library. Exercise interruption, exit/cleanup and reduced motion; inspect actual frames. Native/mobile stacks need their own platform skill. |
| Research facts or uncertain framework behavior | Global `research-and-synthesis`; optionally `addy-source-driven-development` (invocation `source-driven-development`) for installed-version technical research. `google-retrieving-developer-knowledge` is specifically for official Google platforms when its tools are available. | Use current sources appropriate to the claim. Search research is distinct from SEO and local code lookup. Use `rg` and the actual repository for local code search. |
| Audit organic or AI search discoverability | `search-visibility` and `search-audit`, or the `search-visibility` profile. | Use observed crawl, indexing, content and citation evidence. Do not route ordinary web research or a private app's internal search bug to SEO/GEO. |

For a substantial task, state the selected workflow and its purpose briefly.
Read its installed `SKILL.md`, applicable integration note and relevant references;
the catalog description alone does not apply the workflow. If a needed reviewed
entry is absent, register through `project add`, then sync and doctor, within
existing authorization. Do not edit installed copies or hand-edit the lock.

Never follow an upstream sibling name blindly. Confirm it is installed and
reviewed; otherwise use the applicable route above or discover a suitable entry.
`manual`, `indexed-only` and advertisement entries are review candidates, not
active capabilities. In particular the fuller Emil, Impeccable, UI UX Pro Max
and Remotion integrations have separate review/runtime requirements. Report a
material missing capability instead of treating a catalog listing as installation.

Registration, selection, application and outcome are separate evidence levels.
Doctor verifies registration. A fresh-session skill read demonstrates selection.
An observed task result demonstrates application; UI improvement additionally
needs comparable rendered evidence and judgment against the user's visual goal.
